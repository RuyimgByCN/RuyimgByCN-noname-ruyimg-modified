/**
 * 表现层性能采样探针（主线程，**默认关闭，只做测量**）。
 *
 * 开启：战斗页 URL 带 `perf=1`（history 模式的查询串或 hash 路由的查询串），
 * 或预置 `localStorage["noname-next-perf"]="1"`。开启后：
 * - rAF 帧时间戳（帧间隔分布）；
 * - Long Task 与 Event Timing（≥16ms 的输入事件）；
 * - 每个 Worker 请求的分段：输入 → 发送 → Worker 各段 → 回包 → 应用信封 →
 *   Vue 刷新 DOM → 下一帧 rAF → 该帧之后的宏任务。
 * 结果挂在 `window.__nonamePerf`，由 headless 驱动或控制台读取。关闭时所有入口
 * 立即返回，不注册监听、不改变规则、协议或界面行为。
 */
import { nextTick } from "vue";
import {
  frameDeltas,
  summarize,
  summarizeFrames,
  summarizeInteractions,
  toInteractionRecord,
  type InteractionMarks,
  type WorkerStepTimings,
} from "./perfStats";

const STORAGE_KEY = "noname-next-perf";
const MAX_FRAMES = 300_000;
const MAX_INTERACTIONS = 20_000;
const MAX_ENTRIES = 50_000;
/** 输入事件与随后请求的最大关联间隔：超过视为无关联输入（如自动推进）。 */
const INPUT_LINK_MS = 100;

interface LongTaskSample {
  start: number;
  duration: number;
}

interface EventTimingSample {
  name: string;
  start: number;
  duration: number;
  input_delay: number;
  processing: number;
}

let started = false;
let enabled = false;
const frameTs: number[] = [];
const longTasks: LongTaskSample[] = [];
const eventTimings: EventTimingSample[] = [];
const marks = new Map<number, InteractionMarks>();
const orphanWorker = new Map<number, WorkerStepTimings>();
let lastInput: { ts: number; kind: string; used: boolean } | null = null;
let lastReceived: number | null = null;

function detectEnabled(): boolean {
  try {
    if (globalThis.localStorage?.getItem(STORAGE_KEY) === "1") return true;
  } catch {
    /* 隐私模式等场景忽略 */
  }
  if (typeof location === "undefined") return false;
  if (new URLSearchParams(location.search).get("perf") === "1") return true;
  const hashQuery = location.hash.split("?")[1];
  return hashQuery ? new URLSearchParams(hashQuery).get("perf") === "1" : false;
}

const epochNow = () => performance.timeOrigin + performance.now();

function observe(type: string, opts: Record<string, unknown>, sink: (e: PerformanceEntry) => void) {
  if (typeof PerformanceObserver === "undefined") return;
  if (!PerformanceObserver.supportedEntryTypes?.includes(type)) return;
  const po = new PerformanceObserver((list) => {
    for (const entry of list.getEntries()) sink(entry);
  });
  po.observe({ type, buffered: true, ...opts } as PerformanceObserverInit);
}

function onInput(ev: Event) {
  lastInput = { ts: ev.timeStamp, kind: ev.type, used: false };
}

/** 开启检测并启动采样（幂等）；返回是否开启。由 Worker 客户端连接时调用。 */
/** PA08：无 CDP 环境的自驱动（autodrive 参数门控；无参数时惰性）。 */
import { startAutoDrive } from "./autoDrive";

