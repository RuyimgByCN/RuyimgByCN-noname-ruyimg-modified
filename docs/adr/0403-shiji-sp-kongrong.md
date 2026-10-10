# ADR 0403：SHIJI 孔融闭环（新礼让 / 新名士，ABI 411）

本批发布 SHIJI 孔融（`sp_kongrong`，2320817766，男/群/3 血）。技能「新礼让」
（`xinlirang`）与「新名士」（`xinmingshi`）完成闭环；旧 JavaScript 仅作语义来源。

## 决策

- **xinlirang（新礼让）**：纯 DSL 组合，无新增 ABI——
  - 其他角色摸牌阶段开始（可选，无“谦”标记时）：令其多摸两张（临时技能
    `xinlirang_more` 的 `draw_count add 2`，回合结束移除）、自己得 1 枚谦标记、
    临时获得截获技能 `xinlirang_gain`。
  - 截获复用 ABI 461 劫粮管线（`resolve_jieliang_gain`，当前回合角色弃牌阶段
    弃牌后选一张弃牌获得；旧 JS 为至多两张，文档化近似）。
  - 自己回合开始时若有谦标记：跳过判定与摸牌阶段并清空（ABI 450
    `skip_judge_and_draw_phases`；旧 JS 仅跳摸牌，文档化近似同时跳判定）。
  - 谦标记以技能定义 ID 2286538070 兼任。
- **ABI 411 `resolve_xinmingshi(player, source)`**：受伤后（有谦标记）伤害来源
  强制弃一张手牌/装备/判定区牌（`TargetCardScope::HandEquipJudge`）；弃红色牌
  拥有者回复 1，弃牌进入弃牌堆则拥有者获得。一段强制弃牌 choice
  （`ContinuationKind::Custom` + `RESOLVE_XINMINGSHI_CUSTOM_CHOICE`）。
- 武将技能顺序对齐旧 JS（xinlirang/xinmingshi）。

## 验证

- 新增 `original_shiji_xinlirang_xinmingshi.rs` 5 个行为测试：礼让发动
  （摸牌 +2/谦标记/临时截获技能）、截获弃牌、让渡跳过并清空标记、
  新名士红色弃牌回复、黑色弃牌获牌。
- 打包：51 技能 / 11 武将，`rule_vm_abi_version` 502。
- 全量门禁：除并行会话活跃领域的过时断言（refresh RE 黄月英 ABI 292 vs 356、
  bingshi 郭援 ABI 525 vs 529）外全部通过。
