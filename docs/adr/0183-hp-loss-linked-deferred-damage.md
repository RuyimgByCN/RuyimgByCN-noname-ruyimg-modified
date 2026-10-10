# ADR 0183：自损濒死因果绑定与延迟伤害

## 背景

旧版 COLLAB「辰龙」在一次主动技能中选择 1 或 2：拥有者先失去等量体力，若这次自损令其进入
濒死则失去一点体力上限，完整结算濒死后再对另一名角色造成等量伤害。旧实现依赖
`event.getParent("loseHp").name` 和临时辅助技能；普通 `hp_lost`/`dying` 事件不携带足以安全
复原该父事件判断的技能来源。

## 决策

新增通用 typed effect
`lose_hp_then_damage_other_lose_max_hp_if_dying(player, target, amount, self_skill)`。编译器仅允许
它出现在带 `targets 1 1 other` 的主动技能中。Rule VM 发出公开请求后，引擎转换为追加的提交态
请求，验证技能归属并原子提交体力流失。

只有该提交产生 `EnteredDying` 时才减少一点体力上限。目标伤害进入
`resolution.deferred_tail_effects`，因此无论濒死窗口暂停等待救援、直接获救或最终死亡，伤害都
只在 dying/save 流程完整结束后继续。

## 兼容性

Rule VM ABI 从 107 提升到 108，Engine Semantics 从 154 提升到 155。新增 HOST_EFFECT 126；
公开请求与引擎提交态请求均追加在 `EffectRequest` 枚举末尾，状态哈希使用新的尾部标签。Rule IR
保持 v76，内容 schema、包格式、网络协议、回放和快照 schema 不变。ABI 108 主机继续加载
ABI 3–107 内容包。

## 结果

旧 JavaScript 仅作为语义参考，不进入 npack，也不在运行时求值父事件树。相同的“自损后精确
判定濒死、再延迟结算另一效果”需求可复用该 typed effect，而无需开放通用脚本回调。
