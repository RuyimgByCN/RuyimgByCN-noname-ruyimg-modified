# ADR 0269: OFFLINE「抬棺」复用条件决斗并增加动态发动次数

ABI 193 为主动技 `uses_per_turn` 增加通用值源 `owner_lost_hp_min_one`；权威发动入口每次按
拥有者当前的 `max_hp - hp` 计算，最少为一，并继续用现有技能使用计数执行上限。

「抬棺」以 `select_he 1 1` 支付任意手牌或装备牌成本，复用 ABI 192 的目标自弃与条件
【决斗】效果，不复制异步选择链。龙庞德与 STANDARD「马术」组成完整技能闭包，旧
JavaScript 仅保留为语义证据。
