# ADR 0282：EXTRA 神曹操「归心」

旧 JavaScript 仅作为语义证据，不进入运行时。每一点伤害独立触发；玩家选择手牌、装备牌或
判定牌优先级后，引擎按座次逐名处理其他存活角色。每次重新读取当前区域，从首个非空区域通过
`RuleRng` 随机选择一张牌，等待该移动及嵌套触发完整结束后再继续，最终翻面。

神曹操通过 EXTRA 对 STANDARD 的显式依赖复用「飞影」。新效果使用 HOST_EFFECT 200、
续体 131、效果哈希标签 240；Rule VM ABI 为 206，Rule IR 为 92，
Engine semantics revision 为 180。
