// Actual UI controls followed by protocol/restore coverage on the same Vue-created Worker.
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {createServer} from 'node:http';
import {mkdtempSync,openSync,readFileSync,statSync,writeFileSync} from 'node:fs';
import {extname,join,resolve} from 'node:path';
import {createHash} from 'node:crypto';
const [distArg,chromeBin,outArg]=process.argv.slice(2);assert.ok(distArg&&chromeBin&&outArg);
const dist=resolve(distArg),out=resolve(outArg),requested=[],reports=[],expectedRejected=[];
const sha=b=>createHash('sha256').update(b).digest('hex');
const types={'.html':'text/html','.js':'text/javascript','.css':'text/css','.wasm':'application/wasm','.json':'application/json','.png':'image/png','.svg':'image/svg+xml'};
const server=createServer((req,res)=>{
 const path=decodeURIComponent(new URL(req.url??'/','http://localhost').pathname);let file=join(dist,path);
 try{if(!statSync(file).isFile())throw new Error();}catch{file=join(dist,'index.html');}
 res.setHeader('content-type',types[extname(file)]??'application/octet-stream');res.setHeader('cross-origin-opener-policy','same-origin');res.setHeader('cross-origin-embedder-policy','require-corp');
 const bytes=readFileSync(file);requested.push({path,sha256:sha(bytes),bytes:bytes.length});res.end(bytes);
});
await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin=`http://127.0.0.1:${server.address().port}`;
const runDir=mkdtempSync(join('/private/tmp/noname-c331-product-surface-20261102/chrome-private','seven-c331-'));
const chromeLog=openSync(join(runDir,'chrome.log'),'w');
const chrome=spawn(chromeBin,['--headless=new','--disable-gpu','--no-sandbox',`--user-data-dir=${runDir}/profile`,'--remote-debugging-port=0','--window-size=1280,1000','about:blank'],{stdio:['ignore',chromeLog,chromeLog]});
writeFileSync(join(out,'seven-browser-launch.json'),JSON.stringify({pid:chrome.pid,privateRunDir:runDir,profileExcluded:true},null,2)+'\n');
const pause=ms=>new Promise(r=>setTimeout(r,ms));
const wait=async(description,fn)=>{for(let i=0;i<200;i++){const result=await fn();if(result)return result;if(chrome.exitCode!==null)throw new Error(`Chrome exited ${chrome.exitCode}`);await pause(100);}throw new Error(`timed out: ${description}`);};
let ws,ownedBrowserWs,evaluate;
const exceptions=[];
try{
 const port=await wait('Chrome debug port',()=>{try{return readFileSync(join(runDir,'profile/DevToolsActivePort'),'utf8').split('\n')[0];}catch{return null;}});
 const browserInfo=await fetch(`http://127.0.0.1:${port}/json/version`).then(r=>r.json());ownedBrowserWs=new WebSocket(browserInfo.webSocketDebuggerUrl);
 await new Promise((r,j)=>{ownedBrowserWs.onopen=r;ownedBrowserWs.onerror=j;});
 const browserPending=new Map();let browserSeq=0;
 ownedBrowserWs.onmessage=({data})=>{const m=JSON.parse(data),p=browserPending.get(m.id);if(p){browserPending.delete(m.id);m.error?p.reject(Error(JSON.stringify(m.error))):p.resolve(m.result);}};
 const ownership=await new Promise((resolve,reject)=>{const n=++browserSeq;browserPending.set(n,{resolve,reject});ownedBrowserWs.send(JSON.stringify({id:n,method:'SystemInfo.getProcessInfo'}));});
 assert.ok(ownership.processInfo.some(x=>x.type==='browser'&&x.id===chrome.pid),'CDP endpoint must identify owned Chrome PID');
 const target=await wait('Chrome page',async()=>{try{return(await fetch(`http://127.0.0.1:${port}/json/list`).then(r=>r.json())).find(x=>x.type==='page');}catch{return null;}});
 ws=new WebSocket(target.webSocketDebuggerUrl);await new Promise((r,j)=>{ws.onopen=r;ws.onerror=j;});let id=0;const pending=new Map();
 const send=(method,params={})=>new Promise((resolve,reject)=>{const n=++id;pending.set(n,{resolve,reject});ws.send(JSON.stringify({id:n,method,params}));});
 ws.onmessage=({data})=>{const msg=JSON.parse(data);if(msg.id&&pending.has(msg.id)){const{resolve,reject}=pending.get(msg.id);pending.delete(msg.id);msg.error?reject(Error(JSON.stringify(msg.error))):resolve(msg.result);}else if(msg.method==='Runtime.exceptionThrown')exceptions.push(msg.params.exceptionDetails.text);else if(msg.method==='Runtime.consoleAPICalled'&&msg.params.type==='error')exceptions.push(msg.params.args.map(x=>x.value??x.description).join(' '));};
 evaluate=async expression=>{const result=await send('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});if(result.exceptionDetails)throw Error(JSON.stringify(result.exceptionDetails));return result.result.value;};
 await send('Runtime.enable');await send('Page.enable');
 await send('Page.addScriptToEvaluateOnNewDocument',{source:`(()=>{
  const capture=window.__nativeProduct={sent:{},requests:[],replies:{},sequence:[],latest:null,frames:0,rejected:[],workerUrl:null,workerCount:0};let receiptSeq=0;const OriginalWorker=window.Worker;
  window.Worker=class extends OriginalWorker{
   constructor(...args){super(...args);capture.worker=this;capture.workerUrl=String(args[0]);capture.workerCount++;
    this.addEventListener('message',({data})=>{if(!data||!Number.isInteger(data.id))return;capture.sequence.push({seq:++receiptSeq,direction:'reply',data:structuredClone(data)});capture.replies[data.id]=structuredClone({ok:data.ok,error:data.error,data:data.data});if(data.ok===false)capture.rejected.push(data.error);if(data.data?.hash_hex&&typeof data.data?.finished==='boolean'){capture.latest=structuredClone(data.data);capture.frames++;}});
   }
   postMessage(...args){const message=args[0];if(message&&Number.isInteger(message.id)){capture.sent[message.type]=message.id;capture.requests.push(structuredClone(message));capture.sequence.push({seq:++receiptSeq,direction:'request',data:structuredClone(message)});}return super.postMessage(...args);}
  };
 })();`});
 const current=()=>evaluate('window.__nativeProduct?.latest??null');
 const clickRequest=async(label,type)=>{
  const previous=await evaluate(`window.__nativeProduct.sent[${JSON.stringify(type)}]??0`);
  await wait(`${label} enabled`,()=>evaluate(`(()=>{const b=[...document.querySelectorAll('button')].find(b=>b.textContent.trim()===${JSON.stringify(label)});return b&&!b.disabled;})()`));
  assert.equal(await evaluate(`(()=>{const b=[...document.querySelectorAll('button')].find(b=>b.textContent.trim()===${JSON.stringify(label)});b.click();return true;})()`),true);
  const request=await wait(`${type} sent`,()=>evaluate(`(()=>{const id=window.__nativeProduct.sent[${JSON.stringify(type)}];return id>${previous}?id:null;})()`));
  const reply=await wait(`${type} response`,()=>evaluate(`window.__nativeProduct.replies[${request}]??null`));assert.equal(reply.ok,true,`${label}: ${reply.error}`);return current();
 };
 await send('Page.navigate',{url:origin+'/battle?players=3&skipAnims=1'});await wait('app connected',()=>current());
 const slot=async name=>{
  const hex=readFileSync(join(out,name+'.ngsa')).toString('hex');
  await evaluate(`localStorage.setItem('noname-next.native-saves.v1',JSON.stringify([{id:'probe',label:'NC10 formal fixture',hex:${JSON.stringify(hex)},players:3,seed:2306,revision:1,contentLock:[],lockSeed:'',savedAt:'2026-10-31T11:25:00Z'}]))`);
  const frame=await clickRequest('读档','restore');assert.match(frame.hash_hex,/^[0-9a-f]{64}$/);return frame;
 };
 const native=JSON.parse(readFileSync(join(out,'native-probe-results.json')));
 let frame=await slot('park-0');assert.equal(frame.choice.source_skill,'遗计');assert.equal(frame.choice.allow_cancel,true);
 await wait('own optional UI',()=>evaluate(`document.querySelector('.choice h2')?.textContent==='是否发动【遗计】？'`));
 await send('Page.captureScreenshot',{format:'png'}).then(r=>writeFileSync(join(out,'own-optional.png'),Buffer.from(r.data,'base64')));
 const before=frame.revision;frame=await clickRequest('选择 0','resolveChoice');assert.equal(frame.revision,before+1);assert.equal(frame.result.outcome,'pending');reports.push({case:'UI-own-confirm',frame});
 frame=await slot('park-0');const cancelBefore=frame.revision;frame=await clickRequest('取消','resolveChoice');assert.equal(frame.revision,cancelBefore+1);assert.ok(['accepted','pending'].includes(frame.result.outcome));assert.equal(frame.choice,null);assert.equal(frame.hash_hex,native['own-cancel'].hash);reports.push({case:'UI-optional-cancel',frame});
 frame=await slot('shan');const cardToken=frame.choice.options.find(x=>x.label_key.startsWith('card:')).token;const action=frame.legal_actions.find(x=>x.ResolveChoice&&!x.ResolveChoice.cancelled&&x.ResolveChoice.selections.includes(cardToken));assert.ok(action);
 const shanBefore=frame.revision;frame=await clickRequest('选择 '+action.ResolveChoice.selections.join(', '),'resolveChoice');assert.equal(frame.revision,shanBefore+1);assert.equal(frame.hash_hex,native.direct.hash);reports.push({case:'UI-Shan-card',frame});
 frame=await slot('shan');const autoBefore=frame.revision;frame=await clickRequest('自动一步','autoStep');assert.equal(frame.revision,autoBefore+1);assert.equal(frame.hash_hex,native.auto.hash);reports.push({case:'UI-Shan-auto-once',frame});
 frame=await slot('native-park-1');assert.equal(frame.choice,null);await wait('Other hidden in UI',()=>evaluate(`document.querySelector('.choice')===null`));reports.push({case:'UI-Other-hidden',frame});
 await wait('UI pending zero',()=>evaluate('window.__nativeProduct.requests.every(x=>window.__nativeProduct.replies[x.id])'));
 assert.equal(await evaluate(`[...document.querySelectorAll('button')].some(x=>x.textContent.trim()==='停止自动')`),false);
 assert.equal(await evaluate('window.__nativeProduct.workerCount'),1);assert.equal(await evaluate('window.__nativeProduct.requests.some(x=>x.id>=800000)'),false);
 const workerName=await evaluate('window.__nativeProduct.workerUrl');assert.ok(workerName);
 await evaluate(`(()=>{window.__probeWorker=window.__nativeProduct.worker;window.__probeSeq=800000;window.__probeSend=msg=>new Promise((resolve,reject)=>{const id=window.__probeSeq++;if(window.__nativeProduct.requests.some(x=>x.id===id))throw Error('request id collision');const timer=setTimeout(()=>reject(Error('worker timeout')),30000);const listen=ev=>{if(ev.data.id!==id)return;clearTimeout(timer);window.__probeWorker.removeEventListener('message',listen);resolve(structuredClone(ev.data));};window.__probeWorker.addEventListener('message',listen);window.__probeWorker.postMessage({...msg,id});});})()`);
 const rawRequest=msg=>evaluate(`window.__probeSend(${JSON.stringify(msg)})`);
 const request=async msg=>{const x=await rawRequest(msg);assert.equal(x.ok,true,x.error);return x.data;};
 const oracle=JSON.parse(readFileSync(join(out,'seven-full-envelope-oracle.json'))).cases;
 for(const name of ['park-0','park-1','own-cancel','shan-auto','shan-direct','shan-cancel','shan-pass']){
  const expected=oracle[name];assert.deepEqual(await request({type:'start',players:3,seed:2306,observer:1}),expected.start,'complete start '+name);
  const fixture=name==='own-cancel'?'park-0':name.startsWith('park')?name:'shan';const baseline=await request({type:'restore',hex:readFileSync(join(out,fixture+'.ngsa')).toString('hex')});assert.ok(baseline.choice);
  const choice=baseline.choice,cardToken=choice.options.find(x=>x.label_key.startsWith('card:'))?.token;
  const command=name.endsWith('pass')?{type:'resolveChoice',actor:1,choiceId:choice.choice_id,selections:[choice.options.find(x=>x.label_key==='pass').token],cancelled:false}:name.endsWith('direct')?{type:'resolveChoice',actor:1,choiceId:choice.choice_id,selections:[cardToken],cancelled:false}:name.endsWith('cancel')?{type:'resolveChoice',actor:1,choiceId:choice.choice_id,selections:[],cancelled:true}:{type:'autoStep'};
  assert.deepEqual(baseline,expected.baseline,'complete restored envelope '+name);assert.deepEqual(command,expected.command);
  const next=await request(command);assert.deepEqual(next,expected.next,'complete command envelope '+name);assert.equal(next.revision,baseline.revision+(name==='shan-cancel'?0:1));const nativeExpected=native[name]??native[name.replace('shan-','')];assert.equal(next.hash_hex,nativeExpected.hash);assert.deepEqual(next.result,nativeExpected.reply);
  const saved=await request({type:'save'});assert.deepEqual(saved,expected.saved);const archive=Buffer.from(saved.hex,'hex');assert.deepEqual(archive,readFileSync(join(out,'native-'+name+'.ngsa')));writeFileSync(join(out,'browser-'+name+'.ngsa'),archive);
  const restored=await request({type:'restore',hex:saved.hex});assert.deepEqual(restored,expected.restored,'complete re-restored envelope '+name);
  await request({type:'observer'});const atomicBaseline=await request({type:'observer'});assert.deepEqual(atomicBaseline,expected.atomicBaseline);assert.deepEqual(await request({type:'save'}),expected.postRestoreSaved);
  const malformed=await rawRequest({type:'restore',hex:'00'});assert.equal(malformed.ok,false);assert.equal(malformed.error,expected.malformedError);expectedRejected.push(malformed.error);
  assert.deepEqual(await request({type:'observer'}),atomicBaseline,'malformed restore must retain complete post-restore envelope '+name);assert.deepEqual(await request({type:'save'}),expected.afterMalformedSaved);
  const continued=await request(expected.continueCommand);assert.deepEqual(continued,expected.continued,'complete restored-instance continuation '+name);const continuedSaved=await request({type:'save'});assert.deepEqual(continuedSaved,expected.continuedSaved);writeFileSync(join(out,'browser-continued-'+name+'.ngsa'),Buffer.from(continuedSaved.hex,'hex'));
  reports.push({case:'Vue-worker-protocol-'+name,baseline,next,fullArchiveSha256:sha(archive),restored,atomicBaseline,malformedError:malformed.error,continued,continuedArchiveSha256:sha(Buffer.from(continuedSaved.hex,'hex')),restoreArchiveException:expected.restoreArchiveException});
 }
 const receipt=await evaluate('({sequence:window.__nativeProduct.sequence,requests:window.__nativeProduct.requests,workerUrl:window.__nativeProduct.workerUrl,workerCount:window.__nativeProduct.workerCount})');
 assert.deepEqual(receipt.sequence.map(x=>x.seq),receipt.sequence.map((_,i)=>i+1));assert.equal(new Set(receipt.requests.map(x=>x.id)).size,receipt.requests.length);
 writeFileSync(join(out,'seven-worker-receipt.json'),JSON.stringify(receipt,null,2)+'\n');
 await send('Page.captureScreenshot',{format:'png'}).then(r=>writeFileSync(join(out,'product.png'),Buffer.from(r.data,'base64')));
 const rejected=await evaluate('window.__nativeProduct.rejected');assert.deepEqual(rejected,expectedRejected);assert.deepEqual(exceptions,[]);assert.ok(requested.some(x=>x.path.endsWith('.wasm')));
 writeFileSync(join(out,'browser-results.json'),JSON.stringify({passed:true,uiPositiveControls:5,formalProtocolCases:7,restoredContinuations:7,malformedAtomicChecks:7,reports,exceptions,rejected,requested,workerName,dist,privateProfileExcluded:true},null,2)+'\n');
 console.log(JSON.stringify({passed:true,uiPositiveControls:5,formalBrowserProtocolCases:7,restoredContinuations:7,malformedAtomicChecks:7,exceptions:0,expectedMalformedTransportRejections:7,realVueCreatedWorker:true,currentProductionDist:dist}));
}finally{
 if(evaluate&&ws?.readyState===WebSocket.OPEN){try{writeFileSync(join(out,'seven-worker-partial-receipt.json'),JSON.stringify(await evaluate('({sequence:window.__nativeProduct?.sequence,requests:window.__nativeProduct?.requests,workerUrl:window.__nativeProduct?.workerUrl})'),null,2)+'\n');}catch{}}
 if(ownedBrowserWs?.readyState===WebSocket.OPEN){ownedBrowserWs.send(JSON.stringify({id:999999,method:'Browser.close'}));await wait('owned Chrome graceful exit',()=>chrome.exitCode!==null||chrome.signalCode!==null);}
 ws?.close();ownedBrowserWs?.close();await new Promise(r=>server.close(r));
 writeFileSync(join(out,'seven-browser-cleanup.json'),JSON.stringify({ownedChromePID:chrome.pid,exitCode:chrome.exitCode,signalCode:chrome.signalCode,signalsSent:false,gracefulProtocol:'Browser.close',serverClosed:true,privateRunDir:runDir},null,2)+'\n');
}
