# U08 慧眼两辅助的旧等价撤回裁定（2026-11-14）

裁定：`bolehuiyan2`、`bolehuiyan3` 两条均应从“已复核等价”撤回为“简化近似”。依据是下面可逐行追踪的源码反例，不是“本轮未实跑”或搜索未找到消费。它们已有原生落点，故不建议写“缺失”。本补充新增等价签署0，不改清单；具体应用由主agent负责。

HEAD `5c73d1d9158145e33f98f2018013914667d13964`，规则333 / NGSA107 / schema5 / ABI759 / WS8；日期为本机2026-11-14。此前冻结U08独立目录的514文件与manifest均保持只读。本补充只读实际上游/原生代码，没有新跑浏览器、probe、产品全量或增加原31案。下述是源码确认的可达组合反例，不能改写成“已双端实跑”。

| 清单精确key | 核对时旧状态 | 建议 | 证据层级 |
| --- | --- | --- | --- |
| `U08:skills:bolehuiyan2@archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js:2477` | 已复核等价 / u08-signoff | 撤回为简化近似 | 独立源码路径确认；本组合未新实跑 |
| `U08:skills:bolehuiyan3@archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js:2495` | 已复核等价 / u08-signoff | 撤回为简化近似 | 独立源码路径确认；本组合未新实跑 |

## bolehuiyan2：被仁王盾取消的黑杀，在回合结束被原生补作“未命中”判分

给定三人均存活：慧眼主O与杀来源A、目标T互不相同；T有手牌、未受O控制，T装备仁王盾；A不无视防具，使用黑色普通杀。O实际选择“不能命中”。这使两侧都形成有效预测，排除取消按钮异常和不满足主过滤的问题。

