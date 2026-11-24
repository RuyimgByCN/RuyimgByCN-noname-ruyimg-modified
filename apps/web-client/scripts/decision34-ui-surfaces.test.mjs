/**
 * Decision 34 Pure UI/Skin/Audio Frontend Surfaces Verification Suite:
 * - L09 千幻聆音 (8 UI hooks + 12 CSS/surface presets + skinShare.js)
 * - L24 皮肤切换 (22 UI hooks + 10 CSS/surface modules + elemental sha audio + flipX)
 * - L14 手杀ui (53 UI hooks + 40 JS/CSS surfaces + muniu hand sorting)
 * - L08 十周年UI (319 UI hooks incl. 286 markskill.js badges + 43 JS/CSS surfaces)
 */

import assert from "node:assert/strict";
import test from "node:test";

import {
  DECADE_ALL_UI_HOOK_IDS,
  DECADE_CORE_HOOK_IDS,
  DECADE_MARKSKILL_SPECS,
  DECADE_SURFACE_MODULES,
  QHLY_GROUP_COLORS,
  QHLY_SURFACE_PRESETS,
  QHLY_UI_HOOK_IDS,
  SHOUSHA_SURFACE_MODULES,
  SHOUSHA_UI_HOOK_IDS,
  SKIN_SWITCH_SURFACE_IDS,
  SKIN_SWITCH_UI_HOOK_IDS,
  TAIXU_DLC_SEASONS,
  TAIXU_SERVANT_SPECS,
  TAIXU_SURFACE_STYLESHEETS,
  ZUAN_UI_HOOK_IDS,
  computeTaixuShopPrice,
  createInitialDecadeUiRuntimeState,
  createInitialExtensionSurfacesState,
  createInitialQhlyRuntimeState,
  createInitialShoushaUiRuntimeState,
  createInitialSkinSwitchRuntimeState,
  createInitialTaixuUiRuntimeState,
  createInitialZuanUiRuntimeState,
  deserializeExtensionSurfacesSnapshot,
  dispatchDecadeUiHook,
  dispatchQhlyUiHook,
  dispatchShoushaUiHook,
  dispatchSkinSwitchHook,
  dispatchZuanUiHook,
  formatDecadeSkillMarkBadge,
  purchaseTaixuMerchantItem,
  resolveQhlySkinShare,
  selectTaixuServantSurface,
  serializeExtensionSurfacesSnapshot,
  sortShoushaHandCards,
  sortSuitsDecadeSymbols,
  switchTaixuViewport,
  updateSkinSwitchConfigInState,
} from "../src/engine/extensionSurfaceRuntime.ts";

