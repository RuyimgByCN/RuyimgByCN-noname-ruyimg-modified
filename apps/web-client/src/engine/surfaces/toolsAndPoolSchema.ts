/**
 * Batch UI-7 工具、选将池与模式配置 1-to-1 Schema：
 * L02 SJ Settings (4 cfg + 1 surf)
 * L15 手杀全扩 (11 cfg)
 * L16 拖拽读取 (2 cfg + 1 surf)
 * L17 斗转星移 (9 cfg + 1 surf + 1 hook `banCharF`)
 * L18 更新保护 (12 cfg + 1 surf)
 * L21 武皇争霸 (7 cfg + 1 surf)
 * U05 杀海拾遗 (7 cfg)
 */

export interface ConfigSpecEntry<K extends string = string> {
  readonly key: K;
  readonly label: string;
  readonly control: "info" | "boolean" | "select" | "action";
  readonly upstreamLine: number;
  readonly ruleCoupled: boolean;
  readonly options?: ReadonlyArray<{ value: string; label: string }>;
}

function rejectUnknownKeys(obj: unknown, allowed: ReadonlySet<string>, prefix: string): Record<string, unknown> {
  if (!obj || typeof obj !== "object" || Array.isArray(obj)) {
    throw new Error(`${prefix}_INVALID_JSON`);
  }
  const rec = obj as Record<string, unknown>;
  for (const key of Object.keys(rec)) {
    if (!allowed.has(key)) {
      throw new Error(`${prefix}_UNKNOWN_KEY:${key}`);
    }
  }
  return rec;
}

const pickOpt = <T extends string>(v: unknown, opts: ReadonlyArray<T>, fallback: T): T =>
  typeof v === "string" && (opts as ReadonlyArray<string>).includes(v) ? (v as T) : fallback;
const pickBool = (v: unknown, fallback: boolean): boolean => (typeof v === "boolean" ? v : fallback);

// ---------------------------------------------------------------------------
// L02 SJ Settings (extension.js:338,346,359,710 + extension.css:1-3)
// ---------------------------------------------------------------------------
export interface L02SjConfig {
  tutorialapk: boolean;
  openTools: boolean;
  getExtensions: boolean;
  tutorial: boolean;
}

export const L02_CONFIG_SPECS: ReadonlyArray<ConfigSpecEntry<keyof L02SjConfig>> = [
  { key: "tutorialapk", label: "查看 APK 使用教程", control: "action", upstreamLine: 338, ruleCoupled: false },
  { key: "openTools", label: "启用控制台（调试面板）", control: "boolean", upstreamLine: 346, ruleCoupled: false },
  { key: "getExtensions", label: "点击查看离线扩展清单", control: "action", upstreamLine: 359, ruleCoupled: false },
  { key: "tutorial", label: "查看无名杀离线教程", control: "action", upstreamLine: 710, ruleCoupled: false },
];
export const L02_SURFACE_IDS = ["extension.css"] as const;
const L02_KEY_SET: ReadonlySet<string> = new Set(L02_CONFIG_SPECS.map((s) => s.key));

export function createDefaultL02Config(): L02SjConfig {
  return { tutorialapk: false, openTools: false, getExtensions: false, tutorial: false };
}
export function importL02ConfigJson(jsonText: string): L02SjConfig {
  const rec = rejectUnknownKeys(JSON.parse(jsonText), L02_KEY_SET, "L02_CONFIG");
  const d = createDefaultL02Config();
  return {
    tutorialapk: pickBool(rec.tutorialapk, d.tutorialapk),
    openTools: pickBool(rec.openTools, d.openTools),
    getExtensions: pickBool(rec.getExtensions, d.getExtensions),
    tutorial: pickBool(rec.tutorial, d.tutorial),
  };
}

// ---------------------------------------------------------------------------
// L15 手杀全扩 (extension.js:1597..1676)
// ---------------------------------------------------------------------------
export interface L15PoolConfig {
  jinjiang: boolean;
  jinjiang2: boolean;
  huanyuan: boolean;
  yinjian: boolean;
  unseen: "forbidai" | "unseen";
  packs: boolean;
  txhj: boolean;
  sexforbid: "none" | "male" | "female";
  shoushanameX: "none" | "shousha" | "all";
  nailong: boolean;
  infofix: boolean;
}

