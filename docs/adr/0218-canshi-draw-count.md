# ADR 0218：残蚀摸牌计数与逐目标弃牌

## 背景

四象「残蚀」把摸牌数改为场上受伤角色数且至少为一；拥有者以主公身份启用「归命」时，还要
计入其他吴势力角色。随后本回合每次使用【杀】或普通锦囊指定受伤目标，都须弃置一张手牌或
装备牌。

## 决策

Rule VM ABI 145 追加：

- modifier 值源 `alive_wounded_or_guiming_wu_count_min_one`；
- 过滤器
  `owner_source_sha_or_ordinary_trick_target_wounded_has_discardable_hand_equip`。

技能直接组合既有 `add_skill`、`remove_skill` 与 `discard_one_hand_or_equip`；「归命」按精确
技能定义、主公身份和有效技能状态判断。无需新增 HOST_EFFECT、continuation 或
`EffectRequest` 变体。

## 结果

Golden 覆盖受伤角色与吴势力去重计数、非主公不享受归命、逐目标强制弃牌及回合末移除临时
技能。标孙皓通过 SIXIANG 对 SP 的依赖复用「仇海」「归命」。
