# 2026-10-10 进度核对

核验位置：`noname-next`，源码为本日建立的本地 git 基线提交 `72583b2b`（规则 v192、NGSA 存档格式 v41）。当前状态与待办见[进度主稿](../../../architecture/native-progress.md)。本轮没有改产品源码和包数据，只核对进度、更新文档并建立 git 基线。上一轮：[2026-10-07 进度核对](../2026-10-07-progress-check/README.md)。

## 版本控制

- 此前 `noname-next` 与工作区根都没有版本控制。10-10 host.rs 被截断后，只能靠会话记录回放修复（见 [v192 恢复证据](../2026-10-10-replay-golden-v192-recovery/README.md)）。
- 本日执行 `git init -b main` 并提交基线 `72583b2b`（31,534 个文件，`git fsck --full` 无错误）。按用户要求只在本地使用：没有配置 remote，`.git/hooks/pre-push` 拦截一切推送。
- `.gitignore` 新增排除 `node_modules/`、`/.codegraph/`、`/.npack-store/`、`/.scratch/`（本地工具与运行时状态，可再生）。`/target` 与 `/content/original/assets/` 原本已排除。
- 提交前确认 `crates/engine-native/src/content/host.rs` 与恢复锚点 `~/host-recovery-20261010/final_allgreen_525pass.rs` 逐字节一致（见 [source-checks.txt](source-checks.txt)）。

## 实跑结果

在 `noname-next` 根目录执行。测试使用工作区共享 `target/`，没有执行 cargo clean；Gate 1b 改用 `RUSTFLAGS`，为避免使共享缓存失效，在独立 `CARGO_TARGET_DIR` 中运行，用后删除。工具链见 [environment.txt](environment.txt)。

| 命令 | 结果 | 完整输出 |
|---|---|---|
| `cargo test --offline -p engine-native --no-fail-fast` | **525 通过 / 0 失败 / 1 忽略**（24 个测试目标），exit 0；含 `replay_scenarios_match_golden`（14 场景）、`scenarios_are_registered_in_coverage_manifest`、`archive_hardening_rejects_unknown_kind_and_trailing_bytes`、13 个 `*_real_host_full_lifecycle`、`swd_jiguantong_aoe_discard_or_fire` | [engine-native-tests.log](engine-native-tests.log) |
| `cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast` | **63 / 0 / 0**（16 个目标），exit 0 | [deps-tests.log](deps-tests.log) |
| `RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --lib`（CI Gate 1b） | **失败**，exit 101，lib 12 条警告被拒 | [gate1b-check.log](gate1b-check.log) |
| `python3 docs/migration/evidence/2026-09-23-extension-scope/inventory.py --workspace ..` | exit 0；与 9-23 基线**不一致**（diff 160 行） | [inventory-check.txt](inventory-check.txt)、[extension-inventory.json](extension-inventory.json) |

测试日志中 24 个目标共报告 588 条编译警告，lib 占 12 条。全工作区 `cargo test --workspace`、conformance、跨目标哈希、前端构建、联机与目标设备本次均未运行。

## 核对发现

