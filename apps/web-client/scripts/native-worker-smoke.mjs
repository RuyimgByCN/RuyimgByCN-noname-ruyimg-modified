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
const { chooseGeneral, generalCandidates, nativeCreate, nativeDispatch, nativeEnvelope } = await import(
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

// ── NC03 权威动作查询：候选来自内核，且逐项可被真实内核接受 ──
// 修复前适配层用 `hand.slice(0, 8)` 拼 `PlayCard`、合成 `ResolveChoice`；
// 现在 `legal_actions` 必须逐项等于窄壳 `legal_actions_json` 的输出。
// 先自动推进到 0 号座位的出牌阶段：那里才有出牌/主动技候选（开局只有阶段推进）。
const auth = nativeCreate(4, 4242, 0);
let authEnv = nativeEnvelope(auth);
for (let i = 0; i < 40; i++) {
  if (authEnv.view.phase === "Play" && authEnv.view.current_player === 0) break;
  authEnv = nativeDispatch(auth, { type: "autoStep" });
}
const authParsed = JSON.parse(auth.engine.legal_actions_json(0));
check(
  "legal_actions_shape",
  authParsed.seat === 0 &&
    Number.isInteger(authParsed.revision) &&
    Array.isArray(authParsed.actions),
  `seat=${authParsed.seat} revision=${authParsed.revision} n=${authParsed.actions?.length}`,
);
check(
  "legal_actions_revision_matches_engine",
  authParsed.revision === Number(auth.engine.revision()),
  `json=${authParsed.revision} engine=${auth.engine.revision()}`,
);
check(
  "legal_actions_non_empty",
  authParsed.actions.length > 0,
  `n=${authParsed.actions.length} kinds=${JSON.stringify(
    authParsed.actions.map((a) => (typeof a === "string" ? a : Object.keys(a)[0])),
  )}`,
);
check(
  "adapter_passes_kernel_actions_verbatim",
  JSON.stringify(authEnv.legal_actions) === JSON.stringify(authParsed.actions),
  `adapter=${JSON.stringify(authEnv.legal_actions)} kernel=${JSON.stringify(authParsed.actions)}`,
);

/** 把一条权威候选按界面协议提交（与 BattleView/BattleBoard 同款映射）。 */
function submitCandidate(handle, action) {
  if (action === "EndPhase") return nativeDispatch(handle, { type: "endPhase" });
  if (action.PlayCard) {
    return nativeDispatch(handle, {
      type: "playCard",
      actor: 0,
      card: action.PlayCard.card,
      targets: action.PlayCard.targets,
    });
  }
  if (action.ActivateSkill) {
    return nativeDispatch(handle, {
      type: "activateSkill",
      actor: 0,
      skill: action.ActivateSkill.skill,
    });
  }
  return nativeDispatch(handle, {
    type: "resolveChoice",
    actor: 0,
    choiceId: action.ResolveChoice.choice_id,
    selections: action.ResolveChoice.selections,
    cancelled: action.ResolveChoice.cancelled,
  });
}

// P2：Worker 信封必须提供真实权威 hash 与公开事件码（不再为空占位）。
const eventProbe = nativeCreate(4, 4242, 0);
nativeDispatch(eventProbe, { type: "restore", hex: auth.engine.save_hex() });
const eventEnv = nativeDispatch(eventProbe, { type: "endPhase", actor: 0 });
const eventCodes = (eventEnv.events ?? [])
  .map((event) => event?.PublicEvent?.code)
  .filter((code) => Number.isInteger(code));
check(
  "adapter_hash_hex_authoritative",
  /^[0-9a-f]{64}$/.test(eventEnv.hash_hex) &&
    eventEnv.hash_hex === eventProbe.engine.state_hash_hex(),
  `hash=${eventEnv.hash_hex}`,
);
check(
  "adapter_events_authoritative",
  eventCodes.length > 0 && eventCodes.every((code) => code >= 1 && code <= 10),
  `codes=${JSON.stringify(eventCodes)}`,
);
const observerEnv = nativeEnvelope(eventProbe);
check(
  "adapter_observer_does_not_drain_events",
  Array.isArray(observerEnv.events) &&
    observerEnv.events.length === 0 &&
    observerEnv.hash_hex === eventEnv.hash_hex,
  `events=${observerEnv.events.length} hash=${observerEnv.hash_hex}`,
);

// 抽样逐项提交：每条候选都在**同一保存点**的新引擎上走前端提交路径，
// 避免前一条候选改变状态影响后一条。
const startHex = auth.engine.save_hex();
const refused = [];
const sampled = authParsed.actions.slice(0, 6);
for (const action of sampled) {
  const fresh = nativeCreate(4, 4242, 0);
  nativeDispatch(fresh, { type: "restore", hex: startHex });
  let outcome;
  try {
    outcome = submitCandidate(fresh, action).result?.outcome ?? null;
  } catch (e) {
    outcome = `throw:${e}`;
  }
  if (outcome !== "accepted" && outcome !== "pending") {
    refused.push(`${JSON.stringify(action)}→${outcome}`);
  }
}
check(
  "legal_actions_sampled_all_accepted",
  sampled.length > 0 && refused.length === 0,
  `sampled=${sampled.length} refused=${JSON.stringify(refused)}`,
);

// 伪造卡 id／越权座位：内核拒绝且原因可见（不因投影而变合法）。
const forged = nativeDispatch(auth, {
  type: "playCard",
  actor: 0,
  card: 9999,
  targets: [],
});
check(
  "forged_card_rejected_with_reason",
  forged.result?.outcome === "rejected" && typeof forged.result?.reason === "string",
  `result=${JSON.stringify(forged.result)}`,
);
let seatRefused = false;
let seatDetail = "";
try {
  auth.engine.legal_actions_json(9);
  seatDetail = "accepted";
} catch (e) {
  seatRefused = String(e).includes("座位越界");
  seatDetail = String(e);
}
check("legal_actions_seat_boundary_refused", seatRefused, seatDetail);

// ── NC05 选将：候选来自内核目录，装配走正式入口（非 debug_*） ──
// 接线前缺口：适配层/worker 完全没有 general_candidates_json / assign_general
// 面（HEAD 版 grep 计数 0），前端唯一能改座位技能的路径是建局随机装配。
const gen = nativeCreate(4, 4242, 0);
const genBefore = nativeEnvelope(gen);
const genCandidates = generalCandidates(gen);
check(
  "general_candidates_from_catalog",
  genCandidates.count > 0 && genCandidates.count === genCandidates.candidates.length,
  `count=${genCandidates.count} before_skills=${JSON.stringify(
    genBefore.view.seats.find((s) => s.seat === 0)?.skills.map((s) => s.name) ?? [],
  )}`,
);
const pickGeneral = genCandidates.candidates.find((c) => c.skills.length > 0);
const genAssignment = chooseGeneral(gen, 0, pickGeneral.name);
const genAfter = nativeEnvelope(gen);
const genSkills = (genAfter.view.seats.find((s) => s.seat === 0)?.skills ?? [])
  .map((s) => s.name)
  .sort();
check(
  "choose_general_changes_skills",
  JSON.stringify(genSkills) === JSON.stringify([...pickGeneral.skills].sort()),
  `name=${pickGeneral.name} skills=${JSON.stringify(genSkills)}`,
);
check(
  "choose_general_assignment_shape",
  /^[0-9a-f]{64}$/.test(genAssignment.state_hash) &&
    genAssignment.seat === 0 &&
    genAssignment.name === pickGeneral.name &&
    genAssignment.revision === Number(gen.engine.revision()),
  `seat=${genAssignment.seat} revision=${genAssignment.revision} hash=${genAssignment.state_hash.slice(0, 12)}…`,
);
let genUnknownRefused = false;
let genUnknownDetail = "";
try {
  chooseGeneral(gen, 0, "查无此将");
  genUnknownDetail = "accepted";
} catch (e) {
  genUnknownDetail = String(e);
  genUnknownRefused =
    genUnknownDetail.includes("查无此将") && genUnknownDetail.includes("NoSuchGeneral");
}
check("choose_general_unknown_refused", genUnknownRefused, genUnknownDetail);
let genSeatRefused = false;
let genSeatDetail = "";
try {
  chooseGeneral(gen, 9, pickGeneral.name);
  genSeatDetail = "accepted";
} catch (e) {
  genSeatDetail = String(e);
  genSeatRefused = genSeatDetail.includes("座位越界");
}
check("choose_general_seat_boundary_refused", genSeatRefused, genSeatDetail);

// ── NC05 worker 协议面：真实 engine.native.worker.ts 的 onmessage ──
// 适配层已测过；这里补产品入口那一段（消息 → worker switch → 适配层 → 内核），
// 包括失败消息必须回 ok:false 且错误文本可读（绝不静默成功）。
// Node 下补 `self`：wasm-bindgen 的 init 已在本进程用字节完成，worker 的
// `ensureNativeReady()` 会直接复用，不再 fetch（浏览器路径仍走 URL）。
let workerResolve = null;
globalThis.self = {
  postMessage: (resp) => {
    const settle = workerResolve;
    if (settle && settle(resp)) workerResolve = null;
  },
};
await import(pathToFileURL(join(root, "src/worker/engine.native.worker.ts")).href);
const workerSend = (msg) =>
  new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      workerResolve = null;
      reject(new Error(`worker 回包超时: ${msg.type}`));
    }, 30000);
    workerResolve = (resp) => {
      // Timing notifications are unsolicited; only the matching reply completes a request.
      if (resp.id !== msg.id) return false;
      clearTimeout(timer);
      resolve(resp);
      return true;
    };
    globalThis.self.onmessage({ data: msg });
  });

