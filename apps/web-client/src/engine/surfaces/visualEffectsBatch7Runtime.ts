/**
 * Batch UI-7 视效、弹窗与桌牌交互运行时：
 * - L05 / U04 coin (`haveFun.{firework,snow,star,blink}` + `coin_arenaReady`)
 * - L07 全能搜索 (4 cfg + 9 surf: `SimplePagination`, 6 代码高亮主题, `extension.css`, `extension_old.css`)
 * - L11 太虚幻境 (4 CSS surfaces)
 * - L12 如真似幻 (6 surfaces，遵循裁决 35/38 净室回退，不读任何 Spine 运行库)
 * - L13 手杀MVP (7 cfg + 2 CSS + `_qy_chongzhu`)
 * - L23 电脑适配 (`_fixCAILANZI` 桌面端 <10px 点击砸蛋弹层)
 * - L26 笨蛋插件 (5 cfg + `extension.css` + `_Fool_chooseCard`)
 */
import type { CardView, SeatView } from "../../types/engine";
import {
  classifyDesktopPointer,
  incrementMvpRecastCount,
  numToRoma,
  type CoinSurfaceState,
  type MvpSurfaceState,
} from "./sharedSurfaceHelpers";
import type { ConfigSpecEntry } from "./toolsAndPoolSchema";

function rejectUnknown(obj: unknown, allowed: ReadonlySet<string>, prefix: string): Record<string, unknown> {
  if (!obj || typeof obj !== "object" || Array.isArray(obj)) throw new Error(`${prefix}_INVALID_JSON`);
  const rec = obj as Record<string, unknown>;
  for (const k of Object.keys(rec)) {
    if (!allowed.has(k)) throw new Error(`${prefix}_UNKNOWN_KEY:${k}`);
  }
  return rec;
}
const pickOpt = <T extends string>(v: unknown, opts: ReadonlyArray<T>, fb: T): T =>
  typeof v === "string" && (opts as ReadonlyArray<string>).includes(v) ? (v as T) : fb;
const pickBool = (v: unknown, fb: boolean): boolean => (typeof v === "boolean" ? v : fb);

// ---------------------------------------------------------------------------
// L05 & U04 coin (haveFun.{firework,snow,star,blink} + coin_arenaReady)
// ---------------------------------------------------------------------------
export type CoinHaveFunEffect = "firework" | "snow" | "star" | "blink";
export const COIN_HAVE_FUN_SURFACE_IDS: ReadonlyArray<`haveFun.${CoinHaveFunEffect}`> = [
  "haveFun.firework",
  "haveFun.snow",
  "haveFun.star",
  "haveFun.blink",
];
export const U04_HOOK_IDS = ["coin_arenaReady"] as const;

const HAVE_FUN_TO_COIN_KEY: Record<CoinHaveFunEffect, string> = {
  firework: "fireworks",
  snow: "snow",
  star: "nebula",
  blink: "twinkle",
};

export function dispatchHaveFunEffect(
  coin: CoinSurfaceState,
  effect: CoinHaveFunEffect,
  action: "toggle" | "start" | "stop" | "toggleSnowSize",
): CoinSurfaceState {
  if (action === "toggleSnowSize") {
    return { ...coin, snowSize: coin.snowSize === "large" ? "small" : "large" };
  }
  const key = HAVE_FUN_TO_COIN_KEY[effect];
  const has = coin.activeEffects.includes(key);
  const wantOn = action === "toggle" ? !has : action === "start";
  const nextEffects = wantOn
    ? (has ? coin.activeEffects : [...coin.activeEffects, key])
    : coin.activeEffects.filter((x) => x !== key);
  return {
    ...coin,
    snowFall: effect === "snow" ? wantOn : coin.snowFall,
    activeEffects: nextEffects,
  };
}

export function dispatchCoinArenaReady(
  coin: CoinSurfaceState,
  opts: {
    displayMode: "icon" | "text";
    canvasTop: boolean;
    video?: boolean;
    connectMode?: boolean;
    snowFall?: boolean;
  },
): {
  coin: CoinSurfaceState;
  hudText: string;
  canvasTopClass: boolean;
  mounted: boolean;
} {
  if (opts.video || opts.connectMode) {
    return { coin, hudText: "", canvasTopClass: false, mounted: false };
  }
  const nextCoin = opts.snowFall
    ? dispatchHaveFunEffect({ ...coin, snowFall: true }, "snow", "start")
    : coin;
  const hudText = opts.displayMode === "text" ? `${nextCoin.balance}金` : `㉤ ${nextCoin.balance}`;
  return {
    coin: nextCoin,
    hudText,
    canvasTopClass: opts.canvasTop,
    mounted: true,
  };
}

