# T12 swd deferred 消解批 112：孤行（字段批，2026-10-17）

- 日期：2026-10-17
- 范围：一技 deferred 消解（tag 458/459）——**148/184 技条目已映射**（deferred 35→34）
- **规则 v300→301、存档 v85→86**（TurnState 追加 `guxing_armed: bool` + `guxing_count: u8`）
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（guxing1/guxing3 逐段核验）

## 语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 孤行 guxing1（慕容诗 `swd_murongshi`） | `enable: "phaseUse"`, usable 1：若手牌数 <= max(1, 已损失体力) 且 >0，将全部手牌当杀使用（1–3 目标）；结算后摸该杀造成的伤害数 | `SwdGuxing`（tag 458）=主动技（门：手牌数 <= max(1,已损失体力) 且 >0）：Act→SelectPlayers（1–3）→TargetChoice（弃全部手牌 + `use_virtual_card(Sha)` + 置 `guxing_armed`）→**`Flow::Yield`** → SelectCard（结算后 `draw_cards(guxing_count)` + 清旗） |
| 孤行·计数 guxing3 | `source: damageAfter`（parent=guxing）：孤行杀造成伤害 → 计数 +1 | `SwdGuxing3`（tag 459）=Damage（filter：`source=owner ∧ !prevented ∧ guxing_armed`）→ `guxing_count++` |

## 字段批

- `TurnState.guxing_armed: bool` + `TurnState.guxing_count: u8`（state.rs）；入 `canonical_hash`。
- 存档：`archive.rs` 追加；`ARCHIVE_FORMAT_VERSION` 85→**86**；`STANDARD_RULESET_VERSION` 300→**301**。
- 14 golden 重录（[replay-golden-v301.log](replay-golden-v301.log)）；版本回退探针（301→300 全 mismatch）。
- `d4_boss_b0_archive_format_v79_rejects_v78_without_migration` 断言同步 85→86。

## 验证

- **定向回归** `swd_deferred_guxing_all_hand_sha_and_draw`：0 号 hp2/4、手牌 1 张 → 发动孤行 → 1 号受 1 伤（4→3）；结算后摸 1（伤害数）。2 passed / 0 failed（[targeted.log](targeted.log)，含 b111 回归）。
- **全量**：engine-native **894 passed / 0 failed / 0 ignored**。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。
- **键表守护** `host_behavior_keys_bijective` 更新为 456（tag 459）。
- 包哈希：`packages/extensions/shahai-swd-upstream-noname` 经 `rehash_npack` 重算（swd_murongshi 补挂 guxing3）。

## 结论

- **148/184 映射，deferred 35→34**。`T12-SWD-DEFERRED-SKILLS` 仍开放（余 34 项）。
- 首个「结算后继续」型（`use_virtual_card` + `Flow::Yield` 后在续步摸伤害数）。
