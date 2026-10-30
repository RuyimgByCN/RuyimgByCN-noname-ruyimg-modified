import { createServer } from 'node:http';
import { readFileSync, writeFileSync, createWriteStream } from 'node:fs';
import { resolve, join, extname } from 'node:path';
import { spawn } from 'node:child_process';
const root = resolve('../evidence/desktop-webview-dist');
let child;
let finished = false;
const server = createServer((req,res) => {
 if(req.method === 'POST' && req.url === '/report') {
  let body='';req.on('data',data=>body+=data);req.on('end',()=>{
   writeFileSync(resolve('../evidence/desktop-webview-report.json'),body+'\n');
   const report=JSON.parse(body);console.log(JSON.stringify(report,null,2));res.writeHead(204).end();
   finished=true;clearTimeout(timer);setTimeout(()=>{child.kill('SIGTERM');server.close();process.exitCode=report.passed?0:1;},100);
  });return;
 }
 const path = new URL(req.url,'http://localhost').pathname;
 const file = path === '/' ? join(root,'index.html') : join(root,path);
 if(!file.startsWith(root)) {res.writeHead(403).end();return;}
 try {const mime={'.html':'text/html','.js':'text/javascript','.wasm':'application/wasm','.css':'text/css'};res.writeHead(200,{'Content-Type':mime[extname(file)]??'application/octet-stream'}).end(readFileSync(file));}
 catch {res.writeHead(404).end();}
});
await new Promise(resolve=>server.listen(5173,'127.0.0.1',resolve));
const log=createWriteStream(resolve('../evidence/desktop-webview-runtime.log'));
child=spawn('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage3.czy5e9gq/target/debug/noname-next-desktop',[],{stdio:['ignore','pipe','pipe']});
child.stdout.pipe(log);child.stderr.pipe(log);
child.on('exit',code=>{if(!finished)console.error('desktop exited',code);});
const timer=setTimeout(()=>{if(!finished){console.error('WebView acceptance timed out without a report');child.kill('SIGTERM');server.close();process.exitCode=1;}},90000);
