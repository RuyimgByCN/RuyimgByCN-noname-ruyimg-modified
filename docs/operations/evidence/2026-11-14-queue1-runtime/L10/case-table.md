| case | 实际结果 | 判定范围 |
|---|---|---|
| [g_sha_ally](browser-round4/upstream-g_sha_ally.json) | 桃45声明杀、三座均不质疑：目标1由4血降至3，载体进入弃牌；原生同投影。上游真实经过三次质疑询问，原生没有这些询问。 | 限定投影相同 |
| [g_sha_question_fake](browser-round4/upstream-g_sha_question_fake.json) | 座1质疑桃45声明的假杀：上游杀作废，座1保持4血并摸无中104；原生无质疑询问，直接命中座1至3血且未奖励摸牌。 | 实际差异 |
| [g_true_question_discard](browser-round4/upstream-g_true_question_discard.json) | 杀8声明真杀、座2质疑后选择弃桃45：上游座1受伤、座2弃45并获rechanyuan；原生只结算座1受伤，座2仍持45且无缠怨。 | 实际差异 |
| [g_true_question_losehp](browser-round4/upstream-g_true_question_losehp.json) | 座2质疑真杀后取消弃牌：上游座2保留桃45、由4血降至3并获rechanyuan；原生没有惩罚窗，座2仍4血且无缠怨。 | 实际差异 |
| [g_tao](browser-round4/upstream-g_tao.json) | 座0以杀8声明桃且三座均不质疑：上游与原生均由2血回复至3，消耗杀8；只验证此声明与结算投影。 | 限定投影相同 |
| [g_cancel_card](browser-round4/upstream-g_cancel_card.json) | 上游在备份蛊惑的实体牌选择处点可见取消，保留桃45/杀8且未产生reguhuo_phase；原生取消reguhuo_card虽保留牌，却已记once，再ActivateSkill被InvalidChoice拒绝。 | 实际差异 |
| [g_cancel_name](browser-round4/upstream-g_cancel_name.json) | 上游取消声明牌名窗后回到chooseToUse，仍能看到reguhuo；原生取消reguhuo_name后已消耗once，再发动被InvalidChoice拒绝。 | 实际差异 |
| [g_cancel_target](browser-round4/upstream-g_cancel_target.json) | 上游选择载体后点取消可退出而不耗牌、不产生reguhuo_phase；原生reguhuo_targets为不可取消，空答复InvalidChoice且保持原目标询问。 | 实际差异 |
| [g_no_hand](browser-round4/upstream-g_no_hand.json) | 四座无手牌且仅座0有蛊惑：上游chooseToUse无reguhuo选项；原生ActivateSkill拒绝，无询问、无once、无人失牌失血。 | 限定投影相同 |
| [g_no_skill](browser-round4/upstream-g_no_skill.json) | 座0有桃45但没有蛊惑：上游无reguhuo选项；原生发动拒绝，手牌与血量不变。 | 限定投影相同 |
| [g_response](browser-round4/upstream-g_response.json) | 座1真实杀攻击只持桃45的座0：上游chooseToUse响应窗可发动蛊惑声明闪，三座不质疑后以45响应并保持4血；原生没有蛊惑响应入口，直接受伤至3血并保留45。 | 实际差异 |
| [r_basic](browser-round4/upstream-r_basic.json) | 受1伤选获得牌/基本：双方先摸无中104；上游自顶检索桃45，原生自底检索酒85，实体及牌面不同。 | 实际差异 |
| [r_trick](browser-round4/upstream-r_trick.json) | 受1伤选获得牌/锦囊：先摸104后，余堆仅拆67符合；双方获得104和67。该场不能证明多张锦囊的检索顺序。 | 限定投影相同 |
| [r_equip](browser-round4/upstream-r_equip.json) | 受1伤选获得牌/装备：双方先摸104；上游取自顶的青釭114，原生取自底的诸葛118。 | 实际差异 |
| [r_move_A](browser-round4/upstream-r_move_A.json) | 座1先真实装备诸葛118再杀座0；座0受伤后选移动、源1目的0，双方均摸104且将118移至座0装备区。上游先选两座，原生先选实体再目的座。 | 限定投影相同 |
| [r_move_B](browser-round4/upstream-r_move_B.json) | 相同受伤与场上118，改选目的座2：双方均摸104并把118移至座2装备区，座0没有该装备；与A形成不同结果。 | 限定投影相同 |
| [r_cancel_choice](browser-round4/upstream-r_cancel_choice.json) | 上游点真实文字“取消”仍logSkill并摸104，再调用无牌可移的moveCard；原生选Tag2直接结束、不摸牌。保留上游result.control==cancel2与实际中文控件不符的行为。 | 实际差异 |
| [r_cancel_type](browser-round4/upstream-r_cancel_type.json) | 双方已摸104；上游类型chooseButton有取消，取消后技能结束；原生rangjie_type不允许取消，空答复InvalidChoice并保持pending，手牌相同不能掩盖续行差异。 | 实际差异 |
| [r_no_skill](browser-round4/upstream-r_no_skill.json) | 座0未授予让节，座1真实杀令其降至3血；两端均无让节询问且不摸牌。 | 限定投影相同 |
| [r_other_target](browser-round4/upstream-r_other_target.json) | 仅座0有让节，杀改打座2：两端仅座2降至3血，座0未获让节询问、未摸牌。 | 限定投影相同 |
| [r_double](browser-round4/upstream-r_double.json) | 座1先真实使用酒再杀，座0受2伤并完成两次基本牌选择；上游获得104/45/114/10，原生获得104/86/45/43。两次伤害循环均执行，但取牌实体因检索方向不同而分歧。 | 实际差异 |
| [k_A](browser-round4/upstream-k_A.json) | 上游真实phaseBefore进入k_danggu step0，在dui.element.create抛dui未定义，未到伙伴选择；原生选Tag0后first=8、partner=0、active_mask=257，恢复续行成功。不能将原生结果当作上游A已执行。 | 上游实际阻塞 |
| [k_B](browser-round4/upstream-k_B.json) | 独立fresh场同样在k_danggu step0的dui依赖阻塞；原生改选Tag1后first=8、partner=1、active_mask=258，与A不同，未有上游B结算结果。 | 上游实际阻塞 |
| [k_cancel](browser-round4/upstream-k_cancel.json) | 独立fresh场同样未到上游伙伴选择，故没有实际上游取消结论；原生强制k_danggu_partner空答复被InvalidChoice拒绝且仍pending，挂起恢复成功。 | 上游实际阻塞 |
| [k_after_initial_no_trigger](browser-round4/upstream-k_after_initial_no_trigger.json) | 上游phaseNumber=1的真实phase()没有k_danggu触发；原生已置inited=1的下一Phase也不触发。仅比较初始化门控；上游随后正常摸2张至出牌窗，原生停在NextPhase后，后续牌区不作等价比较。 | 仅反例门控 |
| [k_no_skill](browser-round4/upstream-k_no_skill.json) | 均不授予党锢：上游真实phase()没有进入该技能，原生NextPhase没有伙伴问询。上游继续正常摸2张，与原生截点不同，牌区差异不归因党锢。 | 仅反例门控 |
