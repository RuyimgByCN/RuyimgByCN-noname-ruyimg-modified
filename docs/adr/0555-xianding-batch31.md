# ADR 0555：限定包批次三十一（追还·二/沙舞）

D 段（opencode）第三十一个批次：XIANDING（限定包）两个技能以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「追还·二」（`zhuihuan2`，SP 周泰）：`damage` + `owner_is_target_other_source_alive_per_
  damage_point` + `if hp(self) < hp(event_source)` 条件分支——锁定技，受到伤害后若来源体力
  值大于你则对其造成 1 点伤害（`deal_damage(self, event_source, 1)`），否则其随机弃置一张
  手牌/装备牌（`discard_one_hand_or_equip(event_source)`）。DSL 仅支持 `==`/`<` 操作符，
  以反向比较表达“来源体力>你”。来源存活过滤器同时避免无来源伤害下 hp(event_source) 的
  InvalidId 查询。
- 「沙舞」（`shawu`，来莺儿）：`card_targeted` + `owner_is_target_event_card_sha` +
  `deal_damage(self, event_source, 1)`——成为【杀】的目标后可对此【杀】来源造成 1 点伤害。
  旧 JS 的“弃置两张手牌或移去一枚沙标记并摸两张牌”代价省略。

两个技能均从空壳占位转为实现。Golden 见 `original_xianding_batch31.rs`（3 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
