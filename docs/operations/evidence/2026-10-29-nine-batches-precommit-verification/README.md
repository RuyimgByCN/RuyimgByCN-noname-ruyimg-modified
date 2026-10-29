# 九批工作：核验、重建与入库（2026-10-29）

用户已在单独确认问题中明确授权九批分别入库。本轮从 `95f51867` 正向重建九个实际提交，逐树核验，最终以 `c7a55d98` 接入 main；以下保留开工前核验和原提交提案，实际接链结果见末节。没有 push、新增 remote 或丢弃工作文件；F7 第 4 步尚未开始。

## 基准与保护

- 实测 HEAD：`95f51867a3648321da1cc92149b60e2bbfaae748`。
- 开始时间：2026-10-29 20:26:23 +08:00；环境、工具链和 dirty 指纹见 [environment.json](environment.json)。
- 规则 v328、存档 v105、原生 ABI 753；分层棘轮 `BASELINE=240`。
- 原有脏路径 757 个，逐文件 SHA-256 见 [dirty-files-before.json](dirty-files-before.json)；[原始 Git 状态](git-status-before.txt)。核验时真实暂存区为空。
- 不使用 reset/clean/stash，不 push、不新增 remote，不改写原有源码。构建只使用现有 target 和 Gate 1b 独立目录，不清理共享 target。

## 已确认的九批顺序（原提案）

以下是提交主题与证据归属，**尚未表示每个中间提交已重建并验收**。同文件含多批改动时须拆 hunk 或在保留当前工作树的前提下重建候选树；不能按目录整体暂存。版本批次严格保持 v326→v327→v328 的先后顺序，九批最终树须与批准时的指纹核对。

| 批 | 拟提交主题 | 既有证据 |
|---|---|---|
| ① | `fix(ci): restore native Gate 1b warning checks` | [B8](../2026-10-28-b8-gate1b/README.md) |
| ② | `fix(native): batch cards_given once per transfer (v326)` | [NC04 v326](../2026-10-29-nc04-cards-given-v326/README.md) |
| ③ | `fix(extensions): validate local VM packages before import` | [GT-014](../2026-10-29-gt014-import-deep-validation/README.md) |
| ④ | `fix(native): hash archived semantic fields (v327)` | [NC07 v327](../2026-10-29-nc07-archive-hash-fields/README.md) |
| ⑤ | `feat(native): suspend VM effect commits for card choices (v328)` | [ADR 0786 J1](../2026-10-29-adr0786-j1/README.md) |
| ⑥ | `docs(nc08): record core trigger-order overlap survey` | [NC08 统计](../2026-10-29-nc08-core-pack-statistics/README.md) |
| ⑦ | `perf(tests): group native and legacy integration binaries` | [O6](../2026-10-29-o6-test-grouping/README.md) |
| ⑧ | `refactor(native): add layering ratchet and content hooks` | [F7 第 1–2 步](../2026-10-29-native-split-stage1-2/README.md) |
| ⑨ | `refactor(rust): enforce 1000 effective source lines` | [有效行拆分](../2026-10-29-rust-effective-1000/README.md) |

每个提交正文引用对应证据目录。进度主稿、剩余台账、规则基准、测试版本断言和 golden 含多批改动，须随各批正确版本分别入库；F7 与有效行拆分还与 CI/preflight、源码扫描测试及文档 file:line 引用交叠。无法从证据确认归属的改动先列明，不擅自并入。

既有证据完整性限制：第⑨批目录当前仅有 README，没有原始测试日志或逐文件拆分补丁；第⑧批目录有 README 与三份测试/门禁日志。当前工作树的重新验证只能补充当前组合状态的证据，不能反推历史九个中间态逐一通过。

## 本轮命令与结果

- 首轮 `cargo check --offline --workspace --all-targets` 因单次工具调用 60 秒上限中断，未取得完成退出码；保留 [首次日志](workspace-check.log)，不计通过。
- 相同命令后台重跑 **exit 0**，91.466 秒；[完整日志](workspace-check-retry.log)、[退出码](workspace-check-retry.exit)。
- 其余验证的命令、环境覆盖、开始/结束时间、退出码按完成顺序写入 [verification-results.json](verification-results.json)，进行中命令见 [verification-active.json](verification-active.json)。尚未出结果的检查不计通过。

本轮不把旧 WASM 产物对照、历史绿色计数或 README 声明当成本次源码的产品/平台/上游等价验收。

## 入库边界审查与新发现

只读子 agent 的完整审查见 [nine-batches-boundaries.md](nine-batches-boundaries.md)。757 个原始脏路径按职责可归于九批，未发现必须另解释为第十批产品功能的路径；这是文件归属结论，不是全部搬移代码逐字或行为守恒认证。

