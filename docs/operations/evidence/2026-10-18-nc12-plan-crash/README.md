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
