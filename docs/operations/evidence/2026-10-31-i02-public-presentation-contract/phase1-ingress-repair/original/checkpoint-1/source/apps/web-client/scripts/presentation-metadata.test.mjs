import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import ts from "typescript";
import { presentationMetadata, validatePresentationResponse } from "../src/engine/presentationMetadata.ts";
import { NetworkEngineClient } from "../src/engine/networkClient.ts";
import { DesktopEngineTransport } from "../src/engine/desktopEngineTransport.ts";
import { applyEnvelope, createViewStore } from "../src/engine/viewStore.ts";
import { presentationBatch, invalidPresentationBatches, presentationView, presentationEnvelope } from "./presentation-fixtures.mjs";

async function frozenReader(name) {
  const source = readFileSync(new URL(`./fixtures/i02-main5fc/${name}.ts`, import.meta.url), "utf8");
  const relocated = source.replace(/from "(\.\.?\/[^"\n]+)"/g, (_, path) => `from "${new URL(path + ".ts", new URL("../src/engine/", import.meta.url)).href}"`);
  const resolved = relocated.replace('from "@tauri-apps/api/core"', `from "${import.meta.resolve("@tauri-apps/api/core")}"`);
  const js = ts.transpileModule(resolved, {compilerOptions:{module:ts.ModuleKind.ESNext,target:ts.ScriptTarget.ES2022}}).outputText;
  return import(`data:text/javascript;base64,${Buffer.from(js).toString("base64")}`);
}

function network(t, Client = NetworkEngineClient) {
  const previousWindow = globalThis.window;
  const previousStorage = globalThis.sessionStorage;
  const storage = new Map();
  globalThis.sessionStorage = {getItem:key=>storage.get(key) ?? null,setItem:(key,value)=>storage.set(key,value),removeItem:key=>storage.delete(key)};
  globalThis.window = { clearTimeout(){}, clearInterval(){}, setInterval(){return 1;} };
  const client = new Client();
  t.after(() => { client.dispose(); globalThis.window = previousWindow; globalThis.sessionStorage = previousStorage; });
  client.revision = 1; client.roomId = 7;
  const envelopes = []; client.onEnvelope(e => envelopes.push(e));
  return {client, envelopes, storage};
}
function outcome(batch, key = 9) {
  return JSON.stringify({type:"CommandOutcome",payload:{header:{protocol_version:6,room_id:7,conn_seq:9,revision:2}, idempotency_key:key,
    result:{Accepted:{new_revision:2,state_hash:Array(32).fill(0)}},view:presentationView(),choice:null,legal_actions:[],public_event_codes:[3],presentation:batch}});
}

test("actual frozen Main5fc TS network and desktop readers accept the optional response", async t => {
  const {NetworkEngineClient: OldClient} = await frozenReader("networkClient");
  const f=network(t,OldClient); f.client.handleMessage(outcome(presentationBatch()));
  assert.equal(f.envelopes.length,1); assert.equal(f.envelopes[0].presentation,undefined);
  const {DesktopEngineTransport: OldDesktop} = await frozenReader("desktopEngineTransport");
  const old = new OldDesktop(async command => command === "native_engine_open" ? 1 : {id:1,ok:true,data:presentationEnvelope()});
  const response = await new Promise(resolve => {old.onmessage=({data})=>resolve(data);old.postMessage({id:1,type:"autoStep"});});
  assert.equal(response.ok,true); assert.deepEqual(response.data.presentation,presentationBatch());old.terminate();
});

test("new parser accepts all seven events and legacy absence without altering autoStep strings", t => {
  assert.equal(presentationMetadata(undefined,2,2),undefined);
  assert.deepEqual(presentationMetadata(presentationBatch(),2,2),presentationBatch());
  const f=network(t);f.client.handleMessage(outcome(presentationBatch()));
  assert.deepEqual(f.envelopes[0].presentation,presentationBatch());
  const env=presentationEnvelope();env.result={kind:"answer",outcome:'{"outcome":"accepted","revision":2}'};
  const before=JSON.stringify(env.result);validatePresentationResponse(env);assert.equal(JSON.stringify(env.result),before);
  delete env.presentation;validatePresentationResponse(env);assert.equal(Object.hasOwn(env,"presentation"),false);
});

