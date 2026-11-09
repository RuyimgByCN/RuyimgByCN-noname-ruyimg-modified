/**
 * L09 千幻聆音原生前端渲染面与全部 8 项 UI 钩子运行时
 * 上游对照:
 * - `archive/upstream/noname-liuli/noname/extension/千幻聆音/extension.js:1793-1875, 4017-4060, 14313-14520`
 * - `archive/upstream/noname-liuli/noname/extension/千幻聆音/skinShare.js:1-1727`
 * - `archive/upstream/noname-liuli/noname/extension/千幻聆音/extension.css:1-2057`
 * - `archive/upstream/noname-liuli/noname/extension/千幻聆音/theme/newui*.css` (10 套主题样式表)
 */

export type QhlyUiHookId =
  | "_qhcreateYH"
  | "_qhlyChangeSkin"
  | "_qhlyChageKillingSkin"
  | "_qhlyCheckSkin"
  | "_qhly_bgm"
  | "_qhly_autoc"
  | "_qhly_randskin"
  | "_qhly_addButton";

export const QHLY_UI_HOOK_IDS: ReadonlyArray<QhlyUiHookId> = [
  "_qhcreateYH",
  "_qhlyChangeSkin",
  "_qhlyChageKillingSkin",
  "_qhlyCheckSkin",
  "_qhly_bgm",
  "_qhly_autoc",
  "_qhly_randskin",
  "_qhly_addButton",
];

export type QhlyStylesheetSurfaceId =
  | "skinShare"
  | "extension.css"
  | "newui.css"
  | "newui_dc.css"
  | "newui_lolbig.css"
  | "newui_rq.css"
  | "newui_sanguo.css"
  | "newui_sanguo2.css"
  | "newui_shuimo.css"
  | "newui_sksn.css"
  | "newui_ss.css"
  | "newui_wz.css";

export interface QhlyThemeSurfacePreset {
  readonly stylesheet: QhlyStylesheetSurfaceId;
  readonly label: string;
  readonly bgGradient: string;
  readonly accentColor: string;
  readonly borderColor: string;
  readonly vMiddleEligible: boolean;
}

/**
 * 上游 12 项渲染面定义表 (`skinShare.js` + `extension.css` + 10 套 `theme/newui*.css`)
 */
export const QHLY_SURFACE_PRESETS: Readonly<Record<QhlyStylesheetSurfaceId, QhlyThemeSurfacePreset>> = {
  skinShare: {
    stylesheet: "skinShare",
    label: "皮肤语音共享映射表 (skinShare.js)",
    bgGradient: "linear-gradient(135deg, #1b2434, #101723)",
    accentColor: "#93c5fd",
    borderColor: "#334155",
    vMiddleEligible: false,
  },
  "extension.css": {
    stylesheet: "extension.css",
    label: "千幻基础资料窗 (extension.css)",
    bgGradient: "linear-gradient(135deg, #231f1a, #141210)",
    accentColor: "#e2b86b",
    borderColor: "#6b5535",
    vMiddleEligible: false,
  },
  "newui.css": {
    stylesheet: "newui.css",
    label: "新版默认 (newui.css)",
    bgGradient: "linear-gradient(135deg, #1e293b, #0f172a)",
    accentColor: "#38bdf8",
    borderColor: "#334155",
    vMiddleEligible: false,
  },
  "newui_dc.css": {
    stylesheet: "newui_dc.css",
    label: "十周年典藏 (newui_dc.css)",
    bgGradient: "linear-gradient(135deg, #2d2115, #17100a)",
    accentColor: "#f59e0b",
    borderColor: "#92400e",
    vMiddleEligible: false,
  },
  "newui_lolbig.css": {
    stylesheet: "newui_lolbig.css",
    label: "海克斯科技 (newui_lolbig.css)",
    bgGradient: "linear-gradient(135deg, #0a2229, #06131a)",
    accentColor: "#2dd4bf",
    borderColor: "#0f766e",
    vMiddleEligible: true,
  },
  "newui_rq.css": {
    stylesheet: "newui_rq.css",
    label: "燃情金戈 (newui_rq.css)",
    bgGradient: "linear-gradient(135deg, #321616, #190a0a)",
    accentColor: "#f87171",
    borderColor: "#991b1b",
    vMiddleEligible: false,
  },
  "newui_sanguo.css": {
    stylesheet: "newui_sanguo.css",
    label: "三国古卷一 (newui_sanguo.css)",
    bgGradient: "linear-gradient(135deg, #2b261d, #18150f)",
    accentColor: "#d6b981",
    borderColor: "#786646",
    vMiddleEligible: false,
  },
  "newui_sanguo2.css": {
    stylesheet: "newui_sanguo2.css",
    label: "三国古卷二 (newui_sanguo2.css)",
    bgGradient: "linear-gradient(135deg, #26231f, #141210)",
    accentColor: "#eab308",
    borderColor: "#713f12",
    vMiddleEligible: false,
  },
  "newui_shuimo.css": {
    stylesheet: "newui_shuimo.css",
    label: "水墨龙吟 (newui_shuimo.css)",
    bgGradient: "linear-gradient(135deg, #27272a, #111113)",
    accentColor: "#e4e4e7",
    borderColor: "#52525b",
    vMiddleEligible: true,
  },
  "newui_sksn.css": {
    stylesheet: "newui_sksn.css",
    label: "时空少女 (newui_sksn.css)",
    bgGradient: "linear-gradient(135deg, #2e1936, #160b1c)",
    accentColor: "#e879f9",
    borderColor: "#86198f",
    vMiddleEligible: false,
  },
  "newui_ss.css": {
    stylesheet: "newui_ss.css",
    label: "手杀臻享 (newui_ss.css)",
    bgGradient: "linear-gradient(135deg, #291d14, #140e09)",
    accentColor: "#fbbf24",
    borderColor: "#854d0e",
    vMiddleEligible: false,
  },
  "newui_wz.css": {
    stylesheet: "newui_wz.css",
    label: "王者圣域 (newui_wz.css)",
    bgGradient: "linear-gradient(135deg, #172554, #0b1120)",
    accentColor: "#60a5fa",
    borderColor: "#1d4ed8",
    vMiddleEligible: false,
  },
};