// ---------------------------------------------------------------------------
// L07 全能搜索 (4 cfg + 9 surf)
// ---------------------------------------------------------------------------
export type L07HighlightCssId =
  | "a11y-dark.min.css"
  | "an-old-hope.min.css"
  | "atom-one-dark.min.css"
  | "felipec.min.css"
  | "gml.min.css"
  | "vs2015.min.css";

export interface L07OmnisearchConfig {
  version: "3.2";
  css: L07HighlightCssId;
  background: "none" | "相爱相杀" | "才颜双绝" | "新杀春日" | "青铜龙纹" | "随机";
  loadUpdateContent: boolean;
}

export const L07_HIGHLIGHT_CSS_IDS: ReadonlyArray<L07HighlightCssId> = [
  "a11y-dark.min.css",
  "an-old-hope.min.css",
  "atom-one-dark.min.css",
  "felipec.min.css",
  "gml.min.css",
  "vs2015.min.css",
];

export const L07_SURFACE_IDS = [
  "SimplePagination",
  ...L07_HIGHLIGHT_CSS_IDS,
  "extension.css",
  "extension_old.css",
] as const;

export const L07_CONFIG_SPECS: ReadonlyArray<ConfigSpecEntry<keyof L07OmnisearchConfig>> = [
  { key: "version", label: "全能搜索扩展版本 (v3.2)", control: "info", upstreamLine: 2265, ruleCoupled: false },
  {
    key: "css",
    label: "代码高亮样式",
    control: "select",
    upstreamLine: 2270,
    ruleCoupled: false,
    options: L07_HIGHLIGHT_CSS_IDS.map((id) => ({ value: id, label: id.replace(".min.css", "") })),
  },
  {
    key: "background",
    label: "搜索界面背景主题",
    control: "select",
    upstreamLine: 2287,
    ruleCoupled: false,
    options: [
      { value: "none", label: "本体默认背景" },
      { value: "相爱相杀", label: "相爱相杀" },
      { value: "才颜双绝", label: "才颜双绝" },
      { value: "新杀春日", label: "新杀春日" },
      { value: "青铜龙纹", label: "青铜龙纹" },
      { value: "随机", label: "随机" },
    ],
  },
  { key: "loadUpdateContent", label: "查看全能搜索历史更新内容", control: "action", upstreamLine: 2304, ruleCoupled: false },
];
const L07_KEY_SET: ReadonlySet<string> = new Set(L07_CONFIG_SPECS.map((s) => s.key));

export interface L07OmnisearchUiState {
  config: L07OmnisearchConfig;
  searcherStyle: "extension.css" | "extension_old.css";
  updateLogOpen: boolean;
}

export function createDefaultL07Config(): L07OmnisearchConfig {
  return { version: "3.2", css: "vs2015.min.css", background: "none", loadUpdateContent: false };
}

export function importL07ConfigJson(jsonText: string): L07OmnisearchConfig {
  const rec = rejectUnknown(JSON.parse(jsonText), L07_KEY_SET, "L07_CONFIG");
  const d = createDefaultL07Config();
  return {
    version: "3.2",
    css: pickOpt(rec.css, L07_HIGHLIGHT_CSS_IDS, d.css),
    background: pickOpt(rec.background, ["none", "相爱相杀", "才颜双绝", "新杀春日", "青铜龙纹", "随机"], d.background),
    loadUpdateContent: pickBool(rec.loadUpdateContent, d.loadUpdateContent),
  };
}

export function createInitialL07UiState(): L07OmnisearchUiState {
  return { config: createDefaultL07Config(), searcherStyle: "extension.css", updateLogOpen: false };
}

export interface SimplePaginationView {
  pageNumber: number;
  totalPageCount: number;
  canPrev: boolean;
  canNext: boolean;
  ellipsisHead: boolean;
  ellipsisTail: boolean;
  pages: number[];
}

