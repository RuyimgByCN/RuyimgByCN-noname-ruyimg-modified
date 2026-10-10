# ADR 0548：ABI 625 3D精选辛宪英【洞察/智解】

本批按武将闭包发布辛宪英（`ddd_xinxianying`，174041969）、【洞察】（3036718178）、回合末
临时规则（4217095917）与【智解】（1168942044）。旧 JavaScript 只作语义证据，不进入 npack
或运行时。

【洞察】在其他角色回合开始且双方均有手牌时可发动：辛宪英与当前回合角色依次强制选择一张
手牌，置入辛宪英按【洞察】定义隔离的技能牌堆作为“鉴”；本回合结束时辛宪英先取得一张，
若仍有“鉴”且当前回合角色存活，其再取得一张。每个恢复点均重新验证观察者、实体牌所在牌区、
技能牌堆和角色存活状态。

【智解】没有新建第二套响应系统，而是在既有多实体牌视为响应链路增加受限成本
`dddzhijie`：候选来源只取上述【洞察】技能牌堆，恰选两张；有效颜色相同只可转【闪】，不同
只可转【无懈可击】。响应完成后两张实体“鉴”按响应原因进入弃牌堆，再摸两张牌。编译器和
运行时包加载器均只接受这两个精确声明，不向其他扩展放宽 `post_draw` 或技能牌堆来源契约。

ABI 625 增加 `resolve_ddd_xinxianying(player,actor,action)`、两个精确触发过滤器及
`dddzhijie` 多牌响应成本。实现复用技能牌堆、标准 Choice、实体移牌、多牌响应和摸牌管线。

验收：辛宪英 Golden 2/2，DDD 九个武将累计 27/27，content-compiler 522/522、engine-api
4/4、Rule VM 10/10 与 5/5、engine-core lib 55/55、conformance 8/8，以及 31 个扩展空 store
冷导入均通过，内容锁为 `b816aea5b02123f72fd4e0c43839300a3f761104e9e94aefc336dacb08e6bee2`。

`official/ddd@0.1.0` 更新为 ABI 625，累计含 9 名武将、17 项技能、22 个模块、20 个触发、
2 项主动规则、3 项视为规则、1 项修正器和 9 项立绘资源；logic hash 为
`85a14babfbb670176cb019b3c920c56e157e7f12a103b20c9606b53de88773c0`，asset hash 为
`c2bd2758d430f276359cdc36ae7ab792fbcd79bbd5b4caf002785bfe1c2754d4`。源码目录与
`extensions/ddd` 四份 npack SHA-256 均为
`14c7507bd7bab628766edbaf04c9aecf7e36ada2297fe98a223b8c65055dbd7a`；两份辛宪英立绘
SHA-256 均为 `a6e246f77a5a027bb01c719b99325a1ac68cb3db8b66ec68219d46b3aa57ea21`。

按架构计划 §37.10，本批门禁结束后只删除 Rust/Cargo 编译产物：工作区 `target/` 删除
7,129 个文件（3.9 GiB），备用 `/private/tmp/noname-next-cargo-target` 与
`/private/tmp/noname-next-cargo-target-root` 原本不存在；验收时三处编译目录均不存在。
`.npack`、`extensions/`、立绘、许可证、源码及文档全部保留且哈希不变，防止编译缓存无限累积。

本 ADR 只证明 3D精选第九个武将闭包，不代表整个扩展迁移完成。
