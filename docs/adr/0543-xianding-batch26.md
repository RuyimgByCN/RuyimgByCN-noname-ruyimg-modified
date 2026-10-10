# ADR 0543：限定包批次二十六（溃击/膂力·闪/膂力·无懈）

D 段（opencode）第二十六个批次：XIANDING（限定包）三个技能以既有 DSL view_as 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「溃击」（`kuiji`）：`view_as bingliang play_phase black_basic_equip from hand_equip
  uses_per_turn 1`——出牌阶段限一次将一张黑色基本/装备牌当【兵粮寸断】使用（复用断粮的
  bingliang view_as 管线）。旧 JS 的“置于自己判定区后摸一张牌并对体力最大的敌方角色造成
  2 点伤害、目标濒死时可选择体力最大的己方角色回复 1 点体力”未建模（self-judge 约束与
  最大体力目标过滤缺失）。
- 「膂力·闪」（`lvli4`）：`view_as shan any any from hand_equip`——任意牌当【闪】使用或
  打出。膂力主技能的总次数联动省略（独立移植）。
- 「膂力·无懈」（`lvli5`）：`view_as wuxie any any from hand_equip`——任意牌当【无懈可击】
  使用。膂力主技能的总次数联动省略。

三个技能均从空壳占位转为实现。Golden 见 `original_xianding_batch26.rs`（2 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
