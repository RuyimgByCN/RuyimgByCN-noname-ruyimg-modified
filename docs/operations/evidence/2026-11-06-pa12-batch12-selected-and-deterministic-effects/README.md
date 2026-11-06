# PA12 批⑫（预选卡牌/目标桥接、25 项确定性原语、具名滤镜激活与 3 类触发事件映射）核验记录（2026-11-06）

- **核验日期**：2026-11-06
- **源码基准**：`4467d6a38` 之上工作树（提交后记入 `docs/architecture/native-progress.md`）
- **版本不变量（零增号）**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`（NGSA 107）、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`NETWORK_PROTOCOL_VERSION = 8`（WS8）

## 1. 变更范围与模块拆分

1. **`crates/engine-native-kernel/src/rvm_ops` 子模块拆分（保持 Rust 有效行 `<= 1,000`）**：
   - 新增 `crates/engine-native-kernel/src/rvm_ops/codec_ext.rs`，承接标签 `71..=108` 的 `EffectRequest` 二进制编解码（迁入批⑪ `71..=83` 并新增批⑫ `84..=108`），使 `codec.rs` 有效行数由 986 行降至 ~815 行。
   - 新增 `crates/engine-native-kernel/src/rvm_ops/commit_ext.rs`，承接批⑫ 25 项确定性效果原语（tags `84..=108`）的状态落账，使 `commit.rs`（~921 行）与 `commit_combat.rs`（~511 行）均远低于 1,000 有效行上限。
2. **`VmAux` 与 `EngineVmHost` 预选卡牌/目标（`selected_cards` / `selected_targets`）管线**：
   - `crates/engine-native-kernel/src/state.rs`：`VmAux` 新增 `set_selected_cards`、`selected_cards`、`set_selected_targets`、`set_selected_target_seats` 与 `selected_targets` 读写辅助（复用 `vm_aux.piles` 临时键 `#sel_cards` / `#sel_targets`，回合结束 `clear_turn` 自动清理，零存档格式变更）。
   - `crates/engine-native-kernel/src/rvm_bridge.rs` 与 `crates/engine-native-kernel/src/rvm_bridge/machine.rs`：`EngineVmHost` 新增 `selected_cards: Vec<u32>` 与 `selected_targets: Vec<u32>` 并覆写 `VmHost::selected_cards` / `VmHost::selected_targets`；当主动技在 `Phase::Play` 窗口运行且 `selected_targets` 非空时，`event_source` 优先采用 `selected_targets[0]`，与旧线主动技单目标绑定语义对齐。
