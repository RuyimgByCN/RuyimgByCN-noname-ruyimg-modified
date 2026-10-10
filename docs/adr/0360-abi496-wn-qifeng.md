# ADR 0360：ABI 496 齐锋失牌计数虚拟杀

## 决策

新增过滤器 `owner_alive_global_phase_end_lost_cards_target_other_alive` 与效果
`resolve_wn_qifeng(player, target)`。回合结束时按引擎既有的本回合实体失牌次数计数（同一
实体牌重复失去时重复计数），
对回合角色使用无距离、无次数限制的虚拟【杀】，基础伤害为失牌数；普通响应、目标合法性
与伤害管线保持不变。

## 理由

旧 JavaScript 只依赖本回合失牌历史和虚拟【杀】的动态基础伤害。复用现有历史与带伤害
加值的虚拟【杀】执行器即可精确表达，无需技能专属状态或延迟任务。
