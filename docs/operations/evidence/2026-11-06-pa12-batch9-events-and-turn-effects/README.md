# PA12 批⑨（零升号子批）：3 类触发事件映射、9 项阶段与回合效果原语、`extra_events` 传播与滤镜短语验收（2026-11-06）

## 1. 背景与范围

按 [PA12 单一引擎收敛计划](../../../architecture/native-dsl-migration-plan.md) §2 C「事件名未映射」、§2 D「滤镜语义」与 §2 E「宿主解析原语」及用户 2026-10-27 授权（「在迁移计划范围内连续推进各批；新立 ADR、改协议、改版本门禁的批次仍须先问用户」），本批在**不改动规则版本（333）、存档格式（NGSA 107）、content schema（5）、rule-vm ABI（759）、网络协议（WS8）**的前提下，完成以下零升号增量：

1. **三类触发事件映射与 `extra_events` 传播（`content-native-loader` + `engine-native-kernel`）**：
   - `"damage_prevented"` → `EventKind::DamageBefore`（配合 `VmFilterPolicy::DamagePrevented`，校验 `kind == 1 && event_amount == 0`），并在 `VmMachine::run` 中对 `DamagePrevented` 触发将防止前原始伤害量（存储于 `(target, "prevented_damage_amount")`）注入 `HostContext.event_amount`，使模块体内 `HOST_EVENT_AMOUNT` 精确读取被防止的伤害点数（如 `yijiang/renzheng.skill`、`sp/stdrenzheng.skill`）。
   - `"character_state_changed"` → `EventKind::TurnOvered` + `extra_events: vec![EventKind::Linked]`，在 `trigger_ctx_of` 中为 `EventData::TurnOvered`（`face_down=false => 1, true => 2`）与 `EventData::Linked`（`linked=true => 3, false => 4`）注入 `kind: 12` 与状态方向编码，并在 `content-native-loader/src/lib.rs` 与 `vm_active.rs` 传播 `extra_events`，使同一 VM 触发器同时监听翻面与横置/重置（如 `yijiang/faen.skill`、`yijiang/refaen.skill`）。
   - `"skill_added"` → `EventKind::SkillToggled`，在 `rvm_ops/commit.rs` 的 `EffectRequest::AddSkill` 提交时（当存在监听 `SkillToggled` 的启用技能实例时）发射 `EventData::SkillToggled { player, skill, enabled: true }`，并通过 `VmFilterPolicy::OwnerSkillAddedSelf(target_def)` 按 `resolve_skill_def(catalog, raw)` 精确匹配目标技能定义（如 `collab/pxyanmen.skill` + `collab/pxbeihai.skill`、`mobile/zj_hanci.skill`）。
2. **9 项确定性阶段/回合/装备/杀次数 `EffectRequest` 原语接入（`crates/engine-native-kernel/src/rvm_ops.rs` + `rvm_ops/commit.rs` + `rvm_ops/codec.rs`）**：
   - `IncreaseCurrentDrawCount { player, count }`（codec tag 49）：向 `vm_aux.flags[(p, "draw_phase_bonus")]` 累加摸牌阶段额外摸牌数，在 `kernel/draw.rs` `DP::Begin` 消费并累加至 `extra_draw`，回合末 `VmAux::clear_turn` 自动兜底清理。
   - `SkipNextPlayPhase { player }`（tag 50）：置 `player_mut(p).skip_play_phase = true`。
   - `SkipNextDiscardPhase { player }`（tag 51）：置 `player_mut(p).skip_discard_phase = true`；同步修正 `kernel/phase/discard.rs::begin_discard` 与 `kernel/intrinsic.rs` `Phase::Discard`，使 `skip_discard_phase` 在阶段开始时与 `skip_play_phase` 同口径跳过弃牌阶段主结算并清零旗标。
   - `SkipNextJudgePhase { player }`（tag 52）：置 `vm_aux.flags[(p, "skip_judge")] = 1`，在 `kernel/intrinsic.rs` `Phase::Judge` 消费并跳过判定牌结算。
   - `SkipJudgeAndDrawPhases { player }`（tag 53）：同时置 `"skip_judge" = 1` 与 `skip_draw_phase = true`。
   - `SkipRemainingPhases { player }`（tag 54）：对当前回合角色按当前阶段置后续未执行阶段的跳过旗标并置 `turn.skip_current_phase = true`。
   - `QueueExtraPhases { player, phases }`（tag 55）：将 `phases`（`1=Prepare, 2=Judge, 3=Draw, 4=Play, 5=Discard, 6=End`）映射为 `Phase` 并逆序压入 `turn.extra_phases`（如 `mobile/zj_fuli.skill`）。
   - `GainTargetEquipment { player, target, slot }`（tag 56）：按槽位（`1=Weapon, 2=Armor, 3=HorseMinus, 4=HorsePlus, 5=Treasure`）检索 `target` 装备区的对应装备牌，卸载触发失装钩子后移入 `player` 手牌（如 `yijiang/xinduodao.skill`）；同步增强 `machine.rs::to_discard` 守恒守卫，当被卸装备已由前置钩子移入某座位手牌/装备区时不重复塞入弃牌堆。
   - `RefundCurrentShaUse { player }`（tag 57）：将 `turn.sha_used` 饱和减 1（如 `yijiang/oldhuxiao.skill`）。
   - 同时增强 `GainEventCards`：当事件卡牌尚在 `core.table`（如 `card_used` 触发时尚未进弃牌堆的实体杀，`sp/dcyongjue.skill`）或 `core.discard` 时均可获得移入手牌。
