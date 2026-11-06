# PA12 批⑭（通用 VM 静态修正器、主动技选牌／目标／限次扩面、12 项触发事件与 `choice_actor`、Top 25 泛型效果原语及 `tag = 255` 扩展头）核验记录（2026-11-06）

- **核验日期**：2026-11-06
- **源码基准**：`a37406628` 之上工作树（提交后记入 [原生重写进度](../../../architecture/native-progress.md)）
- **版本不变量（零增号）**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`（NGSA 107）、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`NETWORK_PROTOCOL_VERSION = 8`（WS8）

## 1. 变更范围与模块拆分

1. **子模块拆分（保持 Rust 非测试有效行 `<= 1,000`、总行 `<= 2,000`）**：
   - [`crates/engine-native-kernel/src/rvm_bridge/modifiers.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers.rs)：定义 `VmModifierRule`、`VmModifierTarget`（15 类修饰目标）、`VmModifierMode`（`Add`/`Override`/`Max`/`Min`/`Multiply`/`Veto`）、`VmModifierSelector`（43 类牌种／花色／类别选择器）与 `VmModifierValueSource`（73 类动态值来源，含 `OwnerHandEmpty`、`OwnerHealthy`、`OwnerHasHeartEquip`、`OwnerShaUsedZero`、`TaoSaveCancelBeyondDistance2`、`TaoSaveCancelNonDyingDuringOwnerTurn` 等逐项精确求值）及求值器。
   - [`crates/engine-native-kernel/src/state/core_ops_modifiers.rs`](../../../../crates/engine-native-kernel/src/state/core_ops_modifiers.rs)：从 [`core_ops.rs`](../../../../crates/engine-native-kernel/src/state/core_ops.rs) 拆出 `finish_computed_distance`、`attack_range`、`seat_distance_alive`、`effective_gender`、`hand_limit` 并接入 `VmModifierRule` 求值，使 `core_ops.rs` 降至约 650 有效行且保留 `MA_SHU` 源码锚点。
   - [`crates/content-native-loader/src/vm_modifiers.rs`](../../../../crates/content-native-loader/src/vm_modifiers.rs)：桥接纯修饰技、触发+修饰技与主动+修饰技，同时严格拒绝 `is_intentional_noop_modifier`（`modifier hand_limit add 0;` 占位，共 1,540 个 `.skill` 占位文件 100% 拒装）。
   - [`crates/engine-native-kernel/src/rvm_ops/codec_ext2.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/codec_ext2.rs)：承接标签 `129..=153` 二进制编解码，并预留 `tag = 255` 作为 `u16` 扩展标签前缀（`255 + u16::to_le_bytes`），无需 bumped `ARCHIVE_FORMAT_VERSION = 107`。
   - [`crates/engine-native-kernel/src/rvm_ops/commit_batch14.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_batch14.rs)：承接批⑭ 25 项 `EffectRequest` 原语确定性回退提交，使 `commit.rs`、`commit_ext.rs`、`commit_batch13.rs` 均远低于 1,000 有效行上限。
   - [`crates/engine-native-content/src/content/vm_effects/batch14_targets.rs`](../../../../crates/engine-native-content/src/content/vm_effects/batch14_targets.rs) 与 [`batch14_cards.rs`](../../../../crates/engine-native-content/src/content/vm_effects/batch14_cards.rs)：实现批⑭ 25 项 `EffectRequest` 原语的交互式 `validate`/`active_ask`/`step` 状态机及两阶段 `commit_state` 挂起续行。

