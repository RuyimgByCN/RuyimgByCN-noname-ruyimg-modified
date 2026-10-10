# ADR 0374：ABI 503 通用完整卡牌结算边界 + 兵势国渊

## 语义来源

`archive/upstream/noname/apps/core/character/bingshi/skill.js` 中的 `mbqingdao`、
`mbxiugeng`、`mbchenshe` 仅作为语义证据，旧 JavaScript 不进入新运行时。

## ABI 503

- 新增通用 `card_resolved` 事件：保留实体牌、使用者、声明目标及实际受伤目标，等该牌的
  响应、伤害、濒死和嵌套选择全部结束后统一发出。
- `resolve_mbqingdao` 复用牌堆/弃牌堆检索、目标区域弃牌及实体手牌使用管线；后者仅忽略
  距离和普通次数限制。
- `resolve_mbxiugeng` 以临时技能实例保存各目标手牌基线；下个摸牌阶段按 `<=` 与 `>=`
  独立判定，因此相等时同时摸两张并永久增加一点手牌上限。
- `resolve_mbchenshe` 按 `[技能拥有者, 濒死角色, 伤害来源]` 顺序处理，保留重复角色；恰好
  三张同花色时回复至体力上限并失去技能。

## 内容闭环

`guoyuan.skill` 和角色索引已加入 bingshi 包并同步到扩展目录。Golden 覆盖完整【杀】
结算后触发、修耕相等双分支、陈赦同花色回复与失去技能。
