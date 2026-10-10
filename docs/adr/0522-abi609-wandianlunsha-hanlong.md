# ADR 0522：ABI 609 归档扩展“玩点论杀”韩龙

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。归档扩展“玩点论杀”的韩龙
（`wdb_hanlong`，4082905817）以 4 体力、魏势力发布完整武将。「伺机」
（`siji`，2867665051）在弃牌阶段结束后，按该阶段实际弃置的【杀】数量摸两倍牌；即使牌在
嵌套触发中又移出弃牌堆，也仍按旧 `event.cards` 语义计数。「刺酋」
（`ciqiu`，3777188862）在韩龙以【杀】对当前未受伤目标造成伤害前令伤害 +1；若增加后的
伤害值不小于目标当前体力，韩龙获得至本回合结束的「刺酋·死」
（`ciqiu_dying`，3613978472）并失去「刺酋」，目标进入濒死时立即死亡。

ABI 609 新增 `resolve_wdb_hanlong(player, target, card, helper_skill_def, action)`、弃牌阶段
【杀】历史过滤器和健康目标【杀】伤害过滤器，复用现有阶段历史、绑定伤害、临时技能、技能移除
与濒死死亡管线。`community/wandianlunsha@0.1.0` 累积发布费诗与韩龙，携带归档 GPL-3.0
许可及两张原始立绘，不执行或打包旧 JavaScript。

验收：韩龙 Golden 1/1、content-compiler 506/506、Rule VM 15/15、engine-api 4/4、
engine-core lib 55/55、conformance 3/3，以及包含本包的 27 个扩展空 store 冷导入均通过。
包为 ABI 609、2 名武将、5 项技能、4 个模块、4 个触发、2 项修正器和 2 项资源，logic hash 为
`5ab7162b040baec676829bd03f4f4c1e2b1ec61422a421c68563ece051562aa5`；源码目录与
`extensions/wandianlunsha` 四份 npack SHA-256 均为
`e2169967b83cc767cb5225c4e99399d3bb69f773a70dc3feae0d73b6d3672fc1`。归档、源码目录和
extensions 三份韩龙立绘 SHA-256 均为
`5092b124387d4af4e6d4d379c7def9ed52e234ddec2f527dee53ddf29ef03ab4`。

编译产物不得无限累积：本批收尾只清理 Rust/Cargo 编译缓存。工作区 `target/` 当时不存在；
共享 `CARGO_TARGET_DIR` 删除 6,537 个文件（Cargo 报告 4.4 GiB）。验收时两处编译目录均
不存在，`.npack`、`extensions/`、立绘与许可等发布物必须保留。归档扩展的其他内容仍按同一
流程继续迁移，本 ADR 不代表 `/archive/upstream/noname` 全量完成。
