import type {
  ChooseGeneralPayload,
  EngineEnvelope,
  GeneralCandidatesPayload,
  ObserverSnapshot,
  ReplayFrame,
  ReplayMetadata,
  ReplayPrivacy,
  StatusPayload,
  WorkerRequest,
  WorkerResponse,
} from "../types/engine";
import { desktopContentBundle, isTauriShell } from "./desktopHost";
import {
  perfRequestSent,
  perfResponseReceived,
  perfWorkerTimings,
  startPerfProbe,
} from "./perfProbe";
import { urlParam } from "./autoDrive";
import type { WorkerStepTimings } from "./perfStats";
// Vite `?worker` 形式：保证生产构建产出 worker chunk（含 WASM 依赖）
import NativeEngineWorker from "../worker/engine.native.worker.ts?worker";
import LegacyEngineWorker from "../worker/engine.worker.ts?worker";

/** PA08-4 压测入口用：读取载荷文本（外链驱动静态服务）。 */
function fetchText(url: string): Promise<string> {
  return fetch(url).then((r) => {
    if (!r.ok) throw new Error(`压测载荷加载失败 ${url}: ${r.status}`);
    return r.text();
  });
}

/**
 * Main-thread client for the engine Worker.
 * Never runs rule logic here — only messaging + view store updates.
 *
 * T13 产品面：默认原生后端（VITE_ENGINE_BACKEND=legacy 回退旧线）。
 */
export class EngineClient {
  private worker: Worker | null = null;
  private seq = 1;
  private pending = new Map<
    number,
    { resolve: (v: unknown) => void; reject: (e: Error) => void }
  >();
  /** 原生引擎后端（默认 true；legacy 显式关闭）。 */
  private native = true;

  async connect(): Promise<void> {
    if (this.worker) return;
    // 默认 native；仅当 VITE_ENGINE_BACKEND=legacy 时走旧线 wasm。
    const backend = String(import.meta.env.VITE_ENGINE_BACKEND ?? "native").toLowerCase();
    this.native = backend !== "legacy" && backend !== "wasm" && backend !== "old";
    const bundle =
      !this.native && isTauriShell() ? await desktopContentBundle() : undefined;
    // Vite `?worker` 导入：构建期把 worker 及其依赖（含 native-pkg 的 WASM 导入）
    // 打成独立 chunk。此前的 `new Worker(new URL(...))` 在**生产构建**下未生成
    // worker chunk（dist 仅有 index.js），真实浏览器里 worker 加载失败 →
    // 控制台 `engine worker error Event`、战斗页「未连接」。见证据
    // docs/operations/evidence/2026-10-18-nc09-browser-e2e/。
    const worker: Worker = this.native
      ? new NativeEngineWorker()
      : new LegacyEngineWorker();
    this.worker = worker;
    worker.onmessage = (ev: MessageEvent<WorkerResponse>) => {
      // NC12 诊断钩子：worker 初始化分段耗时（fetch/instantiate）随 init 回包上报，
      // 供浏览器档位测量读取（`window.__nativeInitTimings`）。不参与任何业务逻辑。
      const maybe = ev.data as unknown as { type?: string; timings?: unknown };
      if (maybe?.type === "init_timings" && maybe.timings) {
        (globalThis as unknown as { __nativeInitTimings?: unknown[] }).__nativeInitTimings ??= [];
        (globalThis as unknown as { __nativeInitTimings: unknown[] }).__nativeInitTimings.push(
          maybe.timings,
        );
        return;
      }
      if (maybe?.type === "candidate_timings" && maybe.timings) {
        (globalThis as unknown as { __nativeCandidateTimings?: unknown[] })
          .__nativeCandidateTimings ??= [];
        (
          globalThis as unknown as { __nativeCandidateTimings: unknown[] }
        ).__nativeCandidateTimings.push(maybe.timings);
        return;
      }
      if (maybe?.type === "step_timings" && maybe.timings) {
        const step = ev.data as unknown as { id: number; timings: WorkerStepTimings };
        perfWorkerTimings(step.id, step.timings);
        return;
      }
      const msg = ev.data;
      const p = this.pending.get(msg.id);
      // SAB 提前应用后 pending 已删除——此时跳过计时与重复应用
      if (!p) return;
      perfResponseReceived(msg.id);
      this.pending.delete(msg.id);
      (globalThis as unknown as { __latencySabDebug?: { settle?: () => void } })
        .__latencySabDebug?.settle?.();
      if (msg.ok) p.resolve(msg.data);
      else p.reject(new Error(msg.error));
    };
    worker.onerror = (e) => {
      console.error("engine worker error", e);
    };
    try {
      await this.request({
        id: this.nextId(),
        type: "init",
        bundle: bundle ?? undefined,
        perf: startPerfProbe(),
        latencySab: this.initSabMirror(),
      });
    } catch (error) {
      this.dispose();
      throw error;
    }
  }

