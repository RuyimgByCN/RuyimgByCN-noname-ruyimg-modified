# ADR 0238: 存活来源每回合一次过滤器

`owner_is_target_source_alive_once` 组合已有的事件目标、存活伤害来源和技能实例每回合发动计数，
用于旧技能中常见的 `player: damage* + usable: 1 + source.isIn()` 语义。

该过滤器追加到序列化枚举末尾并要求 Rule VM ABI 165；可选触发仅在实际发动后消耗次数。
