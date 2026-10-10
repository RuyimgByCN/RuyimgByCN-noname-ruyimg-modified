# ADR 0636：in_play_phase 查询原语族（Phase 3 phase-parent 子条件）

> 状态：已落地（2026-08-08）
> 领取：本会话
> 前置：[ADR 0635](0635-card-name-length-query-family.md)

## 背景

`event.getParent("phaseUse")`（判断事件是否发生在出牌阶段内）是 131 个占位符的复合过滤器子条件。
引擎 `state.turn.phase: Phase` 已含 `Phase::Play`，但未暴露为 VM 查询。

## 决策

新增 `InPlayPhase` 查询（跨 rule-api/rule-vm/engine-core/content-compiler 全链路）：

```rust
QueryRequest::InPlayPhase  // → Bool: 当前回合是否处于 Play 阶段（phase-parent 近似）
```

- `query_tag::IN_PLAY_PHASE = 36`；DSL 名 `in_play_phase()`（arity 0）。
- 求值：`matches!(state.turn.phase, Phase::Play)`。
- cache key tag 36。
- 近似说明：上游 `event.getParent("phaseUse")` 检查**事件**的父阶段，而本查询检查**当前**
  回合阶段；对在出牌阶段内触发的事件（绝大多数用牌类技能）两者等价，对跨阶段事件为近似。

## 验证

| 检查 | 结果 |
|---|---|
| `in_play_phase() && current_phase_player() == self` DSL 端到端编译 | **通过** ✅ |
| `cargo check --workspace` | **通过** ✅ |
| `cargo test --workspace` | **884 二进制全绿 / 0 FAILED** ✅ |

## 落地情况与结构性观察

本族实现后，审计 phase-parent 占位符（`currentPhase==player && getParent("phaseUse")` 类）发现：
**全部 131 个仍同时含 storage-object 子条件**（`player.storage.X` / `getStorage`），而 storage-object
是逐技能独特语义（每个 storage 有不同的写入/读取/类型逻辑），无通用原语可覆盖。

至此，复合过滤器的**所有常见通用子条件**均已可表达：
`card_name_length` / `current_phase_player` / `has_skill` / `card_attribute(number)` /
`hp`+`max_hp`(isDamaged) / `mark_count` / `cards_drawn_this_turn` / `damage_dealt_this_turn` /
`card_action_count` / `event_target_count` / `in_play_phase` + 组合子 `&&`/`>`/`==`/`<`。

剩余不可表达子条件是**逐技能独特**的：
- `player.storage.X`（自定义存储对象，每技能语义不同）—— 最常见
- 跨回合历史（`actionHistory[i].useCard[index]`）
- choice 上下文（`event.name == "chooseToUse"`）
- card-list 迭代（`event.getl(player).cards2.filter(...)`）

这些无法用通用原语族覆盖，需**逐技能**分析其 storage/历史语义并建模（每技能一个独立 effect/filter
原语或近似）。

## 后续

- 通用子条件原语集已完备。剩余 ~2974 个占位符是逐技能工作：每条占位符的复合过滤器含独特
  storage/历史/choice 子条件，需逐条建模或近似。
- 这是逐技能分析 + 建模的长期工作，非原语族批量推进可覆盖。本 ADR + 0630-0635 建立的可复现
  管线（逐技能 DSL 补写 → rebuild-pack.sh → cargo test --workspace + sync）为后续逐技能推进提供基础。
