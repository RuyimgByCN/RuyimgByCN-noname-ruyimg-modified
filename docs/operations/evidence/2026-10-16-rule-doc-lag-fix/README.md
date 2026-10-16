# 判定顺序与预算声明文档滞后勘正（2026-10-16）

**性质**：只改文档；**不改** `crates/**`、**不改** `native-progress.md`。  
**关闭登记项**：`RULE-JUDGE-ORDER-NOTE`、`RULE-BUDGET-DOC-LAG`（决议见 [ambiguous-resolutions](../2026-10-16-todo-register/ambiguous-resolutions.md) §11 / §13）。

## 改前要点

| ID | 文件 / 位置 | 滞后句 |
|---|---|---|
| `RULE-JUDGE-ORDER-NOTE` | `docs/architecture/native-rules-baseline.md`「当前声明」判定排序/闪电 · 验证边界 | 「顺序规则两线不一致，是否改判上游待决策（见覆盖清单行为差异表）」 |
| （同口径） | `docs/native-coverage-manifest.md` · `judge_zone` 说明列 | 「差异登记与待决策见行为差异表「判定顺序」」 |
| `RULE-BUDGET-DOC-LAG` | 同基准表「预算/时间片」· 验证边界 | 「独立工作池未实现，命令配额阻塞及更广的策略质量仍待验收」 |

对照：覆盖清单行为差异表「判定顺序」与基准「语义选择」表（2026-10-16）均已写明**保留原生**；T13 PlanPool 证据目录（2026-10-14）已记工作池完成——声明表验证边界未跟上。

## 源码依据（只读核对）

### 判定顺序：非闪电 FIFO + 闪电最后

`crates/engine-native/src/kernel.rs`（`EventData::Phase { phase: Judge, begin: true }`）：

```text
// 判定排序分组（N06-a，对齐旧线 pending_delayed_cards）：
// 非闪电延时锦囊按放置序在前、闪电在后，队首先消费。
let mut queue: Vec<CardId> = judge.iter().copied()
    .filter(|c| card_kind(*c) != ShanDian).collect();
queue.extend(judge.iter().copied()
    .filter(|c| card_kind(*c) == ShanDian));
```

回归：`judge_order_delayed_first_then_lightning_hit_kills`、`judge_order_grouping_multi_cards_and_restore`（`crates/engine-native/tests/native_kernel.rs`）。  
上游对照：[judge-order-upstream](../2026-10-16-judge-order-upstream/README.md)。

### PlanPool：独立工作池已实现

`crates/engine-native/src/ai.rs`：

```text
/// T13 工作池：并行多座规划——固定 worker 线程分桶消费座位队列…
pub struct PlanPool { pub workers: usize }
impl PlanPool {
    pub fn plan_seats(&self, engine: &Engine, seats: &[u8], planner: &BoundedPlanner) -> Vec<Plan>
}
```

证据与回归：[t13-plan-pool](../2026-10-14-t13-plan-pool/README.md)（`t13_plan_pool_parallel_matches_sequential` 等）。  
余项：命令配额阻塞 / 策略质量仍属 `T13-AI-STRONGER-POLICY`，不属「工作池未实现」。

## 改后句子

### `native-rules-baseline.md` · 判定排序/闪电 · 验证边界

> 双放置序、多张 + 快照往返（`judge_order_grouping_multi_cards_and_restore`）及转移用例已有；**已定案保留原生**（非闪电 FIFO + 闪电最后）；与上游 LIFO 差异见语义选择表 / 覆盖清单；死亡剩余机器持牌路径不由终态测试完全覆盖

### `native-rules-baseline.md` · 预算/时间片 · 验证边界

> 单选候选合法回退原反例已于 v31 修复；独立工作池（`PlanPool`）**已实现**（2026-10-14）；命令配额阻塞及更广策略质量仍待验收（见 `T13-AI-STRONGER-POLICY`）；令牌原代次反例已修

### `native-coverage-manifest.md` · `judge_zone`（同口径最小改）

将「差异登记与待决策见行为差异表」改为「**已定案保留原生**，差异见行为差异表」（与行为差异表「判定顺序」行一致）。

## 核验摘录

```bash
# 改后应无匹配：
rg -n '是否改判上游待决策|独立工作池未实现|待决策见行为差异表' \
  docs/architecture/native-rules-baseline.md docs/native-coverage-manifest.md
# 改后应命中定案/已实现措辞：
rg -n '已定案保留原生|PlanPool.*已实现' \
  docs/architecture/native-rules-baseline.md docs/native-coverage-manifest.md
```

（本批提交前已对上述路径执行 `rg`，滞后三短语归零。）
