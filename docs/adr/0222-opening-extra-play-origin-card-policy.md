# ADR 0222：额外出牌阶段来源与阶段实体牌策略

## 背景

ONLYOL `oldangxian` 只应在自身创建的额外出牌阶段询问获得【杀】；接受后，仅获得的那张实体
【杀】无距离限制，且本阶段未造成伤害时自伤。旧 JavaScript 依赖 `_extraPhaseReason`、
`gaintag` 和阶段事件历史，不能直接进入确定性运行时。

## 决策

ABI 149 让既有回合开端额外出牌阶段队列保存来源技能实例，并在当前额外阶段公开该来源。
新增两个通用效果：

- `choose_gain_sha_for_opening_extra_play(player)`（HOST_EFFECT 157）只在来源等于当前技能时
  询问；按牌堆、弃牌堆顺序取第一张实体【杀】，并保存该额外阶段开始时的累计伤害基线。
- `finish_opening_extra_play_sha_gain(player)`（HOST_EFFECT 158）清除该实体牌策略；累计伤害
  未增加时通过普通伤害管线对拥有者造成 1 点伤害。

`card_distance_unlimited_for_skill_card_policy` 复用技能实例的精确实体牌策略，不扩大为全部
【杀】。Choice 恢复时重验来源技能、当前额外阶段、实体牌及牌区。

## 结果

ONLYOL `ol_liaohua` 由完整 `oldangxian` 与既有 `olfuli` 闭合。两个新 `EffectRequest`
使用稳定哈希标签 193、194，选择续体使用稳定标签 106；旧 JavaScript 只保留为语义证据。
