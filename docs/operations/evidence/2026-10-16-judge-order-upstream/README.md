# 2026-10-16 判定阶段顺序核对：上游 LIFO vs 原生 FIFO＋闪电分组

核验日期：2026-10-16。**只读核对 + 补断言固化**（无规则/存档版本变化：v290 / v77；
本批新增 1 个测试，14 golden 不变）。采用玩法版本 = 本地 `archive/upstream/noname`
（v1.11.7，HEAD `df2682e6`）。源码摘录：[upstream-judge-order.txt](upstream-judge-order.txt)。

覆盖清单行为差异表「判定顺序」行的处置为「多非闪电相对顺序及组合恢复继续补验」——
本批做完整核对，结论是**两线顺序规则不同**。

## 1. 上游事实（三条代码事实链）

1. `phaseJudge`：`event.cards = player.getCards("j")` 取**判定区数组**作为队列，
   逐张 `event.cards.shift()` 从**队首**消费（`content.ts:4931-4960`）。
2. `getCards("j")`（`iterableGetCards` case `"j"`）按**判定区 DOM 子节点顺序**产出
   （`player.js:5397-5412`；`get.iterableChildNodes` 为 document 顺序，无反转，
   `get/index.js:4292-4298`）。
3. `$addVirtualJudge` 把新放入的判定牌 **`insertBefore(cardx, judges.firstChild)`**，
   即**新放置的牌在 DOM 最前**（`player.js:15791`）；`ui.updatej/updatejm` 只设置
   CSS `transform` 定位，**不重排 DOM**（`ui/index.js:384-412`）。

⇒ 上游判定顺序 = **LIFO：最后放置的先判**，且**没有闪电分组**（闪电按其放置位置参与）。

## 2. 原生线现状

`EventData::Phase(Judge, begin)` 构造 `JudgePhaseMachine` 队列时先取非闪电、再
追加闪电（`kernel.rs:5083-5100`），因此：**非闪电之间 FIFO（放置序）、闪电恒在最后**
（组内保持原相对序；`kernel.rs:8552-8571` 注释）。旧线为同款分组
（`engine-core/src/command/judge.rs:365-378` `collect_delayed_queue`），ADR 0713 即据此落地。

## 3. 差异与本批处理

| 维度 | 上游（采用版本） | 原生 | 差异 |
|---|---|---|---|
| 多张延时锦囊相对顺序 | LIFO（新放置先判） | FIFO（先放置先判） | 有 |
| 闪电位置 | 按放置位置（无特殊分组） | 恒最后 | 有 |

**本批固化**：新增 `judge_order_grouping_multi_cards_and_restore`——两种放置序（乐→闪电 /
闪电→乐）+ 快照往返，冻结原生「非闪电在前、闪电在后、与放置序无关」且恢复件与直跑一致
（乐吃黑桃→跳过出牌、闪电吃红桃→未命中传下家，故玩家 1 不受雷伤）。

**未决（需决策，不在本批改判）**：对齐上游 LIFO，还是保留原生放置序分组？
- 依「采用玩法版本」口径应改判上游（LIFO、去闪电分组）——属规则语义变化，需递增
  `STANDARD_RULESET_VERSION`、重录 14 golden、改造 `judge_order_delayed_first_then_lightning_hit_kills`
  与本批新测试，并写 ADR（部分取代 ADR 0713 的分组决策）。
- 若判断「放置序」才是项目采用的玩法语义（上游 LIFO 系实现细节），则应把该行**从行为差异表
  移入「采用玩法版本的已知差异（不追）**」并写明依据。
- 本批只完成核对与固化，把待决点显式登记在覆盖清单与 ADR 0713 补注，避免后续批次误以为已对齐。

## 4. 影响

- 覆盖清单：`judge_zone` / `delayed_trick` 行补入上游顺序口径；行为差异「判定顺序」行改写为
  「已核对出差异（方向明确）+ 待决策」。
- ADR 0713 增 2026-10-16 补注（分组依据来自旧线参考，与已明确的采用版本不一致）。
- 规则基准「当前声明与验证边界」判定排序行同步。
- 代码：仅新增测试（断言冻结现状），无行为、版本与 golden 变化。
