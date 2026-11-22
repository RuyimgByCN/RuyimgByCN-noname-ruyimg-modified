/**
 * L24 皮肤切换原生前端渲染面与全部 22 项 UI 钩子运行时
 * 上游对照:
 * - `archive/upstream/noname-liuli/noname/extension/皮肤切换/extension.js:176-1680, 11634-11744`
 * - `component/{iconfont,interactive-chukuang,message,skeletonEditor}.css` & `style/{base,cang-zhen-ge,dynamic,edit,light-modal}.css`
 */
import {
  createDefaultSkinSwitchConfig,
  loadPersistedSkinSwitchConfig,
  normalizeSkinSwitchConfig,
  persistSkinSwitchConfig,
  validateSkinSwitchConfigPatch,
  type SkinSwitchUiConfig,
} from "./skinSwitchConfigSchema";

export type SkinSwitchUiHookId =
  | "_qhly_autoc"
  | "_qhly_addButton"
  | "_tsx"
  | "_gj"
  | "_ts"
  | "_playAudioToQueue"
  | "_checkDcdChuChang"
  | "_checkDcdShan"
  | "_fix_yh"
  | "_fix_phase_yh"
  | "_pfqh_check_roundCount"
  | "_check_die_yh"
  | "_pfqh_reapply_saved_params"
  | "_pfqh_reapply_on_init"
  | "_pfqh_check_hp_change"
  | "_pfqh_check_damage_times"
  | "_pfqh_check_damage"
  | "_pfqh_check_jisha"
  | "_pfqh_check_changeGroup"
  | "_qhlyChangeGroupSkin"
  | "_pfqh_check_phaseBegin"
  | "_pfqh_check_phaseEnd";

export const SKIN_SWITCH_UI_HOOK_IDS: ReadonlyArray<SkinSwitchUiHookId> = [
  "_qhly_autoc",
  "_qhly_addButton",
  "_tsx",
  "_gj",
  "_ts",
  "_playAudioToQueue",
  "_checkDcdChuChang",
  "_checkDcdShan",
  "_fix_yh",
  "_fix_phase_yh",
  "_pfqh_check_roundCount",
  "_check_die_yh",
  "_pfqh_reapply_saved_params",
  "_pfqh_reapply_on_init",
  "_pfqh_check_hp_change",
  "_pfqh_check_damage_times",
  "_pfqh_check_damage",
  "_pfqh_check_jisha",
  "_pfqh_check_changeGroup",
  "_qhlyChangeGroupSkin",
  "_pfqh_check_phaseBegin",
  "_pfqh_check_phaseEnd",
];

export type SkinSwitchSurfaceId =
  | "animation"
  | "iconfont.css"
  | "interactive-chukuang.css"
  | "message.css"
  | "skeletonEditor.css"
  | "base.css"
  | "cang-zhen-ge.css"
  | "dynamic.css"
  | "edit.css"
  | "light-modal.css";

export const SKIN_SWITCH_SURFACE_IDS: ReadonlyArray<SkinSwitchSurfaceId> = [
  "animation",
  "iconfont.css",
  "interactive-chukuang.css",
  "message.css",
  "skeletonEditor.css",
  "base.css",
  "cang-zhen-ge.css",
  "dynamic.css",
  "edit.css",
  "light-modal.css",
];

export interface SkinSwitchAudioQueueEntry {
  card: string;
  id: string;
  skinId: string;
  time: number;
}

export interface SavedDynamicParams {
  x: number;
  y: number;
  scale: number;
  angle: number;
  speed: number;
  clipDynamicBg: boolean;
}

export interface SkinSwitchRuntimeState {
  config: SkinSwitchUiConfig;
  activeSurface: SkinSwitchSurfaceId;
  activeModal: "none" | "cang-zhen-ge" | "skeletonEditor" | "edit" | "light-modal" | "interactive-chukuang";
  gxnrDialogOpen: boolean;
  generatedDynamicSnippet: string | null;
  generatedTemplateSnippet: string | null;
  newDecadeStyleOn: boolean;
  isAttackFlipX: boolean;
  useDynamicChukuang: boolean;
  lastChukuangAction: { seat: number; action: string; flipX: boolean } | null;
  audioPlayQueue: SkinSwitchAudioQueueEntry[];
  yuanhuGroupBySeat: Record<number, string>;
  savedParamsByGeneral: Record<string, SavedDynamicParams>;
  appliedParamsBySeat: Record<number, SavedDynamicParams>;
  specialFormBySeat: Record<number, string>;
  damageTimesBySeat: Record<number, number>;
  toastMessage: string | null;
}

