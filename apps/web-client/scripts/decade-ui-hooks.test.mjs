import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  DECADE_ALL_UI_HOOK_IDS,
  DECADE_CORE_HOOK_IDS,
  DECADE_CORE_HOOK_SPECS,
  DECADE_MARKSKILL_SPECS,
  DECADE_NONUMBER_MARK_IDS,
  DECADE_NOSKILL_CONTROL_IDS,
  DECADE_PLAYER_NODE_KEYS,
  buildCardAuxOverlay,
  createInitialDecadeUiRuntimeState,
  createInitialExtensionSurfacesState,
  deriveGeneralRarityTier,
  dispatchDecadeUiHook,
  formatDecadeSkillMarkBadge,
  ingestBattleEnvelopeIntoSurfaces,
  resolveDecadeSeatBorderTier,
  sortCardTypesDecadeSymbols,
  sortSuitsDecadeSymbols,
  syncDecadeUiFromEnvelope,
  validatePoxiSelection,
} from "../src/engine/extensionSurfaceRuntime.ts";

const scriptsDir = dirname(fileURLToPath(import.meta.url));
const webClientDir = resolve(scriptsDir, "..");
const repoRoot = resolve(webClientDir, "..", "..");

function loadInventoryHooks() {
  const invPath = resolve(repoRoot, "docs/migration/g0/inventory_status.json");
  const inv = JSON.parse(readFileSync(invPath, "utf8"));
  return Object.values(inv.items)
    .filter((item) => item.source_id === "L08" && item.category === "ui_hooks")
    .map((item) => item.id);
}

test("L08 319 ui_hooks match g0/inventory_status.json 1-to-1", () => {
  const inventoryHooks = loadInventoryHooks();
  assert.equal(inventoryHooks.length, 319);
  assert.equal(DECADE_CORE_HOOK_IDS.length, 33);
  assert.equal(DECADE_CORE_HOOK_SPECS.length, 33);
  assert.equal(DECADE_MARKSKILL_SPECS.length, 286);
  assert.equal(DECADE_ALL_UI_HOOK_IDS.length, 319);

  const implementedSet = new Set(DECADE_ALL_UI_HOOK_IDS);
  assert.equal(implementedSet.size, 319, "Duplicate hook id detected");
  for (const hookId of inventoryHooks) {
    assert.ok(implementedSet.has(hookId), `Missing L08 ui_hook in runtime: ${hookId}`);
  }
});

test("L08 markskill metadata: 32 nonumberMark, 4 noskillControl, sortBySuit & sortByType", () => {
  assert.equal(DECADE_NONUMBER_MARK_IDS.length, 32);
  assert.equal(DECADE_NOSKILL_CONTROL_IDS.length, 4);

  for (const id of DECADE_NONUMBER_MARK_IDS) {
    const badge = formatDecadeSkillMarkBadge(id, { count: 9 });
    assert.ok(badge, `Expected mark badge for ${id}`);
    assert.equal(badge.nonumberMark, true);
    assert.equal(badge.badgeText, "态", `nonumberMark ${id} must display state glyph`);
    assert.ok(badge.tooltip.includes(badge.label));
  }

  for (const id of DECADE_NOSKILL_CONTROL_IDS) {
    const badge = formatDecadeSkillMarkBadge(id, { count: 2 });
    assert.ok(badge, `Expected mark badge for ${id}`);
    assert.equal(badge.noskillControl, true);
  }

  assert.equal(sortSuitsDecadeSymbols(["diamond", "heart", "spade", "club"]), "♠♥♣♦");
  assert.equal(sortCardTypesDecadeSymbols(["equip", "basic", "trick"]), "基锦装");

  const rejingce = formatDecadeSkillMarkBadge("rejingce_add", {
    cardTypes: ["trick", "basic"],
    suits: ["heart"],
  });
  assert.ok(rejingce);
  assert.equal(rejingce.badgeText, "基锦♥");

  const huamu = formatDecadeSkillMarkBadge("huamu", { count: 2 });
  assert.ok(huamu);
  assert.equal(huamu.label, "灵杉&玉树", "huamu must use intro.name, never \\0");
});