  // ---- PA07 缓解原型：回包 SAB 镜像 + rAF 轮询提前应用（latency=sab 门控） ----
  // Chromium 真实输入后把任务（含 worker postMessage 回包）推迟到下一帧合成后；
  // rAF 帧回调不受该窗口约束——在帧内轮询 SAB 并提前反序列化同一份信封。
  // 规则不迁移：只把 postMessage 会送的同一份数据提前送达。

  private sabPollSeq = -1;

  /** latency=sab 且跨源隔离可用时创建 SAB 并启动 rAF 轮询；否则返回 undefined。 */
  private initSabMirror(): SharedArrayBuffer | undefined {
    if (urlParam("latency") !== "sab") return undefined;
    if (!crossOriginIsolated || typeof SharedArrayBuffer === "undefined") return undefined;
    // rAF 只在有渲染机会时触发；空闲页（无动画）点击后没有新帧 → rAF 沉默，
    // 推迟的回包任务先到、轮询形同虚设（实测 seq 增长而 early=0）。请求在途
    // 期间挂一个主线程 keepalive 动画（布局属性，绕开纯合成器动画）保证每
    // vsync 都有 rAF；由点击处理器（不被推迟）启动。
    const style = document.createElement("style");
    style.textContent =
      ".sab-keepalive{animation:sab-pulse 8ms infinite}@keyframes sab-pulse{50%{margin-left:.5px}}";
    document.head.appendChild(style);
    const keepalive = document.createElement("div");
    keepalive.className = "sab-keepalive";
    keepalive.style.display = "none";
    document.body.appendChild(keepalive);
    const SAB_BYTES = 256 * 1024;
    const sab = new SharedArrayBuffer(SAB_BYTES);
    const ints = new Int32Array(sab, 0, 2);
    Atomics.store(ints, 0, 0);
    Atomics.store(ints, 1, 0);
    const decoder = new TextDecoder();
    const settle = () => {
      keepalive.style.display = "none";
    };
    const stats = { runs: 0, seen: 0, parseErr: 0, miss: 0 };
    const poll = () => {
      stats.runs += 1;
      const seq = Atomics.load(ints, 0);
      if (seq !== this.sabPollSeq) {
        stats.seen += 1;
        this.sabPollSeq = seq;
        const len = Atomics.load(ints, 1);
        if (len > 0 && 8 + len <= SAB_BYTES) {
          try {
            // TextDecoder 不接受 SharedArrayBuffer 视图——先拷贝到普通缓冲
            const copy = new Uint8Array(len);
            copy.set(new Uint8Array(sab, 8, len));
            const resp = JSON.parse(decoder.decode(copy)) as {
              id: number;
              ok: boolean;
              data?: unknown;
              error?: string;
            };
            const p = this.pending.get(resp.id);
            if (p) {
              perfResponseReceived(resp.id);
              this.pending.delete(resp.id);
              settle();
              const dbg = (
                globalThis as unknown as { __latencySabDebug?: { resolvedEarly: number } }
              ).__latencySabDebug;
              if (dbg) dbg.resolvedEarly += 1;
              if (resp.ok) p.resolve(resp.data);
              else p.reject(new Error(resp.error ?? "worker error"));
            } else {
              stats.miss += 1;
            }
          } catch {
            stats.parseErr += 1;
          }
        }
      }
      requestAnimationFrame(poll);
    };
    requestAnimationFrame(poll);
    // 诊断探针（latency=sab 门控内）：seq 计数与解析统计，供驱动 A/B 排障
    (globalThis as unknown as { __latencySabDebug?: unknown }).__latencySabDebug = {
      seq: () => Atomics.load(ints, 0),
      raw: () => {
        const len = Atomics.load(ints, 1);
        const copy = new Uint8Array(Math.min(len, 120));
        copy.set(new Uint8Array(sab, 8, copy.length));
        return { len, head: decoder.decode(copy) };
      },
      wake: () => {
        keepalive.style.display = "block";
      },
      settle: settle,
      stats: stats,
      resolvedEarly: 0,
      resolvedByMessage: 0,
    };
    return sab;
  }

  /** 当前是否使用原生窄壳后端。 */
  isNative(): boolean {
    return this.native;
  }

  dispose() {
    this.worker?.terminate();
    this.worker = null;
    this.pending.clear();
  }

  private request(body: WorkerRequest): Promise<unknown> {
    if (!this.worker) return Promise.reject(new Error("worker not connected"));
    return new Promise((resolve, reject) => {
      this.pending.set(body.id, { resolve, reject });
      perfRequestSent(body.id, body.type);
      // SAB 模式：请求在途即唤起 keepalive 动画（此处运行在输入处理器内，
      // 不受输入后任务推迟约束——保证回包前每个 vsync 都有 rAF 可轮询）。
      (globalThis as unknown as { __latencySabDebug?: { wake?: () => void } })
        .__latencySabDebug?.wake?.();
      this.worker!.postMessage(body);
    });
  }

  private nextId(): number {
    return this.seq++;
  }

