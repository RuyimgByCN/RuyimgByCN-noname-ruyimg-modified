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
