# ADR 0221：OL 将驰的摸牌或重铸选择

## 背景

ONLYOL `oljiangchi` 在摸牌阶段结束后允许取消，或选择摸一张牌并限制本回合【杀】，或重铸一张
手牌/装备牌并强化本回合【杀】。旧 JavaScript 的 `chooseControl`、临时子技能和 AI 回调不能
直接进入确定性运行时。

## 决策

ABI 148 增加
`choose_draw_or_recast_add_skill(player, draw_skill_def, recast_skill_def)`（HOST_EFFECT 156）。
效果复用现有摸牌、`RecastCards`、添加技能和 modifier：

- 初始 Choice 可取消；没有合法重铸牌时只提供摸牌。
- 摸牌分支先添加回合临时助手，再摸一张牌。
- 重铸分支强制选择一张当前手牌/装备牌，先添加临时助手，再走共享重铸管线。
- Choice 恢复时重验来源技能、当前玩家、阶段和卡牌区域；两个助手都通过普通
  `turn_ended owner_is_target` 触发移除自身。

## 结果

ONLYOL `ol_caozhang` 由完整 `oljiangchi` 闭合。新 `EffectRequest` 使用稳定哈希标签 192，
选择续体使用稳定标签 105；旧 JavaScript 只保留为语义证据。
