import {readFileSync,writeFileSync} from 'node:fs';
const base='/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-10-30-j2-main-integration/combined-acceptance';
const port=readFileSync('/tmp/noname-b-j2-main-2zzoa42o/j2-chrome-profile/DevToolsActivePort','utf8').split('\n')[0];
const tab=await(await fetch(`http://127.0.0.1:${port}/json/new?http://127.0.0.1:4186`,{method:'PUT'})).json();
const socket=new WebSocket(tab.webSocketDebuggerUrl);await new Promise(r=>socket.onopen=r);
let next=0;const pending=new Map();socket.onmessage=e=>{const d=JSON.parse(e.data);if(pending.has(d.id)){pending.get(d.id)(d);pending.delete(d.id);}};
const cdp=(method,params={})=>new Promise(resolve=>{const id=++next;pending.set(id,resolve);socket.send(JSON.stringify({id,method,params}));});
const names=['danshou-hand-theft-not-gift','danshou-equipment-theft-not-gift','fenghe-genuine-gift-benefit'];
const scenarios=JSON.parse(readFileSync(base+'/j2-native-matrix.json','utf8')).scenarios.filter(s=>names.includes(s.name));
const expression=`(async()=>{
 const assert=(ok,label)=>{if(!ok)throw Error(label)};
 const create=async()=>{const worker=new Worker('/src/worker/engine.native.worker.ts',{type:'module'});let next=0;const pending=new Map();worker.onmessage=e=>{const d=e.data;if(pending.has(d.id)){const {resolve,reject}=pending.get(d.id);pending.delete(d.id);d.ok?resolve(d.data):reject(Error(d.error));}};const send=message=>new Promise((resolve,reject)=>{const id=++next;pending.set(id,{resolve,reject});worker.postMessage({...message,id});setTimeout(()=>{if(pending.delete(id))reject(Error('Worker timeout '+message.type))},15000)});await send({type:'init'});return {send,worker};};
 const a=await create(),b=await create();const store=await import('/src/engine/extensionStore.ts');const match=await import('/src/engine/nativeMatchContent.ts');localStorage.clear();const scenarios=${JSON.stringify(scenarios)},results=[];
 const command=(step,envelope)=>{const c=step.command;switch(c.type){case'activateSkill':return {type:'activateSkill',actor:c.actor,skill:c.skill};case'playCard':return {type:'playCard',actor:c.actor,card:c.card,targets:c.targets};case'answer':return {type:'resolveChoice',actor:c.actor,choiceId:envelope.choice.choice_id,selections:c.selections.map(index=>envelope.choice.options[index].token)};default:throw Error(c.type)}};
 for(const scenario of scenarios){
  const payload=JSON.parse(scenario.packs_json)[0];await store.importExtensionPackageValidated(JSON.stringify(payload),async packJson=>a.send({type:'validatePackage',packJson}));const key=payload.manifest.namespace+'/'+payload.manifest.package_id;store.activateExtension(key);const frozen=match.prepareNativeMatchContent();let envelope=await a.send({type:'restore',hex:scenario.initial,packsJson:frozen.packsJson});
  for(const step of scenario.steps){
   const saved=await a.send({type:'save'});const resumed=await b.send({type:'restore',hex:saved.hex,packsJson:frozen.packsJson});
   if(step.command.type==='answer'){let rejected=false;try{await a.send({...command(step,envelope),choiceId:envelope.choice.choice_id+1000})}catch{rejected=true}assert(rejected,'invalid choice rejects');assert((await a.send({type:'save'})).hex===saved.hex,'failed retry preserves full archive');}
   const directCommand=command(step,envelope);envelope=await a.send(directCommand);const other=await b.send(command(step,resumed));assert(envelope.hash_hex===step.hash,scenario.name+' native/WASM/Worker hash');assert(other.hash_hex===step.hash,'Worker checkpoint hash');assert(JSON.stringify(envelope.events)===JSON.stringify(other.events),'Worker checkpoint events');
   if(step.command.type==='answer'){const before=(await a.send({type:'save'})).hex;let rejected=false;try{await a.send(directCommand)}catch{rejected=true}assert(rejected,'stale choice retry rejects');assert((await a.send({type:'save'})).hex===before,'stale retry cannot duplicate theft or gift');}
  }
  store.deactivateExtension(key);store.uninstallExtension(key);assert(match.prepareNativeMatchContent().contentLock.length===0,'uninstalled package absent from new matches');await b.send({type:'restore',hex:scenario.initial,packsJson:match.restoreNativeMatchContent(frozen).packsJson});results.push({name:scenario.name,steps:scenario.steps.length,hash:envelope.hash_hex});
 }
 a.worker.terminate();b.worker.terminate();return {realWorker:true,headlessChrome:true,sdkStore:true,failedRetryAtomicity:true,results};
})()`;
const result=await cdp('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});writeFileSync(base+'/j2-real-chrome-worker-result.json',JSON.stringify(result,null,2));if(result.result?.exceptionDetails){console.error(JSON.stringify(result.result.exceptionDetails));process.exitCode=1;}else console.log(JSON.stringify(result.result?.result?.value,null,2));
socket.close();