export const L15_CONFIG_SPECS: ReadonlyArray<ConfigSpecEntry<keyof L15PoolConfig>> = [
  { key: "jinjiang", label: "手杀全扩主开关", control: "boolean", upstreamLine: 1597, ruleCoupled: true },
  { key: "jinjiang2", label: "手杀动态禁将", control: "boolean", upstreamLine: 1602, ruleCoupled: true },
  { key: "huanyuan", label: "过滤将池（排除标将）", control: "boolean", upstreamLine: 1607, ruleCoupled: true },
  { key: "yinjian", label: "全阴将池", control: "boolean", upstreamLine: 1612, ruleCoupled: true },
  { key: "unseen", label: "屏蔽方式", control: "select", upstreamLine: 1617, ruleCoupled: true, options: [{ value: "forbidai", label: "对局选将屏蔽" }, { value: "unseen", label: "武将直接消失" }] },
  { key: "packs", label: "将包同步屏蔽", control: "boolean", upstreamLine: 1626, ruleCoupled: true },
  { key: "txhj", label: "太虚禁将过滤", control: "boolean", upstreamLine: 1631, ruleCoupled: true },
  { key: "sexforbid", label: "同性专场筛选", control: "select", upstreamLine: 1636, ruleCoupled: true, options: [{ value: "none", label: "不限" }, { value: "male", label: "男性" }, { value: "female", label: "女性" }] },
  { key: "shoushanameX", label: "手杀改名（去前缀）", control: "select", upstreamLine: 1656, ruleCoupled: false, options: [{ value: "none", label: "关闭" }, { value: "shousha", label: "仅手杀" }, { value: "all", label: "全部前缀" }] },
  { key: "nailong", label: "我是奶龙（激励提示）", control: "boolean", upstreamLine: 1671, ruleCoupled: false },
  { key: "infofix", label: "技能描述末尾符号补充", control: "boolean", upstreamLine: 1676, ruleCoupled: false },
];
const L15_KEY_SET: ReadonlySet<string> = new Set(L15_CONFIG_SPECS.map((s) => s.key));

export function createDefaultL15Config(): L15PoolConfig {
  return {
    jinjiang: true,
    jinjiang2: false,
    huanyuan: false,
    yinjian: false,
    unseen: "forbidai",
    packs: true,
    txhj: true,
    sexforbid: "none",
    shoushanameX: "shousha",
    nailong: false,
    infofix: true,
  };
}

export function importL15ConfigJson(jsonText: string): L15PoolConfig {
  const rec = rejectUnknownKeys(JSON.parse(jsonText), L15_KEY_SET, "L15_CONFIG");
  const d = createDefaultL15Config();
  return {
    jinjiang: pickBool(rec.jinjiang, d.jinjiang),
    jinjiang2: pickBool(rec.jinjiang2, d.jinjiang2),
    huanyuan: pickBool(rec.huanyuan, d.huanyuan),
    yinjian: pickBool(rec.yinjian, d.yinjian),
    unseen: pickOpt(rec.unseen, ["forbidai", "unseen"], d.unseen),
    packs: pickBool(rec.packs, d.packs),
    txhj: pickBool(rec.txhj, d.txhj),
    sexforbid: pickOpt(rec.sexforbid, ["none", "male", "female"], d.sexforbid),
    shoushanameX: pickOpt(rec.shoushanameX, ["none", "shousha", "all"], d.shoushanameX),
    nailong: pickBool(rec.nailong, d.nailong),
    infofix: pickBool(rec.infofix, d.infofix),
  };
}

// ---------------------------------------------------------------------------
// L16 拖拽读取 (extension.js:646,667 + extension.css:1-185)
// ---------------------------------------------------------------------------
export interface L16DragReadConfig {
  loadUpdateContent: boolean;
  generateZip: boolean;
}
export const L16_CONFIG_SPECS: ReadonlyArray<ConfigSpecEntry<keyof L16DragReadConfig>> = [
  { key: "loadUpdateContent", label: "显示拖拽读取更新日志", control: "action", upstreamLine: 646, ruleCoupled: false },
  { key: "generateZip", label: "一键导出包清单预览（净室）", control: "action", upstreamLine: 667, ruleCoupled: false },
];
export const L16_SURFACE_IDS = ["extension.css"] as const;
const L16_KEY_SET: ReadonlySet<string> = new Set(L16_CONFIG_SPECS.map((s) => s.key));

