# U08 逐案独立重算（2026-11-14）

HEAD `5c73d1d9158145e33f98f2018013914667d13964`；产品受跟踪源码 clean，工作区含并行未提交证据。规则333 / NGSA107 / schema5 / ABI759 / WS8。每行仅评价指定投影；初始弃牌缺21实体、手牌顺序及控制上下文的限制见 [范围](README.md)。新增等价签署0。

| case | 实际选择、条件与结果 | 挂起存档（直接/恢复） | 原始输入 |
| --- | --- | --- | --- |
| `y_target_A` | 实际选弃牌9/42、目标1；损失2体力→回复2、摸106/49，目标HP=4。费用候选与受伤目标候选相同。 | 2/2 | [上游](../../U08/browser-round4/upstream-y_target_A.json) / [原生](../../U08/native-round2/native-y_target_A.json) |
| `y_target_B` | 实际选弃牌9/42、目标2；其6上限/1体力→回复4至5，不摸牌；不是回复满至6。 | 2/2 | [上游](../../U08/browser-round4/upstream-y_target_B.json) / [原生](../../U08/native-round2/native-y_target_B.json) |
| `y_cost_B` | 改选弃牌42/46，剩手牌9；目标1仍回复2并摸106/49，费用A/B产生不同剩手牌。 | 2/2 | [上游](../../U08/browser-round4/upstream-y_cost_B.json) / [原生](../../U08/native-round2/native-y_cost_B.json) |
| `y_self` | 目标实际为自己；HP 2→4，弃9/42后摸106/49；目标候选含自己。 | 2/2 | [上游](../../U08/browser-round4/upstream-y_self.json) / [原生](../../U08/native-round2/native-y_self.json) |
| `y_cancel_cards` | 真实取消选技能后退回，重新打开再次取消；原生两次空答均接受、未弃牌/回复/摸牌/消耗限定标记。 | 2/2 | [上游](../../U08/browser-round4/upstream-y_cancel_cards.json) / [原生](../../U08/native-round2/native-y_cancel_cards.json) |
| `y_cancel_target` | 上游选满两张牌后没有取消按钮，实际点“结束回合”并结束phaseUse；原生空目标答后可在原Play阶段重开。费用和限定标记均未消耗，但取消动作与阶段端点不同。 | 3/3 | [上游](../../U08/browser-round4/upstream-y_cancel_target.json) / [原生](../../U08/native-round2/native-y_cancel_target.json) |
| `y_two_cards` | 恰有2张he；上游候选不含医心，原生ActivateSkill拒绝；没有付费或结算。 | 0/0 | [上游](../../U08/browser-round4/upstream-y_two_cards.json) / [原生](../../U08/native-round2/native-y_two_cards.json) |
| `y_no_wound` | 无人受伤；上游仍可点技能但无合法目标/确认键，取消退回；原生ActivateSkill直接拒绝。均未结算，不把候选UI差别写成规则效果差异。 | 0/0 | [上游](../../U08/browser-round4/upstream-y_no_wound.json) / [原生](../../U08/native-round2/native-y_no_wound.json) |
| `y_reuse` | 首次选9/42与目标1成功；第二次上游候选不含医心、原生命令拒绝。本案只测同一阶段再次发动，未跑下一回合。 | 2/2 | [上游](../../U08/browser-round4/upstream-y_reuse.json) / [原生](../../U08/native-round2/native-y_reuse.json) |
| `y_no_skill` | 未授医心；上游候选不含，原生NoSuchSkill；牌区和HP未变。 | 0/0 | [上游](../../U08/browser-round4/upstream-y_no_skill.json) / [原生](../../U08/native-round2/native-y_no_skill.json) |
| `g_target_A` | 实际目标1，弃其青釭114；依次给座0无中106、座1桃46，另一座2连弩118保留。 | 1/1 | [上游](../../U08/browser-round4/upstream-g_target_A.json) / [原生](../../U08/native-round2/native-g_target_A.json) |
| `g_target_B` | 实际目标2，弃其连弩118；座0摸106、座2摸46，座1青釭114保留。目标A/B确有不同装备及获牌者。 | 1/1 | [上游](../../U08/browser-round4/upstream-g_target_B.json) / [原生](../../U08/native-round2/native-g_target_B.json) |
| `g_cancel_target` | 真实取消后重开又取消；原生空目标答两次接受并返还次数；双方未弃装备、未摸牌。 | 2/2 | [上游](../../U08/browser-round4/upstream-g_cancel_target.json) / [原生](../../U08/native-round2/native-g_cancel_target.json) |
| `g_no_equip` | 全桌无装备；上游可点技能但无合法目标，原生直接拒绝；双方未弃牌/摸牌。 | 0/0 | [上游](../../U08/browser-round4/upstream-g_no_equip.json) / [原生](../../U08/native-round2/native-g_no_equip.json) |
| `g_only_own` | 仅自己有青釭114；上游无合法目标、原生拒绝；自己装备保留，不可对自己发动。 | 0/0 | [上游](../../U08/browser-round4/upstream-g_only_own.json) / [原生](../../U08/native-round2/native-g_only_own.json) |
| `g_reuse` | 首次弃目标1青釭并各摸1；第二次上游候选不含鬼斧、原生SkillAlreadyUsed。 | 1/1 | [上游](../../U08/browser-round4/upstream-g_reuse.json) / [原生](../../U08/native-round2/native-g_reuse.json) |
| `g_no_skill` | 未授鬼斧；上游候选不含，原生NoSuchSkill；他人装备未动。 | 0/0 | [上游](../../U08/browser-round4/upstream-g_no_skill.json) / [原生](../../U08/native-round2/native-g_no_skill.json) |
| `b_hit` | 真实点击“能命中”/原生Tag1；目标未出闪，HP 4→3，慧眼主摸106并消耗本回合次数。 | 1/1 | [上游](../../U08/browser-round4/upstream-b_hit.json) / [原生](../../U08/native-round2/native-b_hit.json) |
| `b_miss` | 真实点击“不能命中”/Tag2，目标真实使用闪42；目标HP=4，慧眼主摸106。原生RespondCard、上游chooseToUse分列。 | 2/2 | [上游](../../U08/browser-round4/upstream-b_miss.json) / [原生](../../U08/native-round2/native-b_miss.json) |
| `b_wrong_A` | 预测能命中后目标真实使用闪42；失败强制弃主手牌4，剩46，不额外摸牌；实际失败弃牌窗口已触发。 | 3/3 | [上游](../../U08/browser-round4/upstream-b_wrong_A.json) / [原生](../../U08/native-round2/native-b_wrong_A.json) |
| `b_wrong_B` | 同一预测失败分支改弃46，剩4；弃牌A/B实际选择不同实体。 | 3/3 | [上游](../../U08/browser-round4/upstream-b_wrong_B.json) / [原生](../../U08/native-round2/native-b_wrong_B.json) |
| `b_wrong_miss` | 预测不能命中、目标未出闪；HP 4→3，慧眼主强制弃4、剩46；没有预测失败摸牌。 | 2/2 | [上游](../../U08/browser-round4/upstream-b_wrong_miss.json) / [原生](../../U08/native-round2/native-b_wrong_miss.json) |
| `b_cancel_predict` | 真实DOM点击cancel返回{confirm:cancel,bool:false}，没有control。上游因此加bolehuiyan4，原生空答不设used_turn；HP和牌区相同仍不能签取消语义相同。 | 1/1 | [上游](../../U08/browser-round4/upstream-b_cancel_predict.json) / [原生](../../U08/native-round2/native-b_cancel_predict.json) |
| `b_cancel_discard` | 预测不能命中后实际受伤，停于强制弃牌；上游无取消按钮、未尝试隐藏空答；原生空答被拒且仍pending。杀9此时上游ordering、原生discard。该案未完成结算。 | 2/2 | [上游](../../U08/browser-round4/upstream-b_cancel_discard.json) / [原生](../../U08/native-round2/native-b_cancel_discard.json) |
| `b_empty_target` | 目标初始无手牌，双方均无预测；杀使其HP 4→3。上游仍打开无闪chooseToUse并被真实取消，原生直接结算。 | 0/0 | [上游](../../U08/browser-round4/upstream-b_empty_target.json) / [原生](../../U08/native-round2/native-b_empty_target.json) |
| `b_owner_attacker` | 慧眼主自己作为来源出杀，双方均不预测；目标1 HP 4→3。 | 0/0 | [上游](../../U08/browser-round4/upstream-b_owner_attacker.json) / [原生](../../U08/native-round2/native-b_owner_attacker.json) |
| `b_owner_target` | 慧眼主自己成为目标，双方均不预测；其HP 4→3。 | 0/0 | [上游](../../U08/browser-round4/upstream-b_owner_target.json) / [原生](../../U08/native-round2/native-b_owner_target.json) |
| `b_under_control` | 上游目标2._trueMe=座0且isUnderControl=true，未出现预测；原生缺该条件仍问Tag1并额外摸106。控制关系不在双方共同初态投影内。 | 1/1 | [上游](../../U08/browser-round4/upstream-b_under_control.json) / [原生](../../U08/native-round2/native-b_under_control.json) |
| `b_repeat` | 同回合两张普通杀只实际预测一次，目标HP=2、慧眼主只摸106一次。上游出杀ID顺序10→9、原生9→10，不能称完整轨迹同一。 | 1/1 | [上游](../../U08/browser-round4/upstream-b_repeat.json) / [原生](../../U08/native-round2/native-b_repeat.json) |
| `b_cancel_then_retry` | 第一次真实cancel已使上游加bolehuiyan4；第二杀不上预测、未摸106。原生第一次空答后第二杀再次Tag1并摸106。出杀实体先后同样不同，取消后的次数差异在第二杀前已直接可见。 | 2/2 | [上游](../../U08/browser-round4/upstream-b_cancel_then_retry.json) / [原生](../../U08/native-round2/native-b_cancel_then_retry.json) |
| `b_no_skill` | 未授慧眼；双方均无预测，目标HP 4→3，慧眼主未摸106。 | 0/0 | [上游](../../U08/browser-round4/upstream-b_no_skill.json) / [原生](../../U08/native-round2/native-b_no_skill.json) |
