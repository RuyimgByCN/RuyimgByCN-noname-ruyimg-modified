# ADR 0370：ABI 499 借兵随机获牌并使用装备 + 杨阜

本批完整移植 MOBILE 杨阜（`yangfu`，2919026286）的「借兵」
（`jiebing`，3832378886），并与既有「悍难」闭合武将。旧 JavaScript 仅作为语义来源，
不进入运行时。

## 原语设计（ABI 499）

- `choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip(player, source)` 在
  伤害结算后强制选择存活的第三方角色，排除技能拥有者和伤害来源。
- 选择与续接均重新使用共享的可获得牌查询；从目标手牌区、装备区的合集通过权威
  `RuleRng` 随机取得一张实体牌。
- 获得装备牌后复用普通 `UseCard` 流程尝试装备；状态变化导致不再合法时由统一用牌校验
  安全跳过，不新增技能专用装备逻辑。

## 打包与验证

- `jiebing.skill` 与 `yangfu` 加入 MOBILE 包并同步 `extensions/mobile/`。
- 黄金测试覆盖来源排除、随机获得装备后使用，以及武将技能闭包。
