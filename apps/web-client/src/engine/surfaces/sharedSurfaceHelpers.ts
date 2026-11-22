/**
 * Leaf pure surface helpers shared between `extensionSurfaceRuntime.ts` and `surfaces/*`
 * without introducing any circular module dependencies.
 */

export interface CoinSurfaceState {
  variant: "noname" | "liuli";
  balance: number;
  betPlaced: number;
  phaseStarted: boolean;
  alwaysSnow: boolean;
  snowFall: boolean;
  snowSize: "small" | "large";
  activeEffects: string[];
  lastSettlement: { payout: number; netDelta: number; video: boolean; mode: string } | null;
  honorTitle?: string;
}

export interface MvpScoreRow {
  seat: number;
  attack: number;
  heal: number;
  assist: number;
  base: number;
  penalty: number;
  total: number;
  kills: number;
  damageDealt: number;
}

export interface MvpSurfaceState {
  enabled: boolean;
  seenSeqs: number[];
  scores: Record<number, MvpScoreRow>;
  mvpSeat: number | null;
}

export function numToRoma(n: number): string {
  const table = ["", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"];
  if (!Number.isInteger(n) || n < 1 || n >= table.length) return String(n);
  return table[n]!;
}

export function classifyDesktopPointer(
  layout: "mouse" | "touch",
  dx: number,
  dy: number,
): "tap" | "drag" | "touch" {
  if (layout === "touch") return "touch";
  return Math.hypot(dx, dy) < 10 ? "tap" : "drag";
}

export function validateSafeExportFilename(name: string): string {
  const trimmed = name.trim();
  if (!trimmed || trimmed === "." || trimmed === ".." || /[\\/]/.test(trimmed)) {
    throw new Error("unsafe export filename");
  }
  return trimmed;
}

export function incrementMvpRecastCount(
  mvp: MvpSurfaceState,
  seat: number,
  count = 1,
): MvpSurfaceState {
  const prevRow = mvp.scores[seat] ?? {
    seat,
    attack: 0,
    heal: 0,
    assist: 0,
    base: 100,
    penalty: 0,
    total: 100,
    kills: 0,
    damageDealt: 0,
  };
  const bonus = Math.max(0, Math.floor(count)) * 3;
  const updatedRow: MvpScoreRow = {
    ...prevRow,
    assist: prevRow.assist + bonus,
    total: prevRow.total + bonus,
  };
  const nextScores = { ...mvp.scores, [seat]: updatedRow };
  let bestSeat = mvp.mvpSeat ?? seat;
  let bestTotal = -Infinity;
  for (const row of Object.values(nextScores)) {
    if (row.total > bestTotal) {
      bestTotal = row.total;
      bestSeat = row.seat;
    }
  }
  return {
    ...mvp,
    scores: nextScores,
    mvpSeat: bestSeat,
  };
}
