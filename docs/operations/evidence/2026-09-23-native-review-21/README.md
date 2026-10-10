# 2026-09-23 第 21 轮：装备技能挂载生命周期 + 3D精选纵向样本（规则版本 33）

本轮交付两项：(1) **装备技能生命周期**——装备时授予、离场撤销（覆盖全部 6 个装备
离场路径）与恢复一致性校验；(2) **3D精选纵向样本**（角色包管线：catalog 扩展武将 +
精彀宿主行为，触发 + 目标选择 + 伤害，字节恢复贯通）。
`STANDARD_RULESET_VERSION` **32 → 33**（目录扩展武将表、装备技能语义、精彀行为；
14 golden 重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **291 passed / 0 failed**（第 20 轮 286 + 装备生命周期 2 + 3D精选纵向 3）。全工作区 4,857 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v33 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 装备技能挂载生命周期

- **声明**：`CardDef.equip_skills`（目录技能名列表；基础目录暂无，欢乐卡牌苦无声明
  `huanle_kuwu`，wuxingpan 待挂载生命周期后补）。
- **授予**：`commit_equip` 装备入栏后授予（`skills` 表 insert，重复装备幂等）。
- **撤销**（6 个离场路径全部覆盖）：同槽替换（旧牌离场）、拆桥、顺手牵羊、
  `take_owned_card`（倾国等以装备响应的通用移除）、身份局主公惩罚清空、手里剑弃装备。
  撤销检查跨槽同名技能共存（剩余装备仍提供则不撤销）。
- **恢复一致性**（信任边界）：快照中「已装备牌声明的技能缺少实例」→
  `EquipSkillMismatch { player }` 拒绝（内存与字节两条路径；反向——实例存在而装备
  离场——不拒绝，兼容临时授予/调试授予，登记为已知宽容方向）。
- 回归：`equip_skill_lifecycle_grant_and_revoke`（装备→授予→生效→手里剑弃装备→撤销）、
  `equip_skill_snapshot_mismatch_rejected`（篡改快照双路径拒绝）。

## 3D精选纵向样本（packages/extensions/d3-upstream-noname/）

- **角色包管线**：`Catalog.extension_generals`（`ExtensionPack.generals` 合并，
  `general_meta(g)` 统一基础/扩展查询：名称/势力/技能名/体力/性别）；
  `RestoreBounds.general_count` 恢复槽位界校验；`declared_faction`/`effective_faction`
  改为带目录参数（扩展武将势力可解析）。
- **切片**：刘冲（ddd_liuchong，群/4血/男 @ 5c02a745）——精彀映射为宿主原语
  `MaxRangeWeaponDamage`（使用武器牌结算结束后攻击范围全场最大 → 选一名其他角色
  造成 1 点伤害：Equipped 强制触发 + SelectPlayers 目标选择 + 字节恢复）；
  **末焱登记暂缓**（包数据 `mapping:null`，测试断言；需展示牌/获得/标记/伤害增幅原语）。
- 回归：`g1_d3_install_bridge_and_metadata`（元数据/暂缓登记/越界 None）、
  `g1_d3_vertical_reveal_jinggou_and_restore`（国战明置扩展武将 → 势力/体力/技能来自
  包元数据 → 装备苦无触发精彀 → 字节恢复 → 1 点伤害）、
  `g1_d3_content_lock_and_bounds`（越界槽位拒绝 + 内容锁）。

## 边界（如实登记）

- 3D精选为**部分转换**（1/33 将，末焱暂缓）；整包验收仍 **0/37**。
- 装备技能生命周期撤销覆盖当前全部 6 个离场路径；未来新增离场路径须同补撤销
  （恢复一致性校验会兜底拒绝，不会静默错局）。
- 精彀的「全场最大攻击范围」含装备自身（刚装备的武器计入）——与上游
  `player.getAttackRange()` 同语义。
