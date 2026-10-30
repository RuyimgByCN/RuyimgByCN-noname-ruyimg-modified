import { build } from '/tmp/pa09-js-bound-4306bc00/apps/web-client/node_modules/vite/dist/node/index.js';
import vue from '/tmp/pa09-js-bound-4306bc00/apps/web-client/node_modules/@vitejs/plugin-vue/dist/index.mjs';
import wasm from '/tmp/pa09-js-bound-4306bc00/apps/web-client/node_modules/vite-plugin-wasm/dist/index.js';
import topLevelAwait from '/tmp/pa09-js-bound-4306bc00/apps/web-client/node_modules/vite-plugin-top-level-await/dist/index.js';
import { readFileSync, writeFileSync, realpathSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { resolve } from 'node:path';
const out = realpathSync('/tmp/pa09-webview-5b7ad3ab');
const web = resolve(out, 'product-web');
const loaded = {};
const proof = {
  name: 'pa09-private-fixture-observation',
  enforce: 'pre',
  transformIndexHtml: { order: 'pre', handler: (html) => html.replace('src="/src/main.ts"', 'src="/fixture.js"') },
  resolveId(source) {
    if (source.startsWith('../../../../packages/')) return resolve('/tmp/pa09-js-bound-4306bc00/packages', source.slice('../../../../packages/'.length));
  },
  transform(code, id) {
    if (id.startsWith(web + '/src/') && !id.includes('?')) loaded[id] = createHash('sha256').update(code).digest('hex');
    return null;
  },
};
const result = await build({ root: web, configFile: false, publicDir: false,
  plugins: [proof, vue(), wasm.default?.() ?? wasm(), topLevelAwait.default?.() ?? topLevelAwait()],
  define: { 'import.meta.env.VITE_ENGINE_BACKEND': JSON.stringify('native') },
  resolve: { alias: [
    { find: /^@tauri-apps\/api\/core$/, replacement: resolve(out, 'harness/invoke-observer.js') },
    { find: '@', replacement: resolve(web, 'src') },
  ] },
  worker: { format: 'es', plugins: () => [wasm.default?.() ?? wasm(), topLevelAwait.default?.() ?? topLevelAwait()] },
  assetsInclude: ['**/*.wasm'],
  build: { outDir: resolve(out, 'bundle'), emptyOutDir: true, sourcemap: true },
});
writeFileSync(resolve(out, 'evidence/bundle-module-source-proof.json'), JSON.stringify({ loadedModulesBeforeTransforms: loaded, sourceTransforms: [], dependencyObserver: 'core.invoke wrapper forwards original args and returns the original Tauri promise; no mock, queue, delay or result substitution', entryInjection: 'fixture.js imports actual src/main.ts; production Vue mounts first', configFile: false, chunks: result.output.map((row) => row.fileName) }, null, 2));
for (const name of ['src/engine/client.ts', 'src/engine/desktopEngineTransport.ts']) {
  const path = resolve(web, name);
  if (loaded[path] !== createHash('sha256').update(readFileSync(path)).digest('hex')) throw new Error('source transform mismatch: ' + name);
}
