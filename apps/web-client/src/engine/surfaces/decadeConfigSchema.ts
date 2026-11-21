/**
 * L08 十周年UI 全部 44 项配置项数据结构、上游默认值、互斥逻辑与 CSS Dataset 映射。
 * 上游对照: `archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:6948, 11995-12684`
 * （严守裁决 35/38 净室边界：不读取 spine.js，不含闭源商业素材）
 */

export const DECADE_PLAYER_NODE_KEYS = [
  "avatar",
  "avatar2",
  "turnedover",
  "framebg",
  "intro",
  "identity",
  "hp",
  "long",
  "wei",
  "name",
  "name2",
  "nameol",
  "count",
  "equips",
  "judges",
  "marks",
  "chain",
  "handcards1",
  "handcards2",
  "expansions",
] as const;

export type DecadePlayerNodeKey = (typeof DECADE_PLAYER_NODE_KEYS)[number];

export type DecadeCardSetPreset =
  | "off"
  | "gold"
  | "hand"
  | "new"
  | "mian"
  | "dian"
  | "zun"
  | "tong"
  | "zhi"
  | "black";

export type DecadeCardHelasisyPreset = "off" | "helasisy" | "xiaosha" | "nsfw" | "hakimi";
export type DecadeCardPrettifyPreset = "off" | "webp" | "png" | "jpg" | "bmp";
export type DecadeCardBorderPreset =
  | "off"
  | "one"
  | "two"
  | "three"
  | "four"
  | "five"
  | "six"
  | "seven"
  | "eight"
  | "night";

export type DecadeGameStartPreset =
  | "off"
  | "shizhounian"
  | "shizhounian_dys"
  | "shizhounian_djj"
  | "shizhounian_zf"
  | "shizhounian_qfz"
  | "shousha_dys"
  | "shousha_djj"
  | "shousha_jj"
  | "shousha_xw"
  | "SF_kaizhan_eff_putong"
  | "SF_kaizhan_eff_jiangjun"
  | "SF_kaizhan_eff_weijiangjun"
  | "SF_kaizhan_eff_cheqijiangjun"
  | "SF_kaizhan_eff_piaoqijiangjun"
  | "SF_kaizhan_eff_dajiangjun"
  | "SF_kaizhan_eff_dasima"
  | "shizhounian_SJ"
  | "shousha_SJ"
  | "xinshizhounian_SJ";

export type DecadeCpzsPreset =
  | "off"
  | "shoushaX"
  | "shousha"
  | "jiangjun"
  | "weijiangjun"
  | "cheqijiangjun"
  | "biaoqijiangjun"
  | "dajiangjun"
  | "dasima";

