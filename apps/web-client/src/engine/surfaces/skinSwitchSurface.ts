/**
 * L24 皮肤切换原生前端渲染面与全部 22 项 UI 钩子运行时
 * 上游对照:
 * - `archive/upstream/noname-liuli/noname/extension/皮肤切换/extension.js:176-1680`
 * - `archive/upstream/noname-liuli/noname/extension/皮肤切换/_其他js/千幻1.5精简版_extension.js:770-810`
 * - `archive/upstream/noname-liuli/noname/extension/皮肤切换/animation.js:1-1321`
 * - `component/{iconfont,interactive-chukuang,message,skeletonEditor}.css` & `style/{base,cang-zhen-ge,dynamic,edit,light-modal}.css`
 */

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
  activeModal: "none" | "cang-zhen-ge" | "skeletonEditor" | "edit" | "light-modal";
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

export function createInitialSkinSwitchRuntimeState(): SkinSwitchRuntimeState {
  const defaultParams: SavedDynamicParams = {
    x: 4,
    y: 0,
    scale: 1.0,
    angle: 0,
    speed: 1.0,
    clipDynamicBg: false,
  };
  return {
    activeModal: "none",
    newDecadeStyleOn: false,
    isAttackFlipX: true,
    useDynamicChukuang: true,
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