try {
  const initReply = await workerSend({ id: 901, type: "init" });
  check("worker_native_init", initReply?.ok === true, JSON.stringify(initReply).slice(0, 200));

  const installed = (dir, key, file) => ({
    manifest: JSON.parse(readFileSync(join(extRoot, dir, "package.npack.json"), "utf8")).manifest,
    [key]: JSON.parse(readFileSync(join(extRoot, dir, "assets", file), "utf8")),
  });
  const installedPacks = [
    installed("huanle-upstream-noname", "cards", "cards.json"),
    installed("d3-upstream-noname", "characters", "character.json"),
    installed("cardpile-upstream-noname", "pile", "pile.json"),
  ];
  const installedJson = JSON.stringify(installedPacks);
  const started = await workerSend({
    id: 910, type: "start", players: 4, seed: 17017, observer: 0, packsJson: installedJson,
  });
  const expectedInstalled = NativeEngine.new_with_packs_json(4, 17017n, installedJson);
  const startedArchive = await workerSend({ id: 923, type: "save" });
  const expectedInstalledView = JSON.parse(expectedInstalled.view_json(0));
  check(
    "worker_installed_cards_and_pile_enter_start",
    started.ok === true && startedArchive.ok === true &&
      startedArchive.data?.hex === expectedInstalled.save_hex() &&
      expectedInstalled.cards_with_prefix("hnl_") === 7 && expectedInstalledView.deckCount > view0.deckCount + 7,
    `deck=${expectedInstalledView.deckCount} standard=${view0.deckCount} error=${started.error ?? ""}`,
  );
  const installedCandidates = await workerSend({ id: 911, type: "generalCandidates" });
  const installedGeneral = installedCandidates.data?.candidates?.find((c) => c.name === "ddd刘冲");
  check("worker_installed_general_candidate", installedCandidates.ok === true && !!installedGeneral);
  const selectedInstalled = await workerSend({ id: 912, type: "chooseGeneral", seat: 0, name: "ddd刘冲" });
  const selectedSkills = selectedInstalled.data?.envelope?.view?.seats[0]?.skills.map((s) => s.name).sort();
  check(
    "worker_installed_general_assignment",
    selectedInstalled.ok === true && JSON.stringify(selectedSkills) === JSON.stringify(["dddjinggou", "dddmoyan"]),
    JSON.stringify(selectedSkills),
  );
  const savedInstalled = await workerSend({ id: 913, type: "save" });
  const installedHex = savedInstalled.data?.hex;
  check("worker_installed_save", savedInstalled.ok === true && typeof installedHex === "string");
  const missing = await workerSend({ id: 914, type: "restore", hex: installedHex, packsJson: "[]" });
  check("worker_installed_missing_refused", missing.ok === false && /恢复失败/.test(missing.error ?? ""));
  const wrongInstalled = structuredClone(installedPacks);
  wrongInstalled[0].manifest.version = "1.0.1";
  const wrong = await workerSend({
    id: 915, type: "restore", hex: installedHex, packsJson: JSON.stringify(wrongInstalled),
  });
  check("worker_installed_exact_version_refused", wrong.ok === false && /恢复失败/.test(wrong.error ?? ""));
  const tamperedInstalled = structuredClone(installedPacks);
  tamperedInstalled[0].cards.cards[0].rank = 10;
  const tampered = await workerSend({
    id: 920, type: "restore", hex: installedHex, packsJson: JSON.stringify(tamperedInstalled),
  });
  check("worker_installed_changed_asset_refused", tampered.ok === false && /恢复失败/.test(tampered.error ?? ""));
  const duplicateVersion = structuredClone(installedPacks[1]);
  duplicateVersion.manifest.version = "2.0.0";
  duplicateVersion.characters.generals = [];
  const duplicateJson = JSON.stringify([...installedPacks, duplicateVersion]);
  const duplicateStart = await workerSend({
    id: 925, type: "start", players: 4, seed: 17017, observer: 0, packsJson: duplicateJson,
  });
  const duplicateIdentity = await workerSend({
    id: 926, type: "start", mode: "identity", players: 4, seed: 17017, observer: 0, packsJson: duplicateJson,
  });
  const duplicateRestore = await workerSend({
    id: 927, type: "restore", hex: installedHex, packsJson: duplicateJson,
  });
  const afterDuplicate = await workerSend({ id: 928, type: "save" });
  check(
    "worker_duplicate_slot_refused_without_changing_match",
    [duplicateStart, duplicateIdentity, duplicateRestore].every((reply) =>
      reply.ok === false && /重复包槽位/.test(reply.error ?? "")) &&
      afterDuplicate.ok === true && afterDuplicate.data?.hex === installedHex,
    JSON.stringify([duplicateStart.error, duplicateIdentity.error, duplicateRestore.error]),
  );
  const restoredInstalled = await workerSend({
    id: 916, type: "restore", hex: installedHex, packsJson: installedJson,
  });
  check(
    "worker_installed_same_lock_restore",
    restoredInstalled.ok === true &&
      JSON.stringify(restoredInstalled.data?.view) === JSON.stringify(selectedInstalled.data?.envelope?.view),
    restoredInstalled.error ?? "",
  );
  const identityJson = JSON.stringify([
    ...installedPacks, installed("wuxing-upstream-liuli", "cards", "cards.json"),
  ]);
  const identity = await workerSend({
    id: 917, type: "start", mode: "identity", players: 4, seed: 17017, observer: 0, packsJson: identityJson,
  });
  const identityCandidates = await workerSend({ id: 918, type: "generalCandidates" });
  const expectedIdentity = NativeEngine.new_identity_with_packs_json(4, 17017n, identityJson);
  const identityArchive = await workerSend({ id: 924, type: "save" });
  check(
    "worker_identity_keeps_packs_and_lord_bonus",
    identity.ok === true && identity.data?.view?.seats[0]?.max_hp === 5 &&
      identityArchive.ok === true && identityArchive.data?.hex === expectedIdentity.save_hex() &&
      identityCandidates.data?.candidates?.some((c) => c.name === "ddd刘冲"),
    `hp=${identity.data?.view?.seats[0]?.max_hp} error=${identity.error ?? ""}`,
  );
  const identityPick = await workerSend({ id: 921, type: "chooseGeneral", seat: 0, name: "ddd刘冲" });
  const identitySeat = identityPick.data?.envelope?.view?.seats[0];
  check(
    "worker_identity_selection_keeps_lord_bonus_and_global_skills",
    identityPick.ok === true && identitySeat?.max_hp === 5 &&
      ["dddjinggou", "dddmoyan", "wuxing_shengke"].every((name) => identitySeat.skills.some((s) => s.name === name)),
    `hp=${identitySeat?.max_hp} skills=${JSON.stringify(identitySeat?.skills.map((s) => s.name))}`,
  );
  const badIdentity = await workerSend({
    id: 919, type: "start", mode: "identity", players: 3, seed: 17017, observer: 0, packsJson: installedJson,
  });
  check("worker_identity_invalid_seats_refused", badIdentity.ok === false && /4-8/.test(badIdentity.error ?? ""));

  // 由 game-server 的 emit_vm_fixture 示例生成：真实 npack + 武将资产。
  const vmFixtureDir = join(root, "scripts/fixtures/vm-trigger");
  const vmPackage = JSON.parse(readFileSync(join(vmFixtureDir, "package.npack.json"), "utf8"));
  vmPackage.characters = JSON.parse(readFileSync(join(vmFixtureDir, "character.json"), "utf8"));
  const badVmPackage = structuredClone(vmPackage);
  badVmPackage.characters.generals[0].skills[0].name = "missing_vm_skill";
  const badVm = await workerSend({
    id: 950, type: "start", players: 4, seed: 42, observer: 0,
    packsJson: JSON.stringify([badVmPackage]),
  });
  check("worker_vm_unmapped_general_skill_refused", badVm.ok === false && /missing_vm_skill/.test(badVm.error ?? ""), badVm.error ?? "");
  const vmPacks = JSON.stringify([vmPackage]);
  const vmStart = await workerSend({ id: 951, type: "start", players: 4, seed: 42, observer: 0, packsJson: vmPacks });
  const vmPick = await workerSend({ id: 952, type: "chooseGeneral", seat: 0, name: "vm_fixture" });
  let vmEnv = vmPick.data?.envelope;
  const vmSkill = vmPackage.trigger_index[0].module;
  check("worker_vm_general_owns_module", vmStart.ok === true && vmPick.ok === true &&
    vmEnv?.view?.seats[0]?.skills.some((skill) => skill.name === vmSkill), vmSkill);
  let vmTriggered = false;
  let vmRestoreMatches = false;
  let vmDetails = "";
  for (let step = 0; step < 80 && vmEnv && !vmTriggered; step++) {
    const before = vmEnv;
    const atDiscard = before.view.current_player === 0 && before.view.phase === "Discard";
    const save = atDiscard ? await workerSend({ id: 1000 + step * 4, type: "save" }) : null;
    const command = before.choice
      ? { type: "autoStep" }
      : { type: "endPhase", actor: before.view.current_player };
    const advanced = await workerSend({ id: 1001 + step * 4, ...command });
    if (!advanced.ok || advanced.data?.result?.outcome === "rejected") {
      vmDetails = advanced.error ?? JSON.stringify(advanced.data?.result);
      break;
    }
    vmEnv = advanced.data;
    if (atDiscard && vmEnv.view.current_player === 0 && vmEnv.view.phase === "End") {
      const delta = vmEnv.view.seats[0].hand_count - before.view.seats[0].hand_count;
      vmTriggered = delta === 1;
      const restored = await workerSend({ id: 1002 + step * 4, type: "restore", hex: save?.data?.hex, packsJson: vmPacks });
      const replayed = await workerSend({ id: 1003 + step * 4, ...command });
      vmRestoreMatches = restored.ok === true && replayed.ok === true && replayed.data?.hash_hex === vmEnv.hash_hex;
      vmDetails = `delta=${delta} restored=${restored.ok} replayed=${replayed.ok}`;
    }
  }
  check("worker_vm_effect_at_end_phase", vmTriggered, vmDetails);
  check("worker_vm_same_package_restore_and_replay", vmRestoreMatches, vmDetails);

  const reset = await workerSend({ id: 922, type: "start", players: 4, seed: 4242, observer: 0 });
  check("worker_standard_restart", reset.ok === true);
  const candReply = await workerSend({ id: 902, type: "generalCandidates" });
  const candData = candReply?.data;
  check(
    "worker_general_candidates",
    candReply?.ok === true &&
      candData?.count > 0 &&
      candData.count === candData.candidates?.length,
    `count=${candData?.count} error=${candReply?.error ?? ""}`,
  );

  const pick = candData.candidates.find((c) => c.skills.length > 0);
  const chosen = await workerSend({ id: 903, type: "chooseGeneral", seat: 0, name: pick.name });
  const chosenSkills = (
    chosen?.data?.envelope?.view?.seats?.find((s) => s.seat === 0)?.skills ?? []
  )
    .map((s) => s.name)
    .sort();
  check(
    "worker_choose_general_reply",
    chosen?.ok === true &&
      chosen.data.assignment.name === pick.name &&
      JSON.stringify(chosenSkills) === JSON.stringify([...pick.skills].sort()),
    `name=${pick.name} skills=${JSON.stringify(chosenSkills)} hash=${chosen?.data?.assignment?.state_hash?.slice(0, 12)}…`,
  );

  const unknown = await workerSend({ id: 904, type: "chooseGeneral", seat: 0, name: "查无此将" });
  check(
    "worker_choose_general_unknown_refused",
    unknown?.ok === false && /NoSuchGeneral/.test(unknown.error ?? ""),
    `reply=${JSON.stringify(unknown).slice(0, 200)}`,
  );

  const oob = await workerSend({ id: 905, type: "chooseGeneral", seat: 9, name: pick.name });
  check(
    "worker_choose_general_seat_refused",
    oob?.ok === false && /座位越界/.test(oob?.error ?? ""),
    `reply=${JSON.stringify(oob).slice(0, 200)}`,
  );
} catch (e) {
  check("worker_general_select_protocol", false, `throw: ${e}`);
}

console.log(JSON.stringify({ total: results.length, failed: results.filter((r) => !r.ok).length }));
process.exit(results.some((r) => !r.ok) ? 1 : 0);
