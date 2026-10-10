# ADR 0317：SXRM 界刘备（逞变）

本批为 SXRM 包界刘备（`sxrm_liubei`）的单技能完整移植，复用既有拼点/虚拟牌原语，
不新增 Rule VM ABI。

## 技能移植

「逞变」（`sxrmchengbian`，3193035603）：
- 准备阶段与结束阶段开始时（`phase_prepare`/`phase_end` +
  `owner_is_target_can_compare_other`，均可选）与一名其他角色拼点
  （`choose_hand_compare(self, win, lose)`）。
- 「逞变·胜」（`sxrmchengbian_win`，2536586505）：`hand_compare_finished owner_is_target`
  触发，`use_virtual_card_on(self, juedou, event_source)` 视为对拼点对手使用【决斗】。
- 「逞变·败」（`sxrmchengbian_lose`，3960895664）：`use_virtual_card_on(event_source, juedou, self)`
  视为拼点对手对你使用【决斗】。

复用先例：`choose_hand_compare` 触发版拼点（chengzhao/nsfeixiong/shuimeng）、
`use_virtual_card_on(self, juedou, event_source)`（hannan 等）。

文档化近似：旧 JS 在决斗结算后二次拼点，最终胜者摸至体力上限；二次拼点需跨事件
链式拼点原语，未移植。

界刘备（`sxrm_liubei`，3011387565，男/群/4 体力）由单一技能构成完整闭包，加入
新创建的 `characters.json` 选将池；sxrm npack 重建为 4 技能 + 1 武将，ABI 68。
旧 JavaScript 不进入 npack 或运行时。
