// 原生粗粒度码合并了用牌与响应、伤害与防止、濒死与救回、选择开始与完成。
const LABELS: Record<number, string> = {
  1: "对局开始",
  2: "摸牌",
  3: "使用或响应牌",
  4: "伤害结算",
  5: "回复体力",
  6: "濒死状态变化",
  7: "玩家阵亡",
  8: "阶段或回合推进",
  9: "对局结束",
  10: "选择状态变化",
};

export function publicEventLabel(code: number): string {
  return LABELS[code] ?? `事件 ${code}`;
}
