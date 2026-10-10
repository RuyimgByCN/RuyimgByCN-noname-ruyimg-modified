# ADR 0408：限定包批次九（返攻/新膂力）

D 段（opencode）第九个批次：两个 XIANDING（限定包）技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI；本批引入 `min` 表达式（omp 的 ABI 451），包 ABI 提升至 451。

- 「返攻」（`dcsbfangong`，谋王平）：限定技，`active play_phase once_per_game` +
  `recover_hp(self, max_hp(self) - hp(self))` + `draw_cards(self, min(5, hand_count(self)))`，
  回复至体力上限并摸至多五张牌。与旧 JS 的 `recoverTo(maxHp)` + `draw(min(5, handCount))`
  一致。
- 「新膂力」（`xinlvli`，文鸯）：`damage` + `owner_source_once_per_turn` 过滤器 +
  `if hand_count(self) < hp(self)` 摸差值张牌 / `if hp(self) < hand_count(self)` 回复差值点
  体力。旧 JS 的“手牌数大于体力值且已受伤”条件中的“已受伤”未建模（回复分支可能触发
  于未受伤状态，由 `recover_hp` 上限自然截断）。

两个技能均为独立移植（所属武将的另一技能 dcsbyouyi 右移 / choujue 仇决需觉醒链与技能修改，
尚未完成，不提前写入 characters.json）。旧 JavaScript 不进入 npack 或运行时。Golden 见
`original_xianding_batch9.rs`（4 例，全过）；`cargo run -p conformance` 全量通过。编译遵循
架构计划 §37.10：使用工作区外 `CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，
批次结束后执行 `cargo clean --target-dir` 并确认工作区 `target/` 不存在。
