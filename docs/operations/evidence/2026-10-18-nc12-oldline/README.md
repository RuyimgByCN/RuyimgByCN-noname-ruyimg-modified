# NC12 旧线（`engine-core`）对照基准（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC12**（[方案 §NC12](../../../architecture/native-convergence-plan.md#nc12)）「旧线对照」
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**旧线四项指标已测得**（建局 153 µs / 提交 15 µs / 快照 15 µs / 恢复 15.6 µs，p50）；
  与新线**并列记录**，跨线比值附语义口径说明

## 1 方法

新增 `crates/engine-core/examples/nc12_oldline_bench.rs`（旧线自身基准，30 次建局 + 200 次提交）：

```bash
cargo run --offline --release -p engine-core --example nc12_oldline_bench -- 30 200
```

## 2 实测（[oldline.log](oldline.log)；采样负载 [load.txt](load.txt)：14.25 / 15.94 / 17.95）

| 指标 | p50 | p95 | max |
|---|---:|---:|---:|
| `build`（`Engine::new` + `StartGame` 4 座） | **152,875 ns ≈ 153 µs** | 225,167 | 3,987,917 |
| `submit`（`EndPhase`） | **15,083 ns ≈ 15.1 µs** | 16,000 | 883,541 |
| `snapshot` | **15,042 ns ≈ 15.0 µs** | 16,333 | 22,416 |
| `restore` | **15,625 ns ≈ 15.6 µs** | 18,541 | 20,500 |

## 3 与新线并列（**不直接等同**）

新线（`engine-native`）NC12 基线（[基线证据](../2026-10-18-nc12-baseline/README.md)）同机 p50：

| 操作 | 旧线 `engine-core` | 新线 `engine-native` | 口径说明 |
|---|---:|---:|---|
| 建局（4 座身份局） | 153 µs | **4.75 µs** | 同名同义（`Engine::new` + StartGame），但**规则实现不同**（新线为重构后的原生内核）→ 比值为**实测**，语义差异需一并阅读 |
| 提交推进阶段 | 15.1 µs | **8.13 µs** | 同上（`EndPhase`） |
| 快照/归档 | `snapshot` 15.0 µs | `save()` **292 ns** | **口径不同**：旧线为结构化快照，新线为归档字节 → **不可比** |
| 恢复 | `restore` 15.6 µs | swd 恢复（基线中 30/30 曾被拒，已修复） | **口径不同**（结构化 vs 归档）→ **不可比** |

**口径纪律（方案 §NC12）**：旧线与新线**分开统计**；上表只把**同名同义**的两项并列
（并注明规则实现差异），**快照/恢复因口径不同不计算比值**；所有数字均为本机同负载下的 p50，
绝对值受并发负载影响。

## 4 未完成 / 待办

- [ ] 旧线 **AI（`engine-ai`）** 与 **WASM（`engine-wasm`）** 的同口径基准（当前只测 `engine-core`）；
- [ ] 旧线**长程**（多局、内存/漂移）与新线同口径对照；
- [ ] 跨线**规则等价**的对照（不是性能）：见 NC08 的旧线↔原生线开局差异判定；
- [ ] 旧线基准纳入常规回归（当前为一次性测量）。

## 5 追加：旧线 **AI（`engine-ai`）** 基准（2026-10-19 同轮）

新增 `crates/engine-ai/examples/nc12_oldline_ai_bench.rs`（信息集 / 合法动作 / 三种搜索，20 次、
节点预算 32、深度 3）：

| 指标 | p50 | p95 | max |
|---|---:|---:|---:|
| `information_set` | **18,833 ns ≈ 18.8 µs** | 19,916 | 33,583 |
| `legal_actions` | **44,834 ns ≈ 44.8 µs** | 50,167 | 748,459 |
| `heuristic`（32 节点） | **103,042 ns ≈ 103 µs** | 106,625 | 106,834 |
| `beam`（32 节点） | **2,683,959 ns ≈ 2.68 ms** | 2,878,583 | 4,210,041 |
| `mcts`（32 节点） | **10,644,708 ns ≈ 10.6 ms** | 11,345,166 | 13,200,291 |

日志：[oldline-ai.log](oldline-ai.log)。

**与新线并列（**不可直接比较**，方案 §NC12 纪律）**：新线 `nc12_bench ai` 的 `plan_action`
p50 为 18,875 ns（none 档）/ 23,459 ns（swd 档）——但新线是**有界启发式规划器**
（节点预算 + 时限 + 候选枚举），旧线是 **Heuristic/Beam/MCTS 三种独立搜索器**；
**算法不同 → 不计算跨线收益**，仅并列记录各自绝对耗时。

## 6 未完成（更新）

- [ ] 旧线 **WASM（`engine-wasm`）** 同口径基准（`o4_bundle` 示例存在，但需先确认其口径）；
- [ ] 旧线**长程**（多局、内存/漂移）与新线同口径对照；
- [ ] 跨线**规则等价**对照（属 NC08）；
- [ ] 旧线基准（core/AI/WASM）纳入常规回归。

## 7 追加：旧线 **WASM（`engine-wasm`）** 档位基准（2026-10-19 同轮）

新增 `crates/engine-wasm/scripts/nc12-old-wasm-bench.mjs`（Node 驱动**既有产物**
`crates/engine-wasm/pkg-node/`，sha256 见 [old-wasm-sha256.txt](old-wasm-sha256.txt)）：

| 指标 | p50 | p95 | max |
|---|---:|---:|---:|
| `build_and_start`（`WasmEngine` + 内嵌标准 + `startGame(4)`） | **51,494,917 ns ≈ 51.5 ms** | 80.5 ms | 122.9 ms |
| `end_phase` | **896,416 ns ≈ 896 µs** | 2.82 ms | 11.0 ms |
| `observer_json` | **29,500 ns ≈ 29.5 µs** | 37.6 µs | 166.5 µs |
| `state_hash_hex` | **17,208 ns ≈ 17.2 µs** | 19.5 µs | 59.0 µs |
| `auto_step` | **149,750 ns ≈ 150 µs** | 1.37 ms | 3.61 ms |

日志：[old-wasm.log](old-wasm.log)。

**与新线 WASM 档并列（同机、同 Node；**不可直接比较**）**：新线 wasm 档（
[证据](../2026-10-18-nc12-wasm-tier/README.md)）`build_standard` p50 为 **16,333 ns ≈ 16.3 µs**、
`legal_actions` 43.3 µs、`save_hex` 119.8 µs、`view_json` 23.9 µs。两者**规则实现、序列化面与
内容装载方式不同**（旧线为 `loadEmbeddedStandard` + 结构化 JSON 命令面；新线为窄壳 + 归档字节）
→ **只并列、不计算收益**（方案 §NC12 纪律）。

## 8 未完成（更新）

- [ ] 旧线**长程**（多局、内存/漂移）与新线同口径对照；
- [ ] 跨线**规则等价**对照（属 NC08）；
- [ ] 旧线基准（core / AI / WASM）纳入常规回归；
- [ ] 旧线 WASM 产物重建后的复测（当前为既有产物，sha256 已记录）。
