import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import ts from "typescript";
import { invalidPresentationBatches, presentationEnvelope } from "./presentation-fixtures.mjs";
import { validatePresentationResponse } from "../src/engine/presentationMetadata.ts";
import { DesktopEngineTransport, MAX_ENGINE_OUTSTANDING } from "../src/engine/desktopEngineTransport.ts";

async function settles(promise) {
  let timer;
  try {
    return await Promise.race([
      promise,
      new Promise((_, reject) => { timer = setTimeout(() => reject(new Error("request remained pending")), 500); }),
    ]);
  } finally {
    clearTimeout(timer);
  }
}

// Use the production client class with a controllable Worker; the browser test
// additionally exercises actual browser event dispatch and a native Worker.
function fixture({ autoInit = true, desktopNative = false, invoke, sab = false } = {}) {
  const source = readFileSync(new URL("../src/engine/client.ts", import.meta.url), "utf8");
  const ast = ts.createSourceFile("client.ts", source, ts.ScriptTarget.Latest, true);
  const declaration = ast.statements.find((node) => ts.isClassDeclaration(node) && node.name?.text === "EngineClient");
  assert.ok(declaration);
  const printed = ts.createPrinter().printNode(ts.EmitHint.Unspecified, declaration, ast)
    .replace(/\bexport\s+class\s+EngineClient/, "class EngineClient")
    .replaceAll("import.meta.env", "({})");
  const js = ts.transpileModule(printed, { compilerOptions: { target: ts.ScriptTarget.ES2022 } })
    .outputText.replace(/\nexport\s*\{\};?\s*$/, "");
  const workers = [];
  const failed = [];
  const sent = [];
  const received = [];
  class FakeWorker {
    sent = [];
    terminated = false;
    constructor() { workers.push(this); }
    postMessage(body) {
      this.sent.push(body);
      if (autoInit && body.type === "init") {
        queueMicrotask(() => this.reply(body.id, "ready"));
      }
    }
    addEventListener(type, listener) { if (type === "messageerror") this.onmessageerror = listener; }
    reply(id, data) { this.onmessage?.({ data: { id, ok: true, data } }); }
    terminate() { this.terminated = true; }
  }
  const EngineClient = new Function(
    "NativeEngineWorker", "LegacyEngineWorker", "isTauriShell", "perfRequestSent",
    "perfRequestFailed", "perfResponseReceived", "perfWorkerTimings", "startPerfProbe",
    "urlParam", "console", "desktopEngineChoice", "DesktopEngineTransport", "validatePresentationResponse", "MAX_ENGINE_OUTSTANDING", js + "\nreturn EngineClient;",
  )(FakeWorker, FakeWorker, () => desktopNative, (id) => sent.push(id), (id) => failed.push(id),
    (id) => received.push(id), () => {}, () => false, () => sab ? "sab" : null, { error: () => {} },
    () => desktopNative ? "native" : "wasm", class extends DesktopEngineTransport {
      constructor() { super(invoke); }
    }, validatePresentationResponse, MAX_ENGINE_OUTSTANDING);
  return { client: new EngineClient(), workers, failed, sent, received };
}

test("Worker 错误拒绝所有等待请求、保留句柄并允许核对和新请求", async () => {
  const f = fixture();
  await f.client.connect();
  const worker = f.workers[0];
  const first = f.client.endPhase(1);
  const second = f.client.autoStep();
  const rejected = Promise.all([
    assert.rejects(first, /结果未知/),
    assert.rejects(second, /结果未知/),
  ]);
  worker.onerror({ message: "fault" });
  await settles(rejected);
  assert.equal(f.client.pending.size, 0);
  assert.deepEqual(f.failed, [2, 3]);
  assert.equal(worker.terminated, false);
  worker.reply(2, "late");
  const retry = f.client.autoStep();
  worker.reply(4, { revision: 5 });
  assert.deepEqual(await retry, { revision: 5 });
  f.client.dispose();
});

test("回包无法解码时等待请求失败，原 Worker 保留", async () => {
  const f = fixture();
  await f.client.connect();
  const worker = f.workers[0];
  const pending = f.client.endPhase(1);
  const rejected = assert.rejects(pending, /结果未知/);
  worker.onmessageerror?.({});
  await settles(rejected);
  assert.equal(f.client.pending.size, 0);
  assert.equal(worker.terminated, false);
  f.client.dispose();
});

test("初始化错误有结果，后续可创建新 Worker", async () => {
  const f = fixture({ autoInit: false });
  const pending = f.client.connect();
  const rejected = assert.rejects(pending, /结果未知/);
  f.workers[0].onerror({ message: "init fault" });
  await settles(rejected);
  assert.equal(f.workers[0].terminated, true);
  const next = f.client.connect();
  const worker = f.workers[1];
  worker.reply(worker.sent[0].id, "ready");
  await next;
  f.client.dispose();
});

