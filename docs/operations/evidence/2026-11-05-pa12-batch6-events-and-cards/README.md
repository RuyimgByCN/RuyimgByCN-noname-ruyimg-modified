# PA12 批⑥（零升号子批）：`phase_any_end` 映射、`trigger_ctx_of` 四事件臂补齐与 `EngineVmHost::event_cards` 接线证据（2026-11-05）

- **核验日期**：2026-11-05
- **基线提交**：`cc2652809`（`main`）
- **当前版本门禁（不变）**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`（NGSA 107）、`RULE_VM_ABI_VERSION = 759`、`NETWORK_PROTOCOL_VERSION = 8`（WS8）、`content schema = 5`

---

## 1. 背景与范围

依据 [DSL/IR → 原生 生成迁移计划（PA12④）](../../../architecture/native-dsl-migration-plan.md) §2 B / §2 C / §2 I 及 [统一剩余台账](../../../architecture/native-remaining-ledger.md) `PA12` 授权边界（计划内零升号批次连续推进）：

1. **`phase_any_end` 装载器映射（§2 B）**：
   - `content/original/ported/**/*.skill` 中共 80 处 `trigger phase_any_end <filter>`。
   - 旧线 `phase_any_end` 对齐“任一阶段结束时”（不加阶段门），而原生 `EventKind::PhaseEnd` 恰好在每个阶段结束帧发出。
   - 在 [`crates/content-native-loader/src/lib.rs`](../../../../crates/content-native-loader/src/lib.rs) 的 `bridge_vm_skill_decls` 中新增 `"phase_any_end" => EventKind::PhaseEnd`（无 `AtPhase` 门），与带阶段门的 `"phase_draw_end" | "phase_play_end" | "phase_discard_end"` 并列。
2. **`trigger_ctx_of` 缺失事件上下文臂补齐（§2 C）**：
   - 装载器早已将 `cards_drawn`（143 处）、`hp_recovered`（69 处）、`hp_lost`（65 处）、`dying_after`（5 处）映射到原生 `EventKind::{CardsDrawn, Recover, LoseHp, DyingSaved}`，且 `rvm_named::compile_named` 已编译对应滤镜程序，但 [`crates/engine-native-kernel/src/rvm_bridge.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge.rs) 的 `trigger_ctx_of(event: &EventData)` 此前落入 `_ => d` 默认空上下文（`event_player: None`、`event_source: None`、`event_amount: 0`、`kind: 0`），导致运行时滤镜谓词（`TargetIsSelf`、`TargetAlive`、`Gained`、`Batch`）恒假、模块体 `HOST_EVENT_PLAYER` / `HOST_EVENT_SOURCE` / `HOST_EVENT_AMOUNT` 恒 0。
   - 在 `trigger_ctx_of` 中补齐：
     - `EventData::CardsDrawn { player, cards }`：`event_player = Some(*player)`、`event_source = Some(*player)`、`event_card = cards.first().copied()`、`event_cards = cards.clone()`、`event_amount = cards.len() as u32`、`batch_len = cards.len() as u32`、`kind = 5`（使 `Atom::Gained` 与 `Atom::Batch` 在摸牌事件上按真实张数求值）；
     - `EventData::Recover { player, amount, by_card }`：`event_player = Some(*player)`、`event_source = Some(*player)`、`event_card = *by_card`、`event_amount = *amount`、`kind = 10`；
     - `EventData::LoseHp { player, amount }`：`event_player = Some(*player)`、`event_source = Some(*player)`、`event_amount = *amount`、`kind = 11`；
     - `EventData::DyingSaved { player, source }`：`event_player = Some(*player)`、`event_source = *source`、`kind = 6`；
     - 同步为 `TurnBegin`、`TurnEndWindow`、`Phase`、`CardResponded` 补齐 `event_source = Some(*player)`（对齐旧线阶段/打出一律以当事人为 `source/target` 的约定），并为 `CardsGiven`、`CardUsed`、`CardsMoved`、`CardsDiscarded`、`CardResponded` 填充 `event_cards` 与 `event_amount`。
