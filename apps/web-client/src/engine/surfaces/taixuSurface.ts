/**
 * L11 太虚幻境 (`archive/upstream/noname-liuli/noname/extension/太虚幻境/`)
 * Authentic native frontend surface runtime for the 4 upstream stylesheets:
 * - `extension_MobileStyle.css:1-494` (`rem`-scaled mobile touch typography & compact console desk)
 * - `extension_PcStyle.css:1-353` (`vw`-scaled widescreen PC typography & coin/rank console bar)
 * - `extension_servant.css:1-310` (`taixuhuanjing_chooseServantDiv` 80%×80% modal & `frame0..frame4` 21-servant gallery)
 * - `extension_style.css:1-6305` (`taixuhuanjing_collectHomeBody` 1920×1080 handbook, 8-DLC season chapter map & merchant shop)
 */

import type { EngineEnvelope } from "../../types/engine";
import { lookupCardKindByDef, lookupCardKindByKey } from "./cardCatalogLabels";

export type TaixuStylesheetId =
  | "extension_MobileStyle.css"
  | "extension_PcStyle.css"
  | "extension_servant.css"
  | "extension_style.css";

export const TAIXU_SURFACE_STYLESHEETS: ReadonlyArray<TaixuStylesheetId> = [
  "extension_MobileStyle.css",
  "extension_PcStyle.css",
  "extension_servant.css",
  "extension_style.css",
];

export type TaixuViewportMode = "pc" | "mobile";

export interface TaixuSeasonChapterSpec {
  id: string;
  dlcFolder: string;
  title: string;
  chapterCount: number;
  upstreamRef: string;
}

/**
 * All 8 DLC seasons from `太虚幻境/dlc/<season>/extension_season.js` rendered on `extension_style.css`.
 */
export const TAIXU_DLC_SEASONS: ReadonlyArray<TaixuSeasonChapterSpec> = [
  { id: "ChongYingChuLin", dlcFolder: "ChongYingChuLin", title: "冲应初临", chapterCount: 4, upstreamRef: "dlc/ChongYingChuLin/extension_season.js:8" },
  { id: "GFHuangJinZhiLuan", dlcFolder: "GFHuangJinZhiLuan", title: "黄巾之乱", chapterCount: 4, upstreamRef: "dlc/GFHuangJinZhiLuan/extension_season.js:8" },
  { id: "GFQiQinMengHuo", dlcFolder: "GFQiQinMengHuo", title: "七擒孟获", chapterCount: 4, upstreamRef: "dlc/GFQiQinMengHuo/extension_season.js:8" },
  { id: "HaiWaiFengHe", dlcFolder: "HaiWaiFengHe", title: "海外逢何", chapterCount: 4, upstreamRef: "dlc/HaiWaiFengHe/extension_season.js:8" },
  { id: "HeJinZhuHuan", dlcFolder: "HeJinZhuHuan", title: "何进诛宦", chapterCount: 4, upstreamRef: "dlc/HeJinZhuHuan/extension_season.js:8" },
  { id: "HuangTianZhiNu", dlcFolder: "HuangTianZhiNu", title: "黄天之怒", chapterCount: 4, upstreamRef: "dlc/HuangTianZhiNu/extension_season.js:8" },
  { id: "LiGuoZhiLuan", dlcFolder: "LiGuoZhiLuan", title: "李郭之乱", chapterCount: 4, upstreamRef: "dlc/LiGuoZhiLuan/extension_season.js:8" },
  { id: "MGBaWangZhengCheng", dlcFolder: "MGBaWangZhengCheng", title: "霸王征程", chapterCount: 4, upstreamRef: "dlc/MGBaWangZhengCheng/extension_season.js:8" },
];

export interface TaixuServantActionSpec {
  daiji: number;
  gongji: number;
  jingzhi: boolean;
  jinnang: number;
  shouji: number;
  zuodian: boolean;
}

export type TaixuServantBattleCue =
  | { kind: "_slDamage"; amount: number; isLeft?: boolean }
  | { kind: "_slUseCard"; cardType?: string; cardName?: string }
  | { kind: "_slHurt"; variant?: number };

