# ADR 0590：限定包批次四十八（迅势/据谏）

D 段（opencode）第四十八个批次：XIANDING（限定包）两个技能以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「迅势」（`dcsbxunshi`）：`locked` + `modifier card_use_unlimited(sha_thunder)` +
  `card_distance_unlimited(sha_thunder)`（雷【杀】无次数/距离限制）+ `trigger damage
  owner_source_thunder` + `discard_one_hand_or_equip(event_player)` +
  `draw_cards(self, 2)`（雷【杀】伤害后目标弃一张 he、你摸两张）。旧 JS 的
  「目标随机弃置」以目标自选弃一张近似（注释已注明）。
- 「据谏」（`dcjujian`）：`lord_only` + `active play_phase uses_per_turn 1 targets 1 1
  other` + `draw_selected_targets(1)` + `ban_use_against(event_source, self)`——主公技，
  出牌阶段限一次，令一名其他角色摸一张牌，然后其本回合不能对你使用牌。旧 JS 的
  「魏势力」目标限定无对应原语（任意其他角色近似）；「不能使用锦囊牌」以
  ban_use_against 全牌禁用近似（注释已注明）。

新发现原语：`draw_selected_targets(count)`（配 `targets` 声明令选中目标摸牌）、
`ban_use_against(user, target)`（回合级 use_ban_pairs，禁用 user 对 target 用牌）、
`owner_source_thunder` 过滤器、selector `sha_thunder`。Golden 见
`original_xianding_batch48.rs`（3 例，全过）；全量回归（批次 1~47）与
`cargo run -p conformance` 全过。编译遵循架构计划 §37.10：工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后删除外部与
工作区 `target/`。
