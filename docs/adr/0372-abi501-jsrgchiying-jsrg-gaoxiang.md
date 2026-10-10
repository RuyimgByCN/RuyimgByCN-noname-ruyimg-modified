# ADR 0372：ABI 501 江山如故驰应顺序弃牌、条件夺基 + JSRG 高翔

## 语义来源

`archive/upstream/noname/apps/core/character/jsrg/skill.js` 的 `jsrgchiying`：出牌阶段
限一次选择任意角色，预先固定其攻击范围内除技能拥有者外的存活角色，并从技能拥有者
座次起依次令这些角色各弃置一张手牌/装备牌；若其中基本牌数不大于目标当前体力，目标
获得这些仍位于弃牌堆的基本牌。旧 JavaScript 不进入运行时。

## ABI 501

- DSL 效果 `resolve_jsrgchiying(player, target)`。
- 状态机在首步固定座次顺序，逐人开启不可取消的单牌弃置选择；无可弃牌者直接跳过。
- 每次选择均重新校验当前回合、出牌阶段、技能实例、目标存活和可弃牌集合。
- 结算复用攻击范围查询、通用弃牌事件和 `dcchiying_gain_basics`；后者同时收紧为只获得
  仍在弃牌堆的基本牌，符合两版“驰应”的旧规则。

## 内容闭环

`jsrgchiying.skill` 与 `jsrg_gaoxiang` 加入 JSRG 包，生成 `.npack` 并同步至
`extensions/jsrg/`。黄金测试覆盖固定座次的两次强制弃牌以及仅获得基本牌。
