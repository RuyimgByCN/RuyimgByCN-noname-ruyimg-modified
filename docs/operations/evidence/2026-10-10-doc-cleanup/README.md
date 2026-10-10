# 2026-10-10 文档清理核对

核验位置：`noname-next`，在提交 `f0bd0af9` 之上改动；规则 v192、存档 v41。本批改动由并行会话以提交 `4b60957b` 原样入库（该会话随后完成了 T1–T3、T7），本页与 [checks.txt](checks.txt) 另行补交。**本批只动文档和一个文档生成脚本的措辞，产品源码、包数据和测试都没改**。过期判断依据 [AGENTS.md](../../../../../AGENTS.md) 的“整理文档”规则：先查引用、独有内容和机器消费者，再决定删、并、改；正式 ADR、原始证据和有独有信息的历史文档保留日期语境。

## 1. 删除

| 路径 | 原因 | 引用与回退 |
|---|---|---|
| `noname-performance-architecture.docx` | v3.2 旧版导出，正文写“不预设第三方动态插件系统”，与 v3.3（按用户要求恢复扩展机制）冲突；md 版本已是 v3.3 | 无入链和机器消费者；md 原句“配套 DOCX/PNG 是旧版导出”已改写；可从 git 历史取回 |
| `docs/migration/upstream-1.11.5-delta.md` 中重复的“批 261——dcmoukui 谋溃状态机”章节（原 8516–8536 行，21 行） | 与 8476–8496 行逐字节相同，是唯一一组完全相同的二级章节；同标题不同内容的三级章节都保留 | 文件名被 `engine-core` 代码与 9 个 `.skill` 注释引用，未改名 |

**审计后决定不删的**：
- `ABI-COORDINATION.md`：被 `engine-api` 用 `include_str!` 读取。
- 覆盖清单：被 `replay_scenarios` 读取。
- `docs/content-sdk/skill-capability-matrix.*`：其 JSON 被 `legacy-analyzer` 测试读取。
- 保真度台账：生成物。
- 旧线内容包 README：`content/original/ported/X` 与 `extensions/X` 有 12 组逐字节相同，这是包副本，不是文档体系的一部分。
- 工作区根的两份 MAINTENANCE 记录：有日期的历史日志，且不在 git 中，删了无法恢复。

## 2. 修正（只改与现状矛盾的陈述，历史数字改为注明日期）

| 文件 | 改动 | 依据 |
|---|---|---|
| [进度主稿](../../../architecture/native-progress.md) | C0/E02 行、原生内核行、O1 行的过时说法；快照段补旧线最近全量（10-07，3777/0）；T4/T5/D1 按 §3 改写；新增 T15（旧线 ABI 账面、哈希缺口、19 将 39 技）和 T16（本次清理未完成部分） | 10-07 维护批证据、§3、§4 |
| `docs/migration/upstream-1.11.5-delta.md` | 页首加 10-10 注；“当前活数字”改为“08-15 当日数字”；10-08 节“stash 后已还原”补核对结果；“下一步”指向主稿 T15 | catalog.json 现为 1.11.7、2576/6190（ported 2557/6151）；§3 |
| `docs/migration/upstream-noname-inventory.md` | “当前活数字”改为“当日数字”；加 10-10 注（19 将 39 技 `source_synced`、上游新增 `card/jiubian.js`）；0581/0512/0582 三个编号注明是 `ABI-COORDINATION.md` 领取记录（无 ADR 文件或与同号文件无关） | catalog.json；`docs/adr` 无 0581/0512 文件，0582 文件是夺锐批 |
| `docs/migration/ruyimg-extensions-inventory.md` | 09-22 横幅改为“由 09-23 范围说明取代”；“ADR 0606”注明为领取记录、无文件 | `docs/adr` 无 0606 文件；`ABI-COORDINATION.md` 1428 行 |
| `docs/migration/phase9-legacy.md` | 加历史参考横幅；页首计数与分级改为当时口径；`apps/extension-studio` 注明已是 headless CLI | `apps/extension-studio/src/main.rs`（analyze/publish/grade） |
| `docs/migration/mode-content-compatibility.md` | 加历史参考横幅（旧线矩阵，原生另有四类模式基线）；指向已删 v3.0 计划的引用注明；M7 接线注明仍未做 | `crates/engine-core/src/mode` 只有 identity/guozhan/single；`engine-wasm` 固定 `identity-standard` |
| `docs/migration/noop-mechanism-survey.md` | “当前活数字 noop 2094”改为普查时数字，当前值指向台账 | 台账 2019 |
| `docs/content-sdk/phase4-dsl-vm.md` | 加历史参考横幅（旧线 DSL/VM 手册）；“Rule VM ABI 为 v651”改指 `version.rs`；画像数“5988”改为以生成文件为准 | `RULE_VM_ABI_VERSION = 734`；Rule IR 129、semantics 255、schema 3 与代码一致 |
| [保真度台账](../../../content-fidelity-ledger.md)与明细（生成物） | 改生成器 `scripts/gen-fidelity-ledger.py` 中两处“架构计划 §47.4”措辞（该计划已删，规则沿用见 ADR 0597），然后用 `--date 2026-10-10` 重跑 | 重跑后数据不变：5595 个 `.skill`，完整 5022 / 近似 573 / TODO 0 / noop 2019；diff 只有日期与这两行 |
| 规则基准、覆盖清单、两仓台账 | 技能名“莫守”→“墨守”、“闪弹”→“霰弹”，共 9 处 | 上游 `translate.js`；覆盖清单中的场景名与机制键未动（`replay_scenarios` 联动守护通过） |
| 两仓台账“当前状态”段、待办整理证据 | 补 §3 的 Liuli 换行复算结果 | §3 |

