# 2026-09-23 扩展状态复核

本次对 ADR 0776 后的工作区执行 `cargo test --offline -p engine-native`，见 [完整日志](native-tests.log)；
[源码摘要](source-hashes.json) 固定 `content/extension.rs`、`content/mod.rs` 和 `tests/native_kernel.rs` 的复核版本。
本次只整理文档及证据，源码改动来自并行实现任务。

| 检查 | 结果 | 能证明的范围 |
|---|---|---|
| 全量 engine-native | **235 passed / 0 failed**：src 3、native_kernel 157、boundary_snapshot 59、auth_archive 6、dbg4 1、replay_scenarios 2、view_isolation 7；doc-tests 0 | 当前工程测试通过；原第 14 轮 [日志](../2026-09-23-native-review-14/native-tests.log) 实为 232 项 |
| 扩展相关 3 项 | `content::extension::tests::extension_registry_lifecycle`、`e01_extension_registry_lifecycle`、`e03_extension_cards_in_catalog_and_game` 通过 | 内存注册表启停/卸载和临时卡牌列表追加通过 |
| 正式对局接入 | `merge_extension_cards` 只在新模块和测试调用；`Engine::new` 与 `Engine::restore` 仍使用标准目录 | 不能把卡牌字段断言算作新卡在真实对局发牌、结算或恢复 |
| 来源扩展 | 两仓 [37 来源实例](../../../migration/upstream-extension-reconstruction.md) 尚未作为原生包运行 | 整包原生验收 **0/37**；逐包状态见重构计划 |

复查时，扩展包名和卡牌名仍是静态字符串，`ExtensionRegistry` 不提供磁盘持久化，示例武将尚未加入实际对局目录。
ADR 0776 中“E01–E03 全部完成”“真实对局中生效”的描述已按上述源码和测试强度更正。
