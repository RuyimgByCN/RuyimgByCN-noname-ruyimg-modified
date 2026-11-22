/**
 * Batch UI-7 统一状态聚合与全量 171 项清单映射（覆盖 16 个来源：131 configs + 35 surfaces + 5 ui_hooks）
 */
import type { CoinSurfaceState } from "./sharedSurfaceHelpers";
import { L01_CONFIG_KEYS } from "./aiOptimizeConfigSchema";
import { createInitialL01AiOptimizeState, type L01AiOptimizeState } from "./aiOptimizeSurface";
import {
  L02_CONFIG_SPECS,
  L02_SURFACE_IDS,
  L15_CONFIG_SPECS,
  L16_CONFIG_SPECS,
  L16_SURFACE_IDS,
  L17_CONFIG_SPECS,
  L17_HOOK_IDS,
  L17_SURFACE_IDS,
  L18_CONFIG_SPECS,
  L18_SURFACE_IDS,
  L21_CONFIG_SPECS,
  L21_SURFACE_IDS,
  U05_CONFIG_SPECS,
} from "./toolsAndPoolSchema";
import {
  createInitialL02State,
  createInitialL15State,
  createInitialL16State,
  createInitialL17State,
  createInitialL18State,
  createInitialL21State,
  createInitialU05State,
  type L02SjState,
  type L15ShoushaPoolState,
  type L16DragReadState,
  type L17DouzhuanState,
  type L18UpdateGuardState,
  type L21MartialUiState,
  type U05ShahaiState,
} from "./toolsAndPoolSurface";
import {
  COIN_HAVE_FUN_SURFACE_IDS,
  L07_CONFIG_SPECS,
  L07_SURFACE_IDS,
  L11_SURFACE_IDS,
  L12_SURFACE_IDS,
  L13_CONFIG_SPECS,
  L13_HOOK_IDS,
  L13_SURFACE_IDS,
  L23_HOOK_IDS,
  L26_CONFIG_SPECS,
  L26_HOOK_IDS,
  L26_SURFACE_IDS,
  U04_HOOK_IDS,
  createInitialL07UiState,
  createInitialL12RzshState,
  createInitialL13UiState,
  createInitialL23State,
  createInitialL26State,
  type L07OmnisearchUiState,
  type L12RzshSurfaceState,
  type L13MvpUiState,
  type L23PcAdaptState,
  type L26FoolPluginState,
} from "./visualEffectsBatch7Runtime";

export interface Batch7UiSuiteState {
  l01: L01AiOptimizeState;
  l02: L02SjState;
  l07: L07OmnisearchUiState;
  l12: L12RzshSurfaceState;
  l13: L13MvpUiState;
  l15: L15ShoushaPoolState;
  l16: L16DragReadState;
  l17: L17DouzhuanState;
  l18: L18UpdateGuardState;
  l21: L21MartialUiState;
  l23: L23PcAdaptState;
  l26: L26FoolPluginState;
  u05: U05ShahaiState;
}

export function createInitialBatch7UiSuiteState(): Batch7UiSuiteState {
  return {
    l01: createInitialL01AiOptimizeState(),
    l02: createInitialL02State(),
    l07: createInitialL07UiState(),
    l12: createInitialL12RzshState(),
    l13: createInitialL13UiState(),
    l15: createInitialL15State(),
    l16: createInitialL16State(),
    l17: createInitialL17State(),
    l18: createInitialL18State(),
    l21: createInitialL21State(),
    l23: createInitialL23State(),
    l26: createInitialL26State(),
    u05: createInitialU05State(),
  };
}

