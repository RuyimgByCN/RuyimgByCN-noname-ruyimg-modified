# ADR 0275：REFRESH 界曹操的奸雄与护驾

ABI 199 增加参数化过滤器
`owner_alive_other_shan_outside_turn_group_<tag>_once_per_turn`，统一匹配其他指定势力角色在
自己回合外使用或打出逻辑【闪】的事件。可选触发由 `event_player` 确认，接受后才消耗当前
回合次数。

「界护驾」直接复用 ABI 184 的 `view_as shan response any from group_wei`，新增摸牌分支只使用
已有 `draw_cards`；「界奸雄」直接读取伤害事件的实体牌上下文，复用 `card_zone`、`move_card`
和 `draw_cards`。本次未增加 Host Effect、continuation 或技能专用状态，旧 JavaScript 仅作为
语义证据，不进入 npack 或运行时。
