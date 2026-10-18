# NC03 权威动作查询：合法动作只由内核定义（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC03**（[方案 §NC03](../../../architecture/native-convergence-plan.md#nc03)）内核侧切片
- 规则版本 v321 / 存档 v105（无规则与格式变化）
- 结论：**内核权威动作查询已实现 + 4 项回归通过 + 全量通过**；前端/AI 接线与协议共享待续

## 1 缺陷（源码依据）

- 前端：按 `hand.slice(0, 8)` 拼接 `PlayCard`、合成 `ResolveChoice`（NC03 前端切片报告已列）；
- AI：`crates/engine-native/src/ai.rs::enumerate_candidates_variant` 在视图上镜像规则枚举候选，注释即「非法由模拟拒绝」；
- 结果：**合法动作的定义散落在内核之外**（客户端与 AI 各一套），可能给出非法动作或漏掉合法动作。

## 2 实现（`crates/engine-native/src/engine/legal_actions.rs`）

| 接口 | 语义 |
|---|---|
| `Engine::legal_actions(seat)` | 该座位此刻可提交的候选动作，**顺序稳定**（应答 → 主动技 → 出牌 → 阶段推进），有上限 `MAX_LEGAL_ACTIONS = 256` |
| `Engine::is_legal_action(seat, &action)` | 单动作是否可提交（内核校验） |
| `Engine::probe_action(seat, &action)` | 非变异校验探针：克隆引擎走**与正式提交相同的 `submit` 路径**，返回 `RejectReason` |

权威性来源：候选先由视图粗筛，再逐个经 `probe_action`（同源校验）过滤——**查询与提交共用一处合法性定义**。

## 3 回归（新增 `crates/engine-native/tests/nc03_legal_actions.rs`，4 项）

| 验收项 | 用例 | 结果 |
|---|---|---|
| 射程外目标不出现、相邻目标出现，且与内核拒绝一致 | `legal_actions_respect_attack_range` | 通过（5 人局：0 号【杀】候选只含 1/4；`vec![2]` 提交被拒） |
| 查询只读（修订号/摘要不变） | `legal_actions_do_not_mutate_engine` | 通过 |
| 非本人回合无出牌候选 | `legal_actions_empty_for_other_seat` | 通过 |
| 开放选择：只给观察者应答候选，每项可被内核接受 | `legal_actions_cover_open_choice` | 通过 |

## 4 验证结果

```
cargo test --offline -p engine-native --test nc03_legal_actions
cargo test --offline -p engine-native --no-fail-fast
CARGO_TARGET_DIR=<tmp> RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --all-targets
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| NC03 内核切片定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量 | **1108 passed / 0 failed / 2 ignored** | [engine-native-tests.log](engine-native-tests.log) |
| Gate 1b | exit 0 | [gate1b-all-targets.log](gate1b-all-targets.log) |
| 行数门禁 | 8225 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 5 未完成 / 待办（NC03）

- [ ] **前端接线**：`nativeBackend.ts` 改用权威查询（当前仍 `hand.slice(0, 8)` 拼接），删除合成 `ResolveChoice`
- [ ] **AI 接线**：`ai.rs::enumerate_candidates_variant` 改用 `legal_actions`（消除规则镜像；需先测性能，克隆校验的代价见 NC12）
- [ ] **协议共享**：Rust↔TS 协议定义共享或自动校验（当前手写两份）
- [ ] `events` / `hash_hex` 空占位补齐（NC03 前端切片已列）
- [ ] 候选空间上限与分页/分步接口（超大局面）；主动技候选未展开技能内部选择
- [ ] 真实 Worker/浏览器端到端、多选/取消/过期选择、大整数表示

## 6 代价测量与 AI 接线决策（2026-10-18 追加）

| 项 | 实测 |
|---|---|
| `legal_actions(0)`（5 人局出牌阶段，3 候选） | **1.62 ms/次**；**0.54 ms/候选**（克隆校验为主） |
| AI 采用该查询的估算 | 变体 1 约 30 候选/节点 × 0.54 ms ≈ **16 ms/节点** → 64 节点约 1 s/决策（不可接受） |
| 前端采用该查询 | 点击触发一次 ≈ 1.6 ms（可接受） |

**决策**：AI 暂**不**改用克隆式查询（性能原因，登记为待办：非克隆校验入口 / 分步动作构建接口）；改为加**一致性回归**保证可观测契约——规划器产出的动作必须通过 `is_legal_action`。

新增 `crates/engine-native/tests/nc03_ai_consistency.rs`（2 项）：

| 验收项 | 用例 | 结果 |
|---|---|---|
| 多种子/预算/变体下规划动作均通过权威校验 | `planner_actions_pass_authoritative_validation`（3 种子 × 2 配置） | 通过 |
| 反向探针：伪造动作（卡 id 9999/座位 9）必被拒绝（校验非恒真） | `forged_action_is_rejected_by_authoritative_validation` | 通过 |

实测：定向 AI 一致性 **2/2**；engine-native 全量 **1110/0/2**；Gate 1b exit 0；行数门禁 8226 文件 ≤2000 行。
