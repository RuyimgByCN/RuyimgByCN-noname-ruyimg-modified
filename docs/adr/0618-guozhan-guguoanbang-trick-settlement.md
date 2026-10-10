# ADR 0618：国战锦囊「固国安邦」结算（CardKind::TrickGuguoanbang + 三阶段续体）

> 承接 [ADR 0617](0617-guozhan-kefuzhongyuan-trick-settlement.md)：国战 12 锦囊结算第五张
> —— gz_guguoanbang（固国安邦）。首个多阶段（摸→弃→给）Custom 续体。**无 VM ABI 变化**
> （CardKind/CardKindTag 末尾追加，全部复用既有 EffectRequest 与 Custom 魔数分发）。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

gz_guguoanbang（国战）：`filterTarget: target === player`（对你自己使用）。
`content`：①摸八张牌；②弃置至少六张手牌（`chooseToDiscard([6, count], true, "h")`）；
③若使用者势力为吴，则循环「选择 1–2 张弃置的牌 → 选择一名其他吴势力角色 → 交给其」
直至弃牌/目标耗尽或放弃（每名至多两张）。

## 做了什么

### CardKind::TrickGuguoanbang = 47（末尾追加，序数稳定）

- `state/cards.rs`：`TrickGuguoanbang = 47` + `from_tag` 47 臂 + `is_trick` 清单。
- `engine-api/view.rs`：`CardKindTag::TrickGuguoanbang` + `projection.rs` 映射臂。

### 锦囊分发（command/play.rs）

- `trick_effect::GUGUOANBANG = 23`。
- 分发臂：目标必须为使用者自己 → `use_independent_trick_targets(actor, card, &[actor], …)`。
- 目标策略 `skill_card_target_spec`：`Immediate([actor])`（自用）。
- `engine-ai/legal.rs`：PlayCard targets=[actor]。

### 三阶段结算（apply_deferred_trick GUGUOANBANG 臂 + GUGUOANBANG_CUSTOM_CHOICE）

阶段 1：`DrawCards 8` → 手牌弃 ≥6 选择（min 6, max = 手牌数，非取消）。
阶段 2（仅吴且弃牌非空且存在其他吴）：从弃置牌中选 1–2 张。
阶段 3：选一名其他吴角色 → `MoveCards` 交给其 → `finish_independent_trick_target` 推进队列。

- 魔数 `GUGUOANBANG_CUSTOM_CHOICE = 0x4747_4142`（pack_sp_2.rs），经 `crate::effect::`
  re-export + resolve_choice/mod.rs Custom 分发臂；解析器 `resolve_guguoanbang_choice`
  按 stage 分发，队列尾随 payload 透传，最终阶段显式推进。
- **近似**：上游循环给多目标（每名 ≤2，直到放弃），本实现为单轮（选 1–2 张给一名
  吴角色）——已写入注释。

### 牌堆（content/deck.rs）

`"gz_guguoanbang"`（红桃 A 一张）改投 `CardKind::TrickGuguoanbang`。国战金标
（guozhan_4_42、guozhan_2_7）再次漂移——已重钉本域向量。

### 单测

`crates/engine-core/tests/guozhan_trick_guguoanbang.rs`：①非吴：摸 8 → 弃 6 → 结算，
手牌 2；②吴：摸 8 → 弃 6 → 选 2 张弃牌 → 选其他吴目标 → 移交，受者手牌 +2。**2 passed**。

## 验证

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过**（国战向量已重钉）✅ |
| 新单测 | **guozhan_trick_guguoanbang 2 passed** ✅ |
| 失败集合 | **210 = 基线 211 − 1**：无新增失败；1 项出集（mbhanyong）为并行代理修复他人域 ✅ |
| `cargo check` | 无编译错误；改行 0 新增警告 ✅ |

**ABI 说明**：无 `RULE_VM_ABI_VERSION` bump——CardKind/CardKindTag 末尾追加，结算复用
DrawCards/DiscardCards/MoveCards 与 Custom 魔数分发，无新 DSL 原语、无新 bytecode。

## 后续

1. **其余 6 张国战锦囊**：huoshaolianying、lianjunshengyan、gz_haolingtianxia、
   gz_wenheluanwu、chiling、zhibi（纯信息揭示，可能不入引擎）、yuanjiao（可映射既有
   TrickYuanjiao）。
2. **gwyewu（叶舞）造卡入口**：需随机弃牌原语（ADR 0613 已登记）。
3. **gw_dudayuanshuai2 per-card 禁弃置**：需按 def 的 card 门禁（ADR 0608 已登记）。
