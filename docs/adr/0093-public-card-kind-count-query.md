# ADR 0093: 公共牌堆实体牌种计数进入通用只读查询

## 决策

新增 DSL 整数查询 `public_card_kind_count(card_kind)`，返回共享牌堆与弃牌堆中指定实体规则牌种
的总数。查询不采用视为牌名，不包含处理区、角色区域或移出游戏区，也不改变牌堆顺序。

DIY「怀霜」据此在两处公共牌堆确有【桃】时，依次获得第一张【桃】并失去 1 点体力；没有【桃】
时两项效果都不执行，与旧 JavaScript 的提前返回一致。获得顺序继续复用
`gain_card_by_kind(player, card_kind)`，不新增技能专用宿主效果。

新宿主查询使用追加的 query tag 22，Rule VM ABI 从 10 提升至 11；ABI 11 主机继续加载
ABI 3–10 内容，只有实际使用 `public_card_kind_count` 的包声明最低 ABI 11。内容 Schema 保持 3，
Rule IR 从 18 提升至 19，Engine Semantics 从 77 提升至 78。清单 VM ABI 轴保持 3。
