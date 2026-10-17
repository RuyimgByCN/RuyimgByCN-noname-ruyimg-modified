# 2026-10-17 覆盖清单两项口径定案：装备即时生效、触发 getIndex/桶

**核验日期**：2026-10-17。**性质**：口径定案（无代码/版本/golden 变化；规则 v295 / 存档 v80）。
采用玩法版本 = 本地 `archive/upstream/noname`（v1.11.7，HEAD `df2682e6`）。源码摘录：[sources.txt](sources.txt)。

本批按 2026-10-16「语义选择：窗口/顺序口径」定案的同一路径处理两个覆盖清单开放行：

## 1. `COV-EQUIP-TIMING-WINDOW`（装备即时生效 vs 上游装备时机窗）

- **上游**：装备牌走的是一个 `equip` 事件（`content.ts:1017 async equip(event, trigger, player)`，含 `equipBegin`/`equip` 等时机），装备在事件链内落位；相关结算/触发在同一事件窗口内进行。
- **原生**：装备提交后**立即生效**，同一次命令内即参与距离/射程/防具判定（如 +1 马在当回合即可影响可指定性）。
- **定案（项目采用）**：**保留原生「装备即时生效」**，不实现上游的装备时机窗口化。理由与 2026-10-16 定案一致：翻转属语义变化（需递增规则版本 + 重录 14 golden + 改造测试），且原生需要可确定编码/可重放的装备状态转移；上游 `equipBegin` 窗口在无交互宿主下无稳定复现口径。
- **边界**：+1 马对**结算中之杀**是否回溯影响目标合法性属窗口时点差异，登记为**已知语义差异**；有真实内容需求（某技能依赖 `equipBegin`）时另立 ADR 与版本次。

## 2. `COV-TRIGGER-GETINDEX-BUCKETS`（触发全序：`getIndex` / `firstDo` / `lastDo` 桶）

- **上游**：`arrangeTrigger` 对每次触发的 `doing` 逐条消费 `todoList`；`firstDo`/`lastDo` 时机**不做技能优先级选择**（`content.ts:4031`），普通时机按 `priority` 降序 + 同优先级 `silent`/无描述技优先、否则交互选择。
- **原生**：确定性键 `(Timing::{First,Normal,Last}, -priority, optional, seat_distance(turn.current, owner), def_id)`（[触发全序对照](../2026-10-16-trigger-order-upstream/README.md)），`First`/`Last` 桶已作为 `Timing` 分量承载；`getIndex` 多次触发未建模。
- **定案（项目采用）**：**明确不立项**（保持原生确定性全序；`firstDo`/`lastDo` 已由 Timing 桶承载，`getIndex` 多次触发**无内容需求**）。触发条件：当某个转换包/技能实际依赖 `getIndex` 多次触发序时，再按其内容证据立项并递增版本。

## 影响

覆盖清单两行处置更新为「已定案：保留原生（已知差异/不立项）」；规则基准「语义选择」表补两行。无代码、版本与 golden 变化。