export interface DecadeUiConfig {
  /** 1. `extension.js:6948` 座位子节点注册表映射 */
  node: ReadonlyArray<DecadePlayerNodeKey>;
  /** 2. `extension.js:11995` 联机模式标记 */
  _connectMode: boolean;
  /** 3. `extension.js:12007` 加速·异步模式 */
  asyncMode: boolean;
  /** 4. `extension.js:12012` 调试助手 */
  eruda: boolean;
  /** 5. `extension.js:12017` 右手布局（手牌在右/自席在左） */
  rightLayout: boolean;
  /** 6. `extension.js:12025` 单机卡牌风格 */
  game_cardSet: DecadeCardSetPreset;
  /** 7. `extension.js:12069` 单机特殊卡牌 */
  cardHelasisy: DecadeCardHelasisyPreset;
  /** 8. `extension.js:12098` 如真线上卡牌风格 */
  game_cardSet2: DecadeCardSetPreset;
  /** 9. `extension.js:12142` 如真特殊卡牌 */
  cardHelasisy2: DecadeCardHelasisyPreset;
  /** 10. `extension.js:12171` 卡牌美化 */
  cardPrettify: DecadeCardPrettifyPreset;
  /** 11. `extension.js:12213` 卡牌边框 */
  cardBorder: DecadeCardBorderPreset;
  /** 12. `extension.js:12272` 视为点数花色高亮 */
  gold_suitnum: "none" | "gold" | "blue";
  /** 13. `extension.js:12282` 动态背景（净室渐变或自备 Spine 4.0） */
  dynamicBackground: string;
  /** 14. `extension.js:12308` 动态皮肤开关（自备导入+静态回退） */
  dynamicSkin: boolean;
  /** 15. `extension.js:12312` 动皮露头 */
  dynamicSkinOutcrop: boolean;
  /** 16. `extension.js:12330` 牌名辅助显示 */
  cardAlternateNameVisible: boolean;
  /** 17. `extension.js:12337` 势力身份美化 */
  campIdentityImageMode: boolean;
  /** 18. `extension.js:12341` 隐藏托管提示 */
  tuoguan_hide: boolean;
  /** 19. `extension.js:12379` 游戏开场切换 */
  SanGuoShaGameStart: DecadeGameStartPreset;
  /** 20. `extension.js:12409` 游戏结算特效 */
  SanGuoShaGameOver: "shizhounian" | "shousha" | "off";
  /** 21. `extension.js:12422` 出牌指示特效 */
  CPZS: DecadeCpzsPreset;
  /** 22. `extension.js:12441` 更多局内特效 */
  JNTX: boolean;
  /** 23. `extension.js:12446` 军八斗地主美化 */
  JBDDZ: boolean;
  /** 24. `extension.js:12455` 玩家击杀特效 */
  playerKillEffect: boolean;
  /** 25. `extension.js:12463` 游戏动画特效 */
  gameAnimationEffect: boolean;
  /** 26. `extension.js:12467` 治疗特效 */
  EffectWood: boolean;
  /** 27. `extension.js:12475` 玩家阵亡特效 */
  playerDieEffect: boolean;
  /** 28. `extension.js:12483` 卡牌使用特效 */
  cardUseEffect: boolean;
  /** 29. `extension.js:12499` 露头皮肤 */
  outcropSkin: "shizhounian" | "shousha" | "off";
  /** 30. `extension.js:12512` 玩家边框等阶 */
  borderLevel: "one" | "two" | "three" | "four" | "five" | "kill";
  /** 31. `extension.js:12527` 按钮文字镶金 */
  skill_control_gold: "none" | "onstart" | "forever";
  /** 32. `extension.js:12537` 获得技能显示(标记) */
  gainSkillsShown: boolean;
  /** 33. `extension.js:12541` 获得技能显示(贴纸) */
  gainSkillsVisible: "on" | "off" | "othersOn";
  /** 34. `extension.js:12554` 人物标记样式 */
  playerMarkStyle: "red" | "yellow" | "decade";
  /** 35. `extension.js:12566` 手杀标记优化 */
  shoushaNewNewMark: boolean;
  /** 36. `extension.js:12571` 样式切换（十周年/手杀） */
  newDecadeStyle: "on" | "off";
  /** 37. `extension.js:12584` 换将特效 */
  changeWujiangTX: "off" | "blur" | "shine";
  /** 38. `extension.js:12597` 弹出文字样式 */
  shadowStyle: "on" | "off";
  /** 39. `extension.js:12609` 龙头美化 */
  rarityLong: "x" | "on" | "off" | "xinon";
  /** 40. `extension.js:12623` 界面布局 */
  decadeLayout: "on" | "off";
  /** 41. `extension.js:12636` 单独装备栏 */
  aloneEquip: boolean;
  /** 42. `extension.js:12647` 手杀装备栏优化 */
  shoushaNewEquip: boolean;
  /** 43. `extension.js:12658` 手杀标记栏优化 */
  shoushaNewMark: boolean;
  /** 44. `extension.js:12669` 更换光标+loading框 */
  loadingStyle: "off" | "on" | "On" | "othersOn";
}

export type DecadeConfigKey = keyof DecadeUiConfig;
export type DecadeConfigCategory = "layout" | "cards" | "effects" | "marks" | "system";

export interface DecadeConfigOptionSpec {
  readonly value: string;
  readonly label: string;
}

export interface DecadeConfigFieldSpec {
  readonly key: DecadeConfigKey;
  readonly label: string;
  readonly category: DecadeConfigCategory;
  readonly control: "boolean" | "select" | "readonly";
  readonly upstreamLine: number;
  readonly intro?: string;
  readonly options?: ReadonlyArray<DecadeConfigOptionSpec>;
}

