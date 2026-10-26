# PA01 活跃触发索引：触发规划与按名查技随目录规模劣化的收敛（2026-10-26）

- 日期：2026-10-26；数据文件落盘 01:54–01:54、首次提交 01:56（8319a3fb）+08:00（原记「2026-10-26 01:20–02:10 +08:00」晚于提交或与文件时间矛盾，2026-10-26 17:30 扫描按数据文件时间与提交历史更正）
- 工作包：[台帐 PA01](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26)；「前」数据与测量程序见[内核性能结构复核（2026-10-25）](../2026-10-25-native-arch-perf-review/README.md)
- 源码：`HEAD = 7e28a93799a9c01794085e911a72a4faa5673883` 加未提交工作树；本批改动仅 `crates/engine-native/src/resolution.rs`、`crates/engine-native/src/catalog.rs` 与新增测试 `tests/pa01_catalog_index_equivalence.rs`（`git diff` 核对：resolution.rs 1 hunk、catalog.rs 3 hunk，均为本批）。规则 **v322**、存档 **v105** 未变
- 测量程序：同[复核证据](../2026-10-25-native-arch-perf-review/archbench/)的 `archbench` crate 原样复制到 `/tmp/pa01-archbench`（path 依赖即本仓库），`CARGO_TARGET_DIR` 独立，`--offline --release`（thin LTO、16 codegen units，与复核口径一致），`GRANT=1`

**一句话结论**：两处改动后，archbench `scale` 在 6215 技能目录档的提交 p50/p95 与 215 档相差 ≤3.2%（门限 10%），规划 p50 相差 ≤2.6%；各档步数/事件数与复核证据完全一致（818/2115、1467/4355），行为轨迹不变。

## 1 根因与改动

复核证据只登记了 `plan_triggers` 一处根因。本批实测发现**两个**都随目录技能总数线性增长的热点：

1. **`resolution.rs` `plan_triggers`**（复核已登记）：对 `catalog.candidates(kind)` 的每个定义遍历全部技能实例（目录候选数 × 实例数）。
   改动：改为从实例侧单遍出发（`core.skills.range(..)`，本局 5–11 个实例），直接比对 `TriggerMeta.event/extra_events`。成员关系与 dispatch 列表严格等价（`Catalog::build` 只由这两个字段构建 dispatch，构建后不可变）；入队集合不变，最终顺序由含 `def_id` 的全序键排序决定，与遍历序无关。**未新增任何每局可变状态**（台帐原建议的增量维护索引不需要：实例表本来就是全集，倒序遍历零维护成本）。
2. **`catalog.rs` `skill_by_name` 线性扫描**（复核未登记，本批采样定位）：`GameStateCore::computed_distance` 每次距离计算按名查 10+ 个技能（马术/凶兽/疾风/义从/地听等），`skill_by_name` 是全表 `position` 扫描；AI 合法动作枚举反复调用距离。**修复①后 `scale` 仍有约 2 倍档间斜率，`sample` 采样显示 `Catalog::skill_by_name` 为最大引擎热点（52 采样，`computed_distance` 30）**。
   改动：`Catalog` 构建时生成 `by_name: HashMap<&'static str, SkillDefId>`（`entry().or_insert()` 保持与原线性扫描一致的**首个匹配**语义），`skill_by_name` 改 O(1) 查表。

归因采样（非记录测量）：带 `debug = true` 的归因构建连跑 `scale 8` 时 `sample <pid> 10s`，引擎侧帧计数前列 `skill_by_name` 52 / `canonical_hash` 44 / `play_sha` 32 / `computed_distance` 30。`canonical_hash` 为 PA03 对象（不随 N 增长），本批不动。

## 2 结果（单位 ns，archbench `scale`，GRANT=1，同机同负载窗口）

「前」= 复核证据表 A（2026-10-25，另一负载窗口）；「中间」= 仅改动①（本批首测，如实记录其未达标）；「最终」= 改动①+②。

