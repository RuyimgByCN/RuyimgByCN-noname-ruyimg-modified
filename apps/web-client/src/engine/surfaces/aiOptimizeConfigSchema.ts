/**
 * L01 AI优化 全部 63 项配置 1-to-1 Schema（对照 `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9856-10943`）
 *
 * 严格遵循净室与裁决 38 边界：
 * - 上游 `editWj` / `editQz` / `editCf` 使用 CodeMirror + `eval(code)`；
 *   原生前端统一改为纯 JSON / 结构化表单校验（`importL01ConfigJson` 等，绝不执行 `eval`/`Function`）。
 * - 诚实标注纯 UI/配置工作台项 与 依赖原生 Rust 规则/AI 引擎配合项（`ruleCoupled`）。
 */

export type L01ConfigCategory =
  | "info"
  | "ai_strategy"
  | "common"
  | "identity"
  | "banlist"
  | "weight_threat";

export type L01ConfigControl = "info" | "boolean" | "select" | "action";

export interface L01ConfigSpec {
  readonly key: keyof L01AiOptimizeConfig;
  readonly label: string;
  readonly category: L01ConfigCategory;
  readonly control: L01ConfigControl;
  readonly upstreamLine: number;
  readonly ruleCoupled: boolean;
  readonly options?: ReadonlyArray<{ value: string; label: string }>;
  readonly note?: string;
}

export interface L01AiOptimizeConfig {
  kzjs: "jieshao";
  bd1: boolean;
  copyQg: string;
  copyWp: string;
  copyYp: string;
  bd2: boolean;
  btAi: "shyg" | "gjcx" | "off";
  zbAi: boolean;
  wjAi: boolean;
  sfjAi: "shyg" | "gjcx" | "off";
  nhFriends: "off" | "1" | "2" | "3" | "4" | "hp";
  ntAoe: boolean;
  mjAi: boolean;
  bd3: boolean;
  viewAtt: boolean;
  xyyh: boolean;
  banQh: boolean;
  seen: boolean;
  hsyh: boolean;
  bd4: boolean;
  findZhong: boolean;
  neiKey: "off" | "can" | "must";
  attceshi: boolean;
  fixFour: boolean;
  bd5: boolean;
  Wj: boolean;
  wjs: "same" | "1" | "2" | "3" | "4" | "5" | "6" | "8" | "10";
  fixWj: boolean;
  editWj: string[];
  copyWj: string;
  clearWj: number;
  tip1: boolean;
  banzhu: string[];
  banzhubiao: number;
  banzhong: string[];
  banzhongbiao: number;
  banfan: string[];
  banfanbiao: number;
  bannei: string[];
  banneibiao: number;
  bandizhu: string[];
  bandizhubiao: number;
  bannongmin: string[];
  bannongminbiao: number;
  bd6: boolean;
  tip2: boolean;
  fixEQz: boolean;
  applyQz: boolean;
  ckQz: "off" | "pj" | "cf";
  qzCf: boolean;
  chooseQz: string;
  deleteQz: string;
  editQz: Record<string, number>;
  copyQz: string;
  clearQz: number;
  tip3: boolean;
  fixECf: boolean;
  applyCf: "off" | "pj" | "sd";
  chooseCf: string;
  deleteCf: string;
  editCf: Record<string, number>;
  copyCf: string;
  clearCf: number;
}

