import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { pathToFileURL } from 'node:url';
const web = '/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client';
const out = '/tmp/nc27-main-functional-458ed333';
const sha = value => crypto.createHash('sha256').update(value).digest('hex');
const sourceModules = {};
process.chdir(web);
globalThis.__dirname = web;
const { build } = await import(pathToFileURL(path.join(web, 'node_modules/vite/dist/node/index.js')).href);
await build({
  configFile: path.join(web, 'vite.config.ts'),
  configLoader: 'runner',
  plugins: [{
    name: 'nc27-main-input-evidence',
    enforce: 'pre',
    transform(code, id) {
      if (id.startsWith(path.join(web, 'src/')) && !id.includes('?') && fs.existsSync(id)) {
        sourceModules[path.relative(web, id)] = { sourceSha256: sha(fs.readFileSync(id)), transformInputSha256: sha(code) };
      }
    },
    writeBundle(_options, bundle) {
      const chunks = Object.fromEntries(Object.entries(bundle).map(([name, item]) => [name, item.type === 'chunk' ? { type: item.type, isEntry: item.isEntry, facadeModuleId: item.facadeModuleId, modules: Object.keys(item.modules), outputSha256: sha(item.code) } : { type: item.type, outputSha256: sha(item.source) }]));
      fs.writeFileSync(path.join(out, 'bundle-module-table.json'), JSON.stringify({ sourceModules, chunks }, null, 2) + '\n');
    },
  }],
  build: { outDir: path.join(out, 'frontend-dist'), emptyOutDir: true },
});
for (const p of ['src/main.ts', 'src/engine/client.ts']) if (!sourceModules[p]) throw new Error(`Main production module missing: ${p}`);
console.log(JSON.stringify({ tracedSourceModules: Object.keys(sourceModules).length, productionOutDir: path.join(out, 'frontend-dist'), mainSource: web, configLoader: 'runner', originalConfigDirname: globalThis.__dirname, generatedModules: 'Existing Main read-only inputs; current actual WASM execution not qualified' }));
