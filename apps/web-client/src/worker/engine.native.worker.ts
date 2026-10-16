/**
 * 原生引擎 Worker 入口（T13 产品面）：协议对齐旧线 WorkerRequest 子集，
 * 引擎后端为 engine-native-wasm 窄壳；默认产品路径。
 */
import type { WorkerRequest, WorkerResponse } from "../types/engine";
import {
  ensureNativeReady,
  nativeCreate,
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
      // 标准目录内建；扩展包装载待窄壳 new_with_extensions
      handle = nativeCreate(4, 4242, 0);
      reply({ id: msg.id, ok: true, data: "native ready" });
      return;
    }
    if (!handle) throw new Error("native engine not initialized");

    switch (msg.type) {
      case "start": {
        handle = nativeCreate(msg.players, msg.seed, msg.observer);
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
        const data = nativeDispatch(handle, { type: "restore", hex: msg.hex });
        reply({ id: msg.id, ok: true, data: data as never });
        return;
      }
      case "activateSkill":
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
