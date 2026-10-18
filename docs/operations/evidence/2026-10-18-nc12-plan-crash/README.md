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

## 9 最小复现：**确定性命中**（seed 44170 / 第 10 步 / 无懈窗口）（2026-10-19 同轮）

新增两个测试文件（测试侧允许输出）：

- `crates/engine-native/tests/nc12_dangling_reference_probe.rs`：按长程同参数（swd 档、4 座、
  variant 0 / budget 8 / lookahead 1）扫描 seed 序列 → **命中**；
- `crates/engine-native/tests/nc12_dangling_reference_repro.rs`：把命中局面固定为
  **seed 44170，第 10 步**（seat 1 回答 choice 2），逐步打印现场。

**命中现场**：

```
[repro] step=10 seat=1 命中 sim_faults=1
[fault] rev=10 phase=Play turn=1 stack=1 parks=1 choice=Some((2, "nullify_window")) table=[161]
[fault]   p0: hp=3/4 hand=[78, 152, 123, 144] equips=[] judge=[] off=[]
[fault]   p1: hp=4/4 hand=[23, 15, 72, 10] ... p2 ... p3 ...
[fault]   park[0] resume_depth=1 slot=Intrinsic { frame: 23 }
         snapshot=NullifyWindow { card: 161, source: 1, nullified: false, queue: [1, 2, 3, 0],
           asked: true, window_open: true, owns_disposal: true,
           effect: Some(Host { def_key: "swd_zhongxinghujia", behavior: 228, stage: 0,
             owner: 1, target: Some(0), card: Some(161), cards: [], targets: [] }) }
```

**判读（定位到具体机器）**：

1. 故障发生在**无懈可击窗口**（`choice_id=2`，prompt `nullify_window`）的应答路径上；
2. 挂起机器是 `NullifyWindow`，其内层效果为 **`swd_zhongxinghujia`**（swd 众星护甲，行为 228），
   `owner=1`、`target=0`、`card=161`；
3. panic 为 `card 93 not in hand of 0`，而现场 p0 手牌 = `[78,152,123,144]` → **93 已不在 p0 手牌**；
4. 外层快照只持有 `card=161`，**93 出现在内层效果机器自身的阶段状态里** → 即
   **该效果机器（或其窗口包装）跨窗口保留了会失效的手牌牌引用**。

**下一步（明确、可执行）**：

- [ ] 检查 `swd_zhongxinghujia` 效果机器（`content/host/step/*`，行为 228）在
      `HostStage` 各阶段对牌 id 的持有方式：窗口期间牌被移动/弃置后，机器再次 `take_hand_card`
      即 panic → 应改为「按需重查（重新解析目标手牌/弃牌堆）」或在使用前校验存在性并降级；
- [ ] 修复后本复现用例（seed 44170）应保持 `faults == 0`，并重跑长程确认
      `sim_faults` / `probe_faults` 归零；
- [ ] 该文件属**未跟踪的 host 拆分目录**（`content/host/` 仍为 `??`）→ 修复需待其落定，
      或与拆分作者协调后一并提交（登记）。

## 10 根因链闭合：`commit_card_used` 的 else 分支 + 事件栈现场（2026-10-19 同轮）

在复现用例中加入**事件栈**打印后，现场完整（[repro 输出](../../../operations/evidence/2026-10-18-nc12-plan-crash/README.md)）：

```
[repro] step=10 seat=1 命中 sim_faults=1
[fault] frame[0] id=23 parent=None kind=CardUsed stage=Intrinsic
        data=CardUsed(CardUseData { card: 161, player: 1, targets: [0], nature: Normal,
                                    use_as: None, cancelled: false })
[fault] park[0] snapshot=NullifyWindow { card: 161, source: 1, queue: [1,2,3,0], window_open: true,
        effect: Some(Host { def_key: "swd_zhongxinghujia", behavior: 228, stage: 0,
                            owner: 1, target: Some(0), card: Some(161) }) }
panic: card 93 not in hand of 0
```

**根因链（已闭合到代码行）**：

1. 崩溃点是 `crates/engine-native/src/kernel/events.rs::commit_card_used`：
   ```rust
   if ctx.core.player(u.player).hand.contains(&u.card) {
       ctx.take_hand_card(u.player, u.card);
   } else if ctx.core.table.contains(&u.card) {
       ctx.core.table.retain(|c| *c != u.card);
   } else {
       // 保持原 panic 语义（调用方先验证）。
       ctx.take_hand_card(u.player, u.card);   // ← panic 在此
   }
   ```
   → 当某次 **用牌提交**的牌**既不在手牌也不在桌面**时，走 else 分支硬 panic；
2. 现场显示栈上只有一张 `CardUsed`（**161**，seat 1 → p0，未取消）与持有该窗口的
   `NullifyWindow` 机器；而 panic 的牌是 **93**（另一张，p0 手牌 `[78,152,123,144]` 中无 93，
   93 在牌库）→ 说明**另一条路径**（无懈窗口内的效果恢复/后续用牌）提交了一张**已失效的牌 id**；
3. 因此：`commit_card_used` 的 else 分支是**硬失败点**，而真正的失效引用产生于
   **效果机器跨窗口持有的牌 id**（与 §9 的结论一致）。

**修复选项（登记，需在 host/kernel 拆分落定后实施）**：

- **A（治本）**：定位提交 93 的路径，改为**提交前重查**（手牌/桌面/弃牌堆）或在牌离区时同步更新
  持有者（效果机器/窗口包装）——需要更细的机器自省；