export function createInitialSkinSwitchRuntimeState(
  opts?: { usePersistedConfig?: boolean },
): SkinSwitchRuntimeState {
  const defaultParams: SavedDynamicParams = {
    x: 4,
    y: 0,
    scale: 1.0,
    angle: 0,
    speed: 1.0,
    clipDynamicBg: false,
  };
  const config = opts?.usePersistedConfig
    ? loadPersistedSkinSwitchConfig()
    : createDefaultSkinSwitchConfig();
  return {
    config,
    activeSurface: "cang-zhen-ge.css",
    activeModal: "none",
    gxnrDialogOpen: false,
    generatedDynamicSnippet: null,
    generatedTemplateSnippet: null,
    newDecadeStyleOn: false,
    isAttackFlipX: config.isAttackFlipX,
    useDynamicChukuang: config.useDynamic,
    lastChukuangAction: null,
    audioPlayQueue: [],
    yuanhuGroupBySeat: { 0: "wei", 1: "shu" },
    savedParamsByGeneral: { caocao: defaultParams },
    appliedParamsBySeat: { 0: defaultParams },
    specialFormBySeat: { 0: "base" },
    damageTimesBySeat: {},
    toastMessage: null,
  };
}

export function updateSkinSwitchConfigInState(
  state: SkinSwitchRuntimeState,
  patch: Partial<SkinSwitchUiConfig>,
  opts?: { persist?: boolean },
): SkinSwitchRuntimeState {
  const { patch: valid } = validateSkinSwitchConfigPatch(patch as Record<string, unknown>, {
    strict: false,
  });
  const config = normalizeSkinSwitchConfig({ ...state.config, ...valid });
  if (opts?.persist !== false) persistSkinSwitchConfig(config);
  return {
    ...state,
    config,
    isAttackFlipX: patch.isAttackFlipX !== undefined ? config.isAttackFlipX : state.isAttackFlipX,
    useDynamicChukuang:
      patch.useDynamic !== undefined ? config.useDynamic : state.useDynamicChukuang,
    toastMessage: patch.GXNR !== undefined ? "已加载 L24 1.24 净室更新摘要" : state.toastMessage,
  };
}

export function runSkinSwitchActionConfig(
  state: SkinSwitchRuntimeState,
  key: "previewDynamic" | "resetArchiveDynamicSkin" | "genDynamicSkin" | "genDyTempFile",
): SkinSwitchRuntimeState {
  if (key === "previewDynamic") {
    return {
      ...state,
      activeSurface: "skeletonEditor.css",
      activeModal: "skeletonEditor",
      config: { ...state.config, previewDynamic: true },
      toastMessage: "已打开自备 Spine 4.0 预览台",
    };
  }
  if (key === "resetArchiveDynamicSkin") {
    const resetParam: SavedDynamicParams = { x: 0, y: 0, scale: 1.0, angle: 0, speed: 1.0, clipDynamicBg: false };
    return {
      ...state,
      config: { ...state.config, resetArchiveDynamicSkin: true },
      savedParamsByGeneral: { caocao: resetParam },
      appliedParamsBySeat: { 0: resetParam },
      toastMessage: "已重置动皮坐标存档参数",
    };
  }
  if (key === "genDynamicSkin") {
    const snippet = JSON.stringify(
      {
        decadeDynamicParams: state.savedParamsByGeneral,
        mode: "cleanroom-param-converter",
        spineRequiredFromUser: true,
      },
      null,
      2,
    );
    return {
      ...state,
      activeSurface: "edit.css",
      config: { ...state.config, genDynamicSkin: true },
      generatedDynamicSnippet: snippet,
      toastMessage: "已生成十周年动态参数模板（零资产纯配置）",
    };
  }
  const tpl = JSON.stringify(
    {
      folderLayout: "武将中文名或id/皮肤名称/骨骼.json|.skel",
      template: { x: [0, 0], y: [0, 0], scale: 1, action: "DaiJi", version: "4.0" },
    },
    null,
    2,
  );
  return {
    ...state,
    activeSurface: "interactive-chukuang.css",
    config: { ...state.config, genDyTempFile: true },
    generatedTemplateSnippet: tpl,
    toastMessage: "已生成自备骨骼标准目录模板",
  };
}

