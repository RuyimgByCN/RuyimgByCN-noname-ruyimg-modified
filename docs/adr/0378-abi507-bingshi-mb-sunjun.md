# ADR 0378：ABI 507 兵势孙峻凶图、险帅

旧 `bingshi/skill.js` 仅作为语义证据，JavaScript 不进入新运行时。

- `first_hand_card_suit_use_unlimited` 复用本回合实体手牌有效花色历史，在记录本次使用前
  查询，从而同时跳过且不消耗普通次数限制。
- `resolve_mbxiongtu` 复用目标手牌展示、弃牌堆花色历史、弃牌、伤害与摸牌管线；技能状态
  仅保存待奖励、出牌阶段第二次发动许可和自身伤害排除位；离开出牌阶段只清第二次许可，
  待奖励状态按旧版临时技能语义保留到本回合结束。
- `mb_sunjun.skill` 与角色索引进入 bingshi 包；Golden 覆盖花色次数豁免与凶图分支闭环。
