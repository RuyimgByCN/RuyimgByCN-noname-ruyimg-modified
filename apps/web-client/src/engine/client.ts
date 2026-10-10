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

  async connect(): Promise<void> {
    if (this.worker) return;
    const bundle = isTauriShell()
      ? await desktopContentBundle()
      : undefined;
    this.worker = new Worker(new URL("../worker/engine.worker.ts", import.meta.url), {
      type: "module",
    });
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
    return this.request({ id: this.nextId(), type: "observer" }) as Promise<ObserverSnapshot>;
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
