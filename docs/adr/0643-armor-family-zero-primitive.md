# ADR 0643：护甲族立项解散——零原语迁移（MARK_HUJIA 复用）

- 状态：**已接受（2026-08-21）**
- 背景：勘察批（delta 2026-08-21 第五批）评估护甲族 5 技需「Player 护甲字段 +
  值源/获得两类原语」的 ADR 级立项。本批实施前侦查证伪了该假设。

## 1. 发现：护甲系统已在引擎内完备

| 底座 | 位置 | 现状 |
|---|---|---|
| `MARK_HUJIA = 3` | `crates/engine-core/src/content/mod.rs:28` | 护甲=玩家标记（def 3），非 Player 字段 |
| 伤害吸收 | `effect/primitives/pack_mobile.rs` `commit_damage` | 上游 content.ts:12261 等价实现：仅 damage（非失去体力）吸收 `min(伤害, 护甲)`，全额吸收发 `DamagePrevented` |
| 授予 | `add_mark(tx, player, MARK_HUJIA, n)` | mbzhuji（兵势孙韶筑墼）已在生产使用，含上限 5 约定 |
| 值源 | `owner_mark_<def>` 参数化值源 | `content/registry.rs`——`owner_mark_3` 即护甲计数 |

**结论**：护甲族迁移不需要 Player 字段（规避了「字面量构造全量波及」）、不需要
新原语、不需要 ABI 领号。与 ADR 0641 §6d（判定改写引擎原生、铸造解体）同型。

## 2. 本批迁移（5/6 技，零 ABI：mobile 500 / key 651 / shahai 645 均不变）

| 技能 | 包 | 实现 | 近似 |
|---|---|---|---|
| yuxiang 玉箱 | mobile | `distance_from add -owner_mark_3` + `distance_to add owner_mark_3`（无护甲时值源为 0 自动失效） | 上游布尔 ±1，此处 ±护甲值（≥2 偏强）；火焰伤害+1 子句无受击火焰过滤器未建模 |
| erika_shisong 诗诵 | key | `hand_limit add owner_mark_3` | 同位序同类型摸牌子句未建模 |
| tomoyo_changshi 长视 | key | `hp_recovered owner_alive → add_mark(self,3,1)` | 「攻击范围内」条件未建模；获得牌≥2 摸牌子句未建模 |
| zhanhou | shahai | `active uses_per_turn 1 → 弃 1 + add_mark(self,3,1)` | 所弃未限定防具 |
| oldzhanhou | shahai | 同上无次数限制 | 同上 |

**遗留 1 项**：mbzhenfu「结束阶段，若本回合因弃置失去过牌，令一名其他角色获得
1 点护甲」——需「选一名其他角色后授予标记」的组合原语（choose_other_gain_armor）
与「本回合因弃置失去过牌」过滤器，两者现无；待 ≥3 同构需求出现再立项
（「己方任意装备失牌」过滤器缺口已在第八批 delta 登记，可合并考虑）。

## 3. 方向映射备查（globalFrom/globalTo）

上游 `globalFrom`（拥有者出向）↔ DSL `distance_from`；`globalTo`（拥有者入向）
↔ DSL `distance_to`（与 rebiluan2/caomao 先例一致）。

## 4. 验收

- Golden：`original_armor_family_zero_primitive.rs`（三包契约 + 长视行为链）。
- 全量回归：952 测试二进制 0 失败。
- npack：mobile（modifiers 247→248）、key（modules 107→108 / triggers 90→91 /
  modifiers 179→178）、shahai（modules 37→39 / modifiers 1062→1060 / actives +2）。
