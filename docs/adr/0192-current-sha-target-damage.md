# ADR 0192：当前【杀】按目标保存伤害增量

## 背景

旧版 `mobile:xinanjian` 在拥有者使用【杀】指定一个攻击范围不包含自己的目标后，强制
选择令该目标不能响应，或只令此【杀】对该目标的伤害基数加一。旧实现把选择结果写入
JavaScript 用牌事件的 `directHit` 和 `customArgs[target].extraDamage`，不能进入确定性运行时。

## 决策

复用现有 DSL `choose(1, 2)` 暂停/恢复和
`make_current_sha_target_unrespondable(source, card, target)`，新增 Rule VM ABI 119 原语：

`increase_current_sha_target_damage(source, card, target, amount)`

原语重新验证当前保存的【杀】来源、实体牌和目标。目标伤害增量编码在既有单目标或多目标
响应标志的高位，不新增全局状态；恢复结算时仅把增量应用到对应目标。增加、移除目标以及
不可响应分支只修改各自位域，不会把增量扩散到同一张【杀】的其他目标。

同时新增
`owner_source_sha_target_alive_owner_outside_target_attack_range` 指定目标过滤器。范围方向按
旧 `!player.inRangeOf(target)` 保持为“目标的攻击范围不包含拥有者”，而不是拥有者不能攻击
目标。

## 结果

- 旧 JavaScript 只作为逐行语义参考，不进入运行包。
- 单目标与多目标【杀】都能精确保存每个目标的选择结果。
- `xinanjian` 与已重写的 `xinduodao` 闭合 `xin_panzhangmazhong` 武将发布。
