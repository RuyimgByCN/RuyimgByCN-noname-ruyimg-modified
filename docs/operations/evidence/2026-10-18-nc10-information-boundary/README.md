# NC10 AI 合法信息边界：判据建立 + 一处确认泄漏（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC10**（[方案 §NC10](../../../architecture/native-convergence-plan.md#nc10)）第一切片：**信息边界判据 + 泄漏确证**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：判据已建立（3 项）；**发现并确证 1 处隐藏信息泄漏（未修，已登记）**；候选集合与公开面已验收不变

## 1 判据（与实现无关、可复跑）

新增 `crates/engine-native/tests/nc10_ai_information_boundary.rs`，用**隐藏信息置换**做判据：
把对手手牌 + 牌库内容做确定性旋转（保持公开视图：阶段/轮次/手牌数/体力不变），
则 AI 的决策必须逐字段不变。

| 用例 | 判据 | 结果 |
|---|---|---|
| `legal_action_set_is_invariant_under_hidden_information` | 权威动作查询的候选集合只由公开信息决定 | **通过** |
| `planner_detects_own_hand_change`（反向对照） | 替换自己一张手牌（内容变化）后规划结果**可**不同——证明探针有分辨力 | **通过** |
| `planner_is_invariant_under_hidden_information` | 规划结果不得依赖隐藏信息 | **失败（已确证泄漏）→ 标 `#[ignore]` 登记** |

## 2 确认的泄漏（NC10 缺陷，未修）

- **复现**：`cargo test --offline -p engine-native --test nc10_ai_information_boundary -- --ignored planner_is_invariant_under_hidden_information`
  → `seed=42022 warm=9 seat=1：规划结果依赖了隐藏信息`（[ignored-repro.log](ignored-repro.log)）。
- **成因（与本轮 NC12 §5.2 修复相关）**：`ai.rs::redact_opponent_hands` 现仅在**静止态**
  （栈空 + 无挂起机器 + 无开放选择）重分配隐藏牌——因为非静止态重分配会改写挂起机器/
  在飞帧引用的实体牌 id（NC12 §5.2 的 `take_hand_card` panic）。于是**非静止态下模拟评分
  可经结果侧信道读到对手手牌内容**（例如对手是否有【闪】）。
- **修复方向（待实施）**：
  1. 机器自省接口：导出「挂起机器/在飞帧引用的实体牌集合」，据此做**保引用重分配**（只置换未被引用的隐藏牌）——可在任意状态安全隐藏；
  2. 或非静止态**关闭模拟评分**（退化为公开信息启发式），并登记策略质量差异；
  3. 无论走哪条，都需保留 NC12 §5.2 的「不得 panic」不变量（本用例的隐藏置换必须不触发 panic）。

## 3 验证结果

```
cargo test --offline -p engine-native --test nc10_ai_information_boundary
cargo test --offline -p engine-native --test nc10_ai_information_boundary -- --ignored
cargo test --offline -p engine-native --no-fail-fast
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| NC10 判据（通过项） | **2 passed / 0 failed / 1 ignored** | [targeted.log](targeted.log) |
| 泄漏复现（ignored 用例） | 失败并打印 seed/warm/seat | [ignored-repro.log](ignored-repro.log) |
| engine-native 全量（含 examples） | **1133 passed / 0 failed / 3 ignored** | [engine-native-tests.log](engine-native-tests.log) |
| 行数门禁 | 8240 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办（NC10）

- [ ] **修隐藏信息泄漏**（上节两条方向，含机器自省接口或非静止态关闭模拟评分）
- [ ] 泄漏面**量化**：多少比例的决策在置换下改变（当前只证明「存在」）
- [ ] AI 受控模拟的其余口径：候选上限与分页、预算耗尽时的降级策略、取消/过期拒绝
- [ ] `ai.rs::redact_opponent_hands` 的确定性置换（LCG 播种）与「同种子同决策」一致性用例
- [ ] 长局内存/时间预算下的 AI 有界性（NC12 已给 p50；缺长局 AI 侧数据）
