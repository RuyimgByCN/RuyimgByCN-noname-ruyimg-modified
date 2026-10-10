# ADR 0196：单张给牌后的同花色弃置或失去体力

## 背景

REFRESH「反间」的旧 JavaScript 不是单纯的“给牌后弃牌”。它先把主动技能选择的唯一手牌
公开并交给另一名角色，等待移动及其嵌套触发完成，再由目标二选一：

1. 公开当前全部手牌，并弃置手牌区、装备区内所有与所给牌物理花色相同且可弃置的牌；
2. 失去一点体力。

若给牌完成后目标已经没有手牌，旧流程直接执行第二项。旧实现用
`target.storage.refanjian` 跨异步步骤保存牌对象。

## 决策

新增 Rule VM ABI 123 通用效果：

```text
give_selected_then_target_choose_discard_same_suit_or_lose_hp(player, target)
```

VM 只传入主动技能的单张选择和目标。引擎在移动前捕获实体牌的物理花色，并通过第二阶段
`EffectRequest` 把目标 Choice 放到 `GiveCards` 屏障之后。弃牌分支公开目标当前手牌，再按
权威弃置保护查询重新扫描手牌区和装备区；不会保存旧事件对象、区域快照或 JavaScript 闭包。

Choice 一旦打开，只依赖目标仍然存活并由目标本人回答。拥有者在给牌触发中死亡或失去技能时，
已开始的效果仍能完成。

## 版本与确定性

- Rule VM ABI：122 → 123
- Rule IR：84 → 85
- Engine Semantics：167 → 168
- HOST_EFFECT：138
- continuation：95
- `EffectRequest` 哈希标签：175

原生端、WASM、联机握手和内容包必须同时声明 ABI 123。
