# ADR 0257: STANDARD 伊籍与赠予归因

ABI 181 增加 `cards_given` 事件、`owner_alive_gave_cards_to_other_alive` 过滤器和
`choose_target_gain_bottom_card(player)` 效果。

`CardsMoved` 继续表达区域移动和失牌语义；共享赠牌管线在其后追加
`CardsGiven { giver, recipient, cards }`，让赠予类技能无需依赖旧 JavaScript 的
`gainEvent.giver` 临时字段。机捷将牌堆底牌公开移入处理区，等移动触发结算后强制选择受赠者；
急援以 DSL 分别监听濒死与赠予事件。旧 JavaScript 仅作为语义参考，不进入包或运行时。
