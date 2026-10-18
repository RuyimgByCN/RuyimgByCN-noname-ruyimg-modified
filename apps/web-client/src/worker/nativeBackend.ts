/**
 * T13 客户端原生引擎后端（engine-native-wasm 窄壳适配层）。
 *
 * 将 WorkerRequest 协议子集映射到 NativeEngine 窄 API，并合成 UI 可消费的
 * EngineEnvelope（view + legal_actions + revision/finished）。
 *
 * **合法动作来自内核权威查询**（NC03）：`legal_actions` 由窄壳
 * `legal_actions_json` 导出（内核 `Engine::legal_actions`，经与正式提交相同的
 * 校验产出）。适配层不再按 `hand.slice(0, 8)` 拼接 `PlayCard`，也不再合成
 * `ResolveChoice`——候选的合法性定义只有内核一处。
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
  /** 被拒原因（内核 `RejectReason` 的调试名）；非 rejected 为 null。 */
  reason?: string | null;
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

/**
 * 权威查询导出面（`crates/engine-native-wasm/src/lib.rs`）。
 *
 * 兼容旧 wasm 产物：方法缺失时**显式报错并给出重建命令**，而不是静默回落到
 * 手牌拼接——回落会让「合法动作」又出现第二处定义，正是 NC03 要消除的缺陷。
 */
interface NativeAuthoritativeExports {
  legal_actions_json: (seat: number) => string;
  activate_skill: (actor: number, skill: number) => string;
}

/** 权威动作信封（`legal_actions_json` 返回面，与 TS `LegalAction` 同形）。 */
export interface NativeLegalActionsJson {
  seat: number;
  revision: number;
  actions: LegalAction[];
}

function requireAuthoritativeExports(
  handle: NativeHandle,
): NativeAuthoritativeExports {
  const engine = handle.engine as unknown as Partial<NativeAuthoritativeExports>;
  const missing = (["legal_actions_json", "activate_skill"] as const).filter(
    (name) => typeof engine[name] !== "function",
  );
  if (missing.length > 0) {
    throw new Error(
      `native: wasm 产物缺少 ${missing.join("/")}（NC03 权威查询导出）——` +
        "执行 pnpm --dir apps/web-client build:native 重建 native-pkg；不回落手牌拼接",
    );
  }
  // 返回引擎实例本身：这些是类方法，解构/绑定不当会丢 `this`。
  return engine as NativeAuthoritativeExports;
}

function isNumberArray(value: unknown): value is number[] {
  return Array.isArray(value) && value.every((n) => Number.isInteger(n));
}

/** 单项动作结构校验（与 `LegalAction` 联合类型逐项对应）。 */
function isLegalAction(value: unknown): value is LegalAction {
  if (value === "EndPhase") return true;
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    return false;
  }
  const entries = Object.entries(value as Record<string, unknown>);
  if (entries.length !== 1) return false;
  const [kind, body] = entries[0];
  if (body === null || typeof body !== "object") return false;
  const b = body as Record<string, unknown>;
  switch (kind) {
    case "PlayCard":
      return Number.isInteger(b.card) && isNumberArray(b.targets);
    case "ActivateSkill":
      return (
        Number.isInteger(b.skill) &&
        isNumberArray(b.targets) &&
        isNumberArray(b.card_targets)
      );
    case "UseSkillCard":
      return (
        Number.isInteger(b.skill) &&
        Number.isInteger(b.card_kind) &&
        isNumberArray(b.targets) &&
        Number.isInteger(b.equip_slot)
      );
    case "ResolveChoice":
      return (
        Number.isInteger(b.choice_id) &&
        isNumberArray(b.selections) &&
        typeof b.cancelled === "boolean"
      );
    default:
      return false;
  }
}

/**
 * 解析并校验权威动作 JSON。
 *
 * 畸形返回（非 JSON／结构不符／动作形态未知）一律报错：静默丢弃会让界面把
 * 「内核没给候选」与「查询坏了」显示成同一件事。
 */
