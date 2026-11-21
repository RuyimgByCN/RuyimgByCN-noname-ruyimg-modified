/**
 * L08 十周年UI 原生前端渲染面与全部 319 项 UI 钩子运行时
 * 上游对照:
 * - `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:54-7630` (9 项主 HUD 钩子)
 * - `archive/upstream/noname-liuli/noname/extension/十周年UI/animation.js:1402-1710` & `animation_new.js:1402-1660` (11 项动画钩子)
 * - `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:1-20770` (286 项标记/角标/阴阳转换技钩子)
 * - `archive/upstream/noname-liuli/noname/extension/十周年UI/meihua.js:3336-6950` (5 项重铸/虚拟装备/手牌数/破袭钩子)
 * - `archive/upstream/noname-liuli/noname/extension/十周年UI/splash.js:5-1040` (8 项国战隐匿/双将立绘开场钩子)
 * - 43 项渲染面 (`animation`, `animation_new`, `animationlist`, `component`, `content`, `effect`, `meihua`, `menu`, `pause`, `splash` 及 33 份 CSS)
 */

import {
  DECADE_MARKSKILL_SPECS,
  getDecadeMarkBehaviorMeta,
  resolveDecadeSubMarkSpec,
  sortCardTypesDecadeSymbols,
  type DecadeMarkSkillSpec,
} from "./decadeMarkSkillsData";
import {
  applyDecadeConfigUpdate,
  createDefaultDecadeConfig,
  loadPersistedDecadeConfig,
  persistDecadeConfig,
  type DecadeUiConfig,
} from "./decadeConfigSchema";
import {
  createInitialChatBoxState,
  resolveActiveDecadeCssSurfaces,
  resolveGameStartBanner,
  type DecadeChatBoxState,
  type DecadeCompareEffectState,
  type DecadeGuanXingState,
  type DecadeLineArcState,
  type DecadeSkillPopupState,
} from "./decadeSurfacesRuntime";

export const DECADE_CORE_HOOK_IDS = [
  // animation.js & animation_new.js (11)
  "_jiuwo_",
  "_delete_jiuwo_",
  "_dyjinengtx",
  "_losetl_",
  "_jinnangpai_",
  "_jiubuff_",
  "_jiubuff_xiaoshi_",
  "_player_hp",
  "_HpRecover",
  "_Hplose",
  "_killFire",
  // extension.js (9)
  "_changeEquipExt",
  "_ui_auto_cheat",
  "_kill_framebg",
  "_rarity",
  "_tiesuo_ying",
  "_usecard",
  "_decadeUI_usecardBegin",
  "_discard",
  "_decadeUI_dieKillEffect",
  // meihua.js (5)
  "_recasting",
  "_xunizhuangbei_gua_gua",
  "_gshoupai",
  "drlt_poxi",
  "pro_drlt_poxi",
  // splash.js (8)
  "_gzyinni",
  "_guozhan",
  "_gzyinniszn",
  "_guozhanszn",
  "_yinni",
  "_shousha_shuangjiang",
  "_shizhounian_shuangjiang",
  "_shizhounian_shuangjiang2",
] as const;

export const DECADE_ALL_UI_HOOK_IDS: ReadonlyArray<string> = [
  ...DECADE_CORE_HOOK_IDS,
  ...DECADE_MARKSKILL_SPECS.map((s) => s.id),
];

export const DECADE_SURFACE_MODULES = [
  "animation",
  "animation_new",
  "animationlist",
  "component",
  "content",
  "effect",
  "meihua",
  "menu",
  "pause",
  "splash",
  "animation.css",
  "animation1.css",
  "animation_new.css",
  "card.css",
  "cardX.css",
  "component.css",
  "decadeLayout.css",
  "dialog.css",
  "dialog_new.css",
  "doudizhu.css",
  "effect.css",
  "equip.css",
  "equip_new.css",
  "font.css",
  "icon.css",
  "junba.css",
  "layout.css",
  "layout_new.css",
  "meihua.css",
  "newdialog.css",
  "newdialog_new.css",
  "pause.css",
  "player.css",
  "player1.css",
  "player1_PC.css",
  "player2.css",
  "player2_PC.css",
  "player3.css",
  "player3_PC.css",
  "player_mark.css",
  "player_mark_fix.css",
  "player_new.css",
  "splash.css",
] as const;