test("L09 千幻聆音: implements all 8 UI hooks, 12 surface presets, and skinShare.js voice resolution", () => {
  // Upstream: archive/upstream/noname-liuli/noname/extension/千幻聆音/{extension.js:1793-14515, skinShare.js:20-1727}
  assert.equal(QHLY_UI_HOOK_IDS.length, 8);
  assert.equal(Object.keys(QHLY_SURFACE_PRESETS).length, 12);
  assert.equal(QHLY_GROUP_COLORS.wei, "#0000CD");
  assert.equal(QHLY_GROUP_COLORS.shu, "#B22222");

  // skinShare.js:196-199: sb_caopi -> re_caopi, sbxingshang -> rexingshang
  const shared = resolveQhlySkinShare("sb_caopi", "sbxingshang");
  assert.deepEqual(shared, {
    resolvedGeneral: "re_caopi",
    resolvedSkill: "rexingshang",
    shared: true,
  });
  const unmapped = resolveQhlySkinShare("caocao", "jianxiong");
  assert.deepEqual(unmapped, {
    resolvedGeneral: "caocao",
    resolvedSkill: "jianxiong",
    shared: false,
  });

  let state = createInitialQhlyRuntimeState();
  // 1. _qhcreateYH (extension.js:1793): dead player stops dynamic & syncs yuanhu faction
  state = dispatchQhlyUiHook(state, { hook: "_qhcreateYH", seat: 2, faction: "wu", isDead: true });
  assert.ok(state.dynamicStoppedSeats.includes(2));
  assert.equal(state.yuanhuSyncedSeats[2], "wu");

  // 2. _qhlyChangeSkin (extension.js:1809): negative hpDelta switches to hp_wounded
  state = dispatchQhlyUiHook(state, { hook: "_qhlyChangeSkin", generalId: "caocao", hpDelta: -1 });
  assert.equal(state.replaceSkinByGeneral.caocao, "hp_wounded");

  // 3. _qhlyChageKillingSkin (extension.js:1845): switches killer to 'kill' skin
  state = dispatchQhlyUiHook(state, { hook: "_qhlyChageKillingSkin", killerGeneralId: "sb_caopi" });
  assert.equal(state.replaceSkinByGeneral.sb_caopi, "kill");

  // 4. _qhlyCheckSkin (extension.js:1859): initializes main + deputy general skins
  state = dispatchQhlyUiHook(state, {
    hook: "_qhlyCheckSkin",
    seat: 1,
    generalId: "liubei",
    deputyGeneralId: "guanyu",
    faction: "shu",
  });
  assert.equal(state.replaceSkinByGeneral.liubei, "origin");
  assert.equal(state.replaceSkinByGeneral.guanyu, "origin");

  // 5. _qhly_bgm (extension.js:4017): resolves via skinShare and plays once per match (bgmFlag)
  state = dispatchQhlyUiHook(state, { hook: "_qhly_bgm", generalId: "sb_caopi" });
  assert.equal(state.bgmFlag, true);
  assert.equal(state.activeBgmTrack, "qhly_bgm/re_caopi.mp3");
  const afterSecondBgm = dispatchQhlyUiHook(state, { hook: "_qhly_bgm", generalId: "caocao" });
  assert.equal(afterSecondBgm.activeBgmTrack, "qhly_bgm/re_caopi.mp3");

  // 6. _qhly_autoc (extension.js:14313): periodic rotation
  state = dispatchQhlyUiHook(state, { hook: "_qhly_autoc", generalId: "caocao" });
  assert.equal(state.autoRotateCount, 1);
  assert.equal(state.replaceSkinByGeneral.caocao, "skin_2");

  // 7. _qhly_randskin (extension.js:14332): skips selfSeat when randSkinSkipSelf is true
  const beforeSelf = state.replaceSkinByGeneral.caocao;
  state = dispatchQhlyUiHook(state, { hook: "_qhly_randskin", seat: 0, selfSeat: 0, generalId: "caocao" });
  assert.equal(state.replaceSkinByGeneral.caocao, beforeSelf);
  state = dispatchQhlyUiHook(state, { hook: "_qhly_randskin", seat: 3, selfSeat: 0, generalId: "sunquan" });
  assert.equal(state.replaceSkinByGeneral.sunquan, "rand_seat_3");

  // 8. _qhly_addButton (extension.js:14465): global sync vs per-general-skin drag persistence
  state = dispatchQhlyUiHook(state, {
    hook: "_qhly_addButton",
    generalId: "caocao",
    dragOffset: { leftPx: 24, topPx: 18 },
  });
  assert.deepEqual(state.dragButtonPositionAll, { leftPx: 24, topPx: 18 });
  state = { ...state, dragSyncMode: "no" };
  state = dispatchQhlyUiHook(state, {
    hook: "_qhly_addButton",
    generalId: "caocao",
    dragOffset: { leftPx: 40, topPx: 12 },
  });
  assert.deepEqual(state.dragButtonPositionByKey.qhly_dragButtonPositionOf_caocao_skin_2, {
    leftPx: 40,
    topPx: 12,
  });
});

