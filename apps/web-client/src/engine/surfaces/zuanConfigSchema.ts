/**
 * L25 祖安武将 全部 68 项配置 Schema、分组元数据与裁决 38 严格 JSON 导入校验。
 * 上游对照: `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:160-332, 50300-51435`
 */

export type ZuanConfigGroup = "info_dividers" | "gameplay_modes" | "visual_fx" | "interaction_audio";

export interface ZuanConfigSpec {
  readonly key: string;
  readonly label: string;
  readonly group: ZuanConfigGroup;
  readonly control: "boolean" | "select" | "info";
  readonly defaultValue: boolean | string;
  readonly tooltip: string;
  readonly options?: ReadonlyArray<{ readonly value: string; readonly label: string }>;
}

export const ZUAN_CONFIG_SPECS: ReadonlyArray<ZuanConfigSpec> = [
  // 23 项介绍/彩蛋/分割线/DIY 门禁配置 (先前简化近似)
  { key: "informknow", label: "◈ 长按详情与彩蛋说明", group: "info_dividers", control: "info", defaultValue: "0", tooltip: "查看祖安设置项悬停与点击彩蛋提示" },
  { key: "about1", label: "分割线 I", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "设置区装饰分割线" },
  { key: "about2", label: "祖安武将徽标区", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "祖安武将扩展标题卡片（净室文字徽标）" },
  { key: "blank1", label: "留白间隔 I", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "面板结构间隔" },
  { key: "blank2", label: "留白间隔 II", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "面板结构间隔" },
  { key: "Myfriend", label: "致谢好友卡片", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "世中人等协作作者致谢条目" },
  { key: "about4", label: "分割线 II", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "扩展列表占位分割" },
  { key: "falseedit", label: "编辑扩展防误触门禁", group: "info_dividers", control: "info", defaultValue: "0", tooltip: "锁定长按直接编辑源码入口" },
  { key: "ssr_luoyi", label: "[规则技能项·UI徽章] SSR许褚·裸衣标识", group: "info_dividers", control: "boolean", defaultValue: false, tooltip: "前端仅提供配置项与UI标识；裸衣实际加伤规则由引擎规则层审核（非UI关闭）" },
  { key: "ssr_lieren", label: "[规则技能项·UI徽章] SSR祝融·烈刃标识", group: "info_dividers", control: "boolean", defaultValue: false, tooltip: "前端仅提供配置项与UI标识；烈刃实际拼点规则由引擎规则层审核（非UI关闭）" },
  { key: "free", label: "预加载说明横幅", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "设置搬迁预加载区分割横幅" },
  { key: "aboutZ", label: "分割线 Z", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "害羞彩蛋分割线" },
  { key: "settext", label: "祖安设置总览栏", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "展示全部玩法与娱乐模式简介" },
  { key: "aboutX", label: "分割线 X", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "分段提示线" },
  { key: "linea", label: "关于模式分组标题", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "模式设置区引导" },
  { key: "lineb", label: "关于特效分组标题", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "特效设置区引导" },
  { key: "linec", label: "关于美化分组标题", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "壁纸与千框美化区引导" },
  { key: "lined", label: "关于互动分组标题", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "对话/表情互动区引导" },
  { key: "linef", label: "隐藏辅助分组标题", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "神圣/Boss/隐藏选项区引导" },
  { key: "lineg", label: "音效分组标题", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "纸张与进局音效区引导" },
  { key: "lineh", label: "更多设置分组标题", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "界面比例与 DIY 编辑器区引导" },
  { key: "aboutH", label: "分割线 H", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "底层分段线" },
  { key: "about0", label: "底部彩蛋分割线", group: "info_dividers", control: "info", defaultValue: "1", tooltip: "列表收尾分隔" },

  // 10 项模式与流程选项
  {
    key: "mode",
    label: "游戏模式选择",
    group: "gameplay_modes",
    control: "select",
    defaultValue: "1",
    tooltip: "普通/攻守京城/群雄争霸/讨伐叛军/天下归心/雷鸣山谷/幻海之诗/暗杀行动",
    options: [
      { value: "1", label: "普通模式" },
      { value: "2", label: "攻守京城" },
      { value: "3", label: "群雄争霸" },
      { value: "4", label: "讨伐叛军" },
      { value: "5", label: "天下归心" },
      { value: "6", label: "雷鸣山谷" },
      { value: "7", label: "幻海之诗" },
      { value: "8", label: "暗杀行动" },
    ],
  },
  {
    key: "fun",
    label: "娱乐玩法选择",
    group: "gameplay_modes",
    control: "select",
    defaultValue: "1",
    tooltip: "大将英魂/群侠论剑/桃花落尽/同将之争/暗夜鬼城/元气系统",
    options: [
      { value: "1", label: "关闭效果" },
      { value: "2", label: "大将英魂" },
      { value: "3", label: "群侠论剑" },
      { value: "4", label: "桃花落尽" },
      { value: "5", label: "同将之争" },
      { value: "6", label: "暗夜鬼城" },
      { value: "7", label: "元气系统" },
    ],
  },
  { key: "goodluck", label: "天选之子模式", group: "gameplay_modes", control: "boolean", defaultValue: false, tooltip: "显示幸运星高光徽章" },
  {
    key: "washhandcard",
    label: "手牌整理",
    group: "gameplay_modes",
    control: "select",
    defaultValue: "close",
    tooltip: "关闭/自动/手动分类整理",
    options: [{ value: "close", label: "关闭" }, { value: "auto", label: "自动" }, { value: "user", label: "手动" }],
  },
  {
    key: "levelstrong",
    label: "等级奖励",
    group: "gameplay_modes",
    control: "select",
    defaultValue: "new",
    tooltip: "关闭/旧规则/新规则",
    options: [{ value: "off", label: "关闭" }, { value: "old", label: "旧规则" }, { value: "new", label: "新规则" }],
  },
  {
    key: "upspeed",
    label: "等级提升速度",
    group: "gameplay_modes",
    control: "select",
    defaultValue: "0",
    tooltip: "经验倍率",
    options: [{ value: "0", label: "关闭" }, { value: "5", label: "50%" }, { value: "10", label: "100%" }, { value: "20", label: "200%" }],
  },
  {
    key: "relife",
    label: "复活时长设置",
    group: "gameplay_modes",
    control: "select",
    defaultValue: "100",
    tooltip: "0=固定3轮 / 50% / 100% / 150%",
    options: [{ value: "0", label: "固定3轮" }, { value: "50", label: "50%" }, { value: "100", label: "100%" }, { value: "150", label: "150%" }],
  },
  { key: "balance", label: "游戏模式平衡", group: "gameplay_modes", control: "boolean", defaultValue: true, tooltip: "启用阵营平衡参数标识" },
  { key: "newover", label: "新版胜负结算", group: "gameplay_modes", control: "boolean", defaultValue: true, tooltip: "函数直接结算胜负提示" },
  { key: "falsedelete", label: "删除扩展防误触锁", group: "gameplay_modes", control: "info", defaultValue: "0", tooltip: "拦截长按快捷删除操作" },

  // 20 项视觉、判定全屏与指示线配置
  { key: "bingletexiao", label: "兵乐闪电判定特效", group: "visual_fx", control: "boolean", defaultValue: true, tooltip: "触发 _bingleBigScreen 判定视觉横幅" },
  { key: "texiao", label: "祖安技能大特效", group: "visual_fx", control: "boolean", defaultValue: true, tooltip: "触发技能视觉特效光晕" },
  { key: "zuancoveropacity", label: "遮罩透明度 (%)", group: "visual_fx", control: "select", defaultValue: "65", options: [{ value: "40", label: "40%" }, { value: "65", label: "65%" }, { value: "90", label: "90%" }], tooltip: "特效叠层透明度" },
  { key: "huihe", label: "回合特效样式", group: "visual_fx", control: "boolean", defaultValue: true, tooltip: "启用回合开始光效" },
  { key: "huiheopa", label: "回合光效透明度", group: "visual_fx", control: "select", defaultValue: "80", options: [{ value: "50", label: "50%" }, { value: "80", label: "80%" }, { value: "100", label: "100%" }], tooltip: "回合标徽透明度" },
  { key: "newhuihe", label: "新版回合流光", group: "visual_fx", control: "boolean", defaultValue: true, tooltip: "新版回合流光边栏" },
  { key: "imgtime", label: "图像持续帧(ms)", group: "visual_fx", control: "select", defaultValue: "600", options: [{ value: "400", label: "400ms" }, { value: "600", label: "600ms" }, { value: "900", label: "900ms" }], tooltip: "祖安弹窗停留时长" },
  { key: "imgfps", label: "帧率节流档位", group: "visual_fx", control: "select", defaultValue: "60", options: [{ value: "30", label: "30 FPS" }, { value: "60", label: "60 FPS" }], tooltip: "帧步进目标" },
  { key: "fujiaset", label: "附加特效集", group: "visual_fx", control: "boolean", defaultValue: true, tooltip: "开启附加判定指示" },
  { key: "newui", label: "祖安新版界面", group: "visual_fx", control: "boolean", defaultValue: true, tooltip: "启用祖安现代深邃质感" },
  { key: "helasisy", label: "关闭开局闪屏动画", group: "visual_fx", control: "boolean", defaultValue: true, tooltip: "联动 _helasisyskill 跳过开局遮罩" },
  { key: "perspective", label: "景深透视场", group: "visual_fx", control: "boolean", defaultValue: false, tooltip: "启用轻微牌桌透视阴影" },
  { key: "bizhi", label: "启用动态壁纸", group: "visual_fx", control: "boolean", defaultValue: false, tooltip: "联动 _newBackgroundUse 切换牌桌壁纸" },
  { key: "wallpaper", label: "内置壁纸预设", group: "visual_fx", control: "select", defaultValue: "shihai_default", options: [{ value: "shihai_default", label: "识海默认" }, { value: "seadream_blue", label: "幻海深蓝" }, { value: "thunder_slate", label: "雷谷苍岩" }], tooltip: "选择净室配色背景" },
  { key: "paperchoose", label: "纸张材质预设", group: "visual_fx", control: "select", defaultValue: "classic", options: [{ value: "classic", label: "经典羊皮" }, { value: "ink", label: "墨韵" }], tooltip: "卡牌与弹窗底纹" },
  { key: "exwallpaper", label: "扩展壁纸兼容", group: "visual_fx", control: "boolean", defaultValue: false, tooltip: "允许外部主题叠加" },
  { key: "qiankuang", label: "千框美化边框", group: "visual_fx", control: "boolean", defaultValue: true, tooltip: "千框渐变金边美化" },
  { key: "backgroundttop", label: "背景顶置层级", group: "visual_fx", control: "boolean", defaultValue: false, tooltip: "前置壁纸氛围层" },
  { key: "zhishixian", label: "指示线样式", group: "visual_fx", control: "select", defaultValue: "guanjie", options: [{ value: "guanjie", label: "官阶流金" }, { value: "classic", label: "经典赤金" }, { value: "laser", label: "霓虹激光" }], tooltip: "控制 _zuanattacklinecheck 指示线样式" },
  { key: "zhishiopi", label: "指示线透明度", group: "visual_fx", control: "select", defaultValue: "100", options: [{ value: "50", label: "50%" }, { value: "80", label: "80%" }, { value: "100", label: "100%" }], tooltip: "控制目标指示线不透明度" },

  // 15 项互动对话、音效与 DIY 门禁配置
  { key: "zuantype", label: "祖安对话版本", group: "interaction_audio", control: "select", defaultValue: "new_chg", options: [{ value: "old", label: "原版本" }, { value: "new", label: "新版本" }, { value: "new_chg", label: "新版优化" }], tooltip: "选择台词词库风味" },
  { key: "ransay", label: "成为龙王聊天几率", group: "interaction_audio", control: "select", defaultValue: "20", options: [{ value: "0", label: "退群" }, { value: "20", label: "20%" }, { value: "100", label: "100%" }], tooltip: "控制回合/伤害触发弹幕频率" },
  { key: "levelsay", label: "升级台词播报", group: "interaction_audio", control: "boolean", defaultValue: true, tooltip: "升级时展示对话气泡" },
  { key: "girlspeak", label: "女将语气包", group: "interaction_audio", control: "boolean", defaultValue: false, tooltip: "女将互动语气微调" },
  { key: "zuanemotion", label: "祖安表情回应", group: "interaction_audio", control: "boolean", defaultValue: true, tooltip: "支持战局事件气泡" },
  { key: "shenst", label: "神圣状态光晕", group: "interaction_audio", control: "boolean", defaultValue: false, tooltip: "特殊姿态高亮框" },
  { key: "showboss", label: "展示 Boss 落幕横幅", group: "interaction_audio", control: "boolean", defaultValue: true, tooltip: "联动 _zuanboss 显示首领败退旗帜" },
  { key: "hidessr", label: "隐藏 SSR 将卡", group: "interaction_audio", control: "boolean", defaultValue: false, tooltip: "过滤 SSR 候选标签" },
  { key: "delkey", label: "屏蔽快捷热键", group: "interaction_audio", control: "boolean", defaultValue: false, tooltip: "禁用冗余调试按键" },
  { key: "hideide", label: "隐藏未知身份槽", group: "interaction_audio", control: "boolean", defaultValue: false, tooltip: "暗身份席位保持简洁" },
  { key: "audio_draw", label: "屏蔽摸/失牌纸张音效", group: "interaction_audio", control: "boolean", defaultValue: false, tooltip: "控制 _extragainskill/_extraloseskill 静音门禁" },
  { key: "audio_start", label: "进入对局音效预设", group: "interaction_audio", control: "select", defaultValue: "dump", options: [{ value: "none", label: "关闭" }, { value: "ring", label: "门铃" }, { value: "wind", label: "风铃" }, { value: "dump", label: "战鼓I" }, { value: "fault", label: "战鼓II" }], tooltip: "开局音效标识" },
  { key: "isPC", label: "界面比例模式", group: "interaction_audio", control: "select", defaultValue: "1", options: [{ value: "1", label: "桌面标准 (1.0)" }, { value: "2", label: "大屏紧凑 (0.9)" }], tooltip: "PC/大屏比例提示" },
  { key: "windowHigh", label: "窗口自适应高度", group: "interaction_audio", control: "select", defaultValue: "1", options: [{ value: "1", label: "标准" }, { value: "7", label: "1090×570" }], tooltip: "兼容窗口尺寸预设" },
  { key: "text", label: "离线自定义测试编辑 (DIY)", group: "interaction_audio", control: "boolean", defaultValue: false, tooltip: "仅离线房间可解锁（裁决 37 联机门禁）" },
];

