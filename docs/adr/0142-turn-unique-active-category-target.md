# ADR 0142：主动技回合唯一类别与目标

## 决策

Rule VM ABI 52 增加两项主动技合同和两个类型化效果：

```text
targets ... unused_by_skill_this_turn
select_hand|select_he ... unused_category_by_skill_this_turn
discard_selected_record_unique_category_target(player, target)
choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(player, target)
```

技能实例用 `AUC1` 保存三位逻辑类别掩码（基本、锦囊、装备）及按玩家 ID 排序的目标集合。候选生成、
命令提交和成本支付三处都按拥有者视角的有效牌类别重检；重复目标、重复类别或过期实体牌令整个命令
回滚。该状态进入现有 `state_blob` 哈希与快照，回合开始只在形状标签匹配时清除，不会误删其他技能
状态。这样无需为每个类似技能增加独立 storage 字段或技能 ID 分支。

成本效果从 VM 当前主动技选牌中取得精确实体牌，在记录成功后通过普通弃牌移动结算。后续分支读取
该技能本回合 1 起算的 `use_count`：目标无可弃牌时直接摸等量牌，否则拥有者选择摸牌或弃牌；弃牌
由目标选择当时仍可弃的手牌/装备，不足时弃置全部。摸/弃及其嵌套触发完成后，再检查目标当前是否
受伤，并通过公共横置状态通道把拥有者设为横置。

首个使用者是 YIJIANG 陈群「品第」。旧 JavaScript 只用于确认有效类别、目标集合、次数和横置时序，
不进入内容包或运行时。

## 版本

- Rule VM ABI：52
- Rule IR：63
- Engine semantics：138
