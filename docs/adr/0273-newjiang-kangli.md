# ADR 0273：NEWJIANG「伉厉」的连续在手牌追踪

ABI 197 增加 `draw_remembered_hand_cards(player, count)` 与
`discard_remembered_hand_cards(player)`。前者在 originating skill 实例中记录本次摸到且仍在
拥有者手牌区的实体牌；牌一旦离开该手牌区即从记录移除，即使之后返回也不会重新加入。

后者先清空记录，再弃置其中仍在手牌区且当前可弃置的牌。实现复用现有摸牌、移动和弃牌原语，
不增加全局卡牌标签、专用续体或旧 JavaScript 运行时。
