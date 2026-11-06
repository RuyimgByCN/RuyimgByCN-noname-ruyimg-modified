# PA12 批⑧（零升号子批）：确定性非挂起 `EffectRequest` 11 项原语接入、`kind_tag` 对齐、`resolve_skill_def` 与 `rvm_named` 弃牌目标短语验收（2026-11-06）

## 1. 背景与范围

按 [PA12 单一引擎收敛计划](../../../architecture/native-dsl-migration-plan.md) §2 E「宿主原语扩面」与用户 2026-10-27 授权（「在迁移计划范围内连续推进各批；新立 ADR、改协议、改版本门禁的批次仍须先问用户」），本批在**不改动规则版本（333）、存档格式（NGSA 107）、content schema（5）、rule-vm ABI（759）、网络协议（WS8）**的前提下，完成以下零升号增量：

1. **`crates/engine-native-kernel/src/rvm_ops.rs` 11 项确定性非挂起 `EffectRequest` 原语接入**：
   - `AddSkillUntilTurnEnd { player, skill_def }`：通过 `resolve_skill_def(catalog, raw)` 解析目录技能定义（优先比对 `package_skill_id() == Some(raw)`，回退 `u16::try_from(raw)`）并授予角色，或对未入目录的包级 `skill_def` 写入 `vm_aux.flags[(p, "skill:{raw}")]` + `"skill_turn:{raw}"`（回合末由 `VmAux::clear_turn` 自动清除）。
   - `RemoveSkillByDef { player, skill_def }`：从 `player.skills` 移除目标技能定义并清理对应 `vm_aux.flags`。
   - `RevealHand { player }` / `RevealHandToPlayer { player, observer }`：校验手牌非空（`RevealHandToPlayer` 同时校验 `player != observer`）并发射 `EventData::CardsRevealed` 与公开 `LogEvent::CardsRevealed`。
   - `GainCardByKind { player, kind }` / `GainDiscardCardByKind { player, kind }`：按编译器 `card_kind_value` 标准标签在牌堆或弃牌堆检索首张匹配卡牌移入角色手牌，记录 `LogEvent::CardsDrawn`（避免发射自指 `EventData::CardsMoved { from: p, to: p }` 导致 `cards_moved` 触发器自激重入）。
   - `GainEquipmentFromDrawPile { player }`：检索牌堆首张装备牌移入角色手牌。
   - `GiveAllHand { player, target }`：校验 `player != target` 且手牌非空，将 `player` 全部手牌交予 `target`，发射 `EventData::CardsMoved` 与 `LogEvent::CardsGiven`。
   - `DiscardAllTargetEquipment { actor, target }`：按武器、防具、-1 马、+1 马、宝物顺序卸除 `target` 全部装备入弃牌堆，触发失装回调并发射 `EventData::CardsDiscarded` 与 `LogEvent::CardDiscarded`。
   - `GainEventCards { player, cards }`：过滤仍在弃牌堆中的事件卡牌（去重）移入 `player` 手牌，记录 `LogEvent::CardsDrawn`。
   - `GainCardIfDiscard { player, card }`：若指定 `card` 仍在弃牌堆则移入 `player` 手牌，记录 `LogEvent::CardsDrawn`。
2. **`kind_tag(kind: CardKind) -> u8` 与编译器 `card_kind_value` 对齐**：
   - 原 `rvm_ops.rs::kind_tag` 使用临时占位值（`Weapon => 20,Le => 10` 等），与 `crates/content-compiler/src/compile.rs::card_kind_value` 产出的字节码常数不一致。现统一对齐为编译器权威编码（`Sha => 1, Tao => 2, Shan => 3, Jiu => 4, FireSha => 5, LeiSha => 6, Weapon => 10, Armor => 11, HorseMinus => 12, HorsePlus => 13, Treasure => 14, Wuzhong => 20, Dismantle => 21, Snatch => 22, Duel => 23, SavageAttack => 24, ArcheryAttack => 25, Taoyuan => 26, Wugu => 27, Le => 30, Bing => 31, ShanDian => 32, Jiedao => 33, Chain => 34, Nullify => 36, Youdishenru => 40, WangmeiZhike => 42`）。
3. **`crates/engine-native-kernel/src/rvm_ops/query.rs` `QueryRequest::HasSkill` 同步升级**：
   - 使用 `resolve_skill_def(catalog, raw)` 替代单纯 `raw as u16` 截断，并同步检查 `vm_aux.flags[(p, "skill:{raw}")] == 1`。
4. **`crates/engine-native-kernel/src/rvm_ops/codec.rs` 补齐标签 `38..=48`**：
   - 为上述 11 项新增允许效果补齐快照编码与解码，使 `accept_effect` 允许面全部 **48/48** 项 `EffectRequest` 变体在提交期挂起窗口下均可无损往返存档。
5. **`crates/engine-native-kernel/src/rvm_named.rs` 多词短语窗口与弃牌目标短语补齐**：
   - 将 `take_known` 最长匹配窗口从 `min(6)` 扩至 `min(8)`，补齐 7 词短语 `"other_target_alive_has_discardable_hand_equip"` 及 `"target_alive_has_discardable_hand_equip"`、`"has_discardable_hand_equip"`、`"other_target_alive"`、`"target_alive"`、`"other_responded_sha_gainable"`（以及 `Atom::CanDiscard(0)`），修复 `mobile/ly_piliche.skill` 因 7 词短语被拆散导致误检拥有者自身手牌/装备的问题。

## 2. 真实 `.skill` 整局回归与旧线差分（11 个真实技能）

