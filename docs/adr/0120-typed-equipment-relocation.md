# ADR 0120：类型化装备迁移状态机

## 背景

旧「调度」在准备阶段连续调用 `chooseTarget`、`chooseButton`、`chooseTarget`，最后直接执行
`target2.equip(card)`。它同时包含四个不能交给客户端或旧 JavaScript 决定的规则条件：装备必须
仍在原持有者装备区；技能拥有者不能成为接收者；接收者不能是原持有者且对应装备栏必须可用；
接收者已有同栏装备时须先替换。移动完成后摸牌的是原持有者，而非技能拥有者或接收者。

## 决策

新增通用 DSL 效果 `move_equipment_then_draw(player)` 与类型化
`EffectRequest::MoveEquipmentThenDraw`。引擎从权威状态构造稳定排序的可移动装备列表，选牌后再
构造合法接收者列表；两个 `ChoiceRequest` 都保存明确的选项白名单，并在 `ResolveChoice` 时重新
检查卡牌位置、存活状态、禁止接收者和装备栏状态。

接收者同栏已有装备时，效果 FIFO 先将旧装备移至弃牌堆，再移动所选装备，最后以原技能实例为
来源令原持有者摸一张牌。每一步产生普通 `CardsMoved`/`CardsDrawn` 事件并经过统一触发调度，
没有技能 ID 特判。准备阶段过滤器 `owner_is_target_has_movable_equipment` 与效果共用同一合法性
查询，因此没有合法移动时不会向玩家展示空选择。

`shiji:mbdiaodu` 是该能力的首个内容。技能只在 SHIJI 包定义一次；SIXIANG 包通过精确包依赖
引用稳定技能 ID，并据此发布完整 `std_lvfan` 武将，避免跨包复制技能实现。

## 版本

该能力要求 Rule VM ABI 30、Rule IR 41、引擎语义修订 116。ABI 30 主机继续按加法兼容规则
加载 ABI 3–29 内容包；内容 Schema、网络协议、回放 Schema、快照 Schema 与包格式均不变。
新 `EffectRequest`、`ContinuationKind`、Rule IR 语句和 host effect tag 均追加在末尾。