3. **25 项既有 ABI（`<= 759`）确定性 `EffectRequest` 原语接入（tags `84..=108`，108/108 往返闭合）**：
   - `84`: `DiscardCards { player, cards }`（`discard_selected`）：从角色手牌/装备区摘除指定实体牌移入弃牌堆，发射 `CardDiscarded`、`CardsDiscarded` 与 `CardsLost`。
   - `85`: `StoreSelectedInSkillPile { player, cards, skill }`（`store_selected_in_skill_pile`）：将 `cards`（若为空则回退首张手牌，如 `quanji.skill`）移入 `vm_aux.piles[(player, skill_key)]` 并发射 `CardsLost`。
   - `86`: `UseVirtualCardOn { player, kind, target }`（`use_virtual_card_on`）：按 `kind` 构造虚拟牌推入 `EventData::CardUsed { card: None, targets: vec![target] }`，并在 `kernel/intrinsic.rs`、`kernel/duel.rs` 与 `kernel/tricks.rs` 补齐虚拟【决斗】、【过河拆桥】、【顺手牵羊】、【五谷丰登】（`card == 0` / `card: None`）结算与弃牌守卫（`shizhan.skill`、`shuimeng.skill` 等）。
   - `87`: `RecastCards { player, cards }`（`recast_selected`）：弃置 `cards` 并令角色摸等量牌（`chongxin.skill` 等）。
   - `88`: `AddCardUsePolicySkill { player, card, policy_mask, skill_def }`（`make_current_card_unrespondable`）：按 `skill_def` 临时授技至回合末，记录 `card_policy` 标记，并在 `policy_mask & 1 != 0` 时标记当前用牌不可响应。
   - `89`: `DrawSelectedTargets { targets, count, skill }`（`draw_selected_targets`）：令 `targets` 中每名存活角色各摸 `count` 张牌（`songci.skill` 等）。
   - `90`: `QueueExtraTurn { player, check, skill }`（`queue_extra_turn`）：置位 `(player, "extra_turn_queued") = 1` 与 `extra_turn_check:{raw}`（`botu.skill` 等）。
   - `91`: `DisableEquipSlots { player, slots }`（`disable_all_equip_slots`）：置位 `disabled_equip_slots` 位掩码与各槽位 `equip_slot_disabled:{slot}`，并弃置对应槽位已装备牌（`zuici.skill` 等）。
   - `92`: `EnableEquipSlot { player, slot, source }`（`enable_equip_slot`）：清除指定装备槽禁用位与标记（`kengo_guidui.skill` 等）。
   - `93`: `GainShadowCards { player, count, skill }`（`gain_shadow_cards`）：令角色摸 `count` 张牌并累计 `shadow_cards_gained`（`bingshen.skill` 等）。
   - `94`: `GainTargetHandCard { player, skill }`（`gain_target_hand_card`）：由 `EngineVmHost::emit_effect` 将宿主目标注入 `skill` 字段，获得目标角色第一张手牌（`dchuichun.skill` 等）。
   - `95`: `AddBoundSkill { player, skill_def, bound_target }`（`add_bound_skill`）：授予 `skill_def` 并记录 `bound_target` 与 `bound_target:{raw}`（`refenxun.skill` 等）。
   - `96`: `GiveSelectedThenRecoverTarget { player, target, cards, amount, skill, ready_to_recover }`（`give_selected_then_recover_target`）：将 `cards` 交予 `target` 手牌，若 `ready_to_recover` 为真则令 `target` 回复 `amount` 点体力（`ziyuan.skill` 等）。
   - `97`: `DistributeShadowCards { player, count, skill }`（`distribute_shadow_cards`）：分发 `count` 张牌并累计 `shadow_cards_distributed`（`mobilezhubi.skill` 等）。
   - `98`: `PlaceHandCardToExpansion { player, skill }`（`place_hand_card_to_expansion`）：将角色首张手牌移入对应技能牌堆并发射 `CardsLost`（`dcxingwu.skill` 等）。
   - `99`: `RecordActiveTarget { player, target, skill }`（`record_active_target`）：记录 `active_target` 与 `active_target:{raw}`（`ol_wuqian.skill` 等）。
   - `100`: `ClearPhaseGiftTarget { player, skill }`（`clear_phase_gift_target`）：清除 `phase_gift_target` 与 `phase_gift_target:{raw}`。
   - `101`: `DealDamageBanActiveIfDying { source, target, amount, skill }`（`deal_damage_ban_active_if_dying`）：造成 `amount` 点伤害，若目标体力将降至 `<= 0` 则置位 `ban_active` 与 `skill_banned_turn`（`eu_xiaoguo.skill` 等）。
   - `102`: `DiscardSelectedDrawAllHandBonus { player, cards, skill }`（`discard_selected_draw_all_hand_bonus`）：弃置 `cards`，若覆盖弃置前全部非空手牌则额外多摸 1 张（`rezhiheng.skill`、`zhiheng.skill` 等）。
   - `103`: `DiscardSelectedRecordUniqueCategoryTarget { player, target, card, skill }`（`discard_selected_record_unique_category_target`）：记录弃牌类别位与目标并弃置 `card`（`pindi.skill` 等）。
   - `104`: `DiscardTargetCardGainMount { player, target, scope }`（`discard_target_card_gain_mount`）：在 `scope` 内取目标首张牌，若为坐骑牌则收入 `player` 手牌，否则弃置（`qiaomeng.skill` 等）。
   - `105`: `DrawForLifetimeCardActionDivisors { player, divisors }`（`draw_for_lifetime_card_action_divisors`）：统计整局累计用牌/打出次数（或本回合用牌数）可整除 `divisors` 中正除数的个数并摸等量牌（`jili.skill` 等）。
   - `106`: `RecoverSelectedTargets { source, targets, amount, skill }`（`recover_selected_targets`）：令 `targets` 中每名已受伤存活角色回复 `amount` 点体力（`olweiyang.skill` 等）。
   - `107`: `RevealTopGainIfColor { player, color_mask }`（`reveal_top_gain_if_color`）：亮出牌堆顶 1 张牌，颜色匹配 `color_mask`（`1 = 红`、`2 = 黑`）则收入手牌，否则置入弃牌堆（`pszhonghun.skill` 等）。
   - `108`: `UseVirtualCardOnSelectedTargets { player, kind, targets }`（`use_virtual_card_on_selected_targets`）：按 `kind` 对存活 `targets` 推入虚拟 `EventData::CardUsed`（单目标锦囊 `Duel`/`Dismantle`/`Snatch` 多目标时按逆序逐目标推入 LIFO 事件栈以保证正向结算序，如 `jylianhuan.skill`、`wn_dutan.skill`）。