/** 对照 `全能搜索/SimplePagination.js:10-200` 纯状态分页控制器 */
export function buildSimplePaginationView(
  totalPages: number,
  currentPage: number,
  maxShowBtnCount = 5,
): SimplePaginationView {
  const total = Math.max(1, Math.floor(totalPages));
  const page = Math.max(1, Math.min(total, Math.floor(currentPage)));
  const maxShow = Math.max(1, Math.floor(maxShowBtnCount));
  if (total <= maxShow + 2) {
    return {
      pageNumber: page,
      totalPageCount: total,
      canPrev: page > 1,
      canNext: page < total,
      ellipsisHead: false,
      ellipsisTail: false,
      pages: Array.from({ length: total }, (_, i) => i + 1),
    };
  }
  const activePos = Math.max(1, Math.min(Math.ceil(maxShow / 2), maxShow));
  const rEllipseSign = total - (maxShow - activePos) - 1;
  const pages: number[] = [1];
  let ellipsisHead = false;
  let ellipsisTail = false;
  if (page <= maxShow && page < rEllipseSign) {
    ellipsisTail = true;
    for (let i = 2; i <= maxShow + 1; i++) pages.push(i);
  } else if (page >= rEllipseSign) {
    ellipsisHead = true;
    const start = total - maxShow;
    for (let i = start; i < total; i++) pages.push(i);
  } else {
    ellipsisHead = !(page === 2 && maxShow === 1);
    ellipsisTail = true;
    for (let i = 1; i <= maxShow; i++) {
      pages.push(page + (i - activePos));
    }
  }
  pages.push(total);
  return {
    pageNumber: page,
    totalPageCount: total,
    canPrev: page > 1,
    canNext: page < total,
    ellipsisHead,
    ellipsisTail,
    pages,
  };
}

// ---------------------------------------------------------------------------
// L11 太虚幻境 (4 CSS surfaces) & L12 如真似幻 (6 surfaces)
// ---------------------------------------------------------------------------
export const L11_SURFACE_IDS = [
  "extension_MobileStyle.css",
  "extension_PcStyle.css",
  "extension_servant.css",
  "extension_style.css",
] as const;

export const L12_SURFACE_IDS = [
  "dynamicSkin",
  "rzsh",
  "font.css",
  "offline.css",
  "offline_old.css",
  "rzsh.css",
] as const;
export type L12SurfaceId = (typeof L12_SURFACE_IDS)[number];

export interface L12RzshSurfaceState {
  activeSurface: L12SurfaceId;
  backgroundX: number;
  backgroundY: number;
  backgroundScale: number;
  standeeScale: number;
  tierRoman: string;
  assetFallbackStatus: "static-portrait-fallback";
}

export function createInitialL12RzshState(): L12RzshSurfaceState {
  return {
    activeSurface: "rzsh.css",
    backgroundX: 0.5,
    backgroundY: 0.3,
    backgroundScale: 1,
    standeeScale: 0.85,
    tierRoman: numToRoma(2),
    assetFallbackStatus: "static-portrait-fallback",
  };
}

// ---------------------------------------------------------------------------
// L13 手杀MVP (7 cfg + 2 CSS + _qy_chongzhu)
// ---------------------------------------------------------------------------
export interface L13MvpConfig {
  qingyao_shoushaMVP: boolean;
  yangshi: "on" | "off";
  mvplutou: "shizhounian" | "shousha" | "off";
  qingyao_shoushapeiyin: boolean;
  shoushajiesuan: boolean;
  jiesuanfix: boolean;
  oneMore: boolean;
}
export const L13_SURFACE_IDS = ["extension.css", "extension_new.css"] as const;
export const L13_HOOK_IDS = ["_qy_chongzhu"] as const;

export const L13_CONFIG_SPECS: ReadonlyArray<ConfigSpecEntry<keyof L13MvpConfig>> = [
  { key: "qingyao_shoushaMVP", label: "手杀 MVP 结算总开关", control: "boolean", upstreamLine: 2710, ruleCoupled: false },
  { key: "yangshi", label: "MVP 样式选择（样式1/样式2）", control: "select", upstreamLine: 2715, ruleCoupled: false, options: [{ value: "on", label: "样式1 (新版倾斜框)" }, { value: "off", label: "样式2 (经典红旗榜)" }] },
  { key: "mvplutou", label: "无原画时 MVP 露头模式", control: "select", upstreamLine: 2723, ruleCoupled: false, options: [{ value: "shousha", label: "手杀露头" }, { value: "shizhounian", label: "十周年露头" }, { value: "off", label: "无露头" }] },
  { key: "qingyao_shoushapeiyin", label: "手杀配音替换开关", control: "boolean", upstreamLine: 2738, ruleCoupled: false },
  { key: "shoushajiesuan", label: "手杀结算面板", control: "boolean", upstreamLine: 2744, ruleCoupled: false },
  { key: "jiesuanfix", label: "结算优化（罕见胜负）", control: "boolean", upstreamLine: 2749, ruleCoupled: false },
  { key: "oneMore", label: "显示「再来一局」按钮", control: "boolean", upstreamLine: 2754, ruleCoupled: false },
];
const L13_KEY_SET: ReadonlySet<string> = new Set(L13_CONFIG_SPECS.map((s) => s.key));

