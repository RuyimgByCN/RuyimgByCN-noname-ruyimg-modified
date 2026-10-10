# 2026-10-10 T2 事故恢复残留清理

任务来源：[进度主稿 T2](../../../architecture/native-progress.md)。开工时 HEAD `4b60957b`（工作区遗留的文档清理批已先照原样入库作回退点），改动前工作区干净。

## 改动内容

1. 删除 `crates/engine-native/src/content/host.rs.pre_assembly`（520,051 字节）与 `host.rs.recovered-v192`（559,445 字节）。两者不参与编译，内容已在 git 基线（`72583b2b` 起）中。
2. 删除 4 处按环境变量开关的 `eprintln`：
   - `src/resolution.rs` `push_frame`：`PT_DBG2` Death 帧打印（原 100–102 行）；
   - `src/resolution.rs` `plan_triggers`：`PT_DBG2` 候选数打印（原 287–293 行）；
   - `src/resolution.rs` `plan_triggers`：`PT_DBG` 逐触发打印（原 324–334 行）；
   - `src/machine.rs` `expire_temporary_skills`：`XLY_DBG` 到期打印（原 518–529 行）。
3. 同步 `tests/g1_huanle.rs` `xly_state_growth_probe` 的 ignore 说明：去掉“PT_DBG=1”字样，改为“手动运行观察规模曲线”。该测试自行输出 `PROBE` 行，不依赖被删的 eprintln。

`git diff --stat`：5 个文件，+1/−23,579（两个备份占 −23,549），定点编辑无意外扩散。

## 核验

| 命令 | 结果 |
|---|---|
| `rg 'env::var' crates/engine-native/src/` | 0 命中（exit 1） |
| `rg -l 'PT_DBG\|XLY_DBG' crates/engine-native/src/` | 0 命中（exit 1） |
| `cargo test --offline -p engine-native --no-fail-fast` | **525 通过 / 0 失败 / 1 ignore**（exit 0），与改前 2026-10-10 进度核对基线一致——[完整日志](engine-native-tests.log) |

执行环境：本机，git `4b60957b` + 本批改动；规则 v192、NGSA v41 不变（本批不涉及规则与存档编码）。

## 边界

- 只删调试输出与备份，不改任何行为；golden 未动，`replay_scenarios_match_golden` 在全量中通过。
- 同目录后续追加 T3（lib 警告清零）的核验记录。

---

# 同日 T3：lib 12 条警告清零与 dbg4.rs 处理

任务来源：[进度主稿 T3](../../../architecture/native-progress.md)；开工 HEAD `aff048b7`。逐条位置以当次 `cargo check --message-format=short` 实测为准（T1 之后 host.rs 行号较[待办整理核对 §2](../2026-10-10-todo-handoff/README.md)整体 +32，逐条内容一致）。

## 改动内容（12 条警告逐条）

| 位置（当次实测） | 处理 |
|---|---|
| host.rs:5121 未用 `oc`（偷梁换柱同色摸 1） | `if let Some(oc) = own_card { if own_color == Some(t_color) {…} }` 合并为 `if own_card.is_some() && own_color == Some(t_color)`，语义不变 |
| host.rs:9773 未用 `eff`（乐动 Act 段） | 删除该绑定；TargetChoice 段重新读取的路径不动 |
| host.rs:11339 未用 `picked`（羊皮卷发现） | 外层改 `if !cands.is_empty()`，内层 RNG 选出 `picked` 不变，`gen_range` 调用次数不变 |
| host.rs:12941 未用 `d`（空 if-let 块） | 删除整块，"target 未用——伤害牌从 cards[0] 取"注释上移保留 |
| host.rs:14200 `self.card = self.card` | 删除该行 |
| host.rs:14345 未用 `def`（sync_land_skill 首行） | 删除该行 |
| host.rs:14385 未用导入 `HostBehavior as B` | 删除该行 |
| engine.rs:230 未用 `chongzheng`（debug_add_fellow） | 改 `matches!(&…mode, GameMode::Boss { .. })` 判空返回，语义不变 |
| kernel.rs:837 未用 `boss` | 绑定改 `_` |
| kernel.rs:5604 多余 `mut`（Kof games） | 去掉 `mut` |
| presentation.rs:185 未用 `count` | 改 `count: _` |
| snap.rs:1503 未用 `liuli_card` | 模式里已有 `..`，删除该字段绑定 |

另删除 `tests/dbg4.rs`（9-16 的 `dbg_nullify` 只打印无断言）：南蛮+无懈的断言覆盖已存在于 `native_kernel.rs::savage_nullify_after_deck_drain_conserves` 与 `boundary_snapshot.rs::build_savage_nullify_reshuffle`/`answer_plays_nullify` 暂停往返，无覆盖损失。

## 核验

| 命令 | 结果 |
|---|---|
| `CARGO_TARGET_DIR=/tmp/t3-gate1b-target RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --lib` | **exit 0**（独立 target 目录，全新编译含依赖树；重跑缓存命中仍 0；临时目录已删）——[日志](gate1b-check.log) |
| `cargo test --offline -p engine-native --no-fail-fast` | **524 通过 / 0 失败 / 1 ignore**（exit 0）；较 T2 基线少 1 = dbg4 删除的 `dbg_nullify`；golden 不变——[完整日志](engine-native-tests-t3.log) |

执行环境：本机，git `aff048b7` + 本批改动；规则 v192、NGSA v41 不变。

