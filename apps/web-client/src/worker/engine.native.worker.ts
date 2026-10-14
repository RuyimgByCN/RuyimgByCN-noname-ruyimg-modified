/**
 * 原生引擎 Worker 入口（T13 客户端切换首片）：与 engine.worker.ts 协议
 * 兼容的子集面，引擎后端为 engine-native-wasm 窄壳。客户端按环境分流
 * （env 标志选 worker 文件）；observer schema 与旧线不同（登记差异，
 * UI 层适配归后续批）。
 */
import type {
  WorkerRequest, WorkerResponse, ObserverSnapshot, EngineEnvelope,
  ReplayFrame, StatusPayload,
} from "../types/engine";
import { nativeCreate, nativeDispatch, type NativeHandle } from "./nativeBackend";

let handle: NativeHandle | null = null;

self.onmessage = async (ev: MessageEvent<WorkerRequest>) => {
  const msg = ev.data;
  const reply = (resp: WorkerResponse) => {
    (self as DedicatedWorkerGlobalScope).postMessage(resp);
  };
  try {
    if (msg.type === "init") {
      // 标准目录内建（扩展包装载面归后续批）
      handle = nativeCreate(4, 4242);
      reply({ id: msg.id, ok: true, data: "native ready" });
      return;
    }
    if (!handle) throw new Error("native engine not initialized");
    if (msg.type === "start") {
      handle = nativeCreate(msg.players, msg.seed);
      reply({
        id: msg.id,
        ok: true,
        data: { revision: 0, finished: false, hash: "", choice: "native start" },
      });
      return;
    }
    const data = nativeDispatch(
      handle,
      msg as Parameters<typeof nativeDispatch>[1],
    );
    const payload: string | ObserverSnapshot | EngineEnvelope | ReplayFrame | StatusPayload =
      data.kind === "status"
        ? { revision: data.revision, finished: data.finished, hash: "", choice: "native" }
        : data.kind === "view"
          ? (data.view as ObserverSnapshot)
          : data.kind === "auto"
            ? (data.payload as EngineEnvelope)
            : (data.text as unknown as StatusPayload);
    reply({ id: msg.id, ok: true, data: payload });
  } catch (error) {
    reply({ id: msg.id, ok: false, error: String(error) });
  }
};
