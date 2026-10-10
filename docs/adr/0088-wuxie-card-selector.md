# ADR 0088: 【无懈可击】使用修正采用通用牌选择器

## 决策

在声明式 `CardSelector` 末尾追加 `Wuxie`，DSL 名称为 `wuxie`。它可与既有
`card_use_unrespondable(selector)` 组合，表示技能拥有者使用的【无懈可击】不可再被响应。

旧 JavaScript 仍只作为语义证据；运行时只加载编译后的修正索引。该能力复用现有无懈链、
有效牌名和有效花色查询，不增加技能专用 Rust 分支，也不改变既有选择器序列化序号。

由于权威修正规则新增一种可解释选择器，Engine Semantics 从 72 提升至 73；Rule IR 与
Rule VM ABI 不变。Native/WASM 与 conformance 哈希门禁必须在本变更后重新执行。
