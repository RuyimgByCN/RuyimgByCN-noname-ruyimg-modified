# ADR 0091：选择续算不得越过伤害前置效果

## 决策

伤害提交请求保存在 `deferred_tail_effects` 时，只有 `deferred_effects` 与
`deferred_triggers` 都已清空，效果调度器才允许取出并执行该尾队列。

伤害生效前的技能可能先打开选择屏障，再继续执行防止伤害等后续效果。选择处理本身也会调用
效果调度器；若它只检查触发队列，就会在仍有前置效果等待续算时提前提交伤害。OFFLINE「惧鞭」
的“先精确弃牌、再防止本次伤害”覆盖这一顺序契约。

本修复不增加 Rule IR、VM 指令或内容包字段，Rule IR 保持 17，Rule VM ABI 保持 9；权威
调度顺序发生修正，因此 Engine Semantics 从 75 提升至 76。Native/WASM 与 conformance
哈希门禁必须重跑。
