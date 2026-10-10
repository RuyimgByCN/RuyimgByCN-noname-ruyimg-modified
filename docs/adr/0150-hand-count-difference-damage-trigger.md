# ADR 0150：伤害参与者的手牌数差异选择

## 决策

旧版 JavaScript 只作为技能语义证据，不进入内容包，也不在运行时执行。新增通用触发过滤器
`owner_source_or_target_hand_count_differs_once_per_turn`：只匹配拥有者作为来源或目标的伤害事件，
要求拥有者存活、本回合尚未接受该技能，并且至少存在一名手牌数与拥有者不同的存活角色。过滤器
声明每回合一次；计数只在可选触发被接受后写入，从而替代旧实现的临时占位技能。

新增 `choose_hand_count_different_target_draw_or_random_discard(player)`。效果快照所有手牌数不同的存活
角色并开放强制单目标选择屏障。恢复时重新校验观察者、来源技能、双方存活、快照成员关系与实时
手牌差异；目标手牌较少时摸一张，较多时由权威 `RuleRng` 选择一张实体手牌并进入普通弃置效果。
非法或重复选择原子拒绝，且不推进随机流。核心只实现伤害参与、次数、手牌比较、选择与确定性随机，
不识别「邀名」、全琮或 YIJIANG 包。

## 兼容性

`EffectRequest`、`ContinuationKind`、触发过滤器、Rule IR 语句与 VM host effect tag 均在末尾追加，
保留既有序列化序号。Rule VM ABI 升至 60，Rule IR 升至 71，Engine Semantics 升至 146；内容
schema 保持 3。旧包不获得新能力，仍按原最低 ABI 加载。
