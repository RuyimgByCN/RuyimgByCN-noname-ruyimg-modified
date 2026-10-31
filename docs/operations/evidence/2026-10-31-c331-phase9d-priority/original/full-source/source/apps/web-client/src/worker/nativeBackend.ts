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
 * `restore_hex_with_packs_json` 接收安装载荷数组（兼容 cards.json 元素）。空数组等于标准目录。
 * 管理面元数据仍由 extensionStore 保存；开局把包 JSON 经 packsJson 传入。
 *
 * 选将（NC05）：`generalCandidates()` / `chooseGeneral(seat, name)` 走正式选将
 * 导出面（`general_candidates_json` / `assign_general`，非 `debug_*`）；候选与
 * 返回逐项结构校验、畸形即报错，旧产物缺导出给出 `build:native` 重建命令。
 */
import { generalSlots, choiceMetadata } from "../engine/observerMetadata";
import { publicEventLabel } from "../engine/publicEvents";
import { normalizeNativeOutcome } from "../engine/nativeOutcome";
import type {
  CardView,
  EngineEnvelope,
  GeneralAssignment,
  GeneralCandidate,
  GeneralCandidatesPayload,
  LegalAction,
  ObserverSnapshot,
} from "../types/engine";
import init, { NativeEngine, validate_pack_json } from "./native-pkg/engine_native_wasm.js";
import { stepMark } from "./stepTimings";

export interface NativeHandle {
  engine: NativeEngine;
  seed: number;
  players: number;
  /** 已消费的 native LogEvent 长度；命令信封只投影此后的公开事件码。 */
  eventCursor: number;
  observer: number;
}

let ready: Promise<void> | null = null;

/**
 * GT-014 导入期深层校验（裁决第 12 项）：单个安装载荷交 Rust 校验器
 * （包/模块 logic_hash、rule-vm 装载期 verify、宿主 ABI 窗口、资产桥接），
 * 与开局装载同一逐包校验，只校验不建局。拒绝时抛出 Rust 侧原始错误文本。
 */
export function nativeValidatePack(packJson: string): void {
  if (typeof packJson !== "string" || packJson.length === 0) {
    throw new Error("native: validatePackage 载荷须为非空 JSON 字符串");
  }
  validate_pack_json(packJson);
}

/** wasm-bindgen 会截断整数；必须先在 JS 边界拒绝不可表示的值。 */
function unsigned(value: number, max: number, field: string): number {
  if (!Number.isInteger(value) || value < 0 || value > max) {
    throw new Error(`native: ${field} 非法整数 ${value}`);
  }
  return value;
}

/** 浏览器/Worker 侧必须先初始化 wasm（web 目标）。 */
/// NC12 冷启动拆分（worker 内计时；`performance` 在 worker 上下文可用）：
/// 记录 **fetch**（下载 .wasm）与 **instantiate**（编译+实例化）两段耗时，
/// 供浏览器档位测量读取（`nativeInitTimings()`），不改变任何行为。
export interface NativeInitTimings {
  fetch_ms: number;
  instantiate_ms: number;
  total_ms: number;
  wasm_bytes: number;
}

let initTimings: NativeInitTimings | null = null;

/** 读取 worker 内初始化分段耗时（未初始化时为 null）。 */
export function nativeInitTimings(): NativeInitTimings | null {
  return initTimings;
}

export async function ensureNativeReady(): Promise<void> {
  if (!ready) {
    ready = (async () => {
      const url = new URL("./native-pkg/engine_native_wasm_bg.wasm", import.meta.url);
      const t0 = performance.now();
      // ① 下载：显式 fetch（随后用同一 Response 的 bytes 实例化，避免二次请求）
      let bytes: ArrayBuffer | undefined;
      let fetchMs = 0;
      try {
        const resp = await fetch(url);
        bytes = await resp.arrayBuffer();
        fetchMs = performance.now() - t0;
      } catch {
        fetchMs = performance.now() - t0; // 失败时仍记录，交由 init 报错
      }
      const t1 = performance.now();
      // ② 编译 + 实例化
      await (bytes
        ? init({ module_or_path: bytes })
        : init({ module_or_path: url }));
      const t2 = performance.now();
      initTimings = {
        fetch_ms: Math.round(fetchMs * 100) / 100,
        instantiate_ms: Math.round((t2 - t1) * 100) / 100,
        total_ms: Math.round((t2 - t0) * 100) / 100,
        wasm_bytes: bytes?.byteLength ?? 0,
      };
    })();
  }
  await ready;
}

