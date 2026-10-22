# 测试与性能证据

> **历史阶段记录**（T16④ 补注，2026-10-11）：本文为旧线阶段存档，「当前/已完成/待做」类陈述仅反映撰写时点；现行状态以[进度主稿](../architecture/native-progress.md)、[覆盖清单](../native-coverage-manifest.md)与[规则基准](../architecture/native-rules-baseline.md)为准。

本页保留测试口径、旧参考线的实测数据，并吸收 2026-09-24 优化报告中可复核的静态结论。**当前通过数、失败和后续项只见[进度主稿](../architecture/native-progress.md)**；此处旧轮次和旧线数字不代表当前原生线。

## 验证入口

- 原生内核：cargo test --offline -p engine-native
- 原生 runner：cargo run --offline --release -p engine-native --example n09_runner -- 20 9000
- 旧线定向：cargo test --offline -p engine-core --lib journal_equivalence_tests；cargo test --offline -p engine-core --lib cache_boundary_tests
- 旧线工作区与跨目标：cargo test --workspace；cargo run -p conformance；bash scripts/cross-target-hash.sh
- benchmark-runner 当前仍测 engine-core，例如 trigger-dispatch、identity-match。

[phase19 原生基准](phase19-native-baseline.md)与本页旧线数据负载不同，不能相除得出提速倍数。第 48 轮 v59 全绿与 9-24 时的 v60 回放失败不能当作同一次验证（当前原生结果见主稿）；14 个 golden 是一个场景测试内部的 14 个期望，不额外计 14 项。正式平台、跨目标和旧上游浏览器记录须单独标注来源。

## 2026-09-24 优化假设复核

