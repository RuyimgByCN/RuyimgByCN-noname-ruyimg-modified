# ADR 0087：本局累计造成伤害查询

## 背景

OL「伏枥」在濒死时回复并补牌后，要比较角色本局累计造成的伤害点数与存活势力数。旧 JS 的 `getAllHistory("sourceDamage")` 不能进入新运行时；既有 `damage_history` 只记录“来源曾伤害过哪些目标”，无法还原伤害点数总和。

## 决策

权威 `GameState` 追加按来源玩家索引的饱和 `damage_dealt_totals`。实际伤害扣减成功后按最终伤害点数累加；体力流失、被防止的伤害和无效目标不计入。该字段进入快照兼容默认值和 canonical state hash。

DSL 追加只读查询 `lifetime_damage_dealt(player)`，非法玩家返回稳定查询错误，超过 `i32` 的累计值按 `i32::MAX` 返回。查询 tag 21 为追加编码。Rule VM ABI 升至 8，ABI 8 主机继续加载 ABI 3–7 包，只有实际使用该查询的包声明最低 ABI 8。内容 Schema 保持 3，Rule IR 版本升至 15，清单 VM ABI 轴保持 3，引擎语义修订号升至 72。

## 后果

伤害累计成为可复用、可回放、Native/WASM 一致的权威状态，不需要为「伏枥」增加技能 ID 特判。旧 JavaScript 仅保留为语义证据。

无伤害的开局 Golden 哈希保持不变；三条会造成伤害的 identity conformance 向量因新增 canonical 状态字段按本决策重生成并重新锁定。
