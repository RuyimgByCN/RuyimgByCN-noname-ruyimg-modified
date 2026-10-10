# ADR 0399：限定包批次六（沉勇/绞）

D 段（opencode）第六个批次：两个 XIANDING（限定包）技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI。

- 「沉勇」（`dcchenyong`，陈泰）：出牌阶段结束时，`phase_end` +
  `owner_is_target_play_phase_card_category_count_positive` 过滤器 +
  `draw_by_play_phase_card_category_count(self)` 按本出牌阶段使用过的逻辑牌类别数摸牌。
  与旧 JS 的 `getHistory("useCard")` 类别去重计数语义一致（引擎以权威阶段历史提供类别位）。
- 「绞」（`dclingfang`，董贵人）：锁定技，`phase_prepare` + `owner_is_target` 或
  `card_targeted` + `owner_is_target_nonred_card` 时 `add_mark(self, def, 1)` 获得 1 枚“绞”
  标记。旧 JS 的“自己使用黑色牌结算后且不是目标”分支未建模。

两个技能均为独立移植（所属武将的另一技能 dcctjiuxian 急贤需半数重铸+视为决斗的动态选牌、
dclianzhi 连枝需开局私密选角，尚未完成，不提前写入 characters.json）。旧 JavaScript 不进入
npack 或运行时。Golden 见 `original_xianding_batch6.rs`（3 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
