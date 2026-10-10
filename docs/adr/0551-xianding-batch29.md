# ADR 0551：限定包批次二十九（清识/残肆）

D 段（opencode）第二十九个批次：XIANDING（限定包）两个技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「清识」（`dcdyqingshi`，DC 曹节）：`card_used` + `owner_source_single_target_sha_or_
  ordinary_trick_current_turn` + `choose_targets_damage(self,1,1,any,1)`——回合内使用【杀】
  或普通锦囊牌指定唯一目标后可对一名角色造成 1 点伤害。旧 JS 的“目标角色限此牌的目标”
  与“此牌为你本回合使用的第 X 张牌（X=手牌数）”条件省略。注：事件必须用 `card_used`
  （过滤器内部检查 EventKind::CardUsed），card_targeted 会永假。
- 「残肆」（`dccansi`，DC 神吕布）：`phase_begin` + `owner_is_target` +
  `recover_hp(self,1)` + `choose_other_recover(self,1)`——锁定技，准备阶段回复 1 点体力并
  选择一名其他角色回复 1 点体力。旧 JS 的“依次对其使用【杀】【决斗】【火攻】”与“其受到
  伤害后每点伤害摸两张牌”未建模。

两个技能均从空壳占位转为实现。Golden 见 `original_xianding_batch29.rs`（3 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
