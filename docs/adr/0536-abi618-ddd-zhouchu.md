# ADR 0536：ABI 618 3D精选周处【侠横】

本批按武将闭包发布周处（`ddd_zhouchu`，2909135916）与锁定技【侠横】
（`dddxiaheng`，3492764267）。旧 JavaScript 只作语义证据，不进入 npack 或运行时。

出牌阶段开始时，周处强制选择一名存活角色；该角色从自己的手牌/装备区选择并弃置两张牌，
不足两张时弃置全部。弃牌移动及其触发完整结算后，周处再强制选择一名存活角色，对其造成
1 点普通伤害。伤害及其嵌套结算完成后，若两次选择均不是周处，周处减 1 点体力上限；若两次
选择为同一角色，再失去【侠横】。技能实例保存以此法执行伤害的不同目标；加入第三名不同目标
后永久关闭上述两项惩罚，但第三次仍先按原顺序结算惩罚。旧实现无条件在 `damage()` 后登记目标，
因此即使伤害被防止，也按同样时序登记。

ABI 618 新增 `resolve_dddxiaheng(player, action)`。实现复用标准 `SelectTargets`、目标本人
`SelectCards`、弃牌、伤害、减体力上限和移除技能管线，仅增加一个可恢复的三段 Choice 状态机
与技能实例内的删除位/不同目标列表。每次恢复均重验技能实例、观察者、角色存活、牌区、数量和
去重，弃牌与伤害后的后续阶段通过 deferred tail 接续，不跳过嵌套触发或濒死结算。

验收：周处 Golden 4/4、韩当回归 3/3、content-compiler 515/515、Rule VM 10/10 与 5/5、
engine-api 4/4、engine-core lib 55/55、conformance 3/3，以及 30 个扩展空 store 冷导入通过。

`official/ddd@0.1.0` 更新为 ABI 618，累计含 2 名武将、2 项技能、3 个模块、3 个触发和 2 项
立绘资源；logic hash 为 `3a147e4f74c8648d6c8b9c99dab5d12f941ce43acc10aeeec98a2d07ef356b1a`。
源码目录与 `extensions/ddd` 四份 npack SHA-256 均为
`72b5b5accc144d83a29d6d9bf5b319a413680312696f906aeea89b77a7455beb`；归档、源码和扩展目录三份
周处立绘 SHA-256 均为 `0f96f0e174cf2ce33e9ead7347f4e5325e8195d699f6d7af049e7c38016de39a`。

按架构计划 §37.10，门禁结束后删除外部共享编译目录 12,236 个文件（Cargo 报告 7.3 GiB）；
工作区 `target/` 与 `/private/tmp/noname-next-cargo-target` 均不存在，`.npack`、`extensions/`、
立绘、许可证、源码和文档保留。本 ADR 只证明 3D精选第二个武将闭包，不代表整个扩展迁移完成。
