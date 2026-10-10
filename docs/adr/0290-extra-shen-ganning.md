# ADR 0290：EXTRA 神甘宁确定性重写

## 决策

旧 `drlt_poxi`、`drlt_jieying` JavaScript 仅作为语义参考，不复制或执行。魄袭使用 typed
effect 与选择续体，目标手牌只经技能拥有者的观察者投影公开，提交时重验双方手牌归属及
四种有效花色；弃牌完成后再按拥有者弃牌数结算。

劫营使用稳定标记 `2172383355`。每个存活技能实例通过 marked-player 全局 modifier 独立
提供摸牌数、杀次数和手牌上限加成；typed effect 负责唯一标记、结束阶段转交，以及其他
携营角色回合结束后的清标记和全部手牌转移。

## 版本

- Rule VM ABI：215
- Rule IR：100
- Engine semantics：188
- HOST_EFFECT：214–215
- `EffectRequest` 稳定哈希：254–255
- 续体：152–153
