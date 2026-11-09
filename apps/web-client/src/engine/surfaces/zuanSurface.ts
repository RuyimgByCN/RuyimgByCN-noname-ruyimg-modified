/**
 * L25 祖安武将原生前端 40 项 UI/音效/判定全屏特效钩子运行时
 * 上游对照: `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:37514-48503`
 */

export const ZUAN_UI_HOOK_IDS = [
  "_zuanattacklinecheck",
  "_extraphaseskill",
  "_extradieskill",
  "_extraturnoverskill",
  "_extradamageskill",
  "_extradlosehpskill",
  "_extraloseskill",
  "_extragainskill",
  "_extraloseCardskill",
  "_extracompareskill",
  "_extrauseskill",
  "_extrajudgeskill",
  "_extralinkskill",
  "_comparesayskill",
  "_equipuseskill",
  "_damsayskill",
  "_saveskill",
  "_dieskill",
  "_shanskill",
  "_useCardsayskill",
  "_phasebeginskill",
  "_phaseendskill",
  "_judgeskill",
  "_linkskill",
  "_newBackgroundUse",
  "_zuanboss",
  "_helasisyskill",
  "_anshaskill",
  "_seadreammodeskill",
  "_relifeskill",
  "_haoyunlai",
  "_bingleBigScreen",
  "_bingletexiaoshandian",
  "_bingletexiaohuoshan",
  "_bingletexiaohongshui",
  "_bingletexiaofulei",
  "_bingletexiaole",
  "_bingletexiaobingliang",
  "_thundertexiao",
  "_thundertexiao2",
] as const;

export type ZuanUiHookId = (typeof ZUAN_UI_HOOK_IDS)[number];

export interface ZuanUiRuntimeState {
  attackLineStyle: "guanjie" | "classic" | "laser";
  attackLineOpacity: number;
  mutePaperAudio: boolean;
  skipOpeningSplash: boolean;
  seaDreamPhaseTint: boolean;
  reviveCountdownBySeat: Record<number, number>;
  anshaHiddenSeats: number[];
  activeWallpaper: string;
  judgmentOverlay:
    | "none"
    | "shandian"
    | "huoshan"
    | "hongshui"
    | "fulei"
    | "lebusishu"
    | "bingliang";
  lastAudioCue: string | null;
  luckyDrawShimmer: boolean;
  bossDefeatedBanner: boolean;
}

export function createInitialZuanUiRuntimeState(): ZuanUiRuntimeState {
  return {
    attackLineStyle: "guanjie",
    attackLineOpacity: 1.0,
    mutePaperAudio: false,
    skipOpeningSplash: true,
    seaDreamPhaseTint: false,
    reviveCountdownBySeat: {},
    anshaHiddenSeats: [],
    activeWallpaper: "shihai_default",
    judgmentOverlay: "none",
    lastAudioCue: null,
    luckyDrawShimmer: false,
    bossDefeatedBanner: false,
  };
}

/**
 * 执行 L25 祖安武将全部 40 项前端 UI/音效/特效钩子 (`extension.js:37514-48503`)
 */
export function dispatchZuanUiHook(
  state: ZuanUiRuntimeState,
  hook: ZuanUiHookId,
  opts?: { seat?: number; reviveTurns?: number; wallpaperId?: string },
): ZuanUiRuntimeState {
  const seat = opts?.seat ?? 0;

  switch (hook) {
    case "_zuanattacklinecheck":
      // extension.js:37514: syncs Decade attack-line preset & opacity
      return { ...state, attackLineStyle: "guanjie" };
    case "_extragainskill":
    case "_extraloseCardskill":
    case "_extraloseskill":
    case "_extrauseskill":
      // extension.js:40597-40693: paper draw/discard/use audio gated by audio_draw mute config
      if (state.mutePaperAudio) return state;
      return { ...state, lastAudioCue: `paper:${hook}` };
    case "_extraphaseskill":
    case "_extradieskill":
    case "_extraturnoverskill":
    case "_extradamageskill":
    case "_extradlosehpskill":
    case "_extracompareskill":
    case "_extrajudgeskill":
    case "_extralinkskill":
    case "_comparesayskill":
    case "_equipuseskill":
    case "_damsayskill":
    case "_saveskill":
    case "_dieskill":
    case "_shanskill":
    case "_useCardsayskill":
    case "_phasebeginskill":
    case "_phaseendskill":
    case "_judgeskill":
    case "_linkskill":
      // extension.js:40500-40925: event SFX & taunt voice cues
      return { ...state, lastAudioCue: `sfx:${hook}@P${seat}` };
    case "_newBackgroundUse":
      // extension.js:41318: dynamic background rotation on card target completion
      return { ...state, activeWallpaper: opts?.wallpaperId ?? `shihai_seat_${seat}` };
    case "_zuanboss":
      // extension.js:41545: boss fall overlay banner
      return { ...state, bossDefeatedBanner: true };
    case "_helasisyskill":
      // extension.js:41890: skips opening splash animation when helasisy === true
      return { ...state, skipOpeningSplash: true, lastAudioCue: "dump_start" };
    case "_anshaskill":
      // extension.js:42659: removes assassination stealth mark at phaseEnd
      return {
        ...state,
        anshaHiddenSeats: state.anshaHiddenSeats.filter((s) => s !== seat),
      };
    case "_seadreammodeskill":
      // extension.js:42815: toggles 幻海 sea-dream phase ambient tint
      return { ...state, seaDreamPhaseTint: !state.seaDreamPhaseTint };
    case "_relifeskill":
      // extension.js:43402: updates revive countdown HUD badge
      return {
        ...state,
        reviveCountdownBySeat: {
          ...state.reviveCountdownBySeat,
          [seat]: Math.max(0, opts?.reviveTurns ?? 3),
        },
      };
    case "_haoyunlai":
      // extension.js:47584: lucky-star drawBegin shimmer cue
      return { ...state, luckyDrawShimmer: true, lastAudioCue: "haoyunlai" };
    case "_bingleBigScreen":
    case "_bingletexiaoshandian":
    case "_thundertexiao":
    case "_thundertexiao2":
      // extension.js:47880, 47984, 48389, 48503: lightning judgment full-screen overlay
      return { ...state, judgmentOverlay: "shandian", lastAudioCue: "judge:shandian" };
    case "_bingletexiaohuoshan":
      return { ...state, judgmentOverlay: "huoshan", lastAudioCue: "judge:huoshan" };
    case "_bingletexiaohongshui":
      return { ...state, judgmentOverlay: "hongshui", lastAudioCue: "judge:hongshui" };
    case "_bingletexiaofulei":
      return { ...state, judgmentOverlay: "fulei", lastAudioCue: "judge:fulei" };
    case "_bingletexiaole":
      return { ...state, judgmentOverlay: "lebusishu", lastAudioCue: "judge:lebusishu" };
    case "_bingletexiaobingliang":
      return { ...state, judgmentOverlay: "bingliang", lastAudioCue: "judge:bingliang" };
  }
}
