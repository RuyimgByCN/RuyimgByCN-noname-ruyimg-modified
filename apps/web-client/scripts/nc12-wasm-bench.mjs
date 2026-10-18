// NC12 目标档位测量：**WASM 产物**（非浏览器）下的建局/查询/存读档/视图耗时。
//
// 口径（方案 §NC12）：同设备、同工具链、同负载；本脚本测量的是**既有 WASM 产物**
// （`src/worker/native-pkg/`），不是重新构建的源码，也不是浏览器运行时——两层边界
// 分别记录（见证据 README）。
//
// 用法：node apps/web-client/scripts/nc12-wasm-bench.mjs [iters]

import { readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const pkgDir = join(root, "src/worker/native-pkg");
const iters = Number(process.argv[2] ?? 30);

const mod = await import(join(pkgDir, "engine_native_wasm.js"));
mod.initSync({ module: readFileSync(join(pkgDir, "engine_native_wasm_bg.wasm")) });

// 一个最小扩展包（与 NC08 跨目标门禁用例同形）
const PACKS = JSON.stringify([
  {
    pack: { name: "nc12-wasm", version: 1, description: "nc12", card_prefix: "nc12w_" },
    cards: [
      { name: "extra", kind: "sha", suit: "spade", rank: 7 },
      { name: "extra2", kind: "shan", suit: "heart", rank: 2 },
    ],
  },
]);

function pct(samples, p) {
  if (samples.length === 0) return 0;
  const v = [...samples].sort((a, b) => a - b);
  const idx = Math.round((v.length - 1) * p);
  return v[idx];
}

function bench(label, fn) {
  const samples = [];
  for (let i = 0; i < iters; i++) {
    const t0 = process.hrtime.bigint();
    fn();
    samples.push(Number(process.hrtime.bigint() - t0));
  }
  console.log(
    `RESULT scenario=wasm_bench tier=wasm metric=${label} n=${samples.length} p50_ns=${pct(samples, 0.5)} p95_ns=${pct(samples, 0.95)} max_ns=${Math.max(...samples)} mean_ns=${Math.round(samples.reduce((a, b) => a + b, 0) / samples.length)}`,
  );
  return samples;
}

// ⓪ 冷启动：本进程从「已 initSync」到「首次可操作」的墙钟时间（含首次建局与首次查询）
{
  const t0 = process.hrtime.bigint();
  const e0 = new mod.NativeEngine(4, 20262500n);
  e0.legal_actions_json(0);
  const cold = Number(process.hrtime.bigint() - t0);
  e0.free();
  console.log(`RESULT scenario=wasm_bench tier=wasm metric=cold_start_first_usable n=1 p50_ns=${cold} p95_ns=${cold} max_ns=${cold} mean_ns=${cold}`);
}

// ① 建局（含扩展包）
bench("build_with_packs", () => {
  const e = mod.NativeEngine.new_with_packs_json(4, 20262501n, PACKS);
  e.free();
});
// ② 建局（标准目录）
bench("build_standard", () => {
  const e = new mod.NativeEngine(4, 20262502n);
  e.free();
});

const engine = mod.NativeEngine.new_with_packs_json(4, 20262503n, PACKS);
// ③ 权威动作查询
bench("legal_actions", () => {
  engine.legal_actions_json(0);
});
// ④ 座位视图
bench("view_json", () => {
  engine.view_json(0);
});
// ⑤ 存档
let hex = "";
bench("save_hex", () => {
  hex = engine.save_hex();
});
// ⑥ 读档（扩展局：必须走 with_packs 入口——标准入口对扩展快照会明确失败）
bench("restore_hex_with_packs", () => {
  const e = mod.NativeEngine.new_with_packs_json(4, 20262503n, PACKS);
  e.restore_hex_with_packs_json(hex, PACKS);
  e.free();
});
// ⑥b 反例：标准入口恢复扩展快照 → **必须失败**（门禁有分辨力）
try {
  const e = mod.NativeEngine.new_with_packs_json(4, 20262503n, PACKS);
  e.restore_hex(hex);
  e.free();
  console.log("RESULT scenario=wasm_bench tier=wasm metric=restore_wrong_catalog result=UNEXPECTED_OK");
} catch (err) {
  console.log(`RESULT scenario=wasm_bench tier=wasm metric=restore_wrong_catalog result=REJECTED err=${String(err).slice(0, 80)}`);
}
// ⑦ 选将候选
bench("general_candidates_json", () => {
  engine.general_candidates_json();
});
engine.free();
console.log("NOTE 本脚本测量既有 WASM 产物（非浏览器、非本次源码重建）；同设备同负载，绝对值受并发负载影响。");
