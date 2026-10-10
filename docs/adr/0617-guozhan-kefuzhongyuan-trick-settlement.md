# ADR 0617：国战锦囊「克复中原」结算（CardKind::TrickKefuzhongyuan + 目标二选一 + 蜀势力加成）

> 承接 [ADR 0616](0616-guozhan-lulitongxin-trick-settlement.md)：国战 12 锦囊结算第四张
> —— gz_kefuzhongyuan（克复中原）。新增 `ChooseUseVirtualShaWithBonus` EffectRequest
> 变体（复用既有 `choose_use_virtual_sha` 原语的 damage_bonus 通道）。**无 VM ABI 变化**
> （EffectRequest 尾部追加序数稳定 + CardKind/CardKindTag 末尾追加）。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

gz_kefuzhongyuan（国战）：`filterTarget: true`、`selectTarget: [1, Infinity]`——对任意名
角色使用。`content` 逐目标：目标 `chooseUseTarget("sha")` 视为使用一张【杀】；若目标为
蜀势力，所使用【杀】的伤害值基数 +1（`oncard` 回调 `baseDamage++`）；若取消则摸一张
（蜀势力摸两张）。

## 做了什么

### CardKind::TrickKefuzhongyuan = 46（末尾追加，序数稳定）

- `state/cards.rs`：`TrickKefuzhongyuan = 46` + `from_tag` 46 臂 + `is_trick` 清单。
- `engine-api/view.rs`：`CardKindTag::TrickKefuzhongyuan` + `projection.rs` 映射臂。

### EffectRequest：`ChooseUseVirtualShaWithBonus`（尾部追加，序数稳定）

- `rule-api/effect.rs`：`{ player, ignore_distance, damage_bonus }`——复用
  `choose_use_virtual_sha` 原语已支持的 `damage_bonus` 通道（经 `VirtualShaTargets`
  续体承载，杀目标选择后对所用杀应用基数加成）。
- `primitives/mod.rs` 调度臂 + `hash.rs` 判别值（120，全文件唯一）。

### 锦囊分发（command/play.rs）

- `trick_effect::KEFUZHONGYUAN = 22`。
- 分发臂：1..N 名其他角色目标（上限 = 存活他人数）→
  `use_independent_trick_targets`。
- 目标策略 `skill_card_target_spec`：Select（1..N 其他角色）。
- `engine-ai/legal.rs`：其他存活角色可选为目标。

### 逐目标结算（apply_deferred_trick KEFUZHONGYUAN 臂 + Custom 续体）

```
open_choice(目标, [1=出杀, 2=摸牌], ChoiceKind::Custom, KEFUZHONGYUAN_CUSTOM_CHOICE + 队列尾)
```

- 解析器 `resolve_kefuzhongyuan_choice`（resolve_choice/group_pre_2.rs）：
  选项 1 → `ChooseUseVirtualShaWithBonus{target, ignore_distance: false, damage_bonus: 蜀?1:0}`
  （蜀判定 = `player_group(target) == "shu"`）；选项 2 → `DrawCards{target, 蜀?2:1}`。
  随后 `run_effects_then_finish_independent_trick_target` 推进队列。
- 魔数 `KEFUZHONGYUAN_CUSTOM_CHOICE = 0x4B46_5A59`（pack_sp_2.rs），经 `crate::effect::`
  re-export + resolve_choice/mod.rs Custom 分发臂。

### 牌堆（content/deck.rs）

`"gz_kefuzhongyuan"`（方块 A 一张）改投 `CardKind::TrickKefuzhongyuan`。国战金标
（guozhan_4_42、guozhan_2_7）再次漂移——已重钉本域向量。

### 单测

`crates/engine-core/tests/guozhan_trick_kefuzhongyuan.rs`：①非蜀目标选摸牌 → 摸 1；
②蜀目标选摸牌 → 摸 2；③目标选出杀 → 打开 `VirtualShaTargets` 杀目标选择。**3 passed**。

## 验证

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过**（国战向量已重钉）✅ |
| 新单测 | **guozhan_trick_kefuzhongyuan 3 passed** ✅ |
| 失败集合 | **211 = 基线 211**（双向差异空）——零新增零变化 ✅ |
| `cargo check` | 无编译错误；改行 0 新增警告 ✅ |

**ABI 说明**：无 `RULE_VM_ABI_VERSION` bump——EffectRequest 尾部追加（序数稳定）、
CardKind/CardKindTag 末尾追加，`damage_bonus` 复用既有 `choose_use_virtual_sha` 通道，
无新 DSL 原语、无新 bytecode。

## 后续

1. **其余 7 张国战锦囊**：gz_guguoanbang、huoshaolianying、lianjunshengyan、
   gz_haolingtianxia、gz_wenheluanwu、chiling、zhibi（纯信息揭示，可能不入引擎）、
   yuanjiao（可映射既有 TrickYuanjiao）。
2. **gwyewu（叶舞）造卡入口**：需随机弃牌原语（ADR 0613 已登记）。
3. **gw_dudayuanshuai2 per-card 禁弃置**：需按 def 的 card 门禁（ADR 0608 已登记）。