**不能直接整文件暂存为九批**：版本、golden、版本断言与后续模块拆分交叠。尤其 NC07 新守卫当前引用第⑨批才新增的 hash 模块，J1 的提交泵、效果编解码和摘要又已搬至第⑨批的新文件，F7 宿主询问文件也被第⑨批继续拆分。每个候选提交必须重建当批形态并验证，不能以当前组合状态编译通过替代。

本轮 `cargo test --offline --workspace --no-fail-fast` 已发现桌面客户端两项失败（全套尚在继续，不提前给最终汇总）：`tests::freeze_blocks_install`、`tests::signed_install_via_host`，均因测试包模块 `m` 的字节码报 `unknown opcode 0x2`；见 [workspace-tests.log](workspace-tests.log)。尚未判定该失败的引入批次，不把它擅自归为第⑨批回归，也不把后续定向补跑写成该次全量通过。

文档审查发现第⑨批修改过的行号范围仍有越界终点，例如 `state/core_ops.rs` 的旧行号 `90/1192`（现文件 929 行）和 `kernel/sha/step.rs` 的旧范围 `693–1243`（现文件 1066 行）。后续须按符号/实际内容修正文档并通过严格检查；不能只验证起点后宣称范围有效。

九批入库许可已获明确答复。核验日志不混入历史九批证据，新发现的修复没有悄悄并入历史语义批。

## 本次重建与实际接链

| 批 | 实际提交 | 当批规则版本 |
|---|---|---|
| ① B8 | `67dadfa64c5d43b76e20f91506b49baee6e6496a` | 325 |
| ② NC04 | `31a30673f29b4f62541ff0c5419a262440b9ce50` | 326 |
| ③ GT-014 | `5ce54b1a498d9cbdf055e15c1ebd248503a09a60` | 326 |
| ④ NC07 | `999031698ea8c78d06f57e47329bcb01f5f16796` | 327 |
| ⑤ J1 | `b7ebdf30a96c3848c30f1466b54f70b004de4b34` | 328 |
| ⑥ 统计 | `5efb7170df0e12ebfefa2b001e23c9abdeb1a886` | 328 |
| ⑦ O6 | `01b286932e48804c43ae96fdce3f33f9423a603a` | 328 |
| ⑧ F7 第 1–2 步 | `23af03f4a725d6953386aed6916578f23318b7e1` | 328 |
| ⑨ 有效行拆分 | `c7a55d9846fc1db311f632dddaaf26231b0a9e10` | 328 |

每批实际命令、退出码和失败经过分别见 [①](reconstruction/01/command-index.json)、[②](reconstruction/02/command-index.json)、[③](reconstruction/03/command-index.json)、[④](reconstruction/04/command-index.json)、[⑤](reconstruction/05/command-index.json)、[⑥](reconstruction/06/command-index.json)、[⑦](reconstruction/07/command-index.json)、[⑧](reconstruction/08/command-index.json)、[⑨](reconstruction/09/command-index.json)。这是本轮重建验证，不能冒称为原始历史中间态快照。

- [逐树边界断言](reconstruction/boundaries.json)：各批版本、golden、模块拆分及扫描消费者没有提前混入后批；O6 测试全名守恒。
- [最终冻结树核验](reconstruction/final-snapshot-verification.json)：757/757 路径匹配，756 个 blob SHA-256 相等、1 个删除相等；变更路径集合无新增、无遗漏。
- [主分支接链](main-adoption.json)：先核对祖先关系和真实暂存区为空，只更新 index 和 fast-forward ref，未写工作树文件；828 个当时工作路径内容全部保留。主线程另写的 B9、桌面夹具与文档门禁修复继续在工作树中，未覆盖。
- 第⑨冻结源码 workspace **6726/2/74** 的两个桌面夹具失败保留；主线程独立修复后当前 workspace 全跑 **6729/0/74**，见 [B9 同轮记录](../2026-10-29-b9-danshou-deferred-hash/README.md)。不能将后者追记为第⑨历史全绿。
- ①–⑧继承旧有效行超限（①–④ 182 个、⑤–⑧ 184 个），当批运行的是原 2,000 总行门禁；⑨引入并强制 1,000 有效行后超限为 0。见[逐树计数](reconstruction/effective-lines-by-tree.json)，没有放宽最终门禁。
- 重建 agent 曾达到 30 分钟上限退出，已保存前七提交及第八未完差异，再从断点续接一次；未重做前七。首次环境/编译/断言失败和后续修正均保存在重建日志。第八因缺隔离父规范链接而失败的完整 preflight，没有被单项补跑写成完整通过；第⑨独立完整 preflight **21/21**。