test("L24 皮肤切换: implements all 22 UI hooks, 10 CSS/animation surfaces, elemental sha audio, and attack flipX", () => {
  // Upstream: archive/upstream/noname-liuli/noname/extension/皮肤切换/extension.js:176-1680
  assert.equal(SKIN_SWITCH_UI_HOOK_IDS.length, 22);
  assert.equal(SKIN_SWITCH_SURFACE_IDS.length, 10);

  let state = createInitialSkinSwitchRuntimeState();
  // extension.js:598-629: _playAudioToQueue maps elemental sha -> sha_fire / sha_thunder, non-sha unchanged
  state = dispatchSkinSwitchHook(state, {
    hook: "_playAudioToQueue",
    generalId: "caocao",
    skinId: "primary_2",
    cardName: "sha",
    cardNature: "fire",
    timestampMs: 1001,
  });
  state = dispatchSkinSwitchHook(state, {
    hook: "_playAudioToQueue",
    generalId: "caocao",
    skinId: "primary_2",
    cardName: "shan",
    cardNature: "fire",
    timestampMs: 1002,
  });
  assert.equal(state.audioPlayQueue[0]?.card, "sha_fire");
  assert.equal(state.audioPlayQueue[1]?.card, "shan");

  // extension.js:337 & 11702: _gj flips X when seatScreenXPct < 50 and isAttackFlipX === true (default init is false)
  assert.equal(state.isAttackFlipX, false);
  state = updateSkinSwitchConfigInState(state, { isAttackFlipX: true }, { persist: false });
  state = dispatchSkinSwitchHook(state, { hook: "_gj", seat: 2, seatScreenXPct: 22 });
  assert.deepEqual(state.lastChukuangAction, { seat: 2, action: "GongJi", flipX: true });
  state = dispatchSkinSwitchHook(state, { hook: "_gj", seat: 1, seatScreenXPct: 75 });
  assert.deepEqual(state.lastChukuangAction, { seat: 1, action: "GongJi", flipX: false });

  // extension.js:632-643: _checkDcdChuChang suppressed on doubleAvatar
  const prevAction = state.lastChukuangAction;
  state = dispatchSkinSwitchHook(state, { hook: "_checkDcdChuChang", seat: 0, doubleAvatar: true });
  assert.deepEqual(state.lastChukuangAction, prevAction);
  state = dispatchSkinSwitchHook(state, { hook: "_checkDcdChuChang", seat: 0, doubleAvatar: false });
  assert.deepEqual(state.lastChukuangAction, { seat: 0, action: "chuchang", flipX: false });

  // Exercise remaining hooks across all 22 hook IDs
  for (const hook of SKIN_SWITCH_UI_HOOK_IDS) {
    state = dispatchSkinSwitchHook(state, {
      hook,
      seat: 0,
      generalId: "caocao",
      cardName: hook === "_checkDcdShan" ? "shan" : "sha",
      roundNumber: 2,
      roundThreshold: 2,
      hp: 1,
      damageAmount: 2,
      faction: "jin",
    });
  }
  assert.equal(state.yuanhuGroupBySeat[0], "jin");
  assert.equal(state.specialFormBySeat[0], "phase_idle");
});

test("L14 手杀ui: implements all 53 UI hooks, 40 JS/CSS surfaces, and muniu-first hand card sorting", () => {
  // Upstream: archive/upstream/noname-liuli/noname/extension/手杀ui/{extension.js:85-6450, lbtn/main1.js:495-552}
  assert.equal(SHOUSHA_UI_HOOK_IDS.length, 53);
  assert.equal(SHOUSHA_SURFACE_MODULES.length, 40);

  // lbtn/main1.js:514-552 (_paixu_paixu_paixu): muniu cards come first, sorted by name -> suit -> number
  const unsorted = [
    { id: 10, name: "shan", suit: "diamond", number: 9 },
    { id: 11, name: "sha", suit: "club", number: 7 },
    { id: 12, name: "tao", suit: "heart", number: 3, muniu: true },
    { id: 13, name: "sha", suit: "spade", number: 10 },
    { id: 14, name: "sha", suit: "spade", number: 2 },
    { id: 15, name: "juedou", suit: "spade", number: 1, muniu: true },
  ];
  const sorted = sortShoushaHandCards(unsorted);
  assert.deepEqual(
    sorted.map((c) => c.id),
    [15, 12, 14, 13, 11, 10],
  );

  let state = createInitialShoushaUiRuntimeState();
  // _menuCheat blocked in onlineRoom (extension.js:103)
  state = dispatchShoushaUiHook(state, "_menuCheat", { onlineRoom: true });
  assert.equal(state.cheatConsoleOpen, false);
  state = dispatchShoushaUiHook(state, "_menuCheat", { onlineRoom: false });
  assert.equal(state.cheatConsoleOpen, true);

  // Dispatch every single one of the 53 hooks and verify terminal state
  for (const hook of SHOUSHA_UI_HOOK_IDS) {
    state = dispatchShoushaUiHook(state, hook, { seat: 1, handCards: unsorted });
  }
  assert.deepEqual(state.sortedHandCardIds, [15, 12, 14, 13, 11, 10]);
  assert.equal(state.zlltOutcropEnabled, true);
  assert.equal(state.skillControlRev, 2);
});

