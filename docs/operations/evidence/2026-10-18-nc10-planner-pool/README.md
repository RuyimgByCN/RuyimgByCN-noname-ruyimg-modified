# NC10 长驻有界规划工作池（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC10**（[方案 §NC10](../../../architecture/native-convergence-plan.md#nc10)）
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**长驻有界池已实现 + 4 项回归通过 + Gate 1b 通过**

## 1 动机（源码依据）

`crates/engine-native/src/ai.rs` 的 `ParallelPlanner::plan_all` 用 `std::thread::scope`
**每次调用创建线程并同步等待**——方案 §NC10 要求「将逐调用创建线程并同步等待的原型收敛为
产品宿主中的**长驻有界工作队列**，限制并发、内存和排队，支持取消、降级与过期结果拒绝」。

## 2 实现（`crates/engine-native/src/ai_pool.rs`，std-only 无新依赖）

| 要求 | 实现 |
|---|---|
| 长驻 | `PlannerPool::new(workers, queue_cap)` 一次创建 `workers` 个线程，复用至 `shutdown`/`Drop` |
| 有界并发 | 线程数上限 `workers`；任务通道容量 `queue_cap` |
| 有界排队 | 提交用 `try_send`：队列满**立即** `Err(PoolFull)`，调用方降级 |
| 降级 | `plan_all` 对未提交/未回收的座位在**本线程串行补齐**（`degraded=true`），**不丢结果、不阻塞** |
| 取消 | 沿用 `BoundedPlanner.cancel`（节点边界检查）；池不引入额外阻塞 |
| 过期拒绝 | 产物 `Plan.revision` 携带规划时修订号，提交侧按修订号拒绝（既有机制，池不改语义） |

**明确代价（登记）**：任务需 `'static` → 每个座位任务持一份 `Engine` 克隆
（NC12 基线：`clone()` 1.3–1.5 ms / swd 档）。相对「共享 `&Engine` + scoped 线程」换取
长驻与安全（无 unsafe）；待办：改为 `Arc` 共享状态或让工作线程持有引擎。

## 3 回归（新增 `crates/engine-native/tests/nc10_planner_pool.rs`，4 项）

| 验收项 | 判据 | 结果 |
|---|---|---|
| 长驻复用 + 产物合法 | 同一池连续 3 轮规划 4 座位，队列充足不降级，产物动作合法（或该座位无合法动作） | 通过 |
| 队列满降级 | 容量 1 时第二次 `try_submit` → **`Err(PoolFull)`**；`plan_all` 仍返回全部座位结果（降级不丢结果） | 通过 |
| 取消 | `cancel` 置位 → 产物 `cancelled=true` | 通过 |
| 过期拒绝 | 旧修订号产物用新修订号提交 → `Rejected(StaleRevision)` | 通过 |

## 4 验证结果

```
cargo test --offline -p engine-native --test nc10_planner_pool
cargo test --offline -p engine-native --no-fail-fast
CARGO_TARGET_DIR=<tmp> RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --lib --tests
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| 池定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | **1152 passed / 0 failed / 2 ignored** | [engine-native-tests.log](engine-native-tests.log) |
| Gate 1b（`-D warnings`，lib+tests） | **exit 0**（顺带清理了本会话新增测试中的未使用导入/变量） | [gate1b-lib-tests.log](gate1b-lib-tests.log) |
| 行数门禁 | 8257 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 5 未完成 / 待办（NC10）

- [ ] **宿主接线**：`ParallelPlanner` 仍走 scoped 线程；产品宿主（game-server/lobby）应改用
      本池并复用同一池实例（当前池为独立组件 + 测试面）
- [ ] 消除每任务引擎克隆（`Arc` 共享或工作线程持有引擎）
- [ ] 池的**压力/尾延迟**测量（NC12 口径：同设备同负载，前后对比）
- [ ] 非静止态信息边界窗口（机器自省保引用重分配或关闭模拟评分）
- [ ] 主动技/多选/多目标候选
