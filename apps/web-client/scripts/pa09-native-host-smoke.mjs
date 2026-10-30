// Run with PA09_NATIVE_BRIDGE pointing at cargo's pa09_native_bridge example.
// This exercises the native host and real wasm32 module; Tauri resource tests cover IPC registration separately.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { createInterface } from "node:readline";
import init from "../src/worker/native-pkg/engine_native_wasm.js";
import {
  nativeCreateIdentity, nativeCreateWithPacks, nativeDispatch, nativeEnvelope,
  generalCandidates, chooseGeneral,
} from "../src/worker/nativeBackend.ts";
import { DesktopEngineTransport } from "../src/engine/desktopEngineTransport.ts";
import {
  activateExtension, deactivateExtension, importExtensionPackageValidated,
  readExtensionStore, setMatchFrozen, uninstallExtension,
} from "../src/engine/extensionStore.ts";
import { prepareNativeMatchContent, restoreNativeMatchContent } from "../src/engine/nativeMatchContent.ts";

assert.ok(process.env.PA09_NATIVE_BRIDGE, "set PA09_NATIVE_BRIDGE to the built native host harness");
await init({ module_or_path: readFileSync(new URL("../src/worker/native-pkg/engine_native_wasm_bg.wasm", import.meta.url)) });
const trace = [];
// IPC JSON omits optional undefined fields that Worker structured clone preserves.
const wire = (value) => JSON.parse(JSON.stringify(value));
const storage = new Map();
globalThis.localStorage = {
  getItem: (key) => storage.get(key) ?? null,
  setItem: (key, value) => storage.set(key, String(value)),
  removeItem: (key) => storage.delete(key),
};

function bridge() {
  const child = spawn(process.env.PA09_NATIVE_BRIDGE, [], { stdio: ["pipe", "pipe", "inherit"] });
  const pending = new Map();
  let seq = 1;
  const lines = createInterface({ input: child.stdout });
  lines.on("line", (line) => {
    const response = JSON.parse(line);
    const waiter = pending.get(response.id);
    assert.ok(waiter, `unexpected native response ${response.id}`);
    pending.delete(response.id);
    waiter.resolve(response);
  });
  child.on("exit", (code) => {
    for (const waiter of pending.values()) waiter.reject(new Error(`native bridge exited ${code}`));
    pending.clear();
  });
  const transport = new DesktopEngineTransport(async (command, args) => {
    if (command === "native_engine_open") return 1;
    if (command === "native_engine_close") { child.stdin.end(); return; }
    assert.equal(command, "native_engine_request");
    return new Promise((resolve, reject) => {
      pending.set(args.request.id, { resolve, reject });
      child.stdin.write(`${JSON.stringify(args.request)}\n`);
    });
  });
  const responses = new Map();
  transport.onmessage = (event) => {
    const response = event.data;
    const waiter = responses.get(response.id);
    responses.delete(response.id);
    if (response.ok) waiter.resolve(response.data); else waiter.reject(new Error(response.error));
  };
  return {
    request: (body) => new Promise((resolve, reject) => {
      const id = seq++;
      responses.set(id, { resolve, reject });
      transport.postMessage({ id, ...body });
    }),
    close: () => transport.terminate(),
  };
}

