# 2026-10-17 延时锦囊无懈窗口扩面（COV-NULLIFY-JUDGE-EXPAND）

**核验日期**：2026-10-17。**待做 ID**：`COV-NULLIFY-JUDGE-EXPAND`（自 `COV-PARTIAL-MECHANISMS` 拆分）。
**规则** `STANDARD_RULESET_VERSION` = **295**（本批不递增）。**存档** `ARCHIVE_FORMAT_VERSION` = **80**（本批不递增）。

## 任务

覆盖清单机制键 `nullify_judge_window` 原记「兵/闪电抵消及多牌组合待扩展」。本批补齐这三类路径的定向断言，并把机制键口径收敛到「已验证」。

## 行为（沿用既有已定案口径）

延时锦囊的窗口采用**使用时开窗**口径（ADR 0708，[语义选择定案](../../../architecture/native-rules-baseline.md)）：
出牌阶段使用延时锦囊时开一个无懈窗口，队列自来源起（含本人），无无懈者自动跳过；被抵消则**不入判定区、直接弃置**，判定阶段不再开窗。该口径对乐不思蜀、兵粮寸断、闪电一致；同目标的多张不同名延时锦囊各开自己的窗口，只抵消被点名的那一张，其余照常置入并结算。

## 测试

新增 `delayed_trick_bing_and_shandian_nullify_window`（`crates/engine-native/tests/native_kernel.rs`）：

| 场景 | 正例 | 反例/对照 |
|---|---|---|
| 兵粮寸断 | 使用时开 `nullify_window`，2 号持无懈被询问；抵消后判定区为空、1 号摸牌不被跳过 | 既有 `bing_judgement_skips_draw_phase` 断言未被抵消时摸牌被跳过 |
| 闪电 | 使用时开 `nullify_window`，2 号被询问；抵消后判定区为空 | 既有 `shandian_hits_and_transfers` 断言未被抵消时判定并转移 |
| 多牌组合 | 对 1 号先放乐被抵消（判定区空），再放兵未被抵消照常入判定区（长度 1） | 三张同开时各窗口独立 |

命令：`cargo test --offline -p engine-native --test native_kernel delayed_trick -- --test-threads=1`
结果：**2 passed / 0 failed**（[targeted.log](targeted.log)）。

## 影响

无代码语义变化（只加测试）；规则/存档/golden 不变。覆盖清单 `nullify_judge_window` 由「已验证（部分）」改为「已验证」，去「兵/闪电抵消及多牌组合待扩展」。
