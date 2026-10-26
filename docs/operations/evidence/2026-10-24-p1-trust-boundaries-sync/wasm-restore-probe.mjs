// P1 真实生成包探针：坏存档受控拒绝且原句柄仍可读/写。
// 运行：node docs/operations/evidence/2026-10-24-p1-trust-boundaries-sync/wasm-restore-probe.mjs
// 依赖本轮 `node apps/web-client/scripts/build-native.mjs` 生成的真实 wasm-pack 包。
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const pkgRoot = new URL("../../../../apps/web-client/src/worker/native-pkg/", import.meta.url);
const { initSync, NativeEngine } = await import(new URL("engine_native_wasm.js", pkgRoot));
initSync({ module: readFileSync(new URL("engine_native_wasm_bg.wasm", pkgRoot)) });

const engine = new NativeEngine(2, 20261024n);
const before = engine.save_hex();
const results = [];
for (const bad of ["0", "中", "zz", "0g", " "]) {
  let error = "";
  try {
    engine.restore_hex(bad);
  } catch (e) {
    error = String(e);
  }
  results.push({ input: bad, rejected: error !== "", error });
}
const after = engine.save_hex();
const view = engine.view_json(0);
const actions = engine.legal_actions_json(0);
const truncated = before.slice(0, -2);
let truncatedError = "";
try {
  engine.restore_hex(truncated);
} catch (e) {
  truncatedError = String(e);
}
const afterTruncated = engine.save_hex();
let valid = "";
try {
  engine.restore_hex(before);
  valid = "ok";
} catch (e) {
  valid = String(e);
}
console.log(JSON.stringify({
  wasm: fileURLToPath(new URL("engine_native_wasm_bg.wasm", pkgRoot)),
  results,
  before_eq_after: before === after,
  view_ok: view.length > 0,
  actions_ok: actions.length > 0,
  truncated_rejected: truncatedError !== "",
  truncated_error: truncatedError,
  after_truncated_eq_after: afterTruncated === after,
  valid_restore: valid,
}, null, 2));
