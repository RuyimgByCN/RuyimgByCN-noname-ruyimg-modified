# ADR 0432：限定包批次十五（连枝/杜君）

D 段（opencode）第十五个批次：XIANDING（限定包）两个技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI；包 ABI 保持 521。

- 「连枝」（`dclianzhi`，董贵人②）：`dying` + `owner_is_target` + `recover_hp(self,1)` +
  `draw_cards(self,1)`——你进入濒死状态时可回复 1 点体力并摸一张牌。旧 JS 为“与连枝①所选
  角色各摸一张牌”且每回合限一次——连枝①（游戏开始秘密选角，无选角原语）与③（连枝角色
  死亡后授予受责）未建模。
- 「杜君」（`dcdujun`，夏侯徽②）：`damage` + `owner_source_or_target_first_damage_this_turn`
  （拥有者是伤害来源或目标且为本回合首次伤害）+ `draw_cards(self,2)`——每回合首次造成或
  受到伤害后可摸两张牌。旧 JS 的“连枝角色”与“然后可以将牌交给一名角色”未建模。

两技能均属独立技（董贵人另两技 dclingfang/dcfengying、夏侯徽另技 dcjikun 未完成），不提前
写入 characters.json。旧 JavaScript 不进入 npack 或运行时。Golden 见
`original_xianding_batch15.rs`（4 例，全过）；`cargo run -p conformance` 全量通过。
编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
