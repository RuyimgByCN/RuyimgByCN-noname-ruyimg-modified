# ADR 0615：国战锦囊「水淹七军」结算（CardKind::TrickShuiyanqijunx + 弃装/雷伤二选一）

> 承接 [ADR 0614](0614-guozhan-yiyi-trick-settlement.md)：国战 12 锦囊结算第二张 ——
> shuiyanqijunx（水淹七军）。复用 yiyi 建立的「CardKind + 锦囊分发 + 逐目标队列」模式，
> 新增「目标二选一」Custom 续体。**无 VM ABI 变化**（同 yiyi：CardKind/CardKindTag 末尾
> 追加 + 复用既有 EffectRequest 与队列/resume 机制）。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

shuiyanqijunx（国战变体）：`filterTarget: target !== player && target.hasCards("e")` ——
对一名装备区有牌的其他角色使用。`content`：若目标无可弃装备（装备被保护）→ 直接受
1 点雷电伤害；否则目标 `chooseControl` 二选一——⒈弃置装备区里的所有牌；⒉受到使用者
造成的 1 点雷电伤害。`cardnature: "thunder"`。

## 做了什么

### CardKind::TrickShuiyanqijunx = 44（末尾追加，序数稳定）

- `state/cards.rs`：`TrickShuiyanqijunx = 44` + `from_tag` 44 臂 + `is_trick` 清单。
- `engine-api/view.rs`：`CardKindTag::TrickShuiyanqijunx` + `projection.rs` 映射臂。

### 锦囊分发（command/play.rs）

- `trick_effect::SHUIYANQIJUNX = 20`。
- 分发臂：单目标校验（非自己、存活、装备区有牌——上游 filterTarget）→
  `use_independent_trick_targets(actor, card, &[target], SHUIYANQIJUNX, field_source)`。
- 目标策略 `skill_card_target_spec`：select_one（装备区有牌的其他角色）。
- `engine-ai/legal.rs`：装备区有牌的其他角色可作目标。

### 逐目标结算（apply_deferred_trick SHUIYANQIJUNX 臂 + Custom 续体）

```
目标无可弃装备（discardable_equipment_cards 空，即装备被保护）→ 直接 1 点雷电伤害。
否则 open_choice(目标, [1,2], ChoiceKind::Custom, SHUIYANQIJUNX_CUSTOM_CHOICE + 队列尾)。
```

- 解析器 `resolve_shuiyanqijunx_choice`（resolve_choice/group_pre_2.rs）：
  选项 1 → `DiscardAllTargetEquipment{actor, target}`（弃全部装备）；选项 2 →
  `DealDamage{source: actor, target, amount: 1, nature: THUNDER}`。
  随后 `run_effects_then_finish_independent_trick_target` 推进队列（无队列尾则直接跑效果）。
- 魔数 `SHUIYANQIJUNX_CUSTOM_CHOICE = 0x5359_514A`（pack_sp_2.rs），经 `crate::effect::`
  re-export + resolve_choice/mod.rs Custom 分发臂。

### 牌堆（content/deck.rs）

`"shuiyanqijunx"`（黑桃 3、梅花 J、红桃 Q 三张）改投 `CardKind::TrickShuiyanqijunx`。
国战金标（guozhan_4_42、guozhan_2_7）再次漂移——已重钉本域向量。

### 单测

`crates/engine-core/tests/guozhan_trick_shuiyanqijunx.rs`：3 测试——
①弃全部装备分支（目标选择选项 1，装备区清空、无伤害）；②无装备目标被拒（上游
filterTarget）；③受 1 点雷电伤害分支（选项 2，扣 1 血、装备保留）。**3 passed**。

## 验证

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过**（国战向量已重钉）✅ |
| 新单测 | **guozhan_trick_shuiyanqijunx 3 passed** ✅ |
| 失败集合 | **211 = 基线 241 − 30**：无新增失败；30 项出集为并行代理修复他人域金标 ✅ |
| `cargo check` | 无编译错误；改行 0 新增警告 ✅ |

**ABI 说明**：无 `RULE_VM_ABI_VERSION` bump——CardKind::TrickShuiyanqijunx（engine-core
内部末尾追加）与 CardKindTag::TrickShuiyanqijunx（engine-api 呈现 tag 末尾追加），结算
复用 DiscardAllTargetEquipment / DealDamage / 队列-resume，无新 DSL 原语、无新 bytecode。

## 后续

1. **其余 9 张国战锦囊**：lulitongxin（横置/摸牌）、gz_kefuzhongyuan、gz_guguoanbang、
   huoshaolianying、lianjunshengyan、gz_haolingtianxia、gz_wenheluanwu、chiling、zhibi
   （纯信息揭示，可能不入引擎）、yuanjiao（可映射既有 TrickYuanjiao）。
2. **gwyewu（叶舞）造卡入口**：需随机弃牌原语（ADR 0613 已登记）。
3. **gw_dudayuanshuai2 per-card 禁弃置**：需按 def 的 card 门禁（ADR 0608 已登记）。
