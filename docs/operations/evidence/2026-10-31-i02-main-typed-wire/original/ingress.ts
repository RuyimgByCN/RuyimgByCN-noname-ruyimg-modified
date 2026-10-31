import { EngineClient } from '/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client/src/engine/client.ts';
import { NetworkEngineClient } from '/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client/src/engine/networkClient.ts';
import { presentationMetadata, validatePresentationResponse } from '/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client/src/engine/presentationMetadata.ts';
import { applyEnvelope, createViewStore } from '/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client/src/engine/viewStore.ts';
import corpus from './ingress-fixtures.json';
const w:any=window;const rows:any[]=[];const check=(v:any,msg:string)=>{if(!v)throw Error(msg);};
const clone=(v:any)=>JSON.parse(JSON.stringify(v));const tauri=!!w.__TAURI_INTERNALS__;
let injection:any=null;let sab:SharedArrayBuffer|undefined;let actualRequests=0;let positiveAbsent=0;let candidateName="caocao";
const originalWorker=w.Worker;const callbacks=w.__TAURI_INTERNALS__?.callbacks;const originalCallbackSet=callbacks?.set;
const view=()=>({revision:2,state_hash:Array(32).fill(0),observer:'Spectator',round:1,current_player:1,phase:'Play',seats:[1,2].map(player=>({player,seat:player-1,hp:4,max_hp:4,alive:true,hand_count:0,identity:null,skills:[]})),known_cards:[],outcome:'InProgress'});
const envelope=(batch:any)=>({result:{outcome:'accepted'},events:[],presentation:batch,view:view(),choice:null,legal_actions:[],hash_hex:'00'.repeat(32),revision:2,finished:false});
function injected(request:any,fixture:any){return {id:request.id,ok:true,data:fixture};}
if(!tauri) w.Worker=class extends originalWorker {
 postMessage(request:any,...rest:any[]){
  if(request.type==='init') sab=request.latencySab;
  if(injection && request.type===injection.type){const use=injection;injection=null;const response=injected(request,use.data);
   if(use.sab){check(sab instanceof SharedArrayBuffer,'real init did not provide SAB');const raw=new TextEncoder().encode(JSON.stringify(response));const ints=new Int32Array(sab!,0,2);new Uint8Array(sab!,8,raw.length).set(raw);Atomics.store(ints,1,raw.length);Atomics.add(ints,0,1);}
   else queueMicrotask(()=>this.dispatchEvent(new MessageEvent('message',{data:response})));return;
  }
  actualRequests++;return super.postMessage(request,...rest);
 }
};
else callbacks.set=function(id:any,callback:any){
 return originalCallbackSet.call(this,id,(data:any)=>{actualRequests++;
  if(injection&&data?.ok===true&&Number.isInteger(data.id)){const use=injection;injection=null;return callback({...data,data:use.data});}
  return callback(data);
 });
};
async function positive(){
 const client:any=new EngineClient();await client.connect();const env:any=await client.start(3,4242,1,'[]');check(env.view.seats.length===3,'real start missing state');check(!Object.hasOwn(env,'presentation'),'default absence changed');positiveAbsent++;
 const store=createViewStore();applyEnvelope(store,env);check(store.revision===env.revision,'positive store application failed');return client;
}
async function badCase(name:string,batch:any,nested:boolean,compound:boolean,sabMode=false){
 const client=await positive();const env:any=envelope(batch);if(compound)env.view.seats[0].generals=[];
 const data=nested?{assignment:{seat:0,name:'caocao',revision:2,state_hash:'00'},envelope:env}:env;
 // In IPC, invoke reaches real Rust first and only its returned successful data is explicitly replaced.
 injection={type:nested?'chooseGeneral':'autoStep',data,sab:sabMode};const promise=nested?client.chooseGeneral(0,candidateName):client.autoStep();
 const pending=[...client.pending];const id=pending.at(-1)[0];let error:any;try{await promise;}catch(e){error=e;}
 check(/协议错误，操作结果未确认/.test(String(error)),name+' did not preserve uncertain error');check(client.pending.size===pending.length&&pending.every(([key,val]:any)=>client.pending.get(key)===val),name+' pending changed');
 const mark=w.__nonamePerf?.interactions().find((v:any)=>v.id===id);check(mark&&mark.roundtrip_ms===null,name+' perf ACK happened');
 const store=createViewStore();const before=JSON.stringify(store);let rejected=false;try{applyEnvelope(store,env);}catch{rejected=true;}check(rejected&&JSON.stringify(store)===before,name+' store mutated');
 client.dispose();check(client.pending.size===0,name+' dispose did not clean pending');rows.push({name,nested,compound,sab:sabMode,uncertain:true,pendingIdentityRetained:true,perfUnacknowledged:true,storeUnchanged:true,disposeClean:true});
}
try{
 localStorage.setItem('noname-next-perf','1');
 const c=await positive();check(c.getTransportKind()===(tauri?'desktop-native':'wasm-worker'),'wrong actual transport');const next:any=await c.autoStep();check(!Object.hasOwn(next,'presentation'),'actual autoStep default presence');const candidates=await c.generalCandidates();check(candidates.count>0,'actual candidates absent');candidateName=candidates.candidates[0].name;const chosen=await c.chooseGeneral(0,candidateName);check(chosen.envelope&&!Object.hasOwn(chosen.envelope,'presentation'),'actual nested absence');c.dispose();
 check(presentationMetadata(clone(corpus.valid),2,2).occurrences.length===7,'seven-event positive failed');
 const good=clone(corpus.valid);good.occurrences[0].event.voice.role.owner.repository='repo🐉';good.occurrences[0].event.effective.key='sha🐉';good.occurrences[0].event.voice.assets[0].reference.path='audio/die/role🐉.mp3';presentationMetadata(good,2,2);
 for(const {name,batch} of corpus.invalid)for(const nested of [false,true])await badCase(name,batch,nested,nested);
 await badCase('compound-direct-null',null,false,true);
 if(!tauri){check(w.crossOriginIsolated,'browser not isolated');await badCase('actual-SAB-null',null,false,false,true);}
 // Actual public parser/client implementation; raw matching fixture injection is explicitly not a server producer.
 const net:any=new NetworkEngineClient();net.roomId=7;net.playerId=1;net.connectionId=4;net.revision=1;const sent:any[]=[];net.send=(m:any)=>sent.push(m);
 const outcome=(batch:any,key:number,room=7,seq=9)=>JSON.stringify({type:'CommandOutcome',payload:{header:{protocol_version:6,room_id:room,conn_seq:seq,revision:2},idempotency_key:key,result:{Accepted:{new_revision:2,state_hash:Array(32).fill(0)}},view:view(),choice:null,legal_actions:[],public_event_codes:[3],presentation:batch}});
 let count=0;
 for(const {name,batch} of corpus.invalid){let rejects=0;const call={key:9,kind:'EndPhase',resolve(){throw Error('unexpected ACK');},reject(){rejects++;}};net.pending.set(9,call);const persisted={roomId:7,key:9,kind:'EndPhase',createdAt:1};net.persistedPending=persisted;sessionStorage.setItem('noname-next.pending-command',JSON.stringify(persisted));const storage=sessionStorage.getItem('noname-next.pending-command');net.handleMessage(outcome(batch,9));check(rejects===1&&net.pending.get(9)===call&&net.revision===1&&net.persistedPending===persisted&&sessionStorage.getItem('noname-next.pending-command')===storage,'network atomicity '+name);net.pending.clear();count++;}
 net.persistedPending=null;sessionStorage.removeItem('noname-next.pending-command');
 for(let i=0;i<8;i++){const p=net.endPhase();const req=sent.at(-1);const reject=p.catch((e:any)=>e);net.handleMessage(outcome(null,req.payload.idempotency_key));check(/未确认/.test(String(await reject)),'retained call not uncertain');}
 const pending=[...net.pending];const idem=net.idemSeq;const seq=net.connSeq;const storage=sessionStorage.getItem('noname-next.pending-command');const sentBefore=sent.length;let ninth:any;try{await net.endPhase();}catch(e){ninth=e;}check(/未提交/.test(String(ninth))&&sent.length===sentBefore&&net.idemSeq===idem&&net.connSeq===seq&&sessionStorage.getItem('noname-next.pending-command')===storage,'ninth allocated/sent');net.replayPending();check(sent.slice(sentBefore).every((m:any,i:number)=>m.payload.idempotency_key===pending[i][0]&&m.payload.replay),'replay keys changed');
 for(const [key] of pending){const raw=JSON.parse(outcome(undefined,key));delete raw.payload.presentation;net.handleMessage(JSON.stringify(raw));}check(net.pending.size===0&&net.persistedPending===null,'legacy absence failed to settle');net.dispose();
 await fetch('/report',{method:'POST',body:JSON.stringify({passed:true,runtime:tauri?'actual hidden Tauri/WK IPC + current Main JS':'actual Chromium production Worker/SAB + current Main JS',actualRequests,positiveAbsent,fixtureInjection:true,notRustProducer:true,sevenEventPositive:true,validAstralPositive:true,invalidCorpus:corpus.invalid.length,rows,networkAtomicCases:count,networkBound8Replay:true,normalAbsenceSuccess:true})});
}catch(error){await fetch('/report',{method:'POST',body:JSON.stringify({passed:false,error:String(error),stack:(error as any)?.stack,rows,actualRequests,positiveAbsent})});}
finally{w.Worker=originalWorker;if(tauri)callbacks.set=originalCallbackSet;}
