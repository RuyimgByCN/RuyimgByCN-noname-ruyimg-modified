import assert from "node:assert/strict";
import { test } from "node:test";
import { DesktopEngineTransport, desktopEngineChoice, DESKTOP_ENGINE_KEY } from "../src/engine/desktopEngineTransport.ts";

const tick = () => new Promise((resolve) => setImmediate(resolve));

test("desktop IPC preserves request order, ids, errors and continues on the same engine", async () => {
  const calls = [];
  let release;
  const first = new Promise((resolve) => { release = resolve; });
  const transport = new DesktopEngineTransport(async (command, args) => {
    if (command === "native_engine_open") return 17;
    if (command === "native_engine_close") return;
    calls.push({ command, ...args });
    const request = args.request;
    if (request.id === 1) return first;
    if (request.id === 2) return { id: 2, ok: false, error: "native backend: useSkillCard 未映射" };
    if (request.id === 3) throw new Error("bridge lost");
    return { id: request.id, ok: true, data: { revision: 7 } };
  });
  const replies = [];
  transport.onmessage = (event) => replies.push(event.data);
  const start = { id: 1, type: "start", players: 4, seed: 42, observer: 0, mode: "identity", packsJson: "[]" };
  transport.postMessage(start);
  transport.postMessage({ id: 2, type: "useSkillCard", actor: 0, skill: 1, cardKind: 0, targets: [], equipSlot: 0 });
  transport.postMessage({ id: 3, type: "autoStep" });
  transport.postMessage({ id: 4, type: "observer" });
  await tick();
  assert.deepEqual(calls, [{ command: "native_engine_request", resourceId: 17, request: start }]);
  release({ id: 1, ok: true, data: { revision: 6 } });
  await tick();
  assert.deepEqual(calls.map((call) => call.request.id), [1, 2, 3, 4]);
  assert.deepEqual(replies[1], { id: 2, ok: false, error: "native backend: useSkillCard 未映射" });
  assert.match(replies[2].error, /bridge lost.*结果可能未知/);
  assert.deepEqual(replies[3], { id: 4, ok: true, data: { revision: 7 } });
  transport.terminate();
});

test("dispose suppresses late IPC replies and never submits queued operations", async () => {
  let release;
  const sent = [];
  const transport = new DesktopEngineTransport((command, args) => {
    if (command === "native_engine_open") return Promise.resolve(19);
    if (command === "native_engine_close") return Promise.resolve();
    sent.push(args.request.id);
    return new Promise((resolve) => { release = resolve; });
  });
  const replies = [];
  transport.onmessage = (event) => replies.push(event.data);
  transport.postMessage({ id: 1, type: "autoStep" });
  transport.postMessage({ id: 2, type: "autoStep" });
  await tick();
  transport.terminate();
  release({ id: 1, ok: true, data: "late" });
  await tick();
  assert.deepEqual(sent, [1]);
  assert.deepEqual(replies, []);
  assert.throws(() => transport.postMessage({ id: 3, type: "save" }), /closed/);
});

test("mismatched IPC response is an explicit unknown-result error, without retry", async () => {
  let calls = 0;
  const transport = new DesktopEngineTransport(async (command) => {
    if (command === "native_engine_open") return 20;
    if (command === "native_engine_close") return;
    calls++; return { id: 99, ok: true, data: "wrong" };
  });
  const response = new Promise((resolve) => { transport.onmessage = (event) => resolve(event.data); });
  transport.postMessage({ id: 1, type: "endPhase", actor: 0 });
  assert.match((await response).error, /invalid IPC response.*结果可能未知/);
  assert.equal(calls, 1);
  transport.terminate();
});

test("Wasm is the default and an explicit persistent/URL fallback remains available", () => {
  const originalStorage = globalThis.localStorage;
  const originalLocation = globalThis.location;
  const values = new Map();
  globalThis.localStorage = { getItem: (key) => values.get(key) ?? null };
  globalThis.location = { search: "", hash: "" };
  try {
    assert.equal(desktopEngineChoice(), "wasm");
    values.set(DESKTOP_ENGINE_KEY, "native");
    assert.equal(desktopEngineChoice(), "native");
    globalThis.location.search = "?engineTransport=wasm";
    assert.equal(desktopEngineChoice(), "wasm");
    globalThis.location.search = "?engineTransport=native";
    values.clear();
    assert.equal(desktopEngineChoice(), "native");
    globalThis.location.search = "";
    globalThis.location.hash = "#/battle?engineTransport=wasm";
    assert.equal(desktopEngineChoice(), "wasm");
  } finally {
    if (originalStorage === undefined) delete globalThis.localStorage; else globalThis.localStorage = originalStorage;
    if (originalLocation === undefined) delete globalThis.location; else globalThis.location = originalLocation;
  }
});

