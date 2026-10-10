# ADR 0572：限定包批次四十（势援）

D 段（opencode）第四十个批次：XIANDING（限定包）技能「势援」以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「势援」：`card_targeted` + `owner_is_target_other_source_alive` +
  `if hp(self) < hp(event_source)` 条件分支——成为其他角色使用牌的目标后：若来源体力值大于
  你则摸三张牌、否则摸一张牌。旧 JS 的“摸 (2 + sgn(来源体力 - 你的体力)) 张”以 3/1 两档
  近似；“当前回合角色是群势力且你拥有〖隅泣〗时每回合至多两次”省略。

本技能从空壳占位转为实现。Golden 见 `original_xianding_batch40.rs`（2 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