export function parseLegalActionsJson(raw: string): NativeLegalActionsJson {
  let value: unknown;
  try {
    value = JSON.parse(raw) as unknown;
  } catch {
    throw new Error(`native: 权威动作查询返回非 JSON: ${raw.slice(0, 120)}`);
  }
  const env = value as Partial<NativeLegalActionsJson> | null;
  const seat = env?.seat;
  const rawActions = env?.actions;
  if (
    env === null ||
    typeof env !== "object" ||
    typeof seat !== "number" ||
    !Number.isInteger(seat) ||
    !Array.isArray(rawActions)
  ) {
    throw new Error(
      `native: 权威动作 JSON 结构不符（需 {seat, revision, actions}）: ${raw.slice(0, 120)}`,
    );
  }
  const actions: LegalAction[] = [];
  for (const [index, action] of rawActions.entries()) {
    if (!isLegalAction(action)) {
      throw new Error(
        `native: 权威动作第 ${index} 项结构不符: ${JSON.stringify(action)}`,
      );
    }
    actions.push(action);
  }
  return {
    seat,
    revision: Number(env.revision ?? 0),
    actions,
  };
}

/** 权威候选缓存：同一 `(handle, revision, seat)` 只查一次内核。 */
const legalCache = new WeakMap<
  object,
  { revision: number; seat: number; actions: LegalAction[] }
>();

/**
 * 权威动作查询（内核 `Engine::legal_actions`）。
 *
 * **性能策略**：一次查询实测约 1.6 ms（5 人局，含内核克隆校验），所以
 * ① 只在构建信封时查询（用户动作／观察刷新触发），不在渲染循环里逐帧调用；
 * ② 按 `(handle, revision, seat)` 记忆化——同一修订号重复构建信封不重复查询。
 * 修订号只在命令被接受时前进，被拒绝的命令不改变候选，缓存因此安全；
 * `restore` 会用另一份状态替换引擎实例，必须显式失效（见 `nativeDispatch`）。
 */
function authoritativeLegalActions(handle: NativeHandle): LegalAction[] {
  const revision = Number(handle.engine.revision());
  const cached = legalCache.get(handle);
  if (
    cached &&
    cached.revision === revision &&
    cached.seat === handle.observer
  ) {
    return cached.actions;
  }
  const raw = requireAuthoritativeExports(handle).legal_actions_json(
    handle.observer,
  );
  const parsed = parseLegalActionsJson(raw);
  legalCache.set(handle, {
    revision: parsed.revision,
    seat: parsed.seat,
    actions: parsed.actions,
  });
  return parsed.actions;
}

/** 丢弃某句柄的候选缓存（读档等替换引擎状态的操作后必须调用）。 */
export function invalidateLegalActions(handle: NativeHandle): void {
  legalCache.delete(handle);
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
    legal_actions: authoritativeLegalActions(handle),
    hash_hex: "",
    revision,
    finished,
  };
}

export type NativeDispatchReq =
  | { type: "endPhase"; actor?: number }
  | { type: "playCard"; actor: number; card: number; targets: number[] }
  | {
      type: "activateSkill";
      actor: number;
      skill: number;
      targets?: number[];
      cardTargets?: number[];
    }
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
    case "activateSkill": {
      // 内核 `GameAction::ActivateSkill` 只有技能 id：技能内部的目标/牌选择走
      // 开放选择应答。窄壳因此不接目标；传了目标即协议误用，显式报错而不是
      // 悄悄丢掉调用方的选择。
      const selected = [...(req.targets ?? []), ...(req.cardTargets ?? [])];
      if (selected.length > 0) {
        throw new Error(
          "native: activateSkill 不接受预选目标（目标由内核开放选择应答决定）",
        );
      }
      const out = requireAuthoritativeExports(handle).activate_skill(
        req.actor,
        req.skill,
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
      // 读档会把引擎换成另一份状态（修订号可能相同），候选缓存必须失效。
      invalidateLegalActions(handle);
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