const MARKSKILL_INDEX: Readonly<Record<string, DecadeMarkSkillSpec>> = Object.fromEntries(
  DECADE_MARKSKILL_SPECS.map((s) => [s.id, s]),
);

const SUIT_SYMBOLS: Readonly<Record<string, string>> = {
  spade: "♠",
  heart: "♥",
  club: "♣",
  diamond: "♦",
};

const SUIT_ORDER: Readonly<Record<string, number>> = {
  spade: 0,
  heart: 1,
  club: 2,
  diamond: 3,
};

/**
 * 上游 `十周年UI/markskill.js:48-72` (`Array.prototype.sortBySuit` + `lib.suitsymbols`)
 */
export function sortSuitsDecadeSymbols(suits: ReadonlyArray<string>): string {
  return [...suits]
    .filter((s) => s in SUIT_ORDER)
    .sort((a, b) => (SUIT_ORDER[a] ?? 9) - (SUIT_ORDER[b] ?? 9))
    .map((s) => SUIT_SYMBOLS[s] ?? s)
    .join("");
}

export interface DecadeMarkBadgeView {
  skillId: string;
  label: string;
  badgeText: string;
  tooltip: string;
  upstreamRef: string;
  nonumberMark?: boolean;
  noskillControl?: boolean;
}

/**
 * 按 `markskill.js` 格式化十周年角色标记角标与悬浮说明 (`markskill.js:18-20741`)
 */
export function formatDecadeSkillMarkBadge(
  skillId: string,
  opts?: {
    count?: number;
    yang?: boolean;
    suits?: ReadonlyArray<string>;
    cardTypes?: ReadonlyArray<string>;
    usedLimited?: boolean;
  },
): DecadeMarkBadgeView | null {
  const spec = MARKSKILL_INDEX[skillId] ?? resolveDecadeSubMarkSpec(skillId);
  if (!spec) return null;
  const meta = getDecadeMarkBehaviorMeta(spec);
  let badgeText = meta.nonumberMark ? "态" : String(opts?.count ?? 1);
  if (spec.kind === "zhuanhuan") {
    badgeText = opts?.yang ? "阳" : "阴";
  } else if (spec.kind === "limited") {
    badgeText = opts?.usedLimited ? "✕" : "限";
  } else if (meta.usesTypeSort && opts?.cardTypes && opts.cardTypes.length > 0) {
    const typeText = sortCardTypesDecadeSymbols(opts.cardTypes);
    const suitText = opts?.suits?.length ? sortSuitsDecadeSymbols(opts.suits) : "";
    badgeText = `${typeText}${suitText}`;
  } else if (spec.kind === "suit" && opts?.suits && opts.suits.length > 0) {
    badgeText = sortSuitsDecadeSymbols(opts.suits);
  }
  return {
    skillId: spec.id,
    label: spec.label,
    badgeText,
    tooltip: meta.tooltipSummary,
    upstreamRef: `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:${spec.line}`,
    nonumberMark: meta.nonumberMark,
    noskillControl: meta.noskillControl,
  };
}

export interface DecadeTrickEffectBanner {
  readonly card: "taoyuan" | "nanman" | "wanjian";
  readonly title: string;
}

export interface DecadeHpDeltaView {
  readonly delta: number;
  readonly kind: "damage" | "recover";
}

export interface DecadePoxiDialogState {
  readonly mode: "drlt_poxi" | "pro_drlt_poxi";
  readonly targetSeat: number;
  readonly selectedIds: ReadonlyArray<number>;
}

