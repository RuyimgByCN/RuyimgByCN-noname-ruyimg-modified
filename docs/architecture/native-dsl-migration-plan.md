# DSL/IR → 原生 生成迁移计划（PA12④，2026-10-27 初版）

> 本文是台帐 [PA12](native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26) ④ 的交付物：原型证据见 [PA12④ 原型](../operations/evidence/2026-10-26-pa12-dsl-native-prototype/README.md)。**批量迁移前须再请用户确认**（PA12 裁决条件④）；本文只定义批次、缺口与验收口径，不自动成为实施授权。规则/存档版本不随本文变化。

## 1 结论与路线

- **路线成立**：现有 ported `.skill`（旧线 DSL 产物）经 content-compiler → `content-native-loader::bridge_vm_skill_decls` → engine-native `VmMachine` 即可在原生整局运行，**不写新的原生 Rust 宿主代码**。原型用 sp 包真实技能 `olyinji`（隐绩：结束阶段体力非唯一最高时 `choose(1,2)` 回复 1 或上限 +1）完成：正式装载路径、触发、挂起、作答续行、效果落账、挂起中存档→恢复→续行（同一快照双分支）、与旧线 vm_host 对同一字节码的差分一致、上游语义（结束阶段、仅并列最高触发）核对。
- **原型暴露并修复了两处桥接缺陷**（随原型提交）：① `VmMachine::run` 在 Paused 分支置 `asked=false`，活引擎首答被丢、再问一次（恢复路径不受影响，故此前测试未发现）；② 装载器把 DSL `phase_end` 映到原生 `PhaseEnd`（任一阶段结束帧），而旧线 `phase_end` = `TurnStateChanged` 进入结束阶段——原生每回合触发 5 次。修法：`VmFilterPolicy::AtPhase` 阶段门 + 装载器阶段事件表（`phase_end`→PhaseBegin@End，`phase_{judge,draw,play,discard}`→PhaseBegin@X，`phase_{draw,play,discard}_end`→PhaseEnd@X）。
- **规模**（[盘点脚本输出](../operations/evidence/2026-10-26-pa12-dsl-native-prototype/inventory.txt)，2026-10-27 对 `content/original/ported/**/*.skill` 静态统计）：7,614 个 skill 声明；触发事件 49 种名、已映射 24 种；滤镜 708 个推迟名（2,358 次触发使用）；效果调用基础变体 2,782 次（原生 `accept_effect` 已允许）vs 宿主解析族 2,307 次 / 719 个名；文件级「事件已映射 ∧ 滤镜命中 ∧ 效果全基础 ∧ 无主动技」近似可直接原生 **579/7,614**。

## 2 缺口盘点（按解决成本排序）

| 类 | 现状 | 代表（次数） | 处理 |
|---|---|---|---|
| A 命名对不上（零语义成本） | 旧线滤镜名与原生精确表拼写不同 | `owner_is_source`（206，原生名 `owner_source`）、`owner_alive`（754，原生 filter_view 有 alive 位但无精确名） | 装载器别名/新增精确策略；每个补一条桥测 |
| B 阶段帧 | 原生 `Phase` 无 Prepare；`phase_begin`（90）旧线产生者未核清；`phase_any_end`（80）无对应 | `phase_prepare`（227） | 原生加准备阶段帧或把 `phase_prepare` 落到回合开始帧（须核对旧线 TurnStateChanged→Prepare 的时机与 Judge 的关系）；`phase_begin` 逐一核对后再映射 |
| C 事件名未映射 | 装载器拒装整包 | `game_started`（96，原生有 StartGame 挂起机制）、`hand_compare_finished`（37）、`skill_added`（36）、`card_resolved`（35）、`global`（23）、`sha_resolved/sha_dodged`（15/15）、`judgment_resolved`（9）、`round_started`（9）、`max_hp_changed`（7）、`debate_*`、`duel_resolved`、`cards_given`（4，PA12② 点名）… | 按原生已有事件逐个映射（`EventKind`/`LogEvent` 已有的直接接，如 game_started）；无帧的登记为原生事件缺口（与 NC04 未映射事件合并） |
| D 滤镜语义（需实现） | 推迟→技能静默不触发（当前装载器行为，**危险**：包装上了但不响） | `owner_source_sha`（76）、`owner_alive_not_target`（65）、`owner_is_target_once`（60）、`owner_alive_once_per_round`（38）、`phase`（34）… | 先把「推迟」改成**装载期拒绝或显式告警**，再按频次实现为精确策略/具名程序（`rvm_named`） |
| E 宿主解析原语 | 719 个 `resolve_*/choose_*` 名，每个对应旧线一段 Rust 状态机 | `discard_selected`（96）、`discard_hand_equip_count`（91）、`choose_other_damage`（68）… | 按包就绪率与频次排批；每个原语=原生 `accept_effect` 变体 + 内核臂 + 差分测试；ABI 号沿用 `ABI-COORDINATION.md` 领取（tag 空间已至 771） |
| F ABI/版本门 | `RULE_VM_ABI_VERSION` 753 < 部分包要求（onlyOL 757） | onlyOL/N01 | 宿主实现 754–757 对应原语后升号重打包（兼容判据对 host 单调） |
| G 主动技/view_as | `active`/视作转化不走触发桥 | 偏置：sp 728 技能中大量主动技 | 需原生主动技桥（`active` → PlayCard 候选 + 选牌/选人） |

