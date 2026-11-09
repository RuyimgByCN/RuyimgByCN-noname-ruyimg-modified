import type { EngineEnvelope, ObserverSnapshot } from "../types/engine";

export type BossHpNormalized =
  | { kind: "finite"; hp: number; maxHp: number; label: string }
  | { kind: "infinity"; label: "∞" };

export const L20_HUD_MARK_SPECS: ReadonlyArray<{ key: string; label: string }> = [
  { key: "longmark", label: "龙印" },
  { key: "biaoji", label: "标记" },
  { key: "danlie", label: "胆裂" },
  { key: "fengyin", label: "封印" },
  { key: "fuhun", label: "父魂" },
  { key: "guimen", label: "鬼门" },
  { key: "hejie", label: "和解" },
  { key: "huxiao", label: "虎啸" },
  { key: "jianying", label: "渐营" },
  { key: "jinfan", label: "锦帆" },
  { key: "kuangbao", label: "狂暴" },
  { key: "lunhui", label: "轮回" },
  { key: "mohua", label: "魔化" },
  { key: "niepan", label: "涅槃" },
];

export interface BossSurfaceState {
  variant: "noname" | "liuli";
  mode: "boss" | "versus_jiange";
  stageIndex: number;
  stages: Array<{ id: string; title: string; hpRaw: number | string; forbidAi: boolean; allowed: boolean }>;
  forbidAiIds: string[];
  shichangshiAliveCount?: number;
}

export interface CoinSurfaceState {
  variant: "noname" | "liuli";
  balance: number;
  betPlaced: number;
  phaseStarted: boolean;
  alwaysSnow: boolean;
  snowFall: boolean;
  snowSize: "small" | "large";
  activeEffects: string[];
  lastSettlement: { payout: number; netDelta: number; video: boolean; mode: string } | null;
  honorTitle?: string;
}

export interface TaixuNode {
  id: string;
  title: string;
  kind: "start" | "battle" | "blessing" | "boss";
  cleared: boolean;
}

export interface TaixuSurfaceState {
  seasonId: string;
  gold: number;
  maxBlessings: number;
  currentNodeIndex: number;
  nodes: TaixuNode[];
  blessings: string[];
  boundPet: string | null;
}

export interface MartialSurfaceState {
  width: number;
  hiddenCount: number;
  totalSlots: number;
  firstMover: 0 | 1;
  friends: string[];
  enemies: string[];
  activeReplacement: string | null;
  replacementDraws: number;
  doubleGeneral: boolean;
  mightyFoeAlive: boolean;
  recordBook: { firstWin: number; firstLose: number; secondWin: number; secondLose: number };
}

export interface DoomsdaySurfaceState {
  endlessMode: boolean;
  wave: number;
  identities: string[];
  infectedSeats: number[];
}

export interface MvpScoreRow {
  seat: number;
  attack: number;
  heal: number;
  assist: number;
  base: number;
  penalty: number;
  total: number;
  kills: number;
  damageDealt: number;
}

export interface MvpSurfaceState {
  enabled: boolean;
  seenSeqs: number[];
  scores: Record<number, MvpScoreRow>;
  mvpSeat: number | null;
}

export interface ThemeAndSkinSurfaceState {
  decadeSkinStyle: "shizhounian" | "shousha" | "xinzhan" | "ol" | "baby" | "classic";
  cardBeautify: boolean;
  markStyle: "decade" | "shousha" | "classic";
  shoushaLayout: "right" | "left";
  pointerLayout: "mouse" | "touch";
  lastPointerClass: "ignored" | "tap" | "drag" | "touch";
  qhlyTheme: "decade" | "shousha" | "classic";
  activeSkinBySeat: Record<number, number>;
  lastSkinTrigger: string | null;
  lastVoiceCue: string | null;
  rzshTheme: "illusion" | "reality";
  rzshTierRoman: string;
  throwEmotionCue: string | null;
  selectBeautifyEnabled: boolean;
  d3ShowcaseEnabled: boolean;
  l20Marks?: Record<string, number>;
  u01SelectedGeneral?: string;
  u01YawDeg?: number;
  u01ZoomPct?: number;
  spineActiveClip?: "daiji" | "gongji" | "teshu" | "chuchang" | "shan";
  spineFrameSeq?: number;
  skinStandX?: number;
  skinStandY?: number;
  skinScalePct?: number;
  shoushaCountdownPct?: number;
  shoushaKillBanner?: string;
}

export interface LongxueSurfaceState {
  shengjian: boolean;
  daoxin: number;
  unlockedThresholds: number[];
  qingzhengUnlocked: boolean;
  unlockedSkills?: string[];
}

export interface ThunderSurfaceState {
  banSameHero: boolean;
  hideExclusivePack: boolean;
  markPatchState: "off" | "basic" | "full";
  replacedPairs: Array<{ from: string; to: string }>;
  chordMask?: number;
}

export interface OmnisearchEntry {
  packageId: string;
  kind: "general" | "skill" | "card";
  id: string;
  displayName: string;
}

export interface OmnisearchSurfaceState {
  query: string;
  results: OmnisearchEntry[];
  lastError: string | null;
  droppedPackages: string[];
}

