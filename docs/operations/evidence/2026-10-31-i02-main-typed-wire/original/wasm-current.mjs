import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
import crypto from 'node:crypto';
const [out,repo]=process.argv.slice(2);const pkg=repo+'/apps/web-client/src/worker/native-pkg';
const wasm=await import(pathToFileURL(pkg+'/engine_native_wasm.js'));await wasm.default({module_or_path:readFileSync(pkg+'/engine_native_wasm_bg.wasm')});
const backend=await import(pathToFileURL(repo+'/apps/web-client/src/worker/nativeBackend.ts'));const expected=JSON.parse(readFileSync(out+'/native-probe-results.json'));
const rows=[];
for(const name of ['park-0','park-1','own-cancel','shan-auto','shan-direct','shan-cancel','shan-pass']){
 const file=name==='own-cancel'?'park-0':name.startsWith('park')?name:'shan';const hex=readFileSync(out+'/'+file+'.ngsa').toString('hex');
 const e=new wasm.NativeEngine(3,0n);e.restore_hex_checked(hex);const restored=e.save_hex();const revision=e.revision();
 const own=JSON.parse(e.open_choice_json(1));const other=JSON.parse(e.open_choice_json(2));assert.equal(other.hidden,true);assert.equal(other.options,undefined);assert.ok(own.options.length);
 const ch=own;let reply;
 if(name.endsWith('direct')){const index=ch.options.findIndex(o=>o.card!==undefined);assert.ok(index>=0);reply=JSON.parse(e.answer(1,BigInt(ch.id),new Uint32Array([index])));}
 else if(name.endsWith('pass'))reply=JSON.parse(e.answer(1,BigInt(ch.id),new Uint32Array([ch.options.findIndex(o=>o==='pass')])));
 else if(name.endsWith('cancel'))reply=JSON.parse(e.answer(1,BigInt(ch.id),new Uint32Array([])));
 else reply=JSON.parse(e.auto_step());
 assert.deepEqual(reply,expected[name]?.reply??expected[name.replace('shan-','')].reply);
 const native=expected[name]??expected[name.replace('shan-','')];assert.equal(Number(e.revision()),native.revision);assert.equal(e.revision(),revision+(name==='shan-cancel'?0n:1n));assert.equal(e.state_hash_hex(),native.hash);assert.deepEqual(JSON.parse(e.public_event_codes_json(0)),native.codes);
 if(name==='shan-cancel'){assert.equal(reply.outcome,'rejected');assert.equal(reply.reason,'InvalidChoice');assert.equal(e.save_hex(),restored);}else assert.ok(['accepted','pending'].includes(reply.kind?JSON.parse(reply.outcome).outcome:reply.outcome));
 const actual=Buffer.from(e.save_hex(),'hex');assert.deepEqual(actual,readFileSync(out+'/native-'+name+'.ngsa'));writeFileSync(out+'/wasm-'+name+'.ngsa',actual);
 const d=new wasm.NativeEngine(3,0n);d.restore_hex_checked(hex);const h={engine:d,seed:0,players:3,observer:1,eventCursor:0};
 const request=name.endsWith('pass')?{type:'resolveChoice',actor:1,choiceId:ch.id,selections:[ch.options.findIndex(o=>o==='pass')],cancelled:false}:name.endsWith('direct')?{type:'resolveChoice',actor:1,choiceId:ch.id,selections:[ch.options.findIndex(o=>o.card!==undefined)],cancelled:false}:name.endsWith('cancel')?{type:'resolveChoice',actor:1,choiceId:ch.id,selections:[],cancelled:true}:{type:'autoStep'};
 const envelope=backend.nativeDispatch(h,request);assert.deepEqual(Buffer.from(d.save_hex(),'hex'),actual);assert.equal(envelope.hash_hex,native.hash);
 if(name.startsWith('park'))assert.equal(JSON.parse(reply.outcome).outcome,'pending');
 rows.push({name,own,other,reply,envelope,restoredSha:crypto.createHash('sha256').update(Buffer.from(restored,'hex')).digest('hex'),fullAfterSha:crypto.createHash('sha256').update(actual).digest('hex'),oneFormalSubmit:true,accepted:name!=='shan-cancel'});e.free();d.free();
}
writeFileSync(out+'/wasm-current-results.json',JSON.stringify({cases:rows.length,rows,fullPlanExported:false,qualification:'Full Plan privacy is native Rust proof; current web WASM exports auto_step summary only.'},null,2));console.log('PASS current Main web-WASM binding + Worker adapter: five genuine formal cases, full reply/hash/events/archive equality');
