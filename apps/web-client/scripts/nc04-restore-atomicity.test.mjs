import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import init, { NativeEngine } from "../src/worker/native-pkg/engine_native_wasm.js";
import { nativeCreate, nativeDispatch, nativeEnvelope } from "../src/worker/nativeBackend.ts";

await init({ module_or_path: readFileSync(new URL("../src/worker/native-pkg/engine_native_wasm_bg.wasm", import.meta.url)) });

function archive(players, seed) {
  const engine = new NativeEngine(players, BigInt(seed));
  const hex = engine.save_hex();
  engine.free();
  return hex;
}

test("读入较少座位时，投影视图失败不替换当前对局", () => {
  const handle = nativeCreate(4, 4242, 3);
  const original = handle.engine;
  const before = original.save_hex();
  const incoming = archive(2, 17017);
  for (const extra of [{}, { packsJson: "[]" }]) {
    assert.throws(() => nativeDispatch(handle, { type: "restore", hex: incoming, ...extra }));
    assert.equal(handle.engine, original, "existing engine reference changed");
    assert.ok(original.save_hex() === before, `failed restore changed save: ${JSON.stringify(extra)}`);
    assert.equal(nativeEnvelope(handle).view.observer, 3);
  }
});

test("投影故障保留原局、缓存和事件游标，并释放候选引擎", () => {
  const handle = nativeCreate(4, 4242, 0);
  const original = handle.engine;
  original.set_planner(0, 50n);
  const before = original.save_hex();
  nativeEnvelope(handle);
  handle.eventCursor = 9;
  const legal = NativeEngine.prototype.legal_actions_json;
  const free = NativeEngine.prototype.free;
  let freed = 0;
  NativeEngine.prototype.legal_actions_json = () => { throw new Error("injected projection failure"); };
  NativeEngine.prototype.free = function () { freed++; return free.call(this); };
  try {
    assert.throws(() => nativeDispatch(handle, { type: "restore", hex: archive(4, 17017) }), /injected projection failure/);
  } finally {
    NativeEngine.prototype.legal_actions_json = legal;
    NativeEngine.prototype.free = free;
  }
  assert.equal(handle.engine, original);
  assert.ok(original.save_hex() === before, "projection failure changed old save");
  assert.equal(handle.eventCursor, 9);
  // archive() and the rejected candidate each own one engine.
  assert.equal(freed, 2);
  let queries = 0;
  original.legal_actions_json = (seat) => { queries++; return legal.call(original, seat); };
  assert.equal(nativeEnvelope(handle).view.observer, 0);
  assert.equal(queries, 0, "failed restore must keep the original action cache");
});

test("正式恢复失败时仍保留原局和游标", () => {
  const handle = nativeCreate(4, 4242, 0);
  const original = handle.engine;
  const before = original.save_hex();
  const incoming = archive(4, 17017);
  handle.eventCursor = 9;
  original.restore_hex = () => { throw new Error("injected commit failure"); };
  assert.throws(() => nativeDispatch(handle, { type: "restore", hex: incoming }), /injected commit failure/);
  assert.equal(handle.engine, original);
  assert.ok(original.save_hex() === before);
  assert.equal(handle.eventCursor, 9);
});

test("成功恢复保留原引擎引用并使权威动作缓存失效", () => {
  const handle = nativeCreate(4, 4242, 0);
  const original = handle.engine;
  let queries = 0;
  const legal = original.legal_actions_json.bind(original);
  original.legal_actions_json = (seat) => { queries++; return legal(seat); };
  nativeEnvelope(handle);
  assert.equal(queries, 1);
  const incoming = archive(4, 17017);
  const expected = new NativeEngine(1, 0n);
  expected.restore_hex(incoming);
  const expectedHex = expected.save_hex();
  expected.free();
  const result = nativeDispatch(handle, { type: "restore", hex: incoming });
  assert.equal(handle.engine, original);
  assert.equal(original.save_hex(), expectedHex);
  assert.equal(result.hash_hex, original.state_hash_hex());
  nativeEnvelope(handle);
  assert.equal(queries, 2, "restore must invalidate original handle cache");
});
