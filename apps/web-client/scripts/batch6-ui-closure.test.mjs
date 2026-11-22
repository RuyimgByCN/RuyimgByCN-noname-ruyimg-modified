import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  L20_UI_HOOK_IDS,
  QHLY_CONFIG_SPECS,
  SKIN_SWITCH_CONFIG_KEYS,
  SKIN_SWITCH_CONFIG_SPECS,
  SKIN_SWITCH_SURFACE_IDS,
  SKIN_SWITCH_UI_HOOK_IDS,
  ZUAN_CONFIG_KEYS,
  ZUAN_CONFIG_SPECS,
  ZUAN_UI_HOOK_IDS,
  batch6PatchDatasetAttrs,
  createInitialBatch6PatchUiState,
  createInitialExtensionSurfacesState,
  createInitialL20RuntimeState,
  createInitialQhlyRuntimeState,
  createInitialSkinSwitchRuntimeState,
  createInitialZuanUiRuntimeState,
  deserializeExtensionSurfacesSnapshot,
  dispatchL10EmojiRebate,
  dispatchL20UiHook,
  dispatchLongxueCmEffects,
  dispatchQhlyUiHook,
  dispatchSkinSwitchHook,
  dispatchZuanUiHook,
  parseSkinSwitchConfigImportJson,
  parseZuanConfigImportJson,
  qhlyArenaDatasetAttrs,
  resolveQhlySkinAssetView,
  resolveQhlyVoiceRoute,
  runSkinSwitchActionConfig,
  serializeExtensionSurfacesSnapshot,
  skinSwitchArenaDatasetAttrs,
  syncL20MarkEffectsFromEnvelope,
  updateExtensionBatch6Patch,
  updateExtensionSkinSwitchConfig,
  updateExtensionZuanConfig,
  updateQhlyConfigInState,
  updateSkinSwitchConfigInState,
  zuanArenaDatasetAttrs,
} from "../src/engine/extensionSurfaceRuntime.ts";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(__dirname, "..", "..", "..");

function loadInventoryItems(sourceId) {
  const raw = JSON.parse(
    readFileSync(join(repoRoot, "docs", "migration", "g0", "inventory_status.json"), "utf8"),
  );
  return Object.values(raw.items).filter((item) => item.source_id === sourceId);
}

test("Batch UI-6 L20 补齐全部 10 项剩余 hooks -> 46/46 具备真实状态/DOM输出", () => {
  const invHooks = loadInventoryItems("L20")
    .filter((i) => i.category === "ui_hooks")
    .map((i) => i.id);
  assert.deepEqual(new Set(L20_UI_HOOK_IDS), new Set(invHooks));

  let s = createInitialL20RuntimeState();
  s = dispatchL20UiHook(s, "_guozhanhalfchghp", { seat: 2, halfTag: "阴阳校平" });
  assert.equal(s.guozhanHalfHpBySeat[2], "阴阳校平");
  assert.ok(s.activeBanner?.title.includes("半血阴阳"));

  for (const hk of [
    "_o_o_recovertrigger",
    "_losehp_",
    "_mianshang_",
    "_binglintu",
    "_dysp",
    "_guozhanChooseControlCHG",
    "_guozhan_marks",
    "_guozhan_diebiaoji_",
    "_chongzhu",
  ]) {
    s = dispatchL20UiHook(s, hk, { seat: 1, markName: "yinyang_mark", binglinSkill: "xiangyang" });
    assert.equal(s.lastDispatchedHook, hk);
  }
  assert.equal(s.recastCount, 1);
  assert.equal(s.binglinTeammateHint, true);
  assert.equal(s.binglinCrest, "xiangyang");
  assert.equal(s.guozhanControlThemeOn, true);

  const lost = syncL20MarkEffectsFromEnvelope(createInitialL20RuntimeState(), {
    events: [{ type: "HpLost", target: 0 }],
    choice: null,
    view: {
      revision: 1,
      state_hash: "h",
      observer: 0,
      round: 1,
      current_player: 0,
      phase: "Play",
      seats: [],
      known_cards: [],
      outcome: null,
    },
    legal_actions: [],
    hash_hex: "h",
    revision: 1,
    finished: false,
  });
  assert.equal(lost.playerHpCueBySeat[0], "体力流失");
});

