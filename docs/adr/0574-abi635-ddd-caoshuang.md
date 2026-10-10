# ADR 0574：ABI 635 3D精选曹爽【专摄/危秋】

本批按武将闭包发布曹爽（`ddd_caoshuang`，1602336898）与【专摄】（`dddzhuanshe`，
902778848）、【危秋】（`dddweiqiu`，208021574）。旧 JavaScript 只作语义证据，不进入
npack 或运行时。

【专摄】在其他角色出牌阶段开始时可正面赠出一张手牌，并按技能实例记录该回合角色及去重的
逻辑牌名。该角色使用同名基本牌或普通锦囊时，先把此牌名记为已使用，再可为当前暂停结算的牌
增加一个合法目标；只忽略普通距离，不绕过目标合法性、牌自身多目标限制或重复目标检查。回合
结束时清空记录；若至少一个记录牌名未被使用，可对该角色造成 1 点伤害。

【危秋】要求真正的回复前替代，不能在 `HpRecovered` 后回滚。ABI 635 因此增加通用
`recover_before` 固定触发队列、规范化回复编号及 `PreventRecovery`/`CommitRecovery`；取消状态
进入 canonical hash，并在结算中止时清理。多个空手曹爽按稳定触发顺序各摸一张，本次回复只
取消一次。

验收：【专摄/危秋】Golden 2/2、content-compiler 525/525、engine-api 4/4、engine-core lib
55/55、Rule VM 单元 10/10 与集成 5/5、conformance 3/3、31 个扩展空 store 冷导入均通过；
内容锁为 `287ae37a2f187d6ff2362d7187533b298c8d059cb959ce5ad7fdf50bdaeee209`。

`official/ddd@0.1.0` 更新为 ABI 635，累计含 19 名武将、41 项技能、56 个模块、50 个触发、
6 项主动规则、5 项视为规则、4 项修正器和 19 项立绘资源；logic hash 为
`2b4387d4b66a431dc26bcbcc287333ab107ab0e075b272df2bd196accaba867f`，asset hash 为
`60c5308fd8cca59da3a9d929c6681c39a8117b29f96423705371c141c5ea664a`。源码目录与
`extensions/ddd` 四份 npack SHA-256 均为
`97236aa6864c4355a81f45de355843010c61f2f559a297fe10a52287e72bf489`；两份曹爽立绘 SHA-256
均为 `bb3f56af8286f7fe48aede5ea2930b32003244331ad7a2470ae6c7e4ca6d986e`。

编译产物按架构计划 §37.10 在本批门禁结束后清理；`.npack`、扩展、立绘、许可证、源码和
文档属于交付物并保留。本 ADR 只证明 3D精选第十九个武将闭包，不代表整个扩展迁移完成。
