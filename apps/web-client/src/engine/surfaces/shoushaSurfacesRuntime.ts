/**
 * L14 手杀ui 40 项渲染面 (8 JS + 32 CSS) 清单、激活解析器与净室交互辅助。
 * 上游对照: `archive/upstream/noname-liuli/noname/extension/手杀ui/`
 */

import type { CardView, EngineEnvelope, SeatView } from "../../types/engine";
import { createDefaultShoushaConfig, type ShoushaUiConfig } from "./shoushaConfigSchema";
import type { ShoushaUiRuntimeState } from "./shoushaUiSurface";

export interface ShoushaSurfaceSpec {
  readonly id: string;
  readonly kind: "js" | "css";
  readonly upstreamFile: string;
  readonly upstreamLines: string;
  readonly role: string;
}

/** 全部 40 项 `L14:surfaces` 规格（8 JS + 32 CSS） */
export const SHOUSHA_SURFACE_SPECS: ReadonlyArray<ShoushaSurfaceSpec> = [
  // 8 JS surfaces
  { id: "character/main1.js", kind: "js", upstreamFile: "character/main1.js", upstreamLines: "1-444", role: "手杀官阶等级与玩家详情弹窗(v1)" },
  { id: "character/main1_new.js", kind: "js", upstreamFile: "character/main1_new.js", upstreamLines: "1-1048", role: "手杀新版官阶与资料页(ZLLT露头)" },
  { id: "character/main2.js", kind: "js", upstreamFile: "character/main2.js", upstreamLines: "1-290", role: "十周年布局资料页兼容插件" },
  { id: "lbtn/main1.js", kind: "js", upstreamFile: "lbtn/main1.js", upstreamLines: "1-1119", role: "手杀理牌(_paixu_paixu_paixu)与左下工具按钮" },
  { id: "lbtn/main2.js", kind: "js", upstreamFile: "lbtn/main2.js", upstreamLines: "1-598", role: "十周年布局手杀工具区与配件(XPJ)" },
  { id: "menu.js", kind: "js", upstreamFile: "menu.js", upstreamLines: "1-7692", role: "手杀美化菜单、壁纸(yangshi_bg)与开关样式(KGMH)" },
  { id: "skill/main1.js", kind: "js", upstreamFile: "skill/main1.js", upstreamLines: "1-1104", role: "手杀主副技能按钮分组(xiandingskill_ss)" },
  { id: "skill/main2.js", kind: "js", upstreamFile: "skill/main2.js", upstreamLines: "1-482", role: "十周年风格手杀技能栏布局" },

  // 32 CSS surfaces
  { id: "KGMH/kaiguan.css", kind: "css", upstreamFile: "KGMH/kaiguan.css", upstreamLines: "1-20", role: "手杀风格选项开关样式 (KGMH=1)" },
  { id: "KGMH/kaiguan_new.css", kind: "css", upstreamFile: "KGMH/kaiguan_new.css", upstreamLines: "1-24", role: "十周年风格选项开关样式 (KGMH=2)" },
  { id: "card/main.css", kind: "css", upstreamFile: "card/main.css", upstreamLines: "1-35", role: "手杀基础手牌边框样式" },
  { id: "card/main1.css", kind: "css", upstreamFile: "card/main1.css", upstreamLines: "1-14", role: "手杀手牌主样式入口" },
  { id: "card/main1_mobile.css", kind: "css", upstreamFile: "card/main1_mobile.css", upstreamLines: "1-35", role: "手杀手牌移动端间距与缩放" },
  { id: "card/main1_pc.css", kind: "css", upstreamFile: "card/main1_pc.css", upstreamLines: "1-35", role: "手杀手牌桌面端间距与缩放" },
  { id: "card/main2.css", kind: "css", upstreamFile: "card/main2.css", upstreamLines: "1-35", role: "十周年样式手杀牌面补充" },
  { id: "character/main1.css", kind: "css", upstreamFile: "character/main1.css", upstreamLines: "1-537", role: "手杀武将资料页样式(v1)" },
  { id: "character/main1_new.css", kind: "css", upstreamFile: "character/main1_new.css", upstreamLines: "1-525", role: "手杀新版武将详情页与官阶徽标样式" },
  { id: "character/main1_old.css", kind: "css", upstreamFile: "character/main1_old.css", upstreamLines: "1-537", role: "手杀经典武将详情页与官阶文本" },
  { id: "character/main2.css", kind: "css", upstreamFile: "character/main2.css", upstreamLines: "1-257", role: "十周年风格武将资料页适配" },
  { id: "dialog/main1.css", kind: "css", upstreamFile: "dialog/main1.css", upstreamLines: "1-105", role: "手杀选将与多选弹窗边框样式" },
  { id: "extension.css", kind: "css", upstreamFile: "extension.css", upstreamLines: "1-230", role: "进度条、阶段提示(JDTS)、狗托播报(GTBB)核心样式" },
  { id: "lbtn/main1.css", kind: "css", upstreamFile: "lbtn/main1.css", upstreamLines: "1-890", role: "手杀移动端牌桌按钮与理牌区样式" },
  { id: "lbtn/main1_boss.css", kind: "css", upstreamFile: "lbtn/main1_boss.css", upstreamLines: "1-896", role: "挑战/Boss模式手杀按键层" },
  { id: "lbtn/main1_window.css", kind: "css", upstreamFile: "lbtn/main1_window.css", upstreamLines: "1-861", role: "手杀桌面端底部控制栏样式" },
  { id: "lbtn/main2.css", kind: "css", upstreamFile: "lbtn/main2.css", upstreamLines: "1-748", role: "十周年样式工具条移动适配" },
  { id: "lbtn/main2_window.css", kind: "css", upstreamFile: "lbtn/main2_window.css", upstreamLines: "1-748", role: "十周年样式工具条桌面适配" },
  { id: "menu.css", kind: "css", upstreamFile: "menu.css", upstreamLines: "1-1803", role: "手杀风格菜单壁纸与选项卡(样式1)" },
  { id: "menu2.css", kind: "css", upstreamFile: "menu2.css", upstreamLines: "1-1677", role: "手杀青铜古风菜单(样式2)" },
  { id: "menu3.css", kind: "css", upstreamFile: "menu3.css", upstreamLines: "1-1734", role: "手杀新杀春日菜单(样式3)" },
  { id: "qidongye/layout_new.css", kind: "css", upstreamFile: "qidongye/layout_new.css", upstreamLines: "1-97", role: "启动页新版紧凑布局" },
  { id: "qidongye/layout_old.css", kind: "css", upstreamFile: "qidongye/layout_old.css", upstreamLines: "1-97", role: "启动页经典布局" },
  { id: "qidongye/layout_xbig.css", kind: "css", upstreamFile: "qidongye/layout_xbig.css", upstreamLines: "1-99", role: "启动页大图宽屏布局" },
  { id: "skill/main1.css", kind: "css", upstreamFile: "skill/main1.css", upstreamLines: "1-952", role: "手杀技能栏移动端样式" },
  { id: "skill/main1_window.css", kind: "css", upstreamFile: "skill/main1_window.css", upstreamLines: "1-652", role: "手杀技能栏桌面端样式" },
  { id: "skill/main2.css", kind: "css", upstreamFile: "skill/main2.css", upstreamLines: "1-243", role: "十周年风格手杀技能栏移动样式" },
  { id: "skill/main2_window.css", kind: "css", upstreamFile: "skill/main2_window.css", upstreamLines: "1-262", role: "十周年风格手杀技能栏桌面样式" },
  { id: "style.css", kind: "css", upstreamFile: "style.css", upstreamLines: "1-405", role: "手杀新版主选将与牌桌核心皮肤" },
  { id: "style2.css", kind: "css", upstreamFile: "style2.css", upstreamLines: "1-444", role: "手杀十周年混搭选将样式" },
  { id: "style_boss.css", kind: "css", upstreamFile: "style_boss.css", upstreamLines: "1-432", role: "手杀挑战模式选将样式" },
  { id: "style_old.css", kind: "css", upstreamFile: "style_old.css", upstreamLines: "1-381", role: "手杀旧版选将边框样式" },
];

