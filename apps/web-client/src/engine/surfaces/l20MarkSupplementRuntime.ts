/**
 * L20 标记补充 运行时契约：全部 46 项 `ui_hooks`（含 34 项简化近似收口）、3 项 CSS 渲染面 (`extension.css`, `group.css`, `mark.css`) 与 15 项配置。
 * 上游对照: `archive/upstream/noname-liuli/noname/extension/标记补充/`
 * 严守裁决 35/38 净室边界：零商业二进制/音频依赖，UI 钩子绝不篡改内核规则。
 */

import type { EngineEnvelope } from "../../types/engine";

export const L20_UI_HOOK_IDS = [
  "_xigua_feiyanbahu_",
  "_shiqihujia_",
  "_guozhanChooseControlCHG",
  "diaohulishan",
  "_guozhan_marks",
  "_zhulianbihe_mark_tao",
  "_yinyang_mark_add",
  "_guozhan_diebiaoji_",
  "_guozhan_duiweibiaoji_gua_",
  "_guozhan_duiweibiaoji_gua_gua_",
  "_aozhan_judge",
  "_guozhanhalfchghp",
  "_xigua_xueliangfanbei_",
  "_xigua_mopaifanbei_",
  "_xigua_zhuwanzhongji_",
  "_shoushajisha_",
  "_shoushajisha_delete_",
  "_diankuangtulu_gua",
  "_wanjunqushou_gua",
  "_o_o_recovertrigger",
  "_o_o_miaoshouhuichun",
  "_o_o_yishugaochao",
  "_losehp_",
  "_mianshang_",
  "_player_baojiA",
  "_player_baojiB",
  "_player_baojiC",
  "_player_baojiD",
  "_player_baojiE",
  "_player_baojiF",
  "_player_baojiG",
  "_player_baojiH",
  "_jiubuff_",
  "_jiubuff_xiaoshi_",
  "_player_game_guagua",
  "_guozhan_quanjubiaoji_guagua",
  "_shiyongkapaitexiao_",
  "_shiyongkapaitexiao_xi_gua_",
  "_shanghaipaishiyongtexiao_",
  "_player_daojianfu_xxgg_fuzi_",
  "_binglintu",
  "_dysp",
  "kuanshi2",
  "mbmowang",
  "zuoxing",
  "_chongzhu",
] as const;

export type L20UiHookId = (typeof L20_UI_HOOK_IDS)[number];

export const L20_CSS_SURFACE_IDS = ["extension.css", "group.css", "mark.css"] as const;
export type L20CssSurfaceId = (typeof L20_CSS_SURFACE_IDS)[number];

export interface L20UiConfig {
  jinengxiugai: boolean;
  biaojixiugai: boolean;
  zhijiechifa: boolean;
  doudizhu_guanqiuxiaoguo: boolean;
  huajiatexiao: boolean;
  guozhanmeihua: boolean;
  shoushajishatexiao: boolean;
  shiyongkapaitixiao: boolean;
  youxishengfu: "identity" | "mobileMode" | "off";
  daojianfu: "shousha" | "4.5" | "off";
  daojianfu_type: "random" | "no" | "dao" | "jian" | "fuzi";
  xinshadoudizhu: boolean;
  xlfbmoshi: boolean;
  mpfbmoshi: boolean;
  zhujizhongwang: boolean;
}

export const L20_CONFIG_KEYS: ReadonlyArray<keyof L20UiConfig> = [
  "jinengxiugai",
  "biaojixiugai",
  "zhijiechifa",
  "doudizhu_guanqiuxiaoguo",
  "huajiatexiao",
  "guozhanmeihua",
  "shoushajishatexiao",
  "shiyongkapaitixiao",
  "youxishengfu",
  "daojianfu",
  "daojianfu_type",
  "xinshadoudizhu",
  "xlfbmoshi",
  "mpfbmoshi",
  "zhujizhongwang",
];

