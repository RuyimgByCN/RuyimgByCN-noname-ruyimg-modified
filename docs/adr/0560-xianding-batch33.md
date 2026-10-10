# ADR 0560：限定包批次三十三（识锦/斗威）

D 段（opencode）第三十三个批次：XIANDING（限定包）两个技能以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「识锦」（`dcsbshijin`，界邓艾）：`damage_before` + `owner_is_target` +
  `draw_cards(self,1)` + `prevent_current_damage()`——锁定技，当你受到伤害时防止此伤害并
  摸一张牌。注：`prevent_current_damage()` 无参数；事件须用 `damage_before`（测试用
  `DealDamage` 效果走完整伤害流程触发）。旧 JS 的“限定技”属性与“你的下个回合开始时
  弃置所有【杀】和锦囊牌并失去等量体力、若未因此失去体力则此技能视为未发动过”未建模。
- 「斗威」（`dcsbdouwei`，界董承）：`dying` + `owner_alive_other_dying_once_per_round` +
  `recover_hp(self,1)`——每轮限一次，一名角色进入濒死状态时可回复 1 点体力。旧 JS 的
  “因你【斗围】进入濒死”来源条件与“恢复一个装备栏”省略。

两个技能均从空壳占位转为实现。Golden 见 `original_xianding_batch33.rs`（3 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
