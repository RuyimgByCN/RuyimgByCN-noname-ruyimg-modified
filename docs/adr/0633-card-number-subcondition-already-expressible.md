# ADR 0633：event.card.number 子条件已可表达 + 复合过滤器可表达性终审

> 状态：已落地（2026-08-07）
> 领取：本会话
> 前置：[ADR 0632](0632-hasskill-query-family.md)

## 背景

ADR 0632 指出 hasSkill_check 占位符的复合过滤器含 `currentPhase==player` 等子条件。本轮
审计 `event.card.number > N` 子条件（出现在 nstuiyan 等 183 个 isDamaged 类、多个 card.number
类占位符），判断是否需新增原语。

## 决策：不新增原语——event.card.number 已可表达

审计发现「事件牌点数」子条件**无需新原语**，已有原语组合即可：

- `event_card`（DSL 表达式 `Op::HostEventCard`）— 事件绑定的牌实例。
- `card_attribute(card, attr)`（`query_tag::CARD_ATTRIBUTE`，attr 1 = rank/number）— 牌点数。
- ADR 0631 的 DSL 表达式扩展（`>`、`&&`）— 比较 + 复合条件。

组合 `card_attribute(event_card, 1) > N` 端到端编译通过（test skill 验证）。同理
`current_phase_player() == self`（既有 `query_tag::CURRENT_PHASE_PLAYER`）、`hp(self) < max_hp(self)`
（既有 hp/max_hp 查询）等子条件均已可表达。

## 复合过滤器可表达性终审

逐族审计（hasSkill/history_count/card_name/currentPhase/card.number/isDamaged）后，引擎查询集
已相当丰富。**结构性瓶颈不在单族原语，而在复合过滤器的多样性**：每条占位符的过滤器是一组
独特子条件的 `&&` 组合，即使每个常见子条件都有原语，组合中常混入一两个**仍不可表达**的子
条件（典型）：

| 不可表达子条件 | 出现场景 | 缺的原语 |
|---|---|---|
| `event.getParent("phaseUse")` | 判断事件是否发生在出牌阶段 | phase-parent 检测 |
| `player.storage.X == N` / `countMark("X")` 比较 | 自定义标记/存储值 | storage/mark 值比较查询（mark_count 既有但限 owner 同 skill） |
| `event.targets.length > 1` | 多目标判定 | target-count 查询 |
| `chooseToUse`/`chooseToDiscard` 上下文 | choice 流水线 | choice-context 检测 |
| `get.cardNameLength(event.card)` | 牌名长度 | card-name-length 查询 |

纯可组合实现（复合过滤器全部子条件都有原语）的占位符子集确实很小（本会话实现 4 个：
reluanzhan/resilv/jsrgshishou/dckuichi）。

## 后续路径

1. **继续扩引擎原语族**覆盖上表的不可表达子条件（phase-parent / storage 比较 / target-count /
   card-name-length 等），每族解锁一批。
2. **逐占位符拆解**复合过滤器为已支持原语的 `&&` 组合；无法整条拆解的，登记近似（实现可表达
   的子集行为，不可表达部分注释，符合 ADR 0597 近似口径）。
3. 这是按原语族 + 逐技能分析推进的多会话长期工作。本 ADR + 0630/0631/0632 建立的可复现管线
   （原语族→DSL→rebuild-pack.sh→cargo test --workspace + sync）为后续批次提供基础。

## 验证

| 检查 | 结果 |
|---|---|
| `card_attribute(event_card, 1) > 5` DSL 端到端编译 | **通过** ✅ |
| `cargo check --workspace` | **通过** ✅ |
| `cargo test --workspace`（既有，无新代码改动本轮） | **全绿** ✅ |
