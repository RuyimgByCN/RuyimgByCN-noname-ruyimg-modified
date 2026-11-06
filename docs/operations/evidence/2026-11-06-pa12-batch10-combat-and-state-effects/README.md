# PA12 批⑩（战斗、区域整弃、武将状态复原与先手额外出牌原语接入）核验记录（2026-11-06）

- **核验日期**：2026-11-06
- **源码基准**：`55e8ea669` 之上工作树（提交后记入 `docs/architecture/native-progress.md`）
- **版本不变量（零增号）**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`（NGSA 107）、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`NETWORK_PROTOCOL_VERSION = 8`（WS8）

## 1. 变更范围与模块拆分

1. **`crates/engine-native-kernel/src/rvm_ops` 模块拆分与 13 项既有 ABI（`<= 759`）确定性效果原语接入（tags `58..=70`）**：
   - 新增子模块 `crates/engine-native-kernel/src/rvm_ops/commit_combat.rs`，承接批⑩新增的 13 项确定性非挂起 `EffectRequest` 落账，保持 `crates/engine-native-kernel/src/rvm_ops/commit.rs` 有效行 `<= 1,000`。
   - `crates/engine-native-kernel/src/rvm_ops.rs`（`accept_effect`）、`crates/engine-native-kernel/src/rvm_ops/codec.rs`（tags `58..=70` `enc_effects` / `dec_effects`）与 `commit_combat.rs` 同步放行并落账：
     1. `DiscardAllHandAndEquip { player }`（tag `58`）：弃置目标存活角色全部手牌与装备牌（`huilei.skill`、`qibie.skill` 等）。
     2. `DiscardAllHandEquipJudge { player }`（tag `59`）：弃置目标存活角色手牌、装备区与判定区全部牌并清理判定延时锦囊声明映射（`oldniepan.skill` 等）。
     3. `RestoreCharacterState { player }`（tag `60`）：复原目标存活角色武将牌——若处于横置则发 `EventData::Linked { player, linked: false }` 解除横置，若处于背面朝上则发 `EventData::TurnOverBefore { player, cancelled: false }` 翻回正面（`oldniepan.skill` 等）。
     4. `RemoveAllSkills { player }`（tag `61`）：移除目标角色全部技能与 `vm_aux` 临时技能旗标（`duanchang.skill` 等）。
     5. `DrawLinkedPlayers { count }`（tag `62`，`count > 0`）：按当前回合角色起的逆时针座次令所有存活横置角色各摸 `count` 张牌（`jysuozhou.skill` 等）。
     6. `RecoverIfMinCurrentHp { player, amount }`（tag `63`，`amount > 0`）：若目标存活角色当前体力值为全场存活最低（含并列），回复 `amount` 点体力。
     7. `EquipWeaponFromDrawPile { player }`（tag `64`）：若目标存活角色武器栏为空，从牌堆检索第一张武器牌置入装备区并调用 `crate::kernel::grant_equip_skills` 授予伴随武器技能（`twfujian.skill` 等）。
     8. `RemoveCardsByKind { kind }`（tag `65`，`kind > 0`）：从牌堆与弃牌堆移出全部匹配 `kind_tag` 的卡牌至 `"removed"` 区，并同步更新 `crates/engine-native-kernel/src/rvm_ops/query.rs` 的 `locate` 使 `"removed"` 区卡牌查询 `QueryRequest::CardZone` 返回 `7`（`CardZone::Removed`）。
     9. `QueueOpeningExtraPlayPhase { player, skill }`（tag `66`）：在回合开始（`TurnBegin`）将当前阶段切至先手额外 `Phase::Play`，并将 `Phase::Judge`（`0`）压入 `extra_phases` 队首，使先手额外出牌阶段结束后经 `Judge -> Draw -> Play -> Discard -> End` 恢复正常阶段序列（`dangxian.skill`、`redangxian.skill` 等）。
     10. `IncreaseCurrentShaBaseDamage { player, card, amount, skill }`（tag `67`，`amount > 0`）：写入 `(player, "card_damage_bonus:{card}")` 修正当前正在使用的【杀】伤害基数 `+amount`（`oljuejue.skill` 等）。
     11. `IncreaseCurrentCardBaseDamage { player, card, amount, skill }`（tag `68`，`amount > 0`）：写入 `(player, "card_damage_bonus:{card}")` 修正当前正在使用的伤害牌（【杀】、【决斗】、【南蛮入侵】、【万箭齐发】）伤害基数 `+amount`（`stdkengqiang.skill`、`hanyong.skill` 等）。
     12. `IncreaseCurrentShaTargetDamage { source, card, target, amount }`（tag `69`，`amount > 0`）：写入 `(source, "sha_target_damage_bonus:{card}:{target}")` 令当前【杀】对指定目标造成的伤害 `+amount`（`xinanjian.skill` 等）。
     13. `MakeCurrentShaTargetUnrespondable { source, card, target }`（tag `70`）：写入 `(source, "sha_unrespondable:{card}:{target}")` 令指定目标不可响应当前【杀】（`liegong.skill` 等）。
2. **战斗结算消费与回合末清理**：
   - `crates/engine-native-kernel/src/kernel/sha.rs`：`ShaResolveMachine` 检查 `sha_unrespondable:{card}:{target}`（及 `card_targeted` 下 `event_player == source` 时的同源键），命中时直接跳过八卦阵判定与出【闪】询问进入伤害段；伤害段累加 `card_damage_bonus:{card}` 与 `sha_target_damage_bonus:{card}:{target}`，结算完毕清理目标键。
   - `crates/engine-native-kernel/src/kernel/intrinsic.rs`、`crates/engine-native-kernel/src/kernel/duel.rs`、`crates/engine-native-kernel/src/kernel/aoe.rs`：伤害牌基数与决斗/南蛮/万箭结算消费 `card_damage_bonus:{card}`。
   - `crates/engine-native-kernel/src/state.rs`：`VmAux::clear_turn` 清理 `card_damage_bonus:`、`sha_target_damage_bonus:`、`sha_unrespondable:` 前缀旗标。
3. **`crates/engine-native-kernel/src/rvm_named` 模块拆分、域前缀修复与复合词补齐**：
   - 拆出 `crates/engine-native-kernel/src/rvm_named/eval.rs` 子模块承载 `eval_program` / `eval_atom` 等求值逻辑，使 `rvm_named.rs` 有效行降至 ~740 行（`<= 1,000`）。
   - 修复 `split_domain` 中 `TABLE` 末尾 `("owner_", &[Atom::OwnerAlive])` 提前截断后续 `owner_source_or_*` / `owner_is_source_or_*` / `owner_gained_or_lost_*` 析取域分支的缺陷，并新增 `"owner_source_or_targeted_"` 析取域前缀。
   - 新增 `Atom::AnyLinked`（`"linked_exists"`）、`Atom::DamageCard`（`"damage_card"`）、`Atom::NanmanWanjianOrNormalSpadeSha`（`"wounded_nanman_wanjian_or_normal_spade_sha"`）、`Atom::TargetHandGteOwnerHpOrLteAttackRange`（`"target_hand_gte_hp_or_lte_attack_range"`）、`Atom::TargetHandLteOwnerOrHpGteOwner`（`"target_hand_lte_owner_or_hp_gte_owner"`）、`Atom::OwnerOutsideTargetAttackRange`（`"target_alive_owner_outside_target_attack_range"`）。

## 2. 首次失败与修复记录（按 AGENTS.md 分开登记）

1. **编译错误 1（`commit_combat.rs` 私有模块路径）**：
   - 现象：`crates/engine-native-kernel/src/rvm_ops/commit_combat.rs:143` 调用 `crate::kernel::equipment::grant_equip_skills` 报 `error[E0603]: module equipment is private`。
   - 修复：改为调用 `crate::kernel::grant_equip_skills(ctx, p, card)` 公开重导出入口。
2. **测试编译错误 2（`pa12_batch10_combat_and_state_effects.rs` 辅助函数 API 名称）**：
   - 现象：测试辅助函数使用了 `e.active_ask()`、`GameAction::Reply`、`GameAction::EndPlayPhase`、`GameAction::Discard`。
   - 修复：统一改为 `Engine` 真实公开接口 `e.open_choice()`、`GameAction::Answer { choice_id, selection }`、`GameAction::NextPhase` 与 `!matches!(out, CommandOutcome::Rejected(_))`。
3. **定向测试首次运行失败（`pa12_batch10_combat_and_state_effects` 首次 1 过 7 败、第二次 6 过 2 败）**：
   - 原因 A：测试中 `debug_place` 使用了非标准牌名 `"bagua"`、`"nanman"`、`"chitu"`（标准目录牌名为 `"renwang"`、`"savage"`、`"horse_minus"`），且 `0` 号回合从 `Play` 推进到 `1` 号回合仅调用一次 `NextPhase`（停在 `0` 号 `Discard`）。
   - 原因 B（真实内核缺陷捕获与修复 1）：`jysuozhou`（`owner_source_or_targeted_club_linked_exists_once_per_turn`）首次运行触发了 2 次（横置角色摸 2 张而非 1 张）。定位发现 `rvm_named.rs::split_domain` 的 `TABLE` 末尾 `("owner_", ...)` 抢先匹配了所有 `owner_*` 滤镜，导致 `owner_source_or_targeted_*` 退化为按 `_or_` 顶层切分、第一析取支丢掉了 `club_linked_exists_once_per_turn` 尾缀。移除 `TABLE` 中冗余的 `("owner_", ...)`（保留函数末尾兜底）后，`jysuozhou` 严格按每回合限一次触发 1 次。
   - 原因 C（真实内核缺陷捕获与修复 2）：`oldniepan` 的 `RestoreCharacterState` 先手动置 `linked = false` 再发射 `EventData::Linked { player, linked: false }`，而内核 `commit_linked` 本征步骤会对 `linked` 取反（`!linked`），导致 `linked` 被翻回 `true`。修复 `RestoreCharacterState` 为当 `linked == true` 时仅发射 `EventData::Linked { player, linked: false }` 交由 `commit_linked` 翻转并记日志。
   - 补跑结果：`cargo test --offline -p engine-native --test misc pa12_batch10_combat_and_state_effects::` **8/8 全绿**；`cargo test --offline -p engine-core --test misc pa12_dsl_legacy_differential::` **10/10 全绿**。

## 3. 全量门禁核验结果

| 门禁项 | 命令 | 结果 |
|---|---|---|
| 行数门禁单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | `Ran 2 tests ... OK` |
| 行数门禁 | `python3 -B scripts/check-source-lines.py` | `9120 files <= 2000 lines; Rust non-test files <= 1000 effective lines` |
| 原生分层棘轮 | `python3 -B scripts/check-native-layering.py` | `0 upward references <= 0` |
| 批⑩原生整局测试 | `cargo test --offline -p engine-native --test misc pa12_batch10_combat_and_state_effects::` | **8 passed; 0 failed** |
| 旧线 `vm_host` 差分测试 | `cargo test --offline -p engine-core --test misc pa12_dsl_legacy_differential::` | **10 passed; 0 failed** |
| 原生三 crate + loader 全量 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader` | **2,061 passed; 0 failed; 74 ignored**（三 crate `2,024/0/74`，loader `37/0/0`） |
| 独立 target Gate 1b `-D warnings` | `CARGO_TARGET_DIR=/private/tmp/noname-gate1b-pa12-b9 RUSTFLAGS="-D warnings" cargo check -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets` | **0 warnings, exit 0** |
| WASM 目标检查 | `cargo check --target wasm32-unknown-unknown -p engine-native-wasm` | **0 warnings, exit 0** |
| 全量 `preflight.py` | `python3 -B scripts/preflight.py` | **21/21 全部通过** |
