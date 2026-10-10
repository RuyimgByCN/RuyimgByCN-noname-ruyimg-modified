# ADR 0039：通用属性伤害效果

- 状态：已接受
- Engine semantics：23

## 背景

typed effect 与伤害结算已经支持普通、火焰和雷电属性，但技能 DSL 的 `deal_damage` 只能发出普通伤害。若继续用技能专用 effect 表达属性伤害，上千技能迁移会产生大量重复分支。

## 决策

把 `deal_damage` 扩展为 `deal_damage(source, target, amount[, nature])`：

- `nature` 仅接受 `normal`、`fire` 或 `thunder`；
- 省略第四参数时继续生成原有普通伤害字节码，已有包行为不变；
- 非普通属性使用追加的稳定 VM effect tag，最终仍生成统一的 `EffectRequest::DealDamage`；
- 护甲、属性修正、濒死与事件派发继续由权威伤害管线处理。

## 结果

「逆取」只包含主动窗口、次数、目标和属性伤害声明。以后火焰或雷电直伤技能可复用同一 DSL/VM 路径，无需增加技能名判断。
