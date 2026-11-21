/**
 * L08 十周年UI 全部 43 项渲染面运行时（10 JS 模块 + 33 CSS 样式表）
 * 上游对照（净室对照，严格不读取 `spine.js`）：
 * - `component.js` (slider, chatBox), `content.js` (chooseGuanXing), `effect.js` (compare/line/kill/skill),
 * - `pause.js` (暂停历史), `menu.js` (44配置菜单与本地配置导入), `splash.js` (开战/结算/双将横幅),
 * - `meihua.js` (美化/重铸/虚拟装备), `animation.js` / `animation_new.js` / `animationlist.js` (净室动画排队与回退),
 * - 33 份 CSS 样式表规则激活映射。
 */

import type { CardView, ObserverSnapshot } from "../../types/engine";
import {
  createDefaultDecadeConfig,
  normalizeDecadeConfig,
  validateDecadeConfigPatch,
  type DecadeGameStartPreset,
  type DecadeUiConfig,
} from "./decadeConfigSchema";

export type { DecadeUiConfig };

export interface DecadeCssSurfaceSpec {
  readonly id: string;
  readonly role: string;
  readonly upstreamLines: string;
}

/** 全部 33 份 L08 CSS 渲染面规格 */
export const DECADE_CSS_SURFACE_SPECS: ReadonlyArray<DecadeCssSurfaceSpec> = [
  { id: "animation.css", role: "动画容器与动皮挂载层定位", upstreamLines: "1-251" },
  { id: "animation1.css", role: "特写立绘动画叠层样式", upstreamLines: "1-214" },
  { id: "animation_new.css", role: "新版动画画布与层级控制", upstreamLines: "1-230" },
  { id: "card.css", role: "基础卡牌排版/角标/牌名辅助切换", upstreamLines: "1-323" },
  { id: "cardX.css", role: "扩充卡牌花色/属性/特效边框样式", upstreamLines: "1-450" },
  { id: "component.css", role: "滑块与牌桌聊天窗组件样式", upstreamLines: "1-158" },
  { id: "decadeLayout.css", role: "2–8 人环绕式桌面牌桌座位坐标", upstreamLines: "1-534" },
  { id: "dialog.css", role: "标准对局选择框与 loading 框皮肤", upstreamLines: "1-1116" },
  { id: "dialog_new.css", role: "十周年新版选择对话框与按钮排版", upstreamLines: "1-1180" },
  { id: "doudizhu.css", role: "斗地主/身份局特殊主题适配", upstreamLines: "1-135" },
  { id: "effect.css", role: "拼点/破敌击杀/技能特写全屏特效", upstreamLines: "1-424" },
  { id: "equip.css", role: "经典装备区条目与单独装备栏样式", upstreamLines: "1-372" },
  { id: "equip_new.css", role: "十周年独立/紧凑装备槽布局", upstreamLines: "1-190" },
  { id: "font.css", role: "牌面/武将名楷体与数字字体栈", upstreamLines: "1-55" },
  { id: "icon.css", role: "花色/装备/身份与状态徽标矢量映射", upstreamLines: "1-105" },
  { id: "junba.css", role: "军八模式主题美化规则", upstreamLines: "1-559" },
  { id: "layout.css", role: "传统布局手牌边框与控制区定位", upstreamLines: "1-857" },
  { id: "layout_new.css", role: "十周年底部手牌/提示/操作区对齐", upstreamLines: "1-781" },
  { id: "meihua.css", role: "按钮与远位座位细节美化补充", upstreamLines: "1-334" },
  { id: "newdialog.css", role: "扩展交互弹窗容器样式", upstreamLines: "1-619" },
  { id: "newdialog_new.css", role: "十周年观星/选牌多区弹窗布局", upstreamLines: "1-793" },
  { id: "pause.css", role: "暂停历史记录侧栏与角色头像条", upstreamLines: "1-352" },
  { id: "player.css", role: "基础玩家框与体力条框架", upstreamLines: "1-1789" },
  { id: "player1.css", role: "手杀风格玩家框预设 1（移动端）", upstreamLines: "1-1363" },
  { id: "player1_PC.css", role: "手杀风格玩家框预设 1（桌面端）", upstreamLines: "1-1363" },
  { id: "player2.css", role: "手杀风格玩家框预设 2（移动端）", upstreamLines: "1-1475" },
  { id: "player2_PC.css", role: "手杀风格玩家框预设 2（桌面端）", upstreamLines: "1-1474" },
  { id: "player3.css", role: "手杀风格玩家框预设 3（移动端）", upstreamLines: "1-1361" },
  { id: "player3_PC.css", role: "手杀风格玩家框预设 3（桌面端）", upstreamLines: "1-1360" },
  { id: "player_mark.css", role: "十周年技能角标/转换技标签基础样式", upstreamLines: "1-151" },
  { id: "player_mark_fix.css", role: "手杀与十周年混排标记防重叠修正", upstreamLines: "1-109" },
  { id: "player_new.css", role: "十周年主玩家框/身份角标/露头/等阶边框", upstreamLines: "1-1687" },
  { id: "splash.css", role: "开战横幅/胜负结算/国战隐匿与双将框架", upstreamLines: "1-493" },
];

