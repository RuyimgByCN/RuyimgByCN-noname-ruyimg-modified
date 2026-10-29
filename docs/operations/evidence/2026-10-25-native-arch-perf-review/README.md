# 原生内核性能结构复核：触发规划、AI 节点与状态哈希（2026-10-25）

- 日期：2026-10-25 23:30–23:38 +08:00 采集（哈希微基准于 2026-10-26 00:15 复跑落盘）
- 工作包：NC12 效能验收的结构复核切片，涉及 NC06/NC07/NC10；**只测量，不改产品源码**
- 源码：`HEAD = 7e28a93799a9c01794085e911a72a4faa5673883` 加未提交工作树（他人改动约 315 项）；规则 **v322**、存档 **v105**
- 测量程序：独立 crate（本目录 [`archbench/`](archbench/)），以 path 依赖链接 `engine-native`、`content-compiler`、`content-schema`、`rule-vm`、`rule-api`，不进入工作区、不改仓库源码
- 后续任务登记：[剩余台帐 PA01–PA12](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26)

**一句话结论**：内核每命令仍是微秒级，但有三处结构会随规模劣化，均已实测复现：
1. 触发规划成本随**目录技能总数**增长（与本局是否持有无关）；
2. AI 每个规划节点的成本随**对局日志长度**线性增长；
3. 每条被接受命令都做一次全量状态哈希，约占权威提交 p50 的六成。

> 本页数字只描述本次快照与负载；不同设备/负载不得相除算收益。同机有其他 agent 并发（load average 8.7–21），p95/max 受污染，p50 相对稳健。构建为 thin LTO + 16 codegen units，**不是**工作区 release 配置（fat LTO + 1 cu）；同一二进制内的相对比较有效，绝对值不能与 NC12 基线直接对比。

## 1 设备与工具链

| 项 | 值 |
|---|---|
| 机型 | Mac15,10（Apple M3 Max），14 核，36 GiB |
| 系统 | macOS 27.0.1（26A434） |
| 工具链 | rustc 1.95.0（59807616e 2026-04-14） |
| 构建 | `CARGO_TARGET_DIR=<独立目录> cargo build --offline --release`；`opt-level=3`、`lto="thin"`、`codegen-units=16` |

## 2 方法

源码见 [`archbench/src/main.rs`](archbench/src/main.rs)（[`Cargo.toml`](archbench/Cargo.toml) 中的 path 依赖是本机绝对路径，复现时按仓库位置调整）。计数分配器统计每段分配次数/字节。

- **scale**：向目录追加 N 个**未被任何玩家持有**的 VM 触发技能（同一字节码，触发事件在 PhaseBegin/PhaseEnd/CardUsed/CardsMoved/CardsDrawn/Damage/DamageBefore/CardsDiscarded/CardsLost/CardResponded 十类间轮换），经 `ExtensionPack.vm_skills` + `Engine::new_with_extensions` 装载。`GRANT=1` 时为每座位 `debug_grant_general` 一名标准武将（座位 p 取 `STANDARD_GENERALS[(p*5+1) % len]`）。原型模式，种子 42007–42009，每局 ≤800 步，`BoundedPlanner::new(32)` 驱动；`submit` = `apply_plan`（含 resume），`plan` = `plan_action`。
  - **有效性校验**：各 N 档步数与事件数完全相同（4 座 818/2115，8 座 1467/4355），即未持有技能不改变对局轨迹，差异只来自成本。
- **growth**：标准目录 `Engine::new`，`BoundedPlanner::new(64)`，3 局（≤2000 步/局）。每步记录：`Engine::clone()`（5 次平均）、`state_hash()`（5 次平均）、`plan_action` 与提交的耗时和分配；按规划前日志长度分四档取 p50。
- **sizes**：关键类型 `size_of`。
- **哈希微基准**（[`hash-microbench/`](hash-microbench/)）：blake3 1.8.5 的 `pure`、默认、`neon` 三种特性；输入近似 `canonical_hash` 形态（900 次 1/4/8 字节小块写入，共 3780 字节）对比「先拼缓冲再一次哈希」，各 20000 次。

## 3 结果

原始输出：[results.txt](results.txt)、[hash-microbench/results.txt](hash-microbench/results.txt)。单位 ns，除注明外为 p50。

### 表 A 目录规模（未持有技能）对每命令耗时的影响（`GRANT=1`）

| 座位 / 技能实例 | 目录技能数 | 提交 p50 | 提交 p95 | 提交合计 ms | 规划 p50 | 规划 p95 | 规划合计 ms |
|---|---:|---:|---:|---:|---:|---:|---:|
| 4 座 / 5 | 215 | 8,750 | 16,250 | 7.30 | 25,709 | 85,000 | 29.34 |
| 4 座 / 5 | 715 | 10,542 | 24,875 | 9.16 | 28,041 | 95,042 | 32.17 |
| 4 座 / 5 | 2,215 | 15,417 | 64,458 | 16.71 | 34,875 | 170,583 | 46.55 |
| 4 座 / 5 | 6,215 | 29,750 | 186,500 | 37.04 | 53,834 | 417,417 | 86.65 |
| 8 座 / 11 | 215 | 13,333 | 24,416 | 19.36 | 49,625 | 210,041 | 112.84 |
| 8 座 / 11 | 715 | 16,125 | 34,041 | 24.33 | 53,417 | 249,958 | 128.07 |
| 8 座 / 11 | 2,215 | 23,959 | 78,292 | 42.72 | 61,916 | 456,167 | 189.69 |
| 8 座 / 11 | 6,215 | 44,417 | 214,500 | 95.56 | 87,625 | 1,149,291 | 419.21 |