test("All 33 core UI hooks dispatch deterministically without mutating callers", () => {
  let state = createInitialDecadeUiRuntimeState();

  state = dispatchDecadeUiHook(state, "_jiuwo_", { seat: 1 });
  assert.deepEqual(state.jiuwoDyingSeats, [1]);
  state = dispatchDecadeUiHook(state, "_delete_jiuwo_", { seat: 1 });
  assert.deepEqual(state.jiuwoDyingSeats, []);

  state = dispatchDecadeUiHook(state, "_dyjinengtx", {
    seat: 0,
    skillName: "武圣",
    generalName: "关羽",
    faction: "shu",
  });
  assert.equal(state.skillPopup?.skillName, "武圣");
  assert.equal(state.factionGlowBySeat[0], "shu");

  state = dispatchDecadeUiHook(state, "_losetl_", { seat: 2, delta: 2 });
  assert.ok(state.loseHpSeats.includes(2));
  assert.equal(state.hpDeltaBySeat[2]?.delta, -2);

  state = dispatchDecadeUiHook(state, "_jinnangpai_", { cardName: "wanjian" });
  assert.equal(state.trickEffectBanner?.card, "wanjian");

  state = dispatchDecadeUiHook(state, "_jiubuff_", { seat: 0 });
  assert.ok(state.jiuBuffSeats.includes(0));
  state = dispatchDecadeUiHook(state, "_jiubuff_xiaoshi_", { seat: 0 });
  assert.ok(!state.jiuBuffSeats.includes(0));

  state = dispatchDecadeUiHook(state, "_player_hp", { seat: 0, delta: 1, hpKind: "recover" });
  assert.deepEqual(state.hpDeltaBySeat[0], { delta: 1, kind: "recover" });
  state = { ...state, config: { ...state.config, EffectWood: true } };
  state = dispatchDecadeUiHook(state, "_HpRecover", { seat: 0 });
  assert.ok(state.healAuraSeats.includes(0));
  state = dispatchDecadeUiHook(state, "_Hplose", { seat: 2, delta: 1 });
  assert.ok(state.damageFlashSeats.includes(2));

  state = dispatchDecadeUiHook(state, "_killFire", { seat: 0 });
  assert.ok(state.killFireSeats.includes(0));
  state = dispatchDecadeUiHook(state, "_changeEquipExt", { seat: 1, hidden: true });
  assert.equal(state.equipHiddenBySeat[1], true);
  state = dispatchDecadeUiHook(state, "_ui_auto_cheat", { active: true });
  assert.ok(state.autoCheatNotice);

  state = dispatchDecadeUiHook(state, "_kill_framebg", { seat: 3, rarity: "legend" });
  state = dispatchDecadeUiHook(state, "_rarity", { seat: 4, rarity: "epic" });
  assert.equal(state.rarityTierBySeat[3], "legend");
  assert.equal(state.rarityTierBySeat[4], "epic");
  assert.equal(resolveDecadeSeatBorderTier({ borderLevel: "kill" }, state.rarityTierBySeat[3]), "five");
  assert.equal(resolveDecadeSeatBorderTier({ borderLevel: "kill" }, state.rarityTierBySeat[4]), "four");
  assert.equal(resolveDecadeSeatBorderTier({ borderLevel: "two" }, "legend"), "two");

  state = dispatchDecadeUiHook(state, "_tiesuo_ying", { seat: 1 });
  assert.ok(state.tiesuoSeats.includes(1));
  state = { ...state, config: { ...state.config, EffectWood: true, cardUseEffect: true, playerKillEffect: true } };
  state = dispatchDecadeUiHook(state, "_decadeUI_usecardBegin", { seat: 0, equipName: "杀" });
  assert.equal(state.cardUseCueBySeat[0], "杀");
  state = dispatchDecadeUiHook(state, "_usecard", { seat: 0, clear: true });
  assert.equal(state.cardUseCueBySeat[0], undefined);
  state = dispatchDecadeUiHook(state, "_discard", { seat: 2 });
  assert.ok(!state.damageFlashSeats.includes(2));
  state = dispatchDecadeUiHook(state, "_decadeUI_dieKillEffect", {
    killerName: "关羽",
    victimName: "华雄",
  });
  assert.equal(state.killOverlay?.victimName, "华雄");

  state = dispatchDecadeUiHook(state, "_recasting", { seat: 0 });
  assert.equal(state.recastCountBySeat[0], 1);
  state = dispatchDecadeUiHook(state, "_xunizhuangbei_gua_gua", { seat: 0, equipName: "虚拟八卦阵" });
  assert.ok(state.virtualEquipsBySeat[0].includes("虚拟八卦阵"));
  state = dispatchDecadeUiHook(state, "_gshoupai", { seat: 1, handCount: 5 });
  assert.equal(state.handCountBadgeBySeat[1], 5);
  assert.ok(state.publicHandSeats.includes(1));

  state = dispatchDecadeUiHook(state, "drlt_poxi", { seat: 2 });
  assert.equal(state.poxiDialog?.mode, "drlt_poxi");
  state = dispatchDecadeUiHook(state, "pro_drlt_poxi", { seat: 3 });
  assert.equal(state.poxiDialog?.mode, "pro_drlt_poxi");

  for (const splashHook of ["_gzyinni", "_guozhan", "_gzyinniszn", "_guozhanszn", "_yinni"]) {
    const next = dispatchDecadeUiHook(state, splashHook, { seat: 1, revealed: false });
    assert.equal(next.splashMode, "guozhan_hidden");
    assert.ok(next.hiddenSeats.includes(1));
  }
  for (const dualHook of ["_shousha_shuangjiang", "_shizhounian_shuangjiang", "_shizhounian_shuangjiang2"]) {
    const next = dispatchDecadeUiHook(state, dualHook, { seat: 0 });
    assert.notEqual(next.splashMode, "standard");
  }
});

