# ADR 0053：按区域获得另一角色的牌

## 决策

新增通用 DSL effect `take_one_card(from, to, hand_equip|equip)`。`to` 负责选择，选择列表和 `ResolveChoice` 都校验相同区域范围；实现复用 `PickTargetCard`，其 mode 2 表示仅装备区。

新增伤害过滤器 `owner_source_target_alive_has_equip`，只在拥有者为伤害来源、受伤角色仍存活且装备区非空时开放可选触发。

## 原因

旧 JS 的 `nscangjian` 需要从受伤角色装备区选一张牌。仅在客户端隐藏手牌不足以保证权威校验；区域限制必须进入 continuation payload 并在命令入口复核。

## 兼容性

新增 VM effect tag 32、`EffectRequest` 变体和触发过滤器；引擎语义版本升至 37。