test("L08 十周年UI: implements all 319 UI hooks (33 core + 286 markskill.js), 43 surfaces, and snapshot round-trip", () => {
  // Upstream: archive/upstream/noname-liuli/noname/extension/十周年UI/{extension.js, animation.js, markskill.js, meihua.js, splash.js}
  assert.equal(DECADE_CORE_HOOK_IDS.length, 33);
  assert.equal(DECADE_MARKSKILL_SPECS.length, 286);
  assert.equal(DECADE_ALL_UI_HOOK_IDS.length, 319);
  assert.equal(DECADE_SURFACE_MODULES.length, 43);

  // markskill.js:48-72: suit order spade -> heart -> club -> diamond -> ♠♥♣♦
  assert.equal(sortSuitsDecadeSymbols(["diamond", "heart", "spade", "club"]), "♠♥♣♦");

  // Verify bounded markskill.js label extraction (no bleed between nzry_chenglve1 and xionghuo)
  const chenglveYang = formatDecadeSkillMarkBadge("nzry_chenglve", { yang: true });
  assert.equal(chenglveYang?.label, "成略");
  assert.equal(chenglveYang?.badgeText, "阳");

  const xionghuoBadge = formatDecadeSkillMarkBadge("xionghuo", { count: 3 });
  assert.equal(xionghuoBadge?.label, "暴戾");
  assert.equal(xionghuoBadge?.badgeText, "3");

  let state = createInitialDecadeUiRuntimeState();
  for (const hookId of DECADE_ALL_UI_HOOK_IDS) {
    state = dispatchDecadeUiHook(state, hookId, {
      seat: 0,
      count: 2,
      yang: true,
      suits: ["club", "spade"],
      equipName: "renwang",
      handCount: 5,
    });
  }
  assert.equal(state.activeMarkBadgesBySeat[0]?.length, 286);
  assert.ok(state.virtualEquipsBySeat[0]?.includes("renwang"));
  assert.equal(state.handCountBadgeBySeat[0], 5);

  // Verify ExtensionSurfacesSnapshot serialization/deserialization preserves uiSuites
  const snap = {
    ...createInitialExtensionSurfacesState(),
    uiSuites: {
      qhly: createInitialQhlyRuntimeState(),
      skinSwitch: createInitialSkinSwitchRuntimeState(),
      shoushaUi: createInitialShoushaUiRuntimeState(),
      decadeUi: state,
    },
  };
  const raw = serializeExtensionSurfacesSnapshot(snap);
  const restored = deserializeExtensionSurfacesSnapshot(raw);
  assert.equal(restored.uiSuites?.decadeUi?.activeMarkBadgesBySeat[0]?.length, 286);
  assert.equal(restored.uiSuites?.qhly?.activeStylesheet, "newui_dc.css");
});

