# ADR 0545：ABI 623 3D精选许靖【虚誉/实荐】

本批按武将闭包发布许靖（`ddd_xujing`，3594755778）、【虚誉】
（`dddxuyu`，3210825357）与【实荐】（`dddshijian`，2262569305）。旧 JavaScript
只作语义证据，不进入 npack 或运行时。

【虚誉】在许靖使用牌结算结束后可摸一张；摸牌及其嵌套触发完成后记录一次本回合债务，许靖
下一张牌完整结算后先清除债务，再强制弃置一张手牌或装备牌。若本回合没有下一张牌，回合结束
直接清除债务。【实荐】只观察其他角色出牌阶段的第二张牌完整结算；许靖可交给该角色一张手牌
或装备牌，并在赠牌及嵌套触发完成后绑定该角色。该角色下一张有目标牌首次指定目标时，许靖
强制选择令原牌在完整结算边界后按原用户、牌类和目标额外结算一次，或自己摸一张；随后解除绑定。

ABI 623 增加 `resolve_ddd_xujing(player,source,card,action)` 及五个精确过滤器。实现复用
CardResolved、逐角色出牌阶段用牌计数、标准 Choice、实体赠牌、版本化技能实例状态和既有
`PendingCardReplay` 结算后重放管线；所有选择恢复点重新验证技能、角色、实体牌位置和观察状态。

验收：许靖 Golden 3/3，DDD 七个武将 Golden 21/21，content-compiler 520/520、
Rule VM 10/10 与 5/5、engine-api 4/4、engine-core lib 55/55、conformance 8/8，
以及 31 个扩展空 store 冷导入通过。

`official/ddd@0.1.0` 更新为 ABI 623，累计含 7 名武将、10 项技能、16 个模块、15 个触发、
1 项主动规则、1 项视为规则和 7 项立绘资源；logic hash 为
`c6225da84b0a9496e6211cf357681cf8590f9de09e47e20311f00e730f10cc62`，asset hash 为
`038eeac80a5fd27c6b9abdf810c7216ea468f666e37aaa67d5c5716e2c31fb5d`。源码目录与
`extensions/ddd` 四份 npack SHA-256 均为
`c28f471869eb87dd17cc5f964d0bbe483f2409192b0b5463689fe7380a021c92`；两份许靖立绘 SHA-256
均为 `58095b9c358b2946a7c48438085a305d0ca0bd93356de2e5a15b4d986975207a`。

按架构计划 §37.10，本批门禁结束后只删除 Rust/Cargo 编译产物和增量缓存：工作区 `target/`
原本不存在；共享 `/private/tmp/noname-next-cargo-target` 删除 4,041 个文件（Cargo 报告
2.2 GiB），本批 `/private/tmp/noname-next-cargo-target-root` 删除 11,028 个文件（Cargo 报告
7.4 GiB），随后移除空目录。验收时三处编译目录均不存在；`.npack`、`extensions/`、立绘、
许可证、源码及文档全部保留，清理后发布包与立绘哈希不变。本 ADR 只证明 3D精选第七个武将
闭包，不代表整个扩展迁移完成。
