# ADR 0152：准备阶段目标弃牌与延迟分支选择

## 决策

旧版 JavaScript 只作为技能语义证据，不进入内容包，也不在运行时执行。新增通用效果
`choose_target_discard_excess_then_self_discard_or_target_draw(player)`，只允许准备阶段触发器调用。
核心以一个带阶段标签的 continuation 表达三道选择屏障：选择目标、精确选择目标牌、可选支付自己的
弃牌成本。目标弃牌与内部后续请求进入同一效果队列，使弃牌产生的嵌套移动触发先完整结算，再打开
第二分支；重连时不依赖闭包、Promise 或旧 JS 对象。

目标数量按 `max(hand_count - hp, 1)` 计算，并在可弃牌不足时取全部。第二分支数量在移动前按所选
实体牌是否为装备牌快照；全为装备牌时没有后续。拥有者不能支付或取消时，原目标通过带来源技能的
统一摸牌效果摸等量牌。所有恢复点重新验证阶段、技能实例、角色存活、选择快照、实体牌区和去重。
核心只实现通用选择语义，不识别「镇军」、于禁或 YIJIANG 包。

## 兼容性

`EffectRequest`、Rule IR 语句、VM host effect tag、触发过滤器和 continuation tag 均在末尾追加，
保留既有序列化序号。Rule VM ABI 升至 62，Rule IR 升至 73，Engine Semantics 升至 148；内容
schema 保持 3。旧包不获得新能力，仍按原最低 ABI 加载。
