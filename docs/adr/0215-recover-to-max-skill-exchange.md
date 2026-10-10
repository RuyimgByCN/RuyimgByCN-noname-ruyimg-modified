# ADR 0215：回满后的限定技交换

## 背景

四象「殇决」是可被另一技能重置的限定技：进入濒死时先回复至体力上限，回复完整结算后，若仍
拥有「铿锵」，才可以失去「铿锵」并令「殇决」视为未发动。普通 DSL `choose` 会先建立 VM
Choice 屏障、后执行屏障前收集的效果，无法保持这段旧 JavaScript 的异步顺序。

## 决策

Rule VM ABI 142 追加
`recover_to_max_then_optional_exchange_skill(player, removable_skill_def)`（HOST_EFFECT 152）：

- 第一阶段验证来源技能实例及拥有者，先移除来源实例以消费限定技，再回复至体力上限；
- 第二阶段经 `deferred_tail_effects` 等待回复事件、嵌套触发和 Choice 全部完成；
- 只有拥有者仍存活且仍拥有可牺牲技能时，才打开确认 Choice；
- continuation 103 在提交时再次验证，接受后依次移除指定技能并添加来源技能定义的新实例。

新 `EffectRequest` 只在枚举末尾追加，稳定哈希标签为 188。

## 结果

技能 DSL 不读取或写入任意 storage，也不模拟 Promise。Golden 覆盖先回满后询问、接受交换后
再次可发动、拒绝交换、首次拒绝发动，以及没有「铿锵」时不出现第二个 Choice。
