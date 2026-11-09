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

import { DECADE_MARKSKILL_SPECS, type DecadeMarkSkillSpec } from "./decadeMarkSkillsData";

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
}

/**
 * 按 `markskill.js` 格式化十周年角色标记角标与悬浮说明 (`markskill.js:500-20741`)
 */
export function formatDecadeSkillMarkBadge(
  skillId: string,
  opts?: { count?: number; yang?: boolean; suits?: ReadonlyArray<string>; usedLimited?: boolean },
): DecadeMarkBadgeView | null {
  const spec = MARKSKILL_INDEX[skillId];
  if (!spec) return null;
  let badgeText = String(opts?.count ?? 1);
  if (spec.kind === "zhuanhuan") {
    badgeText = opts?.yang ? "阳" : "阴";
  } else if (spec.kind === "limited") {
    badgeText = opts?.usedLimited ? "✕" : "限";
  } else if (spec.kind === "suit" && opts?.suits && opts.suits.length > 0) {
    badgeText = sortSuitsDecadeSymbols(opts.suits);
  }
  return {
    skillId: spec.id,
    label: spec.label,
    badgeText,
    tooltip: `${spec.label} (${ badgeText })`,
    upstreamRef: `archive/upstream/noname-liuli/noname/extension/十周年UI/markskill.js:${spec.line}`,
  };
}

export interface DecadeUiRuntimeState {
  layoutVariant: "decadeLayout.css" | "layout_new.css" | "junba.css" | "doudizhu.css";
  playerCssVariant: "player_new.css" | "player1_PC.css" | "player2_PC.css" | "player3_PC.css";
  pauseHistoryOpen: boolean;
  poxiModalTargetSeat: number | null;
  jiuBuffSeats: number[];
  jiuwoDyingSeats: number[];
  tiesuoSeats: number[];
  virtualEquipsBySeat: Record<number, string[]>;
  handCountBadgeBySeat: Record<number, number>;
  rarityTierBySeat: Record<number, "junk" | "common" | "rare" | "epic" | "legend">;
  splashMode: "standard" | "guozhan_hidden" | "shousha_dual" | "decade_dual";
  lastEffectCue: string | null;
  activeMarkBadgesBySeat: Record<number, DecadeMarkBadgeView[]>;
}

export function createInitialDecadeUiRuntimeState(): DecadeUiRuntimeState {
  const initialMark = formatDecadeSkillMarkBadge("nzry_chenglve", { yang: false, suits: ["spade", "heart"] });
  return {
    layoutVariant: "decadeLayout.css",
    playerCssVariant: "player_new.css",
    pauseHistoryOpen: false,
    poxiModalTargetSeat: null,
    jiuBuffSeats: [],
    jiuwoDyingSeats: [],
    tiesuoSeats: [],
    virtualEquipsBySeat: {},
    handCountBadgeBySeat: { 0: 4, 1: 4 },
    rarityTierBySeat: { 0: "legend", 1: "epic" },
    splashMode: "standard",
    lastEffectCue: null,
    activeMarkBadgesBySeat: initialMark ? { 0: [initialMark] } : {},
  };
}

export function dispatchDecadeUiHook(
  state: DecadeUiRuntimeState,
  hookId: string,
  opts?: {
    seat?: number;
    count?: number;
    yang?: boolean;
    suits?: ReadonlyArray<string>;
    equipName?: string;
    handCount?: number;
  },
): DecadeUiRuntimeState {
  const seat = opts?.seat ?? 0;

  switch (hookId) {
    case "_jiuwo_":
      return {
        ...state,
        jiuwoDyingSeats: [...new Set([...state.jiuwoDyingSeats, seat])],
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
    case "_losetl_":
    case "_jinnangpai_":
    case "_player_hp":
    case "_HpRecover":
    case "_Hplose":
    case "_killFire":
    case "_kill_framebg":
    case "_usecard":
    case "_decadeUI_usecardBegin":
    case "_discard":
    case "_decadeUI_dieKillEffect":
    case "_recasting":
      return {
        ...state,
        lastEffectCue: `${hookId}@P${seat}`,
      };
    case "_changeEquipExt":
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
      return { ...state, lastEffectCue: "decade_inspect_overlay" };
    case "_rarity":
      return {
        ...state,
        rarityTierBySeat: { ...state.rarityTierBySeat, [seat]: "legend" },
      };
    case "_tiesuo_ying":
      return {
        ...state,
        tiesuoSeats: [...new Set([...state.tiesuoSeats, seat])],
        lastEffectCue: "tiesuo_chain",
      };
    case "_gshoupai":
      return {
        ...state,
        handCountBadgeBySeat: {
          ...state.handCountBadgeBySeat,
          [seat]: opts?.handCount ?? (state.handCountBadgeBySeat[seat] ?? 4),
        },
      };
    case "drlt_poxi":
    case "pro_drlt_poxi":
      return { ...state, poxiModalTargetSeat: seat, lastEffectCue: hookId };
    case "_gzyinni":
    case "_guozhan":
    case "_gzyinniszn":
    case "_guozhanszn":
    case "_yinni":
      return { ...state, splashMode: "guozhan_hidden", lastEffectCue: hookId };
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
