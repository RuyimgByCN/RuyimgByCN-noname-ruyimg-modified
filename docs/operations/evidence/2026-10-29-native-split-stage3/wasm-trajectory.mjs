// 用本轮生成的真实 wasm 包跑与 native 测试相同的种子/命令，输出轨迹 JSON。
// 运行：node .../wasm-trajectory.mjs <output.json>
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const pkgRoot = new URL("../native-pkg/", import.meta.url);
const { initSync, NativeEngine } = await import(new URL("engine_native_wasm.js", pkgRoot));
initSync({ module: readFileSync(new URL("engine_native_wasm_bg.wasm", pkgRoot)) });

const SEED = 20_261_024n;
const TOTAL = 40;
const MIDPOINT = 20;

function checkAccepted(raw, step) {
  const outcome = JSON.parse(raw);
  if (outcome.outcome !== "accepted" && outcome.outcome !== "pending") {
    throw new Error(`wasm step ${step} rejected: ${raw}`);
  }
}

function run(engine, start, end) {
  const records = [];
  let cursor = Number(engine.log_len());
  for (let index = start; index < end; index += 1) {
    const choice = JSON.parse(engine.open_choice_json(0));
    if (choice) {
      const take = Math.min(Number(choice.min), choice.options.length);
      const indices = new Uint32Array(take);
      for (let i = 0; i < take; i += 1) indices[i] = i;
      checkAccepted(
        engine.answer(Number(choice.observer), BigInt(choice.id), indices),
        index,
      );
    } else {
      checkAccepted(engine.next_phase(0), index);
    }
    const next = Number(engine.log_len());
    const codes = JSON.parse(engine.public_event_codes_json(cursor));
    cursor = next;
    records.push({
      revision: Number(engine.revision()),
      hash: engine.state_hash_hex(),
      codes,
    });
  }
  return records;
}

const directEngine = new NativeEngine(1, SEED);
const direct = run(directEngine, 0, TOTAL);

const first = new NativeEngine(1, SEED);
const before = run(first, 0, MIDPOINT);
const saved = first.save_hex();
const restoredEngine = new NativeEngine(1, SEED);
restoredEngine.restore_hex(saved);
const after = run(restoredEngine, MIDPOINT, TOTAL);

const payload = {
  seed: Number(SEED),
  players: 1,
  direct,
  restore: { midpoint: MIDPOINT, before, after },
  wasm: fileURLToPath(new URL("engine_native_wasm_bg.wasm", pkgRoot)),
};
writeFileSync(process.argv[2], JSON.stringify(payload, null, 2) + "\n");
console.log(JSON.stringify({ steps: direct.length, final_hash: direct.at(-1).hash, wasm: payload.wasm }));
