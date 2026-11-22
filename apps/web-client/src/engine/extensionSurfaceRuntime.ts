import type { EngineEnvelope, ObserverSnapshot } from "../types/engine";
import {
  createInitialDecadeUiRuntimeState,
  updateDecadeConfigInState,
} from "./surfaces/decadeUiSurface";
import { syncDecadeUiFromEnvelope, syncGuanXingChoice } from "./surfaces/decadeHooksRuntime";
import { normalizeDecadeConfig, type DecadeUiConfig } from "./surfaces/decadeConfigSchema";
import {
  normalizeShoushaConfig,
  type ShoushaUiConfig,
} from "./surfaces/shoushaConfigSchema";
import { syncShoushaUiFromEnvelope } from "./surfaces/shoushaSurfacesRuntime";
import {
  createInitialQhlyRuntimeState,
  dispatchQhlyUiHook,
  updateQhlyConfigInState,
} from "./surfaces/qhlySurface";
import { normalizeQhlyConfig, type QhlyUiConfig } from "./surfaces/qhlyConfigSchema";
import {
  createInitialL20RuntimeState,
  syncL20MarkEffectsFromEnvelope,
  type L20RuntimeState,
  type L20UiConfig,
} from "./surfaces/l20MarkSupplementRuntime";
import {
  createInitialShoushaUiRuntimeState,
  dispatchShoushaUiHook,
  updateShoushaConfigInState,
} from "./surfaces/shoushaUiSurface";
import {
  createInitialSkinSwitchRuntimeState,
  dispatchSkinSwitchHook,
  updateSkinSwitchConfigInState,
} from "./surfaces/skinSwitchSurface";
import { normalizeSkinSwitchConfig, type SkinSwitchUiConfig } from "./surfaces/skinSwitchConfigSchema";
import { createInitialTaixuUiRuntimeState } from "./surfaces/taixuSurface";
import {
  createInitialZuanUiRuntimeState,
  dispatchZuanUiHook,
  updateZuanConfigInState,
  type ZuanUiRuntimeState,
} from "./surfaces/zuanSurface";
import { createDefaultZuanConfig } from "./surfaces/zuanConfigSchema";
import {
  createInitialBatch6PatchUiState,
  dispatchL10EmojiRebate,
  dispatchLongxueCmEffects,
  type Batch6PatchUiState,
} from "./surfaces/batch6PatchSurfacesRuntime";
import {
  classifyDesktopPointer,
  incrementMvpRecastCount,
  numToRoma,
  validateSafeExportFilename,
  type CoinSurfaceState,
  type MvpScoreRow,
  type MvpSurfaceState,
} from "./surfaces/sharedSurfaceHelpers";
import {
  createInitialBatch7UiSuiteState,
  normalizeBatch7UiSuiteState,
  type Batch7UiSuiteState,
} from "./surfaces/batch7ClosureState";

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

export type { CoinSurfaceState, MvpScoreRow, MvpSurfaceState };
export {
  classifyDesktopPointer,
  incrementMvpRecastCount,
  numToRoma,
  validateSafeExportFilename,
};

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

