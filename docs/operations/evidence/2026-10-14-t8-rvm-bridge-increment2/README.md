# T8 增量 2：VM 选择挂起/恢复（内核侧 continuation 共存，2026-10-14）

## 任务来源

[T8 增量 1](../2026-10-14-t8-rvm-bridge-increment1/README.md) 登记的增量 2 首项：**选择挂起/恢复**（`Paused`→内核 WaitChoice→`resume`；快照携带 continuation）——C1 验收「选择挂起/恢复在内核接线侧」的落地。

## 交付物

- **VmMachine 挂起态机**：`Paused{continuation, options}` → `Flow::WaitChoice`（observer=持有者，`PickTag`+`ChoiceOption::Tag`，prompt=`rvm_choice`）；已答→`vm.resume(continuation, token)` 续跑；**挂起期间排队效果保留**（原型同款「Paused 层保留至续跑后落账」，恢复后与续跑段效果一并提交）；`Err`/选项超 u8 域（Tag 容量）→ 回滚（整批丢弃）。
- **快照 v2（自包含）**：`MachineSnapshot::Rvm { owner, module: Vec<u8>, paused: Option<RvmPaused> }`——模块载体字节（`from_bytes` 还原验证）+ `RvmPaused { frames, pc, fuel_remaining, id, module_hash, options, pending }`（排队效果为 rvm_bridge 自定义编码，映射面仅 RecoverHp/DrawCards/DealDamage 三种）。**存档格式 63→64**（Rvm 形状扩展；canonical hash 不含存档版本，golden 不动——探针见下）。
- **active_ask 派生**：挂起态派生 `rvm_choice` 询问（BossBuchun stage-3 同款）；重建 `restore_paused` 内容锁核对（continuation.module_hash ≠ module.logic_hash 拒绝）+ `asked=true`（问询已发出，下一应答即续跑——首版 asked=false 导致恢复后重问，回归抓出）。
- **重建臂**：Rvm 入 `MachineSnapshot::restore`（增量 1 时 atomic 无臂；挂起态可在存档点出现，需真臂）。

## 实测记录

| 项 | 结果 |
|---|---|
| `t8_vm_skill_choice_suspend_resume_midrestore` | 阶段结束 VM 挂起（rvm_choice 两选项）→ **挂起中存档**→`restore_with_extensions` 恢复→挂起态在位→应答 Tag(1)→续跑落账 hp 2→3 |
| `t8_vm_skill_content_lock_rejects_pack_drift` | 挂起态存档 + 包版本 +1 漂移 → 恢复被拒（内容锁：名称+版本指纹） |
| 增量 1 回归 | 回血/摸牌/fuel 回滚/往返 4 测不变绿 |
| 定向 | 6/6 绿——[日志](t8-bridge-targeted.log) |
| 全量 | **661/0/0**（659→661）——[日志](engine-native-tests.log)；`replay_scenarios` 联动守护 2/2（存档 64 不触 golden 哈希） |
| Gate 1b | 独立 target 目录 `--all-targets` **exit 0**——[日志](gate1b-all-targets.log) |

## 坑位实录

- 恢复重建挂起机 `asked=false` → 应答被当作「重问」（hp 不动）——挂起态快照的 asked 恒 true（问询已由 active_ask 发出），定向回归（恢复后应答须一次落账）抓出。
- `MachineSnapshot::restore(&self)` 为共享引用——Rvm 挂起态以 `paused.clone()` 进重建（快照克隆路径，登记）。

## 登记边界（增量 3 待接）

- 装载器桥接 npack 内 rule-vm 模块（bridge_pack 产出 VmSkillDecl）。
- DSL filter 表达式映射（现恒 owner 域）；`HostContext.skill_id` 反查；剩余效果/查询变体映射。
- 跨进程恢复、恶意字节码拒收矩阵（C1 验收剩余项）。
