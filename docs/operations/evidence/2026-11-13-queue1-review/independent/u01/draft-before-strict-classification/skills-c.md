# U01 独立逐技能表：C组

核验日期：2026-11-13；HEAD `bd58ef06d59940baed8d7f93e61ce571b152fb81`，初始 tracked clean。规则333／NGSA107／schema5／ABI759／WS8。
本表为独立源码比较及现有测试断言检查；本复核 Cargo执行0、浏览器执行0。反例是待执行场景，不写成已复现。A/B/C/D 及 `_asks` 指本来源对应 batch 文件。

## dddqiahua

键：`U01:skills:dddqiahua@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3068`。

上游：[实际定义](../../../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js#L3068)，规则区至3111行；[冻结副本](../inputs/upstream/skill.js#L3068)。原生：[u01_batch_c.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c.rs#L21)；[u01_batch_c_asks.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c_asks.rs#L1)。

| 核对项 | 具体比较 |
| --- | --- |
| 触发时机 | global phaseBegin（整回合），direct。 |
| 过滤和次数 | 回合角色不是自己，hp>0，尚未明置手牌至少为其hp；选牌数量恰为其当前hp。 |
| 选择与取消 | 主人可选该人未明手牌或取消，不能重复选已经明置的实体。 |
| 上游逐步结算 | 将所选实体明置并保留身份，临时授予该角色恂恂。 |
| 当前原生执行 | C:21–59；C_asks:506–512；Prepare时选目标手牌，写dddfusi_shown_count与dddqiahua_granted。 |
| 源码差异/边界 | 计数代替明牌实体，不能过滤已明牌；仅flag没有实际grant恂恂。已有恂恂实例的filter能读取flag，不能令不持技角色凭空拥有该实例。 |
| 待执行反例 | 目标无恂恂，hp2，主人明置其两牌；推进目标Draw，应出现恂恂但当前没有授技。下回合仍持原牌时检查不能重复明置。 |
| 实际断言与缺口 | 仅注册；没有他人回合、未明数量、hp变化、实际临时恂恂及选择恢复断言。 |

本项建议保持或改为「简化近似」；本复核不签署「已复核等价」。

## dddxunxun

键：`U01:skills:dddxunxun@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3112`。

上游：[实际定义](../../../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js#L3112)，规则区至3148行；[冻结副本](../inputs/upstream/skill.js#L3112)。原生：[u01_batch_c.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c.rs#L60)；[u01_batch_c_asks.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c_asks.rs#L1)。

| 核对项 | 具体比较 |
| --- | --- |
| 触发时机 | 本人 phaseDrawBegin1。 |
| 过滤和次数 | 可选发动；取牌堆顶4，即使中途牌堆不足也按上游取牌/洗牌规则；恂恂是恰化授予的临时技能。 |
| 选择与取消 | 初始可不发动；发动后chooseToMove强制恰2顶2底，各组可重排，不能只选两张而不排顺序。 |
| 上游逐步结算 | 取4公开给操作者，分组排序后将top按所排顺序置顶、bottom置底，继续原摸牌阶段。 |
| 当前原生执行 | C:60–110；C_asks:513–519；直接pop最多4，选其中2放顶，其余放底。 |
| 源码差异/边界 | 没有四牌不足时重洗；只提供选2而无组内重排，且中途允许取消。恰化flag也不等于给目标真正安装此技能。 |
| 待执行反例 | 四张已知牌A/B/C/D，将顶组改成B/A；原生只有集合选择不能表示两种顺序。牌库仅2而弃牌足够时比较实际取牌数。 |
| 实际断言与缺口 | 无当前Host恂恂执行测试；缺强制摆牌、顶/底双组排序、洗牌、恰化临时授技与pending恢复。 |

本项建议保持或改为「简化近似」；本复核不签署「已复核等价」。

## dddfusi

键：`U01:skills:dddfusi@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3149`。

上游：[实际定义](../../../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js#L3149)，规则区至3173行；[冻结副本](../inputs/upstream/skill.js#L3149)。原生：[u01_batch_c.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c.rs#L111)；[u01_batch_c_asks.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c_asks.rs#L1)。

| 核对项 | 具体比较 |
| --- | --- |
| 触发时机 | 常驻modifier及全局借用能力；不是到弃牌阶段才开。 |
| 过滤和次数 | 明置手牌不计入手牌上限且弃牌阶段不可因手限弃；本人回合且非空手牌全明时，其他角色不能使用或打出手牌。 |
| 选择与取消 | 此本体不另选；明牌身份由恰化等变更，使用/响应合法性在对应动作入口检查。 |
| 上游逐步结算 | 持续按具体明牌身份计算上限/可弃及全明锁手条件，随获牌、失牌和回合边界更新。 |
| 当前原生执行 | C:111–119；C_asks:520–524；Discard begin写dddfusi_active；d34_flags.rs:93–95使所有手牌忽略手限。 |
| 源码差异/边界 | 没有明牌身份，仅count；一到Discard就把全部牌忽略上限，未明手牌同样受益。全明锁手所需另flag当前Host未写。 |
| 待执行反例 | 给主人1明牌加3未明牌且hp1，Discard应丢未明超限牌，原生全部忽略。再给全明主人回合中的他人响应闪，核对未禁手牌。 |
| 实际断言与缺口 | 无当前Host腹笥混合明暗手牌/全明锁use与respond、获牌解除或恢复断言。 |

本项建议保持或改为「简化近似」；本复核不签署「已复核等价」。

## dddfusi_global

键：`U01:skills:dddfusi_global@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3174`。

上游：[实际定义](../../../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js#L3174)，规则区至3368行；[冻结副本](../inputs/upstream/skill.js#L3174)。原生：[u01_batch_c.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c.rs#L120)；[u01_batch_c_asks.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c_asks.rs#L1)。

| 核对项 | 具体比较 |
| --- | --- |
| 触发时机 | 全局 chooseToUse，借有腹笥者的明置手牌；不是一般给牌主动技。 |
| 过滤和次数 | 借牌者当前窗口可用该实体；来源必须拥有腹笥且牌已明置；本回合来源的许可/拒绝记忆决定是否再次询问。 |
| 选择与取消 | 借者选明牌及正常目标；牌主四选：本次同意、本回合同意、本次拒绝、本回合拒绝；各拒绝不消耗实体。 |
| 上游逐步结算 | 确定来源明牌，取得牌主许可后直接以借牌者使用该实体，维护本回合许可状态与来源，用完可触发拓迹。 |
| 当前原生执行 | C:120–156；C_asks:7–33、652；Play ActivateSkill仅借一牌到手；once_per_turn。 |
| 源码差异/边界 | fusi_source可回退到没有腹笥的自己或非当前人的任意手牌；只有同意/拒绝，没有四种记忆；转移到手而非借用实体实际用牌。 |
| 待执行反例 | 当前人需要闪/桃而腹笥角色有明牌，源可借用；原生只Play转给当前人。一个来源选择本回合拒绝后再次申请，原生无记忆。 |
| 实际断言与缺口 | 仅注册，没有来源资格、明牌过滤、四分支、真实用牌/响应、次数或两主体pending恢复测试。 |

本项建议保持或改为「简化近似」；本复核不签署「已复核等价」。

## dddtuoji

键：`U01:skills:dddtuoji@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3369`。

上游：[实际定义](../../../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js#L3369)，规则区至3387行；[冻结副本](../inputs/upstream/skill.js#L3369)。原生：[u01_batch_c.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c.rs#L157)；[u01_batch_c_asks.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c_asks.rs#L1)。

| 核对项 | 具体比较 |
| --- | --- |
| 触发时机 | global useCardAfter。 |
| 过滤和次数 | 刚使用的是从本人腹笥明牌借出的实体，且本人已经没有未明置手牌。 |
| 选择与取消 | 符合时主人可摸3或取消；非借用、他人自有牌、本人尚有未明手牌不能进入此选择。 |
| 上游逐步结算 | 在借牌完整使用后核对原所有者及明暗手牌状态，确认后主人摸3。 |
| 当前原生执行 | C:157–171；C_asks:525–532；他人CardUsed且fusi_used或shown_count满足即弹确认。 |
| 源码差异/边界 | 未关联实际借用实体和原主；用牌尚未结束即问，shown_count使普通他人用牌也可能触发。 |
| 待执行反例 | 主人有shown_count且没有发生借用，他人从自己手中用普通杀，源不触发拓迹，原生可问摸3。另让借用牌结算中主人获未明牌。 |
| 实际断言与缺口 | 无借用来源身份、未明牌负例、取消或拓迹pending恢复断言；仅存在注册名。 |

本项建议保持或改为「简化近似」；本复核不签署「已复核等价」。

## dddchashi

键：`U01:skills:dddchashi@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3388`。

上游：[实际定义](../../../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js#L3388)，规则区至3506行；[冻结副本](../inputs/upstream/skill.js#L3388)。原生：[u01_batch_c.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c.rs#L172)；[u01_batch_c_asks.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c_asks.rs#L1)。

| 核对项 | 具体比较 |
| --- | --- |
| 触发时机 | global phaseUseBegin，后续被选角色 useCardAfter 效果。 |
| 过滤和次数 | 回合角色不是自己且自己可弃he；实际用牌的花色或type2与所弃牌一致时兑现，命中后撤临时效果。 |
| 选择与取消 | 主人选一张he或取消；命中后双方各摸1。源码没有把判定只限下一张，提示AI不应当成额外规则。 |
| 上游逐步结算 | 弃牌并为当前角色记录suit/type2；其后匹配用牌时双方摸1，清记录。 |
| 当前原生执行 | C:172–218；C_asks:35–50、533–545；记录suit和trick bool，后续未绑定原回合角色。 |
| 源码差异/边界 | 同type2只实现trick对trick，漏基本对基本及装备对装备；后续任意他人匹配牌可能兑现，未绑定原目标。 |
| 待执行反例 | 主人弃红桃杀，目标用黑桃闪类基本牌（合法窗口）时type2同为basic应兑现；原生不同花色且非trick不认。另让旁人先用匹配牌。 |
| 实际断言与缺口 | 没有当前Host察势suit/type2两路径、原目标限制、匹配后清除、取消或恢复测试。 |

本项建议保持或改为「简化近似」；本复核不签署「已复核等价」。

## dddqice

键：`U01:skills:dddqice@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3507`。

上游：[实际定义](../../../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js#L3507)，规则区至3568行；[冻结副本](../inputs/upstream/skill.js#L3507)。原生：[u01_batch_c.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c.rs#L219)；[u01_batch_c_asks.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c_asks.rs#L1)。

| 核对项 | 具体比较 |
| --- | --- |
| 触发时机 | 本人 phaseZhunbeiBegin；发动时动态when本人useCard1的type2=trick解除禁用。 |
| 过滤和次数 | storage dddqice未置true才可发动；普通与延时锦囊均有type2=trick，实际不是仅普通锦囊。 |
| 选择与取消 | 可发动摸2或取消；成功后禁止再次发动直到自己用锦囊。孤立effect子定义无group/授予，不能凭定义就声称结束复用能力已激活。 |
| 上游逐步结算 | 摸2并记storage；第一次符合type2的useCard1删除storage，下一准备阶段再可发动。 |
| 当前原生执行 | C:219–240；C_asks:3–5、546–557；CardUsed以is_trick_kind清disabled。 |
| 源码差异/边界 | 实际追到card_kind.rs:71后确认Le/Bing/ShanDian均在is_trick_kind内，不能误报缺延时锦囊。内容的摸2/禁用/用锦囊解除有落点；可证差异是触发优先级1且不可选序，而上游本体未声明priority、get.priority返回0。 |
| 待执行反例 | 给同一人另一个默认优先级的准备阶段技能，安排其消耗/检查禁用状态，记录两个技能可选次序及结果；这是选序反例建议，尚无实跑。另补延时锦囊解除回归以守住当前正确范围。 |
| 实际断言与缺口 | u01_ddd_eight.rs:891起覆盖confirm/pass、禁用负例、Wuzhong解除、pending恢复/AI；仅用普通锦囊，未测延时锦囊或useCard1顺序。 |

本项建议保持或改为「简化近似」；本复核不签署「已复核等价」。

## dddmiaoxing

键：`U01:skills:dddmiaoxing@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3569`。

上游：[实际定义](../../../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js#L3569)，规则区至3726行；[冻结副本](../inputs/upstream/skill.js#L3569)。原生：[u01_batch_c.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c.rs#L241)；[u01_batch_c_asks.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c_asks.rs#L1)。

| 核对项 | 具体比较 |
| --- | --- |
| 触发时机 | global gameDrawBegin三倍初发；首phaseBefore/enterGame分组；本人drawAfter调平衡。 |
| 过滤和次数 | 初发给拥有者的张数乘3；分为三个ceil(总手牌/3)组，一组手牌、两组水相；以后两水相不同数量才触发平衡。 |
| 选择与取消 | 开局选三组及手牌组；drawAfter必须选至少一水相、可选两组，少则补，多则由本人选弃至与当前手牌相等。 |
| 上游逐步结算 | 先按真实开局倍率发牌，再分组存实体；每次摸牌后以手牌数为标尺分别调选中水相，不能空组自动再造。 |
| 当前原生执行 | C:241–312；C_asks:558–563；首次本人DrawBegin额外pop2+2，选择一水相自动增减。 |
| 源码差异/边界 | 省略三倍初发与三组分配；每组空时可能重新初始化，数量固定2+2；只选1组且自动弃，无调两组选择。 |
| 待执行反例 | 初发4应得12再分4/4/4，原生普通4后Draw时另造2/2。让一水相耗空后再摸牌，检查是否错误再建而非保持空。 |
| 实际断言与缺口 | 仅注册；无gameDraw倍率、开局分组、两组平衡、选弃实体、0张水相持久化或pending恢复测试。 |

本项建议保持或改为「简化近似」；本复核不签署「已复核等价」。

## dddfushi

键：`U01:skills:dddfushi@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3727`。

上游：[实际定义](../../../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js#L3727)，规则区至3865行；[冻结副本](../inputs/upstream/skill.js#L3727)。原生：[u01_batch_c.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c.rs#L313)；[u01_batch_c_asks.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c_asks.rs#L1)。

| 核对项 | 具体比较 |
| --- | --- |
| 触发时机 | 本人 useCardAfter/respondAfter 基本牌，每回合一次。 |
| 过滤和次数 | 已存在水相且触发牌type=basic；选择一水相与当前手牌交换。 |
| 选择与取消 | 主人可不发动/选水相1或2；交换后可选另一男性；男性再独立选择任一水相交换或取消。 |
| 上游逐步结算 | 主人手牌与所选水相全量互换，之后男目标独立决定其水相交换，不沿用主人选项。 |
| 当前原生执行 | C:313–375；C_asks:564–580；主人组选后问男目标，自动让男性交换同一水相。 |
| 源码差异/边界 | 缺男目标自己的组选与取消，强制沿用主人组选；时机为CardUsed/CardResponded Normal。 |
| 待执行反例 | 主人先交换水相1，再选男性A；A应可拒绝或选水相2，原生直接交换1。分别记录三处实体牌集合，检验不是等张数假相同。 |
| 实际断言与缺口 | 无真实基本响应/回合一次、两主体不同组选、男方拒绝、女性过滤或恢复断言。 |

本项建议保持或改为「简化近似」；本复核不签署「已复核等价」。

## dddfenji

键：`U01:skills:dddfenji@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3866`。

上游：[实际定义](../../../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js#L3866)，规则区至3943行；[冻结副本](../inputs/upstream/skill.js#L3866)。原生：[u01_batch_c.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c.rs#L376)；[u01_batch_c_asks.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c_asks.rs#L1)。

| 核对项 | 具体比较 |
| --- | --- |
| 触发时机 | 本人 phaseDrawBegin1且非fixed，或phaseDiscardBegin。 |
| 过滤和次数 | 本技能未用过的牌堆普通锦囊名，且自己有合法目标；used牌名长期记录。 |
| 选择与取消 | 可选任意合法未用普通锦囊/取消，正常chooseUseTarget。抽牌分支用后按技能造成伤害量摸；弃牌分支用后按牌目标数弃。 |
| 上游逐步结算 | Draw令原num=0，再实际用锦囊，后续按造成伤害摸；Discard先取消该阶段，再用锦囊并按目标数付弃牌。 |
| 当前原生执行 | C:376–416；C_asks:71–77、581–591；固定Tag0决斗/1过河/2顺手，直接emit。 |
| 源码差异/边界 | 只有三种锦囊；没有设置原摸牌为0或取消原弃牌，也没有按伤害摸/按目标数弃的后效。 |
| 待执行反例 | 选择无中生有或五谷，上游牌堆存在时可选，原生没有；选决斗并实际造成1伤，核对原摸2是否仍执行及额外摸伤害数。 |
| 实际断言与缺口 | u01_ddd_eight.rs C组断言只三Tag、决斗伤1和used mask、第二触发排Tag0、pending恢复；未检查取消原阶段和两种后效，实际固化三牌近似。 |

本项建议保持或改为「简化近似」；本复核不签署「已复核等价」。

## dddxiaheng

键：`U01:skills:dddxiaheng@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3944`。

上游：[实际定义](../../../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js#L3944)，规则区至4018行；[冻结副本](../inputs/upstream/skill.js#L3944)。原生：[u01_batch_c.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c.rs#L417)；[u01_batch_c_asks.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c_asks.rs#L1)。

| 核对项 | 具体比较 |
| --- | --- |
| 触发时机 | 本人 phaseUseBegin，forced。 |
| 过滤和次数 | 可选任意角色（包含无牌角色）弃2，再选任意角色伤1；两目标均非本人时减1上限；同一人时失去技能；曾伤3名不同角色后删这两条代价。 |
| 选择与取消 | 主人选择弃牌目标及伤害目标；被弃牌者自行选择可弃牌，不是主人固定拿其前2；强制流程不能随意取消。 |
| 上游逐步结算 | 先弃2，再伤1，随后按两个目标身份减上限/失技，并记录累计不同伤害对象供改写代价。 |
| 当前原生执行 | C:417–460；C_asks:592–596；自动取前2he，仅写same_target flag、maxhp最低1。 |
| 源码差异/边界 | 同目标并未移除实际技能；没有伤过3不同人后的代价改写；原生自动弃牌且上限强制保底1。 |
| 待执行反例 | 两次Play均选同一个他人，第一次上游已失狭横，第二次不应再问，原生flag不阻止。另已伤3人时选两个他人应不再减上限。 |
| 实际断言与缺口 | u01_ddd_eight.rs C组测试两人是否含本人决定maxhp、同目标flag；没有检查实际失技，反而只断言flag。弃牌A/B、三人改写、强制与恢复均缺。 |

本项建议保持或改为「简化近似」；本复核不签署「已复核等价」。

## dddfengzheng

键：`U01:skills:dddfengzheng@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4019`。

上游：[实际定义](../../../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js#L4019)，规则区至4212行；[冻结副本](../inputs/upstream/skill.js#L4019)。原生：[u01_batch_c.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c.rs#L461)；[u01_batch_c_asks.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c_asks.rs#L1)。

| 核对项 | 具体比较 |
| --- | --- |
| 触发时机 | global roundStart统计上一轮全局能力使用者；global phaseUse转换每名角色每阶段一次。 |
| 过滤和次数 | 全局转换用手牌点数之和恰13当无中；回合首看牌数量=上一轮曾使用转换的不同人数；同一受益者获超过2张则主人失丰政。 |
| 选择与取消 | 每名使用者自选和13手牌；轮首主人可把所看牌任意分给多人，不只一个接收者。 |
| 上游逐步结算 | 真实用无中并登记本轮使用者集合；下一roundStart看N并自由分配，结算是否失技能。 |
| 当前原生执行 | C:461–524；C_asks:597–601、653–655；仅持技者自己Play选1–4弃摸2，次数累加，自己Prepare全部给一人。 |
| 源码差异/边界 | 没有点数13校验，非持技者无全局入口；N计使用次数而非人数，分配仅一人且没有>2失技。 |
| 待执行反例 | 选单张点数1在原生可发动；源不允许。让两名不同非持技者各用一次13，再分N给两人，对比入口与轮首牌数。 |
| 实际断言与缺口 | 没有Host丰政sum13反例、全局使用者、上一轮人数、多人分配、失技或pending恢复测试。 |

本项建议保持或改为「简化近似」；本复核不签署「已复核等价」。

## dddyulv

键：`U01:skills:dddyulv@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4213`。

上游：[实际定义](../../../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js#L4213)，规则区至4306行；[冻结副本](../inputs/upstream/skill.js#L4213)。原生：[u01_batch_c.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c.rs#L525)；[u01_batch_c_asks.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c_asks.rs#L1)。

| 核对项 | 具体比较 |
| --- | --- |
| 触发时机 | 多种全局牌进入弃牌堆事件；首phaseBefore或enterGame初始化。 |
| 过滤和次数 | 有律时仅本次仍在弃牌堆的同点实体触发；当前回合角色存在且有对应可选动作；每回合第2次发动后强制更换律。 |
| 选择与取消 | 初始化摸1再选手牌放律；命中时当前人选摸1或自行弃1；第二次必须选新律，没手牌先摸。 |
| 上游逐步结算 | 开局置律，逐次核对入弃实体，执行当前人摸弃，记录本回合次数，第二次替换律并处理旧牌。 |
| 当前原生执行 | C:525–575；C_asks:602–621；仅自己Prepare初始化及CardsDiscarded，弃牌用首he。 |
| 源码差异/边界 | 漏loseAsync/cardsDiscard/equip等其他入弃路径；没有第二次强制换律；当前人弃牌不自选且可以取消。 |
| 待执行反例 | 同回合连续两次弃与律同点牌，第二次源必须换律，原生律不变。再用使用后自然入弃的同点牌检查漏触发。 |
| 实际断言与缺口 | 仅注册；无开局入场、各种入弃原因、同点负例、第二次更换、被询问者选择或恢复测试。 |

本项建议保持或改为「简化近似」；本复核不签署「已复核等价」。

## dddyulv_old

键：`U01:skills:dddyulv_old@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4307`。

上游：[实际定义](../../../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js#L4307)，规则区至4402行；[冻结副本](../inputs/upstream/skill.js#L4307)。原生：[u01_batch_c.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c.rs#L576)；[u01_batch_c_asks.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c_asks.rs#L1)。

| 核对项 | 具体比较 |
| --- | --- |
| 触发时机 | 旧版全局入弃事件；首phaseBefore/enterGame宣1至13。 |
| 过滤和次数 | 本次仍弃实体与所宣点相同；连续两次选择同一种摸/弃，触发重宣点数并立即结束当前回合。 |
| 选择与取消 | 选声明点1–13；命中当前人选摸或自行弃；同选项再次发生必须重宣。 |
| 上游逐步结算 | 保存宣点及上次选项，连续同选后重宣并结束turn；与新版换实体律是不同规则。 |
| 当前原生执行 | C:576–616；C_asks:622–633；Prepare首宣，CardsDiscarded只问摸/弃。 |
| 源码差异/边界 | 没有连续选项历史、重宣和结束当前回合，且漏源其他入弃事件；不能与新版合并背书。 |
| 待执行反例 | 同回合两次点7入弃且两次都选摸，源第二次后必须改点并结束turn，原生照常继续出牌。 |
| 实际断言与缺口 | 无旧玉律声明边界、相同/不同选项、终止turn、取消非法输入或恢复测试。 |

本项建议保持或改为「简化近似」；本复核不签署「已复核等价」。

## dddfenye

键：`U01:skills:dddfenye@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4403`。

上游：[实际定义](../../../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js#L4403)，规则区至4874行；[冻结副本](../inputs/upstream/skill.js#L4403)。原生：[u01_batch_c.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c.rs#L617)；[u01_batch_c_asks.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c_asks.rs#L1)。

| 核对项 | 具体比较 |
| --- | --- |
| 触发时机 | phaseUse 主动，每阶段一次，特殊拼点。 |
| 过滤和次数 | 双方可拼点；所有其他角色可自愿加入任意一方或不加入；胜负按两组实际点数平均值。 |
| 选择与取消 | 初始选对手/取消；旁人分别选加入A/B/取消并出牌；胜方各人依次可对不同败方选虚拟杀或不出。 |
| 上游逐步结算 | 收集两组拼点牌、公开算均值，逐个胜者选择不重目标的败者并正常用杀。 |
| 当前原生执行 | C:617–670；C_asks:656–665；普通两人比一张，胜者直接杀败者。 |
| 源码差异/边界 | 整个旁人加入、组均值、多人胜者/不重复败者和逐次取消被省去；一次普通拼点不能替代。 |
| 待执行反例 | 主人10点对手9点，旁人以1点加入主人方，均值5.5应败；原生仍判主人10胜。赢者拒绝出杀时原生已自动杀。 |
| 实际断言与缺口 | 仅注册；没有组队A/B/取消、均值同点、多名赢家、合法杀选择或挂起恢复断言。 |

本项建议保持或改为「简化近似」；本复核不签署「已复核等价」。

## dddshichao

键：`U01:skills:dddshichao@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4875`。

上游：[实际定义](../../../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js#L4875)，规则区至4974行；[冻结副本](../inputs/upstream/skill.js#L4875)。原生：[u01_batch_c.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c.rs#L671)；[u01_batch_c_asks.rs](../../../../../../../crates/engine-native-content/src/content/host/step/u01_batch_c_asks.rs#L1)。

| 核对项 | 具体比较 |
| --- | --- |
| 触发时机 | 本人phaseZhunbeiBegin，forced；保护到下个本人phaseBegin前。 |
| 过滤和次数 | 需要get.zhu(player)，mark+1不超过人数；按第N大不同手牌数档选人（无该档则最少档），主人将自己手牌调至min(目标手牌数,主公maxHp)。 |
| 选择与取消 | 同档多人主人选1；调牌过多时主人自行弃；所选目标之后将伤主人时可取消该伤害并加mark。 |
| 上游逐步结算 | 选择正确手牌档，调整主人手牌；给目标伤害前的可选取消效果，增加下轮档数，并在本人下个turnBegin清。 |
| 当前原生执行 | C:671–704；C_asks:634–638；任意选目标，调整目标手牌到min(主人maxhp,5)，只写guard。 |
| 源码差异/边界 | 调整的主体倒置；候选不按第N大档；guard没有伤害取消消费者或加mark，次轮递进不存在。 |
| 待执行反例 | 主人2手上限4、主公上限6，选到5手目标，应把主人调成5且目标仍5；原生反把目标调成4，主人仍2。目标之后伤主人，比较取消选择。 |
| 实际断言与缺口 | 无实际逝潮手牌档、自己调整、受伤前A/B、mark递进和恢复测试。 |

本项建议保持或改为「简化近似」；本复核不签署「已复核等价」。