3. **`rvm_ops/codec.rs` 补齐标签 `49..=57`**：
   - 使 `accept_effect` 允许面全部 **57/57** 项 `EffectRequest` 变体 100% 往返闭合。
4. **`rvm_bridge.rs` 与 `rvm_named.rs` 滤镜策略与短语补齐**：
   - 新增 `VmFilterPolicy::{DamagePrevented, CharacterStatePositive, OwnerSkillAddedSelf, OwnerSourcePlayFirstSha}`，补齐 `"source_has_weapon"`（`Atom::SourceHasWeapon` tag 111）、`"other_source_alive"` 与 `"card_in_processing_or_discard"` 短语。

## 2. 真实 `.skill` 整局回归与旧线差分（9 组测试 / 11 个真实技能）

新增集成测试 [`pa12_batch9_events_and_turn_effects.rs`](../../../../crates/engine-native/tests/pa12_batch9_events_and_turn_effects.rs)（登记于 `crates/engine-native/tests/groups/misc.rs`，9 项测试）与旧线差分 [`pa12_dsl_legacy_differential.rs`](../../../../crates/engine-core/tests/pa12_dsl_legacy_differential.rs)（`pa12_legacy_batch9_events_and_turn_effects_match_native`）：

| 编号 | 真实 `.skill` | 验证机制与原语 | 原生整局 | 旧线 `vm_host` 差分 |
|---|---|---|---|---|
| 1 | 合成与边界测试 | `codec.rs` 新增 tags `49..=57` 往返与 `accept_effect` 负控拒绝 | PASS | — |
| 2 | `yijiang/renzheng.skill` + `sp/stdrenzheng.skill` | `damage_prevented` 事件映射、`PreventDamage` 联动与 `event_amount` 原始防止点数读取 | PASS | PASS |
| 3 | `yijiang/faen.skill` + `yijiang/refaen.skill` | `character_state_changed` 翻回正面（`TurnOvered`）与铁索横置（`Linked`）双事件监听与选分支 | PASS | PASS |
| 4 | `collab/pxyanmen.skill` + `collab/pxbeihai.skill` + `mobile/zj_hanci.skill` | `AddSkill` 发射 `SkillToggled`、`skill_added` + `owner_skill_added_self_*` 精确定义匹配 | PASS | PASS |
| 5 | `mobile/zj_fuli.skill` | `QueueExtraPhases([Play])` 结束阶段插入额外出牌阶段 | PASS | PASS |
| 6 | `yijiang/xinduodao.skill` | `GainTargetEquipment(Weapon)` + `source_has_weapon` 获得伤害来源装备区武器牌 | PASS | PASS |
| 7 | `sp/dcyongjue.skill` + `yijiang/oldhuxiao.skill` | `owner_source_play_first_sha` + `GainEventCards`（从处理区/台面获牌）+ `RefundCurrentShaUse` | PASS | PASS |
| 8 | 阶段跳过组合 | `SkipNextPlayPhase`、`SkipNextDiscardPhase`、`SkipNextJudgePhase`、`SkipJudgeAndDrawPhases`、`SkipRemainingPhases` | PASS | PASS |
| 9 | 额外摸牌与阶段跳过 | `IncreaseCurrentDrawCount`（摸牌阶段 2+N 张）+ 回合末清理 | PASS | PASS |

