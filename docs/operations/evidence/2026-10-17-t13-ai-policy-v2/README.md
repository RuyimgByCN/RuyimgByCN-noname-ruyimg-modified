# T13 更强 AI 候选：策略变体 1（全手牌候选 + 牌值评分）与应答路由修复（2026-10-17）

**核验日期**：2026-10-17。**待做 ID**：`T13-AI-STRONGER-POLICY`（本批交付可测质量提升 → 关闭）。
**性质**：AI 候选实现 + 评估 + 内核调度修复（1 场 golden 位移，无可观测语义变化；**无规则/存档版本变化**）。
**规则 v301 / 存档 v86**（不变）。

## 一、交付 1：策略变体 1（`BoundedPlanner::policy_variant`）

现状启发式（变体 0）只在出牌阶段枚举**【杀】**候选，评分只读体力差
（`(敌方体力下降) − (己方体力下降)`）——桃/无中/装备/锦囊永不出手。

变体 1（`with_policy_variant(1)`，默认仍为 0）：

| 面 | 变体 0（现状） | 变体 1 |
|---|---|---|
| 询问回退 | 每个单选项 ∪ `min≥2` 首 min 集合 ∪ 空应答 | 同上 + 小选项集（≤8）补「全选」组合 |
| 出牌候选 | 仅【杀】× 其他存活座位 | **全部手牌**：杀族 × 各目标；非杀牌先试自目标/无目标（桃、无中、装备、AOE），再逐目标试单目标形态；上限 96 条，杀优先占节点预算 |
| 评分 | 体力差 | 定点整数：己方体力 ×8、手牌 ×3、装备 ×4；敌方体力 ×8、手牌 ×1、装备 ×2；击杀 +24、己方死亡 −1000；模拟出终局 → 胜 +100000 / 非胜 −100000 |

手牌项天然抑制「白打一张牌」（出牌 −3，无收益即劣于 `NextPhase`），
摸牌/顺牵等净增手牌为正收益。非法候选由模拟提交拒绝（`Rejected` 不计分）。

代码：`crates/engine-native/src/ai.rs`（`policy_score` 变体分派、`enumerate_candidates_variant`）；
接线：`NativeGameRequest.planner_variant`（serde 默认 0，`apps/game-server/src/native.rs`）；
评估臂：`policy_eval` 增 `strong-v2-64`，`policy_head2head` 增 `seat0-strong-v2`/`all-strong-v2`。

## 二、质量证据（同 seed，对照）

`cargo run --release --offline -p game-server --example policy_head2head -- 40 1200`
（[head2head.log](head2head.log)，seed `8200..8239`）：

| 臂 | 座位 0 胜率 | 平均步数 | 座位 0 终局体力 |
|---|---|---|---|
| control-all-shallow（全座位现状） | **1/40 = 2%** | 229 | 0 |
| seat0-two-ply（512 节点两层，2026-10-17 负结果复跑） | 1/40 = 2% | 228 | 0 |
| **seat0-strong-v2（本批）** | **28/40 = 70%** | 248 | 2 |
| all-strong-v2（全座位变体 1） | 4/40 = 10% | 474 | 0 |

- 座位 0 单独用变体 1（**同 64 节点预算**，非资源倾斜）→ 胜率 2% → **70%**；
- 全座位都用变体 1 → 座位 0 回到 10%（无相对优势），但绝对胜率仍高于对照，
  且平均步数 474（对抗更激烈、对局更长）——说明变体 1 确实在打「有价值的
  牌/选择」，不是靠单侧资源。
- 成本：seat0-strong-v2 平均步数 +8%（229→248），`policy_eval` 平均耗时
  7ms→25ms/局（64 节点预算不变，候选更多）。

`cargo run --release --offline -p game-server --example policy_eval -- 4 800`
（[policy-eval.log](policy-eval.log)）：

