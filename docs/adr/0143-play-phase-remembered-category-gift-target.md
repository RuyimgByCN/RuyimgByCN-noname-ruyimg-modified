# ADR 0143：出牌阶段记忆类别与延迟赠牌目标

## 决策

Rule VM ABI 53 追加三个类型化效果：

- `choose_reveal_other_hand_card_remember_category(player)`；
- `draw_then_give_exact_record_phase_target(player, target, count)`；
- `clear_phase_gift_target(player)`。

技能实例用互斥、带版本标签的 `RCG1` 保存当前出牌阶段记忆的逻辑牌类别，用 `PGT1` 保存一次
延迟阶段后果绑定的目标。每次进入新的出牌阶段只清理 `RCG1` 形状；`PGT1` 由声明式死亡/阶段结束
触发清除。过滤器只比较当前事件、有效牌类别和运行时技能实例，不识别「强识」「献图」、张松或包名。

交牌选择在摸牌及嵌套触发完成后打开，并在恢复时重新校验技能、阶段、目标、精确数量、牌所有权与
手牌/装备区域。伪造、重复或已经离区的牌会使整条命令原子拒绝。

## 理由

旧实现依赖临时子技能、自由格式 `storage` 与动态事件监听。如果照搬 JavaScript，会重新引入不可
验证的脚本生命周期；如果为单技能硬编码，又无法服务后续上千技能。两个小型状态形状和三项通用
效果覆盖“阶段内记忆属性”和“阶段目标有事件则免除后果”这一类常见规则，同时继续进入快照、哈希、
回放和 Native/WASM 一致性合同。

## 版本

- Rule VM ABI：53
- Rule IR：64
- Engine semantics：139