export const L01_CONFIG_SPECS: ReadonlyArray<L01ConfigSpec> = [
  { key: "kzjs", label: "扩展介绍（山河永固/官将重修）", category: "info", control: "select", upstreamLine: 9856, ruleCoupled: false, options: [{ value: "jieshao", label: "点击查看" }] },
  { key: "bd1", label: "渠道与版本说明分隔线", category: "info", control: "info", upstreamLine: 9888, ruleCoupled: false },
  { key: "copyQg", label: "复制交流群号（只读展示）", category: "info", control: "action", upstreamLine: 9892, ruleCoupled: false },
  { key: "copyWp", label: "复制网盘参考信息", category: "info", control: "action", upstreamLine: 9909, ruleCoupled: false },
  { key: "copyYp", label: "复制云盘参考信息", category: "info", control: "action", upstreamLine: 9926, ruleCoupled: false },
  { key: "bd2", label: "AI 相关分隔线", category: "ai_strategy", control: "info", upstreamLine: 9943, ruleCoupled: false },
  { key: "btAi", label: "本体 AI 优化策略", category: "ai_strategy", control: "select", upstreamLine: 9948, ruleCoupled: true, options: [{ value: "shyg", label: "山河永固" }, { value: "gjcx", label: "官将重修" }, { value: "off", label: "关闭" }] },
  { key: "zbAi", label: "装备 AI 优化", category: "ai_strategy", control: "boolean", upstreamLine: 9958, ruleCoupled: true },
  { key: "wjAi", label: "武将技能 AI 优化", category: "ai_strategy", control: "boolean", upstreamLine: 9963, ruleCoupled: true },
  { key: "sfjAi", label: "身份局 AI 优化策略", category: "ai_strategy", control: "select", upstreamLine: 9968, ruleCoupled: true, options: [{ value: "shyg", label: "山河永固" }, { value: "gjcx", label: "官将重修" }, { value: "off", label: "关闭" }] },
  { key: "nhFriends", label: "AI 不砍队友阈值", category: "ai_strategy", control: "select", upstreamLine: 9978, ruleCoupled: true, options: [{ value: "off", label: "关闭" }, { value: "1", label: "<=1血队友" }, { value: "2", label: "<=2血队友" }, { value: "3", label: "<=3血队友" }, { value: "4", label: "<=4血队友" }, { value: "hp", label: "<=其体力值队友" }] },
  { key: "ntAoe", label: "AOE 不受队友保护限制", category: "ai_strategy", control: "boolean", upstreamLine: 9991, ruleCoupled: true },
  { key: "mjAi", label: "盲狙激进 AI", category: "ai_strategy", control: "boolean", upstreamLine: 9995, ruleCoupled: true },
  { key: "bd3", label: "常用功能分隔线", category: "common", control: "info", upstreamLine: 10000, ruleCoupled: false },
  { key: "viewAtt", label: "火眼金睛（查看态度面板）", category: "common", control: "boolean", upstreamLine: 10005, ruleCoupled: false },
  { key: "xyyh", label: "闲云野鹤（跳过部分小游戏）", category: "common", control: "boolean", upstreamLine: 10010, ruleCoupled: true },
  { key: "banQh", label: "屏蔽选将切换按钮", category: "common", control: "boolean", upstreamLine: 10015, ruleCoupled: false },
  { key: "seen", label: "显示本体隐藏武将", category: "common", control: "boolean", upstreamLine: 10020, ruleCoupled: false },
  { key: "hsyh", label: "本体展示与拼点函数优化", category: "common", control: "boolean", upstreamLine: 10026, ruleCoupled: true },
  { key: "bd4", label: "身份局功能分隔线", category: "identity", control: "info", upstreamLine: 10031, ruleCoupled: false },
  { key: "findZhong", label: "慧眼识忠（主公开局知晓一忠）", category: "identity", control: "boolean", upstreamLine: 10036, ruleCoupled: true },
  { key: "neiKey", label: "内奸出牌阶段亮明身份", category: "identity", control: "select", upstreamLine: 10041, ruleCoupled: true, options: [{ value: "off", label: "关闭" }, { value: "can", label: "未暴露可亮" }, { value: "must", label: "暴露也可亮" }] },
  { key: "attceshi", label: "态度测试与伪装", category: "identity", control: "boolean", upstreamLine: 10051, ruleCoupled: true },
  { key: "fixFour", label: "四人身份局修改（1主1内2反）", category: "identity", control: "boolean", upstreamLine: 10056, ruleCoupled: true },
  { key: "bd5", label: "伪禁相关分隔线", category: "banlist", control: "info", upstreamLine: 10061, ruleCoupled: false },
  { key: "Wj", label: "伪玩家可选 AI 禁选", category: "banlist", control: "boolean", upstreamLine: 10065, ruleCoupled: true },
  { key: "wjs", label: "伪禁候选武将数", category: "banlist", control: "select", upstreamLine: 10070, ruleCoupled: true, options: [{ value: "same", label: "默认" }, { value: "1", label: "一" }, { value: "2", label: "二" }, { value: "3", label: "三" }, { value: "4", label: "四" }, { value: "5", label: "五" }, { value: "6", label: "六" }, { value: "8", label: "八" }, { value: "10", label: "十" }] },
  { key: "fixWj", label: "出牌阶段可编辑伪禁列表", category: "banlist", control: "boolean", upstreamLine: 10086, ruleCoupled: false },
  { key: "editWj", label: "编辑伪禁列表（结构化 JSON）", category: "banlist", control: "action", upstreamLine: 10091, ruleCoupled: false },
  { key: "copyWj", label: "导出/复制伪禁列表", category: "banlist", control: "action", upstreamLine: 10174, ruleCoupled: false },
  { key: "clearWj", label: "清空伪禁列表", category: "banlist", control: "action", upstreamLine: 10195, ruleCoupled: false },
  { key: "tip1", label: "伪禁衍生提示", category: "banlist", control: "info", upstreamLine: 10205, ruleCoupled: false },
  { key: "banzhu", label: "主公 AI 禁将操作", category: "banlist", control: "action", upstreamLine: 10209, ruleCoupled: true },
  { key: "banzhubiao", label: "查看主公 AI 禁选表", category: "banlist", control: "action", upstreamLine: 10250, ruleCoupled: false },
  { key: "banzhong", label: "忠臣 AI 禁将操作", category: "banlist", control: "action", upstreamLine: 10274, ruleCoupled: true },
  { key: "banzhongbiao", label: "查看忠臣 AI 禁选表", category: "banlist", control: "action", upstreamLine: 10315, ruleCoupled: false },
  { key: "banfan", label: "反贼 AI 禁将操作", category: "banlist", control: "action", upstreamLine: 10339, ruleCoupled: true },
  { key: "banfanbiao", label: "查看反贼 AI 禁选表", category: "banlist", control: "action", upstreamLine: 10380, ruleCoupled: false },
  { key: "bannei", label: "内奸 AI 禁将操作", category: "banlist", control: "action", upstreamLine: 10404, ruleCoupled: true },
  { key: "banneibiao", label: "查看内奸 AI 禁选表", category: "banlist", control: "action", upstreamLine: 10445, ruleCoupled: false },
  { key: "bandizhu", label: "地主 AI 禁将操作", category: "banlist", control: "action", upstreamLine: 10469, ruleCoupled: true },
  { key: "bandizhubiao", label: "查看地主 AI 禁选表", category: "banlist", control: "action", upstreamLine: 10510, ruleCoupled: false },
  { key: "bannongmin", label: "农民 AI 禁将操作", category: "banlist", control: "action", upstreamLine: 10534, ruleCoupled: true },
  { key: "bannongminbiao", label: "查看农民 AI 禁选表", category: "banlist", control: "action", upstreamLine: 10575, ruleCoupled: false },
  { key: "bd6", label: "杂项分隔线", category: "weight_threat", control: "info", upstreamLine: 10599, ruleCoupled: false },
  { key: "tip2", label: "武将权重用途说明", category: "weight_threat", control: "info", upstreamLine: 10604, ruleCoupled: false },
  { key: "fixEQz", label: "出牌可修改武将权重", category: "weight_threat", control: "boolean", upstreamLine: 10608, ruleCoupled: true },
  { key: "applyQz", label: "武将登场补充权重", category: "weight_threat", control: "boolean", upstreamLine: 10613, ruleCoupled: true },
  { key: "ckQz", label: "第二权重参考", category: "weight_threat", control: "select", upstreamLine: 10618, ruleCoupled: true, options: [{ value: "off", label: "不设置" }, { value: "pj", label: "评级" }, { value: "cf", label: "威胁度" }] },
  { key: "qzCf", label: "权重参与效益结算", category: "weight_threat", control: "boolean", upstreamLine: 10628, ruleCoupled: true },
  { key: "chooseQz", label: "选择已设权重武将", category: "weight_threat", control: "action", upstreamLine: 10633, ruleCoupled: false },
  { key: "deleteQz", label: "删除选中武将权重", category: "weight_threat", control: "action", upstreamLine: 10638, ruleCoupled: false },
  { key: "editQz", label: "编辑武将权重（结构化 JSON）", category: "weight_threat", control: "action", upstreamLine: 10651, ruleCoupled: false },
  { key: "copyQz", label: "复制武将权重映射", category: "weight_threat", control: "action", upstreamLine: 10742, ruleCoupled: false },
  { key: "clearQz", label: "清空武将权重映射", category: "weight_threat", control: "action", upstreamLine: 10763, ruleCoupled: false },
  { key: "tip3", label: "技能威胁度非正数门禁说明", category: "weight_threat", control: "info", upstreamLine: 10777, ruleCoupled: false },
  { key: "fixECf", label: "出牌可修改技能威胁度", category: "weight_threat", control: "boolean", upstreamLine: 10781, ruleCoupled: true },
  { key: "applyCf", label: "威胁度补充模式", category: "weight_threat", control: "select", upstreamLine: 10786, ruleCoupled: true, options: [{ value: "off", label: "不补充" }, { value: "pj", label: "自动补充" }, { value: "sd", label: "手动补充" }] },
  { key: "chooseCf", label: "选择已改技能威胁度", category: "weight_threat", control: "action", upstreamLine: 10796, ruleCoupled: false },
  { key: "deleteCf", label: "删除选中技能威胁度", category: "weight_threat", control: "action", upstreamLine: 10801, ruleCoupled: false },
  { key: "editCf", label: "编辑技能威胁度（正数 JSON）", category: "weight_threat", control: "action", upstreamLine: 10814, ruleCoupled: false },
  { key: "copyCf", label: "复制技能威胁度映射", category: "weight_threat", control: "action", upstreamLine: 10905, ruleCoupled: false },
  { key: "clearCf", label: "清空技能威胁度映射", category: "weight_threat", control: "action", upstreamLine: 10926, ruleCoupled: false },
];