const CARD_SET_OPTIONS: ReadonlyArray<DecadeConfigOptionSpec> = [
  { value: "off", label: "关闭" },
  { value: "gold", label: "金色" },
  { value: "hand", label: "手杀" },
  { value: "new", label: "新版" },
  { value: "mian", label: "面杀" },
  { value: "dian", label: "典藏" },
  { value: "zun", label: "至尊" },
  { value: "tong", label: "铜雀" },
  { value: "zhi", label: "龙纹" },
  { value: "black", label: "黑金" },
];

const CARD_HEL_OPTIONS: ReadonlyArray<DecadeConfigOptionSpec> = [
  { value: "off", label: "关闭" },
  { value: "helasisy", label: "娘化卡牌" },
  { value: "xiaosha", label: "虚拟偶像" },
  { value: "nsfw", label: "艺术鉴赏" },
  { value: "hakimi", label: "南北绿豆" },
];

const CARD_BORDER_OPTIONS: ReadonlyArray<DecadeConfigOptionSpec> = [
  { value: "off", label: "关闭" },
  { value: "one", label: "金凤丹玉" },
  { value: "two", label: "镶金边框" },
  { value: "three", label: "镶银边框" },
  { value: "four", label: "赤焰图腾" },
  { value: "five", label: "寒冰图腾" },
  { value: "six", label: "典藏边框" },
  { value: "seven", label: "龙纹边框" },
  { value: "eight", label: "铜雀边框" },
  { value: "night", label: "至尊边框" },
];

const GAME_START_OPTIONS: ReadonlyArray<DecadeConfigOptionSpec> = [
  { value: "shousha_dys", label: "手杀大元帅" },
  { value: "shousha_djj", label: "手杀大将军" },
  { value: "shousha_jj", label: "手杀将军" },
  { value: "shousha_xw", label: "手杀校尉" },
  { value: "shizhounian", label: "十周年" },
  { value: "shizhounian_dys", label: "十周年大元帅" },
  { value: "shizhounian_djj", label: "十周年大将军" },
  { value: "shizhounian_zf", label: "十周年中锋" },
  { value: "shizhounian_qfz", label: "十周年千夫长" },
  { value: "SF_kaizhan_eff_putong", label: "新十周年普通" },
  { value: "SF_kaizhan_eff_jiangjun", label: "新十周年将军" },
  { value: "SF_kaizhan_eff_weijiangjun", label: "新十周年卫将军" },
  { value: "SF_kaizhan_eff_cheqijiangjun", label: "新十周年车骑将军" },
  { value: "SF_kaizhan_eff_piaoqijiangjun", label: "新十周年骠骑将军" },
  { value: "SF_kaizhan_eff_dajiangjun", label: "新十周年大将军" },
  { value: "SF_kaizhan_eff_dasima", label: "新十周年大司马" },
  { value: "shizhounian_SJ", label: "十周年随机" },
  { value: "shousha_SJ", label: "手杀随机" },
  { value: "xinshizhounian_SJ", label: "新十周年随机" },
  { value: "off", label: "关闭" },
];

const CPZS_OPTIONS: ReadonlyArray<DecadeConfigOptionSpec> = [
  { value: "shoushaX", label: "手杀经典" },
  { value: "shousha", label: "手杀新版" },
  { value: "jiangjun", label: "十周年将军" },
  { value: "weijiangjun", label: "十周年卫将军" },
  { value: "cheqijiangjun", label: "十周年车骑将军" },
  { value: "biaoqijiangjun", label: "十周年骠骑将军" },
  { value: "dajiangjun", label: "十周年大将军" },
  { value: "dasima", label: "十周年大司马" },
  { value: "off", label: "关闭" },
];