3. **`EngineVmHost::event_cards` 接线（§2 I：`HOST_EVENT_CARD_COUNT`）**：
   - 在 `VmTriggerCtx` 中新增 `event_cards: Vec<CardId>`；
   - 在 `EngineVmHost` 中新增 `cards: Vec<u32>` 并覆写 `VmHost::event_cards(&self) -> &[u32]`；
   - 在 [`crates/engine-native-kernel/src/rvm_bridge/machine.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/machine.rs) 的 `VmMachine::run` 中注入 `cards: self.trig.event_cards.clone()`，关闭 `PA12 §2 I`（`HOST_EVENT_CARD_COUNT` 恒 0）。

---

## 2. 真实 `.skill` 整局测试与旧线差分验证

| 测试文件 | 用例 | 覆盖内容 | 结果 |
| --- | --- | --- | --- |
| [`crates/engine-native/tests/pa12_batch6_events_and_cards.rs`](../../../../crates/engine-native/tests/pa12_batch6_events_and_cards.rs) | `pa12_batch6_phase_any_end_owner_and_non_owner_gates` | `huicui/dcmouli.skill`（`phase_any_end owner`）与 `huicui/dcgangjian.skill`（`phase_any_end owner_alive_not_target`）真实装载、本/他座位阶段结束触发、正反例 | ok |
| 同上 | `pa12_batch6_cards_drawn_owner_is_target_and_gained_atom` | `offline/peersheng.skill`（`cards_drawn owner_alive_not_target`）与 `collab/haoxue.skill`（`cards_drawn owner_card_gained_not_target`）真实摸牌帧触发与 `Atom::Gained` 求值 | ok |
| 同上 | `pa12_batch6_hp_recovered_and_hp_lost_triggers` | `xianding/dcjiexing.skill`（`hp_recovered owner`）、`huicui/dcgangu.skill`（`hp_recovered owner_alive`）、`key/yuzuru_kunfen.skill`（`hp_lost owner_is_target`）真实桃回复与失去体力整局链路 | ok |
| 同上 | `pa12_batch6_dying_after_xinjuxiang_damages_saved_player_and_removes_self` | `shiji/xinjuxiang.skill`（`dying_after owner_alive`）：濒死被桃救回发出 `DyingSaved`，模块体对 `event_player` 造 1 伤并移除自身技能 | ok |
| 同上 | `pa12_batch6_event_card_count_mingzhe_outside_vs_inside_play_phase` | `sp/mingzhe.skill`（明哲：`card_used` / `card_responded` / `cards_discarded` + `HOST_EVENT_CARD_COUNT > 0` + `!InPlayPhase`）：回合外闪响应 `event_cards=[shan]` 触发摸 1 牌，本人出牌阶段用牌 `InPlayPhase=true` 静默不摸牌 | ok |
| [`crates/engine-core/tests/pa12_dsl_legacy_differential.rs`](../../../../crates/engine-core/tests/pa12_dsl_legacy_differential.rs) | `pa12_legacy_batch6_modules_match_native` | 同一批 6 个 `.skill` 模块在旧线 `vm_host` 上的效果序列及明哲 `event_card_count={0,1} × Phase::{End,Play}` 四象限差分完全一致 | ok |

---

## 3. 门禁执行命令与原始日志

| 门禁项 | 命令 | 结果 | 原始日志 |
| --- | --- | --- | --- |
| 行数门禁单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | `Ran 11 tests ... OK` | [source-lines-unittest.log](source-lines-unittest.log) |
| 源码与有效行门禁 | `python3 -B scripts/check-source-lines.py` | `check-source-lines: OK` | [source-lines.log](source-lines.log) |
| 分层棘轮门禁 | `python3 -B scripts/check-native-layering.py` | `check-native-layering: OK (0 boundary violations)` | [layering.log](layering.log) |
| 原生定向回归（批⑥ 5 项） | `cargo test --offline -p engine-native --test misc pa12_batch6_` | `5 passed; 0 failed` | [pa12-batch6-native.log](pa12-batch6-native.log) |
| 旧线差分回归（6 项） | `cargo test --offline -p engine-core --test misc pa12_dsl_legacy_differential::` | `6 passed; 0 failed` | [pa12-batch6-legacy-diff.log](pa12-batch6-legacy-diff.log) |
| 原生三 crate 全量测试 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` | `1,992 passed; 0 failed; 75 ignored` | [native-3crates-test.log](native-3crates-test.log) |
| Gate 1b 严格零警告 | `CARGO_TARGET_DIR=/tmp/noname-gate1b-pa12b6 RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets` | exit 0（0 warnings，跑后已清理 `/tmp/noname-gate1b-pa12b6`） | [gate1b-warnings.log](gate1b-warnings.log) |
| WASM 目标编译检查 | `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` | exit 0 | [wasm32-check.log](wasm32-check.log) |
| Preflight 全量收口 | `python3 -B scripts/preflight.py` | 21/21 全部通过 | [preflight.log](preflight.log) |
