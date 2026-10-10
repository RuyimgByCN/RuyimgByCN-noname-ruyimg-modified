# ADR 0225：结算中【杀】的技能-目标绑定

## 背景

SP「谋溃」在使用【杀】指定目标后允许取消、摸一张牌或弃置目标一张牌；选择后，仅当该目标
以【闪】抵消这张【杀】时，目标才弃置来源一张牌。旧 JavaScript 的临时数组不能进入确定性
运行时。

## 决策

ABI 152 在既有 `ShaResolutionRecord` 末尾增加带默认值的 `(SkillId, PlayerId)` 绑定，并在
CardUsed 触发开始前建立精确【杀】记录。新增通用
`choose_draw_or_discard_target_card_record_sha`（HOST_EFFECT 162）负责可取消分支及绑定；
`owner_source_sha_dodged_target_recorded_by_skill_has_discardable_hand_equip` 只匹配同一张【杀】、
同一技能与同一目标。摸牌和双方目标弃牌继续复用现有权威管线。

## 结果

SP npack 发布「谋溃」，联动包通过精确 `official/sp@0.1.0` 依赖发布 SP穆顺。新增 effect
稳定哈希标签 198；没有绑定的旧快照与状态哈希保持原表示。旧 JavaScript 仅作为语义证据。
