/**
 * L09 千幻聆音 全部 82 项配置 Schema、本地持久化与裁决 38 严格 JSON 导入校验。
 * 上游对照: `archive/upstream/noname-liuli/noname/extension/千幻聆音/extension.js:14986-15942`
 */

export type QhlyConfigCategory =
  | "ui"
  | "window"
  | "decade"
  | "audio"
  | "theme_tune"
  | "actions";

export interface QhlyConfigOption {
  readonly value: string;
  readonly label: string;
}

export interface QhlyConfigSpec {
  readonly key: keyof QhlyUiConfig;
  readonly label: string;
  readonly category: QhlyConfigCategory;
  readonly control: "boolean" | "select" | "info" | "action";
  readonly upstreamLine: number;
  readonly intro?: string;
  readonly options?: ReadonlyArray<QhlyConfigOption>;
}

export interface QhlyUiConfig {
  qhly_newui: boolean;
  qhly_uishezhi: boolean;
  qhly_currentViewSkin: string;
  qhly_layoutFitX: boolean;
  qhly_layoutFitY: boolean;
  qhly_vMiddle: boolean;
  qhly_fontsize1: "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9";
  qhly_gongnengshezhi: boolean;
  qhly_replaceCharacterCard2: "nonereplace" | "nonereplace2" | "info" | "window";
  qhly_nolihuiOrigin: boolean;
  qhly_smallwiningame: boolean;
  qhly_smallwindowstyle: "dragon" | "common" | "decade" | "shousha";
  qhly_dragonsize: "0.45" | "0.55" | "0.60" | "0.80" | "1.00" | "1.20" | "1.50" | "1.65" | "1.80";
  qhly_forbidExtPage: boolean;
  qhly_dragonlocation: "head" | "center" | "drag";
  qhly_smallwinclosewhenchange: boolean;
  qhly_recordWin: boolean;
  qhly_randskin: boolean;
  qhly_randskin_me: boolean;
  qhly_extcompat: boolean;
  qhly_lutou: boolean;
  qhly_audioPlus: boolean;
  qhly_lutouType: "decade" | "shousha";
  lutouFixx: "off" | "all";
  lvjingFixx: boolean;
  qhly_skinButton: boolean;
  qhly_showrarity: boolean;
  qhly_dragButton: boolean;
  qhly_dragButtonPosition: "no" | "yes";
  qhly_notbb: "none" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10";
  qhly_notbb_range: "skill" | "character" | "all";
  qhly_originSkinPath: "extension/千幻聆音/sanguoskin/" | "extension/千幻聆音/sanguolutouskin/" | "image/skin/";
  qhly_extSkinPath: "default" | "extension/千幻聆音/sanguoskin/" | "extension/千幻聆音/sanguolutouskin/";
  qhly_autoChangeSkin: "close" | "10" | "30" | "60" | "120" | "600" | "random";
  qhly_listdefaultpage: "introduce" | "skill" | "skin" | "config";
  qhly_doubledefaultpage: "introduce" | "skill" | "skin" | "config";
  qhly_guozhan: boolean;
  qhly_skinconfig: boolean;
  qhly_editmode: boolean;
  qhly_skillingame: boolean;
  qhly_keymarkopen: boolean;
  qhly_chooseButtonOrigin: boolean;
  qhly_chooseButtonOrigin_guozhan: boolean;
  qhly_mvp: boolean;
  qhly_decadeConfig: boolean;
  qhly_decadeCloseDynamic: boolean;
  qhly_playerwindow: boolean;
  qhly_formatDS: boolean;
  qhly_editDynamic: boolean;
  qhly_noSkin: "noSkin" | "origin";
  qhly_dom2image: boolean;
  qhly_decadeDynamic: "none" | "three" | "always";
  qhly_decadeChangeEffect: boolean;
  qhly_guozhanDS: boolean;
  qhly_decadeDengjie: "one" | "two" | "three" | "four" | "five" | "auto";
  qhly_decadeAuto: "10" | "30" | "60" | "120" | "600";
  qhly_shoushaTexiao: boolean;
  qhly_ignoreClips: boolean;
  qhly_yinxiaoshezhi: boolean;
  qhly_closeVoice: boolean;
  qhly_currentMusic: string;
  qhly_enableCharacterMusic: boolean;
  qhly_modemusicconfig: string;
  qhly_shuimolingyin: boolean;
  qhly_titlereplace: "title" | "skin" | "pkg";
  qhly_hanggaoxiufu: string;
  qhly_hanggaoxiufu2: string;
  qhly_shilizihao: string;
  qhly_lihuiSupport: boolean;
  qhly_hideShuimoCover: boolean;
  qhly_lolconfig: boolean;
  qhly_lolhanggaoxiufu: string;
  qhly_lolhanggaoxiufu2: string;
  qhly_lolshilizihao: string;
  qhly_jianrongxing: boolean;
  qhly_funcLoadInPrecontent: boolean;
  qhly_qitashezhi: boolean;
  qhly_clear: boolean;
  skinAudioList: string;
  audioReplace: string;
  qhly_restore: boolean;
  qhly_plugin: boolean;
}

