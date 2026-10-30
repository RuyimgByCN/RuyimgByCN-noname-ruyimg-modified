import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {spawn} from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
const out=path.resolve(process.argv[2]);const build=path.resolve(process.argv[3]);const binary=process.argv[4];
fs.mkdirSync(out,{recursive:true});
const profile=fs.mkdtempSync(path.join(out,'chrome-'));
const mime={'.html':'text/html','.js':'text/javascript','.css':'text/css','.wasm':'application/wasm','.json':'application/json'};
const http=createServer((req,res)=>{let file=path.join(build,new URL(req.url,'http://x').pathname);if(!fs.existsSync(file)||fs.statSync(file).isDirectory())file=path.join(build,'index.html');try{res.writeHead(200,{'Content-Type':mime[path.extname(file)]??'application/octet-stream'}).end(fs.readFileSync(file));}catch{res.writeHead(404).end();}});
await new Promise(r=>http.listen(0,'127.0.0.1',r));
const port=17888;for(const d of ['packages','original','extensions'])fs.mkdirSync(path.join(out,d),{recursive:true});
const log=fs.openSync(path.join(out,'server.log'),'a');
const server=spawn(binary,[],{env:{...process.env,GAME_SERVER_ADDR:`127.0.0.1:${port}`,NONAME_NATIVE_ROOMS:'1',NONAME_NATIVE_ROOM_PACKAGES:'',NONAME_NATIVE_PACKAGES_ROOT:path.join(out,'packages'),NONAME_NATIVE_STORE_ROOT:path.join(out,'native-store'),NONAME_NATIVE_ROOM_STORE:path.join(out,'rooms'),NONAME_ORIGINAL_CONTENT_ROOT:path.join(out,'original'),NONAME_CONTENT_STORE_ROOT:path.join(out,'legacy-store'),NONAME_EXTENSIONS_ROOT:path.join(out,'extensions')},stdio:['ignore',log,log]});fs.closeSync(log);
const chrome=spawn('/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',['--headless=new','--disable-gpu','--no-sandbox',`--user-data-dir=${profile}`,'--remote-debugging-port=0','--window-size=1280,1000','about:blank'],{stdio:'ignore'});
const wait=ms=>new Promise(r=>setTimeout(r,ms));let ws;let id=0;const pending=new Map();const exceptions=[];
try{
 for(let i=0;i<100&&!fs.existsSync(path.join(profile,'DevToolsActivePort'));i++)await wait(50);
 const debugPort=fs.readFileSync(path.join(profile,'DevToolsActivePort'),'utf8').split('\n')[0];
 const pages=await fetch(`http://127.0.0.1:${debugPort}/json/list`).then(r=>r.json());ws=new WebSocket(pages.find(p=>p.type==='page').webSocketDebuggerUrl);await new Promise((r,j)=>{ws.onopen=r;ws.onerror=j;});
 const send=(method,params={})=>new Promise((resolve,reject)=>{const key=++id;pending.set(key,{resolve,reject});ws.send(JSON.stringify({id:key,method,params}));});
 ws.onmessage=({data})=>{const m=JSON.parse(data);if(m.id){const p=pending.get(m.id);pending.delete(m.id);if(p)m.error?p.reject(m.error):p.resolve(m.result);}else if(m.method==='Runtime.exceptionThrown')exceptions.push(m.params);};
 const evaluate=async expression=>{const result=await send('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});assert.ok(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value;};
 const until=async expression=>{for(let i=0;i<240;i++){if(await evaluate(expression))return;await wait(50);}throw Error('UI timeout '+expression);};
 const screenshot=async name=>{const shot=await send('Page.captureScreenshot',{format:'png',captureBeyondViewport:true});fs.writeFileSync(path.join(out,name+'.png'),Buffer.from(shot.data,'base64'));};
 await send('Runtime.enable');await send('Page.enable');await send('Page.navigate',{url:`http://127.0.0.1:${http.address().port}/`});
 await until('document.querySelector(\'[aria-label="选择时限"]\')');
 const options=await evaluate('[...document.querySelector(\'[aria-label="选择时限"]\').options].map(o=>Number(o.value))');assert.deepEqual(options,[10,15,30,60,90]);
 await evaluate(`(()=>{document.querySelector('[aria-label="选择时限"]').value='10';document.querySelector('[aria-label="选择时限"]').dispatchEvent(new Event('change',{bubbles:true}));document.querySelector('input[type="url"]').value='ws://127.0.0.1:${port}/ws';document.querySelector('input[type="url"]').dispatchEvent(new Event('input',{bubbles:true}));})()`);
 await screenshot('config');
 await evaluate(`Array.from(document.querySelectorAll('button')).find(b=>b.textContent.includes('创建并入座')).click()`);
 if(process.argv[5]==='old') {
   await until(`document.querySelector('.status')?.textContent.includes('WS6')`);
   const status=await evaluate(`document.querySelector('.status').textContent`);
   await screenshot('old-server-rejected');
   assert.equal(exceptions.length,0,JSON.stringify(exceptions));
   fs.writeFileSync(path.join(out,'report.json'),JSON.stringify({passed:true,oldServerRejected:true,status,exceptions},null,2));
 } else {
 await until(`document.querySelector('.online-roster')?.textContent.includes('选择时限 10 秒')`);
 for(let i=0;i<3;i++){await until(`Array.from(document.querySelectorAll('button')).some(b=>b.textContent.trim()==='结束阶段'&&!b.disabled)`);await evaluate(`Array.from(document.querySelectorAll('button')).find(b=>b.textContent.trim()==='结束阶段').click()`);await wait(250);}
 await until(`document.querySelector('[role="timer"]')?.textContent.includes('剩余')`);
 const before=await evaluate(`({timer:document.querySelector('[role="timer"]').textContent,roster:document.querySelector('.online-roster').textContent,choice:document.querySelector('.choice').textContent})`);
 assert.match(before.timer,/剩余 [1-9]/);await screenshot('countdown');
 await until(`document.querySelector('.online-roster')?.textContent.includes('托管中')`);
 const managed=await evaluate(`({roster:document.querySelector('.online-roster').textContent,status:document.querySelector('.status')?.textContent,alerts:[...document.querySelectorAll('[role="alert"]')].map(n=>n.textContent)})`);await screenshot('managed');
 assert.equal(exceptions.length,0,JSON.stringify(exceptions));
 const room=JSON.parse(fs.readFileSync(path.join(out,'rooms','1.json'),'utf8'));assert.ok(room.state.wal.entries.some(e=>e.action.TimeoutAction));
 fs.writeFileSync(path.join(out,'report.json'),JSON.stringify({passed:true,options,before,managed,exceptions,wal:room.state.wal.entries},null,2));
  }
}catch(error){fs.writeFileSync(path.join(out,'failure.txt'),error.stack);throw error;}
finally{ws?.close();chrome.kill('SIGKILL');server.kill('SIGKILL');await new Promise(r=>http.close(r));fs.writeFileSync(path.join(out,'exceptions.json'),JSON.stringify(exceptions,null,2));}
