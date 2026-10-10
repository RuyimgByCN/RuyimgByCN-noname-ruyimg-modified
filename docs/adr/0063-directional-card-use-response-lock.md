# ADR 0063：按方向距离限制整次用牌响应

- 状态：已接受
- Engine semantics：47

## 背景

旧版 `fuqi` 与 `refuqi` 都在 `useCard` 后把距离 1 的其他角色加入该牌的
`directHit` 集合，但两者使用的距离方向不同：前者计算响应者到来源，后者计算来源到
响应者。限制覆盖【杀】、AOE、【决斗】与【无懈可击】响应链，不能用“技能拥有者自己
不能响应”的全局修正近似。

## 决策

新增两个来源侧、方向明确的通用 Modifier 通道：

- `card_use_unrespondable_within_responder_to_owner_distance`
- `card_use_unrespondable_within_owner_to_responder_distance`

修正值是最终距离阈值，查询只读取当前被响应牌的来源所拥有的修正，且永不限制来源
本人。统一查询接入【杀】单/多目标、南蛮/万箭、【决斗】首次响应及无懈链。

## 结果

内容包只声明方向和阈值，不保存旧 JS，也不按技能 ID 分支。新增其他同类技能时可复用
相同通道；方向性距离修正（例如防御马）仍通过既有最终距离查询生效。