const PCT_250_400: ReadonlyArray<QhlyConfigOption> = [
  "250", "260", "270", "280", "290", "300", "310", "320",
  "330", "340", "350", "360", "370", "380", "390", "400",
].map((v) => ({ value: v, label: `${v}%` }));

const SIZE_50_100: ReadonlyArray<QhlyConfigOption> = [
  "50", "55", "60", "65", "70", "75", "80", "85", "90", "95", "100",
].map((v) => ({ value: v, label: v }));

/** 全部 82 项 `L09:configs` 定义（1-to-1 对应 `g0/inventory_status.json`） */
export const QHLY_CONFIG_SPECS: ReadonlyArray<QhlyConfigSpec> = [
  // UI (7)
  { key: "qhly_newui", label: "新UI模式", category: "ui", control: "boolean", upstreamLine: 14986, intro: "使用新版千幻聆音UI容器" },
  { key: "qhly_uishezhi", label: "UI设置分组栏", category: "ui", control: "info", upstreamLine: 14995 },
  { key: "qhly_currentViewSkin", label: "UI套装样式", category: "ui", control: "select", upstreamLine: 14999, options: [
    { value: "shousha", label: "手杀臻享" },
    { value: "decade", label: "十周年典藏" },
    { value: "shuimo", label: "水墨龙吟" },
    { value: "lolbig", label: "海克斯科技" },
    { value: "sanguo", label: "三国古卷一" },
    { value: "sanguo2", label: "三国古卷二" },
    { value: "rq", label: "燃情金戈" },
    { value: "sksn", label: "时空少女" },
    { value: "wz", label: "王者圣域" },
    { value: "newui", label: "新版默认" },
  ] },
  { key: "qhly_layoutFitX", label: "横向拉伸适应", category: "ui", control: "boolean", upstreamLine: 15025 },
  { key: "qhly_layoutFitY", label: "纵向拉伸适应", category: "ui", control: "boolean", upstreamLine: 15034 },
  { key: "qhly_vMiddle", label: "水墨/海克斯纵向居中", category: "ui", control: "boolean", upstreamLine: 15043 },
  { key: "qhly_fontsize1", label: "正文字号", category: "ui", control: "select", upstreamLine: 15052, options: [
    { value: "1", label: "很微小" }, { value: "2", label: "微小" }, { value: "3", label: "较小" },
    { value: "4", label: "小" }, { value: "5", label: "中" }, { value: "6", label: "大" },
    { value: "7", label: "较大" }, { value: "8", label: "巨大" }, { value: "9", label: "超级大" },
  ] },

  // Window & Portraits (31)
  { key: "qhly_gongnengshezhi", label: "功能设置分组栏", category: "window", control: "info", upstreamLine: 15072 },
  { key: "qhly_replaceCharacterCard2", label: "替换默认资料卡", category: "window", control: "select", upstreamLine: 15076, options: [
    { value: "nonereplace", label: "不替换" },
    { value: "nonereplace2", label: "系统卡不显示换肤" },
    { value: "info", label: "千幻资料卡" },
    { value: "window", label: "千幻皮肤小窗" },
  ] },
  { key: "qhly_nolihuiOrigin", label: "无立绘显示原皮", category: "window", control: "boolean", upstreamLine: 15091 },
  { key: "qhly_smallwiningame", label: "小窗口换皮肤", category: "window", control: "boolean", upstreamLine: 15100 },
  { key: "qhly_smallwindowstyle", label: "小窗口样式", category: "window", control: "select", upstreamLine: 15109, options: [
    { value: "dragon", label: "龙头" }, { value: "common", label: "经典" },
    { value: "decade", label: "十周年" }, { value: "shousha", label: "手杀" },
  ] },
  { key: "qhly_dragonsize", label: "龙头小窗口大小", category: "window", control: "select", upstreamLine: 15124, options: [
    { value: "0.45", label: "超级小" }, { value: "0.55", label: "特小" }, { value: "0.60", label: "小" },
    { value: "0.80", label: "较小" }, { value: "1.00", label: "适中" }, { value: "1.20", label: "较大" },
    { value: "1.50", label: "大" }, { value: "1.65", label: "特大" }, { value: "1.80", label: "超级大" },
  ] },
  { key: "qhly_forbidExtPage", label: "禁用附加页功能", category: "window", control: "boolean", upstreamLine: 15144 },
  { key: "qhly_dragonlocation", label: "龙头小窗口位置", category: "window", control: "select", upstreamLine: 15153, options: [
    { value: "head", label: "头像上" }, { value: "center", label: "正中央" }, { value: "drag", label: "可拖曳" },
  ] },
  { key: "qhly_smallwinclosewhenchange", label: "自动关闭小窗口", category: "window", control: "boolean", upstreamLine: 15167 },
  { key: "qhly_recordWin", label: "展示战绩", category: "window", control: "boolean", upstreamLine: 15177 },
  { key: "qhly_randskin", label: "随机皮肤", category: "window", control: "boolean", upstreamLine: 15186 },
  { key: "qhly_randskin_me", label: "随机皮肤不随自己", category: "window", control: "boolean", upstreamLine: 15195 },
  { key: "qhly_extcompat", label: "扩展配音兼容", category: "window", control: "boolean", upstreamLine: 15205 },
  { key: "qhly_lutou", label: "适配露头", category: "window", control: "boolean", upstreamLine: 15214 },
  { key: "qhly_lutouType", label: "露头模式", category: "window", control: "select", upstreamLine: 15234, options: [
    { value: "decade", label: "十周年露头" }, { value: "shousha", label: "手杀露头" },
  ] },
  { key: "lutouFixx", label: "露头修复", category: "window", control: "select", upstreamLine: 15247, options: [
    { value: "off", label: "关闭效果" }, { value: "all", label: "启用效果" },
  ] },
  { key: "lvjingFixx", label: "磨皮滤镜", category: "window", control: "boolean", upstreamLine: 15257 },
  { key: "qhly_skinButton", label: "头像显示换肤按钮", category: "window", control: "boolean", upstreamLine: 15262 },
  { key: "qhly_showrarity", label: "显示武将等阶", category: "window", control: "boolean", upstreamLine: 15271 },
  { key: "qhly_dragButton", label: "换肤按钮可拖曳", category: "window", control: "boolean", upstreamLine: 15280 },
  { key: "qhly_dragButtonPosition", label: "换肤按钮拖曳同步", category: "window", control: "select", upstreamLine: 15289, options: [
    { value: "no", label: "不同步" }, { value: "yes", label: "同步" },
  ] },
  { key: "qhly_originSkinPath", label: "本体皮肤目录键", category: "window", control: "select", upstreamLine: 15337, options: [
    { value: "extension/千幻聆音/sanguoskin/", label: "千幻聆音目录" },
    { value: "extension/千幻聆音/sanguolutouskin/", label: "千幻露头目录" },
    { value: "image/skin/", label: "本体目录" },
  ] },
  { key: "qhly_extSkinPath", label: "扩展皮肤目录键", category: "window", control: "select", upstreamLine: 15365, options: [
    { value: "default", label: "扩展skin文件夹" },
    { value: "extension/千幻聆音/sanguoskin/", label: "千幻聆音目录" },
    { value: "extension/千幻聆音/sanguolutouskin/", label: "千幻露头目录" },
  ] },
  { key: "qhly_autoChangeSkin", label: "自动切换皮肤", category: "window", control: "select", upstreamLine: 15393, options: [
    { value: "close", label: "关闭" }, { value: "10", label: "每10秒" }, { value: "30", label: "每半分钟" },
    { value: "60", label: "每1分钟" }, { value: "120", label: "每2分钟" }, { value: "600", label: "每10分钟" }, { value: "random", label: "随机间隔" },
  ] },
  { key: "qhly_listdefaultpage", label: "列表进入默认页面", category: "window", control: "select", upstreamLine: 15428, options: [
    { value: "introduce", label: "人物简介" }, { value: "skill", label: "技能描述" }, { value: "skin", label: "皮肤信息" }, { value: "config", label: "相关配置" },
  ] },
  { key: "qhly_doubledefaultpage", label: "双击默认页面", category: "window", control: "select", upstreamLine: 15443, options: [
    { value: "introduce", label: "人物简介" }, { value: "skill", label: "技能描述" }, { value: "skin", label: "皮肤信息" }, { value: "config", label: "相关配置" },
  ] },
  { key: "qhly_guozhan", label: "国战皮肤开关", category: "window", control: "boolean", upstreamLine: 15458 },
  { key: "qhly_skinconfig", label: "皮肤高级配置", category: "window", control: "boolean", upstreamLine: 15467 },
  { key: "qhly_editmode", label: "台词编辑模式", category: "window", control: "boolean", upstreamLine: 15476 },
  { key: "qhly_skillingame", label: "显示对局技能", category: "window", control: "boolean", upstreamLine: 15485 },
  { key: "qhly_keymarkopen", label: "技能关键字高亮", category: "window", control: "boolean", upstreamLine: 15494 },
  { key: "qhly_chooseButtonOrigin", label: "选将界面显示原皮", category: "window", control: "boolean", upstreamLine: 15520 },
  { key: "qhly_chooseButtonOrigin_guozhan", label: "国战选将显示原皮", category: "window", control: "boolean", upstreamLine: 15529 },
  { key: "qhly_mvp", label: "播放MVP胜利语音", category: "window", control: "boolean", upstreamLine: 15534 },

  // Decade / Shousha Integration (15)
  { key: "qhly_decadeConfig", label: "十周年及手杀样式设置栏", category: "decade", control: "info", upstreamLine: 15538 },
  { key: "qhly_decadeCloseDynamic", label: "关闭所有动皮效果", category: "decade", control: "boolean", upstreamLine: 15542 },
  { key: "qhly_playerwindow", label: "单击武将呼出菜单", category: "decade", control: "boolean", upstreamLine: 15564 },
  { key: "qhly_formatDS", label: "自动调整动皮参数", category: "decade", control: "boolean", upstreamLine: 15568 },
  { key: "qhly_editDynamic", label: "调节待机及大页面", category: "decade", control: "boolean", upstreamLine: 15572 },
  { key: "qhly_noSkin", label: "无静皮时显示", category: "decade", control: "select", upstreamLine: 15577, options: [
    { value: "noSkin", label: "无相应静皮资源" }, { value: "origin", label: "原皮" },
  ] },
  { key: "qhly_dom2image", label: "显示生成静皮按钮", category: "decade", control: "boolean", upstreamLine: 15590 },
  { key: "qhly_decadeDynamic", label: "小窗动皮关闭", category: "decade", control: "select", upstreamLine: 15595, options: [
    { value: "none", label: "不关闭" }, { value: "three", label: "多于3个时关闭" }, { value: "always", label: "保持关闭" },
  ] },
  { key: "qhly_decadeChangeEffect", label: "换肤特效", category: "decade", control: "boolean", upstreamLine: 15608 },
  { key: "qhly_guozhanDS", label: "国战动皮适配", category: "decade", control: "boolean", upstreamLine: 15612 },
  { key: "qhly_decadeDengjie", label: "小窗人物等阶", category: "decade", control: "select", upstreamLine: 15616, options: [
    { value: "one", label: "一阶" }, { value: "two", label: "二阶" }, { value: "three", label: "三阶" },
    { value: "four", label: "四阶" }, { value: "five", label: "五阶" }, { value: "auto", label: "跟随角色评级" },
  ] },
  { key: "qhly_decadeAuto", label: "小窗自动换肤间隔", category: "decade", control: "select", upstreamLine: 15628, options: [
    { value: "10", label: "每10秒" }, { value: "30", label: "每半分钟" }, { value: "60", label: "每1分钟" },
    { value: "120", label: "每2分钟" }, { value: "600", label: "每10分钟" },
  ] },
  { key: "qhly_shoushaTexiao", label: "大页面出框特效", category: "decade", control: "boolean", upstreamLine: 15640 },
  { key: "qhly_ignoreClips", label: "忽略clipSlots", category: "decade", control: "boolean", upstreamLine: 15644 },

  // Audio & BGM (8)
  { key: "qhly_yinxiaoshezhi", label: "音效设置分组栏", category: "audio", control: "info", upstreamLine: 15648 },
  { key: "qhly_audioPlus", label: "音频功能增强", category: "audio", control: "boolean", upstreamLine: 15225 },
  { key: "qhly_closeVoice", label: "关闭点击音效", category: "audio", control: "boolean", upstreamLine: 15652 },
  { key: "qhly_currentMusic", label: "设置BGM", category: "audio", control: "select", upstreamLine: 15661, options: [
    { value: "system", label: "跟随系统" }, { value: "shuimo", label: "水墨雅韵(本地合成)" }, { value: "sanguo", label: "古卷金戈(本地合成)" },
  ] },
  { key: "qhly_enableCharacterMusic", label: "角色BGM", category: "audio", control: "boolean", upstreamLine: 15673 },
  { key: "qhly_modemusicconfig", label: "模式BGM", category: "audio", control: "select", upstreamLine: 15682, options: [
    { value: "system", label: "不特别配置" }, { value: "identity", label: "身份局专属主题" }, { value: "doudizhu", label: "斗地主专属主题" },
  ] },
  { key: "qhly_notbb", label: "防啰嗦冷却时间", category: "audio", control: "select", upstreamLine: 15302, options: [
    { value: "none", label: "关闭" }, { value: "2", label: "2秒" }, { value: "3", label: "3秒" }, { value: "4", label: "4秒" },
    { value: "5", label: "5秒" }, { value: "6", label: "6秒" }, { value: "7", label: "7秒" }, { value: "8", label: "8秒" },
    { value: "9", label: "9秒" }, { value: "10", label: "10秒" },
  ] },
  { key: "qhly_notbb_range", label: "防啰嗦范围", category: "audio", control: "select", upstreamLine: 15323, options: [
    { value: "skill", label: "相同技能" }, { value: "character", label: "相同角色" }, { value: "all", label: "所有角色" },
  ] },

  // Shuimo & LOL Hextech Theme Tuning (12)
  { key: "qhly_shuimolingyin", label: "水墨龙吟设置栏", category: "theme_tune", control: "info", upstreamLine: 15693 },
  { key: "qhly_titlereplace", label: "水墨武将旁小字内容", category: "theme_tune", control: "select", upstreamLine: 15697, options: [
    { value: "title", label: "武将标题" }, { value: "skin", label: "皮肤名" }, { value: "pkg", label: "武将包名" },
  ] },
  { key: "qhly_hanggaoxiufu", label: "水墨技能名行高", category: "theme_tune", control: "select", upstreamLine: 15711, options: PCT_250_400 },
  { key: "qhly_hanggaoxiufu2", label: "水墨按钮行高", category: "theme_tune", control: "select", upstreamLine: 15738, options: PCT_250_400 },
  { key: "qhly_shilizihao", label: "水墨势力字号", category: "theme_tune", control: "select", upstreamLine: 15765, options: SIZE_50_100 },
  { key: "qhly_lihuiSupport", label: "水墨显示立绘", category: "theme_tune", control: "boolean", upstreamLine: 15787 },
  { key: "qhly_hideShuimoCover", label: "水墨隐藏墨迹遮罩", category: "theme_tune", control: "boolean", upstreamLine: 15796 },
  { key: "qhly_lolconfig", label: "海克斯科技设置栏", category: "theme_tune", control: "info", upstreamLine: 15805 },
  { key: "qhly_lolhanggaoxiufu", label: "海克斯技能名行高", category: "theme_tune", control: "select", upstreamLine: 15809, options: PCT_250_400 },
  { key: "qhly_lolhanggaoxiufu2", label: "海克斯按钮行高", category: "theme_tune", control: "select", upstreamLine: 15836, options: PCT_250_400 },
  { key: "qhly_lolshilizihao", label: "海克斯势力字号", category: "theme_tune", control: "select", upstreamLine: 15863, options: SIZE_50_100 },

  // Compatibility, Actions & Local Import Buckets (9)
  { key: "qhly_jianrongxing", label: "兼容性相关设置栏", category: "actions", control: "info", upstreamLine: 15885 },
  { key: "qhly_funcLoadInPrecontent", label: "预处理加载(如真兼容)", category: "actions", control: "boolean", upstreamLine: 15889 },
  { key: "qhly_qitashezhi", label: "其他操作栏", category: "actions", control: "info", upstreamLine: 15898 },
  { key: "qhly_clear", label: "清空本地皮肤配置", category: "actions", control: "action", upstreamLine: 15902 },
  { key: "skinAudioList", label: "本地自定义语音映射JSON", category: "actions", control: "info", upstreamLine: 15910, intro: "玩家自有皮肤语音映射导入槽（不含商业音频）" },
  { key: "audioReplace", label: "本地自定义台词替换JSON", category: "actions", control: "info", upstreamLine: 15913, intro: "玩家自有技能台词替换槽" },
  { key: "qhly_restore", label: "恢复官方皮肤设置", category: "actions", control: "action", upstreamLine: 15921 },
  { key: "qhly_plugin", label: "千幻扩展插件管理入口", category: "actions", control: "action", upstreamLine: 15933 },
];

