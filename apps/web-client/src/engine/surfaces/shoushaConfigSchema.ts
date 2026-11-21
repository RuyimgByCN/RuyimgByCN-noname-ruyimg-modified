/**
 * L14 手杀ui 全部 48 项配置项数据结构、上游默认值、严格校验与 CSS Dataset 映射。
 * 上游对照: `archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:5489-5985, 7377-7454`
 * （严守裁决 35/38 净室边界：不读取 Spine 文件，不含商业资源，零 eval/外链）
 */

export type ShoushaConfigCategory =
  | "jindutiao"
  | "prompts"
  | "appearance"
  | "select"
  | "info";

export interface ShoushaConfigOption {
  readonly value: string;
  readonly label: string;
}

export interface ShoushaConfigSpec {
  readonly key: keyof ShoushaUiConfig;
  readonly label: string;
  readonly category: ShoushaConfigCategory;
  readonly control: "boolean" | "select" | "info";
  readonly upstreamLine: number;
  readonly options?: ReadonlyArray<ShoushaConfigOption>;
  readonly intro?: string;
}

export interface ShoushaUiConfig {
  ShouShaThanks: string;
  MoGaiMengXiu: string;
  KZJS: boolean;
  GXSM: boolean;
  MX: string;
  FL1: boolean;
  JDTSM: boolean;
  jindutiao: boolean;
  jindutiaoYangshi: "1" | "2" | "3";
  jindutiaotuoguan: boolean;
  jindutiaoST: "10" | "50" | "100" | "200" | "500" | "800" | "1000" | "2000";
  jindutiaoUpdata: boolean;
  jindutiaoaiUpdata: boolean;
  jindutiaoSet: string;
  jiangdeng: boolean;
  FL2: boolean;
  JDTSSM: boolean;
  JDTS: boolean;
  JDTSYangshi: "1" | "2";
  FL3: boolean;
  GTBBSM: boolean;
  GTBB: boolean;
  GTBBYangshi: "on" | "off";
  GTBBFont: "on" | "off";
  GTBBTime: "30000" | "60000" | "120000" | "180000" | "300000" | "random";
  FL4: boolean;
  JMYSSM: boolean;
  yangshi: "on" | "off";
  yangshi_bg: "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "follow";
  XPJ: "on" | "off";
  LTAN: boolean;
  xiandingskill_ss: boolean;
  FL5: boolean;
  QTMHSM: boolean;
  KGMH: "0" | "1" | "2";
  qiDongYe: "on" | "othersOn" | "othersTwo" | "off";
  JSAN: boolean;
  分割线01: boolean;
  sayplay: "off" | "old" | "chg";
  charactermenu: "none" | "old" | "new";
  xindmenu: "shizhounian" | "shousha" | "off";
  xindchoose: "shizhounian" | "shousha" | "off";
  xindchoose_type: "old" | "new";
  xindsingle: boolean;
  xinddraw: boolean;
  infinitydraw: boolean;
  分割线02: boolean;
  ZLLT: boolean;
}

const MX_OPTIONS: ReadonlyArray<ShoushaConfigOption> = [
  { value: "1", label: "鸣谢名单总览" },
  { value: "2", label: "永远的萌新（无懈/理牌修补）" },
  { value: "3", label: "Empty city°" },
  { value: "4", label: "阿七" },
  { value: "5", label: "神秘喵" },
  { value: "6", label: "fly" },
  { value: "7", label: "俺杀" },
  { value: "8", label: "呲牙哥" },
  { value: "9", label: "寻" },
  { value: "10", label: "喋血长歌" },
  { value: "11", label: "弩弩弩" },
  { value: "12", label: "萝卜" },
  { value: "13", label: "可宣" },
  { value: "14", label: "黄老板" },
  { value: "15", label: "只叹年华未央" },
  { value: "16", label: "风雪弥漫" },
  { value: "17", label: "阳光微凉" },
  { value: "18", label: "零二哟" },
  { value: "19", label: "群小乔" },
  { value: "20", label: "棘手怀念摧毁" },
];

const JINDUTIAO_HEIGHT_OPTIONS: ReadonlyArray<ShoushaConfigOption> = [
  "10", "15", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30",
  "31", "32", "33", "34", "35", "36", "37", "38", "39",
].map((v) => ({ value: v, label: `${v}%` }));

