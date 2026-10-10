# ADR 0492：ABI 597 XIANDING 陈泰

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。陈泰（`chentai`，3710119822）复用本包
既有「沉勇」（`dcchenyong`，2745284067），并补全「矜贤」（`dcctjiuxian`，3598325442）：
出牌阶段限一次，重铸向上取整的一半手牌，再通过普通用牌管线视为使用一张【决斗】；该实体
虚拟【决斗】对其原始目标造成伤害后，可令伤者攻击范围内一名已受伤且并非陈泰的角色回复 1 点
体力。

ABI 597 新增 `resolve_dcctjiuxian(player, event_player, action)` 和对应精确伤害过滤器。实现复用
既有重铸、虚拟牌目标选择、普通锦囊响应、攻击范围及回复管线；技能实例只暂存本次虚拟牌与
原始目标，并在整张牌结算后清理。共享虚拟牌入口同时补齐来源技能与逻辑牌种上下文，不执行旧
JS helper，也不引入脚本运行时。

验收：陈泰 Golden 2/2、XIANDING 39 个专项测试目标共 135 项、content-compiler 495/495、
Rule VM 10/10、engine-api 4/4、conformance 8/8 与 26 包空 store 冷导入全部通过，ContentLock 为
`851a35afb849d7ae042bb0a2dd5025c0f41767c46840b0e666e6d9e031af7b03`。打包结果为 ABI 597、
16 名武将、85 项技能、90 个模块，logic hash 为
`fa8c60eaa8d17a0694aa83613e426a14ef5d05fa9192b1834655503ea00eb43c`；源码与
`extensions/xianding` 四份 npack SHA-256 均为
`8b7eaf07bc7ada828554333759af54c28c2676d535eef5175936ee3f29c4ca44`。原始内容同步器当前被共享
工作区另一批尚未闭环的 SHENHUA `huashen` 空 DSL 技能拒绝，陈泰包本身及冷导入不受影响；该
并发批次闭环后再恢复全局同步器双检。

编译产物按架构计划 §37.10 在全部并行编译消费者释放后强制清理。清理范围只包含工作区
`target/`、共享 `CARGO_TARGET_DIR` 及其中 Rust/Cargo 增量缓存；`.npack` 和 `extensions/`
发布物明确保留，避免把可交付内容误当缓存删除。本批删除工作区 `target/` 的 939 个文件
（Cargo 报告 210.5 MiB）及共享目录的 14,372 个文件（Cargo 报告 7.5 GiB），两目录均确认
不存在；清理后四份 npack SHA-256 仍完全一致。