export const DECADE_JS_SURFACE_IDS = [
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
] as const;

export interface DecadeAnimationCatalogEntry {
  readonly name: string;
  readonly category: "intro" | "skill" | "equip" | "trick" | "status" | "combat";
  readonly follow?: boolean;
  readonly fallbackLabel: string;
}

/** 对应 `animationlist.js:1-178` 的净室动画名录（含无骨骼时确定性 Canvas/CSS 回退描述） */
export const DECADE_ANIMATION_CATALOG: ReadonlyArray<DecadeAnimationCatalogEntry> = [
  { name: "neijian", category: "intro", fallbackLabel: "主忠对决" },
  { name: "effect_youxikaishi_SSdys", category: "intro", fallbackLabel: "手杀大元帅开战" },
  { name: "shoushakaizhan", category: "intro", fallbackLabel: "手杀开战" },
  { name: "effect_youxikaishi_SZN", category: "intro", fallbackLabel: "十周年开战" },
  { name: "effect_youxikaishi_SZNdys", category: "intro", fallbackLabel: "十周年大元帅开战" },
  { name: "effect_youxikaishi_SZNdjj", category: "intro", fallbackLabel: "十周年大将军开战" },
  { name: "SF_kaizhan_eff_dasima", category: "intro", fallbackLabel: "新十周年大司马开战" },
  { name: "effect_jineng_SZN", category: "skill", fallbackLabel: "十周年技能发动" },
  { name: "effect_jineng_SS_1", category: "skill", fallbackLabel: "手杀技能发光" },
  { name: "SS_zhuanhuanji", category: "skill", fallbackLabel: "阴阳转换技翻转" },
  { name: "juexingji", category: "skill", fallbackLabel: "觉醒技震撼" },
  { name: "xiandingji", category: "skill", fallbackLabel: "限定技特写" },
  { name: "shimingji", category: "skill", fallbackLabel: "使命技达成" },
  { name: "effect_xianding", category: "skill", fallbackLabel: "十周年限定技框" },
  { name: "effect_zhiliao", category: "status", fallbackLabel: "木系治疗光环" },
  { name: "kapaizhuanhuan", category: "status", fallbackLabel: "卡牌转化流光" },
  { name: "gaipan", category: "status", fallbackLabel: "改判高亮" },
  { name: "jiesuan", category: "intro", fallbackLabel: "胜负结算" },
  { name: "effect_baguazhen", category: "equip", fallbackLabel: "八卦阵判定光效" },
  { name: "effect_baiyinshizi", category: "equip", fallbackLabel: "白银狮子护盾" },
  { name: "effect_zhugeliannu", category: "equip", fallbackLabel: "诸葛连弩连射" },
  { name: "effect_fangtianhuaji", category: "equip", fallbackLabel: "方天画戟斩击" },
  { name: "effect_qinglongyanyuedao", category: "equip", fallbackLabel: "青龙偃月刀追击" },
  { name: "effect_renwangdun", category: "equip", fallbackLabel: "仁王盾格挡" },
  { name: "effect_jisha1", category: "combat", fallbackLabel: "破敌击杀" },
  { name: "SF_guanjie_eff_jisha", category: "combat", fallbackLabel: "官阶破敌横幅" },
  { name: "SS_zhenwang", category: "combat", fallbackLabel: "手杀阵亡印记" },
  { name: "SZN_zhenwang", category: "combat", fallbackLabel: "十周年阵亡灰化" },
  { name: "aar_chupaizhishiX", category: "combat", fallbackLabel: "出牌目标指示" },
  { name: "SS_jiuwo", category: "status", fallbackLabel: "濒死救我呼救" },
  { name: "SZN_jiuwo", category: "status", fallbackLabel: "十周年救我" },
  { name: "jiubuff", category: "status", fallbackLabel: "酒态增伤烈焰" },
  { name: "SZN_loseHp", category: "combat", fallbackLabel: "失去体力震颤" },
  { name: "SZN_nanmanruqin", category: "trick", fallbackLabel: "南蛮入侵齐出" },
  { name: "SZN_taoyuanjieyi", category: "trick", fallbackLabel: "桃园结义共愈" },
  { name: "effect_wanjianqifa_full", category: "trick", fallbackLabel: "万箭齐发齐射" },
  { name: "effect_huosha", category: "trick", follow: true, fallbackLabel: "火杀焰纹" },
  { name: "effect_leisha", category: "trick", follow: true, fallbackLabel: "雷杀电弧" },
  { name: "effect_tiesuolianhuan", category: "trick", follow: true, fallbackLabel: "铁索连环" },
  { name: "effect_wuxiekeji", category: "trick", follow: true, fallbackLabel: "无懈可击抵消" },
];