export const SHOUSHA_BG_OPTIONS: ReadonlyArray<ShoushaConfigOption> = [
  { value: "0", label: "经典款式" },
  { value: "1", label: "青铜图腾" },
  { value: "2", label: "新杀晚夜" },
  { value: "3", label: "新杀春日（动态）" },
  { value: "4", label: "曹节花园" },
  { value: "5", label: "谋定天下" },
  { value: "6", label: "羊皮画卷·壹" },
  { value: "7", label: "羊皮画卷·贰" },
  { value: "8", label: "节气壁纸（随机）" },
  { value: "follow", label: "随游戏壁纸" },
];

/** `手杀ui` 全部 48 项配置定义（1-to-1 对应 `g0/inventory_status.json` `L14:configs`） */
export const SHOUSHA_CONFIG_SPECS: ReadonlyArray<ShoushaConfigSpec> = [
  // info / credits / collapsible documentation entries (16 + MX)
  { key: "ShouShaThanks", label: "作者：橙续缘", category: "info", control: "info", upstreamLine: 5489, intro: "手杀UI原作者" },
  { key: "MoGaiMengXiu", label: "修改：萌新（转型中）", category: "info", control: "info", upstreamLine: 5493, intro: "手杀UI维护与转型适配" },
  { key: "KZJS", label: "扩展介绍", category: "info", control: "info", upstreamLine: 5497, intro: "偏向牌桌细节美化与阶段提示、进度条、播报及选将框协同增强。" },
  { key: "GXSM", label: "更新说明", category: "info", control: "info", upstreamLine: 5514, intro: "原生净室端对齐全部 48 配置、53 钩子与 40 项 CSS/JS 渲染面。" },
  { key: "MX", label: "鸣谢表", category: "info", control: "select", upstreamLine: 5529, options: MX_OPTIONS },
  { key: "FL1", label: "分隔线·进度条与将灯", category: "info", control: "info", upstreamLine: 5645 },
  { key: "JDTSM", label: "进度条说明", category: "info", control: "info", upstreamLine: 5652, intro: "覆盖己方回合内、AI回合内、己方回合外与AI回合外；支持10%–39%高度与刷新频率切换。" },
  { key: "FL2", label: "分隔线·阶段提示", category: "info", control: "info", upstreamLine: 5752 },
  { key: "JDTSSM", label: "阶段提示说明", category: "info", control: "info", upstreamLine: 5759, intro: "包含回合开始/判定/摸牌/出牌/弃牌/等待响应/对方思考中；太虚或侍灵模式自动避让。" },
  { key: "FL3", label: "分隔线·狗托播报", category: "info", control: "info", upstreamLine: 5793 },
  { key: "GTBBSM", label: "狗托播报说明", category: "info", control: "info", upstreamLine: 5800, intro: "顶部滚动播报栏，可独立切换手杀/十周年样式、字体与刷新间隔。" },
  { key: "FL4", label: "分隔线·界面样式", category: "info", control: "info", upstreamLine: 5857 },
  { key: "JMYSSM", label: "界面样式说明", category: "info", control: "info", upstreamLine: 5864, intro: "支持手杀布局与十周年布局、自动/手动理牌及身份猜想辅助提示。" },
  { key: "FL5", label: "分隔线·其他美化", category: "info", control: "info", upstreamLine: 5931 },
  { key: "QTMHSM", label: "其他美化说明", category: "info", control: "info", upstreamLine: 5938, intro: "包含开关美化、启动页布局预设与结算隐藏按钮。" },
  { key: "分割线01", label: "分隔线·说话与详情", category: "info", control: "info", upstreamLine: 7377 },
  { key: "分割线02", label: "分隔线·底部", category: "info", control: "info", upstreamLine: 7448 },

  // jindutiao & jiangdeng (8)
  { key: "jindutiao", label: "进度条", category: "jindutiao", control: "boolean", upstreamLine: 5669, intro: "开启对局倒计时进度条与材质样式" },
  { key: "jindutiaoYangshi", label: "进度条样式", category: "jindutiao", control: "select", upstreamLine: 5674, options: [{ value: "1", label: "手杀进度条" }, { value: "2", label: "十周年PC端进度条" }, { value: "3", label: "十周年客户端进度条" }] },
  { key: "jindutiaotuoguan", label: "托管效果", category: "jindutiao", control: "boolean", upstreamLine: 5684, intro: "读秒归零时提示托管联动" },
  { key: "jindutiaoST", label: "进度条时间间隔", category: "jindutiao", control: "select", upstreamLine: 5689, options: [{ value: "10", label: "10毫秒/次" }, { value: "50", label: "50毫秒/次" }, { value: "100", label: "100毫秒/次" }, { value: "200", label: "200毫秒/次" }, { value: "500", label: "500毫秒/次" }, { value: "800", label: "800毫秒/次" }, { value: "1000", label: "1秒/次" }, { value: "2000", label: "2秒/次" }] },
  { key: "jindutiaoUpdata", label: "玩家进度条刷新", category: "jindutiao", control: "boolean", upstreamLine: 5704 },
  { key: "jindutiaoaiUpdata", label: "人机进度条刷新", category: "jindutiao", control: "boolean", upstreamLine: 5709 },
  { key: "jindutiaoSet", label: "进度条高度", category: "jindutiao", control: "select", upstreamLine: 5714, options: JINDUTIAO_HEIGHT_OPTIONS },
  { key: "jiangdeng", label: "手杀将灯展示", category: "jindutiao", control: "boolean", upstreamLine: 5746, intro: "控制牌桌将灯·统帅/领军/枭雄徽标显示" },

  // prompts: JDTS + GTBB + sayplay (7)
  { key: "JDTS", label: "阶段提示", category: "prompts", control: "boolean", upstreamLine: 5778, intro: "回合内显示对应阶段提示徽标" },
  { key: "JDTSYangshi", label: "阶段提示样式", category: "prompts", control: "select", upstreamLine: 5783, options: [{ value: "1", label: "手杀阶段提示" }, { value: "2", label: "十周年阶段提示" }] },
  { key: "GTBB", label: "狗托播报", category: "prompts", control: "boolean", upstreamLine: 5819, intro: "顶部展示手杀滚动播报条" },
  { key: "GTBBYangshi", label: "播报样式", category: "prompts", control: "select", upstreamLine: 5824, options: [{ value: "on", label: "手杀" }, { value: "off", label: "十周年" }] },
  { key: "GTBBFont", label: "播报字体", category: "prompts", control: "select", upstreamLine: 5833, options: [{ value: "on", label: "手杀楷体" }, { value: "off", label: "十周年圆隶" }] },
  { key: "GTBBTime", label: "播报时间间隔", category: "prompts", control: "select", upstreamLine: 5842, options: [{ value: "30000", label: "0.5min/次" }, { value: "60000", label: "1min/次" }, { value: "120000", label: "2min/次" }, { value: "180000", label: "3min/次" }, { value: "300000", label: "5min/次" }, { value: "random", label: "随机间隔" }] },
  { key: "sayplay", label: "表情＆说话", category: "prompts", control: "select", upstreamLine: 7383, options: [{ value: "off", label: "关闭" }, { value: "old", label: "原版" }, { value: "chg", label: "新版" }] },

  // appearance: yangshi, bg, XPJ, LTAN, xiandingskill_ss, KGMH, qiDongYe, JSAN, ZLLT, charactermenu, xindmenu (11)
  { key: "yangshi", label: "界面布局样式", category: "appearance", control: "select", upstreamLine: 5880, options: [{ value: "on", label: "手杀样式" }, { value: "off", label: "十周年样式" }] },
  { key: "yangshi_bg", label: "菜单壁纸（手杀）", category: "appearance", control: "select", upstreamLine: 5889, options: SHOUSHA_BG_OPTIONS },
  { key: "XPJ", label: "小配件（十周年）", category: "appearance", control: "select", upstreamLine: 5910, options: [{ value: "on", label: "原版" }, { value: "off", label: "新版" }] },
  { key: "LTAN", label: "聊天按钮隐藏", category: "appearance", control: "boolean", upstreamLine: 5919 },
  { key: "xiandingskill_ss", label: "限定技美化（手杀）", category: "appearance", control: "boolean", upstreamLine: 5924 },
  { key: "KGMH", label: "开关美化", category: "appearance", control: "select", upstreamLine: 5954, options: [{ value: "0", label: "关闭" }, { value: "1", label: "手杀" }, { value: "2", label: "十周年" }] },
  { key: "qiDongYe", label: "启动页预设", category: "appearance", control: "select", upstreamLine: 5964, options: [{ value: "on", label: "启动页-动态" }, { value: "othersOn", label: "启动页-大图" }, { value: "othersTwo", label: "启动页-新版" }, { value: "off", label: "关闭" }] },
  { key: "JSAN", label: "结算按钮隐藏", category: "appearance", control: "boolean", upstreamLine: 5976 },
  { key: "ZLLT", label: "资料页露头", category: "appearance", control: "boolean", upstreamLine: 5981 },
  { key: "charactermenu", label: "武将详情美化", category: "appearance", control: "select", upstreamLine: 7393, options: [{ value: "none", label: "本体" }, { value: "old", label: "旧版" }, { value: "new", label: "新版" }] },
  { key: "xindmenu", label: "菜单美化及音效", category: "appearance", control: "select", upstreamLine: 7403, options: [{ value: "shizhounian", label: "十周年" }, { value: "shousha", label: "手杀" }, { value: "off", label: "关闭" }] },

  // select: xindchoose, xindchoose_type, xindsingle, xinddraw, infinitydraw (5)
  { key: "xindchoose", label: "选将美化", category: "select", control: "select", upstreamLine: 7413, options: [{ value: "shizhounian", label: "十周年" }, { value: "shousha", label: "手杀" }, { value: "off", label: "关闭" }] },
  { key: "xindchoose_type", label: "美化版本", category: "select", control: "select", upstreamLine: 7423, options: [{ value: "old", label: "旧版" }, { value: "new", label: "新版" }] },
  { key: "xindsingle", label: "单行选将", category: "select", control: "boolean", upstreamLine: 7433 },
  { key: "xinddraw", label: "手气卡美化", category: "select", upstreamLine: 7438, control: "boolean", intro: "显示手气卡 7 次上限与手牌评分条" },
  { key: "infinitydraw", label: "无限手气卡", category: "select", upstreamLine: 7443, control: "boolean", intro: "需配合手气卡美化开启" },
];

