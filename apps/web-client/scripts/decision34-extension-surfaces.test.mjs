// 裁决 34（2026-11-08）：37 来源 deferred 渲染面与单机战役脚本上游规则对照、安全门禁与保存恢复回归测试。
//
// 覆盖来源：
// - U01 3D精选、U02 挑战BOSS、U04/L05 金币/假人、L01 AI微调、L02 SJ Settings、
//   L03 守卫剑阁、L07 全能搜索、L08 十周年UI、L09 千幻聆音、L11 太虚幻境、
//   L12 如真似幻、L13 手杀MVP、L14 手杀UI、L15 手杀全扩、L16 拖动阅读、
//   L17 斗转星移、L18 更新保护、L19 末日乱斗、L20 标记补充、L21 武皇5v5、
//   L22 天梯制裁、L23 电脑端适配、L24 皮肤切换、L25 祖安调试门禁、
//   L26 笨蛋插件、L27 选中美化、L28 惊雷天牢、L29 龙血玄黄。
//
// 运行：pnpm --dir apps/web-client test
import assert from "node:assert/strict";
import { test } from "node:test";

import {
  L20_HUD_MARK_SPECS,
  SPINE_CLIP_ORDER,
  U01_DDD_GALLERY_GENERALS,
  advanceLongxueDaoxin,
  advanceTaixuNode,
  applyMartialReplacement,
  applyZuanDiyDraft,
  assignMartialLineup,
  bindTaixuPet,
  buyCoinShopItem,
  buyTaixuBlessing,
  classifyDesktopPointer,
  compareExtensionVersion,
  createInitialExtensionSurfacesState,
  deserializeExtensionSurfacesSnapshot,
  foolFangtongAllows,
  foolZiyuanAllows,
  hiddenCardIdsFromSnapshot,
  incrementL20HudMark,
  ingestBattleEnvelopeIntoSurfaces,
  normalizeBossHp,
  numToRoma,
  placeCoinBet,
  recordMvpEventFact,
  resolveCoinHonorTitle,
  resolveEloDifficulty,
  restoreExtensionSurfacesForLoad,
  rewriteDouzhuanSkillText,
  rotateU01GalleryStandee,
  runOmnisearchQuery,
  selectL27DualGenerals,
  serializeExtensionSurfacesSnapshot,
  settleCoinMatch,
  triggerShoushaHudBanner,
  triggerSpineSkinClip,
  unlockLongxueSkills,
  validateSafeExportFilename,
} from "../src/engine/extensionSurfaceRuntime.ts";

test("U04/L05 金币商店价格、alwaysSnow 免费重载下雪、下注阶段门禁与录像回放隔离", () => {
  const s = createInitialExtensionSurfacesState();
  assert.equal(s.coin.balance, 100);
  assert.equal(s.coin.snowFall, false);

  // 购买下雪（20金）
  const c1 = buyCoinShopItem(s.coin, "snow");
  assert.equal(c1.balance, 80);
  assert.equal(c1.snowFall, true);
  assert.deepEqual(c1.activeEffects, ["snow"]);

  // alwaysSnow 开启时重载恢复下雪不扣费（上游 changeXue 边界）
  const cAlways = buyCoinShopItem({ ...c1, alwaysSnow: true }, "snow");
  assert.equal(cAlways.balance, 80);

  // 赛前下注 20 金
  const cBet = placeCoinBet(cAlways, 20);
  assert.equal(cBet.balance, 60);
  assert.equal(cBet.betPlaced, 20);
  // 同一局不可重复下注
  assert.throws(() => placeCoinBet(cBet, 20), /本局已下注/);
  // 对局阶段开始后不可下注
  assert.throws(() => placeCoinBet({ ...cAlways, phaseStarted: true }, 20), /对局阶段已开始/);

  // 录像回放模式结算不增减金币（上游 _status.video 边界）
  const cReplay = settleCoinMatch(cBet, {
    mode: "identity",
    role: "Lord",
    players: 8,
    won: true,
    unitRandom: 0,
    video: true,
  });
  assert.equal(cReplay.balance, 60);
  assert.equal(cReplay.lastSettlement?.netDelta, 0);

  // 正常身份局主公 8 人获胜结算：round(8 * 4.5) = 36 + 下注奖励 20 + 返还本金 20 = +76
  const cWin = settleCoinMatch(cBet, {
    mode: "identity",
    role: "Lord",
    players: 8,
    won: true,
    unitRandom: 0,
    video: false,
  });
  assert.equal(cWin.lastSettlement?.payout, 56);
  assert.equal(cWin.lastSettlement?.netDelta, 76);
  assert.equal(cWin.balance, 136);
  assert.equal(cWin.betPlaced, 0);

  // 余额不足时拒绝购买烟花（50金）
  assert.throws(() => buyCoinShopItem({ ...cWin, balance: 10 }, "fireworks"), /金币不足/);
});