/**
 * 上游 `extension.js:4037-4058` 势力色表 (`lib.qhly_groupcolor`)
 */
export const QHLY_GROUP_COLORS: Readonly<Record<string, string>> = {
  wei: "#0000CD",
  shu: "#B22222",
  wu: "#228B22",
  qun: "#DAA520",
  jin: "#800080",
  shen: "#FFD700",
  daqin: "#4682B4",
  key: "#00CED1",
};

/**
 * 上游 `skinShare.js:20-1727` 角色皮肤与技能语音共享映射表 (`lib.qhly_skinShare`)
 */
export const QHLY_SKIN_SHARE_MAP: Readonly<
  Record<string, { readonly targetGeneral: string; readonly skills: Readonly<Record<string, string>> }>
> = {
  baosanniang: {
    targetGeneral: "re_baosanniang",
    skills: { olwuniang: "meiyong", olxushen: "rexushen", olzhennan: "rezhennan" },
  },
  xin_baosanniang: {
    targetGeneral: "re_baosanniang",
    skills: { decadewuniang: "meiyong", decadexushen: "rexushen", decadezhennan: "rezhennan" },
  },
  tw_beimihu: {
    targetGeneral: "beimihu",
    skills: { zongkui: "zongkui", guju: "guju", baijia: "baijia", bmcanshi: "bmcanshi", bingzhao: "bingzhao" },
  },
  ol_bianfuren: {
    targetGeneral: "sp_bianfuren",
    skills: { fuwei: "spwanwei", yuejian: "spyuejian" },
  },
  tw_bianfuren: {
    targetGeneral: "sp_bianfuren",
    skills: { twwanwei: "spwanwei", twyuejian: "spyuejian" },
  },
  sb_caopi: {
    targetGeneral: "re_caopi",
    skills: { sbxingshang: "rexingshang", sbfangzhu: "refangzhu", sbsongwei: "songwei" },
  },
};

export interface QhlyRuntimeState {
  activeStylesheet: QhlyStylesheetSurfaceId;
  detailModalOpen: boolean;
  detailTab: "bio" | "skill" | "voice" | "winrate";
  inspectGeneralId: string;
  bgmFlag: boolean;
  activeBgmTrack: string | null;
  enableCharacterMusic: boolean;
  skinButtonEnabled: boolean;
  dragSyncMode: "sync" | "no";
  dragButtonPositionAll: { leftPx: number; topPx: number } | null;
  dragButtonPositionByKey: Record<string, { leftPx: number; topPx: number }>;
  yuanhuSyncedSeats: Record<number, string>;
  dynamicStoppedSeats: number[];
  replaceSkinByGeneral: Record<string, string>;
  autoRotateCount: number;
  randSkinSkipSelf: boolean;
}

export function createInitialQhlyRuntimeState(): QhlyRuntimeState {
  return {
    activeStylesheet: "newui_dc.css",
    detailModalOpen: false,
    detailTab: "voice",
    inspectGeneralId: "sb_caopi",
    bgmFlag: false,
    activeBgmTrack: null,
    enableCharacterMusic: true,
    skinButtonEnabled: true,
    dragSyncMode: "sync",
    dragButtonPositionAll: { leftPx: 6, topPx: 6 },
    dragButtonPositionByKey: {},
    yuanhuSyncedSeats: { 0: "wei", 1: "shu" },
    dynamicStoppedSeats: [],
    replaceSkinByGeneral: { caocao: "origin", sb_caopi: "origin" },
    autoRotateCount: 0,
    randSkinSkipSelf: true,
  };
}

/**
 * 解析 `skinShare.js` 皮肤与技能台词语音共享关系 (`skinShare.js:1-90`)
 */
export function resolveQhlySkinShare(
  generalId: string,
  skillId?: string,
): { resolvedGeneral: string; resolvedSkill: string | null; shared: boolean } {
  const entry = QHLY_SKIN_SHARE_MAP[generalId];
  if (!entry) {
    return { resolvedGeneral: generalId, resolvedSkill: skillId ?? null, shared: false };
  }
  const resolvedSkill = skillId ? (entry.skills[skillId] ?? skillId) : null;
  return {
    resolvedGeneral: entry.targetGeneral,
    resolvedSkill,
    shared: true,
  };
}

