# ADR 0226：定向手牌展示与可选目标牌弃置

## 决策

ABI 153 追加两个通用 effect：

- `reveal_hand_to_player(player, observer)` 仅向指定存活观察者展示完整当前手牌；
- `choose_optional_discard_target_card(player, target, scope)` 允许取消，并复用既有目标牌选择、
  合法性重验与弃牌流水线。

目标牌范围追加 `hand`，稳定标签为 4。可选性通过既有
`ChooseDiscardTargetCard` continuation 的兼容载荷尾标志表达；旧三字段载荷仍按强制选择处理。

## 原因

旧「尚义」需要双向私下看手牌，并只允许从目标手牌中选择、可不弃牌。公开展示、强制弃牌或
`hand_equip` 都会改变规则语义；技能专用 host effect 又无法服务后续大量迁移。

## 验证

Golden 覆盖定向展示、只列手牌、取消成功、弃牌提交、空手空操作和每回合一次限制。
