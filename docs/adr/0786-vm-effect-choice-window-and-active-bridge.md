# ADR 0786：VM 效果级选择窗挂起与主动技桥（PA12 §2 J + 批⑥，草案）

日期：2026-10-28　|　状态：**J1 已实施（2026-10-29，规则 v328，工作树未提交，[证据](../operations/evidence/2026-10-29-adr0786-j1/README.md)）；J2 仍为草案**
关联：[DSL 迁移计划](../architecture/native-dsl-migration-plan.md)（§2 J、批⑥、第 5/6 条）、[ADR 0782](0782-extension-backend-and-legacy-js-scope.md)、[ADR 0778](0778-native-c0-pack-data-keys.md)、[裁决登记第 5 项](../operations/evidence/2026-10-28-user-decisions/README.md)

## 背景

T8 接线（[rvm_bridge.rs](../../crates/engine-native/src/rvm_bridge.rs)）落地的两层提交协议：

- VM 执行期效果只**排队**（`pending_effects`）；`Halted` 才整批提交（`rvm_ops::commit_effect`）；`Err` 整批回滚；`Paused`（字节码 `HOST_CHOICE`）以 `WaitChoice(PickTag)` 挂起、应答后续行——选择发生在**字节码执行期**。

两个登记缺口（[迁移计划](../architecture/native-dsl-migration-plan.md) §2 J / G、批⑥）：

1. **效果级选择窗**：旧线 `discard_hand_equip_count(N>0)` 实为「开窗选恰好 N 张、按所选弃置」；原生桥无「效果**落地时**开窗挂起」的挂起点——批④-2 以确定性首 N 近似（`rvm_ops.rs`：手牌优先、装备补足、不足尽弃），`N=0`（VM 首发开选牌）在 accept 期直接拒绝（`DiscardHandEquipCount 选牌首发未接`）。牵连同族：`ChooseDiscardEquipmentPreventCurrentDamage`、`choose_other_damage`（48 技能）、`choose_use_virtual_sha`（33 技能）；`discard_hand_equip_count` 影响面 80 技能。另有 §2 L（card_targeted 模块体逐目标 `event_player`）需逐目标窗口，随本基建一并评估。
2. **主动技桥**：npack `active_index`（主动技声明：技能、模块入口、用法约束）在引擎侧**无任何消费**；`active`／view_as 不走触发桥（sp 728 技能中大量主动技偏置）。N01 胆守／平寇依赖「批⑥ + §2 J」两者，迁移计划第 5 条建议合并单批并先立本 ADR。

## 决策（草案）

### A. 效果级选择窗挂起（§2 J）

1. **不扩展字节码面**：挂起点放在**提交期**而非 VM 执行期。`commit_effect` 对「开窗型效果」不再内联近似，而是返回 `NeedsChoice { 后续队列游标, 选择规格派生参数 }`；`VmSkillMachine` 转入**提交期挂起**态并 `Flow::WaitChoice`，应答后按所选继续提交剩余队列。
2. **状态载体**：扩展现有 `MachineSnapshot::Rvm`（B7 后已携带声明技能名/触发语境/限次/命中四身份字段）追加提交期挂起字段：`pending_effects` 序列化、当前效果请求原文、应答游标。归档按 B7 同款**尾部追加**，旧档解码策略在实施批定案（预计保持存档 v105；若必须结构变化则升 v106 并另请示）。
3. **应答通道**：复用现有 `Answer`（choice_id + selections）与两层去重，**不新增命令通道**。`ChoiceKind`/prompt 复用既有种类与受限键（SelectCards/SelectPlayers + `rvm_effect_pick` 类受限 prompt）——**若定案需要新增 ChoiceKind 变体或参数化 prompt 载荷，即选择协议扩展，实施前单独请示**。
4. **DSL 原语面**：两案——①保留 `N=0` 哨兵为「玩家选恰好 N」（N 由调用方另传）；②新增显式原语 `discard_hand_equip_choose(player, min, max)`。倾向②（语义显式、编译期可校验 0≤min≤max），实施批定案；**原语新增=编译器与 VM 双侧同步的 ABI 增量，`RULE_VM_ABI_VERSION` 升号须授权**（迁移计划第 5 条「升号整包重装」）。
5. **确定性与 AI**：选择作为命令进入命令流（可回放、跨端一致）；AI 座位以现有候选启发式应答；挂起不消耗 fuel。
6. **近似差异处置**：迁移为真实选择后，「确定性首 N」与上游的等价性差异（批④-2 起逐技能登记的近似差异）反向收敛：玩家总选前 N 时行为等价。golden 因语义变化重录，规则版本升号（预计 v326 起，以实施时实际占用为准，**须授权**）。

