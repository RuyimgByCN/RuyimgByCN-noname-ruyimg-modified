# 2026-10-28 16:22 完成情况扫描与过期描述更正

本目录记录 16:22 起对工作树的状态扫描、独立复跑结果，以及据此对主稿过期描述所做的更正。**本批没有修改产品代码或测试，没有升级任何版本，也没有提交或推送。** 用户随后要求按批提交，入库经过见第 6 节。 当前状态仍以[进度主稿](../../../architecture/native-progress.md)和[剩余台账](../../../architecture/native-remaining-ledger.md)为准。

## 1. 基准

- 已提交 HEAD：`ec672a1d3c413d39c9972ee02118ae92a21223ee`（规则 v323、存档 v105、全局原生 ABI 753）。
- 工作树（扫描开始时）：20 个已跟踪文件有改动，另有 8 个未跟踪的文件或目录，清单见 [status-before.txt](status-before.txt)，差异指纹见 [source-state.txt](source-state.txt)。最后一次改动在 14:26，扫描时没有 cargo、rustc 或测试进程在运行。
- 未提交改动属于同日三批，各有证据目录：

| 批次 | 内容 | 版本影响 | 证据 |
|---|---|---|---|
| B7／NC04 | `MachineSnapshot::Rvm` 补齐声明技能名、触发事件、限次清单、命中旗标并入摘要；旧档双通道拒绝；14 场 golden 经版本回退探针重录 | 规则 v323→**v324**（裁决⑧） | [B7](../2026-10-28-b7-vm-suspend-restore/README.md) |
| NC03／NC09 | 本地 WASM `view_json` 补观察者过滤的 `identity` 与可见牌面 `cards`，Worker 复用既有字段 | 无（裁决⑥⑦） | [投影](../2026-10-28-nc03-view-projection/README.md) |
| NC08／NC09 | 触发全序反例测试与 A／B 方案；选择窗超时方案骨架 | 无（只取证） | [复核](../2026-10-28-nc08-nc09-review-items/README.md) |

NC07 炼妖壶只有 11:10 的 v323 基线捕获（未跟踪目录 `docs/operations/evidence/2026-10-28-nc07-lianyaohu-hash/`，无 README，未随第 6 节入库），源码未实施。

## 2. 独立复跑（工作树源码）

原始输出在本目录；运行时间见 [runs.txt](runs.txt)。

| 命令 | 结果 | 与各批报告对照 |
|---|---|---|
| `cargo test --offline -p engine-native --no-fail-fast` | **1618/0/73**，332 个测试二进制，编译错误 0（[log](engine-native.log)） | 与复核批最终全量一致 |
| `cargo test --offline -p engine-native-wasm --no-fail-fast` | **68/0/0**，17 个二进制（[log](engine-native-wasm.log)） | 与投影批一致 |
| `cargo test --offline -p game-server --no-fail-fast` | **360/0/0**，25 个二进制（[log](game-server.log)） | 与 B7、投影批一致 |
| `pnpm --dir apps/web-client test` | **116/0**（[log](web-test.log)） | 与投影批一致 |
| `pnpm --dir apps/web-client typecheck` | exit 0（[log](web-typecheck.log)） | 一致 |
| `cargo check --offline --workspace --all-targets` | exit 0，只有既有警告（[末尾 40 行](workspace-check-tail.log)） | 与 B7 批一致 |

本次**没有**重跑：`smoke:native`（需先 `build:native` 重建 WASM 产物；两批证据分别记 62/0、64/0）、engine-core、content-compiler、content-native-loader、rule-vm（三批都没改这些 crate），以及上游对照、目标设备和性能基准。

## 3. 更正的过期描述

| 文件 | 原描述 | 更正 |
|---|---|---|
| [进度主稿](../../../architecture/native-progress.md)“当前基准” | 11:30 扫描：HEAD `f672129f`、规则 v323、engine-native 1612/0/73、B7 未关闭 | 改为本次扫描：HEAD `ec672a1d` 加三批未提交改动、工作树规则 v324、复跑数字、未重跑范围 |
| 进度主稿优先级第 11 行 | “B7 未关闭”“炼妖壶在途未提交” | B7 已修复、待提交；炼妖壶未实施，v324 已被占用，须重新确认版本号 |
| [剩余台账](../../../architecture/native-remaining-ledger.md)头部“当前基线” | 同进度主稿的 11:30 数字 | 同上 |
| 剩余台账 NC07 行 | “在途未提交” | 只有基线捕获、源码未实施（保留原文并注明勘正） |
| [规则基准](../../../architecture/native-rules-baseline.md)头部 | 标题“当前版本（2026-10-27 B6）”；`STANDARD_RULESET_VERSION = 323`（`standard.rs:126`） | 标题注明 B7、未提交；常量改为 324（`standard.rs:132`），并注明 HEAD 仍为 323 |
| [覆盖清单](../../../native-coverage-manifest.md)版本口径 | “源码实测 = 322” | 保留为 2026-10-17 时点，补注当前值只看规则基准头部 |
| [完整执行提示词](../../native-convergence-agent-prompt.md) | 阅读清单停在 10-18 审查，没有剩余台账、裁决登记和实施规范；没有授权门槛 | 补齐三份文档与授权门槛；状态回填改为进度主稿与剩余台账 |
| [B7 证据](../2026-10-28-b7-vm-suspend-restore/README.md) | 全量日志指向已不存在的 `/tmp/b7-native-full.log` | 补注同目录 `native-full.log`（331 个二进制结果） |
| [实施规范](../../../../../NONAME_NEXT_AGENT_SPEC.md) §0、§1.3、§4.1、§4.3、§8.3、GT-002、GT-013 | B7 与投影写成待实施；触发全序复核写成“尚未开始” | 改为指向三批证据；补“服务端牌面与 kind 统一尚未授权”的约束 |

