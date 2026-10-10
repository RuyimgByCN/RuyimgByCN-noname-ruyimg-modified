# ADR 0582：限定包批次四十二（夺锐）

D 段（opencode）第四十二个批次：XIANDING（限定包）技能「夺锐」以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「夺锐」（`dcduorui`，DC 神张辽）：`damage_before` + `owner_is_source` +
  `modify_current_damage(self, -1)`——你造成伤害时可令此伤害 -1（复用 kotarou 的伤害修正
  管线）。旧 JS 的“获得目标一个技能直到其下回合结束”未建模。

本技能从空壳占位转为实现。Golden 见 `original_xianding_batch42.rs`（2 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