for (const {name,batch} of invalidPresentationBatches()) {
  test(`bad present ${name} is atomic in actual network/store ingress and IPC pass-through`, async t => {
    const f=network(t); const errors=[]; const unrelated=[];
    const pending={resolve(){throw new Error("must not ACK");},reject:e=>errors.push(e)};
    const other={resolve(){},reject:e=>unrelated.push(e)};
    f.client.pending.set(9,pending); f.client.pending.set(10,other);
    const persisted={roomId:7,key:9,kind:"EndPhase",createdAt:1}; f.client.persistedPending=persisted;
    const beforePending=[...f.client.pending];const beforeChoice=f.client.choice;
    f.storage.set("noname-next.pending-command",JSON.stringify(persisted));const beforeStorage=[...f.storage];
    f.client.handleMessage(outcome(batch));
    assert.equal(f.client.revision,1);assert.equal(f.client.choice,beforeChoice);assert.deepEqual([...f.client.pending],beforePending);
    assert.deepEqual([...f.storage],beforeStorage);assert.equal(f.client.persistedPending,persisted);assert.equal(f.envelopes.length,0);assert.equal(errors.length,1);assert.equal(unrelated.length,0);
    assert.match(errors[0].message,/结果未确认/);
    const store=createViewStore();const before=JSON.stringify(store);
    assert.throws(()=>applyEnvelope(store,presentationEnvelope(batch)),/presentation metadata invalid/);assert.equal(JSON.stringify(store),before);
    for (const nested of [false,true]) {
      const envelope=presentationEnvelope(batch);
      const data=nested?{assignment:{seat:0,name:"liubei",revision:2,state_hash:"00"},envelope}:envelope;
      const desktop=new DesktopEngineTransport(async command=>command==="native_engine_open"?1:{id:1,ok:true,data});
      const reply=await new Promise(resolve=>{desktop.onmessage=({data})=>resolve(data);desktop.postMessage({id:1,type:nested?"chooseGeneral":"autoStep",seat:0,name:"liubei"});});
      assert.equal(reply.ok,true);
      assert.throws(()=>validatePresentationResponse(reply.data),/presentation metadata invalid/);
      desktop.terminate();
    }
    f.client.dispose();assert.equal(f.client.pending.size,0);
  });
}

test("unmatched malformed broadcast does not reject another call or mutate receipt storage", t => {
  const f=network(t);let rejected=0;f.client.pending.set(10,{resolve(){},reject(){rejected++;}});
  const old=console.error;const reports=[];console.error=e=>reports.push(e);t.after(()=>{console.error=old;});
  f.client.handleMessage(outcome(null,0));assert.equal(rejected,0);assert.equal(f.client.pending.size,1);assert.equal(f.client.revision,1);assert.equal(reports.length,1);
});

test("bounds reject null, >1024 occurrences, >64KiB and typed binding mismatches", () => {
  assert.throws(()=>presentationMetadata(null,2,2));
  const b=presentationBatch();b.occurrences=Array.from({length:1025},(_,ordinal)=>({ordinal,event:{kind:"hp_recovered",actor:1,amount:1}}));assert.throws(()=>presentationMetadata(b,2,2));
  const big=presentationBatch();const voice=big.occurrences[0].event.voice;big.occurrences=Array.from({length:90},(_,ordinal)=>({ordinal,event:{kind:"player_died",actor:1,killer:null,voice}}));assert.throws(()=>presentationMetadata(big,2,2));
  const p=presentationBatch();const role=p.occurrences[0].event.voice.role;
  const owner={kind:"package",namespace:"author",package_id:"voice",version:"1.0.0",logic_hash:"4".repeat(64),content_hash:"5".repeat(64)};
  role.owner=owner;role.local_id=42;presentationMetadata(p,2,2);
  role.local_id={pack:"standard",raw_symbol:"liubei"};assert.throws(()=>presentationMetadata(p,2,2));
});

test("new metadata keeps the existing WS256KiB limit before receipt mutation", t => {
  const f=network(t);const errors=[];const pending={resolve(){},reject:e=>errors.push(e)};f.client.pending.set(9,pending);
  const message=JSON.parse(outcome(presentationBatch()));message.payload.padding="x".repeat(256*1024);
  f.client.handleMessage(JSON.stringify(message));assert.equal(errors.length,1);assert.equal(f.client.pending.get(9),pending);assert.equal(f.client.revision,1);assert.equal(f.envelopes.length,0);
});

