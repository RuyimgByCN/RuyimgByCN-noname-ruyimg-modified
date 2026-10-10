# ADR 0640：skill-provenance 名称匹配过滤（_backup 助手架构限制缓解）

> 状态：已落地（2026-08-08）
> 领取：本会话
> 前置：[ADR 0639](0639-skill-provenance-instance-to-def-resolution.md)

## 背景

ADR 0639 诊断 `_backup` view-as 助手无静态 def id（架构限制）：`event_view_as_skill_<DEF>`
要求 DSL 内容包静态知道 def id，但 `_backup` 助手是引擎运行时动态生成的。本 ADR 落地名称匹配
变体以缓解此限制。

## 决策

新增 `EventViewAsSkillName(String)` 触发过滤（名称匹配变体）：

```rust
TriggerFilter::EventViewAsSkillName(name)  // 事件牌的 view-as 技能 content-metadata 名 == name
```

- `crates/engine-core/src/trigger/catalog.rs`：枚举变体（持 String）。
- `crates/engine-core/src/content/registry.rs` `map_filter`：`strip_prefix("event_view_as_skill_name_")`
  + 小写字母/数字/下划线校验 → `TriggerFilter::EventViewAsSkillName(name)`。**在 DEF 分支之前**
  检查，避免 `event_view_as_skill_name_X` 被 DEF 分支误剥。
- `crates/engine-core/src/trigger/dispatch.rs` `filter_match`：`view_as_skill`（实例 id）→
  `state.skills.get(instance).def` → `state.content.skill_metadata(def).name` 与 `name` 比较。
- `crates/content-compiler/src/typecheck.rs` `valid_event_view_as_skill_filter`：接受
  `event_view_as_skill_name_<NAME>`（NAME 为小写标识符）+ 原 `event_view_as_skill_<DEF>`。

DSL `<NAME>` 是引擎内容注册表中的技能名（如 `jibing_backup`、`dcsaowei_backup`），即使该技能
无静态 def id，运行时其实例可解析到 metadata.name。

## 验证

| 检查 | 结果 |
|---|---|
| `trigger card_used event_view_as_skill_name_jibing_backup { ... }` DSL 端到端编译 | **通过** ✅ |
| `cargo check --workspace` | **通过** ✅ |
| `cargo test --workspace` | **0 FAILED** ✅ |

## 落地情况与残余限制

本变体使 DSL 能按**名称**匹配 view-as 技能来源，覆盖 `_backup` 助手的**过滤侧**。但审计
`_backup`-引用占位符发现多数的**内容侧**同样引用动态助手（`addTempSkill("X_effect")`，`X_effect`
也无静态 def id），形成**链式助手解析阻塞**——过滤侧可表达，内容侧仍不可表达。

因此 `_backup`-引用占位符的完整实现仍需引擎层对动态助手（`_backup`/`_effect`/`_mark`）的统一
建模（助手名称→def 注册表，使 `add_skill(self, <DEF>)` 能引用动态助手）。

## 后续

- 本变体缓解了 `_backup` 助手的**过滤侧**限制。内容侧（`addTempSkill("X_effect")` 引用动态助手）
  仍需引擎助手注册表。
- 两条继续路径：(1) 引擎 view-as 助手注册表（统一过滤+内容侧）；(2) storage-object 逐技能建模。