/** 全部 44 项 L08 配置规格表（与 `inventory_status.json` `L08:configs:*` 一一对应） */
export const DECADE_CONFIG_SPECS: ReadonlyArray<DecadeConfigFieldSpec> = [
  { key: "newDecadeStyle", label: "样式切换", category: "layout", control: "select", upstreamLine: 12571, options: [{ value: "on", label: "十周年样式" }, { value: "off", label: "手杀样式" }] },
  { key: "decadeLayout", label: "界面布局", category: "layout", control: "select", upstreamLine: 12623, options: [{ value: "on", label: "十周年布局" }, { value: "off", label: "手杀布局" }] },
  { key: "rightLayout", label: "右手布局", category: "layout", control: "boolean", upstreamLine: 12017, intro: "主视角置左下、手牌区向右延展（关闭时镜像互换）" },
  { key: "aloneEquip", label: "单独装备栏", category: "layout", control: "boolean", upstreamLine: 12636, intro: "仅十周年样式生效：自席装备脱离立绘框独立排布" },
  { key: "shoushaNewEquip", label: "手杀装备栏优化", category: "layout", control: "boolean", upstreamLine: 12647 },
  { key: "borderLevel", label: "玩家边框等阶", category: "layout", control: "select", upstreamLine: 12512, options: [{ value: "kill", label: "随武将评级" }, { value: "one", label: "一阶" }, { value: "two", label: "二阶" }, { value: "three", label: "三阶" }, { value: "four", label: "四阶" }, { value: "five", label: "五阶" }] },
  { key: "rarityLong", label: "龙头美化", category: "layout", control: "select", upstreamLine: 12609, options: [{ value: "x", label: "关闭" }, { value: "on", label: "经典龙头" }, { value: "off", label: "经典龙头-静" }, { value: "xinon", label: "新版龙头" }] },
  { key: "loadingStyle", label: "光标与框体样式", category: "layout", control: "select", upstreamLine: 12669, options: [{ value: "off", label: "关闭" }, { value: "on", label: "青玉框" }, { value: "On", label: "流金框" }, { value: "othersOn", label: "苍翠十周年框" }] },
  { key: "skill_control_gold", label: "按钮文字镶金", category: "layout", control: "select", upstreamLine: 12527, options: [{ value: "none", label: "关闭" }, { value: "onstart", label: "开局前" }, { value: "forever", label: "保持开启" }] },
  { key: "JBDDZ", label: "军八斗地主美化", category: "layout", control: "boolean", upstreamLine: 12446 },

  { key: "game_cardSet", label: "单机卡牌风格", category: "cards", control: "select", upstreamLine: 12025, options: CARD_SET_OPTIONS },
  { key: "cardHelasisy", label: "单机特殊卡牌", category: "cards", control: "select", upstreamLine: 12069, options: CARD_HEL_OPTIONS },
  { key: "game_cardSet2", label: "如真卡牌风格", category: "cards", control: "select", upstreamLine: 12098, options: CARD_SET_OPTIONS },
  { key: "cardHelasisy2", label: "如真特殊卡牌", category: "cards", control: "select", upstreamLine: 12142, options: CARD_HEL_OPTIONS },
  { key: "cardPrettify", label: "卡牌美化", category: "cards", control: "select", upstreamLine: 12171, options: [{ value: "off", label: "关闭" }, { value: "webp", label: "金色将军" }, { value: "png", label: "手杀风格" }, { value: "jpg", label: "高清卡牌" }, { value: "bmp", label: "面杀风格" }] },
  { key: "cardBorder", label: "卡牌边框", category: "cards", control: "select", upstreamLine: 12213, options: CARD_BORDER_OPTIONS },
  { key: "gold_suitnum", label: "视为点数花色高亮", category: "cards", control: "select", upstreamLine: 12272, options: [{ value: "none", label: "关闭" }, { value: "gold", label: "金色" }, { value: "blue", label: "蓝色" }] },
  { key: "cardAlternateNameVisible", label: "牌名辅助显示", category: "cards", control: "boolean", upstreamLine: 12330 },

  { key: "campIdentityImageMode", label: "势力身份美化", category: "marks", control: "boolean", upstreamLine: 12337 },
  { key: "playerMarkStyle", label: "人物标记样式", category: "marks", control: "select", upstreamLine: 12554, options: [{ value: "red", label: "红灯笼" }, { value: "yellow", label: "黄灯笼" }, { value: "decade", label: "十周年" }] },
  { key: "shoushaNewNewMark", label: "手杀标记优化", category: "marks", control: "boolean", upstreamLine: 12566 },
  { key: "shoushaNewMark", label: "手杀标记栏细体阴影", category: "marks", control: "boolean", upstreamLine: 12658 },
  { key: "gainSkillsShown", label: "获得技能显示(标记)", category: "marks", control: "boolean", upstreamLine: 12537 },
  { key: "gainSkillsVisible", label: "获得技能显示(贴纸)", category: "marks", control: "select", upstreamLine: 12541, options: [{ value: "on", label: "显示" }, { value: "off", label: "不显示" }, { value: "othersOn", label: "显示他人" }] },
  { key: "tuoguan_hide", label: "隐藏托管提示", category: "marks", control: "boolean", upstreamLine: 12341 },
  { key: "shadowStyle", label: "弹出文字样式", category: "marks", control: "select", upstreamLine: 12597, options: [{ value: "on", label: "原样式" }, { value: "off", label: "新样式" }] },

  { key: "gameAnimationEffect", label: "游戏动画特效", category: "effects", control: "boolean", upstreamLine: 12463 },
  { key: "JNTX", label: "更多局内特效", category: "effects", control: "boolean", upstreamLine: 12441 },
  { key: "SanGuoShaGameStart", label: "游戏开场切换", category: "effects", control: "select", upstreamLine: 12379, options: GAME_START_OPTIONS },
  { key: "SanGuoShaGameOver", label: "游戏结算特效", category: "effects", control: "select", upstreamLine: 12409, options: [{ value: "shizhounian", label: "十周年样式" }, { value: "shousha", label: "手杀样式" }, { value: "off", label: "关闭" }] },
  { key: "CPZS", label: "出牌指示特效", category: "effects", control: "select", upstreamLine: 12422, options: CPZS_OPTIONS },
  { key: "playerKillEffect", label: "玩家击杀特效", category: "effects", control: "boolean", upstreamLine: 12455 },
  { key: "EffectWood", label: "治疗特效", category: "effects", control: "boolean", upstreamLine: 12467 },
  { key: "playerDieEffect", label: "玩家阵亡特效", category: "effects", control: "boolean", upstreamLine: 12475 },
  { key: "cardUseEffect", label: "卡牌使用特效", category: "effects", control: "boolean", upstreamLine: 12483 },
  { key: "changeWujiangTX", label: "换将特效", category: "effects", control: "select", upstreamLine: 12584, options: [{ value: "off", label: "关闭特效" }, { value: "blur", label: "过渡特效" }, { value: "shine", label: "闪光特效" }] },
  { key: "outcropSkin", label: "露头皮肤裁剪", category: "effects", control: "select", upstreamLine: 12499, options: [{ value: "shizhounian", label: "十周年露头" }, { value: "shousha", label: "手杀露头" }, { value: "off", label: "关闭" }] },
  { key: "dynamicBackground", label: "动态背景(自备/渐变回退)", category: "effects", control: "select", upstreamLine: 12282, options: [{ value: "off", label: "关闭" }, { value: "decade_jade", label: "翡翠牌桌光影(净室)" }, { value: "shousha_amber", label: "琥珀流沙氛围(净室)" }, { value: "random", label: "随机净室背景" }] },
  { key: "dynamicSkin", label: "动态皮肤(需自备4.0资产)", category: "effects", control: "boolean", upstreamLine: 12308 },
  { key: "dynamicSkinOutcrop", label: "动皮露头", category: "effects", control: "boolean", upstreamLine: 12312 },

  { key: "asyncMode", label: "加速·异步模式", category: "system", control: "boolean", upstreamLine: 12007 },
  { key: "eruda", label: "调试助手面板", category: "system", control: "boolean", upstreamLine: 12012 },
  { key: "_connectMode", label: "联机模式状态", category: "system", control: "readonly", upstreamLine: 11995 },
  { key: "node", label: "座位 DOM 节点契约(20项)", category: "system", control: "readonly", upstreamLine: 6948 },
];

