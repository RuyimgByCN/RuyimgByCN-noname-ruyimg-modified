# ADR 0217：赠牌后的完整手牌公开与按有效牌名弃置

## 背景

四象「讨乱」在其他角色的结束阶段发动：拥有者先交给该角色一张手牌或装备牌，该角色再公开
此时的全部手牌并弃置其中所有【闪】。旧 JavaScript 的顺序是语义的一部分；公开范围必须包含
刚收到的牌，牌名判断也必须经过 modifier。

## 决策

Rule VM ABI 144 追加：

- 过滤器 `owner_alive_not_target_hand_equip_nonempty`；
- `reveal_hand_discard_kind(player, card_kind)`（HOST_EFFECT 153），公开手牌快照，再按目标视角
  的有效牌名弃置当前可弃匹配牌；
- `DrawThenGiveOne` 在抽牌数为零时直接打开既有赠牌 Choice，不再经过抽牌尾部续接。

新效果不引入 continuation；`EffectRequest` 稳定哈希标签为 189。

## 结果

技能 DSL 只组合赠牌与公开弃牌两个通用效果。Golden 覆盖接受、拒绝、自身结束阶段、拥有者
空手/空装备，以及交牌先于公开、公开先于弃置的事件顺序。
