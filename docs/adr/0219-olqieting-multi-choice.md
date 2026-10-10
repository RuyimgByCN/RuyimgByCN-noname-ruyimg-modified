# ADR 0219：OL窃听的回合历史与双选顺序

## 背景

ONLYOL「OL窃听」在其他角色回合结束时检查两段独立历史：是否使用牌指定过其他角色、是否造成
过伤害。前者未发生时可在移动一张兼容装备与摸一张牌之间选择一至两项；前者发生但后者未发生
时只能选择一项。两种窗口都可取消，双选固定先移动装备再摸牌。

## 决策

Rule VM ABI 146 追加：

- 过滤器 `owner_alive_other_turn_ended_card_targeted_other_no_damage_other`；
- 效果 `choose_move_target_equipment_and_or_draw(player, target)`，HOST_EFFECT 154。

权威伤害结算设置回合标志 `DAMAGE_DEALT_OTHER`，`PhaseEnded` 以带缺省值的
`current_dealt_damage_other` 携带该历史，因而自伤不会误阻断技能。
新效果复用 ABI 58 的 `ChooseMoveTargetEquipmentOrDraw` continuation，只追加双选入口与
“移动后摸牌”阶段；原「窃听」单选路径保持不变。`EffectRequest` 稳定哈希标签为 190。

## 结果

Golden 覆盖双选顺序、两条回合历史分支、伤害阻断和取消。`ol_caifuren` 通过 ONLYOL 对
YIJIANG 的现有依赖复用「献州」，无需复制技能实现。
