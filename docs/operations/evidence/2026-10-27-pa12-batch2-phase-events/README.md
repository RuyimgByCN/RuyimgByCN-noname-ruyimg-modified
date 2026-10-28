# PA12 迁移批②：阶段/事件帧补齐（phase_begin 回合开始语义修正 + sha_dodged 映射）（2026-10-27）

- 执行时间：2026-10-27 **10:27:36 → ≤10:52（原记 11:00 晚于本批提交 10:52，2026-10-27 扫描按提交时间更正） +08:00**（`HEAD = 65f89334`，批①之后；用户已于 10:2x 裁决开批②）；执行者 ZCode A 道
- 工作包：[迁移计划](../../../architecture/native-dsl-migration-plan.md) 批②（阶段/事件帧补齐）；勘察结论先行——六候选事件中**只有两个无需新原生帧**即可正确承载，其余登记为内核缺口（见 §3）。

**一句话结论**：① DSL `phase_begin`（旧线 = `DomainEvent::TurnBegan`，**回合开始**，90 处）此前被映到原生「任一阶段开始帧」（每回合 5 次）——修正为 `PhaseBegin` + `AtPhase(Judge)`（原生宿主技能同惯例：Judge 的 begin 帧即回合开始，kernel/phase.rs）；② DSL `sha_dodged`（15 处）映射原生 `EventData::ShaDodged`，新增 `trigger_ctx_of` 臂（来源=杀的使用者、事件主玩家=闪避目标，批① 的 `owner_is_source` 滤名可直接承载）。真实技能全链验收：五脉（key，`phase_begin owner_is_target optional`）只在持有者自己回合开始开确认门、确认摸 1、每回合一次；蹙国（newjiang，`sha_dodged owner_is_source`）持有者的杀被闪避才摸 1，他人被闪避不触发。两模块旧线差分（DrawCards×1）一致。

## 1 事件语义核对（旧线 dispatch vs 原生帧）

| DSL 事件 | 旧线（engine-core dispatch） | 原生 | 本批结论 |
|---|---|---|---|
| `phase_begin` | `TurnBegan{player}` → `EventKind::PhaseBegin`（回合开始） | `Phase{Judge,begin:true}`（宿主惯例=回合开始） | **已映射修正**；登记差异：construction 置 `turn_opened=true`，**第一回合的 Judge begin 帧不发出**（首回合回合开始类技能不触发；修复需改 TurnState 初值→进 golden 哈希，须走版本门禁） |
| `sha_dodged` | `ShaDodged{source,target,card}` | `EventData::ShaDodged{card,source,target}` | **已映射** + ctx 臂（source→event_source） |
| `game_started`（96） | `GameStarted{player_count}` → 全体、amount=人数 | 无帧、**无开局调度点**（construction 不跑 resolution 循环） | 缺口：需新事件种（九处接线）+ 开局帧发出点 |
| `phase_prepare`（227） | `TurnStateChanged{Prepare}` → `PhasePrepare` | `Phase` 无 Prepare；kernel 无 prepare 机制（grep 零命中） | 缺口：需 `Phase::Prepare` 枚举扩展（内核决定） |
| `hand_compare_finished`（37） | `HandCompareFinished{player,player_card,target,winner}` | 拼点机器存在但无事件帧 | 缺口：新事件种 |
| `max_hp_changed`（7） | `MaxHpChanged{player}` | machine.rs 明注「无 MaxHpChanged 事件种」 | 缺口：新事件种 |
| `skill_added`（36） | `SkillAdded{player,skill}` | `SkillToggled{player,skill,enabled,temporary}` 载荷不同 | 缺口：且搭档滤镜 `owner_skill_added_self_*` 未承载（映射事件也跑不通） |

## 2 验收（真实 ported .skill，正式装载路径）

| 测试（engine-native `tests/pa12_batch2_phase_events.rs` 2/2；engine-core 差分 4/4） | 断言 |
|---|---|
| `pa12_batch2_phase_begin_is_turn_start_of_owner_only`（五脉） | 120 步推进内：`trigger_confirm` 只出现在 `(current==1, phase==Judge)`；两圈内 ≥2 次；确认后手牌 +1 |
| `pa12_batch2_sha_dodged_owner_is_source`（蹙国） | 0 号杀→1 号闪：1 号无摸牌（来源≠持有者）；1 号杀→2 号闪：1 号打出杀 -1 + 蹙国摸 1 = 持平 |
| `pa12_legacy_batch2_modules_draw_one` | 两模块旧线 `run_module` 各产出单条 `DrawCards`（效果面与原生一致） |

## 3 变更与回归

- `crates/content-native-loader/src/lib.rs`：`phase_begin`→PhaseBegin@Judge（注释含旧线依据与首回合差异）、`sha_dodged`→ShaDodged。
- `crates/engine-native/src/rvm_bridge.rs`：`trigger_ctx_of` 新增 `ShaDodged` 臂。
- 回归（[b2 汇总](../2026-10-26-pa12-dsl-native-prototype/b2-regressions.txt)）：engine-native、content-native-loader、engine-native-wasm、game-server、engine-core 全量 + `cargo check --workspace --all-targets` + 前端 test/typecheck + 行数/链接/ADR 门禁；盘点重跑：近似可直接原生 1,054→**1,056**（本批主要是**语义修正**，解锁量小）。
