// Reproduce the J2 local author lifecycle with a freshly built web WASM package.
// Usage: node --import ./scripts/ts-register.mjs scripts/adr0786-j2-active-smoke.mjs <author-pack-dir>
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import init, { NativeEngine, validate_pack_json } from "../src/worker/native-pkg/engine_native_wasm.js";
import { activateExtension, deactivateExtension, importExtensionPackageValidated, rollbackExtension, uninstallExtension } from "../src/engine/extensionStore.ts";
import { prepareNativeMatchContent, restoreNativeMatchContent } from "../src/engine/nativeMatchContent.ts";

const directory = resolve(process.argv[2]);
const memory = new Map();
globalThis.localStorage = { getItem:key => memory.get(key) ?? null, setItem:(key,value) => memory.set(key,String(value)), removeItem:key => memory.delete(key) };
await init({ module_or_path: readFileSync(new URL("../src/worker/native-pkg/engine_native_wasm_bg.wasm", import.meta.url)) });
const payload = version => ({ ...JSON.parse(readFileSync(resolve(directory,version,"package.npack.json"),"utf8")), characters:JSON.parse(readFileSync(resolve(directory,version,"character.json"),"utf8")) });
const key = "official/j2-author";
async function install(version) {
  await importExtensionPackageValidated(JSON.stringify(payload(version)), async json => validate_pack_json(json));
}
function start(content) {
  const e = NativeEngine.new_with_packs_json(4,99107867n,content.packsJson);
  e.assign_general(0,"j2-author");
  for (let i=0;i<10 && JSON.parse(e.view_json(0)).phase.toLowerCase() !== "play";i++) e.next_phase(0);
  const active = JSON.parse(e.legal_actions_json(0)).actions.find(a=>a.ActivateSkill)?.ActivateSkill;
  assert.ok(active,"author active handler is a real legal action");
  assert.equal(JSON.parse(e.activate_skill(0,active.skill)).outcome,"pending");
  return e;
}
function finish(e, content) {
  const initial = JSON.parse(e.view_json(0)).players[0].hand.own.length;
  let choices=0;
  for (let i=0;i<10;i++) {
    const choice = JSON.parse(e.open_choice_json(0));
    if (!choice) return { delta:JSON.parse(e.view_json(0)).players[0].hand.own.length-initial, hash:e.state_hash_hex(), choices };
    choices++;
    const selections = choice.prompt === "rvm_danshou_option" ? [choice.options.findIndex(o=>o.tag===4)]
      : choice.prompt === "rvm_danshou_target" ? [choice.options.findIndex(o=>o.player===0)]
      : Array.from({length:choice.min},(_,index)=>index);
    const saved = e.save_hex();
    const restored = new NativeEngine(4,0n);
    restored.restore_hex_with_packs_json(saved,content.packsJson);
    assert.equal(restored.state_hash_hex(),e.state_hash_hex());
    assert.deepEqual(JSON.parse(restored.open_choice_json(0)),choice);
    const originalEvents = JSON.parse(e.public_event_codes_json()).length;
    const restoredEvents = JSON.parse(restored.public_event_codes_json()).length;
    for (const engine of [e,restored]) assert.notEqual(JSON.parse(engine.answer(0,BigInt(choice.id),new Uint32Array(selections))).outcome,"rejected");
    assert.equal(restored.state_hash_hex(),e.state_hash_hex(),"restored continuation matches uninterrupted state");
    assert.deepEqual(JSON.parse(restored.public_event_codes_json()).slice(restoredEvents),JSON.parse(e.public_event_codes_json()).slice(originalEvents));
    restored.free();
  }
  throw Error("J2 settlement did not finish");
}
await install("0.1.0");
activateExtension(key);
const frozen = prepareNativeMatchContent();
const old = start(frozen);
const save = old.save_hex();
const v1 = finish(old,frozen);
assert.equal(v1.delta,1);
await install("0.1.1");
const current = prepareNativeMatchContent();
const v2 = finish(start(current),current);
assert.equal(v2.delta,2);
const restored = new NativeEngine(4,0n);
restored.restore_hex_with_packs_json(save,restoreNativeMatchContent(frozen).packsJson);
assert.deepEqual(finish(restored,frozen),v1);
rollbackExtension(key,"0.1.0");
assert.equal(finish(start(prepareNativeMatchContent()),frozen).delta,1);
deactivateExtension(key);
assert.deepEqual(prepareNativeMatchContent().contentLock,[]);
uninstallExtension(key);
assert.deepEqual(prepareNativeMatchContent().contentLock,[]);
const afterRemoval = new NativeEngine(4,0n);
afterRemoval.restore_hex_with_packs_json(save,restoreNativeMatchContent(frozen).packsJson);
assert.deepEqual(finish(afterRemoval,frozen),v1);
console.log(JSON.stringify({ lifecycle:"install/enable/start/activate/answer/checkpoint/upgrade/frozen-restore/rollback/disable/uninstall", v1, v2, wasm:"fresh web target" },null,2));