export const QHLY_CONFIG_STORAGE_KEY = "noname_next_qhly_config_v1";

export function createDefaultQhlyConfig(): QhlyUiConfig {
  return {
    qhly_newui: true,
    qhly_uishezhi: true,
    qhly_currentViewSkin: "shousha",
    qhly_layoutFitX: false,
    qhly_layoutFitY: false,
    qhly_vMiddle: true,
    qhly_fontsize1: "5",
    qhly_gongnengshezhi: true,
    qhly_replaceCharacterCard2: "info",
    qhly_nolihuiOrigin: false,
    qhly_smallwiningame: false,
    qhly_smallwindowstyle: "shousha",
    qhly_dragonsize: "1.00",
    qhly_forbidExtPage: false,
    qhly_dragonlocation: "center",
    qhly_smallwinclosewhenchange: true,
    qhly_recordWin: false,
    qhly_randskin: true,
    qhly_randskin_me: true,
    qhly_extcompat: true,
    qhly_lutou: false,
    qhly_audioPlus: false,
    qhly_lutouType: "shousha",
    lutouFixx: "all",
    lvjingFixx: false,
    qhly_skinButton: false,
    qhly_showrarity: false,
    qhly_dragButton: false,
    qhly_dragButtonPosition: "yes",
    qhly_notbb: "none",
    qhly_notbb_range: "skill",
    qhly_originSkinPath: "extension/千幻聆音/sanguoskin/",
    qhly_extSkinPath: "default",
    qhly_autoChangeSkin: "random",
    qhly_listdefaultpage: "introduce",
    qhly_doubledefaultpage: "skill",
    qhly_guozhan: true,
    qhly_skinconfig: false,
    qhly_editmode: false,
    qhly_skillingame: false,
    qhly_keymarkopen: false,
    qhly_chooseButtonOrigin: false,
    qhly_chooseButtonOrigin_guozhan: true,
    qhly_mvp: false,
    qhly_decadeConfig: true,
    qhly_decadeCloseDynamic: false,
    qhly_playerwindow: true,
    qhly_formatDS: true,
    qhly_editDynamic: true,
    qhly_noSkin: "noSkin",
    qhly_dom2image: true,
    qhly_decadeDynamic: "three",
    qhly_decadeChangeEffect: true,
    qhly_guozhanDS: true,
    qhly_decadeDengjie: "auto",
    qhly_decadeAuto: "30",
    qhly_shoushaTexiao: true,
    qhly_ignoreClips: false,
    qhly_yinxiaoshezhi: true,
    qhly_closeVoice: false,
    qhly_currentMusic: "system",
    qhly_enableCharacterMusic: false,
    qhly_modemusicconfig: "system",
    qhly_shuimolingyin: true,
    qhly_titlereplace: "title",
    qhly_hanggaoxiufu: "250",
    qhly_hanggaoxiufu2: "250",
    qhly_shilizihao: "65",
    qhly_lihuiSupport: false,
    qhly_hideShuimoCover: false,
    qhly_lolconfig: true,
    qhly_lolhanggaoxiufu: "250",
    qhly_lolhanggaoxiufu2: "250",
    qhly_lolshilizihao: "65",
    qhly_jianrongxing: true,
    qhly_funcLoadInPrecontent: true,
    qhly_qitashezhi: true,
    qhly_clear: false,
    skinAudioList: "{}",
    audioReplace: "{}",
    qhly_restore: false,
    qhly_plugin: false,
  };
}