export function skinSwitchArenaDatasetAttrs(state: SkinSwitchRuntimeState): Record<string, string> {
  return {
    "data-skinswitch-surface": state.activeSurface,
    "data-skinswitch-top-arc": state.config.showTopArc ? "on" : "off",
    "data-skinswitch-hide-huanfu": state.config.hideHuanFu ? "on" : "off",
    "data-skinswitch-cug-bg": state.config.cugDynamicBg ? "on" : "off",
    "data-skinswitch-use-dynamic": state.useDynamicChukuang ? "on" : "off",
    "data-skinswitch-click-player": state.config.clickPlayerDynamic ? "on" : "off",
    "data-skinswitch-preview-dyn": state.config.previewSkinsDynamic ? "on" : "off",
  };
}

export interface SkinSwitchHookEvent {
  hook: SkinSwitchUiHookId;
  seat?: number;
  generalId?: string;
  skinId?: string;
  cardName?: string;
  cardNature?: "fire" | "thunder" | "ice" | "stab" | null;
  seatScreenXPct?: number;
  doubleAvatar?: boolean;
  roundNumber?: number;
  roundThreshold?: number;
  hp?: number;
  damageAmount?: number;
  faction?: string;
  timestampMs?: number;
}

/**
 * 执行皮肤切换 (L24) 全部 22 项上游 UI 钩子 (`extension.js:176-1680`)
 */
