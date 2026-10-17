# 2026-10-17 扩展/客户端视图隔离扩面（COV-VISIBLE-VIEWS-EXT）

**核验日期**：2026-10-17。**待做 ID**：`COV-VISIBLE-VIEWS-EXT`（自 `COV-PARTIAL-MECHANISMS` 拆出的 `visible_views` 扩面）。
**性质**：视图投影修正 + 测试扩面（无状态/版本/golden 变化；规则 v295 / 存档 v80）。

## 发现并修正的投影缺陷

`Engine::build_view` 原用 `standard::faction_of_general` + 无条件 `revealed_generals` 门控生成 `generals` 单元，导致：

1. **扩展/Boss 段武将不可见**：`faction_of_general` 只识别标准表下标；扩展将（def 接在标准表之后）与 Boss 将一律返回 `None` → 视图里 `generals` 为空（扩展武将「投影丢失」）。
2. **非国战模式武将误隐藏**：门控对**所有模式**生效，而 `revealed_generals` 默认 `[false,false]` → 原型/身份/对决/斗地主/Boss 下他人主将也看不到，与 `effective_faction` 的既定口径（「其余模式主将势力直接可见——标准玩法无暗置概念」）矛盾。

**修正**：`build_view` 改用 `catalog.general_meta(g)`（覆盖标准/扩展/Boss 三段）取名称与势力，并仅**在国战模式**保留暗置门控（`own || !kingdoms || revealed_generals[s]`）。

## 测试

新增 `isolation_generals_public_in_non_kingdoms_modes`（`crates/engine-native/tests/t13_info_isolation.rs`）：

| 断言 | 结果 |
|---|---|
| 原型模式装配标准将「刘备」于 1 号，0/1/2 三视角均见其主将名 | 通过 |
| 装 wudian 扩展包建局，def=标准表长（费诗）装配后三视角均见「费诗」（经 `catalog.general_meta`） | 通过 |
| 对手手牌仍 `HandView::Hidden`（武将公开不放松手牌隔离） | 通过 |

既有 `isolation_extension_cards_hidden_like_standard`（扩展卡手牌隐藏）与 `identity_role_visibility_and_death_reveal`（身份可见性）共同覆盖扩面。命令：`cargo test --offline -p engine-native --test t13_info_isolation`，**12 passed / 0 failed**（[targeted.log](targeted.log)）。engine-native 全量 **877/0/0**、Gate 1b exit 0。

## 边界

- 本批覆盖**引擎侧可见视图**（`PlayerView`/`AiSession`）；客户端/浏览器渲染层（T13 产品面）不属本项。
- 国战暗置门控语义不变（仅国战模式门控）；身份可见性沿用本人/主公/死亡公开。
- 视图为纯投影，不入快照/哈希——无版本次、golden 不变。
