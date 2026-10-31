import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const out='/tmp/nc27-main-final-v3-8e1fad00';
const pkg='/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client/src/worker/native-pkg';
const fixture='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-nc10-source27-pkix8fs6/evidence/physical/v2/actual-native-ask.hex';
const hex=readFileSync(fixture,'utf8').trim();
const {default:init,NativeEngine}=await import(pathToFileURL(`${pkg}/engine_native_wasm.js`).href);
await init({module_or_path:readFileSync(`${pkg}/engine_native_wasm_bg.wasm`)});
const engine=new NativeEngine(3,0n);
try {
 engine.restore_hex(hex);
 const before=engine.save_hex();
 const raw={};
 for(let seat=0;seat<3;seat++) {
  raw[seat]=engine.open_choice_json(seat);
  const value=JSON.parse(raw[seat]);
  if(seat===1){assert.equal(value.source_skill,'遗计');assert.equal(value.prompt,'trigger_confirm');}
  else {assert.equal(Object.hasOwn(value,'source_skill'),false);assert.equal(value.hidden,true);}
 }
 assert.equal(engine.save_hex(),before);
 const result={passed:true,fixtureRestoreBytesIdentical:before===hex,restoreNormalization:'Engine::restore increments auth_generation; metadata-read invariance uses post-restore baseline',fixture,fixtureSha256:createHash('sha256').update(readFileSync(fixture)).digest('hex'),fixtureArchiveRuleVersion:Buffer.from(hex,'hex').readBigUInt64LE(6).toString(),raw,owner:1,source_skill:'遗计',otherHidden:true,saveBeforeAfterIdentical:true,wasmSha256:createHash('sha256').update(readFileSync(`${pkg}/engine_native_wasm_bg.wasm`)).digest('hex')};
 assert.equal(result.fixtureArchiveRuleVersion,'330');
 writeFileSync(`${out}/actual-native-source.json`,JSON.stringify(result,null,2)+'\n');
 console.log(JSON.stringify(result));
} finally {engine.free();}