export const L01_CONFIG_KEYS: ReadonlyArray<keyof L01AiOptimizeConfig> = L01_CONFIG_SPECS.map((s) => s.key);
export const L01_CONFIG_KEY_SET: ReadonlySet<string> = new Set(L01_CONFIG_KEYS);

export function createDefaultL01Config(): L01AiOptimizeConfig {
  return {
    kzjs: "jieshao",
    bd1: true,
    copyQg: "715181494",
    copyWp: "local-backup://ai-opt-wp",
    copyYp: "local-backup://ai-opt-yp",
    bd2: true,
    btAi: "shyg",
    zbAi: true,
    wjAi: true,
    sfjAi: "shyg",
    nhFriends: "off",
    ntAoe: false,
    mjAi: false,
    bd3: true,
    viewAtt: false,
    xyyh: false,
    banQh: false,
    seen: false,
    hsyh: true,
    bd4: true,
    findZhong: false,
    neiKey: "off",
    attceshi: true,
    fixFour: false,
    bd5: true,
    Wj: false,
    wjs: "same",
    fixWj: false,
    editWj: [],
    copyWj: "",
    clearWj: 0,
    tip1: true,
    banzhu: [],
    banzhubiao: 0,
    banzhong: [],
    banzhongbiao: 0,
    banfan: [],
    banfanbiao: 0,
    bannei: [],
    banneibiao: 0,
    bandizhu: [],
    bandizhubiao: 0,
    bannongmin: [],
    bannongminbiao: 0,
    bd6: true,
    tip2: true,
    fixEQz: false,
    applyQz: false,
    ckQz: "cf",
    qzCf: true,
    chooseQz: "shen_zhugeliang",
    deleteQz: "",
    editQz: { shen_zhugeliang: 1.75, liangxing: 1.57 },
    copyQz: "",
    clearQz: 0,
    tip3: true,
    fixECf: false,
    applyCf: "off",
    chooseCf: "gjcxs_zhugeliang3",
    deleteCf: "",
    editCf: { gjcxs_zhugeliang3: 1.9, gjcxs_caojie1: 0.7 },
    copyCf: "",
    clearCf: 0,
  };
}

