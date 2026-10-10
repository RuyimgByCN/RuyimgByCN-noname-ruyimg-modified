# ADR 0488：ABI 593 XIANDING 柏灵筠

旧 JavaScript 仅作为语义证据；运行时不加载或改写旧 JS。柏灵筠（`bailingyun`，
4278690663）以新 DSL 与共享引擎原语完成三技能闭包：

- 「灵慧」在全局结束阶段检查当前回合角色或本回合濒死历史，观看牌堆顶三张；可使用其中
  一张，完整结算后由权威 RNG 随机获得一张仍在处理区/弃牌堆的余牌，其余牌按原相对顺序
  放回牌堆顶。
- 「黠策」的两个分支每回合各限一次：受到伤害后可令另一名角色的非锁定技失效至回合结束；
  造成伤害后若已受伤，可弃置一张手牌或装备牌并回复 1 点体力。
- 「御心」沿用既有 DSL：限定技，在任意角色进入濒死时将自己回复至 1 点体力，或令该角色
  回复至自己当前的非负体力值。

ABI 593 新增通用 `resolve_bailingyun(player, action)`，复用共享实体牌使用、处理区、权威 RNG、
弃牌/回复及 ABI 356 非锁定技失效管线；仅新增本回合濒死角色集合和黠策双分支位，并在内容包
确实含「灵慧」时启用濒死历史记录。

验收：柏灵筠 Golden 3/3、XIANDING 122/122、content-compiler 493/493、engine-api、Rule VM、
conformance 8/8 与 26 包空 store 冷导入通过；内容锁为
`5b4c8c57e06bb71de5944b253748577c1d945af96da64edcc4bc6315298be789`。XIANDING 包为
ABI 593、12 名武将、77 项技能、80 个模块，logic hash
为 `6c3ee9a0889791318260640a5aeee7c8cefab28d2ebb3a1ea787e3665298eba2`；源码与
`extensions/xianding` 的四份 npack SHA-256 均为
`6396906b119a5d74d8cfcf49209985b780a95b0e88b6aa7ef13375a8c9b36856`。编译产物按架构计划
§37.10 在批次结束后清理，共删除外部共享 target 的 34,323 个文件（Cargo 报告 15.6 GiB）；
工作区与外部 target 均确认不存在，npack 与 `extensions/` 交付物保留。
