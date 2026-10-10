# ADR 0464：SHIJI 华歆闭环（怨清 / 疏陈，ABI 443 / 444）

本批发布华歆（`sp_huaxin`，3450000113，男/魏/3 血）完成闭环：「怨清」
（`yuanqing`，ABI 443）与「疏陈」（`shuchen`，ABI 444）作为完整武将进入选将池。
旧 JavaScript 仅作语义来源。

## 决策

- **ABI 443 `resolve_yuanqing(player, skill)`**：出牌阶段结束时（`phase_play_end
  owner_is_target`），读 `owner_card_category_mask`（本阶段用过的基本/锦囊/装备
  位）；对每个置位类别从弃牌堆该类牌中权威随机取一张（`RuleRng::from_state` +
  `gen_range_u32`，回写 `tx.state_mut().rng`），移入仁库（`ensure_skill_pile_zone`
  的怨清技能牌堆，旧 JS 全局 `_status.renku` 以拥有者技能牌堆近似）。
- **ABI 444 `resolve_shuchen(player, dying, renku_def, skill)`**：其他角色进入
  濒死时（`dying owner_alive_other_dying_once_per_round`，每轮一次），若仁库
  （`skill_pile_cards(player, renku_def)`）> 3 张——获得全部仁库牌并令濒死者
  回复 1 点体力。仁库跨技能共享以 `renku_def` 参数（DSL 传怨清 def id）传递。

## 验证

- `original_shiji_zhengsu.rs` 新增第 12 个测试：类别掩码 0b101 → 基本+装备两类
  各随机取一张入仁库 → 补至 4 张 → 疏陈获得全部 + 濒死者回复 1。
- 打包：90 技能 / 28 武将，`rule_vm_abi_version` 502（max 语义）。
- conformance 3/3 通过；我的 5 套件 27 测试全绿。