function sanitizeStringArray(v: unknown, label: string): string[] {
  if (!Array.isArray(v)) throw new Error(`L01_CONFIG_INVALID_ARRAY:${label}`);
  return Array.from(new Set(v.map((x) => String(x).trim()).filter(Boolean)));
}

function sanitizePositiveMap(v: unknown, label: string, requirePositive: boolean): Record<string, number> {
  if (!v || typeof v !== "object" || Array.isArray(v)) {
    throw new Error(`L01_CONFIG_INVALID_MAP:${label}`);
  }
  const out: Record<string, number> = {};
  for (const [rawKey, rawVal] of Object.entries(v as Record<string, unknown>)) {
    const k = rawKey.trim();
    if (!k) continue;
    const num = Number(rawVal);
    if (!Number.isFinite(num) || (requirePositive && num <= 0)) {
      throw new Error(`L01_CONFIG_NON_POSITIVE_WEIGHT:${label}:${k}`);
    }
    out[k] = Number(num.toFixed(3));
  }
  return out;
}

export function normalizeL01Config(raw?: Partial<L01AiOptimizeConfig> | null): L01AiOptimizeConfig {
  const base = createDefaultL01Config();
  if (!raw || typeof raw !== "object") return base;
  const pick = <T extends string>(val: unknown, allowed: ReadonlyArray<T>, fallback: T): T =>
    typeof val === "string" && (allowed as ReadonlyArray<string>).includes(val) ? (val as T) : fallback;
  const bool = (val: unknown, fallback: boolean) => (typeof val === "boolean" ? val : fallback);

  return {
    ...base,
    kzjs: "jieshao",
    bd1: bool(raw.bd1, base.bd1),
    copyQg: typeof raw.copyQg === "string" ? raw.copyQg : base.copyQg,
    copyWp: typeof raw.copyWp === "string" ? raw.copyWp : base.copyWp,
    copyYp: typeof raw.copyYp === "string" ? raw.copyYp : base.copyYp,
    bd2: bool(raw.bd2, base.bd2),
    btAi: pick(raw.btAi, ["shyg", "gjcx", "off"], base.btAi),
    zbAi: bool(raw.zbAi, base.zbAi),
    wjAi: bool(raw.wjAi, base.wjAi),
    sfjAi: pick(raw.sfjAi, ["shyg", "gjcx", "off"], base.sfjAi),
    nhFriends: pick(raw.nhFriends, ["off", "1", "2", "3", "4", "hp"], base.nhFriends),
    ntAoe: bool(raw.ntAoe, base.ntAoe),
    mjAi: bool(raw.mjAi, base.mjAi),
    bd3: bool(raw.bd3, base.bd3),
    viewAtt: bool(raw.viewAtt, base.viewAtt),
    xyyh: bool(raw.xyyh, base.xyyh),
    banQh: bool(raw.banQh, base.banQh),
    seen: bool(raw.seen, base.seen),
    hsyh: bool(raw.hsyh, base.hsyh),
    bd4: bool(raw.bd4, base.bd4),
    findZhong: bool(raw.findZhong, base.findZhong),
    neiKey: pick(raw.neiKey, ["off", "can", "must"], base.neiKey),
    attceshi: bool(raw.attceshi, base.attceshi),
    fixFour: bool(raw.fixFour, base.fixFour),
    bd5: bool(raw.bd5, base.bd5),
    Wj: bool(raw.Wj, base.Wj),
    wjs: pick(raw.wjs, ["same", "1", "2", "3", "4", "5", "6", "8", "10"], base.wjs),
    fixWj: bool(raw.fixWj, base.fixWj),
    editWj: Array.isArray(raw.editWj) ? raw.editWj.map(String) : [...base.editWj],
    banzhu: Array.isArray(raw.banzhu) ? raw.banzhu.map(String) : [...base.banzhu],
    banzhong: Array.isArray(raw.banzhong) ? raw.banzhong.map(String) : [...base.banzhong],
    banfan: Array.isArray(raw.banfan) ? raw.banfan.map(String) : [...base.banfan],
    bannei: Array.isArray(raw.bannei) ? raw.bannei.map(String) : [...base.bannei],
    bandizhu: Array.isArray(raw.bandizhu) ? raw.bandizhu.map(String) : [...base.bandizhu],
    bannongmin: Array.isArray(raw.bannongmin) ? raw.bannongmin.map(String) : [...base.bannongmin],
    editQz:
      raw.editQz && typeof raw.editQz === "object" && !Array.isArray(raw.editQz)
        ? { ...raw.editQz }
        : { ...base.editQz },
    editCf:
      raw.editCf && typeof raw.editCf === "object" && !Array.isArray(raw.editCf)
        ? { ...raw.editCf }
        : { ...base.editCf },
    fixEQz: bool(raw.fixEQz, base.fixEQz),
    applyQz: bool(raw.applyQz, base.applyQz),
    ckQz: pick(raw.ckQz, ["off", "pj", "cf"], base.ckQz),
    qzCf: bool(raw.qzCf, base.qzCf),
    chooseQz: typeof raw.chooseQz === "string" ? raw.chooseQz : base.chooseQz,
    fixECf: bool(raw.fixECf, base.fixECf),
    applyCf: pick(raw.applyCf, ["off", "pj", "sd"], base.applyCf),
    chooseCf: typeof raw.chooseCf === "string" ? raw.chooseCf : base.chooseCf,
  };
}

export function importL01ConfigJson(jsonText: string): L01AiOptimizeConfig {
  const parsed = JSON.parse(jsonText) as unknown;
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("L01_CONFIG_INVALID_JSON");
  }
  const obj = parsed as Record<string, unknown>;
  for (const key of Object.keys(obj)) {
    if (!L01_CONFIG_KEY_SET.has(key)) {
      throw new Error(`L01_CONFIG_UNKNOWN_KEY:${key}`);
    }
  }
  if (obj.editWj !== undefined) obj.editWj = sanitizeStringArray(obj.editWj, "editWj");
  for (const roleKey of ["banzhu", "banzhong", "banfan", "bannei", "bandizhu", "bannongmin"]) {
    if (obj[roleKey] !== undefined) obj[roleKey] = sanitizeStringArray(obj[roleKey], roleKey);
  }
  if (obj.editQz !== undefined) obj.editQz = sanitizePositiveMap(obj.editQz, "editQz", false);
  if (obj.editCf !== undefined) obj.editCf = sanitizePositiveMap(obj.editCf, "editCf", true);
  return normalizeL01Config(obj as Partial<L01AiOptimizeConfig>);
}
