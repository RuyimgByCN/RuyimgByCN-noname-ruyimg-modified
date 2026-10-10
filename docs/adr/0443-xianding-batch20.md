# ADR 0443：限定包批次二十（酒盾/全守）

D 段（opencode）第二十个批次：XIANDING（限定包）两个技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI；包 ABI 保持 521。

- 「酒盾」（`dcjiudun`，阮籍②）：`card_targeted` + `owner_is_target_nonred_card` +
  `draw_cards(self,1)` + `use_virtual_card_on(self, jiu, self)`——成为其他角色使用黑色牌的
  目标后可摸一张牌并视为使用一张【酒】。旧 JS 的“未处于【酒】状态”条件（酒状态标记无
  原语）与①（以你为目标的【酒】作用效果改写）未建模。
- 「全守」（`dcquanshou`，孙玉①）：`phase_begin` + `owner_alive_not_target` +
  `choose_target_draw_to_max_hp_cap(self, 5)`——其他角色的回合开始时可令其将手牌摸至体力
  上限（至多摸五张）。旧 JS 的“手牌数不大于体力上限”条件与二选一（摸至上限-杀次数或
  使用牌被抵消后你摸）未建模。

两技能均属独立技（阮籍另一技 dczhaowen、孙玉另一技 dcshexue 未完成），不提前写入
characters.json。旧 JavaScript 不进入 npack 或运行时。Golden 见
`original_xianding_batch20.rs`（3 例，全过）；`cargo run -p conformance` 全量通过。
编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
