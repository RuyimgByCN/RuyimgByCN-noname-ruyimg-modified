# NC12/N10 AI 强度基线（策略面对局长度与终局率）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC12**（[方案 §NC12](../../../architecture/native-convergence-plan.md#nc12)）+ **NC10** AI 面
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**同参数下 200 局全部终局，步数 p50=321 / p90=413 / max=464，故障 0** —— 作为策略面基线固定

## 1 口径

- 用例：`crates/engine-native/tests/nc12_ai_strength_baseline.rs`；
- 参数：swd 档、4 座、`BoundedPlanner::new(8).with_deadline_ms(50).with_lookahead(1).with_policy_variant(0)`
  （**与长程负载同参数**）；
- seed：45000–45199（200 局）；每局步数上限 1200；
- 统计：步数分布（p50/p90/max）、终局率、故障数（`Plan.sim_faults` + `legal_actions::probe_faults()`）；
- 断言：**故障必须为 0**、终局率 ≥ 50%（步数只统计、不断言——步数是策略面观测）。

## 2 实测（[baseline.log](baseline.log)）

```
AI_BASELINE n=200 finished=200 steps_p50=321 steps_p90=413 steps_max=464 faults=0
```

| 指标 | 值 |
|---|---|
| 局数 / 终局 | 200 / **200**（100%） |
| 步数 p50 | **321** |
| 步数 p90 | **413** |
| 步数 max | **464** |
| 故障数 | **0** |

## 3 与单局冻结值的区分（避免误读）

- `ai_planner::policy_v2_drives_all_seats_without_answer_routing_panic`（seed **8211**，budget **64**）
  是**单局冻结值**：2026-10-19 遮蔽填充池收紧后由 **179 → 484**（已按规重录，胜方不变）；
- 本基线用的是**另一批 seed**（45000–45199）与**另一组参数**（budget 8 / lookahead 1）→
  **两者不可直接比较**；本基线的作用是给出「当前策略在同参数下的分布」，供后续判断
  填充策略是否需重新设计（例如 p50 是否显著恶化）。

## 4 未完成 / 待办

- [ ] 在**同一批 seed** 上做「收紧前 vs 收紧后」的对照（需临时还原旧填充池，或新增可切换参数），
      以量化填充策略收紧对 AI 强度的影响；
- [ ] 胜方分布（当前只统计终局率）；
- [ ] 与旧线 AI（`engine-ai`）的同口径对照（NC12 旧线对照的一部分）；
- [ ] 内核悬空引用修复后重跑本基线（应保持 `faults=0` 且分布不劣化）。
