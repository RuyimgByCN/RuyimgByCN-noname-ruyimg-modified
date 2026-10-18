# NC12 长程缺陷：AI 规划模拟 panic → **已加防护并验证**（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC12**（[方案 §NC12](../../../architecture/native-convergence-plan.md#nc12)）+ **NC10** AI 面
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**缺陷根因仍在（内核不变量在模拟状态上被触发）**；**进程级崩溃已消除**（模拟 panic 被捕获、计数、跳过该候选）；长程负载**已可继续跑**

## 1 缺陷与定位

长程负载（`nc12_bench long tier=swd budget=8 min_cmds=20000000`）在 **4,327** 条 `RESULT` 后崩溃：

```
thread 'main' panicked at crates/engine-native/src/machine.rs:408: card 93 not in hand of 0
```

`RUST_BACKTRACE=1` 回溯（[catch-panics.log](catch-panics.log)）：

```
engine_native::kernel::commit_frame
engine_native::engine::commands::Engine::finish_segment
engine_native::engine::commands::Engine::submit
engine_native::ai::Engine::plan_action          ← 模拟提交（克隆上）
nc12_bench::plan_with_catch → drive_game → scenario_long → main
```

→ 崩溃发生在 **`plan_action` 的模拟提交**（对克隆引擎 `submit`）中；与仓库此前修复的
「`redact_opponent_hands` 重分配实体牌 id 致挂起机器引用失效」**同类但不同路径**（该修复已使
身份遮蔽仅在静止态、非静止态仅打乱牌库顺序）→ **说明仍有未覆盖的模拟一致性路径**（根因待查）。

## 2 处置（本轮）：模拟故障不得杀死进程

`crates/engine-native/src/ai.rs`：

- 模拟提交包在 `std::panic::catch_unwind(AssertUnwindSafe(..))` 中；
- 捕获后 `sim_faults += 1` 并 `continue`（该候选不计分）；
- **真实提交路径不捕获**——内核不变量在真实引擎上仍硬失败（不掩盖真实状态损坏）；
- `Plan` 新增 `sim_faults: u32` 字段（规划结果自带故障计数）；
- `nc12_bench` 长程行新增 `sim_faults=` 统计（**不静默**：故障数进入结果）。

## 3 验证（[long-load-guarded.log](long-load-guarded.log)、[after-guard.log](after-guard.log)）

| 判据 | 结果 |
|---|---|
| 进程不再崩溃（原先 4,327 行后 panic） | **通过**：加防护后同参数连续运行，`RESULT` 行持续增长（60 秒内 6,221 行，随后继续） |
| 模拟故障被**计数**而非静默 | **通过**：`sim_faults=1` 出现 1 次，其余为 `sim_faults=0`（`grep` 统计） |
| 真实提交路径仍硬失败 | 未改动该路径（源码事实：捕获只包在 `plan_action` 的模拟 `submit` 外） |

> 长程（`min_cmds=20000000`，目标 ≥30 分钟）本轮**已在后台启动**，结果与内存/时延漂移
> **下一轮收集**；本轮**不宣称 ≥30 分钟已验收**。

## 4 未完成 / 待办

- [ ] **根因**：定位「模拟状态下 `take_hand_card` 找不到牌」的具体路径（候选：遮蔽重分配仍与
      某些引用面冲突 / 模拟中 `table`/`turn.*` 携带的牌 id 未随遮蔽更新）——需要最小复现
      （固定种子 + 命令序号 + 崩溃前状态快照）
- [ ] 收集并分析 ≥30 分钟长程结果（内存/句柄/时延随时间漂移）
- [ ] 其余平台档位、真实网络 RTT、旧线对照

## 5 追加：第二处崩溃路径（候选探测）已加防护（2026-10-19 同轮）

加防护后重跑，进程仍在 **7,027** 行后崩溃 → `RUST_BACKTRACE=1` 取得**第二条**回溯
（[backtrace-2.log](backtrace-2.log)）：

