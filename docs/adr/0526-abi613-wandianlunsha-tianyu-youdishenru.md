# ADR 0526：ABI 613 归档扩展“玩点论杀”田豫与诱敌深入

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。归档扩展“玩点论杀”的田豫
（`wdb_tianyu`，3972209824）以 4 体力、魏势力发布完整武将。「车阵」
（`chezhen`，2288694956）在装备区为空时令其他角色计算至田豫的距离 +1；装备区有任意数量
牌时，田豫计算至其他角色的距离固定 -1。装备阈值值源返回 0/1，而不是装备数，避免多件装备
错误叠加距离收益。

「诱战」（`youzhan`，2340355119）在任意可响应【杀】开始结算其目标时，若田豫至该目标距离
不大于 1 且田豫可弃置手牌区或装备区的一张装备牌，可选择支付并令该目标视为使用
【诱敌深入】。该锦囊使用正常【无懈可击】奇偶窗口；若被抵消，原【杀】继续结算。若生效，
原【杀】目标被取消，其使用者须无距离、无次数限制地对受保护角色使用一张实体或转化【杀】；
本次【杀】未造成伤害则重复，造成伤害则结束，拒绝或没有合法【杀】则受到受保护角色造成的
1 点伤害。三张归档实体【诱敌深入】共用同一响应与可恢复结算管线，不能在普通出牌阶段主动使用。

ABI 613 新增 `resolve_wdb_tianyu(player, target, card, action)`、`youdishenru` 卡牌种类、
`owner_alive_sha_target_distance_le_one_has_equip_card` 过滤器，以及可序列化的
`ContinueYoudishenru` 内部续体。实现复用既有【杀】响应、【无懈可击】、虚拟牌物化、实体牌移动、
视为【杀】和逐牌伤害记录，不引入第二套牌结算框架。

验收：田豫 Golden 6/6、content-compiler 510/510、Rule VM 10/10、engine-api 4/4、
engine-core lib 55/55、conformance 3/3，以及包含本包的 27 个扩展空 store 冷导入均通过。
`community/wandianlunsha@0.1.0` 累积为 ABI 613、6 名武将、14 项技能、11 个模块、9 个触发、
2 项主动技、9 项修正器、1 张卡和 7 项资源，logic hash 为
`a1a072f0da2be98bf3bcdfe059934c3483f056edcd411f817b8bd755fc5f24b7`；源码目录与
`extensions/wandianlunsha` 四份 npack SHA-256 均为
`b041eba559767580dba7d335aa33ebd613a185af52c7dfd40efcb66450231354`。归档、源码目录和
extensions 三份田豫立绘 SHA-256 均为
`6ee8cd5a527ed8c8adfcabb7d21b33aba9a79c3b08b80792164beda61dbe1706`，三份【诱敌深入】卡图
SHA-256 均为 `b2cc3dd430fe92ffe733a6f11a21d63d6a2458bf014087cd37966af1aaa29964`。

编译产物按架构计划 §37.10 在门禁和打包完成后清理：外部共享目录删除 8,858 个文件
（Cargo 报告 6.2 GiB），工作区 `target/` 原本为空，验收时两处目录均不存在；`.npack`、
`extensions/`、立绘、卡图和许可等发布物保留且清理后哈希不变。归档扩展的其他内容继续按
同一流程迁移，本 ADR 不代表
`/archive/upstream/noname` 全量完成。
