import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const [repo,out,referenceRoot]=process.argv.slice(2),pkg=join(repo,'apps/web-client/src/worker/native-pkg');
const sha=b=>createHash('sha256').update(b).digest('hex');
const wasm=await import(pathToFileURL(join(pkg,'engine_native_wasm.js')));await wasm.default({module_or_path:readFileSync(join(pkg,'engine_native_wasm_bg.wasm'))});
const backend=await import(pathToFileURL(join(repo,'apps/web-client/src/worker/nativeBackend.ts')));
const native=JSON.parse(readFileSync(join(out,'native-probe-results.json'))),cases={};
for(const name of ['park-0','park-1','own-cancel','shan-auto','shan-direct','shan-cancel','shan-pass']){
 const handle=backend.nativeCreate(3,2306,1);const start=backend.nativeEnvelope(handle);
 try{
  const fixture=name==='own-cancel'?'park-0':name.startsWith('park')?name:'shan';
  const baseline=backend.nativeDispatch(handle,{type:'restore',hex:readFileSync(join(out,fixture+'.ngsa')).toString('hex')});
  const choice=baseline.choice;assert.ok(choice);
  const command=name.endsWith('pass')?{type:'resolveChoice',actor:1,choiceId:choice.choice_id,selections:[choice.options.find(x=>x.label_key==='pass').token],cancelled:false}:name.endsWith('direct')?{type:'resolveChoice',actor:1,choiceId:choice.choice_id,selections:[choice.options.find(x=>x.label_key.startsWith('card:')).token],cancelled:false}:name.endsWith('cancel')?{type:'resolveChoice',actor:1,choiceId:choice.choice_id,selections:[],cancelled:true}:{type:'autoStep'};
  const next=backend.nativeDispatch(handle,command),saved=backend.nativeDispatch(handle,{type:'save'}),reference=native[name]??native[name.replace('shan-','')];
  assert.equal(next.hash_hex,reference.hash);assert.deepEqual(next.result,reference.reply);assert.deepEqual(Buffer.from(saved.hex,'hex'),readFileSync(join(out,'native-'+name+'.ngsa')));
  const restored=backend.nativeDispatch(handle,{type:'restore',hex:saved.hex});
  backend.nativeDispatch(handle,{type:'observer'});
  const atomicBaseline=backend.nativeDispatch(handle,{type:'observer'}),postRestoreSaved=backend.nativeDispatch(handle,{type:'save'});
  const before=Buffer.from(saved.hex,'hex'),after=Buffer.from(postRestoreSaved.hex,'hex');
  assert.equal(before.length,after.length);assert.deepEqual(before.subarray(0,22),after.subarray(0,22));
  assert.equal(before.toString('ascii',0,4),'NGSA');assert.equal(before.readUInt16LE(4),106);assert.equal(before.readBigUInt64LE(6),331n);assert.equal(Number(before.readBigUInt64LE(14))+54,before.length);
  const generations=[...readFileSync(join(referenceRoot,'snapshot-'+name+'.txt'),'utf8').matchAll(/auth_generation: (\d+),/g)];assert.equal(generations.length,1);
  const generation=BigInt(generations[0][1]);const changed=[];
  for(let i=22;i<before.length-32;i++)if(before[i]!==after[i])changed.push(i);
  assert.equal(changed.length,1);const authOffset=changed[0];assert.equal(before.readBigUInt64LE(authOffset),generation);assert.equal(after.readBigUInt64LE(authOffset),generation+1n);
  assert.deepEqual(before.subarray(22,authOffset),after.subarray(22,authOffset));assert.deepEqual(before.subarray(authOffset+8,-32),after.subarray(authOffset+8,-32));assert.notDeepEqual(before.subarray(-32),after.subarray(-32));
  let malformedError;try{backend.nativeDispatch(handle,{type:'restore',hex:'00'});}catch(error){malformedError=String(error);}
  assert.ok(malformedError);const afterMalformed=backend.nativeDispatch(handle,{type:'observer'}),afterMalformedSaved=backend.nativeDispatch(handle,{type:'save'});
  assert.deepEqual(afterMalformed,atomicBaseline);assert.deepEqual(afterMalformedSaved,postRestoreSaved);
  const continueCommand={type:'autoStep'},continued=backend.nativeDispatch(handle,continueCommand),continuedSaved=backend.nativeDispatch(handle,{type:'save'});
  const continuedResult=continued.result?.kind?JSON.parse(continued.result.outcome):continued.result;
  assert.equal(continued.revision,atomicBaseline.revision+1);assert.ok(['accepted','pending'].includes(continuedResult?.outcome),'continuation must be accepted or pending');assert.notEqual(continued.hash_hex,atomicBaseline.hash_hex);
  writeFileSync(join(out,'node-restored-'+name+'.ngsa'),after);writeFileSync(join(out,'node-continued-'+name+'.ngsa'),Buffer.from(continuedSaved.hex,'hex'));
  cases[name]={start,baseline,command,next,saved,restored,atomicBaseline,postRestoreSaved,malformedError,afterMalformed,afterMalformedSaved,continueCommand,continued,continuedSaved,restoreArchiveException:{generationBefore:String(generation),generationAfter:String(generation+1n),authOffset,checksumBytes:32,beforeSHA256:sha(before),afterSHA256:sha(after),allOtherArchiveBytesExact:true,checksumValidatedByActualRestore:true,decodedGenerationReference:join(referenceRoot,'snapshot-'+name+'.txt')}};
 }finally{handle.engine.free();}
}
writeFileSync(join(out,'seven-full-envelope-oracle.json'),JSON.stringify({scope:'Same current production nativeBackend and WASM across Node/browser; native Rust reply/hash/archive/decoded-generation references retained separately; no independent upstream oracle',cases},null,2)+'\n');
console.log('Seven full envelopes, exact native archives, restored-instance continuations and malformed-restore atomic baselines prepared');
