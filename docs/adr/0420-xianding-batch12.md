# ADR 0420：限定包批次十二（诛寇）

D 段（opencode）第十二个批次：XIANDING（限定包）技能「诛寇」以既有 DSL 原语组合表达，
不新增 Rule VM ABI；包 ABI 保持 451。

- 「诛寇」（`zhukou`，周夷分支②）：`phase_end` + `owner_alive_phase_end_used_card_no_damage`
  过滤器（复用）+ `choose_targets_damage(self, 2, 2, other, 1)`（对两名其他角色各造成 1 点
  伤害）。旧 JS 条件为“本回合未造成伤害”（无“使用过牌”要求），近似过滤器
  `owner_alive_phase_end_used_card_no_damage` 额外要求本回合使用过牌——本回合未使用任何牌时
  漏触发，已注明。分支①（出牌阶段首次造成伤害后摸已用牌数）与周夷另一技 mengqing（觉醒，
  需“场上受伤角色数>体力值”过滤器，不存在）未建模。

## 引擎修正（register track 需求）

`owner_alive_phase_end_used_card_no_damage` 依赖 `turn_flag::card_used_by_player`，但
`content/registry.rs` 的 `track_per_player_turn_card_count` 启用列表未登记该过滤器，导致该
flag 永不被设置、技能永不触发（同过滤器也被 `content/original/ported/tw/twfeiyi.skill`
使用，同样受影响）。已在 `registry.rs` 将该过滤器加入
`track_per_player_turn_card_count` 启用列表。standard 包无此过滤器，turn flag 不会在其
对局中被设置，故 canonical state hash 不受影响——全量 conformance 复跑 ALL PASS。

Golden 见 `original_xianding_batch12.rs`（3 例，全过）；`cargo run -p conformance` 全量通过。
编译遵循架构计划 §37.10：使用工作区外 `CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，
批次结束后执行 `cargo clean --target-dir` 并确认工作区 `target/` 不存在。
