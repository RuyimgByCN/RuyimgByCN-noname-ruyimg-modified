# 2026-10-28 23:14 完成情况扫描（第二次）与过期描述更正

本目录记录 23:14 起对 HEAD 的状态扫描、独立复跑结果，以及据此对主稿过期描述所做的更正。**本批没有修改产品代码或测试，没有升级任何版本，也没有提交或推送。** 当前状态以[进度主稿](../../../architecture/native-progress.md)和[剩余台账](../../../architecture/native-remaining-ledger.md)为准。上一次扫描见[16:22 扫描](../2026-10-28-status-scan/README.md)。

## 1. 基准

- HEAD：`d7b46ea6`（[source-head.txt](source-head.txt)）；规则 v325（`crates/engine-native/src/content/standard.rs:136`）、存档 v105、全局原生 ABI 753（`crates/engine-api/src/version.rs:17`）。
- 扫描开始时工作树干净（[status-before.txt](status-before.txt)），磁盘余量 177 GB（[disk.txt](disk.txt)），没有其他 cargo、rustc 或测试进程在运行。
- 上一基准 `0a0eba8c` 之后入库五批，均有证据目录：

| 提交 | 批次 | 版本影响 | 证据 |
|---|---|---|---|
| `5eec1e9e` | NC07 炼妖壶壶仓入规范摘要（GT-012） | 规则 v324→**v325**（裁决第 9 项） | [NC07](../2026-10-28-nc07-lianyaohu-hash/README.md) |
| `cd028d88` | GT-014 本地 VM 正式导入入口（安装、升级、回退、停用、卸载） | 无 | [GT-014](../2026-10-28-gt014-local-vm-install/README.md) |
| `462433ea` | NC08 方案 A 差分统计（37 来源命中 9 组） | 无（只统计） | [统计](../2026-10-28-nc08-plan-a-statistics/README.md) |
| `39dab651` | ADR 0786 草案：效果级选择窗挂起与主动技桥 | 无（只设计） | [ADR 0786](../../../adr/0786-vm-effect-choice-window-and-active-bridge.md) |
| `d7b46ea6` | NC04 `cards_given` 给牌触发帧接入 | **未升号**，但改变了摘要域（见第 4 节） | [cards_given](../2026-10-28-nc04-cards-given/README.md) |

## 2. 独立复跑（HEAD 源码）

运行时间见 [runs.txt](runs.txt)，原始输出均在本目录。

| 命令 | 结果 | 与各批报告对照 |
|---|---|---|
| `cargo test --offline -p engine-native --no-fail-fast` | **1624/0/73**，334 个测试二进制（[log](engine-native.log)） | 与 `cards_given` 批一致 |
| `cargo test --offline -p engine-native-wasm --no-fail-fast` | **68/0/0**（[log](engine-native-wasm.log)） | 一致 |
| `cargo test --offline -p content-native-loader --no-fail-fast` | **32/0/0**（[log](content-native-loader.log)） | 一致 |
| `cargo test --offline -p content-compiler --no-fail-fast` | **541/0/0**（[log](content-compiler.log)） | 与 11:30 扫描一致 |
| `cargo test --offline -p rule-vm --no-fail-fast` | **21/0/0**（[log](rule-vm.log)） | 与 11:30 扫描一致 |
| `cargo test --offline -p game-server --no-fail-fast` | **362/0/0**（[log](game-server.log)） | 与 GT-014、`cards_given` 批一致 |
| `cargo check --offline --workspace --all-targets` | exit 0，867 条既有警告（[log](workspace-check.log)） | 一致 |
| Gate 1b：`RUSTFLAGS=-D warnings cargo check --offline -p engine-native --all-targets` | **exit 101**，content-compiler 2 条错误（[log](gate1b.log)） | NC07 批已记“被既有警告阻塞” |
| `pnpm --dir apps/web-client test` | **123/0/0**（[log](web-test.log)） | 一致 |
| `pnpm --dir apps/web-client typecheck` | exit 0（[log](web-typecheck.log)） | 一致 |
| `pnpm --dir apps/web-client build:native` 后 `smoke:native` | 重建 exit 0（[log](build-native.log)），smoke **72/0**（[log](smoke-native.log)） | 与 GT-014、`cards_given` 批一致 |

Gate 1b 用独立的 `CARGO_TARGET_DIR`（会话临时目录）运行，避免改变 `RUSTFLAGS` 后让共享 `target/` 整体重编。两条错误分别是 `crates/content-compiler/src/codegen/effect_set_draw_phase_from_bottom.rs:4` 的未使用导入，以及 `crates/content-compiler/src/abi_versions.rs:742` 的未使用常量 `ADDITIVE_RULE_VM_ABI_VERSION_755`。`git log` 显示二者由 B6（`72aa29cf`，删 754、增 758）引入。CI 的 Gate 1b 步骤（`.github/workflows/ci.yml:50`）会因此失败。

本次**没有**跑：engine-core（旧线，最近为 11:30 扫描的 3846/0/1）、CI Gate 2–4（conformance、跨目标哈希、原始内容同步）、上游对照、目标设备和性能基准。

## 3. 更正的过期描述

