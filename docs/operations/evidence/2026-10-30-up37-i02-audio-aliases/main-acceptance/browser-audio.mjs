import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { spawn } from 'node:child_process';
import { readFileSync, writeFileSync, existsSync, mkdtempSync, rmSync } from 'node:fs';
import path from 'node:path';
const repo=path.resolve(process.argv[2]);
const out=path.resolve(process.argv[3]);
const rustBase=process.argv[4];assert.ok(rustBase.startsWith('http://127.0.0.1:'));
const profile=mkdtempSync(path.join(out,'chrome-'));
const mime={'.html':'text/html','.js':'text/javascript','.css':'text/css','.wasm':'application/wasm','.json':'application/json','.jpg':'image/jpeg','.png':'image/png','.mp3':'audio/mpeg'};
const requests=[];
const server=createServer((req,res)=>{
 const url=new URL(req.url,'http://localhost');
 const resource=url.pathname.startsWith('/original-content/');
 assert.equal(resource,false,'Resource requests must go directly to actual Rust HTTP process');
 const root=resource?path.join(repo,'content/original'):path.resolve(out,'../validated-dist');
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
 else if(msg.method==='Fetch.requestPaused'){const p=msg.params;void send('Fetch.continueRequest',{requestId:p.requestId,url:p.request.url.replace('http://127.0.0.1:7700',rustBase)});}
 else if(msg.method==='Runtime.exceptionThrown')errors.push(msg.params);};
 const evaluate=async(expression)=>{const r=await send('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});assert.ok(!r.exceptionDetails,JSON.stringify(r.exceptionDetails));return r.result.value;};
 await send('Runtime.enable');await send('Page.enable');
 await send('Fetch.enable',{patterns:[{urlPattern:'http://127.0.0.1:7700/original-content/*'}]});
 await send('Page.navigate',{url:base+'/collection'});
 for(let i=0;i<100;i++){if(await evaluate("document.querySelectorAll('.character-row').length>0"))break;await wait(100);}
 const matrix=[];
 for(const id of ['bingshi:pot_taishici','bingshi:pot_weiyan','bingshi:pot_yuji','bingshi:pot_zhouyu','standard:liubei','shenhua:liushan','offline:pe_jun_liubei','onlyOL:ol_jianyong','ol_jie_zhuran','dc_sb_zhouyu','star_dongyun','ol_sb_zhouyu','huan_zhouyu','tangtang','ol_sunhanhua']){
  const character=JSON.parse(readFileSync(path.join(repo,'content/original/catalog.json'))).characters.find(c=>c.id===id||c.name===id);
  assert.ok(character,id);const name=character.name;const exact=`${character.pack} · ${name}`;
  await evaluate(`(()=>{const input=document.querySelector('[aria-label="搜索武将"]');input.value=${JSON.stringify(name)};input.dispatchEvent(new Event('input',{bubbles:true}));})()`);
  let found=false;for(let i=0;i<100;i++){found=await evaluate(`Array.from(document.querySelectorAll('.character-row')).some(b=>b.querySelector('small')?.textContent===${JSON.stringify(exact)})`);if(found)break;await wait(50);}assert.ok(found,exact);
  await evaluate(`Array.from(document.querySelectorAll('.character-row')).find(b=>b.querySelector('small')?.textContent===${JSON.stringify(exact)}).click()`);
  for(let i=0;i<100;i++){if(await evaluate(`document.querySelector('.detail header p')?.textContent.includes(${JSON.stringify(exact)})`))break;await wait(50);}
  if(name==='liubei')for(let i=0;i<100;i++){if(await evaluate("document.querySelectorAll('audio').length>0"))break;await wait(50);}
  await wait(100);
  const row=await evaluate(`(async()=>{const image=document.querySelector('.detail header img');if(image&&!image.complete)await image.decode();const audio=[...document.querySelectorAll('audio')];const context=new AudioContext();const decoded=[];for(const a of audio){const response=await fetch(a.src);if(!response.ok)throw new Error('HTTP '+response.status);const bytes=await response.arrayBuffer();const sha256=[...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(b=>b.toString(16).padStart(2,'0')).join('');const byteLength=bytes.byteLength;const buffer=await context.decodeAudioData(bytes);decoded.push({src:a.src,sha256,byteLength,duration:buffer.duration,channels:buffer.numberOfChannels,sampleRate:buffer.sampleRate});}await context.close();return {name:${JSON.stringify(name)},image:image?{loaded:image.complete&&image.naturalWidth>0,width:image.naturalWidth}:null,audio:decoded,status:document.querySelector('.detail .character-status').textContent};})()`);
  if(['tangtang','ol_sunhanhua'].includes(name)){assert.equal(row.audio.length,0);assert.equal(row.status,'待完整移植');}else{assert.ok(row.audio.length>0);if(name==='ol_jie_zhuran')assert.equal(row.image,null);else assert.ok(row.image.loaded);}
  if(name==='ol_jie_zhuran')assert.deepEqual(row.audio.map(a=>a.src.split('/').at(-1)),['xindanshou1.mp3','xindanshou2.mp3']);
  if(name==='dc_sb_zhouyu')assert.equal(row.audio.filter(a=>/dcsb(?:ronghuo|yingmou)_dc_sb_zhouyu_shadow[12]\.mp3$/.test(a.src)).length,4);
  if(id==='bingshi:pot_zhouyu'){assert.ok(row.audio.some(a=>a.src.endsWith('pot_zhouyu_shadow.mp3')));assert.ok(row.audio.some(a=>a.src.endsWith('potchiyun_pot_zhouyu_shadow1.mp3')));}
  row.id=character.id;
  if(id==='standard:liubei'){
   assert.equal(row.audio.length,5);
   const files=row.audio.map(a=>a.src.split('/').at(-1));
   assert.ok(files.includes('jijiang11.mp3'));assert.ok(!files.some(f=>/liushan|re_liubei|sbjijiang/.test(f)),JSON.stringify(files));
   const shot=await send('Page.captureScreenshot',{format:'png'});writeFileSync(path.join(out,'liubei-isolation.png'),Buffer.from(shot.data,'base64'));
  }
  if(id==='shenhua:liushan')assert.ok(!row.audio.some(a=>a.src.includes('jijiang')));
  if(id==='offline:pe_jun_liubei'){assert.ok(row.audio.some(a=>a.src.endsWith('sbjijiang1.mp3')));assert.ok(!row.audio.some(a=>a.src.includes('jijiang1_liushan')));}
  if(id==='onlyOL:ol_jianyong'){assert.ok(row.audio.some(a=>a.src.endsWith('jyzongshi_re_jianyong1.mp3')));assert.ok(!row.audio.some(a=>a.src.endsWith('jyzongshi1.mp3')));}
  const playTargets=name==='liubei'||name==='ol_jie_zhuran'?row.audio.map((a,i)=>i):name==='dc_sb_zhouyu'?row.audio.map((a,i)=>/dcsb(?:ronghuo|yingmou)_dc_sb_zhouyu_shadow[12]\.mp3$/.test(a.src)?i:-1).filter(i=>i>=0):name==='pot_zhouyu'?row.audio.map((a,i)=>/\/(?:pot_zhouyu_shadow|potchiyun_pot_zhouyu_shadow1)\.mp3$/.test(a.src)?i:-1).filter(i=>i>=0):name==='star_dongyun'?[0]:[];
  row.clickedPlayback=[];
  for(const index of playTargets){
   const box=await evaluate(`(()=>{document.querySelectorAll('audio').forEach(a=>a.pause());const a=document.querySelectorAll('audio')[${index}];window.playingEvents=0;a.addEventListener('playing',()=>window.playingEvents++);a.scrollIntoView({block:'center'});const r=a.getBoundingClientRect();return {x:r.x+18,y:r.y+r.height/2};})()`);
   await send('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,...box});await send('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,...box});
   let state;for(let i=0;i<50;i++){state=await evaluate(`(()=>{const a=document.querySelectorAll('audio')[${index}];return {src:a.src,paused:a.paused,time:a.currentTime,duration:a.duration,readyState:a.readyState,error:a.error?.code??null,playingEvents:window.playingEvents,playedRanges:a.played.length};})()`);if(state.time>0&&state.playingEvents>0)break;await wait(100);}
   assert.ok(state.time>0&&state.playingEvents>0&&state.playedRanges>0&&!state.error,JSON.stringify(state));row.clickedPlayback.push(state);
  }
  if(name==='star_dongyun'){const shot=await send('Page.captureScreenshot',{format:'png'});writeFileSync(path.join(out,'collection-audio.png'),Buffer.from(shot.data,'base64'));}
  matrix.push(row);
 }
 assert.deepEqual(errors,[]);
 writeFileSync(path.join(out,'browser-result.json'),JSON.stringify({base,rustBase,sourceRouting:'CDP redirects original-content URL directly to owned actual Rust game-server ServeDir; frontend-only Node HTTP; no resource fabrication or proxy',matrix,errors,requests},null,2)+'\n');
 console.log(JSON.stringify({passed:true,cases:matrix.map(x=>({name:x.name,decoded:x.audio.length,clickedPlayback:x.clickedPlayback}))}));
}finally{ws?.close();chrome.kill('SIGTERM');await new Promise(r=>chrome.once('exit',r));await new Promise(r=>server.close(r));rmSync(profile,{recursive:true,force:true,maxRetries:5,retryDelay:100});}
