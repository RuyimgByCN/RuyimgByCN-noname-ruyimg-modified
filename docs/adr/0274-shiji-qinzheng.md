# ADR 0274：SHIJI「勤政」的累计牌组奖励

ABI 198 增加
`gain_draw_cards_for_lifetime_card_action_divisors(player, divisor, group, ...)`。效果复用引擎
已有的本局使用/打出牌累计数；每个命中的倍数从牌堆顶优先取得一张对应牌组的实体牌，并把同次
命中的牌一次性移动到手牌区。

牌组固定为【杀/闪】、【桃/酒】和【决斗/无中生有】；各种属性【杀】均归入第一组。效果不搜索
弃牌堆，不增加技能专用状态或 continuation。旧 `qinzheng_count` 仅维护界面标记，目录中折叠到
父技能；旧 JavaScript 不进入 npack 或运行时。
