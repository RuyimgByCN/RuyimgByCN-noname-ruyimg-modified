/**
 * T13 客户端原生引擎后端（engine-native-wasm 窄壳适配层）。
 *
 * 将 WorkerRequest 协议子集映射到 NativeEngine 窄 API，并合成 UI 可消费的
 * EngineEnvelope（view + legal_actions + revision/finished）。
 *
 * 扩展包装载面：窄壳尚无 `new_with_extensions`/`restore_with_extensions`，
 * 管理面状态由客户端 extensionStore 维护；开局装载需内核 API（见证据 README）。
 */
import type {
  EngineEnvelope,
  LegalAction,
  ObserverSnapshot,
} from "../types/engine";
import init, { NativeEngine } from "./native-pkg/engine_native_wasm.js";

export interface NativeHandle {
  engine: NativeEngine;
  seed: number;
  players: number;
  observer: number;
}

let ready: Promise<void> | null = null;

/** 浏览器/Worker 侧必须先初始化 wasm（web 目标）。 */
export async function ensureNativeReady(): Promise<void> {
  if (!ready) {
    ready = (async () => {
      await init({
        module_or_path: new URL(
          "./native-pkg/engine_native_wasm_bg.wasm",
          import.meta.url,
        ),
      });
    })();
  }
  await ready;
}

export function nativeCreate(
  players: number,
  seed: number,
  observer = 0,
): NativeHandle {
  return {
    engine: new NativeEngine(players, BigInt(seed)),
    seed,
    players,
    observer,
  };
}

interface NativeView {
  seat: number;
  players: Array<{
    seat: number;
    alive: boolean;
    hp: number;
    maxHp: number;
    hand: { own?: number[]; hidden?: boolean };
    handCount: number;
    equips: number[];
    judge: number[];
    turnedOver: boolean;
    skills: string[];
    revealedGenerals: boolean[];
  }>;
  deckCount: number;
  discard: number[];
  table: number[];
  turn: number;
  phase: string;
  round: number;
  outcome: unknown;
  rulesetVersion?: number;
  ruleset_version?: number;
}

const PHASE_MAP: Record<string, ObserverSnapshot["phase"]> = {
  judge: "Judge",
  draw: "Draw",
  play: "Play",
  discard: "Discard",
  end: "End",
};

/** observer schema 适配：view_json → ObserverSnapshot。 */
export function nativeObserverToSnapshot(view: NativeView): ObserverSnapshot {
  const self = view.players.find((p) => p.seat === view.seat);
  const ownHand = self?.hand?.own ?? [];
  const known_cards = [
    ...ownHand.map((id) => ({
      id,
      def: null,
      suit: null,
      rank: null,
      owner: view.seat,
      zone: "Hand" as const,
      kind: "Other" as const,
    })),
    ...(self?.equips ?? []).map((id) => ({
      id,
      def: null,
      suit: null,
      rank: null,
      owner: view.seat,
      zone: "Equip" as const,
      kind: "Other" as const,
    })),
  ];
  return {
    revision: view.rulesetVersion ?? view.ruleset_version ?? 0,
    state_hash: "",
    observer: view.seat,
    round: view.round,
    current_player: view.turn,
    phase: PHASE_MAP[view.phase] ?? "Idle",
    seats: view.players.map((p) => ({
      player: p.seat,
      character_def: 0,
      seat: p.seat,
      hp: p.hp,
      max_hp: p.maxHp,
      alive: p.alive,
      turned_over: !!p.turnedOver,
      hand_count: p.handCount,
      identity: null,
      skills: (p.skills ?? []).map((name) => ({
        id: 0,
        def: 0,
        enabled: true,
        use_count: 0,
        active: false,
        view_as: false,
        locked: false,
        name,
      })),
    })),
    known_cards,
    outcome: view.outcome ?? null,
  };
}

function optionValue(
  o: number | { card?: number; player?: number; tag?: number } | string,
): number | string {
  if (typeof o === "number" || typeof o === "string") return o;
  return o.card ?? o.player ?? o.tag ?? 0;
}

function deriveLegalActions(
  handle: NativeHandle,
  view: NativeView,
  finished: boolean,
): LegalAction[] {
  if (finished) return [];
  const actions: LegalAction[] = [];
  const phase = PHASE_MAP[view.phase] ?? "Idle";
  if (phase === "Play" || phase === "Discard" || phase === "End") {
    actions.push("EndPhase");
  }
  const self = view.players.find((p) => p.seat === handle.observer);
  const hand = self?.hand?.own ?? [];
  if (phase === "Play" && hand.length > 0) {
    for (const card of hand.slice(0, 8)) {
      actions.push({ PlayCard: { card, targets: [] } });
    }
  }
  try {
    const raw = handle.engine.open_choice_json(handle.observer);
    const choice = JSON.parse(raw) as {
      id: number;
      options: Array<number | { card: number } | { player: number } | { tag: number } | string>;
      min?: number;
      max?: number;
      allowCancel?: boolean;
    } | null;
    if (choice) {
      const first = choice.options[0];
      const sel =
        first === undefined ? [] : [Number(optionValue(first)) || 0];
      actions.push({
        ResolveChoice: {
          choice_id: Number(choice.id),
          selections: sel,
          cancelled: false,
        },
      });
      if (choice.allowCancel) {
        actions.push({
          ResolveChoice: {
            choice_id: Number(choice.id),
            selections: [],
            cancelled: true,
          },
        });
      }
    }
  } catch {
    /* 无开放选择 */
  }
  return actions;
}

