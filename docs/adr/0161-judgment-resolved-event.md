# ADR 0161：判定完成事件与最终实体判定牌

## 决策

ABI 86 增加正式领域事件 `JudgmentResolved`。事件在所有改判窗口结束后只产生一次，保存判定角色、
最终实体判定牌、最终有效花色、点数，以及发起判定的延时锦囊或装备牌。改判 continuation 同时携带
当前最终实体牌；后续改判会替换该字段，不能退回最初翻出的牌。

DSL 增加 `judgment_resolved` 触发点，以及参数化过滤器
`owner_alive_other_judgment_black_event_player_group_<group>`。可选触发仍使用通用
`by event_player`，由被判定角色确认，不在宿主中按技能 ID 分支。

## 原因

旧版 `judgeEnd` 不只暴露颜色。`tiandu` 等技能会取得 `result.card`，延时锦囊技能会读取判定原因牌，
且鬼才类改判后两者必须保持区分。只记录花色/点数可以临时实现「颂威」，却会让后续判定牌获取、
延时锦囊改写和装备判定再次修改事件结构。

## 结果

- 旧 JavaScript 仅作为语义规格，运行时只消费类型化领域事件和 DSL/VM 包。
- 延时锦囊在判定完成触发队列结束后通过显式 continuation 继续，触发确认不会吞掉原效果或后续判定。
- 八卦阵使用装备实体牌作为判定原因；技能判定没有原因牌时使用 `CardId::NONE`。
- `songwei2` 的 grouped 行为折叠进 `songwei` 本体，角色无需加载旧 JS 或隐式子技能。