export const SHOUSHA_CONFIG_STORAGE_KEY = "noname_next_shousha_config_v1";

export function createDefaultShoushaConfig(): ShoushaUiConfig {
  return {
    ShouShaThanks: "作者：橙续缘",
    MoGaiMengXiu: "修改：萌新（转型中）",
    KZJS: false,
    GXSM: false,
    MX: "1",
    FL1: true,
    JDTSM: false,
    jindutiao: true,
    jindutiaoYangshi: "1",
    jindutiaotuoguan: true,
    jindutiaoST: "100",
    jindutiaoUpdata: true,
    jindutiaoaiUpdata: true,
    jindutiaoSet: "26",
    jiangdeng: true,
    FL2: true,
    JDTSSM: false,
    JDTS: true,
    JDTSYangshi: "1",
    FL3: true,
    GTBBSM: false,
    GTBB: true,
    GTBBYangshi: "on",
    GTBBFont: "on",
    GTBBTime: "random",
    FL4: true,
    JMYSSM: false,
    yangshi: "on",
    yangshi_bg: "0",
    XPJ: "off",
    LTAN: false,
    xiandingskill_ss: true,
    FL5: true,
    QTMHSM: false,
    KGMH: "1",
    qiDongYe: "othersOn",
    JSAN: false,
    分割线01: true,
    sayplay: "chg",
    charactermenu: "new",
    xindmenu: "shousha",
    xindchoose: "shousha",
    xindchoose_type: "new",
    xindsingle: true,
    xinddraw: true,
    infinitydraw: false,
    分割线02: true,
    ZLLT: true,
  };
}