export function resolveActiveShoushaSurfaces(
  cfg: ShoushaUiConfig = createDefaultShoushaConfig(),
  opts?: { mobileViewport?: boolean; bossMode?: boolean },
): ReadonlyArray<string> {
  const isShousha = cfg.yangshi === "on";
  const mobile = Boolean(opts?.mobileViewport);
  const active = new Set<string>([
    "extension.css",
    "dialog/main1.css",
    "card/main.css",
    "card/main1.css",
    "menu.js",
    mobile ? "card/main1_mobile.css" : "card/main1_pc.css",
  ]);
  if (isShousha) {
    active.add("lbtn/main1.js");
    active.add("skill/main1.js");
    if (cfg.charactermenu !== "none") {
      active.add(
        cfg.charactermenu === "old" ? "character/main1.js" : "character/main1_new.js",
      );
      active.add(
        cfg.charactermenu === "old" ? "character/main1_old.css" : "character/main1_new.css",
      );
    } else {
      active.add("character/main1.js");
    }
    active.add("character/main1.css");
    active.add(opts?.bossMode ? "lbtn/main1_boss.css" : mobile ? "lbtn/main1.css" : "lbtn/main1_window.css");
    active.add(mobile ? "skill/main1.css" : "skill/main1_window.css");
  } else {
    active.add("character/main2.js");
    active.add("character/main2.css");
    active.add("lbtn/main2.js");
    active.add(mobile ? "lbtn/main2.css" : "lbtn/main2_window.css");
    active.add("skill/main2.js");
    active.add(mobile ? "skill/main2.css" : "skill/main2_window.css");
    active.add("card/main2.css");
  }

  if (cfg.KGMH === "1") active.add("KGMH/kaiguan.css");
  else if (cfg.KGMH === "2") active.add("KGMH/kaiguan_new.css");

  if (cfg.yangshi_bg === "1") active.add("menu2.css");
  else if (cfg.yangshi_bg === "2" || cfg.yangshi_bg === "3") active.add("menu3.css");
  else active.add("menu.css");

  // extension.js:168-177 exact qiDongYe mapping: on->layout_old, othersOn->layout_new, othersTwo->layout_new+layout_xbig
  if (cfg.qiDongYe === "on") {
    active.add("qidongye/layout_old.css");
  } else if (cfg.qiDongYe === "othersOn") {
    active.add("qidongye/layout_new.css");
  } else if (cfg.qiDongYe === "othersTwo") {
    active.add("qidongye/layout_new.css");
    active.add("qidongye/layout_xbig.css");
  }

  if (opts?.bossMode) active.add("style_boss.css");
  else if (cfg.xindchoose === "shizhounian") active.add("style2.css");
  else if (cfg.xindchoose_type === "old") active.add("style_old.css");
  else active.add("style.css");

  return Array.from(active);
}

