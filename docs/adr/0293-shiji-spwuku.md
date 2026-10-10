# ADR 0293：SHIJI「武库」的装备使用标记

SHIJI `spwuku`（杜预·武库）把旧 JS 的全局 `useCard` 装备过滤重写为既有权威过滤器
`owner_alive_equip_used_skill_mark_below_three` 与按技能稳定 ID 定位的标记效果
`add_mark(self, spwuku_def_id, 1)`：有角色使用装备牌时，若拥有者的“武库”标记少于三枚则获得
一枚，非装备牌不触发，四枚封顶语义由 Golden（`original_shiji_spwuku`）证明。与 OFFLINE
「武库」（`pkwuku`）共用同一过滤器与标记上限规则，不新增引擎原语或回合态。

旧 JS 的国战模式二枚上限不建模，随既有 identity-standard 规则集发布；`spwuku` 是唯一
语义来源，旧 JavaScript 不进入 npack 或运行时。杜预的「三陈」「灭吴」尚未迁移，因此本包
只发布技能，不把 `sp_duyu` 放入选将池（闭包门禁断言角色索引不含该武将）。
