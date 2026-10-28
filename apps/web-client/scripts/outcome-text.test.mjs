// NC09/NC12：终局胜负文案映射的自动化断言（跨层一致性）。
import { test } from "node:test";
import assert from "node:assert/strict";
import * as outcome from "../src/engine/outcome.ts";
const { outcomeText } = outcome;

test("身份局胜负类型映射", () => {
  assert.equal(outcomeText({ LoyalistWin: null }), "主公方胜");
  assert.equal(outcomeText({ RebelWin: null }), "反贼胜");
  assert.equal(outcomeText({ RenegadeWin: null }), "内奸胜");
});

test("单挑/势力与未建模形态", () => {
  assert.equal(outcomeText({ SingleWin: { player: 4 } }), "单挑胜者 P4");
  assert.equal(outcomeText({ GroupWin: { group: "shu" } }), "势力胜（shu）");
  assert.equal(outcomeText("finished"), "已结束（胜负类型未建模）");
  assert.equal(outcomeText(null), "胜负未知");
});

test("未知形态如实显示原始 JSON（不猜测语义）", () => {
  assert.equal(outcomeText({ SomethingNew: 1 }), '已结束（{"SomethingNew":1}）');
});

test("原生命令的拒绝原因可显示，未知原因保留", () => {
  assert.equal(outcome.nativeCommandError({ outcome: "rejected", reason: "NotYourTurn" }), "操作被拒绝：还没有轮到该玩家");
  assert.equal(outcome.nativeCommandError({ outcome: "rejected", reason: "NewFailure" }), "操作被拒绝：NewFailure");
  assert.equal(outcome.nativeCommandError({ outcome: "rejected", reason: null }), "操作被拒绝：原因未知");
  assert.equal(outcome.nativeCommandError({ outcome: "accepted", reason: null }), null);
  assert.equal(outcome.nativeCommandError({ outcome: "pending", reason: null }), null);
  assert.equal(outcome.nativeCommandError({ outcome: "suspended" }), "操作已暂停，暂无法继续结算");
  assert.equal(outcome.nativeCommandError(null), "命令结果无法读取");
});

test("自动步进的嵌套拒绝和计划错误不可当作成功", () => {
  const nested = (value) => ({ kind: "answer", outcome: JSON.stringify(value) });
  assert.equal(outcome.nativeCommandError(nested({ outcome: "rejected", reason: "InvalidChoice" })), "操作被拒绝：选择无效");
  assert.equal(outcome.nativeCommandError(nested({ outcome: "accepted" })), null);
  assert.equal(outcome.nativeCommandError({ done: true }), null);
  assert.equal(outcome.nativeCommandError({ kind: "plan", error: "stale" }), "自动操作失败：局面已变化，请重试");
  assert.equal(outcome.nativeCommandError({ kind: "plan", error: "cancelled" }), "自动操作失败：已取消");
  assert.equal(outcome.nativeCommandError({ kind: "plan", error: "new_error" }), "自动操作失败：new_error");
  assert.equal(outcome.nativeCommandError({ kind: "plan", outcome: "{" }), "自动操作结果无法读取");
});
