# ADR 0291：EXTRA 神张辽确定性重写

## 决策

- 旧 `skill.js` 仅作语义参考；`drlt_duorui.skill` 是唯一可执行规则源。
- `non_gainable` 作为包级技能元数据，供夺锐及后续复制技能统一过滤；主公技和限定型主动技仍由已有元数据自动过滤，旧版明确禁用名单保留为兼容表。
- 技能失效记录具体的 helper 技能实例作为原因；移除 helper 时统一解除该原因，允许多个夺锐来源安全叠加。
- 新效果的稳定哈希使用 `0 + u16` 扩展标签，旧 1–255 标签及旧包哈希保持不变。
- 止啼手牌上限由全局 modifier 查询攻击范围和受伤状态；装备栏恢复使用已有 `EnableEquipSlot`。

## 版本

- Rule VM ABI：217
- Rule IR：101
- Engine semantics：189
