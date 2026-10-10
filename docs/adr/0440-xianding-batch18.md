# ADR 0440：限定包批次十八（忿肆/典财）

D 段（opencode）第十八个批次：XIANDING（限定包）两个技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI；包 ABI 保持 521。

- 「忿肆」（`fensi`，曹丕）：`phase_begin` + `owner_is_target` + `choose_other_damage(self,1)`——
  准备阶段可选一名其他角色并对其造成 1 点伤害。旧 JS 为锁定技且目标须体力不小于你、
  全场无体力不小于你的角色时改为对自己造成 1 点伤害——目标约束与自伤分支省略（无
  “体力≥你”过滤器），已注明。
- 「典财」（`diancai`，吕范）：`phase_end` + `owner_alive_lost_hand_this_turn` +
  `choose_target_draw_to_max_hp_cap(self, 5)`——其他角色的出牌阶段结束时，若你于此阶段失去
  过手牌，可将手牌摸至体力上限。旧 JS 的失去牌数阈值 X（X=体力值且至多为 5）省略（过滤器
  仅检查失去过牌）；cap=5 近似体力上限。

## 引擎修复（ABI 405 过滤器残留 bug）

`OwnerAliveLostHandThisTurn` 实现为 `owner_is_player && ctx.player != active.owner`——
两个条件矛盾永假，导致 `owner_alive_lost_hand_this_turn` 永不匹配（ABI 405 时代遗留，影响
本包 diancai 与 shiji 包 spdiancai）。已修复为 `!owner_is_player && ctx.player != active.owner`
（“其他角色的出牌阶段结束”语义）；并补登记 `track_turn_card_loss_history` 启用列表（此前
未登记导致 card_loss_count 永不更新，与批次十二 zhukou 同类遗漏）。standard 包不使用该
过滤器，conformance 全量复跑 ALL PASS。

Golden 见 `original_xianding_batch18.rs`（4 例，全过）。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