```
engine_native::kernel::commit_frame
engine_native::engine::commands::Engine::finish_segment
engine_native::engine::commands::Engine::submit
engine_native::engine::legal_actions::push_action     ← 候选探测（克隆上 submit）
```

即**候选探测**（`probe_action` = 克隆 + 真实 `submit`）同样会触发内核不变量 panic。

**处置**（`crates/engine-native/src/engine/legal_actions.rs`，该文件**已被跟踪**、可独立提交）：

- `probe_action` 调用包在 `catch_unwind(AssertUnwindSafe(..))` 中；
- 捕获后视为「该动作**不合法**」（不计入候选）并累加全局计数
  `legal_actions::probe_faults()`（`AtomicU64`，诊断可读）；
- **真实提交路径不捕获**（内核不变量在真实引擎仍硬失败）。

**验证**（[long-load-guarded2.log](long-load-guarded2.log)）：

| 判据 | 结果 |
|---|---|
| 同参数长程不再中断 | **通过**：日志持续增长（12,641 → 17,477 `RESULT` 行，30 秒增量 4,836 行） |
| 被捕获的 panic 不再杀死进程 | **通过**：日志中出现 2 次 panic 文本（默认 hook 打印）但进程继续运行 |
| 定向回归 | `nc03_legal_actions` 4/4、`nc10_ai_bounds` 3/3（本文件）、`nc08_upstream_ciqiu` 5/5 通过 |

> 长程（`min_cmds=20000000`，目标 ≥30 分钟）**正在后台运行**（04:13 启动）→ **结果下一轮收集**；
> 本轮**仍不宣称 ≥30 分钟已验收**。

### 仍未完成（更新）

- [ ] **根因**：为何「克隆 + 遮蔽」后的模拟状态会让 `take_hand_card` 找不到牌（最小复现待做）
- [ ] 收集 ≥30 分钟长程结果（内存/句柄/时延漂移）与 `probe_faults()`/`sim_faults` 计数
- [ ] 是否将两类故障计数接入产品遥测（当前仅测试/诊断可读）
- [ ] 其余平台档位、真实网络 RTT、旧线对照

## 6 追加：根因方向修正尝试（公开区/在飞区不得被遮蔽重分配）（2026-10-19 同轮）

**假设**：`redact_opponent_hands` 的填充池此前只排除「自己的区域 + 弃牌堆」，于是**对手的
装备区/判定区/离场区**、`table`（在飞牌）与**回合级持牌字段**可能被当作隐藏牌抽走 → 被机器/
帧引用的实体牌 id 失效 → `take_hand_card` panic。

**已实施**（`crates/engine-native/src/ai.rs`）：把这些区域与字段加入 `visible` 集合，使其**永不
进入填充池**（`turn.huanxia_track`、`turn.kunlunjing_snapshot` 的牌亦登记）。

**实测结果（如实）**：

| 判据 | 结果 |
|---|---|
| 长程是否不再出现模拟故障 | **未消除**：同参数长程仍出现 `sim_faults`（`guarded2`：57,261 行 / 7 次故障；`guarded3`：15,445 行 / 1 次） |
| 是否引入回归 | 一个用例失败并已按规重录：`policy_v2_drives_all_seats_without_answer_routing_panic` 步数 **179 → 484**（seed 8211；胜方断言不变） |
| 全量回归 | **1184 passed / 0 failed / 2 ignored** |

**结论（不夸大）**：该改动是**一致性上的正确收紧**（公开区/在飞区本就不该被重分配），但**不是**
本次 panic 的根因修复——根因仍在别处（下一步方向：把故障发生时的**最小状态**（`table`/`turn`
字段/挂起机器引用集合）导出并比对，或直接在 `take_hand_card` 缺失时改为返回 `Result` 并在
模拟侧降级）。

### 仍未完成（更新）