| 文件 | 原描述 | 更正 |
|---|---|---|
| [进度主稿](../../../architecture/native-progress.md)“当前基准” | 20:06 NC07 批：HEAD `0a0eba8c`，只写到 NC07 与 GT-014 | 改为本次扫描：HEAD `d7b46ea6`、五批提交、复跑数字、Gate 1b 失败、未跑范围、待裁决项 |
| 进度主稿第 11 行 | NC07“20:06 批”，无提交号 | 补 `5eec1e9e` |
| 进度主稿优先级表 | 无 Gate 1b 条目 | 新增第 12 行（Gate 1b 红） |
| [剩余台账](../../../architecture/native-remaining-ledger.md)头部“当前基线” | 同进度主稿的 20:06 口径 | 同上 |
| 剩余台账阻断表 | 状态行停在“11:30 扫描”；Gate 1b 只在各批证据中被记为“既有警告阻塞” | 状态行改为本次扫描；新增 **B8 / P2**（Gate 1b 红），写明来源、完成条件与证据 |
| 进度主稿与台账“2026-10-28 进展登记（按提交顺序）” | NC08 统计段落排在 ADR 0786 与 `cards_given` 之后 | 按提交顺序移到 GT-014 之后、ADR 0786 之前，文字未改 |
| [规则基准](../../../architecture/native-rules-baseline.md)标题 | “NC07 炼妖壶摘要，规则 v325，本批随证据入库” | 改为“提交 `5eec1e9e`” |
| [实施规范](../../../../../NONAME_NEXT_AGENT_SPEC.md)（仓库外）GT-002、GT-012、GT-014 | 入口测试只列反例探针与旧快照测试 | 补 `nc07_lianyaohu_hash.rs`、`native-vm-install.test.mjs`、`gt014_vm_import_parity.rs` 与 NC08 统计证据 |

带日期的进展登记、各批证据和[用户裁决登记](../2026-10-28-user-decisions/README.md)属于历史或授权记录，未改写。

## 4. 待裁决项的核对发现

1. **NC08 触发全序**：上游 `arrangeTrigger`（`archive/upstream/noname/apps/core/noname/library/element/content.ts:4017` 起）只要同一角色有多个同优先级、非 silent、有描述的技能，就无条件 `chooseControl` 询问顺序，没有可关闭的配置。统计证据第 3 节第 5 条写明核心武将包（standard、offline、jsrg 等）不在 9 组之内。
2. **NC07 其余字段**：[字段审计](../2026-10-28-nc07-lianyaohu-hash/archive-vs-hash-field-audit.log)列出 PlayerState 9 个、TurnState 1 个入档未入摘要的字段，且都在规则路径上被读取（同文件末尾的引用计数）。该审计是文本匹配，逐项是否影响结算仍须反例确认。
3. **GT-014**：客户端导入期只做结构校验；破坏模块指纹的两类变异在本地导入时被接受、开局时才被 Rust 校验器拒绝，服务端导入时即拒（GT-014 证据第 4 节）。本地验签缺少客户端侧的信任链。
4. **ADR 0786**：J1 的原语方案②需要升 `RULE_VM_ABI_VERSION`。宿主 ABI 是单一整数，当前为 753，而编译器已有 755–758 级常量（`abi_versions.rs:742–745`），N01 onlyOL 需要 758。宿主 ABI 升到任何高于 753 的值，都意味着宣称支持 754 起的各级原语，须先逐级确认这些原语已在宿主实现。
5. **`cards_given`**：新帧使 `next_event_id` 前移，而该计数在规范摘要中，sancai 场 golden 因此位移（NOEMIT 探针已归因）。同为 v325 的 `5eec1e9e` 与 `d7b46ea6` 两个构建，在含给牌的轨迹上摘要不同。另外，原生按张发帧，旧线一次给牌只发一个批量事件：`event.rs` 注释与规则基准都写明“多张给牌触发次数不同”，即监听 `cards_given` 的技能在一次给出 N 张时，原生触发 N 次、旧线 1 次。
6. **NC09 选择窗超时**：上游联机“出牌时限”为房间配置，默认 30 秒，可选 10／15／30／60／90 秒（`ui/create/menu/pages/startMenu.js:174`）。到时加 5 秒宽限后，服务端执行 `player.unwait("ai")` 由 AI 代答，并关闭该座连接（`library/element/player.js:9950` 起）。

用户随后对以上各项答复“照建议登记”，见[裁决登记](../2026-10-28-user-decisions/README.md)第 10–15 项（23:28 登记）。

## 5. 完成情况摘要（2026-10-28 23:14）

- **已实施并入库（本日）**：B7（GT-011）、本地身份与牌面投影（GT-013 本地部分）、NC07 炼妖壶摘要（GT-012）、GT-014 本地 VM 正式导入（第一类真实案例）、`cards_given` 事件层；NC08 反例测试与方案 A 统计；ADR 0786 草案。
- **门禁问题**：Gate 1b 红（B8），自 B6 起未修。
- **待用户裁决**：第 4 节五项；NC08 方案 B、NC09 超时参数与命令扩展、服务端 suit／rank 与 kind 统一仍未实施。
- **仍未完成的大项**（摘自台账，本次未逐项复核）：NC05 混合包、依赖链、真实第三方作者产物；完整 shiji 仍被 ABI 756 与其余未映射事件和滤镜阻断；N01 onlyOL 需要 ABI 758；37 来源没有一个完成 G0–G5；PA08 真机未接入；NC10 VM 与挂起态 AI 信息边界；NC04 故障与恢复矩阵未全。

## 6. 文档核验

见 [doc-checks.log](doc-checks.log)。
