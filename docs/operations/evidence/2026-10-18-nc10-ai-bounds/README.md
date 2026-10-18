# NC10 追加：泄漏面量化、确定性、预算降级、过期拒绝（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC10**（[方案 §NC10](../../../architecture/native-convergence-plan.md#nc10)）
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**4 项回归通过**；量化显示静止态 0/28 泄漏、非静止态窗口 0/4（样本内）；修复 1 处 AI 回退合法性问题与 1 处自身溢出缺陷

## 1 回归（新增 `crates/engine-native/tests/nc10_ai_bounds.rs`，4 项）

| 验收项 | 判据 | 结果 |
|---|---|---|
| **泄漏面量化** | 4 种子 × 8 热身点采样，隐藏信息置换后规划结果改变比例：**静止态 0/28**；**非静止态窗口 0/4**（该窗口是已知未完成面，本用例只量化、不断言为 0） | 通过 |
| 规划确定性 | 同状态重复 `plan_action` 结果一致（置换以座位+修订号播种，可重放） | 通过 |
| 预算降级 | 预算 0/1/2/4：**存在合法动作时**规划结果必须合法；无合法动作时允许占位（提交侧拒绝） | 通过 |
| 过期结果拒绝 | 旧修订号规划的动作用新修订号提交 → `Rejected(StaleRevision)` | 通过 |

## 2 过程中发现并修复的两处问题

1. **AI 回退动作可能非法**（`ai.rs`）：预算耗尽/候选全被拒时无条件返回 `NextPhase`，
   非本人回合或阶段不允许时会被内核拒绝。改为优先取**内核权威动作查询**的首个候选，
   查询为空时才退回 `NextPhase`（提交侧仍会校验）。
2. **`legal_actions` 去重高水位溢出**（`engine/legal_actions.rs`，本轮自身缺陷）：
   `probe_action` 用 `self.last_dedup + 1`，在既有用例
   `policy_and_plan_reject_cleanly_at_dedup_exhaustion`（高水位 = `u64::MAX`）下
   **算术溢出 panic**；改为 `saturating_add(1)`（饱和时探针以 `DuplicateCommand`
   被拒 → 表现为无候选而非 panic）。

## 3 验证结果

```
cargo test --offline -p engine-native --test nc10_ai_bounds -- --nocapture
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| NC10 追加定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | **1148 passed / 0 failed / 2 ignored** | [engine-native-tests.log](engine-native-tests.log) |
| 14 golden | **2/2 零位移** | [replay-golden.log](replay-golden.log) |
| 行数门禁 | 8250 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办（NC10）

- [ ] **非静止态窗口**：样本内 0/4 改变，但该窗口未做重分配（为保 NC12 §5.2 不变量），
      需「机器自省保引用重分配」或「非静止态关闭模拟评分」后才能声称边界完整
- [ ] 并发压力与长驻有界工作队列（当前 `BoundedPlanner` 为逐调用同步，未收敛为宿主工作池）
- [ ] 取消/超时路径的端到端用例（`cancelled()` 分支已存在，未覆盖）
- [ ] 不同模式/座位/种子的质量与尾延迟基线（NC12 已有 p50；缺质量面）
- [ ] 主动技/多选/多目标候选（依赖 NC03 候选面扩展）
