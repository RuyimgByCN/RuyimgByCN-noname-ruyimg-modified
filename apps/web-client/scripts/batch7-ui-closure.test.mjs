import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  BATCH7_INVENTORY_MANIFEST,
  COIN_HAVE_FUN_SURFACE_IDS,
  L01_CONFIG_KEYS,
  L01_CONFIG_SPECS,
  L02_CONFIG_SPECS,
  L07_CONFIG_SPECS,
  L07_HIGHLIGHT_CSS_IDS,
  L07_SURFACE_IDS,
  L11_SURFACE_IDS,
  L12_SURFACE_IDS,
  L13_CONFIG_SPECS,
  L13_SURFACE_IDS,
  L15_CONFIG_SPECS,
  L16_CONFIG_SPECS,
  L17_CONFIG_SPECS,
  L18_CONFIG_SPECS,
  L18_EXPLAIN_MAP,
  L21_CONFIG_SPECS,
  L26_CONFIG_SPECS,
  TAIXU_SERVANT_ACTION_SPECS,
  TAIXU_SERVANT_SPECS,
  U05_CONFIG_SPECS,
  applyL01JsonImportInState,
  applyL01WeightMapJson,
  applyL15InfoFix,
  applyL15NamePrefix,
  batch7ArenaDatasetAttrs,
  buildSimplePaginationView,
  computeAliveSeatDistance,
  computeAttackRangeFromEquips,
  computeFoolEquipMirrorCards,
  computeFoolThrowRotation,
  computeL21DoubleHp,
  computeTaixuShopPrice,
  createInitialBatch7UiSuiteState,
  createInitialExtensionSurfacesState,
  createInitialL01AiOptimizeState,
  createInitialL13UiState,
  createInitialL15State,
  createInitialL17State,
  createInitialL18State,
  createInitialL21State,
  createInitialL23State,
  createInitialL26State,
  getTaixuServantActionSpec,
  getTaixuServantIdleActions,
  getTaixuServantInteractiveActions,
  ingestBattleEnvelopeIntoSurfaces,
  resolveTaixuServantBattleClip,
  deserializeExtensionSurfacesSnapshot,
  dispatchBanCharFHook,
  dispatchCoinArenaReady,
  dispatchFixCaiLanZiTap,
  dispatchHaveFunEffect,
  evaluateFoolLiegongTag,
  exportL01ConfigJson,
  filterGeneralPoolByL15,
  importL01ConfigJson,
  importL02ConfigJson,
  importL07ConfigJson,
  importL13ConfigJson,
  importL15ConfigJson,
  importL16ConfigJson,
  importL17ConfigJson,
  importL18ConfigJson,
  importL21ConfigJson,
  importL26ConfigJson,
  importU05ConfigJson,
  normalizeBatch7UiSuiteState,
  normalizeL21Config,
  rewriteDouzhuanSkillText,
  runL01ConfigAction,
  runL16ManifestDryRun,
  runL18ProgressCheck,
  serializeExtensionSurfacesSnapshot,
  toggleL01RoleBanGeneral,
  triggerQyChongzhuHook,
} from "../src/engine/extensionSurfaceRuntime.ts";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(__dirname, "..", "..", "..");

function loadInventoryItems() {
  const raw = JSON.parse(
    readFileSync(join(repoRoot, "docs", "migration", "g0", "inventory_status.json"), "utf8"),
  );
  return Object.values(raw.items);
}

test("Batch UI-7 exact 1-to-1 closure over all 171 configs/surfaces/ui_hooks across 16 sources", () => {
  const targetCategories = new Set(["configs", "surfaces", "ui_hooks"]);
  const batch7Sources = new Set(Object.keys(BATCH7_INVENTORY_MANIFEST));
  const remaining = loadInventoryItems().filter(
    (item) =>
      targetCategories.has(item.category) &&
      batch7Sources.has(item.source_id),
  );
  assert.equal(remaining.length, 171, "Expected exactly 171 Batch-7 configs/surfaces/ui_hooks items across the 16 sources");

  const expectedBySource = {};
  for (const item of remaining) {
    expectedBySource[item.source_id] ??= { configs: [], surfaces: [], ui_hooks: [] };
    expectedBySource[item.source_id][item.category].push(item.id);
  }

  assert.deepEqual(
    Object.keys(expectedBySource).sort(),
    Object.keys(BATCH7_INVENTORY_MANIFEST).sort(),
    "Source set must match exact 16 Batch-7 sources",
  );

  let coveredTotal = 0;
  for (const [src, cats] of Object.entries(expectedBySource)) {
    const manifest = BATCH7_INVENTORY_MANIFEST[src];
    assert.ok(manifest, `Missing manifest for ${src}`);
    assert.deepEqual(new Set(manifest.configs ?? []), new Set(cats.configs), `${src} configs mismatch`);
    assert.deepEqual(new Set(manifest.surfaces ?? []), new Set(cats.surfaces), `${src} surfaces mismatch`);
    assert.deepEqual(new Set(manifest.ui_hooks ?? []), new Set(cats.ui_hooks), `${src} ui_hooks mismatch`);
    coveredTotal += (manifest.configs?.length ?? 0) + (manifest.surfaces?.length ?? 0) + (manifest.ui_hooks?.length ?? 0);
  }
  assert.equal(coveredTotal, 171);

  assert.equal(L01_CONFIG_SPECS.length, 63);
  assert.equal(L02_CONFIG_SPECS.length, 4);
  assert.equal(COIN_HAVE_FUN_SURFACE_IDS.length, 4);
  assert.equal(L07_CONFIG_SPECS.length, 4);
  assert.equal(L07_SURFACE_IDS.length, 9);
  assert.equal(L11_SURFACE_IDS.length, 4);
  assert.equal(L12_SURFACE_IDS.length, 6);
  assert.equal(L13_CONFIG_SPECS.length, 7);
  assert.equal(L13_SURFACE_IDS.length, 2);
  assert.equal(L15_CONFIG_SPECS.length, 11);
  assert.equal(L16_CONFIG_SPECS.length, 2);
  assert.equal(L17_CONFIG_SPECS.length, 9);
  assert.equal(L18_CONFIG_SPECS.length, 12);
  assert.equal(L21_CONFIG_SPECS.length, 7);
  assert.equal(L26_CONFIG_SPECS.length, 5);
  assert.equal(U05_CONFIG_SPECS.length, 7);
});

