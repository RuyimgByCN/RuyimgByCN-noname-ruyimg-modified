# DSL/IR → 原生 生成迁移计划（PA12④，2026-10-27 初版）

> 本文是台帐 [PA12](native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26) ④ 的交付物：原型证据见 [PA12④ 原型](../operations/evidence/2026-10-26-pa12-dsl-native-prototype/README.md)。**批量迁移前须再请用户确认**（PA12 裁决条件④）；本文只定义批次、缺口与验收口径，不自动成为实施授权。规则/存档版本不随本文变化。

## 1 结论与路线

- **路线成立**：现有 ported `.skill`（旧线 DSL 产物）经 content-compiler → `content-native-loader::bridge_vm_skill_decls` → engine-native `VmMachine` 即可在原生整局运行，**不写新的原生 Rust 宿主代码**。原型用 sp 包真实技能 `olyinji`（隐绩：结束阶段体力非唯一最高时 `choose(1,2)` 回复 1 或上限 +1）完成：正式装载路径、触发、挂起、作答续行、效果落账、挂起中存档→恢复→续行（同一快照双分支）、与旧线 vm_host 对同一字节码的差分一致、上游语义（结束阶段、仅并列最高触发）核对。
- **原型暴露并修复了两处桥接缺陷**（随原型提交）：① `VmMachine::run` 在 Paused 分支置 `asked=false`，活引擎首答被丢、再问一次（恢复路径不受影响，故此前测试未发现）；② 装载器把 DSL `phase_end` 映到原生 `PhaseEnd`（任一阶段结束帧），而旧线 `phase_end` = `TurnStateChanged` 进入结束阶段——原生每回合触发 5 次。修法：`VmFilterPolicy::AtPhase` 阶段门 + 装载器阶段事件表（`phase_end`→PhaseBegin@End，`phase_{judge,draw,play,discard}`→PhaseBegin@X，`phase_{draw,play,discard}_end`→PhaseEnd@X）。
- **规模**（[盘点脚本输出](../operations/evidence/2026-10-26-pa12-dsl-native-prototype/inventory.txt)，2026-10-27 对 `content/original/ported/**/*.skill` 静态统计）：7,614 个 skill 声明；触发事件 49 种名、已映射 24 种；滤镜 707 个推迟名（批① 后 1,418 次触发使用；批① 前 2,358 次/708 名）；效果调用基础变体 2,782 次（原生 `accept_effect` 已允许）vs 宿主解析族 2,307 次 / 719 个名；文件级「事件已映射 ∧ 滤镜命中 ∧ 效果全基础 ∧ 无主动技」近似可直接原生 **1,054/7,614（批① 后；批① 前 579）**。

## 2 缺口盘点（按解决成本排序）

