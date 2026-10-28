# B7：VM 挂起存档恢复丢失技能身份/触发语境/限次/命中（修复）

- 实际执行：2026-10-28 13:0x →（[开始](start-time.txt)）；源码基准 [`ec672a1d3c413d39c9972ee02118ae92a21223ee`](source-head.txt)（基线捕获自干净 `git archive HEAD` 导出）。**规则 v323→v324（用户裁决⑧授权，先于 NC07 批占用 v324）**；存档格式 **v105 不变**、全局原生 ABI **753 不变**、Worker／WS 消息结构不变。
- 反例与缺陷定位见 [NC04 选择重试证据](../2026-10-28-nc04-choice-retry/README.md)：恢复后 `VmFault` 技能声明名变为模块名（`nc04_restored_fault`→`nc04-restored-fault`）、`HOST_CHOICE` 后 `STORAGE_SET` 在恢复局额外写出以模块名为键的条目。

## 缺陷根因

`MachineSnapshot::Rvm`（T8 增量 2）只携带 owner、模块字节与挂起态；[rvm_bridge.rs](../../../../crates/engine-native/src/rvm_bridge.rs) 的 `restore_paused`/`from_module` 以 `module.name` 重建 `skill_name`，并把 `trig`（触发上下文）、`consume`（限次清单）、`count_hits`（命中旗标）重置为默认。续行后：

- `STORAGE_SET`/`commit_effect`/故障身份以模块名而非**声明技能名**为键；
- `once` 限次消费丢失（恢复机 Halted 不落账）、`vm_aux.hits` 命中不计数；
- 续行中宿主查询（HOST_EVENT_*）读到默认触发语境。

## 修复设计

`MachineSnapshot::Rvm` 尾部追加四个语义字段（编码在 paused 块之后，旧格式字节在此读完即止）：

| 字段 | 载体 | 恢复路径 |
|---|---|---|
| `skill_name: String` | 声明技能名（与模块名独立） | 直接作为 `VmSkillProgram` 名，不再从模块名重建 |
| `event: EventData` | 触发事件原文 | 恢复时经 `trigger_ctx_of` 重派生 `trig`（确定性函数，避免 14 字段重复编码；复用 `OptionalGate` 已有的 EventData 机器快照先例） |
| `consume: Vec<(String, u8)>` | 限次 (键, 范围) 清单 | 恢复机 Halted 时照常写 `vm_aux.once` |
| `count_hits: bool` | 命中旗标 | 恢复机伤害触发 Halted 时照常计 `vm_aux.hits` |

同步六处：`snap/types.rs`（变体）、`archive/machine_encode.rs`、`archive/machine_decode.rs`、`engine/hash.rs`（**四字段进入规范摘要**——两个仅声明技能名或限次清单不同的挂起机此前得到相同摘要）、`rvm_bridge.rs`（机器持 `event` 原文；两条恢复路径共用新参数）、`snap/restore.rs`。另在 `snap/machine_validation.rs` 补载入校验：`skill_name` 非空、consume scope ∈ {1,2,3}（编译器词表值域 turn/round/game），越界视为损坏拒绝。

考虑过的替代方案：①仅存 `VmTriggerCtx` 派生 14 字段——三处（编码/解码/哈希）×14 字段样板，且 Phase/Nature/CardKind 枚举映射要在 rvm_bridge 重复；选 EventData 原文（一次编码三处复用已有 EventData 机器快照支撑）。②恢复时从目录 def 的滤镜重新推导 consume/count_hits——`MachineSnapshot::restore()` 无目录访问权，穿透目录参数侵入更大，且快照冻结原始值对确定性回放更稳。

## 版本策略与旧档拒绝（双通道）

- 规则版本 v323→v324（`STANDARD_RULESET_VERSION`）。扩展目录版本派生 `323 ^ (lock | 1<<63)` 随之变化，扩展旧档同样被版本门禁拒绝。
- 普通旧档（无 Rvm 机）：解码成功，`Engine::restore` → `RuleVersionMismatch { snapshot: 323, catalog: 324 }`（[b7_plain_v323_archive_is_rejected_by_rule_version]）。
- 含旧格式 Rvm 机的旧档：新解码器在 paused 块后继续读技能身份字段即遇 EOF，**解码期失败**（`b7_suspended_vm_v323_archive_is_rejected`）——不存在半恢复；该档本就是"不能可靠恢复"的一类，版本门禁与解码失败同为拒绝，语义边界一致。

## 基线与反例证据