export interface WorkbenchToolsState {
  aiTuningVariant: number;
  sjBase64Preview: string;
  sjVersionCompare: string;
  shoushaFilteredPacks: string[];
  dragLastKind: string;
  douzhuanRewrittenText: string;
  updateGuardUrl: string;
  eloDifficulty: { strength: number; bias: number; thinkDelaySec: number };
  zuanDebugAllowed: boolean;
  foolHintLabel: string;
  zuanDiyGeneralId?: string;
  zuanDiyMaxHp?: number;
  zuanDiyOnlineRoom?: boolean;
  zuanDiyStatus?: string;
  l27DualPrimary?: string;
  l27DualSecondary?: string;
  l27DualLocked?: boolean;
}

export interface ExtensionSurfacesSnapshot {
  version: 1;
  activeSources: string[];
  boss: BossSurfaceState;
  coin: CoinSurfaceState;
  taixu: TaixuSurfaceState;
  martial: MartialSurfaceState;
  doomsday: DoomsdaySurfaceState;
  mvp: MvpSurfaceState;
  theme: ThemeAndSkinSurfaceState;
  longxue: LongxueSurfaceState;
  thunder: ThunderSurfaceState;
  omnisearch: OmnisearchSurfaceState;
  workbench: WorkbenchToolsState;
}

const DEFAULT_OMNI_CATALOG: OmnisearchEntry[] = [
  { packageId: "standard", kind: "general", id: "caocao", displayName: "曹操" },
  { packageId: "standard", kind: "general", id: "liubei", displayName: "刘备" },
  { packageId: "standard", kind: "general", id: "sunquan", displayName: "孙权" },
  { packageId: "standard", kind: "general", id: "simayi", displayName: "司马懿" },
  { packageId: "standard", kind: "skill", id: "jianxiong", displayName: "奸雄" },
  { packageId: "standard", kind: "skill", id: "rende", displayName: "仁德" },
  { packageId: "standard", kind: "skill", id: "zhiheng", displayName: "制衡" },
  { packageId: "standard", kind: "card", id: "sha", displayName: "杀" },
  { packageId: "standard", kind: "card", id: "shan", displayName: "闪" },
  { packageId: "standard", kind: "card", id: "tao", displayName: "桃" },
  { packageId: "shahai-swd", kind: "card", id: "cangchizhibi", displayName: "苍螭之璧" },
  { packageId: "huanle", kind: "card", id: "hnl_kuwu", displayName: "苦无" },
  { packageId: "d3", kind: "general", id: "ddd_liuchong", displayName: "刘冲" },
  { packageId: "wudian", kind: "card", id: "diaobingqianjiang", displayName: "调兵遣将" },
  { packageId: "longxue", kind: "skill", id: "mbcmjiushi", displayName: "酒诗" },
  { packageId: "longxue", kind: "skill", id: "mbcmfangzhu", displayName: "放逐" },
  { packageId: "longxue", kind: "skill", id: "mbjuejin", displayName: "绝进" },
  { packageId: "epic-cards", kind: "general", id: "shichangshi", displayName: "十常侍" },
  { packageId: "thunder", kind: "skill", id: "th_jingxie", displayName: "精械" },
  { packageId: "hero-kill", kind: "skill", id: "yxs_menshen2", displayName: "门神·护卫" },
];

/** U02 / L03: 上游体力归一化（数字保留、`"3"` -> Infinity、`"3/4"` -> 分数体力）。 */
export function normalizeBossHp(raw: number | string, isChallenge = true): BossHpNormalized {
  if (typeof raw === "number") {
    const n = Math.max(1, Math.floor(raw));
    return { kind: "finite", hp: n, maxHp: n, label: `${n}/${n}` };
  }
  const trimmed = String(raw).trim();
  if (trimmed.includes("/")) {
    const [hpStr, maxStr] = trimmed.split("/");
    const hp = Math.max(1, Number.parseInt(hpStr ?? "1", 10) || 1);
    const maxHp = Math.max(hp, Number.parseInt(maxStr ?? String(hp), 10) || hp);
    if (isChallenge) {
      return { kind: "finite", hp, maxHp, label: `${hp}/${maxHp}` };
    }
    return { kind: "infinity", label: "∞" };
  }
  return { kind: "infinity", label: "∞" };
}

