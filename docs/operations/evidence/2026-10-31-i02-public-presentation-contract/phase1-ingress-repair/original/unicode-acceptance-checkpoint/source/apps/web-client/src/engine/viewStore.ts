import { validatePresentationResponse } from "./presentationMetadata";
import { reactive } from "vue";
import type { ChoiceRequest, EngineEnvelope, LegalAction, ObserverSnapshot } from "../types/engine";
import { perfEnvelopeApplied, perfEnvelopeApplying } from "./perfProbe";
import { outcomeText } from "./outcome";

/** Presentation timeline event (not state truth). */
export interface PresentationEvent {
  id: number;
  text: string;
  at: number;
}

/** Transient visual effect overlay (damage flash, heal glow, etc.). */
export interface VisualEffect {
  id: number;
  type: "damage" | "heal" | "draw" | "play" | "dying" | "death";
  player?: number;
  amount?: number;
  text: string;
}

export interface ViewState {
  connected: boolean;
  busy: boolean;
  error: string | null;
  revision: number;
  hash: string;
  finished: boolean;
  view: ObserverSnapshot | null;
  choice: ChoiceRequest | null;
  legalActions: LegalAction[];
  lastEvents: unknown[];
  log: PresentationEvent[];
  /** Transient visual effects to display (auto-cleared after animation). */
  effects: VisualEffect[];
  /** Playback rate for presentation (1 = normal). */
  speed: number;
  /** Skip presentation animations. */
  skipAnims: boolean;
  reduceMotion: boolean;
}

let logId = 1;

export function createViewStore(): ViewState {
  const reduce =
    typeof window !== "undefined" &&
    window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
  return reactive<ViewState>({
    connected: false,
    busy: false,
    error: null,
    revision: 0,
    hash: "",
    finished: false,
    view: null,
    choice: null,
    legalActions: [],
    lastEvents: [],
    log: [],
    effects: [],
    speed: 1,
    skipAnims: false,
    reduceMotion: !!reduce,
  });
}

let effectId = 1;

export function applyEnvelope(store: ViewState, env: EngineEnvelope) {
  validatePresentationResponse(env);
  // 表现层性能采样（默认关闭时立即返回）：记录应用信封、DOM 刷新与下一帧时刻。
  perfEnvelopeApplying();
  applyEnvelopeState(store, env);
  perfEnvelopeApplied();
}

function applyEnvelopeState(store: ViewState, env: EngineEnvelope) {
  store.revision = env.revision;
  store.hash = env.hash_hex;
  store.finished = env.finished;
  store.view = env.view;
  store.choice = env.choice;
  store.legalActions = env.legal_actions ?? [];
  store.lastEvents = env.events ?? [];
  store.error = null;

  // Extract visual effects from events.
  const effects = extractEffects(env.events ?? []);
  store.effects = effects;

  const texts = summarizeEvents(env.events ?? []);
  for (const text of texts) {
    pushLog(store, text);
  }
}

function extractEffects(events: unknown[]): VisualEffect[] {
  const effects: VisualEffect[] = [];
  for (const e of events) {
    if (!e || typeof e !== "object") continue;
    const o = e as Record<string, unknown>;
    const tag = Object.keys(o)[0];
    if (!tag) continue;
    const body = o[tag] as Record<string, unknown> | undefined;
    switch (tag) {
      case "DamageDealt":
        effects.push({ id: effectId++, type: "damage", player: body?.target as number, amount: body?.amount as number, text: `-${body?.amount}` });
        break;
      case "HpRecovered":
        effects.push({ id: effectId++, type: "heal", player: body?.player as number, amount: body?.amount as number, text: `+${body?.amount}` });
        break;
      case "CardsDrawn":
        effects.push({ id: effectId++, type: "draw", player: body?.player as number, text: `摸${Array.isArray(body?.cards) ? body!.cards.length : 1}` });
        break;
      case "CardUsed":
        effects.push({ id: effectId++, type: "play", player: body?.player as number, text: `出牌` });
        break;
      case "EnteredDying":
        effects.push({ id: effectId++, type: "dying", player: body?.player as number, text: `濒死` });
        break;
      case "PlayerKilled":
        effects.push({ id: effectId++, type: "death", player: body?.player as number, text: `阵亡` });
        break;
    }
  }
  return effects;
}

/** Clear visual effects after animation completes. */
export function clearEffects(store: ViewState) {
  store.effects = [];
}

export function pushLog(store: ViewState, text: string) {
  store.log.unshift({
    id: logId++,
    text,
    at: Date.now(),
  });
  if (store.log.length > 80) store.log.length = 80;
}

function summarizeEvents(events: unknown[]): string[] {
  const out: string[] = [];
  for (const e of events) {
    if (!e || typeof e !== "object") continue;
    const o = e as Record<string, unknown>;
    const tag = Object.keys(o)[0];
    if (!tag) continue;
    const body = o[tag] as Record<string, unknown> | undefined;
    switch (tag) {
      case "GameStarted":
        out.push(`对局开始 · ${body?.player_count ?? "?"} 人`);
        break;
      case "CardsDrawn":
        out.push(`玩家 ${body?.player} 摸牌 ×${Array.isArray(body?.cards) ? body!.cards.length : "?"}`);
        break;
      case "CardUsed":
        out.push(`玩家 ${body?.player} 使用卡牌 ${body?.card}`);
        break;
      case "DamageDealt":
        out.push(`伤害 ${body?.source} → ${body?.target} (${body?.amount})`);
        break;
      case "HpRecovered":
        out.push(`玩家 ${body?.player} 回复 ${body?.amount}`);
        break;
      case "EnteredDying":
        out.push(`玩家 ${body?.player} 进入濒死`);
        break;
      case "PlayerKilled":
        out.push(`玩家 ${body?.player} 阵亡`);
        break;
      case "TurnStateChanged":
        out.push(`回合 · 玩家 ${body?.current} · ${body?.phase}`);
        break;
      case "MatchEnded":
        out.push(`终局 · ${outcomeText(body?.outcome)}`);
        break;
      case "ChoiceRequested":
        out.push(`等待选择 · 玩家 ${body?.observer}`);
        break;
      case "PublicEvent":
        out.push(String(body?.label ?? `公共事件 ${body?.code ?? "?"}`));
        break;
      default:
        out.push(tag);
    }
  }
  return out;
}

/** Schedule presentation delay; respects reduce-motion / skip / speed. */
export function presentationDelay(store: ViewState, baseMs: number): Promise<void> {
  if (store.skipAnims || store.reduceMotion || store.speed <= 0) {
    return Promise.resolve();
  }
  const ms = Math.max(0, baseMs / store.speed);
  return new Promise((r) => setTimeout(r, ms));
}
