# ADR 0404：限定包批次八（心忧/鸟翔）

D 段（opencode）第八个批次：两个 XIANDING（限定包）技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI。

- 「心忧」（`dcxinyou`，谢灵运）：`active play_phase uses_per_turn 1` +
  `recover_hp(self, max_hp(self) - hp(self))` + `adjust_hand_to(self, max_hp(self))`，
  出牌阶段限一次将体力回复至上限并将手牌补至体力上限。旧 JS 的结束阶段副作用
  （获得至少三张牌则失去体力 / 回复体力则弃一张牌）依赖本回合获得牌数与回复历史，
  未建模。
- 「鸟翔」（`dcniaoxiang`，DC 蒋钦）：锁定技，`modifier sha_response_count_if_source_owner_target_distance_le_one
  add 1`，使用【杀】指定目标后若来源与目标距离 ≤ 1 则目标需要额外使用一张【闪】。
  旧 JS 的“目标在拥有者攻击范围内”以距离 ≤ 1 近似（同 zj_duanbing 先例）。

两个技能均为独立移植（所属武将的另一技能 dcyuandi 渊抵需出牌阶段首次用牌全局事件、
dcshangyi 尚义需选目标弃牌，尚未完成，不提前写入 characters.json）。旧 JavaScript 不进入
npack 或运行时。Golden 见 `original_xianding_batch8.rs`（2 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。

注：候选「点才」（diancai）因引擎 `owner_alive_lost_hand_this_turn` 过滤器（ABI 405）存在
`owner_is_player && ctx.player != active.owner` 矛盾条件且 turn-card-loss 历史 flag 未启用，
本批放弃，待引擎修复后另行移植。
