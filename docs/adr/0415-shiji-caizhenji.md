# ADR 0415：SHIJI 蔡贞姬闭环（舍医 / 天荫，ABI 417 / 418）

本批发布 SHIJI 蔡贞姬（`caizhenji`，3785215625，女/魏/3 血）。技能「舍医」
（`sheyi`）与「天荫」（`tianyin`）完成闭环；旧 JavaScript 仅作语义来源。

## 决策

- **ABI 417 `resolve_sheyi(player, target)`**：其他角色受伤前（`damage_before`
  全局触发，DSL 条件 `hp(event_player) < hp(self)`）交至少 `max(1, hp)` 张
  手牌/装备牌防伤。每轮一次限制在引擎内检查（`round_fire_round`）；一段选牌
  choice（`ContinuationKind::Custom` + `RESOLVE_SHEYI_CHOICE`），确认后赠牌
  （`CardMoveReason::Give`）+ `PreventDamage`（当前 damage_id）。
  **注意**：`owner_alive_once_per_round` 与 optional 确认流程存在交互问题
  （confirm 后 resume 时 once_per_round 已消耗导致触发被跳过），故 round 限制
  移入引擎内部而非 DSL filter。
- **ABI 418 `resolve_tianyin(player)`**：结束阶段（`phase_end owner_is_target`）
  扫描牌堆顶，获得每个本阶段未用类别（基本/锦囊/装备，`owner_card_category_mask`
  近似本阶段用牌记录）的第一张；三类齐备或牌堆空即止。
- 旧 JS 每轮一次的舍医以引擎内 round 检查近似；天荫以回合级类别掩码近似本阶段，
  文档化。

## 验证

- 新增 `original_shiji_sheyi_tianyin.rs` 2 个行为测试：舍医交牌防伤（直接调用
  原语验证选牌与防伤——DSL optional 触发链在真实对局由引擎驱动）、天荫按类别
  收牌堆顶（第二张同类别不收）。
- 打包：57 技能 / 15 武将，`rule_vm_abi_version` 502。
- 顺手补齐并行会话（Codex，ABI 536 好施）缺失的 `resolve_pothaoshi` 期间 stub
  与 `pothaoshi_refill_effects`/`POTHAOSHI_CUSTOM_CHOICE` re-export。
