# ADR 0122：类型化双目标装备弃置

## 背景

旧「调和」在出牌阶段选择两名角色，再从两人的装备区各选择一张牌，并要求两张牌中至少一张为
防具。旧 JavaScript 同时承担目标筛选、卡牌候选和两段选择恢复；若直接随扩展加载，会让服务器
重新依赖任意脚本执行，也无法在选择恢复时统一检查装备位置与弃置保护。

## 决策

主动技目标声明新增 `equipment_pair_with_armor`：只允许 `targets 2 2`，两名目标必须不同且存活，
各自至少有一张技能拥有者可弃置的装备，并且至少一方有可弃置防具。Rule VM 主机向主动技模块
提供完整的已选目标列表，不再只绑定首名目标。

新增通用效果 `discard_equipment_pair_with_armor(player)`。状态机先选择第一名目标的装备，再按首张
是否为防具收窄第二名目标的候选；若第二名目标没有防具，第一步直接收窄为防具。两个强制选择
屏障都会根据权威状态重新检查存活、装备位置和 `card_discard_protected` 修正器。完成后以 FIFO
分别产生两次弃置移动，因此普通失去装备触发仍按既有事件管线工作。目标和装备顺序均稳定，AI
与无头自动玩家复用通用多目标组合枚举。

`sixiang:stdtiaohe` 是首个使用者；它与已移植的 `stdqiansu` 闭合并发布 `std_feiyi`。旧
JavaScript 仅保留为语义证据，npack 不包含或解释脚本。

## 版本

该能力要求 Rule VM ABI 32、Rule IR 43、引擎语义修订 118。ABI 32 主机继续按加法兼容规则
加载 ABI 3–31 内容包；内容 Schema、网络协议、回放 Schema、快照 Schema 与包格式不变。
`EffectRequest`、Rule IR 语句、host effect tag 与 `ContinuationKind` 均追加在末尾。
