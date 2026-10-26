/**
 * Worker 侧请求分段计时（表现层性能采样的 Worker 半边，**默认关闭，只做测量**）。
 *
 * 主线程以 init 请求的 `perf: true` 开启。开启后每个请求回包之后追加一条
 * `{ type: "step_timings", id, timings }`（主线程按 id 合并，见 `perfProbe.ts`）。
 * 关闭时 `stepMark` 等入口立即返回，回包路径与原先完全相同。
 */
import type { WorkerStepTimings } from "../engine/perfStats";

let enabled = false;
let begin = 0;
let enter: number | null = null;
let view: number | null = null;
let parse: number | null = null;
let done: number | null = null;
let viewChars: number | null = null;

export function setStepTimingsEnabled(on: boolean): void {
  enabled = on;
}

/** 请求进入 Worker 处理（就绪等待之后）。 */
export function stepBegin(): void {
  if (!enabled) return;
  begin = performance.now();
  enter = view = parse = done = viewChars = null;
}

/** 信封合成内部的分段点；同一请求多次合成时以最后一次为准。 */
export function stepMark(
  point: "envelope_enter" | "view_json" | "view_parse" | "envelope_done",
  chars?: number,
): void {
  if (!enabled) return;
  const now = performance.now();
  if (point === "envelope_enter") enter = now;
  else if (point === "view_json") {
    view = now;
    viewChars = chars ?? null;
  } else if (point === "view_parse") parse = now;
  else done = now;
}

const span = (a: number | null, b: number | null): number | null =>
  a === null || b === null ? null : b - a;

/**
 * 发送回包；开启时随后追加分段。`envelope_json_chars` 在回包发出之后计算，
 * 不计入任何被测段。
 */
export function postWithStepTimings(
  post: (message: unknown) => void,
  resp: { id?: number; ok?: boolean; data?: unknown },
): void {
  if (!enabled || typeof resp.id !== "number") {
    post(resp);
    return;
  }
  const t0 = performance.now();
  post(resp);
  const t1 = performance.now();
  let envelopeChars: number | null = null;
  try {
    envelopeChars = resp.ok ? JSON.stringify(resp.data ?? null).length : null;
  } catch {
    envelopeChars = null;
  }
  const timings: WorkerStepTimings = {
    dispatch_ms: span(begin, enter),
    view_json_ms: span(enter, view),
    view_parse_ms: span(view, parse),
    adapt_ms: span(parse, done),
    total_ms: t0 - begin,
    post_ms: t1 - t0,
    post_epoch_ms: performance.timeOrigin + t0,
    view_json_chars: viewChars,
    envelope_json_chars: envelopeChars,
  };
  post({ type: "step_timings", id: resp.id, timings });
}
