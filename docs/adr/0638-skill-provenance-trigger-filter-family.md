# ADR 0638：skill-provenance 触发过滤原语族（Phase 3 突破口）

> 状态：已落地（2026-08-08）
> 领取：本会话
> 前置：[ADR 0637](0637-event-card-count-and-per-skill-approximation.md)

## 背景

ADR 0637 终审确认 **skill-provenance（`event.skill == "X"`，事件牌源自某 view-as 技能）**是剩余
占位符最常见的阻塞子条件。引擎在触发分发时已捕获 `view_as_skill`（事件牌的 originating view-as
技能实例），但未暴露为触发过滤。

## 决策

新增参数化触发过滤 `EventViewAsSkill(u32)`（skill-provenance 族）：

```rust
TriggerFilter::EventViewAsSkill(skill_def)  // 事件牌的 originating view-as 技能定义 == skill_def
```

- `crates/engine-core/src/trigger/catalog.rs`：枚举变体。
- `crates/engine-core/src/content/registry.rs` `map_filter`：`strip_prefix("event_view_as_skill_")`
  + `parse::<u32>()`（>0）→ `TriggerFilter::EventViewAsSkill`。
- `crates/engine-core/src/trigger/dispatch.rs` `filter_match`：`view_as_skill == *skill_def`
  （`filter_match` 已接收 `view_as_skill: u32` 参数，来自 `turn_flag::VIEW_AS_SKILL`）。
- `crates/content-compiler/src/typecheck.rs` `valid_event_view_as_skill_filter`：DSL 名
  `event_view_as_skill_<DEF>`，DEF 为正 u32。

## 首批落地

| 技能 | 包 | 上游 filter | 实现 |
|---|---|---|---|
| `nao_shouqing3`（守琴·标记） | key | `event.skill == "nao_shouqing2"` | `trigger card_used event_view_as_skill_4259091944 { effect draw_cards(self, 1); }`；近似：maxHandcard modifier + 标记累计另册登记 |
| `inari_baiwei_draw`（摆尾·补牌） | key | `event.skill.indexOf("inari_baiwei")==0` | `trigger card_used/card_responded event_view_as_skill_3124826192 { effect draw_cards(self, 1); }`；近似：精确匹配主 inari_baiwei，变体另声明 |

落地后 key 包 logic_hash 重算（无 key golden 断言包级 hash）；catalog migrated_skills 2909→**2911**。

## 验证

| 检查 | 结果 |
|---|---|
| `trigger card_used event_view_as_skill_12345 { ... }` DSL 端到端编译 | **通过** ✅ |
| `cargo check --workspace` | **通过** ✅ |
| `node scripts/sync-original-content.mjs`（全量重建+校验） | **通过** ✅ |
| `cargo test --workspace` | **0 FAILED** ✅ |

## 后续

- 本族是剩余占位符最常见的阻塞子条件（skill-provenance）的突破口。继续逐技能拆解：
  凡 filter 含 `event.skill == "X"` 的占位符，用 `event_view_as_skill_<DEF>` 表达该子条件，
  其余子条件用已支持原语 `&&` 组合或登记近似。
- 下一阻塞子条件族：storage-object 读写（`player.storage.X`）。继续按本 ADR 模式推进。
