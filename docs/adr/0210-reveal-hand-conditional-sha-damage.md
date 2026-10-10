# ADR 0210：公开非空手牌后的逻辑【杀】条件伤害

## 背景

旧 SIXIANG「募讨」在准备阶段允许技能拥有者选择一名有手牌的角色，公开其全部手牌；若其中
有对该角色生效的【杀】，拥有者对其造成 1 点伤害。旧实现把目标选择、`showHandcards`、
`get.name(card, target)` 和异步伤害串在 JavaScript 事件链中，不能直接进入权威运行时。

## 决策

Rule VM ABI 137 增加
`choose_target_reveal_hand_damage_if_sha(player, amount)`：

- 候选为当前所有手牌非空的存活角色，包含 `player`；
- 引擎以稳定座次顺序发布强制单目标 Choice；
- Choice 恢复时重新验证技能实例、观察者、参与者存活和目标当前仍有手牌；
- 公开目标的完整当前手牌，并以目标为查询者调用共享 `effective_card_kind`；
- 至少一张牌是逻辑【杀】时，公开事件之后再走普通来源明确的伤害管线。

ABI 137 追加 HOST_EFFECT 149、continuation 101 和 `EffectRequest` 哈希标签 185。

## 结果

- 花色、牌名和 modifier 改写仍由共享查询决定，没有 `stdmutao` 专用牌名分支。
- 客户端不能选择空手、死亡或陈旧目标，公开与伤害顺序由事件流验证。
- 旧 JavaScript 只保留为语义证据，不进入 npack、扩展目录或运行时。
