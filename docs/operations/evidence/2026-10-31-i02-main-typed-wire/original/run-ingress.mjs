import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {spawn} from 'node:child_process';
import {readFileSync,writeFileSync,openSync,mkdtempSync,statSync} from 'node:fs';
import {join,extname,resolve} from 'node:path';
import crypto from 'node:crypto';
const E='/tmp/i02-main-typed-wire-functional-33d21ba5';const mode=process.argv[2];assert.ok(['chrome','tauri'].includes(mode));const root=E+'/ingress-dist';const requestRows=[];const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
let reportResolve;const reportPromise=new Promise(r=>reportResolve=r);
const server=createServer((req,res)=>{
 if(req.method==='POST'&&req.url==='/report'){let raw='';req.on('data',b=>raw+=b);req.on('end',()=>{const report=JSON.parse(raw);writeFileSync(E+`/ingress-${mode}-report.json`,JSON.stringify(report,null,2));res.writeHead(204).end();reportResolve(report);});return;}
 const pathname=decodeURIComponent(new URL(req.url,'http://localhost').pathname);let file=resolve(root,'.'+pathname);if(file===root)file=join(root,'index.html');if(!file.startsWith(root+'/'))return res.writeHead(403).end();
 try{const bytes=readFileSync(file);res.setHeader('Content-Type',({'.html':'text/html','.js':'text/javascript','.css':'text/css','.wasm':'application/wasm','.json':'application/json'})[extname(file)]??'application/octet-stream');res.setHeader('Cross-Origin-Opener-Policy','same-origin');res.setHeader('Cross-Origin-Embedder-Policy','require-corp');requestRows.push({path:pathname,sha256:sha(bytes),bytes:bytes.length});res.end(bytes);}catch{res.writeHead(404).end();}
});
const config=JSON.parse(readFileSync(E+'/tauri-private-config.json'));const port=mode==='tauri'?Number(new URL(config.build.devUrl).port):0;await new Promise((r,j)=>server.once('error',j).listen(port,'127.0.0.1',r));const url=`http://127.0.0.1:${server.address().port}/?engineTransport=${mode==='tauri'?'native':'wasm'}&latency=sab&perf=1`;
let child,ws,timer;const errors=[];const runtimeLog=openSync(E+`/ingress-${mode}-runtime.log`,'w');
try{
 if(mode==='tauri'){const binary='/tmp/nc27-main-functional-458ed333/cargo-target/debug/noname-next-desktop';writeFileSync(E+'/tauri-executed-identity.json',JSON.stringify({binary,sha256:sha(readFileSync(binary)),config,configSha256:sha(readFileSync(E+'/tauri-private-config.json')),qualification:'Actual Main Tauri/Rust bridge, private identifier and hidden window; metadata injection replaces successful JS callback data only.'},null,2));child=spawn(binary,[],{stdio:['ignore',runtimeLog,runtimeLog]});}
 else{
  const profile=mkdtempSync(E+'/chrome-ingress-profile-');child=spawn('/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',['--headless=new','--disable-gpu','--no-sandbox',`--user-data-dir=${profile}`,'--remote-debugging-port=0','about:blank'],{stdio:['ignore',runtimeLog,runtimeLog]});
  const wait=async(label,fn)=>{for(let n=0;n<200;n++){const v=await fn();if(v)return v;if(child.exitCode!==null)throw Error('chrome exited');await new Promise(r=>setTimeout(r,100));}throw Error('timeout '+label);};
  const port=await wait('CDP',()=>{try{return readFileSync(profile+'/DevToolsActivePort','utf8').split('\n')[0];}catch{return null;}});const page=(await fetch(`http://127.0.0.1:${port}/json/list`).then(r=>r.json())).find(x=>x.type==='page');ws=new WebSocket(page.webSocketDebuggerUrl);await new Promise((r,j)=>{ws.onopen=r;ws.onerror=j;});let id=0;const pending=new Map();const send=(method,params={})=>new Promise((r,j)=>{const n=++id;pending.set(n,{r,j});ws.send(JSON.stringify({id:n,method,params}));});ws.onmessage=({data})=>{const m=JSON.parse(data);if(m.id&&pending.has(m.id)){const {r,j}=pending.get(m.id);pending.delete(m.id);m.error?j(Error(JSON.stringify(m.error))):r(m.result);}else if(m.method==='Runtime.exceptionThrown')errors.push(m.params.exceptionDetails);};await send('Runtime.enable');await send('Page.enable');await send('Page.navigate',{url});
 }
 const report=await Promise.race([reportPromise,new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error('runtime report timeout90s')),90000);})]);assert.equal(report.passed,true,JSON.stringify(report));assert.equal(report.invalidCorpus,80);assert.equal(report.networkAtomicCases,80);assert.ok(report.positiveAbsent>=162);assert.ok(report.rows.every(x=>x.pendingIdentityRetained&&x.perfUnacknowledged&&x.storeUnchanged&&x.disposeClean));assert.deepEqual(errors,[]);console.log(JSON.stringify({runtime:mode,passed:true,cases:report.rows.length,actualRequests:report.actualRequests,positiveAbsent:report.positiveAbsent,sharedCorpus:report.invalidCorpus}));
}finally{
 clearTimeout(timer);writeFileSync(E+`/ingress-${mode}-http-binding.json`,JSON.stringify({mode,root,url,requests:requestRows,errors},null,2));ws?.close();if(child?.exitCode===null){const ended=new Promise(r=>child.once('exit',r));child.kill('SIGTERM');await ended;}await new Promise(r=>server.close(r));
}
