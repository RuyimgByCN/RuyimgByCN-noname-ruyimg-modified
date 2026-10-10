# ADR 0541：ABI 622 3D精选吴质【灵涌/虚孝】

本批按武将闭包发布吴质（`ddd_wuzhi`，3862495543）、【灵涌】
（`dddlingyong`，2946205848）与【虚孝】（`dddxuxiao`，3539713105）。旧 JavaScript
只作语义证据，不进入 npack 或运行时。

【灵涌】观察任意角色跳过准备、判定、摸牌、出牌、弃牌或结束阶段；吴质可判定，结果为【杀】
立即停止，否则可使用仍在处理区的判定实体牌，并可重复流程。【虚孝】观察所有角色的弃牌移动，
对仍在弃牌堆的黑色基本牌逐张询问，可直接改作【兵粮寸断】置入任意合法角色判定区并令其摸两张；
直置不经过距离、使用次数或【无懈可击】窗口。

ABI 622 增加 `resolve_ddd_wuzhi(player,card,action)`、`phase_any_skipped` 事件与对应弃牌过滤器。
实现复用判定、标准 Choice、实体牌使用、判定区重复校验、实体移牌和摸牌管线；选择恢复时重新校验
技能、实体牌位置/有效颜色与类别、目标存活状态及判定区合法性。

验收：吴质 Golden 3/3，DDD 六个武将 Golden 18/18，content-compiler 519/519、
Rule VM 10/10 与 5/5、engine-api 4/4、engine-core lib 55/55、conformance 3/3，
以及 31 个扩展空 store 冷导入通过。

`official/ddd@0.1.0` 更新为 ABI 622，累计含 6 名武将、8 项技能、10 个模块、9 个触发、
1 项主动规则、1 项视为规则和 6 项立绘资源；logic hash 为
`6bebdd4fafaa3f92172889f696bf972a1ef5ef53052a62f361cebc78fc3f6c5e`，asset hash 为
`dd8d2d34c3057634798c64b514dc1a9b7ce9af035f3f95fadf8442e461e4f46e`。源码目录与
`extensions/ddd` 四份 npack SHA-256 均为
`217266ffcf02277629b21c8b67c11483f555de726588a059856220532f2ef3d7`；两份吴质立绘 SHA-256 均为
`6e038fe205eb6860b92a93fbf572b9038b4ea4199f3582d910ca1bcef17124c7`。

按架构计划 §37.10，本批门禁结束后只删除 Rust/Cargo 编译产物和增量缓存：工作区 `target/`
由 `cargo clean` 删除 6,823 个文件（Cargo 报告 4.1 GiB），外部
`/private/tmp/noname-next-cargo-target-root` 删除 2,674 个文件（Cargo 报告 1.8 GiB）；共享
`/private/tmp/noname-next-cargo-target` 原本不存在，随后移除空目录，验收时三处编译目录均不存在。
`.npack`、`extensions/`、立绘、许可证、源码及文档全部保留，清理后发布包与立绘哈希不变。
本 ADR 只证明 3D精选第六个武将闭包，不代表整个扩展迁移完成。