export const DECADE_CONFIG_STORAGE_KEY = "noname-next.decade-ui-config.v1";

export function createDefaultDecadeConfig(): DecadeUiConfig {
  return {
    node: [...DECADE_PLAYER_NODE_KEYS],
    _connectMode: false,
    asyncMode: false,
    eruda: false,
    rightLayout: true,
    game_cardSet: "off",
    cardHelasisy: "off",
    game_cardSet2: "off",
    cardHelasisy2: "off",
    cardPrettify: "png",
    cardBorder: "off",
    gold_suitnum: "gold",
    dynamicBackground: "off",
    dynamicSkin: false,
    dynamicSkinOutcrop: true,
    cardAlternateNameVisible: false,
    campIdentityImageMode: true,
    tuoguan_hide: false,
    SanGuoShaGameStart: "shousha_dys",
    SanGuoShaGameOver: "shousha",
    CPZS: "shoushaX",
    JNTX: true,
    JBDDZ: true,
    playerKillEffect: false,
    gameAnimationEffect: true,
    EffectWood: false,
    playerDieEffect: true,
    cardUseEffect: false,
    outcropSkin: "shousha",
    borderLevel: "kill",
    skill_control_gold: "none",
    gainSkillsShown: true,
    gainSkillsVisible: "off",
    playerMarkStyle: "decade",
    shoushaNewNewMark: true,
    newDecadeStyle: "on",
    changeWujiangTX: "shine",
    shadowStyle: "off",
    rarityLong: "on",
    decadeLayout: "on",
    aloneEquip: false,
    shoushaNewEquip: true,
    shoushaNewMark: true,
    loadingStyle: "othersOn",
  };
}