export function nativeCreate(
  players: number,
  seed: number,
  observer = 0,
): NativeHandle {
  unsigned(players, 255, "players");
  unsigned(observer, 255, "observer");
  return {
    engine: new NativeEngine(players, BigInt(seed)),
    seed,
    players,
    eventCursor: 0,
    observer,
  };
}

/**
 * **身份局**建局（NC09 身份局端到端，浏览器侧）。
 *
 * 与 {@link nativeCreate} 的唯一区别是 `GameMode::Identity`（主公/忠臣/反贼/内奸），
 * 胜负结果走 `Winner::{LordSide, Rebels, Traitor}`。
 */
export function nativeCreateIdentity(
  players: number,
  seed: number,
  observer = 0,
  packsJson?: string,
): NativeHandle {
  unsigned(players, 255, "players");
  unsigned(observer, 255, "observer");
  return {
    engine: packsJson === undefined
      ? NativeEngine.new_identity(players, BigInt(seed))
      : NativeEngine.new_identity_with_packs_json(players, BigInt(seed), packsJson),
    seed,
    players,
    eventCursor: 0,
    observer,
  };
}

/** 已安装扩展载荷数组建局。空数组与 nativeCreate 相同。 */
export function nativeCreateWithPacks(
  players: number,
  seed: number,
  observer: number,
  packsJson: string,
): NativeHandle {
  unsigned(players, 255, "players");
  unsigned(observer, 255, "observer");
  return {
    engine: NativeEngine.new_with_packs_json(players, BigInt(seed), packsJson),
    seed,
    players,
    eventCursor: 0,
    observer,
  };
}

