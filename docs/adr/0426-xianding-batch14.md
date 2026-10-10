# ADR 0426：限定包批次十四（武神/递仇）

> 后续状态：本批「递仇」近似实现已由 ADR 0489 / ABI 594 取代；稳定标记改为技能自身 ID，
> 回合开始、临时加成和结束阶段分支均已精确重写，并与「绍威」闭合 `v_guanyinping`。

D 段（opencode）第十四个批次：XIANDING（限定包）两个技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI。引擎 Rule VM ABI 已由其他 agent 从 451 提升至 521（本批次内容不涉及
新 ABI）；包 ABI 断言随之更新为 521。

- 「武神」（`dcwushen`，神关羽）：`view_as sha play_phase heart from hand ignore_distance
  no_use_count`——红桃手牌当【杀】使用且无距离与次数限制。旧 JS 为锁定技“红桃手牌自动视为
  【杀】”，DSL 以主动转化近似；“红桃【杀】造成伤害后摸一张牌”未建模（无红桃杀伤害过滤器）。
- 「递仇」（`dcdichou`，观银屏）：两个 trigger 复用“仇”标记
  （mark 3082515348 = sha256("noname-next/original/mark/dcdichou")）：
  ① `card_targeted` + `owner_is_target_other_source_alive` + `add_mark(event_source, …)`——
  其他角色对你使用牌时其获得 1 枚“仇”；③ `card_targeted` +
  `owner_source_target_mark_positive_3082515348` + `draw_cards(self,1)` +
  `remove_mark(event_player, …, 1)`——你对有“仇”目标的角色使用牌时摸一并移去其 1 枚“仇”。
  旧 JS 的②（回合开始废除装备栏并弃仇角色牌，废除装备栏无原语）与④（回合结束对有仇角色
  各造成 1 点火焰伤害，无标记约束的目标伤害原语）未建模。

两技能均属独立技（神关羽的梦魇 dcwuhun、观银屏另一技 dcshaowei 未完成），不提前写入
characters.json。旧 JavaScript 不进入 npack 或运行时。Golden 见
`original_xianding_batch14.rs`（3 例，全过）；`cargo run -p conformance` 全量通过。
编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