  async start(
    players: number,
    seed: number,
    observer = 1,
    packsJson?: string,
    mode?: "identity",
  ): Promise<EngineEnvelope> {
    // PA08-4 压测入口（表现层采样族，URL 门控，缺省零影响）：
    // packsUrl 覆盖扩展载荷；hexUrl 以存档恢复开局（满手牌等大视图场景）。
    // 载荷由 scripts/web-frame-external.mjs --packs/--hex 提供，避免 URL 长度上限。
    const packsUrl = urlParam("packsUrl");
    const hexUrl = urlParam("hexUrl");
    const packs = packsUrl ? await fetchText(packsUrl) : packsJson;
    if (hexUrl) {
      // worker 的 restore 需要既有句柄与观察者：先建标准局，再以存档整体替换。
      await this.request({
        id: this.nextId(),
        type: "start",
        players,
        seed,
        observer,
        packsJson: packs,
        mode,
      });
      return this.request({
        id: this.nextId(),
        type: "restore",
        hex: await fetchText(hexUrl),
        packsJson: packs,
      }) as Promise<EngineEnvelope>;
    }
    return this.request({
      id: this.nextId(),
      type: "start",
      players,
      seed,
      observer,
      packsJson: packs,
      mode,
    }) as Promise<EngineEnvelope>;
  }

  endPhase(actor = 0): Promise<EngineEnvelope> {
    return this.request({
      id: this.nextId(),
      type: "endPhase",
      actor,
    }) as Promise<EngineEnvelope>;
  }

  playCard(actor: number, card: number, targets: number[]): Promise<EngineEnvelope> {
    return this.request({
      id: this.nextId(),
      type: "playCard",
      actor,
      card,
      targets,
    }) as Promise<EngineEnvelope>;
  }

  activateSkill(
    actor: number,
    skill: number,
    targets: number[],
    cardTargets: number[],
  ): Promise<EngineEnvelope> {
    return this.request({
      id: this.nextId(),
      type: "activateSkill",
      actor,
      skill,
      targets,
      cardTargets,
    }) as Promise<EngineEnvelope>;
  }

  useSkillCard(
    actor: number,
    skill: number,
    cardKind: number,
    targets: number[],
    equipSlot: number,
  ): Promise<EngineEnvelope> {
    return this.request({
      id: this.nextId(),
      type: "useSkillCard",
      actor,
      skill,
      cardKind,
      targets,
      equipSlot,
    }) as Promise<EngineEnvelope>;
  }

  resolveChoice(
    actor: number,
    choiceId: number,
    selections: number[],
    cancelled = false,
  ): Promise<EngineEnvelope> {
    return this.request({
      id: this.nextId(),
      type: "resolveChoice",
      actor,
      choiceId,
      selections,
      cancelled,
    }) as Promise<EngineEnvelope>;
  }

  autoStep(): Promise<EngineEnvelope> {
    return this.request({ id: this.nextId(), type: "autoStep" }) as Promise<EngineEnvelope>;
  }

  /** NC05 选将候选（原生目录全量；旧线后端会显式报 unknown message）。 */
  generalCandidates(): Promise<GeneralCandidatesPayload> {
    return this.request({ id: this.nextId(), type: "generalCandidates" }) as Promise<GeneralCandidatesPayload>;
  }

  /** NC05 正式选将（按名字装配到座位）；未知武将/越界由 worker 显式报错。 */
  chooseGeneral(seat: number, name: string): Promise<ChooseGeneralPayload> {
    return this.request({ id: this.nextId(), type: "chooseGeneral", seat, name }) as Promise<ChooseGeneralPayload>;
  }

  observer(): Promise<ObserverSnapshot> {
    if (!this.native) {
      return this.request({ id: this.nextId(), type: "observer" }) as Promise<ObserverSnapshot>;
    }
    // 原生路径：worker 已合成信封，取 view。
    return this.request({ id: this.nextId(), type: "observer" }).then((data) => {
      const env = data as EngineEnvelope;
      if (env?.view) return env.view;
      throw new Error("native observer: 缺 view");
    });
  }

  /** NGSA 十六进制存档（仅原生后端）。 */
  saveHex(): Promise<string> {
    return this.request({ id: this.nextId(), type: "save" }).then((data) => {
      const payload = data as { kind?: string; hex?: string } | string;
      if (typeof payload === "string") return payload;
      if (payload?.hex) return payload.hex;
      throw new Error("native save: 缺 hex");
    });
  }

  /** 从 NGSA 十六进制恢复。扩展局传入与开局相同的 packsJson。 */
  restoreHex(hex: string, packsJson?: string): Promise<EngineEnvelope> {
    return this.request({
      id: this.nextId(),
      type: "restore",
      hex,
      packsJson,
    }) as Promise<EngineEnvelope>;
  }

  loadReplay(json: string): Promise<ReplayMetadata> {
    return this.request({ id: this.nextId(), type: "loadReplay", json }) as Promise<ReplayMetadata>;
  }

  seekReplay(step: number, privacy: ReplayPrivacy, player = 1): Promise<ReplayFrame> {
    return this.request({
      id: this.nextId(),
      type: "seekReplay",
      step,
      privacy,
      player,
    }) as Promise<ReplayFrame>;
  }

  status(): Promise<StatusPayload> {
    return this.request({ id: this.nextId(), type: "status" }) as Promise<StatusPayload>;
  }
}
