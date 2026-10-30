import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
const root=path.resolve(process.argv[2]);
const binary=process.argv[3];
const mode=process.argv[4] ?? 'current';
fs.mkdirSync(root,{recursive:true});
const trace=[];
const pause=ms=>new Promise(r=>setTimeout(r,ms));
const version=mode==='baseline'?5:6;
const bundle={engine_api:1,rule_vm_abi:167,content_schema:3,network_protocol:version,replay_schema:1,snapshot_schema:1,package_format:1,engine_build_id:'a330-proof'};
let server;
let port=17887;
function start(){
 const out=fs.openSync(path.join(root,'server.log'),'a');
 server=spawn(binary,[],{cwd:process.cwd(),env:{...process.env,GAME_SERVER_ADDR:`127.0.0.1:${port}`,NONAME_NATIVE_ROOMS:'1',NONAME_NATIVE_ROOM_PACKAGES:'',NONAME_NATIVE_PACKAGES_ROOT:path.join(root,'packages'),NONAME_NATIVE_STORE_ROOT:path.join(root,'native-store'),NONAME_NATIVE_ROOM_STORE:path.join(root,'rooms'),NONAME_ORIGINAL_CONTENT_ROOT:path.join(root,'original'),NONAME_CONTENT_STORE_ROOT:path.join(root,'legacy-store'),NONAME_EXTENSIONS_ROOT:path.join(root,'extensions')},stdio:['ignore',out,out]});
 fs.closeSync(out);
}
async function stop(){ if(server && server.exitCode===null && server.signalCode===null){server.kill('SIGKILL');await new Promise(r=>server.once('exit',r));} }
class Peer {
 constructor(ws){this.ws=ws;this.queue=[];this.seq=0;ws.onmessage=e=>{const m=JSON.parse(e.data);trace.push({direction:'in',type:m.type,payload:m.payload});this.queue.push(m);};}
 static async open(hello=true){
  const ws=new WebSocket(`ws://127.0.0.1:${port}/ws`);const p=new Peer(ws);
  await new Promise((resolve,reject)=>{ws.onopen=resolve;ws.onerror=reject;});
  const welcome=await p.until(m=>m.type==='Welcome'); assert.equal(welcome.payload.protocol_version,version);
  if(hello)p.send('Hello',{versions:bundle,display_name:'proof'});
  return p;
 }
 send(type,payload){trace.push({direction:'out',type,payload});this.ws.send(JSON.stringify({type,payload}));}
 async until(predicate,ms=15000){const start=Date.now();while(Date.now()-start<ms){const i=this.queue.findIndex(predicate);if(i>=0)return this.queue.splice(i,1)[0];await pause(10);}throw Error('WS timeout '+predicate.toString());}
 async join(room,seat,token=null){this.send('JoinRoom',{room_id:room,password:'',preferred_seat:seat,reconnect_token:token});const m=await this.until(m=>m.type==='JoinOk');this.room=room;this.seat=m.payload.seat;this.player=m.payload.player_id;this.token=m.payload.reconnect_token;return m.payload;}
 async snapshot(){this.send('RequestSnapshot',{header:{protocol_version:version,room_id:this.room,conn_seq:++this.seq,revision:0},last_revision:0});return (await this.until(m=>m.type==='Snapshot')).payload;}
 async submit(kind,rev,key=++this.seq){this.send('SubmitCommand',{header:{protocol_version:version,room_id:this.room,conn_seq:this.seq,revision:rev},command:{actor:this.player,expected_revision:rev,idempotency_key:key,kind},idempotency_key:key,replay:false});return (await this.until(m=>m.type==='CommandOutcome' && m.payload.header.conn_seq===this.seq)).payload;}
 close(){this.ws.close();}
}
async function ready(){for(let i=0;i<100;i++){try{return await Peer.open();}catch{if(server.exitCode!==null)throw Error('server exited '+server.exitCode);await pause(30);}}throw Error('server readiness');}
function record(room){return JSON.parse(fs.readFileSync(path.join(root,'rooms',`${room}.json`),'utf8'));}
try{
 for(const d of ['packages','original','extensions'])fs.mkdirSync(path.join(root,d),{recursive:true});
 start();const host=await ready();
 if(mode==='baseline'){
  host.send('CreateRoom',{players:4,seed:4242,password:''});const joined=(await host.until(m=>m.type==='JoinOk')).payload;
  fs.writeFileSync(path.join(root,'actual329-room.json'),JSON.stringify(record(joined.room_id),null,2));
  host.close();await stop();
  fs.writeFileSync(path.join(root,'report.json'),JSON.stringify({baseline:true,room:joined.room_id,source:'actual preserved 329 server binary'},null,2));
 }else{
  const unhello=await Peer.open(false);unhello.send('CreateRoom',{players:4,seed:4242,password:'',choice_timeout_secs:10});assert.equal((await unhello.until(m=>m.type==='Error')).payload.code,'VersionMismatch');
  unhello.send('Hello',{versions:{...bundle,network_protocol:0},display_name:'zero'});assert.equal((await unhello.until(m=>m.type==='Error')).payload.code,'VersionMismatch');unhello.send('Hello',{versions:{...bundle,network_protocol:5},display_name:'old'});assert.equal((await unhello.until(m=>m.type==='Error')).payload.code,'VersionMismatch');unhello.close();
  host.send('CreateRoom',{players:4,seed:4242,password:'',choice_timeout_secs:11});assert.equal((await host.until(m=>m.type==='Error')).payload.code,'BadMessage');
  host.send('CreateRoom',{players:4,seed:4242,password:'',choice_timeout_secs:10});const joined=(await host.until(m=>m.type==='JoinOk')).payload;
  host.room=joined.room_id;host.player=joined.player_id;host.seat=joined.seat;host.token=joined.reconnect_token;
  const peers=[host];for(let seat=1;seat<4;seat++){const p=await Peer.open();await p.join(host.room,seat);peers.push(p);}
  let state;let owner;
  for(let step=0;step<1000;step++){
   state=await host.snapshot();
   const saved=record(host.room);const coreChoice=saved.state.timeout.deadline;
   if(coreChoice){owner=peers[coreChoice.observer];state=await owner.snapshot();break;}
   const current=peers[state.view.current_player-1] ?? peers.find(p=>p.player===state.view.turn?.player) ?? peers[record(host.room).state.members.find(m=>m.player_id===state.view.observer?.Player)?.seat ?? 0];
   let moved=false;
   for(const p of peers){const view=await p.snapshot();if(!view.legal_actions.length)continue;const action=view.legal_actions.includes("EndPhase") ? "EndPhase" : view.legal_actions[0];const kind=typeof action==='string'?action:structuredClone(action); if(kind.PlayCard) kind.PlayCard.card_targets=[];const result=await p.submit(kind,view.view.revision);assert.ok(result.result.Accepted,result.result);moved=true;break;}
   assert.ok(moved,'must progress');
  }
  assert.ok(state.choice,'real pending choice');
  const initial=record(host.room);const deadline=initial.state.timeout.deadline.deadline_unix_ms;
  assert.equal(initial.state.timeout.choice_timeout_secs,10);
  const reconnectToken=owner.token;
  owner.close();const reconnected=await Peer.open();const rejoin=await reconnected.join(host.room,owner.seat,reconnectToken);assert.equal(rejoin.choice.deadline_unix_ms,deadline);owner=reconnected;
  assert.equal(record(host.room).state.timeout.deadline.deadline_unix_ms,deadline);
  // Network callers cannot inject privileged actions or durable control fields.
  owner.send('SubmitCommand',{header:{protocol_version:6,room_id:host.room,conn_seq:++owner.seq,revision:state.view.revision},command:{actor:owner.player,expected_revision:state.view.revision,idempotency_key:9001,kind:{TimeoutAction:{choice_id:state.choice.choice_id}}},idempotency_key:9001,replay:false});assert.equal((await owner.until(m=>m.type==='Error')).payload.code,'Unauthorized');
  owner.send('CreateRoom',{players:4,seed:1,password:'',choice_timeout_secs:10,managed:true,wal:[]});assert.equal((await owner.until(m=>m.type==='Error')).payload.code,'BadMessage');
  fs.writeFileSync(path.join(root,'before-kill.json'),JSON.stringify(record(host.room),null,2));
  for(const p of peers)p.close();owner.close();await stop();
  await pause(Math.max(0,deadline-Date.now())+200);
  start();const restored=await ready();await restored.join(host.room,initial.state.timeout.deadline.observer,reconnectToken);
  let after;for(let i=0;i<100;i++){after=record(host.room);if(after.state.wal.entries.some(e=>e.action.TimeoutAction))break;await pause(20);}
  const timeouts=after.state.wal.entries.filter(e=>e.action.TimeoutAction);
  assert.equal(timeouts.length,1);assert.equal(timeouts[0].action.TimeoutAction.choice_id,state.choice.choice_id);
  assert.ok(timeouts[0].dedup_id>0);assert.equal(timeouts[0].expected_revision+1,timeouts[0].revision);
  const raced=await restored.submit({ResolveChoice:{choice_id:state.choice.choice_id,selections:[0,1],cancelled:false}},state.view.revision,9002);assert.ok(raced.result.Rejected,'old human answer loses to committed timeout');assert.equal(record(host.room).state.wal.entries.filter(e=>e.action.TimeoutAction).length,1);fs.writeFileSync(path.join(root,'after-restart.json'),JSON.stringify(after,null,2));
  restored.close();await stop();
  fs.writeFileSync(path.join(root,'report.json'),JSON.stringify({passed:true,room:host.room,deadline,timeout:timeouts[0],downtimeCounted:true,reconnectDidNotExtend:true,privilegedActionRejected:true,forgedControlRejected:true,ws5Rejected:true,noHelloRejected:true,zeroVersionRejected:true,timeoutFirstHumanRaceRejected:true},null,2));
 }
}catch(error){fs.writeFileSync(path.join(root,'failure.txt'),error.stack);throw error;}
finally{await stop();fs.writeFileSync(path.join(root,'wire.json'),JSON.stringify(trace,null,2));}
