# ADR 0139：当前用牌强化与精确卡牌策略

## 决策

Rule VM ABI 49 增加一个类型化效果、两个通用 modifier、两个事件过滤器和一个 modifier 值来源：

```text
choose_current_card_use_enhancements(player, card, helper_skill_def)
ignore_armor_for_skill_card_policy
card_use_unrespondable_for_skill_card_policy
owner_source_single_target_sha_or_ordinary_trick_current_turn
owner_source_event_card_policy_draw
owner_current_turn_card_use_count
```

效果只接受当前正在结算的单目标【杀】或普通锦囊，并强制从“追加一个合法目标、无视防具、不可
响应、每次造成伤害后摸一张牌”中选择一至两项；没有合法额外目标时不提供第一项。两个选择屏障
恢复时都会重新校验技能实例、当前用牌、阶段、目标和所选项，重复项、越界项与伪造目标不会产生
部分结果。追加目标复用现有【杀】与普通锦囊目标规则，只发出 `CardTargetAdded`，不复制用牌事件。

后三项策略保存在辅助技能实例的版本化 `CUP1` 状态中，按实体 `CardId` 绑定并以卡牌 ID 排序；
同一张牌的掩码合并，不影响之后使用的其他牌。伤害准备阶段按精确牌检查无视防具；【杀】响应和
普通锦囊无懈窗口按精确牌检查不可响应。普通锦囊在 `CardUsed` 可选触发后恢复时会携带原牌 ID 并
重新检查策略，因此触发前不能提前打开无懈窗口。摸牌策略按每个实际伤害事件触发一次。

`owner_current_turn_card_use_count` 只在技能拥有者自己的当前回合返回其主动用牌数，其他角色回合
返回零。核心不识别「奔袭」、吴懿或任何具体技能定义 ID。旧 JavaScript 只用于确认距离、选项、
目标与伤害时序；不会被复制、解释或执行。旧角色的宗族字段 `陈留吴氏` 暂无权威角色 schema
承载，保留在迁移证据中，不伪造成对局规则。

## 版本

- Rule VM ABI：49
- Rule IR：60
- Engine semantics：135
