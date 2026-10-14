import type {
  EngineEnvelope,
  ObserverSnapshot,
  ReplayFrame,
  ReplayMetadata,
  ReplayPrivacy,
  StatusPayload,
  WorkerRequest,
  WorkerResponse,
} from "../types/engine";
import { desktopContentBundle, isTauriShell } from "./desktopHost";

/**
 * Main-thread client for the engine Worker.
 * Never runs rule logic here — only messaging + view store updates.
 */
export class EngineClient {
  private worker: Worker | null = null;
  private seq = 1;
  private pending = new Map<
    number,
    { resolve: (v: unknown) => void; reject: (e: Error) => void }
  >();
  /** 原生引擎后端（VITE_ENGINE_BACKEND=native）。 */
  private native = false;

  async connect(): Promise<void> {
    if (this.worker) return;
    // T13 worker 切换第二片：env 分流原生引擎后端（VITE_ENGINE_BACKEND=native）。
    // native 模式：标准目录内建（bundle 忽略），observer 走 schema 适配首层。
    this.native = import.meta.env.VITE_ENGINE_BACKEND === "native";
    const bundle = !this.native && isTauriShell()
      ? await desktopContentBundle()
      : undefined;
    this.worker = new Worker(
      new URL(
        this.native ? "../worker/engine.native.worker.ts" : "../worker/engine.worker.ts",
        import.meta.url,
      ),
      { type: "module" },
    );
    this.worker.onmessage = (ev: MessageEvent<WorkerResponse>) => {
      const msg = ev.data;
      const p = this.pending.get(msg.id);
      if (!p) return;
      this.pending.delete(msg.id);
      if (msg.ok) p.resolve(msg.data);
      else p.reject(new Error(msg.error));
    };
    this.worker.onerror = (e) => {
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

  start(players: number, seed: number, observer = 1): Promise<EngineEnvelope> {
    return this.request({
      id: this.nextId(),
      type: "start",
      players,
      seed,
      observer,
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
    cardTargets: number[]
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
    equipSlot: number
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
    cancelled = false
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

  observer(): Promise<ObserverSnapshot> {
    if (!this.native) {
      return this.request({ id: this.nextId(), type: "observer" }) as Promise<ObserverSnapshot>;
    }
    // T13 切换第二片·observer schema 适配首层：native view_json →
    // ObserverSnapshot 核心字段（seats 深字段 identity/skills 置空、
    // known_cards 暂缺——登记差异，后续批补全）。
    return this.request({ id: this.nextId(), type: "observer" }).then((data) => {
      const payload = data as { kind: string; view?: NativeView };
      const view = payload.view;
      if (!view) throw new Error("native observer: 缺 view");
      // 适配首层与 nativeBackend.nativeObserverToSnapshot 同源逻辑
      const phaseMap: Record<string, ObserverSnapshot["phase"]> = {
        judge: "Judge", draw: "Draw", play: "Play",
        discard: "Discard", end: "End",
      };
      return {
        revision: view.rulesetVersion ?? view.ruleset_version,
        state_hash: "",
        observer: view.seat,
        round: view.round,
        current_player: view.turn,
        phase: phaseMap[view.phase] ?? "Idle",
        seats: view.players.map((p: NativeView["players"][number]) => ({
          player: p.seat,
          character_def: 0,
          seat: p.seat,
          hp: p.hp,
          max_hp: p.maxHp,
          alive: p.alive,
          turned_over: false,
          hand_count: p.handCount,
          identity: null,
          skills: [],
        })),
        known_cards: [],
        outcome: view.outcome ?? null,
      } satisfies ObserverSnapshot;
    });
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

/** native view_json 投影形状（engine-native-wasm 窄壳；结构见窄壳 README）。 */
interface NativeView {
  seat: number;
  players: Array<{
    seat: number; alive: boolean; hp: number; maxHp: number;
    hand: { own?: number[]; hidden?: boolean }; handCount: number;
    equips: number[]; judge: number[]; revealedGenerals: boolean[];
  }>;
  deckCount: number;
  discard: number[];
  table: number[];
  turn: number;
  phase: string;
  round: number;
  outcome: unknown;
  rulesetVersion?: number;
  ruleset_version: number;
}
