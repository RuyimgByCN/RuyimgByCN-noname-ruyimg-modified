# NC12 效能基线**实测**（2026-10-19）

- 工作包：**NC12 效能**（八项验收第 8 项）；规则版本 v322（工作树）/ 存档格式 v105
- 工具：`crates/engine-native/examples/nc12_bench.rs`（**既有**，只测量不改行为）
- 原始输出：**[raw.txt](raw.txt)**（54 行 `RESULT` 制表符记录）

## 0 口径（**必须先读**）

| 项 | 值 |
|---|---|
| 设备 | **Apple M3 Max** / macOS 27.0.1 |
| 工具链 | **rustc 1.95.0**，`--release`（optimized） |
| 负载 | `tier=none`（基础目录）/ `tier=swd`（单扩展）/ `tier=huanle` / `tier=combo`（3 包） |
| 说明 | 这是**同设备同工具链的基线**，**不是**优化前后对比（无旧实现对照）⇒ **不能据此宣称加速比例** |

## 1 建局与装载（`probe`）

| 档位 | 卡数 | 技能数 | `build_ns` p50 | `load_ns` | 存档字节 |
|---|---:|---:|---:|---:|---:|
| `none` | 124 | 215 | **9.7 µs** | 10 µs | 1888 |
| `swd` | 214 | 429 | **78.7 µs** | **51.3 ms**（import 49.6 ms） | 2504 |
| `huanle` | 131 | 220 | 18.8 µs | 40.3 ms | 1916 |
| `combo` | 261 | 452 | **87.9 µs** | 40.3 ms | 2724 |

⇒ 建局本身**微秒级**；**装载（import）是毫秒级**（41 个包目录扫描 + 导入）——这是**扩展装载的主要成本**。

## 2 存读档（`archive tier=swd`，20 样本）

| 指标 | p50 | p95 | max |
|---|---:|---:|---:|
| `save_ns` | **417 ns** | 1.7 µs | 42.7 µs |
| `encode_archive_ns` | **3.2 µs** | 9.7 µs | 172 µs |
| `decode_archive_ns` | **4.75 µs** | 6.6 µs | 352 µs |
| `restore_ns` | **62.7 µs** | 89.3 µs | 133 µs |

⇒ 快照极轻（**417 ns**），**恢复 62.7 µs**（含解码 + 重建）；max 值偏高说明有**偶发长尾**（首次分配/冷缓存）。

## 3 状态复制与规划（`copy tier=swd`）

| 指标 | p50 | p95 |
|---|---:|---:|
| `engine_clone_ns` | **416 ns** | 1.5 µs |
| `plan_action_ns` | **17.3 µs** | 23.9 µs |

## 4 **整局驱动**（`drive tier=swd`，**AI 打完整局**）

```
steps=268 accepted=223 pending=45 rejected=0 finished=true stalled=false
wall_ns=14217417 (14.2 ms)  events=739  nodes_total=539
plan_ns p50=27.75 µs  submit_ns p50=11.96 µs  cmd_ns p50=39.5 µs
```

| 指标 | 值 |
|---|---|
| **整局（268 步）墙钟** | **14.2 ms**（≈ **18.9k 步/秒**）|
| 拒绝命令 | **0** |
| 事件数 | 739 |
| `cmd_ns` p50 | **39.5 µs** |

⇒ **单机 release 下，4 人 + swd 扩展的完整 AI 对局在 ~14 ms 内收敛**，
`rejected=0`、`stalled=false`、`panics=0` ⇒ **性能与稳定性同时可用**。

## 5 与「八项验收」的关系（**如实**）

| 判据 | 状态 |
|---|---|
| 同设备同工具链 | ✅ 本文件 |
| **同负载前后对比**（优化收益） | ❌ **无**——没有旧实现基线，**不能宣称加速** |
| **产品级**（含 server/网络/浏览器） | ❌ **未测**（本项只测引擎 + 装载器） |
| 逐来源性能签署 | ⚠ 仅 **swd（U05）** 有本次数据；其余来源需按同口径各跑一次 |

⇒ 第 8 项由「**0 证据**」推进为「**引擎级基线已测，产品级与逐来源仍待办**」。

## 6 复现命令

```bash
cd noname-next
cargo run --offline --release -p engine-native --example nc12_bench -- probe
cargo run --offline --release -p engine-native --example nc12_bench -- archive tier=swd seats=4 samples=20
cargo run --offline --release -p engine-native --example nc12_bench -- copy tier=swd budget=64 steps=60 samples=20
cargo run --offline --release -p engine-native --example nc12_bench -- drive tier=swd seats=4 budget=64 lookahead=1 variant=0 seed=42007
```