export interface DecadeChatMessage {
  readonly id: number;
  readonly sender: string;
  readonly text: string;
}

export interface DecadeChatBoxState {
  readonly folded: boolean;
  readonly messages: ReadonlyArray<DecadeChatMessage>;
}

export interface DecadeGuanXingCard {
  readonly id: number;
  readonly label: string;
  readonly suit: string;
  readonly rank: string;
}

export interface DecadeGuanXingState {
  readonly open: boolean;
  readonly topCards: ReadonlyArray<DecadeGuanXingCard>;
  readonly bottomCards: ReadonlyArray<DecadeGuanXingCard>;
  readonly topLocked: boolean;
  readonly bottomLocked: boolean;
  readonly selectedCardId: number | null;
}

export interface DecadeCompareEffectState {
  readonly sourceName: string;
  readonly targetName: string;
  readonly sourceCardLabel: string;
  readonly targetCardLabel: string;
  readonly winner: "source" | "target" | "tie";
}

export interface DecadeLineArcState {
  readonly fromSeat: number;
  readonly toSeat: number;
  readonly style: string;
}

export interface DecadeSkillPopupState {
  readonly seat: number;
  readonly generalName: string;
  readonly skillName: string;
  readonly tier: "limited" | "awakening" | "standard";
}

export interface DecadePauseHistoryRow {
  readonly seat: number;
  readonly name: string;
  readonly hpText: string;
  readonly alive: boolean;
  readonly handCount: number;
}

/** 根据当前 44 项配置解析应激活的 CSS 渲染面模块列表 (`33` 份 CSS 子集精确映射) */
export function resolveActiveDecadeCssSurfaces(
  cfg: DecadeUiConfig,
  opts?: { mobileViewport?: boolean },
): ReadonlyArray<string> {
  const active = new Set<string>([
    "animation.css",
    "animation1.css",
    "animation_new.css",
    "card.css",
    "cardX.css",
    "component.css",
    "effect.css",
    "font.css",
    "icon.css",
    "meihua.css",
    "newdialog.css",
    "newdialog_new.css",
    "pause.css",
    "player_mark.css",
    "splash.css",
  ]);
  active.add(cfg.decadeLayout === "on" ? "decadeLayout.css" : "layout.css");
  if (cfg.newDecadeStyle === "on") {
    active.add("layout_new.css");
    active.add("dialog_new.css");
    active.add("equip_new.css");
    active.add("player_new.css");
  } else {
    active.add("dialog.css");
    active.add("equip.css");
    active.add("player.css");
    if (opts?.mobileViewport) {
      active.add("player1.css");
      active.add("player2.css");
      active.add("player3.css");
    } else {
      active.add("player1_PC.css");
      active.add("player2_PC.css");
      active.add("player3_PC.css");
    }
  }
  if (cfg.shoushaNewNewMark || cfg.shoushaNewMark) {
    active.add("player_mark_fix.css");
  }
  if (cfg.JBDDZ) {
    active.add("doudizhu.css");
    active.add("junba.css");
  }
  return Array.from(active);
}

