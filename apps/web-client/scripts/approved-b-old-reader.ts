// Frozen functional pre-B TS reader, HEAD100489f5 plus reviewed PA09 overlay.
import { normalizeNativeOutcome } from "../src/engine/nativeOutcome";
import type { CardView, ObserverSnapshot } from "../src/types/engine";
interface NativeView {
  seat: number;
  players: Array<{
    seat: number;
    characterDef?: number;
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
