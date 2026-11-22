/**
 * L01 AI优化 运行时状态管理（无 eval 伪禁列表/角色禁选/权重/威胁度编辑器 + 火眼金睛态度观测）
 */
import {
  createDefaultL01Config,
  importL01ConfigJson,
  normalizeL01Config,
  type L01AiOptimizeConfig,
} from "./aiOptimizeConfigSchema";

export type L01RoleBanKey = "banzhu" | "banzhong" | "banfan" | "bannei" | "bandizhu" | "bannongmin";

export interface L01AiOptimizeState {
  config: L01AiOptimizeConfig;
  introOpen: boolean;
  inspectedBanRole: L01RoleBanKey | null;
  clipboardBuffer: string;
  lastActionNotice: string;
}

export function createInitialL01AiOptimizeState(
  rawConfig?: Partial<L01AiOptimizeConfig> | null,
): L01AiOptimizeState {
  return {
    config: normalizeL01Config(rawConfig),
    introOpen: false,
    inspectedBanRole: null,
    clipboardBuffer: "",
    lastActionNotice: "L01 AI优化 63 项配置就绪（净室 JSON 校验）",
  };
}

export function updateL01ConfigInState(
  state: L01AiOptimizeState,
  patch: Partial<L01AiOptimizeConfig>,
): L01AiOptimizeState {
  const nextCfg = normalizeL01Config({ ...state.config, ...patch });
  return {
    ...state,
    config: nextCfg,
    lastActionNotice: `已更新 L01 配置 (${Object.keys(patch).join(", ")})`,
  };
}

export function applyL01JsonImportInState(
  state: L01AiOptimizeState,
  jsonText: string,
): L01AiOptimizeState {
  const imported = importL01ConfigJson(jsonText);
  return {
    ...state,
    config: imported,
    lastActionNotice: "已导入并校验 L01 AI优化 63 项配置",
  };
}

export function exportL01ConfigJson(state: L01AiOptimizeState): string {
  return JSON.stringify(state.config, null, 2);
}

export type L01ActionKey =
  | "kzjs"
  | "copyQg"
  | "copyWp"
  | "copyYp"
  | "editWj"
  | "copyWj"
  | "clearWj"
  | "banzhu"
  | "banzhubiao"
  | "banzhong"
  | "banzhongbiao"
  | "banfan"
  | "banfanbiao"
  | "bannei"
  | "banneibiao"
  | "bandizhu"
  | "bandizhubiao"
  | "bannongmin"
  | "bannongminbiao"
  | "chooseQz"
  | "deleteQz"
  | "editQz"
  | "copyQz"
  | "clearQz"
  | "chooseCf"
  | "deleteCf"
  | "editCf"
  | "copyCf"
  | "clearCf";

