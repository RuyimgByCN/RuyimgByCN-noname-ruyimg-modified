/**
 * L11 太虚幻境 (`archive/upstream/noname-liuli/noname/extension/太虚幻境/`)
 * Authentic native frontend surface runtime for the 4 upstream stylesheets:
 * - `extension_MobileStyle.css:1-494` (`rem`-scaled mobile touch typography & compact console desk)
 * - `extension_PcStyle.css:1-353` (`vw`-scaled widescreen PC typography & coin/rank console bar)
 * - `extension_servant.css:1-310` (`taixuhuanjing_chooseServantDiv` 80%×80% modal & `frame0..frame4` 21-servant gallery)
 * - `extension_style.css:1-6305` (`taixuhuanjing_collectHomeBody` 1920×1080 handbook, 8-DLC season chapter map & merchant shop)
 */

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

export interface TaixuServantCardSpec {
  id: string;
  label: string;
  frameTier: 0 | 1 | 2 | 3 | 4;
  skills: ReadonlyArray<string>;
}

/**
 * 21 authentic servants from `太虚幻境/extension_servant.js:350-1760` styled via `extension_servant.css:40-75`.
 */
export const TAIXU_SERVANT_SPECS: ReadonlyArray<TaixuServantCardSpec> = [
  { id: "dundun", label: "盾盾", frameTier: 1, skills: ["txhj_dundunSkill1", "txhj_dundunSkill2"] },
  { id: "jiuwei", label: "九尾", frameTier: 3, skills: ["txhj_jiuweiSkill1", "txhj_jiuweiSkill2", "txhj_jiuweiSkill3"] },
  { id: "tengshe", label: "腾蛇", frameTier: 3, skills: ["txhj_tengsheSkill1", "txhj_tengsheSkill2", "txhj_tengsheSkill3"] },
  { id: "qilin", label: "麒麟", frameTier: 3, skills: ["txhj_qilinSkill1", "txhj_qilinSkill2", "txhj_qilinSkill3"] },
  { id: "diting", label: "谛听", frameTier: 3, skills: ["txhj_ditingSkill1", "txhj_ditingSkill2", "txhj_ditingSkill3"] },
  { id: "yaya", label: "鸭鸭", frameTier: 2, skills: ["txhj_yayaSkill1", "txhj_yayaSkill2"] },
  { id: "youyou", label: "佑佑", frameTier: 2, skills: ["txhj_youyouSkill1", "txhj_youyouSkill2"] },
  { id: "minmin", label: "敏敏", frameTier: 2, skills: ["txhj_minminSkill1", "txhj_minminSkill2"] },
  { id: "qiaoqiao", label: "巧巧", frameTier: 2, skills: ["txhj_qiaoqiaoSkill1", "txhj_qiaoqiaoSkill2"] },
  { id: "lulu", label: "露露", frameTier: 1, skills: ["txhj_luluSkill1", "txhj_luluSkill2"] },
  { id: "datong", label: "大桶", frameTier: 0, skills: ["txhj_datongSkill1"] },
  { id: "ale", label: "阿乐", frameTier: 2, skills: ["txhj_aleSkill1", "txhj_aleSkill2"] },
  { id: "rui", label: "瑞", frameTier: 3, skills: ["txhj_ruiSkill1", "txhj_ruiSkill2", "txhj_ruiSkill3"] },
  { id: "yueer", label: "玥儿", frameTier: 2, skills: ["txhj_yueerSkill1", "txhj_yueerSkill2"] },
  { id: "yan", label: "焱", frameTier: 3, skills: ["txhj_yanSkill1", "txhj_yanSkill2", "txhj_yanSkill3"] },
  { id: "ahao", label: "阿豪", frameTier: 3, skills: ["txhj_aHaoSkill1", "txhj_aHaoSkill2", "txhj_aHaoSkill3"] },
  { id: "liuli", label: "琉璃", frameTier: 2, skills: ["txhj_liuliSkill1", "txhj_liuliSkill2"] },
  { id: "manman", label: "蛮蛮", frameTier: 2, skills: ["txhj_manmanSkill1", "txhj_manmanSkill2"] },
  { id: "xiaoxiao", label: "枭枭", frameTier: 2, skills: ["txhj_xiaoxiaoSkill1", "txhj_xiaoxiaoSkill2"] },
  { id: "xueren", label: "雪人", frameTier: 1, skills: ["txhj_xuerenSkill1"] },
  { id: "xuanwu", label: "玄武", frameTier: 3, skills: ["txhj_xuanwuSkill1", "txhj_xuanwuSkill2", "txhj_xuanwuSkill3"] },
];

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
  coinBalance: number;
  shopPurchaseCount: number;
  hasJuguzhidaoBuff: boolean;
  currentShopPrice: number;
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
 * Merchant shop price formula (`太虚幻境/extension_buff.js:763` `buff_txhj_juguzhidao` 巨贾之道):
 * Without `buff_txhj_juguzhidao`, each purchase inflates shop cost by +25 gold;
 * with `buff_txhj_juguzhidao`, price inflation is frozen at base price.
 */
export function computeTaixuShopPrice(
  basePrice: number,
  purchaseCount: number,
  hasJuguzhidaoBuff: boolean,
): number {
  const safeBase = Math.max(0, Math.floor(basePrice));
  const safeCount = Math.max(0, Math.floor(purchaseCount));
  return hasJuguzhidaoBuff ? safeBase : safeBase + safeCount * 25;
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