test("compat m4a and zero SDK general/skill ids retain concrete leaf semantics", () => {
  const b=presentationBatch();const a=b.occurrences[0].event.voice.assets[0];
  a.reference={kind:"original",path:"audio-declared/key/role/die/mugyu.m4a",source_category:"compat",source_path:"audio/skill/mugyu.m4a"};
  presentationMetadata(b,2,2);
  const role=b.occurrences[0].event.voice.role;role.owner={kind:"package",namespace:"author",package_id:"voice",version:"1",logic_hash:"1".repeat(64),content_hash:"2".repeat(64)};role.local_id=0;
  presentationMetadata(b,2,2);
  const reversed=Object.fromEntries(Object.entries(a).reverse());b.occurrences[0].event.voice.assets.push(reversed);
  assert.throws(()=>presentationMetadata(b,2,2));
});

test("malformed conn_seq0 broadcast with a colliding key cannot reject a pending call", t => {
  const f=network(t);let rejected=0;f.client.pending.set(9,{resolve(){},reject(){rejected++;}});
  const message=JSON.parse(outcome(null,9));message.payload.header.conn_seq=0;
  const old=console.error;console.error=()=>{};t.after(()=>{console.error=old;});
  f.client.handleMessage(JSON.stringify(message));assert.equal(rejected,0);assert.equal(f.client.pending.size,1);assert.equal(f.client.revision,1);
});

test("network sequential malformed outcomes retain at most eight original intents and replay no fresh keys", async t => {
  const f=network(t);f.client.playerId=1;f.client.connectionId=4;
  const sent=[];f.client.send=message=>sent.push(message);
  for(let i=0;i<8;i++) {
    const promise=f.client.endPhase();const request=sent.at(-1);
    const rejected=assert.rejects(promise,/结果未确认/);
    f.client.handleMessage(outcome(null,request.payload.idempotency_key));await rejected;
  }
  const pending=[...f.client.pending];const storage=[...f.storage];const idem=f.client.idemSeq;const seq=f.client.connSeq;
  const ninth=f.client.endPhase();const rejected=assert.rejects(ninth,/not submitted|未提交/);
  if (sent.length>8) { // Bound the old broken implementation's otherwise unresolved promise for red evidence.
    const request=sent.at(-1);f.client.handleMessage(outcome(null,request.payload.idempotency_key));
  }
  await rejected;
  assert.deepEqual([...f.client.pending],pending);assert.deepEqual([...f.storage],storage);
  assert.equal(f.client.idemSeq,idem);assert.equal(f.client.connSeq,seq);assert.equal(sent.length,8);
  const keys=pending.map(([key])=>key);f.client.replayPending();
  assert.equal(f.client.pending.size,8);assert.deepEqual(sent.slice(8).map(m=>m.payload.idempotency_key),keys);
  assert.ok(sent.slice(8).every(m=>m.payload.replay));assert.equal(f.client.idemSeq,idem);
  f.client.dispose();assert.equal(f.client.pending.size,0);
});

test("network full retained map cannot admit a missing persisted recovery entry", t => {
  const f=network(t);f.client.playerId=1;const sent=[];f.client.send=message=>sent.push(message);
  for(let i=1;i<=8;i++) f.client.pending.set(i,{key:i,kind:"EndPhase",resolve(){},reject(){}});
  const persisted={roomId:7,key:99,kind:"EndPhase",createdAt:1};f.client.persistedPending=persisted;
  f.storage.set("noname-next.pending-command",JSON.stringify(persisted));const storage=[...f.storage];
  f.client.replayPending();assert.equal(f.client.pending.size,8);
  assert.deepEqual(sent.map(m=>m.payload.idempotency_key),[1,2,3,4,5,6,7,8]);
  assert.deepEqual([...f.storage],storage);assert.equal(f.client.persistedPending,persisted);
});

