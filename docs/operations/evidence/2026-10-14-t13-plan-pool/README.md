# T13 AI 工作池 + 运行中取消跨线程（2026-10-14）

## 任务来源

[进度主稿 T13](../../../architecture/native-progress.md)（N10/N12）AI 项「工作池、运行中取消」——原生线 `BoundedPlanner` 既有单座规划/预算/时限/cancel 旗标，缺并行工作池与取消的跨线程回归。

## 交付物（crates/engine-native/src/ai.rs）

- **`PlanPool`**：固定 worker 线程分桶（round-robin）消费座位队列，并行多座规划；各线程独立克隆引擎（规划隔离已克隆化）；结果按输入座位序拼接（round-robin 逆操作）。规划为确定性纯函数（无共享 RNG/可变状态）——**并行结果与逐座顺序规划逐一相等**（回归固化）。
- **`SkillMachine: Send + Sync`**（catalog.rs 超特）：机器实现体均为普通结构体（编译期保证），使 `Engine: Sync`——工作线程可共享引擎引用；这也为 game-server 多房间并行规划解锁能力面。
- 运行中取消：cancel 旗标（`Arc<AtomicBool>`）在候选边界检查——既有机制，本轮补**跨线程回归**（预挂旗→规划线程首边界即返回 `cancelled` → `apply_plan` 拒 `Cancelled`）。

## 实测记录

| 项 | 结果 |
|---|---|
| `t13_plan_pool_parallel_matches_sequential` | 4 座并行（workers=4）与逐座顺序规划：座位/修订/节点数/预算面/动作逐一相等 |
| `t13_plan_pool_more_seats_than_workers_keeps_order` | workers=2、5 座位分桶——输入序返回不串位 |
| `t13_plan_cancel_mid_run_cross_thread` | 预挂取消→规划线程返回 cancelled → apply_plan 拒 Cancelled |
| 定向 | ai_planner **12/12**（9 既有+3 新）——[日志](ai-planner-targeted.log) |
| 全量 | engine-native **673/0/0**（670→673）——[日志](engine-native-tests.log) |
| Gate 1b | 独立 target 目录 `--all-targets` **exit 0**——[日志](gate1b-all-targets.log) |

## 坑位实录

- `std::thread::spawn` 需 `'static`——工作线程持引擎**克隆**（plan_action 克隆隔离语义下等价），不持借用。
- `Engine` 原非 Sync（`Resolution.parks` 的 `Box<dyn SkillMachine>`）——根因是 trait 缺 Sync 超特而非实现体，补超特即解（无 unsafe）。

## 登记边界（T13 余项）

- 合法座位视图：部分座位视图用例已过（10-07/08）；余「全部模式/扩展的信息隔离」随模式/扩展批。
- 策略质量（候选评估深度/启发式改进）与产品面（客户端 UI、帧率延迟实测）另批。