| 类 | 现状 | 代表（次数） | 处理 |
|---|---|---|---|
| A 命名对不上（零语义成本） | **批① 已完成（2026-10-27，[证据](../operations/evidence/2026-10-27-pa12-batch1-vocabulary/README.md)）**：`owner_alive` 新策略、`owner_is_source` 别名、`CardUsed` 补 `event_source`（旧线 source=用牌者，此前 `owner_is_source` 对用牌事件恒假）、不承载滤名装载期拒绝 | 剩余：`owner_source_sha`（76）等语义近似名 | 装载器别名/新增精确策略；每个补一条桥测 |
| B 阶段帧 | **批② 部分完成（2026-10-27）**：`phase_begin` 已核清并修正——旧线=`TurnBegan`（回合开始），原映「任一阶段开始帧」（每回合 5 次）改为 `PhaseBegin`+`AtPhase(Judge)`（原生宿主同惯例：Judge begin=回合开始）；**登记差异：construction 置 `turn_opened=true`，第一回合的 Judge begin 帧不发出——首回合（座位 0 第一回合）回合开始类技能不触发**（修复需改 TurnState 初值→进 golden 哈希，须版本门禁）。仍缺：`phase_prepare`（227，原生 `Phase` 无 Prepare——需加阶段枚举，内核决定）、`phase_any_end`（80） | `phase_prepare`、`phase_any_end` | 逐个核对+映射，每个一条真实技能整局测试 |
| C 事件名未映射 | **批② 已映射 `sha_dodged`（15，原生 `EventData::ShaDodged{card,source,target}` + 新增 `trigger_ctx_of` 臂：来源=杀使用者；`owner_is_source` 滤名已可承载）**。仍缺（原生无帧，需按[eventdata 接线清单]新增事件种）：`game_started`（96——需开局帧+**开局调度点**：construction 不跑 resolution 循环）、`hand_compare_finished`（37，拼点无原生帧）、`skill_added`（36，搭档滤镜 `owner_skill_added_self_*` 本身未承载）、`card_resolved`（35）、`global`（23）、`sha_resolved`（15）、`judgment_resolved`（9）、`round_started`（9）、`max_hp_changed`（7，machine.rs 明注无此事件种）、`debate_*`、`duel_resolved`、`cards_given`（4） | 上表 | 新事件种=九处接线（新事件种九处接线（本仓 LoseHp/DyingSaved 等先例）同法），与 NC04 未映射事件合并排批 |
| D 滤镜语义（需实现） | **批③已完成六无状态谓词**（owner_source_sha/owner_alive_not_target/owner_is_target_source_alive/owner_is_source_or_target/owner_source_fire_sha/owner_source_ordinary_trick；批① 已把「推迟」改为装载期拒绝）并修复 event_targets 桥接缺陷（HOST_EVENT_TARGET_COUNT 恒 0） | 剩余：`owner_is_target_once`（60）、`owner_alive_once_per_round`（38）等限次系——**勘正：`vm_aux.once`（座位×键×范围）机制已在、具名程序可承载**，**批③b（2026-10-27）已完成窗口验收**：任一回合/每轮两线对齐（良范/反转真实技能），修复 AtPhase 吞 Named consume 缺陷；遗留：取消时 once 是否消费两线未验（[批③b 证据](../operations/evidence/2026-10-27-pa12-batch3b-once-limits/README.md)）；`phase/use/damage`（34/16/12）为注释噪声（脚本下批跳过 // 行） | 按频次实现为精确策略/具名程序（`rvm_named`） |
| E 宿主解析原语 | **批④-1 `turn_over`（25 技能）、批④-2 `discard_hand_equip_count` N>0、批④-3 `prevent/modify_current_damage`（79 技能，DamageBefore 窗口直写）已接入（2026-10-27）**：允许面 + 内核翻面提交子帧（[证据](../operations/evidence/2026-10-27-pa12-batch4-1-turn-over/README.md)）；技能级重盘：Top-13 全部不在允许面——discard_hand_equip_count（80）、prevent/modify_current_damage（79）、choose_other_damage（48）、choose_use_virtual_sha（33）、discard_selected（23）、storage_set（22）…；SkipCurrentPhase/UseCard 无内核落点（rvm_ops 头注） | `discard_selected`（96）、`discard_hand_equip_count`（91）、`choose_other_damage`（68）… | 按包就绪率与频次排批；每个原语=原生 `accept_effect` 变体 + 内核臂 + 差分测试；ABI 号沿用 `ABI-COORDINATION.md` 领取（tag 空间已至 771） |
| F ABI/版本门 | `RULE_VM_ABI_VERSION` 753 < 部分包要求（onlyOL 757） | onlyOL/N01 | 宿主实现 754–757 对应原语后升号重打包（兼容判据对 host 单调） |
| I（批③发现）HOST_EVENT_CARD_COUNT 恒 0 | EngineVmHost.event_cards 未覆写（默认空切片）；event_targets 已在批③修复 | `event_card_count` 系技能待接线 | 与 game_started 开局帧一并评估 |
| J（批④-4 勘察登记）效果级选择窗缺失 | 旧线 `discard_hand_equip_count` N>0 实为「开窗选恰好 N 张、续行按所选弃置」（N=0 空操作；rule-api 文档注释「VM 首发开选牌」对本 DSL 原语不适用）；原生 VM 桥两层提交协议无「效果落地开窗挂起」的挂起点，批④-2 以确定性首 N 近似（玩家总选前 N 张时等价） | `discard_hand_equip_count`（80 技能）及其它「选哪几张/选谁」效果 | **效果级选择窗挂起基建**（连带 ChooseDiscardEquipmentPreventCurrentDamage、choose_other_damage 48、choose_use_virtual_sha 33 全族）——批④剩余的核心基建，落地前逐技能登记确定性近似差异 |
| H 再查询陈旧（批①发现） | 两线 VM 宿主都在模块 Halt 后落账，模块内 `hand_count` 等再查询看到执行前状态——「loop 内摸至 X」类 DSL 恒按循环上限执行（清励实证：两线同摸 5） | `clanqingli`（清励，摸至体力上限至多 5） | 需要「循环内即时落账」宿主原语或改写 DSL（先算张数再一次性摸）；逐技能排查同模式 |
| G 主动技/view_as | `active`/视作转化不走触发桥 | 偏置：sp 728 技能中大量主动技 | 需原生主动技桥（`active` → PlayCard 候选 + 选牌/选人） |

## 3 批次（每批独立验收、独立提交；**启动批量迁移前先请用户确认**）

1. **桥接词汇对齐（已完成 2026-10-27）**：别名 `owner_is_source`、新增 `owner_alive`、`CardUsed` 补事件来源、不承载滤名装载期拒绝（`UnsupportedVmFilters`）；滤镜命中 1,463→2,403（+940），近似可直接原生 579→1,054。[批①证据](../operations/evidence/2026-10-27-pa12-batch1-vocabulary/README.md)
1b. **滤镜谓词批③（已完成 2026-10-27）**：六无状态谓词 + event_targets 缺陷修复；滤镜命中 2,457→2,644、推迟 1,364→1,177、近似可直接原生 1,059→1,098（批③修正盘点脚本交替名计数后的基数）。[批③证据](../operations/evidence/2026-10-27-pa12-batch3-filter-predicates/README.md)
2. **阶段/事件帧补齐**（B + C 中原生已有事件）：`game_started`、`phase_prepare`（含原生准备帧决定）、`phase_begin` 核对、`hand_compare_finished`、`skill_added`、`max_hp_changed`。每个事件：一条含真实 ported 技能的原生整局测试 + 旧线差分。
3. **滤镜 Top-20**（D）：按频次实现，每个一条桥测；并把 `once`/`once_per_round` 落到原生限次机制。
4. **宿主解析原语 by 包**（E；④-1/②/③ 三原语族已接入）：下一优先 N=0 选牌首发链（VM 交互选牌，discard 族与 ChooseDiscardEquipmentPreventCurrentDamage 同链）、choose_other_damage（48）/choose_use_virtual_sha（33）选择挂起族；仍按包整包推进时改选 sp/mobile（技能级重盘：sp 142/mobile 106 个 ef 阻塞）（盘点：yxs 91/111、huicui 69/255、sixiang 45/181、yijiang 55/281、clan 25/90），每包按「整包八项验收」（37 来源台账 G0–G5）记录。
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