export interface DecadeUiRuntimeState {
  layoutVariant: "decadeLayout.css" | "layout_new.css" | "junba.css" | "doudizhu.css";
  playerCssVariant:
    | "player_new.css"
    | "player1.css"
    | "player1_PC.css"
    | "player2.css"
    | "player2_PC.css"
    | "player3.css"
    | "player3_PC.css";
  pauseHistoryOpen: boolean;
  turnActionLogs: ReadonlyArray<{ seq: number; seat: number; text: string }>;
  pauseGuessBySeat: Record<number, string>;
  selectedPauseFilterSeat: number | null;
  poxiModalTargetSeat: number | null;
  poxiDialog: DecadePoxiDialogState | null;
  jiuBuffSeats: number[];
  jiuwoDyingSeats: number[];
  tiesuoSeats: number[];
  loseHpSeats: number[];
  damageFlashSeats: number[];
  killFireSeats: number[];
  hiddenSeats: number[];
  publicHandSeats: number[];
  factionGlowBySeat: Record<number, string>;
  hpDeltaBySeat: Record<number, DecadeHpDeltaView>;
  equipHiddenBySeat: Record<number, boolean>;
  trickEffectBanner: DecadeTrickEffectBanner | null;
  autoCheatNotice: string | null;
  prevHpBySeat: Record<number, number>;
  prevSeatMaxHpBySeat: Record<number, number>;
  prevSeatHadGeneralBySeat: Record<number, boolean>;
  lastSyncedRevision?: number;
  lastSyncedRound?: number;
  lastSyncedPhase?: string;
  lastSyncedPlayer?: number;
  virtualEquipsBySeat: Record<number, string[]>;
  handCountBadgeBySeat: Record<number, number>;
  rarityTierBySeat: Record<number, "junk" | "common" | "rare" | "epic" | "legend">;
  splashMode: "standard" | "guozhan_hidden" | "shousha_dual" | "decade_dual";
  lastEffectCue: string | null;
  activeMarkBadgesBySeat: Record<number, DecadeMarkBadgeView[]>;
  config: DecadeUiConfig;
  activeCssSurfaces: ReadonlyArray<string>;
  chatBox: DecadeChatBoxState;
  guanXing: DecadeGuanXingState | null;
  compareEffect: DecadeCompareEffectState | null;
  lineArc: DecadeLineArcState | null;
  skillPopup: DecadeSkillPopupState | null;
  killOverlay: { killerName: string; victimName: string } | null;
  startBanner: string | null;
  healAuraSeats: number[];
  cardUseCueBySeat: Record<number, string>;
  recastCountBySeat: Record<number, number>;
  sliderSpeedPct: number;
  importedCardPackName: string | null;
}

export function createInitialDecadeUiRuntimeState(
  opts?: { usePersistedConfig?: boolean },
): DecadeUiRuntimeState {
  const initialMark = formatDecadeSkillMarkBadge("nzry_chenglve", { yang: false, suits: ["spade", "heart"] });
  const config = opts?.usePersistedConfig
    ? loadPersistedDecadeConfig()
    : createDefaultDecadeConfig();
  return {
    layoutVariant: config.decadeLayout === "on" ? "decadeLayout.css" : "layout_new.css",
    playerCssVariant: config.newDecadeStyle === "on" ? "player_new.css" : "player1_PC.css",
    pauseHistoryOpen: false,
    turnActionLogs: [],
    pauseGuessBySeat: {},
    selectedPauseFilterSeat: null,
    poxiModalTargetSeat: null,
    poxiDialog: null,
    jiuBuffSeats: [],
    jiuwoDyingSeats: [],
    tiesuoSeats: [],
    loseHpSeats: [],
    damageFlashSeats: [],
    killFireSeats: [],
    hiddenSeats: [],
    publicHandSeats: [],
    factionGlowBySeat: {},
    hpDeltaBySeat: {},
    equipHiddenBySeat: {},
    trickEffectBanner: null,
    autoCheatNotice: null,
    prevHpBySeat: {},
    prevSeatMaxHpBySeat: {},
    prevSeatHadGeneralBySeat: {},
    lastSyncedRevision: undefined,
    lastSyncedRound: undefined,
    lastSyncedPhase: undefined,
    lastSyncedPlayer: undefined,
    virtualEquipsBySeat: {},
    handCountBadgeBySeat: { 0: 4, 1: 4 },
    rarityTierBySeat: { 0: "legend", 1: "epic" },
    splashMode: "standard",
    lastEffectCue: null,
    activeMarkBadgesBySeat: initialMark ? { 0: [initialMark] } : {},
    config,
    activeCssSurfaces: resolveActiveDecadeCssSurfaces(config),
    chatBox: createInitialChatBoxState(),
    guanXing: null,
    compareEffect: null,
    lineArc: null,
    skillPopup: null,
    killOverlay: null,
    startBanner: resolveGameStartBanner(config.SanGuoShaGameStart),
    healAuraSeats: [],
    cardUseCueBySeat: {},
    recastCountBySeat: {},
    sliderSpeedPct: 50,
    importedCardPackName: null,
  };
}

