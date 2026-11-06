# PA12 批⑬（12 项高频交互式原语、7 项确定性原语）与 UP37（R01 涤仇／R03 英霸／R06 明节／R07 自固／R08 争功）核验记录（2026-11-06）

- **核验日期**：2026-11-06
- **源码基准**：`2aca85767` 之上工作树（提交后记入 `docs/architecture/native-progress.md`）
- **版本不变量（零增号）**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`（NGSA 107）、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`NETWORK_PROTOCOL_VERSION = 8`（WS8）

## 1. 变更范围与模块拆分

1. **子模块拆分（保持 Rust 非测试有效行 `<= 1,000`、总行 `<= 2,000`）**：
   - `crates/engine-native-kernel/src/rvm_ops/commit_batch13.rs`：承接批⑬交互原语回退提交与 tags `121..=128` 确定性提交，使 `commit_ext.rs` 保持远低于 1,000 有效行上限。
   - `crates/engine-native-content/src/content/vm_effects/batch13_targets.rs`：实现 tags `109..=114` 目标/分支选择交互状态机（`validate`、`active_ask`、`step`）。
   - `crates/engine-native-content/src/content/vm_effects/batch13_cards.rs`：实现 tags `115..=120` 选牌/多步弃牌与赠牌交互状态机（含两阶段 `commit_state` 挂起续行）。
   - `crates/engine-native-content/src/content/vm_effects/dcdichou.rs`：实现 tag `121` `ResolveDcdichou`（`action == 0` 废除装备栏并给其他角色挂仇标记、`action == 1/2` 弃带仇角色牌并按牌名字数授予本回合 `dcdichou_eff`、`action == 10` 结束阶段清除全场仇标记并对本回合带仇或消仇角色造成 1 点火焰伤害）。
2. **12 项高频交互式 `EffectRequest` 原语接入（tags `109..=120`）**：
   - `109`: `ChooseOtherDamage { player, amount, skill }`（`choose_other_damage`，68 处 `.skill` 调用，如 `huicui/anyong.skill`）：选择 1 名其他存活角色造成 `amount` 点普通伤害。
   - `110`: `ChooseTargetDraw { player, count, scope, skill }`（`choose_target_draw`，33 处调用，支撑 UP37-R06 `dcmingjie.skill`、UP37-R07 `dczigu.skill` 与 `yanyu.skill`）：按 `scope`（`Any`、`Other`、`SelfOnly`、`MaleOther`、`FemaleOther` 等全部变体）选择 1 名存活角色令其摸 `count` 张牌。
   - `111`: `ChooseOtherRecover { player, amount }`（`choose_other_recover`，12 处调用，如 `shiji/youyi.skill`）：选择 1 名已受伤其他存活角色令其回复 `amount` 点体力。
   - `112`: `ChooseDrawOrRecover { player, draw_count, recover_count, allow_cancel }`（`choose_draw_or_recover`，11 处调用，如 `shenhua/xinkuanggu.skill`）：已受伤时打开 `PickTag`（`0 = 摸牌`、`1 = 回复`），未受伤且不可取消时直接摸 `draw_count` 张牌。
   - `113`: `ChooseOtherAddMark { player, mark, delta, skill }`（`choose_other_add_mark`，12 处调用，如 `sp/kuanshi.skill`）：选择 1 名其他存活角色为其增减 `delta` 枚 `mark` 标记并记录 `LogEvent::MarkChanged`。
   - `114`: `ResolveHfjieying { player, target, grant_def, skill }`（`choose_other_add_skill`，8 处调用，如 `sp2/hfjieying.skill`）：选择 1 名其他存活角色授予 `grant_def` 技能定义。
   - `115`: `ChooseDiscardOneHandOrEquip { player }`（`discard_one_hand_or_equip`，13 处调用，如 `shiji/spmingshi.skill`）：选择并弃置自身 1 张手牌或装备区牌。
   - `116`: `DiscardBasicDrawEqual { player, skill }`（`discard_basic_draw_equal`，40 处调用）：选择弃置任意张手牌中的基本牌（可取消），并摸等量张牌。
   - `117`: `ChooseUseVirtualSha { player, ignore_distance }`（`choose_use_virtual_sha`，47 处调用，如 `sp/xiaoxi.skill`）：选择 1 名合法目标（可取消，支持无视距离）视作对其使用虚拟【杀】。
   - `118`: `ChooseOtherDiscardTargetCard { player, scope, skill }`（`choose_other_discard_target_card`，27 处调用，如 `sp/sijian.skill`）：两阶段交互（Stage 1 选 `scope` 内有牌的其他存活角色，Stage 2 选其 `scope` 内 1 张牌弃置）。
   - `119`: `ChooseDiscardTargetCard { player, target, scope }`（`choose_discard_target_card`，19 处调用，如 `shenhua/mengjin.skill`）：从 `target` 的 `scope`（`Hand`/`Equipment`/`HandAndEquipment`/`HandEquipmentAndJudge`）选择 1 张牌弃置。
   - `120`: `ChooseOtherGiveOneHandEquip { player, skill }`（`choose_other_give_one_hand_equip`，24 处调用，如 `shiji/fengjie.skill`）：两阶段交互（Stage 1 选自身 1 张 `he` 牌，Stage 2 选 1 名其他存活角色交予其手牌）。
3. **UP37 五项技能闭环（`R01` 涤仇、`R03` 英霸、`R06` 明节、`R07` 自固、`R08` 争功）与 7 项确定性原语（tags `121..=128`，128/128 往返闭合）**：
   - **UP37-R01（`dcdichou` `3304378096` + `dcdichou_eff` `2754298115`）**：接入 tag `121` `ResolveDcdichou`；`content-native-loader` 准入 `dcdichou_eff` 双修正器（`attack_range` 与 `sha_use_limit` 按 `owner_skill_mark` 加成），`engine-native/src/engine/play_rules.rs` 将持有者 `2754298115` 标记值计入攻击范围与出牌阶段【杀】次数上限；`rvm_bridge/machine.rs` 对带仇目标用牌时将 `event_player` 绑定至带仇目标，移除仇标记时同步置位 `dcdichou_mark_removed_this_turn`（闭合批⑤-4 登记的增量 17 差异）。
   - **UP37-R03（`yingba` `2187983153` + `yingba_mark` `3219705664` + `pinghe` `1394632613`）**：`content-native-loader/src/vm_active.rs` 准入单目标（`targets 1 1 other` + 可选 `max_hp_gt`）零选牌 `play_phase` 主动技与 `yingba_mark` 修正器（`card_target_cancel_if_other_user_card_use_count_lt_owner_mark`）；`engine-native/src/engine/play_rules.rs` 实现平定标记目标限制（其他角色每回合使用的前 `X` 张牌不能指定拥有 `yingba` 的角色，`X` 为其平定标记数 `3219705664`），并与 `pinghe.skill`（防止伤害、减 1 点体力上限、交出 1 张手牌并为伤害来源叠加平定标记）联合端到端验证。
   - **UP37-R06（`dcmingjie` `3692434408`）**：`content-native-loader` 映射 `"game_started" => EventKind::TurnBegin` + `VmFilterPolicy::GameStarted`（首轮 `round == 1` 且整局每名持有者仅触发一次），配合 `DrawCards` + tag `110` `ChooseTargetDraw` 与 `dying` 触发 `ChangeMaxHp(self, -1)` 完整闭环。
   - **UP37-R07（`dczigu` `2040043381`）**：单目标 0 选牌主动技桥接 + tag `110` `ChooseTargetDraw` 完整闭环。
   - **UP37-R08（`zzhenggong` `2283567045`）与 tags `122..=128`**：接入 `122` `RecordCurrentShaAllTargets`（说明：原任务描述中 tag `122` 笔误写为不存在于 `rule-api` 的 `SetCurrentCardIgnoreDistance`，核对 `crates/rule-api/src/effect/sha_resolution.rs` 与 `dispatch_resolve_twxiawei.rs:138` 确认为 `RecordCurrentShaAllTargets`）、`123` `TakeOneScopedCard`、`124` `DrawAllEventTargets`、`125` `RefreshSkill`、`126` `AddFirstMissingSkill`、`127` `DiscardEquipmentSlot`、`128` `FinishCurrentPhaseAbortResolution`。
   - **`rvm_named` 短语补充**：将 `"other_discardable_hand_equip"` 映射至 `Atom::CanDiscard(0)`，解锁 `sp/sijian.skill`（`owner_from_hand_empty_other_discardable_hand_equip`）。

## 2. 首次失败与修复记录（按 AGENTS.md 分开登记）

1. **定向测试 `pa12_batch13_interactive_and_up37_skills` 构造与首跑问题修复**：
   - 原因 A（`content-compiler` 中 `owner_skill_mark` 修正器 `value` 字段）：编译器对 `modifier attack_range add owner_skill_mark;` 产出的 `PackageModifierEntry.value` 为符号乘数 `1`（非 `0`），修复 `vm_active.rs` 准入条件为 `m.value == 1` 后 `dcdichou_eff` 准入通过。
   - 原因 B（`play_rules.rs` 中 `yingba` 技能 ID 常量）：`YINGBA_RAW` 修正为 `yingba.skill` 声明的 `2_187_983_153`，使平定标记目标限制生效。
   - 原因 C（`sijian.skill` 滤镜短语 `"other_discardable_hand_equip"`）：`rvm_named.rs::phrase` 原仅含 `"has_discardable_hand_equip"`，导致 `"other_discardable_hand_equip"` 回退单字拆解将 `"discardable"` 误作 `Atom::DiscardEvt`（要求 `trig.kind == 10` 而非 `CardsMoved` `trig.kind == 8`）。将 `"other_discardable_hand_equip"` 并入 `Atom::CanDiscard(0)` 后 `sijian.skill` 触发通过。
   - 补跑结果：`cargo test --offline -p engine-native --test misc pa12_batch13_interactive_and_up37_skills::` **9/9 全绿**。
2. **旧线差分测试 `pa12_dsl_legacy_differential` 首跑问题与修复**：
   - 现象：`yingba.skill` 的 `lose_hp` 在 `rule-vm` 中产出 `DealDamage { is_lose_hp: true, .. }`；`zzhenggong.skill` 的 `take_one_card(..., equip)` 产出 `TakeOneCard { equip_only: true }`（`hand_equip_judge` 作用域产出 `TakeOneScopedCard`）。
   - 修复：在 `pa12_batch13_legacy_vm_effects_match_native_expectation` 中同步校验 `yingba`（`DealDamage`）、`zzhenggong`（`TakeOneCard`）与 `take_one_scoped_card_probe`（`TakeOneScopedCard`），重跑 **13/13 全绿**。
3. **三 crate 全量首跑 3 处既有测试锚点同步**：
   - ① `pa12_batch11_zhuzhou_nscangjian_stdyibing_take_cards`：`rvm_bridge.rs` 中 `TakeOneHandCard` 的 `!from.is_some()` 恢复 `targets[0]` -> `event_player` -> `event_source` 优先级，避免 `Dying` 事件误取伤害来源。
   - ② `adr0786_j2_package_versions::shiji_old_lock_retains_published_bytes_and_new_full_package_rejects_missing_native_declarations`：`game_started` 映射后，`shiji` 整包首个未映射事件锚点推进至 `sha_resolved`。
   - ③ `pa12_batch4_1_host_primitives::pa12_batch5_4_dichou_predicates`：UP37-R01 闭合后，`dcdichou.skill` 整包准入通过（2 个 decl）且对带仇目标用牌后仇标记移除至 `0`，同时保留非法 `modifier_index` 拒装断言。

## 3. 全量门禁核验结果

| 门禁项 | 命令 | 结果 |
|---|---|---|
| 行数门禁单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | `Ran 2 tests ... OK` |
| 行数门禁 | `python3 -B scripts/check-source-lines.py` | `9130 files <= 2000 lines; Rust non-test files <= 1000 effective lines` |
| 原生分层棘轮 | `python3 -B scripts/check-native-layering.py` | `0 upward references <= 0` |
| 批⑬ + UP37 原生整局测试 | `cargo test --offline -p engine-native --test misc pa12_batch13_interactive_and_up37_skills::` | **9 passed; 0 failed** |
| 旧线 `vm_host` 差分测试 | `cargo test --offline -p engine-core --test misc pa12_dsl_legacy_differential::` | **13 passed; 0 failed** |
| 原生三 crate + loader 全量 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader` | **2,088 passed; 0 failed; 74 ignored**（三 crate `2,051/0/74`，loader `37/0/0`） |
| 独立 target Gate 1b `-D warnings` | `CARGO_TARGET_DIR=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/target/gate1b-check RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`（跑毕已清理 `target/gate1b-check`） | **0 warnings, exit 0** |
| WASM 目标检查 | `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm && cargo check --offline --target wasm32-unknown-unknown -p engine-wasm` | **exit 0** |
| 全量 `preflight.py` | `python3 -B scripts/preflight.py` | **21/21 全部通过** |