test("L25 祖安武将: implements all 40 frontend UI/SFX/judgment-overlay hooks", () => {
  // Upstream: archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:37514-48503
  assert.equal(ZUAN_UI_HOOK_IDS.length, 40);

  let state = createInitialZuanUiRuntimeState();
  // audio_draw mute suppresses paper draw/discard audio (extension.js:40597, 51272)
  state = { ...state, mutePaperAudio: true };
  state = dispatchZuanUiHook(state, "_extragainskill");
  assert.equal(state.lastAudioCue, null);

  state = { ...state, mutePaperAudio: false, anshaHiddenSeats: [1, 2] };
  for (const hook of ZUAN_UI_HOOK_IDS) {
    state = dispatchZuanUiHook(state, hook, { seat: 1, reviveTurns: 3, wallpaperId: "shihai_shu" });
  }
  assert.deepEqual(state.anshaHiddenSeats, [2]);
  assert.equal(state.reviveCountdownBySeat[1], 3);
  assert.equal(state.activeWallpaper, "shihai_shu");
  assert.equal(state.bossDefeatedBanner, true);
  assert.equal(state.luckyDrawShimmer, true);
  assert.equal(state.judgmentOverlay, "shandian");
});

test("L11 太虚幻境: implements all 4 upstream stylesheets, 8 DLC seasons, 21 servants, and juguzhidao shop price freeze", () => {
  // Upstream: archive/upstream/noname-liuli/noname/extension/太虚幻境/{extension_PcStyle.css,extension_MobileStyle.css,extension_servant.css,extension_style.css}
  assert.equal(TAIXU_SURFACE_STYLESHEETS.length, 4);
  assert.equal(TAIXU_DLC_SEASONS.length, 8);
  assert.equal(TAIXU_SERVANT_SPECS.length, 21);

  let state = createInitialTaixuUiRuntimeState(1280);
  assert.equal(state.responsive.activeStylesheet, "extension_PcStyle.css");
  assert.equal(state.responsive.unit, "vw");

  state = switchTaixuViewport(state, 720);
  assert.equal(state.responsive.activeStylesheet, "extension_MobileStyle.css");
  assert.equal(state.responsive.unit, "rem");

  state = selectTaixuServantSurface(state, "jiuwei");
  assert.equal(state.selectedServantId, "jiuwei");
  assert.equal(state.servantFrameClass, "frame3");

  // Without buff_txhj_juguzhidao (extension_framework.js:7008-7032): +200 per purchase count (50 -> 250 -> 450)
  assert.equal(computeTaixuShopPrice(50, 0, false), 50);
  assert.equal(computeTaixuShopPrice(50, 1, false), 250);
  assert.equal(computeTaixuShopPrice(50, 2, false), 450);
  state = purchaseTaixuMerchantItem(state, 50);
  assert.equal(state.coinBalance, 150);
  assert.equal(state.currentShopPrice, 250);

  // With buff_txhj_juguzhidao (extension_buff.js:763 & extension_framework.js:7016): dec=1 free discount on first purchase, then +200 per extra count
  state = { ...state, hasJuguzhidaoBuff: true, coinBalance: 400 };
  assert.equal(computeTaixuShopPrice(50, 0, true), 50);
  assert.equal(computeTaixuShopPrice(50, 1, true), 50);
  assert.equal(computeTaixuShopPrice(50, 3, true), 450);
  state = purchaseTaixuMerchantItem(state, 50); // count 1 -> 2, dec=1 => pays base + (1-1)*200 = 50, next count=2 => 50 + (2-1)*200 = 250
  assert.equal(state.coinBalance, 350);
  assert.equal(state.currentShopPrice, 250);
});

