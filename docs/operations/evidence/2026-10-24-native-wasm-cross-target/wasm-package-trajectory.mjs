// 正式安装包（huanle-cards）的真实 WASM 轨迹；与 native test 用同一 packs.json。
// 运行：node .../wasm-package-trajectory.mjs <packs.json> <output.json>
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const pkgRoot = new URL("../../../../apps/web-client/src/worker/native-pkg/", import.meta.url);
const { initSync, NativeEngine } = await import(new URL("engine_native_wasm.js", pkgRoot));
initSync({ module: readFileSync(new URL("engine_native_wasm_bg.wasm", pkgRoot)) });

const packs = readFileSync(process.argv[2], "utf8");
const SEED = 20_261_025n;
const TOTAL = 20;
const MIDPOINT = 10;

function checkAccepted(raw, step) {
  const outcome = JSON.parse(raw);
  if (outcome.outcome !== "accepted" && outcome.outcome !== "pending") {
    throw new Error(`package wasm step ${step} rejected: ${raw}`);
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
      checkAccepted(engine.answer(Number(choice.observer), BigInt(choice.id), indices), index);
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

const directEngine = NativeEngine.new_with_packs_json(1, SEED, packs);
const direct = run(directEngine, 0, TOTAL);
const first = NativeEngine.new_with_packs_json(1, SEED, packs);
const before = run(first, 0, MIDPOINT);
const saved = first.save_hex();
const restoredEngine = NativeEngine.new_with_packs_json(1, SEED, packs);
restoredEngine.restore_hex_with_packs_json(saved, packs);
const after = run(restoredEngine, MIDPOINT, TOTAL);

const payload = {
  seed: Number(SEED),
  players: 1,
  package: "huanle-cards",
  direct,
  restore: { midpoint: MIDPOINT, before, after },
  wasm: fileURLToPath(new URL("engine_native_wasm_bg.wasm", pkgRoot)),
};
writeFileSync(process.argv[3], JSON.stringify(payload, null, 2) + "\n");
console.log(JSON.stringify({ steps: direct.length, final_hash: direct.at(-1).hash }));
