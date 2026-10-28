import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { performance } from "node:perf_hooks";
const [repo, label] = process.argv.slice(2);
const root = resolve(repo);
const pkg = await import(pathToFileURL(resolve(root, "apps/web-client/src/worker/native-pkg/engine_native_wasm.js")));
await pkg.default({ module_or_path: readFileSync(resolve(root, "apps/web-client/src/worker/native-pkg/engine_native_wasm_bg.wasm")) });
const backend = await import(pathToFileURL(resolve(root, "apps/web-client/src/worker/nativeBackend.ts")));
const packFile = "/Users/ruyimg/Documents/Claude/sanguosha/noname-next/packages/extensions/huanle-upstream-noname/assets/cards.json";
const packsJson = JSON.stringify([JSON.parse(readFileSync(packFile, "utf8"))]);
for (const [players, packs] of [[4, undefined], [8, undefined], [4, packsJson]]) {
  const handle = packs === undefined ? backend.nativeCreate(players, 4242, 0) : backend.nativeCreateWithPacks(players, 4242, 0, packs);
  const hex = handle.engine.save_hex();
  const hash = handle.engine.state_hash_hex();
  const times = [];
  for (let i = 0; i < 220; i++) {
    const start = performance.now();
    const result = backend.nativeDispatch(handle, { type: "restore", hex, packsJson: packs });
    const elapsed = performance.now() - start;
    if (result.hash_hex !== hash) throw new Error("restore hash changed");
    if (i >= 20) times.push(elapsed);
  }
  times.sort((a, b) => a - b);
  const p = (q) => Math.round(times[Math.ceil(times.length * q) - 1] * 1000) / 1000;
  console.log(JSON.stringify({ label, players, packs: packs === undefined ? "none" : "huanle", samples: times.length, p50_ms: p(.5), p95_ms: p(.95), p99_ms: p(.99) }));
  handle.engine.free();
}
