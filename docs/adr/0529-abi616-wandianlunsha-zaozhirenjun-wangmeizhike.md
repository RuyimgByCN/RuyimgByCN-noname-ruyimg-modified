# ADR 0529：ABI 616 玩点论杀枣祗任峻、五谷丰登与望梅止渴

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。归档扩展“玩点论杀”的枣祗任峻
（`wdb_zaozhirenjun`，2265349363）以 3 体力、魏势力和【粮策】【坚壁】【军屯】发布，至此该
扩展当前启用的九名武将和三张扩展卡全部闭合；上游牌表里注释掉的卡不作为启用内容发布。

【粮策】（`liangce`，3473735644）每个出牌阶段限一次把手牌中的一张基本牌当【五谷丰登】使用。
【五谷丰登】不再采用旧引擎“每名目标摸一张”的占位实现：它按使用时原目标数把牌堆顶实体牌移入
公开处理区，每名未排除目标分别经过标准【无懈可击】奇偶窗口并强制取得一张；余牌以
`CardMoveReason::WuguRemained` 统一进入弃牌堆。【粮策】可在该移动完成后选择任意存活角色，获得
仍位于弃牌堆的全部余牌。牌池、目标队列和每个 Choice 均显式保存并在恢复时重验。

【坚壁】（`jianbi`，3452664776）在拥有者成为目标数不少于二的普通锦囊目标时可选择其中一名，
把该目标加入当前牌的排除集合。上游 `multitarget` 属性与 `selectTarget: -1` 是不同概念；当前已
建模普通锦囊均为前者缺省/false，因此【五谷丰登】等全体选目标锦囊仍可触发。普通锦囊复用既有
保存目标排除管线，【五谷丰登】只给共享牌池续体增加排除列表，不另建卡牌结算框架。

【军屯】（`diyjuntun`，2307382183）复用通用 `recast_selected`：出牌阶段可反复选择手牌区或装备区
一张装备牌，以 Recast 原因置入弃牌堆并摸一张牌。

【望梅止渴】（`wangmeizhike`，2738715878）为可指定任意一名存活角色（包括自己）的普通锦囊，
先经过标准【无懈可击】窗口。生效时若目标并列最低体力且受伤则回复 1 点；回复及其嵌套触发结束后
重新计算并列最低手牌，满足时若本牌执行过回复摸一张，否则摸两张。

ABI 616 新增 `resolve_wdb_zaozhirenjun(player, target, card, action)`、两项触发过滤器、可恢复
`ContinueWugu`、五谷余牌移动原因和 `wangmeizhike` 卡牌种类；【粮策】视为技、【军屯】重铸、
标准无懈、实体牌移动及独立目标锦囊队列均复用现有能力。

验收：专项 Golden 9/9、content-compiler 513/513、Rule VM 10/10 与 5/5、engine-api 4/4、
engine-core lib 55/55、conformance 8/8、golden identity 1/1，以及包含本包的 27 个扩展空 store
冷导入通过。ABI 616 的【五谷丰登】精确结算改变了三组身份局确定性结果，相关 conformance 与
golden regression 哈希已按同一运行结果同步重钉。

`official/wandianlunsha@0.1.0` 为 ABI 616、9 名武将、21 项技能、18 个模块、14 个触发、4 项
主动技、11 项修正器、3 项视为技、3 张卡和 13 项资源，logic hash 为
`3ec24af9ebac579cb3fdc2dfd38ecb2230d5e12676b221b241f78858ceb7987b`；源码目录与
`extensions/wandianlunsha` 四份 npack SHA-256 均为
`bff1e06aec8e501bb748aaee2af8ce299132ab2428c9ab84cf8e1b8576fe7b2f`。归档、源码和扩展目录三份
枣祗任峻立绘 SHA-256 均为 `02b49b0dcc6e28a85b26cd5805c574ee64347b37bfcdb3dc9faf73d0976d34c5`，
三份【望梅止渴】卡图均为 `bf3b7eb3f954da3c52dd79f3b5509f40559ca5d5e41c7a091e4840b379f8732c`，
三份男声卡牌音频均为 `eeefca50c59373953cd73985cf064ddc62e4ad875f2fd88e5dbf4864d7ff49df`。

按架构计划 §37.10，批次结束后只删除工作区 `target/` 和外部
`/private/tmp/noname-next-cargo-target` 的 Rust/Cargo 编译产物；`.npack`、`extensions/`、立绘、
卡图、音频、许可证、源码和文档均保留。验收时两个编译目录均不存在。本 ADR 只证明“玩点论杀”
当前启用内容闭环，不代表 `/archive/upstream/noname` 全量迁移完成。