## 3. 首次运行失败与修复记录（分开留痕）

1. **编译期穷尽匹配检查（已修复）**：新增 `VmFilterPolicy` 4 个变体与 `Atom::SourceHasWeapon` 后，`crates/engine-native-kernel/src/rvm_bridge/identity.rs` 与 `rvm_named/identity.rs` 的 `#[cfg(test)]` 身份映射表报 `E0004: non-exhaustive patterns` → 补齐 tags `43..=46` 与 tag `111`。
2. **`EventData::Damage` 有符号 `d.amount as u32` 回退（已修复）**：初版将 `trigger_ctx_of`（`rvm_bridge.rs:110`）与 `VmMachine::run`（`rvm_bridge/machine.rs:345`）中的 `d.amount` 写了 `.max(0) as u32`，导致 `c331_source_predicates::c9e_hutian_signed_zero_host_amount_and_later_filters_roundtrip` 与单元测试 `rvm_bridge::damage_context_tests::negative_canonical_damage_reaches_filter_and_signed_vm_host_unchanged` 失败（原始失败日志保留于 [`01-3crate-first-run-failed.log`](01-3crate-first-run-failed.log)）。根因：C331 规则要求负值 `DamageData.amount`（如 `hutian` 封顶后的负向修正）按补码 `d.amount as u32` 原样传入滤镜与 `HostContext.event_amount`，使 VM 栈按有符号 `i32` 比较时保持负号；只有 `DamagePrevented`（`d.prevented && d.amount == 0`）才需回退读取 `(d.target, "prevented_damage_amount")`。恢复 `d.amount as u32` 后两项回归与全量测试全部转绿（[`02-3crate-and-gate1b-green.log`](02-3crate-and-gate1b-green.log)）。

## 4. 门禁核验命令与结果

| 检查项 | 命令 | 结果 | 原始日志 |
|---|---|---|---|
| 行数门禁自测与全库扫描 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py' && python3 -B scripts/check-source-lines.py` | **OK (2 tests)**；9,117 文件 ≤2,000 行，Rust 非测试 ≤1,000 有效行 | — |
| 分层棘轮检查 | `python3 -B scripts/check-native-layering.py` | **0 越界** | — |
| 定向整局回归与旧线差分 | `cargo test --offline -p engine-native --test misc pa12_batch9_events_and_turn_effects::` + `cargo test --offline -p engine-core --test misc pa12_dsl_legacy_differential::` | **9 passed; 0 failed** / **9 passed; 0 failed** | — |
| 原生三 crate + loader 全量测试（首跑） | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader` | 1 failed（`negative_canonical_damage_reaches_filter_and_signed_vm_host_unchanged`） | [`01-3crate-first-run-failed.log`](01-3crate-first-run-failed.log) |
| 原生三 crate + loader 全量测试（修复后重跑） | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader` | **2,053 passed; 0 failed; 74 ignored**（29 个测试段，其中原生三 crate **2,016/0/74**、loader **37/0/0**） | [`02-3crate-and-gate1b-green.log`](02-3crate-and-gate1b-green.log) |
| 独立 target Gate 1b (`-D warnings`) + `wasm32` check + 私有缓存清理（裁决 23） | `CARGO_TARGET_DIR=/tmp/noname-gate1b-check RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets && cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm && rm -rf /tmp/noname-gate1b-check` | **exit 0，0 warnings**，私有缓存已清理 | [`02-3crate-and-gate1b-green.log`](02-3crate-and-gate1b-green.log) |
| 全仓 `preflight.py` 门禁 | `python3 -B scripts/preflight.py` | **21/21 全部通过** | [`03-preflight.log`](03-preflight.log) |
