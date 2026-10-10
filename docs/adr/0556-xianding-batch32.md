# ADR 0556：限定包批次三十二（防毒/赏誉）

D 段（opencode）第三十二个批次：XIANDING（限定包）两个技能以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「防毒」（`dcfangdu`，DC 曹节）：`damage` + `owner_is_target_other_source_alive` +
  `lose_hp(event_source, 1)`——受到其他角色造成的伤害后可令伤害来源流失 1 点体力。
  旧 JS 的“令伤害来源选择：交给你一张牌 / 流失 1 点体力”二选一以固定流失体力近似。
- 「赏誉」（`dcshangyu`，许靖）：`game_started` + `owner_alive` +
  `gain_card_by_kind(self, sha)`——锁定技，游戏开始时获得一张【杀】。旧 JS 的“可将此牌
  交给一名角色、当一名角色使用此牌造成伤害后你与其各摸一张、此牌进入弃牌堆后将其交给
  一名本回合未以此法得到过此牌的角色”未建模。注：`game_started` 事件由
  `bootstrap_match` 在开局时触发（测试中不同 seed 分配的武将可能含其他 game_started 选择
  触发器，选择不与本技能冲突的 seed）。

两个技能均从空壳占位转为实现。Golden 见 `original_xianding_batch32.rs`（3 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
