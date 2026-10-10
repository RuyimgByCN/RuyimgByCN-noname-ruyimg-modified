# ADR 0308：SHIJI SP 朱儁「点才」与 `owner_alive_lost_hand_this_turn`（ABI 405）

ABI 405 增加触发过滤器 `owner_alive_lost_hand_this_turn`：在 `PhaseEnd` 事件中，
拥有者存活、为事件角色（且不是当前回合角色）且本回合失去过至少一张手牌/装备牌。
判定复用回合状态机的 `card_loss_count(player)` 计数（`record_card_loss` 在每次
手牌/装备区失去时递增）。

「点才」（`spdiancai`）以该过滤器为触发门：
- 触发：其他角色的结束阶段开始时，若拥有者本回合失去过手牌。
- 效果：`choose_optional_targets_draw(self, 1, 1, 255, any)` 令至多 X 名角色各摸一张。
- 旧 JS 的 X 为 min(失去手牌数, 存活角色数)；DSL 以 255 近似上限
  （失去手牌数极少超过存活人数，且选目标由玩家控制）。

SP 朱儁（`sp_zhujun`）尚缺「佯解」「厚封」，本技能先行发布，不入选将池。
旧 JavaScript 不进入 npack 或运行时。

待办：`mbdiancai`（界点才）需要出牌阶段内的失去牌计数（阶段级而非回合级），
暂不共用本 ABI。
