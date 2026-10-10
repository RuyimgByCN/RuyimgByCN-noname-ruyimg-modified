# ADR 0279：REFRESH 遗计与可选手牌分配

ABI 203 增加效果
`draw_then_optionally_distribute_hand_cards(player, draw_count, give_cap)`。它先完成摸牌及嵌套
触发，再让拥有者从当前全部手牌中规划至多指定数量的牌和其他存活目标；取消首轮可以一张不分，
后续取消则保留已有计划。

规划阶段不移动实体牌。续体保存牌与目标的位置配对，结束时统一验证技能、角色、牌归属、去重
和数量上限，再按目标给牌并产生普通给牌事件。这样保留旧 `new_reyiji` 的批量结算时序，同时
不执行旧 JavaScript 的 `given_map`、`reyiji_tag`、`loseAsync`、联网 UI 状态或 AI 回调。
