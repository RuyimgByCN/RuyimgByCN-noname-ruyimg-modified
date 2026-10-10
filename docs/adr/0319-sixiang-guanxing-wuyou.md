# ADR 0319：SIXIANG 关兴（无有）

本批为 SIXIANG 包关兴（`std_guanxing`）的单技能完整移植，复用既有拼点/虚拟牌/临时技能
原语，不新增 Rule VM ABI。

## 技能移植

「无有」（`stdwuyou`，2234235932）：
- 出牌阶段限一次，与一名其他角色拼点
  （`active play_phase uses_per_turn 1 targets 1 1 other can_compare select_hand 1 1` +
  `choose_compare_selected_hand_card(self, event_source, win, lose)`）。
- 「无有·胜」（`stdwuyou_win`，3805520549）：`hand_compare_finished owner_is_target`
  触发，`use_virtual_card_on(self, juedou, event_source)` 视为对拼点对手使用【决斗】。
- 「无有·败」（`stdwuyou_lose`，3037326263）：`add_skill_until_turn_end(self, 3038912803)`
  本回合获得 REFRESH 的【武圣】（`new_rewusheng`，已移植跨包引用）。

复用先例：拼点胜负分支（hannan/shuimeng/chengzhao/sxrmchengbian）。

关兴（`std_guanxing`，4110138309，男/蜀/4 体力）由单一技能构成完整闭包，加入
`characters.json` 选将池；sixiang npack 重建为 72 技能 + 37 武将，ABI 453。
旧 JavaScript 不进入 npack 或运行时。
