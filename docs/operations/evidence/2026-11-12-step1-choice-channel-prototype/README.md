# Step 1：交互通道打样与扩展技能真实选择迁移指南（2026-11-12）

本文件归档执行序列第 1 步「交互通道打样（单独提交，先做）」的设计、3 个代表技能（`boss_chuanyun`、`boss_tianyun`、`dddxiaxing`）的真实玩家选择落地、零升号兼容性证明与后续扩展技能迁移配方。

---

## 1. 为什么采用 `HostBehavior + HostStage + ChoiceKind` 通道且无需升号

在冻结版本边界（`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`WS_PROTOCOL_VERSION = 8`）下，扩展技能接入真实玩家选择有两条候选路径：

1. **路径 A（在 `rule-vm` / `rvm_bridge` 内新增挂起选择）**：需要在 `RULE_VM_ABI_VERSION = 759` 中引入可中断的 VM 帧快照并修改 `NGSA 107` 的 `FrameSnapshot` 二进制编码，必然触发 ABI 与存档格式升号；
2. **路径 B（复用既有 `HostBehavior + HostStage + ChoiceKind` 通道，本次采用）**：
   - **二进制存档（`NGSA 107`）零变更**：[`resolution.rs`](../../../../crates/engine-native-kernel/src/archive/resolution.rs) 中 `MachineSnapshot::Host` 的 `behavior` 以 `u16` tag（`w.u16(b.tag())` / `HostBehavior::from_tag(r.u16()?)`）编码，`stage` 复用既有 `HostStage` 变体（`0=Init`、`1=Confirm`、`2=SelectCard`、`5=TargetChoice`、`14=Followup`），`prompt` 校验复用编译期静态表 `KNOWN_PROMPTS`（归档中仅存储 `has_pending` 布尔位，恢复时由 [`asks.rs`](../../../../crates/engine-native-content/src/content/host/asks.rs) 的 `active_ask_host` 重建 `PendingChoice`）；
   - **前端与联机协议（`WS_PROTOCOL_VERSION = 8`）零变更**：前端 [`BattleView.vue`](../../../../apps/web-client/src/components/battle/BattleView.vue) 与 `game-server` 已原生消费 `ChoiceView { seat, kind: ChoiceKind, prompt }` 并通过 `Command::Answer { selection }` 回传玩家选择或取消（`selection: vec![]`）；
   - **标准 14 场 Golden 哈希零变化**：[`host_skill_bindings.rs`](../../../../crates/content-native-loader/src/host_skill_bindings.rs) 仅将扩展包中的技能 ID（`"boss_chuanyun"`、`"boss_tianyun"`、`"dddxiaxing"`）映射到对应 `HostBehavior`，不影响标准包目录或 `c331` 源序。

---

## 2. 三个代表技能的真实选择状态机与上游对照

| 技能 ID | 代表交互类型 | 上游源码位置 | 原生 `HostBehavior` (Tag) | 状态机与 `ChoiceKind` 窗口 |
| --- | --- | --- | --- | --- |
| `boss_chuanyun` | 可选发动 + 选目标（条件门控 `target.hp > player.hp`） | `archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_skill.js:3806` | `HostBehavior::BossChuanyun` (`539`) | 结束阶段开始（`Phase::End`）若存在 `hp > player.hp` 的其他存活角色：发起 `ChoiceKind::SelectPlayers { candidates, min: 1, max: 1, allow_cancel: true }`（`prompt: "boss_chuanyun_target"`），取消则不造成伤害，选定目标则对其造成 1 点伤害 |
| `boss_tianyun` | 可选发动 + 选目标 + 随机火焰伤害与卸装 | `archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_skill.js:4085` | `HostBehavior::BossTianyun` (`540`) | 结束阶段开始（`Phase::End`）发起 `ChoiceKind::SelectPlayers { candidates: 全体其他存活角色, min: 1, max: 1, allow_cancel: true }`（`prompt: "boss_tianyun_target"`）：玩家可点取消（`vec![]`）不发动，或任选目标 A/B；选定后失去 1 点体力、经 `ctx.core.rng` 令目标随机受到 2~3 点火焰伤害（`Math.random()>0.4?2:3`）并弃置其装备区所有牌 |
| `dddxiaxing` | 选牌置牌堆顶 + 视为用酒/桃 + 0 卡转化技（`viewAs`） | `archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:59` | `HostBehavior::DddXiaxing` (`541`) | 每轮限一次（`dddxiaxing_round`，对应 `!player.hasSkill("dddxiaxing_used")`）且自身 `he` 区有牌时（持有「侠」标记不阻碍触发），在任意角色准备阶段开始（`Phase::Prepare`）或濒死（`EventKind::Dying`）时发起 `ChoiceKind::SelectCards { cards: he区牌, min: 1, max: 1, allow_cancel: true }`（`prompt: "dddxiaxing_card"`）：取消则不消耗本轮次数；选定卡牌则将其置于牌堆顶（`ctx.core.deck.push(picked)`），视为当前事件角色使用一张【酒】（准备阶段）或【桃】（濒死），并获得跨轮保留的「侠」标记（`dddxiaxing_xia = 1`）；持有「侠」标记期间可移去「侠」标记 0 卡视为使用一张【杀】或【闪】（`dddxiaxing_viewas`） |

---

## 3. AI 座位（`BoundedPlanner`）公开信息决策契约

在 [`crates/engine-native/src/ai.rs`](../../../../crates/engine-native/src/ai.rs) 中新增 `plan_extension_choice_action(&PolicyInput)`，由 `BoundedPlanner::choose_from_input` 在收到扩展技能 `prompt` 时调用：
- **严格只读公开视图（`PolicyInput`）**：仅访问 `input.view`（各座位公开 `hp`、`hand_count`、`equips`、公开明置身份 `role`）、`input.choice` 与自身手牌 `input.own_hand_kinds`；**绝不读取**未亮明身份或他人暗置手牌；
- **对齐上游 `ai` 函数启发式**：
  - `"boss_chuanyun_target"`：对齐上游 `get.damageEffect(target, player, player)`，仅在存在 `public_attitude < 0` 的敌方候选时优先选择体力较低者，若无敌方候选则返回 `vec![]` 取消发动；
  - `"boss_tianyun_target"`：对齐上游 `extension_skill.js:4094`（`if (player.hp <= 1) return 0`），当 `me.hp <= 1` 时返回 `vec![]` 取消发动，否则在 `public_attitude <= -3` 的敌方目标中优先选择装备较多者；
  - `"dddxiaxing_card"`：对齐上游 `skill.js:81-83, 139-145`，当受益角色非己方/友方（`public_attitude <= 0`）时返回 `vec![]` 取消，否则按 `5 - get.value(card)` 优先将低价值牌（如【杀】、装备）置于牌堆顶而保留【桃】。

---

## 4. 后续扩展技能迁移五步配方（供队列第 2 步批量复制）

1. **注册 `HostBehavior` 词表（`engine-native-kernel`）**：
   - 在 [`behavior.rs`](../../../../crates/engine-native-kernel/src/vocabulary/host/behavior.rs) 新增变体；
   - 在 [`tags.rs`](../../../../crates/engine-native-kernel/src/vocabulary/host/tags.rs) 与 [`tag_decode.rs`](../../../../crates/engine-native-kernel/src/vocabulary/host/tag_decode.rs) 分配连续 `u16` tag；
   - 在 [`behavior_list.rs`](../../../../crates/engine-native-kernel/src/vocabulary/host/behavior_list.rs)、[`keys.rs`](../../../../crates/engine-native-kernel/src/vocabulary/host/keys.rs)、[`metadata/common.rs`](../../../../crates/engine-native-kernel/src/vocabulary/host/metadata/common.rs)、[`metadata/source_order.rs`](../../../../crates/engine-native-kernel/src/vocabulary/host/metadata/source_order.rs) 登记键名、触发事件与优先级（并同步更新 [`lib.rs`](../../../../crates/engine-native/src/lib.rs) 的 `host_behavior_keys_bijective`、[`c331_source_order_phase5.rs`](../../../../crates/engine-native/tests/c331_source_order_phase5.rs) 的 `DECLS` 及 [`c331_source33_integration.rs`](../../../../crates/engine-native/tests/c331_source33_integration.rs) 的触发词表计数）；
   - 在 [`archive/resolution.rs`](../../../../crates/engine-native-kernel/src/archive/resolution.rs) 的 `KNOWN_PROMPTS` 登记新 `prompt` 字面量。
2. **实现触发过滤、挂起询问与步进状态机（`engine-native-content`）**：
   - 在 [`filtering/common.rs`](../../../../crates/engine-native-content/src/content/host/filtering/common.rs) 实现 `filter_host` 前置条件；
   - 在 [`asks.rs`](../../../../crates/engine-native-content/src/content/host/asks.rs) 的 `active_ask_host` 返回与当前 `HostStage` 完全一致的 `(SeatId, ChoiceKind, &'static str)`（确保中途 `save() -> restore()` 后能重建相同的 `PendingChoice`）；
   - 在 `content/host/step/*.rs` 实现 `step_host`，使用 `Flow::WaitChoice { player, kind, prompt, next_stage }` 挂起等待玩家 `Command::Answer`。
3. **绑定扩展加载器（`content-native-loader`）**：
   - 在 [`host_skill_bindings.rs`](../../../../crates/content-native-loader/src/host_skill_bindings.rs) 的 `lookup_native_host_behavior` 将技能 ID 映射到该 `HostBehavior`（自动覆盖 `decision34_*` 的 `VmSkillDecl` 回退）。
4. **接入 AI 公开决策（`engine-native`）**：
   - 在 [`ai.rs`](../../../../crates/engine-native/src/ai.rs) 的 `plan_extension_choice_action` 按上游 `ai` 函数补齐基于 `PolicyInput` 的启发式选择。
5. **六维真实对局测试与独立复核（`engine-native/tests/`）**：
   - 经真实对局阶段/事件推进（禁调 `debug_execute_effect`）覆盖：①正向触发、②条件不满足不触发、③选 A 与选 B 结果不同、④取消（`vec![]`）不发动或必选拒答、⑤AI 座位公开决策、⑥选择挂起中途 `save() -> restore() -> Answer` 状态哈希与日志序列完全一致；
   - 由独立复核 agent 对照上游源码逐行核验后，方可更新 [`inventory_status.json`](../../../migration/g0/inventory_status.json)。

---

## 5. 独立复核记录与逐项对照表（Reviewer: `step1-verifier`，2026-11-12）

独立复核员 `step1-verifier` 对照上游源码、原生代码与测试执行了两轮逐行审查：
- **首轮退回（Board `#17`）**：发现并退回 7 处与上游不等价或文档失实问题：① `boss_chuanyun` 误用 `hp >= owner_hp`（上游为严格 `player.hp < target.hp`）且误设 `allow_cancel: false`（上游 `chooseTarget` 未传 `true`，可取消），AI 在无敌方候选时未取消；② `boss_tianyun` 固定造成 2 点火伤（未调 `ctx.core.rng` 实现 `Math.random()>0.4?2:3`）且文档误写红桃判定；③ `dddxiaxing` 将所选 `he` 牌弃入弃牌堆而非置于牌堆顶（`ui.cardPile, "insert"`），受益效果误写为从牌堆摸【闪】/【桃】而非视为使用【酒】（`phaseZhunbeiBegin`）/【桃】（`dying`），且误将「侠」标记与当轮绑定并阻碍后续轮次触发。
- **次轮复核通过（Board `#18` 修复后）**：逐行核对修复后的原生代码、六维真实对局测试与全套门禁，确认 3 项代表技能与上游源码 100% 等价，更新 [`inventory_status.json`](../../../migration/g0/inventory_status.json)（`reviewer: "step1-verifier", review_date: "2026-11-12", status: "已复核等价"`）并重生成 G0 文档（全仓 `已复核等价 3 / 简化近似 4,421 / 缺失 0`）。

| 清单键 (`inventory_status.json`) | 上游源码 `file:line` 与核心语义 | 原生实现位置 | 真实事件路径六维测试 (`file:line`) | 独立复核结论 (`step1-verifier`) |
| --- | --- | --- | --- | --- |
| `L11:skills:boss_chuanyun@archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_skill.js:3806` | `trigger: { player: 'phaseEnd' }, direct: true`：`chooseTarget` 过滤 `player.hp < target.hp`（可取消）；`if (result.bool)` 对所选角色造成 1 点普通伤害；`ai` 为 `get.damageEffect(target, player, player)` | [`filtering/common.rs:896-911`](../../../../crates/engine-native-content/src/content/host/filtering/common.rs#L896-L911)、[`asks.rs:64-86`](../../../../crates/engine-native-content/src/content/host/asks.rs#L64-L86)、[`step/boss.rs:214-263`](../../../../crates/engine-native-content/src/content/host/step/boss.rs#L214-L263)、[`ai.rs:611-638`](../../../../crates/engine-native/src/ai.rs#L611-L638) | [`l11_taixu_boss_events.rs:353-512`](../../../../crates/engine-native/tests/l11_taixu_boss_events.rs#L353-L512) (`test_boss_chuanyun_authentic_lifecycle`)：覆盖正向触发、等血/低血不触发、选座 1 vs 座 2 分化、取消（`vec![]`）0 伤害、AI 选低血敌方与仅友方时取消、挂起中途 `save -> restore -> Answer` 状态哈希与日志一致 | **已复核等价**（`difference: "无"`） |
| `L11:skills:boss_tianyun@archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_skill.js:4085` | `trigger: { player: 'phaseEnd' }, direct: true`：可选目标（`allow_cancel: true`）；选定后 `player.loseHp()`，目标受 `Math.random()>0.4?2:3` 点火焰伤害并弃置装备区所有牌；`ai` 在 `player.hp <= 1` 时取消，否则选敌方并按 `countCards('e')/2` 加权 | [`filtering/common.rs:913-922`](../../../../crates/engine-native-content/src/content/host/filtering/common.rs#L913-L922)、[`asks.rs:87-104`](../../../../crates/engine-native-content/src/content/host/asks.rs#L87-L104)、[`step/boss.rs:266-344`](../../../../crates/engine-native-content/src/content/host/step/boss.rs#L266-L344)、[`ai.rs:639-678`](../../../../crates/engine-native/src/ai.rs#L639-L678) | [`l11_taixu_boss_events.rs:113-281`](../../../../crates/engine-native/tests/l11_taixu_boss_events.rs#L113-L281) (`test_boss_tianyun_authentic_lifecycle`)：覆盖正向失血+2~3 火伤+卸装、选座 1 vs 座 2 分化、取消不失血不伤敌、AI `hp <= 1` 取消与多装优先且不读暗牌、挂起中途 `save -> restore -> Answer` 状态哈希与日志一致 | **已复核等价**（`difference: "无"`） |
| `U01:skills:dddxiaxing@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:59` | 每轮限一次（`dddxiaxing_used` 于 `roundStart` 清除），在任意角色 `phaseZhunbeiBegin`（`dddxiaxing_begin`）或 `dying` 时若自身 `he` 区有牌，可选 1 张 `he` 牌置于牌堆顶（`ui.cardPile, "insert"`），视为当前事件角色使用【酒】（准备阶段）或【桃】（濒死）并获跨轮保留的「侠」标记；`dddxiaxing_viewas` 可移去「侠」标记 0 卡视为使用【杀】或【闪】 | [`filtering/common.rs:925-952`](../../../../crates/engine-native-content/src/content/host/filtering/common.rs#L925-L952)、[`asks.rs:105-144`](../../../../crates/engine-native-content/src/content/host/asks.rs#L105-L144)、[`step/skills_targets.rs:305-436`](../../../../crates/engine-native-content/src/content/host/step/skills_targets.rs#L305-L436)、[`activation/common.rs:483-510`](../../../../crates/engine-native-content/src/content/host/activation/common.rs#L483-L510)、[`d34_view_as.rs:191-214`](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_view_as.rs#L191-L214)、[`ai.rs:679-708`](../../../../crates/engine-native/src/ai.rs#L679-L708) | [`l11_taixu_boss_events.rs:618-865`](../../../../crates/engine-native/tests/l11_taixu_boss_events.rs#L618-L865) (`test_dddxiaxing_authentic_lifecycle`) & [`decision34_pack9_closure.rs:130-228`](../../../../crates/engine-native/tests/decision34_pack9_closure.rs#L130-L228)：覆盖选牌 A vs B 置牌堆顶、虚拟【酒】生效、跨轮保留「侠」标记及持标再次触发、0 卡转化虚拟【杀】清除「侠」标记、取消不耗本轮次数、无牌不触发、AI 友方选低价值牌与敌方取消、挂起中途 `save -> restore -> Answer` 一致 | **已复核等价**（`difference: "无"`） |

