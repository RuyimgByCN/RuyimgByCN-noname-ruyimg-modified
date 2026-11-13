# U06 欢乐卡牌：三技能实际上游对照

## 2026-11-14T02:05:57+08:00 对照结果（尚未提交／发布）

源码HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d`；native探针构建/运行时tracked clean，随后仅L29恢复定义及其测试变化。规则333／NGSA107／schema5／ABI759／WS8。上游现代版1.11.7，独立临时副本与独立headless Chrome，未改技能规则源码。正式样本每case重新加载页面，以8座玩家节点与标准牌加载为就绪条件；角色使用无技能dummy，牌面/HP/技能按native输出构造。

角色初态的HP/maxHP、手牌/装备牌面多重集、相关技能13/13相同。只统一显式名称别名（包括duel→juedou）；牌ID跨引擎不比较。最终上游生成的player_when_*生命周期技能另外完整记录，未声称其期限等价。[比较器](compare.py)含重复牌/牌面/HP断言自检；首次漏duel名称映射的初态断言失败保留于 [首次输出](../restore-fix/11-u06-comparison-first.log)，补映射后[完整输出](../restore-fix/12-u06-comparison-second.log)与[机器表](comparison-fresh-nodes.json)。

| 场景 | 实际分支 | 初态投影 | 最终牌区/技能及弃牌 | 结算状态 |
| --- | --- | --- | --- | --- |
| monkey-confirm | 确认偷桃，弃猴子并获得桃，原目标不回复 | 相同 | 相同 | 两端结束 |
| monkey-cancel | 取消发动，使用者回复1；桃的消耗存在差异 | 相同 | 不同 | 两端结束 |
| monkey-self | 装备者自己用桃，不触发猴子 | 相同 | 相同 | 两端结束 |
| kuwu-hand | 杀伤害后受伤者强制弃手牌 | 相同 | 相同 | 两端结束 |
| kuwu-equip | 同一初态改选装备，保留手牌 | 相同 | 相同 | 两端结束 |
| kuwu-empty | 无he，不询问 | 相同 | 相同 | 两端结束 |
| kuwu-duel | 决斗伤害不触发苦无 | 相同 | 相同 | 两端结束 |
| xuelun-skill-a | 选seat1的马术 | 相同 | 相同 | 两端结束 |
| xuelun-skill-b | 同一目标改选咆哮 | 相同 | 相同 | 两端结束 |
| xuelun-target-b | 改选seat2的武圣 | 相同 | 相同 | 两端结束 |
| xuelun-target-cancel | 选目标时取消 | 相同 | 相同 | 两端结束 |
| xuelun-skill-cancel | 第二步选技能取消 | 相同 | 相同 | 上游结束／原生继续挂起 |
| xuelun-no-skills | 无其他可借技能，不询问 | 相同 | 相同 | 两端结束 |

最终上述投影连同结算状态11/13相同。**猴子取消**：两端seat1 HP2→3，上游桃离手并入弃牌堆；native桃仍在手、弃牌堆空。**写轮眼第二步取消**：上游chooseButton forced=false且取消后返回；native allow_cancel=false、空回答Rejected(InvalidChoice)且保持pending。两项保留为真实差异，不因部分字段相同而记通过。

15次选择前NGSA编码/解码恢复成功，12次回答接受、3次刻意非法空回答拒绝，直接与恢复的state_hash、续行日志、开放选择相同。强制苦无空回答2次拒绝符合该测试分支；写轮眼空回答拒绝是上游差异。这些是一次性运行探针，未称新增回归测试。

实际上游原始记录：[case-results.jsonl](browser/case-results.jsonl)仅run_tag=fresh-nodes的13条；first的13条受跨case虚拟装备残留影响，排除且不删除。首轮根事件顺序错误、fresh页面就绪条件错误及修正分别保留[初始化更正](browser/initialization-correction-01.json)、[隔离更正](browser/fixture-isolation-correction-02.json)、[就绪更正](browser/readiness-correction-03.json)。启动方式沿用正式tabletop-comparison-cdp.mjs的独立profile/Chrome/CDP，未修改正式产品UI。

未比完整牌堆、RNG、事件流、身份/阶段所有字段、临时技能期限或所有组合。monkey、kuwu、xuelunyang仍为简化近似；本来源清单当时7／4／0是历史保留状态，新等价签署0，不代表重新复核其余条目。
