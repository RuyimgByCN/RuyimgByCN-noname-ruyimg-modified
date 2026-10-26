# PA02 AI 模拟与权威会话分离：模拟提交不再付权威会话成本（2026-10-26）

- 日期：2026-10-26；数据文件落盘 02:34–02:34、首次提交 02:36（bca6370e）+08:00（原记「2026-10-26 02:15–03:20 +08:00」晚于提交或与文件时间矛盾，2026-10-26 17:30 扫描按数据文件时间与提交历史更正）
- 工作包：[台帐 PA02](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26)；「前」数据与测量程序见[内核性能结构复核（2026-10-25）](../2026-10-25-native-arch-perf-review/README.md)表 B
- 源码：`HEAD = 8319a3fb` 加未提交工作树（原记 7e28a937；按提交历史更正为本批提交的父提交）；本批改动 `crates/engine-native/src/engine.rs`、`engine/commands.rs`、`engine/legal_actions.rs`、`engine/construction.rs`、`engine/snapshot.rs`、`ai.rs`。规则 **v322**、存档 **v105** 未变（新增 `Engine.simulation` 为会话字段，与 `time_slice` 同类，不入状态/哈希/快照）
- 测量程序：同[复核证据](../2026-10-25-native-arch-perf-review/archbench/) 的 archbench（path 依赖本仓库、thin LTO/16cu release、独立 CARGO_TARGET_DIR）；A/B 行为探针与归因子命令为 scratch 副本新增（见 §4）

**一句话结论**：模拟会话与权威会话分离落地——`plan_action` 的模拟克隆不再携带/复制宿主日志，模拟提交不再做段检查点、状态哈希与日志写入，`probe_action` 探测克隆同享该路径。固定种子下 4/8 座各 722/1718 步 `(seed,step,action,nodes)` 序列改动前后**逐行一致**；8 座每节点 p50 首档 17.4→6.1 µs、末档 45.2→7.8 µs，每步分配 1136→274 次。**growth 末/首比未达 ≤1.2**（8 座 1.27、4 座 1.55），但两组判别实验证明残余增长与权威日志无关、与进程堆无关，是**晚期状态固有的结算复杂度**，不属于会话耦合——已登记为 PA05 归因范畴的残余。

## 1 改动

1. **`Engine.simulation: bool`**（私有会话字段，构建/恢复恒 false）+ **`clone_for_simulation()`**（engine/commands.rs）：手工构造（不走 derive `clone()`——后者会整份复制宿主日志再被丢弃，实测为节点成本随日志增长的根因之一），`log` 从空开始、段检查点弃置，其余字段原样保留（去重高水位、时间片、配额——模拟提交去重语义与正式提交同源）。
2. **`submit`/`finish_segment`**（engine/commands.rs）：`simulation = true` 时——不存段检查点（`save()` 跳过）、日志写入进本段弃置缓冲、`Accepted` 返回全零哈希（`canonical_hash` 跳过）。结算代码与正式提交**完全共用**，仅会话副作用按标志跳过（架构提案 D2 约束）。
3. **`plan_action`**（ai.rs）：基线克隆 `self.clone()` → `self.clone_for_simulation()`；trial/t2 逐层继承模拟标志。
4. **`probe_action`**（engine/legal_actions.rs）：探测克隆 `self.clone()` → `self.clone_for_simulation()`。探测只消费接受/拒绝判定，语义不变；这同时消除了 `legal_actions`（规划回退路径，实测 74% 的 nodes>0 步会触发）每个候选「整引擎克隆（含日志）+ 权威 submit（含哈希/存档）」的成本。

单元守护：`engine::commands::pa02_simulation_tests`（模拟 Accepted 全零哈希/零日志、正式哈希非零/写日志、同一命令两边规则状态与哈希一致）。

## 2 行为不变性（完成条件①）

A/B 探针（scratch `probe` 子命令，4/8 座 × 3 局 × ≤2000 步，BoundedPlanner::new(64)，种子 42007–42009）：

| 座位 | 步数（含终局行） | `(seed,step,action,nodes)` 对比 |
|---|---:|---|
| 4 | 722 | **完全一致**（[probe-ab-4.txt](probe-ab-4.txt)） |
| 8 | 1718 | **完全一致**（[probe-ab-8.txt](probe-ab-8.txt)） |