test("L13 手杀 MVP 事件流计分、seq 幂等去重与信封注入", () => {
  let mvp = createInitialExtensionSurfacesState().mvp;
  mvp = recordMvpEventFact(mvp, { seq: 101, seat: 1, kind: "damage", amount: 2 });
  mvp = recordMvpEventFact(mvp, { seq: 102, seat: 1, kind: "kill_enemy" });
  mvp = recordMvpEventFact(mvp, { seq: 103, seat: 0, kind: "recover", amount: 1 });
  // P1: base 100 + damage(2*3=6) + kill(3) = 109
  assert.equal(mvp.scores[1].total, 109);
  assert.equal(mvp.scores[0].total, 102);
  assert.equal(mvp.mvpSeat, 1);

  // 重复 seq 不得二次累加
  mvp = recordMvpEventFact(mvp, { seq: 101, seat: 1, kind: "damage", amount: 2 });
  assert.equal(mvp.scores[1].total, 109);

  // 完整 EngineEnvelope 注入（推动 MVP、如真似幻罗马数字、龙血道心与终局金币结算）
  let s = createInitialExtensionSurfacesState();
  const env = {
    revision: 3,
    hash_hex: "aa".repeat(32),
    finished: true,
    result: null,
    choice: null,
    legal_actions: [],
    events: [
      { PublicEvent: { code: 4 } },
      { PublicEvent: { code: 2 } },
    ],
    view: {
      revision: 3,
      state_hash: "aa".repeat(32),
      observer: 0,
      round: 4,
      current_player: 0,
      phase: "Play",
      seats: [
        { player: 0, alive: true, hp: 4, max_hp: 4, hand_count: 4, turned_over: false, chained: false, identity: "Lord", general: 1, skills: [], equips: [], judges: [], marks: [] },
        { player: 1, alive: false, hp: 0, max_hp: 4, hand_count: 0, turned_over: false, chained: false, identity: "Rebel", general: 2, skills: [], equips: [], judges: [], marks: [] },
      ],
      known_cards: [],
      outcome: "LordWin",
    },
  };
  s = ingestBattleEnvelopeIntoSurfaces(s, env);
  assert.equal(s.theme.rzshTierRoman, "IV");
  assert.equal(s.longxue.daoxin, 40);
  assert.deepEqual(s.longxue.unlockedThresholds, [25]);
  assert.ok(s.coin.lastSettlement);
  assert.equal(s.coin.balance, 109);
});

test("U02/L03 剑阁与 BOSS 体力归一化、L11 太虚幻境节点赐福、L21 武皇排阵替补", () => {
  // U02 / L03 上游体力解析：数字、"3" -> ∞、"6/8" 挑战模式有限体力 vs 非挑战 ∞
  assert.deepEqual(normalizeBossHp(5, true), { kind: "finite", hp: 5, maxHp: 5, label: "5/5" });
  assert.deepEqual(normalizeBossHp("3", true), { kind: "infinity", label: "∞" });
  assert.deepEqual(normalizeBossHp("6/8", true), { kind: "finite", hp: 6, maxHp: 8, label: "6/8" });
  assert.deepEqual(normalizeBossHp("6/8", false), { kind: "infinity", label: "∞" });

  // L11 太虚幻境（太虚幻境/game_buff.js 焱火、content_Pets.js 九尾）
  let taixu = createInitialExtensionSurfacesState().taixu;
  taixu = advanceTaixuNode(taixu);
  assert.equal(taixu.currentNodeIndex, 1);
  assert.equal(taixu.gold, 130);
  taixu = buyTaixuBlessing(taixu, "焱火", 40);
  assert.equal(taixu.gold, 90);
  assert.equal(taixu.blessings.includes("焱火"), true);
  taixu = bindTaixuPet(taixu, "九尾");
  assert.equal(taixu.boundPet, "九尾");

  // L21 武皇 5v5 先后手排阵与替补补摸 4 张
  const lineup0 = assignMartialLineup(["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"], 0);
  assert.deepEqual(lineup0.friends, ["0", "1", "2", "3", "4"]);
  const lineup1 = assignMartialLineup(["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"], 1);
  assert.deepEqual(lineup1.friends, ["0", "1", "2", "4", "5"]);
  const martial = applyMartialReplacement(createInitialExtensionSurfacesState().martial, ["sp_zhaoyun"]);
  assert.equal(martial.activeReplacement, "sp_zhaoyun");
  assert.equal(martial.replacementDraws, 4);
});

