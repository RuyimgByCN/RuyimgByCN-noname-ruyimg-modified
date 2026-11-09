/**
 * L14 手杀ui 原生前端渲染面与全部 53 项 UI 钩子运行时
 * 上游对照:
 * - `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:85-6450`
 * - `archive/upstream/noname-liuli/noname/extension/手杀ui/fonts/extension.js:2553-2590`
 * - `archive/upstream/noname-liuli/noname/extension/手杀ui/lbtn/main1.js:495-560`
 * - `character/main1*.js`, `lbtn/main*.js`, `skill/main*.js`, `menu.js` 及 32 份 CSS 样式表
 */

export const SHOUSHA_UI_HOOK_IDS = [
  "_dieNoTuoguan",
  "_menuCheat",
  "_jindutiaoO",
  "_jindutiaoA",
  "_jindutiaoMB",
  "_jindutiaoKS",
  "_jindutiao_close",
  "_chupaiA",
  "_chupaiB",
  "_chupaiC",
  "_chupaiD",
  "_chupaiE",
  "_chupaiF",
  "_chupaiG",
  "_chupaiH",
  "_chupaiM",
  "_chupaiN",
  "_chupaiO",
  "_chupaiP",
  "_chupaiI",
  "_chupaiJ",
  "_chupaiK",
  "_chupaiL",
  "_chupaiMX",
  "_chupaiNX",
  "_skilltip_closeB",
  "_jd_ddxyA",
  "_jd_ddxyB",
  "_jd_ddxyC",
  "_jd_ddxyD",
  "_jd_ddxyF",
  "_jd_ddxyAll",
  "_jd_shiyongshanD",
  "_jd_ddxyE",
  "_jd_wjsw",
  "_jd_swxs",
  "_jd_dfsk",
  "_jd_hhks",
  "_jd_pdjd",
  "_jd_mpjd",
  "_jd_cpjd",
  "_jd_qpjd",
  "_jd_hhjs",
  "_jd_phaseAfter",
  "_jindutiao",
  "_jindutiaopl",
  "_wmkzSayChange",
  "_shuohuaSayWordphase",
  "ZLLT",
  "_jindutiaouse",
  "_wuxie",
  "_uicardupdate",
  "_paixu_paixu_paixu",
] as const;

export type ShoushaUiHookId = (typeof SHOUSHA_UI_HOOK_IDS)[number];

export const SHOUSHA_SURFACE_MODULES = [
  "character/main1.js",
  "character/main1_new.js",
  "character/main2.js",
  "lbtn/main1.js",
  "lbtn/main2.js",
  "menu.js",
  "skill/main1.js",
  "skill/main2.js",
  "KGMH/kaiguan.css",
  "KGMH/kaiguan_new.css",
  "card/main.css",
  "card/main1.css",
  "card/main1_mobile.css",
  "card/main1_pc.css",
  "card/main2.css",
  "character/main1.css",
  "character/main1_new.css",
  "character/main1_old.css",
  "character/main2.css",
  "dialog/main1.css",
  "extension.css",
  "lbtn/main1.css",
  "lbtn/main1_boss.css",
  "lbtn/main1_window.css",
  "lbtn/main2.css",
  "lbtn/main2_window.css",
  "menu.css",
  "menu2.css",
  "menu3.css",
  "qidongye/layout_new.css",
  "qidongye/layout_old.css",
  "qidongye/layout_xbig.css",
  "skill/main1.css",
  "skill/main1_window.css",
  "skill/main2.css",
  "skill/main2_window.css",
  "style.css",
  "style2.css",
  "style_boss.css",
  "style_old.css",
] as const;

export interface ShoushaSortableCard {
  id: number;
  name: string;
  suit: "spade" | "heart" | "club" | "diamond" | "none";
  number: number;
  muniu?: boolean;
}

const SUIT_INDEX: Readonly<Record<string, number>> = {
  spade: 0,
  heart: 1,
  club: 2,
  diamond: 3,
  none: 4,
};