两次独立对比均一致（初版修复后、probe_action 修复后各一次）。注：探针程序中途增加了规模字段输出，对比按四元组提取（提取脚本见本 README 源操作记录；原始文件 [probe-before-4.txt](probe-before-4.txt)、[probe-before-8.txt](probe-before-8.txt)）。

## 3 性能（完成条件③：growth；同会话前后对照）

「前」= 同机同会话 PA01-only 二进制（`archbench-pre-pa02`，即 PA02 改动前一刻的源码）；「后」= 本批全部改动（`growth-after3-*`）。每节点 p50 单位 ns（`plan_ns_per_node_p50` 列）。

| 档位 | 每节点 p50 前 → 后 | 变化 |
|---|---:|---:|
| 4 座 0–189 / 568–758 | 11,833 / 28,667 → 4,667 / 7,250 | −61% / −75% |
| 8 座 0–383 / 1149–1532 | 17,375 / 45,208 → 6,125 / 7,750 | −65% / −83% |
| 每步分配次数 4 座 / 8 座 | 539 / 1136 → 183 / 274 | −66% / −76% |
| 末/首比 4 座 / 8 座 | 2.42 / 2.60 → **1.55 / 1.27** | 未达 ≤1.2，见下 |

原始输出：[growth-pre-pa02-4.txt](growth-pre-pa02-4.txt)、[growth-pre-pa02-8.txt](growth-pre-pa02-8.txt)、[growth-after3-4.txt](growth-after3-4.txt)、[growth-after3-8.txt](growth-after3-8.txt)。（对照：复核证据 2026-10-25 表 B 为另一负载窗口的「前」，其 8 座每节点 18,807→46,875，趋势一致，不用于计算比例。）

### 残余增长的归因（未达 ≤1.2 的部分）

1. **与权威日志无关**：定态实验（scratch `planloop`，同一晚期状态原地循环 `plan_action`）中，`save()+restore()` 出同状态但日志为空的引擎，plan p50 4,708 → 4,625（−2%）；早期状态 3,333。模拟路径已无任何日志依赖。
2. **与进程堆无关**：晚期状态存档落盘后在**全新进程** restore 复测，plan p50 = 4,792，与长驻进程同值（4,792）。
3. **状态固有**：晚期状态 nodes=1、手牌更少（2 vs 4）、parks/stack/skills 为 0、区域总字节守恒，单节点结算仍比早期贵约 45%——分配次数全程平坦（60–66/步），增长弥散于结算/视图各帧（符号化采样无单一热点）。该部分属于规则结算复杂度随局面深入的增长，**不是会话耦合**，登记为 PA05（分配/热点归因）范畴的残余；本批不虚报为已达 ≤1.2。

## 4 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --no-fail-fast` | **1558 passed / 0 failed / 71 ignored**（= PA01 基线 1555 + 本批 1 单元守护；14 golden 不变），exit 0 |
| `cargo test -p engine-native --test nc10_ai_information_boundary --test nc10_ai_bounds --test nc10_ai_boundary_and_cancel --test nc10_planner_pool --test nc03_ai_consistency --test ai_planner` | **37 passed / 0 failed** |
| `cargo check -p engine-native-wasm` | exit 0（18 条警告均为 wasm 壳既有 dead code 与他人 rule-vm 在途改动，本批文件零警告） |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` / `check-source-lines.py` | OK / 通过（8609 文件 ≤2000 行） |
| Gate 1b（`-D warnings` check --all-targets，独立 target） | 同[PA01 证据](../2026-10-26-pa01-trigger-index/README.md)：被他人未提交 rule-vm 改动的既有 `unreachable_patterns` 阻塞；本批全部文件在警告位置清单中零出现 |

规则/存档/状态摘要：无变化；`STANDARD_RULESET_VERSION`、`ARCHIVE_FORMAT_VERSION` 未动，golden 未重录。

## 5 未覆盖（不得外推）

1. growth 末/首比 ≤1.2 的**残余部分**：晚期状态固有结算复杂度（§3 归因），待 PA05 分配归因后决定是否另立条目；
2. `Engine::clone`（derive，含日志）仍是房间检查点/权威路径语义，未改——`room_native` checkpoint 等照旧全量克隆；
3. `ai_pool` 长驻池路径的外层每座位克隆仍走 derive `clone()`（本批只改 `plan_action` 内部与 `probe_action`）；
4. wasm 档位实测未跑（仅编译核对；`plan_action` 的 wasm 时限语义未动）。
