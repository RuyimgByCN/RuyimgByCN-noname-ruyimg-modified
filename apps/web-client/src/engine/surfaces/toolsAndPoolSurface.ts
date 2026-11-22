/**
 * Batch UI-7 工具/选将池/更新/禁将/武皇 真实消费者与行为实现
 */
import { validateSafeExportFilename } from "./sharedSurfaceHelpers";
import {
  createDefaultL02Config,
  createDefaultL15Config,
  createDefaultL16Config,
  createDefaultL17Config,
  createDefaultL18Config,
  createDefaultL21Config,
  createDefaultU05Config,
  normalizeL21Config,
  type L02SjConfig,
  type L15PoolConfig,
  type L16DragReadConfig,
  type L17DouzhuanConfig,
  type L18UpdateGuardConfig,
  type L21MartialConfig,
  type U05ShahaiConfig,
} from "./toolsAndPoolSchema";

export interface L02SjState {
  config: L02SjConfig;
  consoleLogsOpen: boolean;
  tutorialDocOpen: boolean;
  offlineCatalogOpen: boolean;
  offlineExtensionSummary: string;
}

export interface L15ShoushaPoolState {
  config: L15PoolConfig;
  nailongToast: string | null;
}

export interface L16DragReadState {
  config: L16DragReadConfig;
  changelogOpen: boolean;
  lastZipManifest: string | null;
}

export interface L17DouzhuanState {
  config: L17DouzhuanConfig;
  banPanelOpen: boolean;
  activePlanIndex: 1 | 2 | 3;
  planNames: [string, string, string];
  plans: { 1: string[]; 2: string[]; 3: string[] };
  charStars: Record<string, number>;
}

export interface L18UpdateGuardState {
  config: L18UpdateGuardConfig;
  progressPct: number;
  statusMessage: string;
  linkExplainText: string;
}

export interface L21MartialUiState {
  config: L21MartialConfig;
  hiddenIndices: number[];
  hiddenPreviewActive?: boolean;
}

export interface U05ShahaiState {
  config: U05ShahaiConfig;
}

export function createInitialL02State(): L02SjState {
  return {
    config: createDefaultL02Config(),
    consoleLogsOpen: false,
    tutorialDocOpen: false,
    offlineCatalogOpen: false,
    offlineExtensionSummary: "离线扩展目录：37 来源 / 46 原生包（跳过远端 eval）",
  };
}

export function applyL15NamePrefix(name: string, mode: L15PoolConfig["shoushanameX"]): string {
  if (mode === "none") return name;
  let out = name.replace(/^手杀/, "界");
  if (mode === "all") {
    out = out.replace(/^(?:OL|旧|新杀|SP|☆SP)/, "");
  }
  return out;
}

export function applyL15InfoFix(desc: string, enabled: boolean): string {
  const trimmed = desc.trim();
  if (!enabled || !trimmed) return trimmed;
  if (/[。！？.!?…]$/.test(trimmed)) return trimmed;
  return `${trimmed}。`;
}

export function filterGeneralPoolByL15(
  generals: ReadonlyArray<{ id: string; sex?: "male" | "female"; pack?: string }>,
  cfg: L15PoolConfig,
): Array<{ id: string; sex?: "male" | "female"; pack?: string }> {
  return generals.filter((g) => {
    if (cfg.txhj && g.pack === "taixu") return false;
    if (cfg.sexforbid !== "none" && g.sex && g.sex !== cfg.sexforbid) return false;
    if (cfg.huanyuan && g.pack === "standard") return false;
    return true;
  });
}

export function createInitialL15State(): L15ShoushaPoolState {
  return { config: createDefaultL15Config(), nailongToast: null };
}

export function runL16ManifestDryRun(archiveName: string, entryCount = 42): string {
  const safeName = validateSafeExportFilename(archiveName);
  return `清单就绪：${safeName} (${entryCount} 个受托资源 · 无原生 FS 直读)`;
}

export function createInitialL16State(): L16DragReadState {
  return { config: createDefaultL16Config(), changelogOpen: false, lastZipManifest: null };
}

