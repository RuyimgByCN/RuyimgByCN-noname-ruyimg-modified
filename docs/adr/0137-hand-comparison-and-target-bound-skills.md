# ADR 0137：通用拼点与技能实例目标绑定

## 决策

Rule VM ABI 47 增加主动目标合同 `can_compare` 和类型化效果：

```text
choose_compare_selected_hand_card(player, target, win_skill_def, lose_skill_def)
choose_add_skill_bound_card_target(player, card)
```

拼点双方各选择一张自己的当前手牌。每个选择屏障恢复时都重新校验牌的所有权与区域，双方牌面
公开后进入弃牌堆；只有发动者点数严格较大算胜，平局按失败处理。结果技能通过通用效果授予，
核心不识别「新陷阵」或任何具体技能定义。

需要绑定目标的技能把 `PlayerId` 写入带魔数和版本的技能实例状态。三个通用 modifier
`card_distance_unlimited_for_skill_bound_target`、
`sha_use_unlimited_for_skill_bound_target` 与 `ignore_armor_for_skill_bound_target` 只在查询目标等于
该实例绑定目标时生效。状态进入既有技能实例规范哈希；无效或未知版本不产生修正。

追加目标效果只接受仍在结算的单目标【杀】、普通锦囊或【铁索连环】，从保存续体恢复原目标队列，
按当前牌的权威目标规则重新校验绑定目标后发布 `CardTargetAdded`。它不会发布第二个 `CardUsed`，
也不会绕过存活、重复目标、距离或牌自身目标限制。

旧 JavaScript 只用于确认 `inherit`、拼点、临时技能、目标存储和无视防具的语义，不进入编译产物或
运行时。没有为某个技能名、武将名或定义 ID 增加 Rust 分支。

## 版本

- Rule VM ABI：47
- Rule IR：58
- Engine semantics：133