const DECADE_CONFIG_SPEC_MAP: Readonly<Record<string, DecadeConfigFieldSpec>> = Object.fromEntries(
  DECADE_CONFIG_SPECS.map((s) => [s.key, s]),
);
const VALID_PLAYER_NODE_KEY_SET: ReadonlySet<string> = new Set(DECADE_PLAYER_NODE_KEYS);

/**
 * 对 44 项配置补丁执行字段名、布尔/枚举类型及 `node` 槽位契约校验。
 * - `strict: true`（用于本地 JSON 导入）：发现未知键或非法值时显式抛错拒绝。
 * - `strict: false`（用于持久化读回归一化）：自动剔除损坏字段并回落默认值。
 */
export function validateDecadeConfigPatch(
  raw: Record<string, unknown>,
  opts?: { strict?: boolean },
): { patch: Partial<DecadeUiConfig>; validKeys: number } {
  const strict = Boolean(opts?.strict);
  const patch: Record<string, unknown> = {};
  let validKeys = 0;

  for (const [key, val] of Object.entries(raw)) {
    const spec = DECADE_CONFIG_SPEC_MAP[key];
    if (!spec) {
      if (strict) throw new Error(`DECADE_CONFIG_IMPORT_UNKNOWN_KEY:${key}`);
      continue;
    }
    if (key === "node") {
      const ok =
        Array.isArray(val) &&
        val.length > 0 &&
        val.every((item) => typeof item === "string" && VALID_PLAYER_NODE_KEY_SET.has(item));
      if (!ok) {
        if (strict) throw new Error("DECADE_CONFIG_IMPORT_INVALID_VALUE:node");
        continue;
      }
      patch.node = [...val];
      validKeys++;
      continue;
    }
    if (spec.control === "boolean" || key === "_connectMode") {
      if (typeof val !== "boolean") {
        if (strict) throw new Error(`DECADE_CONFIG_IMPORT_INVALID_TYPE:${key}`);
        continue;
      }
      patch[key] = val;
      validKeys++;
      continue;
    }
    if (spec.control === "select") {
      const allowed = spec.options?.map((o) => o.value) ?? [];
      if (typeof val !== "string" || !allowed.includes(val)) {
        if (strict) throw new Error(`DECADE_CONFIG_IMPORT_INVALID_ENUM:${key}`);
        continue;
      }
      patch[key] = val;
      validKeys++;
    }
  }
  return { patch: patch as Partial<DecadeUiConfig>, validKeys };
}

export function normalizeDecadeConfig(raw?: Partial<DecadeUiConfig> | null): DecadeUiConfig {
  const base = createDefaultDecadeConfig();
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) return base;
  const { patch } = validateDecadeConfigPatch(raw as Record<string, unknown>, { strict: false });
  return {
    ...base,
    ...patch,
    node: Array.isArray(patch.node) && patch.node.length > 0 ? [...patch.node] : base.node,
  };
}

