/**
 * T13 客户端原生引擎后端（批 11 engine-native-wasm 窄壳适配层）。
 *
 * 将 WorkerRequest 协议子集映射到 NativeEngine 窄 API：
 *   init(backend:"native") → new NativeEngine
 *   start                  → 构造 + view
 *   endPhase / playCard    → next_phase / play_card
 *   resolveChoice          → answer（选项索引直传）
 *   autoStep               → auto_step（BoundedPlanner 驱动）
 *   observer               → view_json（schema 差异登记，见 README）
 *
 * 未映射请求（activateSkill/useSkillCard/loadReplay/seek）返回
 * ok:false + "native backend: <type> 未映射"（批 15 首片子集面）。
 */
import { NativeEngine } from "./native-pkg/engine_native_wasm.js";

export interface NativeHandle {
  engine: NativeEngine;
  seed: number;
  players: number;
}

export function nativeCreate(players: number, seed: number): NativeHandle {
  return { engine: new NativeEngine(players, BigInt(seed)), seed, players };
}

export type NativeReplyData =
  | { kind: "status"; revision: number; finished: boolean }
  | { kind: "view"; view: unknown }
  | { kind: "auto"; payload: unknown }
  | { kind: "text"; text: string };

/** 请求映射（返回 null 表示该请求类型未映射）。 */
export function nativeDispatch(
  handle: NativeHandle,
  req:
    | { type: "endPhase"; actor?: number }
    | { type: "playCard"; actor: number; card: number; targets: number[] }
    | { type: "resolveChoice"; actor: number; choiceId: number; selections: number[] }
    | { type: "autoStep" }
    | { type: "observer" },
): NativeReplyData {
  const asStatus = (raw: string) => {
    const o = JSON.parse(raw) as { revision: number; finished: boolean };
    return { kind: "status", revision: o.revision, finished: o.finished } as const;
  };
  switch (req.type) {
    case "endPhase": {
      const actor = req.actor ?? 0;
      return asStatus(handle.engine.next_phase(actor));
    }
    case "playCard": {
      return asStatus(
        handle.engine.play_card(req.actor, req.card, new Uint32Array(req.targets)),
      );
    }
    case "resolveChoice": {
      // 旧协议 selections=选项值列表；窄壳 answer 按选项索引——由
      // open_choice_json 匹配值→索引（缺失即报错，不静默）。
      const choice = JSON.parse(handle.engine.open_choice_json(req.actor)) as {
        id: number;
        options: Array<number | { card: number } | { player: number } | { tag: number } | string>;
      } | null;
      if (!choice || choice.id !== req.choiceId) {
        throw new Error(`native: choice ${req.choiceId} 不匹配/不存在`);
      }
      const idxs = req.selections.map((v) => {
        const i = choice.options.findIndex((o) => {
          const ov = typeof o === "number" ? o : (o as { card?: number; player?: number; tag?: number }).card
            ?? (o as { player?: number }).player ?? (o as { tag?: number }).tag;
          return ov === v || o === v;
        });
        if (i < 0) throw new Error(`native: 选项值 ${v} 不在候选`);
        return i;
      });
      return asStatus(
        handle.engine.answer(req.actor, BigInt(choice.id), new Uint32Array(idxs)),
      );
    }
    case "autoStep": {
      return { kind: "auto", payload: JSON.parse(handle.engine.auto_step()) };
    }
    case "observer": {
      const view = JSON.parse(handle.engine.view_json(0));
      return { kind: "view", view };
    }
  }
}

/** observer schema 适配首层（client.ts native 路径同源逻辑）：view → ObserverSnapshot 核心。 */
export function nativeObserverToSnapshot(view: {
  seat: number; round: number; turn: number; phase: string;
  rulesetVersion?: number;
  ruleset_version?: number;
  outcome: unknown;
  players: Array<{ seat: number; alive: boolean; hp: number; maxHp: number; handCount: number }>,
}) {
  const phaseMap: Record<string, string> = {
    judge: "Judge", draw: "Draw", play: "Play", discard: "Discard", end: "End",
  };
  return {
    revision: view.rulesetVersion ?? view.ruleset_version,
    state_hash: "",
    observer: view.seat,
    round: view.round,
    current_player: view.turn,
    phase: phaseMap[view.phase] ?? "Idle",
    seats: view.players.map((p) => ({
      player: p.seat, character_def: 0, seat: p.seat, hp: p.hp,
      max_hp: p.maxHp, alive: p.alive, turned_over: false,
      hand_count: p.handCount, identity: null, skills: [],
    })),
    known_cards: [],
    outcome: view.outcome ?? null,
  };
}

/** 未映射请求统一报错面（批 15 首片子集边界）。 */
export function nativeUnsupported(type: string): never {
  throw new Error(`native backend: ${type} 未映射`);
}
