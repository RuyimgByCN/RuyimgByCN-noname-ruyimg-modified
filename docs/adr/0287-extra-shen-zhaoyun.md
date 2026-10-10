# ADR 0287：EXTRA 神赵云与多实体牌视为牌

## 决定

旧 JavaScript 只作为语义合同，神赵云的 `relonghun` 重写为声明式 `view_as`：

- `up_to 2 same_suit` 允许主动用牌或响应时选择一至两张同有效花色的手牌/装备牌。
- `nature fire` 令方片分支进入统一的火【杀】结算。
- `multi_card_bonus` 在选择两张来源牌时声明伤害 +1、回复 +1，或响应结算后弃置当前
  回合角色一张手牌/装备牌。
- 响应窗口使用稳定的不透明技能选项打开二次实体牌选择，避免把两张普通响应牌误判为
  技能成本；每个选择屏障恢复时重新校验技能、牌归属、区域、花色和禁用规则。

神赵云与既有 `xinjuejing` 一并进入 EXTRA 角色闭包。npack 和运行时均不包含或执行旧 JS。

## 兼容性

未增加技能专用 HOST_EFFECT 或 `EffectRequest`。`view_as` 包索引仅尾部增加可选字段，
续体标签 146 只用于多实体牌响应选择。Rule VM ABI 为 211，Rule IR 为 97，
Engine semantics revision 为 185；内容 schema、包格式、网络、回放与快照 schema 不变。
