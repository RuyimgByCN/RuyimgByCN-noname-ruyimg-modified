# ADR 0262: STANDARD「流离」保存中【杀】目标转移

ABI 186 增加过滤器
`owner_sha_target_can_discard_redirect_to_other_in_attack_range` 与
`choose_discard_redirect_current_sha(player, source, card)`（HOST_EFFECT 183）。

技能拥有者先选择一张可弃置的手牌或装备牌，再选择其攻击范围内、不是【杀】使用者且未在当前
目标列表中的合法角色；任一步取消都不支付费用。两项都确认后先结算弃牌及其嵌套触发，再通过
已有保存中【杀】目标槽替换函数提交转移。选择使用 continuation 122，`EffectRequest` 稳定
哈希标签为 223。大乔以「国色、流离」完整发布，旧 JavaScript 不执行。
