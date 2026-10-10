# ADR 0277：SIXIANG 尚俭与回合失牌溯源

ABI 201 增加按需启用的通用回合失牌溯源、过滤器
`owner_is_target_lost_cards_at_most_hp_gainable` 和效果
`choose_gain_own_lost_discard_card_this_turn(player)`。权威移动路径记录玩家从手牌区或
装备区实际失去的累计次数，并以首次失去顺序保存唯一实体牌；同一牌反复失去会重复计数，
但候选只出现一次。

「尚俭」在拥有者结束阶段检查累计数为一至当前体力值，并要求至少一张记录牌仍在弃牌堆。
发动确认后，续体重新验证技能、回合、阈值、失牌归属和弃牌堆位置，再将所选牌移入手牌。
能力未加载时不记录失牌历史；旧 JavaScript 的历史遍历、按钮 AI 与动画不进入 npack 或运行时。
