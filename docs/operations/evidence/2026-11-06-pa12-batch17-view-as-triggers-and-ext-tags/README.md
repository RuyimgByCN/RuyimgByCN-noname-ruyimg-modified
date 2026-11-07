# PA12 批⑰（`compile_rejected` 清零 `7 -> 0`、`trigger_rejected` 清零 `62 -> 0`、`view_as_rejected` 清零 `48 -> 0` 及 60 项 `EffectRequest` 原语 tags `209..=254` + `ext_tag = 256..=269`）核验记录（2026-11-06）

- **核验日期**：2026-11-06
- **源码基准**：`ddba684c8` 之上工作树（提交后记入 [原生重写进度](../../../architecture/native-progress.md)）
- **版本不变量（零增号）**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`（NGSA 107）、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`NETWORK_PROTOCOL_VERSION = 8`（WS8）

## 1. 变更范围与模块拆分

1. **子模块拆分（保持 Rust 非测试有效行 `<= 1,000`、总行 `<= 2,000`）**：
   - [`crates/engine-native-kernel/src/rvm_ops/codec_ext3/batch17.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/codec_ext3/batch17.rs) 与 [`codec_ext3/batch17/rest.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/codec_ext3/batch17/rest.rs)：承接批⑰ 46 项单字节标签 `209..=254` 与 14 项 `tag = 255` 扩展标签 `ext_tag = 256..=269`（经 [`codec_ext3.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/codec_ext3.rs) `decode_ext_u16` 与 [`codec.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/codec.rs) 分派）的二进制编解码，实现 **268/268** 已接线 `EffectRequest` 变体 100% 往返闭合。
   - [`crates/engine-native-kernel/src/rvm_ops/accept_batch15/batch17.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/accept_batch15/batch17.rs)：承接批⑰ 60 项 `EffectRequest` 变体的准入校验。
   - [`crates/engine-native-kernel/src/rvm_ops/commit_batch17.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_batch17.rs) 与 [`commit_batch17/rest.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_batch17/rest.rs)：承接批⑰ 60 项 `EffectRequest` 原语的确定性权威落账。
   - [`crates/engine-native-content/src/content/vm_effects/batch17_resolvers.rs`](../../../../crates/engine-native-content/src/content/vm_effects/batch17_resolvers.rs)：实现批⑰交互式选牌／选目标／选分支状态机（`step` / `active_ask` / `validate`）。
   - [`crates/engine-core/tests/pa12_dsl_legacy_differential/resolvers.rs`](../../../../crates/engine-core/tests/pa12_dsl_legacy_differential/resolvers.rs)：从 [`pa12_dsl_legacy_differential.rs`](../../../../crates/engine-core/tests/pa12_dsl_legacy_differential.rs) 拆出批⑮～⑰结算器与视为技编译执行差分测试，使主测试文件由 2,004 行收敛至 1,638 行（`<= 2,000`）。

