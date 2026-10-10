# ADR 0630：card-name-scoped 触发过滤原语族（Phase 3 第一族）

> 状态：已落地（2026-08-07）
> 领取：本会话
> 前置：[ADR 0629](0629-catalog-placeholder-policy-and-crosspack-ref-restore.md)（catalog 占位符策略 + Phase 3 开工）

## 背景

ADR 0629 诊断出 Phase 3 的 ~3000 个占位符按引擎原语族分布，其中
**card_name_scoped（463 个）**是第三大宗：占位符的触发条件依赖「事件牌名为某张具体牌」
（如「使用【酒】时」「成为【杀】的目标时」），而引擎此前只有按 `trigger def` 硬编码的
具名过滤，无「卡牌名参数化」的通用过滤族。

例如 `jsrg:jsrgshishou`（恃守）：上游 `filter(event): event.card.name == "jiu"`、
`content(): player.draw(3)`。旧引擎无「使用【酒】时」的触发过滤名，故占位为空壳。

## 决策

复用既有的**参数化触发过滤机制**（`map_filter` 里 `strip_prefix` + `parse` 后缀的模式，
如 `owner_source_target_mark_positive_<MARK_DEF>`），新增一族**卡牌名限定**触发过滤，
后缀为 `CardSelector` 名（既有 `map_card_selector` 已映射 sha/nanman/juedou/jiu/trick/…）。

### D1：三个新 `TriggerFilter` 枚举变体（`trigger/catalog.rs`）

```rust
OwnerSourceCardName(CardSelector),       // owner 使用/打出匹配选择器的牌（card_used/card_responded）
OwnerIsTargetCardName(CardSelector),     // owner 成为匹配选择器牌的目标（card_targeted）
OwnerIsTargetCardDamage(CardSelector),   // owner 受到匹配选择器牌造成的伤害（damage）
```

`CardSelector` 已 `Copy + Eq + Hash + Serialize + Deserialize`，可直接作为枚举载荷。

### D2：`map_filter` 接线（`content/registry.rs`）

新增三个 `strip_prefix` 分支，把 DSL 名映射到枚举：

```rust
if let Some(selector) = s.strip_prefix("owner_source_card_") {
    if let Some(card) = map_card_selector(selector) { return Some(TriggerFilter::OwnerSourceCardName(card)); }
}
if let Some(selector) = s.strip_prefix("owner_is_target_card_damage_") {
    if let Some(card) = map_card_selector(selector) { return Some(TriggerFilter::OwnerIsTargetCardDamage(card)); }
}
if let Some(selector) = s.strip_prefix("owner_is_target_card_") {
    if let Some(card) = map_card_selector(selector) { return Some(TriggerFilter::OwnerIsTargetCardName(card)); }
}
```

**关键**：选择器未知时**不 `return None`**，而是 fall-through——避免遮蔽共享前缀的既有过滤
（如 `owner_is_target_card_in_processing`、`owner_is_target_card_damage_before`，它们的后缀
`in_processing`/`before` 不是 `CardSelector` 名，须由 `map_filter` 后续分支解析）。

### D3：`filter_match` 求值（`trigger/dispatch.rs`）

新增 `event_card_matches_selector(state, owner, ctx, selector)` 辅助：从 `ctx.card` 取事件牌，
用 `effective_card_suit` + `matches_physical_card`（改为 `pub(crate)`）做逻辑花色/类别匹配；
`ctx.card` 缺失时回退到 `ctx.card_kind` + `matches_kind_and_suit`。三个新过滤变体的求值：
- `OwnerSourceCardName`：`owner_is_source && card_used/card_responded && 事件牌匹配`
- `OwnerIsTargetCardName`：`owner_is_player && card_targeted && 事件牌匹配`
- `OwnerIsTargetCardDamage`：`owner_is_player && damage && 事件牌匹配`

### D4：DSL 词法接线（`content-compiler/src/typecheck.rs`）

新增 `valid_card_name_scoped_filter(filter)`：接受 `owner_source_card_<sel>` /
`owner_is_target_card_<sel>` / `owner_is_target_card_damage_<sel>`，`<sel>` 必须是
镜像 `map_card_selector` 的已知选择器名白名单（sha/nanman/juedou/jiu/trick/…）。

## 首批落地

| 技能 | 包 | 过滤 | effect | 近似说明 |
|---|---|---|---|---|
| `jsrgshishou`（恃守） | jsrg | `card_used owner_source_card_jiu` | `draw_cards(self, 3)` | 「本回合不能使用牌」(`jsrgshishou_nouse` modifier) 与 `jsrgshishou_burn` 子技能需 modifier 扩展，另册登记；本定义实现摸 3 张核心行为 |

落地后：jsrg 包 trigger_index 10→11、logic_hash 重算；2 个 jsrg golden 测试
（`original_jsrg_zhaohan.rs`/`original_jsrg_zuwang.rs`）的 `trigger_index.len()` 重钉 10→11；
catalog migrated_skills 2904→**2905**（jsrgshishou 从占位升回 ported）。

## 验证

| 检查 | 结果 |
|---|---|
| `node scripts/sync-original-content.mjs`（全量重建+校验） | **通过** ✅ |
| `cargo check -p engine-core -p content-compiler` | **通过** ✅ |
| `cargo test --workspace` | **全绿** ✅ |
| 三过滤变体端到端编译（test skill: jiu/sha/nanman） | **3 triggers 解析正确** ✅ |
| 前缀遮蔽回归（`owner_is_target_card_in_processing` 仍解析） | **通过** ✅（fall-through 修复） |

## 后续

- 本族解锁 463 个 card_name_scoped 占位符的「过滤」侧；多数占位符的 **content** 仍含
  复杂副作用（addTempSkill/trigger.cancel/excluded.add/多步 choice），需逐技能写或扩
  effect 原语。本 ADR 的过滤族把「触发时机」从阻塞项移除，剩余是 content 侧工作。
- 下一族候选：`history_count`（485，回合内摸牌/伤害历史计数查询，需 query 原语）、
  `hasSkill_check`（507，技能存在性查询）、`view_as`（246，转化技 effect 原语）。
- 每族按本 ADR 模式推进：枚举变体 → map_filter/typecheck 接线 → filter_match 求值 →
  批量补 DSL → rebuild-pack.sh 重建 → cargo test --workspace + sync。