新增集成测试 [`pa12_batch8_deterministic_effects.rs`](../../../../crates/engine-native/tests/pa12_batch8_deterministic_effects.rs)（登记于 `crates/engine-native/tests/groups/misc.rs`，9 项测试）与旧线差分 [`pa12_dsl_legacy_differential.rs`](../../../../crates/engine-core/tests/pa12_dsl_legacy_differential.rs)（`pa12_legacy_batch8_deterministic_effects_skills_match_native`）：

| 编号 | 真实 `.skill` | 验证原语与短语 | 原生整局 | 旧线 `vm_host` 差分 |
|---|---|---|---|---|
| 1 | `sp/wuji.skill` + `sp/huxiao.skill` | `RemoveSkillByDef`（`remove_skill huxiao`）+ `AddSkillUntilTurnEnd` + `QueryRequest::HasSkill` | PASS | PASS |
| 2 | `yijiang/zhichi.skill` + `yijiang/jigong.skill` | `AddSkillUntilTurnEnd` 跨技能授予 + 回合末 `VmAux::clear_turn` 自动剥离 + `RevealHand` | PASS | PASS |
| 3 | `collab/dcqiusuo.skill` + `mobile/friendmanjuan.skill` | `GainCardByKind(Chain=34)` + `GainDiscardCardByKind(Tao=2)`（验证不重入 `cards_moved`） | PASS | PASS |
| 4 | `collab/dclonggong.skill` | `GainEquipmentFromDrawPile`（受伤害后确认从牌堆获得首张装备牌） | PASS | PASS |
| 5 | `mobile/ly_piliche.skill` | `DiscardAllTargetEquipment` + 7 词短语 `other_target_alive_has_discardable_hand_equip` | PASS | PASS |
| 6 | `yijiang/oldrenxin.skill` + `shiji/spshiji.skill` | `GiveAllHand`（濒死时将全部手牌交予濒死角色） | PASS | PASS |
| 7 | `sp/naman.skill` + `collab/olhaizhu.skill` | `GainEventCards` + `GainCardIfDiscard` + `other_responded_sha_gainable` | PASS | PASS |

## 3. 构造阶段首次运行问题与修复记录

在编写 `pa12_batch8_deterministic_effects.rs` 过程中首次运行曾出现以下 4 处问题，均已定位根因并修复：
1. `accept_effect` 中 `GainCardByKind` / `GainDiscardCardByKind` 初版落在通用 `ok_player(*player, "player")?` 分支之后，导致 `kind == 0` 負控测试未能命中的专门校验分支 → 将 `GainCardByKind`、`GainDiscardCardByKind`、`GainCardIfDiscard` 移出通用分支并执行专门校验。
2. `GainDiscardCardByKind` 初版发射 `EventData::CardsMoved { cards, from: p, to: p }`，导致监听 `cards_moved` 的 `mobile/friendmanjuan.skill` 在获得弃牌堆中的桃后再次自触发 → 参照 `luo_shen.rs` / `ji_jie.rs`，从牌堆/弃牌堆获牌统一记录 `LogEvent::CardsDrawn` 而不发座位间 `EventData::CardsMoved`。
3. `rvm_named::take_known` 窗口上限为 `min(6)`，7 词短语 `"other_target_alive_has_discardable_hand_equip"` 未被整体识别，拆为 `"other_target_alive"` + `"has_hand_equip"`（检查拥有者 `0` 而非目标 `1`）→ 扩至 `min(8)` 并补齐 `"has_discardable_hand_equip"`。
4. 测试辅助脚本：`wuji` 测试从 `Phase::Play` 仅调一次 `GameAction::NextPhase`（落在 `Phase::Discard` 而非 `Phase::End`）；`naman` 测试在 `0` 使用决斗指 `1` 后未先回答 `0` 的决斗打出杀询问即断言 `2` 的 `naman` 询问 → 修正测试驱动步骤后 9/9 全绿。

## 4. 门禁核验命令与结果

| 检查项 | 命令 | 结果 | 原始日志 |
|---|---|---|---|
| 行数门禁自测与全库扫描 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py' && python3 -B scripts/check-source-lines.py` | **OK (2 tests)**；9,115 文件 ≤2,000 行，Rust 非测试 ≤1,000 有效行 | [`targeted-tests.txt`](targeted-tests.txt) |
| 分层棘轮检查 | `python3 -B scripts/check-native-layering.py` | **0 越界** | [`targeted-tests.txt`](targeted-tests.txt) |
| 定向整局回归与旧线差分 | `cargo test --offline -p engine-native --test misc pa12_batch8_deterministic_effects::` + `cargo test --offline -p engine-core --test misc pa12_dsl_legacy_differential::` | **9 passed; 0 failed** / **8 passed; 0 failed** | [`targeted-tests.txt`](targeted-tests.txt) |
| 原生三 crate 全量测试 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` | **2,007 passed; 0 failed; 74 ignored**（17 个测试二进制） | [`three-crate-tests.txt`](three-crate-tests.txt) |
| 独立 target Gate 1b (`-D warnings`) + `wasm32` check + 私有缓存清理（裁决 23） | `CARGO_TARGET_DIR=/tmp/noname-pa12b8-check RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets && ... --target wasm32-unknown-unknown -p engine-native-wasm && rm -rf /tmp/noname-pa12b8-check` | **exit 0，0 warnings**，私有缓存已清理 | [`gate1b-check.txt`](gate1b-check.txt) |
| 全仓 `preflight.py` 门禁 | `python3 -B scripts/preflight.py` | **21/21 全部通过** | [`preflight.txt`](preflight.txt) |