test("L07 全能搜索：苍螭之璧公开百科检索、暗牌隐私拦截与零 eval 安全门禁", () => {
  const s = createInitialExtensionSurfacesState();

  // 检索苍螭之璧（裁决 33 新增）
  const r1 = runOmnisearchQuery(s.omnisearch, "cangchizhibi", []);
  assert.equal(r1.lastError, null);
  assert.equal(r1.results.length, 1);
  assert.equal(r1.results[0].id, "cangchizhibi");

  // 他人手牌（暗牌实例号）拦截
  const view = {
    revision: 1,
    state_hash: "00".repeat(32),
    observer: 0,
    round: 1,
    current_player: 0,
    phase: "Play",
    seats: [],
    known_cards: [
      { id: 12, suit: "Spade", number: 1, kind: "Sha", zone: "Hand", owner: 0 },
      { id: 77, suit: "Heart", number: 2, kind: "Shan", zone: "Hand", owner: 1 },
    ],
    outcome: null,
  };
  const hiddenIds = hiddenCardIdsFromSnapshot(view, 0);
  assert.deepEqual(hiddenIds, [77]);
  const rHidden = runOmnisearchQuery(s.omnisearch, "#77", hiddenIds);
  assert.equal(rHidden.lastError, "error:hidden");
  assert.equal(rHidden.results.length, 0);

  // 动态执行/全局对象刺探拦截
  for (const dangerous of ["eval:alert(1)", "game.me.draw(99)", "window.localStorage"]) {
    const rEval = runOmnisearchQuery(s.omnisearch, dangerous, []);
    assert.equal(rEval.lastError, "error:eval_refused");
    assert.equal(rEval.results.length, 0);
  }
});

test("L02/L12/L17/L22/L23/L26/L29 辅助算法与安全门禁", () => {
  // L02 / L12 / L18 版本比较
  assert.equal(compareExtensionVersion("1.2", "1.10"), -1);
  assert.equal(compareExtensionVersion("1.10", "1.2"), 1);
  assert.equal(compareExtensionVersion("2.0.0", "2.0"), 0);

  // L12 罗马数字
  assert.equal(numToRoma(1), "I");
  assert.equal(numToRoma(9), "IX");

  // L17 斗转星移符号改写
  assert.equal(rewriteDouzhuanSkillText("点数不大于13且不小于1，阴阳转换"), "点数≤13且≥1，阳阴转换");

  // L26 笨蛋插件选牌约束
  assert.equal(foolZiyuanAllows(8, 5), true);
  assert.equal(foolZiyuanAllows(9, 5), false);
  assert.equal(foolFangtongAllows([], 10, true), false);
  assert.equal(foolFangtongAllows([12, 12], 12, false), true);
  assert.equal(foolFangtongAllows([20, 10], 8, false), false);

  // L23 电脑端指针分类与导出文件名白名单
  assert.equal(classifyDesktopPointer("mouse", 3, 4), "tap");
  assert.equal(classifyDesktopPointer("mouse", 8, 8), "drag");
  assert.equal(classifyDesktopPointer("touch", 100, 100), "touch");
  assert.equal(validateSafeExportFilename("match_seed42.npack.save"), "match_seed42.npack.save");
  assert.throws(() => validateSafeExportFilename("../passwd"), /unsafe export filename/);
  assert.throws(() => validateSafeExportFilename("dir/file.json"), /unsafe export filename/);

  // L22 天梯段位难度
  assert.deepEqual(resolveEloDifficulty(false, "rank", 0), { strength: 1, bias: 0, thinkDelaySec: 0 });
  assert.deepEqual(resolveEloDifficulty(true, "rank", 0, 0.5), { strength: 4, bias: 1, thinkDelaySec: 2.25 });

  // L29 龙血道心上限 99 与阈值解锁
  let lx = createInitialExtensionSurfacesState().longxue;
  lx = advanceLongxueDaoxin(lx, "damage_dealt", 6);
  assert.equal(lx.daoxin, 99);
  assert.deepEqual(lx.unlockedThresholds, [25, 50, 75, 99]);
  assert.equal(lx.qingzhengUnlocked, true);
});

