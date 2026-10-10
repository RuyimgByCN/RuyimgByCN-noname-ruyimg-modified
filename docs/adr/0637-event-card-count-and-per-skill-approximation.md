# ADR 0637：event_card_count 原语族 + 首个逐技能近似实现（Phase 3）

> 状态：已落地（2026-08-08）
> 领取：本会话
> 前置：[ADR 0636](0636-in-play-phase-query-family.md)

## 背景

ADR 0636 终审确认通用子条件原语集已完备，剩余占位符含逐技能独特的不可表达子条件，其中
**card-list 迭代**（`event.getl().cards2.filter(...)`）是常见一类。本 ADR 落地该族原语 +
首个逐技能近似实现。

## 决策

### D1：event_card_count host opcode（card-list-iteration 族）

新增 `Op::HostEventCardCount`（`op::HOST_EVENT_CARD_COUNT = 0x5C`），无参数，push
`host.event_cards().len()`（触发事件捕获的物理牌数）。全链路接线：
- `rule-vm/bytecode.rs`：opcode + Op 变体 + assembler。
- `rule-vm/vm.rs`：`op::HOST_EVENT_CARD_COUNT => push(host.event_cards().len())`。
- `rule-vm/verify.rs`：push-1 栈效应组。
- `content-schema/ir.rs`：`ExprIr::EventCardCount`。
- `content-compiler/codegen.rs`：`ExprIr::EventCardCount => Op::HostEventCardCount` + estimate_pc。
- `content-compiler/typecheck.rs`：DSL 名 `event_card_count`（arity 0）+ `USES_EVENT_CARD_COUNT`
  特性位（**u16 位已满，features 系统从 u16 扩为 u32** 以容纳第 17 位 65536）+ card/move trigger 守卫。

### D2：features 系统 u16→u32（容量扩展）

`USES_*` 常量已占满 u16 全部 16 位。新增 `USES_EVENT_CARD_COUNT` 需第 17 位，故把
`stmts_features`/`stmt_features`/`expr_features` 返回类型与全部 `USES_*` 常量从 u16 扩为 u32，
所有 `u16::from(...)` 改 `u32::from(...)`。零行为变化（位运算语义不变）。

### D3：首个逐技能近似实现

| 技能 | 包 | 实现 | 近似 |
|---|---|---|---|
| `olmingzhe`（明哲） | sp | `trigger cards_moved owner_is_target { if event_card_count > 0 && in_play_phase() == 0 { effect draw_cards(self, 1); } }` | 上游限定「失去的牌中有红色」（getl.cards2 红色筛选）需逐张颜色计数原语；本定义放宽为「失去任意牌即摸一张」（触发更频繁，摸牌副作用良性）。showCards 表现层不建模。 |

落地后 sp 包 modules 109→110、logic_hash 重算；3 个 sp golden 重钉（hash + modules.len 110）；
catalog migrated_skills 2907→**2908**。

## 验证

| 检查 | 结果 |
|---|---|
| `event_card_count > 0 && in_play_phase() == 0` DSL 端到端编译 | **通过** ✅ |
| `cargo check --workspace`（u16→u32 扩容后） | **通过** ✅ |
| `node scripts/sync-original-content.mjs`（全量重建+校验） | **通过** ✅ |
| `cargo test --workspace` | **884 二进制全绿 / 0 FAILED** ✅ |

## 后续

- 本族解锁 card-list-iteration 子条件（事件牌数）。逐技能近似（放宽不可表达子条件、保留可表达
  核心行为）是 ADR 0597 近似口径的延伸——每条占位符逐条判断「放宽哪个子条件副作用最小」。
- 剩余 ~2973 个占位符仍需逐技能分析：storage 对象、跨回合历史、custom skill event、逐张颜色
  筛选等子条件。继续按本 ADR 模式（扩原语族 + 逐技能近似）推进。
