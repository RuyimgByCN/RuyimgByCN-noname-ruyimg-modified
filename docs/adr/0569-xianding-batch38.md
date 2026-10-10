# ADR 0569：限定包批次三十八（孔武）

D 段（opencode）第三十八个批次：XIANDING（限定包）技能「孔武」以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「孔武」（`dcsbkongwu`，界呼厨儿）：`modifier draw_count add -1`——锁定技，你的摸牌
  阶段少摸一张牌（负数 modifier，先例 jsrglangan 的 `attack_range add -owner_skill_mark`）。
  旧 JS 的“装备牌失效至其下回合结束”部分未建模。

本技能从空壳占位转为实现。Golden 见 `original_xianding_batch38.rs`（2 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