| 档位 | 指标 | 前(10-25) | 中间(①) | 最终(①+②) | 6215 vs 215（最终） |
|---|---|---:|---:|---:|---:|
| 4 座 提交 p50 | 215 / 6215 | 8,750 / 29,750 | 8,917 / 23,333 | 7,333 / 7,458 | **+1.7%** |
| 4 座 提交 p95 | 215 / 6215 | 16,250 / 186,500 | 15,750 / 173,000 | 10,917 / 11,250 | **+3.1%** |
| 4 座 规划 p50 | 215 / 6215 | 25,709 / 53,834 | 26,334 / 48,334 | 25,000 / 24,708 | **−1.2%** |
| 8 座 提交 p50 | 215 / 6215 | 13,333 / 44,417 | 13,291 / 28,208 | 11,709 / 12,083 | **+3.2%** |
| 8 座 提交 p95 | 215 / 6215 | 24,416 / 214,500 | 25,708 / 197,000 | 16,584 / 17,041 | **+2.8%** |
| 8 座 规划 p50 | 215 / 6215 | 49,625 / 87,625 | 49,500 / 71,292 | 47,916 / 49,167 | **+2.6%** |
| 8 座 规划 p95 | 215 / 6215 | 210,041 / 1,149,291 | 222,958 / 1,061,250 | 188,417 / 187,458 | **−0.5%** |

原始输出：[intermediate-fix1-scale-4.txt](intermediate-fix1-scale-4.txt)、[intermediate-fix1-scale-8.txt](intermediate-fix1-scale-8.txt)、[final-fix2-scale-4.txt](final-fix2-scale-4.txt)、[final-fix2-scale-8.txt](final-fix2-scale-8.txt)。

- 完成条件「6215 档提交 p50/p95 与 215 档相差不超过 10%」：**达成**（≤3.2%）。
- 各档 steps/events 与复核证据逐字一致（4 座 818/2115、8 座 1467/4355）——未持有技能不改变对局轨迹的校验继续成立。
- 负载口径：本批测量窗口 load average 约 14–17（同机他 agent 并发），与复核窗口（8.7–21）同量级；「前」列数字属另一窗口，**不用于计算收益比例**，跨档相对比较与同窗口前后（中间→最终）为有效结论。中间→最终同窗口对比：8 座 6215 档提交 p50 28,208→12,083、规划 p95 1,061,250→187,458。

## 3 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --no-fail-fast`（改动①后） | **1555 passed / 0 failed / 71 ignored**，exit 0 |
| `cargo test --offline -p engine-native --no-fail-fast`（改动①+②后） | **1555 passed / 0 failed / 71 ignored**，exit 0；14 个 golden 不变 |
| `cargo test --offline -p engine-native --test pa01_catalog_index_equivalence` | **2/2**（新增守护：dispatch⟺TriggerMeta 成员等价全事件类扫描 + 合成目录 extra_events/重名首匹配） |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | OK（1 test） |
| `python3 -B scripts/check-source-lines.py` | 通过（8609 文件 ≤ 2000 行） |
| Gate 1b：`CARGO_TARGET_DIR=/tmp/pa01-gate1b-target RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` | **被既有问题阻塞，非本批引入**：`rule-vm` `unreachable_patterns`（`resolve_qixing.rs:768`，来自他 agent 未提交的 N01 tag 656/770/771 改动）、content-vm 既有警告与若干测试文件警告。本批两文件**零警告**（全量警告位置清单中无 `src/resolution.rs`、`src/catalog.rs`） |

规则/存档/状态摘要：无变化（改动均为调度与查询路径，不触状态编码）；`STANDARD_RULESET_VERSION`、`ARCHIVE_FORMAT_VERSION` 未动，golden 未重录。

## 4 未覆盖（不得外推）

1. `growth`（日志长度）与哈希微基准未复测——分属 PA02/PA03 的对象与口径；
2. 真实 37 来源目录的事件分布、两层前瞻宽候选 AI（沿复核 §4 口径）；
3. 旧线 `engine-core` 的 `ActiveTriggerIndex` 未动（台帐本就只要求原生侧）；
4. `card_by_name` 仍为线性扫描（牌表不随技能档增长，本测不敏感；若后续卡牌目录大规模扩张再评估）。