const SPEC_MAP: Readonly<Record<string, ShoushaConfigSpec>> = Object.fromEntries(
  SHOUSHA_CONFIG_SPECS.map((s) => [s.key, s]),
);

export function validateShoushaConfigPatch(
  raw: Record<string, unknown>,
  opts?: { strict?: boolean },
): { patch: Partial<ShoushaUiConfig>; validKeys: number } {
  const strict = Boolean(opts?.strict);
  const patch: Record<string, unknown> = {};
  let validKeys = 0;

  for (const [key, val] of Object.entries(raw)) {
    const spec = SPEC_MAP[key];
    if (!spec) {
      if (strict) throw new Error(`SHOUSHA_CONFIG_IMPORT_UNKNOWN_KEY:${key}`);
      continue;
    }
    if (spec.control === "info") {
      if (typeof val === "boolean" || typeof val === "string") {
        patch[key] = val;
        validKeys++;
      } else if (strict) {
        throw new Error(`SHOUSHA_CONFIG_IMPORT_INVALID_TYPE:${key}`);
      }
      continue;
    }
    if (spec.control === "boolean") {
      if (typeof val !== "boolean") {
        if (strict) throw new Error(`SHOUSHA_CONFIG_IMPORT_INVALID_BOOL:${key}`);
        continue;
      }
      patch[key] = val;
      validKeys++;
      continue;
    }
    if (spec.control === "select") {
      const allowed = spec.options?.map((o) => o.value) ?? [];
      if (typeof val !== "string" || !allowed.includes(val)) {
        if (strict) throw new Error(`SHOUSHA_CONFIG_IMPORT_INVALID_ENUM:${key}`);
        continue;
      }
      patch[key] = val;
      validKeys++;
    }
  }
  return { patch: patch as Partial<ShoushaUiConfig>, validKeys };
}

