// Reload the same real WebView without Vue unmount or JS disposal; native lifecycle must own cleanup.
import { invoke } from "@tauri-apps/api/core";
import { Image } from "@tauri-apps/api/image";
import { EngineClient } from "../src/engine/client";
import v1 from "./fixtures/vm-trigger/package.npack.json";

interface ReloadState {
  ids: number[];
  imageId?: number;
  checks: Array<{ document: number; oldResourceRefused: boolean; unrelatedImageRetained: boolean; pendingRequests: number }>;
}

async function run() {
  const state: ReloadState = await (await fetch("/state")).json();
  const documentNumber = state.ids.length;
  history.replaceState(null, "", `/?engineTransport=native&document=${documentNumber}`);
  if (state.imageId === undefined) state.imageId = (await Image.new([11, 22, 33, 255], 1, 1)).rid;
  const size = await new Image(state.imageId).size();
  const unrelatedImageRetained = size.width === 1 && size.height === 1;
  if (!unrelatedImageRetained) throw new Error("unrelated image resource lost during navigation");
  let oldResourceRefused = documentNumber === 0;
  if (documentNumber > 0) {
    try {
      await invoke("native_engine_request", { resourceId: state.ids.at(-1), request: { id: 99, type: "status" } });
    } catch { oldResourceRefused = true; }
  }
  const validator = new EngineClient();
  await validator.connect();
  await validator.validatePackage(JSON.stringify(v1));
  // SAFETY: this test reads the allocated resource promise from the production client's IPC transport without replacing either implementation.
  const resourceId = await (validator as unknown as { worker: { resource: Promise<number> } }).worker.resource;
  if (!Number.isInteger(resourceId)) throw new Error("validator did not allocate an actual native resource");
  const spareResources: number[] = [];
  try {
    for (let slot = 0; slot < 7; slot++) spareResources.push(await invoke<number>("native_engine_open"));
  } finally {
    for (const resourceId of spareResources) await invoke("native_engine_close", { resourceId });
  }
  state.ids.push(resourceId);
  // Send actual invoke requests before replacing the document. Their old renderer must not own a live host afterward.
  const pendingRequests = documentNumber < 12 ? 8 : 0;
  state.checks.push({ document: documentNumber, oldResourceRefused, unrelatedImageRetained, pendingRequests });
  await fetch("/state", { method: "POST", body: JSON.stringify(state) });
  if (documentNumber < 12) {
    let settled = 0;
    for (let id = 1; id <= pendingRequests; id++) {
      void invoke("native_engine_request", { resourceId, request: { id: 100 + id, type: "autoStep" } })
        .then(() => { settled++; }, () => { settled++; });
    }
    for (let allocation = 0; allocation < 2; allocation++) {
      void invoke("native_engine_open").then(() => { settled++; }, () => { settled++; });
    }
    navigator.sendBeacon("/pending", JSON.stringify({ document: documentNumber, submitted: pendingRequests + 2, pendingRequests, pendingOpens: 2, settledAtNavigation: settled }));
    location.reload();
    return;
  }
  if (!state.checks.every((check) => check.oldResourceRefused)) throw new Error("old document resource survived navigation");
  const nextClient = new EngineClient();
  await nextClient.connect();
  const status = await nextClient.status();
  if (status.revision !== 0) throw new Error("new client after repeated reloads did not have an independent session");
  nextClient.dispose();
  validator.dispose();
  await new Image(state.imageId).close();
  const report = { passed: true, reloads: 12, documents: state.checks.length, nextClientConnected: status.revision === 0, checks: state.checks };
  await fetch("/report", { method: "POST", body: JSON.stringify(report) });
}

try { await run(); }
catch (error) {
  await fetch("/report", { method: "POST", body: JSON.stringify({ passed: false, error: String(error), stack: error instanceof Error ? error.stack : null }) });
}