export function createDefaultL20UiConfig(): L20UiConfig {
  return {
    jinengxiugai: true,
    biaojixiugai: true,
    zhijiechifa: true,
    doudizhu_guanqiuxiaoguo: true,
    huajiatexiao: true,
    guozhanmeihua: true,
    shoushajishatexiao: true,
    shiyongkapaitixiao: true,
    youxishengfu: "off",
    daojianfu: "4.5",
    daojianfu_type: "random",
    xinshadoudizhu: false,
    xlfbmoshi: false,
    mpfbmoshi: false,
    zhujizhongwang: false,
  };
}

export interface L20RuntimeState {
  configs: L20UiConfig;
  activeSurfaces: ReadonlyArray<L20CssSurfaceId>;
  killStreakBySeat: Record<number, number>;
  jiuAuraSeats: number[];
  feiyanBahuSeats: number[];
  guozhanFormationsBySeat: Record<number, "wei" | "shu" | "wu" | "qun" | "jin" | "flank">;
  guozhanGlobalMarks: ReadonlyArray<"xianqu_mark" | "yexinjia_mark" | "yinyang_mark" | "zhulianbihe_mark">;
  guozhanControlThemeOn: boolean;
  baojiPopupBySeat: Record<number, string>;
  weaponStrikeBySeat: Record<number, "dao" | "jian" | "fuzi" | "no">;
  cardFxCueBySeat: Record<number, string>;
  rankSweepBySeat: Record<number, number>;
  playerHpCueBySeat: Record<number, string>;
  guozhanHalfHpBySeat: Record<number, string>;
  binglinCrest: string | null;
  binglinTeammateHint: boolean;
  activeBanner: { code: string; title: string; tier: "epic" | "heal" | "kill" } | null;
  recastCount: number;
  lastDispatchedHook: string | null;
}

export function createInitialL20RuntimeState(): L20RuntimeState {
  return {
    configs: createDefaultL20UiConfig(),
    activeSurfaces: [...L20_CSS_SURFACE_IDS],
    killStreakBySeat: {},
    jiuAuraSeats: [],
    feiyanBahuSeats: [],
    guozhanFormationsBySeat: {},
    guozhanGlobalMarks: [],
    guozhanControlThemeOn: false,
    baojiPopupBySeat: {},
    weaponStrikeBySeat: {},
    cardFxCueBySeat: {},
    rankSweepBySeat: {},
    playerHpCueBySeat: {},
    guozhanHalfHpBySeat: {},
    binglinCrest: null,
    binglinTeammateHint: false,
    activeBanner: null,
    recastCount: 0,
    lastDispatchedHook: null,
  };
}

const BAOJI_TITLES: Readonly<Record<string, string>> = {
  _player_baojiA: "奋威暴击",
  _player_baojiB: "无双重击",
  _player_baojiC: "破竹一击",
  _player_baojiD: "雷霆暴击",
  _player_baojiE: "狂澜猛击",
  _player_baojiF: "烈焰摧敌",
  _player_baojiG: "震岳千钧",
  _player_baojiH: "绝杀爆发",
};

const KILL_STREAK_LABELS = [
  "",
  "一破 · 卧龙出山",
  "双连 · 一战成名",
  "三连 · 举世皆惊",
  "四连 · 天下无敌",
  "五连 · 诛天灭地",
  "六连 · 癫狂屠戮",
  "七连 · 神鬼莫测",
];

function resolveWeaponKind(cfg: L20UiConfig, seat: number, explicit?: string): "dao" | "jian" | "fuzi" | "no" {
  if (cfg.daojianfu === "off") return "no";
  if (explicit === "dao" || explicit === "jian" || explicit === "fuzi" || explicit === "no") return explicit;
  if (cfg.daojianfu_type !== "random") return cfg.daojianfu_type;
  const order: ReadonlyArray<"dao" | "jian" | "fuzi"> = ["dao", "jian", "fuzi"];
  return order[Math.abs(seat) % order.length]!;
}

