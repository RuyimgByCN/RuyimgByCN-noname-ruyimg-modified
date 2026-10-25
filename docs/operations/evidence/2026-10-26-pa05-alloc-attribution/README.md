# PA05 分配归因与首批热点改造：池克隆去日志化（2026-10-26）

- 日期：2026-10-26 04:50–05:25 +08:00
- 工作包：[台帐 PA05](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26)；PA02 残余（growth 末/首比）一并复核
- 源码：`HEAD = 7e28a937` 加未提交工作树；本批改动 `engine-native/src/engine.rs`（GameAction Ord）、`choice.rs`（ChoiceOption Ord）、`engine/legal_actions.rs`（去重键去字符串化）、`ai_pool.rs`（池克隆去日志化）。规则 **v322**、存档 **v105** 未变
- 测量：scratch archbench 新增 `allocbench`（隔离操作分配计数）与 `clonebench`（池路径量化）；growth 前后同程序同会话

**一句话结论**：归因显示**最大分配热点是 `Engine::clone` 的日志复制**（`LogEvent` 内嵌 Vec——每事件一条堆分配；4 座 log≈509 时 198 次/克隆、8 座 log≈1021 时 386 次/克隆），命中生产路径 **ai_pool 每座位克隆**：改用 `clone_for_simulation`（日志从空开始）后**每轮规划 792→36 次（4 座 −95%）、3088→96 次（8 座 −97%）**；`legal_actions` 去重键去字符串化（`GameAction: Ord`）。**growth 同负载每步分配持平（184/275，无回归但未降）**——该负载不经过池路径，剩余分配为探针克隆/提交克隆的结构性成本（非变异校验入口是 legal_actions.rs 头注已登记的 NC12 待办）。PA05 按台帐完成条件判为**部分完成**，如实登记。

## 1 归因（allocbench，定态隔离计量，均值次/调用）

| 操作 | 4 座(log 12→509) | 8 座(log 1021) | 说明 |
|---|---:|---:|---|
| `Engine::clone`（derive，含日志） | 15 → **198** | **386** | 随日志线性增长：`LogEvent` 各变体内嵌 `Vec`（cards/targets/redirected_from…），每事件 1 次堆分配 |
| 同状态空日志引擎 `.clone()` | 9 | 12 | restore 后克隆 ≈ `clone_for_simulation` 外部等价 |
| `legal_actions(seat)` | 21–23 | 34–35 | 主体是每候选 `probe_action`（模拟克隆+提交，PA02 已去掉哈希/存档/日志副作用） |
| `state_hash()` | **1** | **1** | PA03 缓冲化后仅一次 Vec 分配 |
| `ai_session.view()` | 3 | 3 | |
| `plan_action(seat)` 全程 | 54–59 | 74 | sim 克隆 + 每节点 trial 克隆 + 视图×2 + 回退 legal_actions |

原始输出：[allocbench-4.txt](allocbench-4.txt)、[allocbench-8.txt](allocbench-8.txt)、[clonebench-4.txt](clonebench-4.txt)、[clonebench-8.txt](clonebench-8.txt)。

## 2 改造

1. **`ai_pool.rs` 池克隆去日志化**：每座位任务克隆 `engine.clone()` → `engine.clone_for_simulation()`。规划不读日志（PA02 A/B 已证模拟会话克隆的规划行为与全量克隆逐行一致；Plan 产物不含日志），池轮收益见上。服务端多座位 AI 驱动（game-server 原生房间）为直接受益路径。
2. **`legal_actions` 去重键**：`BTreeSet<String>`（每候选一次 `format!("{action:?}")` 分配）→ `BTreeSet<GameAction>`（`GameAction`/`ChoiceOption` 补 `PartialOrd, Ord` 派生，纯 additive），`contains` 借用检查 + 仅接受后克隆入集。
3. 台帐点名的 `BTreeMap<(u8, String), _>` 字符串键与每触发 `Box<dyn SkillMachine>`：归因显示原型局中非主导（skills 表为空、触发少）；随真实目录扩展再评。

## 3 前后数据

| 负载 | 前 | 后 |
|---|---:|---:|
| growth 4 座 allocs/step | 184（[PA03 证据](../2026-10-26-pa03-hash-buffered/growth-pa03-4.txt)） | **184**（持平，[growth-after-4.txt](growth-after-4.txt)） |
| growth 8 座 allocs/step | 275 | **275**（持平，[growth-after-8.txt](growth-after-8.txt)） |
| 池每轮分配 4 座（log 509） | 792 | **36（−95%）** |
| 池每轮分配 8 座（log 1021） | 3,088 | **96（−97%）** |

PA02 残余复核：growth 末/首每节点比仍 1.5x/1.3x 量级（状态固有结算复杂度，PA02 证据 §3 结论不变，本批未触及）。

## 4 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --no-fail-fast` | **1558 passed / 0 failed / 71 ignored**（含 nc10_planner_pool、nc03_legal_actions 定向 10/0）；golden 不变 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` / `check-source-lines.py` | OK / 通过 |
| Gate 1b | 同前批口径（他人 rule-vm 在途改动既有警告阻塞；本批文件零警告） |

## 5 未完成（PA05 剩余）

1. **growth 同负载每步分配下降**未达成（持平）：剩余 184/275 的构成是 sim/trial/提交工作副本的 GameState 深拷贝（每处 ~9–15 次）与 legal_actions 探针克隆——下限改造需**非变异合法性校验入口**（legal_actions.rs 头注已登记的 NC12 待办）或状态 Arena 化，均超出本批范围；
2. `BTreeMap<(u8,String),_>` 字符串键的实目录规模测量（真实内容包下手牌/标记量级）未做；
3. 池改造的端到端收益（game-server 多房间并发下的分配/延迟）未测——建议随 NC12 多房间 p95/p99 一并。