- **B（安全网）**：`commit_card_used` 的 else 分支由 panic 改为**记账式降级**（不移动牌、记一次
  异常计数，与 `sim_faults`/`probe_faults` 同风格），使**真实提交**也不会因该类失效引用崩溃；
  语义上仍是缺陷（少一次牌的区转移），故必须同时保留计数与告警；
- 两个选项都触及**未跟踪的 host/kernel 拆分文件**（`content/host/`、`kernel/events.rs` 均为 `??`）
  → 按「只提交自己的文件」登记，待拆分落定后实施。

**复现用例状态**：`nc12_dangling_reference_repro`（seed 44170）当前**失败**（断言 faults==0），
这是**预期的**——它是该缺陷的守护用例；修复后应转为通过。已在进度主稿登记为未验收项。

## 11 诊断能力补齐：**内存式故障上下文日志**（合规，测试可读）（2026-10-19 同轮）

维护门禁禁止生产源码打印（`println!`/`eprintln!`/`env::var`），但**内存记录**是允许的 →
新增两处环形缓冲（各保留最近 32 条，**不打印**）：

| 位置 | API | 记录内容 |
|---|---|---|
| `crates/engine-native/src/ai.rs` | `note_sim_fault`（内部）/ `sim_fault_log()` / `clear_sim_fault_log()` | 模拟故障时的 `seat/rev/phase/action/table/各座位手牌/事件栈(frames)/挂起机器(parks)` |
| `crates/engine-native/src/engine/legal_actions.rs` | `note_probe_fault_context`（内部）/ `probe_fault_log()` / `clear_probe_fault_log()` | 候选探测故障时的 `seat/action/rev/phase/table` |

复现用例（`nc12_dangling_reference_repro`，seed 44170）已接入：命中故障时**由测试打印**上下文
（测试允许输出）：

```
[sim-fault 0] seat=1 rev=10 phase=Play action=Answer { choice_id: 2, selection: [Pass] }
  table=[161] hands=[p0:hp=3/4 hand=[78,152,123,144] | p1:[23,15,72,10] | p2 | p3]
  frames=[23/CardUsed/CardUsed(CardUseData { card: 161, player: 1, targets: [0], … })]
  parks=[NullifyWindow { card: 161, source: 1, queue: [1,2,3,4→0], window_open: true,
           effect: Some(Host { def_key: "swd_zhongxinghujia", behavior: 228, owner: 1,
                               target: Some(0), card: Some(161) }) }]
```

**本轮结论（不夸大）**：上下文已能固定「故障时的动作/栈/挂起机器」，但**仍未直接暴露发出失效
用牌的路径**（panic 发生在被捕获的提交内部，状态为中途态）→ 根因修复仍待做；本能力为后续
修复与回归提供**可复现的现场**（且合规、无输出污染）。

### 仍未完成（更新）

- [ ] panic 根因：定位「提交失效用牌」的具体机器（下一步：在被捕获提交**之前**记录状态，
      或对 `commit_card_used` 的 else 分支加「记账式降级」安全网——后者触及未跟踪的
      `kernel/events.rs`，待拆分落定）
- [ ] 修复后：`sim_faults`/`probe_faults` 与两个日志缓冲应保持空
- [ ] 长程同时开 `alloc=1`；RSS/句柄/线程数；温控；产品整机 30 分钟

## 12 **根因定位完成**：`commit_equip` ↔ 同帧「先装后弃」竞态（2026-10-19 同轮）

用「提交前/提交后上下文 + 完整回溯」把根因钉到**具体函数**：

```
machine::Ctx::take_hand_card
kernel::equipment::commit_equip        ← 发出取牌提交的路径
kernel::commit_frame
resolution::run
commands::finish_segment
commands::submit
ai::plan_action（模拟提交）
```

**因果链（已闭合）**：`swd_zhongxinghujia`（众星护甲，`step/swd_card_effects.rs`）的效果在**同一步**内：

1. 从牌堆+弃堆**发现**一张防具实体 `x`（本例 `x = 93`）→ `hand.push(x)` → **emit `Equipped{player: t, card: x}`**
   （事件是**排队帧**，稍后才提交）；
2. 紧接着**随机弃置目标一张手牌**（`hand[k]`）→ 若 `k` 恰好选中**刚放入的 `x`**，则该牌被移出手牌；
3. 随后 `Equipped` 帧提交 → `commit_equip` → `take_hand_card(t, 93)` → **牌已不在手牌** → panic。

**为何此前难定位**：panic 发生在**被捕获的模拟提交内部**、且提交用的是 `work` 克隆（panic 时未提交）
→ 事后状态里看不到「93 曾被放入手牌」这一步（PRE/POST 上下文完全相同）。

**修法（明确、待拆分落定后实施）**：二选一——
- **A**：把「随机弃置」移到**发现/装配之前**（上游顺序为 `chooseToDiscard` → 装配？需按上游 `haidao` 同族核对）；
- **B**：随机弃置的候选**排除**刚放入的 `x`（或改为「从**原有**手牌中随机弃一张」）。
两处都在 `content/host/step/swd_card_effects.rs`（**未跟踪**，属他人 host 拆分）→ 按「只提交自己的文件」
登记，待拆分落定后修复；修复后 `nc12_dangling_reference_repro`（seed 44170）应转为通过、
长程 `sim_faults`/`probe_faults` 应归零。
