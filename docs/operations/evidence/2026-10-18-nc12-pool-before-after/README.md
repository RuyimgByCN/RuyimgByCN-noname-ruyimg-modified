# NC12 前后对比：长驻有界池 vs scoped 线程（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC12**（[方案 §NC12](../../../architecture/native-convergence-plan.md#nc12)）+ **NC10** 工作池评估
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**同机同负载前后实测：长驻池在本实现下比 scoped 慢 1.4–1.8×** → **默认路径回退为 scoped**，池保留为可选组件（**不预设收益**，以测量为准）

## 1 口径

- 工具：`crates/engine-native/examples/nc12_pool_bench.rs`（新增）——同一进程内**交替采样**
  两条路径（避免顺序偏差），预热后各 30 样本，报告 p50/p95/max/mean；
- 路径：`PlanPool::plan_seats_scoped`（逐调用 `std::thread::scope`）vs
  `PlanPool::plan_seats_pooled`（`crate::ai_pool::PlannerPool` 长驻有界池）；
- 设备/负载：同机同工具链（release）；采样时 `load average 18.80 / 17.26 / 17.11`（本机其他
  任务并行）→ **只有 p50 可作比较量**，绝对值受负载影响；
- 命令：`./target/release/examples/nc12_pool_bench tier=<none|swd|huanle> seats=4 iters=30 budget=32`。

## 2 结果（p50，纳秒；完整日志见 [pool-before-after.log](pool-before-after.log)）

| tier | scoped（原路径） | pooled（长驻池） | 比值 |
|---|---:|---:|---:|
| none | 52,500 | 87,458 | **1.67×** |
| swd | 48,833 | 79,000 | **1.62×** |
| huanle | 39,000 | 71,625 | **1.84×** |

线程创建：scoped = **每调用 4 个线程**；pooled = 池内线程**每调用重建**（本实现每次
`plan_seats_pooled` 建池/销毁），另加**每座位一份 `Engine` 克隆**与通道/装箱开销。

## 3 结论与处置（不预设收益）

1. **未取得收益**：本实现下池路径慢 1.4–1.8×，根因是**每调用建池**（与 scoped 同为每次创建
   线程）＋**每座位引擎克隆**（`'static` 任务要求）＋通道/装箱开销；
2. **默认路径回退为 scoped**（`PlanPool::plan_seats` → `plan_seats_scoped`），池保留为
   **可选组件** `plan_seats_pooled` / `crate::ai_pool::PlannerPool`，并在文档注释中写明
   实测结论与不启用原因；
3. **待办（后续若要做池）**：① 池跨调用复用（宿主自持）；② 消除每座位克隆（`Arc` 共享状态
   或工作线程持有引擎）；③ 复测后再决定是否切换默认路径。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 前后对比（3 tier × 30 样本） | 见 §2（池更慢） | [pool-before-after.log](pool-before-after.log) |
| 池组件回归 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | **1152 passed / 0 failed / 2 ignored** | [engine-native-tests.log](engine-native-tests.log) |
| Gate 1b（`-D warnings`，lib+tests+examples） | exit 0 | [gate1b.log](gate1b.log) |
| 行数门禁 | 8259 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |
| 采样时负载 | load average 18.80 / 17.26 / 17.11 | [load.txt](load.txt) |

## 5 未完成 / 待办

- [ ] 池跨调用复用 + 消除克隆后再测（本机同负载），据结果决定是否切换默认路径
- [ ] 其余 NC12 项：浏览器/WASM 档位、移动/桌面平台、动画帧与输入响应、联机 RTT、
      ≥30 分钟持续负载、冷启动「创建到可操作」、其余包逐包性能、旧线对照