export function updateDecadeConfigInState(
  state: DecadeUiRuntimeState,
  patch: Partial<DecadeUiConfig>,
  opts?: { persist?: boolean; mobileViewport?: boolean },
): DecadeUiRuntimeState {
  const nextCfg = applyDecadeConfigUpdate(state.config, patch);
  if (opts?.persist !== false) persistDecadeConfig(nextCfg);
  const nextPlayerCssVariant: DecadeUiRuntimeState["playerCssVariant"] =
    patch.newDecadeStyle !== undefined
      ? nextCfg.newDecadeStyle === "on"
        ? "player_new.css"
        : opts?.mobileViewport
          ? "player1.css"
          : "player1_PC.css"
      : state.playerCssVariant;
  return {
    ...state,
    config: nextCfg,
    layoutVariant: nextCfg.decadeLayout === "on" ? "decadeLayout.css" : "layout_new.css",
    playerCssVariant: nextPlayerCssVariant,
    activeCssSurfaces: resolveActiveDecadeCssSurfaces(nextCfg, {
      mobileViewport: opts?.mobileViewport,
      playerCssVariant: nextPlayerCssVariant,
    }),
    startBanner:
      patch.SanGuoShaGameStart !== undefined
        ? resolveGameStartBanner(nextCfg.SanGuoShaGameStart)
        : state.startBanner,
  };
}

export function updateDecadePlayerCssVariant(
  state: DecadeUiRuntimeState,
  variant: DecadeUiRuntimeState["playerCssVariant"],
  opts?: { mobileViewport?: boolean },
): DecadeUiRuntimeState {
  return {
    ...state,
    playerCssVariant: variant,
    activeCssSurfaces: resolveActiveDecadeCssSurfaces(state.config, {
      mobileViewport: opts?.mobileViewport,
      playerCssVariant: variant,
    }),
  };
}

export function assignPauseGuessInState(
  state: DecadeUiRuntimeState,
  seat: number,
  guess: string,
): DecadeUiRuntimeState {
  const nextMap = { ...state.pauseGuessBySeat };
  const clean = guess.trim().slice(0, 8);
  if (!clean) {
    delete nextMap[seat];
  } else {
    nextMap[seat] = clean;
  }
  return { ...state, pauseGuessBySeat: nextMap };
}

