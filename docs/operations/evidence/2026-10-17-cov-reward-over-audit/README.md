# 2026-10-17 奖惩与终局顺序：上游 over 后是否仍发奖惩（COV-REWARD-OVER-VERIFY）

**核验日期**：2026-10-17。**待做 ID**：`COV-REWARD-OVER-VERIFY`（覆盖清单行为差异表「奖惩与终局判定顺序」的待核项）。
**性质**：只读上游源码审计（无代码/版本/golden 变化）。采用玩法版本 = 本地 `archive/upstream/noname`（v1.11.7，HEAD `df2682e6`）。源码摘录：[sources.txt](sources.txt)。

## 待核问题

2026-10-16 对照已确认上游顺序为 **胜负判定（`dieAfter`→`checkResult`→`game.over`）在前、奖惩（`dieAfter2`）在后**，原生相反（奖惩先于终局，ADR 0705）。当时把「上游 `game.over` 之后事件链是否仍执行 `dieAfter2`」列为待核项：若仍发放，则差异只表现为顺序/终局快照口径；若被短路，则上游可能漏发奖励。

## 结论：上游 over 后仍执行 dieAfter2（奖惩照发）

`die` 事件的内容步在 `content.ts` 里是一个顺序 `await` 的内容链：

1. `player.dieAfter(source)`（`content.ts:12537-12538`）——身份模式实现在 `identity.js:3159+`，其末尾调用 `game.checkResult()`（`identity.js:3187`），后者对普通身份局调用 `game.over(...)`。
2. `game.over(result, bool)`（`game/index.js:6547+`）以 `if (_status.over) return;` 幂等门禁，随后**快照当前全体手牌**（`hsMap`，`game/index.js:6556-6559`）并置 `_status.over = true`，展示结算界面后返回——**不抛异常、不中断 death 事件内容链**。
3. 之后的内容步继续执行：`trigger("die")`、清理标记/临时技能、死亡弃牌，最后到 `player.dieAfter2(source)`（`content.ts:12615-12616`）——其执行条件为 `player.dieAfter2 && !event.reserveOut && !event.noDieAfter2`，**不含 `_status.over` 短路**。

⇒ 上游在 `game.over` 之后**仍会发放奖惩**（反贼来源摸 3 等，`identity.js:3103+`）。

## 与原生差异的处置

两边都发放奖惩，差异只在**顺序与终局快照口径**：

- 上游：`over` 的手牌快照发生在 `dieAfter2` 之前 ⇒ 结算界面展示的手牌不含奖励；奖励在其后追加，但已无终局状态含义。
- 原生：奖惩先执行、再记录终局 ⇒ 奖励计入终局状态（`last_rebel_reward_precedes_victory` 断言「末反贼奖励不被终局短路」）。

**处置：保留原生顺序**（奖励计入终局前状态），上游对应表现为**顺序与终局快照差异**，无刻意的奖励缺失。覆盖清单「奖惩与终局判定顺序」的待核项据此关闭，处置由「继续验收」改为「**已决策：保留原生；上游 over 后仍发奖惩，差异为顺序与终局快照口径**」。
