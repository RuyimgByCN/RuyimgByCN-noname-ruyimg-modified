# ADR 0254: STANDARD「骁果」两阶段结算

## 决策

ABI 178 增加过滤器 `owner_alive_other_phase_end_has_discardable_basic_hand` 与效果
`choose_discard_basic_then_target_discard_equip_or_damage(player, target)`（HOST_EFFECT 177）。

其他角色结束阶段开始时，拥有者可弃置一张按其视角有效类别为基本牌的手牌。费用及其嵌套触发
结算后，目标可弃置一张按其视角有效类别为装备牌的手牌或装备区牌；目标取消或没有合法装备牌
时，受到拥有者造成的一点普通伤害。

两段选择使用 continuation 117；新增 `EffectRequest` 稳定哈希标签 218。两次屏障均重新查询
权威牌区与有效类别。旧 JavaScript 只作为语义参考，其 AI、音频、提示和动画不进入 npack 或
运行时。
