# ADR 0027：获得技能触发上下文

## 决策

将既有 `DomainEvent::SkillAdded` 映射为 DSL 事件 `skill_added`。事件角色是获得该技能实例的角色，事件来源为空；每个 `AddSkill` 产生一个独立事件，不把多个技能折叠为一个不透明批次。

增加通用过滤器 `owner_alive_event_player_alive`。它同时要求技能拥有者和事件角色存活：前者补齐旧触发框架对死亡角色普通技能的隐式禁用，后者对应旧 JS 的 `event.player?.isIn()`。

OFFLINE「寒慈」只组合该事件、过滤器与两个既有 `draw_cards` 效果，不增加技能专用 Rust 分支。

## 兼容性

`EventKind` 与 `TriggerFilter` 新变体均追加在枚举末尾，旧 npack 的序列化变体编号不变。规则可表达范围发生变化，`EngineSemanticsId` 从修订 10 升至 11；Native 与浏览器 WASM 一致性门禁必须重新通过。
