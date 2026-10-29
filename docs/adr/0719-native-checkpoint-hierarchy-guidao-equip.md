# ADR 0719：检查点再嵌套拒绝与装备改判切片（验收未完成）

- 状态：第二十一批切片已落地；2026-09-20 独立实跑 **118 passed / 0 failed**，装备仍有反例。
- 关联：[engine](../../crates/engine-native/src/engine.rs)、[snap](../../crates/engine-native-kernel/src/snap.rs)、
  [kernel](../../crates/engine-native-kernel/src/kernel.rs)、[当前进度](../architecture/native-progress.md)。
- 前序：[0715](0715-native-choice-binding-validation.md)、[0716](0716-native-judgement-events-regressions.md)、
  [0717](0717-native-runner-baseline.md)、[0718](0718-native-neijiang-pure-command.md)。

## 已落地

1. restore 拒绝 segment_checkpoint 自身继续嵌套检查点，错误 checkpoint_nested；
   新增 restore_rejects_nested_checkpoint。已有内层版本、core 和 resolution 校验保持。
   这不等于当前段/检查点的玩家数、模式、revision/dedup 和执行状态关系全部验证。
2. 鬼道选项构造追加黑色装备，take_owned_card 可从手牌或装备区移除；旧判定牌仍由改判者获得。
3. 新增名为 guidao_rewrite_with_black_equip 的测试，但实际只放黑色手牌，未装备任何牌，
   并选择 options[0]=Pass；不能记为装备改判覆盖。

## N08-c 独立反例

seed 217，正常推进到 2 号出牌并以 PlayCard 装备 weapon2；2 号鬼道、3 号判定乐、红桃压顶：

- 无黑色手牌、只有黑色装备：has_black 仍只检查手牌，未询问鬼道。
- 同时有黑色手牌/装备：实际询问包含装备，但 active_ask 仍只推导手牌，
  save 后 restore 拒绝自身生成的快照，报 OpenChoiceMismatch { observer: 2 }。
- 原实例直接选择装备牌可以完成改判，装备移出且终态逐牌守恒；缺口在资格过滤与恢复选项未同步。

后续统一资格/选项来源，覆盖“仅装备”“手牌加装备”两组，真实选择装备牌，
同时断言原实例/恢复件的牌区、射程、旧牌归属、事件与守恒。

## 测试数与覆盖纠正

- 当前分解：源码单元 2、native_kernel **68**、boundary_snapshot 45、场景 2、调试 1，总计 118。
  原稿 70 项机制测试的分解不符；本批新增 2 项在 boundary_snapshot。
- reshuffle_conserves_every_card_exactly_once 存在，在完整段后检查重复/丢失，
  未逐次 Answer 检查、也未断言必有洗牌事件。
- 原稿 savage_nullify_after_deck_drain_conserves 在当前源码/测试列表不存在，不能计为交付。
- 事件定义与相关断言已增加，但重复/顺序问题见 [0716 复核](0716-native-judgement-events-regressions.md)。
- 规则版本仍 5、14 golden 未变并通过。新增装备应答已改变规则语义，须在修复后递增版本并登记；
  不因已有 golden 不变而认定无需版本变更。

## 后续

优先修复 N08-c 合法装备恢复、N08-d 事件全序，然后补身份人数范围、检查点关联、
主动改判/成功救活/真实洗牌恢复和逐提交守恒。N09 判定 runner 当前 panic，具体数据见
[phase19](../operations/phase19-native-baseline.md)；首组技能与阶段 2 尚未整体验收。