export function createDefaultL16Config(): L16DragReadConfig {
  return { loadUpdateContent: false, generateZip: false };
}
export function importL16ConfigJson(jsonText: string): L16DragReadConfig {
  const rec = rejectUnknownKeys(JSON.parse(jsonText), L16_KEY_SET, "L16_CONFIG");
  const d = createDefaultL16Config();
  return {
    loadUpdateContent: pickBool(rec.loadUpdateContent, d.loadUpdateContent),
    generateZip: pickBool(rec.generateZip, d.generateZip),
  };
}

// ---------------------------------------------------------------------------
// L17 斗转星移 (extension.js:1778..1833 + base.css:1-1046 + hook banCharF)
// ---------------------------------------------------------------------------
export interface L17DouzhuanConfig {
  JinJiang: boolean;
  JinJiangType: "forbidai" | "unseen";
  JinJiangRuZhen: "online" | "offline" | "online+offline";
  jjGroupColor: boolean;
  qnssBackground: boolean;
  AIqianghuaXuanjiang: boolean;
  qidongyeMeiHua: boolean;
  miaoshuXiuGai: boolean;
  forbidReload: boolean;
}
export const L17_CONFIG_SPECS: ReadonlyArray<ConfigSpecEntry<keyof L17DouzhuanConfig>> = [
  { key: "JinJiang", label: "禁将菜单栏入口", control: "boolean", upstreamLine: 1778, ruleCoupled: false },
  { key: "JinJiangType", label: "禁将方案", control: "select", upstreamLine: 1783, ruleCoupled: false, options: [{ value: "forbidai", label: "选将禁选" }, { value: "unseen", label: "全扩隐藏" }] },
  { key: "JinJiangRuZhen", label: "选择禁将模式", control: "select", upstreamLine: 1792, ruleCoupled: false, options: [{ value: "online", label: "仅如真生效" }, { value: "offline", label: "仅原版生效" }, { value: "online+offline", label: "如真/原版生效" }] },
  { key: "jjGroupColor", label: "禁将势力颜色区分", control: "boolean", upstreamLine: 1802, ruleCoupled: false },
  { key: "qnssBackground", label: "禁将界面跟随壁纸", control: "boolean", upstreamLine: 1808, ruleCoupled: false },
  { key: "AIqianghuaXuanjiang", label: "AI 强化选将偏向高星", control: "boolean", upstreamLine: 1813, ruleCoupled: true },
  { key: "qidongyeMeiHua", label: "启动页美化", control: "boolean", upstreamLine: 1822, ruleCoupled: false },
  { key: "miaoshuXiuGai", label: "技能描述 ≤/≥ 与阴阳互换", control: "boolean", upstreamLine: 1828, ruleCoupled: false },
  { key: "forbidReload", label: "禁止切换禁将时强行重启", control: "boolean", upstreamLine: 1833, ruleCoupled: false },
];
export const L17_SURFACE_IDS = ["base.css"] as const;
export const L17_HOOK_IDS = ["banCharF"] as const;
const L17_KEY_SET: ReadonlySet<string> = new Set(L17_CONFIG_SPECS.map((s) => s.key));

export function createDefaultL17Config(): L17DouzhuanConfig {
  return {
    JinJiang: true,
    JinJiangType: "forbidai",
    JinJiangRuZhen: "online",
    jjGroupColor: true,
    qnssBackground: true,
    AIqianghuaXuanjiang: true,
    qidongyeMeiHua: false,
    miaoshuXiuGai: false,
    forbidReload: true,
  };
}

