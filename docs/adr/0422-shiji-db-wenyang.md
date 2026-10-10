# ADR 0422：SHIJI 标文鸯闭环（追锋 / 曲迪 / 重剑，ABI 419 / 420）

本批发布 SHIJI 标文鸯（`db_wenyang`，3732811817，男/魏/4 血）。技能「追锋」
（`dbzhuifeng`）、「曲迪」（`dbquedi`）、「重剑」（`dbchongjian`）与既有
「却敌」（`dbchoujue`）完成闭环；旧 JavaScript 仅作语义来源。

## 决策

- **ABI 419 `resolve_dbzhuifeng(player)`**：出牌阶段限两次的主动技——失 1 点体力
  （`DealDamage` + `is_lose_hp`，可濒死）后选择一名其他角色并对其视为使用虚拟
  【决斗】（`UseVirtualCardOn`）。一段选目标 choice（`RESOLVE_DBZHUIFENG_TARGET`）。
- **ABI 420 `resolve_dbquedi(player, target, card)`**：单目标【杀】/【决斗】指定后
  三选一——获得目标一张手牌 / 弃置一张基本牌并令此牌对其伤害 +1
  （`IncreaseCurrentCardBaseDamage`）/ 背水（减 1 点体力上限并依次执行）。
  新过滤器 `owner_source_sha_or_juedou_single_target`（来源使用杀/决斗且仅一个
  目标）。一段三选一 choice（`RESOLVE_DBQUEDI_CHOICE`）。
- **dbchongjian（重剑）**：纯 DSL `view_as`——装备区装备牌当【杀】使用/打出、
  当【酒】使用（`from equip`）。
- 顺带补齐并行会话（omp，ABI 540-542 孟获）缺失的 ABI 常量与
  `TriggerContext::damaged_targets` 字段（31 处构造点批量补 `Vec::new()`）。

## 验证

- 新增 `original_shiji_db_wenyang.rs` 3 个行为测试：追锋失体+选目标、曲迪三选一
  获牌分支、重剑 view_as 注册。
- 打包：62 技能 / 17 武将，`rule_vm_abi_version` 502。
