/**
 * NC05 选将面板的纯视图模型与提交控制器。
 *
 * 为什么单独成层：玩家界面是 Vue SFC，`node:test` 不能直接装载 `.vue`；把
 * 「候选 → 可渲染行」「点击 → chooseGeneral → 刷新投影」「失败 → 可显示错误」
 * 放在无 Vue/DOM 依赖的纯函数里，测试覆盖的就不再只是「组件源码里有个字符串」。
 *
 * 边界：
 * - 候选面的权威结构校验在 Worker 适配层（`nativeBackend.parseGeneralCandidatesJson`）。
 *   这里再做一次**渲染边界**校验：畸形回包不得被过滤成更小的候选列表，也不得
 *   进入模板；空列表是合法状态（明确显示「候选为空」）。
 * - `submitGeneralSelection` 只在 `chooseGeneral` 成功后调用 `applyEnvelope`
 *   刷新主视图；失败保留点击前的投影并返回错误文本，不留下「已装配」的假象。
 *   装配不推进 `revision`（内核实测），所以成功态额外记录 `state_hash`。
 */
import type {
  EngineEnvelope,
  GeneralAssignment,
  ObserverSnapshot,
} from "../types/engine";

export type GeneralSelectLoadStatus =
  | "idle"
  | "loading"
  | "ready"
  | "empty"
  | "error";

export type GeneralSelectionStatus = "idle" | "pending" | "accepted" | "rejected";

/** 候选列表的一行；模板只消费这个结构。 */
export interface GeneralCandidateRow {
  /** 目录内武将定义 id；候选主键。 */
  def: number;
  name: string;
  skills: string[];
  /** 直接展示的技能文本；无技能时是显式文案而不是空串。 */
  skillLabel: string;
}

/** 刷新信封里当前座位的可见投影。 */
export interface SeatProjection {
  /** 原生视图当前恒为 0（未暴露目录 id），保留字段供后续接线。 */
  characterDef: number | null;
  skillIds: number[];
  /** 原生视图按技能名投影（`SkillView.name`），用于展示装配后的变化。 */
  skillNames: string[];
}

export interface GeneralSelectState {
  seat: number;
  loadStatus: GeneralSelectLoadStatus;
  candidates: GeneralCandidateRow[];
  error: string | null;
  selectionStatus: GeneralSelectionStatus;
  pendingName: string | null;
  assignment: GeneralAssignment | null;
  /** 最近一次成功装配的 `state_hash`（装配不推进 revision）。 */
  lastStateHash: string | null;
  projection: SeatProjection;
}

