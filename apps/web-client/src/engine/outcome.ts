/**
 * 终局胜负文案（NC09/NC12）。
 *
 * 数据来源：视图下发的 `outcome`。原生 WASM 壳（`engine-native-wasm`）把
 * `engine_native::Winner` 映射为协议形态：
 * `{ "LoyalistWin": null }` / `{ "RebelWin": null }` / `{ "RenegadeWin": null }` /
 * `{ "SingleWin": { "player": n } }` / `{ "GroupWin": { "group": "shu" } }`；
 * **未建模的模式胜者**（对决/斗地主/Boss）保持字符串 `"finished"`。
 *
 * 本函数只做**展示映射**，不推断语义：未知形态**如实显示原始 JSON**。
 */
export function outcomeText(outcome: unknown): string {
  if (outcome === null || outcome === undefined) return "胜负未知";
  if (outcome === "finished") return "已结束（胜负类型未建模）";
  if (typeof outcome === "object") {
    const o = outcome as Record<string, unknown>;
    if ("LoyalistWin" in o) return "主公方胜";
    if ("RebelWin" in o) return "反贼胜";
    if ("RenegadeWin" in o) return "内奸胜";
    if ("SingleWin" in o) {
      const p = (o.SingleWin as { player?: number } | null)?.player;
      return p ? `单挑胜者 P${p}` : "单挑结束";
    }
    if ("GroupWin" in o) {
      const g = (o.GroupWin as { group?: string } | null)?.group ?? "?";
      return `势力胜（${g}）`;
    }
  }
  return `已结束（${JSON.stringify(outcome)}）`;
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
