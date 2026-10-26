/**
 * 表现层性能采样的纯统计函数（主线程探针与 headless 驱动共用，可在 Node 下单测）。
 *
 * 分位口径与 NC12 内核基准一致：nearest-rank，`排序后 v[ceil(p/100·n)−1]`。
 */

/** nearest-rank 分位；空输入返回 `null`（不以 0 冒充数据）。 */
export function percentile(values: readonly number[], p: number): number | null {
  if (values.length === 0) return null;
  const sorted = [...values].sort((a, b) => a - b);
  const rank = Math.ceil((p / 100) * sorted.length);
  return sorted[Math.min(sorted.length, Math.max(1, rank)) - 1];
}

export interface DistributionSummary {
  count: number;
  p50: number | null;
  p95: number | null;
  p99: number | null;
  max: number | null;
  mean: number | null;
}

export function summarize(values: readonly number[]): DistributionSummary {
  const finite = values.filter((v) => Number.isFinite(v));
  const sum = finite.reduce((acc, v) => acc + v, 0);
  return {
    count: finite.length,
    p50: percentile(finite, 50),
    p95: percentile(finite, 95),
    p99: percentile(finite, 99),
    max: finite.length ? Math.max(...finite) : null,
    mean: finite.length ? sum / finite.length : null,
  };
}

/** 帧时间戳（ms，单调）→ 相邻帧间隔。 */
export function frameDeltas(timestamps: readonly number[]): number[] {
  const out: number[] = [];
  for (let i = 1; i < timestamps.length; i++) out.push(timestamps[i] - timestamps[i - 1]);
  return out;
}

export interface FrameSummary extends DistributionSummary {
  /** 帧间隔中位数，近似本环境的刷新周期（headless 常为 60Hz）。 */
  refresh_estimate_ms: number | null;
  /** 超过 120Hz 预算（8.33ms）的帧占比。 */
  over_8_33_ratio: number | null;
  /** 超过 60Hz 预算（16.67ms）的帧占比。 */
  over_16_67_ratio: number | null;
  /** 超过 1.5 倍刷新周期的帧占比（掉帧近似，与环境刷新率无关）。 */
  over_1_5x_refresh_ratio: number | null;
}

export function summarizeFrames(deltas: readonly number[]): FrameSummary {
  const base = summarize(deltas);
  const ratio = (limit: number) =>
    deltas.length ? deltas.filter((d) => d > limit).length / deltas.length : null;
  const refresh = base.p50;
  return {
    ...base,
    refresh_estimate_ms: refresh,
    over_8_33_ratio: ratio(1000 / 120),
    over_16_67_ratio: ratio(1000 / 60),
    over_1_5x_refresh_ratio: refresh === null ? null : ratio(refresh * 1.5),
  };
}

/** Worker 侧单次请求分段（ms；字节为 UTF-16 码元数）。 */
export interface WorkerStepTimings {
  /** 请求进入 → 进入信封合成：引擎命令（及合成前的选择读取）。 */
  dispatch_ms: number | null;
  /** `view_json` 引擎生成 + 序列化。 */
  view_json_ms: number | null;
  /** 视图 `JSON.parse`。 */
  view_parse_ms: number | null;
  /** 视图适配 + 选择 + 合法动作 + hash + 公开事件。 */
  adapt_ms: number | null;
  /** 请求进入 → 回包 `postMessage` 前。 */
  total_ms: number | null;
  /** 回包 `postMessage` 调用本身（结构化克隆序列化）。 */
  post_ms: number | null;
  /** 回包发出时刻（`performance.timeOrigin + now()`，用于跨线程对齐）。 */
  post_epoch_ms: number | null;
  view_json_chars: number | null;
  envelope_json_chars: number | null;
}