export function dispatchSkinSwitchHook(
  state: SkinSwitchRuntimeState,
  ev: SkinSwitchHookEvent,
): SkinSwitchRuntimeState {
  const seat = ev.seat ?? 0;
  const gen = ev.generalId ?? "caocao";
  const skinId = ev.skinId ?? "primary_1";

  switch (ev.hook) {
    case "_qhly_autoc":
      return {
        ...state,
        specialFormBySeat: { ...state.specialFormBySeat, [seat]: "auto_cycled" },
      };
    case "_qhly_addButton":
      return {
        ...state,
        toastMessage: `已挂载角色换肤按钮 (P${seat})`,
      };
    case "_tsx":
    case "_ts": {
      // extension.js:176 & 440: logSkillBegin / useSkillBefore -> out-of-frame skill Spine action
      if (!state.useDynamicChukuang) return state;
      return {
        ...state,
        lastChukuangAction: {
          seat,
          action: ev.hook === "_ts" ? "decade_skill_ts" : "chukuang_teshu",
          flipX: false,
        },
      };
    }
    case "_gj": {
      // extension.js:337 & 11702: useCardBefore -> attack action, flipping X if AI on left half (<50%)
      if (!state.useDynamicChukuang) return state;
      const flipX = state.isAttackFlipX && (ev.seatScreenXPct ?? 60) < 50;
      return {
        ...state,
        lastChukuangAction: { seat, action: "GongJi", flipX },
      };
    }
    case "_playAudioToQueue": {
      // extension.js:598-629: sha + fire/thunder/ice/stab -> sha_<nature> queued
      const baseCard = ev.cardName ?? "sha";
      const card =
        baseCard === "sha" &&
        (ev.cardNature === "fire" ||
          ev.cardNature === "thunder" ||
          ev.cardNature === "ice" ||
          ev.cardNature === "stab")
          ? `${baseCard}_${ev.cardNature}`
          : baseCard;
      const item: SkinSwitchAudioQueueEntry = {
        card,
        id: gen,
        skinId,
        time: ev.timestampMs ?? 1700000000000,
      };
      return {
        ...state,
        audioPlayQueue: [...state.audioPlayQueue.slice(-15), item],
      };
    }
    case "_checkDcdChuChang": {
      // extension.js:632-643: phaseBefore -> chukuangAction(player, 'chuchang') when !doubleAvatar
      if (ev.doubleAvatar || !state.useDynamicChukuang) return state;
      return {
        ...state,
        lastChukuangAction: { seat, action: "chuchang", flipX: false },
      };
    }
    case "_checkDcdShan": {
      // extension.js:646-664: useCard(shan) -> play3 / shan
      if ((ev.cardName ?? "shan") !== "shan") return state;
      return {
        ...state,
        lastChukuangAction: { seat, action: "play3", flipX: false },
      };
    }
    case "_fix_yh":
    case "_fix_phase_yh": {
      // extension.js:667-692: skinSwitchCheckYH when newDecadeStyle !== 'on'
      if (state.newDecadeStyleOn) return state;
      return {
        ...state,
        yuanhuGroupBySeat: {
          ...state.yuanhuGroupBySeat,
          [seat]: ev.faction ?? state.yuanhuGroupBySeat[seat] ?? "wei",
        },
      };
    }
    case "_check_die_yh": {
      // extension.js:785: dieBefore -> mark yuanhu dead
      return {
        ...state,
        yuanhuGroupBySeat: {
          ...state.yuanhuGroupBySeat,
          [seat]: "dead",
        },
      };
    }
    case "_pfqh_reapply_saved_params":
    case "_pfqh_reapply_on_init": {
      // extension.js:804 & 852: reapply saved dynamic parameters on gameStart / enterGame
      const saved = state.savedParamsByGeneral[gen] ?? {
        x: 4,
        y: 0,
        scale: 1.0,
        angle: 0,
        speed: 1.0,
        clipDynamicBg: false,
      };
      return {
        ...state,
        appliedParamsBySeat: {
          ...state.appliedParamsBySeat,
          [seat]: saved,
        },
      };
    }
    case "_pfqh_check_roundCount": {
      // extension.js:695-720: transform when roundNumber === effs.rounds
      const targetRound = ev.roundThreshold ?? 2;
      if ((ev.roundNumber ?? 2) !== targetRound) return state;
      return {
        ...state,
        specialFormBySeat: { ...state.specialFormBySeat, [seat]: `round_${targetRound}` },
      };
    }
    case "_pfqh_check_hp_change": {
      const lowHp = (ev.hp ?? 2) <= 2;
      return {
        ...state,
        specialFormBySeat: { ...state.specialFormBySeat, [seat]: lowHp ? "hp_awakened" : "base" },
      };
    }
    case "_pfqh_check_damage_times": {
      const nextTimes = (state.damageTimesBySeat[seat] ?? 0) + 1;
      return {
        ...state,
        damageTimesBySeat: { ...state.damageTimesBySeat, [seat]: nextTimes },
        specialFormBySeat: {
          ...state.specialFormBySeat,
          [seat]: nextTimes >= 2 ? `damage_times_${nextTimes}` : (state.specialFormBySeat[seat] ?? "base"),
        },
      };
    }
    case "_pfqh_check_damage": {
      const amt = Math.max(1, ev.damageAmount ?? 1);
      return {
        ...state,
        specialFormBySeat: { ...state.specialFormBySeat, [seat]: `damage_${amt}` },
      };
    }
    case "_pfqh_check_jisha": {
      return {
        ...state,
        specialFormBySeat: { ...state.specialFormBySeat, [seat]: "jisha_form" },
      };
    }
    case "_pfqh_check_changeGroup":
    case "_qhlyChangeGroupSkin": {
      const grp = ev.faction ?? "shen";
      return {
        ...state,
        yuanhuGroupBySeat: { ...state.yuanhuGroupBySeat, [seat]: grp },
        specialFormBySeat: { ...state.specialFormBySeat, [seat]: `group_${grp}` },
      };
    }
    case "_pfqh_check_phaseBegin":
      return {
        ...state,
        specialFormBySeat: { ...state.specialFormBySeat, [seat]: "phase_active" },
      };
    case "_pfqh_check_phaseEnd":
      return {
        ...state,
        specialFormBySeat: { ...state.specialFormBySeat, [seat]: "phase_idle" },
      };
    default:
      return state;
  }
}