const SPEC_MAP: Readonly<Record<string, QhlyConfigSpec>> = Object.fromEntries(
  QHLY_CONFIG_SPECS.map((s) => [s.key, s]),
);

export function validateQhlyConfigPatch(
  raw: Record<string, unknown>,
  opts?: { strict?: boolean },
): { patch: Partial<QhlyUiConfig>; validKeys: number } {
  const strict = Boolean(opts?.strict);
  const patch: Record<string, unknown> = {};
  let validKeys = 0;

  for (const [key, val] of Object.entries(raw)) {
    const spec = SPEC_MAP[key];
    if (!spec) {
      if (strict) throw new Error(`QHLY_CONFIG_IMPORT_UNKNOWN_KEY:${key}`);
      continue;
    }
    if (spec.control === "info" || spec.control === "action") {
      if (typeof val === "boolean" || typeof val === "string") {
        patch[key] = val;
        validKeys++;
      } else if (strict) {
        throw new Error(`QHLY_CONFIG_IMPORT_INVALID_TYPE:${key}`);
      }
      continue;
    }
    if (spec.control === "boolean") {
      if (typeof val !== "boolean") {
        if (strict) throw new Error(`QHLY_CONFIG_IMPORT_INVALID_BOOL:${key}`);
        continue;
      }
      patch[key] = val;
      validKeys++;
      continue;
    }
    if (spec.control === "select") {
      const allowed = spec.options?.map((o) => o.value) ?? [];
      if (typeof val !== "string" || !allowed.includes(val)) {
        if (strict) throw new Error(`QHLY_CONFIG_IMPORT_INVALID_ENUM:${key}`);
        continue;
      }
      patch[key] = val;
      validKeys++;
    }
  }
  return { patch: patch as Partial<QhlyUiConfig>, validKeys };
}

