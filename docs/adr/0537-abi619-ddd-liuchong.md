# ADR 0537：ABI 619 3D精选刘宠【精彀/末焱】

本批按武将闭包发布刘宠（`ddd_liuchong`，2788416783）、锁定技【精彀】
（`dddjinggou`，3419786472）与【末焱】（`dddmoyan`，4038206557）。旧 JavaScript 只作
语义证据，不进入 npack 或运行时。

【精彀】在刘宠使用武器牌完整结算后，若其当前攻击范围不小于任一存活角色，强制选择一名
其他角色并造成 1 点普通伤害。【末焱】每轮只在首个可行动角色的阶段开端提供一次可取消窗口：
公开牌堆顶三张，刘宠可选择一名存活角色，由该角色获得其中一至三张，余牌进入弃牌堆；技能
实例保存本轮目标与实际获得数。刘宠本轮受到伤害前，若该目标仍存活且手牌数不大于获得数，
本次伤害加 1。下一轮首次检查时旧状态自然失效。

ABI 619 新增 `resolve_ddd_liuchong(player, actor, card, action)`。实现复用攻击范围查询、牌堆
重洗、处理区、公开牌、标准目标/卡牌 Choice、实体移牌、伤害和当前伤害修改管线，只增加一份
固定 16 字节技能状态与两段可恢复分牌续体。恢复时重验技能、轮数、观察者、角色存活、处理区
牌集合、数量和去重；取消目标时公开牌全部弃置，同轮不会重复询问。

验收：刘宠 Golden 3/3，韩当/周处回归 7/7，content-compiler 516/516、Rule VM 10/10 与
5/5、engine-api 4/4、engine-core lib 55/55、conformance 3/3，以及 31 个扩展空 store 冷导入
通过。

`official/ddd@0.1.0` 更新为 ABI 619，累计含 3 名武将、4 项技能、6 个模块、6 个触发和 3 项
立绘资源；logic hash 为 `859b202dfd88589551f10cc9a0dc2f29c9f019dab785d94696702d0129a38c72`，
asset hash 为 `e9f2159f848c681dc6c62cfa9ea4632ce859104beb63ce73d6e8ed50a2d73d6e`。
源码目录与 `extensions/ddd` 四份 npack SHA-256 均为
`c8e03adfdb78e46177586a3ebb35464daea39f963196262f0cf158a0189a75da`；源码与扩展目录两份刘宠
立绘 SHA-256 均为 `c9569a77a53b5049031a46490f1b6d1d5e9aeceb298f8700037276d58ae61c68`。

按架构计划 §37.10，门禁结束后清理约 3.5 GiB、6552 个外部 Rust/Cargo 编译文件；因该目录
缺少 Cargo 要求的 `CACHEDIR.TAG`，`cargo clean --target-dir` 为防误删主动拒绝，随后按 §37.10
的元数据损坏回退规则精确删除该目录内容。验收时工作区 `target/` 与外部
`/private/tmp/noname-next-cargo-target` 均不存在；`.npack`、`extensions/`、立绘、许可证、源码和
文档保留且哈希不变。本 ADR 只证明 3D精选第三个武将闭包，不代表整个扩展迁移完成。
