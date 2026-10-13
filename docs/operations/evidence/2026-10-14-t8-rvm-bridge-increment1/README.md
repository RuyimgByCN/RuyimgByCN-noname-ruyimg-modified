# T8 增量 1：engine-native 触发调度接 rule-vm（2026-10-14）

## 任务来源与上游依据

[进度主稿 T8](../../../architecture/native-progress.md)（ADR 0782 定案后的「剩余集成」首增量）。原型前提：[T8 后端对照原型](../2026-10-11-t8-backend-prototype/README.md)（候选 B 七场景实测，两层提交协议/内容锁/fuel 回滚均已在原型验证）。本增量把该能力接进 engine-native 触发调度。

## 交付物

- **桥模块 `crates/engine-native/src/rvm_bridge.rs`**：
  - `VmSkillDecl`（extension.rs）——扩展包数据声明 VM 技能（name + BytecodeModule + TriggerMeta），经 `standard_catalog_with_extensions` 合并为目录 `SkillDef`（名称 intern 进 T6② 全局符号表，重复名拒绝）。
  - `VmSkillBehavior`/`VmMachine`——触发时以内核状态为只读查询面执行字节码；**两层提交协议**（原型同款）：排队效果仅 `Halted` 落账（`RecoverHp`/`DrawCards`/`DealDamage`[含 LoseHp 通道/nature 映射]），`Paused`（选择挂起，增量 2）与 `Err`（fuel 耗尽/trap）排队效果整批丢弃——半次效果不落账。
  - `EngineVmHost`——查询映射 `Hp`/`MaxHp`/`Distance`（computed_distance）；未映射查询/效果在 emit/query 时即 `InvalidRequest`（模块 trap → 回滚）。
  - 玩家 id 映射：rule-api 1 基（NONE=0）↔ 内核 0 基座位，桥内 +1/−1。
  - 过滤面增量 1 =恒定 owner 域（`event_player==owner`，DSL owner_is_target 族）；其余 filter 表达式映射属增量 2。
- **快照面**：`MachineSnapshot::Rvm { owner, module_hash }`（tag82/机哈希 82/a0 变体 80）——Rvm 机 start→step 原子完成，存档点不停在中段，快照仅内容锁哈希核对面；与 BossWangxiang 等 atomic 机同口径（无重建臂）。
- **依赖**：engine-native 运行时依赖新增 `rule-api`/`rule-vm`；dev-dep 新增 `content-compiler`（测试 DSL 编译）。
- **连带**：Gate 1b 图内新增 content-compiler，其 2 条存量警告（typecheck `let value` 未用绑定、codegen 两个被 `patch_jumps_in_bytes` 取代的死方法）按门禁口径清零。

## 实测记录（端到端，数据驱动、不重编译）

| 项 | 结果 |
|---|---|
| `t8_vm_skill_recover_branch_end_to_end` | DSL→编译模块→ExtensionPack.vm_skills→建局→debug_grant→1 号阶段结束：hp 2→3（受伤回 1） |
| `t8_vm_skill_draw_branch_end_to_end` | 满血分支：阶段结束手牌 +1 |
| `t8_vm_skill_fuel_exhaustion_rolls_back` | 无限循环样本 fuel 耗尽→排队效果整批丢弃（与无技能对照逐牌一致，无半次落账） |
| `t8_vm_skill_save_restore_roundtrip` | 扩展局存档经 `restore_with_extensions` 往返（内容锁：名称+版本指纹），VM def 恢复一致 |
| 定向 | 4/4 绿——[日志](t8-bridge-targeted.log) |
| 全量 | **659/0/0**（655→659，新增 4 测；golden 无动——扩展面不触标准目录哈希）——[日志](engine-native-tests.log) |
| Gate 1b | 独立 target 目录 `--all-targets` **exit 0**——[日志](gate1b-all-targets.log)（首跑抓出 content-compiler 存量警告 2 条，已清） |

## 登记边界（增量 2 待接）

- 选择挂起/恢复（`Paused`→内核 WaitChoice→`resume`；快照携带 continuation）。
- 装载器桥接 npack 内 rule-vm 模块（`bridge_pack` 产出 VmSkillDecl；本增量声明由包数据在代码侧构造）。
- DSL filter 表达式映射（现恒 owner 域）；`HostContext.skill_id` 反查回填；剩余效果/查询变体映射（MoveCards/UseCard/AddSkill/marks/RequestChoice、TargetLegal/CardUsable/HandLimit 等）。
- 跨进程恢复、恶意字节码拒收矩阵（C1 验收剩余项）。
