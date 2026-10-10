# ADR 0533：限定包批次二十五（义指）

D 段（opencode）第二十五个批次：XIANDING（限定包）技能「义指」以既有 DSL 原语组合表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「义指」（`dcsbyizhi`，界刘璋主公技）：`view_as wugu play_phase any from hand uses_per_turn 1`
  + `lord_only`——出牌阶段限一次视为对群势力角色使用一张【五谷丰登】。
  `wugu` 已随引擎扩展加入 VIEW_AS_KINDS（此前评估时缺失而放弃）；旧 JS 为无牌代价
  （selectCard 0），此处以弃一张手牌近似（`from hand`）；群目标约束由五谷丰登自身的
  多目标结算处理。

注：xianding 包现已 100% 技能名覆盖（488 个 .skill 文件 / 504 个技能定义 / 196 武将 /
ABI 607，实现与空壳占位并存，A 段主导）。本批次把 `dcsbyizhi` 从空壳转为实现。
Golden 见 `original_xianding_batch25.rs`（2 例，全过）；`cargo run -p conformance` 全量通过。
编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
