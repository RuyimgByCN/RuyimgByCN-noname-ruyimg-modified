# PA12 批⑯（`modifier_rejected` 清零 `718 -> 0`、`active_rejected` 清零 `47 -> 0` 及 30 项具名 `Resolve*` 效果原语 tags `179..=208`）核验记录（2026-11-06）

- **核验日期**：2026-11-06
- **源码基准**：`28098d620` 之上工作树（提交后记入 [原生重写进度](../../../architecture/native-progress.md)）
- **版本不变量（零增号）**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`（NGSA 107）、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`NETWORK_PROTOCOL_VERSION = 8`（WS8）

## 1. 变更范围与模块拆分

1. **子模块拆分（保持 Rust 非测试有效行 `<= 1,000`、总行 `<= 2,000`）**：
   - [`crates/engine-native-kernel/src/rvm_bridge/modifiers/selector.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/selector.rs) 与 [`modifiers/eval.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/eval.rs)：从 [`modifiers.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers.rs) 拆出 `matches_vm_modifier_selector`（含伤害属性选择器 `DamageNormal` / `DamageFire` / `DamageThunder`）及运行期聚合求值函数族，使三文件均远低于 1,000 有效行上限。
   - [`crates/engine-native-kernel/src/rvm_bridge/handlers/active_specs.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/handlers/active_specs.rs)：从 [`handlers.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/handlers.rs) 拆出 `VmSelectionZone`、`VmActiveSelection` 与 `VmActiveTargets` 定义及选牌／目标过滤逻辑。
   - [`crates/engine-native-kernel/src/rvm_ops/codec_ext3/batch16.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/codec_ext3/batch16.rs)：承接批⑯ tags `179..=208` 二进制编解码，与 `codec.rs`（`1..=70`）、`codec_ext.rs`（`71..=108`）、`codec_ext2.rs`（`109..=153` + `255` `u16` 扩展头）、`codec_ext3.rs`（`154..=178`）共同构成 **208/208** 允许效果 100% 往返闭合。
   - [`crates/engine-native-kernel/src/rvm_ops/commit_batch16.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_batch16.rs) 与 [`commit_batch16/rest.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_batch16/rest.rs)：承接批⑯ tags `179..=208` 共 30 项具名复合结算器的确定性权威落账。
   - [`crates/engine-native-content/src/content/vm_effects/batch16_resolvers.rs`](../../../../crates/engine-native-content/src/content/vm_effects/batch16_resolvers.rs)：实现批⑯ 30 项具名复合结算器的交互式选牌／选目标／选分支状态机（`step` / `active_ask` / `validate`）。

2. **Part A — `modifier_rejected` 清零（`718 -> 0`）**：
   - **单技能包伴随 `INTENTIONAL_EMPTY_NOOP` 子技放行与空 `#modifier` 过滤**：在 [`crates/content-native-loader/src/vm_modifiers.rs`](../../../../crates/content-native-loader/src/vm_modifiers.rs) 新增 `package_has_trigger_or_active(pkg)`（`pkg.character_index.is_empty() && (!pkg.trigger_index.is_empty() || !pkg.active_index.is_empty() || !pkg.view_as_index.is_empty() || pkg.modifier_index.iter().any(|m| !is_intentional_noop_modifier(m)))`），对含真实 trigger、active、view_as 或非占位 modifier 的单技能包跳过伴随 `modifier hand_limit add 0;` 占位子技（并在 `bridge_modifier_only_entries` 中跳过 `modifiers.is_empty()` 的空子技），同时保持全部 **2,029** 个纯 `INTENTIONAL_EMPTY_NOOP` `.skill` 文件及含未移植占位角色的整包 `.npack.json`（如 `onlyOL`）严格拒装。
   - **移除 `skill_index` 元数据误拒**：在 [`vm_modifiers.rs`](../../../../crates/content-native-loader/src/vm_modifiers.rs)、[`vm_active.rs`](../../../../crates/content-native-loader/src/vm_active.rs) 与 [`vm_view_as.rs`](../../../../crates/content-native-loader/src/vm_view_as.rs) 移除 `if skill.lord_only || skill.conversion || skill.non_gainable` 拒装检查。
   - **3 项伤害属性选择器**：`VmModifierSelector` 新增 `DamageNormal`、`DamageFire`、`DamageThunder` 并映射 `"normal" | "fire" | "thunder"`。
   - **补齐剩余 21 项 `VmModifierValueSource`**：新增参数化前缀 `AliveWoundedCountAtLeast(u8)`（`"alive_wounded_count_at_least_"`）、`QueriedPlayerMarkPositive(u32)`（`"queried_player_mark_positive_"`）及 19 项精确值来源（`AliveWoundedCountZeroOrAtLeastFour`、`QueriedPlayerLinkedFirstLivingSkillOwner`、`OwnerShaUsedExactlyOne`、`OwnerShaUsedExactlyTwo`、`OwnerQunOtherShu`、`OwnerUniqueMaxHand`、`OwnerTurnedOverDiffOther`、`QueriedPlayerLinkedOtherThanOwner`、`QueriedPlayerWoundedInOwnerAttackRangeOther`、`QueriedPlayerOtherThanOwner`、`QueriedPlayerJudgeNonemptyOtherThanOwner`、`QueriedSourceEnemyOwnerAllyTargetOtherThanOwner`、`OwnerHandGtOther`、`OwnerHandLtOther`、`OwnerHandEquipGteOther`、`OwnerShaDamageDealtZeroThisTurn`、`OwnerMaxHandBeforeSkillFiresThisTurn`、`OwnerLinkedDiffOther`、`OwnerTurnDamageDealt`）。
   - **补齐全部 `MOD_TARGETS` 与 5 类前缀族并接通内核战斗路径**：`VmModifierTarget` 新增 `ShaIgnoredHandLimit`、`CardIgnoredHandLimit`、`CardUseLimit`、`AttackRangeFixed`、`DamageAmount`、`DamageTakenAmount`、`DamageTakenCancel`、`IgnoreArmor`、`CardUseUnrespondable`、`ShaUnrespondable`、`DyingSavePeachUsable`、`CardUsableOnSelf`、`CardSuit`、`CardName`、`CardNature`、`CardNumber`、`CardUsable`、`CardTargetEnabled`、`CardSavable`、`CardRespondable`、`CardUnrespondable` 并映射 `content-compiler` `MOD_TARGETS` 全集及 `"card_usable_"` / `"card_target_enabled_"` / `"card_savable_"` / `"card_respondable_"` / `"card_unrespondable_"` 前缀族；在 [`core_ops_modifiers.rs`](../../../../crates/engine-native-kernel/src/state/core_ops_modifiers.rs) 实现 `ignored_hand_card_count`、`effective_hand_limit`、`effective_damage_amount`、`is_damage_taken_canceled`、`ignores_target_armor`、`is_card_use_unrespondable`，并联入 [`kernel/intrinsic.rs`](../../../../crates/engine-native-kernel/src/kernel/intrinsic.rs)、[`kernel/damage.rs`](../../../../crates/engine-native-kernel/src/kernel/damage.rs)（`commit_damage`）、[`kernel/sha/step.rs`](../../../../crates/engine-native-kernel/src/kernel/sha/step.rs)（`qinggang_pierce`）与 [`kernel/sha.rs`](../../../../crates/engine-native-kernel/src/kernel/sha.rs)（多目标次目标不可响应）。

3. **Part B — `active_rejected` 清零（`47 -> 0`）**：
   - 在 [`handlers.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/handlers.rs)、[`active_specs.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/handlers/active_specs.rs) 与 [`vm_active.rs`](../../../../crates/content-native-loader/src/vm_active.rs) 补齐：
     - 3 项前置条件：`owner_missing_skill`（`require_missing_skill: Option<u32>`，保留自指环路拦截）、`hand_limit_positive`（`require_hand_limit_positive: bool`）、`hand_missing_gaintag`（`require_hand_missing_gaintag: Option<u32>`）；
     - 选牌约束：`same_color`、`same_suit`、`distinct_suits`、`distinct_categories`、`sum_effective_number_exact`、`unused_category_by_skill_this_play_phase`、`unused_category_by_owner_history_this_turn`；
     - 目标约束与动态上限：`max_by_owner_mark`、`max_by_owner_lost_hp`、`max_by_owner_hp`、`match_selected_card_count`、`selected_cards_match_target_equip_count`、`has_gainable_equip`、`has_discardable_hand_equip`、`has_discardable_equip`、`has_gainable_hand_equip`、`hp_lte_owner`、`hp_gt_owner`、`hp_gt`、`hand_lt`、`owner_in_target_attack_range`、`can_use_on_owner`（`"sha" | "juedou"`）、`pair_hand_difference_lte_three`、`equipment_pair_with_armor`。

4. **Part C — 30 项具名 `Resolve*` 效果原语（tags `179..=208`）**：
   - `179`: `ResolveStddechong`、`180`: `ResolveDddYujin`、`181`: `ResolveDddZhenji`、`182`: `ResolveZhengjian`、`183`: `ResolveQiaobian`、`184`: `ResolveTwfeiyi`、`185`: `ResolveZhengsu`、`186`: `ResolveTwcaoang`、`187`: `ResolveTwcaozhao`、`188`: `ResolveTwchengpu`、`189`: `ResolveTwchenxun`、`190`: `ResolveTwchenzhen`、`191`: `ResolveTwyizhu`、`192`: `ResolveRuanji`、`193`: `ResolveQiaojian`、`194`: `ResolveZhupeilan`、`195`: `ResolveMbduanyang`、`196`: `ResolveMbrunwei`、`197`: `ResolveMbkuangxiang`、`198`: `ResolvePotsifeng`、`199`: `ResolvePotwanglie`、`200`: `ResolvePotchengpu`、`201`: `ResolvePottuntian`、`202`: `ResolvePotjixi`、`203`: `ResolvePotjiyu`、`204`: `ResolvePotnigu`、`205`: `ResolvePotfuan`、`206`: `ResolveDddGuanning`、`207`: `ResolveDddJianshuo`、`208`: `ResolveShenSimayi`（并在 [`rvm_bridge/machine.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/machine.rs) `EngineVmHost::emit_effect` 补齐 `179..=208` 的 `self.ctx.skill_id` 回填）。

## 2. 首次失败、复审发现与修复记录（按 AGENTS.md 分开登记）

1. **`pa12_batch16_modifiers_actives_and_resolvers` 构造与首跑问题修复**：
   - ① 首跑 `pa12_batch16_active_preconditions_selection_and_target_filters` 调用 `e.debug_place(0, "guohe")` 报 `unknown card name guohe`（标准目录锦囊牌名为英文键 `"dismantle"`），改为 `"dismantle"` 后通过。
   - ② 首跑 `pa12_batch16_ported_skill_inventory_zero_modifier_and_active_rejections` 捕获 1 项 `active_rejected`（`targets.can_use_on_owner = Some("juedou")`）、3 项 `view-as skill metadata`（`vm_view_as.rs` 中 `lord_only || non_gainable`）与 114 项 `MOD_TARGETS` 长尾修饰目标名未列入 `compile_modifier_entry`；补齐 `Some("sha" | "juedou")`、移除 `vm_view_as.rs` 元数据误拒并映射 `vocabulary.rs` 全部 `MOD_TARGETS` 后，`modifier_rejected == 0` 且 `active_rejected == 0`。
   - ③ 首跑 `ResolveZhengjian`（`stage: 0`）：`.skill` 传入 `event_player = self`（非 `PlayerId::NONE`），移除 `batch16_resolvers.rs` 中 `ResolveZhengjian { stage: 0, .. }` 的 `!event_player.is_some()` 限制后交互选角与「诤」标记（`2_446_053_157`）落账通过。
2. **全量首跑与独立复审修复记录**：
   - ① `pa12_batch14` / `pa12_batch15` / `pa12_batch16` 盘点测试与 `native_onlyol_general`（`onlyOL.npack.json`）：补齐 `package_has_trigger_or_active` 覆盖 `!pkg.view_as_index.is_empty() || pkg.modifier_index.iter().any(|m| !is_intentional_noop_modifier(m))` 并在 `bridge_modifier_only_entries` 跳过 `modifiers.is_empty()` 空子技，使 `key/hinata_qiulve.skill`、`huicui/llqshenwei.skill`、`sp/jugu.skill`、`shenhua/xueyi.skill`、`refresh/olhongyan.skill`、`tw/twshenxing.skill` 等 9 个含伴随占位子技的 `view_as`/非占位 `modifier` 多技能包全部桥接通过（`bridged_ok = 3,524 / 5,670`），同时保持全部 **2,029** 个纯 `INTENTIONAL_EMPTY_NOOP` `.skill` 文件及 `onlyOL.npack.json` 严格拒装。
   - ② `adr0786_j2_package_versions::shiji_old_lock_retains_published_bytes_and_new_full_package_rejects_missing_native_declarations`：批⑯补齐 `same_color` 等主动技选牌约束后，`shiji 0.2.5` 中的 `shameng` 通过 `active_index` 桥接，整包首个未桥接拒装锚点由 `"shameng"` 推进至 `"mixed trigger ordering/confirmation tuples"`（`trigger_index`），同步断言后通过。
   - ③ 战斗修饰器与批⑯结算器复审补齐：在 `kernel/damage.rs`（`commit_damage`）、`kernel/sha/step.rs`（`qinggang_pierce`）与 `kernel/sha.rs`（次目标 `advance_target`）接通 `is_damage_taken_canceled`、`effective_damage_amount`、`ignores_target_armor` 与 `is_card_use_unrespondable`；在 `batch16_resolvers.rs` 与 `commit_batch16.rs` 修复 `ResolveQiaobian` 按玩家所选 `picked_cards[0]` 弃置指定手牌（而非恒弃 `hand[0]`）；在 `rvm_bridge/machine.rs` 补齐 `179..=208` `skill` 回填使 `ResolveZhengsu`（`stage: 1`）结算后移除临时技能。

## 3. Ported `.skill` 装载桥接盘点

- `content/original/ported/**/*.skill` 共 **5,670** 个文件：
  - **通过 `compile_source_named` + `bridge_vm_skill_decls`**：**3,524 / 5,670**（较批⑮的 `3,315` 净增 **+209** 个 `.skill` 文件）。
  - **`modifier_rejected`（`modifier_index` / `skill_index` 拒装）**：**0** 个（较批⑭/⑮的 `718` 清零）。
  - **`active_rejected`（`active_index` 拒装）**：**0** 个（较批⑭/⑮的 `47` 清零）。
  - **`INTENTIONAL_EMPTY_NOOP`（纯占位修饰文件）严格拒装**：**2,029** 个（含全部 1,540 个单技能空壳文件与 489 个多技能纯占位文件，0 漏放）。
  - **剩余拒装分布**：`trigger_index` 混合触发排序/确认元组 62 个、`view_as_index` 多牌/动态转化 48 个、编译期未尽语法 7 个。

## 4. 全量门禁核验结果

| 门禁项 | 命令 | 结果 |
|---|---|---|
| 行数门禁单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | `Ran 2 tests ... OK` |
| 行数门禁 | `python3 -B scripts/check-source-lines.py` | `9154 files <= 2000 lines; Rust non-test files <= 1000 effective lines` |
| 原生分层棘轮 | `python3 -B scripts/check-native-layering.py` | `0 upward references <= 0` |
| 批⑯ 原生整局与盘点测试 | `cargo test --offline -p engine-native --test misc pa12_batch16` | **5 passed; 0 failed** |
| 旧线 `vm_host` 差分测试 | `cargo test --offline -p engine-core --test misc pa12_dsl_legacy_differential` | **16 passed; 0 failed**（含 `pa12_batch16_resolvers_compile_and_execute`） |
| 原生三 crate + loader 全量 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader` | **2,115 passed; 0 failed; 74 ignored**（三 crate `2,078/0/74`，loader `37/0/0`） |
| 独立 target Gate 1b `-D warnings` | `CARGO_TARGET_DIR=/tmp/noname-gate1b-check RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets` | **0 warnings, exit 0**（临时目录已清理） |
| WASM 目标检查 | `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` | **exit 0** |
| 全量 `preflight.py` | `python3 -B scripts/preflight.py` | **21/21 全部通过** |
