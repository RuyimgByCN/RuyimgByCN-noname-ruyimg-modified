// NC12 旧线 WASM（engine-wasm）档位基准：建局 / 提交 / 观察者视图 / 存档字段。
//
// 口径（方案 §NC12）：旧线（engine-wasm）与新线（engine-native-wasm）**分档并列**，
// 不跨线计算收益；本脚本只测**既有产物**（crates/engine-wasm/pkg-node/），Node 下驱动。
//
// 用法：node crates/engine-wasm/scripts/nc12-old-wasm-bench.mjs [iters]

import { readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const pkgDir = join(root, "pkg-node");
const iters = Number(process.argv[2] ?? 30);

const mod = await import(join(pkgDir, "engine_wasm.js"));
mod.start?.();

function pct(samples, p) {
  const v = [...samples].sort((a, b) => a - b);
  return v[Math.min(v.length - 1, Math.round((v.length - 1) * p))];
}
function bench(label, fn) {
  const samples = [];
  for (let i = 0; i < iters; i++) {
    const t0 = process.hrtime.bigint();
    fn();
    samples.push(Number(process.hrtime.bigint() - t0));
  }
  console.log(
    `RESULT scenario=old_wasm metric=${label} n=${samples.length} p50_ns=${pct(samples, 0.5)} p95_ns=${pct(samples, 0.95)} max_ns=${Math.max(...samples)} mean_ns=${Math.round(samples.reduce((a, b) => a + b, 0) / samples.length)}`,
  );
  return samples;
}

bench("build_and_start", () => {
  const e = new mod.WasmEngine();
  e.loadEmbeddedStandard();
  e.startGame(4, 20263300);
  e.free();
});

const e = new mod.WasmEngine();
e.loadEmbeddedStandard();
e.startGame(4, 20263301);
bench("end_phase", () => {
  e.endPhase(1);
});
bench("observer_json", () => {
  e.observerJson();
});
bench("state_hash_hex", () => {
  e.stateHashHex();
});
bench("auto_step", () => {
  e.autoStep();
});
e.free();
console.log("NOTE 旧线 WASM 档（既有产物）；与新线 wasm 档并列记录，不跨线计算收益。");
