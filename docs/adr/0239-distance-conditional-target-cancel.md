# ADR 0239: 最终距离条件目标取消

`card_target_cancel_if_other_user_distance_gt_one(selector)` 复用既有卡牌 selector、最终距离
查询和 `TargetLegal` modifier 汇总；另一角色到技能拥有者的距离大于一时才加入匹配修正器。

新 `ModifierTarget` 枚举项追加到末尾并要求 Rule VM ABI 166；距离仍由统一查询处理座次、坐骑
和技能修正，不引入技能专属触发器、状态或 Host effect。引擎语义修订号递增到 172。
