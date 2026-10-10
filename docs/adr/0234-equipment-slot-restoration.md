# ADR 0234：装备栏恢复

ABI 161 增加通用 `enable_equip_slot(player, slot, source)`（HOST_EFFECT 170）。权威状态按旧
规则只减少一层装备栏废除计数；栏位未废除、角色已离场时为空操作，无效栏位标签是内容错误。

成功恢复会发出带恢复来源的 `EquipSlotEnabled`，并开放 `equip_slot_enabled` 触发时点。
`owner_is_target_source_alive_disabled_weapon` 过滤器用于伤害结算后核对存活来源及其武器栏。
效果请求稳定哈希标签为 211。
