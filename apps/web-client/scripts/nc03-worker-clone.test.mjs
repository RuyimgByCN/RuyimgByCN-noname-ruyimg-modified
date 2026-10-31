import { MAX_ENGINE_OUTSTANDING } from "../src/engine/desktopEngineTransport.ts";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import ts from "typescript";
import { reactive } from "vue";
import * as perf from "../src/engine/perfProbe.ts";

// Execute the actual client class with a clone-checking Worker; Vite's ?worker imports
// cannot be loaded by node:test. The browser regression exercises the production build.
function fixture() {
  const source = readFileSync(new URL("../src/engine/client.ts", import.meta.url), "utf8");
  const ast = ts.createSourceFile("client.ts", source, ts.ScriptTarget.Latest, true);
  const declaration = ast.statements.find((node) => ts.isClassDeclaration(node) && node.name?.text === "EngineClient");
  assert.ok(declaration);
  const printed = ts.createPrinter().printNode(ts.EmitHint.Unspecified, declaration, ast)
    .replace(/\bexport\s+class\s+EngineClient/, "class EngineClient")
    .replaceAll("import.meta.env", "({})");
  const js = ts.transpileModule(printed, { compilerOptions: { target: ts.ScriptTarget.ES2022 } })
    .outputText.replace(/\nexport\s*\{\};?\s*$/, "");
  const sent = [];
  const failed = [];
  const EngineClient = new Function("perfRequestSent", "perfRequestFailed", "MAX_ENGINE_OUTSTANDING", js + "\nreturn EngineClient;")(
    () => {}, (id) => failed.push(id), MAX_ENGINE_OUTSTANDING,
  );
  const client = new EngineClient();
  const worker = {
    postMessage(body) {
      sent.push(structuredClone(body));
      queueMicrotask(() => {
        const pending = client.pending.get(body.id);
        client.pending.delete(body.id);
        pending.resolve({ ok: true });
      });
    },
  };
  client.worker = worker;
  return { client, worker, sent, failed };
}

test("本地命令将响应式数值数组原样交给可克隆的 Worker 消息", async () => {
  const { client, sent } = fixture();
  const targets = reactive([2, 1]);
  const cards = reactive([19, 23]);
  const choices = reactive([1, 0]);
  await client.playCard(1, 8, targets);
  await client.activateSkill(1, 4, targets, cards);
  await client.useSkillCard(1, 4, 6, targets, 2);
  await client.resolveChoice(1, 7, choices, false);
  assert.deepEqual(sent.map(({ id, ...body }) => body), [
    { type: "playCard", actor: 1, card: 8, targets: [2, 1] },
    { type: "activateSkill", actor: 1, skill: 4, targets: [2, 1], cardTargets: [19, 23] },
    { type: "useSkillCard", actor: 1, skill: 4, cardKind: 6, targets: [2, 1], equipSlot: 2 },
    { type: "resolveChoice", actor: 1, choiceId: 7, selections: [1, 0], cancelled: false },
  ]);
  assert.deepEqual([...targets], [2, 1]);
  assert.equal(client.pending.size, 0);
});

test("postMessage 同步失败释放请求、性能记录和 SAB 唤醒，随后可重试", async (t) => {
  const { client, worker, sent, failed } = fixture();
  const previous = globalThis.__latencySabDebug;
  let wakes = 0;
  let settles = 0;
  globalThis.__latencySabDebug = { wake: () => wakes++, settle: () => settles++ };
  t.after(() => {
    if (previous === undefined) delete globalThis.__latencySabDebug;
    else globalThis.__latencySabDebug = previous;
  });
  const originalPost = worker.postMessage;
  worker.postMessage = () => { throw new DOMException("cannot clone", "DataCloneError"); };
  await assert.rejects(client.playCard(1, 8, reactive([])), { name: "DataCloneError" });
  assert.equal(client.pending.size, 0);
  assert.deepEqual(failed, [1]);
  assert.equal(wakes, 1);
  assert.equal(settles, 1);
  worker.postMessage = originalPost;
  await client.endPhase(1);
  assert.equal(sent.length, 1);
  assert.equal(sent[0].type, "endPhase");
  assert.equal(client.pending.size, 0);
});

test("其他请求在途时发送失败不提前结束 SAB 唤醒", async (t) => {
  const { client, worker } = fixture();
  const previous = globalThis.__latencySabDebug;
  let settles = 0;
  globalThis.__latencySabDebug = { wake: () => {}, settle: () => settles++ };
  t.after(() => {
    if (previous === undefined) delete globalThis.__latencySabDebug;
    else globalThis.__latencySabDebug = previous;
  });
  worker.postMessage = () => {};
  const first = client.endPhase(1);
  worker.postMessage = () => { throw new Error("send failed"); };
  await assert.rejects(client.endPhase(1), /send failed/);
  assert.equal(client.pending.size, 1);
  assert.equal(settles, 0);
  client.pending.get(1).resolve({ ok: true });
  client.pending.delete(1);
  await first;
});

test("性能探针丢弃未发送请求，保留成功请求", (t) => {
  const previous = { window: globalThis.window, location: globalThis.location, raf: globalThis.requestAnimationFrame };
  globalThis.window = { addEventListener: () => {} };
  globalThis.location = { search: "?perf=1", hash: "" };
  globalThis.requestAnimationFrame = () => 0;
  t.after(() => {
    globalThis.window = previous.window;
    globalThis.location = previous.location;
    globalThis.requestAnimationFrame = previous.raf;
  });
  assert.equal(perf.startPerfProbe(), true);
  perf.perfRequestSent(100, "playCard");
  perf.perfRequestSent(101, "endPhase");
  perf.perfRequestFailed(100);
  assert.deepEqual(window.__nonamePerf.interactions().map((entry) => entry.id), [101]);
});
