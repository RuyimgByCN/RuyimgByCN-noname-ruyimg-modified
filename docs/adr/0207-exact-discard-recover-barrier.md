# ADR 0207：精确弃牌后的回复屏障

## 背景

旧 SIXIANG「聚进」在主公受到群势力角色造成的伤害后，只有拥有者至少有两张手牌或装备牌时
才可发动。确认后拥有者必须选择并弃置恰好两张牌，弃牌完成后若仍受伤则回复 1 点体力。旧实现
依赖 `chooseToDiscard(..., 2, "he")`、`await discard` 和客户端 AI 回调；直接把弃牌与回复放进
同一效果队列会让回复早于卡牌移动触发，破坏旧异步顺序。

## 决策

Rule VM ABI 134 增加：

- `owner_is_target_source_group_qun_discardable_hand_equip_gte_two`，在伤害触发时权威复核拥有者、
  来源势力和当前可弃牌数量；
- `choose_discard_hand_equip_then_recover(player, count, amount)`，要求从当前手牌区或装备区精确
  选择 `count` 张合法牌；
- 弃牌按一个确定性移动批次提交，回复通过 movement deferred-tail 等待移动及嵌套触发完成；
- 候选恰好等于所需数量时直接提交，候选不足时不部分弃置、不回复。

ABI 134 追加 HOST_EFFECT 146 和 `EffectRequest` 哈希标签 182；精确选牌恢复复用既有
continuation 88，不引入技能专用状态机。

## 结果

- 旧 JavaScript 只保留为语义证据，不进入 DSL 包、npack、扩展目录或运行时。
- 选择恢复会重新验证技能、角色、候选卡实体及其当前区域。
- 弃牌触发始终先于回复；回复阶段读取当前体力，因此已满体力时自然为空操作。