test("network legacy successful absence releases capacity and creation does not allocate an intent", async t => {
  const f=network(t);f.client.playerId=1;f.client.connectionId=5;const sent=[];f.client.send=m=>sent.push(m);
  const calls=Array.from({length:8},()=>f.client.endPhase());
  calls.forEach(p=>p.catch(()=>{})); // Observe earlier calls even if the capacity assertion fails.
  const rejected=assert.rejects(f.client.endPhase(),/未提交/);
  if(sent.length>8) {const m=sent.at(-1);f.client.handleMessage(outcome(null,m.payload.idempotency_key));}
  await rejected;
  for(const request of sent.slice(0,8)) {const message=JSON.parse(outcome(undefined,request.payload.idempotency_key));delete message.payload.presentation;f.client.handleMessage(JSON.stringify(message));}
  await Promise.all(calls);assert.equal(f.client.pending.size,0);
  const idem=f.client.idemSeq;const joined=f.client.createRoom(2,1);assert.equal(f.client.pending.size,0);assert.equal(f.client.idemSeq,idem);
  assert.equal(sent.at(-1).type,"CreateRoom");f.client.joinWaiter.resolve({roomId:7});f.client.joinWaiter=null;await joined;
});

test("every network product command refuses ninth admission without allocating, sending or replacing storage", async t => {
  const f=network(t);f.client.playerId=1;const sent=[];f.client.send=m=>sent.push(m);
  for(let i=1;i<=8;i++) f.client.pending.set(i,{key:i,kind:"EndPhase",resolve(){},reject(){}});
  const before=[...f.client.pending];const storage=[...f.storage];const idem=f.client.idemSeq;const seq=f.client.connSeq;
  const calls=[()=>f.client.endPhase(),()=>f.client.playCard(1,2,[2]),()=>f.client.activateSkill(1,2,[2],[]),
    ()=>f.client.useSkillCard(1,2,3,[2],0),()=>f.client.resolveChoice(1,2,[0]),()=>f.client.surrender()];
  for(const call of calls) await assert.rejects(call(),error=>/未提交/.test(error.message)&&!/未知|未确认/.test(error.message));
  assert.deepEqual([...f.client.pending],before);assert.deepEqual([...f.storage],storage);assert.equal(f.client.idemSeq,idem);assert.equal(f.client.connSeq,seq);assert.equal(sent.length,0);
});

test("network rejoin replays exactly eight retained keys and legacy replies clear them", async t => {
  const f=network(t);f.client.playerId=1;f.client.connectionId=4;const sent=[];f.client.send=m=>sent.push(m);
  for(let i=0;i<8;i++) {const p=f.client.endPhase();const request=sent.at(-1);const rejection=assert.rejects(p,/未确认/);f.client.handleMessage(outcome(null,request.payload.idempotency_key));await rejection;}
  const keys=[...f.client.pending.keys()];const idem=f.client.idemSeq;
  f.client.session={role:"Player",roomId:7,reconnectToken:"token"};
  const joined=f.client.rejoinRoom();assert.equal(sent.at(-1).type,"JoinRoom");const offset=sent.length;
  f.client.handleMessage(JSON.stringify({type:"JoinOk",payload:{room_id:7,player_id:1,seat:0,role:"Player",reconnect_token:"token",view:presentationView(1),choice:null,legal_actions:[]}}));
  await joined;
  assert.deepEqual(sent.slice(offset).filter(m=>m.type==="SubmitCommand").map(m=>m.payload.idempotency_key),keys);
  assert.equal(f.client.pending.size,8);assert.equal(f.client.idemSeq,idem);
  for(const key of keys) {const message=JSON.parse(outcome(undefined,key));delete message.payload.presentation;f.client.handleMessage(JSON.stringify(message));}
  assert.equal(f.client.pending.size,0);assert.equal(f.client.persistedPending,null);assert.equal(f.storage.size,0);
});

test("network malformed broadcast and wrong-room reply leave retained call unchanged", t => {
  const f=network(t);let rejected=0;const intent={key:9,kind:"EndPhase",resolve(){},reject(){rejected++;}};f.client.pending.set(9,intent);
  const old=console.error;const reports=[];console.error=e=>reports.push(e);t.after(()=>{console.error=old;});
  for(const room of [8,0]) {const message=JSON.parse(outcome(null,9));message.payload.header.room_id=room;f.client.handleMessage(JSON.stringify(message));}
  const broadcast=JSON.parse(outcome(null,9));broadcast.payload.header.conn_seq=0;f.client.handleMessage(JSON.stringify(broadcast));
  assert.equal(rejected,0);assert.equal(f.client.pending.get(9),intent);assert.equal(f.client.revision,1);assert.equal(reports.length,3);
});
