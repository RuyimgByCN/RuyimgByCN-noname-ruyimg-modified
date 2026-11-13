# U01 严格分类与清单建议

基准：2026-11-13，HEAD bd58ef06d59940baed8d7f93e61ce571b152fb81。64技能全部逐项复读，62项有具体源码差异，2项只列待核验。本复核没有修改inventory。

62项中61项列为新增降级候选；暗织保持既有简化近似。此表不是62条运行反例，也没有新增任何等价签署。后续聚贤局部修补见独立增量报告，不能覆盖这里的冻结比较。

| ID | 技能与inventory key | 具体源码依据 | 建议 |
| --- | --- | --- | --- |
| U01-S01 | `dddyuanzhen`；`U01:skills:dddyuanzhen@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:6` | 已经核对单目标、他人、存活、距离不等于1及目标弃牌/取消摸牌的实际守卫。尚未证实CardUsed适配与useCardToPlayered在追加/删除目标下的行为不同；本轮不以事件名或测试缺口要求降级。 | 待核验；本项不要求变更 |
| U01-S02 | `dddzhishu`；`U01:skills:dddzhishu@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:37` | 上游moveCard可移判定区；原生movable_equips只枚举装备。 | 改为简化近似 |
| U01-S03 | `dddxiaxing`；`U01:skills:dddxiaxing@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:59` | 原生持侠标记的ActivateSkill杀候选枚举所有其他存活者，没有正常杀距离过滤。 | 改为简化近似 |
| U01-S04 | `dddjuxian`；`U01:skills:dddjuxian@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:244` | 冻结输入top分支固定取目标首张he，没有gainPlayerCard选牌；本阶段模式次数按turn_stamp保存。 | 改为简化近似 |
| U01-S05 | `dddjungui`；`U01:skills:dddjungui@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:384` | 弃牌窗口仅限制张数，未要求各不相同花色，可接受两张同花色牌。 | 改为简化近似 |
| U01-S06 | `dddjinggou`；`U01:skills:dddjinggou@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:443` | 原生订阅Equipped，直接装备也满足武器和最大射程过滤；上游必须本人useCardAfter。 | 改为简化近似 |
| U01-S07 | `dddmoyan`；`U01:skills:dddmoyan@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:472` | 原生以每轮首次Judge开始代替global roundStart，展示及增伤生效边界随之推迟。 | 改为简化近似 |
| U01-S08 | `dddshilie`；`U01:skills:dddshilie@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:511` | 原生Host没有选择闪的响应入口；共享view-as要求的flag只由旧VM写入。 | 改为简化近似 |
| U01-S09 | `dddyeshen`；`U01:skills:dddyeshen@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:656` | 原生固定首张黑牌及至多2目标，直接发Linked；上游选黑牌并真实使用铁索，可0目标重铸。 | 改为简化近似 |
| U01-S10 | `dddqiaoduan`；`U01:skills:dddqiaoduan@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:747` | 源chooseCardTarget默认手牌区；原生he_options额外允许装备。 | 改为简化近似 |
| U01-S11 | `ddddiedang`；`U01:skills:ddddiedang@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:802` | 上游每出牌阶段一次；原生SkillLimits.once_per_turn阻止同回合额外出牌阶段再次发动。 | 改为简化近似 |
| U01-S12 | `dddanliu`；`U01:skills:dddanliu@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:861` | 上游展示异色牌立即结束；原生跳过confirm后继续交换。 | 改为简化近似 |
| U01-S13 | `dddguiying`；`U01:skills:dddguiying@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:968` | 源按座次循环吴角色且各可拒绝；原生只first_wu_responder并自动取牌，无后续响应者。 | 改为简化近似 |
| U01-S14 | `dddbingjian`；`U01:skills:dddbingjian@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1204` | 源帮助角色与虚拟牌目标分离；原生把可选帮助目标直接作为杀目标，取消帮助便不使用牌。 | 改为简化近似 |
| U01-S15 | `dddtongyu`；`U01:skills:dddtongyu@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1443` | 源真实五谷牌池是目标人数加支付张数；原生只pop支付张数，未执行五谷逐人选牌。 | 改为简化近似 |
| U01-S16 | `dddzhilian`；`U01:skills:dddzhilian@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1584` | 源仅收本回合入弃的杀；原生从整个弃牌堆回收全部杀。 | 改为简化近似 |
| U01-S17 | `dddjijian`；`U01:skills:dddjijian@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1676` | 源展示后双方依次真实用不重名虚拟牌；原生将该段替换为双方各摸1。 | 改为简化近似 |
| U01-S18 | `dddjijian_old`；`U01:skills:dddjijian_old@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1807` | 源按同色数循环虚拟用牌；原生仅同色时摸1，没有虚拟用牌段。 | 改为简化近似 |
| U01-S19 | `dddzhengjun`；`U01:skills:dddzhengjun@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2005` | 源从本事件getl/getg等定位受影响者；原生选全场首个数值相等的other，未绑定实际受影响者。 | 改为简化近似 |
| U01-S20 | `dddxianxi`；`U01:skills:dddxianxi@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2194` | 源向原杀追加多个targets并在useCardAfter按未伤者计算N；原生不改targets，提前固定N=1。 | 改为简化近似 |
| U01-S21 | `dddlingyong`；`U01:skills:dddlingyong@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2296` | 源非杀判定牌可实际使用并无两次上限；原生将其拿到手且循环上限2。 | 改为简化近似 |
| U01-S22 | `dddxuxiao`；`U01:skills:dddxuxiao@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2333` | 源同次弃牌可循环逐张用黑基本作兵粮；原生完成一张便结束。 | 改为简化近似 |
| U01-S23 | `dddxuyu`；`U01:skills:dddxuyu@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2407` | 源下一牌弃后本体仍可再选摸1；原生must_discard分支清flag后立即Done。 | 改为简化近似 |
| U01-S24 | `dddshijian`；`U01:skills:dddshijian@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2423` | 源额外结算选项增加原牌effectCount；原生选项仅令用牌者摸1。 | 改为简化近似 |
| U01-S25 | `dddxuyu_old`；`U01:skills:dddxuyu_old@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2519` | 源按当前出牌阶段第1/2张分支；原生按整回合计数，额外出牌阶段不重置。 | 改为简化近似 |
| U01-S26 | `dddshijian_old`；`U01:skills:dddshijian_old@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2574` | 源动态临时全局条件与useCardAfter；原生只记特定角色后继并按turn牌数处理。 | 改为简化近似 |
| U01-S27 | `dddtaisi`；`U01:skills:dddtaisi@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2636` | 源只能赠本回合进入弃牌堆的牌；原生候选是全弃牌堆。 | 改为简化近似 |
| U01-S28 | `dddquche`；`U01:skills:dddquche@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2706` | 源可循环给牌/拒绝并逐次减距离，黑杀可另选对象；原生只有一次给牌并对同一人用杀。 | 改为简化近似 |
| U01-S29 | `dddqianlong`；`U01:skills:dddqianlong@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2819` | 源取消击杀奖惩并改为摸3；原生只摸3，没有取消原奖惩的写入或消费者。 | 改为简化近似 |
| U01-S30 | `ddddongcha`；`U01:skills:ddddongcha@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2851` | 鉴超过2张时源当前角色只选1；原生主人取1后把全部余鉴交当前角色。 | 改为简化近似 |
| U01-S31 | `dddzhijie`；`U01:skills:dddzhijie@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2941` | 源将两鉴用于真实闪/无懈响应；原生仅自己Play弃2鉴摸2并写无人读取的flag。 | 改为简化近似 |
| U01-S32 | `dddqiahua`；`U01:skills:dddqiahua@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3068` | 源临时授予目标恂恂技能；原生只写granted flag，不创建目标的恂恂实例。 | 改为简化近似 |
| U01-S33 | `dddxunxun`；`U01:skills:dddxunxun@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3112` | 源chooseToMove可重排顶2底2；原生只有选2张集合，无组内顺序选择。 | 改为简化近似 |
| U01-S34 | `dddfusi`；`U01:skills:dddfusi@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3149` | 源仅具体明牌忽略手限；原生dddfusi_active使全部手牌忽略上限。 | 改为简化近似 |
| U01-S35 | `dddfusi_global`；`U01:skills:dddfusi_global@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3174` | 源获许可后实际借用明牌；原生将牌移到手中，且仅有同意/拒绝两项而非四种回合记忆。 | 改为简化近似 |
| U01-S36 | `dddtuoji`；`U01:skills:dddtuoji@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3369` | 源须刚借用主人腹笥实体且主人无未明牌；原生他人普通CardUsed也可因shown_count满足过滤。 | 改为简化近似 |
| U01-S37 | `dddchashi`；`U01:skills:dddchashi@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3388` | 源同type2包括basic/equip；原生仅记录trick bool，漏同类基本和同类装备。 | 改为简化近似 |
| U01-S38 | `dddqice`；`U01:skills:dddqice@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3507` | 内容摸2/禁用/本人使用锦囊解除均有实际落点，is_trick_kind也覆盖延时锦囊。元数据priority=1/order_selectable=false与源默认priority=0确有不同；本轮未构造组合窗口，暂不将此当内容结算差异要求降级。 | 待核验；本项不要求变更 |
| U01-S39 | `dddmiaoxing`；`U01:skills:dddmiaoxing@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3569` | 源开局三倍初发并分三组；原生首次Draw只另取2+2，没有三倍发牌与分组。 | 改为简化近似 |
| U01-S40 | `dddfushi`；`U01:skills:dddfushi@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3727` | 源男目标可独立选组或取消；原生沿用主人组选并直接交换。 | 改为简化近似 |
| U01-S41 | `dddfenji`；`U01:skills:dddfenji@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3866` | 源替代原摸/弃阶段并有按伤害/目标数后效；原生不取消原阶段，只发三种锦囊。 | 改为简化近似 |
| U01-S42 | `dddxiaheng`；`U01:skills:dddxiaheng@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3944` | 源同目标后实际移除本技能；原生仅写same_target flag且过滤不以此禁止再发动。 | 改为简化近似 |
| U01-S43 | `dddfengzheng`；`U01:skills:dddfengzheng@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4019` | 源仅手牌点数和恰13可当无中；原生任意1至4张，不核对点数。 | 改为简化近似 |
| U01-S44 | `dddyulv`；`U01:skills:dddyulv@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4213` | 源本回合第二次必须换律；原生无该后继，律保持不变。 | 改为简化近似 |
| U01-S45 | `dddyulv_old`；`U01:skills:dddyulv_old@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4307` | 源连续同选后重宣并立即结束当前回合；原生没有选项历史和结束回合段。 | 改为简化近似 |
| U01-S46 | `dddfenye`；`U01:skills:dddfenye@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4403` | 源允许旁人加入两组并按平均点数；原生只做普通两人拼点。 | 改为简化近似 |
| U01-S47 | `dddshichao`；`U01:skills:dddshichao@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4875` | 源将主人手牌调至min(所选目标手牌,主公maxHp)；原生调整的是目标手牌。 | 改为简化近似 |
| U01-S48 | `dddyouxue`；`U01:skills:dddyouxue@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4975` | 源场上共享游标且首次立即移给无标记者；原生每owner独立holder、首次候选含本人。 | 改为简化近似 |
| U01-S49 | `dddchengjing`；`U01:skills:dddchengjing@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5018` | 源只取有游标角色本回合实际用牌的最后合格弃牌实体；原生任意End取弃牌堆末牌，没有经仍默认杀。 | 改为简化近似 |
| U01-S50 | `dddduanbing`；`U01:skills:dddduanbing@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5131` | 源只在卡牌伤害后移全部可转移兵粮；原生未要求卡牌且pop任意判定牌一张，可移走乐/闪电。 | 改为简化近似 |
| U01-S51 | `dddxiaoxing`；`U01:skills:dddxiaoxing@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5223` | 源gameDrawBegin/init补3；原生到首次本人Draw才补3，获得技能时没有init补牌。 | 改为简化近似 |
| U01-S52 | `dddlangzhi`；`U01:skills:dddlangzhi@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5296` | 源仅攻击范围内角色逐人选牌；原生从全场other各取第一张，未过滤攻击范围。 | 改为简化近似 |
| U01-S53 | `dddfuyi`；`U01:skills:dddfuyi@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5427` | 源filter中的(!非负数量)<2恒真，当前本体恒不触发；原生lost flag满足可发动。 | 改为简化近似 |
| U01-S54 | `dddzhuanshe`；`U01:skills:dddzhuanshe@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5593` | 源同名实际用牌可加目标且用过后不罚伤；原生无CardUsed入口，End总会提供伤害选择。 | 改为简化近似 |
| U01-S55 | `dddweiqiu`；`U01:skills:dddweiqiu@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5714` | 源cancel原recover；原生只摸1，原recover仍然提交。 | 改为简化近似 |
| U01-S56 | `dddlianer`；`U01:skills:dddlianer@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5729` | 源回收单实体后use/respond/save受点数cap限制；原生只写rank_cap且没有消费者。 | 改为简化近似 |
| U01-S57 | `dddanzhi`；`U01:skills:dddanzhi@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5796` | 当前Prepare询问仍不同于源整回合phaseBegin；已修末色/独占免疫仍不足以消除该边界。 | 保持简化近似 |
| U01-S58 | `dddshixing`；`U01:skills:dddshixing@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5840` | 源独立两bool可同时授享乐与酒诗；原生if/else互斥mode且不实际授技。 | 改为简化近似 |
| U01-S59 | `ddddanggu`；`U01:skills:ddddanggu@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5889` | 源仅横置数<群数且最少选择差额；原生只需有未横置者，最少1。 | 改为简化近似 |
| U01-S60 | `dddfuzong`；`U01:skills:dddfuzong@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5948` | 源死亡分支只限群角色；原生任意Death都会自动弃主人首he。 | 改为简化近似 |
| U01-S61 | `dddlanghuai`；`U01:skills:dddlanghuai@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5989` | 源取消原摸牌并允许替代数0；原生额外摸max(数,1)且不取消原摸。 | 改为简化近似 |
| U01-S62 | `dddxuanlun`；`U01:skills:dddxuanlun@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:6058` | 源放牌选项可顶底分置并排序、正常恰4；原生选1至4且全部置顶，已有del仍可再选删条款。 | 改为简化近似 |
| U01-S63 | `dddjiexing`；`U01:skills:dddjiexing@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:6236` | 源recover前改接收者保留num；原生不取消原人且另给主人回固定1。 | 改为简化近似 |
| U01-S64 | `dddbailei`；`U01:skills:dddbailei@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:6302` | 源hp1伤害致目标实际离场才失技；原生伤害前即写lost，被救回也失效。 | 改为简化近似 |

可保留的限定结论：31将四个静态字段逐项相同；远振已有单目标/距离及弃牌取消内容；齐策已有摸2/禁用/含延时锦囊的解除。以上均不等于整技能或整来源等价。

缺少测试只用于说明验证范围，不决定本表分类；元数据差异、界面/主公资格和尚未执行的反例没有被一概换算成已复现错误。
