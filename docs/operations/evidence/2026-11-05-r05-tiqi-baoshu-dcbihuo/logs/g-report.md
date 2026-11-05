# R05-G 批报告：存档 NGSA 106→107（detached 实体区入档＋golden 哈希域补 detached）

日期：2026-11-04。基线：r05-private @ dba05fe3（勘察 g-survey.md）。提交：**057967b3**
（`wip(r05-g): archive 107 — detached zone persistence + digest domain`，14 文件，+172/−21）。
命令环境：`CARGO_TARGET_DIR=/private/tmp/noname-r05/target CARGO_INCREMENTAL=0 cargo test --offline`。

## 改动清单（源码 6 处＋测试 8 处）

| # | 落点 | 内容 |
| --- | --- | --- |
| 1 | `crates/engine-native-kernel/src/archive/state_encode.rs:345` | 删「暂不入档」注释；table 之后、rng 之前编码 `s.core.detached_cards`：`w.len`＋逐条 `w.u32(id)`＋`w.u8(kind.tag())` |
| 2 | `crates/engine-native-kernel/src/archive/state_decode.rs:357,535` | `detached_cards: Vec::new()` → `dec_detached(r)?`（读序 table 后、rng 前，与编码严格对应）；去掉 `#[allow(dead_code)]` |
| 3 | `crates/engine-native-kernel/src/archive.rs:120` | `ARCHIVE_FORMAT_VERSION` 106→107＋v107 版本注释（v106 拒收无迁移） |
| 4 | `crates/engine-native/src/engine/hash.rs:436` | `push_cards(&c.table)` 之后追加 detached 域：`push_u32(len)`＋逐条 `push_u32(id)`＋`push_u8(kind.tag())`（b27 先例）。镜像核查：`push_cards` 其余出现仅在 hash/native_invocation.rs 与 hash/machine.rs 的各自载荷域，与核心牌区序无关，无需同步 |
| 5 | `crates/engine-native-kernel/src/snap/ownership.rs` | **勘察外必要补充**：`PhysicalCardZone` 新增 `Detached` 变体＋`physical_cards` 枚举 `core.detached_cards`。没有它，含非空 detached 的快照在 `Engine::restore` 的 N04-f 牌归属校验以 `card_missing_from_all_zones` 拒收（红1 的「ownership 校验通过」断言必需） |
| 6 | `crates/engine-native/src/engine/views.rs:125` | 可见性 match 补 `Detached => false`（对席位/观战隐藏，Referee 全信息路径可见；与 DrawHeld 同门） |
| 7 | 版本门禁断言同步 5 处（各注明「R05-G 批 106→107」）：`tests/pa12_tag656_version.rs:9`、`tests/native_kernel/d4.rs:48`、`tests/nc09_machine_ownership.rs:515`（rg 补漏）、`tests/c331_source33_integration.rs:92`（rg 补漏，全量跑出）、`tests/nc09_gaiming_cancel.rs`（rg 补漏，全量跑出：真实 106 夹具改为解码入口拒收断言；restore 侧 RuleVersionMismatch 语义仍由 g1_d3/e02_c3_lifecycle 等快照级测试覆盖） |
| 8 | 新测试 `tests/r05g_archive107.rs`（115 行，3 测）＋登记 `tests/groups/misc.rs`；fixture `tests/fixtures/detached-106.ngsa` |

## 先红后绿（g-01.log）

- **探针（升号前，pristine tree）**：`debug_start_draw_phase_fixed(0,2)`＋牌守恒布阵（弃堆清空、牌堆全入手、仅留堆顶 1 张）→ R 抽 1 后双枯竭 → `detached=[(103, Unclaimed)]`，deck=discard=0，真实 106 编码 1644 字节落 fixture。
- **红1 守恒**：`r05g_detached_zone_survives_archive_round_trip` 修前红＝解码侧 detached `[]` ≠ `[(103, Unclaimed)]`（整区丢失）。
- **红2 哈希域**：`r05g_detached_is_in_hash_domain` 修前红＝kind 维度（Unclaimed/Destroyed）与存在维度（有/无）哈希全部相同。
- **门禁拒收**：`r05g_genuine_106_archive_with_detached_is_version_rejected` 修前红＝106 档可解码（当时即当前版本）；升号后绿＝`Err(UnsupportedFormatVersion(106))`。106 档解码快照实证缺牌 103（save 时在 detached、106 布局静默丢弃）——升号理由的直接证据。
- 修复后三测全绿（g-01/g-02 间）；restore 路径 N04-f 归属校验经 `Engine::restore(&decoded)` 通过（改动 #5）。

## fixture 信息

- 路径：`crates/engine-native/tests/fixtures/detached-106.ngsa`；**sha256 `d4883fc526714a2ed290f3457422f7852574d3671567051b13f3f3234417ca97`**；1644 字节；头 `NGSA`＋LE 106（`6a 00`）＋ruleset 332。原件自生成后未改动（git 提交内容＝探针原样字节）。

## 全量收尾（g-02.log）

- `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --no-fail-fast`：
  **1951 通过 / 1 失败 / 75 忽略**（1948 存量＋3 新增全绿）。
- 唯一失败＝`replay_scenarios::replay_scenarios_match_golden`，14 场全位移（golden 表期望值为 R05 前旧值）——**照旧失败**，属哈希域无条件长度前缀的预期位移（NC07 v325 空壶仓长度字节同理），J 批三步探针统一重录，本批不修。已在干净 HEAD（stash 验证）复核：修前同为该测试失败、14 场 actual 值不同——本批只移动哈希、无行为变化。
- 门禁：`python3 -B scripts/check-source-lines.py` exit 0（9082 文件）；`python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` OK。

## 边界确认

- 升号只此一批：NGSA 107＝G 批；规则 332 / ABI 758 / WS 7 未动（J 批统一）。
- 机器快照 tag 90/91/92 与 EventData::Draw held_cards 载荷此前已入档（勘察确认），本批不动。
