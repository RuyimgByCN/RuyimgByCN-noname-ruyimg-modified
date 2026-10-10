# ADR 0313：SHIJI 神太史慈「当陌」与 `owner_source_first_sha_this_play_phase`（ABI 407）

ABI 407 增加触发过滤器 `owner_source_first_sha_this_play_phase`：在 `CardUsed` 事件中，
拥有者存活、为来源、是当前回合角色、处于出牌阶段，且本回合尚未使用过逻辑【杀】
（复用 `turn_flag::sha_used_by_player` 计数）。每次触发后按 `play_phase_limit` 限制一次。

「当陌」（`dangmo`）以该过滤器为触发门：
- 触发：出牌阶段使用第一张【杀】时（体力大于 1）。
- 效果：`choose_add_current_sha_target(self, event_card)` 为其增加其他合法目标。
- 旧 JS 的上限为 `hp - 1` 个目标；DSL 以玩家自选数量近似（引擎校验目标合法性）。

神太史慈（`shen_taishici`）尚缺「破围」「神破围」，本技能先行发布，不入选将池。
旧 JavaScript 不进入 npack 或运行时。

待办：`liubing`（流冰）复用本过滤器做“本阶段第一张杀改方块”，需要条件
`card_suit` modifier 支持，另行领取。
