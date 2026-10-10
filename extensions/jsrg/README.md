# JSRG 确定性移植

- `jsrgzuwang`：准备阶段和结束阶段将手牌补至体力上限。
- `jsrgbahu`：准备阶段摸一张牌，且每回合使用【杀】次数上限 +1。
- `jsrglangan`：其他角色死亡后回复 1 点体力并摸两张牌，攻击范围累计 -1（至多 -3）。
- `jsrgshili`：出牌阶段限一次，手牌中的装备牌当【决斗】使用。
- `jsrgzhaohan`：准备阶段按本局牌堆是否重洗，回复或失去 1 点体力。
- `jsrgchiying`：出牌阶段限一次，选择任意角色；其攻击范围内除技能拥有者外的角色
  依座次各弃一张手牌/装备牌。若其中基本牌数不大于目标体力，目标获得这些仍在弃牌
  堆的基本牌。完整武将 `jsrg_gaoxiang` 已发布（ABI 501 / ADR 0372）。

荀谌的 `jsrgsheju` 尚未完成，因此暂不登记整名武将。

> 交付物：`package.npack.json` 与 `jsrg.npack.json` 同源；详见 ABI-COORDINATION.md。