export interface L13MvpUiState {
  config: L13MvpConfig;
  lastRecastCue: string | null;
  oneMoreClicks: number;
}

export function createDefaultL13Config(): L13MvpConfig {
  return {
    qingyao_shoushaMVP: true,
    yangshi: "on",
    mvplutou: "shousha",
    qingyao_shoushapeiyin: true,
    shoushajiesuan: true,
    jiesuanfix: true,
    oneMore: true,
  };
}

export function importL13ConfigJson(jsonText: string): L13MvpConfig {
  const rec = rejectUnknown(JSON.parse(jsonText), L13_KEY_SET, "L13_CONFIG");
  const d = createDefaultL13Config();
  return {
    qingyao_shoushaMVP: pickBool(rec.qingyao_shoushaMVP, d.qingyao_shoushaMVP),
    yangshi: pickOpt(rec.yangshi, ["on", "off"], d.yangshi),
    mvplutou: pickOpt(rec.mvplutou, ["shizhounian", "shousha", "off"], d.mvplutou),
    qingyao_shoushapeiyin: pickBool(rec.qingyao_shoushapeiyin, d.qingyao_shoushapeiyin),
    shoushajiesuan: pickBool(rec.shoushajiesuan, d.shoushajiesuan),
    jiesuanfix: pickBool(rec.jiesuanfix, d.jiesuanfix),
    oneMore: pickBool(rec.oneMore, d.oneMore),
  };
}

export function createInitialL13UiState(): L13MvpUiState {
  return { config: createDefaultL13Config(), lastRecastCue: null, oneMoreClicks: 0 };
}

export function triggerQyChongzhuHook(
  l13: L13MvpUiState,
  mvp: MvpSurfaceState,
  seat: number,
  sex: "male" | "female" = "male",
): { l13: L13MvpUiState; mvp: MvpSurfaceState } {
  const nextMvp = incrementMvpRecastCount(mvp, seat, 1);
  return {
    mvp: nextMvp,
    l13: {
      ...l13,
      lastRecastCue: l13.config.qingyao_shoushapeiyin ? `qy_chongzhu_${sex}@P${seat}` : `recast@P${seat}(muted)`,
    },
  };
}

// ---------------------------------------------------------------------------
// L23 电脑适配 (`_fixCAILANZI`)
// ---------------------------------------------------------------------------
export const L23_HOOK_IDS = ["_fixCAILANZI"] as const;

export interface L23PcAdaptState {
  enabled: boolean;
  lastTappedSeat: number | null;
  emojiPickerSeat: number | null;
  lastTapClassification: "tap" | "drag" | "touch" | "ignored";
}

export function createInitialL23State(): L23PcAdaptState {
  return {
    enabled: true,
    lastTappedSeat: null,
    emojiPickerSeat: null,
    lastTapClassification: "ignored",
  };
}

export function dispatchFixCaiLanZiTap(
  state: L23PcAdaptState,
  ev: { seat: number; dx: number; dy: number; isSelf: boolean; isDead: boolean; isOver: boolean; useThrowEmoji?: boolean },
): L23PcAdaptState {
  if (!state.enabled || ev.isSelf || ev.isDead || ev.isOver || ev.useThrowEmoji === false) {
    return { ...state, lastTapClassification: "ignored", emojiPickerSeat: null };
  }
  const cls = classifyDesktopPointer("mouse", ev.dx, ev.dy);
  if (cls !== "tap") {
    return { ...state, lastTapClassification: cls };
  }
  return {
    ...state,
    lastTappedSeat: ev.seat,
    emojiPickerSeat: ev.seat,
    lastTapClassification: "tap",
  };
}

