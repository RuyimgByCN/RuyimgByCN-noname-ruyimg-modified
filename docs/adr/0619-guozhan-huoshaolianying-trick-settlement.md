# ADR 0619：国战锦囊「火烧连营」结算（CardKind::TrickHuoshaolianying + 队列目标）

> 承接 [ADR 0618](0618-guozhan-guguoanbang-trick-settlement.md)：国战 12 锦囊结算第六张
> —— huoshaolianying（火烧连营）。新增「队列」（下家为中心的同势力连续座次块）目标
> 计算。**无 VM ABI 变化**（CardKind/CardKindTag 末尾追加，结算复用 DealDamage）。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

huoshaolianying（国战变体）：`filterTarget` —— 目标 = 使用者的下家（`player.getNext()`）
或其「队列」成员（`target.inline(next)`）。`inline`：以某人为中心，沿座次向两翼扩展
连续同势力角色（isFriendOf），野心家/未知身份不参与。`content`：对每个目标造成 1 点
火属性伤害。

## 做了什么

### CardKind::TrickHuoshaolianying = 48（末尾追加，序数稳定）

- `state/cards.rs`：`TrickHuoshaolianying = 48` + `from_tag` 48 臂 + `is_trick` 清单。
- `engine-api/view.rs`：`CardKindTag::TrickHuoshaolianying` + `projection.rs` 映射臂。
- `mode/guozhan.rs`：`same_side` 由私有改为 `pub`（经 `crate::mode::` re-export），
  供队列扩展复用（同势力或野心联盟，野心家各自为方）。

### 锦囊分发（command/play.rs）

- `trick_effect::HUOSHAOLIANYING = 24`。
- 分发臂：计算下家（座位序下一名存活角色）→ 从下家向两翼扩展连续
  `same_side` 角色得队列 → `use_independent_trick_targets(actor, card, &queue, …)`。
- 目标策略 `skill_card_target_spec`：`Immediate(Vec::new())`（自动目标）。
- `engine-ai/legal.rs`：PlayCard targets=[]。

### 逐目标结算（apply_deferred_trick HUOSHAOLIANYING 臂）

```
目标非存活 → finish；否则 DealDamage{source: actor, target, amount: 1, nature: FIRE}
经 run_effects_then_finish_independent_trick_target 推进队列。
```

### 牌堆（content/deck.rs）

`"huoshaolianying"`（黑桃 3、梅花 J、红桃 Q 三张）改投 `CardKind::TrickHuoshaolianying`。
国战金标（guozhan_4_42、guozhan_2_7）再次漂移——已重钉本域向量。

### 单测

`crates/engine-core/tests/guozhan_trick_huoshaolianying.rs`：4 人局，势力
wei/shu/shu/wei，下家为蜀 → 其队列 {下家, 其后} 各受 1 火伤，魏方两人不受影响。
**1 passed**。

## 验证

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过**（国战向量已重钉）✅ |
| 新单测 | **guozhan_trick_huoshaolianying 1 passed** ✅ |
| 失败集合 | **208 = 基线 210 − 2**：无新增失败；2 项出集（mbkuili/mbyilie3）为并行代理修复他人域 ✅ |
| `cargo check` | 无编译错误；改行 0 新增警告 ✅ |

**ABI 说明**：无 `RULE_VM_ABI_VERSION` bump——CardKind/CardKindTag 末尾追加，结算复用
DealDamage（FIRE）与队列-resume，无新 DSL 原语、无新 bytecode。

## 后续

1. **其余 5 张国战锦囊**：lianjunshengyan、gz_haolingtianxia、gz_wenheluanwu、chiling、
   zhibi（纯信息揭示，可能不入引擎）、yuanjiao（可映射既有 TrickYuanjiao）。
2. **gwyewu（叶舞）造卡入口**：需随机弃牌原语（ADR 0613 已登记）。
3. **gw_dudayuanshuai2 per-card 禁弃置**：需按 def 的 card 门禁（ADR 0608 已登记）。
