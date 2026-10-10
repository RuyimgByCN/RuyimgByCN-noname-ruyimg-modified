# ADR 0567：限定包批次三十六（华衣）

D 段（opencode）第三十六个批次：XIANDING（限定包）技能「华衣」以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「华衣」：`phase_end` + `owner_alive_not_target` + `draw_cards(self,1)`——一名角色的回合
  结束时你摸一张牌。旧 JS 的“结束阶段判定：红色则直到你的下回合开始每名角色回合结束时
  摸一张、黑色则受伤后摸两张”（判定分支+跨回合效果）以固定“其他角色回合结束时摸一张”
  近似。

本技能从空壳占位转为实现。Golden 见 `original_xianding_batch36.rs`（2 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
