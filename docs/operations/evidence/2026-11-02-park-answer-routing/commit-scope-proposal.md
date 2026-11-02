# 提交范围裁定与清单（2026-11-02）

用户裁定：**只提交本次修复**（不夹带主树既有 WIP、不 push）。本档记录该范围、混合档的拆分方式与建议提交讯息；实际提交与哈希见同日 git log。修复与验收结果见 [README](README.md)。

## 一、本次修复的完整档案（我改的）

### A. 修复前为干净状态 → 仅含本修复（可整档提交）

| 状态 | 档案 | 内容 |
| --- | --- | --- |
| M | `crates/engine-native-kernel/src/resolution.rs` | `Park::awaiting_answer`；`run` 有应答时只唤醒提问者 |
| M | `crates/engine-native-kernel/src/snap/validation.rs` | 新不变量：有应答必须有开放选择 |
| M | `crates/engine-native-kernel/src/kernel/phase.rs` | 新字段初始化（1 行） |
| M | `crates/engine-native/src/engine/snapshot.rs` | 读档重建提问者标记 |
| M | `crates/engine-native/src/engine/apply_command.rs` | 新字段初始化（1 行） |
| M | `crates/engine-native/src/engine/play_card.rs` | 新字段初始化（1 行） |
| M | `crates/engine-native/src/engine.rs` | 测试内 `Park` 字面量（1 行） |
| M | `crates/engine-native/src/engine/choice_source.rs` | 测试内重复 `Park`（1 行） |
| A | `crates/engine-native/tests/nc10_answer_routing.rs` | 回归 2 项（修复前会失败） |
| A | `crates/engine-native-wasm/tests/nc10_park_answer_routing.rs` | typed/byte 双路径回归 2 项 |
| M | `docs/native-coverage-manifest.md` | 场景登记（本档先前为干净状态） |
| A | `docs/operations/evidence/2026-11-02-park-answer-routing/`（20 档） | 证据 README、17 份日志／摘要、重现脚本、本提案 |

### B. 修复前已有未提交改动 → 整档提交会混入既有 WIP

| 档案 | 既有 WIP 内容 | 我的新增 | 建议 |
| --- | --- | --- | --- |
| `crates/engine-native/tests/groups/nc_rounds.rs` | `nc10_public_other_plan` 模块注册（3 行） | `nc10_answer_routing` 模块注册（3 行） | 用 `git add -p` 只挑我的 3 行，或与 WIP 一起提交 |
| `docs/architecture/native-progress.md` | I02／NC10 的既有文档改动 | 档头新增「恢复后继续 panic 已修根因」一段 | 同上（我的段落在最前面，易分离） |

## 二、不含在我改动内的既有未提交项（本轮未验收、未提交）

- 主树原有 36 个 WIP 源码档（I02 定义语音资源、NC10 public-other-plan）＋ 既有 `native-progress.md` 文档改动。
- 既有未跟踪证据目录（`docs/operations/evidence/2026-10-30-*`、`2026-10-31-*` 等）与 `.pi-lens.json`。
- 私有 C331 工作（`$TMPDIR/noname-c331-phase7-ktqqj1fb/worktree`，331/106/WS7）**完全没动**。

## 三、建议提交讯息（若只提交 A + B 中的我的 hunk）

```
fix(native): route validated choices to the asking park

Same-depth parks could resume in vec order, so a validated answer was
delivered to a machine that never asked; the real asker then read
ctx.answer() and panicked ("answer present on resume"), trapping WASM
after save/restore (park-1, seven legacy cases).

Track the asking park at runtime (Park::awaiting_answer), rebuild it on
restore from the existing N04-e ask binding, and reject archives that
carry an answer without an open choice. Rules 330 / NGSA 105 / ABI 758 /
WS6 and golden state unchanged; source_skill gating (decision 27) kept.

Regressions: nc10_answer_routing (engine, 2), nc10_park_answer_routing
(NativeSession typed + byte archive, 2). See docs/operations/evidence/
2026-11-02-park-answer-routing/README.md.
```

## 四、提交前若要我重跑

本轮结束时源码自全量跑完后未再改动（已用 `find -newer` 核对 [02-workspace.log](02-workspace.log)）。若你希望在提交前再跑一次完整门禁，我可依序重跑：三 crate → strict Gate 1b → workspace → 前端 test/build → Worker smoke → preflight → 七浏览器案例，全部结果写回同一证据目录。