/** 上游 `component.js:4-33` 滑块数值归一化 */
export function clampSliderValue(min: number, max: number, value: number): { value: number; percent: number } {
  const lo = Math.min(min, max);
  const hi = Math.max(min, max);
  const clamped = Math.max(lo, Math.min(hi, value));
  const span = Math.max(1e-6, hi - lo);
  return {
    value: clamped,
    percent: Math.round(((clamped - lo) / span) * 1000) / 10,
  };
}

/** 上游 `component.js:34-114` 牌桌聊天框状态与消息追加 */
export function createInitialChatBoxState(): DecadeChatBoxState {
  return {
    folded: true,
    messages: [{ id: 1, sender: "系统", text: "十周年牌桌已就绪（点击展开快捷发言）" }],
  };
}

export function appendChatMessage(
  state: DecadeChatBoxState,
  sender: string,
  text: string,
): DecadeChatBoxState {
  const cleanText = text.trim().slice(0, 120);
  if (!cleanText) return state;
  const nextId = (state.messages.at(-1)?.id ?? 0) + 1;
  return {
    ...state,
    messages: [...state.messages.slice(-29), { id: nextId, sender: sender || "玩家", text: cleanText }],
  };
}

/** 上游 `content.js:4-240` `chooseGuanXing` 状态构造与牌堆顶/底交换切换 */
export function createGuanXingState(
  top: ReadonlyArray<DecadeGuanXingCard>,
  bottom: ReadonlyArray<DecadeGuanXingCard> = [],
): DecadeGuanXingState {
  return {
    open: true,
    topCards: [...top],
    bottomCards: [...bottom],
    topLocked: false,
    bottomLocked: false,
    selectedCardId: null,
  };
}

export function swapGuanXingCards(
  state: DecadeGuanXingState,
  sourceId: number,
  targetId: number,
): DecadeGuanXingState {
  if (sourceId === targetId) return state;
  const top = [...state.topCards];
  const bottom = [...state.bottomCards];
  const sTop = top.findIndex((c) => c.id === sourceId);
  const sBot = bottom.findIndex((c) => c.id === sourceId);
  const tTop = top.findIndex((c) => c.id === targetId);
  const tBot = bottom.findIndex((c) => c.id === targetId);

  if (sTop >= 0 && tTop >= 0 && !state.topLocked) {
    [top[sTop], top[tTop]] = [top[tTop]!, top[sTop]!];
  } else if (sBot >= 0 && tBot >= 0 && !state.bottomLocked) {
    [bottom[sBot], bottom[tBot]] = [bottom[tBot]!, bottom[sBot]!];
  } else if (sTop >= 0 && tBot >= 0 && !state.topLocked && !state.bottomLocked) {
    [top[sTop], bottom[tBot]] = [bottom[tBot]!, top[sTop]!];
  } else if (sBot >= 0 && tTop >= 0 && !state.topLocked && !state.bottomLocked) {
    [bottom[sBot], top[tTop]] = [top[tTop]!, bottom[sBot]!];
  }
  return { ...state, topCards: top, bottomCards: bottom, selectedCardId: null };
}