test("释放时等待请求不悬挂，旧 Worker 错误不影响新连接", async () => {
  const f = fixture();
  await f.client.connect();
  const old = f.workers[0];
  const pending = f.client.endPhase(1);
  const rejected = assert.rejects(pending, /结果未知/);
  f.client.dispose();
  await settles(rejected);
  assert.equal(old.terminated, true);
  await f.client.connect();
  const fresh = f.workers[1];
  const next = f.client.endPhase(1);
  old.onerror({ message: "late" });
  old.onmessageerror({});
  old.reply(fresh.sent.at(-1).id, { revision: 999 });
  assert.equal(f.client.pending.size, 1);
  fresh.reply(fresh.sent.at(-1).id, { revision: 4 });
  assert.deepEqual(await next, { revision: 4 });
  f.client.dispose();
});

test("初始化被释放后立即重连，旧连接的失败不能终止新 Worker", async () => {
  const f = fixture({ autoInit: false });
  const old = f.client.connect();
  const rejected = assert.rejects(old, /结果未知/);
  f.client.dispose();
  const next = f.client.connect();
  const fresh = f.workers[1];
  await settles(rejected);
  assert.equal(fresh.terminated, false);
  fresh.reply(fresh.sent[0].id, "ready");
  await next;
  f.client.dispose();
});

test("空闲 Worker 的错误或释放不会结束其他请求的共享等待", async () => {
  const f = fixture();
  await f.client.connect();
  const previous = globalThis.__latencySabDebug;
  let settled = 0;
  globalThis.__latencySabDebug = { settle: () => { settled += 1; } };
  try {
    f.workers[0].onerror({ message: "idle fault" });
    f.client.dispose();
    assert.equal(settled, 0);
  } finally {
    globalThis.__latencySabDebug = previous;
  }
});

test("desktop IPC uses the production client's sequence, pending, errors and disposal without Worker fallback", async () => {
  const calls = [];
  let release;
  const f = fixture({ desktopNative: true, invoke: async (command, args) => {
    calls.push({ command, args });
    if (command === "native_engine_open") return 31;
    if (command === "native_engine_close") return;
    const { request } = args;
    if (request.type === "init") return { id: request.id, ok: true, data: "native ready" };
    if (request.type === "useSkillCard") return { id: request.id, ok: false, error: "useSkillCard 未映射" };
    if (request.type === "autoStep") return new Promise((resolve) => { release = resolve; });
    return { id: request.id, ok: true, data: { kind: "status", revision: 4, finished: false } };
  } });
  await f.client.connect();
  assert.equal(f.client.getTransportKind(), "desktop-native");
  assert.equal(f.workers.length, 0);
  await assert.rejects(f.client.useSkillCard(0, 1, 0, [], 0), /未映射/);
  assert.equal((await f.client.status()).revision, 4);
  assert.deepEqual(calls.filter((c) => c.command === "native_engine_request").map((c) => c.args.request.id), [1, 2, 3]);
  const pending = f.client.autoStep();
  const rejected = assert.rejects(pending, /结果未知/);
  await new Promise((resolve) => setImmediate(resolve));
  f.client.dispose();
  await settles(rejected);
  release({ id: 4, ok: true, data: { revision: 5 } });
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(f.client.pending.size, 0);
  assert.deepEqual(f.failed, [4]);
  assert.equal(calls.filter((c) => c.command === "native_engine_close").length, 1);
  assert.equal(f.workers.length, 0);
});

