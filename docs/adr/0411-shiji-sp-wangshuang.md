# ADR 0411：SHIJI 王双闭环（异勇 / 善械，ABI 415 / 416）

本批发布 SHIJI 王双（`sp_wangshuang`，2460149242，男/魏/4 血）。技能「异勇」
（`yiyong`）与「善械」（`shanxie`）完成闭环；旧 JavaScript 仅作语义来源。

## 决策

- **ABI 415 `resolve_yiyong(player)`**：受伤后（伤害牌为【杀】、拥有者装备武器时）
  获得弃牌堆中该杀的实体牌；若来源存活且手牌含该实体，用该实体牌对来源反击
  （`play_as_kind_with_extra_damage_flags`，`ignore_distance=true`），来源无武器
  则伤害基数 2（`multi_card_bonus=1`）。事件牌与来源由 vm_host 从 damage 事件
  填充。
- **ABI 416 `resolve_shanxie(player)`**：出牌阶段限一次——牌堆（仅 DrawPile）有
  武器则获得一张；否则随机夺一名有武器角色的武器（权威 `RuleRng`）。
- 旧 JS 的「能对来源用杀」距离/次数校验以 `ignore_distance` 近似；无反击条件时
  仅获牌，文档化。

## 验证

- 新增 `original_shiji_yiyong_shanxie.rs` 3 个行为测试：异勇获牌+反击
  （来源无武器 2 点伤害、实体牌被消耗）、善械牌堆获武器、善械夺取武器。
- 打包：55 技能 / 14 武将，`rule_vm_abi_version` 502。
