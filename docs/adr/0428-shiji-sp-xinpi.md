# ADR 0428：SHIJI 辛毗闭环（隐据 / 持节，ABI 421 / 422）

本批发布 SHIJI 辛毗（`sp_xinpi`，3953044318，男/魏/3 血）。技能「隐据」
（`spyinju`）与「持节」（`spchijie`）完成闭环；旧 JavaScript 仅作语义来源。

## 决策

- **ABI 421 `resolve_spyinju(player, target)`**：出牌阶段限一次的主动技——目标
  二选一（`RESOLVE_SPYINJU_CHOICE`）：
  - 对拥有者使用一张手牌【杀】；
  - 或获得「隐据·跳过」标记——由纯 DSL 辅助技能 `spyinju_skip` 在目标回合的
    **准备阶段**消费（`skip_next_play_phase` + `skip_next_discard_phase` +
    移除标记）。「跳过下回合出牌/弃牌」的跨回合语义以标记 + 目标回合触发表达，
    避免引入跨回合状态（无 hash 变化）。
- **ABI 422 `resolve_spchijie(player, source, card)`**：成为其他角色单目标牌
  （`owner_targeted_by_other_card_once_per_turn`）的目标时（可选）：判定
  （`flip_judgment` + `Neutral`），点数 > 6 则取消该牌对你的指定
  （`exclude_saved_sha_target` 保持排除；当前实现仅覆盖【杀】目标，
  锦囊目标的取消文档化近似）。

## 验证

- 新增 `original_shiji_sp_xinpi.rs` 3 个行为测试：隐据跳过标记、隐据用杀分支
  （`UseCard` 成功）、持节无目标上下文安全返回。
- 打包：65 技能 / 18 武将，`rule_vm_abi_version` 502。
- 顺带补齐并行会话（omp/Codex）缺失的 ABI 545-547 常量、`VmHost::event_targets`
  trait 方法与 `POTCEDUAN`/`ZHENGRONG`/`HONGJU`/`QINGJIAN` 常量 re-export。
