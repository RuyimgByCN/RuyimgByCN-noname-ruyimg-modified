# 2026-10-28 NC07 炼妖壶壶仓入状态摘要（GT-012，规则 v324→v325）

范围：NC07／NC06，GT-012；授权条目为[用户裁决登记](../2026-10-28-user-decisions/README.md)第 4 项（范围）与第 9 项（版本号 v324→v325）。本批只把 `PlayerState.lianyaohu_store` 纳入规范摘要并升规则版本；**存档格式 v105、全局原生 ABI 753、Worker／WS 消息协议均不变**，没有旧档迁移。本目录同时保存 11:10 的 v323 基线捕获（第 1 节）。

## 1. 基准与捕获

| 时点 | 文件 | 内容 |
|---|---|---|
| 11:10（HEAD `f672129f`，规则 v323） | [start-time.txt](start-time.txt)、[source-head.txt](source-head.txt)、[clean-dir.txt](clean-dir.txt)、[capture-v323.rs](capture-v323.rs)、[capture-v323.log](capture-v323.log) | 在干净副本捕获 `native-v323.ngsa`（标准 4 人局）与 `lianyaohu-v323.ngsa`（shahai 包，0 号授炼化、壶仓 `[8, 80]`） |
| 20:06（HEAD `0a0eba8c`，规则 v324，实施起点） | [impl-start-time.txt](impl-start-time.txt)、[impl-source-head.txt](impl-source-head.txt)、[impl-status-before.txt](impl-status-before.txt)、[impl-disk.txt](impl-disk.txt) | 三批（B7 `cfacaf1c`、投影 `79cccc0b`、NC08 `103b7499`）已入库，工作树只剩本目录未跟踪；磁盘余量 177 GB |
| 20:08（同 HEAD，修复前） | [capture-v324.rs](capture-v324.rs)、[capture-v324.log](capture-v324.log) | 同布阵捕获 `native-v324.ngsa` 与 `lianyaohu-v324.ngsa`（壶仓 `[8, 80]`），作升号后版本拒绝回归的夹具 |

开工核对：`STANDARD_RULESET_VERSION` 为 324（`standard.rs:132`）；v325 未被任何已入库或在途批次占用（`git log`、台账与裁决登记第 9 项）。

## 2. 反例与根因

原始反例（[NC06 证据探针](../2026-10-28-nc06-zhucheng-restore/lianyaohu-hash-probe.rs)）：两份只在 `lianyaohu_store` 顺序不同的存档，`state_hash()` 相同，但发动炼化（`lianyaohu_lianhua`）得到的 `ChoiceSpec.options` 顺序不同。根因：`engine/hash.rs` 的 `canonical_hash` 逐玩家写入 `zhucheng`、`off_court`、`sxianjing_traps`、`hutian_hold` 等牌区，但没有写 `lianyaohu_store`；而归档（`archive/state_encode.rs:150`）、恢复校验（`snap/validation.rs:398`）与炼化步骤（`content/host/step/swd_equipment.rs`：选项 = `lianyaohu_store.iter()` 顺序，结算按作答序 `remove`+`to_discard`）都依赖该顺序。

## 3. 改动

| 文件 | 改动 |
|---|---|
| `crates/engine-native/src/engine/hash.rs` | `canonical_hash` 玩家段在 `zhucheng` 之后无条件写入 `lianyaohu_store` 长度与牌序（与相邻牌区同惯例，+6 行） |
| `crates/engine-native/src/content/standard.rs` | `STANDARD_RULESET_VERSION` 324→**325**，补 v325 版本注释 |
| `crates/engine-native/tests/nc07_lianyaohu_hash.rs`（新增） | ①`different_jar_orders_have_different_hashes_and_trajectories`：反例断言由「hash 相同」**反转为「hash 不同」**，并检查完整轨迹——归档往返各自摘要不变且仍区分、炼化选项精确等于各自壶序、各按自己窗口顺序作答后壶仓清空、弃牌堆末两张分别为 `[first, second]`／`[second, first]`、结算后摘要仍不同；②`v324_and_earlier_archives_are_rejected_by_rule_version`：`native-v323/v324.ngsa` 以 `RuleVersionMismatch { snapshot: 323/324, catalog: 325 }` 拒绝；`lianyaohu-v323/v324.ngsa`（扩展目录版本 = 标准版 ^ 内容锁高位混合）同样拒绝，并断言 `snapshot ^ catalog == 323/324 ^ 325`（差异只来自标准版本位） |
| `crates/engine-native/tests/pa12_tag656_version.rs` | 目录版本断言 324→325（v322 旧档仍拒绝；新档往返仍一致） |
| `crates/engine-native/tests/nc04_vm_resume.rs` | `b7_plain_v323_archive_is_rejected_by_rule_version` 的 `catalog` 324→325 |
| `crates/engine-native/tests/replay_scenarios.rs` | 14 场 golden 重录 + 探针依据注释（第 4 节） |

