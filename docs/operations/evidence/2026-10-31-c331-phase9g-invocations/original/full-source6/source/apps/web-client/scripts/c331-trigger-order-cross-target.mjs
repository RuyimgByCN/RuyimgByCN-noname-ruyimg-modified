// Actual current wasm32 + production adapter replay; all rule state and continuations are compared.
import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { nativeEnvelope } from "../src/worker/nativeBackend.ts";
const [input, output] = process.argv.slice(2).map(value => resolve(value));
const vector = JSON.parse(readFileSync(input, "utf8"));
const pkg = resolve("apps/web-client/src/worker/native-pkg");
const wasm = await import(pathToFileURL(resolve(pkg, "engine_native_wasm.js")).href);
await wasm.default({ module_or_path: readFileSync(resolve(pkg, "engine_native_wasm_bg.wasm")) });
function snapshot(engine) {
  return { archive: engine.save_hex(), hash: engine.state_hash_hex(),
    views: [0,1,2,3].map(seat => JSON.parse(engine.view_json(seat))),
    choices: [0,1,2,3].map(seat => JSON.parse(engine.open_choice_json(seat))),
    events: JSON.parse(engine.public_event_codes_json(0)) };
}
const engine = new wasm.NativeEngine(4,0n);
engine.restore_hex_with_packs_json(vector.initial,vector.packs_json);
assert.deepEqual(snapshot(engine),vector.before,"full native/WASM pending state and multi-seat privacy");
const handle = { engine, observer:0 };
const envelope = nativeEnvelope(handle);
assert.equal(envelope.choice.options.length,257);
assert.deepEqual(envelope.choice.options[256],{ token:256,label_key:"skill:c331_author_256" });
assert.equal(envelope.choice.kind,"OrderTriggers");
assert.equal(envelope.choice.native_kind,"pick_tag");
for (const [seat,id,tokens] of [[1,vector.choice_id,[256]],[0,vector.choice_id,[]],[0,vector.choice_id,[0,1]],[0,vector.choice_id,[257]]]) {
  let rejected = false;
  try { rejected = JSON.parse(engine.answer(seat,BigInt(id),Uint32Array.from(tokens))).outcome === "rejected"; } catch { rejected = true; }
  assert.ok(rejected); assert.deepEqual(snapshot(engine),vector.before);
}
assert.deepEqual(JSON.parse(engine.answer(0,BigInt(vector.choice_id),new Uint32Array([256]))),vector.outcome);
assert.deepEqual(snapshot(engine),vector.after,"full native/WASM answer state/events/RNG/queue/continuation");
const restored = new wasm.NativeEngine(4,0n);
restored.restore_hex_with_packs_json(vector.initial,vector.packs_json);
restored.answer(0,BigInt(vector.choice_id),new Uint32Array([256]));
assert.deepEqual(snapshot(restored),vector.after,"separate byte restore branch");
writeFileSync(output,JSON.stringify({ passed:true,choices:257,selectedToken:256,privacySeats:4,fullArchive:true,actualWasm:true,adapter:true },null,2));
engine.free(); restored.free();
