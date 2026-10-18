// NC03 前端接线：权威查询代价实测（真实 wasm 产物，Node + web 目标 pkg）。
//
// 用途：给「按需查询 + 按 revision 记忆化」的策略一个本机数字——内核侧上轮
// 实测 1.62 ms/次（克隆校验为主），这里量的是经 wasm-bindgen + JSON 投影后
// 前端看到的一次调用代价。
//
// 用法（工作目录 apps/web-client）：
//   node --import ./scripts/ts-register.mjs \
//     ../../docs/operations/evidence/2026-10-18-nc03-frontend-authoritative/probe-query-cost.mjs
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const pkgDir = join(here, "../../../../apps/web-client/src/worker/native-pkg");
const mod = await import(
  pathToFileURL(join(pkgDir, "engine_native_wasm.js")).href
);
await mod.default({
  module_or_path: readFileSync(join(pkgDir, "engine_native_wasm_bg.wasm")),
});

const engine = new mod.NativeEngine(5, 20261018n);
for (let i = 0; i < 40; i++) {
  const view = JSON.parse(engine.view_json(0));
  if (view.phase === "play" && view.turn === 0) break;
  engine.next_phase(view.turn);
}

const warm = JSON.parse(engine.legal_actions_json(0));
const runs = 30;
let total = 0;
for (let i = 0; i < runs; i++) {
  const t0 = performance.now();
  engine.legal_actions_json(0);
  total += performance.now() - t0;
}

// 适配层：同一修订号第二次构建信封应命中缓存（不重复查询）。
const { nativeCreate, nativeEnvelope } = await import(
  pathToFileURL(join(here, "../../../../apps/web-client/src/worker/nativeBackend.ts")).href
);
const handle = nativeCreate(5, 20261018, 0);
const coldStart = performance.now();
nativeEnvelope(handle);
const cold = performance.now() - coldStart;
const warmStart = performance.now();
nativeEnvelope(handle);
const warmEnvelope = performance.now() - warmStart;

console.log(
  JSON.stringify(
    {
      wasm_pkg: "apps/web-client/src/worker/native-pkg (web, release, --no-opt)",
      revision: warm.revision,
      actions: warm.actions.length,
      query_ms_avg: Number((total / runs).toFixed(3)),
      query_runs: runs,
      envelope_cold_ms: Number(cold.toFixed(3)),
      envelope_warm_cached_ms: Number(warmEnvelope.toFixed(3)),
      note: "envelope_cold 含一次查询；envelope_warm_cached 命中 (handle, revision) 记忆化",
    },
    null,
    2,
  ),
);
