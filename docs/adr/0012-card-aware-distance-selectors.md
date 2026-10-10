# ADR 0012: Card-aware distance selectors

## Decision

- `card_distance_unlimited` 保留原有“所有牌无距离限制”语义。
- `card_distance_unlimited(selector)` 使用既有 `CardSelector`，可按【杀】、类别、花色或颜色限制距离豁免。
- 所有持有实体牌 ID 的命令、查询、AI 与无头跑局路径统一按实体牌匹配选择器；旧 npack 未携带 `selector` 时行为不变。

## Compatibility

旧 JS 的 `targetInRange` 经常只对锦囊、颜色或花色生效。复用 modifier 条目已有的可选
`selector`，比为每种牌新增目标通道更适合上千技能。标准「奇才」作为首个迁移案例，以
【顺手牵羊】、【兵粮寸断】和【杀】的命令级用例证明选择器不会扩大到不匹配的牌。