export function startPerfProbe(): boolean {
  if (started) return enabled;
  started = true;
  enabled = detectEnabled();
  if (!enabled || typeof window === "undefined") return enabled;
  const loop = (ts: number) => {
    if (frameTs.length < MAX_FRAMES) frameTs.push(ts);
    requestAnimationFrame(loop);
  };
  requestAnimationFrame(loop);
  window.addEventListener("click", onInput, { capture: true });
  window.addEventListener("keydown", onInput, { capture: true });
  observe("longtask", {}, (e) => {
    if (longTasks.length < MAX_ENTRIES) longTasks.push({ start: e.startTime, duration: e.duration });
  });
  observe("event", { durationThreshold: 16 }, (e) => {
    const et = e as PerformanceEventTiming;
    if (eventTimings.length < MAX_ENTRIES) {
      eventTimings.push({
        name: et.name,
        start: et.startTime,
        duration: et.duration,
        input_delay: et.processingStart - et.startTime,
        processing: et.processingEnd - et.processingStart,
      });
    }
  });
  (window as unknown as { __nonamePerf: unknown }).__nonamePerf = {
    version: 1,
    enabled: true,
    frames: () => frameTs.slice(),
    longTasks: () => longTasks.slice(),
    eventTimings: () => eventTimings.slice(),
    interactions: () => [...marks.values()].map(toInteractionRecord),
    summary: () => {
      const records = [...marks.values()].map(toInteractionRecord);
      return {
        frames: summarizeFrames(frameDeltas(frameTs)),
        interactions: summarizeInteractions(records),
        long_tasks: summarize(longTasks.map((t) => t.duration)),
        event_timings: summarize(eventTimings.map((t) => t.duration)),
      };
    },
    reset: () => {
      frameTs.length = 0;
      longTasks.length = 0;
      eventTimings.length = 0;
      marks.clear();
      orphanWorker.clear();
      lastReceived = null;
    },
  };
  if (enabled) startAutoDrive();
  return enabled;
}

export function perfProbeEnabled(): boolean {
  return enabled;
}

/** 请求发出（`postMessage` 前）。 */
export function perfRequestSent(id: number, type: string): void {
  if (!enabled || marks.size >= MAX_INTERACTIONS) return;
  const now = performance.now();
  const input =
    lastInput && !lastInput.used && now - lastInput.ts <= INPUT_LINK_MS ? lastInput : null;
  if (input) input.used = true;
  marks.set(id, {
    id,
    type,
    input_ts: input?.ts ?? null,
    input_kind: input?.kind ?? null,
    sent: now,
    received: null,
    received_epoch_ms: null,
    apply_start: null,
    apply_end: null,
    patched: null,
    frame: null,
    after_paint: null,
    worker: orphanWorker.get(id) ?? null,
  });
  orphanWorker.delete(id);
}

/** 未能发出的请求不计入性能样本。 */
export function perfRequestFailed(id: number): void {
  if (!enabled) return;
  marks.delete(id);
  orphanWorker.delete(id);
}

/** 回包到达主线程（`onmessage` 入口）。 */
export function perfResponseReceived(id: number): void {
  if (!enabled) return;
  const m = marks.get(id);
  if (!m) return;
  m.received = performance.now();
  m.received_epoch_ms = epochNow();
  lastReceived = id;
}

/** Worker 回传的分段（在回包之后到达；先到则暂存）。 */
export function perfWorkerTimings(id: number, timings: WorkerStepTimings): void {
  if (!enabled) return;
  const m = marks.get(id);
  if (m) m.worker = timings;
  else if (orphanWorker.size < MAX_ENTRIES) orphanWorker.set(id, timings);
}

function pendingApply(): InteractionMarks | null {
  if (lastReceived === null) return null;
  const m = marks.get(lastReceived);
  return m && m.apply_start === null ? m : null;
}

/** 开始把最近一次回包应用到视图状态。 */
export function perfEnvelopeApplying(): void {
  if (!enabled) return;
  const m = pendingApply();
  if (m) m.apply_start = performance.now();
}

/** 视图状态已同步更新；异步记录 DOM 刷新、下一帧与绘制后时刻（不阻塞业务路径）。 */
export function perfEnvelopeApplied(): void {
  if (!enabled || lastReceived === null) return;
  const m = marks.get(lastReceived);
  if (!m || m.apply_start === null || m.apply_end !== null) return;
  m.apply_end = performance.now();
  void nextTick(() => {
    m.patched = performance.now();
    requestAnimationFrame(() => {
      m.frame = performance.now();
      const channel = new MessageChannel();
      channel.port1.onmessage = () => {
        m.after_paint = performance.now();
        channel.port1.close();
      };
      channel.port2.postMessage(0);
    });
  });
}
