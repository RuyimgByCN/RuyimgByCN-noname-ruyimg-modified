import { build } from '/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client/node_modules/vite/dist/node/index.js';
import { resolve, join } from 'node:path';
import { writeFileSync } from 'node:fs';
const entry = process.argv[2] ?? 'pa09-document-reload';
const web = resolve('apps/web-client');
const out = resolve(`/tmp/pa09-validation-webview/${entry}-dist`);
const result = await build({ root: web, configFile: join(web, 'vite.config.ts'), build: { outDir: out, emptyOutDir: true, rollupOptions: { input: join(web, `scripts/${entry}.ts`) } } });
const file = result.output.find(file => file.type === 'chunk' && file.isEntry).fileName;
writeFileSync(join(out, 'index.html'), `<!doctype html><html><head><meta charset="utf-8"><script type="module" src="/${file}"></script></head><body>PA09 document lifecycle acceptance</body></html>`);