### B. 主动技桥（批⑥）

1. `active_index` 经 [content-native-loader](../../crates/content-native-loader) 桥成 `VmActiveDecl`（技能名、已验证模块、入口 pc、用法约束：每回合/阶段限次、距离与目标数规则、手牌消耗声明）并入目录 def——与触发技同一目录身份/内容锁口径。
2. **激活命令**：复用现有 `GameAction::ActivateSkill { skill }`（def 由目录持有），`legal_actions` 在出牌阶段按 `VmActiveDecl` 约束给出候选——**不新增 GameAction 变体即无命令协议变化**；若验收发现必须新变体（如需携带牌面消耗声明），按协议扩展另行请示。
3. **执行**：激活走与触发同款 `VmSkillProgram` 执行（active 入口）；激活中的选牌/选人复用 §2 A 的提交期挂起——**两者因此合并一批实施**（迁移计划第 5 条）。
4. **限次/用法**：映射既有 `vm_aux.once`（owner, skill, scope）与 TurnState used 旗标臂；view_as 转化沿用 `use_as`/`use_virtual_card` 既有原语（v169/v178 已落地）。
5. **N01 试点**：胆守（`ol_jie_danshou` 四选项主动技）作为首个真实验收：DSL 编译子包核对原字节码做定向测试；整包通过须 `RULE_VM_ABI_VERSION` 升号后整包重装（同上）。

### 实施批次建议（裁决后立项，均须按门槛另行授权）

- **J1**：提交期挂起基建 + `discard_hand_equip_count` 真实选择 + 单族验收（choose_other_damage 或 choose_use_virtual_sha）；
- **J2**：`active_index` 桥 + ActivateSkill 通道 + N01 胆守整包试点（含 ABI 升号与整包重装）。

## 已知代价与风险

- 提交期挂起使 `pending_effects` 成为可恢复状态：快照体积与校验面增长（逐效果序列化 + 恢复所有权校验，GT-011 同款断言要求）；
- 选择窗嵌套（效果选择触发濒死/救援等固有流程）：须复用既有嵌套挂起恢复矩阵（nc08_save_restore_matrix 同口径）；
- fuel 与挂起交互、挂起中限次/命中计数的时点（B7 字段语义不回退）；
- 「确定性首 N 近似 → 真实选择」是**语义变化**：AI 与回放轨迹、golden、跨端对照全部受影响，影响面按 80+48+33 技能族分批收敛。

## 状态

草案。本 ADR 属[裁决登记第 5 项](../operations/evidence/2026-10-28-user-decisions/README.md)授权的「设计与 ADR 起草」范围；**VM 提交协议扩展（`MachineSnapshot::Rvm` 提交期挂起字段）、选择/命令协议的具体改动、`RULE_VM_ABI_VERSION` 与规则/存档升号，实施前仍须分别请示**。未拍板前，批④的确定性近似与登记差异照旧，不因本草案改变。

**2026-10-28 23:28 补记**：用户采纳 J1（[裁决登记第 13 项](../operations/evidence/2026-10-28-user-decisions/README.md)）：DSL 原语取方案①、不新增原语、`RULE_VM_ABI_VERSION` 不变，规则升 v327→v328，存档预计保持 v105，应答复用既有 Answer 与 ChoiceKind；需要新增 ChoiceKind 变体、prompt 载荷或升存档格式时另行请示。J2 与 ABI 升号未授权；先做 754–758 级原语盘点（只分析）。

**2026-10-29 实施补记**：J1 已按上述边界实施（规则 **v328**、存档 v105 尾追、ABI 753 不变）。§A.3 的「受限 prompt 键」落地为 `KNOWN_PROMPTS` 表尾追加 `"rvm_discard_pick"`（`ChoiceKind::SelectCards` 复用、Answer 通道不变；解读见证据 §6）。反例红→绿、存读档（GT-011 口径）、不足封顶／空过、N=0 编译期拒绝与 golden 探针见[证据](../operations/evidence/2026-10-29-adr0786-j1/README.md)。§5 的 754–758 原语盘点亦在该证据（756–758 三级在原生宿主均未实现，757/758 依赖 J2）。
