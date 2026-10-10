# ADR 0711：判定队首消费与装备惩罚测试（第十四批历史记录）

- 状态：第十四批历史记录；当时分组排序与版本递增尚未完成。
- 后续：2026-09-17 第十五批已补稳定分组、版本 4 和双放置序验收，见 [ADR 0713](0713-native-judge-grouping-version4.md)。
  **下文“当前/本轮”均指第十四批核验时点**；原反向布阵现在不再触发死亡清剩余 queue，最新状态见 [重写进度](../architecture/native-progress.md)。
- 关联：[原生 kernel](../../crates/engine-native/src/kernel.rs)、[机制测试](../../crates/engine-native/tests/native_kernel.rs)。
- 对照：[旧线 judge.rs](../../crates/engine-core/src/command/judge.rs) 的 collect_delayed_queue / resolve_judge_queue；
  前序 [ADR 0708](0708-native-rule-source-alignment.md)，当前验收见 [重写进度](../architecture/native-progress.md)。

## 目标与当前实现

旧线 collect_delayed_queue 按原放置序收集非闪电，再追加闪电，resolve_judge_queue 从 queue[0] 消费。
本批拟采用同样的“非闪电在前、闪电在后”规则，但当前仅将原生消费方式由尾部 pop 改为队首 remove(0)。
判定阶段直接取出玩家 judge 向量，放置机器也直接 push，**没有分组排序步骤**。
因此实际行为是按原始放置顺序先入先判，不能标记为已对齐旧线。

## 独立排序反例（N06-a）

seed 179、4 人、1 号 3 血、清全部手牌，判定牌依次黑桃 2–9、红桃；仅改变判定区放置顺序：

| 放置顺序 | 当前结果 | 声明的分组规则应得结果 |
|---|---|---|
| 乐→闪电 | 乐先用黑桃，闪电用红桃，1 号存活，2 次 Judged | 相同 |
| 闪电→乐 | 闪电先用黑桃致死，仅 1 次 Judged，待判乐被死亡清队列分支弃置 | 应仍先乐后闪电，1 号存活 |

两种布阵已独立实跑。`judge_order_delayed_first_then_lightning_hit_kills` 只放乐→闪电，
能区分原栈序与新 FIFO，不能区分“原样 FIFO”和“分组后 FIFO”。
修复须补闪电在前/中间、非闪电保持相对顺序、排序后保存恢复的断言。

## N02 / N05 覆盖更正

- “闪电必在队尾，所以死亡时不可能还有待判牌”不符合当前源码与探针；闪电→乐仍可触发死亡清 queue，且清理保持单份牌。
- 原 `judge_phase_death_discards_remaining_tricks` 已被改写为上面的乐→闪电排序测试。
  新布阵在闪电前已处理乐，不能代替原清 queue 分支；专用工程回归待恢复。
- `pause_roundtrip_judge_lightning_dying_save` 布阵未变：1 血、3 雷伤、仅 1 桃。
  本轮按同布阵实跑，出桃后玩家仍死亡；不是成功救回用例。
- 足量桃救活、完成当前判定段和恢复一致性仍属 N05；分组实现后按最终规则调整后继事件布阵，
  不能以拟实现的排序取消救活验收或宣称已覆盖。

## 装备惩罚用例：已落地

`identity_lord_penalty_discards_equips` 先以出牌命令装备 weapon2，再调试击杀忠臣。
断言主公手牌/装备清空、武器在弃牌堆恰一份、牌区守恒且尚未终局。
它关闭“装备惩罚测试未放装备”的缺口；不证明奖惩与终局先后规则已完成核对。

## 版本与测试记录

- `cargo test --offline -p engine-native`：**106 passed / 0 failed**。
  原 N02 测试改写为排序测试，另新增 1 项装备惩罚；105→106，不是净增两项。
- 14 个 golden 保持原值并通过；它们不能证明混合判定语义未变。
- 当前 STANDARD_RULESET_VERSION 仍为 3。尾部消费改队首已改变规则语义，
  尚未遵守 [规则基准](../architecture/native-rules-baseline.md) 的递增约定；须补版本、旧版拒绝和 golden 更新记录。
- 排序、单桃救援和死亡清队列的独立探针不额外计入 106 项；已有编译警告仍在。

## 后续

1. N06-a 补稳定分组并覆盖两种放置序，连同版本递增、旧快照拒绝和 golden 原因登记验收。
2. N04 精确身份配比/结构检查；N05 成功救活、N02 专用回归、洗牌/无懈恢复组合。
3. N06 奖惩先后、距离/传导、逐命令守恒、内奸终局与重放恢复；再推进 N08 官方技能和 N09 性能基线。
