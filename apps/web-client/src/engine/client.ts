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
// Vite `?worker` 形式：保证生产构建产出 worker chunk（含 WASM 依赖）
import NativeEngineWorker from "../worker/engine.native.worker.ts?worker";
import LegacyEngineWorker from "../worker/engine.worker.ts?worker";

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
      const msg = ev.data;
      const p = this.pending.get(msg.id);
      if (!p) return;
      this.pending.delete(msg.id);
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
      });
    } catch (error) {
      this.dispose();
      throw error;
    }
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
      this.worker!.postMessage(body);
    });
  }

  private nextId(): number {
    return this.seq++;
  }

  start(
    players: number,
    seed: number,
    observer = 1,
    packsJson?: string,
    mode?: "identity",
  ): Promise<EngineEnvelope> {
    return this.request({
      id: this.nextId(),
      type: "start",
      players,
      seed,
      observer,
      packsJson,
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
