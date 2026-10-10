# ADR 0515：ABI 604 XIANDING 张臶

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。张臶（`zhangjian`，2536216702）以 105
体力、群势力发布完整武将。「技能A」（`dc_zj_a`，2792442500）为锁定技：牌造成伤害前，实体牌
有正点数时把本次伤害值改为该点数；虚拟牌或无有效点数时防止本次伤害；无牌伤害不触发。
「技能B」（`dc_zj_b`，3520470433）在结束阶段可弃置所有可弃置的手牌和装备牌，令一名其他
存活角色获得「技能A」，直到张臶的下个回合开始。

ABI 604 新增 `resolve_zhangjian(player, card, grant_skill_def, action)`，复用当前绑定伤害、标准目标
Choice、全手牌/装备弃置和既有 source-turn-bound 授技状态。多个来源向同一目标授技时分别记录、
分别到期，不新增长期状态结构。

验收：张臶 Golden 2/2、content-compiler 501/501、Rule VM 15/15、engine-api 4/4、conformance
8/8、26 个官方扩展空 store 冷导入均通过，ContentLock 为
`97f847b05de820f53bbfd56b340120f64706db5556a048d7168fbe9c601a874e`。XIANDING 包为 ABI 604、
22 名武将、98 项技能、110 个模块、97 个触发，logic hash 为
`beecac43f24a35ab7177eb271eb16132e33c7ec3f7dcbd1e9e884e30de55a553`；源码与
`extensions/xianding` 四份 npack SHA-256 均为
`8f79868921955bfacf0505dd4e4961f4a1e0e23449fb0fb162cd1c365faa544b`。

全局 `sync-original-content --verify-only` 仍被既有 `diy:junkjizhi` 定义 ID
`3934300054/2783324283` 冲突阻断，不以覆盖或白名单跨包修复。

收尾按架构计划 §37.10 只清理 Rust/Cargo 编译缓存：工作区 `target/` 原本不存在；共享
`CARGO_TARGET_DIR` 删除 5,850 个文件（Cargo 报告 3.7 GiB）。两目录最终均不存在；`.npack`、
`extensions/` 与 README 发布物保留。
