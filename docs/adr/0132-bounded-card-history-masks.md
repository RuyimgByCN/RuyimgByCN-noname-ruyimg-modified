# ADR 0132：有界手牌花色与出牌阶段类别历史

## 决策

Rule VM ABI 42 增加三项通用声明能力：

```text
owner_is_target_play_phase_card_category_count_positive
draw_by_play_phase_card_category_count(player)
owner_distinct_hand_card_suits_used_this_turn
```

引擎按玩家保存两个固定大小的历史事实：4-bit 掩码表示本回合从实体手牌使用过的有效花色，
3-bit 掩码表示当前出牌阶段使用过的逻辑基本、锦囊和装备类别。新出牌阶段只清类别掩码；新回合
清除两者。技能生成的共享处理区牌即使暂经手牌区也不算“源自手牌”，视为技使用的真实手牌材料
仍按其有效花色记录。动态摸牌在权威效果执行时读取类别数，手牌上限在查询时读取花色数。

## 原因

旧版「精策」扫描 `useCard` 与 `loseEnd` 历史，并用临时技能 storage 保存花色数组。把该结构逐
技能复制会让上千技能各自维护无界事件列表或临时数组。固定掩码能回答同一规则问题，状态大小只
与最多八名玩家有关，并统一覆盖普通用牌、视为用牌、额外出牌阶段和回合边界。

历史跟踪由 ABI 42 包按需启用，使未加载新能力的旧内容继续保持既有零状态哈希。

## 版本

- Rule VM ABI：42
- Rule IR：53
- Engine semantics：128