test("裁决 34 完整状态快照序列化与读档恢复往返一致", () => {
  let original = createInitialExtensionSurfacesState();
  original = {
    ...original,
    coin: buyCoinShopItem(original.coin, "snow"),
    taixu: buyTaixuBlessing(advanceTaixuNode(original.taixu), "焱火", 40),
    longxue: advanceLongxueDaoxin(original.longxue, "damage_dealt", 4),
  };

  const json = serializeExtensionSurfacesSnapshot(original);
  const restored = deserializeExtensionSurfacesSnapshot(json);
  assert.deepEqual(restored, original);
  assert.equal(restored.coin.snowFall, true);
  assert.equal(restored.coin.balance, 80);
  assert.equal(restored.taixu.blessings.includes("焱火"), true);
  assert.deepEqual(restored.longxue.unlockedThresholds, [25, 50, 75]);

  assert.throws(() => deserializeExtensionSurfacesSnapshot("{}"), /invalid extension surfaces snapshot/);
  assert.throws(
    () => deserializeExtensionSurfacesSnapshot('{"version":1,"coin":{},"taixu":{},"mvp":{}}'),
    /invalid extension surfaces snapshot/,
  );
  assert.throws(
    () => deserializeExtensionSurfacesSnapshot("not-valid-json"),
    /invalid extension surfaces snapshot/,
  );

  // restoreExtensionSurfacesForLoad：有快照时精确恢复（不因读档信封二次累加），无快照历史槽时先重置再注入信封
  const loadEnv = {
    revision: 5,
    hash_hex: "bb".repeat(32),
    finished: false,
    result: null,
    choice: null,
    legal_actions: [],
    events: [{ PublicEvent: { code: 4 } }],
    view: {
      revision: 5,
      state_hash: "bb".repeat(32),
      observer: 0,
      round: 3,
      current_player: 0,
      phase: "Play",
      seats: [
        { player: 0, alive: true, hp: 4, max_hp: 4, hand_count: 4, turned_over: false, chained: false, identity: "Lord", general: 1, skills: [], equips: [], judges: [], marks: [] },
      ],
      known_cards: [],
      outcome: null,
    },
  };
  const fromSaved = restoreExtensionSurfacesForLoad(json, loadEnv);
  assert.deepEqual(fromSaved, original);
  assert.equal(fromSaved.longxue.daoxin, 80); // 保持存档时的 80，不被 loadEnv 的伤害事件二次 +15

  const fromLegacySlot = restoreExtensionSurfacesForLoad(undefined, loadEnv);
  assert.equal(fromLegacySlot.coin.balance, 100); // 重置回初始 100，不残留上一局消费后的 80
  assert.equal(fromLegacySlot.coin.snowFall, false);
  assert.equal(fromLegacySlot.longxue.daoxin, 35); // 初始 20 + loadEnv 伤害事件 15
  assert.equal(fromLegacySlot.theme.rzshTierRoman, "III");
});

test("Batch 25 补齐：L20 14 项战局标记、L29 潜龙四技解锁、U08 塔防波次、金币称号与百科检索", () => {
  assert.equal(L20_HUD_MARK_SPECS.length, 14);
  let theme = createInitialExtensionSurfacesState().theme;
  for (const spec of L20_HUD_MARK_SPECS) {
    theme = incrementL20HudMark(theme, spec.key, 2);
    assert.equal(theme.l20Marks?.[spec.key], 2);
  }
  assert.throws(() => incrementL20HudMark(theme, "nonexistent_mark", 1), /unknown L20 HUD mark/);

  assert.deepEqual(unlockLongxueSkills(20), []);
  assert.deepEqual(unlockLongxueSkills(25), ["mbcmqingzheng"]);
  assert.deepEqual(unlockLongxueSkills(50), ["mbcmqingzheng", "mbcmjiushi"]);
  assert.deepEqual(unlockLongxueSkills(75), ["mbcmqingzheng", "mbcmjiushi", "mbcmfangzhu"]);
  assert.deepEqual(unlockLongxueSkills(99), ["mbcmqingzheng", "mbcmjiushi", "mbcmfangzhu", "mbjuejin"]);

  assert.equal(resolveCoinHonorTitle(50), "初入茅庐");
  assert.equal(resolveCoinHonorTitle(120), "挥金如土");
  assert.equal(resolveCoinHonorTitle(250), "富甲天下");

  const omni = createInitialExtensionSurfacesState().omnisearch;
  for (const q of ["diaobingqianjiang", "mbcmjiushi", "mbcmfangzhu", "mbjuejin", "shichangshi", "th_jingxie", "yxs_menshen2"]) {
    const res = runOmnisearchQuery(omni, q, []);
    assert.equal(res.lastError, null);
    assert.equal(res.results.length, 1);
    assert.equal(res.results[0].id, q);
  }
});