原独立优化扫描中的 P0/P1 排序、预估收益和按周路线图**没有同口径基准，已撤回**。以下只确认源码、目录与构建配置的静态事实，不能由“存在”推断当前瓶颈。执行项 O1–O5 和验收条件见[进度主稿](../architecture/native-progress.md#性能优化核验后的待办2026-09-24)。

| 范围 | 已核实的事实 | 处理边界 |
|---|---|---|
| 构建/测试 | engine-core/tests 有 1,078 个根目录集成测试文件，宽口径 setup/create_game 匹配 603 个文件；release 配置启用 LTO 与单 codegen unit | 先测干净/热构建、链接数及磁盘；仅在试点有效后合并测试。90% 编译收益和大规模夹具删减未测 |
| 查询/投影/AI | [query](../../crates/engine-core/src/query/mod.rs)经 query_explain；缓存命中会提前返回。[命令提交](../../crates/engine-core/src/engine.rs)与[投影](../../crates/engine-core/src/projection.rs)计算哈希；[AI fork](../../crates/engine-ai/src/fork.rs)走正式 submit | 测命中、分配、扇出和节点成本；不得直接跳过合法性/不变量校验。5–10 倍与固定 8 次重复哈希未证实 |
| Store 遍历 | 多个 sorted_live 收集排序，PlayerStore 已有 iter_live | 先证明 ID→槽位顺序在所有变更/恢复路径成立并与 golden 对照；座次不能直接替换为 ID 顺序 |
| WASM/依赖 | 本地 WASM 文件约 7.75 MB，wasm-opt 为 false；[engine-core 依赖](../../crates/engine-core/Cargo.toml)的普通 tempfile 在 src 未使用，workspace 已声明；原扫描的 Tokio features 漏实际使用的 fs | 固定工具链测原始/压缩体积、加载与功能；先做低风险依赖修正。1.5 MB 目标和 panic/unwind 归因未证实 |
| 格式/原生派发 | 旧线[快照](../../crates/engine-core/src/snapshot.rs)与[回放](../../crates/engine-core/src/replay/mod.rs)用 JSON，回放命令字节参与哈希链；原生[技能接口](../../crates/engine-native/src/catalog.rs)用 trait object | 改格式必须兼容旧读取与版本化哈希链；动态派发分配次数和收益未测，不因文本出现次数引入大枚举 |
| 文档/大文件/工具 | ABI 文档、ADR、catalog 较大且有消费者；部分 Rust 文件超过 2,000 行；Blake3 小辅助与签名实现有重复；目录名称不统一 | 不按体积、行数或命名直接立性能任务。仅在维护缺陷或剖析指向它们时局部整理，并核对路径消费者 |

该复核仅做静态核对，**未运行干净构建、分配剖析、WASM 配置对照或可重复性能实验**。原报告中“测试代码量为业务代码 2 倍”也不成立：粗口径两组 Rust 行数约 272,361 / 337,551，且都含混合内容；此比值不宜作质量指标。

## 旧参考线 release 历史记录（2026-09-15）

原 agent 在修正触发场景口径后记录；本轮未重跑旧线性能，不将其当作当前独立复验。本笔同时是
`trigger-dispatch` 场景口径修正的对照锚点：**修正前**该场景以固定
`ExpectedRevision(0)`/`IdempotencyKey(0)` 提交并**忽略 submit 结果**，全部按提交数计入
"吞吐"；逐层修正时实测两条拒绝路径：仅修 revision/幂等键而不应答选择时，
100,000 次提交 accepted=3、rejected=99,997（全部 `illegal_action:choice pending`）——
即旧口径的数字几乎全部来自拒绝路径。**修正后**吞吐只统计实际接受的命令，
选择以确定性中性策略自动应答，
并记录事件数、终局条件（finished/outcome/rounds）与 P50/P95/P99。

环境：Apple Silicon `aarch64`（macOS 27.0，10 性能核）、**release profile**、
`ENGINE_BUILD_ID` 同日。命令：`cargo run --release -p benchmark-runner`。

| 场景 | 本次结果（release） |
|---|---:|
| `start-hash`，4 人 seed 42，200 samples | P50 243,125 ns / P99 355,000 ns |
| `identity-match`，4 人 seed 42，10 samples | P50 5 ms/整局（未触 5000 步上限） |
| `trigger-dispatch`，100,000 次 | **accepted=100,000**（EndPhase 85,714 + ResolveChoice 14,286）rejected=0 events=203,570 P50 67,917 ns / P95 80,583 ns / P99 91,042 ns finished=false rounds=1,786 |
| `distance-query`，100,000 次 | 251 ns/query |
| `snapshot-restore`，500 次 | 85,344 ns/op |
| `observer-projection`，50,000 次 | 21,061 ns/op |

口径说明：

- `trigger-dispatch` 的 `finished=false, rounds=1,786` 表示 10 万条命令驱动后
  对局仍在进行（8 人局未到终局条件），该行不是整局耗时；整局耗时看
  `identity-match`（含终局断言 `!capped`）。
- 本笔只描述旧线。已删除的 dev 数字与此表负载/构建/计数口径不同，不能相除计算提速倍数。
- 目标设备与预算遵循性能架构第 11 节；旧 M7 档位不自动成为新线发布标准。
- `per_op_ns` 当前按总耗时 /（接受 + 拒绝）计算，不能在含拒绝时当作纯接受命令耗时；P50/P95/P99 仅来自接受样本。
- 新线必须另立场景并报告相同规则与信息条件，不能与此表混排；新增每命令段前快照的复制/编码开销须计入新线测量。

---

## 原生单机数据点（2026-10-19，**不可用于验收**）

新增证据：[`2026-10-19-native-perf-datapoint/README.md`](2026-10-19-native-perf-datapoint/README.md)

| 观测 | 值 | 限制 |
|---|---|---|
| 双仓桥接 + 建局（350 张牌）| **476 µs** | **debug** 构建；单次采样 |
| 一次被拒命令 | 125 ns | 被拒路径，不代表正常命令 |

**为什么不能当验收**：八项的性能门槛要求**同设备、同工具链、同负载的前后数据**；
旧引擎（`engine-core`/`engine-ai`/`engine-wasm`）**没有在本机跑过同负载**，也没有 release 数据。
⇒ 本项**仍为待办**，此表只固定「原生侧有一个可复现的起点」。