test("Battle EngineEnvelope stream wires live CardPlayed / DamageDealt / PlayerDied events into uiSuites and eliminates fabricated labels", async () => {
  const { ingestBattleEnvelopeIntoSurfaces } = await import("../src/engine/extensionSurfaceRuntime.ts");
  const init = createInitialExtensionSurfacesState();
  const serializedInit = JSON.stringify(init);
  for (const forbidden of ["苍雷幼麟", "颍川前哨", "spshidi", "zhefu"]) {
    assert.ok(!serializedInit.includes(forbidden), `initial surface snapshot must not contain fabricated ${forbidden}`);
  }
  assert.ok(init.uiSuites?.decadeUi, "initial snapshot must initialize decadeUi suite");
  assert.ok(init.uiSuites?.qhly, "initial snapshot must initialize qhly suite");
  assert.ok(init.uiSuites?.skinSwitch, "initial snapshot must initialize skinSwitch suite");
  assert.ok(init.uiSuites?.shoushaUi, "initial snapshot must initialize shoushaUi suite");
  assert.ok(init.uiSuites?.taixuUi, "initial snapshot must initialize taixuUi suite");
  assert.ok(init.uiSuites?.zuanUi, "initial snapshot must initialize zuanUi suite");

  const afterCombat = ingestBattleEnvelopeIntoSurfaces(init, {
    revision: 7,
    finished: false,
    view: {
      round: 2,
      current_player: 0,
      seats: [
        { player: 0, alive: true, hp: 3, max_hp: 4, hand_count: 4, general: "caocao" },
        { player: 1, alive: true, hp: 2, max_hp: 4, hand_count: 2, general: "liubei" },
      ],
    },
    events: [
      { type: "CardPlayed" },
      { type: "DamageDealt" },
    ],
  });

  // L08 hand count badge synced from view.seats, L24 audio/action triggered on CardPlayed, L09 skin reacts to DamageDealt, L11 servant clip advances on DamageDealt
  assert.equal(afterCombat.uiSuites?.decadeUi?.handCountBadgeBySeat[0], 4);
  assert.ok((afterCombat.uiSuites?.skinSwitch?.audioPlayQueue.length ?? 0) >= 1);
  assert.equal(afterCombat.uiSuites?.qhly?.replaceSkinByGeneral.liubei, "hp_wounded");
  assert.equal(afterCombat.uiSuites?.taixuUi?.lastServantClip, "gongji1");

  // 2. Real WASM presentation.occurrences + Network PublicEvent servant clip routing
  // 2a. WASM presentation card_used(actor:1 wuzhong/tao/jiu) -> "jinnang1"; damage_dealt(target:1 while curPlayer=0) -> "shouji1" (NOT "gongji1"!)
  let wasmState = createInitialExtensionSurfacesState();
  wasmState = ingestBattleEnvelopeIntoSurfaces(wasmState, {
    result: null,
    events: [{ PublicEvent: { code: 2, label: "摸牌" } }],
    presentation: { revision: 1, status: "complete", unavailable: [], occurrences: [] },
    choice: null,
    hash_hex: "b".repeat(64),
    revision: 1,
    finished: false,
    view: {
      observer: 0,
      round: 1,
      current_player: 0,
      phase: "Draw",
      state_hash: "b",
      known_cards: [],
      seats: [
        { player: 0, seat: 0, character_def: 0, alive: true, hp: 4, max_hp: 4, hand_count: 4, skills: [], turned_over: false, identity: null },
        { player: 1, seat: 1, character_def: 1, alive: true, hp: 4, max_hp: 4, hand_count: 4, skills: [], turned_over: false, identity: null },
      ],
      outcome: null,
    },
    legal_actions: [],
  });
  assert.equal(wasmState.uiSuites?.taixuUi?.lastServantClip, "daiji1", "Draw PublicEvent code:2 without card_used must remain daiji1");

  wasmState = ingestBattleEnvelopeIntoSurfaces(wasmState, {
    result: null,
    events: [{ PublicEvent: { code: 3, label: "使用或响应牌" } }],
    presentation: {
      revision: 2,
      status: "complete",
      unavailable: [],
      occurrences: [
        {
          ordinal: 0,
          event: {
            kind: "card_used",
            actor: 1,
            targets: null,
            effective: { kind: "wuzhong", key: "wuzhong", nature: "normal" },
            voice: null,
          },
        },
      ],
    },
    choice: null,
    hash_hex: "c".repeat(64),
    revision: 2,
    finished: false,
    view: {
      observer: 0,
      round: 1,
      current_player: 0,
      phase: "Play",
      state_hash: "c",
      known_cards: [],
      seats: [
        { player: 0, seat: 0, character_def: 0, alive: true, hp: 4, max_hp: 4, hand_count: 5, skills: [], turned_over: false, identity: null },
        { player: 1, seat: 1, character_def: 1, alive: true, hp: 4, max_hp: 4, hand_count: 4, skills: [], turned_over: false, identity: null },
      ],
      outcome: null,
    },
    legal_actions: [],
  });
  assert.equal(wasmState.uiSuites?.taixuUi?.lastServantClip, "jinnang1", "WASM presentation card_used(wuzhong) triggers jinnang1");

  // WASM presentation damage_dealt targeting observer=0 on own turn (curPlayer=0) -> "shouji1", NOT "gongji1"
  wasmState = ingestBattleEnvelopeIntoSurfaces(wasmState, {
    result: null,
    events: [{ PublicEvent: { code: 4, label: "伤害结算" } }],
    presentation: {
      revision: 3,
      status: "complete",
      unavailable: [],
      occurrences: [
        {
          ordinal: 0,
          event: {
            kind: "damage_dealt",
            source: 2,
            target: 1,
            amount: 1,
            nature: "normal",
          },
        },
      ],
    },
    choice: null,
    hash_hex: "d".repeat(64),
    revision: 3,
    finished: false,
    view: {
      observer: 0,
      round: 1,
      current_player: 0,
      phase: "Play",
      state_hash: "d",
      known_cards: [],
      seats: [
        { player: 0, seat: 0, character_def: 0, alive: true, hp: 3, max_hp: 4, hand_count: 5, skills: [], turned_over: false, identity: null },
        { player: 1, seat: 1, character_def: 1, alive: true, hp: 4, max_hp: 4, hand_count: 4, skills: [], turned_over: false, identity: null },
      ],
      outcome: null,
    },
    legal_actions: [],
  });
  assert.equal(wasmState.uiSuites?.taixuUi?.lastServantClip, "shouji1", "WASM presentation damage_dealt on observer during own turn triggers shouji1");

  // 2b. Network/bare PublicEvent without presentation:
  // PublicEvent code:3 with TrickWuzhong in known_cards -> "jinnang1"; subsequent Sha in DiscardPile does NOT re-trigger old TrickWuzhong!
  let netState = createInitialExtensionSurfacesState();
  netState = ingestBattleEnvelopeIntoSurfaces(netState, {
    result: null,
    events: [{ PublicEvent: { code: 3, label: "使用或响应牌" } }],
    choice: null,
    hash_hex: "e".repeat(64),
    revision: 1,
    finished: false,
    view: {
      observer: 0,
      round: 1,
      current_player: 0,
      phase: "Play",
      state_hash: "e",
      known_cards: [{ id: 88, def: 91, suit: "Heart", rank: 7, owner: null, zone: "DiscardPile", kind: "TrickWuzhong" }],
      seats: [
        { player: 0, seat: 0, character_def: 0, alive: true, hp: 4, max_hp: 4, hand_count: 4, skills: [], turned_over: false, identity: null },
        { player: 1, seat: 1, character_def: 1, alive: true, hp: 4, max_hp: 4, hand_count: 4, skills: [], turned_over: false, identity: null },
      ],
      outcome: null,
    },
    legal_actions: [],
  });
  assert.equal(netState.uiSuites?.taixuUi?.lastServantClip, "jinnang1", "Network PublicEvent code:3 with TrickWuzhong in DiscardPile triggers jinnang1");
  assert.equal(netState.uiSuites?.skinSwitch?.audioPlayQueue?.[0]?.card, "wuzhong", "L24 skinSwitch sees new discard wuzhong before seenKnownDiscardIds updates");

  // Subsequent Sha after already-seen TrickWuzhong must NOT falsely trigger jinnang1
  netState = {
    ...netState,
    uiSuites: {
      ...netState.uiSuites,
      taixuUi: { ...netState.uiSuites.taixuUi, lastServantClip: "gongji1" },
    },
  };
  netState = ingestBattleEnvelopeIntoSurfaces(netState, {
    result: null,
    events: [{ PublicEvent: { code: 3, label: "使用或响应牌" } }],
    choice: null,
    hash_hex: "e2".repeat(32),
    revision: 2,
    finished: false,
    view: {
      observer: 0,
      round: 1,
      current_player: 0,
      phase: "Play",
      state_hash: "e2",
      known_cards: [
        { id: 88, def: 91, suit: "Heart", rank: 7, owner: null, zone: "DiscardPile", kind: "TrickWuzhong" },
        { id: 89, def: 0, suit: "Spade", rank: 7, owner: null, zone: "DiscardPile", kind: "Sha" },
      ],
      seats: [
        { player: 0, seat: 0, character_def: 0, alive: true, hp: 4, max_hp: 4, hand_count: 3, skills: [], turned_over: false, identity: null },
        { player: 1, seat: 1, character_def: 1, alive: true, hp: 4, max_hp: 4, hand_count: 4, skills: [], turned_over: false, identity: null },
      ],
      outcome: null,
    },
    legal_actions: [],
  });
  assert.equal(netState.uiSuites?.taixuUi?.lastServantClip, "gongji1", "Subsequent Sha must NOT re-trigger seen TrickWuzhong");
  assert.equal(netState.uiSuites?.skinSwitch?.audioPlayQueue?.[1]?.card, "sha", "L24 skinSwitch sees subsequent Sha discard");

  // Observer hurt on curPlayer === observer via bare PublicEvent code:4 (no presentation) -> "shouji1", NOT "gongji1"!
  netState = ingestBattleEnvelopeIntoSurfaces(netState, {
    result: null,
    events: [{ PublicEvent: { code: 4, label: "伤害结算" } }],
    choice: null,
    hash_hex: "f".repeat(64),
    revision: 2,
    finished: false,
    view: {
      observer: 0,
      round: 1,
      current_player: 0,
      phase: "Play",
      state_hash: "f",
      known_cards: [{ id: 88, def: 91, suit: "Heart", rank: 7, owner: null, zone: "DiscardPile", kind: "TrickWuzhong" }],
      seats: [
        { player: 0, seat: 0, character_def: 0, alive: true, hp: 3, max_hp: 4, hand_count: 4, skills: [], turned_over: false, identity: null },
        { player: 1, seat: 1, character_def: 1, alive: true, hp: 4, max_hp: 4, hand_count: 4, skills: [], turned_over: false, identity: null },
      ],
      outcome: null,
    },
    legal_actions: [],
  });
  assert.equal(netState.uiSuites?.taixuUi?.lastServantClip, "shouji1", "Bare PublicEvent code:4 when observer HP dropped on curPlayer=0 triggers shouji1");

  // Stateless bare PublicEvent code:4 when ONLY observer (curPlayer=0) is wounded (no prevHp) -> "shouji1"
  const statelessSelfHurt = ingestBattleEnvelopeIntoSurfaces(createInitialExtensionSurfacesState(), {
    result: null,
    events: [{ PublicEvent: { code: 4, label: "伤害结算" } }],
    choice: null,
    hash_hex: "1".repeat(64),
    revision: 4,
    finished: false,
    view: {
      observer: 0,
      round: 1,
      current_player: 0,
      phase: "Play",
      state_hash: "1",
      known_cards: [],
      seats: [
        { player: 0, seat: 0, character_def: 0, alive: true, hp: 3, max_hp: 4, hand_count: 4, skills: [], turned_over: false, identity: null },
        { player: 1, seat: 1, character_def: 1, alive: true, hp: 4, max_hp: 4, hand_count: 4, skills: [], turned_over: false, identity: null },
      ],
      outcome: null,
    },
    legal_actions: [],
  });
  assert.equal(statelessSelfHurt.uiSuites?.taixuUi?.lastServantClip, "shouji1", "Stateless bare PublicEvent code:4 with only observer wounded triggers shouji1");
});