export interface ExtensionUiSuitesState {
  qhly?: import("./surfaces/qhlySurface").QhlyRuntimeState;
  skinSwitch?: import("./surfaces/skinSwitchSurface").SkinSwitchRuntimeState;
  shoushaUi?: import("./surfaces/shoushaUiSurface").ShoushaUiRuntimeState;
  decadeUi?: import("./surfaces/decadeUiSurface").DecadeUiRuntimeState;
  taixuUi?: import("./surfaces/taixuSurface").TaixuUiRuntimeState;
  zuanUi?: ZuanUiRuntimeState;
  l20Ui?: L20RuntimeState;
  batch6Patch?: Batch6PatchUiState;
  batch7?: Batch7UiSuiteState;
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
  uiSuites?: ExtensionUiSuitesState;
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
  banner = "手杀播报·天下无双",
): ThemeAndSkinSurfaceState {
  const nextPct = Math.max(15, ((theme.shoushaCountdownPct ?? 85) - 15 + 100) % 100);
  return {
    ...theme,
    shoushaCountdownPct: nextPct,
    shoushaKillBanner: banner,
    lastVoiceCue: "手杀ui/audio/huan.mp3",
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
      seasonId: "ChongYingChuLin（冲应初临）",
      gold: 100,
      maxBlessings: 3,
      currentNodeIndex: 0,
      nodes: [
        { id: "node_0", title: "序章·冲应初临（起点）", kind: "start", cleared: true },
        { id: "node_1", title: "幻境守护童子（战斗）", kind: "battle", cleared: false },
        { id: "node_2", title: "天命加身（事件）", kind: "blessing", cleared: false },
        { id: "node_3", title: "天匠神兵（首领）", kind: "boss", cleared: false },
      ],
      blessings: ["丰裕生财"],
      boundPet: "盾盾",
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
      shoushaKillBanner: "手杀播报·乱世豪杰",
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
    uiSuites: {
      qhly: createInitialQhlyRuntimeState(),
      skinSwitch: createInitialSkinSwitchRuntimeState(),
      shoushaUi: createInitialShoushaUiRuntimeState(),
      decadeUi: createInitialDecadeUiRuntimeState(),
      taixuUi: createInitialTaixuUiRuntimeState(),
      zuanUi: createInitialZuanUiRuntimeState(),
      l20Ui: createInitialL20RuntimeState(),
      batch6Patch: createInitialBatch6PatchUiState(),
      batch7: createInitialBatch7UiSuiteState(),
    },
  };
}

