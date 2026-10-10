# ADR 0135：当前【杀】火焰转换、追加目标与整次结算

## 决策

Rule VM ABI 45 增加：

```text
convert_current_sha_to_fire_lose_hp_if_damage(player, card)
choose_add_current_sha_target(player, card)
```

引擎把“本次【杀】是否由普通【杀】转换”“整次用牌是否实际造成过伤害”保存为当前用牌续体中的
有界标志。追加目标只能从当前仍合法且尚未包含的目标中选择，并发布一个类型化
`CardTargetAdded`；不会伪造第二次 `CardUsed`。所有目标及其中嵌套的濒死流程完成后，若转换牌
至少造成过一点未被防止、未被闪避的伤害，使用者才失去一点体力，而且整次至多一次。

## 原因

旧「疠火」修改可变牌对象、父事件目标数组，并通过父事件伤害历史判断事后代价。直接复制这些
JavaScript 对象会让多目标、选择屏障与重放恢复依赖隐式引用。类型化当前用牌续体把牌、目标、
响应位和事后代价放在同一个确定性生命周期中；天然火【杀】可以复用追加目标能力，但不会误付
转换代价。核心不识别「疠火」技能 ID，旧 JavaScript 只保留为语义证据。

## 版本

- Rule VM ABI：45
- Rule IR：56
- Engine semantics：131