## 3. Liuli 本地修改 50→7 的复算

- **复算方法**：[liuli-crlf-check.py](liuli-crlf-check.py) 只读，输出见 [liuli-crlf-check.txt](liuli-crlf-check.txt)。以 9-23 基线记录的 50 个修改文件为准，把今天的对应文件转成 CRLF，按 `inventory.py` 的同一算法重算每个来源的 `all_sha256` 与字节数。
- **复算结果**：10 个内容变化来源中，9 个与 9-23 完全一致。其中手杀ui 需把仍在修改中的入口文件也转成 CRLF。只有**史诗卡牌**不一致：消失的 3 个文件（`asset/lib/better-scroll.js`、`cardPhantom.js`、`emoji.js`）还有 148 字节差异，换行无法解释，不同文件组合转 CRLF 也凑不出。
- **10-08 的 stash 仍在对象库里**：`archive/upstream/noname-liuli` 中有悬空提交 `625f8770`（2026-10-08 19:50，“local-mods-before-sync-2026-10-08”）。它只包含今天仍在修改的 7 个文件，与当前工作区逐字节相同，说明 stash 已经重新应用过。史诗卡牌那 3 个文件在 HEAD、该 stash 和当前工作区里完全相同，因此它们在 9-23 时的未提交内容，在 10-08 stash 之前就已被覆盖，git 中无法取回。`archive/Noname-ruyimg-Modified` 中的同名文件也与当前一致，复原不了。
- **结论**：主稿 D1 改为只问史诗卡牌这 3 个文件。T4 中其余 9 个 Liuli 来源只有换行差异，无需复核功能清单。

## 4. 新发现并登记的旧线问题（主稿 T15）

审计中发现的旧线问题都已登记到主稿 T15，这里只列核对依据。

- **ABI 账面**：`ABI-COORDINATION.md` 的 ABI-MAX 与 `crates/engine-api/src/version.rs` 的 `RULE_VM_ABI_VERSION` 都是 734。领取表最后一行却是 753（2026-09-15）。`content-compiler` 常量与 `engine-core` 宿主实现也到 753，例如 `ResolveXiangshu`、`ResolveNhlundao`。
- **npack 版本**：源 npack `content/original/ported/sp|sp2|xianding` 要求 745/753/751。`extensions/` 下 64 个 npack 都 ≤734，但有 11 个包的 `package.npack.json` 低于源 npack，例如 sp2 部署的是 713，源是 753。按 `crates/content-runtime/src/compat.rs` 的兼容规则，宿主 734 会拒装高于它的包。
- **哈希缺口**：`hand_limit_bonus` 不在 `crates/engine-core/src/hash.rs` 的编码内，此前只登记在 `docs/migration/content-registry-arc-split.md`。
- **未移植余项**：catalog 有 19 将 39 技为 `source_synced`。

## 5. 未完成（主稿 T16）

本批由 5 个只读审计 agent 分头调查，其中 2 个因 API 额度错误中断，1 个被停止。下列范围未完成审计或修正，已列为主稿 T16：

- `docs/architecture/architecture-evolution-plan.md`：至少 S6 的 catalog 计数、S11/S13 与 A5 的“当前未修/待迁移”陈述过时。
- 覆盖清单其余行（`kof_session` 行见 T14）。
- 规则基准的非历史段落，例如“本地上游对齐的新增核查”表“标准武将与牌堆”行仍写“80 张自造测试牌”。
- `docs/operations/` 下 18 份旧线阶段文档的过时陈述与横幅。
- `docs/rule-semantics/`、`docs/protocol/`、`docs/extensions/author-guide.md`、`docs/architecture/phase6-web-client.md`、fuzz、cross-target、desktop、content README。本批只做了路径存在性检查，没有发现失效路径。
- [旧实现参考](../../../architecture/legacy-reference.md)的“文档整理”记录还缺本次一节。

## 6. 验证

见 [checks.txt](checks.txt)：本地链接与锚点检查，覆盖清单联动守护 `replay_scenarios`，以及 delta 页重复章节删除前的逐字节比较。
