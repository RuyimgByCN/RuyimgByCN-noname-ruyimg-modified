# ADR 0175：安排跳过本回合后续出牌阶段

## 状态

已接受（2026-07-23）。

## 背景

旧版「旧自守」在摸牌阶段增加摸牌数后调用 `player.skip("phaseUse")`。引擎已经用
`turn_flag::SKIP_PLAY` 处理【乐不思蜀】等规则，并在真正跨过出牌阶段时统一产生
`PhaseSkipped` 事件和跳过计数，但 DSL 只能跳过已经进入的当前阶段，不能安排稍后的出牌阶段。

## 决策

Rule VM ABI 100 增加通用效果 `skip_next_play_phase(player)`：

1. 只接受当前存活回合角色，且当前阶段必须位于本回合出牌阶段之前；
2. 效果只设置既有 `SKIP_PLAY` 回合标记，不保存旧 JavaScript 事件或新增技能私有状态；
3. Draw → Play 的权威阶段转换负责消费标记、产生 `PhaseSkipped(Play)`、增加跳过计数并进入
   Discard；
4. 「旧自守」继续复用 ABI 61 的 `increase_current_draw_count`，用
   `max_hp(self) - hp(self)` 计算已损失体力。

## 后果

- 以后所有“安排跳过本回合出牌阶段”的旧技能共享同一条可哈希、可审计阶段路径。
- 非当前角色、已进入出牌阶段后的调用会被宿主拒绝，不会把标记泄漏到下一回合。
- ABI 99 主机拒绝 ABI 100 包；ABI 100 主机继续加载 ABI 3–99 的加法式内容包。
