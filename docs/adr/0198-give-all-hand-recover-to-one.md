# ADR 0198：交出全部手牌后将濒死目标回复至一点

## 背景

四象标贾逵「挽澜」在另一名角色进入濒死时，可限定发动：先交给该角色全部手牌，再将其体力
回复至 1。旧 JavaScript 对 `give` 使用 `await`，因此卡牌移动及其嵌套触发必须先完整结算，
恢复不能提前修改权威体力。

已有单张给牌和“摸牌后给牌”效果都带有不同的选牌或后续分支契约，不能安全复用为全部手牌
的无选择转移。

## 决策

新增 Rule VM ABI 125 通用效果：

```text
give_all_hand_recover_to_one(player, target)
```

初始阶段验证双方存活、目标不是自身且拥有者有手牌，把拥有者当时的全部手牌以 `Give` 原因
移动到目标手牌区。引擎把恢复提交态放入 deferred tail；移动事件和由其产生的嵌套触发全部
结束后，重新读取目标当前体力，只补至 1，不会把已经高于 1 的目标降回 1。

同时新增 `owner_alive_other_dying_hand_nonempty_once_per_game` 过滤器。它只在存活拥有者有
手牌、另一名存活角色处于非正体力且技能实例尚未发动时开放可选触发；确认后使用既有
`lifetime_fire_count` 消耗限定次数，拒绝不消耗。

## 版本与确定性

- Rule VM ABI：124 → 125
- Rule IR：86 → 87
- Engine Semantics：169 → 170
- HOST_EFFECT：140
- `EffectRequest` 哈希标签：177

原生端、WASM、联机握手和包含该效果的内容包必须同时声明 ABI 125。