2. **Part A — `compile_rejected` 清零（`7 -> 0`）与 `trigger_rejected` 清零（`62 -> 0`）**：
   - **A.1 `add_skill_by_name` 跨文件技能 ID 確定性回退**：在 [`crates/content-compiler/src/typecheck.rs`](../../../../crates/content-compiler/src/typecheck.rs) 新增 `cross_file_skill_def_id(skill_name)`（内置 `chixueqingfeng2 => 3_863_274_410`、`rebiluan2 => 3_680_299_345`、`wangzun3 => 2_630_565_864`、`decadexushen2 => 1_919_822_977` 精确映射与 FNV-1a `u32` 回退），使 `sp/chixueqingfeng.skill`、`sp/lixia.skill`、`sp/olbiluan.skill`、`sp/ollixia.skill`、`sp/rebiluan.skill`、`sp/wangzun.skill`、`sp2/decadexushen.skill` 共 7 个跨文件授予伴随技能的 `.skill` 文件全部改写为 `add_skill(target, def_id)` 并通过编译（`compile_rejected == 0`）。
   - **A.2 放行单技能内 `active + damage_before`（无同技能 `damage` 冲突时）**：在 [`crates/content-native-loader/src/vm_active.rs`](../../../../crates/content-native-loader/src/vm_active.rs) 仅对同技能同时含 `EventKind::DamageBefore` 与 `EventKind::Damage` 的冲突组合拒装（保持 `c331_review_k4_active_damage_before_remains_explicitly_unsupported` 守卫不变），解锁 `bingshi/pot_sunchen.skill`、`bingshi/sp_zhonghui.skill`、`sp/xinfu_xionghuo.skill`、`tw/twzhenliang.skill`、`ddd/jianshuo.skill` 共 5 个 `.skill` 文件。
   - **A.3 逐 `VmTriggerProgram` 独立 `TriggerMeta`（`optional` / `priority` / `timing` / `order_selectable`）与 `choice_actor`**：在 [`crates/engine-native-kernel/src/catalog.rs`](../../../../crates/engine-native-kernel/src/catalog.rs)、[`rvm_bridge.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge.rs)、[`resolution.rs`](../../../../crates/engine-native-kernel/src/resolution.rs) 与 [`resolution/trigger_order.rs`](../../../../crates/engine-native-kernel/src/resolution/trigger_order.rs) 新增 `SkillBehavior::{window_trigger_meta, kind_trigger_meta}`，按当前触发窗口 `EventKind` 查询匹配 `VmTriggerProgram` 的独立 `TriggerMeta` 与 `choice_actor`（并在 [`vm_active.rs`](../../../../crates/content-native-loader/src/vm_active.rs) 仅拒装同事件 `a.meta.event == b.meta.event` 下 `(timing, priority, order_selectable)` 不一致的折入冲突），解锁全部 57 个跨事件混合可选性／确认主体的多触发技能文件（`trigger_rejected == 0`）。

3. **Part B — `view_as_rejected` 清零（`48 -> 0`）**：
   - 在 [`crates/engine-native-kernel/src/rvm_bridge/handlers.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/handlers.rs) 与 [`crates/content-native-loader/src/vm_view_as.rs`](../../../../crates/content-native-loader/src/vm_view_as.rs) 扩展：
     - `VmViewAsZone::{HandExpansion, HandEquipExpansion, GroupWei, None}`（`"hand_expansion"`、`"hand_equip_expansion"`、`"group_wei"`、`"none"`）；
     - `VmViewAsFilter::{CardYing, ColorNeShuangxiong}`（`"card_ying"`、`"color_ne_shuangxiong"`）；
     - 目标牌种 `"huogong" => CardKind::Huogong`、`"chuqibuyi" => CardKind::Chuqibuyi` 及动态类别目标 `"basic"`（展开为 `[Sha,FireSha,ThunderSha,IceSha,Shan,Tao,Jiu]`）、`"ordinary_trick"` 与 `"trick"`（展开为普通锦囊牌种序列）；
     - 多牌转化与前置代价字段：`min_cards`、`max_cards`、`same_suit`、`owner_hp_min_one_cards`、`owner_basic_uses_plus_one_cards`、`multi_card_bonus`（`VmViewAsMultiCardBonus`）、`cost`（`VmViewAsCost::{TurnOver, LoseHp1, Damage1, Recover1, DiscardCurrentHandEquip}`）、`mark_cost`、`post_draw`、`once_per_round`、`once_per_turn_per_branch`、`branch`、`next_branch`。
   - 在 [`crates/engine-native/src/engine/play_rules.rs`](../../../../crates/engine-native/src/engine/play_rules.rs)、[`play_card.rs`](../../../../crates/engine-native/src/engine/play_card.rs)、[`legal_actions.rs`](../../../../crates/engine-native/src/engine/legal_actions.rs)、[`crates/engine-native-kernel/src/kernel/events.rs`](../../../../crates/engine-native-kernel/src/kernel/events.rs)、[`kernel/combat_rules.rs`](../../../../crates/engine-native-kernel/src/kernel/combat_rules.rs)、[`kernel/sha/step.rs`](../../../../crates/engine-native-kernel/src/kernel/sha/step.rs) 与 [`snap/asks/shapes.rs`](../../../../crates/engine-native-kernel/src/snap/asks/shapes.rs) 接通多牌转化（自动补扣配对牌）、木牛流马扩展区牌摘除与无实体牌代价转化（`card == 0` 时执行翻面／失体力／扣标记等代价并生成虚拟牌结算），使 `view_as_rejected == 0`。

