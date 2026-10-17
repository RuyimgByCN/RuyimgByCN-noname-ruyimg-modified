# T12 swd deferred 消解批 109：聚悉（字段批，2026-10-17）

- 日期：2026-10-17
- 范围：一技 deferred 消解（tag 451/452）——**145/184 技条目已映射**（deferred 38→37）
- **规则 v297→298、存档 v82→83**（PlayerState 尾部追加 `juxi_count`）
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（juxi/juxi_count 逐段核验）

## 语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 聚悉 juxi（小黄龙 `swd_xiaohuanglong`） | `enable: "phaseUse"`：计数 >= 存活数时可发动：扣存活数并选一名角色——已受伤则自选受伤/回复，否则受伤 | `SwdJuxi`（tag 451）=主动技（`activate()` 门：计数 >= 存活数）：Act→SelectPlayers→TargetChoice（扣计数；已受伤→PickTag）→SelectNature（Tag(1)=回复/其余=伤害） |
| 聚悉·计数 juxi_count | `global: "discardAfter"`，forced：非当前回合角色弃牌 → 计数 +1 | `SwdJuxiCount`（tag 452）=CardsDiscarded（filter：弃牌者≠当前回合玩家）；效果步计数 +1 |

## 字段批

- `PlayerState.juxi_count: u8`（state.rs）；入 `canonical_hash`。
- 存档：`archive.rs` 尾部追加；`ARCHIVE_FORMAT_VERSION` 82→**83**；`STANDARD_RULESET_VERSION` 297→**298**。
- 14 golden 重录（[replay-golden-v298.log](replay-golden-v298.log)）；版本回退探针（298→297 全 mismatch）。
- `d4_boss_b0_archive_format_v79_rejects_v78_without_migration` 断言同步 82→83。

## 验证

- **定向回归** `swd_deferred_juxi_damage_or_recover`：未受伤目标 → 受 1 伤、扣存活数 4→0；已受伤目标 → 自选回复（2→3）。2 passed / 0 failed（[targeted.log](targeted.log)，含 b108 回归）。
- **全量**：engine-native **891 passed / 0 failed / 0 ignored**。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。
- **键表守护** `host_behavior_keys_bijective` 更新为 449（tag 452）。
- 包哈希：`packages/extensions/shahai-swd-upstream-noname` 经 `rehash_npack` 重算（swd_xiaohuanglong 补挂 juxi 组）。

## 结论

- **145/184 映射，deferred 38→37**。`T12-SWD-DEFERRED-SKILLS` 仍开放（余 37 项）。
