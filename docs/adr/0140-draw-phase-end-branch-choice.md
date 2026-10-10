# ADR 0140：摸牌阶段结束与分支成本续体

## 决策

Rule VM ABI 50 增加：

```text
phase_draw_end
choose_draw_or_discard_add_skill(player, draw_skill_def, discard_skill_def)
```

`PhaseEnded(Draw)` 映射为独立 `phase_draw_end`，不复用摸牌阶段开始或通用阶段结束事件。引擎在
阶段切换事务中先把权威状态设为出牌阶段，再按顺序派发摸牌结束和出牌开始事件；因此本效果只在
同一当前角色、状态已为出牌阶段且原技能实例仍有效时继续。

可选触发由通用 `OptionalTrigger` 先处理取消；接受后，本效果打开不可取消的分支选择。选项 1 先
授予 `draw_skill_def` 再摸一张牌。选项 2 仅在拥有者有可弃置手牌或装备时出现，随后使用同一个
`ChooseDrawOrDiscardAddSkill` 续体进入成本阶段，先授予 `discard_skill_def` 再弃置所选牌。两个
阶段都重新校验选择者、当前角色、阶段、原技能实例和卡牌位置；非法或过期选择由事务回滚，不会
产生半个分支结果。

效果只接收稳定技能定义 ID，不识别「将驰」、曹彰或包名。STANDARD 定义主技能和摸牌分支辅助
技能，YIJIANG 定义弃牌分支辅助技能及曹彰，并精确依赖 STANDARD。旧 JavaScript 只用于确认
选择、授予、弃牌和清理时序，不进入包或运行时。

## 版本

- Rule VM ABI：50
- Rule IR：61
- Engine semantics：136
