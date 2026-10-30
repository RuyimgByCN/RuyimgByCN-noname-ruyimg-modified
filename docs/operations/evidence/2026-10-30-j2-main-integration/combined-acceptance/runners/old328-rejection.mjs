import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
import {resolve} from 'node:path';
const root=process.cwd();
const pkg=resolve(root,'apps/web-client/src/worker/native-pkg');
const {default:init,NativeEngine}=await import(pathToFileURL(resolve(pkg,'engine_native_wasm.js')));
await init({module_or_path:readFileSync(resolve(pkg,'engine_native_wasm_bg.wasm'))});
const vector=JSON.parse(readFileSync(resolve(root,'docs/operations/evidence/2026-10-30-pa09-desktop-native/recovery-empty-hujia-vector.json'),'utf8'));
const game=new NativeEngine(8,45000n);
try {
 const before=game.save_hex();let error=null;
 try{game.restore_hex(vector.archiveHex);}catch(e){error=String(e);}
 assert(error && error.includes('328') && error.includes('329'),`expected explicit328/329 rules rejection: ${error}`);
 assert.equal(game.save_hex(),before,'old-version rejection must preserve current game');
 const report={passed:true,actualWasm:true,oldRules:328,currentRules:329,error,currentArchiveUnchanged:true,fixtureFallbackUsed:false};
 writeFileSync(process.argv[2],JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
} finally {game.free();}
