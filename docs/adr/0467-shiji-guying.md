# ADR 0467：SHIJI 孤影（ABI 447）

本批发布向宠（`xiangchong`）首技能「孤影」（`guying`，ABI 447）；武将待
「睦阵」三选一落地后发布，技能先行入包。旧 JavaScript 仅作语义来源。

## 决策

- **ABI 447 `resolve_guying(player, actor, card, stage, skill)`**：你失去一张
  手/装/判区牌后（新过滤器
  `owner_lost_one_hand_equip_judge_other_current_phase_once`：cards_moved 事件、
  拥有者为丢失方、恰一张、来源区为手/装/判、当前回合角色为他人、每回合一次）
  当前回合角色须二选一（`RESOLVE_GUYING_CHOICE` 续体）：
  - 0 随机交给你一张其可获得的牌（`gainable_target_cards` + `RuleRng` 权威随机
    + `MoveCardsWithReason` Give）；
  - 1 你收回失去的牌（若仍在弃牌堆）。
  单选项时直接结算（不打开选择）。

## 验证

- `original_shiji_zhengsu.rs` 新增第 15 个测试：失去一张手牌（经引擎移牌管线）→
  当前回合角色二选一 → 选收回 → 牌回到拥有者手牌。
- 打包：93 技能 / 30 武将，`rule_vm_abi_version` 502（max 语义）。
- conformance 3/3 通过；我的 5 套件 30 测试全绿。
- 顺带协作补位：Codex 势曹双/势于吉/拒战等中间态 stub、TurnState
  `use_ban_pairs` Default 补齐、ABI 570/571 常量。