export function moveGuanXingCardRow(state: DecadeGuanXingState, cardId: number): DecadeGuanXingState {
  if (state.topLocked || state.bottomLocked) return state;
  const topIdx = state.topCards.findIndex((c) => c.id === cardId);
  if (topIdx >= 0) {
    const card = state.topCards[topIdx]!;
    return {
      ...state,
      topCards: state.topCards.filter((c) => c.id !== cardId),
      bottomCards: [...state.bottomCards, card],
      selectedCardId: null,
    };
  }
  const botIdx = state.bottomCards.findIndex((c) => c.id === cardId);
  if (botIdx >= 0) {
    const card = state.bottomCards[botIdx]!;
    return {
      ...state,
      bottomCards: state.bottomCards.filter((c) => c.id !== cardId),
      topCards: [...state.topCards, card],
      selectedCardId: null,
    };
  }
  return state;
}

/** 上游 `splash.js` / `extension.js:12379-12420` 开战/结算标题文案解析 */
export function resolveGameStartBanner(preset: DecadeGameStartPreset): string | null {
  if (preset === "off") return null;
  if (preset.includes("dasima")) return "大司马出阵 · 战局开启";
  if (preset.includes("dajiangjun") || preset.includes("djj")) return "大将军点兵 · 对局开始";
  if (preset.includes("dys")) return "大元帅统军 · 开战";
  if (preset.includes("jiangjun")) return "将军出征 · 开战";
  if (preset.includes("shizhounian")) return "十周年 · 乱世启幕";
  return "手杀开战 · 旗开得胜";
}

export function resolveGameOverBanner(
  preset: DecadeUiConfig["SanGuoShaGameOver"],
  won: boolean,
): string | null {
  if (preset === "off") return null;
  const prefix = preset === "shizhounian" ? "十周年结算" : "手杀结算";
  return won ? `${prefix} · 凯旋胜利` : `${prefix} · 败北惜退`;
}

/** 上游 `pause.js:219-286` 暂停历史视图所需的场上全部角色摘要 */
export function buildPauseHistoryRows(view: ObserverSnapshot | null): DecadePauseHistoryRow[] {
  if (!view) return [];
  return [...view.seats]
    .sort((a, b) => a.seat - b.seat)
    .map((s) => ({
      seat: s.seat,
      name: s.generals?.[0]?.name || `P${s.player}`,
      hpText: `${Math.max(s.hp, 0)}/${s.max_hp}`,
      alive: s.alive,
      handCount: s.hand_count,
    }));
}

/**
 * 裁决 38 本地配置 JSON 导入校验（`menu.js` 扩展设置导入安全边界：拒绝含脚本/非 JSON 输入）。
 */
export function parseDecadeConfigImportJson(raw: string): {
  patch: Partial<DecadeUiConfig>;
  config: DecadeUiConfig;
  importedKeys: number;
} {
  const trimmed = raw.trim();
  if (!trimmed || trimmed.includes("<script") || trimmed.includes("javascript:")) {
    throw new Error("DECADE_CONFIG_IMPORT_UNSAFE_OR_EMPTY");
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(trimmed);
  } catch {
    throw new Error("DECADE_CONFIG_IMPORT_INVALID_JSON");
  }
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("DECADE_CONFIG_IMPORT_OBJECT_REQUIRED");
  }
  const obj = parsed as Record<string, unknown>;
  const { patch, validKeys } = validateDecadeConfigPatch(obj, { strict: true });
  const base = createDefaultDecadeConfig();
  const config = normalizeDecadeConfig({ ...base, ...patch });
  return {
    patch,
    config,
    importedKeys: validKeys,
  };
}

export function cardsToGuanXingItems(
  cards: ReadonlyArray<CardView>,
  labelOf: (kind: CardView["kind"], def?: number | null) => string,
): DecadeGuanXingCard[] {
  const suitMap: Record<string, string> = {
    Spade: "♠",
    Heart: "♥",
    Club: "♣",
    Diamond: "♦",
  };
  return cards.slice(0, 5).map((c) => ({
    id: c.id,
    label: labelOf(c.kind, c.def),
    suit: (c.suit && suitMap[c.suit]) || "?",
    rank:
      c.rank === 1
        ? "A"
        : c.rank === 11
          ? "J"
          : c.rank === 12
            ? "Q"
            : c.rank === 13
              ? "K"
              : String(c.rank ?? "?"),
  }));
}
