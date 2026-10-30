import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath, pathToFileURL } from 'node:url';
const out = path.dirname(fileURLToPath(import.meta.url));
const paths = JSON.parse(fs.readFileSync(path.join(out, 'paths.json'), 'utf8'));
const web = '/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client';
const sha = value => crypto.createHash('sha256').update(value).digest('hex');
const sourceModules = {};
process.chdir(web);
globalThis.__dirname = web;
const { build } = await import(pathToFileURL(path.join(web, 'node_modules/vite/dist/node/index.js')).href);
await build({
  configLoader: 'runner',
  publicDir: path.join(paths.owninput, 'public'),
  resolve: { alias: [
    { find: '../wasm-pkg', replacement: path.join(paths.owninput, 'wasm-pkg') },
    { find: './native-pkg', replacement: path.join(paths.owninput, 'native-pkg') },
    { find: path.join(web, 'src/wasm-pkg'), replacement: path.join(paths.owninput, 'wasm-pkg') },
    { find: path.join(web, 'src/worker/native-pkg'), replacement: path.join(paths.owninput, 'native-pkg') },
  ] },
  plugins: [{
    name: 'pa09-main-source-evidence',
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
  build: { outDir: paths.products, emptyOutDir: true },
});
const transport = sourceModules['src/engine/desktopEngineTransport.ts'];
if (!transport || transport.sourceSha256 !== transport.transformInputSha256 || transport.sourceSha256 !== '6224bdc95960bdca458693a67176af6936b12d28edbc256e105f6a283197face') throw new Error('Frozen Main transport was not an exact Vite input');
for (const p of ['src/main.ts', 'src/engine/client.ts']) if (!sourceModules[p]) throw new Error(`Main production module missing: ${p}`);
console.log(JSON.stringify({ exactFrozenTransport: true, tracedSourceModules: Object.keys(sourceModules).length, productionOutDir: paths.products, publicInputs: 'Own copies of public/pkg only; portraits/media are outside this JS slice.' }));
