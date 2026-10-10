# ADR 0312：SHENHUA 郝昭（镇骨）

本批为 SHENHUA 包郝昭（`haozhao`）的单技能完整移植，复用既有原语与 ABI 451，不新增 ABI。

## 技能移植

「镇骨」（`drlt_zhengu`，3942271682）：
- 出牌阶段限一次，`add_bound_skill(self, drlt_zhengu_sync, event_source)` 记录一名其他角色；
- 「镇骨·调整」（`drlt_zhengu_sync`，3509249751）：目标回合结束时
  `trigger phase_end owner_alive_skill_bound_target_is_event_player` 触发，
  `adjust_hand_to(event_source, min(5, hand_count(self)))` 将目标手牌调至与拥有者相同
  （至多摸至五张）。

语义验证：PhaseEnded 事件上下文 `source = current`（回合结束角色），bound skill 触发时
`event_source` = 被记录目标、`self` = 拥有者，与旧 JS 的 `sync(player, target)` 一致；
摸牌上限 `min(5, 拥有者手牌)` 以 ABI 451 min 表达式等价表达。

文档化近似：旧 JS 同时在拥有者回合结束时结算（「你的回合结束时和其下个回合结束时」）；
DSL 的 bound-target 玩家表达式未落地，拥有者回合结束分支未移植（目标回合结束分支为
核心结算）。

郝昭（`haozhao`，3007130668，男/魏/4 体力）由单一技能构成完整闭包，加入
`characters.json` 选将池；shenhua npack 重建为 88 技能 + 21 武将，ABI 451。
旧 JavaScript 不进入 npack 或运行时。
