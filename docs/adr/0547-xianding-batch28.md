# ADR 0547：限定包批次二十八（恩遇）

D 段（opencode）第二十八个批次：XIANDING（限定包）技能「恩遇」以既有 DSL 原语组合表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「恩遇」（`dcenyu`，DC 王允）：`card_targeted` + `owner_is_target` +
  `draw_cards(self,1)` + `choose_give_hand_equip_to_event_player(self)`——锁定技，你成为
  其他角色使用牌的目标后摸一张牌，并可交给此牌使用者一张牌（交牌选择沿用 ABI 459 的
  Custom continuation 分派）。旧 JS 的“每回合每项各限一次”限次省略。

本技能从空壳占位转为实现。Golden 见 `original_xianding_batch28.rs`（2 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