test("dispose during resource allocation closes it once and submits no request", async () => {
  let opened;
  const calls = [];
  const transport = new DesktopEngineTransport((command, args) => {
    calls.push({ command, args });
    if (command === "native_engine_open") return new Promise((resolve) => { opened = resolve; });
    return Promise.resolve();
  });
  for (let id = 1; id <= 8; id++) transport.postMessage({ id, type: "init" });
  await tick();
  transport.terminate();
  transport.terminate();
  opened(25);
  await tick();
  assert.deepEqual(calls.map((call) => call.command), ["native_engine_open", "native_engine_close"]);
  assert.deepEqual(calls[1].args, { resourceId: 25 });
});

test("IPC and Worker convert native SingleWin seats exactly once without modifying raw projections", async () => {
  const { nativeObserverToSnapshot } = await import("../src/worker/nativeBackend.ts");
  for (const player of [1, 4]) {
    const raw = { SingleWin: { player } };
    const view = { seat: 0, players: [], discard: [], table: [], turn: 0, phase: "end", round: 1, outcome: raw };
    const envelope = { result: null, events: [], choice: null, view: { outcome: raw }, legal_actions: [], hash_hex: "hash", revision: 1, finished: true };
    const transport = new DesktopEngineTransport(async (command, args) => {
      if (command === "native_engine_open") return 26;
      if (command === "native_engine_close") return;
      return { id: args.request.id, ok: true, data: args.request.type === "chooseGeneral" ? { assignment: {}, envelope } : envelope };
    });
    const pending = new Map();
    transport.onmessage = (event) => pending.get(event.data.id)(event.data.data);
    const send = (body) => new Promise((resolve) => { pending.set(body.id, resolve); transport.postMessage(body); });
    assert.deepEqual(nativeObserverToSnapshot(view).outcome, { SingleWin: { player: player - 1 } });
    assert.deepEqual((await send({ id: 1, type: "observer" })).view.outcome, { SingleWin: { player: player - 1 } });
    assert.deepEqual((await send({ id: 2, type: "chooseGeneral", seat: 0, name: "曹操" })).envelope.view.outcome, { SingleWin: { player: player - 1 } });
    assert.deepEqual(raw, { SingleWin: { player } });
    assert.equal(envelope.view.outcome, raw);
    transport.terminate();
  }
});

test("IPC refuses malformed B metadata before delivering a view envelope", async () => {
  const transport = new DesktopEngineTransport(async (command, args) => {
    if (command === "native_engine_open") return 27;
    if (command === "native_engine_close") return;
    return { id: args.request.id, ok: true, data: { view: { seats: [{ generals: [{ def: "0", name: "x", faction: "wei", revealed: false }, null] }] }, choice: null } };
  });
  const reply = new Promise((resolve) => { transport.onmessage = (event) => resolve(event.data); });
  transport.postMessage({ id: 1, type: "observer" });
  const response = await reply;
  assert.equal(response.ok, false);
  assert.match(response.error, /generals metadata invalid/);
  transport.terminate();
});

