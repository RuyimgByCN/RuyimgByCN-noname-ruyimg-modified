# ADR 0385: ABI 516 兵势臧洪（烈志/拒孤）

- 状态：已完成
- 日期：2026-08-02
- ABI：516

旧 JavaScript 仅作为语义证据。烈志扩展既有无实体牌视为使用管线，以失去 1 点体力上限为成本，同一技能每回合限一次且不计普通用牌次数，仅允许【桃】或【酒】。拒孤复用技能实例的回合发动计数：满体力时成为牌的目标可发动，摸两张牌，再强制弃置 `min(可弃置手牌装备数, 本回合发动次数)` 张牌。

门禁：势臧洪 Golden 3/3、兵势全量 Golden、content-compiler 464/464、Rule VM、content-runtime、engine-api ABI 协调、扩展导入与 conformance 8/8 全部通过。全量 engine-core tests 另被并行中的 ABI 511–513 测试编译错误阻断，与本 ADR 无关。