| 策略 | 终局率 | 平均步数 | 平均耗时 |
|---|---|---|---|
| planner-heuristic（64） | 100% | 246 | 7ms |
| two-ply-512 | 100% | 263 | 9ms |
| **strong-v2-64** | **100%** | 380 | 25ms |
| nextphase-baseline（0） | 0%（停滞） | 67 | 0ms |

定向回归（`crates/engine-native/tests/ai_planner.rs`）：
`policy_v2_plays_value_card_and_is_deterministic`——0 号只持「无中生有」时
变体 0 只能 `NextPhase`、变体 1 出牌摸 2，两次规划逐字一致、计划可应用。

## 三、交付 2：应答路由修复（1 场 golden 位移）

变体 1 的全座位驱动在 seed 8211 第 110 步触发内核 panic
`answer present on resume`（`crates/engine-native/src/machine.rs:258`）。

**根因**（挂起机器按栈深恢复的失配）：`resolution::run` 用
`park.resume_depth == 栈深` 选择恢复对象，而该深度是机器**创建**时的值。
机器若在挂起期间发射过子帧（AOE 逐目标派发：前一目标的伤害帧尚未结算就
为下一目标开响应窗），等待应答时栈已更深——应答恢复时深度失配，落在
**深度恰好相等的另一台机器**上（本案：濒死救援机器消费了 AOE 响应应答），
原机器再恢复时 `ctx.answer()` 为空 → panic。

**修复**（`crates/engine-native/src/resolution.rs`）：机器返回
`Flow::WaitChoice` 时把恢复深度改写为**开窗时刻**的栈深
（开窗后对局暂停、栈不推进，故开窗时刻栈深即恢复点）。

**位移核验**（AGENTS.md 第 4 条：先查原因与语义，再决定重录）：

- 修复后 40 seed 全座位变体 1 驱动零 panic（`head2head` `all-strong-v2` 臂完成）。
- `nullify_chain_on_savage` 终态快照逐字段 diff（[nullify-snapshot-diff.txt](nullify-snapshot-diff.txt)）：**唯一差异 `resolution.budget`
  4072 → 4068**（调度步数少 4 步）；体力/手牌/弃牌/询问序列逐项相同
  （南蛮被无懈抵消→反无懈恢复→1 出杀免伤→2/3 各受 1 伤，`hp=[4,4,3,3]`）。
- 无规则版本/存档格式变化（无新字段）；仅该 1 场 golden 重录，重录依据写入
  `replay_scenarios.rs` 表头注释（[replay-golden.log](replay-golden.log)）。
- 回归守护：`policy_v2_drives_all_seats_without_answer_routing_panic`
  （seed 8211 全座位变体 1 → 218 步终局、胜方冻结；修复前此测试 panic）。

## 四、验证

- **engine-native 全量**：`cargo test --offline -p engine-native --no-fail-fast`
  **897 passed / 0 failed / 1 ignored**（exit 0；1 ignored＝既有诊断探针）
  ——[engine-native-tests.log](engine-native-tests.log)。
- **ai_planner**：15/15（含本批 2 个新测）。
- **game-server**：见 [game-server-tests.log](game-server-tests.log)。
- **Gate 1b**：独立 `CARGO_TARGET_DIR` 下
  `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0
  ——[gate1b-all-targets.log](gate1b-all-targets.log)。
- **14 golden**：`replay_scenarios` 2/2（1 场按上述依据重录）
  ——[replay-golden.log](replay-golden.log)。

## 五、结论

- `T13-AI-STRONGER-POLICY` **关闭**：更强候选（全手牌候选 + 牌值评分）
  交付并有同 seed 头对头质量提升（2% → 70%）、成本可复查；两层搜索
  负结果复跑一致（2% → 2%），登记为不采用。
- 附带修复内核应答路由缺陷（合法命令可致 panic），1 场 golden 因内部
  调度步数重录，无可观测语义变化、无版本变化。
- 余项：身份局/国战/对决等模式的按身份目标匹配与候选剪枝（本批为
  Prototype 模式度量），按需另批。