export const ZUAN_CONFIG_KEYS: ReadonlyArray<string> = ZUAN_CONFIG_SPECS.map((s) => s.key);

export type ZuanUiConfigMap = Record<string, boolean | string>;

export function createDefaultZuanConfig(): ZuanUiConfigMap {
  return Object.fromEntries(ZUAN_CONFIG_SPECS.map((s) => [s.key, s.defaultValue]));
}

const ZUAN_SPEC_MAP: Readonly<Record<string, ZuanConfigSpec>> = Object.fromEntries(
  ZUAN_CONFIG_SPECS.map((s) => [s.key, s]),
);

export function validateZuanConfigPatch(
  raw: Record<string, unknown>,
  opts?: { strict?: boolean },
): { patch: ZuanUiConfigMap; validKeys: number } {
  const strict = Boolean(opts?.strict);
  const patch: ZuanUiConfigMap = {};
  let validKeys = 0;

  for (const [key, val] of Object.entries(raw)) {
    const spec = ZUAN_SPEC_MAP[key];
    if (!spec) {
      if (strict) throw new Error(`ZUAN_CONFIG_UNKNOWN_KEY:${key}`);
      continue;
    }
    if (spec.control === "boolean") {
      if (typeof val !== "boolean") {
        if (strict) throw new Error(`ZUAN_CONFIG_INVALID_BOOL:${key}`);
        continue;
      }
      patch[key] = val;
      validKeys++;
      continue;
    }
    if (spec.control === "select") {
      const allowed = spec.options?.map((o) => o.value) ?? [];
      if (typeof val !== "string" || !allowed.includes(val)) {
        if (strict) throw new Error(`ZUAN_CONFIG_INVALID_ENUM:${key}`);
        continue;
      }
      patch[key] = val;
      validKeys++;
      continue;
    }
    if (typeof val === "string" || typeof val === "boolean") {
      patch[key] = val;
      validKeys++;
    } else if (strict) {
      throw new Error(`ZUAN_CONFIG_INVALID_TYPE:${key}`);
    }
  }
  return { patch, validKeys };
}

export function parseZuanConfigImportJson(
  jsonText: string,
): { patch: ZuanUiConfigMap; importedKeys: number } {
  const parsed: unknown = JSON.parse(jsonText);
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("ZUAN_CONFIG_IMPORT_MUST_BE_OBJECT");
  }
  const res = validateZuanConfigPatch(parsed as Record<string, unknown>, { strict: true });
  return { patch: res.patch, importedKeys: res.validKeys };
}
