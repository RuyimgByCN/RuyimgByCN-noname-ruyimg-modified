# ADR 0525：ABI 612 归档扩展“玩点论杀”留赞

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。归档扩展“玩点论杀”的留赞
（`wdb_liuzan`，3844268588）以 4 体力、吴势力发布完整武将。「亢音」
（`kangyin`，2250380873）出牌阶段限一次：先失去 1 点体力，再弃置一名其他角色的一张
手牌或装备牌。若弃置基本牌，可取消选择或令 1 至 X 名存活角色各摸一张牌；若弃置非基本牌，
本回合留赞计算至其他角色的距离 -X，且使用【杀】的额外目标数上限 +X。X 在弃牌结算完成后按
留赞当前已损失体力值计算。

ABI 612 新增 `resolve_wdb_liuzan(player, target, helper_skill_def, action)` 和
`owner_skill_state_u32_negative` 修正值源。复合原语复用既有失去体力、跨角色弃牌、可选多目标
摸牌和临时技能清理管线；目标牌移动及其嵌套触发全部完成后才进入基本/非基本分支。非基本分支
把 X 以小端 `u32` 写入不可获得辅助技能 `kangyin2`（2524514324），正值用于【杀】目标上限，
负值用于来源距离，并绑定当前回合结束自动移除。

验收：留赞 Golden 1/1、content-compiler 509/509、Rule VM 15/15、engine-api 4/4、
engine-core lib 55/55、conformance 8/8，以及包含本包的 27 个扩展空 store 冷导入均通过。
`community/wandianlunsha@0.1.0` 累积为 ABI 612、5 名武将、12 项技能、10 个模块、8 个触发、
2 项主动技、6 项修正器和 5 项资源，logic hash 为
`cdedbaac8451a31a31555bf592ef019d1c0ca5afd84380f498a991be96b9181d`；源码目录与
`extensions/wandianlunsha` 四份 npack SHA-256 均为
`a3d8b5ca0794a3220e35f1e80101772fab18ae8955833608d3b918ca8b6bd830`。归档、源码目录和
extensions 三份留赞立绘 SHA-256 均为
`606d6886509130aaf989715b1c654677f29d014f2c7d6dd098fc942dab1b22d4`。

编译产物不得无限累积：按架构计划 §37.10 清理共享
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target` 的 8,465 个文件（Cargo 报告 5.6 GiB）；
收尾审阅修正后重跑 Golden，再清理 2,347 个文件（1.7 GiB）。工作区 `target/` 原本不存在，
验收时两处编译目录和冷导入临时 store 均不存在。
`.npack`、`extensions/`、立绘和许可等发布物保留。归档扩展的其他内容继续按同一流程迁移，
本 ADR 不代表 `/archive/upstream/noname` 全量完成。
