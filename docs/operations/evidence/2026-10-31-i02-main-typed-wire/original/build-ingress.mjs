import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {pathToFileURL} from 'node:url';
const E='/tmp/i02-main-typed-wire-functional-33d21ba5',web='/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client';
process.chdir(web);globalThis.__dirname=web;const {build}=await import(pathToFileURL(web+'/node_modules/vite/dist/node/index.js'));
const sources={};const result=await build({configFile:web+'/vite.config.ts',configLoader:'runner',plugins:[{name:'i02-ingress-source-binding',enforce:'pre',transform(code,id){if(id.startsWith(web+'/src/')&&!id.includes('?')&&fs.existsSync(id))sources[id]=crypto.createHash('sha256').update(fs.readFileSync(id)).digest('hex');}}],build:{outDir:E+'/ingress-dist',emptyOutDir:true,rollupOptions:{input:E+'/ingress.ts'}}});
const entry=result.output.find(x=>x.type==='chunk'&&x.isEntry).fileName;fs.writeFileSync(E+'/ingress-dist/index.html',`<!doctype html><meta charset="utf-8"><script type="module" src="/${entry}"></script><body>I02 private transport acceptance</body>`);fs.writeFileSync(E+'/ingress-bundle-sources.json',JSON.stringify({sources,entry,qualification:'Private injected corpus entry using actual current production modules; original Main production UI bundle is separately built/tested.'},null,2));console.log(JSON.stringify({entry,sources:Object.keys(sources).length}));
