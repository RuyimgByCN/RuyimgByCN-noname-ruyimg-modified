# ADR 0229：拼点历史与座次路径逐个伤害

## 决策

ABI 156 增加
`choose_compare_selected_hand_card_second_failure_seat_damage(player, target, draw_count, failure_count, lose_hp, damage)`。
主动技必须声明
`targets 1 1 other can_compare unused_by_skill_this_turn select_hand 1 1`。

运行时复用既有 `hand_compare_started`、实体牌公开/移动和 `hand_compare_finished` 边界。
严格获胜后，目标无可弃手牌/装备时直接摸牌，否则拥有者强制选择摸牌或弃置目标一张牌。
未赢目标按发动顺序记录；第二次未赢时技能立即禁用至回合结束，拥有者失去体力，待濒死/
求桃链结束后按静态座次计算两条路径。路径长度不同时自动取较短者，长度相同时由拥有者选择；
路径中的死亡中间角色被移除，两个原目标参与长度计算但造成伤害前仍须存活。

路径伤害使用通用 `DealDamageEachInOrder` 队列：一次只提交一个伤害，下一个目标放入结算尾部，
因此每次伤害的触发、濒死和求桃都先完整结束。拼点完成事件同样作为中断边界，完成事件触发
先于胜负后续分支。

## 理由

旧 JavaScript 的 `useSkill` 历史、事件对象布尔字段、`previousSeat`/`nextSeat` 循环和
`doAsyncInOrder` 不能进入确定性运行时。技能实例保存规范化的回合目标/失败目标历史；
选择载荷、效果请求和状态哈希覆盖全部决定性输入，重放无需执行旧 JS。

## 兼容性

新增 HOST_EFFECT 167、continuation 110–112，以及 `EffectRequest` 稳定哈希标签 203–208。
旧 ABI 和旧序号保持不变。
