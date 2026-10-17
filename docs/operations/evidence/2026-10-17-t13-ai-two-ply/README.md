# 2026-10-17 T13 更强 AI 候选：两层搜索（T13-AI-STRONGER-POLICY，进行中）

**核验日期**：2026-10-17。**待做 ID**：`T13-AI-STRONGER-POLICY`（**保持 open**——本批交付候选与评估，但未观测到质量提升）。
**性质**：AI 候选实现 + 评估（无语义/版本/golden 变化；规则 v295 / 存档 v80）。

## 交付

1. **两层搜索候选**（`crates/engine-native/src/ai.rs`）：`BoundedPlanner` 增 `lookahead_depth`（默认 1=现状）与 `with_lookahead(2)`。两层模式在模拟我方候选后，再枚举**对手最优应对**（同 `enumerate_candidates`，最多 `MAX_LOOKAHEAD_CANDIDATES=8` 个）并取对我方最差结果作为该候选得分；节点预算/时限仍受控。候选枚举抽为 `enumerate_candidates`，单层与两层共用（单层行为不变）。
2. **评估接线**：`NativeGameRequest` 增 `planner_lookahead`（serde 默认 1），`create_native_game` 传入。
3. **评估脚本**：`policy_eval` 增 `two-ply-512` 策略；新增 `policy_head2head`（座位 0 用两层、其余单层 vs 全部单层，同 seed 比较座位 0 胜率/成本/终局体力）。

## 评估结果（同 seed）

`policy_eval`（4 seed × 800 步，[policy-eval.log](policy-eval.log)）：

| 策略 | 终局率 | 平均步数 | 平均耗时 |
|---|---|---|---|
| planner-heuristic（64） | 100% | 246 | 64ms |
| **two-ply-512** | 100% | 263 | 73ms |
| nextphase-baseline（0） | 0%（停滞） | 67 | 0ms |

`policy_head2head`（40 seed × 1200 步，[head2head.log](head2head.log)）：

| 臂 | 座位 0 胜率 | 平均步数 | 座位 0 终局体力 |
|---|---|---|---|
| control-all-shallow | 2%（1/40） | 229 | 0 |
| **seat0-two-ply** | 2%（1/40） | 228 | 0 |

## 结论（负结果）

- 两层搜索候选**可运行、确定性、受预算约束**（`two_ply_lookahead_deterministic_and_bounded`），成本可控（+9ms/局均）。
- **标准对称头对头未观测到质量提升**（座位 0 胜率 2% vs 2%，终局体力同为 0）。原因：当前启发式只读体力差，4 人标准局结果由座位/牌运主导；两层对手模型同用该启发式，通常不改变候选排序。
- 因此**不关闭** `T13-AI-STRONGER-POLICY`：本批交付了「深搜候选 + 即插即评」的骨架与负结果，但「更强」尚未证明。下一步需要**牌值/身份启发式**（如按将面/手牌质量/身份目标加权）或非对称对抗度量，属另批。

## 验证

- 定向：`two_ply_lookahead_deterministic_and_bounded`（engine-native/ai_planner）1/1。
- engine-native 全量 **881/0/0**；game-server 全绿；Gate 1b（`-D warnings --all-targets`）exit 0。
- 默认 `lookahead_depth=1` 与既有单层行为一致（无 golden/哈希变化）。