1. **版本**：`STANDARD_RULESET_VERSION = 192`、`ARCHIVE_FORMAT_VERSION = 41`。主稿版本行仍写 v191，规则基准没有 v192 条目，本次已补登。v192 是机关桶批：`SwdJiguantongCard` tag 220，回归 `swd_jiguantong_aoe_discard_or_fire`。对照上游 `杀海拾遗/card/swd/card.js` 的 `jiguantong`，原生**未实现** contentAfter「若没有人选择受到伤害，使用者摸一张牌」，结算顺序也和上游 `reverseOrder` 相反；两项已登记为差异。本批没有独立批次证据，golden 重录依据见 v192 恢复证据。
2. **测试数**：主稿快照表写 524 通过，开头的说明写 525；本次实测 525。524 是事故前 10-10 swd 第十八批的数字。
3. **整包计数**：主稿 P2 行把“整包验收”从 1/37 一路写到 5/37，但同页“完成状态”表、P2/E04–E06 行和台账都写 0/37。那 5 个包只补齐了生命周期一项，台账逐行状态仍是“部分重构”。按 AGENTS.md 的口径，整包仍为 **0/37**，已在主稿与历史页说明。
4. **转换包**：`packages/extensions` 现有 15 个目录（台账写的是 10 个）。13 个有真实宿主全生命周期测试，本次全部通过；ow、xianjian 两个武将包还没有。张数和未映射声明见 [package-facts.txt](package-facts.txt)：swd 30 张（含机关牌 6 张）、mtg 10 张、zhenfa 3 张（deferred 5）、玩点论杀 14 张（deferred 调兵遣将×2，2 条技能声明未映射）。
5. **上游来源变化**：noname HEAD 于 10-08 由 `5c02a745` 快进到 `df2682e6`（42 个提交）。8 个扩展目录与 `card/standard.js` 没有改动，标准武将只有 `standard/skill.js` 中洛神判定回调的签名改动。Liuli HEAD 不变，但 10-08 的 `git reset` 让本地修改从 50 个文件降到 7 个，按工作区字节有 10 个来源实例内容变化。来源仍为 37 项。基线**未重录**，逐项影响待复核（主稿 P1）。
6. **事故恢复遗留**（主稿 P0）：
   - `content/host.rs` 有 14 个 `TODO(事故恢复)` 空分支。抽查刺猬、探囊，它们的效果由内核钩子承担；13 项有同名回归测试通过，元宝肉没有同名测试。
   - `src/content/` 下有两个恢复期备份（`host.rs.pre_assembly` 520,051 字节、`host.rs.recovered-v192` 559,445 字节），不参与编译。
   - `machine.rs` 1 处、`resolution.rs` 3 处留有按环境变量开关的 eprintln 调试探针。
7. **编译警告回退**：10-07 主稿记“lib 零警告已完成”，CI 随后开启 Gate 1b；本次 lib 有 12 条警告，门禁失败（主稿 P3 已重开）。
8. **文档结构**：主稿有两个表格单元格分别约 1.3 万和 1.7 万字（版本流水与 P2 批次流水），后者中间还混入了一个 `||` 字符。台账 U05 行的“待处理”和“批次”两列重复出现，前后内容互相矛盾；U03、L04 行各多一列。均已修正，见下节。

## 文档更新

- [进度主稿](../../../architecture/native-progress.md)：重写页首与核验快照（v192、525/0/1、Gate 1b 失败、来源变化、git 基线）。P2 行改为当前状态摘要，新增 P0（事故恢复遗留）和 P1（上游来源变化），P3 重开；修正已完成切片表和完成状态表中过时的数字（80 张→124 张、NGSA v2→v41、模式范围）。
- [批次流水存档](../../../architecture/native-progress-history.md)（新建）：原样收存主稿移出的版本流水 v69–v191、P2 批次流水和事故恢复逐轮记录。移出时逐字比对，内容一致；只做了换行，没有改写。
- [规则基准](../../../architecture/native-rules-baseline.md)：页首状态更新为 v192/v41；补登 v192 条目；从主稿流水原样补入此前缺失的 v85–v95 条目，现在 v61–v192 都有条目可查（v82、v84 以“批三-a/批三-b 执行”标题登记）；模式入口行更正为 7 类 `GameMode`。
- [两仓台账](../../../migration/upstream-extension-reconstruction.md)：更新“当前状态”段、仓库 HEAD 表、E02 行、G1 行与 U05 行（补 v192，合并重复列）；U03、L04 行修正多出的一列。
- [覆盖清单](../../../native-coverage-manifest.md)：版本口径更新为 v192，并写明扩展切片逐技回归在台账登记。

本地链接与锚点检查见 [doc-checks.txt](doc-checks.txt)。
