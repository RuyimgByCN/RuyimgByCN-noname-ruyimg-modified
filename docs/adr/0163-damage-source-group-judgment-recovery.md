# ADR 0163：伤害来源势力触发与跨角色判定回复

## 决策

ABI 88 增加参数化过滤器 `owner_alive_other_damage_source_group_<group>`：只匹配存活技能拥有者观察到
另一名存活角色造成的伤害，且伤害来源的武将势力等于后缀。过滤器不绑定任何技能 ID。

同时增加 `judge_recover_other_on_suit(judge_player, recover_player, suit, amount)`。它为第一名角色
执行完整判定、改判和 `JudgmentResolved` 流程；最终有效花色匹配时，才由技能拥有者作为来源令第二名
角色回复体力。判定和回复对象都会在效果开始时按权威状态验证。

## 原因

旧 JS 的「暴虐」把主技能与 grouped `baonue2` 分开：其他群势力角色造成伤害后，由伤害来源决定是否
发动并亲自判定，黑桃时主公回复。既有同人判定回复效果不能表达两个不同角色，复制技能专用宿主逻辑
也不利于后续上千技能迁移，因此补充最小的通用组合能力。

## 验证

Golden 覆盖群/非群来源、排除主公本人、确认人归属、黑桃回复、非黑桃不回复、拒绝时不消耗判定牌，
并断言 `JudgmentResolved.player` 是伤害来源而 `cause_card` 为空。
