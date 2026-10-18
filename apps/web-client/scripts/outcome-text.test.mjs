// NC09/NC12：终局胜负文案映射的自动化断言（跨层一致性）。
import { test } from "node:test";
import assert from "node:assert/strict";
import { outcomeText } from "../src/engine/outcome.ts";

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