/** `character/main1.js:46-74` 手杀官阶映射 */
export function resolveShoushaGuanjie(seat: Pick<SeatView, "identity" | "max_hp" | "hp">, aliveCount: number): {
  key: string;
  label: string;
} {
  if (seat.identity === "Lord") return { key: "dayuanshuai", label: "大元帅" };
  if (aliveCount >= 7) return { key: "dajiangjun", label: "大将军" };
  if (aliveCount >= 5) return { key: "lingjunjiangjun", label: "领军" };
  if (seat.max_hp >= 4) return { key: "xianfengjiangjun", label: "先锋" };
  return { key: "xiaowei", label: "校尉" };
}

/** `extension.js:1968-2101` `GTBB` 滚动播报语录（本地净室文案） */
export const SHOUSHA_GTBB_MESSAGES: ReadonlyArray<string> = [
  "手杀播报 · 乱世豪杰齐聚牌桌",
  "手杀播报 · 逢凶化吉，手牌如虹",
  "手杀播报 · 天下归心，蓄势待发",
  "手杀播报 · 兵贵神速，一击定音",
];

export function resolveShoushaGtbbBanner(cfg: ShoushaUiConfig, index = 0): string | null {
  if (!cfg.GTBB) return null;
  const msg = SHOUSHA_GTBB_MESSAGES[Math.abs(index) % SHOUSHA_GTBB_MESSAGES.length];
  return cfg.GTBBYangshi === "off" ? `【十周年播报】${msg.replace("手杀播报 · ", "")}` : msg;
}

/** `extension.js:3657` `sayplay` 常用语与鲜花/鸡蛋表情（零音频二进制依赖） */
export const SHOUSHA_SAY_WORDS: ReadonlyArray<string> = [
  "快点吧，我等得花儿都谢了！",
  "无懈可击，请君再思！",
  "得饶人处且饶人~",
  "这一波，稳稳拿下！",
];

