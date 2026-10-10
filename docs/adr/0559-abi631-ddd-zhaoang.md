# ADR 0559：ABI 631 3D精选赵昂【奋计】

本批按武将闭包发布赵昂（`ddd_zhaoang`，3558500769）与【奋计】（`dddfenji`，782265781）。
旧 JavaScript 只作语义证据，不进入 npack 或运行时。

自己的摸牌阶段或弃牌阶段开始时，赵昂可选择当前内容牌池中一个本局未以【奋计】使用过、且
当前存在合法使用方式的普通锦囊牌。选择后立即永久记录牌名，并按该锦囊的普通目标规格和完整
结算管线虚拟使用；单目标、多目标、全体目标与【借刀杀人】有序双目标均沿用共享规则。

摸牌阶段把通常摸牌数固定为零，锦囊完整结算后统计该次结算产生的来源伤害总量并摸等量牌；
弃牌阶段跳过通常手牌上限弃置，改为按该锦囊声明目标数从手牌/装备区强制弃牌，可弃牌不足时
弃置全部。牌名记录保存在技能实例持久位中，不随阶段、回合或轮次重置。

ABI 631 增加 `resolve_ddd_zhaoang(player,action,stage,card,kind,count)`、
`owner_alive_ddd_fenji_available` 和可恢复自定义 Choice。实现复用现有普通锦囊目标规格、虚拟牌、
完整牌结算、阶段固定摸牌数、弃牌阶段跳过标记及通用手牌/装备弃置管线。

验收：【奋计】Golden 3/3、content-compiler 522/522、engine-api 4/4、engine-core lib 55/55、
Rule VM 单元 10/10 与集成 5/5、conformance 8/8 及可执行门禁均通过；31 个扩展空 store 冷导入
通过，内容锁为 `202a114692fb05546d7f76ed4bbcc72eda1d82d724d614972c11065641f3530d`。

`official/ddd@0.1.0` 更新为 ABI 631，累计含 15 名武将、32 项技能、41 个模块、38 个触发、
3 项主动规则、5 项视为规则、4 项修正器和 15 项立绘资源；logic hash 为
`1371bdadac3d0dfe2cd865aa4398f1257e70b8854337c01d2b068136dc4d6afe`，asset hash 为
`53c314d309e9fda87ef4da02897239ab863d715d21f926bf26cf7cf4ba5848a6`。源码目录与
`extensions/ddd` 四份 npack SHA-256 均为
`7255833aafb6b768b7d4c48630da3a2fd1148cb1bea403b613e1a74608475947`；两份赵昂立绘 SHA-256
均为 `cdf468939671d635aa1af6cae1c03707832a43908d059af9ca2870593b1d4b07`。

按架构计划 §37.10，本批门禁结束后只清理 Rust/Cargo 编译产物与增量缓存：外部
`/private/tmp/noname-next-cargo-target` 删除 6,717 个文件（Cargo 统计 5.2 GiB），本批冷导入临时库
同时删除；工作区 `target/`、外部 target 与临时库验收时均不存在。`.npack`、扩展、立绘、许可证、
源码和文档全部保留，避免编译产物随迁移批次无限累积。

本 ADR 只证明 3D精选第十五个武将闭包，不代表整个扩展迁移完成。