export interface TaixuServantCardSpec {
  id: string;
  label: string;
  frameTier: 0 | 1 | 2 | 3 | 4;
  skills: ReadonlyArray<string>;
  action: TaixuServantActionSpec;
}

/**
 * 21 authentic servants from `太虚幻境/extension_servant.js:350-1760` styled via `extension_servant.css:40-75`,
 * including exact `action` animation clip specs from `extension_servant.js:1329-1505`.
 */
export const TAIXU_SERVANT_SPECS: ReadonlyArray<TaixuServantCardSpec> = [
  { id: "dundun", label: "盾盾", frameTier: 1, skills: ["txhj_dundunSkill1", "txhj_dundunSkill2"], action: { daiji: 3, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: true } },
  { id: "jiuwei", label: "九尾", frameTier: 3, skills: ["txhj_jiuweiSkill1", "txhj_jiuweiSkill2", "txhj_jiuweiSkill3"], action: { daiji: 4, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: true } },
  { id: "tengshe", label: "腾蛇", frameTier: 3, skills: ["txhj_tengsheSkill1", "txhj_tengsheSkill2", "txhj_tengsheSkill3"], action: { daiji: 4, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: true } },
  { id: "qilin", label: "麒麟", frameTier: 3, skills: ["txhj_qilinSkill1", "txhj_qilinSkill2", "txhj_qilinSkill3"], action: { daiji: 4, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: true } },
  { id: "diting", label: "谛听", frameTier: 3, skills: ["txhj_ditingSkill1", "txhj_ditingSkill2", "txhj_ditingSkill3"], action: { daiji: 4, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: true } },
  { id: "yaya", label: "鸭鸭", frameTier: 2, skills: ["txhj_yayaSkill1", "txhj_yayaSkill2"], action: { daiji: 4, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: true } },
  { id: "youyou", label: "佑佑", frameTier: 2, skills: ["txhj_youyouSkill1", "txhj_youyouSkill2"], action: { daiji: 4, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: true } },
  { id: "minmin", label: "敏敏", frameTier: 2, skills: ["txhj_minminSkill1", "txhj_minminSkill2"], action: { daiji: 4, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: true } },
  { id: "qiaoqiao", label: "巧巧", frameTier: 2, skills: ["txhj_qiaoqiaoSkill1", "txhj_qiaoqiaoSkill2"], action: { daiji: 4, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: true } },
  { id: "lulu", label: "露露", frameTier: 1, skills: ["txhj_luluSkill1", "txhj_luluSkill2"], action: { daiji: 4, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: true } },
  { id: "datong", label: "大桶", frameTier: 0, skills: ["txhj_datongSkill1"], action: { daiji: 2, gongji: 2, jingzhi: true, jinnang: 1, shouji: 1, zuodian: false } },
  { id: "ale", label: "阿乐", frameTier: 2, skills: ["txhj_aleSkill1", "txhj_aleSkill2"], action: { daiji: 3, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: true } },
  { id: "rui", label: "瑞", frameTier: 3, skills: ["txhj_ruiSkill1", "txhj_ruiSkill2", "txhj_ruiSkill3"], action: { daiji: 4, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: true } },
  { id: "yueer", label: "玥儿", frameTier: 2, skills: ["txhj_yueerSkill1", "txhj_yueerSkill2"], action: { daiji: 3, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: true } },
  { id: "yan", label: "焱", frameTier: 3, skills: ["txhj_yanSkill1", "txhj_yanSkill2", "txhj_yanSkill3"], action: { daiji: 4, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: false } },
  { id: "ahao", label: "阿豪", frameTier: 3, skills: ["txhj_aHaoSkill1", "txhj_aHaoSkill2", "txhj_aHaoSkill3"], action: { daiji: 4, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: false } },
  { id: "liuli", label: "琉璃", frameTier: 2, skills: ["txhj_liuliSkill1", "txhj_liuliSkill2"], action: { daiji: 4, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: true } },
  { id: "manman", label: "蛮蛮", frameTier: 2, skills: ["txhj_manmanSkill1", "txhj_manmanSkill2"], action: { daiji: 3, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: false } },
  { id: "xiaoxiao", label: "枭枭", frameTier: 2, skills: ["txhj_xiaoxiaoSkill1", "txhj_xiaoxiaoSkill2"], action: { daiji: 3, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: false } },
  { id: "xueren", label: "雪人", frameTier: 1, skills: ["txhj_xuerenSkill1"], action: { daiji: 2, gongji: 2, jingzhi: true, jinnang: 1, shouji: 1, zuodian: false } },
  { id: "xuanwu", label: "玄武", frameTier: 3, skills: ["txhj_xuanwuSkill1", "txhj_xuanwuSkill2", "txhj_xuanwuSkill3"], action: { daiji: 4, gongji: 2, jingzhi: true, jinnang: 1, shouji: 2, zuodian: false } },
];