export function normalizeQhlyConfig(raw?: Partial<QhlyUiConfig> | null): QhlyUiConfig {
  const base = createDefaultQhlyConfig();
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) return base;
  const { patch } = validateQhlyConfigPatch(raw as Record<string, unknown>, { strict: false });
  return { ...base, ...patch };
}

export function parseQhlyConfigImportJson(jsonText: string): {
  patch: Partial<QhlyUiConfig>;
  importedKeys: number;
} {
  const trimmed = jsonText.trim();
  if (!trimmed || trimmed.length > 32 * 1024) {
    throw new Error("QHLY_CONFIG_IMPORT_SIZE_INVALID");
  }
  if (/<script|javascript:|__proto__|constructor|prototype/i.test(trimmed)) {
    throw new Error("QHLY_CONFIG_IMPORT_UNSAFE_PAYLOAD");
  }
  const parsed = JSON.parse(trimmed) as unknown;
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("QHLY_CONFIG_IMPORT_NOT_OBJECT");
  }
  const res = validateQhlyConfigPatch(parsed as Record<string, unknown>, { strict: true });
  return { patch: res.patch, importedKeys: res.validKeys };
}

export function loadPersistedQhlyConfig(): QhlyUiConfig {
  try {
    const raw = globalThis.localStorage?.getItem(QHLY_CONFIG_STORAGE_KEY);
    if (!raw) return createDefaultQhlyConfig();
    return normalizeQhlyConfig(JSON.parse(raw) as Partial<QhlyUiConfig>);
  } catch {
    return createDefaultQhlyConfig();
  }
}

export function persistQhlyConfig(cfg: QhlyUiConfig): void {
  try {
    globalThis.localStorage?.setItem(QHLY_CONFIG_STORAGE_KEY, JSON.stringify(cfg));
  } catch {
    // Hardened environments keep in-memory state.
  }
}

export function qhlyArenaDatasetAttrs(cfg: QhlyUiConfig): Record<string, string> {
  return {
    "data-qhly-newui": cfg.qhly_newui ? "on" : "off",
    "data-qhly-viewskin": cfg.qhly_currentViewSkin,
    "data-qhly-lutou": cfg.qhly_lutou ? cfg.qhly_lutouType : "off",
    "data-qhly-lvjing": cfg.lvjingFixx ? "on" : "off",
    "data-qhly-skin-btn": cfg.qhly_skinButton ? "on" : "off",
    "data-qhly-showrarity": cfg.qhly_showrarity ? "on" : "off",
    "data-qhly-smallwin": cfg.qhly_smallwindowstyle,
    "data-qhly-hide-shuimo-cover": cfg.qhly_hideShuimoCover ? "on" : "off",
  };
}