/** 根据对局 `EngineEnvelope` 推进扩展表现面、五套 UI 皮肤钩子与单机战役脚本状态。 */
export function ingestBattleEnvelopeIntoSurfaces(
  state: ExtensionSurfacesSnapshot,
  env: EngineEnvelope,
): ExtensionSurfacesSnapshot {
  let qhly = state.uiSuites?.qhly ?? createInitialQhlyRuntimeState();
  let skinSwitch = state.uiSuites?.skinSwitch ?? createInitialSkinSwitchRuntimeState();
  let shoushaUi = state.uiSuites?.shoushaUi ?? createInitialShoushaUiRuntimeState();
  let decadeUi = state.uiSuites?.decadeUi ?? createInitialDecadeUiRuntimeState();
  const taixuUi = state.uiSuites?.taixuUi ?? createInitialTaixuUiRuntimeState();
  let zuanUi = state.uiSuites?.zuanUi ?? createInitialZuanUiRuntimeState();
  let l20Ui = state.uiSuites?.l20Ui ?? createInitialL20RuntimeState();
  let batch6Patch = state.uiSuites?.batch6Patch ?? createInitialBatch6PatchUiState();
  const baseBatch7 = state.uiSuites?.batch7 ?? createInitialBatch7UiSuiteState();
  const batch7: Batch7UiSuiteState = {
    ...baseBatch7,
    l12: {
      ...baseBatch7.l12,
      tierRoman: numToRoma(Math.max(1, Math.min(10, env.view?.round ?? 1))),
    },
  };

  let next: ExtensionSurfacesSnapshot = {
    ...state,
    coin: {
      ...state.coin,
      phaseStarted: env.revision > 0 || state.coin.phaseStarted,
    },
    theme: {
      ...state.theme,
      rzshTierRoman: numToRoma(Math.max(1, Math.min(10, env.view?.round ?? 1))),
    },
  };

  const seats = env.view?.seats ?? [];
  const seatGeneralName = (seatIdx: number): string => {
    const found = seats.find((s) => s.player === seatIdx);
    const fallbackGen = (found as unknown as { general?: string } | undefined)?.general;
    return found?.generals?.[0]?.name ?? fallbackGen ?? `seat_${seatIdx}`;
  };

  for (const seat of seats) {
    if (!seat.alive) {
      qhly = dispatchQhlyUiHook(qhly, {
        hook: "_qhcreateYH",
        seat: seat.player,
        faction: seat.generals?.[0]?.faction ?? "qun",
        isDead: true,
      });
    }
    if (!(seat.player in next.mvp.scores)) {
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

  const curPlayer = env.view?.current_player ?? 0;
  const curGen = seatGeneralName(curPlayer);
  const woundedSeat =
    seats.find((s) => s.player !== curPlayer && s.hp < s.max_hp)?.player ??
    seats.find((s) => s.player !== curPlayer)?.player ??
    curPlayer;
  const woundedGen = seatGeneralName(woundedSeat);

  const events = env.events ?? [];
  events.forEach((rawEv, idx) => {
    const seq = env.revision * 100 + idx + 1;
    const pubCode = (rawEv as { PublicEvent?: { code?: number } })?.PublicEvent?.code;
    const evType = (rawEv as { type?: string })?.type;
    if (pubCode === 4 || evType === "DamageDealt") {
      qhly = dispatchQhlyUiHook(qhly, { hook: "_qhlyChangeSkin", generalId: woundedGen, hpDelta: -1 });
      skinSwitch = dispatchSkinSwitchHook(skinSwitch, {
        hook: "_pfqh_check_damage",
        seat: woundedSeat,
        generalId: woundedGen,
        damageAmount: 1,
      });
      zuanUi = dispatchZuanUiHook(zuanUi, "_extradamageskill", { seat: woundedSeat });
      next = {
        ...next,
        mvp: recordMvpEventFact(next.mvp, { seq, seat: curPlayer, kind: "damage", amount: 1 }),
        longxue: advanceLongxueDaoxin(next.longxue, "damage_dealt", 1),
        theme: { ...next.theme, lastSkinTrigger: `damage@rev${env.revision}` },
      };
    } else if (pubCode === 5 || evType === "HpRecovered") {
      qhly = dispatchQhlyUiHook(qhly, { hook: "_qhlyChangeSkin", generalId: curGen, hpDelta: 1 });
      batch6Patch = dispatchL10EmojiRebate(batch6Patch, {
        trigger: "recover",
        fromSeat: curPlayer,
        toSeat: woundedSeat,
        isEnemy: false,
      });
      next = {
        ...next,
        mvp: recordMvpEventFact(next.mvp, { seq, seat: curPlayer, kind: "recover", amount: 1 }),
      };
    } else if (evType === "EnteredDying" || "EnteredDying" in (rawEv as Record<string, unknown>)) {
      const dyingBody = (rawEv as { EnteredDying?: { player?: number }; target?: number; player?: number });
      const dyingSeat = dyingBody.EnteredDying?.player ?? dyingBody.player ?? dyingBody.target ?? woundedSeat;
      zuanUi = dispatchZuanUiHook(zuanUi, "_saveskill", { seat: dyingSeat });
    } else if (pubCode === 2 || evType === "CardPlayed") {
      skinSwitch = dispatchSkinSwitchHook(skinSwitch, {
        hook: "_playAudioToQueue",
        seat: curPlayer,
        generalId: curGen,
        skinId: "primary_1",
        cardName: "sha",
        timestampMs: env.revision * 10 + idx,
      });
      shoushaUi = dispatchShoushaUiHook(shoushaUi, "_chupaiA", { seat: curPlayer });
      zuanUi = dispatchZuanUiHook(zuanUi, "_extrauseskill", { seat: curPlayer });
      if (batch6Patch.cardPhantom) {
        batch6Patch = { ...batch6Patch, lastPhantomLabel: `P${curPlayer} 幻影出牌 · rev${env.revision}` };
      }
      next = {
        ...next,
        mvp: recordMvpEventFact(next.mvp, { seq, seat: curPlayer, kind: "damage_card", amount: 1 }),
        longxue: advanceLongxueDaoxin(next.longxue, "gain_card", 1),
      };
    } else if (pubCode === 7 || evType === "PlayerDied") {
      qhly = dispatchQhlyUiHook(qhly, { hook: "_qhlyChageKillingSkin", killerGeneralId: curGen });
      skinSwitch = dispatchSkinSwitchHook(skinSwitch, {
        hook: "_pfqh_check_jisha",
        seat: curPlayer,
        generalId: curGen,
      });
      shoushaUi = dispatchShoushaUiHook(shoushaUi, "_jd_wjsw", { seat: woundedSeat });
      zuanUi = dispatchZuanUiHook(zuanUi, "_dieskill", { seat: woundedSeat });
      batch6Patch = dispatchL10EmojiRebate(batch6Patch, {
        trigger: "dying",
        fromSeat: curPlayer,
        toSeat: woundedSeat,
        isEnemy: true,
      });
    } else if (pubCode === 4 || evType === "SkillTriggered") {
      if (next.longxue.shengjian) {
        batch6Patch = dispatchLongxueCmEffects(batch6Patch, { daoxin: next.longxue.daoxin });
      }
    }
  });

  decadeUi = syncDecadeUiFromEnvelope(decadeUi, env);
  decadeUi = syncGuanXingChoice(decadeUi, env);
  shoushaUi = syncShoushaUiFromEnvelope(shoushaUi, env);
  l20Ui = syncL20MarkEffectsFromEnvelope(l20Ui, env);

  next = {
    ...next,
    uiSuites: {
      qhly,
      skinSwitch,
      shoushaUi,
      decadeUi,
      taixuUi,
      zuanUi,
      l20Ui,
      batch6Patch,
      batch7,
    },
  };

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
  const snap = parsed as ExtensionSurfacesSnapshot;
  if (snap.uiSuites?.decadeUi) {
    const baseDecade = createInitialDecadeUiRuntimeState();
    const rawDecade = snap.uiSuites.decadeUi;
    snap.uiSuites = {
      ...snap.uiSuites,
      decadeUi: {
        ...baseDecade,
        ...rawDecade,
        config: normalizeDecadeConfig(rawDecade.config),
        turnActionLogs: Array.isArray(rawDecade.turnActionLogs) ? rawDecade.turnActionLogs : [],
        pauseGuessBySeat:
          rawDecade.pauseGuessBySeat && typeof rawDecade.pauseGuessBySeat === "object"
            ? rawDecade.pauseGuessBySeat
            : {},
        selectedPauseFilterSeat:
          typeof rawDecade.selectedPauseFilterSeat === "number"
            ? rawDecade.selectedPauseFilterSeat
            : null,
      },
    };
  }
  if (snap.uiSuites?.shoushaUi) {
    const baseShousha = createInitialShoushaUiRuntimeState();
    const rawShousha = snap.uiSuites.shoushaUi;
    snap.uiSuites = {
      ...snap.uiSuites,
      shoushaUi: {
        ...baseShousha,
        ...rawShousha,
        config: normalizeShoushaConfig(rawShousha.config),
        seatPromptBySeat:
          rawShousha.seatPromptBySeat && typeof rawShousha.seatPromptBySeat === "object"
            ? rawShousha.seatPromptBySeat
            : {},
      },
    };
  }
  if (snap.uiSuites?.qhly) {
    const baseQhly = createInitialQhlyRuntimeState();
    const rawQhly = snap.uiSuites.qhly;
    snap.uiSuites = {
      ...snap.uiSuites,
      qhly: {
        ...baseQhly,
        ...rawQhly,
        config: normalizeQhlyConfig(rawQhly.config),
      },
    };
  }
  if (snap.uiSuites?.l20Ui) {
    const baseL20 = createInitialL20RuntimeState();
    snap.uiSuites = {
      ...snap.uiSuites,
      l20Ui: {
        ...baseL20,
        ...snap.uiSuites.l20Ui,
        configs: { ...baseL20.configs, ...(snap.uiSuites.l20Ui.configs ?? {}) },
      },
    };
  }
  if (snap.uiSuites?.skinSwitch) {
    const baseSs = createInitialSkinSwitchRuntimeState();
    snap.uiSuites = {
      ...snap.uiSuites,
      skinSwitch: {
        ...baseSs,
        ...snap.uiSuites.skinSwitch,
        config: normalizeSkinSwitchConfig(snap.uiSuites.skinSwitch.config),
      },
    };
  }
  if (snap.uiSuites?.zuanUi) {
    const baseZuan = createInitialZuanUiRuntimeState();
    snap.uiSuites = {
      ...snap.uiSuites,
      zuanUi: {
        ...baseZuan,
        ...snap.uiSuites.zuanUi,
        config: { ...createDefaultZuanConfig(), ...(snap.uiSuites.zuanUi.config ?? {}) },
      },
    };
  }
  if (snap.uiSuites?.batch6Patch) {
    snap.uiSuites = {
      ...snap.uiSuites,
      batch6Patch: {
        ...createInitialBatch6PatchUiState(),
        ...snap.uiSuites.batch6Patch,
      },
    };
  }
  if (snap.uiSuites?.batch7) {
    snap.uiSuites = {
      ...snap.uiSuites,
      batch7: normalizeBatch7UiSuiteState(snap.uiSuites.batch7),
    };
  }
  return snap;
}

/** 原子更新 L24 皮肤切换 14 项配置 */
export function updateExtensionSkinSwitchConfig(
  state: ExtensionSurfacesSnapshot,
  patch: Partial<SkinSwitchUiConfig>,
  opts?: { persist?: boolean },
): ExtensionSurfacesSnapshot {
  const prev = state.uiSuites?.skinSwitch ?? createInitialSkinSwitchRuntimeState();
  return {
    ...state,
    uiSuites: {
      ...state.uiSuites,
      skinSwitch: updateSkinSwitchConfigInState(prev, patch, opts),
    },
  };
}

/** 原子更新 L25 祖安武将 68 项配置 */
export function updateExtensionZuanConfig(
  state: ExtensionSurfacesSnapshot,
  patch: Record<string, unknown>,
): ExtensionSurfacesSnapshot {
  const prev = state.uiSuites?.zuanUi ?? createInitialZuanUiRuntimeState();
  return {
    ...state,
    uiSuites: {
      ...state.uiSuites,
      zuanUi: updateZuanConfigInState(prev, patch),
    },
  };
}

/** 原子更新 L27/L28/L29/L10 补齐状态 */
export function updateExtensionBatch6Patch(
  state: ExtensionSurfacesSnapshot,
  patch: Partial<Batch6PatchUiState>,
): ExtensionSurfacesSnapshot {
  const prev = state.uiSuites?.batch6Patch ?? createInitialBatch6PatchUiState();
  return {
    ...state,
    uiSuites: {
      ...state.uiSuites,
      batch6Patch: { ...prev, ...patch },
    },
  };
}

/** 原子更新 L09 千幻聆音 全部 82 项配置 */
export function updateExtensionQhlyConfig(
  state: ExtensionSurfacesSnapshot,
  patch: Partial<QhlyUiConfig>,
  opts?: { persist?: boolean; syncTheme?: boolean },
): ExtensionSurfacesSnapshot {
  const prevQhly = state.uiSuites?.qhly ?? createInitialQhlyRuntimeState();
  const nextQhly = updateQhlyConfigInState(prevQhly, patch, opts);
  const nextTheme = { ...state.theme };
  if (opts?.syncTheme !== false && patch.qhly_currentViewSkin !== undefined) {
    nextTheme.qhlyTheme =
      nextQhly.config.qhly_currentViewSkin === "decade"
        ? "decade"
        : nextQhly.config.qhly_currentViewSkin === "shousha"
          ? "shousha"
          : "classic";
  }
  return {
    ...state,
    theme: nextTheme,
    uiSuites: {
      ...state.uiSuites,
      qhly: nextQhly,
    },
  };
}

/** 原子更新 L20 标记补充 15 项配置 */
export function updateExtensionL20Config(
  state: ExtensionSurfacesSnapshot,
  patch: Partial<L20UiConfig>,
): ExtensionSurfacesSnapshot {
  const prevL20 = state.uiSuites?.l20Ui ?? createInitialL20RuntimeState();
  return {
    ...state,
    uiSuites: {
      ...state.uiSuites,
      l20Ui: {
        ...prevL20,
        configs: { ...prevL20.configs, ...patch },
      },
    },
  };
}

/** 原子更新 L14 手杀ui 全部 48 项配置并联动 `theme` 字段 */
export function updateExtensionShoushaConfig(
  state: ExtensionSurfacesSnapshot,
  patch: Partial<ShoushaUiConfig>,
  opts?: { persist?: boolean; mobileViewport?: boolean; bossMode?: boolean; syncTheme?: boolean },
): ExtensionSurfacesSnapshot {
  const prevShousha = state.uiSuites?.shoushaUi ?? createInitialShoushaUiRuntimeState();
  const nextShousha = updateShoushaConfigInState(prevShousha, patch, opts);
  const nextTheme = { ...state.theme };
  if (opts?.syncTheme !== false && patch.yangshi !== undefined) {
    nextTheme.decadeSkinStyle = nextShousha.config.yangshi === "on" ? "shousha" : "shizhounian";
  }
  return {
    ...state,
    theme: nextTheme,
    uiSuites: {
      ...state.uiSuites,
      shoushaUi: nextShousha,
    },
  };
}

/** 原子更新 L08 十周年UI 全部 44 项配置并联动 `theme` 字段 */
export function updateExtensionDecadeConfig(
  state: ExtensionSurfacesSnapshot,
  patch: Partial<DecadeUiConfig>,
  opts?: { persist?: boolean; mobileViewport?: boolean },
): ExtensionSurfacesSnapshot {
  const prevDecade = state.uiSuites?.decadeUi ?? createInitialDecadeUiRuntimeState();
  const nextDecade = updateDecadeConfigInState(prevDecade, patch, opts);
  const nextTheme = { ...state.theme };
  if (patch.rightLayout !== undefined) {
    nextTheme.shoushaLayout = nextDecade.config.rightLayout ? "right" : "left";
  }
  if (patch.newDecadeStyle !== undefined) {
    nextTheme.decadeSkinStyle = nextDecade.config.newDecadeStyle === "on" ? "shizhounian" : "shousha";
  }
  if (patch.playerMarkStyle !== undefined) {
    nextTheme.markStyle = nextDecade.config.playerMarkStyle === "decade" ? "decade" : "shousha";
  }
  return {
    ...state,
    theme: nextTheme,
    uiSuites: {
      ...state.uiSuites,
      decadeUi: nextDecade,
    },
  };
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

/**
 * U04 / L05 (`archive/upstream/noname/apps/core/extension/coin/extension.js:24-398`):
 * Renders `haveFun.firework`, `haveFun.snow`, `haveFun.star`, and `haveFun.blink`
 * particles onto a 2D `<canvas>` context.
 */
export function renderCoinCanvasParticles(
  ctx: CanvasRenderingContext2D,
  coin: CoinSurfaceState,
  frame = 0,
): { renderedParticles: number; activeModes: string[] } {
  const width = ctx.canvas?.width ?? 240;
  const height = ctx.canvas?.height ?? 48;
  ctx.clearRect(0, 0, width, height);
  const modes = new Set<string>(coin.activeEffects);
  if (coin.snowFall) modes.add("snow");
  let count = 0;

  if (modes.has("snow")) {
    const rBase = coin.snowSize === "large" ? 2.6 : 1.8;
    ctx.fillStyle = "rgba(255, 255, 255, 0.48)";
    for (let i = 0; i < 12; i++) {
      // Deterministic golden-ratio scatter across the full [0, width) x [0, height) tabletop
      const h1 = ((Math.imul(i + 1, 2654435761) >>> 0) % 10000) / 10000;
      const h2 = ((Math.imul(i + 7, 2246822519) >>> 0) % 10000) / 10000;
      const driftX = frame * (0.55 + (i % 3) * 0.2) + Math.sin((frame + i * 19) * 0.04) * 16;
      const fallY = frame * (0.85 + (i % 4) * 0.25);
      const x = (((h1 * width + driftX) % width) + width) % width;
      const y = (((h2 * height + fallY) % height) + height) % height;
      ctx.beginPath();
      ctx.arc(x, y, rBase + (i % 2) * 0.5, 0, Math.PI * 2);
      ctx.fill();
      count++;
    }
  }
  if (modes.has("fireworks")) {
    ctx.strokeStyle = "rgba(255, 190, 64, 0.9)";
    const cx = width * 0.5;
    const cy = height * 0.5;
    for (let ray = 0; ray < 8; ray++) {
      const ang = (ray * Math.PI) / 4 + frame * 0.05;
      ctx.beginPath();
      ctx.moveTo(cx, cy);
      ctx.lineTo(cx + Math.cos(ang) * 14, cy + Math.sin(ang) * 14);
      ctx.stroke();
      count++;
    }
  }
  if (modes.has("nebula")) {
    ctx.fillStyle = "rgba(128, 170, 255, 0.45)";
    ctx.beginPath();
    ctx.arc(width * 0.3, height * 0.5, 12, 0, Math.PI * 2);
    ctx.fill();
    count++;
  }
  if (modes.has("twinkle")) {
    ctx.fillStyle = "rgba(255, 248, 170, 0.9)";
    for (let i = 0; i < 6; i++) {
      const sx = ((i * 37 + 15) % width);
      const sy = ((i * 17 + 9) % height);
      ctx.fillRect(sx, sy, 2, 2);
      count++;
    }
  }
  return { renderedParticles: count, activeModes: Array.from(modes) };
}

/**
 * L07 (`archive/upstream/noname-liuli/noname/extension/全能搜索/SimplePagination.js:1-147`
 * & `extension.js:1-2354`): Paginates omnisearch entries and produces highlighted segments.
 */
export function paginateOmnisearchResults(
  entries: ReadonlyArray<OmnisearchEntry>,
  page = 1,
  pageSize = 5,
): { items: OmnisearchEntry[]; page: number; totalPages: number } {
  const safeSize = Math.max(1, pageSize);
  const totalPages = Math.max(1, Math.ceil(entries.length / safeSize));
  const safePage = Math.min(totalPages, Math.max(1, page));
  const start = (safePage - 1) * safeSize;
  return {
    items: entries.slice(start, start + safeSize),
    page: safePage,
    totalPages,
  };
}

export function highlightOmnisearchTerm(
  text: string,
  query: string,
): Array<{ text: string; matched: boolean }> {
  const q = query.trim();
  if (!q) return [{ text, matched: false }];
  const lowerText = text.toLowerCase();
  const lowerQ = q.toLowerCase();
  const idx = lowerText.indexOf(lowerQ);
  if (idx < 0) return [{ text, matched: false }];
  const out: Array<{ text: string; matched: boolean }> = [];
  if (idx > 0) out.push({ text: text.slice(0, idx), matched: false });
  out.push({ text: text.slice(idx, idx + q.length), matched: true });
  if (idx + q.length < text.length) {
    out.push({ text: text.slice(idx + q.length), matched: false });
  }
  return out;
}

/**
 * U04 / L05 (`archive/upstream/noname/apps/core/extension/coin/extension.js:68-140` &
 * `archive/upstream/noname-liuli/noname/extension/富甲天下/extension.js:68-140`):
 * Implements `_buy` drag-purchase of shop canvas effects (`snow`, `fireworks`, `nebula`, `twinkle`).
 */
export function dragBuyCoinShopItem(
  coin: CoinSurfaceState,
  item: "snow" | "fireworks" | "nebula" | "twinkle",
  cost = 20,
): CoinSurfaceState {
  const validCost = Math.max(0, Math.floor(cost));
  if (coin.balance < validCost) {
    throw new Error("INSUFFICIENT_COINS_FOR_BUY");
  }
  const activeEffects = coin.activeEffects.includes(item)
    ? coin.activeEffects
    : [...coin.activeEffects, item];
  return {
    ...coin,
    balance: coin.balance - validCost,
    snowFall: item === "snow" ? true : coin.snowFall,
    activeEffects,
  };
}

/**
 * L10 (`archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js:34553`):
 * Implements `_emojiRebate1` throw-emotion rebate interaction.
 */
export function triggerL10EmojiRebate(
  theme: ThemeAndSkinSurfaceState,
  targetSeat: number,
  emotion = "flower",
): { theme: ThemeAndSkinSurfaceState; rebatedEmotion: string } {
  const rebatedEmotion = `rebate:${emotion}->seat${targetSeat}`;
  return {
    theme: {
      ...theme,
      throwEmotionCue: rebatedEmotion,
    },
    rebatedEmotion,
  };
}

/**
 * L10 (`archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js:34517`):
 * Implements `scs_die_audio` Ten Regular Attendants death voice cue.
 */
export function triggerL10ScsDieAudio(
  theme: ThemeAndSkinSurfaceState,
  deadGeneralId: string,
): ThemeAndSkinSurfaceState {
  const cleanId = deadGeneralId.trim() || "shichangshi";
  return {
    ...theme,
    lastVoiceCue: `die/${cleanId}.mp3`,
  };
}

/**
 * L12 (`archive/upstream/noname-liuli/noname/extension/如真似幻/extension.js:510-2100`):
 * Applies dynamic skin lighting filter and Roman tier badge (`numtoroma`).
 */
export function applyRzshDynamicSkinLighting(
  theme: ThemeAndSkinSurfaceState,
  tier = 1,
): { theme: ThemeAndSkinSurfaceState; cssFilter: string; badgeRoman: string } {
  const badgeRoman = numToRoma(Math.max(1, Math.min(10, tier)));
  const cssFilter =
    theme.rzshTheme === "illusion"
      ? "drop-shadow(0 0 8px rgba(148, 108, 255, 0.75)) saturate(1.18)"
      : "drop-shadow(0 0 6px rgba(255, 200, 96, 0.65)) contrast(1.08)";
  return {
    theme: {
      ...theme,
      rzshTierRoman: badgeRoman,
    },
    cssFilter,
    badgeRoman,
  };
}

/**
 * L20 (`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:40-90`):
 * Switches HUD mark CSS stylesheet (`decade` / `shousha` / `classic`).
 */
export function applyL20MarkCssTheme(
  theme: ThemeAndSkinSurfaceState,
  style: "decade" | "shousha" | "classic",
): { theme: ThemeAndSkinSurfaceState; stylesheet: string } {
  return {
    theme: {
      ...theme,
      markStyle: style,
    },
    stylesheet: `extension/标记补充/${style}.css`,
  };
}

/**
 * L26 (`archive/upstream/noname-liuli/noname/extension/笨蛋插件/extension.js:18-62` & `style.css:1-45`):
 * Computes card hover hint badge and CSS class for Ziyuan / Fangtong / Qiangzhong.
 */
export function applyFoolCardHoverTheme(
  workbench: WorkbenchToolsState,
  skillId: "ziyuan" | "fangtong" | "qiangzhong",
): { workbench: WorkbenchToolsState; hoverBadge: string; cssClass: string } {
  const hoverBadge =
    skillId === "ziyuan"
      ? "资援≤13"
      : skillId === "fangtong"
        ? "方统=36"
        : "强中加伤";
  return {
    workbench: {
      ...workbench,
      foolHintLabel: hoverBadge,
    },
    hoverBadge,
    cssClass: `fool-hint-${skillId}`,
  };
}

/**
 * L27 (`archive/upstream/noname-liuli/noname/extension/选将美化/extension.js:42-95` & `style.css:1-120`):
 * Toggles character selection grid beautification CSS class.
 */
export function applyL27SelectBeautifyTheme(
  theme: ThemeAndSkinSurfaceState,
  enabled: boolean,
): { theme: ThemeAndSkinSurfaceState; cssClass: string } {
  return {
    theme: {
      ...theme,
      selectBeautifyEnabled: enabled,
    },
    cssClass: enabled ? "l27-select-beautify-active" : "l27-select-beautify-classic",
  };
}

/**
 * L29 (`archive/upstream/noname-liuli/noname/extension/龙血玄黄/extension.js:68-140`):
 * Implements `_CMeffects` holy-sword visual effect toggle (`game.shengjian`).
 */
export function triggerLongxueHolySwordEffect(
  longxue: LongxueSurfaceState,
  active: boolean,
): { longxue: LongxueSurfaceState; effectCue: string } {
  return {
    longxue: {
      ...longxue,
      shengjian: active,
    },
    effectCue: active ? "longxue:shengjian:on" : "longxue:shengjian:off",
  };
}

export * from "./surfaces/decadeConfigSchema";
export * from "./surfaces/decadeHooksRuntime";
export * from "./surfaces/decadeMarkSkillsData";
export * from "./surfaces/decadeSurfacesRuntime";
export * from "./surfaces/decadeUiSurface";
export * from "./surfaces/l20MarkSupplementRuntime";
export * from "./surfaces/qhlyConfigSchema";
export * from "./surfaces/qhlySurface";
export * from "./surfaces/shoushaConfigSchema";
export * from "./surfaces/shoushaSurfacesRuntime";
export * from "./surfaces/shoushaUiSurface";
export * from "./surfaces/skinSwitchConfigSchema";
export * from "./surfaces/skinSwitchSurface";
export * from "./surfaces/taixuSurface";
export * from "./surfaces/zuanConfigSchema";
export * from "./surfaces/zuanSurface";
export * from "./surfaces/batch6PatchSurfacesRuntime";
export * from "./surfaces/aiOptimizeConfigSchema";
export * from "./surfaces/aiOptimizeSurface";
export * from "./surfaces/toolsAndPoolSchema";
export * from "./surfaces/toolsAndPoolSurface";
export * from "./surfaces/visualEffectsBatch7Runtime";
export * from "./surfaces/batch7ClosureState";