function readChoice(handle: NativeHandle): EngineEnvelope["choice"] {
  try {
    const raw = handle.engine.open_choice_json(handle.observer);
    const choice = JSON.parse(raw) as {
      id: number;
      options: unknown[];
      min?: number;
      max?: number;
      allowCancel?: boolean;
      kind?: string;
    } | null;
    if (!choice) return null;
    return {
      choice_id: Number(choice.id),
      observer: handle.observer,
      min_count: choice.min,
      max_count: choice.max,
      allow_cancel: choice.allowCancel,
      kind: choice.kind ?? "Custom",
      options: (choice.options ?? []).map((o, i) => ({
        token: typeof o === "number" ? o : i,
        label_key: String(optionValue(o as never)),
      })),
    };
  } catch {
    return null;
  }
}

/** 合成 UI 信封：动作后刷新座位视图与合法动作。 */
export function nativeEnvelope(handle: NativeHandle): EngineEnvelope {
  const raw = handle.engine.view_json(handle.observer);
  const view = JSON.parse(raw) as NativeView;
  const finished = handle.engine.is_finished();
  const revision = Number(handle.engine.revision());
  const snap = nativeObserverToSnapshot(view);
  snap.revision = revision;
  return {
    result: null,
    events: [],
    choice: readChoice(handle),
    view: snap,
    legal_actions: deriveLegalActions(handle, view, finished),
    hash_hex: "",
    revision,
    finished,
  };
}

export type NativeDispatchReq =
  | { type: "endPhase"; actor?: number }
  | { type: "playCard"; actor: number; card: number; targets: number[] }
  | {
      type: "resolveChoice";
      actor: number;
      choiceId: number;
      selections: number[];
      cancelled?: boolean;
    }
  | { type: "autoStep" }
  | { type: "observer" }
  | { type: "save" }
  | { type: "restore"; hex: string }
  | { type: "status" };

/** 请求映射；返回信封或存档文本。 */
export function nativeDispatch(
  handle: NativeHandle,
  req: NativeDispatchReq,
): EngineEnvelope | { kind: "save"; hex: string } | { kind: "status"; revision: number; finished: boolean } {
  switch (req.type) {
    case "endPhase": {
      handle.engine.next_phase(req.actor ?? handle.observer);
      return nativeEnvelope(handle);
    }
    case "playCard": {
      handle.engine.play_card(
        req.actor,
        req.card,
        new Uint32Array(req.targets),
      );
      return nativeEnvelope(handle);
    }
    case "resolveChoice": {
      const choice = JSON.parse(
        handle.engine.open_choice_json(req.actor),
      ) as {
        id: number;
        options: Array<
          number | { card?: number; player?: number; tag?: number } | string
        >;
      } | null;
      if (!choice || Number(choice.id) !== req.choiceId) {
        throw new Error(`native: choice ${req.choiceId} 不匹配/不存在`);
      }
      if (req.cancelled) {
        // 窄壳无显式 cancel：传空选择索引；若不允许取消则由引擎拒
        handle.engine.answer(req.actor, BigInt(choice.id), new Uint32Array([]));
        return nativeEnvelope(handle);
      }
      const idxs = req.selections.map((v) => {
        const i = choice.options.findIndex((o) => optionValue(o) === v || o === v);
        if (i < 0) throw new Error(`native: 选项值 ${v} 不在候选`);
        return i;
      });
      handle.engine.answer(
        req.actor,
        BigInt(choice.id),
        new Uint32Array(idxs),
      );
      return nativeEnvelope(handle);
    }
    case "autoStep": {
      handle.engine.auto_step();
      return nativeEnvelope(handle);
    }
    case "observer":
      return nativeEnvelope(handle);
    case "save":
      return { kind: "save", hex: handle.engine.save_hex() };
    case "restore": {
      handle.engine.restore_hex(req.hex);
      return nativeEnvelope(handle);
    }
    case "status":
      return {
        kind: "status",
        revision: Number(handle.engine.revision()),
        finished: handle.engine.is_finished(),
      };
  }
}

/** 未映射请求统一报错面。 */
export function nativeUnsupported(type: string): never {
  throw new Error(`native backend: ${type} 未映射`);
}
