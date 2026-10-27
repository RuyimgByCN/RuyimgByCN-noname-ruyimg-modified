# PA12 迁移批④-3：伤害改写对 `prevent_current_damage` / `modify_current_damage` 接入原生（2026-10-27）

- 执行时间：2026-10-27 **13:44:23 → 14:00 +08:00**（`HEAD = a7ab0d88`；裁决问询未获答复，按计划顺序小步代理开工；真机复测仍 0 台接入）；执行者 ZCode A 道
- 工作包：[迁移计划](../../../architecture/native-dsl-migration-plan.md) §3 批④第三步（79 技能）

**一句话结论**：DSL `prevent_current_damage()`（tag → `EffectRequest::PreventDamage{damage_id:0}`，绑定当前前置窗口伤害帧）与 `modify_current_damage(self, ±N)`（tag → `ModifyCurrentDamage{delta}`，正值增伤/负值减伤钳制到 0，解锁 lianhuo/dckanghui/renjia 族）接入原生：`accept_effect` 允许（PreventDamage 无参；Modify 校验座位）+ `commit_effect` **与宿主技 v187（张目 +1 增伤）/v126（swd prevented）同法直写当前伤害帧 `DamageData`**——只在 DamageBefore 前置窗口内生效（`current_damage()` 取不到当前伤害帧时静默空操作），不新建事件、不改伤害管线。真实技能全链：刺背（collab，防止+摸 5——目标 hp 不变）、kotarou 改写（key，+1——1 伤变 2 伤）。

## 1 验收（真实 ported .skill 全链 4/4 含前两步回归）

| 测试（`pa12_batch4_1_host_primitives.rs`） | 断言 |
|---|---|
| `pa12_batch4_3_damage_rewrite_window` | ① 刺背（`damage_before owner_is_source optional`）：确认后目标 hp 不变（防止）+ 摸 5；② kotarou（`damage_before owner_is_source`）：来源伤害 1 → 扣 2 |
| 批④-2/④-1 回归 | 消面弃 1、N=0 拒绝、雪堕翻面/E03 全不受影响 |

## 2 变更与回归

- `rvm_ops.rs`：允许面两臂 + commit 两臂（`current_damage_mut()` 直写 `prevented`/`amount`；负 delta 钳 0——中途一度误用 `.max(0)` 钳制已撤，负值有效依变体文档）；头注更新。
- 回归（[b43 汇总](../2026-10-26-pa12-dsl-native-prototype/b43-regressions.txt)，13:55–13:59）：engine-native **1580/0/71**（+1）、loader 32/0、wasm 52/0、game-server 358/0、engine-core 3841/0/1。

## 3 边界与后续

- `ChooseDiscardEquipmentPreventCurrentDamage`（tag 别名、带弃装备成本）未接（需 N=0 同款选择链）；非伤害窗口发射两变体＝静默空操作（旧线同 damage_id=0 绑定语义）。
- 盘点口径注记：文件级近似 1,098 不变（该对原语的技能普遍组合其它阻塞原语/事件）；技能级解锁量以 [批④-1 附表](../2026-10-27-pa12-batch4-1-turn-over/skill-level-survey.txt) 44 个「damage_before+已映射滤镜+其余效果已承载」候选为准（本批全部可装载运行）。
- 批④下一优先：N=0 选牌首发链（discard 族交互选牌）。