test("Batch 26 全 37 来源收口：U01 31将3D陈列馆、L08/L24 Spine骨骼动作与皮肤偏移、L14手杀HUD、L25离线DIY门禁、L27双将同选", () => {
  assert.equal(U01_DDD_GALLERY_GENERALS.length, 31);
  assert.equal(SPINE_CLIP_ORDER.length, 5);

  let theme = createInitialExtensionSurfacesState().theme;
  theme = rotateU01GalleryStandee(theme);
  assert.equal(theme.u01SelectedGeneral, "ddd_xinxianying");
  assert.equal(theme.u01YawDeg, 30);
  assert.equal(theme.u01ZoomPct, 115);
  assert.throws(() => rotateU01GalleryStandee(theme, "not_a_ddd_general"), /unknown U01 3D general/);

  theme = triggerSpineSkinClip(theme, "gongji");
  assert.equal(theme.spineActiveClip, "gongji");
  assert.equal(theme.skinStandX, 12);
  assert.equal(theme.skinStandY, -8);
  assert.equal(theme.skinScalePct, 108);

  theme = triggerShoushaHudBanner(theme, "手杀播报·天下无双");
  assert.equal(theme.shoushaKillBanner, "手杀播报·天下无双");
  assert.equal(theme.shoushaCountdownPct, 70);

  let wb = createInitialExtensionSurfacesState().workbench;
  wb = applyZuanDiyDraft(wb, { generalId: "ssr_sunquan", maxHp: 5, onlineRoom: false });
  assert.equal(wb.zuanDiyGeneralId, "ssr_sunquan");
  assert.equal(wb.zuanDiyMaxHp, 5);
  assert.throws(
    () => applyZuanDiyDraft(wb, { generalId: "ssr_sunquan", maxHp: 5, onlineRoom: true }),
    /ONLINE_MULTIPLAYER_DIY_FORBIDDEN/,
  );

  wb = selectL27DualGenerals(wb, "ddd_caomao", "ddd_liuchong");
  assert.equal(wb.l27DualPrimary, "ddd_caomao");
  assert.equal(wb.l27DualSecondary, "ddd_liuchong");
  assert.equal(wb.l27DualLocked, true);
  assert.throws(() => selectL27DualGenerals(wb, "caocao", "caocao"), /DUAL_PICK_REQUIRES_TWO_DISTINCT_GENERALS/);
});

test("U04/L05 Canvas 粒子渲染器与 L07 全能搜索分页/关键词高亮 (coin/extension.js:24-398, SimplePagination.js:1-147)", async () => {
  const {
    highlightOmnisearchTerm,
    paginateOmnisearchResults,
    renderCoinCanvasParticles,
  } = await import("../src/engine/extensionSurfaceRuntime.ts");

  const calls = [];
  const fakeCtx = {
    canvas: { width: 240, height: 48 },
    fillStyle: "",
    strokeStyle: "",
    clearRect: (...a) => calls.push(["clearRect", ...a]),
    beginPath: () => calls.push(["beginPath"]),
    arc: (...a) => calls.push(["arc", ...a]),
    fill: () => calls.push(["fill"]),
    moveTo: (...a) => calls.push(["moveTo", ...a]),
    lineTo: (...a) => calls.push(["lineTo", ...a]),
    stroke: () => calls.push(["stroke"]),
    fillRect: (...a) => calls.push(["fillRect", ...a]),
  };
  const coinState = {
    ...createInitialExtensionSurfacesState().coin,
    snowFall: true,
    activeEffects: ["fireworks", "nebula", "twinkle"],
  };
  const rendered = renderCoinCanvasParticles(fakeCtx, coinState, 2);
  assert.equal(rendered.renderedParticles, 12 + 8 + 1 + 6);
  assert.deepEqual(rendered.activeModes.sort(), ["fireworks", "nebula", "snow", "twinkle"]);
  assert.ok(calls.length > 20);

  const entries = createInitialExtensionSurfacesState().omnisearch.results;
  const page1 = paginateOmnisearchResults(entries, 1, 2);
  assert.equal(page1.items.length, 2);
  assert.equal(page1.page, 1);
  assert.equal(page1.totalPages, 3);

  const hl = highlightOmnisearchTerm("苍螭之璧", "螭之");
  assert.deepEqual(hl, [
    { text: "苍", matched: false },
    { text: "螭之", matched: true },
    { text: "璧", matched: false },
  ]);
});

