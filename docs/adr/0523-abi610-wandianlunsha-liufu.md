# ADR 0523：ABI 610 归档扩展“玩点论杀”刘馥

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。归档扩展“玩点论杀”的刘馥
（`wdb_liufu`，3857174889）以 3 体力、魏势力发布完整武将。「筑城」
（`zhucheng`，2400116759）只在结束阶段没有“筑”时可发动，将牌堆顶
`max(1, 已损失体力值)` 张牌置于公开技能牌堆；成为【杀】的目标时，使用者须弃置等量手牌或
装备牌，否则该【杀】对刘馥无效，且“筑”不因防御而消耗。旧 JS 中“已有筑则全部获得”的
分支被同一技能的过滤条件排除，属于不可达代码，本次按实际可执行语义保留，不按描述文字扩写。
辅助定义「筑城·防御」（`zhucheng2`，3898919118）保留稳定 ID，但其分组触发已折入
「筑城」主定义。「夺气」（`duoqi`，2671845513）在其他角色出牌阶段因弃牌事件触发时，可弃置
一张“筑”并立即结束该出牌阶段；弃置“筑”自身引发的嵌套触发先完整处理，再结束阶段。

ABI 610 新增 `resolve_wdb_liufu(player, target, card, zhucheng_skill_def, action)`，并补齐结束阶段
空技能牌堆、以【杀】指定且技能牌堆非空、他人出牌阶段弃牌且技能牌堆非空三类过滤器，复用公开
技能牌堆、精确手牌/装备弃置、当前【杀】目标排除、deferred tail 与阶段结束管线。同期修正稳定
定义 ID 的字节码常量编码：定义 ID 统一按无符号 32 位压栈，避免大于 `i32::MAX` 的合法 ID 在
VM 中被截为 0。

验收：刘馥 Golden 1/1、content-compiler 507/507、Rule VM 15/15、engine-api 4/4、
engine-core lib 55/55、conformance 3/3，以及包含本包的 27 个扩展空 store 冷导入均通过。
`community/wandianlunsha@0.1.0` 累积为 ABI 610、3 名武将、8 项技能、7 个模块、7 个触发、
2 项修正器和 3 项资源，logic hash 为
`59201a8870bfed4d419935ee3f4cb57030b23dfe4fbc4b1a79371e0923d12d80`；源码目录与
`extensions/wandianlunsha` 四份 npack SHA-256 均为
`c0612ff75f5d167a389bfdc90937122cd480a422cca6d13b3d5bc0d28a6e1d9b`。归档、源码目录和
extensions 三份刘馥立绘 SHA-256 均为
`0dec58f74e7e7c0a0235c7cb3321c3a8980a52cdaca62b5c638c33ce5528db04`。

编译产物不得无限累积：本批收尾只清理 Rust/Cargo 编译缓存。工作区 `target/` 原本不存在；
共享 `CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target` 删除 8,311 个文件（Cargo 报告
5.4 GiB）。验收时两处编译目录均不存在；`.npack`、`extensions/`、立绘和许可等发布物保留。
归档扩展的其他内容继续按同一流程迁移，本 ADR 不代表 `/archive/upstream/noname` 全量完成。
