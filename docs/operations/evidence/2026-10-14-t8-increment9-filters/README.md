# T8 增量 9：DSL filter 基元首梯队承载（2026-10-14）

## 任务来源

[进度主稿 T8 余项](../../../architecture/native-progress.md)「具名组合 filter 承载」首梯队：从 90 个 DSL filter 词表中挑出**引擎侧已有承载语义的 9 个基元**落 `VmFilterPolicy`；其余 ~80 个具名组合 filter 继续经 `deferred_filters` 跳过登记。

## 交付

- **`VmFilterPolicy` 扩展**（rvm_bridge.rs）：`OwnerIsSource`/`Always`/`OwnerWounded`（hp 0<hp<maxHp）/`OwnerLinked`/`OwnerHandNonempty`/`OwnerHandEmpty`/`OwnerNoTurnDamage`（turn_damage==0）/`OwnerTurnDamage`（>0）+ 既有 `OwnerIsTarget`——共 9 策略，全部对应引擎侧已有状态承载（PlayerState/turn_damage/linked），无新原语。
- **装载器映射**（content-native-loader `bridge_vm_skill_decls`）：filter 严格映射 9 基元；**未承载的具名组合 filter 跳过入 `deferred_filters`**（不阻断同包其余 VM 技能装载）。
- **行为门控**（rvm_bridge `SkillBehavior::filter`）：`OwnerWounded`=持有者已受伤（0<hp<maxHp）；`OwnerLinked`=持有者横置；其余同基语义。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | t8_rvm_bridge **16/16**（15 既有——含具名组合跳过面改用 `owner_is_target_wounded_sha` 为未承载样本——+1 既有 owner_is_source 回归）——[日志](bridge-targeted.log) |
| engine-native 全量 | **711/0/0**（不变，纯 filter 语义收敛）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[gate1b-all-targets.log](gate1b-all-targets.log) |

## 登记边界

- 剩余 ~80 个具名组合 filter 为复合语义（需跨状态查询/计数/记忆面），逐批按需立项；装载器以 `deferred_filters` 显式跳过登记。
- T8 余项收敛为：具名组合 filter 深化（按需）、剩余效果/查询变体（部分需内核新原语）。

## 补正（2026-10-14，增量 13 复核）

- 本文"行为门控"一节把 `OwnerNoTurnDamage`/`OwnerTurnDamage` 描述为"本回合未受到/已受到
  伤害"——**笔误**。内核 `turn_damage` 记在伤害**来源**侧（kernel.rs，`src != target` 时
  累加），lsp 权威语义亦为 "has dealt damage"；实现读的是持有者自己的造成计数，与权威
  语义一致，实现无错。
- 本批 12 个派生策略的实现**全部漏掉「事件主玩家=持有者」合取段**（lsp 权威语义为合取），
  且当时"定向 16/16"为既有套件数，派生策略无直接测试。两处均由
  [增量 13](../2026-10-14-t8-increment13-filter-family/README.md) 修复并补家族双闸回归。
