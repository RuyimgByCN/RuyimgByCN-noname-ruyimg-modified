/**
 * 原生引擎 Worker 入口（T13 产品面）：协议对齐旧线 WorkerRequest 子集，
 * 引擎后端为 engine-native-wasm 窄壳；默认产品路径。
 */
import type { WorkerRequest, WorkerResponse } from "../types/engine";
import {
  ensureNativeReady,
  nativeCreate,
  nativeCreateWithPacks,
  nativeDispatch,
  nativeEnvelope,
  nativeUnsupported,
  type NativeHandle,
} from "./nativeBackend";

let handle: NativeHandle | null = null;

self.onmessage = async (ev: MessageEvent<WorkerRequest>) => {
  const msg = ev.data;
  const reply = (resp: WorkerResponse) => {
    (self as DedicatedWorkerGlobalScope).postMessage(resp);
  };
  try {
    await ensureNativeReady();
    if (msg.type === "init") {
      handle = msg.packsJson
        ? nativeCreateWithPacks(4, 4242, 0, msg.packsJson)
        : nativeCreate(4, 4242, 0);
      reply({ id: msg.id, ok: true, data: "native ready" });
      return;
    }
    if (!handle) throw new Error("native engine not initialized");

    switch (msg.type) {
      case "start": {
        handle = msg.packsJson
          ? nativeCreateWithPacks(msg.players, msg.seed, msg.observer, msg.packsJson)
          : nativeCreate(msg.players, msg.seed, msg.observer);
        reply({ id: msg.id, ok: true, data: nativeEnvelope(handle) });
        return;
      }
      case "endPhase":
      case "playCard":
      case "resolveChoice":
      case "autoStep":
      case "observer":
      case "status": {
        const data = nativeDispatch(handle, msg as Parameters<typeof nativeDispatch>[1]);
        reply({ id: msg.id, ok: true, data: data as never });
        return;
      }
      case "save": {
        const data = nativeDispatch(handle, { type: "save" });
        reply({ id: msg.id, ok: true, data: data as never });
        return;
      }
      case "restore": {
        const data = nativeDispatch(handle, {
          type: "restore",
          hex: msg.hex,
          packsJson: msg.packsJson,
        });
        reply({ id: msg.id, ok: true, data: data as never });
        return;
      }
      case "activateSkill": {
        // NC03：主动技候选来自内核权威查询；此处只把技能 id 交给内核，
        // 目标/选牌由内核的开放选择应答流程决定（见 nativeDispatch）。
        const data = nativeDispatch(handle, {
          type: "activateSkill",
          actor: msg.actor,
          skill: msg.skill,
          targets: msg.targets,
          cardTargets: msg.cardTargets,
        });
        reply({ id: msg.id, ok: true, data: data as never });
        return;
      }
      case "useSkillCard":
      case "loadReplay":
      case "seekReplay":
        nativeUnsupported(msg.type);
        return;
      default:
        nativeUnsupported((msg as { type: string }).type);
    }
  } catch (error) {
    reply({ id: msg.id, ok: false, error: String(error) });
  }
};
