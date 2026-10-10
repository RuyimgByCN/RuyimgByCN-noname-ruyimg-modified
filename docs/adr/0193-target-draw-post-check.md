# ADR 0193：目标摸牌后的手牌条件检查

## 背景

旧版 `mobile:rejieming` 每受到一点伤害可独立发动一次：选择一名角色摸两张牌，等待该次摸牌
完成后，若目标的手牌数仍小于其体力上限，技能拥有者再摸一张牌。旧 JavaScript 依赖
`await target.draw(2)` 保存事件执行栈，不能进入确定性内容包。

## 决策

复用现有 `damage owner_is_target_per_damage_point optional` 逐伤害点触发契约，新增 Rule VM
ABI 120 原语：

`choose_target_draw_then_owner_if_below_max_hp(player, count)`

原语允许拥有者选择任一存活角色。目标摸牌后，同一个 `EffectRequest` 作为延迟续执行排在
后续技能工作之前；该批摸牌产生的移动事件及嵌套触发会插入其前方，因此续执行读取的是目标的
最终手牌数和体力上限，同时不会被同一伤害的下一次技能发动越过。
Choice 恢复时重新验证观察者、技能实例、拥有者和目标；旧 JavaScript、闭包和事件对象都不
进入运行时。

## 结果

- 多点伤害为每一点保留独立的可选发动和目标选择。
- 后续判断基于摸牌及其嵌套触发结束后的权威状态。
- `rejieming` 可独立发布；ABI 121 完成 SHENHUA `quhu` 后，`re_xunyu` 已闭包发布。
