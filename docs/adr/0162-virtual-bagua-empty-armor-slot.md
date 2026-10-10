# ADR 0162：空防具栏虚拟八卦响应

## 决策

ABI 87 增加 modifier `virtual_bagua_if_armor_slot_empty`。非零值只在拥有者存活、防具栏未废除且
没有实体防具时提供虚拟【八卦阵】响应；装备任意防具或废除防具栏后自动失效。

实体与虚拟八卦共用既有响应 token、判定、改判、`JudgmentResolved` 和响应 continuation。
实体八卦把装备牌写入 `cause_card`；虚拟八卦使用 `CardId::NONE`，不会伪造物理卡牌。

## 原因

旧 JS 的「八阵」通过额外装备系统和 grouped `bazhen_bagua` 模拟空防具栏八卦。运行时不加载该
JavaScript，也不按技能 ID 分支；内容包只声明通用 modifier，权威引擎负责统一查询和结算。

## 验证

Golden 覆盖空栏可用、实体防具占用、防具栏废除、红色判定抵消【杀】及虚拟原因牌为空。
