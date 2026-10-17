# T12 swd deferred 消解批 100：兰芷（2026-10-17）

- 日期：2026-10-17
- 范围：一技 deferred 消解（tag 439）——**134/184 技条目已映射**（deferred 49→48）
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（lanzhi 逐段核验）

## 语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 兰芷 lanzhi（环婉之 `swd_huanyuanzhi`） | `trigger: {player: "useCard"}`：使用**梅花牌**后，体力不大于自己的**已受伤**角色（含自己）按座位序各回复 1。含 `check`（可放弃） | `SwdLanzhi`（tag 439）=CardUsed 触发（`u.player==owner && !cancelled && 牌花色==Club`）；效果步按座位序对「存活 ∧ 已受伤 ∧ hp<=自己 hp」者 `ctx.recover(t,1,None)`。**恒发动近似**（上游可放弃）登记 |

## 验证

- **定向回归** `swd_deferred_lanzhi_heals_le_own_hp_damaged`：梅花酒（♣9，自身使用不造成伤害）→ 1/2/3 号（hp 3/2/3，均 <= 0 号 4）各回 1，0 号满血不回复；反例黑桃装备不触发。2 passed / 0 failed（[targeted.log](targeted.log)，含 b99 旧测回归）。
- **全量**：engine-native **882 passed / 0 failed / 0 ignored**（golden 零扰动）。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。
- **键表守护** `host_behavior_keys_bijective` 更新为 436（tag 439）。
- 包哈希：`packages/extensions/shahai-swd-upstream-noname` 的 `assets/cards.json`（lanzhi 映射）经 `rehash_npack` 重算 `source_hash`/`asset_hash`。

## 结论

- **134/184 映射，deferred 49→48**。`T12-SWD-DEFERRED-SKILLS` 仍开放（余 48 项）。
- 新增工具 `crates/content-runtime/examples/rehash_npack.rs`（手工维护包改动资产后的哈希重算）。
