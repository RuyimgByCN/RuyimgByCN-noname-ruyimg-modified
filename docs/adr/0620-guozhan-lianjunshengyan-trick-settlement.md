# ADR 0620：国战锦囊「联军盛宴」结算（CardKind::TrickLianjunshengyan + 摸回分配）

> 承接 [ADR 0619](0619-guozhan-huoshaolianying-trick-settlement.md)：国战 12 锦囊结算第七张
> —— lianjunshengyan（联军盛宴）。**无 VM ABI 变化**（CardKind/CardKindTag 末尾追加，
> 结算复用 DrawCards/RecoverHp/RemoveMark 与 Custom 魔数分发）。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

lianjunshengyan（国战变体）：`enable` 要求明置；`filterTarget` —— 选一名敌方势力角色
（非己、非未知身份、非友）为代表；`changeTarget` —— 目标 = 代表 + 使用者自己 + 该势力
全体（isFriendOf 代表，剔除 diaohulishan）。`content` 逐目标：
- 非己目标：摸一张，然后重置（`link(false)` = 解除横置）。
- 使用者自己：X = 敌方势力人数（目标数 − 1）；缺体为 0 → 摸 X；否则 `chooseControl`
  分配「摸 Y 回 X−Y」（Y∈[0,X]，回复量以缺体为上限 `min(X, damaged)`）。

## 做了什么

### CardKind::TrickLianjunshengyan = 49（末尾追加，序数稳定）

- `state/cards.rs`：`TrickLianjunshengyan = 49` + `from_tag` 49 臂 + `is_trick` 清单。
- `engine-api/view.rs`：`CardKindTag::TrickLianjunshengyan` + `projection.rs` 映射臂。

### 锦囊分发（command/play.rs）

- `trick_effect::LIANJUNSHENGYAN = 25`。
- 分发臂：选一名敌方势力代表（非己、非 `same_side`）→ 目标 = `[自己, 代表] + 代表的
  同势力成员` → `use_independent_trick_targets`。
- 目标策略 `skill_card_target_spec`：select_one（非己且非 `same_side` 的存活角色）。
- `engine-ai/legal.rs`：敌方势力角色可选为代表。

### 逐目标结算（apply_deferred_trick LIANJUNSHENGYAN 臂）

- 非己：`DrawCards 1` +（若横置）`RemoveMark(MARK_CHAIN)` 重置 → 队列推进。
- 自己：X = 队列目标数 − 1；缺体 0 → `DrawCards X`；否则开摸回分配 choice
  （`LIANJUNSHENGYAN_CUSTOM_CHOICE`，选项 0..=min(X, damaged)，token = 回复量）。
- 解析器 `resolve_lianjunshengyan_choice`：回复 = token，摸 = X − token →
  `DrawCards` + `RecoverHp` → 队列推进。

### 牌堆（content/deck.rs）

`"lianjunshengyan"`（红桃 A 一张）改投 `CardKind::TrickLianjunshengyan`。国战金标
（guozhan_4_42、guozhan_2_7）再次漂移——已重钉本域向量。

### 单测

`crates/engine-core/tests/guozhan_trick_lianjunshengyan.rs`：①自己满血 + 敌方已横置：
自己摸 X=1、敌方摸 1 并重置；②自己受伤：开摸回分配 choice，选「回 1 摸 0」→ 恢复 1、
敌方摸 1。**2 passed**。

## 验证

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过**（国战向量已重钉）✅ |
| 新单测 | **guozhan_trick_lianjunshengyan 2 passed** ✅ |
| 失败集合 | **206 = 基线 208 − 2**：无新增失败；2 项出集（mobilejingce/refuli）为并行代理修复他人域 ✅ |
| `cargo check` | 无编译错误；改行 0 新增警告 ✅ |

**ABI 说明**：无 `RULE_VM_ABI_VERSION` bump——CardKind/CardKindTag 末尾追加，结算复用
DrawCards/RecoverHp/RemoveMark（MARK_CHAIN 重置）与 Custom 魔数分发，无新 DSL 原语。

## 后续

1. **其余 5 张国战锦囊**：gz_haolingtianxia、gz_wenheluanwu、chiling、zhibi（纯信息
   揭示，可能不入引擎）、yuanjiao（可映射既有 TrickYuanjiao）。
2. **gwyewu（叶舞）造卡入口**：需随机弃牌原语（ADR 0613 已登记）。
3. **gw_dudayuanshuai2 per-card 禁弃置**：需按 def 的 card 门禁（ADR 0608 已登记）。