设计与替代方案：
- **无条件写长度（采用）** vs 非空才写：hash.rs 内所有玩家牌区都是无条件写长度，沿用惯例避免分支与歧义；代价是标准场景每玩家多 4 字节零长度，golden 位移来源由此变成「版本字段 + 空长度字节」两项，故探针需三步而非 B7 的一步（第 4 节）。
- 不改 `state_encode/decode`、`snap/validation`（壶仓早已入档与校验），不改选项生成顺序。
- 旧档策略按授权：v324 及更早一律版本拒绝，不做迁移。

## 4. 反例先红后绿与 golden 探针

| 步骤 | 命令 | 结果 | 日志 |
|---|---|---|---|
| 红（v324 源码，仅新增测试） | `cargo test -p engine-native --test nc07_lianyaohu_hash` | **0/2**：①在 `:91` `assert_ne!(hash)` 失败，两摘要 32 字节逐字相同；②在 `:135` 失败（目录仍 324） | [red-v324-prefix.log](red-v324-prefix.log) |
| 绿（修复 + v325 + 断言同步） | 同上 + `pa12_tag656_version` + `nc04_vm_resume` | nc07 **2/0**、pa12 **1/0**、nc04_vm_resume **9/0** | [green-v325-targeted.log](green-v325-targeted.log) |
| P1：修复 + v325 | `--test replay_scenarios -- --nocapture` | 14 场全部位移；`pa12_print_golden_hex` 输出本批重录值 | [golden-p1-fix-v325.log](golden-p1-fix-v325.log) |
| P2：临时回退 v324、保留修复 | 同上 | **仍 14 场位移**——空壶仓的零长度字节独立移动摘要 | [golden-p2-fix-v324.log](golden-p2-fix-v324.log) |
| P3：临时回退 v324、壶仓临时改为非空才写 | 同上 | **3/0 全绿**——标准 14 场终态壶仓全空（标准目录无炼妖壶，壶仓只由 shahai 包宿主技写入），位移只由版本字段与空长度编码构成，**无行为变化** | [golden-p3-emptyneutral-v324.log](golden-p3-emptyneutral-v324.log) |
| 重录后（恢复 v325 与无条件编码） | 同上 | **3/0** | [golden-final-v325.log](golden-final-v325.log) |

两处探针临时改动已逐字撤销（`git diff` 只含第 3 节改动）。

## 5. 影响面验证（2026-10-28，HEAD `0a0eba8c` + 本批工作树，规则 v325／存档 v105／ABI 753）

| 命令 | 结果 | 日志 |
|---|---|---|
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | OK | [source-line-tests.log](source-line-tests.log) |
| `python3 -B scripts/check-source-lines.py` | 8654 文件 ≤2000 行（+1 新测试） | [source-lines.log](source-lines.log) |
| `cargo test --offline -p engine-native --no-fail-fast` | **1620/0/73**，332 个二进制，0 编译错误（基线 1618 + 本批 2） | [engine-native-full.log](engine-native-full.log) |
| Gate 1b `RUSTFLAGS=-D warnings cargo check -p engine-native --all-targets` | **exit 101**：`content-compiler`（路径依赖）既有 2 条警告在 `-D warnings` 下成错误（`ADDITIVE_RULE_VM_ABI_VERSION_755` 未用，提交 `72aa29cf` 引入）——与本批无关，口径同 B7「他人改动照旧」。补偿核对：不带 `-D warnings` 全目标 check 共 65 条既有警告，**无一落在本批 6 个文件** | [gate1b.log](gate1b.log)、[gate1b-ownfiles-compensate.log](gate1b-ownfiles-compensate.log) |
| `cargo test --offline -p engine-native-wasm --no-fail-fast` | **68/0/0** | [engine-native-wasm.log](engine-native-wasm.log) |
| `cargo test --offline -p game-server --no-fail-fast` | **360/0/0** | [game-server.log](game-server.log) |
| `cargo check --offline --workspace --all-targets` | exit 0（867 条既有警告，0 错误） | [workspace-check-tail.log](workspace-check-tail.log) |
| `pnpm --dir apps/web-client build:native`（从本次源码重建 WASM，输出到 gitignored `src/worker/native-pkg/`，未触碰已跟踪 dist/） | exit 0 | [build-native.log](build-native.log) |
| `pnpm --dir apps/web-client smoke:native` | **64/0** | [smoke-native.log](smoke-native.log) |
| `pnpm --dir apps/web-client test` | **116/0/0** | [web-test.log](web-test.log) |
| `pnpm --dir apps/web-client typecheck` | 0 错 | [web-typecheck.log](web-typecheck.log) |
| 文档：`check_local_links.py --all`、`check-adr-index.py` | 见 [doc-checks.log](doc-checks.log) | |

