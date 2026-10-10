# ADR 0124：类型化目标牌区域作用域

## 背景

旧界「趫猛」与原「趫猛」共享“弃置后若为坐骑则获得”的时序，但前者允许选择目标的手牌区、
装备区或判定区，后者只允许装备区。为每个旧技能复制一套异步选择和续体会让上千技能迁移产生
大量近似状态机，也容易遗漏隐藏手牌、弃置保护和恢复时移区重检。

## 决策

新增类型化作用域 `equip`、`hand_equip`、`hand_equip_judge`，以及效果
`discard_target_card_gain_mount(player, target, scope)`。引擎通过统一的
`discardable_target_cards` 查询按作用域收集候选，并在打开选择和恢复续体时执行同一套存活、
所有权、区域与弃置保护校验。普通牌只产生一次弃置移动；坐骑仍先进入弃牌堆，再以第二次移动
进入选择者手牌。

ABI 33 的 `discard_target_equipment_gain_mount` 指令和续体标签继续保留，并映射到 `equip`
作用域；ABI 34 内容使用新指令和带作用域的续体。`refresh:qiaomeng` 改用新效果的 `equip`
声明，`refresh:reqiaomeng` 使用 `hand_equip_judge`，两者共享运行时实现。旧 JavaScript 只作为
语义证据，不进入 npack 或运行时。

## 版本

该能力要求 Rule VM ABI 34、Rule IR 45、引擎语义修订 120。ABI 34 主机继续按加法兼容规则
加载 ABI 3–33 内容包；内容 Schema、网络协议、回放 Schema、快照 Schema 与包格式不变。
新的 `EffectRequest`、Rule IR 语句、host effect tag、触发过滤器和 `ContinuationKind` 都追加在
末尾，旧序列化枚举序号保持稳定。
