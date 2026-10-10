# ADR 0318：SHIJI 神孙策「覆海」与 `owner_source_target_mark_positive_<id>`（ABI 408）

ABI 408 增加触发过滤器 `owner_source_target_mark_positive_<mark_id>`：在 `CardTargeted`
事件中，拥有者存活、为牌使用者，且事件目标带有指定标记（按 `Player.marks` 计数）。

「覆海」（`scfuhai`）以该过滤器为触发门（标记为「英霸」的“平定” 3219705664）：
- 触发：拥有者使用牌指定带“平定”标记的角色为目标时。
- 效果：`make_current_sha_target_unrespondable(self, event_card, event_player)`
  （该目标不可响应）+ `draw_cards(self, 1)`。
- 注：旧 JS 的 `directHit` 为任意牌的通用目标直击；当前以【杀】直击近似，
  锦囊直击待通用 directHit 原语。旧 JS 限每回合前两次触发，当前按每次触发结算。

神孙策（`shen_sunce`）尚缺「冯河」，本技能配合已移植的「英霸」先行发布，不入选将池。
旧 JavaScript 不进入 npack 或运行时。