/**
 * 复现上游 `extension.js:12181-12241, 12640` 互斥与联动逻辑：
 * - 单机下修改 `cardPrettify`/`cardBorder` 会关闭 `game_cardSet`；联机(`_connectMode`)下关闭 `game_cardSet2`。
 */
export function applyDecadeConfigUpdate(
  current: DecadeUiConfig,
  patch: Partial<DecadeUiConfig>,
): DecadeUiConfig {
  const next = normalizeDecadeConfig({ ...current, ...patch });
  if (patch.cardPrettify !== undefined || patch.cardBorder !== undefined) {
    if (next._connectMode) {
      next.game_cardSet2 = "off";
    } else {
      next.game_cardSet = "off";
    }
  }
  return next;
}

export function loadPersistedDecadeConfig(): DecadeUiConfig {
  try {
    const raw = globalThis.localStorage?.getItem(DECADE_CONFIG_STORAGE_KEY);
    if (!raw) return createDefaultDecadeConfig();
    return normalizeDecadeConfig(JSON.parse(raw) as Partial<DecadeUiConfig>);
  } catch {
    return createDefaultDecadeConfig();
  }
}

export function persistDecadeConfig(cfg: DecadeUiConfig): void {
  try {
    globalThis.localStorage?.setItem(DECADE_CONFIG_STORAGE_KEY, JSON.stringify(cfg));
  } catch {
    // Hardened webviews without storage keep in-memory state.
  }
}

/**
 * 导出上游 CSS 匹配的 `data-*` DOM 属性集 (`player_new.css`, `layout_new.css`, `equip.css`, `card.css`)。
 */
export function decadeArenaDatasetAttrs(cfg: DecadeUiConfig): Record<string, string> {
  const activeCardSet = cfg._connectMode ? cfg.game_cardSet2 : cfg.game_cardSet;
  const activeHelasisy = cfg._connectMode ? cfg.cardHelasisy2 : cfg.cardHelasisy;
  const effectiveAloneEquip = cfg.newDecadeStyle === "on" && cfg.aloneEquip ? "on" : "off";
  return {
    "data-decade-layout": cfg.decadeLayout,
    "data-new-decade-style": cfg.newDecadeStyle,
    "data-right-layout": cfg.rightLayout ? "on" : "off",
    "data-alone-equip": effectiveAloneEquip,
    "data-shousha-new-equip": cfg.shoushaNewEquip ? "on" : "off",
    "data-card-border": cfg.cardBorder,
    "data-borders": cfg.cardBorder,
    "data-arena-card-border": cfg.cardBorder,
    "data-card-prettify": cfg.cardPrettify,
    "data-card-set": activeCardSet,
    "data-card-helasisy": activeHelasisy,
    "data-gold-suitnum": cfg.gold_suitnum,
    "data-card-alternate-name-visible": cfg.cardAlternateNameVisible ? "on" : "off",
    "data-camp-identity-image": cfg.campIdentityImageMode ? "on" : "off",
    "data-tuoguan-hide": cfg.tuoguan_hide ? "on" : "off",
    "data-border-level": cfg.borderLevel,
    "data-outcrop-skin": cfg.outcropSkin,
    "data-dynamic-skin-outcrop": cfg.dynamicSkinOutcrop ? "on" : "off",
    "data-player-mark-style": cfg.playerMarkStyle,
    "data-shousha-new-new-mark": cfg.shoushaNewNewMark ? "on" : "off",
    "data-shousha-new-mark": cfg.shoushaNewMark ? "on" : "off",
    "data-gain-skills-shown": cfg.gainSkillsShown ? "on" : "off",
    "data-gain-skills-visible": cfg.gainSkillsVisible,
    "data-skill-gold": cfg.skill_control_gold,
    "data-shadow-style": cfg.shadowStyle,
    "data-rarity-long": cfg.rarityLong,
    "data-change-wujiang-tx": cfg.changeWujiangTX,
    "data-loading-style": cfg.loadingStyle,
    "data-cpzs": cfg.CPZS,
    "data-jbddz": cfg.JBDDZ ? "on" : "off",
    "data-game-anim": cfg.gameAnimationEffect ? "on" : "off",
    "data-dynamic-background": cfg.dynamicBackground,
  };
}