export interface GeneralSelectDeps {
  chooseGeneral(seat: number, name: string): Promise<unknown>;
  /**
   * 选将成功后把刷新信封应用回主视图（技能/座位/统计可见变化）。
   * 失败路径不会调用；测试用桩记录调用次数与参数。
   */
  applyEnvelope?(envelope: EngineEnvelope): void;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** wasm-bindgen 抛 JS 字符串、Worker 回包抛 Error，两种都要可读。 */
export function errorText(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return String(error);
}

function describeValue(value: unknown): string {
  try {
    const text = JSON.stringify(value);
    return (text === undefined ? String(value) : text).slice(0, 160);
  } catch {
    return String(value);
  }
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}

/**
 * 严格解析候选回包：任一畸形都抛错，不静默丢弃。
 *
 * 与适配层同口径（count 整数、逐项 def/name/skills、def 不重复、count 一致），
 * 但这里是 UI 的最后一道边界：旧 Worker、混装产物或手写桩都可能绕过适配层。
 */
export function candidateRowsFromPayload(payload: unknown): GeneralCandidateRow[] {
  if (!isRecord(payload)) {
    throw new Error(
      `选将候选结构不符（需 {count, candidates}）：${describeValue(payload)}`,
    );
  }
  const count = payload.count;
  const rawCandidates = payload.candidates;
  if (
    typeof count !== "number" ||
    !Number.isInteger(count) ||
    count < 0 ||
    !Array.isArray(rawCandidates)
  ) {
    throw new Error(
      `选将候选结构不符（需 {count:非负整数, candidates:数组}）：${describeValue(payload)}`,
    );
  }

  const rows: GeneralCandidateRow[] = [];
  const seenDefs = new Set<number>();
  for (const [index, item] of rawCandidates.entries()) {
    if (!isRecord(item)) {
      throw new Error(
        `选将候选第 ${index} 项结构不符（需对象）：${describeValue(item)}`,
      );
    }
    const def = item.def;
    const name = item.name;
    const skills = item.skills;
    if (
      typeof def !== "number" ||
      !Number.isInteger(def) ||
      def < 0 ||
      typeof name !== "string" ||
      name.trim().length === 0 ||
      !isStringArray(skills)
    ) {
      throw new Error(
        `选将候选第 ${index} 项结构不符（需 {def:非负整数, name:非空字符串, skills:字符串数组}）：${describeValue(item)}`,
      );
    }
    if (seenDefs.has(def)) {
      throw new Error(`选将候选第 ${index} 项 def=${def} 与前面重复`);
    }
    seenDefs.add(def);
    rows.push({
      def,
      name,
      skills: [...skills],
      skillLabel: skills.length > 0 ? skills.join(" · ") : "无技能",
    });
  }

  if (rows.length !== count) {
    throw new Error(
      `选将候选 count=${count} 与候选数 ${rows.length} 不一致`,
    );
  }
  return rows;
}

export function emptySeatProjection(): SeatProjection {
  return { characterDef: null, skillIds: [], skillNames: [] };
}

export function initialGeneralSelectState(
  seat: number,
  projection: SeatProjection = emptySeatProjection(),
): GeneralSelectState {
  return {
    seat,
    loadStatus: "idle",
    candidates: [],
    error: null,
    selectionStatus: "idle",
    pendingName: null,
    assignment: null,
    lastStateHash: null,
    projection: cloneProjection(projection),
  };
}

function cloneProjection(projection: SeatProjection): SeatProjection {
  return {
    characterDef: projection.characterDef,
    skillIds: [...projection.skillIds],
    skillNames: [...projection.skillNames],
  };
}

/**
 * 从观察者投影取某座位的可见面；座位不存在时返回空投影。
 *
 * 原生适配层把窄壳 `skills: string[]` 映射成 `{name}` 对象；旧线可能没有
 * `name`，两种输入都容错，但不伪造任何技能名。
 */
export function seatProjectionFromView(
  view: ObserverSnapshot | null | undefined,
  seat: number,
): SeatProjection {
  const seats = Array.isArray(view?.seats) ? view.seats : [];
  const row = seats.find((candidate) => candidate && candidate.player === seat);
  if (!row) return emptySeatProjection();

  const skillIds: number[] = [];
  const skillNames: string[] = [];
  const skills = Array.isArray(row.skills) ? row.skills : [];
  for (const skill of skills) {
    const id = skill?.id;
    if (typeof id === "number" && Number.isInteger(id) && !skillIds.includes(id)) {
      skillIds.push(id);
    }
    const name = (skill as { name?: unknown } | null | undefined)?.name;
    if (
      typeof name === "string" &&
      name.length > 0 &&
      !skillNames.includes(name)
    ) {
      skillNames.push(name);
    }
  }
  return {
    characterDef:
      typeof row.character_def === "number" ? row.character_def : null,
    skillIds,
    skillNames,
  };
}

/** 用新的观察者投影刷新状态；不清除候选与最近一次装配结果。 */
export function withSeatProjection(
  state: GeneralSelectState,
  projection: SeatProjection,
): GeneralSelectState {
  return { ...state, projection: cloneProjection(projection) };
}

/**
 * 绑定面板到某座位（新对局/换观察者/读档）：座位相关的最近装配与投影必须
 * 清掉，候选目录可复用。**同一个座位也要清**——调用方用它在引擎重建后开始
 * 新一轮，不能把上一局的「已装配」成功态留在界面上。
 */
export function setGeneralSelectSeat(
  state: GeneralSelectState,
  seat: number,
): GeneralSelectState {
  return {
    ...state,
    seat,
    projection: emptySeatProjection(),
    assignment: null,
    lastStateHash: null,
    selectionStatus: "idle",
    pendingName: null,
    error: null,
  };
}

export function markCandidatesLoading(
  state: GeneralSelectState,
): GeneralSelectState {
  return { ...state, loadStatus: "loading", error: null };
}

/** 候选拉取成功：空列表是显式状态，不是错误，也不是静默白屏。 */
export function applyCandidatePayload(
  state: GeneralSelectState,
  payload: unknown,
): GeneralSelectState {
  try {
    const candidates = candidateRowsFromPayload(payload);
    return {
      ...state,
      loadStatus: candidates.length > 0 ? "ready" : "empty",
      candidates,
      error: null,
    };
  } catch (error) {
    return {
      ...state,
      loadStatus: "error",
      candidates: [],
      error: `选将候选不可用：${errorText(error)}`,
    };
  }
}

/** 候选拉取失败（Worker 报错/旧产物缺导出）：清空列表并显示原始错误。 */
export function failCandidateLoad(
  state: GeneralSelectState,
  error: unknown,
): GeneralSelectState {
  return {
    ...state,
    loadStatus: "error",
    candidates: [],
    error: `选将候选加载失败：${errorText(error)}`,
  };
}

function choosePayloadProblem(
  payload: unknown,
  expected: { seat: number; name: string },
): string | null {
  if (!isRecord(payload)) return `回包不是对象：${describeValue(payload)}`;
  const assignment = payload.assignment;
  const envelope = payload.envelope;
  if (!isRecord(assignment)) return "缺 assignment";
  if (assignment.seat !== expected.seat) {
    return `座位回显 ${String(assignment.seat)} ≠ 请求 ${expected.seat}`;
  }
  if (assignment.name !== expected.name) {
    return `武将回显 ${String(assignment.name)} ≠ 请求 ${expected.name}`;
  }
  if (
    typeof assignment.state_hash !== "string" ||
    assignment.state_hash.length === 0
  ) {
    return "缺 assignment.state_hash";
  }
  if (!isRecord(envelope) || !isRecord(envelope.view)) {
    return "缺刷新信封 view";
  }
  return null;
}

/**
 * 玩家点击候选后的完整动作：调用 `chooseGeneral(seat, name)`，成功则应用
 * 刷新信封并更新状态；失败则保留点击前投影并返回可显示的错误。
 *
 * `onPending` 在发起请求前同步触发，供界面立即显示「装配中」并禁用重复点击。
 */
export async function submitGeneralSelection(
  state: GeneralSelectState,
  name: string,
  deps: GeneralSelectDeps,
  onPending?: (pending: GeneralSelectState) => void,
): Promise<GeneralSelectState> {
  if (typeof name !== "string" || name.trim().length === 0) {
    return {
      ...state,
      selectionStatus: "rejected",
      pendingName: null,
      error: "选将失败：候选名为空",
    };
  }

  const pending: GeneralSelectState = {
    ...state,
    selectionStatus: "pending",
    pendingName: name,
    error: null,
  };
  onPending?.(pending);

  let payload: unknown;
  try {
    payload = await deps.chooseGeneral(state.seat, name);
  } catch (error) {
    return {
      ...state,
      selectionStatus: "rejected",
      pendingName: null,
      error: `选将失败（seat=${state.seat}, name=${name}）：${errorText(error)}`,
    };
  }

  const problem = choosePayloadProblem(payload, {
    seat: state.seat,
    name,
  });
  if (problem) {
    return {
      ...state,
      selectionStatus: "rejected",
      pendingName: null,
      error: `选将返回无法确认装配：${problem}`,
    };
  }

  const assignment = (payload as Record<string, unknown>)
    .assignment as GeneralAssignment;
  const envelope = (payload as Record<string, unknown>).envelope as EngineEnvelope;
  const refreshed = withSeatProjection(
    {
      ...pending,
      assignment,
      lastStateHash: assignment.state_hash,
    },
    seatProjectionFromView(envelope.view, state.seat),
  );

  let refreshError: string | null = null;
  try {
    deps.applyEnvelope?.(envelope);
  } catch (error) {
    // 装配已在引擎生效；主视图刷新失败是另一个错误，不能伪装成「选将失败」。
    refreshError = `选将已装配，但视图刷新失败：${errorText(error)}`;
  }

  return {
    ...refreshed,
    selectionStatus: "accepted",
    pendingName: null,
    error: refreshError,
  };
}
