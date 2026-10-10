# ADR 0524：ABI 611 归档扩展“玩点论杀”刘焉

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。归档扩展“玩点论杀”的刘焉
（`wdb_liuyan`，4053361006）以 3 体力、群势力发布完整武将。「绝道」
（`juedao`，3304739063）在未横置时可于出牌阶段弃置一张手牌并横置；横置期间，刘焉计算至
其他角色以及其他角色计算至刘焉的距离均 +1。「割据」（`geju`，4049779435）在准备阶段可摸
X 张牌：先取其他存活角色的已知势力集合，再移除任何至少有一名成员能以当前攻击范围覆盖刘焉
的势力，X 为剩余势力数。未知势力不计入，判断包含【绝道】和其他距离/攻击范围修正。

ABI 611 新增 `resolve_wdb_liuyan(player, card, action)`、主动技 `owner_unlinked` 权威前置条件、
`owner_linked` 修正值源，以及“存在全势力均不在拥有者攻击范围内”的准备阶段过滤器。实现复用
主动手牌选择、弃牌、铁索横置标记、双向距离查询和摸牌管线；不为这两个技能引入持久专属状态。

验收：刘焉 Golden 1/1、content-compiler 508/508、Rule VM 15/15、engine-api 4/4、
engine-core lib 55/55、conformance 3/3，以及包含本包的 27 个扩展空 store 冷导入均通过。
`community/wandianlunsha@0.1.0` 累积为 ABI 611、4 名武将、10 项技能、9 个模块、8 个触发、
1 项主动技、4 项修正器和 4 项资源，logic hash 为
`f268f5e5bacd22e76b7ee98184aac3ee0e3f30c223532bd6566be8cf108aed73`；源码目录与
`extensions/wandianlunsha` 四份 npack SHA-256 均为
`69fd75093fd7e37dc101ccf6709548feb0b56b13aef10ce54d4c3075fcefa82f`。归档、源码目录和
extensions 三份刘焉立绘 SHA-256 均为
`1bf07f28f01c41418d1af25972b0578af058fe0654501130f35cc706e2d3567a`。

编译产物不得无限累积：本批收尾只清理 Rust/Cargo 编译缓存。工作区 `target/` 原本不存在；
共享 `CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target` 删除 8,145 个文件（Cargo 报告
5.6 GiB）。验收时两处编译目录均不存在；`.npack`、`extensions/`、立绘和许可等发布物保留。
归档扩展的其他内容继续按同一流程迁移，本 ADR 不代表 `/archive/upstream/noname` 全量完成。