/** L12: 上游 `numtoroma` 罗马数字转换（1..=10）。 */
export function numToRoma(n: number): string {
  const table = ["", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"];
  if (!Number.isInteger(n) || n < 1 || n >= table.length) return String(n);
  return table[n]!;
}

/** L02 / L12 / L18: 上游版本号比较（`1.2` vs `1.10` -> `-1`）。 */
export function compareExtensionVersion(left: string, right: string): -1 | 0 | 1 {
  if (left === "无" || right === "无") return 1;
  const a = left.split(".").map((x) => Number.parseInt(x, 10) || 0);
  const b = right.split(".").map((x) => Number.parseInt(x, 10) || 0);
  const len = Math.max(a.length, b.length);
  for (let i = 0; i < len; i++) {
    const lv = a[i] ?? 0;
    const rv = b[i] ?? 0;
    if (lv < rv) return -1;
    if (lv > rv) return 1;
  }
  return 0;
}

/** L17: 斗转星移技能描述符号改写（`不大于`->`≤`、`不小于`->`≥`、`阴阳`->`阳阴`）。 */
export function rewriteDouzhuanSkillText(text: string): string {
  return text
    .replaceAll("不大于", "≤")
    .replaceAll("不小于", "≥")
    .replaceAll("阴阳", "阳阴");
}

/** L26: 笨蛋插件选牌辅助（资援/敏思 <= 13，方统首张禁方且凑 36）。 */
export function foolZiyuanAllows(currentSum: number, candidateNumber: number): boolean {
  return currentSum + candidateNumber <= 13;
}

export function foolFangtongAllows(selectedNumbers: number[], candidateNumber: number, isFangCard = false): boolean {
  if (selectedNumbers.length === 0 && isFangCard) return false;
  const sum = selectedNumbers.reduce((a, b) => a + b, 0);
  return sum + candidateNumber <= 36;
}

/** L23: 电脑适配指针分类（鼠标模式位移 < 10px 为 tap，>= 10px 为 drag；触屏模式为 touch）。 */
export function classifyDesktopPointer(
  layout: "mouse" | "touch",
  dx: number,
  dy: number,
): "tap" | "drag" | "touch" {
  if (layout === "touch") return "touch";
  return Math.hypot(dx, dy) < 10 ? "tap" : "drag";
}

/** L23: 电脑适配安全导出文件名校验（拒绝空名、`.`、`..` 及路径分隔符）。 */
export function validateSafeExportFilename(name: string): string {
  const trimmed = name.trim();
  if (!trimmed || trimmed === "." || trimmed === ".." || /[\\/]/.test(trimmed)) {
    throw new Error("unsafe export filename");
  }
  return trimmed;
}

/** L22: 段位制裁难度与偏向计算。 */
export function resolveEloDifficulty(
  online: boolean,
  mode: string,
  tierLevel: number | null,
  unitRandom = 0.5,
): { strength: number; bias: number; thinkDelaySec: number } {
  if (!online || mode === "taixu") {
    return { strength: 1, bias: 0, thinkDelaySec: 0 };
  }
  let strength = 2;
  let bias = 1;
  if (tierLevel !== null && tierLevel >= 0) {
    const table: Record<number, [number, number]> = {
      0: [4, 1],
      1: [3, 1],
      2: [2, 1],
      3: [1, 0],
      4: [2, -1],
      5: [3, -1],
      6: [4, -1],
    };
    const hit = table[tierLevel];
    if (hit) [strength, bias] = hit;
  }
  const thinkDelaySec = Math.round((1.5 + unitRandom * 1.5) * 100) / 100;
  return { strength, bias, thinkDelaySec };
}

/** L29: 根据道心阈值返回已解锁的 4 项技能名列表（25 清正 / 50 酒诗 / 75 放逐 / 99 绝进）。 */
export function unlockLongxueSkills(daoxin: number): string[] {
  const skills: string[] = [];
  if (daoxin >= 25) skills.push("mbcmqingzheng");
  if (daoxin >= 50) skills.push("mbcmjiushi");
  if (daoxin >= 75) skills.push("mbcmfangzhu");
  if (daoxin >= 99) skills.push("mbjuejin");
  return skills;
}

/** L29: 龙血玄黄潜龙道心值推进与阈值解锁（上限 99，阈值 25/50/75/99）。 */
export function advanceLongxueDaoxin(
  state: LongxueSurfaceState,
  deltaKind: "gain_card" | "damaged" | "damage_dealt",
  amount = 1,
): LongxueSurfaceState {
  const delta =
    deltaKind === "gain_card"
      ? 5 * amount
      : deltaKind === "damaged"
        ? 10 * amount
        : 15 * amount;
  const daoxin = Math.min(99, Math.max(0, state.daoxin + delta));
  const unlockedThresholds = [25, 50, 75, 99].filter((t) => daoxin >= t);
  return {
    ...state,
    daoxin,
    unlockedThresholds,
    qingzhengUnlocked: daoxin >= 25,
  };
}

/** L20: 标记补充 14 项战局 HUD 标记增减与归一化。 */
export function incrementL20HudMark(
  theme: ThemeAndSkinSurfaceState,
  markKey: string,
  delta = 1,
): ThemeAndSkinSurfaceState {
  const valid = L20_HUD_MARK_SPECS.some((m) => m.key === markKey);
  if (!valid) {
    throw new Error(`unknown L20 HUD mark: ${markKey}`);
  }
  const prev = theme.l20Marks ?? {};
  const nextCount = Math.max(0, (prev[markKey] ?? 0) + delta);
  return {
    ...theme,
    l20Marks: {
      ...prev,
      [markKey]: nextCount,
    },
  };
}

export const U01_DDD_GALLERY_GENERALS: ReadonlyArray<string> = [
  "ddd_handang", "ddd_wuzhi", "ddd_xujing", "ddd_caomao", "ddd_xinxianying",
  "ddd_xianglang", "ddd_yujin", "ddd_liuye", "ddd_baosanniang", "ddd_zhenji",
  "ddd_zhaoang", "ddd_zhouchu", "ddd_liuba", "ddd_jianshuo", "ddd_guanning",
  "ddd_dingfeng", "ddd_caoshuang", "ddd_xuelingyun", "ddd_liuhong", "ddd_xiahouxuan",
  "ddd_zhangkai", "ddd_liangxi", "ddd_wangkanglvkai", "ddd_sunliang", "ddd_lie",
  "ddd_kebineng", "ddd_qianzhao", "ddd_zhangmiao", "ddd_zhangcheng", "ddd_liuchong",
  "ddd_luoxian",
];

export const SPINE_CLIP_ORDER: ReadonlyArray<"daiji" | "gongji" | "teshu" | "chuchang" | "shan"> = [
  "daiji",
  "gongji",
  "teshu",
  "chuchang",
  "shan",
];

/** U01: 3D 精选 31 武将透视立绘陈列馆旋转与缩放。 */
export function rotateU01GalleryStandee(
  theme: ThemeAndSkinSurfaceState,
  nextGeneral?: string,
  deltaYaw = 15,
): ThemeAndSkinSurfaceState {
  const curGen = theme.u01SelectedGeneral ?? "ddd_caomao";
  const curIdx = U01_DDD_GALLERY_GENERALS.indexOf(curGen);
  const chosen =
    nextGeneral ??
    U01_DDD_GALLERY_GENERALS[(curIdx + 1) % U01_DDD_GALLERY_GENERALS.length]!;
  if (!U01_DDD_GALLERY_GENERALS.includes(chosen)) {
    throw new Error(`unknown U01 3D general: ${chosen}`);
  }
  const rawYaw = (theme.u01YawDeg ?? 0) + deltaYaw;
  const yaw = rawYaw > 45 ? -30 : Math.max(-45, Math.min(45, rawYaw));
  const zoom = Math.max(80, Math.min(150, (theme.u01ZoomPct ?? 100) + 5));
  return {
    ...theme,
    u01SelectedGeneral: chosen,
    u01YawDeg: yaw,
    u01ZoomPct: zoom,
  };
}

/** L08 / L24: Spine 骨骼动画剪辑与皮肤立绘坐标变换（daiji/gongji/teshu/chuchang/shan）。 */
export function triggerSpineSkinClip(
  theme: ThemeAndSkinSurfaceState,
  clip?: "daiji" | "gongji" | "teshu" | "chuchang" | "shan",
): ThemeAndSkinSurfaceState {
  const curClip = theme.spineActiveClip ?? "daiji";
  const nextClip =
    clip ?? SPINE_CLIP_ORDER[(SPINE_CLIP_ORDER.indexOf(curClip) + 1) % SPINE_CLIP_ORDER.length]!;
  if (!SPINE_CLIP_ORDER.includes(nextClip)) {
    throw new Error(`invalid spine clip: ${nextClip}`);
  }
  const slot = theme.activeSkinBySeat[0] ?? 1;
  const [standX, standY, scalePct] =
    nextClip === "gongji"
      ? [slot * 12, -8, 108]
      : nextClip === "teshu"
        ? [slot * 8, -14, 112]
        : nextClip === "chuchang"
          ? [0, -20, 115]
          : nextClip === "shan"
            ? [-slot * 10, 4, 96]
            : [slot * 4, 0, 100];
  return {
    ...theme,
    spineActiveClip: nextClip,
    spineFrameSeq: (theme.spineFrameSeq ?? 1) + 1,
    skinStandX: standX,
    skinStandY: standY,
    skinScalePct: scalePct,
    lastSkinTrigger: `spine:${nextClip}@slot${slot}`,
  };
}

/** L14: 手杀 UI 倒计时进度条、操作按钮栏与击杀播报横幅更新。 */
export function triggerShoushaHudBanner(
  theme: ThemeAndSkinSurfaceState,
  banner = "无双·千军破(zhefu)",
): ThemeAndSkinSurfaceState {
  const nextPct = Math.max(15, ((theme.shoushaCountdownPct ?? 85) - 15 + 100) % 100);
  return {
    ...theme,
    shoushaCountdownPct: nextPct,
    shoushaKillBanner: banner,
    lastVoiceCue: "skill/zhefu1.mp3",
  };
}

/** L25: 祖安扩展单机离线 DIY 武将/技能编辑器（联机房间强制拦截）。 */
export function applyZuanDiyDraft(
  workbench: WorkbenchToolsState,
  draft: { generalId: string; maxHp: number; onlineRoom: boolean },
): WorkbenchToolsState {
  if (draft.onlineRoom) {
    throw new Error("ONLINE_MULTIPLAYER_DIY_FORBIDDEN");
  }
  const trimmed = draft.generalId.trim();
  if (!trimmed) {
    throw new Error("EMPTY_DIY_GENERAL_ID");
  }
  const hp = Math.max(1, Math.min(15, Math.floor(draft.maxHp)));
  return {
    ...workbench,
    zuanDiyGeneralId: trimmed,
    zuanDiyMaxHp: hp,
    zuanDiyOnlineRoom: false,
    zuanDiyStatus: `已应用离线 DIY: ${trimmed} (${hp}血)`,
  };
}

/** L27: 选将美化双将同选面板（chooseCharacterTwo）校验与锁定。 */
export function selectL27DualGenerals(
  workbench: WorkbenchToolsState,
  primary: string,
  secondary: string,
): WorkbenchToolsState {
  const p = primary.trim();
  const s = secondary.trim();
  if (!p || !s || p === s) {
    throw new Error("DUAL_PICK_REQUIRES_TWO_DISTINCT_GENERALS");
  }
  return {
    ...workbench,
    l27DualPrimary: p,
    l27DualSecondary: s,
    l27DualLocked: true,
  };
}

/** U04 / L05: 金币余额对应荣誉称号。 */
export function resolveCoinHonorTitle(balance: number): string {
  if (balance >= 200) return "富甲天下";
  if (balance >= 100) return "挥金如土";
  return "初入茅庐";
}

/** U04 / L05: 商店购买与下注、终局金币结算（对齐上游 `extension/coin/extension.js`）。 */
export function buyCoinShopItem(
  state: CoinSurfaceState,
  item: "fireworks" | "snow" | "nebula" | "twinkle",
): CoinSurfaceState {
  const prices: Record<typeof item, number> = {
    fireworks: 50,
    snow: 20,
    nebula: 10,
    twinkle: 10,
  };
  const cost = item === "snow" && state.alwaysSnow ? 0 : prices[item];
  if (state.balance < cost) {
    throw new Error(`金币不足（需要 ${cost}，当前 ${state.balance}）`);
  }
  const activeEffects = state.activeEffects.includes(item)
    ? state.activeEffects
    : [...state.activeEffects, item];
  return {
    ...state,
    balance: state.balance - cost,
    snowFall: item === "snow" ? true : state.snowFall,
    activeEffects,
  };
}

export function placeCoinBet(state: CoinSurfaceState, amount = 20): CoinSurfaceState {
  if (state.phaseStarted) {
    throw new Error("对局阶段已开始，不可下注");
  }
  if (state.betPlaced > 0) {
    throw new Error("本局已下注");
  }
  if (state.balance < amount) {
    throw new Error("金币不足，无法下注");
  }
  return {
    ...state,
    balance: state.balance - amount,
    betPlaced: amount,
  };
}

export function settleCoinMatch(
  state: CoinSurfaceState,
  opts: {
    mode: "identity" | "other" | "chess";
    role?: "Lord" | "Loyalist" | "Rebel" | "Renegade";
    players?: number;
    won: boolean;
    unitRandom?: number;
    video?: boolean;
    chessLeader?: boolean;
  },
): CoinSurfaceState {
  const r = opts.unitRandom ?? 0;
  let payout = 0;
  if (opts.mode === "identity") {
    const n = opts.players ?? 8;
    if (opts.won) {
      payout = opts.role === "Lord" ? Math.round(n * 4.5 + r * 10) : Math.round(n * 3.375 + r * 10);
    } else {
      payout = 10;
    }
  } else if (opts.mode === "chess") {
    payout = opts.won ? 20 : opts.chessLeader ? 2 + Math.floor(r * 5) : 5;
  } else {
    payout = opts.won ? Math.round(20 + r * 20) : 10;
  }
  if (state.betPlaced > 0 && opts.won) {
    payout += state.betPlaced;
  }
  const returnedBet = state.betPlaced > 0 && opts.won ? state.betPlaced : 0;
  const video = Boolean(opts.video);
  const netDelta = video ? 0 : payout + returnedBet;
  return {
    ...state,
    balance: state.balance + netDelta,
    betPlaced: 0,
    lastSettlement: {
      payout,
      netDelta,
      video,
      mode: opts.mode,
    },
  };
}

/** L11: 太虚幻境节点推进、赐福购买与灵宠绑定。 */
export function advanceTaixuNode(state: TaixuSurfaceState): TaixuSurfaceState {
  const idx = state.currentNodeIndex;
  const nodes = state.nodes.map((n, i) => (i === idx ? { ...n, cleared: true } : n));
  const nextIdx = Math.min(state.nodes.length - 1, idx + 1);
  return {
    ...state,
    currentNodeIndex: nextIdx,
    nodes,
    gold: state.gold + 30,
  };
}

export function buyTaixuBlessing(
  state: TaixuSurfaceState,
  blessingName: string,
  cost = 40,
): TaixuSurfaceState {
  if (state.blessings.length >= state.maxBlessings) {
    throw new Error("赐福数量已达上限");
  }
  if (state.gold < cost) {
    throw new Error("太虚金币不足");
  }
  if (state.blessings.includes(blessingName)) {
    return state;
  }
  return {
    ...state,
    gold: state.gold - cost,
    blessings: [...state.blessings, blessingName],
  };
}

export function bindTaixuPet(state: TaixuSurfaceState, petName: string): TaixuSurfaceState {
  return {
    ...state,
    boundPet: petName,
  };
}

/** L21: 武皇争霸排阵、挫锐替补与强敌结算。 */
export function assignMartialLineup(
  pool: string[],
  firstMover: 0 | 1,
): { friends: string[]; enemies: string[] } {
  const padded = pool.length >= 10 ? pool.slice(0, 10) : ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];
  if (firstMover === 0) {
    return {
      friends: [padded[0]!, padded[1]!, padded[2]!, padded[3]!, padded[4]!],
      enemies: [padded[9]!, padded[8]!, padded[7]!, padded[6]!, padded[5]!],
    };
  }
  return {
    friends: [padded[0]!, padded[1]!, padded[2]!, padded[4]!, padded[5]!],
    enemies: [padded[9]!, padded[8]!, padded[7]!, padded[3]!, padded[6]!],
  };
}

