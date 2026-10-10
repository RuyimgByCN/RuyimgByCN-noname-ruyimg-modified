# ADR 0067：卡牌伤害来源与指定玩家摸牌/回复选择

## 决策

- `DealShaDamage` 携带物理牌；内部 `CommitDamage` 同时携带物理牌与逻辑牌名。
- 元素连环使用 `DealCardDamage` 继续传递来源，但不重复套用【杀】来源侧伤害修正。
- `damage_before` 与 `damage` 触发上下文均可读取该来源；普通无牌伤害保持空值。
- DSL 增加 `choose_draw_or_recover(player, draw_count, recover_count, allow_cancel)`；满体力时直接摸牌。

## 理由

旧技能普遍通过 `trigger.card` 判断造成伤害的牌，且连环伤害继承原事件的牌。把来源保留在通用
伤害流水线中，比为每个技能建立状态或 ID 特判更小，也能复用到后续迁移。

`chooseDrawRecover` 在上游被多处使用，选择者不一定是技能拥有者，因此作为指定玩家的通用效果
发布，而不扩展旧 JS 运行时。

## 兼容性

新增字段和枚举均追加并提供反序列化默认值；引擎语义修订号递增到 51。