test("U04/L05 _buy 拖拽购买、L10 返利/十常侍阵亡语音、L12 如真似幻光效、L13 _qy_chongzhu、L20/L26/L27 样式与 L29 _CMeffects", async () => {
  const {
    applyFoolCardHoverTheme,
    applyL20MarkCssTheme,
    applyL27SelectBeautifyTheme,
    applyRzshDynamicSkinLighting,
    dragBuyCoinShopItem,
    incrementMvpRecastCount,
    triggerL10EmojiRebate,
    triggerL10ScsDieAudio,
    triggerLongxueHolySwordEffect,
  } = await import("../src/engine/extensionSurfaceRuntime.ts");

  const s = createInitialExtensionSurfacesState();
  // U04/L05: coin/extension.js:68-140 (_buy)
  const coinAfterBuy = dragBuyCoinShopItem(s.coin, "fireworks", 20);
  assert.equal(coinAfterBuy.balance, 80);
  assert.ok(coinAfterBuy.activeEffects.includes("fireworks"));
  assert.throws(
    () => dragBuyCoinShopItem({ ...s.coin, balance: 5 }, "snow", 20),
    /INSUFFICIENT_COINS_FOR_BUY/,
  );

  // L10: 史诗卡牌/extension.js:34553 (_emojiRebate1) & 34517 (scs_die_audio)
  const rebate = triggerL10EmojiRebate(s.theme, 2, "wine");
  assert.equal(rebate.rebatedEmotion, "rebate:wine->seat2");
  assert.equal(rebate.theme.throwEmotionCue, "rebate:wine->seat2");
  const afterDieAudio = triggerL10ScsDieAudio(rebate.theme, "scs_zhangrang");
  assert.equal(afterDieAudio.lastVoiceCue, "die/scs_zhangrang.mp3");

  // L12: 如真似幻/extension.js:510-2100
  const rzsh = applyRzshDynamicSkinLighting(s.theme, 4);
  assert.equal(rzsh.badgeRoman, "IV");
  assert.ok(rzsh.cssFilter.includes("drop-shadow"));

  // L13: 手杀MVP/extension.js:244 (_qy_chongzhu)
  const mvpAfterRecast = incrementMvpRecastCount(s.mvp, 0, 2);
  assert.equal(mvpAfterRecast.scores[0].assist, 6);
  assert.equal(mvpAfterRecast.scores[0].total, 106);

  // L20: 标记补充/extension.js:40-90
  const l20Css = applyL20MarkCssTheme(s.theme, "shousha");
  assert.equal(l20Css.theme.markStyle, "shousha");
  assert.equal(l20Css.stylesheet, "extension/标记补充/shousha.css");

  // L26: 笨蛋插件/extension.js:18-62
  const l26Hover = applyFoolCardHoverTheme(s.workbench, "fangtong");
  assert.equal(l26Hover.hoverBadge, "方统=36");
  assert.equal(l26Hover.cssClass, "fool-hint-fangtong");

  // L27: 选将美化/extension.js:42-95
  const l27Css = applyL27SelectBeautifyTheme(s.theme, false);
  assert.equal(l27Css.theme.selectBeautifyEnabled, false);
  assert.equal(l27Css.cssClass, "l27-select-beautify-classic");

  // L29: 龙血玄黄/extension.js:68-140 (_CMeffects)
  const l29Fx = triggerLongxueHolySwordEffect(s.longxue, true);
  assert.equal(l29Fx.longxue.shengjian, true);
  assert.equal(l29Fx.effectCue, "longxue:shengjian:on");
});





