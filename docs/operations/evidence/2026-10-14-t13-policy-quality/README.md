# T13 策略质量候选评估（2026-10-14）

## 任务来源与口径

[进度主稿 T13](../../../architecture/native-progress.md) AI 项「策略质量」——原生线 `BoundedPlanner` 启发式（敌方体力下降−己方体力下降，64 节点/50ms）对照候选基线的可重复评估。

## 评估装置

- `NativeGameRequest` 新增 `planner_nodes` 参数（serde 默认 64=现状；0=无规划基线——候选预算即耗尽，动作退化为 NextPhase 兜底）。产品路径不变（默认 64）。
- 评估器：`apps/game-server/examples/policy_eval.rs`——同 seed 集（8100–8109，4 人身份局标准目录）× 双策略 × max_steps=1200，key=value 输出。

## 结果（[policy-eval-10seeds.txt](policy-eval-10seeds.txt)，release）

| 指标 | planner-heuristic（现状） | nextphase-baseline（无规划） |
|---|---|---|
| 终局率 | **100%（10/10）** | **0%**（10/10 全部停滞退出） |
| 平均步数 | 235 | 67（停滞判据触发） |
| 平均耗时 | 6ms/局 | ≈0ms |
| 胜方分布 | Player(0)×1 / Player(2)×4 / Player(3)×5 | — |

## 结论

- 启发式规划器为**对局可进行的必要组件**：无规划基线在 67 步内全部停滞（观察者挂起无法被有意义应答，连续拒绝计数触发停滞退出）——「策略质量」的下界确立。
- 现状启发式的质量刻画：10/10 终局、无停滞、胜方跨座位分布（非单点过拟合）。
- **登记**：更强的候选策略（更深搜索/MCTS/多候选评估）与胜率对身份目标的匹配度（忠反内 Побед 判定）为后续立项面——评估装置（`planner_nodes` 参数+本例）已就绪，新候选即插即评。评估结论不预设「质量已够」——仅确立基线差距与装置。

## 验证

- game-server 全量 **47/0/0**（含 T6③ 3 测+T8 增量 4 测+本批参数化的 2 测）——[game-server-tests.log](game-server-tests.log)。
- 评估器改动面：native.rs 参数化（默认行为不变，T6③/T8 测试原值通过）；无内核/规则改动。