export const TAIXU_SERVANT_ACTION_SPECS: Readonly<Record<string, TaixuServantActionSpec>> = Object.fromEntries(
  TAIXU_SERVANT_SPECS.map((s) => [s.id, s.action]),
);

export function getTaixuServantActionSpec(servantId: string): TaixuServantActionSpec {
  return TAIXU_SERVANT_ACTION_SPECS[servantId] ?? TAIXU_SERVANT_SPECS[0]!.action;
}

/**
 * Returns idle loop & stance clip names for a servant (`daiji1..N`, plus `jingzhi` / `zuodian` flags).
 */
export function getTaixuServantIdleActions(servantId: string): string[] {
  const spec = getTaixuServantActionSpec(servantId);
  const actions: string[] = [];
  for (let i = 1; i <= spec.daiji; i++) {
    actions.push(`daiji${i}`);
  }
  if (spec.jingzhi) {
    actions.push("jingzhi");
  }
  if (spec.zuodian) {
    actions.push("zuodian");
  }
  return actions;
}

/**
 * Upstream `Servant.prototype.getAction()` (`extension_servant.js:154-168`):
 * expands `daiji`/`jinnang`/`shouji` indices, excluding `daiji1`, `gongji`, `jingzhi`, `zuodian`.
 */
export function getTaixuServantInteractiveActions(servantId: string): string[] {
  const spec = getTaixuServantActionSpec(servantId);
  const list: string[] = [];
  for (const [key, count] of [
    ["daiji", spec.daiji],
    ["jinnang", spec.jinnang],
    ["shouji", spec.shouji],
  ] as const) {
    for (let i = 1; i <= count; i++) {
      const clip = `${key}${i}`;
      if (clip !== "daiji1") {
        list.push(clip);
      }
    }
  }
  return list;
}

/**
 * Resolves `_slDamage`, `_slUseCard`, and `_slHurt` Spine animation clips (`extension_servant.js:205-279`).
 */
export function resolveTaixuServantBattleClip(
  servantId: string,
  cue: TaixuServantBattleCue,
): string | null {
  const spec = getTaixuServantActionSpec(servantId);
  if (cue.kind === "_slDamage") {
    if (!(cue.amount > 0)) return null;
    if (cue.isLeft && spec.gongji >= 2) {
      return "gongji2";
    }
    return spec.gongji >= 1 ? "gongji1" : null;
  }
  if (cue.kind === "_slUseCard") {
    const isTrickOrRecover =
      cue.cardType === "trick" || cue.cardName === "tao" || cue.cardName === "jiu";
    return isTrickOrRecover && spec.jinnang >= 1 ? "jinnang1" : null;
  }
  const shoujiCount = Math.max(1, spec.shouji);
  if (shoujiCount === 1) {
    return "shouji1";
  }
  const idx = ((Math.max(1, Math.floor(cue.variant ?? 1)) - 1) % shoujiCount) + 1;
  return `shouji${idx}`;
}

