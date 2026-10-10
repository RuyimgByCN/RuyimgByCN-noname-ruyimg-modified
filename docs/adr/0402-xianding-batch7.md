# ADR 0402：限定包批次七（虞袭/复盏）

D 段（opencode）第七个批次：两个 XIANDING（限定包）技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI。

- 「虞袭」（`dcyuxi`，张辽）：`damage` + `owner_is_source_or_target` 过滤器 +
  `draw_cards(self, 1)`，造成或受到伤害时可摸一张牌，与旧 JS 的 source/player 双触发语义一致。
  旧 JS 的“以此法获得的牌无次数限制”（gaintag + 临时 modifier）未建模。
- 「复盏」（`dcsbfuzhan`，谋姜维）：`dying_after` + `owner_alive_not_target` 过滤器 +
  `recover_hp(self, max_hp(self) - hp(self))`，有角色脱离濒死时回复至体力上限。旧 JS 的
  “然后修改绝谋”分支与限定技整局一次（`awakenSkill`）未建模。

两个技能均为独立移植（所属武将的另一技能 dcporong 破绒 / dcsbjuemou 绝谋依赖连招历史或
转换技，尚未完成，不提前写入 characters.json）。旧 JavaScript 不进入 npack 或运行时。
Golden 见 `original_xianding_batch7.rs`（4 例，全过）；`cargo run -p conformance` 全量通过。
编译遵循架构计划 §37.10：使用工作区外 `CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，
批次结束后执行 `cargo clean --target-dir` 并确认工作区 `target/` 不存在。