/** `extension.js:5999` `xinddraw` 手气卡评分（纯粹手牌花色/基本牌多样度 UI 评估，不影响发牌规则） */
export function computeHandLuckScore(hand: ReadonlyArray<CardView>): number {
  if (hand.length === 0) return 0;
  const suits = new Set(hand.map((c) => c.suit).filter(Boolean));
  const kinds = new Set(hand.map((c) => c.kind));
  const hasPeachOrNull = hand.some((c) => c.kind === "Tao" || c.kind === "TrickWuxie");
  const raw = 45 + suits.size * 8 + Math.min(20, kinds.size * 4) + (hasPeachOrNull ? 8 : 0);
  return Math.max(10, Math.min(99, raw));
}

const PHASE_TO_HOOK = {
  Prepare: "_jd_hhks",
  Judge: "_jd_pdjd",
  Draw: "_jd_mpjd",
  Play: "_jd_cpjd",
  Discard: "_jd_qpjd",
  End: "_jd_hhjs",
  Idle: "_jd_phaseAfter",
} as const;

export function syncShoushaUiFromEnvelope(
  prev: ShoushaUiRuntimeState,
  env: EngineEnvelope,
  selfId = 0,
): ShoushaUiRuntimeState {
  const view = env.view;
  if (!view) return prev;
  const cfg = prev.config;
  const phaseHook = PHASE_TO_HOOK[view.phase] ?? "_jd_phaseAfter";
  const phaseCode =
    phaseHook === "_jd_phaseAfter" ? "idle" : (phaseHook.slice(4) as ShoushaUiRuntimeState["phaseRibbon"]);
  const phaseLabels: Record<ShoushaUiRuntimeState["phaseRibbon"], string> = {
    idle: "待机",
    hhks: "回合开始",
    pdjd: "判定阶段",
    mpjd: "摸牌阶段",
    cpjd: "出牌阶段",
    qpjd: "弃牌阶段",
    hhjs: "回合结束",
  };

  const cur = view.current_player;
  const selfSeat = (view.seats ?? []).find((s) => s.player === selfId);
  const tuoguanActive = selfSeat && !selfSeat.alive ? false : prev.tuoguanActive;
  const progressVisible = cfg.jindutiao && (cur === selfId ? cfg.jindutiaoUpdata : cfg.jindutiaoaiUpdata);

  const playPromptText = !cfg.JDTS
    ? null
    : view.phase === "Play"
      ? cur === selfId
        ? "出牌阶段，请选择一张牌"
        : `P${cur} 出牌中...`
      : view.phase === "Discard"
        ? cur === selfId
          ? "弃牌阶段，请弃置手牌"
          : `P${cur} 弃牌中...`
        : `${phaseLabels[phaseCode]}`;

  const seatPromptBySeat: Record<number, string> = {};
  if (cfg.JDTS && cur !== selfId && (view.phase === "Play" || view.phase === "Discard")) {
    seatPromptBySeat[cur] = view.phase === "Play" ? "出牌中..." : "弃牌中...";
  }
  const waitingSeats: Record<number, string> = {};
  if (cfg.JDTS && env.choice && env.choice.observer !== selfId) {
    waitingSeats[env.choice.observer] = "等待响应...";
  }

  let wuxiePromptActive = false;
  for (const rawEv of env.events ?? []) {
    const evType = (rawEv as { type?: string })?.type;
    const cardKind = (rawEv as { card?: string; kind?: string })?.card ?? (rawEv as { kind?: string })?.kind;
    if (evType === "CardPlayed" && typeof cardKind === "string" && cardKind.startsWith("Trick")) {
      wuxiePromptActive = true;
    }
  }

  const selfHand = cfg.xinddraw
    ? (view.known_cards ?? []).filter((c) => c.owner === selfId && c.zone === "Hand")
    : [];

  return {
    ...prev,
    styleVariant: cfg.yangshi === "on" ? "shousha" : "decade",
    tuoguanActive,
    progressVisible,
    progressTargetSeat: progressVisible ? cur : null,
    phaseRibbon: phaseCode,
    phaseRibbonLabel: cfg.JDTS ? phaseLabels[phaseCode] : "",
    playPromptText,
    waitingSeats,
    seatPromptBySeat,
    wuxiePromptActive,
    skillControlRev: prev.skillControlRev + (view.phase === "End" ? 1 : 0),
    zlltOutcropEnabled: prev.zlltOutcropEnabled,
    activeSurfaces: resolveActiveShoushaSurfaces(cfg),
    lastGtbbText: resolveShoushaGtbbBanner(cfg, view.round),
    handLuckScore: cfg.xinddraw && selfHand.length > 0 ? computeHandLuckScore(selfHand) : null,
  };
}
