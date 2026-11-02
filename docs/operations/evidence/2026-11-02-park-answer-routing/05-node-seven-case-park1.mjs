import {readFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
const repo='/Users/ruyimg/Documents/Claude/sanguosha/noname-next';
const pkg=repo+'/apps/web-client/src/worker/native-pkg';
const wasm=await import(pathToFileURL(pkg+'/engine_native_wasm.js'));
await wasm.default({module_or_path:readFileSync(pkg+'/engine_native_wasm_bg.wasm')});
const fx='/private/tmp/noname-main36-product-host-20261101T070732Z/seven-legacy-20261101T091200Z/oracle-diagnostic-attempt1';
const e=new wasm.NativeEngine(3,2306n);
const out=[];
const step=(tag,fn)=>{try{const v=fn();out.push([tag,'OK',String(v).slice(0,140)]);return v;}catch(err){out.push([tag,'THROW',err?.message||String(err)]);throw err;}};
try{
step('restore-fixture',()=>e.restore_hex(readFileSync(fx+'/park-1.ngsa').toString('hex')));
step('autoStep-1',()=>e.auto_step());
const saved=step('save',()=>e.save_hex());
out.push(['saved==native-park-1', String(saved===readFileSync(fx+'/native-park-1.ngsa').toString('hex'))]);
step('restore-saved',()=>e.restore_hex(saved));
step('view',()=>e.view_json(1));
step('legal',()=>e.legal_actions_json(1));
step('choice',()=>e.open_choice_json(0));
step('hash',()=>e.state_hash_hex());
step('save2',()=>e.save_hex());
step('autoStep-2',()=>e.auto_step());
step('view2',()=>e.view_json(1));
const legal2=step('legal2',()=>e.legal_actions_json(1));
out.push(['legal2-actions', String(JSON.parse(legal2).actions.length)]);
step('choice2',()=>e.open_choice_json(1));
step('hash2',()=>e.state_hash_hex());
} catch(e){out.push(['FAILED',e?.message]);}
finally{try{e.free();out.push(['free','OK']);}catch(err){out.push(['free','THROW',err?.message]);}}
for(const row of out) console.log(row.join(' | '));
