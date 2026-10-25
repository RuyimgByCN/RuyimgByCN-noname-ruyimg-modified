# PA03 状态哈希：缓冲后一次哈希（字节恒等）与记忆化方案评估（2026-10-26）

- 日期：2026-10-26 03:25–04:00 +08:00
- 工作包：[台帐 PA03](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26)；「前」数据见[内核复核（2026-10-25）表 C](../2026-10-25-native-arch-perf-review/README.md) 与[哈希微基准](../2026-10-25-native-arch-perf-review/hash-microbench/)
- 源码：`HEAD = 7e28a937` 加未提交工作树；本批改动 `crates/engine-native/src/engine/hash.rs`（唯一源码文件）
- 测量程序：同 archbench（thin LTO/16cu release、独立 target）；「前」= 同会话 PA01+PA02 二进制（`growth-pre-pa03-*` 即 [PA02 证据](../2026-10-26-pa02-ai-sim-session-split/README.md)的 `growth-after3-*`），「后」= 本批

**一句话结论**：`canonical_hash` 改为先拼 `Vec` 缓冲再一次 blake3 update——**字节序与原逐字段流式写入完全一致**（全部 golden 与跨目标对照不改期望值即通过），哈希 p50 约减半（4 座 5.43→2.49 µs、8 座 8.73→4.15 µs），提交 p50 −44%～−45%，且提交成本随日志档走平（4 座档间比 1.02、8 座 0.99）。**边界缓存（记忆化）方案已实现并实测后撤销**：状态原地修改面（`core_mut`/`state_core_mut`/公开变体 API/调试入口）远多于整 struct 写回点，18 个测试暴露缓存陈旧；且 growth 只测 submit（每命令本就需要一次哈希），记忆化在该负载下几乎不可见——按台帐「内部改为缓冲后一次哈希**或**分区增量」的内部选项收口。

## 1 改动

`engine/hash.rs`：新增 `HashSink`（`Vec<u8>` 缓冲 + `update` 追加），全部 `push_*` 助手参数 `&mut Hasher` → `&mut HashSink`；`canonical_hash` 末尾一次 `blake3::Hasher::update(&buf)`。blake3 流式与一次性对同一字节序列产出相同摘要，故**摘要字节不变**，无版本门禁需求（规则 v322 / 存档 v105 / golden 均未动、未重录）。

### 已评估并撤销：`state_hash` 记忆化

曾实现 `Engine.hash_cache`（`Mutex<Option<(state_version, hash)>>`）+ `commit_state` 唯一写回点递增版本。全量跑出 **18 failed / 4 个测试面**（g1_hearth、native_kernel、nc06、nc07——含 `[piles] 未参与状态摘要` 这类「应不同却相同」的缓存陈旧特征）：原地修改面（`core_mut()`/`state_core_mut()` 供 ai/general_select，`add_boss_fellow`/`reinit_boss_seat` 直接改字段，debug 入口散布）无法被整 struct 赋值点收敛。即使把版本递增塞进可变访问器，未来新增修改路径仍会静默陈旧——属回归陷阱，撤销。原始失败记录见本 README 源操作日志（`/tmp/pa03_full_test.log`，18 failed / 1541 passed）；撤销后同套件 **1558/0/71** 全绿。

## 2 结果（同会话前后；单位 ns，p50）

| 档位 | hash_p50 前→后 | submit_p50 前→后 | 提交档间比 前→后 |
|---|---:|---:|---:|
| 4 座 0–189 → 568–758 | 5,433→2,491 / 5,766→2,566 | 7,167→3,917 / 7,541→4,000 | 1.05→**1.02** |
| 8 座 0–383 → 1149–1532 | 8,733→4,150 / 9,083→4,150 | 10,834→6,083 / 11,125→6,000 | 1.03→**0.99** |

- 哈希成本 −53%～−54%（微基准预测 1.4–1.7×，与实测一致量级）；提交 p50 −44%～−45%。
- 提交成本随日志档**走平**：此前 hash 是提交里唯一随档增长的分量（复核表 C「约占提交 p50 六至七成」）。
- 与复核证据（2026-10-25 另一负载窗口）对照：8 座 hash_p50 8,600–9,050 → 4,133–4,158；提交 p50 11,958–12,583 → 6,000–6,083（PA01+PA02+PA03 累计）。

原始输出：[growth-pre-pa03-4.txt](growth-pre-pa03-4.txt)、[growth-pre-pa03-8.txt](growth-pre-pa03-8.txt)、[growth-pa03-4.txt](growth-pa03-4.txt)、[growth-pa03-8.txt](growth-pa03-8.txt)。

## 3 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --no-fail-fast` | **1558 passed / 0 failed / 71 ignored**，exit 0；14 golden 未改期望、未重录（字节恒等的最强证据） |
| 失败面复测（g1_hearth/native_kernel/nc06/nc07 + pa01 守护） | **369/0**（记忆化撤销后） |
| `CARGO_TARGET_DIR=… cargo check -p engine-native-wasm` | exit 0（复用 PA02 批的独立 target 增量） |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` / `check-source-lines.py` | OK / 通过（8609 文件） |
| Gate 1b | 同[PA01/PA02 证据](../2026-10-26-pa01-trigger-index/README.md)：被他人未提交 rule-vm 改动的既有警告阻塞；本批仅改 hash.rs，该文件零警告 |

## 4 未覆盖（不得外推）

1. **对外边界按需计算**（Worker/WS 信封/存档/跨目标比对的缓存复用）：评估后不做（§1 撤销理由）；若未来某消费面重复查询成为实测热点，须先收敛状态修改入口再评估；
2. 分区增量哈希未做（台帐的另一个可选内部方案）；
3. 复核哈希微基准的三种 blake3 特性未重跑（aarch64 无差别结论已有原证据）。
