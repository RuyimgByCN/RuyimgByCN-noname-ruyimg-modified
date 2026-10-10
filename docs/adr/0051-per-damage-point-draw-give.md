# ADR 0051：逐伤害点触发与摸牌后交牌

## 决策

- `owner_is_source_or_target_other_alive_per_damage_point` 把一次伤害按点数展开为独立可选触发。
- `draw_each_by_seat` 表达两名角色按座次同步摸牌。
- `draw_then_give_one` 在摸牌事件及其触发完成后再开强制交牌选择；`drawn_only` 只允许本次摸到且仍在手牌/装备区的牌。

这些能力不识别技能名或定义 ID；三版“忘隙”和后续同类技能共用同一路径。
