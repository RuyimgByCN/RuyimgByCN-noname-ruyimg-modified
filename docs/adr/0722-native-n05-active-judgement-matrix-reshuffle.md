# ADR 0722：N05 主动判定恢复矩阵、逐提交守恒与洗牌组合回归

日期：2026-09-16
状态：已实施（engine-native）
关联：[0714](0714-native-guidao-tiandu-semantics-n04d.md)、[0720](0720-native-judgement-order-equip-qualification-v6.md)、[0721](0721-native-runner-judgement-load-exit0.md)、进度文档 N05

## 背景与决定

进度文档 N05 行登记：pause_roundtrip 恢复矩阵没有主动鬼道/天妒用例（策略取首选项 Pass）；
多目标无懈×真实洗牌×恢复组合缺失；文档原称的
`savage_nullify_after_deck_drain_conserves` 不在源码中。本批逐项闭环。

### 1. 主动判定进入恢复矩阵（3 组新用例）

`assert_pause_roundtrip` 矩阵新增（boundary_snapshot）：

- `pause_roundtrip_guidao_active_rewrite`（seed 263）：2 号鬼道持黑牌，
  首挂起点=改判询问，策略**真实出黑牌**替换判定牌。
- `pause_roundtrip_tiandu_confirm`：1 号天妒（限本人），首挂起点=天妒询问，
  策略 Confirm 获得（judge_held→手牌守恒路径）。
- `pause_roundtrip_rewrite_then_tiandu_combo`：1 号兼持鬼道+天妒，
  改判→天妒两点链逐点往返。

统一策略 `answer_active_judgement`：改判出第一张黑牌（无牌才 Pass）、天妒 Confirm、
其余沿用 answer_by_prompt。基准线/快照线每点哈希、事件增量、终态与守恒全部一致。

### 2. 机器持牌守恒口径修正（ADR 0720 的后续）

`machine_cards`（矩阵守恒）此前按 `!gained` 计判定牌，且无条件计 `current.trick`。
0720 重排后：效果生效时锦囊即离机（弃置/闪电转移），`current.trick` 仅簿记；
天妒放弃路径 `judge_held=false`。修正为：

- `trick` 仅在 `!finalized` 时计入机器持牌；
- `judge_card` 按 `judge_held` 计入（获得/放弃/随效果弃置后不计）。

该修正使天妒放弃（Decline）路径也可安全纳入守恒覆盖，不再有双重计数盲区。

### 3. 多目标无懈 × 真实洗牌 × 恢复组合（`savage_multi_nullify_judge_reshuffle_restore`）

seed 379 布阵：牌库经 `debug_drain_deck_to_discard`（新增上帝视角 helper）清空，
南蛮多目标（1 号持无懈，窗口/反无懈链按策略排干），随后对 1 用乐、推进判定阶段——
翻判定牌时牌库为空，触发**真实 `DeckReshuffled`**（事件断言，非构造）。

- 每次无懈应答后 `assert_multiset_conservation_named`（牌区+机器持牌多重集恰一次）。
- 快照线在首个无懈窗口挂起 save→restore，同脚本继续：终态哈希与基准线一致，
  恢复线同样恰发生一次洗牌。

## 影响

- 测试 132 项（native_kernel 74、boundary_snapshot 53、场景 2、源码 2、调试 1），净增 4。
- 规则版本不变（6）；golden 不变。
- N05 剩余：负体力连续救援逐点恢复、无懈×洗牌组合纳入 `assert_pause_roundtrip`
  通用矩阵、DeckReshuffled 后的恢复点细分；N02 机器持牌死亡清理专用回归仍待补。
