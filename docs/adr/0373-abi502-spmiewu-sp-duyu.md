# ADR 0373：ABI 502 灭吴通用实体牌转化

`spmiewu` 不保留旧 JavaScript，也不使用技能名特判。DSL 的 source-free view-as 新增
`hand_equip_mark` 代价，并用 `mark_cost <def_id>`、`post_draw <count>` 参数声明标记消耗
与结算后摸牌；`trick` 牌族覆盖普通及延时锦囊。

运行时复用既有实体牌使用/响应管线：强制选择一张手牌或装备牌作为材料，扣除一枚指定
标记，以统一的每回合次数限制覆盖使用和打出，并在相应牌完成使用或响应后摸牌。该能力
由 ABI 502 门控，首个消费者是史纪杜预【灭吴】。
