/**
 * T13 客户端原生引擎后端（engine-native-wasm 窄壳适配层）。
 *
 * 将 WorkerRequest 协议子集映射到 NativeEngine 窄 API，并合成 UI 可消费的
 * EngineEnvelope（view + legal_actions + revision/finished）。
 *
 * 扩展包装载：`NativeEngine.new_with_packs_json` /
 * `restore_hex_with_packs_json` 接收 cards.json 对象数组。空数组等于标准目录。
 * 管理面元数据仍由 extensionStore 保存；开局把包 JSON 经 packsJson 传入。
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

/** 已安装扩展的 cards.json 数组建局。空数组与 nativeCreate 相同。 */
export function nativeCreateWithPacks(
  players: number,
  seed: number,
  observer: number,
  packsJson: string,
): NativeHandle {
  return {
    engine: NativeEngine.new_with_packs_json(players, BigInt(seed), packsJson),
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

/**
 * 选择候选 JSON，与 engine-native-wasm `open_choice_json` 同口径：
 * `"confirm"`、`"pass"`、`{card}`、`{player}`、`{tag}`、`{kind}`。
 */
export type ChoiceOptionJson = number | string | ChoiceOptionPayload;

/** 载荷型候选（每项恰有一个键）。 */
export interface ChoiceOptionPayload {
  card?: number;
  player?: number;
  tag?: number;
  kind?: number;
}

/** observer 本人可见的开放选择 JSON（他人视图只有 `{id, observer, hidden}`）。 */
interface NativeChoiceJson {
  id: number;
  prompt?: string;
  options?: ChoiceOptionJson[];
  min?: number;
  max?: number;
  allowCancel?: boolean;
  hidden?: boolean;
  kind?: string;
}

/** 窄壳命令结果，与 engine-native-wasm `outcome_json` 同口径。 */
export interface NativeCommandOutcome {
  outcome: "accepted" | "pending" | "rejected" | "suspended";
  revision: number;
  finished: boolean;
}

/**
 * 候选的**值**（卡牌 id／座位／tag／kind 载荷）。
 *
 * 值只用于展示与标注，**不得当作回传 token**：窄壳 `answer()` 按选项下标直取
 * `spec.options[idx]`（`crates/engine-native-wasm/src/lib.rs`）。把值当 token
 * 会在 `{card: 42}` 一类载荷候选上失配（提交值 → 找不到下标）。
 */
export function optionValue(o: ChoiceOptionJson): number | string {
  if (typeof o === "number" || typeof o === "string") return o;
  return o.card ?? o.player ?? o.tag ?? o.kind ?? 0;
}

/**
 * 候选展示标注：带上字段名，避免 `Kind(105)` 与裸卡牌值都退化成同一个数字
 * （旧实现 `String(optionValue(o))` 对 `{kind: 105}` 给出 `"0"`）。
 */
export function optionLabel(o: ChoiceOptionJson): string {
  if (typeof o === "string") return o;
  if (typeof o === "number") return String(o);
  if (o.card !== undefined) return `card:${o.card}`;
  if (o.player !== undefined) return `player:${o.player}`;
  if (o.tag !== undefined) return `tag:${o.tag}`;
  if (o.kind !== undefined) return `kind:${o.kind}`;
  return "unknown";
}

/** 读取当前开放选择；异常（无选择/窄壳报错）按 null 处理。 */
function readChoiceJson(
  handle: NativeHandle,
  seat: number = handle.observer,
): NativeChoiceJson | null {
  try {
    const raw = handle.engine.open_choice_json(seat);
    return (JSON.parse(raw) as NativeChoiceJson | null) ?? null;
  } catch {
    return null;
  }
}

/** 解析窄壳命令返回（`outcome_json` 或 autoStep 步骤摘要）；非 JSON 返回 null。 */
export function parseCommandResult(raw: unknown): unknown {
  if (typeof raw !== "string") return null;
  try {
    return JSON.parse(raw) as unknown;
  } catch {
    return null;
  }
}

/**
 * 取命令结果里的窄壳判决（`accepted`/`pending`/`rejected`/`suspended`）。
 *
 * 旧适配层把所有命令返回值丢弃、信封 `result` 恒为 null，引擎的 `rejected`
 * 因此对界面不可见；回传必须原样放进 `EngineEnvelope.result`。
 */
export function commandOutcome(
  result: unknown,
): NativeCommandOutcome["outcome"] | null {
  if (result === null || typeof result !== "object") return null;
  const outcome = (result as { outcome?: unknown }).outcome;
  return typeof outcome === "string"
    ? (outcome as NativeCommandOutcome["outcome"])
    : null;
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
    const choice = readChoiceJson(handle);
    if (choice) {
      const options = choice.options ?? [];
      actions.push({
        ResolveChoice: {
          choice_id: Number(choice.id),
          // token 即选项下标：0 号候选（如 {card:42}）以 token 0 回传。
          selections: options.length > 0 ? [0] : [],
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
  const choice = readChoiceJson(handle);
  if (!choice) return null;
  const options = choice.options ?? [];
  return {
    choice_id: Number(choice.id),
    observer: handle.observer,
    min_count: choice.min,
    max_count: choice.max,
    allow_cancel: choice.allowCancel,
    kind: choice.kind ?? "Custom",
    // token = 候选下标（回传即 `answer()` 的索引）；候选值只进 label_key。
    options: options.map((o, index) => ({
      token: index,
      label_key: optionLabel(o),
    })),
  };
}

/** 合成 UI 信封：动作后刷新座位视图与合法动作。`result` 为窄壳命令结果。 */
export function nativeEnvelope(
  handle: NativeHandle,
  result: unknown = null,
): EngineEnvelope {
  const raw = handle.engine.view_json(handle.observer);
  const view = JSON.parse(raw) as NativeView;
  const finished = handle.engine.is_finished();
  const revision = Number(handle.engine.revision());
  const snap = nativeObserverToSnapshot(view);
  snap.revision = revision;
  return {
    result,
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
  | { type: "restore"; hex: string; packsJson?: string }
  | { type: "status" };

/** 请求映射；返回信封或存档文本。 */
export function nativeDispatch(
  handle: NativeHandle,
  req: NativeDispatchReq,
): EngineEnvelope | { kind: "save"; hex: string } | { kind: "status"; revision: number; finished: boolean } {
  switch (req.type) {
    case "endPhase": {
      const out = handle.engine.next_phase(req.actor ?? handle.observer);
      return nativeEnvelope(handle, parseCommandResult(out));
    }
    case "playCard": {
      const out = handle.engine.play_card(
        req.actor,
        req.card,
        new Uint32Array(req.targets),
      );
      return nativeEnvelope(handle, parseCommandResult(out));
    }
    case "resolveChoice": {
      const choice = readChoiceJson(handle, req.actor);
      if (!choice || Number(choice.id) !== req.choiceId) {
        throw new Error(`native: choice ${req.choiceId} 不匹配/不存在`);
      }
      const options = choice.options ?? [];
      if (req.cancelled) {
        // 窄壳无显式 cancel：传空选择索引；若不允许取消则由引擎拒
        const out = handle.engine.answer(
          req.actor,
          BigInt(choice.id),
          new Uint32Array([]),
        );
        return nativeEnvelope(handle, parseCommandResult(out));
      }
      // selections 是 token（= 候选下标），不是候选值：值只用于选择，
      // 回传必须能按下标回查候选，否则 `{card:42}` 一类载荷永远失配。
      const idxs = req.selections.map((token) => {
        if (!Number.isInteger(token) || token < 0 || token >= options.length) {
          throw new Error(
            `native: 选项 token ${token} 不在候选（0..${options.length - 1}）`,
          );
        }
        return token;
      });
      const out = handle.engine.answer(
        req.actor,
        BigInt(choice.id),
        new Uint32Array(idxs),
      );
      return nativeEnvelope(handle, parseCommandResult(out));
    }
    case "autoStep": {
      const out = handle.engine.auto_step();
      return nativeEnvelope(handle, parseCommandResult(out));
    }
    case "observer":
      return nativeEnvelope(handle);
    case "save":
      return { kind: "save", hex: handle.engine.save_hex() };
    case "restore": {
      if (req.packsJson !== undefined) {
        handle.engine.restore_hex_with_packs_json(req.hex, req.packsJson);
      } else {
        handle.engine.restore_hex(req.hex);
      }
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
