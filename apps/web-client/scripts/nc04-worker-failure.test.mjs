import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import ts from "typescript";

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
function fixture({ autoInit = true } = {}) {
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
    "urlParam", "console", js + "\nreturn EngineClient;",
  )(FakeWorker, FakeWorker, () => false, () => {}, (id) => failed.push(id),
    () => {}, () => {}, () => false, () => null, { error: () => {} });
  return { client: new EngineClient(), workers, failed };
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
