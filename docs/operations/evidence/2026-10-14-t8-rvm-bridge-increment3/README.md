# T8 增量 3：装载器桥接 npack 内 rule-vm 模块（2026-10-14）

## 任务来源

[T8 增量 2](../2026-10-14-t8-rvm-bridge-increment2/README.md) 登记的装载器项：C0 装载器产出 `VmSkillDecl`——「DSL→编译 npack→装载器→引擎」全数据驱动链路打通（不重编译）。

## 交付物

- **content-native-loader 新增 `bridge_vm_skill_decls(&ContentPackage)`**：npack `trigger_index`×`modules` → `VmSkillDecl`（声明名=模块名，intern 进符号表并过上限门）；事件名严格映射 `phase_end/phase_begin/damage/card_used/death`→`EventKind`，未知事件名/缺模块拒装（`InvalidField`）；`optional` 透传 TriggerMeta。装载器新增 `content-schema` 依赖（无环）。
- **端到端**：DSL→`compile_source_named`→`bridge_vm_skill_decls`→`ExtensionPack.vm_skills`→建局→触发→落账——装载器在环全数据流。

## 实测记录

| 项 | 结果 |
|---|---|
| `t8_vm_skill_loader_bridge_end_to_end` | 装载器桥接 1 声明（`hnl_jiuwei_t8_phase_end`）→建局→授技→阶段结束受伤回 1 |
| 增量 1+2 回归 | 6 测不变绿（含挂起中存档恢复/包漂移拒绝） |
| 定向 | 7/7 绿——[日志](t8-bridge-targeted.log) |
| engine-native 全量 | **662/0/0**（661→662）——[日志](engine-native-tests.log) |
| 装载/包面 | content-native-loader **4/0/0**——[日志](loader-tests.log)；content-runtime+engine-ai **62/0/0** |
| Gate 1b | 独立 target 目录 `--all-targets` **exit 0**——[日志](gate1b-all-targets.log) |

## 登记边界（后续增量）

- game-server `/native` 磁盘路径接 `bridge_vm_skill_decls`（现 C0 能力在装载器，服务端 drop-in 目录读 npack 载体字节接线待批）。
- DSL filter 表达式映射（现恒 owner 域）；`HostContext.skill_id` 反查；剩余效果/查询变体（MoveCards/UseCard/AddSkill/marks/RequestChoice 等）与查询（TargetLegal/CardUsable/HandLimit 等）映射。
- 跨进程恢复、恶意字节码拒收矩阵（C1 验收剩余项）。
