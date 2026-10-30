import { createServer } from '/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client/node_modules/vite/dist/node/index.js';
import { spawn } from 'node:child_process';
import { createWriteStream, existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
const tmp = readFileSync('/tmp/b-j2-main-tmp.txt','utf8');
const evidence = resolve('docs/operations/evidence/2026-10-30-j2-main-integration/combined-acceptance');
const web = resolve('apps/web-client');
const server = await createServer({ root:web, configFile:resolve(web,'vite.config.ts'), server:{host:'127.0.0.1',port:4187,strictPort:true} });
let chrome;
try {
  await server.listen();
  const log=createWriteStream(resolve(evidence,'j2-chrome-runtime.log'));
  chrome=spawn('/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',['--headless=new','--disable-gpu','--no-sandbox','--remote-debugging-port=0',`--user-data-dir=${tmp}/j2-chrome-profile-privateport`,'http://127.0.0.1:4187/'],{stdio:['ignore','pipe','pipe']});
  chrome.stdout.pipe(log);chrome.stderr.pipe(log);
  console.log(JSON.stringify({ownedChromePid:chrome.pid,root:web,port:4187}));
  const marker=resolve(tmp,'j2-chrome-profile-privateport/DevToolsActivePort');
  const start=Date.now();
  while(!existsSync(marker)) {
    if(chrome.exitCode!==null)throw Error(`Chrome exited before CDP ready: ${chrome.exitCode}`);
    if(Date.now()-start>30000)throw Error('Chrome CDP startup did not complete');
    await new Promise(resolve=>setTimeout(resolve,100));
  }
  const script=spawn(process.execPath,[resolve(tmp,'runners/headless-theft-private-port.mjs')],{stdio:'inherit'});
  const code=await new Promise((resolve,reject)=>{script.once('error',reject);script.once('exit',resolve);});
  process.exitCode=code===0?0:1;
} finally {
  if(chrome && chrome.exitCode===null) {
    chrome.kill('SIGTERM');
    await new Promise(resolve=>{const timer=setTimeout(()=>{if(chrome.exitCode===null)chrome.kill('SIGKILL');resolve();},5000);chrome.once('exit',()=>{clearTimeout(timer);resolve();});});
  }
  await server.close();
}
