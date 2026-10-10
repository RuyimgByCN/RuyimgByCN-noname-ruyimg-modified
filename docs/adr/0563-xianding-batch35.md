# ADR 0563：限定包批次三十五（弹翼）

D 段（opencode）第三十五个批次：XIANDING（限定包）技能「弹翼」以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「弹翼」（`dcdanyi`，DC 曹节）：`card_used` + `owner_source_play_card_repeats_previous_
  targets` + `draw_cards(self,1)`——使用牌指定目标后，若此牌与上一张牌有共同目标，可摸
  一张牌。旧 JS 的“摸共同目标数张牌（等量）”以固定摸一张近似（目标交集计数无原语）。

本技能从空壳占位转为实现。Golden 见 `original_xianding_batch35.rs`（2 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
