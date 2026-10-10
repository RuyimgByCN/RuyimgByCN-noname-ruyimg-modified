# ADR 0534：ABI 617 3D精选韩当【险袭】

归档扩展“3D精选”不是纯皮肤包：它还注册了 31 名 `ddd_*` 武将及规则技能。本批先按武将闭包
发布韩当（`ddd_handang`，3743314674）和【险袭】（`dddxianxi`，3400504078）；旧 JavaScript
只作语义证据，不进入 npack 或运行时。

【险袭】在韩当于自己的出牌阶段使用【杀】时，可无视距离选择任意个尚未成为目标、且满足其余
目标规则的存活角色。新增目标按座次写入本技能实例，并绑定该实体【杀】；该牌结算后，只统计这些
新增目标中未被此牌伤害者。若数量为 X，韩当可弃置恰好 X 张手牌/装备牌，再按座次对这些目标各
造成 1 点普通伤害；取消或牌数不足时，改为摸 X 张牌并失去 1 点体力。所有 Choice 恢复均重验
技能、实体牌、目标和支付牌区域。

ABI 617 新增 `resolve_dddxianxi(player, card, action)` 与两项精确触发过滤器。追加目标复用【杀】
保存续体，只绕过距离，不绕过存活、重复目标和目标无效等规则。实现同时修复共享入口：追加、替换
或重定向保存中的【杀】目标时，响应队列、Sha 记录和 `CardResolved` 目标快照现在同步更新，避免
其他依赖整张牌最终目标的技能读取旧目标列表。

验收：专项 Golden 3/3、相关追加目标回归 1/1、content-compiler 514/514、Rule VM 10/10 与
5/5、engine-api 4/4、engine-core lib 55/55、conformance 8/8，以及包含本包的 29 个扩展空
store 冷导入通过。

`official/ddd@0.1.0` 为 ABI 617，含 1 名武将、1 项技能、2 个模块、2 个触发和 1 项立绘资源；
logic hash 为 `affbd4cfd6d7c90f866f853e8d00957642e4040452c21605df05fc84c7883067`。
源码目录与 `extensions/ddd` 四份 npack SHA-256 均为
`c7e01b1658a829aae730cdf002b4776c7a3e5066eb4ec3586562bbb8f1c593ae`；归档、源码和扩展目录三份
韩当立绘 SHA-256 均为 `de0b1db4fe2f9c03b9b0e071e8a122aa165de4f226d0f8672d21c8ac96e16d3e`。

按架构计划 §37.10，批次结束后只删除工作区 `target/` 和外部
`/private/tmp/noname-next-cargo-target` 的 Rust/Cargo 编译产物；`.npack`、`extensions/`、立绘、
许可证、源码和文档均保留。本 ADR 只证明 3D精选首个武将闭包，不代表该扩展或整个归档迁移完成。