export interface QhlyHookEvent {
  hook: QhlyUiHookId;
  seat?: number;
  generalId?: string;
  deputyGeneralId?: string;
  faction?: string;
  isDead?: boolean;
  hpDelta?: number;
  killerSeat?: number;
  killerGeneralId?: string;
  selfSeat?: number;
  dragOffset?: { leftPx: number; topPx: number };
}

/**
 * 执行千幻聆音全部 8 项上游 UI 钩子 (`extension.js:1793-14510`)
 */
export function dispatchQhlyUiHook(state: QhlyRuntimeState, ev: QhlyHookEvent): QhlyRuntimeState {
  const seat = ev.seat ?? 0;
  const gen = ev.generalId ?? state.inspectGeneralId;

  switch (ev.hook) {
    case "_qhcreateYH": {
      // extension.js:1793-1808: if (player.isDead()) player.stopDynamic(); game.qhly_checkYH(player);
      const stopped = new Set(state.dynamicStoppedSeats);
      if (ev.isDead) stopped.add(seat);
      return {
        ...state,
        dynamicStoppedSeats: [...stopped],
        yuanhuSyncedSeats: {
          ...state.yuanhuSyncedSeats,
          [seat]: ev.faction ?? state.yuanhuSyncedSeats[seat] ?? "qun",
        },
      };
    }
    case "_qhlyChangeSkin": {
      // extension.js:1809-1844: hp_ form switching on changeHp / phaseEnd
      const form = (ev.hpDelta ?? 0) < 0 ? "hp_wounded" : "hp_healthy";
      return {
        ...state,
        replaceSkinByGeneral: {
          ...state.replaceSkinByGeneral,
          [gen]: form,
        },
      };
    }
    case "_qhlyChageKillingSkin": {
      // extension.js:1845-1858: game.qhly_changeSkillSkin(trigger.source, 'kill')
      const killerGen = ev.killerGeneralId ?? gen;
      if (!killerGen) return state;
      return {
        ...state,
        replaceSkinByGeneral: {
          ...state.replaceSkinByGeneral,
          [killerGen]: "kill",
        },
      };
    }
    case "_qhlyCheckSkin": {
      // extension.js:1859-1875: gameStart -> qhly_checkYH + setCurrentSkin for name1 & name2
      const nextReplace = { ...state.replaceSkinByGeneral };
      nextReplace[gen] = nextReplace[gen] ?? "origin";
      if (ev.deputyGeneralId) {
        nextReplace[ev.deputyGeneralId] = nextReplace[ev.deputyGeneralId] ?? "origin";
      }
      return {
        ...state,
        yuanhuSyncedSeats: {
          ...state.yuanhuSyncedSeats,
          [seat]: ev.faction ?? state.yuanhuSyncedSeats[seat] ?? "wei",
        },
        replaceSkinByGeneral: nextReplace,
      };
    }
    case "_qhly_bgm": {
      // extension.js:4017-4032: if (!event.qhly_bgmflag && lib.config.qhly_enableCharacterMusic) playBackgroundMusic()
      if (state.bgmFlag || !state.enableCharacterMusic) return state;
      const shared = resolveQhlySkinShare(gen);
      return {
        ...state,
        bgmFlag: true,
        activeBgmTrack: `qhly_bgm/${shared.resolvedGeneral}.mp3`,
      };
    }
    case "_qhly_autoc": {
      // extension.js:14313: periodic skin rotation
      const nextCount = state.autoRotateCount + 1;
      return {
        ...state,
        autoRotateCount: nextCount,
        replaceSkinByGeneral: {
          ...state.replaceSkinByGeneral,
          [gen]: `skin_${(nextCount % 4) + 1}`,
        },
      };
    }
    case "_qhly_randskin": {
      // extension.js:14332: gameStart random skin; skips self when qhly_randskin_me is true
      if (state.randSkinSkipSelf && seat === (ev.selfSeat ?? 0)) {
        return state;
      }
      return {
        ...state,
        replaceSkinByGeneral: {
          ...state.replaceSkinByGeneral,
          [gen]: `rand_seat_${seat}`,
        },
      };
    }
    case "_qhly_addButton": {
      // extension.js:14465-14515: mounts .qhly_skinplayerbutton & syncs drag position globally or per general_skin
      if (!state.skinButtonEnabled) return state;
      const pos = ev.dragOffset ?? state.dragButtonPositionAll ?? { leftPx: 6, topPx: 6 };
      if (state.dragSyncMode !== "no") {
        return {
          ...state,
          dragButtonPositionAll: pos,
        };
      }
      const curSkin = state.replaceSkinByGeneral[gen] ?? "origin";
      const key = `qhly_dragButtonPositionOf_${gen}_${curSkin}`;
      return {
        ...state,
        dragButtonPositionByKey: {
          ...state.dragButtonPositionByKey,
          [key]: pos,
        },
      };
    }
  }
}
