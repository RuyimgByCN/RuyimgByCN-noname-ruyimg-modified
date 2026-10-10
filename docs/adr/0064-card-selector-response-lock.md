# ADR 0064：按牌选择器锁定响应

- 状态：已采纳
- 日期：2026-07-15

## 背景

旧版 `sp:zhenlue` 在普通锦囊的 `useCard` 事件上设置 `nowuxie`，并通过
`targetEnabled` 禁止延时锦囊指定技能拥有者。现有引擎只有按距离锁定响应的通道，
而无懈链的 continuation 只保存原始锦囊效果，不能区分当前正在被响应的是原牌还是
链上的【无懈可击】。

## 决策

1. 增加 `card_use_unrespondable(selector)`，仅应用牌使用者本人拥有的 modifier。
2. 增加 `delayed_trick` 牌选择器，复用 `card_target_cancel(selector)` 的目标合法性检查。
3. 无懈 continuation 固定保存当前被响应牌的逻辑类别和有效花色；每次打出【无懈可击】
   后将二者更新为该次响应牌的类别和花色。
4. `sp:zhenlue` 仅由上述两个通用 modifier 表达，不加入技能 ID 特判。

## 结果

普通锦囊及无懈链都能按当前牌精确判断是否允许响应，延时锦囊仍可进入正常无懈流程，
但不能把技能拥有者选为目标。continuation 载荷语义变化，因此引擎语义修订号升至 48。