test("Batch UI-6 L09 剩余 44 配置 真实消费者 + _qhly_randskin 门禁", () => {
  assert.equal(QHLY_CONFIG_SPECS.length, 82);
  let q = createInitialQhlyRuntimeState();
  q = updateQhlyConfigInState(
    q,
    {
      qhly_randskin: false,
      qhly_originSkinPath: "image/skin/",
      qhly_noSkin: "origin",
      qhly_decadeCloseDynamic: true,
      qhly_decadeDynamic: "three",
      qhly_audioPlus: true,
      qhly_currentMusic: "shuimo",
      qhly_mvp: true,
      qhly_dragButton: true,
      qhly_playerwindow: true,
    },
    { persist: false },
  );
  const unchanged = dispatchQhlyUiHook(q, { hook: "_qhly_randskin", seat: 2, generalId: "caocao" });
  assert.equal(unchanged.replaceSkinByGeneral.caocao, "origin");

  const resolved = resolveQhlySkinAssetView(q.config, "caocao", "epic", { isExtensionGeneral: false });
  assert.equal(resolved.assetRoot, "image/skin/");
  assert.equal(resolved.dynamicPermitted, false);
  assert.equal(resolved.dynamicSlotCount, 3);
  assert.ok(resolved.fallbackPolicyLabel.includes("原皮"));

  const voice = resolveQhlyVoiceRoute(q.config, "mvp");
  assert.equal(voice.allowed, true);
  assert.equal(voice.trackLabel, "shuimo+AudioPlus:mvp");

  const attrs = qhlyArenaDatasetAttrs(q.config);
  assert.equal(attrs["data-qhly-drag-button"], "on");
  assert.equal(attrs["data-qhly-playerwindow"], "on");
});

test("Batch UI-6 L24 皮肤切换 46 项 (14 configs + 10 surfaces + 22 hooks) 1-to-1 对齐与裁决 38", () => {
  const l24 = loadInventoryItems("L24");
  const invCfgs = l24.filter((i) => i.category === "configs").map((i) => i.id);
  const invSurfs = l24.filter((i) => i.category === "surfaces").map((i) => i.id);
  const invHooks = l24.filter((i) => i.category === "ui_hooks").map((i) => i.id);

  assert.equal(SKIN_SWITCH_CONFIG_SPECS.length, 14);
  assert.deepEqual(new Set(SKIN_SWITCH_CONFIG_KEYS), new Set(invCfgs));
  assert.deepEqual(new Set(SKIN_SWITCH_SURFACE_IDS), new Set(invSurfs));
  assert.deepEqual(new Set(SKIN_SWITCH_UI_HOOK_IDS), new Set(invHooks));

  let ss = createInitialSkinSwitchRuntimeState();
  ss = updateSkinSwitchConfigInState(ss, { isAttackFlipX: false, showTopArc: false, cugDynamicBg: true }, { persist: false });
  const gjNoFlip = dispatchSkinSwitchHook(ss, { hook: "_gj", seat: 1, seatScreenXPct: 15 });
  assert.equal(gjNoFlip.lastChukuangAction?.flipX, false);

  const dyn = runSkinSwitchActionConfig(ss, "genDynamicSkin");
  assert.ok(dyn.generatedDynamicSnippet?.includes("cleanroom-param-converter"));
  const tpl = runSkinSwitchActionConfig(dyn, "genDyTempFile");
  assert.ok(tpl.generatedTemplateSnippet?.includes("folderLayout"));
  assert.equal(skinSwitchArenaDatasetAttrs(dyn)["data-skinswitch-top-arc"], "off");
  assert.equal(skinSwitchArenaDatasetAttrs(dyn)["data-skinswitch-cug-bg"], "on");

  const defSs = createInitialSkinSwitchRuntimeState();
  assert.equal(defSs.isAttackFlipX, false);

  assert.throws(() => parseSkinSwitchConfigImportJson('{"badSs":1}'), /SKINSWITCH_CONFIG_UNKNOWN_KEY/);
});

test("Batch UI-6 Round2: _zuanattacklinecheck 同步新配置, _saveskill 仅触发于 EnteredDying 而非 HpRecovered", async () => {
  const { ingestBattleEnvelopeIntoSurfaces, updateZuanConfigInState } = await import(
    "../src/engine/extensionSurfaceRuntime.ts"
  );
  let z = createInitialZuanUiRuntimeState();
  z = updateZuanConfigInState(z, { zhishixian: "laser", zhishiopi: "50" });
  z = dispatchZuanUiHook(z, "_zuanattacklinecheck", { seat: 1 });
  assert.equal(z.attackLineStyle, "laser");
  assert.equal(z.attackLineOpacity, 0.5);
  assert.equal(z.zuanSeatCueBySeat[1], "指示线:laser");

  const baseSnap = createInitialExtensionSurfacesState();
  const baseEnv = {
    choice: null,
    view: {
      revision: 2,
      state_hash: "h",
      observer: 0,
      round: 1,
      current_player: 0,
      phase: "Play",
      seats: [],
      known_cards: [],
      outcome: null,
    },
    legal_actions: [],
    hash_hex: "h",
    revision: 2,
    finished: false,
  };
  const afterRecover = ingestBattleEnvelopeIntoSurfaces(baseSnap, {
    ...baseEnv,
    events: [{ type: "HpRecovered" }],
  });
  assert.notEqual(afterRecover.uiSuites?.zuanUi?.lastControlHook, "_saveskill");

  const afterDying = ingestBattleEnvelopeIntoSurfaces(baseSnap, {
    ...baseEnv,
    events: [{ EnteredDying: { player: 2 } }],
  });
  assert.equal(afterDying.uiSuites?.zuanUi?.lastControlHook, "_saveskill");
});

