# ADR 0072：伤害双方手牌比较值

## 决策

为现有伤害 modifier 增加两个上下文值：`owner_hand_gt_other` 与 `owner_hand_lt_other`。来源侧 `damage_amount` 的 `other` 是受伤角色，受伤侧 `damage_taken_amount` 的 `other` 是伤害来源；任一方不存在或死亡时贡献为零。

## 原因

旧版 `goblin_shiqiang` 分别在 `damageBegin1` 与 `damageBegin3` 比较伤害双方手牌并增加伤害。现有来源侧、受伤侧 modifier 的执行顺序与之相同，仅缺少对另一端玩家的只读上下文；复用该管线比新增修改当前伤害事件的 VM 效果更小且不会产生额外触发。

## 兼容性

两个枚举项追加在末尾以保持既有序列化序号。引擎语义修订号递增到 57；未使用新值的内容包行为不变。
