# ADR 0568：限定包批次三十七（毒露/猝袭）

D 段（opencode）第三十七个批次：XIANDING（限定包）两个技能以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「毒露」（`dcdulu`，DC 曹节）：`active play_phase uses_per_turn 1 targets 1 3 other` +
  `deal_damage(self, event_source, 1)`——出牌阶段限一次，对至多三名其他角色各造成 1 点
  伤害。旧 JS 的“选择本回合对你使用过牌的角色”历史筛选与“随机弃一张手牌、其濒死时你
  获得其黑色牌”省略。
- 「猝袭」（`dccuxi`，DC 曹节）：`phase_begin` + `owner_alive_not_target` +
  `discard_hand(self,2)` + `deal_damage(self, event_player, 1)`——其他角色的回合开始时可
  弃置两张手牌并对其造成 1 点伤害。旧 JS 的“令其选择：让你执行其本回合摸牌阶段 / 受到你
  造成的 1 点伤害”二选一以固定造成 1 点伤害近似。

两个技能均从空壳占位转为实现。Golden 见 `original_xianding_batch37.rs`（3 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
