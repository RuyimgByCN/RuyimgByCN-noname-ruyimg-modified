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
