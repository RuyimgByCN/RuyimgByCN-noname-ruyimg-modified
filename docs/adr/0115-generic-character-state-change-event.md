# ADR 0115：统一武将牌状态变化事件

## 背景

旧 `nsqiyue` 同时监听翻面、横置/重置、亮将、暗置和移除武将牌。直接把这些旧事件名逐个加入
技能 DSL，会让技能与当前模式实现耦合，也会在以后加入双将或国战模式时重复扩展触发定义。

## 决策

DSL 新增 `character_state_changed`，其中 `event_player` 是状态发生变化的角色。当前权威规则把
`PlayerTurnedOver` 和 `PlayerLinkedChanged` 映射到该事件；未来亮将、暗置或移除武将牌的权威操作
也必须映射到同一事件，技能定义无需改动。

横置状态以 `MARK_CHAIN > 0` 为真。增减铁索标记时只有这个布尔值实际改变才发出
`PlayerLinkedChanged`：0→1 和 1→0 会触发，1→2、2→1、无关标记以及被防具取消的横置不会触发。
`MarkChanged` 仍按原契约保留，通用武将牌事件不从它猜测语义。

`nsqiyue` 因而只保留一个锁定触发，旧 JavaScript 仅作为语义证据，不进入运行时。

## 版本

该能力要求 Rule VM ABI 25、Rule IR 36、引擎语义修订 111。ABI 25 主机继续按加法兼容规则
加载 ABI 3–24 内容包。
