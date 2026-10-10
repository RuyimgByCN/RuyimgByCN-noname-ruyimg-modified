# ADR 0621：国战锦囊「远交近攻」映射既有 TrickYuanjiao

> 承接 [ADR 0620](0620-guozhan-lianjunshengyan-trick-settlement.md)：国战 12 锦囊结算第八张
> —— yuanjiao（远交近攻）。与标准 TrickYuanjiao（CardKind 37，远交近攻）机制一致，
> **直接映射**而非新增结算——零引擎改动（仅牌堆 kind 映射 + 金标重钉）。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

guozhan 版 yuanjiao：`filterTarget` —— 对一名势力不同且已确定势力的其他角色使用。
`content`：目标摸一张牌，然后使用者摸三张牌。与标准版差异仅在目标过滤（国战要求
「已确定势力」）。

## 做了什么

### 牌堆映射（content/deck.rs）

国战牌堆 `"yuanjiao"`（红桃 9）由 `CardKind::Other` 改为 `CardKind::TrickYuanjiao`——
复用既有：
- 目标策略 `skill_card_target_spec`（TrickYuanjiao）：select_one 且 `actor_group !=
  target_group`（势力不同过滤，与国战「势力不同」一致）。
- 分发臂（TrickYuanjiao）：单目标 + `use_card_then_wuxie(YUANJIAO)`。
- 结算（`apply_deferred_trick` YUANJIAO 臂）：目标 `DrawCards 1` + 使用者 `DrawCards 3`。

国战金标（guozhan_4_42、guozhan_2_7）漂移——已重钉本域向量。

**近似**：国战版要求目标「已确定势力」（未明置不可选）；引擎 `player_group` 对未明置
目标可能返回未解析值，此时按「与使用者势力不同」放行——已写入注释（轻微放宽）。

### 单测

`crates/engine-core/tests/guozhan_trick_yuanjiao.rs`：2 人局 wei/shu，注入 TrickYuanjiao
对敌方使用 → 目标摸 1、使用者摸 3。**1 passed**。

## 验证

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过**（国战向量已重钉）✅ |
| 新单测 | **guozhan_trick_yuanjiao 1 passed** ✅ |
| 失败集合 | **204 = 基线 206 − 2**：无新增失败；2 项出集（rejiuchi_dongzhuo/rw_tengjia3）为并行代理修复他人域 ✅ |
| `cargo check` | 无编译错误；改行 0 新增警告 ✅ |

**ABI 说明**：无 `RULE_VM_ABI_VERSION` bump——纯牌堆 kind 映射，零新引擎面。

## 后续

1. **其余 4 张国战锦囊**：gz_haolingtianxia（全员依次二选一，最复杂）、gz_wenheluanwu
   （展示手牌 + 弃牌选择）、chiling（全局临时技能）、zhibi（纯信息揭示——观看手牌/
   暗置武将，引擎为确定性规则引擎，观看类效果不入引擎，建议维持元数据登记）。
2. **gwyewu（叶舞）造卡入口**：需随机弃牌原语（ADR 0613 已登记）。
3. **gw_dudayuanshuai2 per-card 禁弃置**：需按 def 的 card 门禁（ADR 0608 已登记）。
