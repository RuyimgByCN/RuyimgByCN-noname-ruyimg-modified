# T8 增量 8：HostContext.skill_id 目录反查回填（2026-10-14）

## 任务来源

[进度主稿 T8](../../../architecture/native-progress.md) 余项「skill_id 反查」——增量 1 起.HostContext.skill_id 恒 0（def id 未回填，登记项）。

## 交付

- **`VmSkillProgram` 增 `skill_name: String`**：VM 技能声明名（=目录 def 名）随声明传入 program。
- **`rvm_bridge` step：skill_id 经目录反查回填**——`catalog.skill_by_name(&program.skill_name)` 查不到保持 0（NONE 语义）。
- 装载/重建构造点同步：standard_catalog_with_extensions 合并点传 `decl.name.clone()`；`from_module`/`restore_paused` 以 `module.name.clone()`（重建路径 HostContext 仅诊断用，登记）。
- 定向回归 `t8_vm_skill_id_reverse_lookup_wired`：装配 def 名=声明名同源核对 + def id 非 NONE。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | t8_rvm_bridge **15/15**（14 既有+1 本批）——[日志](skillid-targeted.log) |
| engine-native 全量 | **704/0/0**（703→704）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[gate1b-all-targets.log](gate1b-all-targets.log) |

## 登记边界

- T8 余项收敛：具名组合 filter 承载与剩余效果/查询变体（部分需内核新原语）按需立项；skill_id 反查本批闭环。