/** 主线程侧单次交互的原始时间点（`performance.now()`，ms）。 */
export interface InteractionMarks {
  id: number;
  type: string;
  /** 触发该请求的输入事件 `timeStamp`；无关联输入（自动推进等）为 null。 */
  input_ts: number | null;
  input_kind: string | null;
  sent: number;
  received: number | null;
  received_epoch_ms: number | null;
  apply_start: number | null;
  apply_end: number | null;
  /** Vue `nextTick` 回调：响应式更新已刷新到 DOM。 */
  patched: number | null;
  /** 刷新后的第一个 rAF 回调。 */
  frame: number | null;
  /** 该 rAF 之后的第一个宏任务（近似该帧样式/布局/绘制已完成；不是像素呈现）。 */
  after_paint: number | null;
  worker: WorkerStepTimings | null;
}

export interface InteractionRecord {
  id: number;
  type: string;
  input_kind: string | null;
  input_to_send_ms: number | null;
  roundtrip_ms: number | null;
  worker_total_ms: number | null;
  dispatch_ms: number | null;
  view_json_ms: number | null;
  view_parse_ms: number | null;
  adapt_ms: number | null;
  post_ms: number | null;
  transfer_ms: number | null;
  /** 回包到达 → 开始应用信封（main 线程排队，含 await 续延）。 */
  receive_to_apply_ms: number | null;
  apply_ms: number | null;
  vue_patch_ms: number | null;
  patch_to_frame_ms: number | null;
  frame_to_paint_ms: number | null;
  /** 输入（无输入时为发送）→ 绘制后宏任务：端到端近似。 */
  input_to_paint_ms: number | null;
  view_json_chars: number | null;
  envelope_json_chars: number | null;
}

const diff = (a: number | null, b: number | null): number | null =>
  a === null || b === null ? null : a - b;

export function toInteractionRecord(m: InteractionMarks): InteractionRecord {
  const w = m.worker;
  const start = m.input_ts ?? m.sent;
  return {
    id: m.id,
    type: m.type,
    input_kind: m.input_kind,
    input_to_send_ms: diff(m.sent, m.input_ts),
    roundtrip_ms: diff(m.received, m.sent),
    worker_total_ms: w?.total_ms ?? null,
    dispatch_ms: w?.dispatch_ms ?? null,
    view_json_ms: w?.view_json_ms ?? null,
    view_parse_ms: w?.view_parse_ms ?? null,
    adapt_ms: w?.adapt_ms ?? null,
    post_ms: w?.post_ms ?? null,
    transfer_ms: diff(m.received_epoch_ms, w?.post_epoch_ms ?? null),
    receive_to_apply_ms: diff(m.apply_start, m.received),
    apply_ms: diff(m.apply_end, m.apply_start),
    vue_patch_ms: diff(m.patched, m.apply_end),
    patch_to_frame_ms: diff(m.frame, m.patched),
    frame_to_paint_ms: diff(m.after_paint, m.frame),
    input_to_paint_ms: diff(m.after_paint, start),
    view_json_chars: w?.view_json_chars ?? null,
    envelope_json_chars: w?.envelope_json_chars ?? null,
  };
}

/** 交互记录的逐字段分布（只统计非 null 值）。 */
export function summarizeInteractions(
  records: readonly InteractionRecord[],
): Record<string, DistributionSummary> {
  const keys: (keyof InteractionRecord)[] = [
    "input_to_send_ms",
    "roundtrip_ms",
    "worker_total_ms",
    "dispatch_ms",
    "view_json_ms",
    "view_parse_ms",
    "adapt_ms",
    "post_ms",
    "transfer_ms",
    "receive_to_apply_ms",
    "apply_ms",
    "vue_patch_ms",
    "patch_to_frame_ms",
    "frame_to_paint_ms",
    "input_to_paint_ms",
    "view_json_chars",
    "envelope_json_chars",
  ];
  const out: Record<string, DistributionSummary> = {};
  for (const key of keys) {
    const values = records
      .map((r) => r[key])
      .filter((v): v is number => typeof v === "number" && Number.isFinite(v));
    out[key] = summarize(values);
  }
  return out;
}