运行时序见 [chain-summary.txt](chain-summary.txt)。未重跑：engine-core、content-compiler、content-native-loader、rule-vm（本批未改这些 crate）；CI Gate 3 `cross-target-hash.sh`（旧线 engine-wasm 门禁，本批未触及旧线；原生跨目标由 `nc08_cross_target_real` 与真实 WASM smoke 覆盖）；上游对照、目标设备与性能基准。

## 6. GT-012 与测试映射

| 要求 | 测试 | 状态 |
|---|---|---|
| 壶序不同 → 摘要区分 | `nc07_lianyaohu_hash::different_jar_orders_have_different_hashes_and_trajectories`（反转的反例断言） | 反例已修、回归已加、定向通过 |
| 恢复前后牌区顺序、选择与实体守恒一致 | 同上（归档往返摘要与壶序不变、选项精确等于壶序、结算后弃牌序与壶仓清空）；`nc06_zhucheng_restore::all_owned_piles_are_counted_once`（壶仓重复/缺失/越界拒绝，既有） | 定向通过 |
| 旧档按版本门禁拒绝 | `nc07_lianyaohu_hash::v324_and_earlier_archives_are_rejected_by_rule_version`、`pa12_tag656_version`、`nc04_vm_resume::b7_plain_v323_*` | 定向通过 |
| golden 变化原因 | 第 4 节三步探针 | 已查明（无行为变化） |
| 全量 | engine-native／wasm／game-server／前端／smoke | 全量通过（Gate 1b 例外如上） |
| 产品／平台、上游等价 | 未在本批验收 | 未覆盖 |

## 7. 兼容性

- 规则：**v324→v325**（裁决第 9 项，本批独占）。所有规则 ≤324 的原生存档（含 B7 批产出的 v324 档与本目录 4 份夹具）按 `SnapshotError::RuleVersionMismatch` 拒绝；含挂起 VM 机的 v323 档仍在解码期失败（B7 边界不变）。
- 存档格式 v105、全局原生 ABI 753、Worker／WS 消息结构、`view_json` 字段：不变。前端只透传 `ruleset_version`，无改动。
- 旧档迁移：未提供（按授权，另行请示）。

## 8. 未覆盖项与新发现（不在本批授权范围，交裁决）

1. **同类遗漏的静态清单**：用 [archive-vs-hash-field-audit.py](archive-vs-hash-field-audit.py) 比对 `state_encode.rs` 与 `hash.rs`，另有 **PlayerState 9 个、TurnState 1 个**字段入档但从未在摘要中引用（[输出](archive-vs-hash-field-audit.log)）：`card_mods`、`chaofeng_active`、`jiguanyuan_cards`、`jileng_active`、`nigong_lv`、`xingtigaizao_count`、`xuehuang_used`、`xuejibingbao`、`yinmi_active`；`minwan_targets`。其中 `chaofeng_active`／`jileng_active`／`yinmi_active`／`minwan_targets` 直接参与 `play_card.rs`／`play_sha.rs` 的目标合法性判定。这只是字段名口径，每项仍须按 GT-012 做「只改一个字段」反例并经**新的规则版本号授权**后才能纳入摘要；本批未动。
2. 标准 14 场终态壶仓全空，golden 不覆盖非空壶仓的摘要；非空面只由 `nc07_lianyaohu_hash`（shahai 包）回归覆盖。
3. 炼妖壶收纳（`lianyaohu_shouna`）→ 炼化的整局真实触发链、产品 UI 与联机房间未在本批验收（NC05／NC09 范围）。
4. NC07 其余项（手工字段登记收敛、版本／归档／迁移策略统一）照旧。

结束时间见 [end-time.txt](end-time.txt)。
