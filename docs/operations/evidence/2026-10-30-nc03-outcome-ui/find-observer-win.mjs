import fs from "node:fs";
import assert from "node:assert/strict";
import init, { NativeEngine } from "../../../../apps/web-client/src/worker/native-pkg/engine_native_wasm.js";
await init({module_or_path:fs.readFileSync(new URL("../../../../apps/web-client/src/worker/native-pkg/engine_native_wasm_bg.wasm",import.meta.url))});
const found=[];
const trials=[];
for (const mode of ["prototype","identity"]) {
  for (let seed=1;seed<=32;seed++) {
    const game=mode==="prototype"?new NativeEngine(4,BigInt(seed)):NativeEngine.new_identity(4,BigInt(seed));
    try {
      let steps=0;
      while(!game.is_finished()&&steps<2000){game.auto_step();steps++;}
      const view=JSON.parse(game.view_json(1));
      const role=view.players[1].identity;
      const won=mode==="prototype"?view.outcome?.SingleWin?.player===2:
        view.outcome&&(("RebelWin" in view.outcome&&role==="Rebel")||("LoyalistWin" in view.outcome&&["Lord","Loyalist"].includes(role))||("RenegadeWin" in view.outcome&&role==="Renegade"));
      trials.push({mode,seed,steps,role,outcome:view.outcome,won:!!won});
      if(won){found.push({mode,seed,steps,role,outcome:view.outcome});break;}
    }finally{game.free();}
  }
}
fs.writeFileSync(new URL("observer-win-seeds.json",import.meta.url),JSON.stringify({found,trials},null,2)+"\n");
assert(found.length>0, "no positive observer fixture found in the bounded seed search");
console.log(JSON.stringify(found));