for (const blockedAt of ["open", "request"]) {
  test(`desktop admission bounds 128 submissions while ${blockedAt} is blocked, before extending the promise tail`, async () => {
    let release;
    const blocked = new Promise((resolve) => { release = resolve; });
    const calls = [];
    const transport = new DesktopEngineTransport(async (command, args) => {
      calls.push({ command, args });
      if (command === "native_engine_open") return blockedAt === "open" ? blocked : 40;
      if (command === "native_engine_close") return;
      if (blockedAt === "request" && args.request.id === 1) await blocked;
      return { id: args.request.id, ok: true, data: args.request.id };
    });
    const replies = [];
    transport.onmessage = (event) => replies.push(event.data.id);
    const accepted = [];
    const rejected = [];
    for (let id = 1; id <= 128; id++) {
      const tailBefore = transport.tail;
      try {
        transport.postMessage({ id, type: "autoStep" });
        accepted.push(id);
        if (id === 1) await tick();
      } catch (error) {
        assert.match(error.message, /not submitted|未提交/);
        assert.doesNotMatch(error.message, /unknown|未知/);
        assert.equal(transport.tail, tailBefore, "rejection must not append another promise");
        rejected.push(id);
      }
    }
    await tick();
    const beforeRelease = calls.map((call) => call.command);
    release(40);
    await tick();
    assert.deepEqual(accepted, [1, 2, 3, 4, 5, 6, 7, 8]);
    assert.equal(rejected.length, 120);
    assert.deepEqual(beforeRelease, blockedAt === "open" ? ["native_engine_open"] : ["native_engine_open", "native_engine_request"]);
    assert.deepEqual(calls.filter((call) => call.command === "native_engine_request").map((call) => call.args.request.id), accepted);
    assert.deepEqual(replies, accepted);
    transport.postMessage({ id: 129, type: "status" });
    await tick();
    assert.deepEqual(replies, [...accepted, 129]);
    transport.terminate();
  });
}

test("IPC errors, normalization errors and a throwing listener release admission and leave the tail usable", async () => {
  const calls = [];
  const errors = [];
  const originalError = console.error;
  const transport = new DesktopEngineTransport(async (command, args) => {
    if (command === "native_engine_open") return 41;
    if (command === "native_engine_close") return;
    const id = args.request.id;
    calls.push(id);
    if (id === 1) throw new Error("bridge lost");
    if (id === 2) return { id, ok: false, error: "controlled failure" };
    if (id === 3) return { id: 999, ok: true };
    if (id === 5) return { id, ok: true, data: { view: { seats: [{ generals: [{ revealed: false }, null] }] } } };
    return { id, ok: true, data: id };
  });
  const replies = [];
  console.error = (...args) => errors.push(args);
  transport.onmessage = (event) => {
    replies.push(event.data);
    if (event.data.id === 4) throw new Error("listener failed");
  };
  try {
    for (let id = 1; id <= 8; id++) transport.postMessage({ id, type: "autoStep" });
    await tick();
    assert.deepEqual(calls, [1, 2, 3, 4, 5, 6, 7, 8]);
    assert.match(replies[0].error, /bridge lost.*结果可能未知/);
    assert.equal(replies[1].error, "controlled failure");
    assert.match(replies[2].error, /invalid IPC response.*结果可能未知/);
    assert.match(replies[4].error, /generals metadata invalid/);
    assert.deepEqual(replies.map((reply) => reply.id), calls);
    assert.equal(errors.length, 1);
    assert.match(String(errors[0]), /listener failed/);
    for (let id = 9; id <= 16; id++) transport.postMessage({ id, type: "status" });
    await tick();
    assert.deepEqual(calls, Array.from({ length: 16 }, (_, i) => i + 1));
    assert.deepEqual(replies.map((reply) => reply.id), calls);
  } finally {
    console.error = originalError;
    transport.terminate();
  }
});

test("a rejected resource open drains accepted slots without submitting requests", async () => {
  const calls = [];
  const transport = new DesktopEngineTransport(async (command) => {
    calls.push(command);
    throw new Error("open unavailable");
  });
  const replies = [];
  transport.onmessage = (event) => replies.push(event.data);
  for (let id = 1; id <= 8; id++) transport.postMessage({ id, type: "init" });
  await tick();
  for (let id = 9; id <= 16; id++) transport.postMessage({ id, type: "status" });
  await tick();
  assert.deepEqual(replies.map((reply) => reply.id), Array.from({ length: 16 }, (_, i) => i + 1));
  assert.ok(replies.every((reply) => !reply.ok && /open unavailable/.test(reply.error)));
  assert.deepEqual(calls, ["native_engine_open"]);
  // A failed open retains the existing resource promise; terminate still reports its failure.
  const originalError = console.error;
  const errors = [];
  console.error = (...args) => errors.push(args);
  try {
    transport.terminate();
    await tick();
    assert.equal(errors.length, 1);
    assert.match(String(errors[0]), /open unavailable/);
  } finally { console.error = originalError; }
});
