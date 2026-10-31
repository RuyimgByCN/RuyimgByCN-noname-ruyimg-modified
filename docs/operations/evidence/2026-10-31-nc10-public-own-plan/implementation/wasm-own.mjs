import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
const root='/tmp/nc10-own-36992565';
const pkg=root+'/worktree/apps/web-client/src/worker/native-pkg';
const wasm=await import(pathToFileURL(pkg+'/engine_native_wasm.js'));
await wasm.default({module_or_path:readFileSync(pkg+'/engine_native_wasm_bg.wasm')});
const backend=await import(pathToFileURL(root+'/worktree/apps/web-client/src/worker/nativeBackend.ts'));
const rows=[];
for (const side of [0,1]) {
 const hex=readFileSync(root+`/evidence/final-cases/park-${side}.ngsa`).toString('hex');
 for (const transport of ['binding','worker-adapter']) {
  const engine=new wasm.NativeEngine(3,0n); engine.restore_hex_checked(hex);
  const before=engine.revision(); const choice=JSON.parse(engine.open_choice_json(1));
  assert.equal(choice.source_skill,'遗计'); assert.equal(choice.prompt,'trigger_confirm');
  let result;
  if(transport==='binding') result=JSON.parse(engine.auto_step());
  else result=backend.nativeDispatch({engine,seed:0,players:3,observer:1,eventCursor:0},{type:'autoStep'}).result;
  assert.equal(engine.revision(),before+1n);
  const nexts=[0,1,2].map(seat=>JSON.parse(engine.open_choice_json(seat)));
  const next=nexts.find(value=>value&&value.hidden!==true);
  assert.equal(next.prompt,side===0?'jiji_target':'jijie_give_target');
  rows.push({side,transport,before:String(before),after:String(engine.revision()),choice,nexts,result,public_codes:JSON.parse(engine.public_event_codes_json(0))});
  writeFileSync(root+`/evidence/wasm-${transport}-${side}-after.ngsa`,Buffer.from(engine.save_hex(),'hex'));
  engine.free();
 }
}
writeFileSync(root+'/evidence/wasm-own-results.json',JSON.stringify(rows,null,2)+'\n');
console.log(JSON.stringify({cases:rows.length,one_revision_per_submit:true,source:'遗计',next:['jiji_target','jijie_give_target']}));