## 3 批次（每批独立验收、独立提交；**启动批量迁移前先请用户确认**）

1. **桥接词汇对齐**（A + 推迟改拒绝/告警）：别名 `owner_is_source`、新增 `owner_alive`；装载器对推迟滤镜改为显式拒绝（或 `--allow-deferred` 显式降级）；重跑 `inventory` 看近似可原生数。预计 +~900 触发使用。
2. **阶段/事件帧补齐**（B + C 中原生已有事件）：`game_started`、`phase_prepare`（含原生准备帧决定）、`phase_begin` 核对、`hand_compare_finished`、`skill_added`、`max_hp_changed`。每个事件：一条含真实 ported 技能的原生整局测试 + 旧线差分。
3. **滤镜 Top-20**（D）：按频次实现，每个一条桥测；并把 `once`/`once_per_round` 落到原生限次机制。
4. **宿主解析原语 by 包**（E）：先选就绪率高、技能数中等的包整包推进（盘点：yxs 91/111、huicui 69/255、sixiang 45/181、yijiang 55/281、clan 25/90），每包按「整包八项验收」（37 来源台账 G0–G5）记录。
5. **N01/R01/R03 原生化**（PA12③）：onlyOL/shiji 相关原语（含 ABI 754–757 对应项）→ 升 `RULE_VM_ABI_VERSION` → 重打包 → 原生整局 + 与旧线差分 + 存档互恢复。
6. **主动技桥**（G）：另立 ADR（影响 PlayCard 候选与 UI 选择协议）。

## 4 每技能验收口径（原型已示范，作为模板）

1. 源：`content/original/ported/<pkg>/<skill>.skill` 原文 `include_str!`（不手写样本）。
2. 装载：`compile_source_named` → `bridge_vm_skill_decls` 零推迟、包 ABI ≤ 宿主。
3. 整局：真实触发时机（阶段/事件）、挂起选项集、每个分支落账；滤镜反例（不应触发时不触发）。
4. 恢复：挂起中 `save` → `restore_with_extensions` → 续行；同一快照各分支独立落账；原引擎不受影响。
5. 差分：旧线 `vm_host::run_module/resume_module` 对同一模块的挂起选项与效果种类/落账逐面相同（`crates/engine-core/tests/pa12_dsl_legacy_differential.rs` 模板）。
6. 上游：在规则基准登记该技能上游语义与两线一致性结论；差异写明。
7. 全量：engine-native 全量 + loader + 受影响产品面（game-server/wasm）；行数/链接/ADR 门禁。

## 5 工具

- 盘点脚本：[inventory.py](../operations/evidence/2026-10-26-pa12-dsl-native-prototype/inventory.py)（事件/滤镜/效果三面 + 包就绪率），随每批重跑更新本文表格。
- 原型测试：`crates/engine-native/tests/pa12_dsl_native_prototype.rs`、`crates/engine-core/tests/pa12_dsl_legacy_differential.rs`。
