# PA05 分配归因与热点改造：池克隆去日志化 + 惰性段检查点 + 保守快拒（2026-10-26）

- 日期：2026-10-26；数据文件落盘 03:45–04:02、首次提交 03:46（037529c8）+08:00；收尾批提交 04:03（2bcd6088）（原记「2026-10-26 04:50–06:10 +08:00（收尾批 05:40–06:10）」晚于提交或与文件时间矛盾，2026-10-26 17:30 扫描按数据文件时间与提交历史更正）
- 工作包：[台帐 PA05](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26)；PA02 残余（growth 末/首比）一并复核
- 源码：`HEAD = 3eb67194` 加未提交工作树（原记 7e28a937；按提交历史更正为本批提交的父提交）；本批改动 `engine-native/src/engine.rs`（GameAction Ord）、`choice.rs`（ChoiceOption Ord）、`engine/legal_actions.rs`（去重键去字符串化）、`ai_pool.rs`（池克隆去日志化）。规则 **v322**、存档 **v105** 未变
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

## 5 收尾批（同日）：惰性段检查点 + 保守快拒 → growth 实测下降

上节登记的「growth 每步持平」在本日收尾批解决，完成条件达成：

1. **惰性段检查点**（engine/commands.rs）：`time_slice == None`（默认）时段不可能 `Yielded` 暂停，`submit` 里预计算的 `save()` 全量快照在任何结果臂都不会被消费——改为该条件下跳过（`if self.simulation || self.time_slice.is_none() { None } else { Some(self.save()) }`）。game-server 不调用 `set_time_slice`（grep 证实），无行为面。
2. **保守快拒**（engine/legal_actions.rs + play_card.rs）：`play_card` 开头的纯只读校验段（授权/阶段/封锁/限用/手牌持有/目标界与目标禁指）抽取为 `common_play_validation`——`play_card` 自己调用同一段代码（结构性防漂移）；`push_action` 先跑 `certainly_illegal`（NextPhase 回合校验 / PlayCard 共享前缀 / Answer 三段纯校验），**只**对「与正式校验同代码判定必拒」的候选跳过探针克隆——接受集与纯探针实现单调一致，其余拒绝面（牌种分派校验、结算期失败）仍由探针兜底。

**行为不变性**：固定种子 4/8 座 **722/1718 行 (seed,step,action,nodes) 与 PA02 基线（即全部性能批之前的原始行为）逐行一致**（[probe-pa05-4.txt](probe-pa05-4.txt)、[probe-pa05-8.txt](probe-pa05-8.txt)，对比 [PA02 probe-ab](../2026-10-26-pa02-ai-sim-session-split/README.md) §2）。

| 负载 | 收尾批前 | 收尾批后 |
|---|---:|---:|
| growth 4 座 allocs/step | 184 | **175（−4.9%）**（[growth-closeout-4.txt](growth-closeout-4.txt)） |
| growth 8 座 allocs/step | 275 | **262（−4.7%）**（[growth-closeout-8.txt](growth-closeout-8.txt)） |
| 全弧线（PA02 前 → 今） | 539 / 1136 | **175 / 262（−68% / −77%）** |

验证：engine-native 全量 **1558/0/71**（定向 nc03_legal_actions/nc03_ai_consistency/ai_planner/nc09_identity_end_to_end/nc10_planner_pool/nc10_ai_bounds 共 36/0）；行数门禁通过；wasm 编译 exit 0。**game-server 套件当前无法编译**：engine-core（旧线）存在其他 agent 未提交在途改动（`resolve_choice/*` 等）的 3 个编译错误，与本批无关（本批五次提交均未触及 engine-core；game-server 不用时间片、legal_actions 输出不变，无行为面）——如实记录，待该在途批修复后可复跑。

## 6 剩余（登记为后续）

1. 每步分配的进一步下限仍是 sim/trial/提交工作副本的 GameState 深拷贝（~9–15 次/处）——需状态 Arena 化或探针免克隆化（状态级 submit 入口），超出 PA05「改造热点」范围；
2. `BTreeMap<(u8,String),_>` 字符串键的实目录规模测量未做；
3. 池改造端到端收益（game-server 多房间并发）建议随 NC12 多房间 p95/p99 一并。
