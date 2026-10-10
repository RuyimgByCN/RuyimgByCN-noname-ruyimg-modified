# ADR 0108：通用装备栏废除效果与领域事件

## 决策

DSL 新增 `disable_all_equip_slots(player)`。它编译为 Rule VM effect tag 47，并由宿主产生
`DisableEquipSlots` 类型化请求；请求只废除仍启用的栏位，按武器、防具、防御坐骑、进攻坐骑、
宝物的稳定顺序执行，同时把栏内装备移入弃牌堆。`youfeng` 的单栏成本改走同一原语，不再直接
修改玩家状态或发送魔法数字事件。

每个实际废除的栏位发布 `EquipSlotDisabled { player, slot }` 领域事件，并映射为
`equip_slot_disabled` 触发时机。`owner_has_enabled_equip_slot` 过滤器用于精确表达只在尚有可用栏位
时执行的旧规则。

该能力要求 Rule VM ABI 18、Rule IR 29、引擎语义修订 104。领域事件会进入联机事件流；协议轴
统一为 ADR 0107 已确定的版本 5，修正此前服务端与 Web 客户端仍残留版本 4 的实现偏差。

## 理由

装备栏状态是多技能共享规则，不属于某个武将的私有实现。统一效果与事件让扩展只声明差异，
也让后续技能能订阅语义事件，而无需检查技能 ID、解析 `RuleNote` 载荷或执行旧 JavaScript。
