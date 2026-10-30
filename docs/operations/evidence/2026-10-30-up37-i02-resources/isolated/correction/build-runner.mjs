// The existing config uses __dirname. Supply its normal value while using the
// runner loader so Vite does not write a bundled config into shared node_modules.
import path from "node:path";
import { pathToFileURL } from "node:url";
const web = path.resolve(process.argv[2]);
const out = path.resolve(process.argv[3]);
process.chdir(web);
globalThis.__dirname = web;
const { build } = await import(pathToFileURL(path.join(web, "node_modules/vite/dist/node/index.js")).href);
await build({ configLoader: "runner", build: { outDir: out } });
