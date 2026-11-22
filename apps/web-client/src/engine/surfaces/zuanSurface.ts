/**
 * L25 祖安武将原生前端 40 项 UI/音效/判定全屏特效钩子 + 68 项配置运行时
 * 上游对照: `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:160-332, 37514-51435`
 */
import {
  createDefaultZuanConfig,
  validateZuanConfigPatch,
  type ZuanUiConfigMap,
} from "./zuanConfigSchema";

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
  config: ZuanUiConfigMap;
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
  lastControlHook: string | null;
  zuanSeatCueBySeat: Record<number, string>;
  luckyDrawShimmer: boolean;
  bossDefeatedBanner: boolean;
}

export function createInitialZuanUiRuntimeState(): ZuanUiRuntimeState {
  const config = createDefaultZuanConfig();
  return {
    config,
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
    lastControlHook: null,
    zuanSeatCueBySeat: {},
    luckyDrawShimmer: false,
    bossDefeatedBanner: false,
  };
}

export function updateZuanConfigInState(
  state: ZuanUiRuntimeState,
  patch: Record<string, unknown>,
): ZuanUiRuntimeState {
  const { patch: valid } = validateZuanConfigPatch(patch, { strict: false });
  const config: ZuanUiConfigMap = { ...state.config, ...valid };
  const attackStyleRaw = String(config.zhishixian ?? "guanjie");
  const attackLineStyle: ZuanUiRuntimeState["attackLineStyle"] =
    attackStyleRaw === "classic" || attackStyleRaw === "laser" ? attackStyleRaw : "guanjie";
  const opiNum = Number.parseFloat(String(config.zhishiopi ?? "100"));
  return {
    ...state,
    config,
    attackLineStyle,
    attackLineOpacity: Number.isFinite(opiNum) ? Math.min(1, Math.max(0.1, opiNum / 100)) : 1.0,
    mutePaperAudio: Boolean(config.audio_draw),
    skipOpeningSplash: Boolean(config.helasisy),
    activeWallpaper: String(config.wallpaper ?? state.activeWallpaper),
  };
}

export function zuanArenaDatasetAttrs(state: ZuanUiRuntimeState): Record<string, string> {
  return {
    "data-zuan-seadream": state.seaDreamPhaseTint ? "on" : "off",
    "data-zuan-attack-line": state.attackLineStyle,
    "data-zuan-qiankuang": state.config.qiankuang ? "on" : "off",
    "data-zuan-perspective": state.config.perspective ? "on" : "off",
    "data-zuan-newui": state.config.newui ? "on" : "off",
    "data-zuan-wallpaper": state.activeWallpaper,
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
  const withControl = (next: ZuanUiRuntimeState): ZuanUiRuntimeState => ({
    ...next,
    lastControlHook: hook,
  });

  switch (hook) {
    case "_zuanattacklinecheck": {
      // extension.js:37514: syncs Decade attack-line preset & opacity
      const resolvedStyle: ZuanUiRuntimeState["attackLineStyle"] =
        state.config.zhishixian === "classic" || state.config.zhishixian === "laser"
          ? state.config.zhishixian
          : "guanjie";
      return withControl({
        ...state,
        attackLineStyle: resolvedStyle,
        zuanSeatCueBySeat: { ...state.zuanSeatCueBySeat, [seat]: `指示线:${resolvedStyle}` },
      });
    }
    case "_extragainskill":
    case "_extraloseCardskill":
    case "_extraloseskill":
    case "_extrauseskill":
      // extension.js:40597-40693: paper draw/discard/use audio gated by audio_draw mute config
      if (state.mutePaperAudio) {
        return withControl({
          ...state,
          zuanSeatCueBySeat: { ...state.zuanSeatCueBySeat, [seat]: "纸张静音" },
        });
      }
      return withControl({
        ...state,
        lastAudioCue: `paper:${hook}`,
        zuanSeatCueBySeat: { ...state.zuanSeatCueBySeat, [seat]: `牌效:${hook.replace("_extra", "")}` },
      });
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
    case "_linkskill": {
      // extension.js:40500-40925: event SFX & taunt voice cues
      const shortTag = hook.replace(/^_/, "").replace(/skill$/, "");
      return withControl({
        ...state,
        lastAudioCue: `sfx:${hook}@P${seat}`,
        zuanSeatCueBySeat: { ...state.zuanSeatCueBySeat, [seat]: `语音:${shortTag}` },
      });
    }
    case "_newBackgroundUse":
      // extension.js:41318: dynamic background rotation on card target completion
      return withControl({ ...state, activeWallpaper: opts?.wallpaperId ?? `shihai_seat_${seat}` });
    case "_zuanboss":
      // extension.js:41545: boss fall overlay banner
      return withControl({ ...state, bossDefeatedBanner: true });
    case "_helasisyskill":
      // extension.js:41890: skips opening splash animation when helasisy === true
      return withControl({ ...state, skipOpeningSplash: true, lastAudioCue: "dump_start" });
    case "_anshaskill":
      // extension.js:42659: removes assassination stealth mark at phaseEnd
      return withControl({
        ...state,
        anshaHiddenSeats: state.anshaHiddenSeats.filter((s) => s !== seat),
        zuanSeatCueBySeat: { ...state.zuanSeatCueBySeat, [seat]: "暗杀显影" },
      });
    case "_seadreammodeskill":
      // extension.js:42815: toggles 幻海 sea-dream phase ambient tint
      return withControl({ ...state, seaDreamPhaseTint: !state.seaDreamPhaseTint });
    case "_relifeskill":
      // extension.js:43402: updates revive countdown HUD badge
      return withControl({
        ...state,
        reviveCountdownBySeat: {
          ...state.reviveCountdownBySeat,
          [seat]: Math.max(0, opts?.reviveTurns ?? 3),
        },
      });
    case "_haoyunlai":
      // extension.js:47584: lucky-star drawBegin shimmer cue
      return withControl({
        ...state,
        luckyDrawShimmer: true,
        lastAudioCue: "haoyunlai",
        zuanSeatCueBySeat: { ...state.zuanSeatCueBySeat, [seat]: "✨好运来" },
      });
    case "_bingleBigScreen":
    case "_bingletexiaoshandian":
    case "_thundertexiao":
    case "_thundertexiao2":
      // extension.js:47880, 47984, 48389, 48503: lightning judgment full-screen overlay
      return withControl({ ...state, judgmentOverlay: "shandian", lastAudioCue: "judge:shandian" });
    case "_bingletexiaohuoshan":
      return withControl({ ...state, judgmentOverlay: "huoshan", lastAudioCue: "judge:huoshan" });
    case "_bingletexiaohongshui":
      return withControl({ ...state, judgmentOverlay: "hongshui", lastAudioCue: "judge:hongshui" });
    case "_bingletexiaofulei":
      return withControl({ ...state, judgmentOverlay: "fulei", lastAudioCue: "judge:fulei" });
    case "_bingletexiaole":
      return withControl({ ...state, judgmentOverlay: "lebusishu", lastAudioCue: "judge:lebusishu" });
    case "_bingletexiaobingliang":
      return withControl({ ...state, judgmentOverlay: "bingliang", lastAudioCue: "judge:bingliang" });
  }
}
