# ADR 0494：限定包批次二十三（清棱/接缨）

D 段（opencode）第二十三个批次：XIANDING（限定包）两个技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI。引擎 Rule VM ABI 已由其他 agent 从 521 提升至 597（本批次内容不涉及
新 ABI）；包 ABI 断言随之更新（现有测试统一为 `>= 587` 软断言）。

- 「清棱」（`dcqingleng`，董?）：`card_targeted` + `owner_is_target_nonred_card` +
  `choose_other_discard_target_card(self, hand_equip)`——你成为其他角色使用黑色牌的目标后
  可弃置一名角色一张牌（两步选择：先选目标、再选牌）。旧 JS 的“弃置装备牌则此牌对你无效 /
  非装备则令你下次〖酖毒〗使用牌可视为使用此牌”分支未建模（酖毒联动依赖记录机制）。
- 「接缨」（`dcjieying`，神刘备）：`phase_end` + `owner_is_target` +
  `choose_link_other(self)` + `draw_linked_players(1)`——结束阶段可横置一名其他角色并令所有
  已横置角色（MARK_CHAIN 标记）各摸一张牌。旧 JS 的“锁定技，你始终处于横置状态”与
  “已横置角色手牌上限+2”未建模。

注：另一 agent（A 段）自 2026-08-03 起并发移植 xianding 包其余技能（新增约 21 个技能文件、
characters.json 扩展至 16 武将、ABI 597）——本批次打包结果（85 技能）包含双方内容，
extensions 同步一致。Golden 见 `original_xianding_batch23.rs`（3 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