interface NativeView {
  seat: number;
  players: Array<{
    seat: number;
    characterDef?: number;
    generals?: unknown;
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
    // NC03/NC09 裁决⑥：观察者过滤后的身份（本人/主公/已死亡可见，其余 null）。
    identity?: "Lord" | "Loyalist" | "Rebel" | "Renegade" | null;
  }>;
  deckCount: number;
  discard: number[];
  table: number[];
  // 裁决⑦：观察者可见牌的真实牌面（id → def/suit/rank/kind）；仅含
  // 本人手牌与全场装备/判定/弃牌/处理区——他人手牌与牌堆不在其中。
  cards?: Record<string, { def: number; suit: string | null; rank: number | null; kind: string }>;
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

type FaceJson = { def: number; suit: string | null; rank: number | null; kind: string };

/** 可见牌 → CardView（复用已有字段：face 缺失时不虚构牌面，def 置 null 由
 * 渲染门挡住——正常路径 WASM 对全部可见 id 都提供牌面）。 */
function faceToCardView(
  view: NativeView,
  id: number,
  owner: number | null,
  zone: CardView["zone"],
): CardView {
  const face: FaceJson | undefined = view.cards?.[String(id)];
  return {
    id,
    def: face?.def ?? null,
    suit: (face?.suit as CardView["suit"]) ?? null,
    rank: face?.rank ?? null,
    owner,
    zone,
    kind: (face?.kind as CardView["kind"]) ?? "Other",
  };
}

/** observer schema 适配：view_json → ObserverSnapshot。 */
export function nativeObserverToSnapshot(view: NativeView): ObserverSnapshot {
  const self = view.players.find((p) => p.seat === view.seat);
  const ownHand = self?.hand?.own ?? [];
  // 裁决⑦：可见牌面覆盖——本人手牌 + 全场装备/判定（公开）+ 弃牌堆 +
  // 处理区；他人手牌 id 不进入 known_cards（WASM 侧本就不下发）。
  const known_cards = [
    ...ownHand.map((id) => faceToCardView(view, id, view.seat, "Hand")),
    ...view.players.flatMap((p) => [
      ...p.equips.map((id) => faceToCardView(view, id, p.seat, "Equip")),
      ...p.judge.map((id) => faceToCardView(view, id, p.seat, "Judge")),
    ]),
    ...view.discard.map((id) => faceToCardView(view, id, null, "DiscardPile")),
    ...view.table.map((id) => faceToCardView(view, id, null, "Processing")),
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
      character_def: p.characterDef ?? 0,
      generals: p.generals === undefined ? undefined : generalSlots(p.generals),
      seat: p.seat,
      hp: p.hp,
      max_hp: p.maxHp,
      alive: p.alive,
      turned_over: !!p.turnedOver,
      hand_count: p.handCount,
      // 裁决⑥：复用 SeatView.identity——不再恒置 null。
      identity: p.identity ?? null,
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
    outcome: normalizeNativeOutcome(view.outcome ?? null),
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
  skill?: string;
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
  native_kind?: unknown;
  source_skill?: unknown;
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
  return o.card ?? o.player ?? o.tag ?? o.kind ?? o.skill ?? 0;
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
  if (o.skill !== undefined) return `skill:${o.skill}`;
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
  } catch (error) {
    throw new Error(`native: choice JSON invalid: ${errorText(error)}`);
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

/**
 * NC05 选将导出面（`crates/engine-native-wasm/src/lib.rs`）。
 *
 * 与 NC03 权威动作查询同款纪律：旧 wasm 产物缺导出时**显式报错并给出重建
 * 命令**。静默降级（当作「无候选」或「忽略选将」）会让「目录里真的没收这个
 * 武将」与「产物过期」在界面上变成同一件事。
 */
interface NativeGeneralSelectExports {
  general_candidates_json: () => string;
  assign_general: (seat: number, name: string) => string;
}

function requireGeneralSelectExports(
  handle: NativeHandle,
): NativeGeneralSelectExports {
  const engine = handle.engine as unknown as Partial<NativeGeneralSelectExports>;
  const missing = (
    ["general_candidates_json", "assign_general"] as const
  ).filter((name) => typeof engine[name] !== "function");
  if (missing.length > 0) {
    throw new Error(
      `native: wasm 产物缺少 ${missing.join("/")}（NC05 选将导出）——` +
        "执行 pnpm --dir apps/web-client build:native 重建 native-pkg；" +
        "不静默降级为「无候选/忽略选将」",
    );
  }
  return engine as NativeGeneralSelectExports;
}

/** wasm-bindgen 的错误是 JS 字符串（`JsValue::from_str`），不是 `Error`。 */
function errorText(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return String(error);
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((s) => typeof s === "string");
}

/**
 * 解析并逐项校验选将候选 JSON（内核 `Engine::general_candidates` 投影）。
 *
 * 畸形返回（非 JSON／缺字段／类型错／def 重复／count 与候选数不一致）一律
 * 报错：候选面是「哪些武将能选」的唯一来源，坏数据必须可见，不能被过滤成
 * 一个更小的候选集。
 */
export function parseGeneralCandidatesJson(
  raw: string,
): GeneralCandidatesPayload {
  let value: unknown;
  try {
    value = JSON.parse(raw) as unknown;
  } catch {
    throw new Error(`native: 选将候选返回非 JSON: ${raw.slice(0, 120)}`);
  }
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(
      `native: 选将候选 JSON 结构不符（需 {count, candidates}）: ${raw.slice(0, 120)}`,
    );
  }
  const env = value as { count?: unknown; candidates?: unknown };
  const count = env.count;
  if (
    typeof count !== "number" ||
    !Number.isInteger(count) ||
    !Array.isArray(env.candidates)
  ) {
    throw new Error(
      `native: 选将候选 JSON 结构不符（需 {count:int, candidates:[]}）: ${raw.slice(0, 120)}`,
    );
  }
  const candidates: GeneralCandidate[] = [];
  const seen = new Set<number>();
  for (const [index, item] of env.candidates.entries()) {
    if (item === null || typeof item !== "object" || Array.isArray(item)) {
      throw new Error(
        `native: 选将候选第 ${index} 项结构不符: ${JSON.stringify(item)}`,
      );
    }
    const row = item as Record<string, unknown>;
    const def = row.def;
    const name = row.name;
    const skills = row.skills;
    if (
      typeof def !== "number" ||
      !Number.isInteger(def) ||
      def < 0 ||
      typeof name !== "string" ||
      name.length === 0 ||
      !isStringArray(skills)
    ) {
      throw new Error(
        `native: 选将候选第 ${index} 项结构不符（需 {def:非负整数, name:非空字符串, skills:字符串数组}）: ${JSON.stringify(item)}`,
      );
    }
    if (seen.has(def)) {
      throw new Error(`native: 选将候选第 ${index} 项 def=${def} 与前面重复`);
    }
    seen.add(def);
    candidates.push({ def, name, skills: [...skills] });
  }
  if (candidates.length !== count) {
    throw new Error(
      `native: 选将候选 count=${count} 与候选数 ${candidates.length} 不一致`,
    );
  }
  return { count: candidates.length, candidates };
}

/**
 * 解析并校验选将返回（`assign_general` 投影）。
 *
 * 回包必须自证与请求一致（`seat`/`name` 回显）：错座位的装配结果若被当成
 * 成功，界面会把「选了 A 座」显示成「B 座已装配」。
 */
export function parseGeneralAssignmentJson(
  raw: string,
  expected: { seat: number; name: string },
): GeneralAssignment {
  let value: unknown;
  try {
    value = JSON.parse(raw) as unknown;
  } catch {
    throw new Error(`native: 选将返回非 JSON: ${raw.slice(0, 120)}`);
  }
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(
      `native: 选将返回结构不符（需 {seat, name, revision, state_hash}）: ${raw.slice(0, 120)}`,
    );
  }
  const row = value as Record<string, unknown>;
  const seat = row.seat;
  const name = row.name;
  const revision = row.revision;
  const stateHash = row.state_hash;
  if (
    typeof seat !== "number" ||
    !Number.isInteger(seat) ||
    seat !== expected.seat ||
    typeof name !== "string" ||
    name !== expected.name ||
    typeof revision !== "number" ||
    !Number.isInteger(revision) ||
    typeof stateHash !== "string" ||
    !/^[0-9a-f]+$/.test(stateHash)
  ) {
    throw new Error(
      `native: 选将返回结构不符或与请求不一致（期望 seat=${expected.seat}, name=${expected.name}）: ${raw.slice(0, 120)}`,
    );
  }
  return { seat, name, revision, state_hash: stateHash };
}

/**
 * NC05 选将候选（内核 `Engine::general_candidates`，只读、顺序稳定）。
 *
 * 候选面**当前不含座位**：内核 `general_candidates()` 是目录全量，尚未按座位
 * 门控（赛前大厅阶段、双将明置与选将信息隔离是 NC05 待办）。适配层如实透传，
 * 不假装做过座位过滤。
 */
/// 最近一次 `generalCandidates` 的分段耗时（worker 内；由 worker 回传给页面）。
export let lastCandidateTimings: {
  engine_ms: number;
  parse_ms: number;
  json_bytes: number;
} | null = null;

export function generalCandidates(
  handle: NativeHandle,
): GeneralCandidatesPayload {
  // NC12 诊断（不参与业务逻辑）：拆分「引擎生成 + 序列化」与「JS 侧解析」两段耗时，
  // 供浏览器档位测量读取（客户端钩子 window.__nativeCandidateTimings）。
  const t0 = performance.now();
  const raw = requireGeneralSelectExports(handle).general_candidates_json();
  const t1 = performance.now();
  const parsed = parseGeneralCandidatesJson(raw);
  const t2 = performance.now();
  lastCandidateTimings = {
    engine_ms: Math.round((t1 - t0) * 100) / 100,
    parse_ms: Math.round((t2 - t1) * 100) / 100,
    json_bytes: raw.length,
  };
  return parsed;
}

/**
 * NC05 正式选将（按名字装配到座位；不使用 `debug_grant`）。
 *
 * 失败（座位越界／未知武将／对局已结束）由 WASM 抛出：这里带上下文重抛，
 * **原始错误文本原样保留**，调用方能区分「武将名打错」与「产物过期」。
 * 成功后调用 `invalidateSeatCaches`：装配路径**不推进 revision**（实测装配
 * 前后 `revision()` 相同），缓存若按 `(revision, seat)` 记忆化会命中装配前的
 * 旧候选。
 */
export function chooseGeneral(
  handle: NativeHandle,
  seat: number,
  name: string,
): GeneralAssignment {
  unsigned(seat, 255, "seat");
  const exports = requireGeneralSelectExports(handle);
  let raw: string;
  try {
    raw = exports.assign_general(seat, name);
  } catch (error) {
    throw new Error(
      `native: 选将失败(seat=${seat}, name=${name}): ${errorText(error)}`,
    );
  }
  // 正常返回 = 装配已生效（内核 `Err` 才表示拒绝）。**先失效缓存再解析**：
  // 解析失败时状态同样已经变了，留着旧候选会让界面继续按装配前的技能集渲染。
  invalidateSeatCaches(handle, seat);
  try {
    return parseGeneralAssignmentJson(raw, { seat, name });
  } catch (error) {
    throw new Error(
      `native: 选将已生效但返回无法解析（该座位候选缓存已失效）: ${errorText(error)}`,
    );
  }
}

/**
 * 选将后使该座位的动作/选择缓存失效。
 *
 * 粒度说明：`legalCache` 每句柄一条 `(revision, seat)` 记录，且装配会改变
 * **共享状态**（`state_hash` 变而 `revision` 不变）——只丢该座位的话，观察者
 * 座位的旧候选仍会被命中。因此这里保守地丢弃整条记录（是该座位失效的超集）。
 * 选择面（`open_choice_json`）逐次直读引擎、无缓存，无需处理。
 */
export function invalidateSeatCaches(handle: NativeHandle, seat: number): void {
  invalidateLegalActions(handle);
  // seat 参与缓存键的历史设计；当前实现按句柄整体失效（见上）。
  void seat;
}

function readChoice(handle: NativeHandle): EngineEnvelope["choice"] {
  const choice = readChoiceJson(handle);
  if (!choice || choice.hidden) return null;
  const options = choice.options ?? [];
  return {
    choice_id: Number(choice.id),
    observer: handle.observer,
    min_count: choice.min,
    max_count: choice.max,
    allow_cancel: choice.allowCancel,
    ...choiceMetadata(choice),
    revision: Number(handle.engine.revision()),
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
  stepMark("envelope_enter");
  const raw = handle.engine.view_json(handle.observer);
  stepMark("view_json", raw.length);
  const view = JSON.parse(raw) as NativeView;
  stepMark("view_parse");
  const finished = handle.engine.is_finished();
  const revision = Number(handle.engine.revision());
  const snap = nativeObserverToSnapshot(view);
  snap.revision = revision;
  // 兼容旧测试桩：真实生成包一定带这两个导出；缺失时退回空占位，
  // 生产路径由 native-worker-smoke 对真实 WASM 断言非空。
  const exposesAuthority =
    typeof handle.engine.state_hash_hex === "function" &&
    typeof handle.engine.log_len === "function" &&
    typeof handle.engine.public_event_codes_json === "function";
  const envelope: EngineEnvelope = {
    result,
    events: exposesAuthority && result !== null ? readPublicEvents(handle) : [],
    choice: readChoice(handle),
    view: snap,
    legal_actions: authoritativeLegalActions(handle),
    hash_hex: exposesAuthority ? handle.engine.state_hash_hex() : "",
    revision,
    finished,
  };
  stepMark("envelope_done");
  return envelope;
}

/** 读取自上次信封以来的真实公开事件码；不含暗牌/选项内容。 */
function readPublicEvents(handle: NativeHandle): EngineEnvelope["events"] {
  const end = Number(handle.engine.log_len());
  if (handle.eventCursor > end) handle.eventCursor = end;
  const start = handle.eventCursor;
  const raw = handle.engine.public_event_codes_json(start);
  handle.eventCursor = end;
  let codes: number[] = [];
  try {
    const parsed = JSON.parse(raw) as unknown;
    if (Array.isArray(parsed)) codes = parsed.map(Number).filter(Number.isFinite);
  } catch {
    codes = [];
  }
  return codes.map((code) => ({
    PublicEvent: { code, label: publicEventLabel(code) },
  }));
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
      const out = handle.engine.next_phase(unsigned(req.actor ?? handle.observer, 255, "actor"));
      return nativeEnvelope(handle, parseCommandResult(out));
    }
    case "playCard": {
      const out = handle.engine.play_card(
        unsigned(req.actor, 255, "actor"),
        unsigned(req.card, 0xffffffff, "card"),
        new Uint32Array(req.targets.map((target) => unsigned(target, 255, "target"))),
      );
      return nativeEnvelope(handle, parseCommandResult(out));
    }
    case "activateSkill": {
      unsigned(req.actor, 255, "actor");
      unsigned(req.skill, 0xffff, "skill");
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
      unsigned(req.actor, 255, "actor");
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
      const restore = (engine: NativeEngine) => {
        if (req.packsJson !== undefined) {
          engine.restore_hex_with_packs_json(req.hex, req.packsJson);
        } else {
          engine.restore_hex(req.hex);
        }
      };
      // 先在临时引擎验证整个信封，保留原句柄及其规划设置。
      // shortcut: 读档会解码两次，具备原子交换的窄壳接口时再优化。
      const candidate = new NativeEngine(1, 0n);
      let envelope: EngineEnvelope;
      try {
        restore(candidate);
        envelope = nativeEnvelope({ ...handle, engine: candidate, eventCursor: 0 });
      } finally {
        candidate.free();
      }
      restore(handle.engine);
      // 读档会把引擎换成另一份状态（修订号可能相同），候选缓存必须失效；
      // 恢复后的引擎从空日志开始，事件游标同步归零。
      handle.eventCursor = 0;
      invalidateLegalActions(handle);
      return envelope;
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