| 文件 | 内容 |
|---|---|
| [native-v323.ngsa](native-v323.ngsa) | v323 普通新局档（65101 种子，4 座），升版后按版本拒绝 |
| [vm-suspended-v323.ngsa](vm-suspended-v323.ngsa) | v323 HOST_CHOICE 挂起中的 VM 局档（旧 Rvm 编码），升版后解码失败 |
| [original-continuation.txt](original-continuation.txt) | 缺陷版（v323）原局续答后的 vm_aux 基线：`storage={"b7_storage_skill#7": 11}` |
| [mutation.log](mutation.log) | 变异验证：把 `restore_paused` 临时退回缺陷行为（模块名重建+重置清单/旗标），四个身份回归全部失败且失败形态与反例逐字一致；还原后 9/9 |
| [capture-v323.rs](capture-v323.rs) | 基线捕获脚本（干净 HEAD 导出实跑） |

## 回归（GT-011：精确相等替换 shortcut 非空）

`crates/engine-native/tests/nc04_vm_resume.rs`：

1. `nc04_vm_fault_after_archive_restores_choice_and_allows_retry`——原 `shortcut:` 非空断言替换为 `assert_eq!(skill, "nc04_restored_fault")`（故障身份逐字一致）。
2. `nc04_vm_restore_uses_declared_skill_for_storage`（新）——HOST_CHOICE 后编码→恢复→续答 STORAGE_SET：恢复局存储键 = `nc04_storage_skill#7`（非模块名），双局 storage/core/resolution/哈希逐字一致。
3. `nc04_vm_restore_preserves_once_consume`（新）——`owner_alive_once_per_round` 真实滤名（consume=[(空键,2)]）：双局 `once` 均落账 `[(1,"nc04_once_skill",2)]`。夹具先推进到 1 号弃牌阶段再授技（否则更早阶段尾触发被 drain 代答消耗限次）。
4. `nc04_vm_restore_preserves_hit_counting`（新）——`owner_alive_first` + 伤害触发（`debug_deal_damage`，kind==1）：双局 `hits[(1,"nc04_hits_skill")]` 均 = 1。
5. 旧档拒绝双通道两项（见上）。

## golden 重录（先查明原因再重录）

14 个标准场景 golden 全部位移。**版本回退探针**：临时把 `STANDARD_RULESET_VERSION` 改回 323、修复代码不动 → 14 场全绿——证明位移**仅因规则版本进入规范摘要**（标准场景无 VM 技能，Rvm 摘要扩展面由上述身份回归覆盖）。据此重录 `replay_scenarios.rs` 14 个期望哈希并补记注释。`pa12_tag656_version.rs` 的目录版本断言同步 323→324（v322 旧档仍拒绝）。

## 验证

| 命令／范围 | 完整结果 |
|---|---|
| `cargo test --offline -p engine-native --test nc04_vm_resume` | **9/0/0**（含变异验证回合，见 mutation.log） |
| `cargo test --offline -p engine-native --test replay_scenarios` | **3/0/0**（14 golden 重录后） |
| `cargo check --offline --workspace --all-targets` | 通过（存量警告不变，Gate 1b 他人未提交改动口径照旧） |
| `cargo test --offline -p engine-native --no-fail-fast` | 见 [native-full-summary.txt](native-full-summary.txt)（全量日志 /tmp/b7-native-full.log） |
| `cargo test --offline -p game-server` | 见 [game-server-summary.txt](game-server-summary.txt) |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py`；`python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py` | 见 [gates.log](gates.log) |

## 兼容性

- 规则版本 v323→v324（授权：裁决登记第 8 项；本批先提交占用 v324，NC07 炼妖壶批次如需升号须重新确认——见台账登记）。
- 存档格式 v105、全局原生 ABI 753、Worker／WS 消息结构不变。
- 旧档：v323 及更早一律拒绝（普通档版本拒绝；含旧 Rvm 机档解码失败）。无迁移器（按裁决"若需要旧档迁移，另行请示"）。

## 未覆盖项

- `from_module`（非挂起 Rvm 机）恢复路径已同参数化，但快照点不停在 start→step 中段（既有论断），无非挂起实例可回归——按同一快照数据流覆盖，不再单测。
- 恢复后 `HOST_EVENT_*` 查询读回触发语境（trig 保留的运行时效果）未单独成测——由快照 core/resolution 逐字一致间接覆盖。
- NC04 其余矩阵（续行来源认证、异常 VM token 等）不在本批范围。