/**
 * 上游 `手杀ui/lbtn/main1.js:514-552` (`_paixu_paixu_paixu`):
 * 先分离木牛流马牌 (`muniu`) 与普通手牌，分别按 `name -> suit -> number` 排序后拼接 (`cardMuNiu.concat(cardNotMuNiu)`)。
 */
export function sortShoushaHandCards(cards: ReadonlyArray<ShoushaSortableCard>): ShoushaSortableCard[] {
  if (cards.length <= 1) return [...cards];
  const cmp = (a: ShoushaSortableCard, b: ShoushaSortableCard): number => {
    if (a.name !== b.name) return a.name.localeCompare(b.name);
    if (a.suit !== b.suit) return (SUIT_INDEX[a.suit] ?? 4) - (SUIT_INDEX[b.suit] ?? 4);
    return a.number - b.number;
  };
  const cardMuNiu = cards.filter((c) => Boolean(c.muniu)).sort(cmp);
  const cardNotMuNiu = cards.filter((c) => !c.muniu).sort(cmp);
  return [...cardMuNiu, ...cardNotMuNiu];
}

export interface ShoushaUiRuntimeState {
  styleVariant: "shousha" | "decade" | "boss" | "old";
  tuoguanActive: boolean;
  progressVisible: boolean;
  progressPct: number;
  progressTargetSeat: number | null;
  phaseRibbon: "idle" | "hhks" | "pdjd" | "mpjd" | "cpjd" | "qpjd" | "hhjs";
  phaseRibbonLabel: string;
  playPromptText: string | null;
  skillTipOpen: boolean;
  waitingSeats: Record<number, string>;
  wuxiePromptActive: boolean;
  skillControlRev: number;
  zlltOutcropEnabled: boolean;
  chatBubbleBySeat: Record<number, string>;
  sortedHandCardIds: number[];
  cheatConsoleOpen: boolean;
}

export function createInitialShoushaUiRuntimeState(): ShoushaUiRuntimeState {
  return {
    styleVariant: "shousha",
    tuoguanActive: false,
    progressVisible: true,
    progressPct: 85,
    progressTargetSeat: null,
    phaseRibbon: "cpjd",
    phaseRibbonLabel: "出牌阶段",
    playPromptText: "出牌阶段，请选择一张牌",
    skillTipOpen: false,
    waitingSeats: {},
    wuxiePromptActive: false,
    skillControlRev: 1,
    zlltOutcropEnabled: false,
    chatBubbleBySeat: {},
    sortedHandCardIds: [],
    cheatConsoleOpen: false,
  };
}

const PHASE_LABELS: Readonly<Record<ShoushaUiRuntimeState["phaseRibbon"], string>> = {
  idle: "待机",
  hhks: "回合开始",
  pdjd: "判定阶段",
  mpjd: "摸牌阶段",
  cpjd: "出牌阶段",
  qpjd: "弃牌阶段",
  hhjs: "回合结束",
};