上游：[上游预测priority11](../../../../../../archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js#L2407)先于[仁王盾priority6与cancel](../../../../../../archive/upstream/noname/apps/core/card/standard.js#L3486)在同一次shaBegin执行（priority11大于6），保存“不能命中”。仁王盾随后执行`trigger.cancel()`。[cancel调用untrigger并finish](../../../../../../archive/upstream/noname/apps/core/noname/library/element/gameEvent.ts#L677)调用[untrigger把触发阶段改为5](../../../../../../archive/upstream/noname/apps/core/noname/library/element/gameEvent.ts#L650)，将该杀事件`_triggered`设为5并finish；[End仅在触发阶段2派发](../../../../../../archive/upstream/noname/apps/core/noname/library/element/gameEvent.ts#L278)只有`_triggered===2`才派发`shaEnd`，因此此杀不产生shaEnd。[上游bolehuiyan2](../../../../../../archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js#L2477)只有global shaEnd触发、filter只检查storage，不在phaseAfter/回合结束触发。没有其他杀/技能插入时，上游不会在这个回合结束凭空给O摸1；其预测storage也没有被2辅助在该时点清除。

原生：[原生记录预测并授两辅助](../../../../../crates/engine-native-content/src/content/host/step/u08_batch_d_asks.rs#L854)保存Tag2并授2/3；[原生黑杀仁王盾直接结束](../../../../../crates/engine-native-kernel/src/kernel/sha/step.rs#L374)对被挡黑杀直接to_discard并Done，没有Damage或ShaDodged，预测暂存。[原生两辅助事件登记](../../../../../crates/engine-native-content/src/content/host/step/u08_batch_d.rs#L559)却给2辅助登记ShaDodged及TurnEnded，[原生两辅助filter](../../../../../crates/engine-native-content/src/content/host/step/u08_batch_d.rs#L641)在TurnEnded仅要求pred大于0；主技能亦登记TurnEnded，[原生回合结束按未命中处理](../../../../../crates/engine-native-content/src/content/host/step/u08_batch_d_asks.rs#L837)会把未结预测交给[原生判分与清预测](../../../../../crates/engine-native-content/src/content/host/step/u08_batch_d_asks.rs#L765)，按hit=false给Tag2判正确、摸1并清预测/撤两辅助。主技能与2辅助哪个先运行不影响这个差异：同一个“未命中判分”职责在原生回合结束作了上游没有的结算，不能只因日志可能写主技能名而把2辅助条目继续写无差异。

具体反例端点是当前回合结束、尚未开始下一回合摸牌：上游O未因本预测摸牌且storage仍在；原生O多摸1且pred已清。不是把“仁王盾取消”错写成“上游也会shaEnd”。这足以否定2辅助“shaEnd职责无语义差异”的旧签署。

## bolehuiyan3：第一张闪的界雷击正伤害被当成本杀命中

给定A有无双、向另一名T使用普通杀；O为第三人持慧眼，实际选择“不能命中”。T有两张闪及界雷击（上游releiji、原生目录“雷击”），其座次下一人V存活、没有减免伤害能力。T使用第一张闪，界雷击选择V，判定为黑桃，产生2点非杀雷伤；随后T继续使用第二张闪，使本杀最终被完全闪避。这是现有技能的合法组合，未添加新实体、未要求修改产品代码。

上游：[无双将该杀需要的闪改为2](../../../../../../archive/upstream/noname/apps/core/character/standard/skill.js#L2134)把所需闪设为2；[界雷击在使用闪时判定并伤害](../../../../../../archive/upstream/noname/apps/core/character/shenhua/skill.js#L7483)在使用/打出闪时先判定、调用V.damage(2,thunder)。这只是普通damage事件，没有shaDamage；[上游bolehuiyan3](../../../../../../archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js#L2495)仅听global shaDamage，因此不会在此雷击上判“本杀命中”，预测保留。[上游等两闪后结束杀内容](../../../../../../archive/upstream/noname/apps/core/card/standard.js#L243)继续等待第二闪，最终本杀正常结束，[上游bolehuiyan2](../../../../../../archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js#L2477)在shaEnd按“不能命中”判正确、摸1并删storage。[上游杀伤害才显式派发shaDamage](../../../../../../archive/upstream/noname/apps/core/card/standard.js#L145)也说明shaDamage是杀自己的damageTarget显式派发的窗口，不是所有damage的别名。

原生：[第一闪发射响应及界雷击](../../../../../crates/engine-native-kernel/src/kernel/sha/step.rs#L940)在第一闪后调用[原生界雷击发射非杀Damage](../../../../../crates/engine-native-kernel/src/kernel/events.rs#L566)；后者发射`Damage(source=None,target=V,amount=2,card=None)`。[第一闪不足2张时Yield](../../../../../crates/engine-native-kernel/src/kernel/sha/step.rs#L949)发现第一闪不足无双需要的2闪即Yield，此处尚未发ShaDodged；[emit将伤害压为子帧](../../../../../crates/engine-native-kernel/src/machine.rs#L403)把这次Damage作为待处理子帧。此时[原生记录预测并授两辅助](../../../../../crates/engine-native-content/src/content/host/step/u08_batch_d_asks.rs#L854)授予的3辅助仍在，[原生两辅助filter](../../../../../crates/engine-native-content/src/content/host/step/u08_batch_d.rs#L641)对3只检查正的未prevented Damage及pred大于0，既不检查card/kind，也不检查保存的来源/目标。因此V的雷伤会触发3，[原生辅助2/3结算入口](../../../../../crates/engine-native-content/src/content/host/step/u08_batch_d_asks.rs#L871)把它传成hit=true，[原生判分与清预测](../../../../../crates/engine-native-content/src/content/host/step/u08_batch_d_asks.rs#L765)把O的Tag2提前判错、要求弃1并清预测。原生主技能的Damage分支虽有来源/目标匹配，但这次source=None/target=V使主过滤失败，不能挡住3辅助自己的宽过滤。

差异在第二闪前已出现：上游预测仍待本杀结果，原生已要求O弃1且清预测。即使之后第二闪都成功，原生也不能再按本杀未命中补回上游应有的摸1。因此3辅助旧“无语义差异”同样应撤回。无双的第二闪条件使这条推导不依赖Damage与ShaDodged同批压栈的先后；第一闪时根本尚未发ShaDodged。

## 应写入差别栏的限定

- 2：原生以ShaDodged/TurnEnded代替上游shaEnd；黑杀被仁王盾cancel后，上游该杀无shaEnd，原生在TurnEnded仍按未命中判分，存在额外摸/弃和清预测。源码确认，尚未新增该组合双端实跑。
- 3：原生辅助以所有正的未prevented Damage为命中依据，缺杀事件族限定及card/来源/目标约束；无双第一闪的界雷击可先把预测错误结算。上游辅助只听shaDamage。源码确认，尚未新增该组合双端实跑。

上游两辅助自身也只有storage过滤，并非保存并绑定同一次杀invocation；本裁定不写“上游保证绑定同一杀”。也不把原31案的普通杀/闪测试或恢复通过当成以上两个组合已触发。原冻结审查仍按其原范围解读，新增精确证据字段见 [JSON](u08-auxiliary-adjudication.json)。
