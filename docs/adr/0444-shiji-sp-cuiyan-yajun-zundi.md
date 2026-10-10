# ADR 0444：SHIJI 崔琰闭环（压军 / 尊帝，ABI 428 / 429）

本批发布 SHIJI 崔琰（`sp_cuiyan`，3918645328，男/魏/3 血）。技能「压军」
（`spyajun`，ABI 428）、「尊帝」（`spzundi`，ABI 429）完成闭环；旧 JavaScript
仅作语义来源。

## 决策

- **ABI 428 `resolve_spyajun(player, less_def, card, target, target_card, stage, skill)`**：
  出牌阶段开始时（`phase_play` + 新过滤器
  `owner_alive_hand_gained_this_turn_can_compare_other`，改为按
  `turn.gained_hand_cards` 实际判定），选一张本回合新获手牌与一名其他角色拼点；
  赢则可选将拼点牌置于牌堆顶，输则加“戒”标记（`AddMark`，标记键 = 辅助技能
  `spyajun_less` 的 def id）。五段 choice 状态机：
  stage 0 选新获牌 → 1 选目标 → 2 目标选拼点牌 → 3 提交（复用
  `hand_compare_commit_context` 判定胜负：展示、双方拼点牌入弃牌堆、
  `EmitHandCompareFinished`；胜者开可选置顶 choice）→ 5 置顶
  （`MoveCards` 至 DrawPile 顶部）。旧 JS 允许置顶两张拼点牌任一张，
  以“自己的拼点牌置顶”为文档化近似。
- **纯 DSL 辅助技能**：`spyajun_less`（`modifier hand_limit add -owner_skill_mark`，
  手牌上限减戒标记数）、`spyajun_draw`（`modifier draw_count add 1`，
  旧 JS 的 phaseDrawBegin2 `num++`）。
- **ABI 429 `resolve_spzundi(player, target, card, destination, stage, skill)`**：
  出牌阶段限一次（DSL `select_hand 1` 弃牌为代价）对一名其他角色判定：
  黑色其摸三张（`draw_cards`）；红色其移动一张场上牌——目标选一张可移动的
  装备/判定区牌（复用 `movable_field_cards_from` + `field_card_move_destinations`
  管线）与落点，三段 choice 后提交移动（同 TW 振威移动语义）。

## 验证

- 新增 `original_shiji_sp_cuiyan.rs` 4 个行为测试：压军赢家置顶、压军败方得戒标记
  （无置顶选择、拼点牌入弃牌堆）、尊帝黑判定摸三、尊帝红判定移动武器到落点。
- 打包：75 技能 / 22 武将，`rule_vm_abi_version` 502（max 语义，
  包内已有更高 ABI 原语）。
- conformance 3/3 通过，无 golden 漂移；全量 workspace 因并行会话构建争用
  未复跑（另行确认）。
