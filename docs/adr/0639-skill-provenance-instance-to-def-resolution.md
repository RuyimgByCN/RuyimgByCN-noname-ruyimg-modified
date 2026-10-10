# ADR 0639：skill-provenance instance→def 解析修复 + _backup 助手架构限制

> 状态：已落地（2026-08-08）
> 领取：本会话
> 前置：[ADR 0638](0638-skill-provenance-trigger-filter-family.md)

## 背景

ADR 0638 落地 `EventViewAsSkill(skill_def)` 触发过滤，但 `filter_match` 直接比较
`view_as_skill == *skill_def`。审计 `turn_flag::VIEW_AS_SKILL` 的写入发现：`view_as_skill` 持有
的是技能**实例 id**（`skill.raw()`），而非定义 id。故 ADR 0638 的比较是实例 vs 定义——语义错误。

## 决策

### D1：instance→def 解析（语义修复）

`filter_match` 的 `EventViewAsSkill` 臂改为先解析实例再比较定义：

```rust
TriggerFilter::EventViewAsSkill(skill_def) => {
    view_as_skill != 0
        && state.skills.get(SkillId::new(view_as_skill))
            .is_some_and(|skill| skill.def.raw() == *skill_def)
}
```

`state.skills.get(instance)` 解析实例→定义，使比较正确。这也覆盖 `_backup` 助手（其实例可解析）。

## 验证

| 检查 | 结果 |
|---|---|
| `cargo check -p engine-core` | **通过** ✅ |
| `cargo test --workspace` | **0 FAILED** ✅ |

## _backup 助手架构限制（终审）

ADR 0638 的 `event_view_as_skill_<DEF>` 要求 DSL 内容包静态知道 view-as 技能的定义 id。但
审计 42 个 `event.skill` 占位符发现：大多数引用 `_backup` 助手（`X_backup`），这些是引擎
view-as 框架**运行时动态生成**的助手，**无静态 def id**（不在任何 npack 的 skill_index 中）。

DSL 内容包在编译时无法引用这些动态 def id。因此 `event_view_as_skill_<DEF>` 无法覆盖
`_backup`-引用占位符——这是**架构限制**，非原语可解决。

ADR 0638 的 2 个落地技能（`nao_shouqing3`、`inari_baiwei_draw`）引用的是**静态发布**的 view-as
技能（有 skill_index 条目与稳定 def id），故可表达。`_backup`-引用占位符需引擎层：
1. 暴露 view-as 助手名称→def 的注册表；或
2. 改用名称匹配（`event_view_as_skill_name_<NAME>`），但 DSL 需引入字符串参数化过滤。

## 后续

- 本修复使 `event_view_as_skill_<DEF>` 语义正确（实例→定义解析）。
- `_backup` 助手覆盖需引擎 view-as 子系统的架构改动（助手 def 注册表或名称匹配过滤）。
- storage-object 逐技能建模是另一条路径（`storage.X` 数组/对象语义）。两者均需后续会话的较深改动。