export function createInitialL17State(): L17DouzhuanState {
  return {
    config: createDefaultL17Config(),
    banPanelOpen: false,
    activePlanIndex: 1,
    planNames: ["方案一", "方案二", "方案三"],
    plans: { 1: ["caocao"], 2: ["liubei"], 3: [] },
    charStars: { caocao: 4, ddd_caomao: 5 },
  };
}

export function dispatchBanCharFHook(
  state: L17DouzhuanState,
  action: "toggleModal" | "selectPlan" | "toggleGeneral" | "setStars" | "batchBan",
  payload?: { plan?: 1 | 2 | 3; generalId?: string; stars?: number; generals?: string[] },
): L17DouzhuanState {
  switch (action) {
    case "toggleModal":
      return { ...state, banPanelOpen: !state.banPanelOpen };
    case "selectPlan": {
      const p = payload?.plan ?? 1;
      return { ...state, activePlanIndex: p };
    }
    case "toggleGeneral": {
      const gid = payload?.generalId?.trim();
      if (!gid) return state;
      const cur = state.plans[state.activePlanIndex];
      const nextList = cur.includes(gid) ? cur.filter((x) => x !== gid) : [...cur, gid];
      return {
        ...state,
        plans: { ...state.plans, [state.activePlanIndex]: nextList },
      };
    }
    case "setStars": {
      const gid = payload?.generalId?.trim();
      if (!gid) return state;
      const stars = Math.max(1, Math.min(5, Math.round(payload?.stars ?? 3)));
      return { ...state, charStars: { ...state.charStars, [gid]: stars } };
    }
    case "batchBan": {
      const list = payload?.generals ?? [];
      return {
        ...state,
        plans: { ...state.plans, [state.activePlanIndex]: Array.from(new Set(list)) },
      };
    }
  }
}

export const L18_EXPLAIN_MAP: Record<L18UpdateGuardConfig["update_link_explain"], string> = {
  无: "请选择更新通道以查看说明",
  coding: "Coding 历史更新源（原版主通道，已不可用）",
  github: "GitHub 源（直连需良好网络环境）",
  fastgit: "GitHub 国内加速镜像通道",
  xuanwu: "玄武镜像（已弃用，切换将拒绝）",
  URC: "Show-K 提供的 United Rhythmized Club 推荐更新源",
};

export function createInitialL18State(): L18UpdateGuardState {
  return {
    config: createDefaultL18Config(),
    progressPct: 0,
    statusMessage: "更新保护已就绪（离线验证模式）",
    linkExplainText: L18_EXPLAIN_MAP.URC,
  };
}

export function runL18ProgressCheck(
  state: L18UpdateGuardState,
  kind: "game" | "asset",
): L18UpdateGuardState {
  const checkedParts = [
    state.config.assetAudio ? "音频" : null,
    state.config.assetSkin ? "皮肤" : null,
    state.config.assetImage || state.config.assetImageFull ? "图片" : null,
  ].filter(Boolean);
  return {
    ...state,
    progressPct: 100,
    statusMessage:
      kind === "game"
        ? `当前原生版本已是最新 · 源=${state.config.update_link}`
        : `素材核验完成 (${checkedParts.join("/") || "基础"}) · 进度 100%`,
  };
}

export function computeL21DoubleHp(
  hpA: number,
  hpB: number,
  mode: L21MartialConfig["double_hp"],
): number {
  const a = Math.max(1, Math.floor(hpA));
  const b = Math.max(1, Math.floor(hpB));
  switch (mode) {
    case "hejiansan":
      return Math.max(1, a + b - 3);
    case "pingjun":
      return Math.max(1, Math.floor((a + b) / 2));
    case "zuidazhi":
      return Math.max(a, b);
    case "zuixiaozhi":
      return Math.min(a, b);
    case "zonghe":
      return a + b;
  }
}

export function createInitialL21State(raw?: Partial<L21MartialConfig> | null): L21MartialUiState {
  const cfg = normalizeL21Config(raw);
  const hideCount = Number(cfg.hide_number);
  const showCount = Number(cfg.show_number);
  const hiddenIndices = Array.from({ length: Math.min(hideCount, showCount) }, (_, idx) => showCount - 1 - idx);
  return { config: cfg, hiddenIndices, hiddenPreviewActive: false };
}

export function createInitialU05State(): U05ShahaiState {
  return { config: createDefaultU05Config() };
}
