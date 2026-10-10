# ADR 0390：限定包批次四（安境/伪涕）

D 段（opencode）第四个批次：两个 XIANDING（限定包）技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI。

- 「安境」（`dcanjing`，胡遵）：造成伤害后可选触发，`choose_optional_targets_draw(self, 1, 1,
  255, wounded)` 令至多 X 名已受伤角色各摸一张牌（X 为本局发动次数，目标数上限以 255 近似），
  再 `choose_other_recover(self, 1)` 令其中一名角色回复 1 点体力。旧 JS 的“体力值最少的随机
  一名”以任选一名回复近似。
- 「伪涕」（`dcweiti`，吴质）：出牌阶段限两次，`active play_phase uses_per_turn 2` +
  `choose(0,1)` 二选一：`choose_other_damage(self, 1)` 令一名角色受到 1 点伤害，或
  `choose_other_recover(self, 1)` 令一名角色回复 1 点体力。旧 JS 的“获得两张点数不同牌 / 弃置
  两张点数不同牌”联动未建模。

两个技能均为独立移植（所属武将的另一技能 dczhantao 斩涛 / dcyuanrong 圆融需要判定+点数
比较、弃牌堆转化用牌等复杂机制，尚未完成，不提前写入 characters.json）。旧 JavaScript 不进入
npack 或运行时。Golden 见 `original_xianding_batch4.rs`（4 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
