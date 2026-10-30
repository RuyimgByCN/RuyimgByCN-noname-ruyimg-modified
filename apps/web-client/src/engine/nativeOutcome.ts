import type { ObserverSnapshot } from "../types/engine";

/** Local Rust projections use one-based SingleWin seats; local UI seats are zero-based. */
export function normalizeNativeOutcome(outcome: unknown): ObserverSnapshot["outcome"] {
  if (outcome === null || typeof outcome !== "object" || Array.isArray(outcome)) return outcome;
  const single = (outcome as { SingleWin?: unknown }).SingleWin;
  if (single === null || typeof single !== "object" || Array.isArray(single)) return outcome;
  const player = (single as { player?: unknown }).player;
  if (typeof player !== "number" || !Number.isInteger(player) || player <= 0) return outcome;
  return { ...outcome, SingleWin: { ...single, player: player - 1 } };
}