根因：[`resolution.rs:295`](../../../../crates/engine-native-kernel/src/resolution.rs) `plan_triggers` 对 `catalog.candidates(kind)` 的每个定义都完整遍历 `core.skills`（`range(..)`），复杂度为「目录候选数 × 本局技能实例数」。真实对局的技能实例更多（武将 2–3 技能 + 装备 + 临时技能），且真实目录的触发事件分布比本测的十类轮换更集中，本表应视为下限。旧线已有增量维护的 [`ActiveTriggerIndex`](../../../../crates/engine-core/src/trigger/index.rs)。

### 表 B 对局推进（日志变长）时的成本（标准目录）

| 座位 | 日志长度档 | 步数 | `Engine::clone` | `state_hash` | 规划 p50 | 每节点 p50 | 规划分配次数 | 规划分配 KB | 提交 p50 | 提交分配次数 |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 4 | 0–189 | 223 | 1,683 | 5,491 | 16,792 | 14,597 | 168 | 41 | 8,417 | 24 |
| 4 | 189–379 | 220 | 3,833 | 5,550 | 22,250 | 19,000 | 307 | 77 | 8,459 | 22 |
| 4 | 379–568 | 205 | 5,825 | 5,575 | 25,458 | 24,042 | 440 | 113 | 8,500 | 23 |
| 4 | 568–758 | 74 | 8,450 | 5,775 | 33,417 | 31,125 | 581 | 150 | 8,875 | 23 |
| 8 | 0–383 | 458 | 2,950 | 8,600 | 24,916 | 18,807 | 309 | 81 | 11,958 | 32 |
| 8 | 383–766 | 451 | 7,433 | 8,875 | 35,541 | 30,083 | 572 | 152 | 12,375 | 33 |
| 8 | 766–1149 | 447 | 11,966 | 9,058 | 42,958 | 38,458 | 834 | 227 | 12,583 | 29 |
| 8 | 1149–1532 | 362 | 15,783 | 9,050 | 49,750 | 46,875 | 1,068 | 291 | 12,542 | 26 |

合计：4 座 722 步，每步（规划+提交）平均分配 539 次；8 座 1718 步，平均 1136 次。

根因：
- `Engine` 派生 `Clone`（[`engine.rs:147`](../../../../crates/engine-native/src/engine.rs)），其中 `log: Vec<LogEvent>` 整局只增不减；`plan_action` 先克隆一次 Engine，再为**每个候选节点**克隆一次（[`ai.rs:523`、`ai.rs:566`](../../../../crates/engine-native/src/ai.rs)）。
- 模拟节点走正式 `submit`：再做一次 `state.clone()`、一次 `save()` 检查点（[`commands.rs:125–129`](../../../../crates/engine-native/src/engine/commands.rs)）与一次全量 `canonical_hash`（同文件第 248 行）。
- 表中 `nodes_p50 = 1`（多数步只有一个候选），候选多或开两层前瞻时上述成本按节点数放大。
- 另：`plan_action` 每节点无条件格式化 `pre_context` 诊断串；在静止点栈为空时实测仅约 83–91 ns（`fmt_p50` 列，未列入上表），**不是**当前热点。

### 表 C 状态哈希

- `state_hash()`（即 `canonical_hash`）：4 座 5.5–5.8 µs、8 座 8.6–9.1 µs，约为同档提交 p50（8.4–12.6 µs）的 60%–70%。实现为逐字段约 800 次 `Hasher::update` 小块写入（[`engine/hash.rs:68`](../../../../crates/engine-native/src/engine/hash.rs)）。
- 微基准（3780 字节、900 次小块写入）：`pure` 11,096 ns / 缓冲后一次 6,377 ns；默认 11,047 / 7,714；`neon` 10,953 / 6,739。**特性开关在 aarch64 上无差别**；小块写入比缓冲后一次哈希慢约 1.4–1.7 倍。

### 表 D 类型大小（`size_of`，字节）

`GameStateCore` 776、`PlayerState` 512、`TurnState` 248、`GameState` 1048、`Engine` 2184、`rule_api::EffectRequest` 96、`EventData` 56、`LogEvent` 48。`TurnState` 61 个字段、`PlayerState` 105 个字段，其中大部分是具体技能专用状态（静态核对 [`state.rs`](../../../../crates/engine-native-kernel/src/state.rs)）；每局无论是否使用这些内容，都在复制、哈希、归档中付费（NC06 已列入）。

## 4 未覆盖（不得外推）

1. 工作区正式 release 配置（fat LTO / 1 cu）、PGO；
2. 身份局/国战/Boss 等真实选将模式与 37 来源真实目录分布（本测为原型模式 + 标准武将授予 + 合成未持有技能）；
3. 两层前瞻、宽候选集时的 AI 节点成本；
4. 真实浏览器 / WASM 档位（表现层见 [2026-10-26 帧间隔与输入延迟](../2026-10-26-web-frame-latency/README.md)）；
5. 优化前后对照：本页即「前」数据，任何改动须用同一程序同机复测。

## 5 复现

```bash
# 1) 把 archbench/ 复制到任意临时目录，按本机路径修改 Cargo.toml 中的 path 依赖
CARGO_TARGET_DIR=$PWD/target cargo build --offline --release
GRANT=1 ./target/release/archbench scale 4
GRANT=1 ./target/release/archbench scale 8
./target/release/archbench growth 4 64 3
./target/release/archbench growth 8 64 3
./target/release/archbench sizes
# 2) 哈希微基准：hash-microbench/main.rs 依赖 blake3 = "=1.8.5"，分别以 features = ["pure"] / 默认 / ["neon"] 构建运行
```
