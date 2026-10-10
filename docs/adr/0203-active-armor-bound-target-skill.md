# ADR 0203：防具成本与临时攻击范围使用通用目标绑定技能

## 背景

旧 SIXIANG「奋迅」在出牌阶段限一次：弃置一张防具牌，选择一名其他角色，并在本回合把该角色
视为技能拥有者攻击范围内。旧实现通过 `markAuto` 保存目标、`addTempSkill` 管理生命周期，再由
JavaScript `mod.inRange` 回调读取 storage；这些运行时对象不能进入确定性内容包。

## 决策

Rule VM ABI 130 为主动技 `select_he` 增加 `card_armor`，只提供拥有者手牌区或装备区内当前
可弃置的物理防具牌。DSL 同时开放既有
`add_bound_skill(player, skill_def, target)`：引擎验证玩家与绑定目标存活，为玩家添加或更新
目标绑定技能实例，并使用既有 `BND1` 状态格式保存稳定目标 ID。

「奋迅」的父技能只组合 `discard_selected(self)` 与 `add_bound_skill`。临时子技能复用
`card_distance_unlimited_for_skill_bound_target`，仅对绑定目标取消距离限制；普通
`turn_ended owner_is_target` 触发器在拥有者回合结束时移除该技能实例。

## 结果

- 旧 JavaScript 只保留为语义证据，不进入 npack 或运行时。
- 同类“支付一张特定装备并临时绑定目标”的技能可以复用选择约束、绑定状态和生命周期触发器。
- ABI 130 只追加 HOST_EFFECT 143；沿用既有 `EffectRequest::AddBoundSkill`、哈希标签 82，
  不新增 continuation 或角色专用 Rust 分支。
