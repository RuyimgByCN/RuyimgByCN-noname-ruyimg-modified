/**
 * L24 皮肤切换 全部 14 项配置 Schema、本地持久化与裁决 38 严格 JSON 导入校验。
 * 上游对照: `archive/upstream/noname-liuli/noname/extension/皮肤切换/extension.js:11634-11744`
 * （严守裁决 35/38：绝不读取 Spine 运行时，绝不捆绑闭源骨骼资产。）
 */

export interface SkinSwitchUiConfig {
  GXNR: "xin";
  previewDynamic: boolean;
  resetArchiveDynamicSkin: boolean;
  showEditMenu: boolean;
  showPreviewDynamicMenu: boolean;
  hideHuanFu: boolean;
  useDynamic: boolean;
  isAttackFlipX: boolean;
  cugDynamicBg: boolean;
  genDynamicSkin: boolean;
  genDyTempFile: boolean;
  previewSkinsDynamic: boolean;
  clickPlayerDynamic: boolean;
  showTopArc: boolean;
}

export interface SkinSwitchConfigSpec {
  readonly key: keyof SkinSwitchUiConfig;
  readonly label: string;
  readonly control: "select" | "boolean" | "action";
  readonly upstreamLine: number;
  readonly options?: ReadonlyArray<{ readonly value: string; readonly label: string }>;
}

export const SKIN_SWITCH_CONFIG_SPECS: ReadonlyArray<SkinSwitchConfigSpec> = [
  {
    key: "GXNR",
    label: "更新内容",
    control: "select",
    upstreamLine: 11636,
    options: [{ value: "xin", label: "查看 1.24 净室更新摘要" }],
  },
  { key: "previewDynamic", label: "预览 Spine 4.0 自备动画", control: "action", upstreamLine: 11663 },
  { key: "resetArchiveDynamicSkin", label: "重置动皮存档参数", control: "action", upstreamLine: 11667 },
  { key: "showEditMenu", label: "编辑动皮加入顶部栏", control: "boolean", upstreamLine: 11677 },
  { key: "showPreviewDynamicMenu", label: "预览 Spine 加入顶部栏", control: "boolean", upstreamLine: 11682 },
  { key: "hideHuanFu", label: "隐藏更换动皮按钮", control: "boolean", upstreamLine: 11687 },
  { key: "useDynamic", label: "启用出框演出", control: "boolean", upstreamLine: 11692 },
  { key: "isAttackFlipX", label: "左侧座位攻击自动翻转 X 轴", control: "boolean", upstreamLine: 11702 },
  { key: "cugDynamicBg", label: "待机裁剪动态背景以适配露头", control: "boolean", upstreamLine: 11707 },
  { key: "genDynamicSkin", label: "转换十周年动态参数模板", control: "action", upstreamLine: 11712 },
  { key: "genDyTempFile", label: "生成自备骨骼标准模板结构", control: "action", upstreamLine: 11716 },
  { key: "previewSkinsDynamic", label: "预览皮肤时启用动皮合成", control: "boolean", upstreamLine: 11722 },
  { key: "clickPlayerDynamic", label: "点击战场座位触发换肤面板", control: "boolean", upstreamLine: 11727 },
  { key: "showTopArc", label: "显示座位顶部圆弧光环", control: "boolean", upstreamLine: 11732 },
];

export const SKIN_SWITCH_CONFIG_KEYS: ReadonlyArray<keyof SkinSwitchUiConfig> =
  SKIN_SWITCH_CONFIG_SPECS.map((s) => s.key);

export function createDefaultSkinSwitchConfig(): SkinSwitchUiConfig {
  return {
    GXNR: "xin",
    previewDynamic: false,
    resetArchiveDynamicSkin: false,
    showEditMenu: false,
    showPreviewDynamicMenu: false,
    hideHuanFu: true,
    useDynamic: true,
    isAttackFlipX: false,
    cugDynamicBg: false,
    genDynamicSkin: false,
    genDyTempFile: false,
    previewSkinsDynamic: true,
    clickPlayerDynamic: false,
    showTopArc: true,
  };
}

const SPEC_MAP: Readonly<Record<string, SkinSwitchConfigSpec>> = Object.fromEntries(
  SKIN_SWITCH_CONFIG_SPECS.map((s) => [s.key, s]),
);

export function validateSkinSwitchConfigPatch(
  raw: Record<string, unknown>,
  opts?: { strict?: boolean },
): { patch: Partial<SkinSwitchUiConfig>; validKeys: number } {
  const strict = Boolean(opts?.strict);
  const patch: Record<string, unknown> = {};
  let validKeys = 0;

  for (const [key, val] of Object.entries(raw)) {
    const spec = SPEC_MAP[key];
    if (!spec) {
      if (strict) throw new Error(`SKINSWITCH_CONFIG_UNKNOWN_KEY:${key}`);
      continue;
    }
    if (spec.control === "select") {
      if (val !== "xin") {
        if (strict) throw new Error(`SKINSWITCH_CONFIG_INVALID_ENUM:${key}`);
        continue;
      }
      patch[key] = val;
      validKeys++;
      continue;
    }
    if (typeof val !== "boolean") {
      if (strict) throw new Error(`SKINSWITCH_CONFIG_INVALID_BOOL:${key}`);
      continue;
    }
    patch[key] = val;
    validKeys++;
  }
  return { patch: patch as Partial<SkinSwitchUiConfig>, validKeys };
}

export function normalizeSkinSwitchConfig(
  raw?: Partial<SkinSwitchUiConfig> | null,
): SkinSwitchUiConfig {
  const base = createDefaultSkinSwitchConfig();
  if (!raw || typeof raw !== "object") return base;
  const { patch } = validateSkinSwitchConfigPatch(raw as Record<string, unknown>, { strict: false });
  return { ...base, ...patch };
}

export function parseSkinSwitchConfigImportJson(
  jsonText: string,
): { patch: Partial<SkinSwitchUiConfig>; importedKeys: number } {
  const parsed: unknown = JSON.parse(jsonText);
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("SKINSWITCH_CONFIG_IMPORT_MUST_BE_OBJECT");
  }
  const res = validateSkinSwitchConfigPatch(parsed as Record<string, unknown>, { strict: true });
  return { patch: res.patch, importedKeys: res.validKeys };
}

const STORAGE_KEY = "noname-next:skinswitch-config:v1";

export function loadPersistedSkinSwitchConfig(): SkinSwitchUiConfig {
  try {
    if (typeof globalThis.localStorage === "undefined") return createDefaultSkinSwitchConfig();
    const raw = globalThis.localStorage.getItem(STORAGE_KEY);
    if (!raw) return createDefaultSkinSwitchConfig();
    return normalizeSkinSwitchConfig(JSON.parse(raw) as Partial<SkinSwitchUiConfig>);
  } catch {
    return createDefaultSkinSwitchConfig();
  }
}

export function persistSkinSwitchConfig(cfg: SkinSwitchUiConfig): void {
  try {
    if (typeof globalThis.localStorage === "undefined") return;
    globalThis.localStorage.setItem(STORAGE_KEY, JSON.stringify(normalizeSkinSwitchConfig(cfg)));
  } catch {
    // 忽略沙盒存储限制
  }
}
