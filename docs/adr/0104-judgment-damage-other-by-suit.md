# ADR 0104：按判定花色选择其他角色造成属性伤害

## 决策

DSL 新增参数化效果 `judge_damage_other_on_suit(player, suit, amount, nature)`。它执行普通判定；
有效花色匹配且伤害值非零时，由判定者从其他存活角色中强制选择一名目标，再以判定者为来源造成
指定的普通、火焰或雷电伤害。没有其他存活角色或花色不匹配时，判定仍发生，但不打开目标选择。

目标选择使用追加的 continuation tag 30，VM host effect 使用追加的 tag 45；`StmtIr`、
`EffectRequest` 和散列分支均只在末尾追加，既有序列化序号保持不变。该能力提升 Rule VM ABI 至
15、Rule IR 至 25、Engine Semantics 至 100；只有实际使用新效果的包声明 ABI 15。

## 迁移边界

KEY「音爆」的旧 JavaScript 仅用于核对可选时机、判定花色、强制目标和雷电伤害语义。运行时只
加载类型化 DSL 与 npack，不执行旧对象，也不按技能 ID 写宿主特判。