export function importL17ConfigJson(jsonText: string): L17DouzhuanConfig {
  const rec = rejectUnknownKeys(JSON.parse(jsonText), L17_KEY_SET, "L17_CONFIG");
  const d = createDefaultL17Config();
  return {
    JinJiang: pickBool(rec.JinJiang, d.JinJiang),
    JinJiangType: pickOpt(rec.JinJiangType, ["forbidai", "unseen"], d.JinJiangType),
    JinJiangRuZhen: pickOpt(rec.JinJiangRuZhen, ["online", "offline", "online+offline"], d.JinJiangRuZhen),
    jjGroupColor: pickBool(rec.jjGroupColor, d.jjGroupColor),
    qnssBackground: pickBool(rec.qnssBackground, d.qnssBackground),
    AIqianghuaXuanjiang: pickBool(rec.AIqianghuaXuanjiang, d.AIqianghuaXuanjiang),
    qidongyeMeiHua: pickBool(rec.qidongyeMeiHua, d.qidongyeMeiHua),
    miaoshuXiuGai: pickBool(rec.miaoshuXiuGai, d.miaoshuXiuGai),
    forbidReload: pickBool(rec.forbidReload, d.forbidReload),
  };
}

// ---------------------------------------------------------------------------
// L18 更新保护 (extension.js:1225..2058 + extension.css:1-18)
// ---------------------------------------------------------------------------
export interface L18UpdateGuardConfig {
  show_version: boolean;
  show_qqqun: boolean;
  update_link_explain: "无" | "coding" | "github" | "fastgit" | "xuanwu" | "URC";
  update_link: "coding" | "github" | "URC";
  checkForUpdate: boolean;
  checkForAssetUpdate: boolean;
  cancelForbid: boolean;
  autoAddExts: boolean;
  assetAudio: boolean;
  assetSkin: boolean;
  assetImage: boolean;
  assetImageFull: boolean;
}
export const L18_CONFIG_SPECS: ReadonlyArray<ConfigSpecEntry<keyof L18UpdateGuardConfig>> = [
  { key: "show_version", label: "显示琉璃专属版本信息", control: "info", upstreamLine: 1225, ruleCoupled: false },
  { key: "show_qqqun", label: "显示交流群说明", control: "info", upstreamLine: 1230, ruleCoupled: false },
  {
    key: "update_link_explain",
    label: "更新通道说明查看",
    control: "select",
    upstreamLine: 1235,
    ruleCoupled: false,
    options: [
      { value: "无", label: "无" },
      { value: "coding", label: "Coding" },
      { value: "github", label: "GitHub" },
      { value: "fastgit", label: "GitHub镜像" },
      { value: "xuanwu", label: "玄武镜像(已弃用)" },
      { value: "URC", label: "URC" },
    ],
  },
  {
    key: "update_link",
    label: "更新源地址",
    control: "select",
    upstreamLine: 1271,
    ruleCoupled: false,
    options: [{ value: "coding", label: "Coding" }, { value: "github", label: "GitHub" }, { value: "URC", label: "URC" }],
  },
  { key: "checkForUpdate", label: "检查游戏更新（离线版本探测）", control: "action", upstreamLine: 1380, ruleCoupled: false },
  { key: "checkForAssetUpdate", label: "检查素材更新（进度条）", control: "action", upstreamLine: 1702, ruleCoupled: false },
  { key: "cancelForbid", label: "锁定基础扩展（防误关）", control: "boolean", upstreamLine: 2024, ruleCoupled: false },
  { key: "autoAddExts", label: "自动识别扩展目录", control: "boolean", upstreamLine: 2029, ruleCoupled: false },
  { key: "assetAudio", label: "检查音频素材", control: "boolean", upstreamLine: 2040, ruleCoupled: false },
  { key: "assetSkin", label: "检查皮肤素材", control: "boolean", upstreamLine: 2046, ruleCoupled: false },
  { key: "assetImage", label: "检查图片素材(部分)", control: "boolean", upstreamLine: 2052, ruleCoupled: false },
  { key: "assetImageFull", label: "检查图片素材(全部)", control: "boolean", upstreamLine: 2058, ruleCoupled: false },
];
export const L18_SURFACE_IDS = ["extension.css"] as const;
const L18_KEY_SET: ReadonlySet<string> = new Set(L18_CONFIG_SPECS.map((s) => s.key));

