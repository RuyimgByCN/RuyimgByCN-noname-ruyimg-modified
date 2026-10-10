# ADR 0539：ABI 621 3D精选丁奉【短兵】

本批按武将闭包发布丁奉（`ddd_dingfeng`，4008783817）与【短兵】
（`dddduanbing`，2748520735）。旧 JavaScript 只作语义证据，不进入 npack 或运行时。

【短兵】出牌阶段可把一张黑色非锦囊手牌或装备牌直接作为【兵粮寸断】置入自己的
判定区，摸两张牌，再可使用一张无距离限制、但仍计入通常次数的虚拟【杀】。之后丁奉
以牌造成伤害时，可把判定区内的全部【兵粮寸断】移至受伤角色判定区。上游译文把后一
条件写为“使用【杀】造成伤害”，但可执行旧 JS 只检查伤害具有牌上下文；本重写以可执行代码
语义为准。

ABI 621 增加 `resolve_ddd_duanbing(player,target,card,action)` 及精确伤害触发过滤器。
实现复用标准 Choice、判定区与延时锦囊重复校验、摸牌、虚拟【杀】、伤害触发和实体移牌管线；
选择恢复时重新校验观察者、技能、阶段、成本牌归属/颜色/类别以及自身判定区仍可接受【兵粮寸断】。

验收：丁奉 Golden 2/2，DDD 五个武将 Golden 15/15，content-compiler 518/518、
Rule VM 10/10 与 5/5、engine-api 4/4、engine-core lib 55/55、conformance 3/3，
以及 31 个扩展空 store 冷导入通过。

`official/ddd@0.1.0` 更新为 ABI 621，累计含 5 名武将、6 项技能、8 个模块、7 个触发、
1 项主动规则、1 项视为规则和 5 项立绘资源；logic hash 为
`953b068ae73860c162210da30a7e834aa7e11ef91eb7a4b06adb3ff417d125b4`，asset hash 为
`d5352286a0553a3e21d69ca47b52178bc5117b70da0f7d4c811e7577c323056c`。源码目录与
`extensions/ddd` 四份 npack SHA-256 均为
`f6115def3135adc16ba7e287af3cd5b649ad258076673f54fa39f6d0ed06aa5f`；两份丁奉立绘 SHA-256 均为
`c0f95ec8bb0d68f07f2e1785386957be0adf32bfaa5060d533f193113120d72f`。

按架构计划 §37.10，本批门禁结束后只删除 Rust/Cargo 编译产物和增量缓存：工作区 `target/`
原本不存在，外部 `/private/tmp/noname-next-cargo-target` 由 `cargo clean` 删除 13,993 个文件
（Cargo 报告 8.5 GiB），随后移除空目录，验收时两处编译目录均不存在。`.npack`、
`extensions/`、立绘、许可证、源码及文档全部保留，清理后发布包与立绘哈希不变。本 ADR 只证明
3D精选第五个武将闭包，不代表整个扩展迁移完成。
