# T8 增量 6：DSL filter 三基语义映射（2026-10-14）

## 任务来源

[T8 增量 5](../2026-10-14-t8-rvm-bridge-increment5/README.md) 登记的「filter 表达式映射」：DSL 触发 filter 不再恒定 owner 域——三基语义严格落引擎。

## 交付物

- **`VmFilterPolicy`**（rvm_bridge）：`OwnerIsTarget`（event_player==owner）/`OwnerIsSource`（event_source==owner）/`Always`（无条件）；`VmSkillBehavior.filter` 按策略门控（与 def TriggerMeta 事件门 AND）。
- **`VmSkillDecl.filter`** 字段贯通（extension/merge/loader）。
- **装载器 `bridge_vm_skill_decls` 返回 `VmBridgeOutcome { decls, deferred_filters }`**：三基 filter 映射；**具名组合 filter**（typecheck 词表 `owner_is_target_wounded` 等）engine 无承载——跳过入 `deferred_filters` 登记（不阻断同包其余 VM 技能装载）。

## 实测记录

| 项 | 结果 |
|---|---|
| `t8_vm_skill_filter_owner_is_source` | 持有者造成伤害→摸 1；承受伤害→不触发（坑位：decl TriggerMeta 须与 DSL 事件一致[Damage]，PhaseEnd 门会静默拦下——测试首版踩中） |
| `t8_vm_skill_filter_always` | 他人阶段结束即触发（不受 owner 域限定） |
| `t8_vm_skill_loader_skips_named_filters` | `owner_is_target_wounded` → decls 空 + deferred_filters 登记 |
| 增量 1–5 回归 | 11 测不变绿 |
| 定向 | **14/14** 绿——[日志](t8-bridge-targeted.log) |
| 全量 | engine-native **669/0/0**（666→669）、game-server 47/0/0、loader 4/0/0——[日志](engine-native-tests.log) |
| Gate 1b | 独立 target 目录 `--all-targets` **exit 0**——[日志](gate1b-all-targets.log) |

## 登记边界（后续增量）

- 具名组合 filter 的 engine 侧承载（逐 filter 语义落核，按需立项）。
- 剩余效果/查询变体映射（ChangeMaxHp/AddSkill/marks/RequestChoice、TargetLegal/CardUsable/HandLimit 等——部分需内核新原语）。
- 跨进程恢复（同宿主双进程存档接力；内核侧单进程往返/挂起中存档已实测）。
- `HostContext.skill_id` 反查。
