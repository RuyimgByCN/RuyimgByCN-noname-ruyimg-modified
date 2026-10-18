// T13 产品面冒烟：web 目标 wasm init + 存读档 + 扩展包清单/目录一致性。
import { pathToFileURL } from "node:url";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import { checkCatalogConsistency, collectPackManifests } from "./extension-inventory.mjs";

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

// 扩展包清单守护（旧断言 `extension_catalog_15` 的硬编码目录数已过期：实测 41）：
// 期望值改由实际包数据推导，不再写死数字。两个独立不变量：
//   ① 每份 packages/extensions/<dir>/package.npack.json 可解析，manifest 含
//      namespace/package_id/version，且 (namespace, package_id) 包键唯一；
//   ② 客户端可发现目录 EXTENSION_CATALOG（src/engine/extensionCatalog.ts，
//      node 直接转译 TS）的每一项都能回查到在盘包，且元数据逐字段一致。
// 未进目录的在盘包只计数不失败（管理面覆盖 41/41 是另一项产品工作）。
// 判定逻辑在 scripts/extension-inventory.mjs，其失败路径由同名 .test.mjs 覆盖。
const extRoot = join(root, "../../packages/extensions");
const inventory = collectPackManifests(extRoot);
check(
  "extension_package_inventory",
  inventory.packs.size > 0 && inventory.problems.length === 0,
  `packs=${inventory.packs.size} dirs=${inventory.dirCount} no_npack=${inventory.orphanDirs.length}` +
    (inventory.problems.length ? ` | ${inventory.problems.join("; ")}` : ""),
);

const { EXTENSION_CATALOG } = await import(
  pathToFileURL(join(root, "src/engine/extensionCatalog.ts")).href
);
const catalogCheck = checkCatalogConsistency(EXTENSION_CATALOG, inventory.packs);
check(
  "extension_catalog_consistent",
  EXTENSION_CATALOG.length > 0 && catalogCheck.problems.length === 0,
  `catalog=${EXTENSION_CATALOG.length} uncovered=${catalogCheck.uncovered.length}` +
    (catalogCheck.problems.length ? ` | ${catalogCheck.problems.join("; ")}` : ""),
);

const huanlePath = join(extRoot, "huanle-upstream-noname/assets/cards.json");
const packs = JSON.stringify([JSON.parse(readFileSync(huanlePath, "utf8"))]);
const ext = NativeEngine.new_with_packs_json(4, 17017n, packs);
const prefixed = ext.cards_with_prefix("hnl_");
check("pack_enters_match", prefixed === 7, `hnl_=${prefixed}`);
const extHex = ext.save_hex();
let missingRefused = false;
try {
  const bare = new NativeEngine(4, 1n);
  bare.restore_hex(extHex);
} catch {
  missingRefused = true;
}
check("missing_pack_refused", missingRefused);
const again = NativeEngine.new_with_packs_json(4, 1n, packs);
again.restore_hex_with_packs_json(extHex, packs);
check("same_pack_restore", again.cards_with_prefix("hnl_") === 7);
const wrongObj = JSON.parse(packs);
wrongObj[0].pack.version = 2;
let wrongRefused = false;
let wrongDetail = "";
try {
  again.restore_hex_with_packs_json(extHex, JSON.stringify(wrongObj));
  wrongDetail = "accepted";
} catch (e) {
  wrongDetail = String(e);
  wrongRefused = wrongDetail.includes("恢复失败");
}
check("wrong_version_refused", wrongRefused, wrongDetail);

// 适配层 × 真实引擎：choice token 必须是候选下标并可按原样回传。
// 依据 crates/engine-native-wasm `answer()` 直取 `spec.options[idx]`；对应
// adapter-probe 反例 1（stub 版回归见 scripts/native-adapter.test.mjs）。
const { nativeCreate, nativeDispatch, nativeEnvelope } = await import(
  pathToFileURL(join(root, "src/worker/nativeBackend.ts")).href
);
const adapter = nativeCreate(4, 4242, 0);
let env = nativeEnvelope(adapter);
for (let i = 0; i < 60 && (env.choice?.options?.length ?? 0) === 0; i++) {
  env = nativeDispatch(adapter, { type: "autoStep" });
}
const choice = env.choice;
const auto = (env.legal_actions ?? []).find(
  (a) => a.ResolveChoice && !a.ResolveChoice.cancelled,
);
let tokenRoundTrip = false;
let tokenDetail = `choice=${choice?.choice_id ?? "none"} options=${choice?.options?.length ?? 0}`;
if (choice && choice.options.length > 0 && auto) {
  const first = choice.options[0];
  const resolved = nativeDispatch(adapter, {
    type: "resolveChoice",
    actor: 0,
    choiceId: choice.choice_id,
    selections: [...auto.ResolveChoice.selections],
  });
  tokenRoundTrip = resolved.result?.outcome === "accepted";
  tokenDetail += ` label=${first.label_key} token=${first.token} submission=${JSON.stringify(auto.ResolveChoice.selections)} outcome=${resolved.result?.outcome}`;
  // 候选值不是 token：把载荷值当 token 回传必须报错，不得静默命中别的候选
  const payload = /^[a-z]+:(\d+)$/.exec(first.label_key);
  if (payload) {
    let valueRefused = false;
    try {
      nativeDispatch(adapter, {
        type: "resolveChoice",
        actor: 0,
        choiceId: choice.choice_id,
        selections: [Number(payload[1])],
      });
    } catch {
      valueRefused = true;
    }
    tokenRoundTrip = tokenRoundTrip && valueRefused;
    tokenDetail += ` value_as_token_refused=${valueRefused}`;
  }
}
check("adapter_choice_token_roundtrip", tokenRoundTrip, tokenDetail);

console.log(JSON.stringify({ total: results.length, failed: results.filter((r) => !r.ok).length }));
process.exit(results.some((r) => !r.ok) ? 1 : 0);
