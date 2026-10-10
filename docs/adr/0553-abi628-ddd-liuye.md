# ADR 0553：ABI 628 3D精选刘晔【察势/齐策】

本批按武将闭包发布刘晔（`ddd_liuye`，3136644997）、【察势】（2811969013）与【齐策】
（4207915436）。旧 JavaScript 只作语义证据，不进入 npack 或运行时。

【察势】在其他角色出牌阶段开始时可弃置一张手牌/装备牌，并保存该阶段角色、弃牌的有效花色
和逻辑类别。可执行旧 JS 的强制 helper 只在后来出现首张花色或类别匹配牌的 `useCard` 时触发，
随后双方按当前回合角色在先、刘晔在后的顺序各摸一张并清除记录；此前不匹配的牌不会消费记录，
阶段结束仍未命中则清除。译文写作“下一张牌结算结束后”，与代码的实际监听点不一致，本迁移
延续可执行代码语义。

【齐策】在准备阶段且当前未锁定时可摸两张并写入技能实例持久锁定位；该状态跨回合保留，自己
使用任意锦囊牌时清除，下一次准备阶段才可再次发动。实现复用标准弃牌、有效花色/类别查询、
摸牌和技能实例状态，没有新增独立牌型系统或第二套阶段缓存。

验收：刘晔 Golden 3/3，DDD 十二个武将累计 37/37，content-compiler 522/522、engine-api
4/4、Rule VM 10/10 与 5/5、engine-core lib 55/55、conformance 测试 3/3、可执行门禁 8/8，
以及 31 个扩展空 store 冷导入均通过，内容锁为
`f924129729b2e86a8b68f003c66b6b658c3a4f841e3c99fa5819554c9c5a4d7b`。

`official/ddd@0.1.0` 更新为 ABI 628，累计含 12 名武将、25 项技能、33 个模块、31 个触发、
2 项主动规则、5 项视为规则、3 项修正器和 12 项立绘资源；logic hash 为
`380f60cee75c15afa52ea2a71bac8db76eed506c08fd86a4bceb6dd50a5cf05a`，asset hash 为
`3664001b8cfc4c6098be7a5d05d84877d7c62caba6d595e4885e39d5d72f5eb8`。源码目录与
`extensions/ddd` 四份 npack SHA-256 均为
`c3f9110d135cb1c049283d2da8fa3d281a4fa5536b5d1c2b97920b06eb0b3d9a`；两份刘晔立绘
SHA-256 均为 `b92803dca97c1d9b3eff42af04473c7d9c70f6c926ba3f2c2b828ad9e83c58d2`。

按架构计划 §37.10，本批门禁结束后只清理 Rust/Cargo 编译产物与增量缓存：外部 target 删除
7,405 个文件（Cargo 统计 4.9 GiB）；工作区 `target/`、外部
`/private/tmp/noname-next-cargo-target` 与 `/private/tmp/noname-next-cargo-target-root` 验收时均
不存在。冷导入临时 store 也已删除；`.npack`、`extensions/`、立绘、许可证、源码和文档全部
保留，清理规则用于防止编译产物随迁移批次无限累积，不清理发布物。

本 ADR 只证明 3D精选第十二个武将闭包，不代表整个扩展迁移完成。
