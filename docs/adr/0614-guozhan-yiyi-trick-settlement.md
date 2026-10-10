# ADR 0614：国战锦囊「以逸待劳」结算（CardKind::TrickYiyi + 逐目标摸2弃2）

> 承接 ADR 0604/0607/0608/0612/0613 反复标记的「国战 12 锦囊结算」待办：落地第一个
> 真实结算 —— yiyi（以逸待劳），建立「CardKind + 锦囊分发 + 逐目标队列 + 弃牌 choice」
> 模式，供其余 11 张锦囊复用。**无 VM ABI 变化**（修正此前「ABI 645+」的估算——结算
> 完全落在既有引擎面内：CardKind 追加 + 复用 DiscardHandEquipCount 与
> IndependentTrickQueue resume）。

## 背景

国战 12 锦囊（gz_haolingtianxia、gz_kefuzhongyuan、gz_guguoanbang、gz_wenheluanwu、zhibi、
chiling、huoshaolianying、lulitongxin、lianjunshengyan、shuiyanqijunx、yiyi、yuanjiao）此前
全部以 `CardKind::Other` + crc32 def 入国战牌堆，使用即空结算（装饰性）。本 ADR 以
**yiyi（以逸待劳）** 为模式样板落地第一个真实结算。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

yiyi（guozhan 变体）：`selectTarget: -1` + `filterTarget: target.isFriendOf(player)` ——
对己方全体角色自动使用；逐目标 `target.draw(2)` 然后 `target.chooseToDiscard(2, "he", true)`
（摸两张、弃置两张手牌或装备牌）。己方 = 同势力或野心联盟。

## 做了什么

### CardKind::TrickYiyi = 43（追加在枚举末尾，序数稳定）

- `state/cards.rs`：`TrickYiyi = 43` + `from_tag` 43 臂 + `is_trick` 清单。
- `engine-api/view.rs`：`CardKindTag::TrickYiyi`（追加，序数稳定）。
- `projection.rs`：CardKind → CardKindTag 映射臂。

### 锦囊分发（command/play.rs）

- `trick_effect::YIYI = 19`。
- 分发臂：`CardKind::TrickYiyi` → 计算己方全体（`player_group` 同势力 或 `mode::are_allies`
  野心联盟，含自己）→ `use_independent_trick_targets(actor, card, allies, YIYI, field_source)`。
- 目标策略 `skill_card_target_spec`：TrickYiyi → `Immediate(allies)`（自动目标，无选择）。
- `engine-ai/legal.rs`：TrickYiyi → `PlayCard { targets: vec![] }`（AI 可直接使用）。

### 逐目标结算（apply_deferred_trick YIYI 臂）

```
目标 = queued_independent_trick(args) 当前目标；非存活 → finish（推进）。
run_effects_then_finish_independent_trick_target(
    tx,
    [DrawCards { player: 目标, count: 2 },
     DiscardHandEquipCount { player: 目标, count: 2, skill: NONE }],
    args)
```

- 摸 2 即时完成；弃 2 由既有 `DiscardHandEquipCount` 开 Custom choice。
- 弃牌 choice 挂起时，`run_effects_then_finish_independent_trick_target` 把队列存为
  `IndependentTrickQueue` resume；choice 完成后经 `resume_interrupted_continuation`
  → `advance_independent_trick_queue` 推进下一目标（既有机制，零新增）。
- 手牌/装备可弃数 < 2 时 DiscardHandEquipCount 跳过弃牌（近似，已注明）。

### 牌堆（content/deck.rs）

国战牌堆中 `"yiyi"`（方块 4、红桃 J 两张）改投 `CardKind::TrickYiyi`；其余 11 张锦囊
保持 `CardKind::Other` 占位（结算为独立后续工作）。因此**国战金标漂移**（guozhan_4_42、
guozhan_2_7 两张向量）——已按规矩重钉本域向量（新 hash 见 conformance）。

### 单测

`crates/engine-core/tests/guozhan_trick_yiyi.rs`：guozhan-standard 双人局，强制双方同势力，
注入一张 TrickYiyi，完整走 PlayCard → 无懈窗口跳过 → 逐目标（席位序）摸 2 → 弃 2 →
队列推进 → 结算结束。**1 passed**。

## 验证

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过**（国战向量已重钉：guozhan_4_42/guozhan_2_7 新 hash）✅ |
| 新单测 | **guozhan_trick_yiyi 1 passed**（端到端结算）✅ |
| 失败集合 | **241 = 基线 246 − 5**：无新增失败；5 项出集（dcjianxiong 等）为并行代理修复他人域金标 ✅ |
| `cargo check` | 无编译错误；engine-core warnings 59（改行 0 新增）✅ |

**ABI 说明**：无 `RULE_VM_ABI_VERSION` bump——新增 CardKind::TrickYiyi（engine-core 内部，
末尾追加序数稳定）与 CardKindTag::TrickYiyi（engine-api 呈现 tag，末尾追加），结算全部
复用既有 EffectRequest（DrawCards / DiscardHandEquipCount）与既有队列/resume 机制，无新
DSL 原语、无新 bytecode、无新 ContinuationKind。修正此前 ADR「ABI 645+」的估算。

## 后续

1. **其余 11 张国战锦囊**：复用本模式（CardKind + trick_effect + 分发臂 + 结算）。建议顺序：
   shuiyanqijunx（复用弃装备/伤害 choice）、lulitongxin（横置/摸牌）、gz_kefuzhongyuan、
   gz_guguoanbang、huoshaolianying、lianjunshengyan、gz_haolingtianxia、gz_wenheluanwu、
   chiling、zhibi（纯信息揭示，可能不入引擎）、yuanjiao（可映射既有 TrickYuanjiao）。
2. **gwyewu（叶舞）造卡入口**：需随机弃牌原语（ADR 0613 已登记）。
3. **gw_dudayuanshuai2 per-card 禁弃置**：需按 def 的 card 门禁（ADR 0608 已登记）。
