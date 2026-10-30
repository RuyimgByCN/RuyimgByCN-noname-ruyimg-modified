import { build } from '/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client/node_modules/vite/dist/node/index.js';
import { resolve, join } from 'node:path';
import { writeFileSync } from 'node:fs';
const web = resolve('apps/web-client');
const out = resolve('../evidence/desktop-webview-dist');
const result = await build({ root: web, configFile: join(web, 'vite.config.ts'), build: { outDir: out, emptyOutDir: true, rollupOptions: { input: join(web, 'scripts/pa09-desktop-webview.ts') } } });
const entry = result.output.find(file => file.type === 'chunk' && file.isEntry).fileName;
writeFileSync(join(out, 'index.html'), `<!doctype html><html><head><meta charset="utf-8"><script type="module" src="/${entry}"></script></head><body>Running actual IPC and Worker acceptance...</body></html>`);
