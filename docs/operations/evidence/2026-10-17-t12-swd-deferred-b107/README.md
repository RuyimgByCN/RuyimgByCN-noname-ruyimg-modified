# T12 swd deferred 消解批 107：魂身（字段批，2026-10-17）

- 日期：2026-10-17
- 范围：一技 deferred 消解（tag 446/447）——**143/184 技条目已映射**（deferred 40→39）
- **规则 v295→296、存档 v80→81**（PlayerState 尾部追加 `hyunshen_count`）
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（hyunshen/hyunshen2 逐段核验）

## 语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 魂身 hyunshen（胡月 `swd_huyue`） | `trigger: {player: ["respond", "useCard"]}`：使用/打出闪后计数 +1 | `SwdHyunshen`（tag 446）=CardUsed + `extra_events [CardResponded]`（filter：牌为闪）；效果步 `hyunshen_count += 1` |
| 魂身·收束 hyunshen2 | `trigger: {player: "phaseBegin"}`，forced：计数 >0 则摸等量并清零 | `SwdHyunshen2`（tag 447）=PhaseBegin（filter：计数 >0）；效果步 `draw_cards(计数)` + 清零 |
| mod globalTo | 他人算向持有者距离 +计数 | `computed_distance` 加一处：`b` 持 hyunshen 时 `plus += b.hyunshen_count`（上游 globalTo 同口径） |

## 字段批

- `PlayerState.hyunshen_count: u8`（state.rs）；入 `canonical_hash`（engine.rs）。
- 存档编码：`archive.rs` 尾部追加 `hyunshen_count`；`ARCHIVE_FORMAT_VERSION` 80→**81**。
- `STANDARD_RULESET_VERSION` 295→**296**。
- 14 golden 重录（[replay-golden-v296.log](replay-golden-v296.log)）；**版本回退探针**：临时把规则版本改回 295、新代码不动 → 14 场全 mismatch，证明位移含规则版本字段；魂身分支仅经 swd 扩展将可达，标准 14 场景不可达。
- `d4_boss_b0_archive_format_v79_rejects_v78_without_migration` 断言同步 80→81（伪造 79 拒收）。

## 验证

- **定向回归** `swd_deferred_hyunshen_counter_draw_and_distance`：1 号打出闪 → 计数 1、距离(0→1) 1→2；1 号准备阶段 → 摸 1 并清零。3 passed / 0 failed（[targeted.log](targeted.log)，含 b105/b106 回归）。
- **全量**：engine-native **889 passed / 0 failed / 0 ignored**。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。
- **键表守护** `host_behavior_keys_bijective` 更新为 444（tag 447）。
- 包哈希：`packages/extensions/shahai-swd-upstream-noname` 经 `rehash_npack` 重算（swd_huyue 补挂 hyunshen2）。

## 结论

- **143/184 映射，deferred 40→39**。`T12-SWD-DEFERRED-SKILLS` 仍开放（余 39 项）。
- 首个「状态字段」型 swd 消解（字段批：版本 + 存档 + golden 重录 + 回退探针）。
