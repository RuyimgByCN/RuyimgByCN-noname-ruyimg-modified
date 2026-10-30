/**
 * Engine Web Worker — loads WASM and runs all rule logic off the main thread.
 */
import type { WorkerRequest, WorkerResponse } from "../types/engine";

// Built by `pnpm build:wasm` into src/wasm-pkg
import init, { WasmEngine } from "../wasm-pkg/engine_wasm.js";

type ContentAwareEngine = InstanceType<typeof WasmEngine> & {
  loadContentBundle(json: string): void;
  loadEmbeddedStandard(): void;
};

let engine: ContentAwareEngine | null = null;
let ready: Promise<void> | null = null;

async function ensureReady() {
  if (!ready) {
    ready = (async () => {
      await init({ module_or_path: new URL("../wasm-pkg/engine_wasm_bg.wasm", import.meta.url) });
    })();
  }
  await ready;
}

function initializeEngine(bundle: Extract<WorkerRequest, { type: "init" }>["bundle"]) {
  const candidate = new WasmEngine() as ContentAwareEngine;
  try {
    if (bundle) candidate.loadContentBundle(JSON.stringify(bundle));
    else candidate.loadEmbeddedStandard();
  } catch (error) {
    candidate.free();
    throw error;
  }
  engine?.free();
  engine = candidate;
}

function parseJson(json: string) {
  return JSON.parse(json);
}

self.onmessage = async (ev: MessageEvent<WorkerRequest>) => {
  const msg = ev.data;
  const reply = (resp: WorkerResponse) => {
    (self as DedicatedWorkerGlobalScope).postMessage(resp);
  };

  try {
    await ensureReady();
    if (msg.type === "init") {
      initializeEngine(msg.bundle);
      reply({ id: msg.id, ok: true, data: "ready" });
      return;
    }
    if (!engine) throw new Error("engine not initialized");

    switch (msg.type) {
      case "start": {
        engine.setObserverPlayer(msg.observer);
        const out = engine.startGame(msg.players, msg.seed);
        reply({ id: msg.id, ok: true, data: parseJson(out) });
        break;
      }
      case "endPhase": {
        const out = engine.endPhase(msg.actor ?? 0);
        reply({ id: msg.id, ok: true, data: parseJson(out) });
        break;
      }
      case "playCard": {
        const out = engine.playCard(
          msg.actor,
          msg.card,
          JSON.stringify(msg.targets)
        );
        reply({ id: msg.id, ok: true, data: parseJson(out) });
        break;
      }
      case "activateSkill": {
        const out = engine.activateSkill(
          msg.actor,
          msg.skill,
          JSON.stringify(msg.targets),
          JSON.stringify(msg.cardTargets)
        );
        reply({ id: msg.id, ok: true, data: parseJson(out) });
        break;
      }
      case "useSkillCard": {
        const out = engine.useSkillCard(
          msg.actor,
          msg.skill,
          msg.cardKind,
          JSON.stringify(msg.targets),
          msg.equipSlot
        );
        reply({ id: msg.id, ok: true, data: parseJson(out) });
        break;
      }
      case "resolveChoice": {
        const out = engine.resolveChoice(
          msg.actor,
          msg.choiceId,
          JSON.stringify(msg.selections),
          msg.cancelled
        );
        reply({ id: msg.id, ok: true, data: parseJson(out) });
        break;
      }
      case "autoStep": {
        const out = engine.autoStep();
        reply({ id: msg.id, ok: true, data: parseJson(out) });
        break;
      }
      case "observer": {
        const out = engine.observerJson();
        reply({ id: msg.id, ok: true, data: parseJson(out) });
        break;
      }
      case "loadReplay": {
        const out = engine.loadReplay(msg.json);
        reply({ id: msg.id, ok: true, data: parseJson(out) });
        break;
      }
      case "seekReplay": {
        const out = engine.seekReplay(msg.step, msg.privacy, msg.player);
        reply({ id: msg.id, ok: true, data: parseJson(out) });
        break;
      }
      case "status": {
        reply({
          id: msg.id,
          ok: true,
          data: {
            revision: Number(engine.revision()),
            finished: engine.isFinished(),
            hash: engine.stateHashHex(),
            choice: engine.pendingChoiceJson(),
          },
        });
        break;
      }
      default:
        reply({
          id: (msg as { id: number }).id,
          ok: false,
          error: "unknown message",
        });
    }
  } catch (e) {
    reply({
      id: msg.id,
      ok: false,
      error: e instanceof Error ? e.message : String(e),
    });
  }
};