export interface TaixuResponsiveTokens {
  mode: TaixuViewportMode;
  activeStylesheet: "extension_PcStyle.css" | "extension_MobileStyle.css";
  unit: "vw" | "rem";
  popupTextSize: string;
  homeBodyTextSize: string;
  pointTitleSize: string;
  pointTitleLineHeight: string;
  consoleCoinSize: string;
}

export interface TaixuUiRuntimeState {
  viewportMode: TaixuViewportMode;
  responsive: TaixuResponsiveTokens;
  loadedStylesheets: ReadonlyArray<TaixuStylesheetId>;
  activeSeasonId: string;
  activePanel: "console-desk" | "collect-home" | "servant-select" | "merchant-shop";
  selectedServantId: string;
  servantFrameClass: string;
  servantIdleActions: ReadonlyArray<string>;
  lastServantClip: string | null;
  coinBalance: number;
  shopPurchaseCount: number;
  hasJuguzhidaoBuff: boolean;
  currentShopPrice: number;
  seenKnownDiscardIds?: ReadonlyArray<number>;
}

/**
 * Computes responsive typography tokens matching `extension_PcStyle.css:1-60` (`vw`)
 * vs `extension_MobileStyle.css:1-60` (`rem`).
 */
export function resolveTaixuResponsiveTokens(viewportWidthPx: number): TaixuResponsiveTokens {
  const isMobile = viewportWidthPx < 960;
  if (isMobile) {
    return {
      mode: "mobile",
      activeStylesheet: "extension_MobileStyle.css",
      unit: "rem",
      popupTextSize: "1.2rem",
      homeBodyTextSize: "1.8rem",
      pointTitleSize: "2.5rem",
      pointTitleLineHeight: "3.5rem",
      consoleCoinSize: "1.5rem",
    };
  }
  return {
    mode: "pc",
    activeStylesheet: "extension_PcStyle.css",
    unit: "vw",
    popupTextSize: "1.2vw",
    homeBodyTextSize: "1.8vw",
    pointTitleSize: "2.5vw",
    pointTitleLineHeight: "3.5vw",
    consoleCoinSize: "1.5vw",
  };
}

/**
 * Merchant shop price formula (`太虚幻境/extension_framework.js:7008-7032` & `extension_buff.js:763`):
 * `dec = hasJuguzhidaoBuff ? 1 : 0`; each shop purchase beyond `dec` adds +200 coin.
 */
export function computeTaixuShopPrice(
  basePrice: number,
  purchaseCount: number,
  hasJuguzhidaoBuff: boolean,
): number {
  const safeBase = Math.max(0, Math.floor(basePrice));
  const safeCount = Math.max(0, Math.floor(purchaseCount));
  const dec = hasJuguzhidaoBuff ? 1 : 0;
  return safeBase + Math.max(safeCount - dec, 0) * 200;
}

export function createInitialTaixuUiRuntimeState(viewportWidthPx = 1280): TaixuUiRuntimeState {
  const responsive = resolveTaixuResponsiveTokens(viewportWidthPx);
  const defaultServant = TAIXU_SERVANT_SPECS[0]!;
  return {
    viewportMode: responsive.mode,
    responsive,
    loadedStylesheets: TAIXU_SURFACE_STYLESHEETS,
    activeSeasonId: TAIXU_DLC_SEASONS[0]!.id,
    activePanel: "console-desk",
    selectedServantId: defaultServant.id,
    servantFrameClass: `frame${defaultServant.frameTier}`,
    servantIdleActions: getTaixuServantIdleActions(defaultServant.id),
    lastServantClip: "daiji1",
    coinBalance: 200,
    shopPurchaseCount: 0,
    hasJuguzhidaoBuff: false,
    currentShopPrice: computeTaixuShopPrice(50, 0, false),
  };
}

export function switchTaixuViewport(
  state: TaixuUiRuntimeState,
  viewportWidthPx: number,
): TaixuUiRuntimeState {
  const responsive = resolveTaixuResponsiveTokens(viewportWidthPx);
  return {
    ...state,
    viewportMode: responsive.mode,
    responsive,
  };
}