test("desktop burst keeps production EngineClient pending and unresolved promises at eight, rejects before IPC, and resumes without retry", async (t) => {
  const previousDebug = globalThis.__latencySabDebug;
  let wakes = 0;
  let settlements = 0;
  globalThis.__latencySabDebug = { wake: () => { wakes++; }, settle: () => { settlements++; } };
  t.after(() => {
    if (previousDebug === undefined) delete globalThis.__latencySabDebug;
    else globalThis.__latencySabDebug = previousDebug;
  });
  let release;
  const blocked = new Promise((resolve) => { release = resolve; });
  const calls = [];
  const f = fixture({ desktopNative: true, invoke: async (command, args) => {
    calls.push({ command, args });
    if (command === "native_engine_open") return 51;
    if (command === "native_engine_close") return;
    const { request } = args;
    if (request.type !== "init") await blocked;
    return { id: request.id, ok: true, data: { revision: request.id } };
  } });
  await f.client.connect();
  const settled = [];
  const promises = [];
  for (let i = 0; i < 128; i++) {
    promises.push(f.client.autoStep().then(
      (value) => { settled.push({ ok: true, value }); },
      (error) => { settled.push({ ok: false, error }); },
    ));
    assert.ok(f.client.pending.size <= 8, "excess requests must not insert pending");
    if (i === 0) await new Promise((resolve) => setImmediate(resolve));
  }
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(f.client.pending.size, 8);
  assert.deepEqual([...f.client.pending.keys()], [2, 3, 4, 5, 6, 7, 8, 9]);
  assert.equal(settled.length, 120, "only eight caller promises may remain unresolved");
  assert.equal(wakes, 9);
  assert.equal(settlements, 1, "rejected excess requests must not settle accepted shared waits");
  assert.equal(f.sent.length, 9);
  assert.deepEqual(f.received, [1]);
  assert.ok(settled.every((result) => !result.ok && /not submitted|未提交/.test(result.error.message) && !/未知|unknown/.test(result.error.message)));
  assert.deepEqual(f.failed, [], "capacity rejection precedes perf submission");
  assert.deepEqual(calls.filter((call) => call.command === "native_engine_request").map((call) => call.args.request.id), [1, 2]);
  release();
  await settles(Promise.all(promises));
  assert.equal(f.client.pending.size, 0);
  assert.equal(settled.filter((result) => result.ok).length, 8);
  assert.deepEqual(f.received, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
  assert.equal(settlements, 9);
  assert.equal(f.sent.length, f.failed.length + f.received.length);
  assert.deepEqual(calls.filter((call) => call.command === "native_engine_request").map((call) => call.args.request.id), [1, 2, 3, 4, 5, 6, 7, 8, 9]);
  assert.equal((await f.client.status()).revision, 130);
  assert.equal(f.workers.length, 0);
  assert.equal(f.client.getTransportKind(), "desktop-native");
  f.client.dispose();
});

test("disposing eight accepted desktop requests retires the old resource and reconnects independently", async () => {
  let release;
  let resource = 60;
  const calls = [];
  const f = fixture({ desktopNative: true, invoke: async (command, args) => {
    calls.push({ command, args });
    if (command === "native_engine_open") return ++resource;
    if (command === "native_engine_close") return;
    if (args.resourceId === 61 && args.request.type !== "init") {
      return new Promise((resolve) => { release = resolve; });
    }
    return { id: args.request.id, ok: true, data: { revision: args.resourceId } };
  } });
  await f.client.connect();
  const rejected = Promise.all(Array.from({ length: 8 }, () => assert.rejects(f.client.autoStep(), /结果未知/)));
  await new Promise((resolve) => setImmediate(resolve));
  f.client.dispose();
  await settles(rejected);
  assert.equal(f.client.pending.size, 0);
  await f.client.connect();
  assert.equal((await f.client.status()).revision, 62);
  release({ id: 2, ok: true, data: { revision: 999 } });
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(f.client.pending.size, 0);
  assert.deepEqual(calls.filter((call) => call.command === "native_engine_request" && call.args.resourceId === 61).map((call) => call.args.request.id), [1, 2]);
  assert.deepEqual(calls.filter((call) => call.command === "native_engine_close").map((call) => call.args.resourceId), [61]);
  assert.equal(f.workers.length, 0);
  f.client.dispose();
  await new Promise((resolve) => setImmediate(resolve));
  assert.deepEqual(calls.filter((call) => call.command === "native_engine_close").map((call) => call.args.resourceId), [61, 62]);
});

// Actual Worker and SAB settlement must validate before removing/acknowledging a pending call.
for (const {name,batch} of invalidPresentationBatches()) {
  test(`Worker bad present ${name} preserves pending until dispose`, async () => {
    const f=fixture();await f.client.connect();const worker=f.workers[0];
    const promise=f.client.autoStep();const id=worker.sent.at(-1).id;
    const pending=[...f.client.pending];const received=f.received.length;
    worker.reply(id,presentationEnvelope(batch));
    await assert.rejects(promise,/协议错误，操作结果未确认/);
    assert.deepEqual([...f.client.pending],pending);assert.equal(f.received.length,received);
    f.client.dispose();assert.equal(f.client.pending.size,0);
  });
}

for (const {name,batch} of invalidPresentationBatches()) {
  test(`actual IPC to EngineClient bad present ${name} preserves pending`, async () => {
    const f=fixture({desktopNative:true,invoke:async (command,args)=>command==="native_engine_open"?1:
      command==="native_engine_request"?{id:args.request.id,ok:true,data:args.request.type==="init"?"ready":presentationEnvelope(batch)}:undefined});
    await f.client.connect();const promise=f.client.autoStep();const pending=[...f.client.pending];
    const received=f.received.length;await assert.rejects(promise,/协议错误，操作结果未确认/);
    assert.deepEqual([...f.client.pending],pending);assert.equal(f.received.length,received);
    f.client.dispose();assert.equal(f.client.pending.size,0);
  });
}

test("actual SAB poll rejects malformed presentation before settling or deleting pending", async t => {
  const old={document:globalThis.document,crossOriginIsolated:globalThis.crossOriginIsolated,requestAnimationFrame:globalThis.requestAnimationFrame};
  const frames=[];globalThis.document={createElement:()=>({style:{}}),head:{appendChild(){}},body:{appendChild(){}}};
  globalThis.crossOriginIsolated=true;globalThis.requestAnimationFrame=fn=>{frames.push(fn);return frames.length;};
  t.after(()=>{Object.assign(globalThis,old);});
  const f=fixture({sab:true});await f.client.connect();const worker=f.workers[0];
  const promise=f.client.autoStep();const id=worker.sent.at(-1).id;
  const pending=[...f.client.pending];const received=f.received.length;
  const sab=worker.sent[0].latencySab;assert.ok(sab instanceof SharedArrayBuffer);
  const raw=new TextEncoder().encode(JSON.stringify({id,ok:true,data:presentationEnvelope(null)}));
  new Uint8Array(sab,8,raw.length).set(raw);const ints=new Int32Array(sab,0,2);Atomics.store(ints,1,raw.length);Atomics.store(ints,0,1);
  frames.shift()();await assert.rejects(promise,/协议错误，操作结果未确认/);
  assert.deepEqual([...f.client.pending],pending);assert.equal(f.received.length,received);assert.ok(frames.length>0);
  f.client.dispose();assert.equal(f.client.pending.size,0);
});

test("actual Worker and IPC nested chooseGeneral preserve mapping on bad presentation", async () => {
  for (const desktopNative of [false,true]) {
    const bad={assignment:{seat:0,name:"liubei",revision:2,state_hash:"00"},envelope:presentationEnvelope(null)};
    const f=fixture({desktopNative,invoke:async (command,args)=>command==="native_engine_open"?1:
      command==="native_engine_request"?{id:args.request.id,ok:true,data:args.request.type==="init"?"ready":bad}:undefined});
    await f.client.connect();const promise=f.client.chooseGeneral(0,"liubei");const pending=[...f.client.pending];const received=f.received.length;
    if (!desktopNative) f.workers[0].reply(f.workers[0].sent.at(-1).id,bad);
    await assert.rejects(promise,/协议错误，操作结果未确认/);assert.deepEqual([...f.client.pending],pending);assert.equal(f.received.length,received);
    f.client.dispose();assert.equal(f.client.pending.size,0);
  }
});

test("shared Worker request guard sends eight and rejects ninth before map/postMessage", async () => {
  const f=fixture();await f.client.connect();const worker=f.workers[0];
  const calls=Array.from({length:8},()=>f.client.autoStep());const pending=[...f.client.pending];const sent=worker.sent.length;
  await assert.rejects(f.client.autoStep(),/未提交/);assert.deepEqual([...f.client.pending],pending);assert.equal(worker.sent.length,sent);
  for (const request of worker.sent.slice(-8)) worker.reply(request.id,{kind:"plan",outcome:'{"outcome":"accepted"}'});
  await Promise.all(calls);assert.equal(f.client.pending.size,0);
  const next=f.client.autoStep();worker.reply(worker.sent.at(-1).id,{outcome:"accepted"});await next;f.client.dispose();
});

test("eight retained malformed responses keep capacity until explicit dispose", async () => {
  for (const desktopNative of [false,true]) {
    const f=fixture({desktopNative,invoke:async (command,args)=>command==="native_engine_open"?1:
      command==="native_engine_request"?{id:args.request.id,ok:true,data:args.request.type==="init"?"ready":presentationEnvelope(null)}:undefined});
    await f.client.connect();const calls=Array.from({length:8},()=>f.client.autoStep());
    const results=Promise.all(calls.map(p=>assert.rejects(p,/结果未确认/)));
    if (!desktopNative) for (const request of f.workers[0].sent.slice(-8)) f.workers[0].reply(request.id,presentationEnvelope(null));
    await results;const pending=[...f.client.pending];assert.equal(pending.length,8);
    await assert.rejects(f.client.autoStep(),/未提交/);assert.deepEqual([...f.client.pending],pending);
    f.client.dispose();assert.equal(f.client.pending.size,0);
  }
});