// ---------------------------------------------------------------------------
// L26 笨蛋插件 (5 cfg + `extension.css` + `_Fool_chooseCard`)
// ---------------------------------------------------------------------------
export interface L26FoolConfig {
  Fool_throwEmotion: boolean;
  Fool_skill: boolean;
  Fool_chooseAudio: boolean;
  Fool_liegong: "shousha" | "olten" | "close";
  Fool_derivationSkill: boolean;
}
export const L26_SURFACE_IDS = ["extension.css"] as const;
export const L26_HOOK_IDS = ["_Fool_chooseCard"] as const;

export const L26_CONFIG_SPECS: ReadonlyArray<ConfigSpecEntry<keyof L26FoolConfig>> = [
  { key: "Fool_throwEmotion", label: "表情砸蛋随机旋转落点", control: "boolean", upstreamLine: 1568, ruleCoupled: false },
  { key: "Fool_skill", label: "技能计算优化（资援/敏思/方统）", control: "boolean", upstreamLine: 1573, ruleCoupled: true },
  { key: "Fool_chooseAudio", label: "选将语音播放开关", control: "boolean", upstreamLine: 1578, ruleCoupled: false },
  { key: "Fool_liegong", label: "选中卡牌提示样式（不可响应）", control: "select", upstreamLine: 1588, ruleCoupled: true, options: [{ value: "shousha", label: "手杀样式" }, { value: "olten", label: "十周年样式" }, { value: "close", label: "关闭" }] },
  { key: "Fool_derivationSkill", label: "衍生技能详细显示", control: "boolean", upstreamLine: 1638, ruleCoupled: false },
];
const L26_KEY_SET: ReadonlySet<string> = new Set(L26_CONFIG_SPECS.map((s) => s.key));

export interface L26FoolPluginState {
  config: L26FoolConfig;
  chooseCardEquipCopyEnabled: boolean;
  mirroredEquipCardIds: number[];
  derivationPreviewSkill: string | null;
}

export function createDefaultL26Config(): L26FoolConfig {
  return {
    Fool_throwEmotion: true,
    Fool_skill: true,
    Fool_chooseAudio: true,
    Fool_liegong: "shousha",
    Fool_derivationSkill: false,
  };
}

export function importL26ConfigJson(jsonText: string): L26FoolConfig {
  const rec = rejectUnknown(JSON.parse(jsonText), L26_KEY_SET, "L26_CONFIG");
  const d = createDefaultL26Config();
  return {
    Fool_throwEmotion: pickBool(rec.Fool_throwEmotion, d.Fool_throwEmotion),
    Fool_skill: pickBool(rec.Fool_skill, d.Fool_skill),
    Fool_chooseAudio: pickBool(rec.Fool_chooseAudio, d.Fool_chooseAudio),
    Fool_liegong: pickOpt(rec.Fool_liegong, ["shousha", "olten", "close"], d.Fool_liegong),
    Fool_derivationSkill: pickBool(rec.Fool_derivationSkill, d.Fool_derivationSkill),
  };
}

export function createInitialL26State(): L26FoolPluginState {
  return {
    config: createDefaultL26Config(),
    chooseCardEquipCopyEnabled: false,
    mirroredEquipCardIds: [],
    derivationPreviewSkill: null,
  };
}

/** 对照 `笨蛋插件/extension.js:1219-1255` `_Fool_chooseCard`（非弃牌阶段且允许选装备区时映射手牌辅助视图） */
export function computeFoolEquipMirrorCards(
  state: L26FoolPluginState,
  knownCards: ReadonlyArray<CardView>,
  selfId: number,
  opts: { positionAllowsEquip: boolean; inPhaseDiscard: boolean },
): L26FoolPluginState {
  if (!state.chooseCardEquipCopyEnabled || !opts.positionAllowsEquip || opts.inPhaseDiscard) {
    return { ...state, mirroredEquipCardIds: [] };
  }
  const eqIds = knownCards.filter((c) => c.owner === selfId && c.zone === "Equip").map((c) => c.id);
  return { ...state, mirroredEquipCardIds: eqIds };
}

export function isShaCard(card: Pick<CardView, "kind"> | null | undefined): boolean {
  if (!card) return false;
  return card.kind === "Sha" || card.kind === "ShaFire" || card.kind === "ShaThunder";
}

