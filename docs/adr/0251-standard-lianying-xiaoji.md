# ADR 0251：标准连营与枭姬失牌触发

## 决策

「连营」在拥有者失去最后的手牌后提供可选摸一张牌；「枭姬」在拥有者失去装备牌时，按本次
移动的装备牌张数分别提供可选摸两张牌。

## 实现

ABI 175 增加 `owner_from_equip_per_card` 过滤器。它只匹配从技能拥有者装备区离开的
`CardsMoved`，并按事件卡牌数复制固定触发任务。连营继续复用 `owner_from_hand_empty`；
两个技能都使用已有的 `optional` 触发确认和 `draw_cards`，不增加 HOST_EFFECT、
`EffectRequest` 或 continuation。

旧 JavaScript 的音频、AI 估值和自动发动偏好不进入运行时。
