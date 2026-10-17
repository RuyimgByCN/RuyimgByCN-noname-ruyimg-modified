# T12 swd deferred 消解批 110：破舟（字段批，2026-10-17）

- 日期：2026-10-17
- 范围：一技 deferred 消解（tag 453/454）——**146/184 技条目已映射**（deferred 37→36）
- **规则 v298→299、存档 v83→84**（PlayerState 尾部追加 `pozhou_count`）
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（pozhou/pozhou2 逐段核验）

## 语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 破舟 pozhou（轩辕剑仙 `swd_xuanyuanjianxian`） | `trigger: {player: "damageEnd"}`，forced：受伤计数累加伤害值 | `SwdPozhou`（tag 453）=Damage（自己）；效果步 `pozhou_count += 伤害` |
| 破舟·封印 pozhou2 | `enable: "phaseUse"`：计数 >0 时可发动：消耗 1 计数，令一名其他角色非锁定技失效至其下一回合结束 | `SwdPozhou2`（tag 454）=主动技（`activate()` 门：计数 >0 且存在未封印其他角色）：Act→SelectPlayers→TargetChoice（消耗 1 计数；禁用目标全部技能实例 + 授予 `SwdFengyinState`，其下一阶段开始解封）。**登记差异**：解封时点=下一阶段开始（上游下一回合结束）；原生无锁定技区分=全部技能失效 |

## 字段批

- `PlayerState.pozhou_count: u8`（state.rs）；入 `canonical_hash`。
- 存档：`archive.rs` 尾部追加；`ARCHIVE_FORMAT_VERSION` 83→**84**；`STANDARD_RULESET_VERSION` 298→**299**。
- 14 golden 重录（[replay-golden-v299.log](replay-golden-v299.log)）；版本回退探针（299→298 全 mismatch）。
- `d4_boss_b0_archive_format_v79_rejects_v78_without_migration` 断言同步 83→84。

## 验证

- **定向回归** `swd_deferred_pozhou_seal_cycle`：0 号受 2 伤 → 计数 2；发动 → 消耗 1、1 号技能被封印；1 号阶段开始 → 解封（技能恢复）。2 passed / 0 failed（[targeted.log](targeted.log)，含 b109 回归）。
- **全量**：engine-native **892 passed / 0 failed / 0 ignored**。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check --engine-native --all-targets` exit 0。
- **键表守护** `host_behavior_keys_bijective` 更新为 451（tag 454）。
- 包哈希：`packages/extensions/shahai-swd-upstream-noname` 经 `rehash_npack` 重算（swd_xuanyuanjianxian 补挂 pozhou2）。

## 结论

- **146/184 映射，deferred 37→36**。`T12-SWD-DEFERRED-SKILLS` 仍开放（余 36 项）。
- 复用既有「封印」基建（`SwdFengyinState` + 技能实例禁用）。
