// Same Vite build API/flags as reviewed PA09 build-webview.mjs; paths point to this checkout/evidence.
import {build} from '/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client/node_modules/vite/dist/node/index.js';
import {resolve,join} from 'node:path';import {writeFileSync} from 'node:fs';
const root=resolve('apps/web-client');const out=resolve('../evidence/approved-b-ui-dist-readable-final');
const result=await build({root,configFile:join(root,'vite.config.ts'),build:{outDir:out,emptyOutDir:false,rollupOptions:{input:join(root,'scripts/approved-b-ui.ts')}}});
const file=result.output.find(file=>file.type==='chunk'&&file.isEntry).fileName;
writeFileSync(join(out,'index.html'),`<!doctype html><html><head><meta charset="utf-8"><script type="module" src="/${file}"></script></head><body>Approved B actual UI acceptance</body></html>`);