4. **`NamedPredicate` 具名滤镜激活与 3 类触发事件映射**：
   - `crates/engine-native-kernel/src/rvm_named.rs`、`eval.rs`、`identity.rs`：激活 `TargetHandNonEmpty`、`TargetHandEmpty`、`HandOrEquipEmpty`、`CardEquipKind` 4 项既有 `Atom`，新增 `Atom::TargetWounded`（tag `115`）、`Atom::DrawPhaseSkipped`（tag `116`）、`Atom::MinHp`（tag `117`）及 `"hp_le_one"`（`Atom::HpBelow(2)`）、`"lifetime_card_action"`、`"target_alive_has_discardable_equip"`。
   - `crates/content-native-loader/src/lib.rs`、`crates/engine-native-kernel/src/event/damage.rs`、`crates/engine-native-kernel/src/kernel/judge.rs`、`crates/engine-native-kernel/src/resolution/trigger_order.rs`：映射 `"recover_before" => EventKind::Recover`、`"judgment_resolved" => EventKind::OrdinaryJudge`（仅当存在 VM 监听技时在 `JudgePhaseMachine` 判定生效窗发出 `OrdinaryJudge` 事件，保持宿主技轨迹与全部既有 golden 哈希零变化）与 `"phase_draw_skipped" => EventKind::DrawPhaseAfter`。

## 2. 首次失败与修复记录（按 AGENTS.md 分开登记）

