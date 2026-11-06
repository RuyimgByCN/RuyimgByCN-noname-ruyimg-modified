# C331（290/290 `HostBehavior` 源顺序注册与 F02/F04/F08/F10/F11–F16 续补）、NC10（静止态 `vm_aux` 脱敏、挂起续行深度门与 20 种子 AI 质量基线）、PA05（模拟 `submit` 双克隆消除与 `certainly_illegal` 扩面）及 NC07（`validate_core` 扩展引用校验与 `hash.rs` 穷尽解构守卫）核验记录（2026-11-06）

- **核验日期**：2026-11-06
- **源码基准**：`a09a6b1de` 之上工作树（提交后记入 `docs/architecture/native-progress.md`）
- **版本不变量（零增号）**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`（NGSA 107）、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`NETWORK_PROTOCOL_VERSION = 8`（WS8），全部 14 场 golden 重放哈希与 `content_lock_id` 零变化

## 1. 变更范围与模块拆分

1. **C331 `262/290 -> 290/290` `HostBehavior` 源顺序注册闭环**：
   - `crates/engine-native-kernel/src/vocabulary/host/metadata/source_order.rs`：将 `pub fn source_order(&self) -> Option<(Timing, i32, bool)>` 公开，注册全部默认 `(Timing::Normal, 0, true)` 触发变体、16 项翻译表相关 `order_selectable = false` 变体（`BossQingmu2x`/`3x`、`BossBaimang2x`/`3x`、`BossChiyan2x`/`3x`、`BossXuanlin2x`/`3x`、`BossBianshen3x`/`4x` 为 `-1000`，`JinlianzhuCancel`、`ShengkeCheck`、`ZhuWanzhongJi`、`SwdLexue`、`SwdShencaojieGuard`、`SwdShenmiguoWindow` 为 `0`）以及 12 项机制耦合变体（`SanjianWeaponDamage` `-25`、`SwdNigongAccumulate` `-25`、`SwdXuanyingDamage` `0`、`MbWeitong` `1000`、`MbQianlong` `600`、`Kunlunjing` `1000`、`GwentFengyinState`、`SwdFengyinState`、`Niepan`、`SwdShouyin`、`XianjianChengxin`、`XianjianShuiyun5`），实现 290 个带 `trigger_meta()` 的 `HostBehavior` 变体 **290/290 100% 显式源顺序注册**（`remaining.len() == 0`）。
   - `crates/engine-native-content/src/content/host/filtering/xianjian.rs`：移除 `HostBehavior::XianjianChengxin` 与 `HostBehavior::XianjianShuiyun5` 中多余的 `*player != owner` 限制，使仙剑橙心与水云五濒死自救与上游一致。
   - `crates/engine-native/tests/c331_source_order_phase5.rs`、`c331_host_source_metadata.rs`、`c331_source33_integration.rs`：更新 `DECLS`、`REMAINING_28`（现为空集 `&[]`）与 `LIULI_COMPAT_DELTAS`（补入 `SanjianWeaponDamage (-25, -30)`、`SwdNigongAccumulate (-25, 0)`），断言 `290/290` 全覆盖。

2. **C331 `F02`/`F04`/`F08`/`F10`/`F11–F16` 续补修复与证明测试**：
   - `crates/engine-native-kernel/src/state/core_ops.rs`（`F14`）：在 `kingdom_friends` 与 `declare_revealed_kingdom` 中增加国战君主（`jun_liubei`、`jun_zhangjiao`、`jun_sunquan`、`jun_caocao`）同势力结盟判定（即使此前记为 `KingdomIdentity::Ye`，同势力君主亮将后直接调用 `kingdom_friends` 或 `is_friend_of` 均视为同阵营，且保持 997 有效行 `<= 1,000`）。
   - `crates/engine-native/tests/c331_atomic_source_repair.rs` 与 `c331_review_roots.rs`：新增并通过 `c9f_f02_alias_host_definitions_and_modified_chain_child_restore`、`c9f_f04_shuchui_direct_virtual_sha_first_hit_then_second_hit_excluded_and_turn_renewal`、`c9f_f08_yinzei_mixed_hand_and_equip_rng_revokes_equip_skill_and_orders_children`、`c9f_f10_zlfanshi_first_second_third_hit_expiry_and_renewal`、扩展 `c9f_abi_nature_draw_and_source_are_real`（`F11` 火/雷双分支逐步字节恢复）、扩展 `c9f_duqu_other_enabled_holder_is_excluded`（`F15` 禁用 `boss_duqu` 不排除 + 启用别名 `SkillDef` 映射 `NativeDamageEffect::Duqu` 排除）、新增 `c9i_invocation_before_cancel_transitions_to_omitted_and_restores`（`F13`/`F16`）与 `c9i_kingdoms_jun_and_cross_mode_cuidui_matrix`（`F14`，直接断言 `kingdom_friends` 与 `source_is_enemy_of`）。

3. **NC10 静止态 `vm_aux` 脱敏、挂起续行深度门与 20 种子 AI 质量基线**：
   - `crates/engine-native/src/ai/redact.rs`：启用 `redact_traps = true`，在静止态根（`quiescent`）擦除非观察者的 `vm_aux.storage`（对手 `StorageSet` 键与非观察者座位前缀键）、`vm_aux.flags` 与 `vm_aux.drawn`，并仅对 `vm_aux.storage` 中的陷阱实体牌引用与 `selected_card` 执行牌 ID 重映射（避免非卡牌整数计数器被 `remap` 污染）。
   - `crates/engine-native/src/ai.rs`：在 depth-2 前瞻入口增加 `trial.open_choice().is_none()` 门控，防止跨挂起续行栈直接执行第二层前瞻。
   - `crates/engine-native-kernel/src/kernel/events.rs`：在 `commit_card_used` 中兼容嵌套 `use_virtual_card`（如【封印之蛋】连续发出虚拟【五谷丰登】与【决斗】时，牌堆顶借用的载体牌在【决斗】响应中被打入弃牌堆或被其他角色获得），允许 `u.use_as.is_some()` 时从 `ctx.core.discard` 或持有者手牌回收载体牌。
   - `crates/engine-native/tests/nc10_ai_information_boundary.rs`：新增非忽略门禁测试 `nc10_ai_quality_baseline_finishes_majority_with_zero_faults`（20 个种子 `1..=20`，Host + VM 混合包，`BoundedPlanner::new(32).with_policy_variant(1)`，断言 `sim_faults == 0`、`probe_faults == 0`、`sim_fault_log().is_empty()` 且完局率 `>= 50%`）与 `nc10_quiescent_storage_scrubs_opponent_entries_and_preserves_observer_counters`。

4. **PA05 模拟 `submit` 双克隆消除与 `certainly_illegal` 扩面**：
   - `crates/engine-native/src/engine/commands.rs`：当 `self.simulation == true` 时跳过 `submit` 内部的二次 `self.state.clone()`（`let prev = (!self.simulation).then(|| self.state.clone());`），通过 `finish_segment_in_place` 原地完成步进，同时完整保留权威会话（`!self.simulation`）的原子回滚语义。
   - `crates/engine-native/src/engine/legal_actions.rs`：扩展 `certainly_illegal` 保守快拒，覆盖 `GameAction::ActivateSkill`、`GameAction::RevealGeneral` 与 `GameAction::PlayCard`（`CardKind::Shan`、零目标 AOE/全局锦囊带目标、单目标锦囊目标数 `!= 1` 或指自己、【借刀杀人】目标数 `!= 2` 或含自己/重复目标），与 `apply_command.rs` / `play_card.rs` 准入前缀严格对齐。

5. **NC07 `validate_core` 扩展引用校验与 `hash.rs` 穷尽解构守卫**：
   - `crates/engine-native-kernel/src/snap/validation.rs`：在 `validate_core` 中补齐 `PlayerState`（`yuexing_pair`、`fuchou_marked`、`ctianfu_source`、`ctianfu_card`、`gouhunluo_source`、`jiguanyuan_cards`、`card_mods`、`shenyan_pending`、`yishan_lost`、`swdtianshu_from`、`swdtianshu_granted`、`songci_targets`、`hutian_deposit`、`hutian_owner`）、`TurnState`（`ciqiu_dying`、`kangyin2`、`huanxia_track`、`kunlunjing_snapshot`、`extra_phases`、`zhufangshenshi_target`、`liangou_hook`、`feitiao_target`、`zhuyue_targets`、`luanjian_card`、`minwan_targets`、`qizuo_copy`）与 `VmAux`（`marks`、`piles`、`lost`、`once`、`flags`、`hits`、`drawn`）的座位、卡牌、技能定义与阶段/作用域边界校验。
   - `crates/engine-native/src/engine/hash.rs` 与 `crates/engine-native/src/engine/hash/event.rs`：在 `canonical_hash` 与 `push_vm_aux` 中为 `PlayerState`、`TurnState` 与 `VmAux` 增加无 `..` 的编译期穷尽解构守卫，并将 `push_event_data` 抽出至子模块 `hash/event.rs`（保持 `hash.rs` 有效行远低于 1,000 行上限）。
   - `crates/engine-native/tests/nc07_state_same_source.rs`：扩展 `nc07_restore_rejects_out_of_range_references` 逐项验证新增 `PlayerState`、`TurnState` 与 `VmAux` 越界引用被 `Engine::restore` 拒绝。

## 2. 首次失败与修复记录（按 AGENTS.md 分开登记）

1. **`core_ops.rs` 有效行检查首跑（`1083 > 1000`）**：
   - 原因：`crates/engine-native-kernel/src/state/core_ops.rs` 原本已接近 1,000 有效行上限，新增君主辅助函数后达 1,083 有效行。
   - 修复：收敛复用 `is_revealed_jun`、`jun_group` 与 `faction_name` 辅助函数，并在 `kingdom_friends` 中兼容亮将后 `Declared(faction)` 的扩展君主，降至 997 有效行。
2. **`nc10_ai_quality_baseline_finishes_majority_with_zero_faults` 首跑种子 14 panic（`card 8 not in hand of 2`）**：
   - 原因：`CardKind::Fengyinzhidan`（`kernel/events.rs:257-277`）连续调用 `ctx.use_virtual_card(u.player, CardKind::Wugu, ...)` 与 `ctx.use_virtual_card(u.player, CardKind::Duel, ...)`；`use_virtual_card` 从牌堆顶借用 `card 8`（一张【杀】）放入 `u.player` 手牌以构造虚拟帧，随后【决斗】先结算时 `u.player` 将手牌中的 `card 8` 作为实体【杀】打出并移入弃牌堆，导致外层虚拟【五谷丰登】在 `commit_card_used` 提交时找不到手牌中的 `card 8`。
   - 修复：在 `commit_card_used`（`kernel/events.rs:71`）增加 `u.use_as.is_some()` 时从 `ctx.core.discard` 或持有者手牌回收载体牌分支，重跑 20 个种子全部 0 故障通过。
3. **`hash.rs` 有效行检查首跑（`1032 > 1000`）**：
   - 原因：加入 `PlayerState` 与 `TurnState` 穷尽解构守卫后 `hash.rs` 有效行达 1,032 行。
   - 修复：按职责将事件帧载荷哈希 `push_event_data` 拆分为 `crates/engine-native/src/engine/hash/event.rs` 子模块，重跑行数门禁通过。

## 3. 全量门禁核验结果

| 门禁项 | 命令 | 结果 |
|---|---|---|
| 行数门禁单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | `Ran 2 tests ... OK` |
| 行数门禁 | `python3 -B scripts/check-source-lines.py` | `9131 files <= 2000 lines; Rust non-test files <= 1000 effective lines` |
| 原生分层棘轮 | `python3 -B scripts/check-native-layering.py` | `0 upward references <= 0` |
| C331 专项测试集 | `cargo test --offline -p engine-native --test nc_rounds c331_` | **145 passed; 0 failed** |
| NC10 专项测试集 | `cargo test --offline -p engine-native --test nc_rounds nc10_` | **72 passed; 0 failed; 4 ignored** |
| NC07 专项测试集 | `cargo test --offline -p engine-native --test nc_rounds nc07_` | **27 passed; 0 failed** |
| 原生三 crate + loader 全量 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader` | **2,098 passed; 0 failed; 74 ignored**（三 crate `2,061/0/74`，loader `37/0/0`） |
| 独立 target Gate 1b `-D warnings` | `CARGO_TARGET_DIR=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/target/gate1b-check RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`（跑毕已清理 `target/gate1b-check`） | **0 warnings, exit 0** |
| WASM 目标检查 | `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` | **exit 0** |
| 全量 `preflight.py` | `python3 -B scripts/preflight.py` | **21/21 全部通过** |