/** 对照 `笨蛋插件/extension.js:473-573` 选中【杀】且角色具备烈弓/武神/诈降/玲珑/伏骑等强中加伤条件时计算提示文本 */
export function evaluateFoolLiegongTag(opts: {
  mode: L26FoolConfig["Fool_liegong"];
  selectedCard: CardView | null;
  selfSeat: SeatView | null;
  targetSeat: SeatView | null;
  selfHandCount: number;
  selfEquipsCount: number;
  attackRange?: number;
  distance?: number;
}): { label: string; cssClass: "Fool_liegone_shousha" | "Fool_liegone_olten" } | null {
  if (opts.mode === "close" || !opts.selectedCard || !opts.selfSeat || !opts.targetSeat) return null;
  if (opts.selfSeat.player === opts.targetSeat.player || !opts.targetSeat.alive) return null;
  if (!isShaCard(opts.selectedCard)) return null;

  const activeSkills = new Set(
    opts.selfSeat.skills
      .filter((s) => s.enabled !== false && s.name)
      .map((s) => String(s.name).trim().toLowerCase()),
  );
  const range = Math.max(1, opts.attackRange ?? 1);
  const dist = Math.max(1, opts.distance ?? 1);
  const targetHand = opts.targetSeat.hand_count;
  const selfHand = Math.max(0, opts.selfHandCount - 1);
  const isHeart = opts.selectedCard.suit === "Heart";
  const isRed = opts.selectedCard.suit === "Heart" || opts.selectedCard.suit === "Diamond";
  const isBlack = opts.selectedCard.suit === "Spade" || opts.selectedCard.suit === "Club";

  let foolLiegong = false;
  let qiangming = false;
  let jiaShang = false;

  if (activeSkills.has("diy_liegong") && targetHand <= selfHand) {
    foolLiegong = true;
    qiangming = true;
  }
  if (
    (activeSkills.has("liegong") ||
      activeSkills.has("fz_liegong") ||
      activeSkills.has("sbliegong") ||
      activeSkills.has("烈弓") ||
      activeSkills.has("谋烈弓")) &&
    (targetHand >= opts.selfSeat.hp || targetHand <= range)
  ) {
    foolLiegong = true;
    qiangming = true;
  }
  if (activeSkills.has("xinliegong") || activeSkills.has("新烈弓")) {
    const canCond = targetHand <= selfHand || opts.targetSeat.hp >= opts.selfSeat.hp;
    if (canCond) {
      foolLiegong = true;
      if (targetHand <= selfHand) qiangming = true;
      if (opts.targetSeat.hp >= opts.selfSeat.hp) jiaShang = true;
    }
  }
  if ((activeSkills.has("wushen") || activeSkills.has("武神")) && isHeart) qiangming = true;
  if ((activeSkills.has("zhaxiang2") || activeSkills.has("sbzhaxiang") || activeSkills.has("诈降")) && isRed) {
    qiangming = true;
  }
  if ((activeSkills.has("relinglong") || activeSkills.has("玲珑")) && opts.selfEquipsCount === 0) qiangming = true;
  if ((activeSkills.has("fuqi") || activeSkills.has("refuqi") || activeSkills.has("伏骑")) && dist <= 1) {
    qiangming = true;
  }
  if (activeSkills.has("spshidi") && isBlack) qiangming = true;
  if (activeSkills.has("liyong2") || activeSkills.has("sbpaoxiao")) qiangming = true;
  if (activeSkills.has("dcgonghu") && isRed) qiangming = true;
  if (activeSkills.has("jintao") || activeSkills.has("金韬")) jiaShang = true;

  if (!foolLiegong && !qiangming && !jiaShang) return null;

  let label = "";
  if (foolLiegong) {
    if (qiangming && jiaShang) label = opts.mode === "shousha" ? "强中加伤" : "可烈弓 不可响应 伤害+1";
    else if (qiangming) label = opts.mode === "shousha" ? "烈弓强中" : "可烈弓 不可响应";
    else if (jiaShang) label = opts.mode === "shousha" ? "烈弓加伤" : "可烈弓 伤害+1";
    else label = "可烈弓";
  } else if (qiangming && jiaShang) {
    label = opts.mode === "shousha" ? "强中加伤" : "不可响应 伤害+1";
  } else if (qiangming) {
    label = "不可响应";
  } else if (jiaShang) {
    label = opts.mode === "shousha" ? "加伤" : "伤害+1";
  }

  return {
    label,
    cssClass: opts.mode === "shousha" ? "Fool_liegone_shousha" : "Fool_liegone_olten",
  };
}

export function computeFoolThrowRotation(seat: number, selfId: number, enabled: boolean): number {
  if (!enabled) return 0;
  const raw = ((Math.abs(seat) * 37 + Math.abs(selfId) * 19 + 11) % 35) - 17;
  return raw === 0 ? 9 : raw;
}
