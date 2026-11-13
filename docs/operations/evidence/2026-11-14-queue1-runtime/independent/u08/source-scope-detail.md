# 慧眼主技能与辅助事件范围补记（2026-11-14）

这是对 [U08-S1](findings.json) 中“过滤检查来源/目标”的精确限定，不替换此前记录，也没有新增运行场景。

原生主技能的Damage/ShaDodged分支确实检查保存的来源和目标；临时授予的 [bolehuiyan2](../../../../../../crates/engine-native-content/src/content/host/step/u08_batch_d.rs#L641) 则对ShaDodged/TurnEnded只检查pred大于0，[bolehuiyan3](../../../../../../crates/engine-native-content/src/content/host/step/u08_batch_d.rs#L647) 对普通Damage仅检查未prevented、正伤害与pred大于0，不检查来源、目标、卡或kind。不能把主技能的限制推广到独立辅助触发。

上游两辅助的filter也只是storage.bolehuiyan存在；分别依赖global shaEnd/shaDamage事件族，没有自己保存并匹配来源、目标或同一杀invocation。因此本轮静态差别是双方事件族和分支过滤的不同，不写“上游必定绑定同一杀”。31案没有实际插入非杀嵌套伤害或无伤shaEnd；此处仍是源码发现，未升级为已实跑反例。精确字段与清单key见 [source-scope-detail.json](source-scope-detail.json)。
