# ADR 0364：ABI 487 营图邻位取予 + HUICUI 陈珪

本批完整移植 HUICUI 陈珪（`chengui`，3537583790）的「营图」
（`dcyingtu`，3170147690），并与既有「从势」闭合武将。旧 JavaScript 仅作为语义
来源，不进入运行时。

## 原语设计（ABI 487）

- 过滤器 `owner_alive_neighbor_gained_cards_outside_own_draw_once_per_turn` 精确限制为存活
  邻位获得手牌、非该角色摸牌阶段、目标存在可获得手牌/装备牌，且每回合一次。
- `resolve_dcyingtu(player)` 复用通用可获得牌判定、实体移牌和普通用牌管线：获得触发
  邻位一张牌，再交给另一侧邻位一张牌；所给装备仍在其手牌且合法时立即使用。
- 邻位由当前存活座次统一计算，两人局自动去重。

## 打包与验证

- `dcyingtu.skill` 与 `chengui` 加入 HUICUI 包并同步 `extensions/huicui/`。
- 黄金测试覆盖可选发动、邻位取牌、对位交牌和每回合防重入。
