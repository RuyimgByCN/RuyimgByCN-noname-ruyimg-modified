# ADR 0204：连续同色用牌使用按需回合历史过滤器

## 背景

旧 SIXIANG「奋音」允许拥有者在摸牌阶段额外摸两张牌；若接受，本回合使用的牌与紧邻上一张
用牌颜色相同，就必须弃置一张牌。旧实现直接读取 `_status.currentPhase`、`getLastUsed(1)`，
并通过 `addTempSkill` 安装带 AI 次序修正的 JavaScript 子技能。

## 决策

Rule VM ABI 131 增加
`owner_source_repeats_color_has_discardable_hand_equip`。内容注册表只在包声明该过滤器时启用既有
`LAST_CARD_META` 回合历史；触发调度在写入当前 `CardUsed` 元数据之前，将当前牌和上一张牌的
有效花色归一为红/黑并比较。拥有者必须存活、处于自己的当前回合，双方颜色都不能为无色，
且拥有者当前至少有一张可弃手牌或装备牌。

父技能在可选摸牌阶段复用 `increase_current_draw_count(self, 2)`，并通过普通 `add_skill`
安装助手。助手复用 `discard_one_hand_or_equip(self)` 打开强制精确一张弃牌选择，再由
`turn_ended owner_is_target` 移除自身。

## 结果

- 旧 JavaScript 只保留为语义证据，不进入 npack 或运行时。
- 连续同色代价类技能复用同一过滤器和已有卡牌历史，不维护角色专用 storage。
- ABI 131 不新增 HOST_EFFECT、continuation 或 `EffectRequest`；只增加可复用触发过滤器。
