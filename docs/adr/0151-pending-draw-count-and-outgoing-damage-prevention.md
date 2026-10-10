# ADR 0151：待消费摸牌数与来源伤害防止

## 决策

旧版 JavaScript 只作为技能语义证据，不进入内容包，也不在运行时执行。新增通用效果
`increase_current_draw_count(player, count)`：只允许 `phase_draw` 触发器为当前存活摸牌阶段角色
累加非负摸牌数。增加量保存在回合状态中，由权威阶段机在普通摸牌或摸牌替代入口读取并立即清零，
因此触发选择暂停期间可安全恢复，同一增加量不会被重复消费，最终普通摸牌仍是单个事件批次。

公开通用伤害过滤器 `owner_source`，只匹配技能拥有者作为当前伤害来源。内容可结合事件目标判断、
`prevent_current_damage()` 和 `turn_ended` 移除临时技能，表达“当前回合防止拥有者对其他角色造成
伤害”，同时保留自伤。核心只实现阶段、待消费数量、伤害来源与绑定伤害防止，不识别「界自守」、
刘表、REFRESH 或 YIJIANG 包。

## 兼容性

`EffectRequest`、回合标志、Rule IR 语句与 VM host effect tag 均在末尾追加，保留既有序列化序号。
Rule VM ABI 升至 61，Rule IR 升至 72，Engine Semantics 升至 147；内容 schema 保持 3。旧包不
获得新能力，仍按原最低 ABI 加载。
