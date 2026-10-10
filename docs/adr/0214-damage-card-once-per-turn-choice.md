# ADR 0214：伤害牌每回合一次分支

## 背景

四象「铿锵」要求技能拥有者使用伤害牌时每回合限一次选择摸两张牌，或失去 1 点体力并令当前
牌伤害增加 1。旧实现依赖 `get.is.damageCard`、`usable`、`chooseControl` 和直接改写
`trigger.baseDamage`，不能进入确定性运行时。

## 决策

Rule VM ABI 141 只追加过滤器 `owner_source_damage_card_once_per_turn`：

- 事件必须是技能拥有者发起的 `card_used`；
- 有效牌种必须是逻辑【杀】或伤害类锦囊；
- 接受发动后复用技能实例的 `turn_fire_count`，上限为 1。

DSL 分支直接复用既有 `choose(1, 2)`、`draw_cards`、`lose_hp` 和
`increase_current_card_base_damage`。本版本不增加 HOST_EFFECT、continuation 或
`EffectRequest` 变体。

## 结果

旧 JavaScript 只保留为语义证据。Golden 覆盖非伤害牌不触发且不消耗次数、摸两张分支、
失去体力并提高本次伤害、同回合不再询问，以及正式回合推进后的次数重置。