export function applyMartialReplacement(
  state: MartialSurfaceState,
  candidates: string[],
): MartialSurfaceState {
  const pick = candidates[0] ?? "a";
  return {
    ...state,
    activeReplacement: pick,
    replacementDraws: 4,
  };
}

/** L07: 全能搜索公开目录检索（严格拒绝暗牌实例号查询与脚本执行）。 */
export function runOmnisearchQuery(
  state: OmnisearchSurfaceState,
  rawQuery: string,
  hiddenCardIds: number[] = [],
  extraEntries: OmnisearchEntry[] = [],
): OmnisearchSurfaceState {
  const q = rawQuery.trim();
  if (!q) {
    return { ...state, query: rawQuery, results: [], lastError: "error:empty" };
  }
  if (q.startsWith("eval:") || q.includes("game.") || q.includes("window.")) {
    return { ...state, query: rawQuery, results: [], lastError: "error:eval_refused" };
  }
  const numeric = Number.parseInt(q.replace(/^#/, ""), 10);
  if (Number.isInteger(numeric) && hiddenCardIds.includes(numeric)) {
    return { ...state, query: rawQuery, results: [], lastError: "error:hidden" };
  }
  const dropped = new Set(state.droppedPackages);
  const catalog = [...DEFAULT_OMNI_CATALOG, ...extraEntries].filter(
    (entry) => !dropped.has(entry.packageId),
  );
  const lower = q.toLowerCase();
  const results = catalog.filter(
    (entry) =>
      entry.id.toLowerCase().includes(lower) ||
      entry.displayName.toLowerCase().includes(lower) ||
      entry.packageId.toLowerCase().includes(lower),
  );
  return {
    ...state,
    query: rawQuery,
    results,
    lastError: null,
  };
}

/** L13: 手杀MVP 事件计分（按 seq 去重，对齐上游 `extension/手杀MVP/extension.js`）。 */
export function recordMvpEventFact(
  state: MvpSurfaceState,
  fact: {
    seq: number;
    seat: number;
    kind: "damage_card" | "wuxie" | "multi_trick" | "damage" | "recover" | "cards" | "kill_enemy" | "kill_ally";
    amount?: number;
  },
): MvpSurfaceState {
  if (state.seenSeqs.includes(fact.seq)) return state;
  const n = Math.max(1, fact.amount ?? 1);
  const prev = state.scores[fact.seat] ?? {
    seat: fact.seat,
    attack: 0,
    heal: 0,
    assist: 0,
    base: 100,
    penalty: 0,
    total: 100,
    kills: 0,
    damageDealt: 0,
  };
  const row: MvpScoreRow = { ...prev };
  switch (fact.kind) {
    case "damage_card":
      row.attack += 2;
      break;
    case "wuxie":
      row.assist += 2;
      break;
    case "multi_trick":
      row.assist += 1;
      break;
    case "damage":
      row.attack += n > 5 ? 15 : 3 * n;
      row.damageDealt += n;
      break;
    case "recover":
      row.heal += n > 5 ? 10 : 2 * n;
      break;
    case "cards":
      row.assist += n;
      break;
    case "kill_enemy":
      row.attack += 3;
      row.kills += 1;
      break;
    case "kill_ally":
      row.penalty += 5;
      break;
  }
  row.total = row.attack + row.heal + row.assist + row.base - row.penalty;
  const scores = { ...state.scores, [fact.seat]: row };
  const ranked = Object.values(scores).sort((a, b) => a.total - b.total || a.seat - b.seat).reverse();
  return {
    ...state,
    seenSeqs: [...state.seenSeqs, fact.seq],
    scores,
    mvpSeat: ranked[0]?.seat ?? null,
  };
}

export function createInitialExtensionSurfacesState(
  activeSources: string[] = [
    "U01", "U02", "U03", "U04", "U05", "U06", "U07", "U08",
    "L01", "L02", "L03", "L04", "L05", "L06", "L07", "L08", "L09", "L10",
    "L11", "L12", "L13", "L14", "L15", "L16", "L17", "L18", "L19", "L20",
    "L21", "L22", "L23", "L24", "L25", "L26", "L27", "L28", "L29",
  ],
): ExtensionSurfacesSnapshot {
  const lineup = assignMartialLineup(["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"], 0);
  return {
    version: 1,
    activeSources: [...activeSources],
    boss: {
      variant: "noname",
      mode: "boss",
      stageIndex: 0,
      stages: [
        { id: "boss_lvbu1", title: "虎牢关·神吕布", hpRaw: "8/8", forbidAi: true, allowed: true },
        { id: "jiange_gundam", title: "剑阁·高达一号", hpRaw: "3", forbidAi: true, allowed: true },
        { id: "boss_baonu", title: "挑战·暴怒战神", hpRaw: "6/8", forbidAi: false, allowed: true },
      ],
      forbidAiIds: ["boss_lvbu1", "jiange_gundam"],
    },
    coin: {
      variant: "noname",
      balance: 100,
      betPlaced: 0,
      phaseStarted: false,
      alwaysSnow: false,
      snowFall: false,
      snowSize: "small",
      activeEffects: [],
      lastSettlement: null,
    },
    taixu: {
      seasonId: "S1-黄天当立",
      gold: 100,
      maxBlessings: 3,
      currentNodeIndex: 0,
      nodes: [
        { id: "node_0", title: "广宗营地（起点）", kind: "start", cleared: true },
        { id: "node_1", title: "颍川前哨（战斗）", kind: "battle", cleared: false },
        { id: "node_2", title: "太平祭坛（赐福）", kind: "blessing", cleared: false },
        { id: "node_3", title: "天公将军（首领）", kind: "boss", cleared: false },
      ],
      blessings: ["黄天之庇（摸牌+1）"],
      boundPet: "玄木灵鹿",
    },
    martial: {
      width: 5,
      hiddenCount: 4,
      totalSlots: 20,
      firstMover: 0,
      friends: lineup.friends,
      enemies: lineup.enemies,
      activeReplacement: null,
      replacementDraws: 0,
      doubleGeneral: false,
      mightyFoeAlive: true,
      recordBook: { firstWin: 1, firstLose: 1, secondWin: 1, secondLose: 0 },
    },
    doomsday: {
      endlessMode: true,
      wave: 1,
      identities: ["主公", "忠臣", "反贼", "内奸", "僵尸", "退治者"],
      infectedSeats: [],
    },
    mvp: {
      enabled: true,
      seenSeqs: [],
      scores: {
        0: { seat: 0, attack: 0, heal: 0, assist: 0, base: 100, penalty: 0, total: 100, kills: 0, damageDealt: 0 },
      },
      mvpSeat: 0,
    },
    theme: {
      decadeSkinStyle: "shizhounian",
      cardBeautify: true,
      markStyle: "decade",
      shoushaLayout: "right",
      pointerLayout: "mouse",
      lastPointerClass: "tap",
      qhlyTheme: "decade",
      activeSkinBySeat: { 0: 1, 1: 1, 2: 1, 3: 1 },
      lastSkinTrigger: null,
      lastVoiceCue: "skill/rende1.mp3",
      rzshTheme: "illusion",
      rzshTierRoman: numToRoma(1),
      throwEmotionCue: null,
      selectBeautifyEnabled: true,
      d3ShowcaseEnabled: true,
      u01SelectedGeneral: "ddd_caomao",
      u01YawDeg: 15,
      u01ZoomPct: 110,
      spineActiveClip: "daiji",
      spineFrameSeq: 1,
      skinStandX: 4,
      skinStandY: 0,
      skinScalePct: 100,
      shoushaCountdownPct: 85,
      shoushaKillBanner: "连破·一血(spshidi)",
    },
    longxue: {
      shengjian: true,
      daoxin: 20,
      unlockedThresholds: [],
      qingzhengUnlocked: false,
    },
    thunder: {
      banSameHero: true,
      hideExclusivePack: true,
      markPatchState: "full",
      replacedPairs: [
        { from: "guanning", to: "th_guanning" },
        { from: "majun", to: "th_majun" },
      ],
    },
    omnisearch: {
      query: "",
      results: DEFAULT_OMNI_CATALOG.slice(0, 6),
      lastError: null,
      droppedPackages: [],
    },
    workbench: {
      aiTuningVariant: 2,
      sjBase64Preview: "eyJhIjoxfQ==",
      sjVersionCompare: `1.2 vs 1.10 => ${compareExtensionVersion("1.2", "1.10")}`,
      shoushaFilteredPacks: ["standard", "sp", "refresh", "shiji"],
      dragLastKind: "zip",
      douzhuanRewrittenText: rewriteDouzhuanSkillText("伤害不大于2且不小于1，阴阳"),
      updateGuardUrl: "https://unitedrhythmized.club/libccy/noname",
      eloDifficulty: resolveEloDifficulty(true, "rank", 1, 0.5),
      zuanDebugAllowed: true,
      foolHintLabel: "强中加伤",
      zuanDiyGeneralId: "ssr_luxun",
      zuanDiyMaxHp: 4,
      zuanDiyOnlineRoom: false,
      zuanDiyStatus: "离线沙箱就绪 (60将/122技)",
      l27DualPrimary: "caocao",
      l27DualSecondary: "simayi",
      l27DualLocked: true,
    },
  };
}

/** 根据对局 `EngineEnvelope` 推进扩展表现面与单机战役脚本状态。 */
export function ingestBattleEnvelopeIntoSurfaces(
  state: ExtensionSurfacesSnapshot,
  env: EngineEnvelope,
): ExtensionSurfacesSnapshot {
  let next: ExtensionSurfacesSnapshot = {
    ...state,
    coin: {
      ...state.coin,
      phaseStarted: env.revision > 0 || state.coin.phaseStarted,
    },
    theme: {
      ...state.theme,
      rzshTierRoman: numToRoma(Math.max(1, Math.min(10, (env.view?.round ?? 1)))),
    },
  };

  const seats = env.view?.seats ?? [];
  for (const seat of seats) {
    if ( !(seat.player in next.mvp.scores)) {
      next = {
        ...next,
        mvp: {
          ...next.mvp,
          scores: {
            ...next.mvp.scores,
            [seat.player]: {
              seat: seat.player,
              attack: 0,
              heal: 0,
              assist: 0,
              base: 100,
              penalty: 0,
              total: 100,
              kills: 0,
              damageDealt: 0,
            },
          },
        },
      };
    }
  }

  const events = env.events ?? [];
  events.forEach((rawEv, idx) => {
    const seq = env.revision * 100 + idx + 1;
    const pubCode = (rawEv as { PublicEvent?: { code?: number } })?.PublicEvent?.code;
    if (pubCode === 4 || (rawEv as { type?: string })?.type === "DamageDealt") {
      next = {
        ...next,
        mvp: recordMvpEventFact(next.mvp, { seq, seat: env.view?.current_player ?? 0, kind: "damage", amount: 1 }),
        longxue: advanceLongxueDaoxin(next.longxue, "damage_dealt", 1),
        theme: { ...next.theme, lastSkinTrigger: `damage@rev${env.revision}` },
      };
    } else if (pubCode === 5 || (rawEv as { type?: string })?.type === "HpRecovered") {
      next = {
        ...next,
        mvp: recordMvpEventFact(next.mvp, { seq, seat: env.view?.current_player ?? 0, kind: "recover", amount: 1 }),
      };
    } else if (pubCode === 2 || (rawEv as { type?: string })?.type === "CardPlayed") {
      next = {
        ...next,
        mvp: recordMvpEventFact(next.mvp, { seq, seat: env.view?.current_player ?? 0, kind: "damage_card", amount: 1 }),
        longxue: advanceLongxueDaoxin(next.longxue, "gain_card", 1),
      };
    }
  });

  if (env.finished && !next.coin.lastSettlement) {
    const selfSeat = seats[0];
    const won = Boolean(selfSeat?.alive);
    next = {
      ...next,
      coin: settleCoinMatch(next.coin, {
        mode: "identity",
        role: (selfSeat?.identity as "Lord" | "Loyalist" | "Rebel" | "Renegade" | undefined) ?? "Lord",
        players: seats.length || 4,
        won,
        unitRandom: 0,
        video: false,
      }),
    };
  }

  return next;
}

export function serializeExtensionSurfacesSnapshot(state: ExtensionSurfacesSnapshot): string {
  return JSON.stringify(state);
}

export function deserializeExtensionSurfacesSnapshot(raw: string): ExtensionSurfacesSnapshot {
  let parsed: Partial<ExtensionSurfacesSnapshot>;
  try {
    parsed = JSON.parse(raw) as Partial<ExtensionSurfacesSnapshot>;
  } catch {
    throw new Error("invalid extension surfaces snapshot");
  }
  const valid =
    Boolean(parsed && typeof parsed === "object") &&
    parsed.version === 1 &&
    Array.isArray(parsed.activeSources) &&
    Boolean(
      parsed.boss &&
        typeof parsed.boss === "object" &&
        Array.isArray(parsed.boss.stages) &&
        Array.isArray(parsed.boss.forbidAiIds),
    ) &&
    Boolean(
      parsed.coin &&
        typeof parsed.coin === "object" &&
        typeof parsed.coin.balance === "number" &&
        Array.isArray(parsed.coin.activeEffects),
    ) &&
    Boolean(
      parsed.taixu &&
        typeof parsed.taixu === "object" &&
        Array.isArray(parsed.taixu.nodes) &&
        Array.isArray(parsed.taixu.blessings),
    ) &&
    Boolean(
      parsed.martial &&
        typeof parsed.martial === "object" &&
        Array.isArray(parsed.martial.friends) &&
        Array.isArray(parsed.martial.enemies) &&
        Boolean(parsed.martial.recordBook && typeof parsed.martial.recordBook === "object"),
    ) &&
    Boolean(
      parsed.doomsday &&
        typeof parsed.doomsday === "object" &&
        Array.isArray(parsed.doomsday.identities) &&
        Array.isArray(parsed.doomsday.infectedSeats),
    ) &&
    Boolean(
      parsed.mvp &&
        typeof parsed.mvp === "object" &&
        Array.isArray(parsed.mvp.seenSeqs) &&
        Boolean(parsed.mvp.scores && typeof parsed.mvp.scores === "object"),
    ) &&
    Boolean(
      parsed.theme &&
        typeof parsed.theme === "object" &&
        Boolean(parsed.theme.activeSkinBySeat && typeof parsed.theme.activeSkinBySeat === "object"),
    ) &&
    Boolean(
      parsed.longxue &&
        typeof parsed.longxue === "object" &&
        typeof parsed.longxue.daoxin === "number" &&
        Array.isArray(parsed.longxue.unlockedThresholds),
    ) &&
    Boolean(
      parsed.thunder &&
        typeof parsed.thunder === "object" &&
        Array.isArray(parsed.thunder.replacedPairs),
    ) &&
    Boolean(
      parsed.omnisearch &&
        typeof parsed.omnisearch === "object" &&
        Array.isArray(parsed.omnisearch.results),
    ) &&
    Boolean(
      parsed.workbench &&
        typeof parsed.workbench === "object" &&
        Array.isArray(parsed.workbench.shoushaFilteredPacks) &&
        Boolean(parsed.workbench.eloDifficulty && typeof parsed.workbench.eloDifficulty === "object"),
    );
  if (!valid) {
    throw new Error("invalid extension surfaces snapshot");
  }
  return parsed as ExtensionSurfacesSnapshot;
}

/**
 * 读档时恢复扩展表现面状态：
 * - 若存档槽包含 `extensionSurfacesJson`，精确反序列化该快照（不被读档信封二次累加污染）；
 * - 若为未含 `extensionSurfacesJson` 的历史存档槽，先重置为初始状态再注入读档信封，避免上一局会话状态泄漏。
 */
export function restoreExtensionSurfacesForLoad(
  raw: string | undefined,
  env: EngineEnvelope,
): ExtensionSurfacesSnapshot {
  if (raw) {
    return deserializeExtensionSurfacesSnapshot(raw);
  }
  return ingestBattleEnvelopeIntoSurfaces(createInitialExtensionSurfacesState(), env);
}

export function hiddenCardIdsFromSnapshot(view: ObserverSnapshot | null, selfId: number): number[] {
  if (!view) return [];
  return view.known_cards
    .filter((c) => c.owner !== selfId && c.zone === "Hand")
    .map((c) => c.id);
}

