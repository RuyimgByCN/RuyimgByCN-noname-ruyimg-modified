# 阶段 3：Trigger 与 Query

> **历史快照（阶段 3 时点，2026-08-21 注）**：排序键/Query/Modifier 约束仍是
> 有效语义（只读、不耗 RuleRng、缓存不进哈希）；"Bootstrap 触发技"样例表是
> M0 时代样本，当前触发/过滤器能力以 `docs/content-sdk/phase4-dsl-vm.md`
> 与 `ABI-COORDINATION.md` 为准。本文 Trigger 全序第二键按实现写作
> `TimingBand`；架构计划 §16.2 的分层是同一全序在不同抽象层的表述。

对照上游 noname `event.trigger` / `arrangeTrigger` 的**排序意图**与技能索引方式，不复刻 Promise 链。

## Trigger

### 数据流

```text
DomainEvent
→ EventKind + TriggerContext
→ ActiveTriggerIndex.collect_sorted (全序)
→ filter (OwnerIsEventPlayer / Source / Always)
→ anti-loop (max_per_segment)
→ TriggerHandler → EffectRequest 入 FIFO 队列
```

### 全序键（升序）

```text
TimingBand (FirstDo=0 / Normal=100 / LastDo=200)
→ Priority (越小越先)
→ Optional (0 强制 / 1 可选；阶段 3 可选自动跳过)
→ SeatRel (相对当前回合角色的顺时针距离)
→ SkillInstanceId
→ TriggerDefId
```

对应 noname 的 firstDo / priority / 座位序 / lastDo 心智，字段固定、跨 Native/WASM 一致。

### 活动索引

- 仅在 `AddSkill` / `RemoveSkill` 时更新
- **不参与** Canonical State Hash（可由 skill store + catalog 重建）
- Snapshot 恢复后 `rebuild_derived_indexes()`

### Bootstrap 触发技

| SkillDef | 名称 | 时机 | 效果 |
|---|---|---|---|
| 1 | 奸雄 | Damage 且 owner=目标 | 摸 1 张 |

## Query / Modifier

### 要求

- 强类型 `QueryRequest` / `QueryResponse`
- 只读规范状态；不产生 Effect；不消耗 RuleRng
- 显式 `QueryContext`：深度上限 + 环检测
- `query_explain` 返回 base + `ModifierSource[]`

### 组合顺序

`Override → Add → Multiply → Min → Max → Veto`

### Bootstrap 修正

| SkillDef | 名称 | 通道 | 模式 | 值 |
|---|---|---|---|---|
| 2 | 马术 | DistanceFrom | Add | -1 |
| 3 | 奇策(示意) | AttackRange | Add | +1 |

### 缓存

- 可选、可清空
- **不序列化、不进 State Hash**
- 验收：开关缓存与清空后结果完全一致

## 与上游差异

- 无可选触发的 `chooseControl`（阶段 4+ / Choice 扩展）
- 无 `relatedTrigger` / `phaseAny` 动态别名（内容编译器阶段可加）
- Handler 为 Rust 枚举，非任意 JS
