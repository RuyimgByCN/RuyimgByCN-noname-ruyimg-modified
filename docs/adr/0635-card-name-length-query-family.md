# ADR 0635：card_name_length 查询原语族（Phase 3 子条件原语）

> 状态：已落地（2026-08-08）
> 领取：本会话
> 前置：[ADR 0634](0634-event-target-count-opcode-fix.md)

## 背景

`get.cardNameLength(event.card)` 是复合过滤器子条件（如 olcuipo「牌名长度 == 本回合第 N 张」）。
引擎既有 `crate::query::card_name_length(state, player, card)` 内部函数（精确映射上游
`get.cardNameLength`），但未暴露为 VM 可访问的查询。

## 决策

新增 `CardNameLength` 查询（跨 rule-api/rule-vm/engine-core/content-compiler 全链路）：

```rust
QueryRequest::CardNameLength { player, card }  // → Int: 牌名长度（0 若无）
```

- `query_tag::CARD_NAME_LENGTH = 35`；DSL 名 `card_name_length(player, card)`（arity 2，card 在位置 1）。
- 求值：复用既有 `crate::query::card_name_length(state, player, card)`，None → 0。
- cache key tag 35 + player + card。

## 首批落地

| 技能 | 包 | 实现 |
|---|---|---|
| `olcuipo`（摧破） | sp | `trigger card_used owner_is_source { if card_name_length(self, event_card) == card_action_count(self) { effect draw_cards(self, 1); } }` |

上游 filter：`cardNameLength(event.card) == useCardHistory.indexOf(event) + 1`（== 当前本回合用牌计数，
`indexOf+1 == card_action_count`）；content：sha/damage-trick → baseDamage++，else → draw。
近似：「baseDamage++」伤害修正需新 modifier 原语，本定义实现 filter + 非伤害牌摸牌分支；
杀/伤害锦囊增伤分支另册登记。filter 用 `==` 精确表达。

落地后 sp 包 modules 108→109、logic_hash 重算；3 个 sp golden（dingpan_buzhi/hengzheng_dongzhuo/
ziyuan_mizhu）的 `PACKAGE_LOGIC_HASH` + `modules.len()` 重钉；catalog migrated_skills 2906→**2907**。

## 验证

| 检查 | 结果 |
|---|---|
| `card_name_length(self, event_card) == card_action_count(self) + 1` DSL 端到端编译 | **通过** ✅ |
| `cargo check --workspace` | **通过** ✅ |
| `node scripts/sync-original-content.mjs`（全量重建+校验） | **通过** ✅ |
| `cargo test --workspace` | **884 二进制全绿 / 0 FAILED** ✅ |

## 后续

- 本族解锁 card-name-length 子条件。与 ADR 0633 终审一致：复合过滤器可表达性瓶颈是子条件
  多样性；每补一族子条件原语，可组合实现的占位符子集扩大。继续按 phase-parent / storage 比较
  等子条件原语推进，逐技能拆解复合过滤器。
