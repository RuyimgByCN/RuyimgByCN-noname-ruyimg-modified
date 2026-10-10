# ADR 0434：限定包批次十六（聚土/复学）

D 段（opencode）第十六个批次：XIANDING（限定包）两个技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI；包 ABI 保持 521。

- 「聚土」（`dcsbjutu`，界刘璋②）：`phase_end` + `owner_is_target_no_turn_damage` +
  `draw_cards(self,2)`——结束阶段若本回合未造成过伤害可摸两张牌。旧 JS 为“本回合未受到过
  伤害且手牌数不大于体力值”——过滤器为来源视角（DAMAGE_DEALT 标志，由当前回合角色造成
  伤害时设置），与目标视角略有偏移，已注明；“手牌数不大于体力值”条件省略；①（手牌数
  大于体力值时受伤害可弃一张红色牌防止伤害）无红牌防伤原语，未建模。
- 「复学」（`dcfuxue`，陆奕②）：`phase_end` + `owner_is_target` +
  `draw_cards(self, hp(self))`——结束阶段可摸体力值张牌（首次使用 hp() 表达式）。
  旧 JS 的“手牌区中没有复学①的牌”标记条件省略；①（准备阶段从弃牌堆选择获得至多 X 张
  非因使用进入弃牌堆的牌）未建模。

两技能均属独立技（界刘璋另两技 dcsbrenshan/dcsbyizhi、陆奕另两技 dcyaoyi/dcshoutan 未完成），
不提前写入 characters.json。旧 JavaScript 不进入 npack 或运行时。Golden 见
`original_xianding_batch16.rs`（4 例，全过）；`cargo run -p conformance` 全量通过。
编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
