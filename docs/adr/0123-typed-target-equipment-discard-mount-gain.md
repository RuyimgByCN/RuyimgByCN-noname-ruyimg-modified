# ADR 0123：类型化目标装备弃置与坐骑回收

## 背景

旧「趫猛」在伤害后检查实体牌颜色与逻辑【杀】，让技能拥有者弃置受伤角色装备区的一张牌，若
该牌是坐骑且确实进入弃牌堆，再将其获得。旧 JavaScript 同时承担触发筛选、可弃置牌判断、异步
选择和弃置后的区域检查。直接加载这段脚本会绕过服务器的确定性内容边界；仅把角色登记进包又
会把尚未实现的行为误报为迁移完成。

## 决策

新增触发过滤器 `owner_source_black_sha_target_alive_has_discardable_equip`。它只接受拥有者以实体
黑色牌承载的逻辑【杀】对存活角色造成的伤害，并通过统一的 `card_discard_protected` 查询确认至少
有一张当前可弃置装备。无实体牌、无色虚拟牌、红色【杀】和受保护装备都不会打开询问。

新增通用效果 `discard_target_equipment_gain_mount(player, target)`。可选技能确认后，效果打开一次
不可取消的装备选择；续体恢复时重新验证选择者与目标仍存活、牌仍由目标持有且位于装备区，并且
弃置保护仍未生效。普通装备只产生一次弃置移动；进攻马或防御马先产生带 `Discard` 原因的移动，
仅在牌仍位于弃牌堆时再产生一次到技能拥有者手牌的移动。两个事件保持旧技能可观察的先弃后得
时序，伪造取消、移区牌和恢复期间新受保护的牌均被拒绝且不产生部分结算。

`refresh:qiaomeng` 是首个使用者。MOBILE 只登记 `xin_gongsunzan` 武将闭包，并以精确依赖
`official/refresh@0.1.0` 复用 `xinyicong` 与 `qiaomeng` 的稳定技能 ID，不复制实现。旧 JavaScript
只保留为语义证据，npack 不包含或解释脚本。

## 版本

该能力要求 Rule VM ABI 33、Rule IR 44、引擎语义修订 119。ABI 33 主机继续按加法兼容规则加载
ABI 3–32 内容包；内容 Schema、网络协议、回放 Schema、快照 Schema 与包格式不变。
`EffectRequest`、Rule IR 语句、host effect tag、触发过滤器与 `ContinuationKind` 均追加在末尾。