export function runL01ConfigAction(
  state: L01AiOptimizeState,
  actionKey: L01ActionKey,
  payload?: { generalId?: string; skillId?: string; jsonText?: string },
): L01AiOptimizeState {
  const cfg = state.config;
  switch (actionKey) {
    case "kzjs":
      return { ...state, introOpen: !state.introOpen, lastActionNotice: "切换山河永固/官将重修介绍" };
    case "copyQg":
      return { ...state, clipboardBuffer: cfg.copyQg, lastActionNotice: `已复制群号 ${cfg.copyQg}` };
    case "copyWp":
      return { ...state, clipboardBuffer: cfg.copyWp, lastActionNotice: `已缓存网盘备份标记` };
    case "copyYp":
      return { ...state, clipboardBuffer: cfg.copyYp, lastActionNotice: `已缓存云盘备份标记` };
    case "editWj": {
      if (payload?.jsonText) {
        const parsed = importL01ConfigJson(JSON.stringify({ editWj: JSON.parse(payload.jsonText) }));
        return {
          ...state,
          config: { ...cfg, editWj: parsed.editWj },
          lastActionNotice: `已更新伪禁列表 (${parsed.editWj.length} 项)`,
        };
      }
      const target = payload?.generalId?.trim();
      return target
        ? toggleL01RoleBanGeneral(state, "editWj", target)
        : { ...state, clipboardBuffer: JSON.stringify(cfg.editWj), lastActionNotice: `已装载伪禁 JSON (${cfg.editWj.length} 项)` };
    }
    case "copyWj": {
      const copyWj = JSON.stringify(cfg.editWj);
      return {
        ...state,
        config: { ...cfg, copyWj },
        clipboardBuffer: copyWj,
        lastActionNotice: `已导出伪禁列表 (${cfg.editWj.length} 项)`,
      };
    }
    case "clearWj":
      return {
        ...state,
        config: { ...cfg, editWj: [], clearWj: cfg.clearWj + 1 },
        lastActionNotice: "已清空伪禁列表",
      };
    case "banzhu":
    case "banzhong":
    case "banfan":
    case "bannei":
    case "bandizhu":
    case "bannongmin": {
      const target = (payload?.generalId ?? "").trim();
      if (target) {
        const toggled = toggleL01RoleBanGeneral(state, actionKey, target);
        return { ...toggled, inspectedBanRole: actionKey };
      }
      return {
        ...state,
        inspectedBanRole: actionKey,
        lastActionNotice: `已切换 ${actionKey} 操作目标（当前 ${cfg[actionKey].length} 项）`,
      };
    }
    case "banzhubiao":
      return { ...state, inspectedBanRole: "banzhu", lastActionNotice: `主公禁选表 (${cfg.banzhu.length} 项)` };
    case "banzhongbiao":
      return { ...state, inspectedBanRole: "banzhong", lastActionNotice: `忠臣禁选表 (${cfg.banzhong.length} 项)` };
    case "banfanbiao":
      return { ...state, inspectedBanRole: "banfan", lastActionNotice: `反贼禁选表 (${cfg.banfan.length} 项)` };
    case "banneibiao":
      return { ...state, inspectedBanRole: "bannei", lastActionNotice: `内奸禁选表 (${cfg.bannei.length} 项)` };
    case "bandizhubiao":
      return { ...state, inspectedBanRole: "bandizhu", lastActionNotice: `地主禁选表 (${cfg.bandizhu.length} 项)` };
    case "bannongminbiao":
      return { ...state, inspectedBanRole: "bannongmin", lastActionNotice: `农民禁选表 (${cfg.bannongmin.length} 项)` };
    case "chooseQz": {
      const keys = Object.keys(cfg.editQz);
      const nextChoice =
        payload?.generalId && cfg.editQz[payload.generalId] !== undefined
          ? payload.generalId
          : keys.length > 0
            ? keys[(keys.indexOf(cfg.chooseQz) + 1) % keys.length]!
            : "";
      return {
        ...state,
        config: { ...cfg, chooseQz: nextChoice },
        lastActionNotice: nextChoice ? `选中权重武将：${nextChoice} (${cfg.editQz[nextChoice]})` : "当前无已设权重武将",
      };
    }
    case "deleteQz": {
      const target = payload?.generalId ?? cfg.chooseQz;
      const nextQz = { ...cfg.editQz };
      delete nextQz[target];
      const nextKeys = Object.keys(nextQz);
      return {
        ...state,
        config: { ...cfg, editQz: nextQz, deleteQz: target, chooseQz: nextKeys[0] ?? "" },
        lastActionNotice: `已移除权重：${target}`,
      };
    }
    case "editQz":
      if (payload?.jsonText) return applyL01WeightMapJson(state, "editQz", payload.jsonText);
      return {
        ...state,
        clipboardBuffer: JSON.stringify(cfg.editQz),
        lastActionNotice: `已聚焦武将权重 JSON (${Object.keys(cfg.editQz).length} 项)`,
      };
    case "copyQz": {
      const copyQz = JSON.stringify(cfg.editQz);
      return { ...state, config: { ...cfg, copyQz }, clipboardBuffer: copyQz, lastActionNotice: "已导出武将权重" };
    }
    case "clearQz":
      return {
        ...state,
        config: { ...cfg, editQz: {}, chooseQz: "", clearQz: cfg.clearQz + 1 },
        lastActionNotice: "已清空武将权重",
      };
    case "chooseCf": {
      const keys = Object.keys(cfg.editCf);
      const nextChoice =
        payload?.skillId && cfg.editCf[payload.skillId] !== undefined
          ? payload.skillId
          : keys.length > 0
            ? keys[(keys.indexOf(cfg.chooseCf) + 1) % keys.length]!
            : "";
      return {
        ...state,
        config: { ...cfg, chooseCf: nextChoice },
        lastActionNotice: nextChoice ? `选中威胁度技能：${nextChoice} (${cfg.editCf[nextChoice]})` : "当前无已改技能威胁度",
      };
    }
    case "deleteCf": {
      const target = payload?.skillId ?? cfg.chooseCf;
      const nextCf = { ...cfg.editCf };
      delete nextCf[target];
      const nextKeys = Object.keys(nextCf);
      return {
        ...state,
        config: { ...cfg, editCf: nextCf, deleteCf: target, chooseCf: nextKeys[0] ?? "" },
        lastActionNotice: `已移除技能威胁度：${target}`,
      };
    }
    case "editCf":
      if (payload?.jsonText) return applyL01WeightMapJson(state, "editCf", payload.jsonText);
      return {
        ...state,
        clipboardBuffer: JSON.stringify(cfg.editCf),
        lastActionNotice: `已校验技能威胁度正数映射 (${Object.keys(cfg.editCf).length} 项 > 0)`,
      };
    case "copyCf": {
      const copyCf = JSON.stringify(cfg.editCf);
      return { ...state, config: { ...cfg, copyCf }, clipboardBuffer: copyCf, lastActionNotice: "已导出技能威胁度" };
    }
    case "clearCf":
      return {
        ...state,
        config: { ...cfg, editCf: {}, chooseCf: "", clearCf: cfg.clearCf + 1 },
        lastActionNotice: "已清空技能威胁度",
      };
    default: {
      const _exhaustive: never = actionKey;
      throw new Error(`Unhandled L01 action: ${String(_exhaustive)}`);
    }
  }
}

export function toggleL01RoleBanGeneral(
  state: L01AiOptimizeState,
  roleKey: L01RoleBanKey | "editWj",
  generalId: string,
): L01AiOptimizeState {
  const clean = generalId.trim();
  if (!clean) return state;
  const prev = state.config[roleKey];
  const exists = prev.includes(clean);
  const nextList = exists ? prev.filter((x) => x !== clean) : [...prev, clean];
  return {
    ...state,
    config: { ...state.config, [roleKey]: nextList },
    lastActionNotice: `${exists ? "移出" : "加入"} ${roleKey}: ${clean}`,
  };
}

export function applyL01WeightMapJson(
  state: L01AiOptimizeState,
  target: "editQz" | "editCf",
  jsonText: string,
): L01AiOptimizeState {
  const parsed = importL01ConfigJson(JSON.stringify({ [target]: JSON.parse(jsonText) }));
  return {
    ...state,
    config: { ...state.config, [target]: parsed[target] },
    lastActionNotice: `已更新 ${target}（${Object.keys(parsed[target]).length} 项）`,
  };
}

export function resetL01State(): L01AiOptimizeState {
  return createInitialL01AiOptimizeState(createDefaultL01Config());
}
