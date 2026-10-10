# ADR 0442：限定包批次十九（立刃/讽言）

D 段（opencode）第十九个批次：XIANDING（限定包）两个技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI；包 ABI 保持 521。

- 「立刃」（`liren`，施蛾）：`modifier attack_range override 2` + `sha_dodged` +
  `owner_source_sha_dodged` + `draw_cards(self,1)`——攻击范围始终为 2；你的【杀】被抵消后
  摸一张牌。旧 JS 的②“【杀】造成伤害后获得此【杀】”无基础杀伤害过滤器未建模；
  ④“每回合至多以此法获得 3 张牌”限次省略。
- 「讽言」（`dcfengyan`，丁尚婉①）：`active play_phase uses_per_turn 1 targets 1 1
  hp_lte_owner` + `take_one_card(event_source, self, hand_equip)`——出牌阶段限一次，选择
  一名体力值不大于你的其他角色并令其交给你一张手牌（以获其一张手牌/装备牌近似）。
  旧 JS 的②“视为对一名手牌数不大于你的角色使用一张【杀】”无对应目标约束原语未建模。

两技能均属独立技（施蛾另一技 sejianchu 觉醒、丁尚婉另一技 dcfudao 未完成），不提前写入
characters.json。旧 JavaScript 不进入 npack 或运行时。Golden 见
`original_xianding_batch19.rs`（3 例，全过）；`cargo run -p conformance` 全量通过。
编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