export interface L20HookOptions {
  seat?: number;
  faction?: "wei" | "shu" | "wu" | "qun" | "jin" | "flank";
  markName?: "xianqu_mark" | "yexinjia_mark" | "yinyang_mark" | "zhulianbihe_mark";
  cardName?: string;
  isDamageCard?: boolean;
  stars?: number;
  weaponType?: "dao" | "jian" | "fuzi" | "no";
  binglinSkill?: "jiangling" | "xiangyang" | "fancheng" | "jianxi";
  halfTag?: string;
}

export function dispatchL20UiHook(
  state: L20RuntimeState,
  hook: L20UiHookId | string,
  opts?: L20HookOptions,
): L20RuntimeState {
  const seat = opts?.seat ?? 0;
  const base: L20RuntimeState = { ...state, lastDispatchedHook: hook };

  if (hook === "_xigua_feiyanbahu_") {
    if (!state.configs.doudizhu_guanqiuxiaoguo) return base;
    return {
      ...base,
      feiyanBahuSeats: [...new Set([...state.feiyanBahuSeats, seat])],
      activeBanner: { code: hook, title: "飞扬跋扈 · 地主入场", tier: "epic" },
    };
  }
  if (hook === "_guozhanChooseControlCHG") {
    return { ...base, guozhanControlThemeOn: true };
  }
  if (hook === "_guozhan_marks" || hook === "_zhulianbihe_mark_tao" || hook === "_yinyang_mark_add") {
    const nextMarks = opts?.markName
      ? [...new Set([...state.guozhanGlobalMarks, opts.markName])]
      : state.guozhanGlobalMarks;
    return { ...base, guozhanGlobalMarks: nextMarks };
  }
  if (hook === "_guozhan_quanjubiaoji_guagua") {
    const mark = opts?.markName ?? "xianqu_mark";
    const labels = {
      xianqu_mark: "先驱 · 显赫开局",
      yexinjia_mark: "野心家 · 孤立枭雄",
      yinyang_mark: "阴阳鱼 · 调和生息",
      zhulianbihe_mark: "珠联璧合 · 同舟共济",
    };
    return {
      ...base,
      guozhanGlobalMarks: [...new Set([...state.guozhanGlobalMarks, mark])],
      activeBanner: { code: mark, title: labels[mark], tier: "epic" },
    };
  }
  if (hook === "_guozhan_diebiaoji_") {
    return { ...base, guozhanGlobalMarks: [] };
  }
  if (hook === "_guozhan_duiweibiaoji_gua_") {
    if (!state.configs.guozhanmeihua) return base;
    return {
      ...base,
      guozhanFormationsBySeat: {
        ...state.guozhanFormationsBySeat,
        [seat]: opts?.faction ?? "wei",
      },
    };
  }
  if (hook === "_guozhan_duiweibiaoji_gua_gua_") {
    const next = { ...state.guozhanFormationsBySeat };
    delete next[seat];
    return { ...base, guozhanFormationsBySeat: next };
  }
  if (hook === "_shoushajisha_") {
    if (!state.configs.shoushajishatexiao) return base;
    const nextCount = Math.min(7, (state.killStreakBySeat[seat] ?? 0) + 1);
    return {
      ...base,
      killStreakBySeat: { ...state.killStreakBySeat, [seat]: nextCount },
      activeBanner: {
        code: `kill_${nextCount}`,
        title: KILL_STREAK_LABELS[nextCount] ?? `连杀 ×${nextCount}`,
        tier: "kill",
      },
    };
  }
  if (hook === "_shoushajisha_delete_") {
    const next = { ...state.killStreakBySeat };
    delete next[seat];
    return { ...base, killStreakBySeat: next };
  }
  if (hook === "_diankuangtulu_gua") {
    return { ...base, activeBanner: { code: hook, title: "癫狂屠戮 · 一击3伤", tier: "kill" } };
  }
  if (hook === "_wanjunqushou_gua") {
    return { ...base, activeBanner: { code: hook, title: "万军取首 · 一击必杀", tier: "kill" } };
  }
  if (hook === "_o_o_recovertrigger" || hook === "_o_o_miaoshouhuichun") {
    return {
      ...base,
      playerHpCueBySeat: { ...state.playerHpCueBySeat, [seat]: "妙手回春" },
      activeBanner:
        hook === "_o_o_miaoshouhuichun"
          ? { code: hook, title: "妙手回春 · 救死扶伤", tier: "heal" }
          : state.activeBanner,
    };
  }
  if (hook === "_o_o_yishugaochao") {
    return {
      ...base,
      playerHpCueBySeat: { ...state.playerHpCueBySeat, [seat]: "医术高超" },
      activeBanner: { code: hook, title: "医术高超 · 悬壶济世", tier: "heal" },
    };
  }
  if (hook === "_losehp_") {
    return { ...base, playerHpCueBySeat: { ...state.playerHpCueBySeat, [seat]: "体力流失" } };
  }
  if (hook === "_mianshang_") {
    return { ...base, playerHpCueBySeat: { ...state.playerHpCueBySeat, [seat]: "免疫伤害" } };
  }
  if (hook in BAOJI_TITLES) {
    return {
      ...base,
      baojiPopupBySeat: { ...state.baojiPopupBySeat, [seat]: BAOJI_TITLES[hook]! },
    };
  }
  if (hook === "_jiubuff_") {
    return { ...base, jiuAuraSeats: [...new Set([...state.jiuAuraSeats, seat])] };
  }
  if (hook === "_jiubuff_xiaoshi_") {
    return { ...base, jiuAuraSeats: state.jiuAuraSeats.filter((s) => s !== seat) };
  }
  if (hook === "_player_game_guagua") {
    const stars = Math.max(1, Math.min(5, opts?.stars ?? 3));
    return { ...base, rankSweepBySeat: { ...state.rankSweepBySeat, [seat]: stars } };
  }
  if (
    hook === "_shiyongkapaitexiao_" ||
    hook === "_shiyongkapaitexiao_xi_gua_" ||
    hook === "_shanghaipaishiyongtexiao_"
  ) {
    if (!state.configs.shiyongkapaitixiao) return base;
    const label =
      hook === "_shanghaipaishiyongtexiao_" || opts?.isDamageCard
        ? `伤害牌·${opts?.cardName ?? "杀"}`
        : `${opts?.cardName ?? "牌效"}`;
    return { ...base, cardFxCueBySeat: { ...state.cardFxCueBySeat, [seat]: label } };
  }
  if (hook === "_player_daojianfu_xxgg_fuzi_") {
    const kind = resolveWeaponKind(state.configs, seat, opts?.weaponType);
    if (kind === "no") return base;
    return { ...base, weaponStrikeBySeat: { ...state.weaponStrikeBySeat, [seat]: kind } };
  }
  if (hook === "_binglintu") {
    return { ...base, binglinCrest: opts?.binglinSkill ?? "jiangling" };
  }
  if (hook === "_dysp") {
    return { ...base, binglinTeammateHint: true };
  }
  if (hook === "_chongzhu") {
    return { ...base, recastCount: state.recastCount + 1 };
  }
  if (hook === "_guozhanhalfchghp") {
    const tag = opts?.halfTag ?? "半血阴阳";
    return {
      ...base,
      guozhanHalfHpBySeat: { ...state.guozhanHalfHpBySeat, [seat]: tag },
      activeBanner: { code: hook, title: `半血阴阳 · P${seat} ${tag}`, tier: "epic" },
    };
  }
  return base;
}

