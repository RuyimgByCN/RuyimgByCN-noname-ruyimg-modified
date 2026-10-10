# ADR 0205：结算屏障后的存活目标绑定技能替换

## 背景

旧 SIXIANG「明伐」对体力值大于 1 的角色造成伤害，等待整次伤害结算后，若目标仍在场，则令
主技能失效并记录该目标；目标以后死亡或回复体力时恢复主技能。若在伤害前安装回复监听，目标
于濒死求桃中回复会错误地提前恢复；若伤害后立即替换，又会早于伤害后触发器和死亡结算。

## 决策

Rule VM ABI 132 增加三个可复用合同：

- 主动目标约束 `hp_gt N`，由服务端在提交时重新验证字面量体力下限；
- 过滤器 `owner_alive_skill_bound_target_is_event_player`，比较事件角色与技能实例的 `BND1`
  绑定目标；
- 效果
  `replace_skill_with_bound_after_resolution_if_target_alive(player, skill, skill_def, target)`，
  将引擎拥有的第二阶段排入 deferred-tail。当前效果、触发器和濒死求桃全部结束后，目标仍
  存活才添加绑定助手并移除原技能，目标已死亡则不改变技能。

助手在绑定目标的 `hp_recovered` 或 `death` 上重新添加主技能并移除自身。ABI 132 追加
HOST_EFFECT 144 和 `EffectRequest` 哈希标签 180，不增加 Choice continuation。

## 结果

- 濒死获救期间的回复不会被误认为“之后回复”，替换发生在获救完成后。
- 伤害直接杀死目标时主技能保持可用；非致命伤害才进入目标绑定禁用状态。
- 旧 JavaScript 只作为语义证据，不进入 npack 或运行时。
