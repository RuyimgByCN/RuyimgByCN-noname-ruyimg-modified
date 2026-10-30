import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
const web='/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client';
const out=path.dirname(new URL(import.meta.url).pathname),paths=JSON.parse(fs.readFileSync(path.join(out,'paths.json')));
process.chdir(web);globalThis.__dirname=web;
const {build}=await import(pathToFileURL(path.join(web,'node_modules/vite/dist/node/index.js')).href);
await build({configLoader:'runner',publicDir:path.join(paths.owninput,'public'),resolve:{alias:[{find:'../wasm-pkg',replacement:path.join(paths.owninput,'wasm-pkg')},{find:'./native-pkg',replacement:path.join(paths.owninput,'native-pkg')},{find:path.join(web,'src/wasm-pkg'),replacement:path.join(paths.owninput,'wasm-pkg')},{find:path.join(web,'src/worker/native-pkg'),replacement:path.join(paths.owninput,'native-pkg')}]},build:{outDir:paths.products,emptyOutDir:true}});
