import { createServer } from 'node:http';
import { readFileSync, writeFileSync, createWriteStream } from 'node:fs';
import { resolve, join, extname } from 'node:path';
import { spawn } from 'node:child_process';
const entry = process.argv[2] ?? 'pa09-document-reload';
const run = process.argv[3] ?? 'red';
const binary = process.argv[4] ?? '/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage3.czy5e9gq/target/debug/noname-next-desktop';
const root=resolve(`/tmp/pa09-validation-webview/${entry}-dist`);
const prefix=resolve(`/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-10-30-pa09-desktop-native/validation-${entry}-${run}`);
let child, finished=false, state={ids:[],checks:[]};
const pendingSnapshots=[];
const server=createServer((req,res)=>{
 if(req.url==='/pending') {
  let body='';req.on('data',data=>body+=data);req.on('end',()=>{pendingSnapshots.push(JSON.parse(body));writeFileSync(prefix+'-pending.json',JSON.stringify(pendingSnapshots,null,2)+'\n');res.writeHead(204).end();});return;
 }
 if(req.url==='/state') {
  if(req.method==='GET') {res.writeHead(200,{'Content-Type':'application/json'}).end(JSON.stringify(state));return;}
  let body='';req.on('data',data=>body+=data);req.on('end',()=>{state=JSON.parse(body);writeFileSync(prefix+'-state.json',JSON.stringify(state,null,2)+'\n');res.writeHead(204).end();});return;
 }
 if(req.method==='POST' && req.url==='/report') {
  let body='';req.on('data',data=>body+=data);req.on('end',()=>{
   const report={...JSON.parse(body),state,pendingSnapshots};
   if(entry==='pa09-document-reload' && report.passed) {
    const valid=report.reloads===12 && report.documents===13 && report.nextClientConnected===true && state.ids.length===13
      && state.checks.length===13 && state.checks.every(row=>row.oldResourceRefused && row.unrelatedImageRetained)
      && pendingSnapshots.length===12 && pendingSnapshots.every(row=>row.submitted===10 && row.pendingRequests===8 && row.pendingOpens===2 && row.settledAtNavigation===0);
    if(!valid){report.passed=false;report.harnessError='Reload proof incomplete';}
   }
   writeFileSync(prefix+'-report.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));res.writeHead(204).end();finished=true;clearTimeout(timer);
   setTimeout(()=>{child.kill('SIGTERM');server.close();process.exitCode=report.passed?0:1;},100);
  });return;
 }
 const path=new URL(req.url,'http://localhost').pathname;
 const file=path==='/'?join(root,'index.html'):join(root,path);
 if(!file.startsWith(root)){res.writeHead(403).end();return;}
 try {const mime={'.html':'text/html','.js':'text/javascript','.wasm':'application/wasm','.css':'text/css'};res.writeHead(200,{'Content-Type':mime[extname(file)]??'application/octet-stream'}).end(readFileSync(file));}
 catch {res.writeHead(404).end();}
});
await new Promise(resolve=>server.listen(5173,'127.0.0.1',resolve));
const log=createWriteStream(prefix+'-runtime.log');child=spawn(binary,[],{stdio:['ignore','pipe','pipe']});child.stdout.pipe(log);child.stderr.pipe(log);console.log(JSON.stringify({ownedDesktopPid:child.pid,binary,entry,run}));
const timer=setTimeout(()=>{if(!finished){console.error('Timed out without WebView report');child.kill('SIGTERM');server.close();process.exitCode=1;}},90000);
