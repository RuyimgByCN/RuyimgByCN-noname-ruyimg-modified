# ADR 0456：限定包批次二十一（守户/烈骧）

D 段（opencode）第二十一个批次：XIANDING（限定包）两个技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI；包 ABI 保持 521。

- 「守户」（`dcshouhu`，凌烈）：`active play_phase uses_per_turn 1 targets 1 1 other` +
  `choose_use_virtual_sha(self, ignore_distance)`——出牌阶段限一次，视为对一名其他角色使用
  一张无距离限制的【杀】（此前因 `view_as sha from none` 被 typecheck 拒绝而放弃，本次以
  active + choose_use_virtual_sha 表达）。旧 JS 的“结算后本回合你与其计算距离+1”、“若其因此
  濒死你摸体力上限张牌”、“本阶段有角色进入你的攻击范围时此技能视为未发动过”未建模。
- 「烈骧」（`dcliexiang`，SP 刘备）：`active play_phase uses_per_turn 1` +
  `draw_cards(self,5)` + `choose_targets_damage(self,1,1,other,1)`——出牌阶段限一次，摸至多
  五张牌，然后对一名其他角色造成 1 点伤害。旧 JS 的“摸至多五张”以摸五张近似；手牌数比较
  三分支（大于：杀次数+1 并交牌；等于：获牌且其下回合前 X 张牌无效；小于：失体并改限次）
  与后续联动未建模。

两技能均属独立技（凌烈另一技 dcningzhun、SP 刘备另一技 dcrengou 未完成），不提前写入
characters.json。旧 JavaScript 不进入 npack 或运行时。Golden 见
`original_xianding_batch21.rs`（3 例，全过）；`cargo run -p conformance` 全量通过。
编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
