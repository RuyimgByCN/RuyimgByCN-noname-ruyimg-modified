// Actual generated WASM32/Worker converter, server expectations and native IPC host: same archives.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {spawn} from 'node:child_process';
import {createInterface} from 'node:readline';
import init, {NativeEngine} from '../src/worker/native-pkg/engine_native_wasm.js';
import {nativeObserverToSnapshot, nativeEnvelope} from '../src/worker/nativeBackend.ts';
import {nativeObserverToSnapshot as oldReader} from './approved-b-old-reader.ts';
const [path,binary] = process.argv.slice(2);
assert(path && binary,'fixture JSON and actual pa09_native_bridge binary required');
const fixtures=JSON.parse(readFileSync(path,'utf8'));
await init({module_or_path:readFileSync(new URL('../src/worker/native-pkg/engine_native_wasm_bg.wasm',import.meta.url))});
const host=spawn(binary,[],{stdio:['pipe','pipe','inherit']});
const lines=createInterface({input:host.stdout});const pending=[];let id=0;
lines.on('line',line=>{const waiter=pending.shift();if(waiter) waiter(JSON.parse(line));});
async function request(body){const response=await new Promise(resolve=>{pending.push(resolve);host.stdin.write(JSON.stringify({id:++id,...body})+'\n');});assert.equal(response.ok,true,JSON.stringify(response));return response.data;}
const seatFields=env=>env.view.seats.map(({generals,character_def})=>({generals,character_def}));
const choiceFields=(choice,server=false)=>choice===null?null:({choice_id:choice.choice_id,observer:choice.observer-(server?1:0),kind:choice.kind,prompt:choice.prompt,native_kind:choice.native_kind,min_count:choice.min_count,max_count:choice.max_count,allow_cancel:choice.allow_cancel,revision:choice.revision,options:choice.options.map(option=>({...option,label_key:server&&option.label_key.startsWith('player:')?`player:${Number(option.label_key.slice(7))-1}`:option.label_key}))});
const kinds=new Set();const prompts=new Set();let comparisons=0;
try{
 await request({type:'init'});
 for(const fixture of fixtures){
  const engine=new NativeEngine(fixture.seats.length,0n);
  try{
   for(let seat=0;seat<fixture.seats.length;seat++){
    await request({type:'start',players:fixture.seats.length,seed:0,observer:seat,packsJson:fixture.packsJson});
    for(let restore=0;restore<2;restore++){
     engine.restore_hex_with_packs_json(fixture.archiveHex,fixture.packsJson);
     const restoredHex=engine.save_hex();
     const ipc=await request({type:'restore',hex:fixture.archiveHex,packsJson:fixture.packsJson});
     const raw=JSON.parse(engine.view_json(seat));
     const current=nativeObserverToSnapshot(raw);const legacy=oldReader(raw);
     const withoutMetadata={...current,seats:current.seats.map(({generals,...seat})=>seat)};
     assert.deepEqual(legacy,withoutMetadata,`${fixture.name}/${seat}: functional old TS reader accepts actual new WASM payload`);
     const worker=nativeEnvelope({engine,observer:seat,eventCursor:0});
     const server=fixture.seats[seat];
     assert.deepEqual(seatFields(worker),seatFields(server),`${fixture.name}/${seat}: server/Worker both slots`);
     assert.deepEqual(seatFields(ipc),seatFields(worker),`${fixture.name}/${seat}: IPC/Worker both slots`);
     assert.deepEqual(choiceFields(worker.choice),choiceFields(server.choice,true),`${fixture.name}/${seat}: server/Worker all choice fields`);
     assert.deepEqual(choiceFields(ipc.choice),choiceFields(worker.choice),`${fixture.name}/${seat}: IPC/Worker all choice fields`);
     assert.equal(engine.save_hex(),restoredHex,'WASM metadata read changed archive bytes');
     assert.equal((await request({type:'save'})).hex,restoredHex,'IPC/Worker restored archives differ (both rotate auth generation once)');
     if(worker.choice){kinds.add(worker.choice.native_kind);prompts.add(worker.choice.prompt);}
     comparisons++;
    }
   }
   assert.equal(fixture.spectator.choice,null);
   for(const [index,publicSeat] of fixture.spectator.view.seats.entries())for(let slot=0;slot<2;slot++){
    const general=publicSeat.generals[slot];
    if(general)assert.deepEqual(general,fixture.referee.view.seats[index].generals[slot]);
   }
   console.log(`PASS ${fixture.name}: ${fixture.seats.length} seats, two restores, both slots/all choice fields, identical archive bytes`);
  }finally{engine.free();}
 }
 assert.deepEqual([...kinds].sort(),['confirm','pick_tag','respond_card','select_cards','select_players']);
 console.log(JSON.stringify({passed:true,fixtures:fixtures.length,comparisons,kinds:[...kinds].sort(),prompts:[...prompts].sort(),failed:0}));
}finally{host.stdin.end();host.kill('SIGTERM');}
