# ADR 0583：限定包批次四十三（奇绝/毒逝·诅咒）

D 段（opencode）第四十三个批次：XIANDING（限定包）两个技能以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「奇绝」（`dcsbqijue`，界杨奉）：`dying` + `owner_alive_other_dying_once_per_round` +
  `choose_target_draw(self,2,any)` + `choose_recover_target(self,event_player,2)`——限定技，
  有角色进入濒死状态后可令其摸两张牌并回复 2 点体力（dcjinjie 模式）。旧 JS 的“将
  〖烛帛〗中的造成替换为受到、若濒死者是你则修改时机”未建模。
- 「毒逝·诅咒」（`dushi2`，弘农王）：`modifier card_target_cancel(tao) override 1`——
  锁定技，你不能成为其他角色【桃】的目标（复用 nanman 的 card_target_cancel 管线）。

两个技能均从空壳占位转为实现。Golden 见 `original_xianding_batch43.rs`（3 例，全过）；
`cargo run -p conformance` 全量通过。注：本批期间 standard 包被其他 agent 修改（角色池变化），
批次 9/30/40 的三个 seed 敏感测试已换 seed 适配。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
