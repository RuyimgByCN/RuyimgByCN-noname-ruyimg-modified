# ADR 0552：限定包批次三十（醮影·摸牌）

D 段（opencode）第三十个批次：XIANDING（限定包）技能「醮影·摸牌」以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「醮影·摸牌」（`jiaoying3_draw`，教缨子技能）：`phase_end` + `owner_is_target` +
  `choose_target_draw_to_max_hp_cap(self, 5)`——结束阶段可令一名角色将手牌摸至五张。
  旧 JS 的“手牌数<5 的目标约束”与“重复至没有合法目标”省略。

本技能从空壳占位转为实现。Golden 见 `original_xianding_batch30.rs`（2 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
