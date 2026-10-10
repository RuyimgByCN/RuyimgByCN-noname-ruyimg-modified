# ADR 0616：国战锦囊「戮力同心」结算（CardKind::TrickLulitongxin + 大势力判定 + 横置/摸牌）

> 承接 [ADR 0615](0615-guozhan-shuiyanqijunx-trick-settlement.md)：国战 12 锦囊结算第三张
> —— lulitongxin（戮力同心）。新增「大势力」判定助手（mode/guozhan.rs），结算为
> 逐目标「未横置→横置 / 已横置→摸一张」。**无 VM ABI 变化**（同前两张：CardKind/
> CardKindTag 末尾追加 + 复用既有 EffectRequest 与队列机制）。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

lulitongxin（国战变体）：`enable` 要求存在大势力角色；`filterTarget: true`（选任一存活
角色为代表）；`changeTarget` 把目标扩展为与代表 `isMajor` 相同（同为或同为小势力）的
全部角色（剔除 diaohulishan 持有者）。`content` 逐目标：已横置 → 摸一张；未横置 → 横置。

`isMajor`（国战，player.js）：按 `isFriendOf` 把存活角色分组为「方」，本方 ≥ 2 人且
没有其他方人数更多即大势力；野心家身份未知时 false。

## 做了什么

### 大势力判定（mode/guozhan.rs）

- `same_side(a, b)`：同势力（非野心）或野心联盟；野心家各自为方（除非联盟）。
- `is_major_faction_player(state, player)`：按 same_side 分组，本方 ≥ 2 且不小于任何
  其他方 = 大势力（上游 isMajor 的近似，野心家身份未知视为 false 同上游）。
- 经 `crate::mode::is_major_faction_player` re-export。

### CardKind::TrickLulitongxin = 45（末尾追加，序数稳定）

- `state/cards.rs`：`TrickLulitongxin = 45` + `from_tag` 45 臂 + `is_trick` 清单。
- `engine-api/view.rs`：`CardKindTag::TrickLulitongxin` + `projection.rs` 映射臂。

### 锦囊分发（command/play.rs）

- `trick_effect::LULITONGXIN = 21`。
- 分发臂：选一名存活代表 → 计算其大势力状态 → 目标 = 全部同状态存活角色（含代表）→
  `use_independent_trick_targets(actor, card, &group, LULITONGXIN, field_source)`。
- 目标策略 `skill_card_target_spec`：select_one（任一存活角色为代表）。
- `engine-ai/legal.rs`：任一存活角色可选为代表。

### 逐目标结算（apply_deferred_trick LULITONGXIN 臂）

```
目标已横置（MARK_CHAIN 标记 > 0）→ DrawCards 1；否则 AddMark(MARK_CHAIN)。
run_effects_then_finish_independent_trick_target(...) 推进队列。
```

- 横置以 `MARK_CHAIN` 标记表达（与铁索连环同机制）。
- 无 choice（横置/摸牌均即时）——队列经 run_effects_then_finish 直接推进。

### 牌堆（content/deck.rs）

`"lulitongxin"`（黑桃 Q、梅花 10 两张）改投 `CardKind::TrickLulitongxin`。国战金标
（guozhan_4_42、guozhan_2_7）再次漂移——已重钉本域向量。

### 单测

`crates/engine-core/tests/guozhan_trick_lulitongxin.rs`：4 人局，势力 2/1/1（wei 大势力）。
①选大势力代表 → 目标 = wei 全体，均被横置，小势力不受影响；②预置代表已横置 → 代表
摸一张、未横置者被横置。**2 passed**。

## 验证

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过**（国战向量已重钉）✅ |
| 新单测 | **guozhan_trick_lulitongxin 2 passed** ✅ |
| 失败集合 | **211 = 基线 211**（双向差异空）——零新增零变化 ✅ |
| `cargo check` | 无编译错误；改行 0 新增警告 ✅ |

**ABI 说明**：无 `RULE_VM_ABI_VERSION` bump——CardKind/CardKindTag 末尾追加，结算复用
AddMark/RemoveMark（MARK_CHAIN）/DrawCards 与队列-resume，无新 DSL 原语、无新 bytecode。

## 后续

1. **其余 8 张国战锦囊**：gz_kefuzhongyuan、gz_guguoanbang、huoshaolianying、
   lianjunshengyan、gz_haolingtianxia、gz_wenheluanwu、chiling、zhibi（纯信息揭示，
   可能不入引擎）、yuanjiao（可映射既有 TrickYuanjiao）。
2. **gwyewu（叶舞）造卡入口**：需随机弃牌原语（ADR 0613 已登记）。
3. **gw_dudayuanshuai2 per-card 禁弃置**：需按 def 的 card 门禁（ADR 0608 已登记）。