4. **Part C — 60 项 `EffectRequest` 原语（tags `209..=254` + `tag = 255` `ext_tag = 256..=269`）**：
   - **Family 1（`209..=214`）**：`209`: `ChooseCardUserOrAddTarget`、`210`: `ChooseCurrentCardUseEnhancements`、`211`: `ChooseDrawOrDiscardTargetCardRecordSha`、`212`: `ChooseOptionalDiscardTargetCard`、`213`: `ChooseUseVirtualCard`、`214`: `UseVirtualShaFromCardColorLockSourceIfNoSha`。
   - **Family 2（`215..=221`）**：`215`: `ChooseDiscardHandEquipThenRecover`、`216`: `ChooseDiscardHandEquipThenRecoverTarget`、`217`: `ChooseDiscardWeaponOrLoseHp`、`218`: `ChooseMoveTargetEquipmentOrDraw`、`219`: `ChooseOtherHandStoreSkillPile`、`220`: `ChooseOtherYinghun`、`221`: `ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw`。
   - **Family 3（`222..=229, 237..=238`）**：`222`: `ResolveDiliYuheng`、`223`: `ChooseOtherTargetsBySkippedPhaseCountDamage`、`224`: `GiveAllEquipmentChooseRecoverOrDamage`、`225`: `LoseHpDoubleCurrentDamageThenReplace`、`226`: `DrawThenOptionallyDistributeHandCards`、`227`: `ReplaceDrawWithDistinctSuitSelection`、`228`: `StartDebateAllOthers`、`229`: `StartDebateTargets`、`237`: `ForceOthersUseShaOnNearestOrLoseHp`、`238`: `TargetDiscardThenDuelIfNonShaAndHpGteOwner`。
   - **Family 4（`230..=236, 239..=244`）**：`230`: `ResolveBailingyun`、`231`: `ResolveJieliangGain`、`232`: `ResolveJsrgchaozheng`、`233`: `ResolveStarcuxiaWeiming`、`234`: `ResolveReanxu`、`235`: `ResolveRebingyi`、`236`: `ResolveRejizhi`、`239`: `ResolveHoufeng`、`240`: `ResolveHuzun`、`241`: `ResolveMalingli`、`242`: `ResolveZhugemengxue`、`243`: `ResolveTwyuyan`、`244`: `ResolveTwzhibian`。
   - **Family 5（`245..=254` + `ext_tag = 256..=269`）**：`245`: `ChooseAddSkillBoundCardTarget`、`246`: `ChooseDiscardSkillPileCardAddMarkedSkill`、`247`: `ChooseDiscardTargetCardResolveCurrentSha`、`248`: `ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus`、`249`: `ChooseDiscardTargetCardsRecoverSameCategory`、`250`: `ChooseGainCardFromEachOtherReplaceDraw`、`251`: `ChooseGainCardFromEachOtherThenTurnOver`、`252`: `ChooseGainOwnLostDiscardCardThisTurn`、`253`: `ChooseGainTargetCardAlternateZoneThenTargetDraw`、`254`: `ChooseGainTopCardsBySuit`、`256`: `ChooseRevealHandCardBindCategoryTarget`、`257`: `ChooseRevealOtherHandCardRememberCategory`、`258`: `ChooseTargetHandCardToPile`、`259`: `ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver`、`260`: `CityConsume`、`261`: `DiscardEquipmentPairWithArmor`、`262`: `DiscardOneOwnXumou`、`263`: `DiscardRememberedHandCards`、`264`: `DiscardTargetEquipmentDrawWeaponRange`、`265`: `ExtraResolveCurrentTrick`、`266`: `GainDrawCardsForLifetimeCardActionDivisors`、`267`: `GainHandCompareResultCard`、`268`: `GainHighestRankShaFromEventCards`、`269`: `MoveEquipmentThenDraw`（并在 [`rvm_bridge/machine.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/machine.rs) `EngineVmHost::emit_effect` 补齐批⑰全部携带 `skill: SkillId` 变体的 `self.ctx.skill_id` 回填）。

## 2. 首次失败、复审发现与修复记录（按 AGENTS.md 分开登记）

1. **`pa12_batch17_view_as_triggers_and_ext_tags` 首跑修复**：
   - 首跑 `pa12_batch17_multi_card_expansion_and_cost_gated_view_as_execution` 中 `luanji`（两张同花色手牌当【万箭齐发】）因两张实体牌为【杀】（`CardKind::Sha`），`play_rules.rs::play_kind` 仅在 `!raw_kind.is_sha_like() && raw_kind != CardKind::Le` 时尝试出牌阶段 `VmViewAsSpec` 转化；修复 `play_kind` 对多牌转化（`spec.required_cards(core, actor) > 1`）放行物理【杀】／【乐不思蜀】作为转化底牌后，`luanji` 双【杀】转【万箭齐发】及 `jiushi1` 翻面无牌转【酒】全部通过。
2. **全量回归首跑捕获的 3 处守卫与锚点对齐**：
   - ① `pa12_dsl_legacy_differential.rs` 追加批⑰测试后达 2,004 行（超出 2,000 行上限）且首版误写 `pkg.skills`（应为 `pkg.skill_index`）；拆出子模块 [`pa12_dsl_legacy_differential/resolvers.rs`](../../../../crates/engine-core/tests/pa12_dsl_legacy_differential/resolvers.rs) 并改为 `pkg.skill_index` 后通过。
   - ② `c331_package_schema::order_segment_binds_each_original_position_and_handler_even_with_duplicate_trigger_ids` 与 `nc08_trigger_order_counterexample::c331_review_k4_active_damage_before_remains_explicitly_unsupported`：在 [`vm_active.rs`](../../../../crates/content-native-loader/src/vm_active.rs) 新增 `validate_same_event_order_tuples`（拒绝同技能同一 `EventKind` 上 `(timing, priority, order_selectable)` 不一致的折入冲突）及同技能同时含 `active + DamageBefore + Damage` 的共享载荷窗口冲突守卫，两项守卫测试与 5,670 文件全量盘点同时 0 失败通过。
   - ③ `adr0786_j2_package_versions::shiji_old_lock_retains_published_bytes_and_new_full_package_rejects_missing_native_declarations`：批⑰清零 `trigger_index` 与 `view_as_index` 拒装后，`shiji 0.2.5` 整包拒装锚由 `"mixed trigger ordering/confirmation tuples"` 推进至 `modifier_index`（含未移植角色的整包 `INTENTIONAL_EMPTY_NOOP` 严格拒装：`"native VM modifier declarations are unsupported"`），同步断言后通过。
3. **复审捕获并修复的 3 处运行时边界与 1 处测试清理**：
   - ① `VmViewAsSpec::consume_and_record` 中 `multi_card_bonus`（`relonghun` 的 `up_to 2 same_suit`：`Damage1`、`Recover1`、`DiscardCurrentHandEquip`）在初版仅完成声明解析而未接通运行期消费与奖励生效：修复 [`rvm_bridge/handlers.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/handlers.rs)、[`kernel/events.rs`](../../../../crates/engine-native-kernel/src/kernel/events.rs) 与 [`kernel/dying.rs`](../../../../crates/engine-native-kernel/src/kernel/dying.rs)，当 `multi_card_bonus.is_some() && max_cards >= 2` 时自动配对消耗第 2 张同花色手牌／装备并触发额外 +1 伤害（`jiu_active`）、+1 回复（`vm_va_recover_bonus`）或弃置当前回合角色 1 张牌（同时保持 `nc08_upstream_jiuyuan` 源码依据子串不变）。
   - ② [`play_rules.rs`](../../../../crates/engine-native/src/engine/play_rules.rs) `play_kind` 初版以 `spec.required_cards(core, actor) > 1` 无条件将实体【杀】／【乐不思蜀】覆写为 `spec.as_kind`，导致持有 `luanji` 与两张同花色牌时无法将其中【杀】作为单目标普通【杀】打出：改为向 `play_kind` 传入 `targets`，仅在 `targets.is_empty()`（无目标出牌，物理【杀】／【乐】本身恒非法）或本回合常规出【杀】次数已用尽（`sha_used_play_phase`）时将【杀】／【乐】转化为 `spec.as_kind`，使单目标出【杀】与无目标 `luanji`／`relonghun` 转化共存。
   - ③ [`rvm_bridge.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge.rs) `VmSkillBehavior::confirmation_actor` 初版对含多个 `PhaseBegin`／`PhaseEnd` 子触发器的技能未按阶段门 `AtPhase` 区分 `choice_actor`：新增 `VmFilterPolicy::matches_event_phase(event)` 并在 `confirmation_actor` 优先匹配当前阶段对应的子触发器。
   - ④ 清理 [`pa12_dsl_legacy_differential.rs`](../../../../crates/engine-core/tests/pa12_dsl_legacy_differential.rs) 拆出 `resolvers.rs` 后残留的未引用 `fn module()` 辅助函数，消除 `engine-core` 测试目标 `dead_code` 警告。

## 3. Ported `.skill` 装载桥接盘点（非占位文件 100% 闭合）

- `content/original/ported/**/*.skill` 共 **5,670** 个文件：
  - **通过 `compile_source_named` + `bridge_vm_skill_decls`**：**3,641 / 5,670**（较批⑯的 `3,524` 净增 **+117** 个 `.skill` 文件；`3,641 + 2,029 = 5,670`，即除纯占位文件外的 **100%（3,641 / 3,641）** `.skill` 文件全部编译并桥接通过）。
  - **`compile_rejected`（编译期拒装）**：**0** 个（较批⑯的 `7` 清零）。
  - **`modifier_rejected`（`modifier_index` / `skill_index` 拒装）**：**0** 个。
  - **`active_rejected`（`active_index` 拒装）**：**0** 个。
  - **`trigger_rejected`（`trigger_index` 拒装）**：**0** 个（较批⑯的 `62` 清零）。
  - **`view_as_rejected`（`view_as_index` 拒装）**：**0** 个（较批⑯的 `48` 清零）。
  - **`bytecode_rejected`（字节码效果标签拒装）**：**0** 个。
  - **`INTENTIONAL_EMPTY_NOOP`（纯占位修饰文件）严格拒装**：**2,029** 个（含全部 1,540 个单技能空壳文件与 489 个多技能纯占位文件，0 漏放）。

## 4. 全量门禁核验结果

| 门禁项 | 命令 | 结果 |
|---|---|---|
| 行数门禁单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | `Ran 2 tests ... OK` |
| 行数门禁 | `python3 -B scripts/check-source-lines.py` | `9162 files <= 2000 lines; Rust non-test files <= 1000 effective lines` |
| 原生分层棘轮 | `python3 -B scripts/check-native-layering.py` | `0 upward references <= 0` |
| 批⑰ 原生整局与盘点测试 | `cargo test --offline -p engine-native --test misc pa12_batch17_` | **5 passed; 0 failed** |
| 旧线 `vm_host` 差分测试 | `cargo test --offline -p engine-core --test misc pa12_dsl_legacy_differential` | **17 passed; 0 failed**（含 `resolvers::pa12_batch17_resolvers_view_as_and_cross_file_skills_compile_and_execute`） |
| 原生三 crate + loader 全量 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader` | **2,120 passed; 0 failed; 74 ignored**（三 crate `2,083/0/74`，loader `37/0/0`） |
| 独立 target Gate 1b `-D warnings` | `CARGO_TARGET_DIR=/tmp/noname-gate1b-check RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets` | **0 warnings, exit 0**（临时目录已清理） |
| WASM 目标检查 | `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` | **exit 0** |
| 全量 `preflight.py` | `python3 -B scripts/preflight.py` | **21/21 全部通过** |