带日期的进展登记（如进度主稿和台账中“NC08 后续（2026-10-28）：WASM 身份投影仍缺”）属于历史记录，后续条目已覆盖，未改写。[用户裁决登记](../2026-10-28-user-decisions/README.md)只记录授权范围，实施状态看台账，同样未改。

## 4. 完成情况摘要（2026-10-28 16:22）

- **已实施、定向和影响面全量复跑通过、未提交**：B7（GT-011）；本地身份与可见牌面投影（GT-013 的本地部分）；触发全序反例测试（GT-002 现行定案的锁定）。
- **已授权、未实施**：NC07 炼妖壶摘要（GT-012；v324 已被 B7 占用，须重新确认版本号）；PA12 §2 J 效果级选择窗与批⑥主动技桥的设计及 ADR 起草（ADR 目录没有新草案）。
- **待用户裁决**：三批改动是否按批提交（扫描后已获答复并入库，见第 6 节）；NC08 触发全序选 A 还是 B；NC09 选择窗超时的时限、到时动作和命令扩展；服务端 suit／rank、公开区牌面与 `kind` 粒度统一（会改变 WS 载荷内容）；NC07 版本号；实施规范是否迁入仓库。
- **仍未完成的大项**（摘自台账，本次未复核）：GT-014 本地 VM 正式安装（`bundledNativePackages.ts` 仍拒绝含 VM 模块的包）；NC05 三类真实案例全生命周期；37 来源没有一个完成 G0–G5；PA08 真机未接入；NC10 VM 与挂起态 AI 信息边界；N01 onlyOL 需要 ABI 758 而原生宿主仍是 753。

## 5. 文档核验

见 [doc-checks.log](doc-checks.log)。

## 6. 按批提交（2026-10-28 17:39 起）

用户在本次扫描后要求“按批提交”。提交前在工作树跑了一次门禁（[commit-gates.log](commit-gates.log)）：行数单测 OK、8653 个文件不超过 2000 行、1865 个文件 0 坏链、ADR 索引通过、`git diff --check` 无问题。代码文件的最后修改时间仍是 14:22:48，所以 16:22 复跑的源码就是入库的源码。

| 顺序 | 提交 | 内容 | 证据 |
|---|---|---|---|
| 1 | `5be78373` | 实施规范 v1.1／v1.2 审查证据；台账 NC08、NC09 两行的登记与对应进展登记 | [v1.1](../2026-10-28-agent-spec-review/README.md)、[v1.2](../2026-10-28-agent-spec-review-v1.2/README.md) |
| 2 | `cfacaf1c` | B7：12 个源码与测试文件、规则 v324、14 场 golden 重录；台账 NC04 行、NC07 行的占号说明、阻断表 B7 行；规则基准 B7 段 | [B7](../2026-10-28-b7-vm-suspend-restore/README.md) |
| 3 | `79cccc0b` | NC03／NC09 本地投影：WASM `view_json`、Worker、3 个测试入口；台账 NC03 行 | [投影](../2026-10-28-nc03-view-projection/README.md) |
| 4 | `103b7499` | NC08 触发全序反例测试；台账 NC08、NC09 两行的复核与方案骨架 | [复核](../2026-10-28-nc08-nc09-review-items/README.md) |
| 5 | 本目录所在提交 | 本次扫描的更正（进度主稿当前基准与第 11 行、台账头部与 NC07 勘正、规则基准头部、覆盖清单、完整执行提示词、B7 证据日志指针），以及本节的入库回填 | 本目录 |

拆分方式：

- 进度主稿、剩余台账、规则基准和 B7 证据 README 里有几批交织的改动。我按批生成中间版本，用 `git hash-object -w` 与 `git update-index --cacheinfo` 分别暂存，每批只带自己的行；工作树始终保持最终内容。每个中间版本都核对过“按批依次叠加后与工作树逐字一致”。
- 台账“2026-10-28 进展登记（按提交顺序）”和进度主稿对应位置原先把 B7 段落写在最后。按实际提交顺序，B7 段落已移到投影与复核两段之前，文字未改。
- 每批暂存后，对源码与主稿跑 `git diff --cached --check`，并检查暂存区内 Markdown 链接的目标都在暂存区（4 批合计 3613 条，0 缺失）。原始日志中的行尾空格（`2026-10-28-agent-spec-review/document-whitespace-initial.log`，记录的正是当时规范的行尾空格）和 cargo 日志末尾空行会被 `--check` 报出。这些属于原始输出，没有改动。
- 中间提交是否可构建，按修改时间和文件范围判断。B7 代码修改于 13:14–13:27，批内 13:35 前全量通过；投影代码修改于 13:42–13:50，在 B7 之上验证；NC08 测试 14:22 写入，在前两批之上验证。三批改的是互不重叠的文件，投影与反例测试不引用 B7 新增的接口。**本次没有逐个检出中间提交重新构建。**
- 未入库：`2026-10-28-nc07-lianyaohu-hash/`。这是 NC07 批在途的基线捕获，源码未实施，留给该批处理。
