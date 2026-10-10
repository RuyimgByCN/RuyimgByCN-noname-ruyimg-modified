# ADR 0571：限定包批次三十九（困欲/矜节）

D 段（opencode）第三十九个批次：XIANDING（限定包）两个技能以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「困欲」（`kunyu`，谋袁谭）：`dying_after` + `owner_is_target` + `recover_hp(self,1)`——
  锁定技，当你进入濒死被救回时回复 1 点体力。旧 JS 的“移出游戏一张火焰伤害牌”与“你的
  体力上限锁定为 1”（无 max_hp modifier）省略。
- 「矜节」（`oldjinjie`，旧版）：`dying` + `owner_alive_other_dying_once_per_round` +
  `choose_target_draw(self,1,any)` + `choose_recover_target(self,event_player,1)`——其他角色
  进入濒死状态时可令其摸一张牌并回复 1 点体力（dcjinjie 模式）。旧 JS 的“若你已于本回合
  行动过”条件与“非首次发动：弃置等量张牌令其回复”分支省略。

两个技能均从空壳占位转为实现。Golden 见 `original_xianding_batch39.rs`（3 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
