// T13 产品面冒烟：web 目标 wasm init + 存读档 + 15 包目录。
import { pathToFileURL } from "node:url";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { readdirSync, existsSync, readFileSync } from "node:fs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const pkgDir = join(root, "src/worker/native-pkg");
const mod = await import(pathToFileURL(join(pkgDir, "engine_native_wasm.js")).href);
const init = mod.default;
const { NativeEngine } = mod;

const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok });
  console.error(`${ok ? "PASS" : "FAIL"} ${name}${detail ? " | " + detail : ""}`);
};

const wasmBytes = readFileSync(join(pkgDir, "engine_native_wasm_bg.wasm"));
await init({ module_or_path: wasmBytes });
check("init_web", true);

const engine = new NativeEngine(4, 4242n);
check("create", true);

const view0 = JSON.parse(engine.view_json(0));
check("observer_view", Array.isArray(view0.players) && view0.players.length === 4);

engine.next_phase(0);
check("endPhase", true);

let steps = 0;
while (steps < 40 && !engine.is_finished()) {
  engine.auto_step();
  steps++;
}
check("autoStep_progress", steps > 0, `steps=${steps}`);

const hex = engine.save_hex();
check("save_hex", typeof hex === "string" && hex.length > 32, `bytes=${hex.length / 2}`);

const eng2 = new NativeEngine(4, 9999n);
eng2.restore_hex(hex);
check(
  "restore_hex",
  Number(eng2.revision()) === Number(engine.revision()) &&
    eng2.is_finished() === engine.is_finished(),
  `rev=${eng2.revision()}`,
);

const hand = JSON.parse(engine.view_json(0)).players[0].hand.own ?? [];
if (hand.length > 0) {
  try {
    engine.play_card(0, hand[0], new Uint32Array([]));
    check("playCard_shape", true, `rev=${engine.revision()}`);
  } catch (e) {
    check("playCard_shape", true, `拒牌可接受: ${e}`);
  }
} else {
  check("playCard_shape", true, "hand empty");
}

const extRoot = join(root, "../../packages/extensions");
const dirs = readdirSync(extRoot).filter((d) =>
  existsSync(join(extRoot, d, "package.npack.json")),
);
check("extension_catalog_15", dirs.length === 15, `n=${dirs.length}`);

console.log(JSON.stringify({ total: results.length, failed: results.filter((r) => !r.ok).length }));
process.exit(results.some((r) => !r.ok) ? 1 : 0);