test("All 286 markskill hooks dispatch cleanly + poxi validation & cardAuxOverlay", () => {
  let state = createInitialDecadeUiRuntimeState();
  for (const spec of DECADE_MARKSKILL_SPECS) {
    state = dispatchDecadeUiHook(state, spec.id, {
      seat: 0,
      count: 2,
      yang: true,
      suits: ["heart", "spade"],
      cardTypes: ["basic", "trick"],
    });
  }
  assert.equal(state.activeMarkBadgesBySeat[0].length, 286);

  assert.equal(validatePoxiSelection("drlt_poxi", [
    { id: 1, suit: "Spade" },
    { id: 2, suit: "Heart" },
    { id: 3, suit: "Club" },
    { id: 4, suit: "Diamond" },
  ]).valid, true);
  assert.equal(validatePoxiSelection("drlt_poxi", [
    { id: 1, suit: "Spade" },
    { id: 2, suit: "Spade" },
    { id: 3, suit: "Club" },
    { id: 4, suit: "Diamond" },
  ]).valid, false);
  assert.equal(validatePoxiSelection("pro_drlt_poxi", [{ id: 1, suit: "Heart" }]).valid, true);
  assert.equal(validatePoxiSelection("pro_drlt_poxi", []).valid, false);

  assert.equal(DECADE_PLAYER_NODE_KEYS.length, 20);
  assert.equal(buildCardAuxOverlay("Armor"), "防具护体");
  assert.equal(buildCardAuxOverlay("ShaFire"), "属性杀 · 传伤");
  assert.equal(deriveGeneralRarityTier({ character_def: 0, identity: "Lord", generals: [] }), "legend");
});