export function l20ArenaDatasetAttrs(state: L20RuntimeState): Record<string, string> {
  return {
    "data-l20-daojianfu": state.configs.daojianfu,
    "data-l20-guozhan": state.configs.guozhanmeihua ? "on" : "off",
    "data-l20-card-fx": state.configs.shiyongkapaitixiao ? "on" : "off",
    "data-l20-surfaces": state.activeSurfaces.join(" "),
    "data-l20-control-chg": state.guozhanControlThemeOn ? "on" : "off",
    "data-l20-binglin": state.binglinCrest ?? "off",
  };
}

export function syncL20MarkEffectsFromEnvelope(
  prev: L20RuntimeState,
  env: EngineEnvelope,
): L20RuntimeState {
  const view = env.view;
  if (!view) return prev;
  let next: L20RuntimeState = {
    ...prev,
    baojiPopupBySeat: {},
    weaponStrikeBySeat: {},
    cardFxCueBySeat: {},
    playerHpCueBySeat: {},
  };
  const cur = view.current_player ?? 0;

  if (view.phase === "Prepare" || (view.round >= 1 && (next.rankSweepBySeat[cur] ?? 0) === 0)) {
    next = dispatchL20UiHook(next, "_player_game_guagua", { seat: cur, stars: 4 });
  }

  for (const rawEv of env.events ?? []) {
    const pubEv = (rawEv as { PublicEvent?: { code?: number; player?: number; arg?: number } })?.PublicEvent;
    const pubCode = pubEv?.code;
    const evType = (rawEv as { type?: string })?.type;
    const cardKind = (rawEv as { card?: string; kind?: string })?.card ?? (rawEv as { kind?: string })?.kind;
    const target = typeof (rawEv as { target?: unknown })?.target === "number"
      ? (rawEv as { target: number }).target
      : typeof (rawEv as { victim?: unknown })?.victim === "number"
        ? (rawEv as { victim: number }).victim
        : cur;
    const source = typeof (rawEv as { source?: unknown })?.source === "number"
      ? (rawEv as { source: number }).source
      : typeof pubEv?.player === "number"
        ? pubEv.player
        : cur;

    if (pubCode === 2 || pubCode === 3 || evType === "CardPlayed") {
      if (cardKind === "Jiu") {
        next = dispatchL20UiHook(next, "_jiubuff_", { seat: source });
      } else if (cardKind === "Sha" || cardKind === "ShaFire" || cardKind === "ShaThunder") {
        next = dispatchL20UiHook(next, "_jiubuff_xiaoshi_", { seat: source });
        next = dispatchL20UiHook(next, "_shanghaipaishiyongtexiao_", {
          seat: source,
          cardName: cardKind,
          isDamageCard: true,
        });
      } else {
        next = dispatchL20UiHook(next, "_shiyongkapaitexiao_", {
          seat: source,
          cardName: typeof cardKind === "string" ? cardKind : "出牌",
        });
      }
    } else if (pubCode === 4 || evType === "DamageDealt") {
      const amt = typeof (rawEv as { amount?: unknown })?.amount === "number"
        ? (rawEv as { amount: number }).amount
        : 1;
      next = dispatchL20UiHook(next, "_player_daojianfu_xxgg_fuzi_", { seat: target });
      if (amt >= 3) next = dispatchL20UiHook(next, "_diankuangtulu_gua", { seat: source });
      if (amt >= 2) next = dispatchL20UiHook(next, "_player_baojiB", { seat: source });
      if (amt === 0) next = dispatchL20UiHook(next, "_mianshang_", { seat: target });
    } else if (pubCode === 5 || evType === "HpRecovered") {
      next = dispatchL20UiHook(next, "_o_o_recovertrigger", { seat: target });
    } else if (pubCode === 6 || evType === "HpLost") {
      next = dispatchL20UiHook(next, "_losehp_", { seat: target });
    } else if (pubCode === 7 || evType === "PlayerDied") {
      next = dispatchL20UiHook(next, "_shoushajisha_", { seat: source });
      next = dispatchL20UiHook(next, "_guozhan_diebiaoji_", { seat: target });
    } else if (pubCode === 8 && view.phase === "End") {
      next = dispatchL20UiHook(next, "_jiubuff_xiaoshi_", { seat: cur });
      next = dispatchL20UiHook(next, "_shoushajisha_delete_", { seat: cur });
    }
  }
  return next;
}
