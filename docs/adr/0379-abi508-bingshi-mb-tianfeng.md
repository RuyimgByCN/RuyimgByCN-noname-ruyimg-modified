# ADR 0379：ABI 508 兵势田丰刚鲠、死谏

旧 `bingshi/skill.js` 仅作为语义证据，JavaScript 不进入新运行时。

- `resolve_mbganggeng` 复用多张手牌交给、全场手牌极值、摸牌和目标区域弃牌管线；技能状态只
  保存本回合待结算目标，并在阶段末结算后清除。
- `resolve_mbsijian` 让失去最后手牌与濒死事件共享每回合两次计数；分支选择可取消，“均执行”
  复用当前回合角色摸牌和一次性弃牌 helper，并只保存该分支的终身累计次数以计算后续失去体力。
- `mb_tianfeng.skill` 与角色索引进入 bingshi 包；Golden 覆盖刚鲠两种结算及死谏累计代价、
  一次性弃牌 helper 闭环。