1. **原生定向测试首跑失败（`pa12_batch12_selected_and_deterministic_effects` 首跑 2 过 7 败）与复审补充修复**：
   - 原因 A（主动技 `event_source` 缺省指向自身）：`VmMachine::run` 中主动技以 `EventData::Phase { player: owner, phase: Phase::Play, begin: true }` 启动，`trigger_ctx_of` 将 `event_source` 置为 `Some(owner)`，导致依赖 `event_source` 读取选中目标的主动技（`shizhan`、`ziyuan`、`refenxun`、`ol_wuqian`、`eu_xiaoguo`、`pindi`）把目标误作发动者自身。修复：当主动技处于 `Phase::Play`（`begin: true`）且 `selected_targets` 非空时，`event_source` 优先取 `selected_targets[0]`。
   - 原因 B（虚拟锦囊在 `intrinsic.rs`、`DuelMachine`、`DismantleMachine`、`SnatchMachine` 及多目标 `UseVirtualCardOnSelectedTargets` 的处理）：① `UseVirtualCardOn` 发出 `card: None` 的虚拟牌时，`intrinsic.rs` 原仅对实体牌启动 `DuelMachine`、`DismantleMachine`、`SnatchMachine`、`WuguMachine`，且 `DuelMachine::finish` / `DismantleMachine::step` / `SnatchMachine::step` 对 `card == 0` 调 `to_discard` 会将哨兵卡 `0` 推入弃牌堆；② `UseVirtualCardOnSelectedTargets` 对多目标（如 `wn_dutan.skill` 多目标虚拟【决斗】）若只发单帧 `targets: living_targets`，`DuelMachine` / `DismantleMachine` / `SnatchMachine` 仅取 `u.targets.first()` 会漏算其余目标。修复：`intrinsic.rs` 放行虚拟 `CardKind::{Duel, Dismantle, Snatch, Wugu}`，`DuelMachine` / `DismantleMachine` / `SnatchMachine` 仅在 `self.card != 0` 时入弃牌堆，`UseVirtualCardOnSelectedTargets` 对多目标 `Duel | Dismantle | Snatch` 按逆序逐目标推栈。
   - 原因 C（`judgment_resolved` 与 `DrawPhaseSkipped`）：`JudgePhaseMachine` 判定生效时补充对含 VM 监听技的对局发射 `EventData::OrdinaryJudge`（并将 `judge_card` 注入 `event_cards`），使 `tiandu.skill` 在判定结算后获得判定牌；`Atom::DrawPhaseSkipped` 补充检查 `vm_aux.flag(ep, "draw_phase_skipped") != 0`（`DrawPhaseAfter` 结算后 `turn.skip_draw` 已消费复位，由 `draw_phase_skipped` 标记承接），并由 `jiezi.skill` 在 `EventKind::DrawPhaseAfter` 正常触发。
   - 原因 D（测试种子前置条件）：`ol_wuqian.skill` 需预置暴怒标记 `marks[(0, 1252226867)] = 2`；`eu_xiaoguo.skill` 验证濒死禁发前需清除首轮 `once` 记录以隔离测试 `ban_active` 门禁。
   - 补跑结果：`cargo test --offline -p engine-native --test misc pa12_batch12_selected_and_deterministic_effects::` **9/9 全绿**。
2. **旧线差分测试首跑问题与修复（`pa12_dsl_legacy_differential`）**：
   - 现象：① `ZoneKind::Discard` 编译错误（应为 `ZoneKind::DiscardPile`）；② `jylianhuan.skill` 的 `lose_hp` 在 `rule-vm` 中产出 `DealDamage { is_lose_hp: true, .. }`；③ `tiandu.skill` 属于 `standard` 包（`bootstrap_match` 已装载），`module(&entry.module)` 回退为 `VerifiedModule::verify(pkg.modules[0].clone())`，且旧线 `vm_host` 原始效果名为 `GainEventCards`。
   - 修复后重跑：`cargo test --offline -p engine-core --test misc pa12_dsl_legacy_differential::` **12/12 全绿**。

## 3. 全量门禁核验结果

| 门禁项 | 命令 | 结果 |
|---|---|---|
| 行数门禁单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | `Ran 2 tests ... OK` |
| 行数门禁 | `python3 -B scripts/check-source-lines.py` | `9125 files <= 2000 lines; Rust non-test files <= 1000 effective lines` |
| 原生分层棘轮 | `python3 -B scripts/check-native-layering.py` | `0 upward references <= 0` |
| 批⑫原生整局测试 | `cargo test --offline -p engine-native --test misc pa12_batch12_selected_and_deterministic_effects::` | **9 passed; 0 failed** |
| 旧线 `vm_host` 差分测试 | `cargo test --offline -p engine-core --test misc pa12_dsl_legacy_differential::` | **12 passed; 0 failed** |
| 原生三 crate + loader 全量 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader` | **2,079 passed; 0 failed; 74 ignored**（三 crate `2,042/0/74`，loader `37/0/0`） |
| 独立 target Gate 1b `-D warnings` | `CARGO_TARGET_DIR=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/target/gate1b-check RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`（跑毕已清理 `target/gate1b-check`） | **0 warnings, exit 0** |
| WASM 目标检查 | `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` | **0 warnings, exit 0** |
| 全量 `preflight.py` | `python3 -B scripts/preflight.py` | **21/21 全部通过** |