export function createDefaultL18Config(): L18UpdateGuardConfig {
  return {
    show_version: true,
    show_qqqun: true,
    update_link_explain: "无",
    update_link: "URC",
    checkForUpdate: false,
    checkForAssetUpdate: false,
    cancelForbid: true,
    autoAddExts: true,
    assetAudio: true,
    assetSkin: false,
    assetImage: true,
    assetImageFull: true,
  };
}

export function importL18ConfigJson(jsonText: string): L18UpdateGuardConfig {
  const rec = rejectUnknownKeys(JSON.parse(jsonText), L18_KEY_SET, "L18_CONFIG");
  const d = createDefaultL18Config();
  return {
    show_version: pickBool(rec.show_version, d.show_version),
    show_qqqun: pickBool(rec.show_qqqun, d.show_qqqun),
    update_link_explain: pickOpt(rec.update_link_explain, ["无", "coding", "github", "fastgit", "xuanwu", "URC"], d.update_link_explain),
    update_link: pickOpt(rec.update_link, ["coding", "github", "URC"], d.update_link),
    checkForUpdate: pickBool(rec.checkForUpdate, d.checkForUpdate),
    checkForAssetUpdate: pickBool(rec.checkForAssetUpdate, d.checkForAssetUpdate),
    cancelForbid: pickBool(rec.cancelForbid, d.cancelForbid),
    autoAddExts: pickBool(rec.autoAddExts, d.autoAddExts),
    assetAudio: pickBool(rec.assetAudio, d.assetAudio),
    assetSkin: pickBool(rec.assetSkin, d.assetSkin),
    assetImage: pickBool(rec.assetImage, d.assetImage),
    assetImageFull: pickBool(rec.assetImageFull, d.assetImageFull),
  };
}

// ---------------------------------------------------------------------------
// L21 武皇争霸 (extension.js:1201..1275 + hiddenAvatar.css:1-17)
// ---------------------------------------------------------------------------
export type L21CountOption = "2" | "3" | "4" | "5" | "6" | "7" | "8";
export interface L21MartialConfig {
  show_number: L21CountOption;
  battle_number: L21CountOption;
  hide_number: L21CountOption;
  double_character: boolean;
  double_hp: "hejiansan" | "pingjun" | "zuidazhi" | "zuixiaozhi" | "zonghe";
  free_choose: boolean;
  change_choice: boolean;
}
const L21_NUMS: ReadonlyArray<L21CountOption> = ["2", "3", "4", "5", "6", "7", "8"];
export const L21_CONFIG_SPECS: ReadonlyArray<ConfigSpecEntry<keyof L21MartialConfig>> = [
  { key: "show_number", label: "选池人数", control: "select", upstreamLine: 1201, ruleCoupled: true, options: L21_NUMS.map((v) => ({ value: v, label: `${v}人` })) },
  { key: "battle_number", label: "出场人数", control: "select", upstreamLine: 1217, ruleCoupled: true, options: L21_NUMS.map((v) => ({ value: v, label: `${v}人` })) },
  { key: "hide_number", label: "暗置背面人数", control: "select", upstreamLine: 1233, ruleCoupled: false, options: L21_NUMS.map((v) => ({ value: v, label: `${v}人` })) },
  { key: "double_character", label: "武皇双将模式", control: "boolean", upstreamLine: 1249, ruleCoupled: true },
  {
    key: "double_hp",
    label: "双将体力上限算法",
    control: "select",
    upstreamLine: 1256,
    ruleCoupled: true,
    options: [
      { value: "hejiansan", label: "和减三" },
      { value: "pingjun", label: "平均值" },
      { value: "zuidazhi", label: "最大值" },
      { value: "zuixiaozhi", label: "最小值" },
      { value: "zonghe", label: "相加" },
    ],
  },
  { key: "free_choose", label: "自由选将", control: "boolean", upstreamLine: 1268, ruleCoupled: true },
  { key: "change_choice", label: "换选重抽", control: "boolean", upstreamLine: 1275, ruleCoupled: true },
];
export const L21_SURFACE_IDS = ["hiddenAvatar.css"] as const;
const L21_KEY_SET: ReadonlySet<string> = new Set(L21_CONFIG_SPECS.map((s) => s.key));

export function createDefaultL21Config(): L21MartialConfig {
  return {
    show_number: "5",
    battle_number: "5",
    hide_number: "2",
    double_character: false,
    double_hp: "hejiansan",
    free_choose: false,
    change_choice: false,
  };
}

