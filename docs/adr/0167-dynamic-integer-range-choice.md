# ADR 0167：动态整数闭区间选择

## 决策

Rule VM ABI 92 增加无立即数操作码 `HOST_CHOICE_RANGE`。字节码先压入动态计算的 `min`、`max`，
操作码弹出边界并生成非负整数闭区间的选择令牌，随后复用现有 `Continuation`、`ChoiceRequest`
和 `VmResume` 恢复路径。DSL 暴露为表达式 `choose_range(min, max)`；区间必须有效且至多包含
255 个选项，否则确定性报错。

## 原因

旧「奇谋」必须提供从 1 到发动时当前体力值的每一个整数，固定参数的 `choose(...)` 无法表达
运行时变化的完整选项集。为「奇谋」增加专用 Effect 会把技能身份写入引擎，也无法复用于后续
数值选择技能；动态整数区间是能覆盖该语义的最小通用表面。

## 结果

- 上下界由普通 DSL 表达式和 Query 计算，不向 Rule VM 开放任意宿主代码。
- 暂停帧、选项令牌、快照、状态哈希和重放继续使用既有 Choice 屏障。
- 「奇谋」只组合 `choose_range`、`add_skill`、`add_mark` 和 `lose_hp`，旧 JavaScript 不进入运行时。
- 优化 VM 与参考 VM 共用相同 ABI 契约，并以跨 Choice 恢复测试保持一致。
