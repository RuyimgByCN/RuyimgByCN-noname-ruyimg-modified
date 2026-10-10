# ADR 0023: 目标侧伤害属性修正

- 状态：接受
- 日期：2026-07-14

## 决策

新增通用 DSL 修正目标 `damage_taken_nature`，在伤害来源侧属性修正之后、目标侧属性免疫与伤害值修正之前，以 `override 0/1/2` 分别表示普通、火焰和雷电伤害。失去体力不进入该通道。

MOBILE `tiansuan2_2` 用该通道合并旧 `group: ["tiansuan2_fire", "tiansuan2_ai"]` 中的权威改属性行为，再用 `damage_taken_final_amount min 1` 表达 `damageBegin4` 的最终封顶；`tiansuan2_fire` 仍以独立 DSL 定义保留稳定技能身份。

## 兼容性

新枚举变体追加在末尾，既有 npack 的序列化变体索引不变。本扩展与 ADR 0022 同批纳入 `EngineSemanticsId` 修订 7，并以 Native/WASM 一致性门禁验收。
