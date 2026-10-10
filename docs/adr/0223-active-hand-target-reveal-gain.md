# ADR 0223：主动技手牌目标与公开后分类获得

## 背景

OFFLINE `zj_cuijian` 要求选择一名有手牌的其他角色，公开其全部手牌，获得其中所有【闪】和
防具；没有获得牌时摸两张。旧 JavaScript 的筛牌与异步移动流程不能进入确定性运行时。

## 决策

ABI 150 为主动技目标增加 `hand_nonempty` 约束，并增加
`reveal_target_hand_gain_shan_armor_or_draw_two(player, target)`（HOST_EFFECT 159）。
目标选择和效果执行时都重验角色存活、互异及目标手牌非空。效果先公开目标当前完整手牌，再
按有效牌名匹配【闪】、按实体类别匹配防具；匹配牌通过既有 `GiveCards` 管线移动，没有匹配
牌则通过既有技能摸牌管线摸两张。

## 结果

OFFLINE `zj_zhanghu` 以完整 `zj_cuijian` 闭合。新增 `EffectRequest` 使用稳定哈希标签 195，
不增加 continuation；旧 JavaScript 只保留为语义证据。
