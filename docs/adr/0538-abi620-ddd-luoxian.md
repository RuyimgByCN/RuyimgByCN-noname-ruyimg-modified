# ADR 0538：ABI 620 3D精选罗宪【示烈】

本批按武将闭包发布罗宪（`ddd_luoxian`，2803410298）与【示烈】（`dddshilie`，
3904237061）。旧 JavaScript 只作语义证据，不进入 npack 或运行时。

【示烈】每回合限一次，在需要使用【杀】或【闪】时，可选择任意张此前未公开的手牌；其有效
点数和必须不小于罗宪当前体力与当前回合角色当前体力之和。所选牌公开后仍留在手牌并持续可见，
随后以无实体来源的对应基本牌继续结算；点数和精确相等时，罗宪摸所选牌数量的牌。

ABI 620 为既有 source-free 视为牌成本增加 `dddshilie`。实现复用标准选牌续体、公开牌、
有效点数查询、每回合次数、摸牌以及无实体来源【杀/闪】结算；选择恢复时重新校验观察者、技能
状态、当前回合、牌仍属未公开手牌、选择非空且不重复、有效点数和达到动态阈值。没有新增专用
EffectRequest、VM opcode 或平行结算管线。

验收：罗宪 Golden 3/3，DDD 四个武将 Golden 13/13，content-compiler 517/517、Rule VM
10/10 与 5/5、engine-api 4/4、engine-core lib 55/55、conformance 3/3，以及 31 个扩展空
store 冷导入通过。

`official/ddd@0.1.0` 更新为 ABI 620，累计含 4 名武将、5 项技能、6 个模块、6 个触发、1 项
视为规则和 4 项立绘资源；logic hash 为
`91ba4029257cd7f4cc4bea940bd097e05e88d78b73583b9578a12158045d0bf0`，asset hash 为
`6f3e04c8ced0b0590c1e0e35f2c93f0242a4a118ca5447d986429e69ead91715`。源码目录与
`extensions/ddd` 四份 npack SHA-256 均为
`de5654030b0251bc8ef4853dfeaa620f660fd91d63d847ab3862a3712c5e7e1d`；源码与扩展目录两份
罗宪立绘 SHA-256 均为
`3108823cf615ecfc9189c456efb78071e8defbfcd7ddf90c6c53adcfa4e67c29`。

按架构计划 §37.10，本批门禁结束后只删除 Rust/Cargo 编译产物和增量缓存：工作区 `target/`
原本不存在，外部 `/private/tmp/noname-next-cargo-target` 由 `cargo clean` 删除 8036 个文件
（Cargo 报告 5.5 GiB），随后移除空目录，验收时两处编译目录均不存在。`.npack`、
`extensions/`、立绘、许可证、源码及文档全部保留，清理后发布包与立绘哈希不变。本 ADR 只证明
3D精选第四个武将闭包，不代表整个扩展迁移完成。
