# ADR 0092: 按牌种从公共牌堆获得牌进入通用权威效果

## 决策

新增 DSL 效果 `gain_card_by_kind(player, card_kind)`。权威引擎按旧版 `get.cardPile`
的默认顺序查找实体牌：先从牌堆顶向下，再从弃牌堆顶向下，找到第一张指定规则牌种后移入
目标角色手牌；两处都没有匹配牌时不产生效果。查找依据实体牌 `Card.kind`，不采用当前视为牌名。

联动包「求索」据此完整重写：拥有者造成或受到伤害后可发动，获得第一张【铁索连环】。
旧 JavaScript 仅保留为语义证据。该宿主效果可继续承载其他“从牌堆、弃牌堆按牌种获得”技能，
不在技能层增加专用效果。

新宿主效果使用追加的 effect tag 42，Rule VM ABI 从 9 提升至 10；ABI 10 主机继续加载
ABI 3–9 内容，只有实际使用 `gain_card_by_kind` 的包声明最低 ABI 10。内容 Schema 保持 3，
Rule IR 从 17 提升至 18，Engine Semantics 从 76 提升至 77。Native/WASM 与 conformance
哈希门禁必须重跑。