- [ ] panic 根因最小复现（导出故障时的状态切片）
- [ ] ≥30 分钟长程结果（`long-load-final.log` 运行中，04:25 启动）与故障率统计
- [ ] AI 强度面评估：填充池收窄导致的对局变长（179→484）是否可接受，或需重新设计填充策略
- [ ] 其余平台档位、真实网络 RTT、旧线对照

## 7 关键诊断：故障**不是**遮蔽造成的（2026-10-19 同轮）

给模拟/探测故障加了**现场状态切片**与**遮蔽时静止态诊断**后重跑，抓到一次完整现场
（[fault-dump2.log](fault-dump2.log)）：

```
card 93 not in hand of 0
AI_SIM_FAULT seat=1 redact_quiescent=false redact_stack=1 redact_parks=1 redact_choice=true
  rev=10 phase=Play action=Answer { choice_id: 2, selection: [Pass] } table=[161]
  deck_len=194 discard_len=3
  p0:hp=3/4 hand=[78, 152, 123, 144] equips=[] judge=[] off=[]
  p1:hp=4/4 hand=[23, 15, 72, 10] ... p2 ... p3 ...
```

**判读（重要）**：

1. 遮蔽时 **`redact_quiescent=false`（stack=1、parks=1、choice=true）** → 身份遮蔽**已被正确跳过**，
   只做了牌库顺序打乱（不打乱区域归属）；
2. 但 panic 仍是 `card 93 not in hand of 0`，而切片显示 **p0 手牌 = [78,152,123,144]**，
   且 93 不在任何列出的区域（`table=[161]`、`deck_len=194`、`discard_len=3`）→ **93 在牌库或弃牌堆**；
3. 打乱牌库**不会**改变牌的**区域归属** → 因此**卡 93 在遮蔽之前就已不在 p0 手牌**，
   而某台机器仍以它为「手牌牌」引用 → **这是内核自身状态里的悬空引用**，与 AI 遮蔽**无关**；
4. 触发动作是 `Answer { choice_id: 2, selection: [Pass] }`（应答路由），发生在 **rev=10** 的早期局面
   —— **现场足够小，可据此构造最小复现**。

**下一步（明确）**：
- [ ] 从该切片出发构造最小复现：按 seed/game 重放到 rev=10（或直接用 `debug_*` 布阵复刻
      「p0 手牌 4 张、table=[161]、某机器引用 93」），定位是哪台机器/哪条路径把 93 移出手牌却保留引用；
- [ ] 修复后重跑长程，确认 `sim_faults`/`probe_faults` 归零；
- [ ] 该缺陷同时解释了此前「长程崩溃」与两处防护的必要性（防护保留：真实提交仍硬失败）。

## 8 合规修正：诊断输出不得进入生产源码（2026-10-19 同轮）

§7 的现场切片曾以 `eprintln!` + `NONAME_AI_FAULT_DUMP` 环境开关实现 → **被仓库既有维护门禁
拦下**（`crates/engine-native/tests/maintenance_guards.rs::production_sources_have_no_debug_output_or_env_switches`
禁止生产源码出现 `println!`/`eprintln!`/`env::var(`）。

**处置**：移除生产源码中的诊断输出与环境开关；诊断信息改由**计数器**承载：

- `Plan.sim_faults`（模拟提交被捕获的 panic 数）；
- `legal_actions::probe_faults()`（候选探测被捕获的 panic 数）；
- `ai::redact_last_state()`（最近一次遮蔽的静止态判据，纯 getter、无输出）。

现场切片内容（§7 引用）来自**修正前**的运行日志，仍作为诊断证据保留（其结论不依赖输出形式）。

**回归**：`maintenance_guards` 通过；全量 **1184 passed / 0 failed / 2 ignored**。

> 教训（写入证据）：新增诊断**先查维护门禁**（`tests/maintenance_guards.rs`），生产源码只允许
> 计数器/返回式诊断，输出与开关须留在测试或工具侧。