test("Cleanroom check: zero Node-only imports, eval/Function, or Spine runtime references in Batch 7 runtime modules", () => {
  const files = [
    "aiOptimizeConfigSchema.ts",
    "aiOptimizeSurface.ts",
    "toolsAndPoolSchema.ts",
    "toolsAndPoolSurface.ts",
    "visualEffectsBatch7Runtime.ts",
    "batch7ClosureState.ts",
  ];
  const forbidden = /\bfrom\s+["']node:|require\(|child_process|\beval\s*\(|new\s+Function|spine\.js|spine-lib|spine-pixi|pixi6/i;
  for (const fname of files) {
    const raw = readFileSync(join(repoRoot, "apps", "web-client", "src", "engine", "surfaces", fname), "utf8");
    const stripped = raw.replace(/\/\*[\s\S]*?\*\/|\/\/.*$/gm, "");
    assert.equal(forbidden.test(stripped), false, `Forbidden Node/eval/Spine reference in ${fname}`);
  }
});

test("Decision 38 strict JSON schema validation rejects unknown keys across L01/L02/L07/L13/L15/L16/L17/L18/L21/L26/U05", () => {
  assert.throws(() => importL01ConfigJson('{"bogus":true}'), /L01_CONFIG_UNKNOWN_KEY:bogus/);
  assert.throws(() => importL01ConfigJson('{"editCf":{"x":0}}'), /L01_CONFIG_NON_POSITIVE_WEIGHT/);
  assert.throws(() => importL02ConfigJson('{"bad":1}'), /L02_CONFIG_UNKNOWN_KEY:bad/);
  assert.throws(() => importL07ConfigJson('{"bad":1}'), /L07_CONFIG_UNKNOWN_KEY:bad/);
  assert.throws(() => importL13ConfigJson('{"bad":1}'), /L13_CONFIG_UNKNOWN_KEY:bad/);
  assert.throws(() => importL15ConfigJson('{"bad":1}'), /L15_CONFIG_UNKNOWN_KEY:bad/);
  assert.throws(() => importL16ConfigJson('{"bad":1}'), /L16_CONFIG_UNKNOWN_KEY:bad/);
  assert.throws(() => importL17ConfigJson('{"bad":1}'), /L17_CONFIG_UNKNOWN_KEY:bad/);
  assert.throws(() => importL18ConfigJson('{"bad":1}'), /L18_CONFIG_UNKNOWN_KEY:bad/);
  assert.throws(() => importL21ConfigJson('{"bad":1}'), /L21_CONFIG_UNKNOWN_KEY:bad/);
  assert.throws(() => importL26ConfigJson('{"bad":1}'), /L26_CONFIG_UNKNOWN_KEY:bad/);
  assert.throws(() => importU05ConfigJson('{"bad":1}'), /U05_CONFIG_UNKNOWN_KEY:bad/);

  const clampedL21 = normalizeL21Config({ show_number: "3", battle_number: "7", hide_number: "6" });
  assert.equal(clampedL21.battle_number, "3");
  assert.equal(clampedL21.hide_number, "3");

  const validL01 = importL01ConfigJson('{"viewAtt":true,"nhFriends":"3","editQz":{"caocao":1.5}}');
  assert.equal(validL01.viewAtt, true);
  assert.equal(validL01.nhFriends, "3");
  assert.equal(validL01.editQz.caocao, 1.5);
});

test("L01 AI优化 actions, role bans, weight maps & ruleCoupled metadata honesty", () => {
  let s = createInitialL01AiOptimizeState();
  const roundtrip = applyL01JsonImportInState(s, exportL01ConfigJson(s));
  assert.equal(roundtrip.config.applyCf, s.config.applyCf);

  s = toggleL01RoleBanGeneral(s, "banzhu", "caocao");
  assert.ok(s.config.banzhu.includes("caocao"));
  s = toggleL01RoleBanGeneral(s, "banzhu", "caocao");
  assert.ok(!s.config.banzhu.includes("caocao"));

  s = applyL01WeightMapJson(s, "editCf", '{"guose":2.5}');
  assert.equal(s.config.editCf.guose, 2.5);

  const l01Actions = L01_CONFIG_SPECS.filter((c) => c.control === "action" || c.key === "kzjs").map((c) => c.key);
  assert.equal(l01Actions.length, 29, "Expected kzjs + all 28 L01 action keys");
  for (const act of l01Actions) {
    s = runL01ConfigAction(s, act, { generalId: "caocao", skillId: "guose" });
    assert.ok(s && s.lastActionNotice.length > 0, `Action ${act} must return defined state with notice`);
  }
  const ruleCoupledCount = L01_CONFIG_SPECS.filter((c) => c.ruleCoupled).length;
  const uiOnlyCount = L01_CONFIG_SPECS.filter((c) => !c.ruleCoupled).length;
  assert.ok(ruleCoupledCount > 0 && uiOnlyCount > 0 && ruleCoupledCount + uiOnlyCount === L01_CONFIG_KEYS.length);
  assert.equal(L15_CONFIG_SPECS.find((s) => s.key === "unseen")?.ruleCoupled, true);
  assert.equal(L15_CONFIG_SPECS.find((s) => s.key === "packs")?.ruleCoupled, true);
  assert.equal(L26_CONFIG_SPECS.find((s) => s.key === "Fool_liegong")?.ruleCoupled, true);
  assert.equal(L26_CONFIG_SPECS.find((s) => s.key === "Fool_skill")?.ruleCoupled, true);
});

test("L15/L16/L17/L18/L21 pool, dry-run zip manifest, banCharF, progress & double_hp behaviors", () => {
  assert.equal(applyL15NamePrefix("手杀张飞", "shousha"), "界张飞");
  assert.equal(applyL15NamePrefix("OL关羽", "all"), "关羽");
  assert.equal(applyL15InfoFix("出牌阶段限一次", true), "出牌阶段限一次。");
  assert.equal(
    filterGeneralPoolByL15(
      [
        { id: "a", sex: "male", pack: "taixu" },
        { id: "b", sex: "female", pack: "standard" },
        { id: "c", sex: "male", pack: "shenhua" },
      ],
      { ...createInitialL15State().config, txhj: true, sexforbid: "male", huanyuan: true },
    ).map((g) => g.id).join(","),
    "c",
  );

  assert.throws(() => runL16ManifestDryRun("../escape.zip"), /unsafe export filename/);
  assert.ok(runL16ManifestDryRun("safe_pack.zip", 12).includes("safe_pack.zip"));

  let l17 = createInitialL17State();
  l17 = dispatchBanCharFHook(l17, "selectPlan", { plan: 2 });
  l17 = dispatchBanCharFHook(l17, "toggleGeneral", { generalId: "zhugeliang" });
  l17 = dispatchBanCharFHook(l17, "setStars", { generalId: "zhugeliang", stars: 5 });
  assert.equal(l17.activePlanIndex, 2);
  assert.ok(l17.plans[2].includes("zhugeliang"));
  assert.equal(l17.charStars.zhugeliang, 5);
  assert.equal(rewriteDouzhuanSkillText("伤害不大于2且不小于1，阴阳"), "伤害≤2且≥1，阳阴");

  let l18 = createInitialL18State();
  l18 = runL18ProgressCheck(l18, "asset");
  assert.equal(l18.progressPct, 100);
  assert.ok(L18_EXPLAIN_MAP.URC.includes("United Rhythmized Club"));

  const l21Init = createInitialL21State();
  assert.equal(l21Init.hiddenPreviewActive, false);
  assert.equal(computeL21DoubleHp(4, 3, "hejiansan"), 4);
  assert.equal(computeL21DoubleHp(4, 3, "pingjun"), 3);
  assert.equal(computeL21DoubleHp(4, 3, "zuidazhi"), 4);
  assert.equal(computeL21DoubleHp(4, 3, "zuixiaozhi"), 3);
  assert.equal(computeL21DoubleHp(4, 3, "zonghe"), 7);
});

test("L05/U04 coin haveFun + L07 SimplePagination + L13 _qy_chongzhu + L23 _fixCAILANZI + L26 _Fool_chooseCard", () => {
  const base = createInitialExtensionSurfacesState();
  let coin = dispatchHaveFunEffect(base.coin, "firework", "start");
  coin = dispatchHaveFunEffect(coin, "snow", "start");
  coin = dispatchHaveFunEffect(coin, "snow", "toggleSnowSize");
  assert.ok(coin.activeEffects.includes("fireworks"));
  assert.equal(coin.snowFall, true);
  assert.equal(coin.snowSize, "large");

  const arenaReady = dispatchCoinArenaReady(coin, { displayMode: "text", canvasTop: true });
  assert.equal(arenaReady.mounted, true);
  assert.equal(arenaReady.canvasTopClass, true);
  assert.ok(arenaReady.hudText.endsWith("金"));

  assert.equal(L07_HIGHLIGHT_CSS_IDS.length, 6);
  const pgSmall = buildSimplePaginationView(4, 2, 5);
  assert.deepEqual(pgSmall.pages, [1, 2, 3, 4]);
  const pgMid = buildSimplePaginationView(15, 8, 5);
  assert.equal(pgMid.ellipsisHead, true);
  assert.equal(pgMid.ellipsisTail, true);
  assert.deepEqual(pgMid.pages, [1, 6, 7, 8, 9, 10, 15]);

  const l13Res = triggerQyChongzhuHook(createInitialL13UiState(), base.mvp, 2, "female");
  assert.equal(l13Res.mvp.scores[2].assist, 3);
  assert.equal(l13Res.mvp.scores[2].total, 103);
  assert.equal(l13Res.l13.lastRecastCue, "qy_chongzhu_female@P2");

  let l23 = createInitialL23State();
  l23 = dispatchFixCaiLanZiTap(l23, { seat: 1, dx: 2, dy: 3, isSelf: false, isDead: false, isOver: false });
  assert.equal(l23.lastTapClassification, "tap");
  assert.equal(l23.emojiPickerSeat, 1);
  l23 = dispatchFixCaiLanZiTap(l23, { seat: 1, dx: 20, dy: 0, isSelf: false, isDead: false, isOver: false });
  assert.equal(l23.lastTapClassification, "drag");

  let l26 = { ...createInitialL26State(), chooseCardEquipCopyEnabled: true };
  const fakeCards = [
    { id: 10, owner: 0, zone: "Equip", suit: "Spade", rank: 6, kind: "Weapon", def: 1 },
    { id: 11, owner: 0, zone: "Hand", suit: "Heart", rank: 1, kind: "BasicPeach", def: 3 },
  ];
  l26 = computeFoolEquipMirrorCards(l26, fakeCards, 0, { positionAllowsEquip: true, inPhaseDiscard: false });
  assert.deepEqual(l26.mirroredEquipCardIds, [10]);
  l26 = computeFoolEquipMirrorCards(l26, fakeCards, 0, { positionAllowsEquip: true, inPhaseDiscard: true });
  assert.deepEqual(l26.mirroredEquipCardIds, []);

  // L26 Fool_liegong negative + positive skill/card gating (ASCII + localized catalog skill names)
  const shaCard = { id: 20, owner: 0, zone: "Hand", suit: "Heart", rank: 7, kind: "Sha", def: 1 };
  const weaponCard = { id: 91, owner: 0, zone: "Hand", suit: "Spade", rank: 1, kind: "Weapon", def: 10 };
  const targetSeat = { player: 1, seat: 1, hp: 4, max_hp: 4, alive: true, turned_over: false, hand_count: 3, identity: null, skills: [] };
  const liegongSelf = { player: 0, seat: 0, hp: 4, max_hp: 4, alive: true, turned_over: false, hand_count: 4, identity: null, skills: [{ id: 1, def: 1, name: "liegong", enabled: true, use_count: 0, active: false, view_as: false, locked: true }] };
  const plainSelf = { ...liegongSelf, skills: [{ id: 2, def: 2, name: "guidao", enabled: true, use_count: 0, active: false, view_as: false, locked: false }] };
  const zhangfeiSelf = { ...liegongSelf, skills: [{ id: 4, def: 4, name: "咆哮", enabled: true, use_count: 0, active: false, view_as: false, locked: true }] };
  assert.equal(evaluateFoolLiegongTag({ mode: "shousha", selectedCard: weaponCard, selfSeat: liegongSelf, targetSeat, selfHandCount: 4, selfEquipsCount: 0 }), null);
  assert.equal(evaluateFoolLiegongTag({ mode: "shousha", selectedCard: shaCard, selfSeat: plainSelf, targetSeat, selfHandCount: 4, selfEquipsCount: 0 }), null);
  assert.equal(evaluateFoolLiegongTag({ mode: "shousha", selectedCard: shaCard, selfSeat: zhangfeiSelf, targetSeat, selfHandCount: 4, selfEquipsCount: 0 }), null);
  assert.equal(evaluateFoolLiegongTag({ mode: "close", selectedCard: shaCard, selfSeat: liegongSelf, targetSeat: { ...targetSeat, hand_count: 4 }, selfHandCount: 4, selfEquipsCount: 0 }), null);
  assert.deepEqual(evaluateFoolLiegongTag({ mode: "shousha", selectedCard: shaCard, selfSeat: liegongSelf, targetSeat: { ...targetSeat, hand_count: 4 }, selfHandCount: 4, selfEquipsCount: 0 }), { label: "可烈弓", cssClass: "Fool_liegone_shousha" });
  assert.deepEqual(evaluateFoolLiegongTag({ mode: "olten", selectedCard: shaCard, selfSeat: { ...liegongSelf, skills: [{ id: 5, def: 5, name: "烈弓", enabled: true, use_count: 0, active: false, view_as: false, locked: true }] }, targetSeat: { ...targetSeat, hand_count: 4 }, selfHandCount: 4, selfEquipsCount: 0 }), { label: "可烈弓", cssClass: "Fool_liegone_olten" });
  assert.deepEqual(evaluateFoolLiegongTag({ mode: "ol", selectedCard: shaCard, selfSeat: { ...plainSelf, skills: [{ id: 7, def: 7, name: "xinliegong", enabled: true, use_count: 0, active: false, view_as: false, locked: false }] }, targetSeat, selfHandCount: 5, selfEquipsCount: 0 }), { label: "可烈弓", cssClass: "Fool_liegone_ol" });
  assert.deepEqual(evaluateFoolLiegongTag({ mode: "olten", selectedCard: shaCard, selfSeat: { ...plainSelf, skills: [{ id: 3, def: 3, name: "wushen", enabled: true, use_count: 0, active: false, view_as: false, locked: false }] }, targetSeat, selfHandCount: 4, selfEquipsCount: 0 }), { label: "不可响应", cssClass: "Fool_liegone_olten" });
  assert.deepEqual(evaluateFoolLiegongTag({ mode: "shousha", selectedCard: shaCard, selfSeat: { ...plainSelf, skills: [{ id: 6, def: 6, name: "诈降", enabled: true, use_count: 0, active: false, view_as: false, locked: false }] }, targetSeat, selfHandCount: 4, selfEquipsCount: 0 }), { label: "不可响应", cssClass: "Fool_liegone_shousha" });
  assert.deepEqual(evaluateFoolLiegongTag({ mode: "shousha", selectedCard: shaCard, selfSeat: { ...plainSelf, skills: [{ id: 7, def: 7, name: "新烈弓", enabled: true, use_count: 0, active: false, view_as: false, locked: false }] }, targetSeat, selfHandCount: 5, selfEquipsCount: 0 }), { label: "强中加伤", cssClass: "Fool_liegone_shousha" });

  // Upstream targetInRange, weapon attackRange, alive/horse distances, and live skill counters
  assert.equal(evaluateFoolLiegongTag({ mode: "shousha", selectedCard: shaCard, selfSeat: liegongSelf, targetSeat: { ...targetSeat, hand_count: 4 }, selfHandCount: 4, selfEquipsCount: 0, distance: 2, attackRange: 1 }), null);
  assert.deepEqual(evaluateFoolLiegongTag({ mode: "shousha", selectedCard: shaCard, selfSeat: liegongSelf, targetSeat: { ...targetSeat, hand_count: 4 }, selfHandCount: 4, selfEquipsCount: 1, distance: 2, attackRange: 5 }), { label: "可烈弓", cssClass: "Fool_liegone_shousha" });
  assert.deepEqual(evaluateFoolLiegongTag({ mode: "shousha", selectedCard: shaCard, selfSeat: liegongSelf, targetSeat: { ...targetSeat, hand_count: 4 }, selfHandCount: 4, selfEquipsCount: 0, distance: 2, attackRange: 1, inRange: true }), { label: "可烈弓", cssClass: "Fool_liegone_shousha" });
  const sbPaoxiaoSelf = { ...plainSelf, skills: [{ id: 8, def: 8, name: "sbpaoxiao", enabled: true, use_count: 0, active: false, view_as: false, locked: true }] };
  assert.equal(evaluateFoolLiegongTag({ mode: "shousha", selectedCard: shaCard, selfSeat: sbPaoxiaoSelf, targetSeat, selfHandCount: 4, selfEquipsCount: 0, phaseHasSha: false }), null);
  assert.deepEqual(evaluateFoolLiegongTag({ mode: "shousha", selectedCard: shaCard, selfSeat: sbPaoxiaoSelf, targetSeat, selfHandCount: 4, selfEquipsCount: 0, phaseHasSha: true }), { label: "不可响应", cssClass: "Fool_liegone_shousha" });
  const sbZhaxiangSelf = { ...plainSelf, hp: 4, max_hp: 4, skills: [{ id: 9, def: 9, name: "sbzhaxiang", enabled: true, use_count: 0, active: false, view_as: false, locked: true }] };
  assert.equal(evaluateFoolLiegongTag({ mode: "shousha", selectedCard: shaCard, selfSeat: sbZhaxiangSelf, targetSeat, selfHandCount: 4, selfEquipsCount: 0, useCountTurn: 0 }), null);
  assert.deepEqual(evaluateFoolLiegongTag({ mode: "shousha", selectedCard: shaCard, selfSeat: { ...sbZhaxiangSelf, hp: 3 }, targetSeat, selfHandCount: 4, selfEquipsCount: 0, useCountTurn: 0 }), { label: "不可响应", cssClass: "Fool_liegone_shousha" });
  const jintaoSelf = { ...plainSelf, skills: [{ id: 10, def: 10, name: "jintao", enabled: true, use_count: 0, active: false, view_as: false, locked: true }] };
  assert.deepEqual(evaluateFoolLiegongTag({ mode: "shousha", selectedCard: shaCard, selfSeat: jintaoSelf, targetSeat, selfHandCount: 4, selfEquipsCount: 0, phaseShaCount: 0, distance: 3 }), { label: "加伤", cssClass: "Fool_liegone_shousha" });
  assert.deepEqual(evaluateFoolLiegongTag({ mode: "shousha", selectedCard: shaCard, selfSeat: jintaoSelf, targetSeat, selfHandCount: 4, selfEquipsCount: 0, phaseShaCount: 1 }), { label: "不可响应", cssClass: "Fool_liegone_shousha" });
  assert.equal(evaluateFoolLiegongTag({ mode: "shousha", selectedCard: shaCard, selfSeat: jintaoSelf, targetSeat, selfHandCount: 4, selfEquipsCount: 0, phaseShaCount: 2 }), null);
  assert.equal(computeAttackRangeFromEquips([{ def: 97, kind: "Weapon" }]), 5);
  assert.equal(
    computeAliveSeatDistance(
      [
        { player: 0, alive: true, skills: [{ id: 1, def: 1, name: "mashu", enabled: true, use_count: 0, active: false, view_as: false, locked: true }] },
        { player: 1, alive: false, skills: [] },
        { player: 2, alive: true, skills: [] },
        { player: 3, alive: true, skills: [] },
      ],
      (p) => (p === 2 ? [{ kind: "HorseDef" }] : []),
      0,
      2,
    ),
    1,
  );

  assert.notEqual(computeFoolThrowRotation(1, 0, true), 0);
  assert.equal(computeFoolThrowRotation(1, 0, false), 0);
});

test("Decision 34 serialize/deserialize roundtrip preserves uiSuites.batch7 + dataset attrs", () => {
  const init = createInitialExtensionSurfacesState();
  const modifiedBatch7 = normalizeBatch7UiSuiteState({
    ...createInitialBatch7UiSuiteState(),
    l01: { ...createInitialL01AiOptimizeState({ viewAtt: true }) },
    l23: { ...createInitialL23State(), emojiPickerSeat: 1, lastTappedSeat: 1, lastTapClassification: "tap" },
  });
  const snap = {
    ...init,
    uiSuites: {
      ...init.uiSuites,
      batch7: modifiedBatch7,
    },
  };
  const attrs = batch7ArenaDatasetAttrs(modifiedBatch7, snap.coin);
  assert.equal(attrs["data-l01-view-att"], "on");
  assert.equal(attrs["data-l23-cailanzi"], "on");

  const restored = deserializeExtensionSurfacesSnapshot(serializeExtensionSurfacesSnapshot(snap));
  assert.equal(restored.uiSuites?.batch7?.l01.config.viewAtt, true);
  assert.equal(restored.uiSuites?.batch7?.l23.emojiPickerSeat, 1);
});

test("P3-D L11 Taixu store +200 escalation, juguzhidao dec=1, and 21-servant Spine action parity", async () => {
  // 1. All 21 servant specs carry upstream extension_servant.js:1329-1505 action definitions
  assert.equal(TAIXU_SERVANT_SPECS.length, 21);
  assert.equal(Object.keys(TAIXU_SERVANT_ACTION_SPECS).length, 21);
  assert.deepEqual(getTaixuServantActionSpec("datong"), {
    daiji: 2, gongji: 2, jingzhi: true, jinnang: 1, shouji: 1, zuodian: false,
  });
  assert.deepEqual(getTaixuServantActionSpec("xueren"), {
    daiji: 2, gongji: 2, jingzhi: true, jinnang: 1, shouji: 1, zuodian: false,
  });
  assert.deepEqual(getTaixuServantActionSpec("manman"), {
    daiji: 3, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: false,
  });
  assert.deepEqual(getTaixuServantActionSpec("jiuwei"), {
    daiji: 4, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: true,
  });

  // 2. Idle & getAction() clip generation
  assert.deepEqual(getTaixuServantIdleActions("jiuwei"), [
    "daiji1", "daiji2", "daiji3", "daiji4", "jingzhi", "zuodian",
  ]);
  assert.deepEqual(getTaixuServantIdleActions("datong"), ["daiji1", "daiji2", "jingzhi"]);
  assert.deepEqual(getTaixuServantInteractiveActions("datong"), ["daiji2", "jinnang1", "shouji1"]);
  assert.deepEqual(getTaixuServantInteractiveActions("jiuwei"), [
    "daiji2", "daiji3", "daiji4", "jinnang1", "shouji1", "shouji2",
  ]);

  // 3. _slDamage / _slUseCard / _slHurt clip resolution & live EngineEnvelope ingestion
  assert.equal(resolveTaixuServantBattleClip("jiuwei", { kind: "_slDamage", amount: 1, isLeft: false }), "gongji1");
  assert.equal(resolveTaixuServantBattleClip("jiuwei", { kind: "_slDamage", amount: 2, isLeft: true }), "gongji2");
  assert.equal(resolveTaixuServantBattleClip("jiuwei", { kind: "_slDamage", amount: 0, isLeft: true }), null);
  assert.equal(resolveTaixuServantBattleClip("jiuwei", { kind: "_slUseCard", cardType: "trick", cardName: "wuzhong" }), "jinnang1");
  assert.equal(resolveTaixuServantBattleClip("jiuwei", { kind: "_slUseCard", cardType: "basic", cardName: "tao" }), "jinnang1");
  assert.equal(resolveTaixuServantBattleClip("jiuwei", { kind: "_slUseCard", cardType: "basic", cardName: "jiu" }), "jinnang1");
  assert.equal(resolveTaixuServantBattleClip("jiuwei", { kind: "_slUseCard", cardType: "basic", cardName: "sha" }), null);
  assert.equal(resolveTaixuServantBattleClip("datong", { kind: "_slHurt", variant: 2 }), "shouji1");
  assert.equal(resolveTaixuServantBattleClip("jiuwei", { kind: "_slHurt", variant: 2 }), "shouji2");

  const init = createInitialExtensionSurfacesState();
  const afterTrick = ingestBattleEnvelopeIntoSurfaces(init, {
    revision: 10,
    finished: false,
    view: {
      observer: 0,
      round: 1,
      current_player: 0,
      seats: [
        { player: 0, alive: true, hp: 4, max_hp: 4, hand_count: 3 },
        { player: 1, alive: true, hp: 4, max_hp: 4, hand_count: 3 },
      ],
    },
    events: [{ type: "CardPlayed", player: 0, cardType: "trick", cardName: "wuzhong" }],
  });
  assert.equal(afterTrick.uiSuites?.taixuUi?.lastServantClip, "jinnang1");

  const afterHurt = ingestBattleEnvelopeIntoSurfaces(afterTrick, {
    revision: 11,
    finished: false,
    view: {
      observer: 0,
      round: 1,
      current_player: 1,
      seats: [
        { player: 0, alive: true, hp: 3, max_hp: 4, hand_count: 3 },
        { player: 1, alive: true, hp: 4, max_hp: 4, hand_count: 2 },
      ],
    },
    events: [{ type: "DamageDealt", source: 1, target: 0, amount: 1 }],
  });
  assert.equal(afterHurt.uiSuites?.taixuUi?.lastServantClip, "shouji1");

  // 4. TaixuCampaignModal.vue executes real purchaseTaixuMerchantItem escalation + juguzhidao dec=1 via props.taixu.gold sync
  assert.equal(computeTaixuShopPrice(80, 0, false), 80);
  assert.equal(computeTaixuShopPrice(80, 1, false), 280);
  assert.equal(computeTaixuShopPrice(80, 1, true), 80);
  assert.equal(computeTaixuShopPrice(80, 2, true), 280);
  const modalVuePath = join(dirname(fileURLToPath(import.meta.url)), "../src/components/battle/modals/TaixuCampaignModal.vue");
  const modalSrc = readFileSync(modalVuePath, "utf8");
  assert.ok(modalSrc.includes("computeTaixuShopPrice"), "TaixuCampaignModal must invoke computeTaixuShopPrice");
  assert.ok(!modalSrc.includes("baseCost / 2"), "TaixuCampaignModal must NOT charge flat baseCost / 2");

  const { parse, compileScript } = await import("vue/compiler-sfc");
  const tsMod = (await import("typescript")).default;
  const sfc = parse(modalSrc, { filename: modalVuePath });
  const compiled = compileScript(sfc.descriptor, { id: "taixu-modal-test", inlineTemplate: false });
  const jsCode = tsMod.transpileModule(compiled.content, {
    compilerOptions: { module: tsMod.ModuleKind.ESNext, target: tsMod.ScriptTarget.ES2022 },
  }).outputText;
  const { reactive } = await import("vue");
  const runtimeMod = await import(new URL("../src/engine/extensionSurfaceRuntime.ts", import.meta.url).href);
  const resolvedModUrl = new URL("../src/engine/extensionSurfaceRuntime.ts", import.meta.url).href;
  const rewriteImport = (spec, varName) =>
    spec.replace(/\bas\b/g, ":");
  const stripped = jsCode
    .replace(/import\s*\{([^}]*)\}\s*from\s*["']vue["'];?/g, (_, syms) => `const {${rewriteImport(syms, "vue")}} = vue;`)
    .replace(/import\s*\{([^}]*)\}\s*from\s*["']\.\.\/\.\.\/\.\.\/engine\/extensionSurfaceRuntime["'];?/g, (_, syms) => `const {${rewriteImport(syms, "rt")}} = rt;`)
    .replace(/import\s+type\s+[^;]+;?/g, "")
    .replace(/export\s+default\s+/g, "const __comp = ");
  const execFn = new Function(
    "vue",
    "rt",
    `${stripped}\nreturn __comp;`,
  );
  const compDef = execFn({ computed: (await import("vue")).computed, ref: (await import("vue")).ref, defineComponent: (x) => x }, runtimeMod);
  assert.ok(resolvedModUrl.length > 0 && compDef && typeof compDef.setup === "function");

  const modalInit = runtimeMod.createInitialExtensionSurfacesState();
  const modalState = reactive({
    taixu: structuredClone(modalInit.taixu),
    taixuUi: structuredClone(modalInit.uiSuites.taixuUi),
  });
  const bindings = compDef.setup(modalState, {
    expose() {},
    emit(ev, payload) {
      if (ev === "update") {
        modalState.taixu = payload.taixu;
        modalState.taixuUi = payload.taixuUi;
      }
    },
  });

  // Initial: gold=100, coinBalance=200; advance x2 => gold=340, coinBalance synced to 340
  bindings.advanceCampaignStep();
  bindings.advanceCampaignStep();
  assert.equal(modalState.taixu.gold, 340);
  assert.equal(modalState.taixuUi.coinBalance, 340);

  // Purchase #1 (aozhan, base 80): pays 80 -> gold=260, coinBalance=260, shopPurchaseCount=1
  bindings.buyUpstreamBlessing("buff_txhj_aozhan", 80);
  assert.equal(modalState.taixu.gold, 260);
  assert.equal(modalState.taixuUi.coinBalance, 260);
  assert.equal(modalState.taixuUi.shopPurchaseCount, 1);

  // Purchase #2 (yanhuo, base 60): price 60 + 200 = 260 > legacy stale 120 -> MUST succeed & increment shopPurchaseCount to 2!
  bindings.buyUpstreamBlessing("buff_txhj_yanhuo", 60);
  assert.deepEqual(modalState.taixu.blessings.slice(-2), ["buff_txhj_aozhan", "buff_txhj_yanhuo"]);
  assert.equal(modalState.taixu.gold, 0);
  assert.equal(modalState.taixuUi.coinBalance, 0);
  assert.equal(modalState.taixuUi.shopPurchaseCount, 2);

  // Purchase #3 (xiongziyingfa, base 90) costs 90 + 2*200 = 490: advance x3 (+360 gold -> 360) MUST be blocked!
  bindings.advanceCampaignStep();
  bindings.advanceCampaignStep();
  bindings.advanceCampaignStep();
  assert.equal(modalState.taixu.gold, 360);
  bindings.buyUpstreamBlessing("buff_txhj_xiongziyingfa", 90);
  assert.ok(!modalState.taixu.blessings.includes("buff_txhj_xiongziyingfa"), "360 gold must NOT buy 490-coin 3rd blessing");
  assert.equal(modalState.taixuUi.shopPurchaseCount, 2);

  // Advance x2 more (+240 -> gold=600) => Purchase #3 succeeds at 490 -> gold=110, shopPurchaseCount=3!
  bindings.advanceCampaignStep();
  bindings.advanceCampaignStep();
  bindings.buyUpstreamBlessing("buff_txhj_xiongziyingfa", 90);
  assert.ok(modalState.taixu.blessings.includes("buff_txhj_xiongziyingfa"));
  assert.equal(modalState.taixu.gold, 110);
  assert.equal(modalState.taixuUi.coinBalance, 110);
  assert.equal(modalState.taixuUi.shopPurchaseCount, 3);

  // Buying buff_txhj_juguzhidao enables dec=1 discount immediately in synced taixuUi!
  for (let i = 0; i < 5; i += 1) bindings.advanceCampaignStep(); // +600 -> 710 gold
  const juguPrice = computeTaixuShopPrice(70, 3, false); // 70 + 3*200 = 670
  bindings.buyUpstreamBlessing("buff_txhj_juguzhidao", 70);
  assert.equal(modalState.taixu.gold, 710 - juguPrice);
  assert.equal(modalState.taixuUi.coinBalance, 710 - juguPrice);
  assert.equal(modalState.taixuUi.shopPurchaseCount, 4);
  assert.equal(modalState.taixuUi.hasJuguzhidaoBuff, true);
  assert.equal(computeTaixuShopPrice(100, modalState.taixuUi.shopPurchaseCount, modalState.taixuUi.hasJuguzhidaoBuff), 700);
});
