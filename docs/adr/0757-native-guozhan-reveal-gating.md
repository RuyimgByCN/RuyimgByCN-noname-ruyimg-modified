# ADR 0757：国战明置门控与 M3 回归矩阵

日期：2026-09-22（第五十八批）
状态：已采纳
上游：[0756](0756-native-guozhan-m3-faction-victory.md) / [0755](0755-native-n11-faction-kof-m2.md)

## 背景

ADR 0756 交付的 Kingdoms 胜负臂按主将槽配置判定势力，未区分明置/暗置——
「暗置武将无势力」的门控语义与对手不可窥探暗置将的信息隔离均未落地；
M3 的多点回归矩阵也尚未补齐。本批闭合国战 M3。

## 决策

1. **声明势力（门控核心）**：`GameStateCore::declared_faction(p)`——主将
   明置按主将势力；主将暗置而副将明置按副将势力（半明置）；双暗置或槽空
   → None（势力未定）。目录侧配套 `faction_of_general(g)`（下标→势力，
   越界 None），胜负臂改按 `declared_faction` 判定：任一存活角色势力未定
   → 不判负；全场唯一声明势力 → `Winner::Faction` 胜。此为对 ADR 0756
   判定输入的**更正**（由「槽配置」收紧为「已明置」）。
2. **明置规则机（命令面）**：`GameAction::RevealGeneral { slot: u8 }`
   （0=主将 1=副将），校验链：仅国战模式（`WrongMode` 新增）→ 槽号合法
   （`IllegalTargets`）→ 自己回合（`NotYourTurn`）→ 出牌阶段
   （`WrongPhase`）→ 槽已配置（`NoSuchGeneral` 新增）→ 未明置
   （`AlreadyRevealed` 新增）。明置行为公开、无隐藏事件载荷；成为牌目标
   时的补偿明置时机留待后续批次。
3. **视图信息隔离**：`PlayerCellView` 新增 `generals: [Option<GeneralView>; 2]`
   （名称+势力）与公开的 `revealed_generals: [bool; 2]`。投影规则：本座位
   两槽可见（含自己的暗置将）；他人/观战者只见明置槽，暗置槽恒 `None`
   ——将领与势力一律不泄露。视图为纯投影，不入快照与摘要（N10 约定）。
4. **M3 回归矩阵**：`guozhan_m3_victory_regression_matrix`——8 种子 ×
   {全明置双杀终局、同种子逐哈希确定性、第一杀后存档→恢复续行与不间断
   推演终态逐哈希一致、全暗置独存不判负}。

## 验收

- `guozhan_reveal_gating_view_isolation`：六段校验链 + 明置前后他人/
  观战者/本人三视角隔离 + 明置入哈希 + 重复明置拒绝。
- `guozhan_m3_faction_victory`（改造）：全明置终局、槽空/暗置不判负、
  半明置副将势力参与判定（吕布群+大乔吴仅副明 → 吴胜）、存档与字节层
  往返。
- 全量 **194 passed / 0 failed**（native_kernel 117 + boundary_snapshot 59
  + view_isolation 7 + auth_archive 6 + replay_scenarios 2 + src 2 + dbg4 1）。

## 后果

- 国战 M3（势力胜负 + 回归矩阵）与明置门控全部闭环；国战剩余项仅明置
  时机扩展（补偿明置）与武将牌配置流程的宿主侧装配。
- 无规则版本变化：`revealed_generals` 早已入哈希与编码（版本 20），
  `GameAction`/视图均不参与状态编码，14 golden 复放零变化。
- 视图结构体新增字段为纯增量，`AiSession`/`view_isolation` 既有 7 项
  隔离验收不受影响。
