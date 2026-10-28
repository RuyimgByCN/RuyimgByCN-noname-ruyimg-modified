import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import init, { NativeEngine } from "../src/worker/native-pkg/engine_native_wasm.js";

await init({ module_or_path: readFileSync(new URL("../src/worker/native-pkg/engine_native_wasm_bg.wasm", import.meta.url)) });
const replies = [];
globalThis.self = { postMessage: (reply) => replies.push(reply) };
await import("../src/worker/engine.native.worker.ts");
let nextId = 0;
async function send(message) {
  const id = ++nextId;
  await globalThis.self.onmessage({ data: { ...message, id } });
  const reply = replies.find((item) => item.id === id);
  assert.ok(reply, `missing ${message.type} reply`);
  return reply;
}
async function save() {
  const reply = await send({ type: "save" });
  assert.equal(reply.ok, true, reply.error);
  return reply.data.hex;
}
async function startGame(seed) {
  const reply = await send({ type: "start", players: 4, seed, observer: 0 });
  assert.equal(reply.ok, true, reply.error);
}

test("新局视图失败时保留原局，合法新局仍可替换", async () => {
  assert.equal((await send({ type: "init" })).ok, true);
  await startGame(4242);
  const before = await save();
  for (const extra of [{}, { mode: "identity" }, { packsJson: "[]" }, { mode: "identity", packsJson: "[]" }]) {
    const reply = await send({ type: "start", players: 4, seed: 17017, observer: 4, ...extra });
    assert.equal(reply.ok, false, "invalid observer accepted");
    assert.equal(await save(), before, `failed ${JSON.stringify(extra)} start replaced game`);
    assert.equal((await send({ type: "observer" })).ok, true, "old observer stopped working");
  }
  await startGame(17017);
  const expected = new NativeEngine(4, 17017n);
  assert.notEqual(await save(), before);
  assert.equal(await save(), expected.save_hex());
  expected.free();
});

test("其他新局信封错误也保留原局并释放新引擎", async () => {
  await startGame(4242);
  const before = await save();
  const legal = NativeEngine.prototype.legal_actions_json;
  const free = NativeEngine.prototype.free;
  let freed = 0;
  NativeEngine.prototype.legal_actions_json = () => { throw new Error("injected projection failure"); };
  NativeEngine.prototype.free = function () { freed++; return free.call(this); };
  let reply;
  try {
    reply = await send({ type: "start", players: 4, seed: 17017, observer: 0 });
  } finally {
    NativeEngine.prototype.legal_actions_json = legal;
    NativeEngine.prototype.free = free;
  }
  assert.equal(reply.ok, false);
  assert.match(reply.error, /injected projection failure/);
  assert.equal(await save(), before);
  assert.equal(freed, 1, "failed candidate must be freed");
  assert.equal((await send({ type: "observer" })).ok, true);
});