export function dispatchShoushaUiHook(
  state: ShoushaUiRuntimeState,
  hook: ShoushaUiHookId,
  opts?: {
    seat?: number;
    onlineRoom?: boolean;
    handCards?: ReadonlyArray<ShoushaSortableCard>;
    sayText?: string;
  },
): ShoushaUiRuntimeState {
  const seat = opts?.seat ?? 0;

  if (hook === "_dieNoTuoguan") {
    // extension.js:85: dieAfter -> game.unbindControl / cancel auto-trust
    return { ...state, tuoguanActive: false, progressVisible: false };
  }
  if (hook === "_menuCheat") {
    // extension.js:103: blocked in online connect rooms
    if (opts?.onlineRoom) return { ...state, cheatConsoleOpen: false };
    return { ...state, cheatConsoleOpen: !state.cheatConsoleOpen };
  }
  if (
    hook === "_jindutiaoO" ||
    hook === "_jindutiaoA" ||
    hook === "_jindutiaoKS" ||
    hook === "_jindutiao" ||
    hook === "_jindutiaopl" ||
    hook === "_jindutiaouse"
  ) {
    return {
      ...state,
      progressVisible: true,
      progressPct: hook === "_jindutiaouse" ? 100 : 90,
    };
  }
  if (hook === "_jindutiaoMB") {
    return { ...state, progressVisible: true, progressTargetSeat: seat };
  }
  if (hook === "_jindutiao_close") {
    return { ...state, progressVisible: false, progressPct: 0, progressTargetSeat: null };
  }
  if (
    hook === "_chupaiA" ||
    hook === "_chupaiC" ||
    hook === "_chupaiE" ||
    hook === "_chupaiG" ||
    hook === "_chupaiI" ||
    hook === "_chupaiK" ||
    hook === "_chupaiM" ||
    hook === "_chupaiO" ||
    hook === "_chupaiMX"
  ) {
    const prompt =
      hook === "_chupaiA"
        ? "出牌阶段，请选择一张牌"
        : hook === "_chupaiC"
          ? "弃牌阶段，请弃置手牌"
          : hook === "_chupaiK"
            ? "阶段结算中..."
            : "请使用或打出一张牌响应";
    return { ...state, playPromptText: prompt, skillTipOpen: true };
  }
  if (
    hook === "_chupaiB" ||
    hook === "_chupaiD" ||
    hook === "_chupaiF" ||
    hook === "_chupaiH" ||
    hook === "_chupaiJ" ||
    hook === "_chupaiL" ||
    hook === "_chupaiN" ||
    hook === "_chupaiP" ||
    hook === "_chupaiNX" ||
    hook === "_skilltip_closeB"
  ) {
    return { ...state, playPromptText: null, skillTipOpen: false };
  }
  if (
    hook === "_jd_ddxyA" ||
    hook === "_jd_ddxyB" ||
    hook === "_jd_ddxyC" ||
    hook === "_jd_ddxyD" ||
    hook === "_jd_ddxyF" ||
    hook === "_jd_ddxyAll" ||
    hook === "_jd_dfsk"
  ) {
    return {
      ...state,
      waitingSeats: { ...state.waitingSeats, [seat]: hook === "_jd_dfsk" ? "对方思考中..." : "等待响应..." },
    };
  }
  if (hook === "_jd_shiyongshanD") {
    return {
      ...state,
      waitingSeats: { ...state.waitingSeats, [seat]: "打出【闪】" },
    };
  }
  if (hook === "_jd_ddxyE" || hook === "_jd_wjsw" || hook === "_jd_swxs") {
    const nextWaiting = { ...state.waitingSeats };
    delete nextWaiting[seat];
    return { ...state, waitingSeats: nextWaiting };
  }
  if (
    hook === "_jd_hhks" ||
    hook === "_jd_pdjd" ||
    hook === "_jd_mpjd" ||
    hook === "_jd_cpjd" ||
    hook === "_jd_qpjd" ||
    hook === "_jd_hhjs"
  ) {
    const code = hook.slice(4) as ShoushaUiRuntimeState["phaseRibbon"];
    return {
      ...state,
      phaseRibbon: code,
      phaseRibbonLabel: PHASE_LABELS[code],
    };
  }
  if (hook === "_jd_phaseAfter") {
    return { ...state, phaseRibbon: "idle", phaseRibbonLabel: PHASE_LABELS.idle };
  }
  if (hook === "_wmkzSayChange" || hook === "_shuohuaSayWordphase") {
    return {
      ...state,
      chatBubbleBySeat: {
        ...state.chatBubbleBySeat,
        [seat]: opts?.sayText ?? "快点吧，我等得花儿都谢了！",
      },
    };
  }
  if (hook === "ZLLT") {
    return { ...state, zlltOutcropEnabled: !state.zlltOutcropEnabled };
  }
  if (hook === "_wuxie") {
    return { ...state, wuxiePromptActive: true, playPromptText: "是否使用【无懈可击】响应？" };
  }
  if (hook === "_uicardupdate") {
    // lbtn/main1.js:495: phaseJieshuBegin -> ui.updateSkillControl(game.me, true)
    return { ...state, skillControlRev: state.skillControlRev + 1, wuxiePromptActive: false };
  }
  // `_paixu_paixu_paixu`: lbtn/main1.js:514
  const sorted = sortShoushaHandCards(opts?.handCards ?? []);
  return {
    ...state,
    sortedHandCardIds: sorted.map((c) => c.id),
  };
}
