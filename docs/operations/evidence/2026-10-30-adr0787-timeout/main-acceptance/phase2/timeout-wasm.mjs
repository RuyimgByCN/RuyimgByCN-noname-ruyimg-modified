import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import assert from 'node:assert/strict';
const fixture=JSON.parse(fs.readFileSync(process.argv[2]));
const pkg=path.resolve('apps/web-client/src/worker/native-pkg');
const {initSync,NativeEngine}=await import(pathToFileURL(path.join(pkg,'engine_native_wasm.js')));
initSync({module:fs.readFileSync(path.join(pkg,'engine_native_wasm_bg.wasm'))});
const engine=new NativeEngine(4,4242n);engine.restore_hex(fixture.before_hex);
const outcome=JSON.parse(engine.answer(fixture.actor,BigInt(fixture.choice_id),new Uint32Array(fixture.tokens)));
assert.ok(['accepted','pending'].includes(outcome.outcome),JSON.stringify(outcome));
assert.equal(engine.save_hex(),fixture.after_hex,'all archive bytes including core/RNG/resolution/dedup must agree');
assert.equal(engine.state_hash_hex(),fixture.hash);
assert.deepEqual(JSON.parse(engine.public_event_codes_json(0)),fixture.codes);
const restored=new NativeEngine(4,1n);restored.restore_hex(engine.save_hex());
assert.equal(restored.state_hash_hex(),engine.state_hash_hex());
for(let seat=0;seat<4;seat++){assert.deepEqual(JSON.parse(restored.view_json(seat)),JSON.parse(engine.view_json(seat)));assert.deepEqual(JSON.parse(restored.open_choice_json(seat)),JSON.parse(engine.open_choice_json(seat)));}
fs.writeFileSync(process.argv[3],JSON.stringify({passed:true,outcome,fullArchiveBytes:fixture.after_hex.length/2,hash:fixture.hash,publicCodes:fixture.codes,restoredAllSeats:true},null,2));
