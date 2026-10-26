// 真实生成包（npack + characters 资产）经 Worker API 安装并运行 VM 技能；
// 不是 host 包装层：加载 apps/web-client/src/worker/native-pkg 的本轮产物。
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const pkgRoot = new URL("../../../../apps/web-client/src/worker/native-pkg/", import.meta.url);
const { initSync, NativeEngine } = await import(new URL("engine_native_wasm.js", pkgRoot));
initSync({ module: readFileSync(new URL("engine_native_wasm_bg.wasm", pkgRoot)) });

const npack = JSON.parse(readFileSync(new URL("./worker-vm.npack.json", import.meta.url)));
const characters = JSON.parse(readFileSync(new URL("./characters-asset.json", import.meta.url)));
npack.characters = characters;
const engine = NativeEngine.new_with_packs_json(4, 42n, JSON.stringify([npack]));

const candidates = JSON.parse(engine.general_candidates_json()).candidates ?? [];
const picked = candidates.map((c) => c.name).find((name) => name.includes("worker_vm_probe"));
if (!picked) throw new Error(`worker_vm_probe missing: ${JSON.stringify(candidates).slice(0, 300)}`);
engine.assign_general(0, picked);
const handBefore = JSON.parse(engine.view_json(0)).players[0].handCount;
for (let i = 0; i < 8; i += 1) {
  const choice = JSON.parse(engine.open_choice_json(0));
  if (choice) {
    const take = Math.min(Number(choice.min), choice.options.length);
    const indices = new Uint32Array(take);
    for (let j = 0; j < take; j += 1) indices[j] = j;
    engine.answer(Number(choice.observer), BigInt(choice.id), indices);
  } else {
    engine.next_phase(0);
  }
  if (JSON.parse(engine.view_json(0)).players[0].handCount > handBefore) break;
}
const handAfter = JSON.parse(engine.view_json(0)).players[0].handCount;
if (handAfter <= handBefore) throw new Error(`VM skill did not draw: ${handBefore} -> ${handAfter}`);
console.log(JSON.stringify({
  general: picked,
  hand_before: handBefore,
  hand_after: handAfter,
  revision: Number(engine.revision()),
  hash: engine.state_hash_hex(),
  wasm: fileURLToPath(new URL("engine_native_wasm_bg.wasm", pkgRoot)),
}));
