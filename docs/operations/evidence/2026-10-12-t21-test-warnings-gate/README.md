# T21 测试与示例警告清理（D3 落地，2026-10-12）

> 本 README 为 2026-10-14 补记（孟婆批回填断链时发现当日只落了日志未写 README）；任务叙述见[进度主稿 T21 行](../../../architecture/native-progress.md)。

## 内容

- 清理 engine-native 测试与示例目标的 **623 条编译警告**（改前清单：[warnings-before.txt](warnings-before.txt)，git `45a3e72a`，`cargo check --all-targets --message-format=json` 去重）。
- 处理：409 条 `drop(引用)` 空操作行删除；未用变量/多余 `mut`/未用导入经 `cargo fix`；其余 37 条逐条人工（`a0_tag_registry::host_behavior_tags_unique` 原无断言改为对 `HostBehavior::ALL` 断言 tag 唯一/`from_tag` 往返/可达=ALL[240]；删除迁移遗留 7 个 serde 结构体与 4 个未用辅助等）。只改 `crates/engine-native` 测试、示例与 `extension.rs` 测试模块；规则/存档版本与 golden 不变。
- 门禁扩展：`.github/workflows/ci.yml` 与本页口径由 `--lib` 改 **`--all-targets`**。

## 实测记录

| 项 | 结果 | 日志 |
|---|---|---|
| Gate 1b（新口径） | `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets`（独立 target 目录）**exit 0** | [gate1b-all-targets.log](gate1b-all-targets.log) |
| 原生全量 | **616/0/1**，改前改后测试名集合逐条一致 | [engine-native-tests.log](engine-native-tests.log)、[test-names-diff.txt](test-names-diff.txt)（617 条 test 行：616 ok + 1 ignored） |
