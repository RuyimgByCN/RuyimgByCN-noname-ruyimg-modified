import path from 'node:path';
import {pathToFileURL} from 'node:url';
const web='/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client';
process.chdir(web);globalThis.__dirname=web;
const {build}=await import(pathToFileURL(path.join(web,'node_modules/vite/dist/node/index.js')).href);
await build({configLoader:'runner',build:{outDir:'/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-10-30-native-ci-corrections/main-acceptance/frontend-products',emptyOutDir:true}});