2. **Part 1 — 通用 VM 静态修正器（`VmModifierRule`）**：
   - 在 [`catalog.rs`](../../../../crates/engine-native-kernel/src/catalog.rs) `SkillBehavior` 上新增 `fn vm_modifiers(&self) -> &[VmModifierRule] { &[] }`，由 `VmFilterPolicy::Declared` 与 `VmHandlerSet` 返回装载期编译的修饰规则。
   - 接入 `hand_limit`、`attack_range`、`computed_distance`（[`core_ops_modifiers.rs`](../../../../crates/engine-native-kernel/src/state/core_ops_modifiers.rs)）、摸牌阶段 `DrawCount`（[`kernel/draw.rs`](../../../../crates/engine-native-kernel/src/kernel/draw.rs)）、濒死【桃】目标限制 `DyingPeachTargetCancel`（[`kernel/combat_rules.rs`](../../../../crates/engine-native-kernel/src/kernel/combat_rules.rs)）、出【杀】次数与额外目标 `ShaUseLimit`/`ShaExtraTargets`/`ShaUseCancel`/`CardUseUnlimited`（[`engine/play_sha.rs`](../../../../crates/engine-native/src/engine/play_sha.rs)）以及用牌距离／目标／使用禁止 `CardDistanceUnlimited`/`CardTargetCancel`/`CardUseCancel`/`HandCardUseResponseCancel`（[`engine/play_rules.rs`](../../../../crates/engine-native/src/engine/play_rules.rs)、[`engine/play_card.rs`](../../../../crates/engine-native/src/engine/play_card.rs)）。

3. **Part 2 — 主动技选牌、目标与限次扩面（[`vm_active.rs`](../../../../crates/content-native-loader/src/vm_active.rs)、[`handlers.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/handlers.rs)、[`machine.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/machine.rs)）**：
   - `VmActiveLimits` 扩展支持 `once_per_game`、`uses_per_turn: Option<u8>`、`uses_per_play_phase: Option<u8>`、`uses_per_turn_source: Option<VmTurnUseSource>`（`OwnerExtraTimes`、`Mark(u32)`、`DamageDealtPlusOne`）、`selection: Option<VmActiveSelection>`（`Hand` / `HandEquip` 范围选牌及 `unique_category_in_turn` 同回合类别去重记录）与 `targets: Option<VmActiveTargets>`（多目标及 `lowest_hp`/`highest_hp`/`wounded`/`male`/`unlinked`/`linked`/`hand_nonempty`/`can_compare`/`has_equip`/`in_attack_range` 等目标谓词）。
   - [`state.rs`](../../../../crates/engine-native-kernel/src/state.rs) `VmAux` 新增 `once_count` 与 `inc_once`（复用既有 `once: BTreeSet<(u8, String, u8)>` 存储，`NGSA 107` 字节格式零改动），[`snap/validation.rs`](../../../../crates/engine-native-kernel/src/snap/validation.rs) 同步放行 `scope = 4`（`play_phase`）与 `machine_validation.rs` 对齐。

