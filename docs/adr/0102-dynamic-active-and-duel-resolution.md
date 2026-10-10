# ADR 0102：动态主动次数与决斗完成语义

## 背景

旧版 KEY「寻衅」按拥有者当前体力值限制每回合发动次数，以无实体牌方式使用【决斗】，并在【决斗】结束后要求败者向胜者交牌。旧 JavaScript 只作为语义与审计证据，不进入新运行时。

## 决策

- 主动技次数契约支持 `uses_per_turn owner_hp`，服务器在每次发动时读取当前体力并与 `SkillInstance.use_count` 比较；固定整数契约保持不变。
- 主动目标契约支持 `hp_at_least_owner`，由服务器按选择时的当前体力权威校验。
- `use_virtual_card_on` 创建无花色、无定义来源的临时牌，并进入普通用牌、响应、伤害、濒死和移出处理区流程；技能不复制【决斗】规则。
- 引擎在【决斗】伤害及其濒死调度之后发出 `DuelResolved { initiator, target, winner, loser }`。触发器据此区分原目标、胜者和败者，不从日志或牌移动反推结果。
- 交牌继续使用通用 `GiveCards` 与 `CardMoveReason::Give`，因此「奉承」等监听赠予语义的技能自然组合，不添加 KEY 专用分支。

## 兼容性

该能力提升 Rule VM ABI 至 13、Rule IR 至 23、Engine Semantics 至 98。旧包仍按其声明版本校验；使用动态次数或虚拟指定目标用牌的新包必须声明 ABI 13。
