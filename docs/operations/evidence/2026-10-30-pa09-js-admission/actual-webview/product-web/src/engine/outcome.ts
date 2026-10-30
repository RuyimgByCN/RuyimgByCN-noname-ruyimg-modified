import type { ObserverSnapshot, SeatView } from "../types/engine";

/** 展示使用视图中的玩家编号：本地适配后为零基，联机协议仍为一基。 */
export function outcomePresentation(
  outcome: unknown,
  self?: Pick<SeatView, "player" | "identity">,
): { label: string; isSelfWin: boolean | null } {
  const neutral = (label: string) => ({ label, isSelfWin: null });
  if (outcome === null || outcome === undefined) return neutral("胜负未知");
  // 未建模的对决/斗地主/Boss 胜者不推断语义。
  if (outcome === "finished") return neutral("已结束（胜负类型未建模）");
  const o = typeof outcome === "object" ? outcome as Record<string, unknown> : {};
  const tag = typeof outcome === "string" ? outcome : Object.keys(o)[0];
  switch (tag) {
    case "LoyalistWin":
      return { label: "主公方胜", isSelfWin: self?.identity
        ? self.identity === "Lord" || self.identity === "Loyalist" : null };
    case "RebelWin":
      return { label: "反贼胜", isSelfWin: self?.identity ? self.identity === "Rebel" : null };
    case "RenegadeWin":
      return { label: "内奸胜", isSelfWin: self?.identity ? self.identity === "Renegade" : null };
    case "SingleWin": {
      const player = (o.SingleWin as { player?: unknown } | null)?.player;
      if (typeof player === "number" && Number.isInteger(player) && player >= 0) {
        return { label: `单挑胜者 P${player}`, isSelfWin: self ? self.player === player : null };
      }
      break;
    }
    case "GroupWin": {
      const group = (o.GroupWin as { group?: unknown } | null)?.group;
      if (typeof group === "string") return neutral(`势力胜（${group}）`);
      break;
    }
    case "Draw": return neutral("平局");
    case "Aborted": return neutral("对局中止");
  }
  return neutral(`已结束（${JSON.stringify(outcome)}）`);
}

/** 未知形态如实显示原始 JSON，不猜测胜者或阵营。 */
export function outcomeText(outcome: unknown): string {
  return outcomePresentation(outcome).label;
}

/** 视图是权威来源；旧事件信封仅在视图缺少终局结果时回退。 */
export function matchOutcomePresentation(
  view: ObserverSnapshot | null,
  events: unknown[],
  selfId: number,
) {
  let outcome = view?.outcome;
  if (outcome === null || outcome === undefined) {
    for (const event of events) {
      if (!event || typeof event !== "object") continue;
      const e = event as { type?: string; outcome?: unknown; MatchEnded?: { outcome?: unknown } };
      outcome = e.type === "MatchEnded" ? e.outcome : e.MatchEnded?.outcome;
      if (outcome !== null && outcome !== undefined) break;
    }
  }
  return outcomePresentation(outcome, view?.seats.find((seat) => seat.player === selfId));
}

/** 原生窄壳命令及自动步进结果的可见错误。 */
export function nativeCommandError(result: unknown): string | null {
  if (!result || typeof result !== "object") return "命令结果无法读取";
  let value = result as Record<string, unknown>;
  if (value.kind === "plan" && typeof value.error === "string") {
    const reason = value.error === "stale" ? "局面已变化，请重试"
      : value.error === "cancelled" ? "已取消" : value.error;
    return `自动操作失败：${reason}`;
  }
  if (value.kind === "plan" || value.kind === "answer") {
    try {
      const nested = typeof value.outcome === "string" ? JSON.parse(value.outcome) : null;
      if (!nested || typeof nested !== "object" || Array.isArray(nested)) {
        return "自动操作结果无法读取";
      }
      value = nested as Record<string, unknown>;
    } catch {
      return "自动操作结果无法读取";
    }
  }
  if (value.done === true) return null;
  if (value.outcome === "accepted" || value.outcome === "pending") return null;
  if (value.outcome === "suspended") return "操作已暂停，暂无法继续结算";
  if (value.outcome !== "rejected") return "命令结果无法读取";
  const reasons: Record<string, string> = {
    ChoicePending: "请先完成当前选择",
    NoSuchChoice: "当前选择已失效",
    InvalidChoice: "选择无效",
    NotYourTurn: "还没有轮到该玩家",
    WrongPhase: "当前阶段不能执行此操作",
    CardNotInHand: "这张牌不在手牌中",
    IllegalTargets: "目标不合法",
    WrongPlayer: "当前玩家无权处理此选择",
    StaleRevision: "局面已变化，请重试",
    MatchOver: "对局已结束",
  };
  const reason = typeof value.reason === "string" && value.reason ? value.reason : "原因未知";
  return `操作被拒绝：${reasons[reason] ?? reason}`;
}
