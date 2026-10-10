# ADR 0552：ABI 627 3D精选于禁【整军】

本批按武将闭包发布于禁（`ddd_yujin`，3260070084）与【整军】（`dddzhengjun`，
2986915915）。旧 JavaScript 只作语义证据，不进入 npack 或运行时。

【整军】只在于禁自己的出牌阶段观察体力、手牌数或装备数发生变化的角色；每类在该阶段各限
一次。变化后对应数值等于于禁时，于禁可选择该角色：体力类令其回复或失去 1 点体力；手牌类
令其摸一张或由其自行弃置一张手牌/装备牌；装备类移动其装备区内一张牌。一次移牌事件会同时
检查来源与目的角色，也会同时保留手牌和装备两类，符合条件时按类别分别结算。

ABI 627 增加 `resolve_ddd_yujin(player,actor,action)`、出牌阶段三分类过滤器和移牌事件分类
载荷。实现复用技能实例已有的出牌阶段分类位、标准 Choice、摸牌/弃牌、回复/失去体力以及既有
场上装备移动管线；没有新增第二套装备移动框架。

验收：于禁 Golden 4/4，DDD 十一个武将累计 34/34，content-compiler 522/522、engine-api
4/4、Rule VM 10/10 与 5/5、engine-core lib 55/55、conformance 测试 3/3、可执行门禁 8/8，
以及 31 个扩展空 store 冷导入均通过，内容锁为
`7dc769056840a7fac0f2d258f4c72f52a58af12aaa775a4deadf8fc0e8bf9359`。

`official/ddd@0.1.0` 更新为 ABI 627，累计含 11 名武将、23 项技能、28 个模块、26 个触发、
2 项主动规则、5 项视为规则、3 项修正器和 11 项立绘资源；logic hash 为
`03d8e925200897fcc8b0e5b9e8a9ecabb1173e089bf6c977246f97f2131f7e18`，asset hash 为
`41d2c2575d5cc56051da4b5445fbd4fde0c9ddc1065c163bfff0dea82d6d56f2`。源码目录与
`extensions/ddd` 四份 npack SHA-256 均为
`f440691a5d6c7c7af2d9a07750666b52ac21c69090f4772be0feded2fb6ab46b`；两份于禁立绘
SHA-256 均为 `48253f89214c178f405fc69497bee399b34be17d3e022c984f817234889a3773`。

按架构计划 §37.10，本批门禁结束后只清理 Rust/Cargo 编译产物与增量缓存：外部 target 删除
5,936 个文件（Cargo 统计 3.9 GiB）；工作区 `target/`、外部
`/private/tmp/noname-next-cargo-target` 与 `/private/tmp/noname-next-cargo-target-root` 验收时均
不存在。`.npack`、`extensions/`、立绘、许可证、源码和文档全部保留；该规则用于防止编译产物
随迁移批次无限累积，不是清理发布物。

本 ADR 只证明 3D精选第十一个武将闭包，不代表整个扩展迁移完成。
