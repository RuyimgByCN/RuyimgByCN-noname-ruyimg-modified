// Replay theft/gift author packages through the real SDK store and actual web WASM.
// Usage: node --import ./scripts/ts-register.mjs scripts/adr0786-j2-theft-sdk-smoke.mjs <matrix.json>
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import init, { NativeEngine, validate_pack_json } from "../src/worker/native-pkg/engine_native_wasm.js";
import { activateExtension, deactivateExtension, importExtensionPackageValidated, uninstallExtension } from "../src/engine/extensionStore.ts";
import { prepareNativeMatchContent, restoreNativeMatchContent } from "../src/engine/nativeMatchContent.ts";

const memory = new Map();
globalThis.localStorage = { getItem:key => memory.get(key) ?? null, setItem:(key,value) => memory.set(key,String(value)), removeItem:key => memory.delete(key) };
await init({ module_or_path:readFileSync(new URL("../src/worker/native-pkg/engine_native_wasm_bg.wasm",import.meta.url)) });
const matrix = JSON.parse(readFileSync(process.argv[2],"utf8"));
const names = ["danshou-hand-theft-not-gift","danshou-equipment-theft-not-gift","fenghe-genuine-gift-benefit"];
function apply(engine,command) {
  switch (command.type) {
    case "activateSkill": return JSON.parse(engine.activate_skill(command.actor,command.skill));
    case "playCard": return JSON.parse(engine.play_card(command.actor,command.card,new Uint32Array(command.targets)));
    case "answer": return JSON.parse(engine.answer(command.actor,BigInt(command.choice_id),new Uint32Array(command.selections)));
    default: throw Error(`Unexpected command ${command.type}`);
  }
}
function replay(scenario,content) {
  const engine = new NativeEngine(4,0n);
  engine.restore_hex_with_packs_json(scenario.initial,content.packsJson);
  for (const step of scenario.steps) {
    const saved = engine.save_hex();
    const restored = new NativeEngine(4,0n);
    restored.restore_hex_with_packs_json(saved,content.packsJson);
    if (step.command.type === "answer") {
      for (const game of [engine,restored]) {
        const before = game.save_hex();
        let rejected = false;
        try { rejected = JSON.parse(game.answer(step.command.actor,BigInt(step.command.choice_id),new Uint32Array([999]))).outcome === "rejected"; }
        catch { rejected = true; }
        assert.ok(rejected);
        assert.equal(game.save_hex(),before,"failed retry preserves the frozen game");
      }
    }
    assert.equal(apply(engine,step.command).outcome,step.outcome);
    assert.equal(apply(restored,step.command).outcome,step.outcome);
    assert.equal(engine.save_hex(),step.archive,"SDK package path matches complete native archive");
    assert.equal(restored.save_hex(),step.resumed_archive,"SDK byte checkpoint matches native continuation");
    assert.equal(engine.state_hash_hex(),step.hash);
    restored.free();
  }
  const result={ name:scenario.name,steps:scenario.steps.length,hash:engine.state_hash_hex() };
  engine.free();
  return result;
}
const results=[];
for (const name of names) {
  const scenario=matrix.scenarios.find(s=>s.name===name);
  assert.ok(scenario,`missing ${name}`);
  const payload=JSON.parse(scenario.packs_json)[0];
  await importExtensionPackageValidated(JSON.stringify(payload),async json=>validate_pack_json(json));
  const key=`${payload.manifest.namespace}/${payload.manifest.package_id}`;
  activateExtension(key);
  const frozen=prepareNativeMatchContent();
  const direct=replay(scenario,frozen);
  deactivateExtension(key);
  uninstallExtension(key);
  assert.equal(prepareNativeMatchContent().contentLock.length,0);
  assert.deepEqual(replay(scenario,restoreNativeMatchContent(frozen)),direct,"frozen theft/gift game survives uninstall");
  results.push(direct);
}
console.log(JSON.stringify({ sdk:true,actualWasm:true,failedRetryAtomicity:true,frozenRestoreAfterUninstall:true,results },null,2));