export function normalizeShoushaConfig(raw?: Partial<ShoushaUiConfig> | null): ShoushaUiConfig {
  const base = createDefaultShoushaConfig();
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) return base;
  const { patch } = validateShoushaConfigPatch(raw as Record<string, unknown>, { strict: false });
  return { ...base, ...patch };
}

export function parseShoushaConfigImportJson(jsonText: string): {
  patch: Partial<ShoushaUiConfig>;
  importedKeys: number;
} {
  const trimmed = jsonText.trim();
  if (!trimmed || trimmed.length > 32 * 1024) {
    throw new Error("SHOUSHA_CONFIG_IMPORT_SIZE_INVALID");
  }
  if (/<script|javascript:|__proto__|constructor|prototype/i.test(trimmed)) {
    throw new Error("SHOUSHA_CONFIG_IMPORT_UNSAFE_PAYLOAD");
  }
  const parsed = JSON.parse(trimmed) as unknown;
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("SHOUSHA_CONFIG_IMPORT_NOT_OBJECT");
  }
  const res = validateShoushaConfigPatch(parsed as Record<string, unknown>, { strict: true });
  return { patch: res.patch, importedKeys: res.validKeys };
}

export function loadPersistedShoushaConfig(): ShoushaUiConfig {
  try {
    const raw = globalThis.localStorage?.getItem(SHOUSHA_CONFIG_STORAGE_KEY);
    if (!raw) return createDefaultShoushaConfig();
    return normalizeShoushaConfig(JSON.parse(raw) as Partial<ShoushaUiConfig>);
  } catch {
    return createDefaultShoushaConfig();
  }
}

export function persistShoushaConfig(cfg: ShoushaUiConfig): void {
  try {
    globalThis.localStorage?.setItem(SHOUSHA_CONFIG_STORAGE_KEY, JSON.stringify(cfg));
  } catch {
    // Hardened environments keep in-memory state.
  }
}

export function shoushaArenaDatasetAttrs(cfg: ShoushaUiConfig): Record<string, string> {
  return {
    "data-shousha-yangshi": cfg.yangshi,
    "data-shousha-bg": cfg.yangshi_bg,
    "data-shousha-kgmh": cfg.KGMH,
    "data-shousha-jindutiao": cfg.jindutiao ? "on" : "off",
    "data-shousha-jindutiao-style": cfg.jindutiaoYangshi,
    "data-shousha-jdts": cfg.JDTS ? "on" : "off",
    "data-shousha-jdts-style": cfg.JDTSYangshi,
    "data-shousha-gtbb": cfg.GTBB ? "on" : "off",
    "data-shousha-gtbb-style": cfg.GTBBYangshi,
    "data-shousha-gtbb-font": cfg.GTBBFont,
    "data-shousha-jiangdeng": cfg.jiangdeng ? "on" : "off",
    "data-shousha-ltan": cfg.LTAN ? "hide" : "show",
    "data-shousha-xianding": cfg.xiandingskill_ss ? "on" : "off",
    "data-shousha-xpj": cfg.XPJ,
    "data-shousha-sayplay": cfg.sayplay,
    "data-shousha-choose": cfg.xindchoose,
    "data-shousha-choose-type": cfg.xindchoose_type,
    "data-shousha-single-row": cfg.xindsingle ? "on" : "off",
    "data-shousha-qidongye": cfg.qiDongYe,
    "data-shousha-jsan": cfg.JSAN ? "on" : "off",
    "data-shousha-zllt": cfg.ZLLT ? "on" : "off",
  };
}
