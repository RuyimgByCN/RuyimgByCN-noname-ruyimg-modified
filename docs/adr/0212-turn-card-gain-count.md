# ADR 0212：按需记录每玩家回合获得牌数

## 背景

旧版 SIXIANG「淑慎」在每回合结束时检查拥有者本回合是否获得至少三张牌，满足时可令一名
其他角色回复 1 点体力。新引擎已有选择其他角色并回复的效果，但没有通用的回合获得牌计数。

## 决策

Rule VM ABI 139 增加过滤器 `owner_alive_gained_three_this_turn_other_alive`。内容注册表只有在
加载该过滤器时才开启获得牌计数；所有实体牌进入玩家手牌都经过 `move_one_card`，因此在该
共享根路径按玩家累加，不在摸牌、赠牌和获得牌效果中分别埋点。计数使用 `TurnState` 的稳定
玩家键 1100 段，并随 `clear_per_player_card_counts` 在回合边界清空。

`stdshushenx` 复用既有
`choose_other_draw_or_recover(self, 0, 1, 1)`，不新增 HOST_EFFECT 或回复 continuation。

## 结果

- 摸牌、赠牌和其他进入手牌的移动共享同一统计口径。
- 未加载相关技能时不产生额外状态或哈希变化。
- 标甘夫人仍需等待「智诫」迁移后才作为完整武将发布。
