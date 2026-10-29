# 2026-10-17 距离语义 `undist`/`isMin` 登记（RULE-DISTANCE-UNMODELED）

**核验日期**：2026-10-17。**待做 ID**：`RULE-DISTANCE-UNMODELED`（规则基准「距离/座距」行的未建模项）。
**性质**：差异登记（无代码/版本/golden 变化；规则 v295 / 存档 v80）。
采用玩法版本 = 本地 `archive/upstream/noname`（v1.11.7，HEAD `df2682e6`）。源码摘录：[sources.txt](sources.txt)。

## 结论：`undist` / `isMin` 登记为未建模（无当前内容依赖）

上游 `get.distance`（`player.js:3180-3199`）在座位计数中**排除**两类玩家：

1. **`undist` 技能**：`!player.hasSkill("undist")`——持该标记技的玩家不参与距离计数（长度也不计入）。
2. **`isMin(true)`**：处于「最小化/隐藏」态（模式级，如排位/国战派生）的玩家同样被排除；且当 from/to 任一方 `isMin(true)` 时整段距离计算**短路为空**（返回 NaN/未定义由调用方处理）。

**原生**：`computed_distance`/`seat_distance_alive`（`state/core_ops.rs:355/675`）按「存活座位环形计数」计算，**不建模** `undist`/`isMin`；国战阵法派生判定处已就地登记「构逆/野心家结盟/undist 不建模」（`state/core_ops.rs:356`）。当前目录**无**任何标准/扩展将持 `undist`，也无 `isMin` 模式态入口，故不影响现有对局语义。

## 处置

- **`undist`**：登记为未建模差异。解除条件：某转换包/技能实际声明 `undist` 标记技时，按 `computed_distance` 加一处「持技者排除」臂（涉及是否升版本/重录 golden 另批）。
- **`isMin`**：登记为未建模差异。解除条件：引入「最小化/隐藏态」模式（排位/国战派生）时随该模式建模。
- 其余距离类技能（逐技修饰）仍按覆盖清单 [COV-STANDARD-FULL-CONTENT](../2026-10-16-todo-register/README.md) 与行为差异表逐条验收，**不在本项范围**。

## 影响

无代码、版本、golden 变化。规则基准「距离/座距」行的 `undist`/`isMin` 未建模表述保留为**已登记差异**（附解除条件）；登记表 `RULE-DISTANCE-UNMODELED` resolved。
