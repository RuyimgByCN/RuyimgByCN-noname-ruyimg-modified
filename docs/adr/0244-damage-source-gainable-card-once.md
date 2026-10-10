# ADR 0244：伤害来源可获得牌的单次过滤器

## 决策

ABI 169 增加 `owner_is_target_source_alive_gainable_card`。它只在拥有者受到有存活来源的正数
伤害，且来源存在可获得牌时匹配一次：来源为自己时仅检查装备区，否则检查手牌区和装备区。

标准「反馈」复用既有 `take_one_card`，不新增 Host effect。既有
`owner_is_target_source_alive_gainable_card_per_damage_point` 保持「界反馈」的逐伤害点语义。

## 原因

旧标准 JavaScript 在一次 `damageEnd` 中只发动一次；直接复用界反馈过滤器会让两点伤害错误发动
两次。把次数差异放在共享触发分发层，是满足两种技能且不复制取牌结算的最小改动。
