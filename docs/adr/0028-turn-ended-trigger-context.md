# ADR 0028：整回合结束触发上下文

## 决策

新增 DSL 事件 `turn_ended`，仅由 `DomainEvent::PhaseEnded { phase: End }` 产生。事件角色是刚结束整个回合的角色；它与 `phase_end`（结束阶段开始）及 `phase_play_end`（出牌阶段结束）保持独立。

新增通用过滤器 `owner_alive_not_target_neither_unique_max_hand`：技能拥有者与事件角色都必须存活且不是同一角色，并且双方手牌数都不严格大于其他所有存活角色。并列最多不算“唯一最多”。

限定「沉声」只组合该事件、过滤器、可选触发与既有摸牌效果，不增加技能专用 Rust 分支。

## 兼容性

`EventKind` 与 `TriggerFilter` 新变体均追加在枚举末尾，旧 npack 的序列化变体编号不变。规则可表达范围发生变化，`EngineSemanticsId` 从修订 11 升至 12；Native 与浏览器 WASM 一致性门禁必须重新通过。