export function dispatchDecadeUiHook(
  state: DecadeUiRuntimeState,
  hookId: string,
  opts?: {
    seat?: number;
    count?: number;
    yang?: boolean;
    suits?: ReadonlyArray<string>;
    cardTypes?: ReadonlyArray<string>;
    usedLimited?: boolean;
    equipName?: string;
    handCount?: number;
    skillName?: string;
    generalName?: string;
    killerName?: string;
    victimName?: string;
    faction?: string;
    cardName?: "taoyuan" | "nanman" | "wanjian";
    delta?: number;
    hpKind?: "damage" | "recover";
    rarity?: "junk" | "common" | "rare" | "epic" | "legend";
    hideEquips?: boolean;
    publicVisible?: boolean;
    revealed?: boolean;
    clear?: boolean;
  },
): DecadeUiRuntimeState {
  const seat = opts?.seat ?? 0;

  switch (hookId) {
    case "_jiuwo_":
      return {
        ...state,
        jiuwoDyingSeats:
          state.config.JNTX && state.config.gameAnimationEffect
            ? [...new Set([...state.jiuwoDyingSeats, seat])]
            : state.jiuwoDyingSeats,
        lastEffectCue: "SZN_jiuwo",
      };
    case "_delete_jiuwo_":
      return {
        ...state,
        jiuwoDyingSeats: state.jiuwoDyingSeats.filter((s) => s !== seat),
      };
    case "_jiubuff_":
      return {
        ...state,
        jiuBuffSeats: [...new Set([...state.jiuBuffSeats, seat])],
        lastEffectCue: "jiubuff",
      };
    case "_jiubuff_xiaoshi_":
      return {
        ...state,
        jiuBuffSeats: state.jiuBuffSeats.filter((s) => s !== seat),
      };
    case "_dyjinengtx":
      return {
        ...state,
        factionGlowBySeat:
          state.config.gameAnimationEffect && state.config.JNTX
            ? { ...state.factionGlowBySeat, [seat]: opts?.faction ?? "qun" }
            : state.factionGlowBySeat,
        skillPopup:
          state.config.gameAnimationEffect && state.config.JNTX
            ? {
                seat,
                generalName: opts?.generalName ?? `P${seat}`,
                skillName: opts?.skillName ?? opts?.equipName ?? "技能发动",
                tier: "standard",
              }
            : state.skillPopup,
        lastEffectCue: `${hookId}@P${seat}`,
      };
    case "_HpRecover": {
      const mag = Math.abs(opts?.delta ?? opts?.count ?? 1);
      return {
        ...state,
        healAuraSeats: state.config.EffectWood
          ? [...new Set([...state.healAuraSeats, seat])]
          : state.healAuraSeats,
        hpDeltaBySeat: {
          ...state.hpDeltaBySeat,
          [seat]: { delta: mag, kind: "recover" },
        },
        lastEffectCue: `${hookId}@P${seat}`,
      };
    }
    case "_Hplose": {
      const mag = Math.abs(opts?.delta ?? opts?.count ?? 1);
      return {
        ...state,
        damageFlashSeats: [...new Set([...state.damageFlashSeats, seat])],
        hpDeltaBySeat: {
          ...state.hpDeltaBySeat,
          [seat]: { delta: -mag, kind: "damage" },
        },
        lastEffectCue: `${hookId}@P${seat}`,
      };
    }
    case "_losetl_": {
      const mag = Math.abs(opts?.delta ?? opts?.count ?? 1);
      return {
        ...state,
        loseHpSeats: [...new Set([...state.loseHpSeats, seat])],
        hpDeltaBySeat: {
          ...state.hpDeltaBySeat,
          [seat]: { delta: -mag, kind: "damage" },
        },
        lastEffectCue: `${hookId}@P${seat}`,
      };
    }
    case "_jinnangpai_": {
      const card = opts?.cardName ?? "nanman";
      const title =
        card === "taoyuan"
          ? "桃园结义 · 群雄共愈"
          : card === "wanjian"
            ? "万箭齐发 · 箭雨蔽日"
            : "南蛮入侵 · 战鼓齐鸣";
      return {
        ...state,
        trickEffectBanner:
          state.config.JNTX && state.config.gameAnimationEffect
            ? { card, title }
            : state.trickEffectBanner,
        lastEffectCue: `${hookId}:${card}`,
      };
    }
    case "_player_hp": {
      const raw = opts?.delta ?? opts?.count ?? 1;
      const hpKind = opts?.hpKind ?? (raw > 0 ? "recover" : "damage");
      const signedDelta = hpKind === "damage" ? -Math.abs(raw) : Math.abs(raw);
      return {
        ...state,
        hpDeltaBySeat: {
          ...state.hpDeltaBySeat,
          [seat]: {
            delta: signedDelta,
            kind: hpKind,
          },
        },
        lastEffectCue: `${hookId}@P${seat}`,
      };
    }
    case "_killFire":
      return {
        ...state,
        killFireSeats: opts?.clear
          ? state.killFireSeats.filter((s) => s !== seat)
          : [...new Set([...state.killFireSeats, seat])],
        lastEffectCue: `${hookId}@P${seat}`,
      };
    case "_usecard":
    case "_decadeUI_usecardBegin": {
      if (opts?.clear) {
        const nextCues = { ...state.cardUseCueBySeat };
        delete nextCues[seat];
        return { ...state, cardUseCueBySeat: nextCues, lastEffectCue: `${hookId}@P${seat}` };
      }
      return {
        ...state,
        cardUseCueBySeat: state.config.cardUseEffect
          ? { ...state.cardUseCueBySeat, [seat]: opts?.equipName ?? "出牌光效" }
          : state.cardUseCueBySeat,
        lastEffectCue: `${hookId}@P${seat}`,
      };
    }
    case "_discard": {
      const nextCues = { ...state.cardUseCueBySeat };
      delete nextCues[seat];
      return {
        ...state,
        cardUseCueBySeat: nextCues,
        damageFlashSeats: state.damageFlashSeats.filter((s) => s !== seat),
        loseHpSeats: state.loseHpSeats.filter((s) => s !== seat),
        lastEffectCue: `${hookId}@P${seat}`,
      };
    }
    case "_decadeUI_dieKillEffect":
      return {
        ...state,
        jiuBuffSeats: state.jiuBuffSeats.filter((s) => s !== seat),
        jiuwoDyingSeats: state.jiuwoDyingSeats.filter((s) => s !== seat),
        killOverlay:
          state.config.playerKillEffect && state.config.gameAnimationEffect
            ? {
                killerName: opts?.killerName ?? `P${seat}`,
                victimName: opts?.victimName ?? "敌将",
              }
            : state.killOverlay,
        lastEffectCue: `${hookId}@P${seat}`,
      };
    case "_recasting":
      return {
        ...state,
        recastCountBySeat: {
          ...state.recastCountBySeat,
          [seat]: (state.recastCountBySeat[seat] ?? 0) + 1,
        },
        lastEffectCue: `${hookId}@P${seat}`,
      };
    case "_changeEquipExt":
      return {
        ...state,
        equipHiddenBySeat: {
          ...state.equipHiddenBySeat,
          [seat]: opts?.hideEquips ?? !state.equipHiddenBySeat[seat],
        },
        lastEffectCue: `${hookId}@P${seat}`,
      };
    case "_xunizhuangbei_gua_gua": {
      const eq = opts?.equipName ?? "bagua";
      const prev = state.virtualEquipsBySeat[seat] ?? [];
      return {
        ...state,
        virtualEquipsBySeat: {
          ...state.virtualEquipsBySeat,
          [seat]: prev.includes(eq) ? prev : [...prev, eq],
        },
        lastEffectCue: `${hookId}:${eq}`,
      };
    }
    case "_ui_auto_cheat":
      return {
        ...state,
        autoCheatNotice: state.config._connectMode
          ? null
          : "单机调试 · _ui_auto_cheat 即时指令就绪",
        lastEffectCue: "decade_inspect_overlay",
      };
    case "_kill_framebg":
      return {
        ...state,
        rarityTierBySeat: { ...state.rarityTierBySeat, [seat]: opts?.rarity ?? "epic" },
        lastEffectCue: `${hookId}@P${seat}`,
      };
    case "_rarity":
      return {
        ...state,
        rarityTierBySeat: { ...state.rarityTierBySeat, [seat]: opts?.rarity ?? "legend" },
      };
    case "_tiesuo_ying":
      return {
        ...state,
        tiesuoSeats: opts?.clear
          ? state.tiesuoSeats.filter((s) => s !== seat)
          : [...new Set([...state.tiesuoSeats, seat])],
        lastEffectCue: "tiesuo_chain",
      };
    case "_gshoupai":
      return {
        ...state,
        handCountBadgeBySeat: {
          ...state.handCountBadgeBySeat,
          [seat]: opts?.handCount ?? (state.handCountBadgeBySeat[seat] ?? 4),
        },
        publicHandSeats:
          opts?.publicVisible === false
            ? state.publicHandSeats.filter((s) => s !== seat)
            : [...new Set([...state.publicHandSeats, seat])],
      };
    case "drlt_poxi":
    case "pro_drlt_poxi":
      return {
        ...state,
        poxiModalTargetSeat: seat,
        poxiDialog: { mode: hookId, targetSeat: seat, selectedIds: [] },
        lastEffectCue: hookId,
      };
    case "_gzyinni":
    case "_guozhan":
    case "_gzyinniszn":
    case "_guozhanszn":
    case "_yinni":
      return {
        ...state,
        splashMode: "guozhan_hidden",
        hiddenSeats: opts?.revealed
          ? state.hiddenSeats.filter((s) => s !== seat)
          : [...new Set([...state.hiddenSeats, seat])],
        lastEffectCue: hookId,
      };
    case "_shousha_shuangjiang":
      return { ...state, splashMode: "shousha_dual", lastEffectCue: hookId };
    case "_shizhounian_shuangjiang":
    case "_shizhounian_shuangjiang2":
      return { ...state, splashMode: "decade_dual", lastEffectCue: hookId };
    default: {
      const badge = formatDecadeSkillMarkBadge(hookId, {
        count: opts?.count,
        yang: opts?.yang,
        suits: opts?.suits,
        cardTypes: opts?.cardTypes,
        usedLimited: opts?.usedLimited,
      });
      if (!badge) return state;
      const existing = (state.activeMarkBadgesBySeat[seat] ?? []).filter((b) => b.skillId !== hookId);
      return {
        ...state,
        activeMarkBadgesBySeat: {
          ...state.activeMarkBadgesBySeat,
          [seat]: [...existing, badge],
        },
        lastEffectCue: `mark:${hookId}`,
      };
    }
  }
}
