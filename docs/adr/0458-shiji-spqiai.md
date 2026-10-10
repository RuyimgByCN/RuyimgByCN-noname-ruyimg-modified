# ADR 0458：SHIJI 弃哀（ABI 439）

本批发布王粲（`sp_wangcan`）首技能「弃哀」（`spqiai`，ABI 439）；武将待
「擅袭」檄标记机制落地后发布，技能先行入包。旧 JavaScript 仅作语义来源。

## 决策

- **ABI 439 `resolve_spqiai(player, target, card, stage, skill)`**：出牌阶段限
  一次（DSL `active select_hand 1 1 card_nonbasic targets 1 1 other`）——
  将一张非基本牌交给一名其他角色（`move_cards_with_reason` + Give，触发
  `CardsGiven` 事件）；满血直接摸两张；已受伤则目标二选一（0 令拥有者回复
  1 点体力 / 1 令拥有者摸两张，`RESOLVE_SPQIAI_CHOICE` 续体）。所选牌由
  vm_host 的 `selected_cards` 填充（照博明 `ResolveBoming` 模式）。
- 非基本牌选择由 DSL selection `card_nonbasic` 过滤器表达（既有能力）。

## 验证

- `original_shiji_zhengsu.rs` 新增第 9 个测试：受伤时赠牌并开二选一，目标选
  回复 → 拥有者回 1、目标获得该牌。
- 打包：86 技能 / 26 武将，`rule_vm_abi_version` 502（max 语义）。
- conformance 3/3 通过；engine-core 42 套件全绿（唯一失败
  `original_clan_draw_to` 为 E 段 Hermes 活跃领域，未动；顺带清理其
  `original_shenhua_zhugezhan` 调试打印残留）。