const native = bridge();
try {
  await native.request({ type: "init" });
  let wasm = nativeCreateIdentity(4, 42, 0);
  const start = { type: "start", players: 4, seed: 42, observer: 0, mode: "identity" };
  assert.deepEqual(await native.request(start), nativeEnvelope(wasm));
  let finished = false;
  for (let step = 0; step < 2000; step++) {
    const actual = await native.request({ type: "autoStep" });
    const expected = nativeDispatch(wasm, { type: "autoStep" });
    assert.deepEqual(actual, wire(expected), `full wasm32/native trajectory at step ${step}`);
    trace.push({ workload: "identity", step, hash: actual.hash_hex, revision: actual.revision, outcome: actual.view.outcome });
    if (step === 30 || step === 70) {
      const saved = await native.request({ type: "save" });
      assert.deepEqual(saved, nativeDispatch(wasm, { type: "save" }));
      const expectedRestored = nativeDispatch(wasm, { type: "restore", hex: saved.hex });
      assert.deepEqual(await native.request({ type: "restore", hex: saved.hex }), wire(expectedRestored));
    }
    if (actual.finished) { finished = true; break; }
  }
  assert.ok(finished, "identity game must reach its terminal outcome");
  wasm.engine.free();

  wasm = nativeCreateWithPacks(2, 42, 0, "[]");
  assert.deepEqual(await native.request({ type: "start", players: 2, seed: 42, observer: 0 }), nativeEnvelope(wasm));
  let prototypeFinished = false;
  for (let step = 0; step < 2000; step++) {
    const actual = await native.request({ type: "autoStep" });
    assert.deepEqual(actual, wire(nativeDispatch(wasm, { type: "autoStep" })), `prototype outcome trajectory ${step}`);
    if (actual.finished) {
      const raw = JSON.parse(wasm.engine.view_json(0)).outcome;
      assert.ok(raw.SingleWin.player >= 1 && raw.SingleWin.player <= 2);
      assert.equal(actual.view.outcome.SingleWin.player, raw.SingleWin.player - 1);
      prototypeFinished = true;
      break;
    }
  }
  assert.ok(prototypeFinished, "prototype game must reach a real SingleWin");
  wasm.engine.free();

  wasm = nativeCreateWithPacks(4, 42, 0, "[]");
  assert.deepEqual(await native.request({ type: "start", players: 4, seed: 42, observer: 0 }), nativeEnvelope(wasm));
  const activeAssignment = chooseGeneral(wasm, 0, "黄盖");
  assert.deepEqual(await native.request({ type: "chooseGeneral", seat: 0, name: "黄盖" }), { assignment: activeAssignment, envelope: nativeEnvelope(wasm) });
  for (let phase = 0; phase < 2; phase++) {
    const req = { type: "endPhase", actor: 0 };
    assert.deepEqual(await native.request(req), wire(nativeDispatch(wasm, req)));
  }
  const active = nativeEnvelope(wasm).legal_actions.find((action) => action.ActivateSkill)?.ActivateSkill;
  assert.ok(active, "fixture general must offer a real active skill");
  const skillReq = { type: "activateSkill", actor: 0, skill: active.skill, targets: [], cardTargets: [] };
  assert.deepEqual(await native.request(skillReq), wire(nativeDispatch(wasm, skillReq)));
  const play = nativeEnvelope(wasm).legal_actions.find((action) => action.PlayCard)?.PlayCard;
  assert.ok(play, "fixture hand must offer a real playable card");
  const playReq = { type: "playCard", actor: 0, card: play.card, targets: play.targets };
  const played = await native.request(playReq);
  assert.deepEqual(played, wire(nativeDispatch(wasm, playReq)));
  assert.notEqual(played.result.outcome, "rejected");
  wasm.engine.free();

  const load = (dir) => {
    const packageJson = JSON.parse(readFileSync(new URL(`./fixtures/${dir}/package.npack.json`, import.meta.url)));
    packageJson.characters = JSON.parse(readFileSync(new URL(`./fixtures/${dir}/character.json`, import.meta.url)));
    return JSON.stringify(packageJson);
  };
  const v1 = load("vm-trigger");
  const v2 = load("vm-trigger-v2");
  const key = "official/vm-fixture";
  const validate = (packJson) => native.request({ type: "validatePackage", packJson });
  await importExtensionPackageValidated(v1, validate);
  activateExtension(key);
  const frozen = prepareNativeMatchContent();
  wasm = nativeCreateWithPacks(4, 42, 0, frozen.packsJson);
  assert.deepEqual(await native.request({ type: "start", players: 4, seed: 42, observer: 0, packsJson: frozen.packsJson }), nativeEnvelope(wasm));
  assert.deepEqual(await native.request({ type: "generalCandidates" }), generalCandidates(wasm));
  const assignment = chooseGeneral(wasm, 0, "vm_fixture");
  assert.deepEqual(await native.request({ type: "chooseGeneral", seat: 0, name: "vm_fixture" }), { assignment, envelope: nativeEnvelope(wasm) });
  setMatchFrozen(true);
  await assert.rejects(importExtensionPackageValidated(v2, validate), /对局中/);
  setMatchFrozen(false);
  const runVmPhaseEnd = async (expectedDelta) => {
    for (let step = 0; step < 80; step++) {
      const before = nativeEnvelope(wasm);
      const req = before.choice
        ? { type: "resolveChoice", actor: before.choice.observer, choiceId: before.choice.choice_id,
            selections: before.choice.options.slice(0, Math.max(1, before.choice.min_count ?? 0)).map((option) => option.token), cancelled: false }
        : { type: "endPhase", actor: before.view.current_player };
      const actual = await native.request(req);
      assert.deepEqual(actual, wire(nativeDispatch(wasm, req)));
      trace.push({ workload: `VM-draw-${expectedDelta}`, step, hash: actual.hash_hex, hand: actual.view.seats[0].hand_count });
      if (before.view.phase === "Discard" && actual.view.phase === "End") {
        assert.equal(actual.view.seats[0].hand_count - before.view.seats[0].hand_count, expectedDelta, "VM effect must add its version-specific draw at phase end");
        return;
      }
    }
    assert.fail("VM fixture did not reach its actual phase-end effect");
  };
  await runVmPhaseEnd(1);
  const save = { ...(await native.request({ type: "save" })), ...frozen };
  await importExtensionPackageValidated(v2, validate);
  const upgraded = prepareNativeMatchContent();
  assert.notEqual(upgraded.lockSeed, frozen.lockSeed);
  assert.deepEqual(restoreNativeMatchContent(save), frozen);
  // Upgrading or disabling does not mutate the live engine or its frozen package copy.
  assert.deepEqual(await native.request({ type: "observer" }), nativeEnvelope(wasm));
  deactivateExtension(key);
  assert.equal(prepareNativeMatchContent().packsJson, undefined);
  activateExtension(key);
  assert.equal(prepareNativeMatchContent().lockSeed, upgraded.lockSeed);
  deactivateExtension(key);
  uninstallExtension(key);
  assert.equal(readExtensionStore().installed.length, 0);
  const original = restoreNativeMatchContent(save);
  const req = { type: "restore", hex: save.hex, packsJson: original.packsJson };
  assert.deepEqual(await native.request(req), wire(nativeDispatch(wasm, req)));
  const beforeRejected = await native.request({ type: "save" });
  await assert.rejects(native.request({ type: "restore", hex: save.hex, packsJson: upgraded.packsJson }), /恢复|Content|content|Mismatch|mismatch/);
  assert.deepEqual(await native.request({ type: "save" }), beforeRejected);
  const corrupt = JSON.parse(v2);
  corrupt.modules[0].logic_hash[0] ^= 1;
  const beforeStore = storage.get("noname-next.extension-store.v1");
  await assert.rejects(importExtensionPackageValidated(JSON.stringify(corrupt), validate), /logic_hash|fingerprint/);
  assert.equal(storage.get("noname-next.extension-store.v1"), beforeStore);
  wasm.engine.free();
  wasm = nativeCreateWithPacks(4, 42, 0, upgraded.packsJson);
  assert.deepEqual(await native.request({ type: "start", players: 4, seed: 42, observer: 0, packsJson: upgraded.packsJson }), nativeEnvelope(wasm));
  const upgradedAssignment = chooseGeneral(wasm, 0, "vm_fixture");
  assert.deepEqual(await native.request({ type: "chooseGeneral", seat: 0, name: "vm_fixture" }), { assignment: upgradedAssignment, envelope: nativeEnvelope(wasm) });
  await runVmPhaseEnd(2);
  wasm.engine.free();
  if (process.env.PA09_TRACE_PATH) writeFileSync(process.env.PA09_TRACE_PATH, JSON.stringify(trace, null, 2));
  console.log(JSON.stringify({ full_wasm32_native_steps: trace.filter((row) => row.workload === "identity").length, terminal: trace.findLast((row) => row.workload === "identity"), vm_lifecycle: "passed", transport: "actual native host over stdio through production IPC transport adapter (not a WebView measurement)" }, null, 2));
} finally {
  native.close();
}