test("Batch UI-6 L25 祖安武将 68 配置 + 40 hooks (含 23+29 此前简化项) 1-to-1 校验", () => {
  const l25 = loadInventoryItems("L25");
  const invCfgs = l25.filter((i) => i.category === "configs").map((i) => i.id);
  const invHooks = l25.filter((i) => i.category === "ui_hooks").map((i) => i.id);

  assert.equal(ZUAN_CONFIG_SPECS.length, 68);
  assert.deepEqual(new Set(ZUAN_CONFIG_KEYS), new Set(invCfgs));
  assert.deepEqual(new Set(ZUAN_UI_HOOK_IDS), new Set(invHooks));

  let z = createInitialZuanUiRuntimeState();
  for (const hk of ZUAN_UI_HOOK_IDS) {
    z = dispatchZuanUiHook(z, hk, { seat: 1, reviveTurns: 2 });
    assert.equal(z.lastControlHook, hk);
  }
  assert.equal(z.reviveCountdownBySeat[1], 2);
  assert.equal(z.bossDefeatedBanner, true);
  assert.equal(z.luckyDrawShimmer, true);
  assert.ok(z.zuanSeatCueBySeat[1]);
  assert.ok(zuanArenaDatasetAttrs(z)["data-zuan-qiankuang"] === "on");

  assert.throws(() => parseZuanConfigImportJson('{"badZuan":true}'), /ZUAN_CONFIG_UNKNOWN_KEY/);
});

test("Batch UI-6 L27/L28/L29/L10 表面状态 + 快照序列化往返 + 净室边界", () => {
  let b6 = createInitialBatch6PatchUiState();
  b6 = dispatchLongxueCmEffects(b6, { skillName: "潜龙决进", daoxin: 99 });
  assert.ok(b6.longxueCmBurstLabel?.includes("99/99"));
  b6 = dispatchL10EmojiRebate(b6, { trigger: "recover", fromSeat: 0, toSeat: 1, isEnemy: true });
  assert.equal(b6.activeEmojiRebate?.kind, "humiliation");
  assert.equal(batch6PatchDatasetAttrs(b6)["data-l10-shichangshi"], "on");

  let snap = createInitialExtensionSurfacesState();
  snap = updateExtensionSkinSwitchConfig(snap, { showEditMenu: true }, { persist: false });
  snap = updateExtensionZuanConfig(snap, { zhishixian: "laser", ssr_luoyi: true });
  snap = updateExtensionBatch6Patch(snap, {
    thunderYuqiTheme: "standard",
    cardFold: false,
    cardFoldProportion: 5,
  });
  const restored = deserializeExtensionSurfacesSnapshot(serializeExtensionSurfacesSnapshot(snap));
  assert.equal(restored.uiSuites?.skinSwitch?.config.showEditMenu, true);
  assert.equal(restored.uiSuites?.zuanUi?.attackLineStyle, "laser");
  assert.equal(restored.uiSuites?.zuanUi?.config.ssr_luoyi, true);
  assert.equal(restored.uiSuites?.batch6Patch?.thunderYuqiTheme, "standard");
  assert.equal(restored.uiSuites?.batch6Patch?.cardFold, false);
  assert.equal(restored.uiSuites?.batch6Patch?.cardFoldProportion, 5);

  for (const rel of [
    "skinSwitchConfigSchema.ts",
    "skinSwitchSurface.ts",
    "zuanConfigSchema.ts",
    "zuanSurface.ts",
    "batch6PatchSurfacesRuntime.ts",
    "l20MarkSupplementRuntime.ts",
    "qhlyConfigSchema.ts",
    "qhlySurface.ts",
  ]) {
    const code = readFileSync(join(repoRoot, "apps", "web-client", "src", "engine", "surfaces", rel), "utf8");
    assert.ok(!/from\s+["']node:|require\(["']node:/.test(code), `${rel} contains Node import`);
    assert.ok(!/spine-pixi-v8|spine-lib/.test(code), `${rel} references blocked spine runtime`);
  }
});
