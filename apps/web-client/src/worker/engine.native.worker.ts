/**
 * 原生引擎 Worker 入口（T13 产品面）：协议对齐旧线 WorkerRequest 子集，
 * 引擎后端为 engine-native-wasm 窄壳；默认产品路径。
 */
import type { WorkerRequest, WorkerResponse } from "../types/engine";
import {
  nativeInitTimings,
  chooseGeneral,
  ensureNativeReady,
  generalCandidates,
  lastCandidateTimings,
  nativeCreate,
  nativeCreateIdentity,
  nativeCreateWithPacks,
  nativeDispatch,
  nativeEnvelope,
  nativeUnsupported,
  type NativeHandle,
} from "./nativeBackend";
import { postWithStepTimings, setSabMirror, setStepTimingsEnabled, stepBegin } from "./stepTimings";

let handle: NativeHandle | null = null;

self.onmessage = async (ev: MessageEvent<WorkerRequest>) => {
  const msg = ev.data;
  // 表现层性能采样：开启时回包后追加分段（见 stepTimings.ts）；关闭时即原样 postMessage。
  const reply = (resp: WorkerResponse) => {
    postWithStepTimings((m) => (self as DedicatedWorkerGlobalScope).postMessage(m), resp);
  };
  try {
    await ensureNativeReady();
    if (msg.type === "init") {
      setStepTimingsEnabled(msg.perf === true);
      setSabMirror(msg.latencySab ?? null);
    }
    stepBegin();
    if (msg.type === "init") {
      // NC12：把 worker 内初始化分段耗时随 init 回包发出（浏览器档位测量读取）
      const timings = nativeInitTimings();
      if (timings) {
        reply({ type: "init_timings", timings } as unknown as WorkerResponse);
      }
      handle = msg.packsJson
        ? nativeCreateWithPacks(4, 4242, 0, msg.packsJson)
        : nativeCreate(4, 4242, 0);
      reply({ id: msg.id, ok: true, data: "native ready" });
      return;
    }
    if (!handle) throw new Error("native engine not initialized");

    switch (msg.type) {
      case "start": {
        // mode: "identity" → 身份局（NC09 身份局端到端）；缺省为原型模式
        handle =
          msg.mode === "identity"
            ? nativeCreateIdentity(msg.players, msg.seed, msg.observer)
            : msg.packsJson
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
      case "generalCandidates": {
        // NC05：候选来自内核目录（含扩展武将）；畸形返回在适配层即抛错。
        const data = generalCandidates(handle);
        // NC12：候选生成分段耗时随回包发出（页面侧钩子记录；不参与业务逻辑）
        if (lastCandidateTimings) {
          reply({
            type: "candidate_timings",
            timings: lastCandidateTimings,
          } as unknown as WorkerResponse);
        }
        reply({ id: msg.id, ok: true, data });
        return;
      }
      case "chooseGeneral": {
        // NC05：正式选将（按名字）；失败带原始错误文本上抛，不静默。
        // 成功回包带刷新后的信封：该座位技能集变化对界面可见。
        const assignment = chooseGeneral(handle, msg.seat, msg.name);
        reply({
          id: msg.id,
          ok: true,
          data: { assignment, envelope: nativeEnvelope(handle) },
        });
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
