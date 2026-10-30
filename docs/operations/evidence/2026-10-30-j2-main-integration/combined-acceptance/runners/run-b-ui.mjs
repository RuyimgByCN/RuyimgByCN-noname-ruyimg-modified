// Reviewed PA09 run-webview.mjs HTTP report flow; actual Chrome or newly built WKWebView host.
import {createServer} from 'node:http';import {readFileSync,writeFileSync,createWriteStream} from 'node:fs';
import {resolve,join,extname} from 'node:path';import {spawn} from 'node:child_process';
const [mode,binary,run="initial"]=process.argv.slice(2);const root=resolve('/tmp/noname-b-j2-main-2zzoa42o/b-ui-dist');const prefix=resolve(`/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-10-30-j2-main-integration/combined-acceptance/b-ui-${mode}-${run}`);
let child,timer,finished=false;
const server=createServer((req,res)=>{
 if(req.method==='POST'&&req.url==='/report'){let body='';req.on('data',data=>body+=data);req.on('end',()=>{const report=JSON.parse(body);writeFileSync(prefix+'-report.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));res.writeHead(204).end();finished=true;clearTimeout(timer);setTimeout(()=>{child.kill('SIGTERM');server.close();process.exitCode=report.passed?0:1;},100);});return;}
 const path=new URL(req.url,'http://localhost').pathname;
 const file=path==='/metadata-fixtures.json'?resolve('/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-10-30-j2-main-integration/combined-acceptance/b-metadata-fixtures.json'):path==='/'?join(root,'index.html'):join(root,path);
 if(!file.startsWith(root)&&path!=='/metadata-fixtures.json'){res.writeHead(403).end();return;}
 try{const mime={'.html':'text/html','.js':'text/javascript','.wasm':'application/wasm','.css':'text/css','.json':'application/json'};const bytes=readFileSync(file);res.writeHead(200,{'Content-Type':mime[extname(file)]??'application/octet-stream'}).end(bytes);}catch{res.writeHead(404).end();}
});
await new Promise(resolve=>server.listen(5173,'127.0.0.1',resolve));
const args=mode==='chrome'?['--headless=new','--disable-gpu','--no-sandbox',`--user-data-dir=/tmp/noname-b-j2-main-2zzoa42o/b-ui-${mode}-profile`,'http://127.0.0.1:5173/']:[];
const log=createWriteStream(prefix+'-runtime.log');child=spawn(binary,args,{stdio:['ignore','pipe','pipe']});child.stdout.pipe(log);child.stderr.pipe(log);
timer=setTimeout(()=>{if(!finished){console.error('Timed out without actual UI report');child.kill('SIGTERM');server.close();process.exitCode=1;}},90000);
