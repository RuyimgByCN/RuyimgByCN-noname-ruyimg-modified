// Affected rebuilt legacy-WASM sink only; no Snapshot restore API is added.
import assert from "node:assert/strict";
import {readFileSync, writeFileSync} from "node:fs";
import {pathToFileURL} from "node:url";
const pkg="/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-nc10-source27-pkix8fs6/evidence/p2-legacy-serde/legacy-wasm-pkg";
const {default:init,WasmEngine}=await import(pathToFileURL(`${pkg}/engine_wasm.js`).href);
await init({module_or_path:readFileSync(`${pkg}/engine_wasm_bg.wasm`)});
const engine=new WasmEngine();
let seen=0,steps=0;
try {
 const first=JSON.parse(engine.startGame(3,7));
 assert.ok(!first.choice||!Object.hasOwn(first.choice,"source_skill"));
 for(;steps<120&&!engine.isFinished();steps++){
  const choice=JSON.parse(engine.pendingChoiceJson());
  if(choice){assert.equal(Object.hasOwn(choice,"source_skill"),false);seen++;}
  const envelope=JSON.parse(engine.autoStep());
  assert.ok(!envelope.choice||!Object.hasOwn(envelope.choice,"source_skill"));
 }
 assert.ok(seen>0,"must observe actual pending legacy choices");
 const result={passed:true,steps,pendingChoicesObserved:seen,wasmRuntime:true,scope:"real generated WASM pendingChoiceJson and envelopes; typed poisoned restore covered by native-target sink test, not exposed in this JS API"};
 writeFileSync(new URL("legacy-wasm-sink-smoke.json",import.meta.url),JSON.stringify(result,null,2));
 console.log(JSON.stringify(result));
}finally{engine.free();}
