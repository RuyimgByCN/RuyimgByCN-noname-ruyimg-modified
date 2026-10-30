import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { spawn } from 'node:child_process';
import { readFileSync, writeFileSync, existsSync, mkdtempSync, rmSync } from 'node:fs';
import path from 'node:path';
const repo=path.resolve(process.argv[2]);
const out=path.resolve(process.argv[3]);
const profile=mkdtempSync(path.join(out,'chrome-'));
const mime={'.html':'text/html','.js':'text/javascript','.css':'text/css','.wasm':'application/wasm','.json':'application/json','.jpg':'image/jpeg','.png':'image/png','.mp3':'audio/mpeg'};
const requests=[];
const server=createServer((req,res)=>{
 const url=new URL(req.url,'http://localhost');
 const resource=url.pathname.startsWith('/original-content/');
 const root=resource?path.join(repo,'content/original'):path.join(out,'../validated-dist');
 let file=path.resolve(root, '.'+decodeURIComponent(resource?url.pathname.slice('/original-content'.length):url.pathname));
 if(!file.startsWith(root+path.sep)){res.writeHead(403).end();return;}
 if(!resource&&!existsSync(file))file=path.join(root,'index.html');
 try{const data=readFileSync(file);res.writeHead(200,{'Content-Type':mime[path.extname(file)]??'application/octet-stream','Access-Control-Allow-Origin':'*','Cross-Origin-Resource-Policy':'cross-origin'}).end(data);requests.push({url:url.pathname,status:200,bytes:data.length,mime:mime[path.extname(file)]});}
 catch{res.writeHead(404).end();requests.push({url:url.pathname,status:404});}
});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
const base=`http://127.0.0.1:${server.address().port}`;
const chrome=spawn('/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',['--headless=new','--disable-gpu','--no-sandbox',`--user-data-dir=${profile}`,'--remote-debugging-port=0','--window-size=1280,1000','about:blank'],{stdio:'ignore'});
const wait=ms=>new Promise(r=>setTimeout(r,ms));
let ws;let id=0;const pending=new Map();const errors=[];
try{
 for(let i=0;i<100&&!existsSync(path.join(profile,'DevToolsActivePort'));i++)await wait(100);
 const port=readFileSync(path.join(profile,'DevToolsActivePort'),'utf8').split('\n')[0];
 const pages=await fetch(`http://127.0.0.1:${port}/json/list`).then(r=>r.json());
 ws=new WebSocket(pages.find(x=>x.type==='page').webSocketDebuggerUrl);
 await new Promise((r,j)=>{ws.onopen=r;ws.onerror=j;});
 const send=(method,params={})=>new Promise((resolve,reject)=>{const key=++id;pending.set(key,{resolve,reject});ws.send(JSON.stringify({id:key,method,params}));});
 ws.onmessage=({data})=>{const msg=JSON.parse(data);if(msg.id){const p=pending.get(msg.id);pending.delete(msg.id);if(p)(msg.error?p.reject(msg.error):p.resolve(msg.result));}
 else if(msg.method==='Fetch.requestPaused'){const p=msg.params;void send('Fetch.continueRequest',{requestId:p.requestId,url:p.request.url.replace('http://127.0.0.1:7700',base)});}
 else if(msg.method==='Runtime.exceptionThrown')errors.push(msg.params);};
 const evaluate=async(expression)=>{const r=await send('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});assert.ok(!r.exceptionDetails,JSON.stringify(r.exceptionDetails));return r.result.value;};
 await send('Runtime.enable');await send('Page.enable');
 await send('Fetch.enable',{patterns:[{urlPattern:'http://127.0.0.1:7700/original-content/*'}]});
 await send('Page.navigate',{url:base+'/collection'});
 for(let i=0;i<100;i++){if(await evaluate("document.querySelectorAll('.character-row').length>0"))break;await wait(100);}
 const matrix=[];
 const select=async(name)=>{
  await evaluate(`(()=>{const input=document.querySelector('[aria-label="搜索武将"]');input.value=${JSON.stringify(name)};input.dispatchEvent(new Event('input',{bubbles:true}));})()`);
  for(let i=0;i<30;i++){if(await evaluate(`Array.from(document.querySelectorAll('.character-row')).some(b=>b.innerText.includes(${JSON.stringify(name)}))`))break;await wait(30);}
  await evaluate(`Array.from(document.querySelectorAll('.character-row')).find(b=>b.innerText.includes(${JSON.stringify(name)})).click()`);
  await wait(60);
 };
 await select('star_dongyun');
 await evaluate("(()=>{const a=document.querySelector('audio');a.src='/broken.mp3';a.load();})()");
 for(let i=0;i<30;i++){if(await evaluate("!!document.querySelector('audio')?.error"))break;await wait(50);}
 matrix.push(await evaluate("({case:'actual corrupt response',error:document.querySelector('audio').error?.code,alert:document.querySelector('.detail [role=alert]')?.textContent})"));
 assert.ok(matrix[0].error);
 await select('huan_zhouyu');
 await evaluate("new Promise((resolve,reject)=>{const a=document.querySelector('audio');a.addEventListener('loadeddata',()=>resolve(true),{once:true});a.addEventListener('error',()=>reject(new Error('good audio failed')),{once:true});a.load();})");
 matrix.push(await evaluate("({case:'new character valid loadeddata',error:document.querySelector('audio').error?.code??null,readyState:document.querySelector('audio').readyState,alert:document.querySelector('.detail [role=alert]')?.textContent})"));
 assert.equal(matrix[1].error,null);assert.ok(matrix[1].readyState>=2);assert.ok(matrix[1].alert,'expected reproduced stale error');
 writeFileSync(path.join(out,'lifecycle-checkpoint.json'),JSON.stringify(matrix,null,2));
 const box=await evaluate("(()=>{const a=document.querySelector('audio');a.scrollIntoView({block:'center'});const r=a.getBoundingClientRect();return {x:r.x+18,y:r.y+r.height/2};})()");
 await send('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,...box});
 await send('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,...box});
 await evaluate("document.querySelector('audio').play()");
 await evaluate("document.querySelectorAll('audio')[1].play()");
 matrix.push(await evaluate("({case:'two controls',playing:[...document.querySelectorAll('audio')].filter(a=>!a.paused).length,labels:[...document.querySelectorAll('audio')].map(a=>a.getAttribute('aria-label'))})"));
 await evaluate("window.oldReviewAudio=[...document.querySelectorAll('audio')]");
 await select('tangtang');
 await wait(100);
 matrix.push(await evaluate("({case:'character without audio',current:document.querySelectorAll('audio').length,old:window.oldReviewAudio.map(a=>({connected:a.isConnected,paused:a.paused})),alert:document.querySelector('.detail [role=alert]')?.textContent})"));
 await evaluate("document.querySelector('a[href=\"/settings\"]').click()");
 await wait(100);
 matrix.push(await evaluate("({case:'leave Collection',audio:document.querySelectorAll('audio').length,old:window.oldReviewAudio.map(a=>({connected:a.isConnected,paused:a.paused}))})"));
 assert.deepEqual(errors,[]);
 writeFileSync(path.join(out,'browser-result.json'),JSON.stringify({base,sourceRouting:'CDP redirects original-content port to own private static ServeDir equivalent; actual manifest and file bytes',matrix,errors,requests},null,2)+'\n');
 console.log(JSON.stringify({passed:true,cases:matrix}));
}finally{ws?.close();chrome.kill('SIGTERM');await new Promise(r=>chrome.once('exit',r));await new Promise(r=>server.close(r));rmSync(profile,{recursive:true,force:true});}