4. **Part 3 — 12 项剩余触发事件名、`event_view_as_skill_<u32>` 与 `choice_actor != Owner`**：
   - [`content-native-loader/src/lib.rs`](../../../../crates/content-native-loader/src/lib.rs)、[`rvm_bridge.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge.rs) 与 [`rvm_named.rs`](../../../../crates/engine-native-kernel/src/rvm_named.rs) 映射全部 12 项剩余触发事件名：`hand_compare_finished`、`card_resolved`、`sha_resolved`、`round_started`、`max_hp_changed`、`debate_resolved`、`duel_resolved`、`debate_started`、`equip_slot_disabled`、`hand_compare_started`、`round_ended`、`phase_any_skipped`，使 `content/original/ported/**/*.skill` 的触发事件与滤镜拒装数降为 **0**（`filter_rejected = 0`）。
   - `map_vm_filter` 支持 `"event_view_as_skill_"` 参数化前缀（`VmFilterPolicy::EventViewAsSkill(u32)`）。
   - 新增 `VmChoiceActor::{Owner, EventPlayer, EventSource}` 与 `SkillBehavior::confirmation_actor`，在 [`resolution.rs`](../../../../crates/engine-native-kernel/src/resolution.rs) `OptionalGate::step`、[`trigger_order.rs`](../../../../crates/engine-native-kernel/src/resolution/trigger_order.rs) 与 [`snap/asks.rs`](../../../../crates/engine-native-kernel/src/snap/asks.rs) 将可选触发确认询问路由至 `EventPlayer` / `EventSource`（对 `choice_actor == Owner` 保持 `OptionalGate.event` 原帧不变，既有 14 个 golden 哈希 100% 字节恒等）。

5. **Part 4 — Top 25 泛型 `EffectRequest` 原语（tags `129..=153` + `tag = 255` `u16` 扩展头）**：
   - `129`: `ChooseOtherDrawOrRecover`、`130`: `ChooseAddOrRemoveCurrentCardTarget`、`131`: `ChooseCompareSelectedHandCard`、`132`: `ChooseTargetsDamage`、`133`: `ChooseAddCurrentShaTarget`、`134`: `DrawThenGiveOne`、`135`: `ChooseHandCompare`、`136`: `ChooseOptionalTargetsDraw`、`137`: `ChooseTargetsDraw`、`138`: `GuanxingTop`、`139`: `ChooseLinkOther`、`140`: `ChooseOtherDrawOwnerLostHpTurnOver`、`141`: `ChooseOneFromSkillPileDiscard`、`142`: `ChooseAddCurrentShaTargetWithinDistance`、`143`: `ChooseRecoverTarget`、`144`: `ChooseTargetDrawToMaxHpCap`、`145`: `ChooseDiscardHandRange`、`146`: `ChooseDiscardHandEquipThenRemoveSelfSkill`、`147`: `ChooseHandCardsToSkillPile`、`148`: `ChooseDiscardEquipmentPreventCurrentDamage`、`149`: `ChooseGainDiscardCardByKind`、`150`: `ChooseAddMissingSkill`、`151`: `ChooseMoveFieldCardTo`、`152`: `JudgeMakeCurrentShaTargetUnrespondableOnRed`、`153`: `ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan`。

## 2. 首次失败与修复记录（按 AGENTS.md 分开登记）

1. **定向测试 `pa12_batch14_modifiers_actives_and_choices` 构造与首跑问题修复**：
   - 原因 A（`DamageWindow` 内 `OptionalGate` 事件解包与 `trigger_order` 绑定校验）：`DamageBefore`/`Damage` 帧在 `DamageWindow` 内层执行时 `frame.data` 为外层牌使用事件，需用 `crate::event::damage_event(&ctx.res.stack, frame)` 解析伤害来源/受伤害者；为保持既有 `choice_actor == Owner` 技能的快照与 golden 哈希零变化，仅当 `observer != owner` 时在 `OptionalGate.event` 存入 `actual_ev` 并在 `trigger_order.rs::validate_trigger_bindings` 放行 `*event == actual_ev`。
   - 原因 B（`snap/validation.rs` 中 `vm_aux.once` 的 `scope = 4` 校验）：`machine_validation.rs` 已允许 `scope` `1..=4`（`4 = play_phase`），而 `snap/validation.rs` 仍检查 `!(1..=3).contains(&scope)`，导致多频次 `play_phase` 主动技存读档报 `IdOutOfBounds { what: "vm_aux_once_scope", got: 4 }`；同步为 `!(1..=4).contains(&scope)` 后恢复通过。
   - 原因 C（`NGSA 107` 已知 prompt 表校验）：`DrawThenGiveOne` 与 `GuanxingTop` 初版使用未注册的 `"rvm_give_pick"` / `"rvm_guanxing_top"`，导致 `decode_archive` 报 `BadTag("prompt_unknown", 0)`；统一复用已注册的 `"rvm_choice"` 后存读档零升号通过。
   - 原因 D（`ChooseMoveFieldCardTo` 允许 `player == target`）：`shebian.skill` 调用 `choose_move_field_card_to(self, self)` 将其他角色场上的装备移至自己装备区；将场牌来源过滤修正为 `from != target` 并允许 `player == target` 后通过。
   - 原因 E（复核补强）：补齐 `ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan`（`dcwuhun`）死亡触发时不要求 `owner.alive` 且按判定牌非【桃】/【桃园结义】致死、`JudgeMakeCurrentShaTargetUnrespondableOnRed`（`tieji`）非界铁骑时按红色花色判定、`ChooseCompareSelectedHandCard`（`xianzhen`）消耗指定 `card` 并记录 `compare_win_turn = 1`、条件型修正器（`eval_card_target_canceled`、`eval_card_use_canceled`、`eval_dying_peach_target_canceled`、`HandLimitBaseMaxHp`）严格检查 `effective_value > 0` 及 30+ 项 `value_source` 独立求值、主动技 `unique_category_in_turn` 完结后更新 `unique_cat_mask:<skill>`。
2. **三 crate 全量首跑 3 处既有源码/拒装锚点同步**：
   - ① `adr0786_j2_package_versions::shiji_old_lock_retains_published_bytes_and_new_full_package_rejects_missing_native_declarations`：批⑭映射 `sha_resolved` 等剩余 12 项触发事件后，`shiji 0.2.5` 整包 `trigger_index` 全部通过，首个拒装锚点推进至 `active_index.selection`（`shameng`）。
   - ② `nc08_upstream_mashu::upstream_mashu_implemented_as_minus`：该测试直接读取 `core_ops.rs` 查找 `"MA_SHU"`；将 `computed_distance` 入口与 `MA_SHU` 保留在 `core_ops.rs`、后半段委托 `core_ops_modifiers.rs::finish_computed_distance` 后通过（`core_ops.rs` 仍仅约 650 有效行）。
   - ③ `nc08_upstream_paoxiao::upstream_paoxiao_bypass_difference_is_fixed`：该测试读取 `play_sha.rs` 首个 `"if !paoxiao"` 后 240 字符断言含 `"sha_used_play_phase"`；将 `vm_sha_limit == 0` 合并入 `(core.turn.sha_used_play_phase || vm_sha_limit == 0)` 条件块后通过。

## 3. Ported `.skill` 装载桥接盘点

- `content/original/ported/**/*.skill` 共 **5,670** 个文件（其中非 `shahai` 武将包目录 **5,660** 个）：
  - **通过 `compile_source_named` + `bridge_vm_skill_decls`**：**3,079 / 5,670**（较批⑬显著跃升）。
  - **`INTENTIONAL_EMPTY_NOOP`（`modifier hand_limit add 0;`）严格拒装**：**1,540** 个（0 漏放）。
  - **触发事件与滤镜拒装（`filter_rejected`）**：**0** 个。
  - **剩余拒装分布**：`modifier_index` 未覆盖复杂修饰 718 个、`view_as_index` 219 个、`active_index`/`skill_index` 复杂选牌约束 47 个、其他 67 个。

## 4. 全量门禁核验结果

| 门禁项 | 命令 | 结果 |
|---|---|---|
| 行数门禁单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | `Ran 2 tests ... OK` |
| 行数门禁 | `python3 -B scripts/check-source-lines.py` | `9139 files <= 2000 lines; Rust non-test files <= 1000 effective lines` |
| 原生分层棘轮 | `python3 -B scripts/check-native-layering.py` | `0 upward references <= 0` |
| 批⑭ 原生整局与盘点测试 | `cargo test --offline -p engine-native --test misc pa12_batch14_modifiers_actives_and_choices::` | **8 passed; 0 failed** |
| 旧线 `vm_host` 差分测试 | `cargo test --offline -p engine-core --test misc pa12` | **20 passed; 0 failed**（含 `pa12_batch14_legacy_vm_effects_match_native_expectation`） |
| 原生三 crate + loader 全量 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader` | **2,105 passed; 0 failed; 74 ignored**（三 crate `2,068/0/74`，loader `37/0/0`） |
| 独立 target Gate 1b `-D warnings` | `RUSTFLAGS="-D warnings" cargo check --offline --all-targets -p content-native-loader -p engine-native-kernel -p engine-native-content -p engine-native` | **0 warnings, exit 0** |
| WASM 目标检查 | `cargo check --offline --target wasm32-unknown-unknown -p engine-native-kernel -p engine-native-content -p engine-native -p engine-wasm` | **exit 0** |
| 全量 `preflight.py` | `python3 -B scripts/preflight.py` | **21/21 全部通过** |
