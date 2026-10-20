# 2026-10-10 T14：覆盖清单 kof_session 行修正与技能名统一（墨守/霰弹）

任务来源：[进度主稿 T14](../../../architecture/native-progress.md)；开工 HEAD `a0b1834c`。

## ① kof_session 行

`docs/native-coverage-manifest.md` 原行写"近似……会话矛盾仍接受、拒绝循环超时及真正恢复续跑待修"，与同页模式行及[规则基准](../../../architecture/native-rules-baseline.md)（`kof_session_consistency_and_midrestore` 已固化）矛盾。

对照测试实际覆盖（`native_kernel.rs::kof_session_consistency_and_midrestore`，2026-10-07）逐条改写：

| 测试实际覆盖 | 新行表述 |
|---|---|
| 会话行（双方将池/比分/局数）文本往返一致；结构非法/将池越界 restore 返回 None | 会话行往返一致（结构非法/将池越界拒绝） |
| 每局后存档恢复续跑与不恢复路径终局一致 | 恢复续跑终局一致 |
| 单局中途快照恢复，30/60 命令检查点哈希与不中断一致 | 中途恢复哈希轨迹一致 |
| 终局败者各区清空 | 死亡清理 |

仍真实保留的边界：语义矛盾行（字段间不一致，如比分与池淘汰不符）`restore` 不校验、仍接受；KOF 多局连续会话循环与示例策略超时判负在宿主/示例层（非内核）。新行状态由"近似"改为"已验证（会话行与单局恢复路径）"，测试列表补 `kof_session_consistency_and_midrestore`、`kof_engine_mode_session_advance`。

## ② 技能名统一

上游对照：`玩点论杀/character/translate.js:4` moshou="墨守"；`杀海拾遗/character/ow/translate.js:124` xiandan="霰弹"。规则基准与进度流水中的错名已由同日[文档清理批](../2026-10-10-doc-cleanup/README.md)改正；本批改其余位置：

- "莫守"→"墨守"（12 处）：`engine.rs`、`catalog.rs`、`content/standard.rs`、`content/host.rs`×2、`tests/g1_wudian.rs`×10、`docs/architecture/legacy-reference.md`×1（实改 16 处含 tests）。
- "闪弹"→"霰弹"（9 处）：`kernel.rs`×6、`snap.rs`、`tests/g1_wudian.rs`×2。

全部为注释/文档文本，无行为改动；`docs/operations/evidence/**` 带日期证据文档保留原文（历史语境）。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --test replay_scenarios` | 2/2（覆盖清单联动守护通过） |
| `cargo test --offline -p engine-native --test g1_wudian` | 115/0（注释改动后定向复跑） |
| `rg '莫守|闪弹' crates/engine-native docs/architecture` | 仅剩进度流水/历史文档的历史记录与进度主稿本行描述 |

执行环境：本机，git `a0b1834c` + 本批改动。
