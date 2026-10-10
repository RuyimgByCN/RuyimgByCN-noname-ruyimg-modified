# ADR 0296：SHIJI 纯 DSL 批次（神助/英霸/割卜/毅烈/向镇/放纵/三陈）

本批七个 SHIJI 技能全部以既有 DSL 原语组合表达，不新增 Rule VM ABI。

- 「神助」（`shenzhu`）：使用【杀】后 `choose(0, 1)` 分支，授予 `sha_use_limit +1`
  或 `sha_use_cancel` 临时技能。旧 JS 要求单张实体来源牌的【杀】，DSL 以
  `owner_source_sha` 覆盖全部【杀】，轻微放宽来源牌数量限制。
- 「英霸」（`yingba`）：`change_max_hp(event_source, -1)` + `add_mark` +
  `change_max_hp(self, -1)`；`yingba_mark` 以 `owner_skill_mark` 提供手牌上限加成
  （跨角色拥有者计数在 DSL 中近似为标记数本身）。
- 「割卜」（`gebo`）：`hp_recovered` 全局触发 + `gain_top_card_to_skill_pile`，
  把旧 JS 的全局“仁库”映射为拥有者的技能牌堆。后续仁库系技能（输诚/济世/送枢等）
  复用同一映射。
- 「毅烈」（`spyilie`）：`phase_play` 三选一 `choose(0,1,2)`，授予杀次数+1、
  被杀抵消摸牌两个临时子技能；背水分支额外 `lose_hp`。
- 「向镇」（`spxiangzhen`）：`card_target_cancel(nanman)`。旧 JS 的第二段
  （南蛮造成伤害后来源与拥有者各摸一张）需要按伤害来源区分的全局事件，暂未建模。
- 「放纵」（`spfangzong`）：`phase_end` 手牌不足存活数（至多 8）时 `draw_cards`
  补足；`card_target_cancel_from_current(any)` 近似旧 JS 的“当前回合角色不能对你
  使用伤害牌”（该通道按当前回合角色统一取消，不区分牌类）。
- 「三陈」（`spsanchen`）：觉醒技，`mark_count(spwuku) > 2` 时 `change_max_hp` +
  `recover_hp` + `add_skill(spmiewu)` + `remove_skill(self_skill)`。`spmiewu`
  （灭吴）的消耗标记视为牌通道尚未接入，以空定义占位。

杜预（`sp_duyu`）由「武库」+「三陈」组成第 N 名完整 4 体力群势力武将闭包发布。
旧 JavaScript 不进入 npack 或运行时。

待办（需新 ABI 原语）：存嗣（获得牌交目标）、慧识（花色循环）、躁厉（gaintag 机制）、
各仁库系技能的“从仁库获得”通道、拼点改目标类技能。
分明已由 ADR 0297 / ABI 402 闭环；嬉战已由 ADR 0298 / ABI 403 闭环。
