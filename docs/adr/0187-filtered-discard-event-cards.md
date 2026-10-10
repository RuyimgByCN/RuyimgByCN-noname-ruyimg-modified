# ADR 0187：弃置事件牌按通用过滤契约暴露

## 状态

已接受。

## 背景

旧 COLLAB「亥猪」监听 `loseAfter`、`loseAsyncAfter`，从旧事件对象的 `getl(target).cards2`
中查找其他角色因弃置进入弃牌堆的黑色牌并取得。新运行时不能执行旧 JavaScript，也不能把
旧事件树、动态方法或牌堆位置探测带入内容包。

现有权威模型已经提供带 `CardMoveReason::Discard`、来源区、目标区和实体牌列表的
`CardsMoved`，VM 也已有 `gain_event_cards(player)`。缺口只是：混合红黑的同一移动批次中，
技能必须只看到匹配的黑牌。

## 决策

追加通用触发筛选器 `owner_alive_other_discarded_black_to_discard`：

- 技能拥有者必须存活；
- 来源区必须属于另一名角色；
- 移动原因必须为 `Discard`，目标必须为弃牌堆；
- 批次至少包含一张原始黑色实体牌；
- 进入该触发器的 VM 前，`event_cards` 只暴露同批原始黑牌。

技能继续调用既有 `gain_event_cards(self)`；效果执行时仍重验实体牌是否留在弃牌堆，所以多个
取得类触发器按既有座次/时序竞争，不会复制牌。准备阶段的并列最多手牌判断和失去体力分别复用
`owner_is_target_tied_max_hand` 与 `lose_hp`。

## 版本与兼容性

- Rule VM ABI：111 → 112；
- Engine Semantics：158 → 159；
- Rule IR：保持 76；
- 内容 schema、包格式、协议、回放和快照 schema：不变。

没有新增 Rule IR 语句、HOST_EFFECT 或 `EffectRequest`。`TriggerFilter` 仅在枚举末尾追加，
保留已有序列化序号；只有实际使用新筛选器的内容包声明 ABI 112。

## 后果

弃置后取得指定颜色牌的后续技能可以复用同一事件边界，不需要为每个技能增加宿主效果。旧
JavaScript 的 `loseAfter`/`loseAsyncAfter`、`getl`、动态数组归并和弃牌堆位置判断均不会进入
新运行时。
