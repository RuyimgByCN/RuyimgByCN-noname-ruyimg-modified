# ADR 0181：当前出牌阶段失去体力事件量

## 背景

旧「诈降」每失去一点体力摸三张牌；若事件发生在拥有者自己的出牌阶段，还按失去体力点数增加
临时【杀】次数，并令红色【杀】无距离限制且不可响应。旧实现依赖 JavaScript 事件对象的
`num`、当前阶段对象和临时子技能，不能原样进入确定性运行时。

## 决策

- 追加过滤器 `owner_alive_is_target_current_play_phase`，只读取权威回合角色和阶段状态。
- 将既有 DSL 值 `event_amount` 开放给 `hp_lost`，读取已经进入事件载荷的失去体力数量。
- 「诈降」的摸牌分支始终以 `event_amount * 3` 执行；出牌阶段分支添加同一临时辅助技能，并
  以 `event_amount` 增加辅助技能标记。
- 辅助技能复用现有 `sha_use_limit`、`card_distance_unlimited(sha_red)` 和
  `sha_unrespondable(red)` 修正器，在 `turn_ended` 时移除全部标记和自身。

## 兼容性

Rule VM ABI 从 105 提升到 106，Engine Semantics 从 152 提升到 153。触发过滤器枚举只在末尾
追加；Rule IR 保持 76，内容 schema、包格式、协议和快照 schema 不变。ABI 106 主机继续加载
ABI 3–105 内容包；使用新过滤器的包必须声明 ABI 106。

## 结果

失去体力点数与当前阶段判断均来自可快照、可回放的权威状态，不保留旧 JavaScript 事件或阶段
对象，也不增加角色专用 Host 原语。REFRESH npack 可独立发布完整界黄盖闭包；旧 JavaScript
只作为语义合同。
