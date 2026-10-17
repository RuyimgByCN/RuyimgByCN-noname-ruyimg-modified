# T12 swd deferred 消解批 108：待行（字段批，2026-10-17）

- 日期：2026-10-17
- 范围：一技 deferred 消解（tag 448/449/450）——**144/184 技条目已映射**（deferred 39→38）
- **规则 v296→297、存档 v81→82**（PlayerState 尾部追加 `daixing_count`）
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（daixing/daixing2/daixing3 逐段核验）

## 语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 待行 daixing（吕晓雪 `swd_yuxiaoxue`） | `trigger: {player: "phaseEnd"}`：弃 1..张 he 换取等量护甲 | `SwdDaixing`（tag 448）=PhaseEnd（End 阶段末帧）；多步机 Act→SelectCard（弃 he）→ `change_hujia(n)` + `daixing_count = n` |
| 待行·清甲 daixing2 | `phaseBegin`，silent：按剩余计数移除护甲 | `SwdDaixing2`（tag 449）=PhaseBegin（计数 >0）；`change_hujia(-计数)` + 清零 |
| 待行·吸收 daixing3 | `damageEnd`/`damageZero`：计数 -= 护甲吸收量 | `SwdDaixing3`（tag 450）=Damage（自己）；`计数 -= min(计数, 伤害)`（**近似**护甲吸收量） |

## 字段批

- `PlayerState.daixing_count: u8`（state.rs）；入 `canonical_hash`。
- 存档：`archive.rs` 尾部追加；`ARCHIVE_FORMAT_VERSION` 81→**82**；`STANDARD_RULESET_VERSION` 296→**297**。
- 14 golden 重录（[replay-golden-v297.log](replay-golden-v297.log)）；版本回退探针（297→296 全 mismatch）证明位移含版本字段。
- `d4_boss_b0_archive_format_v79_rejects_v78_without_migration` 断言同步 81→82。

## 验证

- **定向回归** `swd_deferred_daixing_hujia_cycle`：0 号结束阶段弃 2 he → 计数 2、护甲 +2；下回合准备阶段 → 计数清零、护甲移除。2 passed / 0 failed（[targeted.log](targeted.log)，含 b107 回归）。
- **全量**：engine-native **890 passed / 0 failed / 0 ignored**。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。
- **键表守护** `host_behavior_keys_bijective` 更新为 447（tag 450）。
- 包哈希：`packages/extensions/shahai-swd-upstream-noname` 经 `rehash_npack` 重算（swd_yuxiaoxue 补挂 daixing 组）。

## 结论

- **144/184 映射，deferred 39→38**。`T12-SWD-DEFERRED-SKILLS` 仍开放（余 38 项）。
