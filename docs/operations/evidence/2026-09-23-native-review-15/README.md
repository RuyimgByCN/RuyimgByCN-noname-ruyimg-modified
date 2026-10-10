# 2026-09-23 原生内核第 15 轮独立核验

范围：规则版本 28 的当前工作区；本轮只修改文档与独立探针，没有修改引擎、工程测试或上游扩展。实现版本见 [源码摘要](source-hashes.json)，当前扩展来源复盘见 [清单](current-extension-inventory.json)。[进度主稿](../../../architecture/native-progress.md) 维护完成/待办状态。

| 证据 | 结果与边界 |
|---|---|
| [全量测试](native-tests.log) | `cargo test --offline -p engine-native`：**238 passed / 0 failed**，分组 3+6+59+1+160+2+7，doc-tests 0；其中扩展相关模块 1 项、集成 5 项。旧第 14 轮 232 项和 ADR 0776 早期 235 项属不同源码快照。 |
| [文档守护](doc-guard.log) | 更新覆盖清单后重跑 `replay_scenarios`，场景登记和 14 golden 的 2 项测试通过。 |
| [独立探针](probes.rs) / [输出](probes.log) | 高水位 1,000,000 后策略和规划提交已接受；决斗、AOE、流离字节恢复通过。无懈选择仍被拒；尾端缺牌、AI 采样重复物理牌、漏选合法闪、模拟高水位退化、护驾/离间恢复与护驾队列问题可复现。 |
| [来源清单](current-extension-inventory.json) | 与 [原清单](../../../migration/evidence/2026-09-23-extension-scope/inventory.json) 逐字节 `diff -q` 相同：8 + 29 来源实例、34 个不同目录名，仍无整包原生验收记录。 |
| 扩展接线检查 | `merge_extension_cards`、`ExtensionRegistry`、`check_general_conflicts` 仅在扩展模块/测试使用；`Engine::new`/`restore` 仍建标准目录。`e03/e05` 只查临时 `Vec`，`e06` 只查元数据。探针确认可启用未安装名称，同一扩展的重复卡名也放行。 |

`e04_extension_conflict_detection` 断言基础卡名冲突引发 panic；这不是可恢复的安装错误。测试名 `e04/e05/e06` 不能当作 [扩展路线图](../../../architecture/native-extensions.md) 中 E04 模式/AI/UI、E05 联机/存档、E06 全量迁移的完成凭据。示例「赤闪电」仍使用标准 `ShanDian` 类型，代码注释中的火焰 2 点规则没有执行实现。

## 复跑

在 `noname-next` 根目录运行：

```bash
cargo test --offline -p engine-native
python3 docs/migration/evidence/2026-09-23-extension-scope/inventory.py --workspace .. > /tmp/current-extension-inventory.json
diff -q docs/migration/evidence/2026-09-23-extension-scope/inventory.json /tmp/current-extension-inventory.json
```

独立探针用本轮 `cargo test` 编译出的最新 `libengine_native-*.rlib` 链接 `probes.rs`，执行结果见 `probes.log`。该探针是复核证据，不计入 238 项工程测试；其失败反例应在实现修复后改为正确行为断言。第 14 轮性能/KOF/跨进程记录仍见 [历史证据](../2026-09-23-native-review-14/README.md)，本轮没有重跑这些目标，也没有宣称正式客户端、跨平台或 37 扩展已验收。
