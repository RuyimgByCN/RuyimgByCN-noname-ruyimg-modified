# ADR 0429：SHIJI 江庆闭环（上议 / 简议，ABI 423 / 424）

本批发布 SHIJI 江庆（`sp_jiangqing`，2297663542，男/蜀/3 血）。技能「上议」
（`spshangyi`）与「简议」（`spjianyi`）完成闭环；旧 JavaScript 仅作语义来源。

## 决策

- **ABI 423 `resolve_spshangyi(player, target)`**：出牌阶段限一次的主动技——
  展示目标手牌（`RevealCards`）并打开选牌 choice（`RESOLVE_SPSHANGYI_CHOICE`），
  获得所选牌（`MoveCards`）。
- **ABI 424 `resolve_spjianyi(player)`**：其他角色结束阶段（`phase_end
  owner_alive_not_target`）获得弃牌堆中的防具牌（`CardKind::Armor` 检索）。
  旧 JS 限定「本回合进入弃牌堆的防具」，以弃牌堆扫描近似，文档化。

## 验证

- 新增 `original_shiji_sp_jiangqing.rs` 2 个行为测试：上议展示获牌、简议获
  弃牌堆全部防具。
- 打包：67 技能 / 19 武将，`rule_vm_abi_version` 502。
