# ADR 0258: STANDARD「洛神」连续判定

ABI 182 增加 `judge_black_gain_repeat(player)`（HOST_EFFECT 180）。

效果复用标准判定与延迟尾队列：红色结果留在弃牌堆并停止；黑色结果若仍在弃牌堆则移入判定
角色手牌，待移动触发结算完成后再询问是否继续。重复选择使用 continuation 120，
`EffectRequest` 稳定哈希标签为 221。甄姬以「洛神、倾国」完整发布；旧 JavaScript 不执行。
