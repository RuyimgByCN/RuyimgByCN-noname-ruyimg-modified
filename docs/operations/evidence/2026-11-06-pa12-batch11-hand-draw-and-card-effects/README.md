# PA12 批⑪（手牌调整、底牌摸取、目标排除、单张取牌与技能禁用/压制原语接入）核验记录（2026-11-06）

- **核验日期**：2026-11-06
- **源码基准**：`710b38566` 之上工作树（提交后记入 `docs/architecture/native-progress.md`）
- **版本不变量（零增号）**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`（NGSA 107）、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`NETWORK_PROTOCOL_VERSION = 8`（WS8）

## 1. 变更范围与模块拆分

1. **`crates/engine-native-kernel/src/rvm_bridge` 模块拆分**：
   - 新增子模块 `crates/engine-native-kernel/src/rvm_bridge/context.rs`，抽出 `VmTriggerCtx`、`kind_is_ordinary_trick`、`trigger_ctx_of` 与 `filter_view`，使 `crates/engine-native-kernel/src/rvm_bridge.rs` 有效行数降至 1,000 行上限以内。
2. **13 项既有 ABI（`<= 759`）确定性效果原语接入（tags `71..=83`）**：
   - `crates/engine-native-kernel/src/rvm_ops.rs`（`accept_effect`）、`crates/engine-native-kernel/src/rvm_ops/codec.rs`（tags `71..=83` `enc_effects` / `dec_effects`）与 `crates/engine-native-kernel/src/rvm_ops/commit_combat.rs` 同步放行并落账：
     1. `AdjustHandTo { player, count }`（tag `71`）：将目标存活角色手牌数调整至 `count`——手牌不足时补摸 `count - hand.len()` 张，超出时从手牌首起弃置 `hand.len() - count` 张（`natulie.skill` 等）。
     2. `DrawByPlayPhaseCardCategoryCount { player, skill }`（tag `72`）：按本回合出牌阶段已使用过的牌类别位掩码（`play_cat_mask`：基本牌 `1`、锦囊牌 `2`、装备牌 `4`）的 1-bit 个数摸牌（`dcchenyong.skill` 等）。
     3. `DrawEachBySeat { first, second, count, skill }`（tag `73`，`count > 0`）：令两名存活角色按当前回合角色起的逆时针座次各摸 `count` 张牌（`wangxi.skill` 等）。
     4. `SetDrawPhaseFromBottom { player }`（tag `74`）：置位 `(player, "draw_phase_bottom") = 1`，令当前摸牌阶段 `DrawPhaseMachine` 改为从牌堆底（`deck[0]`）依次摸牌，摸牌阶段结算后自动清理（`nzry_cunmu.skill` 等）。
     5. `DrawCardsFromBottom { player, count }`（tag `75`，`count > 0`）：直接从牌堆底（`deck[0]`，牌堆空时洗混弃牌堆）摸 `count` 张牌并发射 `EventData::CardsDrawn` 与 `LogEvent::CardsDrawn`（`duchi.skill` 等）。
     6. `CancelIncomingCardUse { player, source, card, skill }`（tag `76`，`card != NONE`）：从当前栈顶 `EventData::CardUsed` 目标集中移除 `player`，若移除成功且实体牌 `card` 位于处理区 `core.table` 或 `core.discard`，将其收入 `player` 手牌并记 `LogEvent::CardsDrawn`（`danlao.skill` 等）。
     7. `ExcludeSavedShaTarget { card, target }`（tag `77`，`card != NONE`）：从当前 `EventData::CardUsed` 目标集中移除 `target`；若移除后目标集为空，将处理区 `core.table` 中的实体牌移入弃牌堆。
     8. `TakeOneCard { from, to, equip_only }`（tag `78`）：当 `equip_only == false` 时优先获得来源存活角色第一张手牌，否则获得其第一张装备牌并触发 `revoke_equip_skills` 与 `EventData::CardsMoved`（`zhuzhou.skill`、`nscangjian.skill` 等）。
     9. `TakeOneHandCard { from, to }`（tag `79`）：获得来源存活角色第一张手牌；当字节码缺省 `from == NONE`（`take_one_event_target_hand`）时，由 `EngineVmHost::emit_effect` 从事件首个目标或 `event_player` 自动补齐（`stdyibing.skill` 等）。
     10. `DisableSkillByDefUntilTurnEnd { player, skill_def }`（tag `80`，`skill_def != NONE`）：置位 `skill_disabled_raw:{raw}` 并将对应 `SkillInstance.enabled` 置 `false`（记录 `skill_disabled_turn:{def}`），回合结束自动恢复 `enabled = true`（`wxdlqianqian.skill` 等）。
     11. `BanActiveUntilTurnEnd { player, skill }`（tag `81`，`skill != NONE`）：置位 `skill_banned_turn:{executing}`，封禁该技能本回合主动发动与触发过闸，回合结束清理。
     12. `SetNonLockedSkillSuppression { player, enabled, skill }`（tag `82`）：置位或清除 `(player, "non_locked_suppression_source")`，回合结束兜底清理（`rewansha.skill` 等）。
     13. `SuppressNonLockedSkillsUntilTurnEnd { player }`（tag `83`）：置位 `(player, "suppress_non_locked_turn") = 1`，回合结束清理。
3. **处理区查询/摘牌、出牌类别历史与具名滤镜修复**：
   - `crates/engine-native-kernel/src/rvm_ops/query.rs`：`locate` 为处理区 `core.table` 返回 `CardZone == 6`（`Processing`）；`QueryRequest::HasSkill` 在 `skill_disabled_raw:{raw}` 置位时返回 `Bool(false)`。
   - `crates/engine-native-kernel/src/rvm_ops/commit.rs`：`detach` 支持从 `core.table` 摘牌；`MoveCards` 移入手牌（`kind == 3`）补记 `LogEvent::CardsDrawn`（`dcjianxiong.skill` 等）。
   - `crates/engine-native-kernel/src/kernel/events.rs`、`crates/engine-native-kernel/src/kernel/events/turn.rs`、`crates/engine-native-kernel/src/rvm_bridge.rs`、`crates/engine-native-kernel/src/rvm_named/eval.rs`：`record_vm_history` 对出牌阶段 `CardUsed` 与 `Equipped` 记录 `play_cat_mask`（基本 `1` / 锦囊 `2` / 装备 `4`），回合末清理；`OwnerPlayPhaseBteUsed` / `OwnerAlivePlayPhaseBteUsed` 与 `Atom::BtePos` 支持 `phase_end`（进入 `Phase::End`）及 `play_cat_mask > 0`。
   - `crates/engine-native-kernel/src/rvm_named.rs`、`crates/engine-native-kernel/src/rvm_named/eval.rs`、`crates/engine-native-kernel/src/rvm_named/identity.rs`：修正 `Atom::InTargets`（当 `v.targets` 非空时仅检查 `v.targets.contains(&owner)`，不再误将用牌者 `event_player` 当作目标），新增 `Atom::TargetEquipNonEmpty`（`"target_alive_has_equip"`）与 `Atom::SourceNotTarget`（`"other_alive_per_damage_point"`）。

## 2. 首次失败与修复记录（按 AGENTS.md 分开登记）

1. **测试编译错误 1（`pa12_batch11_hand_draw_and_card_effects.rs` 主动技动作签名）**：
   - 现象：`GameAction::ActivateSkill` 传入不存在的 `cards`/`targets` 字段且 `skill` 传字符串。
   - 修复：按 `GameAction::ActivateSkill { skill: u16 }` 传入 `e.catalog().skill_by_name("aux_ban_active_and_suppress").unwrap()`。
2. **原生定向测试首跑失败（`pa12_batch11_hand_draw_and_card_effects` 首跑 4 过 5 败、次跑 7 过 2 败、三跑 8 过 1 败）**：
   - 原因 A（真实内核缺口捕获与修复）：`dcchenyong.skill`（`trigger phase_end owner_is_target_play_phase_card_category_count_positive optional`）首跑摸牌数为 `0` 而非 `3`。定位发现两处原因：① 内核出牌阶段使用装备牌走 `EventData::Equipped` 帧而非 `EventData::CardUsed`，`record_vm_history` 未对 `EventData::Equipped` 置位 `play_cat_mask |= 4`；② 装载器将 `"phase_end"` 映射为 `EventKind::PhaseBegin` + `AtPhase(Phase::End)`（`trig.phase == Some(Phase::End), trig.begin == Some(true)`），而 `VmFilterPolicy::OwnerPlayPhaseBteUsed` 仅允许 `trig.phase == Some(Phase::Play)`。修复 `record_vm_history` 补记 `EventData::Equipped` 的 `play_cat_mask |= 4`，并在 `OwnerPlayPhaseBteUsed` / `OwnerAlivePlayPhaseBteUsed` 允许 `Phase::End && begin == Some(true)` 后，`dcchenyong` 整局测试通过。
   - 原因 B（测试场景构造修正）：① 单目标【杀】在 `ShaResolveMachine::emit_sha_damage` 发出 `Damage` 帧的同时调用 `advance_target` 将【杀】移入弃牌堆，改用装备【方天画戟】对 `[1, 2]` 打出双目标【杀】（对首目标结算伤害时 `extra_targets` 非空，【杀】保持在 `core.table` 即 `CardZone == 6`）验证 `dcjianxiong` 从处理区获牌与渐进摸牌；② 4 人局 0 号对 2 号距离为 2，督持测试改用无距离限制的【决斗】；③ 遗秉测试中 2 号同时持有 `zhuzhou`（在 `TargetDamage` 窗先于 `Dying` 拿走 1 号首张手牌）且连续两次 `debug_place(1, "duel")` 因牌堆 `duel` 耗尽复用同卡，改为放置 `"sha"` 与 `"wuzhong"` 两张不同手牌；④ `wxdlqianqian.skill` 与 `rewansha.skill` 含 `modifier` 声明，`pack_for` 桥接前清空 `modifier_index` 并重算 `logic_hash`。
   - 边界场景补充核验：额外覆盖 `DrawCardsFromBottom` 空牌堆重洗、`SetDrawPhaseFromBottom` 叠加 `draw_phase_bonus` 且牌堆仅剩 1 张触发中途重洗、多名角色同持 `danlao` 对同一 AOE 锦囊依次发动 `CancelIncomingCardUse`（首名获实体牌、次名仅排除目标不重复复制实体牌）、以及 `rewansha` 在濒死角色吃【桃】脱离濒死时触发 `dying_after`（`SetNonLockedSkillSuppression(false)` 即时清除压制标记），并把 `wxdlqianqian` 与 `rewansha` 纳入旧线 `pa12_dsl_legacy_differential` 12 项样本对比。
   - 补跑结果：`cargo test --offline -p engine-native --test misc pa12_batch11_hand_draw_and_card_effects::` **9/9 全绿**。
3. **旧线差分测试编译错误 2（`pa12_dsl_legacy_differential.rs` 处理区查找 API）**：
   - 现象：`tx.state().processing_zone()` 方法不存在。
   - 修复：改为 `tx.state().zones.find(engine_core::state::ZoneKind::Processing, PlayerId::NONE).expect("processing zone")`，编译并运行 **11/11 全绿**。

## 3. 全量门禁核验结果

| 门禁项 | 命令 | 结果 |
|---|---|---|
| 行数门禁单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | `Ran 2 tests ... OK` |
| 行数门禁 | `python3 -B scripts/check-source-lines.py` | `9122 files <= 2000 lines; Rust non-test files <= 1000 effective lines` |
| 原生分层棘轮 | `python3 -B scripts/check-native-layering.py` | `0 upward references <= 0` |
| 批⑪原生整局测试 | `cargo test --offline -p engine-native --test misc pa12_batch11_hand_draw_and_card_effects::` | **9 passed; 0 failed** |
| 旧线 `vm_host` 差分测试 | `cargo test --offline -p engine-core --test misc pa12_dsl_legacy_differential::` | **11 passed; 0 failed** |
| 原生三 crate + loader 全量 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader` | **2,070 passed; 0 failed; 74 ignored**（三 crate `2,033/0/74`，loader `37/0/0`） |
| 独立 target Gate 1b `-D warnings` | `CARGO_TARGET_DIR=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/target/gate1b-check RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`（跑毕已清理 `target/gate1b-check`） | **0 warnings, exit 0** |
| WASM 目标检查 | `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` | **0 warnings, exit 0** |
| 全量 `preflight.py` | `python3 -B scripts/preflight.py` | **21/21 全部通过** |
