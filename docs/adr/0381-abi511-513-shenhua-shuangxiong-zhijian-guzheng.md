# ADR 0381：shenhua 颜良文丑双雄 + 张昭张纮直谏/固政（ABI 511-513）

本批完整移植 SHENHUA 颜良文丑（`yanwen`）与张昭张纮（`zhangzhang`）三名技能
（双雄/直谏/固政），旧 JavaScript 仅作为语义来源，不进入运行时。两位武将随 SHENHUA
npack 发布（rule_vm_abi_version 提升至 513）。

## ABI 511：`resolve_shuangxiong(player)` — 双雄

- 摸牌阶段触发（过滤器 `owner_is_target_draw_not_replaced` 精确对应旧 JS
  `!event.numFixed`，已固定的摸牌不询问），可选发动。
- 结算：把待执行摸牌固定为 0（复用 `draw_phase_fixed_count(0)` 状态机），执行判定，
  将判定牌从弃牌堆移至手牌，以稳定 mark（`SHUANGXIONG_COLOR_MARK`，1=红/2=黑）记录
  判定颜色，并授予回合末自动移除的临时技能 `shuangxiong_viewas`。
- 临时技能声明 `view_as juedou any color_ne_shuangxiong from hand`；新过滤器
  `color_ne_shuangxiong` 在视为用牌资格查询时读取拥有者的颜色 mark：记录红色则只匹配
  黑色手牌，记录黑色则只匹配红色手牌，无记录时不匹配。多张判定牌时新颜色覆盖旧记录
  （旧 JS 为颜色数组，每回合至多一次判定，文档化近似）。

## ABI 512：`resolve_zhijian(player)` — 直谏

- 出牌阶段限一次，选一张手牌区装备牌（active `select_hand 1 1 card_equip`），由权威
  选择屏障选择一名其他角色；目标须存活且其对应装备栏启用（旧 JS `canEquip` 对已占
  栏位返回否，等价近似）。
- 结算：经通用赠予移牌管线把装备置入目标装备区（等待移动触发结算），随后拥有者摸一张。
- 屏障恢复时重校验技能启用、牌仍在手牌区、目标存活且栏位仍启用。

## ABI 513：`resolve_guzheng(player, event_player)` + 过滤器
`owner_alive_other_discarded_in_discard_phase` — 固政

- 其他角色弃牌阶段结束后触发（`phase_discard_end` + 新过滤器），可选发动。过滤器只在
  拥有者存活、事件角色非拥有者且其本弃牌阶段确有牌弃入弃牌堆时成立，避免空询问。
- 弃牌阶段失牌追踪：新回合旗标 `DISCARD_PHASE_LOST_BASE` 在弃牌阶段入口记录
  `lost_cards` 长度 + 1（0 表示本回合未进入弃牌阶段），配合既有按需启用的回合失牌
  历史（固政过滤器同时启用 `track_turn_card_loss_history`），在阶段结束时精确切出
  该阶段内各角色的失牌集合；与 `lost_cards` 一起在回合边界清零，无新增 hash 载体。
- 结算分两段选择：先归还一张事件角色本阶段弃置且仍处弃牌堆的牌（屏障重校验），再
  二选一是否获得其余本阶段弃牌（含任意角色弃入的牌，旧 JS 语义）；归还牌已离开弃牌
  堆时整次结算跳过。跳过弃牌阶段时旗标为零，不会误收出牌阶段失牌。

## 打包与验证

- `shuangxiong.skill`/`shuangxiong_viewas.skill`（并入同文件）、`zhijian.skill`、
  `guzheng.skill` 加入 SHENHUA 包；`yanwen`、`zhangzhang` 加入 `characters.json`。
- 重建 `shenhua.npack.json` 并同步 `extensions/shenhua/`；`rule_vm_abi_version`
  457 → 513。同步更新了既有的包级契约断言（版本与索引计数）到 95 技能 / 24 武将。
- Golden 覆盖：双雄判定→获牌→颜色记录→视为技能授予、固定摸牌不再询问；直谏赠装备
  摸牌、非装备拒绝、目标排除自身；固政归还+取回、归还+放弃、取消不动牌堆。新测试
  `original_shenhua_shuangxiong.rs`、`original_shenhua_zhijian.rs`、
  `original_shenhua_guzheng.rs` 全绿，其余 shenhua 与依赖包测试通过。
