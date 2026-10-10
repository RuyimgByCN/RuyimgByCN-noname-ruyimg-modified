# ADR 0228：攻击范围目标弃牌与同类别延迟回复

## 决策

ABI 155 为主动技目标追加 `in_attack_range` 和 `has_discardable_hand_equip`，在扣除代价前
权威验证目标处于技能拥有者攻击范围内，且至少有一张拥有者当前可弃的手牌或装备。

追加通用 effect
`choose_discard_target_cards_recover_same_category(player, target, count, amount)`。它强制选择
并弃置 `min(count, 当前候选数)` 张目标手牌/装备；只有恰好弃置 `count` 张且类别相同，才将
回复放入 deferred tail，等待全部失牌触发和嵌套选择完成。新增 continuation 109、
HOST_EFFECT 166 与稳定哈希标签 202。

## 原因

旧「血拼」先失去体力，濒死链结束后再重验目标牌；目标只有一张可弃牌时仍弃一张但不回复。
手牌的 `type2` 按目标视角计算，装备按实体类别计算。已有自弃后回复效果不能表达目标牌选择
及该类别判定，把规则做成武将专用宿主分支也无法复用。

## 验证

Golden 覆盖攻击范围与可弃牌激活约束、失去体力后的一张/两张选择、同类别延迟回复、不同类别
不回复，以及非法或重复选牌的原子拒绝。
