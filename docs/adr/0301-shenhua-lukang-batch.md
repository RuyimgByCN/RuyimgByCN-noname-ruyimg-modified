# ADR 0301：SHENHUA 纯 DSL 批次 2（陆抗：谦节/破势）

本批为陆抗（`lukang`）两名技能的纯 DSL 移植，全部以既有 DSL 原语组合表达，不新增 Rule VM ABI。

- 「谦节」（`drlt_qianjie`）：`modifier armor_link_gain_cancel override 1` 表达旧 JS
  `drlt_qianjie_1`（linkBegin 时未横置则取消横置）；`modifier card_target_cancel(delayed_trick)
  override 1` 表达 `drlt_qianjie_2`（不能成为延时锦囊牌的目标）。
  旧 JS 的 `drlt_qianjie_3` 仅为 AI 提示（不拼点），不移植。
- 「破势」（`drlt_poshi`）：觉醒技，旧 JS 以单一 `phaseZhunbeiBegin` 触发 +
  「无可用装备栏或体力值为 1」过滤。DSL 无逻辑或表达式，拆为两个触发分支：
  - `phase_prepare` + `hp(self) == 1`；
  - `equip_slot_disabled` + `owner_is_event_player_no_enabled_equip_slot_once_per_game`
    （与 key 包 `chihaya_liewu` 同一模式）。
  两分支共用觉醒标记（`mark_count(drlt_poshi) == 0`）防重复觉醒；觉醒效果为
  `change_max_hp(-1)`、补牌至 `max_hp - hand_count`、`remove_skill_by_def(drlt_jueyan)` +
  `add_skill(drlt_huairou)`。

陆抗（`lukang`，id 3741484100，男/吴/4 体力）由「谦节」+「决堰」（已移植）+「破势」
构成完整闭包，加入 `characters.json` 选将池。旧 JavaScript 不进入 npack 或运行时。
