# ADR 0195：死亡后仍可完成的封顶摸牌与弃牌

## 背景

旧版 `refresh:oljieming` 在每受到一点伤害后，或技能拥有者死亡时，可选择一名角色。目标先摸
X 张牌；该次摸牌及其嵌套事件结束后，若目标手牌数大于 X，目标自行选择弃牌至 X 张，其中
X 为目标当时的体力上限且至多为 5。旧 JavaScript 依赖 `forceDie`、异步 `chooseTarget`、
`draw` 和 `chooseToDiscard` 事件栈，不能进入确定性内容包。

## 决策

新增 Rule VM ABI 122 通用效果：

`choose_target_draw_max_hp_cap_then_discard_excess(player, cap)`

效果允许仍拥有有效技能实例的死亡玩家完成目标选择，但候选目标必须存活且体力上限为正。
选定目标后按 `min(target.max_hp, cap)` 摸牌；待摸牌移动及嵌套触发结束，再从权威状态重新读取
目标体力上限。仅当手牌超过新的上限时，复用 `AdjustHandTo` 的强制精确选牌弃置屏障，不在手牌
不足时补牌。

目标选择恢复时重新验证观察者、技能实例、目标存活和正体力上限；后置阶段若目标已经死亡则
安全结束。旧 JavaScript、闭包、`forceDie` 标记和事件对象都不进入运行时。

## 结果

- 多点伤害通过既有 `owner_is_target_per_damage_point` 为每一点独立询问。
- `death owner_is_target optional` 可由已死亡拥有者确认并完成一次结算。
- `refresh:oljieming` 与依赖 SHENHUA `quhu` 的 `refresh:ol_xunyu` 完整发布。
- Rule IR 提升至 84，Engine Semantics 提升至 167，追加 HOST_EFFECT 137、
  continuation 94 和 `EffectRequest` 哈希标签 174。
