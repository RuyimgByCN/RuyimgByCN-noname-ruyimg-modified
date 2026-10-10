# ADR 0414：限定包批次十一（独仗/风势）

D 段（opencode）第十一个批次：两个 XIANDING（限定包）技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI；包 ABI 保持 451。

- 「独仗」（`dcduzhang`，神钟会）：锁定技，`card_targeted` + `owner_is_target_nonred_card`
  过滤器，`add_mark` + `draw_cards(self,1)`，手牌上限 `hand_limit add owner_skill_mark`。
  旧 JS 的“使用黑牌指定唯一目标”场景未建模（仅覆盖“成为非红牌目标”），且“唯一目标”
  约束无 DSL 表达；“凛”标记复用 dclinjie 的定义。
- 「风势」（`dcfengshi`，DC 张任）：`card_targeted` +
  `owner_source_sha_target_alive_owner_outside_target_attack_range` 过滤器（精确对应旧 JS 的
  “你不在其攻击范围内”条件），`choose_discard_target_card(self, event_source, equip)`。
  旧 JS 的“防具或防御马”约束以“装备”近似（DSL 弃装备 scope 无防具专用变体）。

两个技能均属已评估武将的独立技（神钟会的凛界 dcjianghuo、DC 张任的其它技能未完成，
不提前写入 characters.json）。旧 JavaScript 不进入 npack 或运行时。
Golden 见 `original_xianding_batch11.rs`（3 例，全过）；`cargo run -p conformance` 全量通过
（identity_8_99 由改引擎方重钉，非本批次内容所致）。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
