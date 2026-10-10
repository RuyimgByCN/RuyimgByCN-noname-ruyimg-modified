# ADR 0446：SHIJI 整肃系统 + 朱儁 / 吕范闭环（ABI 432 / 433 / 434）

本批移植核心库通用「整肃」系统（`zhengsu_leijin/bianzhen/mingzhi`，ABI 432）、
朱儁「侯封」（`houfeng`，ABI 433）与吕范「燕集」（`spyanji`，ABI 433）/「点才」
（`mbdiancai`，ABI 434）；朱儁（`sp_zhujun`，2632350505，男/群/4 血）与吕范
（`sp_lvfan`，2691901687，男/吴/3 血）完成闭环，作为完整武将进入选将池。
旧 JavaScript 仅作语义来源。

## 决策

- **ABI 432 `resolve_zhengsu(player, mode, stage, card, skill)`**：整肃三条件
  的统一状态机。mode 0 垒金：card_used 时把点数序列写入技能实例 `state_blob`
  （"ZS0"+fail+ranks 段），非严格递增即记失败，弃牌阶段末判定长度 ≥ 3 且未失败；
  mode 1 变阵：弃牌阶段末由花色掩码（`play_phase_card_suit_history_by_player`
  popcount == 1）+ 用牌计数（≥ 2）判定；mode 2 明志：由
  `discard_phase_discards_for` 判定花色全异且 ≥ 2。判定成功时受益者（"GR" 授予者
  段 + 自己）各自 `ChooseDrawOrRecover(2, 1)`（满血自动摸二），随后移除临时整肃
  技能；失败仅移除技能。授予者段与跟踪段以 0xFE 分隔共存于同一 state_blob。
- **ABI 433 `resolve_houfeng(player, target, mode, stage, skill)`**：出牌阶段开始
  选整肃类型（未拥有的 1-3 项 choice）授予目标——mode 0 侯封（授他人，把授予者
  写入目标整肃实例 GR 段供奖励共享，旧 JS `houfeng_share` 授予者名单）；mode 1
  燕集（授自己，无授予者）。新过滤器
  `owner_alive_other_play_phase_zhengsu_missing_in_range`（他人出牌阶段开始、
  缺至少一种整肃、拥有者在目标攻击范围内——`dcchiying_in_range`）与
  `owner_is_target_missing_zhengsu`（自己缺整肃）。
- **ABI 434 `resolve_mbdiancai(player, actor, skill)`**：其他角色出牌阶段结束时
  （`phase_play_end owner_alive_not_target`），本回合失牌数 ≥ 体力且手牌 < 上限
  时摸至上限（`turn.card_loss_count`，旧 JS 出牌阶段内失牌以回合级近似）。
- 三个整肃技能均为临时技能（弃牌阶段判定后自移除），与核心库
  `zhengsu` 的 charlotte 语义一致。

## 验证

- 新增 `original_shiji_zhengsu.rs` 7 个行为测试：垒金成功摸二并移除、垒金失败无
  奖励、变阵成功、明志成功、侯封授技 + 授予者共享奖励、燕集自授、点才摸至上限。
- 打包：83 技能 / 25 武将，`rule_vm_abi_version` 502（max 语义）。
- conformance 3/3 通过，无 golden 漂移；engine-core 41 套件全绿
  （`original_clan_draw_to` 栈溢出为 E 段 Hermes 活跃领域，未动）。
- 顺带协作补位：Hermes `modify_current_damage`（ABI 562）codegen/vm/hash 缺臂、
  势周瑜（ABI 561）stub 重复定义与类型错误、potweiyan 迁移残留等并行中间态修复。
