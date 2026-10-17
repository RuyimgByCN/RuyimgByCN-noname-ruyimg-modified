# T12 swd deferred 消解批 111：陷阱（字段批，2026-10-17）

- 日期：2026-10-17
- 范围：一技 deferred 消解（tag 455/456/457）——**147/184 技条目已映射**（deferred 36→35）
- **规则 v299→300、存档 v84→85**（PlayerState 尾部追加 `sxianjing_traps: Vec<CardId>`）
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（sxianjing/sxianjing_gain/sxianjing_damage 逐段核验）

## 语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 陷阱 sxianjing（舒莞儿 `swd_shuwaner`） | `enable: "phaseUse"`：置出一张花色未在陷阱表中的手牌为陷阱 | `SwdSxianjing`（tag 455）=主动技（门：有花色未在表中的手牌）：Act→SelectCard（置出）→ `sxianjing_traps.push` |
| 陷阱·触发 sxianjing_gain | `target: useCardToBegin`，forced：他人以同花色牌指定你时，弃陷阱 + 随机获得其一张牌 | `SwdSxianjingGain`（tag 456）=CardUsed（filter：`u.player!=owner ∧ targets 含 owner ∧ 同花色在表中`）；效果步弃同花色陷阱 + 获得使用者首个 he 牌（**确定性近似**） |
| 陷阱·取回 sxianjing_damage | `player: damageEnd`，forced：你受伤时随机取回一张陷阱 | `SwdSxianjingDamage`（tag 457）=Damage（自己，表非空）；效果步取首个陷阱回手（确定性近似） |

## 字段批

- `PlayerState.sxianjing_traps: Vec<CardId>`（state.rs，`len + u32` 序列）；入 `canonical_hash`。
- 存档：`archive.rs` 尾部追加；`ARCHIVE_FORMAT_VERSION` 84→**85**；`STANDARD_RULESET_VERSION` 299→**300**。
- 14 golden 重录（[replay-golden-v300.log](replay-golden-v300.log)）；版本回退探针（300→299 全 mismatch）。
- `d4_boss_b0_archive_format_v79_rejects_v78_without_migration` 断言同步 84→85。

## 验证

- **定向回归** `swd_deferred_sxianjing_trap_cycle`：1 号出牌阶段置出黑桃牌为陷阱；0 号以黑桃杀指定 1 号 → 弃陷阱 + 获得 0 号一张牌；受伤 → 取回陷阱。2 passed / 0 failed（[targeted.log](targeted.log)，含 b110 回归）。
- **全量**：engine-native **893 passed / 0 failed / 0 ignored**。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。
- **键表守护** `host_behavior_keys_bijective` 更新为 454（tag 457）。
- 包哈希：`packages/extensions/shahai-swd-upstream-noname` 经 `rehash_npack` 重算（swd_shuwaner 补挂 sxianjing 组）。

## 结论

- **147/184 映射，deferred 36→35**。`T12-SWD-DEFERRED-SKILLS` 仍开放（余 35 项）。