export function normalizeL21Config(raw?: Partial<L21MartialConfig> | null): L21MartialConfig {
  const d = createDefaultL21Config();
  if (!raw) return d;
  const show = pickOpt(raw.show_number, L21_NUMS, d.show_number);
  const showN = Number(show);
  const battleRaw = pickOpt(raw.battle_number, L21_NUMS, d.battle_number);
  const hideRaw = pickOpt(raw.hide_number, L21_NUMS, d.hide_number);
  return {
    show_number: show,
    battle_number: String(Math.min(showN, Number(battleRaw))) as L21CountOption,
    hide_number: String(Math.min(showN, Number(hideRaw))) as L21CountOption,
    double_character: pickBool(raw.double_character, d.double_character),
    double_hp: pickOpt(raw.double_hp, ["hejiansan", "pingjun", "zuidazhi", "zuixiaozhi", "zonghe"], d.double_hp),
    free_choose: pickBool(raw.free_choose, d.free_choose),
    change_choice: pickBool(raw.change_choice, d.change_choice),
  };
}

export function importL21ConfigJson(jsonText: string): L21MartialConfig {
  const rec = rejectUnknownKeys(JSON.parse(jsonText), L21_KEY_SET, "L21_CONFIG");
  return normalizeL21Config(rec as Partial<L21MartialConfig>);
}

// ---------------------------------------------------------------------------
// U05 杀海拾遗 (archive/upstream/noname/apps/core/extension/杀海拾遗/main/config.js:4..44)
// ---------------------------------------------------------------------------
export interface U05ShahaiConfig {
  tip: boolean;
  wuxing: boolean;
  rand: "0.1" | "0.2" | "0.3" | "0.5";
  mtg: boolean;
  zhenfa: boolean;
  gwent: boolean;
  yunchou: boolean;
}
export const U05_CONFIG_SPECS: ReadonlyArray<ConfigSpecEntry<keyof U05ShahaiConfig>> = [
  { key: "tip", label: "长按提示说明", control: "info", upstreamLine: 4, ruleCoupled: false },
  { key: "wuxing", label: "五行生克随机属性", control: "boolean", upstreamLine: 14, ruleCoupled: true },
  { key: "rand", label: "带属性卡牌概率", control: "select", upstreamLine: 19, ruleCoupled: true, options: [{ value: "0.1", label: "10%" }, { value: "0.2", label: "20%" }, { value: "0.3", label: "30%" }, { value: "0.5", label: "50%" }] },
  { key: "mtg", label: "万智牌扩展子包", control: "boolean", upstreamLine: 29, ruleCoupled: true },
  { key: "zhenfa", label: "阵法牌扩展子包", control: "boolean", upstreamLine: 34, ruleCoupled: true },
  { key: "gwent", label: "昆特牌扩展子包", control: "boolean", upstreamLine: 39, ruleCoupled: true },
  { key: "yunchou", label: "运筹帷幄扩展子包", control: "boolean", upstreamLine: 44, ruleCoupled: true },
];
const U05_KEY_SET: ReadonlySet<string> = new Set(U05_CONFIG_SPECS.map((s) => s.key));

export function createDefaultU05Config(): U05ShahaiConfig {
  return { tip: true, wuxing: false, rand: "0.3", mtg: true, zhenfa: true, gwent: true, yunchou: true };
}
export function importU05ConfigJson(jsonText: string): U05ShahaiConfig {
  const rec = rejectUnknownKeys(JSON.parse(jsonText), U05_KEY_SET, "U05_CONFIG");
  const d = createDefaultU05Config();
  return {
    tip: pickBool(rec.tip, d.tip),
    wuxing: pickBool(rec.wuxing, d.wuxing),
    rand: pickOpt(rec.rand, ["0.1", "0.2", "0.3", "0.5"], d.rand),
    mtg: pickBool(rec.mtg, d.mtg),
    zhenfa: pickBool(rec.zhenfa, d.zhenfa),
    gwent: pickBool(rec.gwent, d.gwent),
    yunchou: pickBool(rec.yunchou, d.yunchou),
  };
}
