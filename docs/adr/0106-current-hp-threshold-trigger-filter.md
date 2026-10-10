# ADR 0106：事件角色当前体力阈值过滤器

## 决策

DSL 新增参数化触发过滤器 `owner_is_target_hp_below_N`。`N` 必须是正整数；技能拥有者必须仍然
存活、是当前事件角色，并且事件处理后的当前体力值严格小于 `N`。阈值进入包触发索引与逻辑
哈希，运行时从通用玩家状态读取，不按技能 ID 分支。

该能力不新增 VM 指令，Rule VM ABI 保持 16；Rule IR 提升至 27，Engine Semantics 提升至
102。既有过滤器的序列化枚举顺序不变，新变体只在末尾追加。

## 迁移边界

KEY「消元」的旧 JavaScript `changeHp` 负数事件在新引擎中分别对应受到伤害的 `damage` 和直接
失去体力的 `hp_lost`，因此类型化技能声明两条强制触发并共享 `owner_is_target_hp_below_4`。
运行时只加载 `.skill` 编译后的 npack，不执行旧 JavaScript。觉醒通过移除自身技能实例消耗，
并按旧逻辑依次减 3 点体力上限、摸三张牌、移除“绯驱”。
