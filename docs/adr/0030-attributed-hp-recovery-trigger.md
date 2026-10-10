# ADR 0030：带来源的回复事件与技能触发

## 决策

`EffectRequest::RecoverHp` 与 `DomainEvent::HpRecovered` 均记录稳定的 `source`；无来源回复使用
`PlayerId::NONE`，旧序列化数据缺少该字段时也按 `NONE` 读取。卡牌、救援、原生触发器和 Rule VM
产生回复时写入真实来源。技能目录新增 `hp_recovered` 事件，触发上下文公开回复目标、来源和实际
回复量；`owner_is_target_other_source_alive` 统一表达“目标为拥有者、来源为另一名存活角色”。

## 原因

旧 JS 的 `recoverEnd` 技能经常依赖 `event.source` 和 `event.num`。此前领域事件只记录目标与数量，
无法区分自我回复、他人回复和无来源回复。把来源留在客户端或技能私有状态会破坏服务器权威与
回放确定性，因此来源必须从类型化效果一路进入领域事件和 Rule VM 上下文。

首个使用者是一将成名法正的 `enyuan2`：其他存活角色每令其回复 1 点体力，该来源摸一张牌；
自我回复、无来源回复和已死亡来源均不触发。

## 兼容性

新增字段带 `serde` 默认值；`EventKind` 与 `TriggerFilter` 新变体均追加在末尾，不改变旧 npack
的枚举序号。效果规范哈希纳入回复来源，规则可表达范围发生变化，`EngineSemanticsId` 从修订 13
升至 14；Native、浏览器 WASM 与 skill-vm 必须重新通过跨目标一致性门禁。