export function normalizeBatch7UiSuiteState(raw?: Partial<Batch7UiSuiteState> | null): Batch7UiSuiteState {
  const base = createInitialBatch7UiSuiteState();
  if (!raw || typeof raw !== "object") return base;
  return {
    l01: raw.l01 ? { ...base.l01, ...raw.l01, config: { ...base.l01.config, ...raw.l01.config } } : base.l01,
    l02: raw.l02 ? { ...base.l02, ...raw.l02, config: { ...base.l02.config, ...raw.l02.config } } : base.l02,
    l07: raw.l07 ? { ...base.l07, ...raw.l07, config: { ...base.l07.config, ...raw.l07.config } } : base.l07,
    l12: raw.l12 ? { ...base.l12, ...raw.l12 } : base.l12,
    l13: raw.l13 ? { ...base.l13, ...raw.l13, config: { ...base.l13.config, ...raw.l13.config } } : base.l13,
    l15: raw.l15 ? { ...base.l15, ...raw.l15, config: { ...base.l15.config, ...raw.l15.config } } : base.l15,
    l16: raw.l16 ? { ...base.l16, ...raw.l16, config: { ...base.l16.config, ...raw.l16.config } } : base.l16,
    l17: raw.l17 ? { ...base.l17, ...raw.l17, config: { ...base.l17.config, ...raw.l17.config } } : base.l17,
    l18: raw.l18 ? { ...base.l18, ...raw.l18, config: { ...base.l18.config, ...raw.l18.config } } : base.l18,
    l21: raw.l21 ? { ...base.l21, ...raw.l21, config: { ...base.l21.config, ...raw.l21.config } } : base.l21,
    l23: raw.l23 ? { ...base.l23, ...raw.l23 } : base.l23,
    l26: raw.l26 ? { ...base.l26, ...raw.l26, config: { ...base.l26.config, ...raw.l26.config } } : base.l26,
    u05: raw.u05 ? { ...base.u05, ...raw.u05, config: { ...base.u05.config, ...raw.u05.config } } : base.u05,
  };
}

export function batch7ArenaDatasetAttrs(
  state: Batch7UiSuiteState,
  coin?: CoinSurfaceState,
): Record<string, string> {
  return {
    "data-l01-view-att": state.l01.config.viewAtt ? "on" : "off",
    "data-l02-devtools": state.l02.config.openTools ? "on" : "off",
    "data-l07-highlight": state.l07.config.css,
    "data-l07-searcher-style": state.l07.searcherStyle,
    "data-l12-surface": state.l12.activeSurface,
    "data-l13-surface": state.l13.config.yangshi === "on" ? "extension_new.css" : "extension.css",
    "data-l15-sexforbid": state.l15.config.sexforbid,
    "data-l17-group-color": state.l17.config.jjGroupColor ? "on" : "off",
    "data-l21-hidden-count": String(state.l21.hiddenIndices.length),
    "data-l23-cailanzi": state.l23.enabled ? "on" : "off",
    "data-l26-liegong": state.l26.config.Fool_liegong,
    "data-u04-canvas-top": coin?.activeEffects.length || coin?.snowFall ? "on" : "off",
  };
}

export const BATCH7_INVENTORY_MANIFEST: Readonly<
  Record<string, { configs?: ReadonlyArray<string>; surfaces?: ReadonlyArray<string>; ui_hooks?: ReadonlyArray<string> }>
> = {
  L01: { configs: L01_CONFIG_KEYS },
  L02: { configs: L02_CONFIG_SPECS.map((s) => s.key), surfaces: L02_SURFACE_IDS },
  L05: { surfaces: COIN_HAVE_FUN_SURFACE_IDS },
  L07: { configs: L07_CONFIG_SPECS.map((s) => s.key), surfaces: L07_SURFACE_IDS },
  L11: { surfaces: L11_SURFACE_IDS },
  L12: { surfaces: L12_SURFACE_IDS },
  L13: { configs: L13_CONFIG_SPECS.map((s) => s.key), surfaces: L13_SURFACE_IDS, ui_hooks: L13_HOOK_IDS },
  L15: { configs: L15_CONFIG_SPECS.map((s) => s.key) },
  L16: { configs: L16_CONFIG_SPECS.map((s) => s.key), surfaces: L16_SURFACE_IDS },
  L17: { configs: L17_CONFIG_SPECS.map((s) => s.key), surfaces: L17_SURFACE_IDS, ui_hooks: L17_HOOK_IDS },
  L18: { configs: L18_CONFIG_SPECS.map((s) => s.key), surfaces: L18_SURFACE_IDS },
  L21: { configs: L21_CONFIG_SPECS.map((s) => s.key), surfaces: L21_SURFACE_IDS },
  L23: { ui_hooks: L23_HOOK_IDS },
  L26: { configs: L26_CONFIG_SPECS.map((s) => s.key), surfaces: L26_SURFACE_IDS, ui_hooks: L26_HOOK_IDS },
  U04: { surfaces: COIN_HAVE_FUN_SURFACE_IDS, ui_hooks: U04_HOOK_IDS },
  U05: { configs: U05_CONFIG_SPECS.map((s) => s.key) },
};