export function selectTaixuServantSurface(
  state: TaixuUiRuntimeState,
  servantId: string,
): TaixuUiRuntimeState {
  const spec = TAIXU_SERVANT_SPECS.find((s) => s.id === servantId) ?? TAIXU_SERVANT_SPECS[0]!;
  return {
    ...state,
    activePanel: "servant-select",
    selectedServantId: spec.id,
    servantFrameClass: `frame${spec.frameTier}`,
    servantIdleActions: getTaixuServantIdleActions(spec.id),
    lastServantClip: "daiji1",
  };
}

export function triggerTaixuServantBattleClip(
  state: TaixuUiRuntimeState,
  cue: TaixuServantBattleCue,
): TaixuUiRuntimeState {
  const clip = resolveTaixuServantBattleClip(state.selectedServantId, cue);
  if (!clip) {
    return state;
  }
  return {
    ...state,
    lastServantClip: clip,
  };
}

export function purchaseTaixuMerchantItem(
  state: TaixuUiRuntimeState,
  basePrice = 50,
): TaixuUiRuntimeState {
  const cost = computeTaixuShopPrice(basePrice, state.shopPurchaseCount, state.hasJuguzhidaoBuff);
  if (state.coinBalance < cost) {
    return {
      ...state,
      activePanel: "merchant-shop",
      currentShopPrice: cost,
    };
  }
  const nextCount = state.shopPurchaseCount + 1;
  return {
    ...state,
    activePanel: "merchant-shop",
    coinBalance: state.coinBalance - cost,
    shopPurchaseCount: nextCount,
    currentShopPrice: computeTaixuShopPrice(basePrice, nextCount, state.hasJuguzhidaoBuff),
  };
}

function pickStringField(obj: Record<string, unknown> | undefined, keys: string[]): string | undefined {
  if (!obj) return undefined;
  for (const key of keys) {
    const val = obj[key];
    if (typeof val === "string" && val.trim()) return val.trim();
  }
  return undefined;
}

function pickNumberField(obj: Record<string, unknown> | undefined, keys: string[]): number | undefined {
  if (!obj) return undefined;
  for (const key of keys) {
    const val = obj[key];
    if (typeof val === "number" && Number.isFinite(val)) return val;
  }
  return undefined;
}

export function resolvePublicEventCardHint(
  rawObj: Record<string, unknown>,
  env: EngineEnvelope,
  cardActor: number,
  prevKnownDiscardIds: ReadonlySet<number>,
  cardEventOrdinal: number,
  totalCardEvents: number,
): { cardType: string; cardName: string; hasExplicitHint: boolean } {
  const pubObj =
    rawObj.PublicEvent && typeof rawObj.PublicEvent === "object"
      ? (rawObj.PublicEvent as Record<string, unknown>)
      : undefined;
  const rawName =
    pickStringField(rawObj, ["cardName", "card_name", "card", "kind"]) ??
    pickStringField(pubObj, ["cardName", "card_name", "card", "kind"]);
  const rawType =
    pickStringField(rawObj, ["cardType", "card_type"]) ??
    pickStringField(pubObj, ["cardType", "card_type"]);
  const rawDef = pickNumberField(rawObj, ["def"]) ?? pickNumberField(pubObj, ["def"]);

  if (rawName || rawType || rawDef != null) {
    const fromDef = rawDef != null ? lookupCardKindByDef(rawDef) : null;
    const fromKey = rawName ? lookupCardKindByKey(rawName) : null;
    const cardName = (fromKey?.key ?? rawName ?? "").toLowerCase();
    const cardType = rawType?.toLowerCase() ?? fromKey?.category ?? fromDef?.category ?? "";
    return { cardType, cardName, hasExplicitHint: true };
  }

  const knownCards = env.view?.known_cards ?? [];
  const processingCards = knownCards.filter((c) => c.zone === "Processing");
  const processingMatch = [
    ...processingCards.filter((c) => c.owner === cardActor).reverse(),
    ...processingCards.filter((c) => c.owner !== cardActor).reverse(),
  ];
  for (const c of processingMatch) {
    const resolved = lookupCardKindByKey(c.kind) ?? lookupCardKindByDef(c.def);
    if (resolved) {
      return { cardType: resolved.category, cardName: resolved.key, hasExplicitHint: true };
    }
  }

  const newDiscardCards = knownCards.filter(
    (c) => c.zone === "DiscardPile" && !prevKnownDiscardIds.has(c.id),
  );
  if (newDiscardCards.length > 0) {
    const picked =
      newDiscardCards.find((c) => c.owner === cardActor) ??
      (totalCardEvents > 1
        ? newDiscardCards[Math.min(cardEventOrdinal, newDiscardCards.length - 1)]
        : newDiscardCards[newDiscardCards.length - 1]);
    if (picked) {
      const resolved = lookupCardKindByKey(picked.kind) ?? lookupCardKindByDef(picked.def);
      if (resolved) {
        return { cardType: resolved.category, cardName: resolved.key, hasExplicitHint: true };
      }
    }
  }

  const label = typeof pubObj?.label === "string" ? pubObj.label : "";
  if (label && label !== "使用或响应牌" && label !== "摸牌") {
    const lowerLabel = label.toLowerCase();
    if (lowerLabel.includes("tao") || label.includes("桃")) {
      return { cardType: "basic", cardName: "tao", hasExplicitHint: true };
    }
    if (lowerLabel.includes("jiu") || label.includes("酒")) {
      return { cardType: "basic", cardName: "jiu", hasExplicitHint: true };
    }
    if (lowerLabel.includes("trick") || label.includes("锦囊") || label.includes("jinnang")) {
      return { cardType: "trick", cardName: "", hasExplicitHint: true };
    }
  }

  return { cardType: "", cardName: "sha", hasExplicitHint: false };
}

