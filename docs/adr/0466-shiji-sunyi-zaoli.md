# ADR 0466：SHIJI 孙翊闭环（躁厉，ABI 446）

本批发布孙翊（`sunyi`，3826635754，男/吴/4 血）完成闭环：「躁厉」（`zaoli`，
ABI 446）作为完整武将进入选将池。旧 JavaScript 仅作语义来源。

## 决策

- **ABI 446 `resolve_zaoli(player, mark, skill)`**：回合开始时有“躁厉”标记
  （DSL `phase_begin owner_is_target` + `if 0 < mark_count(self, def)` 条件，
  标记键 = 技能 def id）→ 移除全部标记（`RemoveMark` amount 0）并开“弃至少
  一张可全弃”弃牌 choice（`RESOLVE_ZAOLI_CHOICE` 续体）→ 续体结算：弃牌 +
  摸（标记数 + 弃牌数）张；总数 > 2 则失去 1 点体力（`DealDamage is_lose_hp`）。
- 「使用/打出牌后标记 +1（至多 4）」由 DSL `card_used owner_is_source` +
  `if mark_count(self, def) < 4` 条件 + `add_mark` 表达（与 TW 孙翊的躁厉
  变体区分：TW 版以阶段开始重铸驱动，本技能以标记驱动阶段开始结算）。
- 「出牌阶段仅可使用本回合获得的牌」复用既有 `hand_card_gained_this_turn_only
  override 1` modifier（ABI 380 管线）。
- **顺带语义修正**：`bootstrap_match` 开局发牌后清空 `gained_hand_cards`
  （开局发牌不计入“本回合获得”，与旧 JS 的 gain 历史自 gameStart 后开始一致；
  主公首回合 phaseDraw 仍计入）。

## 验证

- `original_shiji_zhengsu.rs` 新增第 14 个测试：2 枚标记 → 弃 1 张 → 摸 3 张、
  总数 3 > 2 失 1 体、标记清空。
- 打包：92 技能 / 30 武将，`rule_vm_abi_version` 502（max 语义）。
- conformance 3/3 通过；我的 5 套件 29 测试全绿。
