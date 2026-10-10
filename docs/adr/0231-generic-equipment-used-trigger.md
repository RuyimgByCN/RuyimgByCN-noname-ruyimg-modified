# ADR 0231：通用装备牌使用触发过滤器

## 决策

ABI 158 增加 `owner_alive_equip_used`：存活技能拥有者观察到任意逻辑装备牌的
`card_used` 事件时匹配。它不绑定牌的使用者、技能标记或装备区数量。

## 理由

旧 JavaScript 的全局 `useCardAfter` 子技能只表达“使用牌是装备牌”。复用通用过滤器和既有
可选触发、`draw_cards`，比复制技能专用状态或放宽已有「武库」过滤器更小且语义准确。

## 兼容性

包格式和索引结构不变；使用该过滤器的包要求 ABI 158。未增加 HOST_EFFECT、continuation、
`EffectRequest` 或序列化字段。