/**
 * Syncs companion servant battle clips (`_slDamage`, `_slUseCard`, `_slHurt`) from real
 * WASM `presentation.occurrences` and Network/legacy `PublicEvent` envelopes.
 */
export function syncTaixuServantClipsFromEnvelope(
  state: TaixuUiRuntimeState,
  env: EngineEnvelope,
  prevHpBySeat: Readonly<Record<number, number>> | undefined,
): TaixuUiRuntimeState {
  let next = state;
  const seats = env.view?.seats ?? [];
  const curPlayer = env.view?.current_player ?? 0;
  const selfObserverSeat = (env.view as { observer?: number } | undefined)?.observer ?? 0;
  const woundedSeat =
    seats.find((s) => s.player !== curPlayer && s.hp < s.max_hp)?.player ??
    seats.find((s) => s.player !== curPlayer)?.player ??
    curPlayer;

  const prevHpMap = prevHpBySeat ?? {};
  const hpDroppedSeats = new Set<number>();
  for (const seat of seats) {
    const prevHp = prevHpMap[seat.player];
    if (typeof prevHp === "number" && prevHp > seat.hp) {
      hpDroppedSeats.add(seat.player);
    }
  }
  const prevKnownDiscardIds = new Set(next.seenKnownDiscardIds ?? []);
  const currentDiscardIds = (env.view?.known_cards ?? [])
    .filter((c) => c.zone === "DiscardPile")
    .map((c) => c.id);

  let hasPresentationCardUse = false;
  let hasPresentationDamage = false;
  (env.presentation?.occurrences ?? []).forEach((occ, occIdx) => {
    const pev = occ.event;
    if (pev.kind === "card_used") {
      hasPresentationCardUse = true;
      if (pev.actor - 1 === selfObserverSeat) {
        const resolved =
          lookupCardKindByKey(pev.effective.key) ?? lookupCardKindByKey(pev.effective.kind);
        next = triggerTaixuServantBattleClip(next, {
          kind: "_slUseCard",
          cardType: resolved?.category ?? "",
          cardName: resolved?.key ?? pev.effective.key ?? pev.effective.kind ?? "",
        });
      }
    } else if (pev.kind === "damage_dealt") {
      hasPresentationDamage = true;
      const dmgSource = pev.source != null ? pev.source - 1 : null;
      const dmgTarget = pev.target - 1;
      const dmgAmount = pev.amount ?? 1;
      if (dmgAmount > 0) {
        if (dmgSource === selfObserverSeat && dmgTarget !== selfObserverSeat) {
          next = triggerTaixuServantBattleClip(next, {
            kind: "_slDamage",
            amount: dmgAmount,
            isLeft: false,
          });
        } else if (dmgTarget === selfObserverSeat) {
          next = triggerTaixuServantBattleClip(next, {
            kind: "_slHurt",
            variant: occIdx + 1,
          });
        }
      }
    }
  });

  const events = env.events ?? [];
  const totalCardEvents = events.filter((rawEv) => {
    const pubCode = (rawEv as { PublicEvent?: { code?: number } })?.PublicEvent?.code;
    const evType = (rawEv as { type?: string })?.type;
    return pubCode === 3 || evType === "CardPlayed";
  }).length;
  let cardEventOrdinal = 0;

  events.forEach((rawEv, idx) => {
    const rawObj = rawEv as Record<string, unknown>;
    const pubCode = (rawEv as { PublicEvent?: { code?: number } })?.PublicEvent?.code;
    const evType = (rawEv as { type?: string })?.type;
    if (pubCode === 4 || evType === "DamageDealt") {
      const explicitSource = pickNumberField(rawObj, ["source"]);
      const explicitTarget = pickNumberField(rawObj, ["target"]);
      const dmgSource = explicitSource ?? curPlayer;
      const onlyNonCurWounded = seats.find((s) => s.player !== curPlayer && s.hp < s.max_hp)?.player;
      const selfSeatView = seats.find((s) => s.player === selfObserverSeat);
      const inferredTarget = hpDroppedSeats.has(selfObserverSeat)
        ? selfObserverSeat
        : [...hpDroppedSeats][0] ??
          onlyNonCurWounded ??
          (selfSeatView && selfSeatView.hp < selfSeatView.max_hp ? selfObserverSeat : woundedSeat);
      const dmgTarget = explicitTarget ?? inferredTarget;
      const dmgAmount = pickNumberField(rawObj, ["amount"]) ?? 1;
      if (!hasPresentationDamage && dmgAmount > 0) {
        if (dmgTarget === selfObserverSeat) {
          next = triggerTaixuServantBattleClip(next, { kind: "_slHurt", variant: idx + 1 });
        } else if (dmgSource === selfObserverSeat) {
          next = triggerTaixuServantBattleClip(next, {
            kind: "_slDamage",
            amount: dmgAmount,
            isLeft: Boolean(rawObj.isLeft),
          });
        }
      }
    } else if (pubCode === 2 || pubCode === 3 || evType === "CardPlayed") {
      const pubObj =
        rawObj.PublicEvent && typeof rawObj.PublicEvent === "object"
          ? (rawObj.PublicEvent as Record<string, unknown>)
          : undefined;
      const cardActor =
        pickNumberField(rawObj, ["player", "source", "actor"]) ??
        pickNumberField(pubObj, ["player", "source", "actor"]) ??
        curPlayer;
      const hint = resolvePublicEventCardHint(
        rawObj,
        env,
        cardActor,
        prevKnownDiscardIds,
        cardEventOrdinal,
        totalCardEvents,
      );
      if (pubCode === 3 || evType === "CardPlayed") {
        cardEventOrdinal += 1;
      }
      const isCardPlayOrHinted =
        pubCode === 3 || evType === "CardPlayed" || hint.hasExplicitHint;
      if (!hasPresentationCardUse && isCardPlayOrHinted && cardActor === selfObserverSeat) {
        next = triggerTaixuServantBattleClip(next, {
          kind: "_slUseCard",
          cardType: hint.cardType,
          cardName: hint.cardName,
        });
      }
    }
  });

  if (currentDiscardIds.length > 0 || next.seenKnownDiscardIds) {
    next = {
      ...next,
      seenKnownDiscardIds: currentDiscardIds,
    };
  }
  return next;
}
