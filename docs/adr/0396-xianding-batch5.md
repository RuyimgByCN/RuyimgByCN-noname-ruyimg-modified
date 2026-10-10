# ADR 0396：限定包批次五（逾制/长姬）

D 段（opencode）第五个批次：两个 XIANDING（限定包）技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI。

- 「逾制」（`dcyuzhi`，崔氏）：锁定技，成为【杀】的目标时，`card_targeted` +
  `owner_is_target_event_card_sha` 过滤器 + `increase_current_card_base_damage(self, event_card, 1)`
  令此【杀】伤害+1（怒斩先例）。旧 JS 的“弃置一张装备牌自动出闪”二选一分支未建模，
  以固定伤害+1 近似。
- 「长姬」（`dczhangji`，DC 清河公主）：锁定技，成为多目标牌的目标时，
  `card_targeted` + `owner_alive_targeted_by_other` 过滤器，`if 1 < event_target_count` 时
  `draw_cards(self, event_target_count - 1)` 摸 X 张牌（X 为此牌的其他目标数）。
  旧 JS 的“你先结算此牌效果”目标排序调整未建模。

两个技能均为独立移植（所属武将的另一技能 dchuashang 华裳 / dczengou 增购依赖自定义牌或
多步选牌，尚未完成，不提前写入 characters.json）。旧 JavaScript 不进入 npack 或运行时。
Golden 见 `original_xianding_batch5.rs`（4 例，全过）；`cargo run -p conformance` 全量通过。
编译遵循架构计划 §37.10：使用工作区外 `CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，
批次结束后执行 `cargo clean --target-dir` 并确认工作区 `target/` 不存在。
