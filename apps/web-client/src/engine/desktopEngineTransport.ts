import { invoke } from "@tauri-apps/api/core";
import { urlParam } from "./autoDrive";
import { validateObserverMetadata } from "./observerMetadata";
import { normalizeNativeOutcome } from "./nativeOutcome";
import type { ChooseGeneralPayload, EngineEnvelope, WorkerRequest, WorkerResponse } from "../types/engine";

/** The messaging seam shared by a Worker and the desktop IPC adapter. */
export interface EngineTransport {
  onmessage: ((event: MessageEvent<WorkerResponse>) => void) | null;
  onerror: ((event: ErrorEvent) => void) | null;
  postMessage(request: WorkerRequest): void;
  terminate(): void;
  addEventListener(type: "messageerror", listener: () => void): void;
}

type Invoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;
export const DESKTOP_ENGINE_KEY = "noname-next-desktop-engine";
export type DesktopEngineChoice = "wasm" | "native";

/** Wasm stays the default until all six PA09 conditions have evidence. */
export function desktopEngineChoice(): DesktopEngineChoice {
  const override = typeof location === "undefined" ? null : urlParam("engineTransport");
  if (override === "native" || override === "wasm") return override;
  try { return localStorage.getItem(DESKTOP_ENGINE_KEY) === "native" ? "native" : "wasm"; }
  catch { return "wasm"; }
}

function normalizeEnvelope(envelope: EngineEnvelope): EngineEnvelope {
  validateObserverMetadata(envelope.view, envelope.choice);
  return { ...envelope, view: { ...envelope.view, outcome: normalizeNativeOutcome(envelope.view.outcome) } };
}

/** Normalize only the two existing local response locations; network envelopes never enter here. */
function normalizeResponse(response: WorkerResponse, request: WorkerRequest): WorkerResponse {
  if (!response.ok || !response.data || typeof response.data !== "object") return response;
  if (request.type === "chooseGeneral") {
    const data = response.data as ChooseGeneralPayload;
    if (data.envelope?.view) return { ...response, data: { ...data, envelope: normalizeEnvelope(data.envelope) } };
  } else {
    const data = response.data as EngineEnvelope;
    if (data.view) return { ...response, data: normalizeEnvelope(data) };
  }
  return response;
}

/** Serial invoke calls carry the same WorkerRequest/WorkerResponse, including controlled errors. */
export class DesktopEngineTransport implements EngineTransport {
  onmessage: EngineTransport["onmessage"] = null;
  onerror: EngineTransport["onerror"] = null;
  private closed = false;
  private tail: Promise<void> = Promise.resolve();
  private resource: Promise<number> | null = null;

  constructor(private call: Invoke = invoke) {}

  postMessage(request: WorkerRequest): void {
    if (this.closed) throw new Error("Desktop engine transport closed");
    this.tail = this.tail.then(async () => {
      if (this.closed) return;
      let response: WorkerResponse;
      try {
        this.resource ??= this.call<number>("native_engine_open");
        const resourceId = await this.resource;
        if (this.closed) return;
        response = await this.call<WorkerResponse>("native_engine_request", { resourceId, request });
        response = normalizeResponse(response, request);
        if (response.id !== request.id || typeof response.ok !== "boolean") {
          throw new Error("invalid IPC response");
        }
      } catch (error) {
        response = { id: request.id, ok: false, error: `Desktop IPC: ${String(error)}; 操作结果可能未知，请先确认当前局面` };
      }
      if (!this.closed) this.onmessage?.({ data: response } as MessageEvent<WorkerResponse>);
    });
  }

  terminate(): void {
    if (this.closed) return;
    this.closed = true;
    void this.resource?.then((resourceId) => this.call("native_engine_close", { resourceId }))
      .catch((error) => console.error("Desktop engine resource close failed", error));
  }
  // invoke returns structured JSON; decoding failures reject its promise.
  addEventListener(_type: "messageerror", _listener: () => void): void {}
}
