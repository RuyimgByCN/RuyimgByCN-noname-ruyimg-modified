# ADR 0048：防具效果与无视防具的共享修正通道

- 状态：已接受
- Engine semantics：32

## 背景

旧版防具技能在目标限制和伤害修正前都会检查两类绕过条件：来源无视防具，或目标自身的防具效果无效。直接复用普通 `card_target_cancel` / `damage_taken_amount` 会让这些绕过条件丢失，也会迫使引擎按技能名特判。

## 决策

- 新增 `armor_card_target_cancel(selector)`，只承担防具来源的目标取消；
- 新增 `armor_damage_taken_amount(nature)`，只承担防具来源的伤害修正；
- `ignore_armor` 作为来源侧通道，`armor_effect_disabled` 作为目标侧通道；任一非零时统一跳过上述防具修正；
- 标准藤甲的既有硬编码火焰加伤也复用同一绕过判断；
- 新增精确选择器 `wanjian`、`sha_normal` 与 `sha_heart`，不扩大旧技能的牌类范围。

## 结果

国风玉袍与桐油百韧甲的目标保护、火焰加伤可以从旧 JS 精确重写，后续藤甲、仁王盾、白银狮子等防具继续复用相同通道，不增加技能名分支。
