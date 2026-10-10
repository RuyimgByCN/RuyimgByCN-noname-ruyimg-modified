# ADR 0220：OL 倾袭的攻击范围选择

## 背景

ONLYOL `olqingxi` 与已迁移的 YIJIANG `qingxi` 不是同一规则：前者没有武器也能发动，目标从
手牌/装备区弃牌；有武器时不再二选一，而是依次弃牌、伤害加一并弃置来源武器。直接复用 ABI 59
会漏掉无武器分支并错误保留有武器二选一。

## 决策

ABI 147 增加 `owner_source_sha_direct_damage_before`，只匹配技能拥有者作为来源的直连逻辑
【杀】伤害前事件，排除铁索传导。

增加
`choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(player, target)`
（HOST_EFFECT 155）。效果绑定当前伤害 ID并快照来源攻击范围：

- 无武器时，目标在弃置与伤害加一之间强制选择；没有可弃牌时直接加伤。
- 有实体武器时，目标弃置 `min(攻击范围, 当前可弃牌数)` 张手牌/装备牌，然后绑定伤害及传导
  基数加一，最后弃置来源当前全部实体武器。
- 选择恢复统一重验伤害、技能、角色、攻击范围、武器状态、候选快照与卡牌区域；铁索传导不会
  再次触发。

复用现有伤害绑定、攻击范围查询、弃牌移动、武器查询与延迟效果队列；旧 JavaScript 仅作语义
证据，不进入包或运行时。

## 结果

ONLYOL `ol_caoxiu` 可由本包的 `olqianju`、`olqingxi` 闭合。新增 `EffectRequest` 使用稳定哈希
标签 191，选择续体使用稳定标签 104。