test("syncDecadeUiFromEnvelope projects hooks cleanly without mutating rule envelope", () => {
  const env = {
    view: {
      round: 1,
      phase: "Play",
      current_player: 0,
      seats: [
        {
          player: 0,
          seat: 0,
          hp: 3,
          max_hp: 4,
          hand_count: 4,
          alive: true,
          identity: "Lord",
          character_def: 1,
          generals: [
            { name: "诸葛亮", faction: "shu", def: 1, revealed: true },
            { name: "月英", faction: "shu", def: 2, revealed: false },
          ],
          skills: [
            { id: 1, name: "bazhen", use_count: 0 },
            { id: 2, name: "xionghuo", use_count: 2 },
          ],
        },
        {
          player: 1,
          seat: 1,
          hp: 0,
          max_hp: 4,
          hand_count: 2,
          alive: true,
          identity: "Rebel",
          character_def: 2,
          skills: [{ id: 3, name: "nzry_jieying", use_count: 0 }],
        },
      ],
    },
    events: [
      { type: "CardPlayed", card: "Jiu" },
      { type: "CardPlayed", card: "TrickNanman" },
      { type: "DamageDealt" },
      { type: "SkillActivated", skill: "空城" },
    ],
  };
  const beforeJson = JSON.stringify(env);

  let ui = createInitialDecadeUiRuntimeState();
  ui = syncDecadeUiFromEnvelope({ ...ui, prevHpBySeat: { 0: 4, 1: 2 } }, env);

  assert.equal(JSON.stringify(env), beforeJson, "UI sync MUST NOT mutate rule envelope");
  assert.ok(ui.jiuwoDyingSeats.includes(1));
  assert.ok(ui.tiesuoSeats.includes(1));
  assert.ok(ui.virtualEquipsBySeat[0].includes("八卦阵"));
  assert.ok(ui.hiddenSeats.includes(0));
  assert.ok(ui.jiuBuffSeats.includes(0));
  assert.equal(ui.trickEffectBanner?.card, "nanman");
  assert.equal(ui.hpDeltaBySeat[1]?.delta, -2);
  assert.equal(ui.skillPopup?.skillName, "空城");
  assert.ok(ui.activeMarkBadgesBySeat[0].some((m) => m.skillId === "xionghuo"));

  const extState = ingestBattleEnvelopeIntoSurfaces(createInitialExtensionSurfacesState(), env);
  assert.equal(extState.uiSuites.decadeUi.trickEffectBanner?.card, "nanman");

  // Initial general selection changing max_hp (4 -> 3) must NOT produce false damage/-1 badge
  let initUi = syncDecadeUiFromEnvelope(createInitialDecadeUiRuntimeState(), {
    revision: 1,
    view: {
      round: 1,
      phase: "Play",
      current_player: 1,
      seats: [
        { player: 0, seat: 0, hp: 5, max_hp: 5, hand_count: 4, alive: true, identity: "Lord", character_def: 0, skills: [] },
        { player: 1, seat: 1, hp: 4, max_hp: 4, hand_count: 4, alive: true, identity: "Loyalist", character_def: 0, skills: [] },
      ],
    },
    events: [],
  });
  initUi = syncDecadeUiFromEnvelope(initUi, {
    revision: 2,
    view: {
      round: 1,
      phase: "Play",
      current_player: 1,
      seats: [
        { player: 0, seat: 0, hp: 5, max_hp: 5, hand_count: 4, alive: true, identity: "Lord", character_def: 0, skills: [] },
        { player: 1, seat: 1, hp: 3, max_hp: 3, hand_count: 4, alive: true, identity: "Loyalist", character_def: 4, generals: [{ def: 4, name: "张角", faction: "qun", revealed: true }], skills: [] },
      ],
    },
    events: [],
  });
  assert.equal(initUi.hpDeltaBySeat[1], undefined, "General init max_hp shift must not emit false HP delta");
  assert.deepEqual(initUi.loseHpSeats, []);

  // Self-damage and opponent-recovery event attribution must hit exact seat, and expire on next phase
  const selfDmg = syncDecadeUiFromEnvelope(initUi, {
    revision: 3,
    view: {
      round: 1,
      phase: "Play",
      current_player: 1,
      seats: [
        { player: 0, seat: 0, hp: 4, max_hp: 5, hand_count: 4, alive: true, identity: "Lord", character_def: 0, skills: [] },
        { player: 1, seat: 1, hp: 2, max_hp: 3, hand_count: 4, alive: true, identity: "Loyalist", character_def: 4, generals: [{ def: 4, name: "张角", faction: "qun", revealed: true }], skills: [] },
      ],
    },
    events: [{ type: "DamageDealt", target: 1 }],
  });
  assert.equal(selfDmg.hpDeltaBySeat[1]?.delta, -1);
  assert.ok(selfDmg.damageFlashSeats.includes(1));
  assert.ok(!selfDmg.damageFlashSeats.includes(0));

  const nextPhase = syncDecadeUiFromEnvelope(selfDmg, {
    revision: 4,
    view: {
      round: 1,
      phase: "Discard",
      current_player: 1,
      seats: [
        { player: 0, seat: 0, hp: 4, max_hp: 5, hand_count: 4, alive: true, identity: "Lord", character_def: 0, skills: [] },
        { player: 1, seat: 1, hp: 2, max_hp: 3, hand_count: 4, alive: true, identity: "Loyalist", character_def: 4, generals: [{ def: 4, name: "张角", faction: "qun", revealed: true }], skills: [] },
      ],
    },
    events: [],
  });
  assert.deepEqual(nextPhase.hpDeltaBySeat, {}, "Transient HP delta must clear on next phase");
  assert.deepEqual(nextPhase.damageFlashSeats, []);
  assert.deepEqual(nextPhase.loseHpSeats, []);
});

test("Cleanroom boundary: surface runtime modules contain zero Node-only imports", () => {
  const files = [
    "src/engine/surfaces/decadeUiSurface.ts",
    "src/engine/surfaces/decadeHooksRuntime.ts",
    "src/engine/surfaces/decadeMarkSkillsData.ts",
    "src/engine/surfaces/decadeConfigSchema.ts",
    "src/engine/surfaces/decadeSurfacesRuntime.ts",
  ];
  for (const rel of files) {
    const src = readFileSync(resolve(webClientDir, rel), "utf8");
    assert.ok(!/from\s+["']node:/.test(src), `Forbidden node: import in ${rel}`);
    assert.ok(!/from\s+["'](fs|path|child_process|crypto|http)["']/.test(src), `Forbidden Node built-in in ${rel}`);
  }
});
