# OFFLINE 确定性移植

武将 `jun_lvbu`（君吕布）：群，4 体力，主公武将，授予标准包的 `wushuang` 与 `stdqingjiao`。

武将 `goblin`（哥布林）：魔，4 体力，授予 `goblin_shiqiang` 与 `goblin_lingruo`。

武将 `ty_liue`（刘阿）：吴，5 体力，授予 `tyxiyu`。

武将 `yy_quyi`（燕幽麴义）：群，4 体力，授予本包 `yyfuqi` 与 SP 包 `jiaozi`；OFFLINE
通过精确包依赖引用 SP 的稳定技能 ID，不复制技能定义。

武将 `yy_wenchou`（燕幽文丑）：群，4 体力，授予 `yyxuezhan` 与 `yyyazhen`。

武将 `eu_zhangliao`（欧陆张辽）：魏，4 体力，授予 `eu_tuxi`。

武将 `tianfeng`（田丰）：群，3 体力，授予 SP 包的 `sijian` 与本包的 `gzsuishi`；沿用
OFFLINE 对 SP 的精确包依赖，不复制技能定义。

武将 `wn_chengyi`（成宜）：群，4 体力，授予 `wn_dutan`。

武将 `eu_yuejin`（欧陆乐进）：魏，4 体力，授予 `eu_xiaoguo`。

武将 `jiangqing`（蒋钦）：吴，4 体力，授予 `zyshangyi`。

武将 `drag_pangde`（龙庞德）：魏，4 体力，授予 `dragtaiguan` 与 STANDARD `mashu`。

武将 `ty_huangzhong`（桃黄忠）：蜀，4 体力，通过精确 SHENHUA 依赖复用 `xinliegong`，
并授予本包 `tyyizhuang`。

武将 `yy_zhaoyun`（燕幽赵云）：群，3 体力，通过精确 STANDARD 与 SP 依赖复用
`longdan`、`chongzhen` 与 `yicong`。

武将 `ns_lijue`（SP 李傕）：群，初始 4 体力、6 体力上限，授予 `nsfeixiong` 与
`nscesuan`。

武将 `xk_cuilian`（崔廉）：群，3 体力，授予 `xktanlu` 与 `xkjubian`。

武将 `sp_wangyuanji`（SP 王元姬）：魏，3 体力，授予 `spfuluan` 与 `spshude`。

武将 `jd_sb_pangtong`（九鼎庞统）：蜀，3 体力，授予本包 `jdlianhuan`，并通过精确
SHENHUA 依赖复用 `oldniepan`。

武将 `jy_pangtong`（荆扬庞统）：魏，3 体力，授予 `jylianhuan`、`jysuozhou` 与
`jyyuhuo`。

武将 `jy_huanggai`（黄盖）：吴，初始 4 体力、5 体力上限，授予本包 `jyliezhou`
与 `jyzhaxiang`。

武将 `yj_zhenji`（甄姬）：魏，3 体力，授予本包 `yjluoshen`，并通过精确 STANDARD
依赖复用 `qingguo`。

武将 `ns_fanchou`（SP 樊稠）：群，4 体力，授予 `nsyangwu`；弃置红桃手牌后按目标与
拥有者当前手牌差的一半（向上取整）公开获得目标手牌。

武将 `pe_wenqin`（战役篇文钦）：魏，4 体力，授予 `gzjinfa`；弃置一张手牌或装备牌后，
目标交装备或由文钦获得其一张牌，黑桃装备会令目标对文钦使用无距离虚拟【杀】。

武将 `wn_yangqiú`（杨秋）：群，5 体力，授予 `wn_qifeng`；一名其他角色回合结束时，
若杨秋本回合失去过牌，可对其使用无距离、无次数限制且伤害等于失牌数的虚拟【杀】。

`zj_guluo`：空手进入濒死时减一点体力上限并回复至一点；若体力上限因此降至零则死亡。

`yjchenggong`：一名角色使用目标数大于一的牌时，可令使用者摸一张牌。

`xy_zhangu`：回合开始时，若体力上限大于一且手牌区或装备区为空，减一点体力上限并摸两张牌。

`spfuluan`（扶乱）：出牌阶段限一次，弃置三张有效花色相同的手牌或装备牌，令攻击范围内
的一名角色翻面；随后直到回合结束不能主动使用实体或虚拟【杀】，但不影响响应【杀】。
选择可取消，取消或非法选择均不支付牌且不消耗次数。

`spfuluan2`：临时效果存在时，不能主动使用【杀】。

`jdlianhuan`（连环）：梅花手牌可当【铁索连环】使用或以零目标重铸；使用任意实体或
转换【铁索连环】指定一名存活、未横置且手牌区或装备区非空的角色后，可取消或弃置其一张
当前可弃的手牌或装备牌。该选择先于【无懈可击】与铁索效果结算。

`psyuhuo`：防止受到的火属性伤害；火【杀】与【火攻】不计入手牌上限且不能在弃牌阶段弃置。

`jxxiongzi`：摸牌数和手牌上限均增加当前体力值。

`jun_jiang`：拥有者使用有效花色为红色的牌时，可摸一张牌。

`spshude`：结束阶段开始时，若手牌数小于体力上限，可将手牌摸至体力上限。

`jie`：红色【杀】造成的伤害增加一点；黑色【杀】与【决斗】不受影响。

`yjqingjiao`：主公本回合对其他群势力角色造成过伤害时，结束阶段摸一张牌。

`eu_xiru`：摸牌阶段额外摸全场存活角色装备区坐骑牌总数的一半（向上取整）。

`scls_yinshi`：锁定跳过判定阶段；判定区牌保留到后续判定阶段。

`zymingshi`：防具栏为空时，火【杀】和雷【杀】不能选择拥有者。

`gzsuishi2`：与拥有者势力相同的角色死亡后，拥有者失去 1 点体力。

`gzsuishi`（随势）：其他角色因伤害进入濒死，且伤害来源与拥有者势力相同时，拥有者摸一张牌。

`jylijun`（励军）：魏势力角色受到伤害并存活后，拥有者可令其摸一张牌。

`jytongbei`（统北）：对非魏势力角色造成伤害前，可声明一种逻辑牌类别；目标可交给拥有者
一张手牌/装备区同类别牌，否则伤害 +1。与 `jylijun` 闭合发布 `jy_caocao`；旧 JS 不进入运行时。

`eu_jiquan`（集权）：西势力角色的回合开始时，拥有者锁定回复一点体力并摸一张牌。

`psdiwan`（帝挽）：使用【杀】指定目标后，可摸等同于目标数的牌。

`hm_zhong_diamond_skill`（众）：装备技生效时，受到伤害后摸两张牌。

`hm_yingzhan`（营战）：造成和受到的火、雷属性伤害增加一点。

`zj_hanci`（寒慈）：一名存活角色获得一个技能后，存活的拥有者与其依次各摸一张牌；一次获得多个技能时由逐个 `SkillAdded` 事件分别结算。

`jdjizhi`（集智）：使用普通锦囊牌后锁定摸一张牌，并令本回合手牌上限加一；每次发动可叠加，当前回合结束时清空。

`dragchengyuan`（承缘）：限定技，自己或女性角色进入濒死时可令其回复至体力上限；发动后移除自身，拒绝发动时保留。

`sclwansha`（完杀）：拥有者的回合内，其他角色进入濒死时立即死亡，死亡原因沿用原濒死来源。

`scl_jiaxu`（SCL贾诩）：通过精确 REFRESH 依赖复用 `reluanwu`、`reweimu`，并与本包
`sclwansha` 形成完整武将闭包；跨包技能不复制实现。

`ps_guanyu`（★关羽）：通过精确 STANDARD 与 SP 依赖复用 `wusheng`、`nuzhan`，并授予
本包 `pszhonghun`（忠魂）。使用或打出红色牌后可展示牌堆顶；若顶牌印刷颜色为红色则
获得，否则置入弃牌堆。展示与移动复用 ABI 352 的通用确定性效果。

`yysudi`（肃敌）：攻击范围包含拥有者的角色响应拥有者使用的牌后，拥有者锁定摸一张牌。

`pkwuku`（武库）：有角色使用装备牌时，若拥有者的“武库”标记少于三枚，锁定获得一枚。

`zj_cuijian`（摧坚）：出牌阶段限一次，展示一名其他角色的全部手牌，获得其中所有有效【闪】与物理防具；未获得牌则摸两张。

`pszhiyi`（志异）：出牌阶段限一次，令一名角色摸一张牌，然后对其造成一点普通伤害。

`zymoucheng`（谋逞）：出牌阶段每回合限一次，可将一张黑色手牌或装备牌当【借刀杀人】使用。

`yjzhenlve`（缜略）：普通锦囊及其无懈链不可被响应；拥有者不能成为延时锦囊的目标。

`nscesuan`（策算）：受到伤害前锁定防止此伤害并失去一点体力上限；若体力随上限下降则摸一张牌。

`nsfeixiong`（飞熊）：出牌阶段开始时可与一名其他角色拼点；严格胜者对败者造成一点
普通伤害，平局不造成伤害。

`goblin_lingruo`（凌弱）：造成伤害后，锁定获得仍存活的受伤角色一张手牌或装备牌。

`goblin_shiqiang`（恃强）：造成伤害时若手牌多于目标，或受到伤害时若手牌少于来源，伤害增加一点。

`tyxiyu`（戏语）：转化牌或虚拟牌指定第一个目标后可摸一张牌；多目标牌每次使用只触发一次。

`chixin1/chixin2`（赤心子技能）：方片手牌或装备牌可分别当【杀】使用/打出、当【闪】使用/打出。

`yyyazhen`（压阵）：装备区内任意一张牌可当【杀】使用或打出。

`yyxuezhan`（血战）：普通锦囊牌视为【决斗】；使用【决斗】时跳过【无懈可击】响应窗口，
但不影响【决斗】内部的出【杀】响应。

`vtbshuli`（淑丽）：每回合限两次，其他角色使用【杀】造成伤害后，可令伤害来源与拥有者按座次各摸一张牌。

`tyfuli`（伏枥）：限定技，自己进入濒死时可回复至 2 点体力并将手牌补至 2 张；拒绝发动时不消耗。

`tydangxian`（当先）：回合开始时从弃牌堆全部【杀】中强制选择一张获得；选择结算后插入一个先于准备阶段的独立额外出牌阶段。与「伏枥」闭合桃廖化。

`scls_qinzheng`（勤政）：每次使用或打出牌后，按本局累计次数分别检查 3、5、8 的倍数；
每命中一项摸一张牌。与 SCL 骆统组成完整武将闭包。

`scls_zhanshen`（战神）：每个准备阶段强制从尚未获得的三项永久强化中选择一项：摸牌数 +1、【杀】伤害 +1、【杀】目标数 +1；仅余一项时直接获得。与 SCL 吕布组成完整武将闭包。

`pshuoluan`（惑乱）：群势力拥有者与蜀势力角色互相造成伤害时，伤害增加一点；攻守两个方向复用同一个伤害参与者势力条件，不执行旧 JavaScript。

`xkjubian`（惧鞭）：伤害生效前若手牌多于体力，锁定选择超出的手牌弃置并防止该次伤害；按旧 JS 不额外限制伤害来源。

`xktanlu`（贪赂）：其他角色回合开始时可发动，冻结双方当前有符号体力差的绝对值 X。
X 为零或大于目标原始手牌数时强制进入伤害分支；否则目标选择一次性交给当前可被获得的
X 张手牌（不足则全部），或受到一点普通伤害。完整伤害链结束后，存活目标强制弃置崔廉
一张当前可弃手牌；后续链不因失技或崔廉死亡取消。

`jyyuhuo`（浴火）：每名存活的技能拥有者各提供一层最终受伤光环；已横置的其他角色受到属性伤害每层 +1，受到普通伤害每层 -1。

`jylianhuan`（连环）：每个出牌阶段限一次，失去 1 点体力后选择一至两名角色（可含自己），
以普通目标、无懈和横置流程使用一张无实体牌的【铁索连环】。

`jysuozhou`（索舟）：使用有效花色为梅花的牌，或成为此类牌的目标时，每回合共享限一次；
可令当前所有存活且横置的角色按座次各摸一张牌。

`yy_baimaxiaoqi_skill`（白马骁骑）：装备数直接增加攻击范围；装备数至少为 2/3/4 时，分别额外增加等量【杀】次数、减少等量计算距离、增加等量摸牌数。

`yyfuqi`（伏骑）：与你距离不大于 1 的角色不能响应你使用的牌；使用牌每指定一个与你距离大于 1 的目标，便摸一张牌。

`eu_tuxi`（突袭）：造成伤害后可摸一张牌；摸牌及其嵌套触发结算完毕后，按受伤角色、其上家、其下家的旧版顺序，依次获得一张牌。二人局保留上下家重复；序列遇到自己时只可将装备牌收入手牌。

`wn_dutan`（独探）：出牌阶段限一次，不消耗实体牌，将一张虚拟【决斗】同时用于本次主动技选择的一至任意名其他角色。目标列表由主动技契约验证，随后复用普通【无懈可击】与逐目标【决斗】结算。

`eu_xiaoguo`（骁果）：出牌阶段可弃置一张基本手牌，对一名其他角色造成一点普通伤害；仅当这一次伤害使目标首次进入濒死时，本主动技禁用至当前回合结束。若未造成濒死，可在同一出牌阶段继续发动。

`jsp_ganfuren`（SP 甘夫人）：通过精确 SP 依赖复用「淑慎」「神智」。淑慎使用 ABI 101
的通用双层选择链：每回复一点体力可独立选择另一名角色，再由该角色强制选择摸二或回复一。

`jsp_caoren`（SP 曹仁）：通过精确 SP 依赖复用「溃围」「严整」。溃围使用 ABI 102
的场上装备类别计数查询，并以一次性延迟技能在下个摸牌阶段强制弃置当时武器数张牌；
数量不足时全弃，清理动作会等待失牌触发及其嵌套选择全部结束。

`mateng`（战役篇马腾）：通过精确 STANDARD 与 SP 依赖复用「马术」「雄异」。雄异使用
ABI 103 的主动目标列表摸牌效果与结算时最低体力判断；目标可包含自己，摸牌按座次稳定
执行，限定技状态由主动技运行时权威记录。

`zyshangyi`（尚义）：出牌阶段限一次，令一名其他角色私下观看拥有者完整手牌；随后拥有者
私下观看目标手牌，并可取消或弃置其中一张。目标空手时第二阶段直接结束。

`yjxuepin`（血拼）：出牌阶段限一次，选择攻击范围内且有可弃手牌或装备的一名其他角色，
失去一点体力；濒死结算后若存活，强制弃置该角色至多两张牌。恰好弃置两张且按旧 JS
规则判定为同一基本/锦囊/装备类别时，待失牌触发与嵌套选择结束后回复一点体力。

`ylygqiangxi`（强袭）：出牌阶段与本回合未选择过的角色拼点；严格获胜后选择摸一张牌或
弃置目标一张手牌/装备牌。第二次未赢时失去一点体力，待濒死链结束后选择等长座次路径或
自动取较短路径，按旧 JS 顺序逐个造成一点伤害，并令本技能本回合失效。

`nslulve`（掳掠）：出牌阶段限一次，选择一名装备区有牌的其他角色，弃置与其当前装备数
相同的手牌或装备牌，然后对其造成一点无牌来源的普通伤害。

`tyzhuan`（驻岸）：出牌阶段弃置一张有效【杀】，选择装备区有牌的其他角色并获得其一张
装备牌；任意角色使用装备牌后，拥有者可摸一张牌。主动段与触发段共用同一稳定技能定义。

`tybenxiang`（奔襄）：杀死一名角色后，强制选择一名仍存活的其他角色摸三张牌。

`tyxianshou`（献首）：杀死一名角色后，强制选择一名仍存活的其他角色；若其已受伤，
令其回复两点体力。

`nsjianglie`（将烈）：使用【杀】指定有手牌的目标后可令其公开完整手牌；若只有一种有效
颜色则直接选中，红黑均有时由目标自行选择一种颜色，随后弃置该色全部当前可弃手牌。

`dragtaiguan`（抬棺）：出牌阶段可发动当前损失体力次且至少一次；弃置一张手牌或装备牌，
令攻击范围内的一名其他角色自行弃置一张手牌或装备牌。其弃置的不是【杀】且当前体力不低于
拥有者时，经普通响应管线视为对其使用【决斗】。

`tyyizhuang`（益壮）：准备阶段判定区有牌时可发动；经普通伤害管线对自己造成一点伤害，
待濒死与救援完整结束后，弃置当时判定区里的全部牌。

`yjluoshen`（洛神）：准备阶段可判定并获得仍在弃牌堆中的判定牌；第一次判定的改判后
有效颜色会写入可序列化续接状态，之后可反复判定，直到出现不同颜色。每张同色牌的移动
触发完整结束后才询问是否继续；旧 JavaScript 只作语义证据，不进入 npack 或运行时。

`jyliezhou`（烈舟）：造成的伤害改为火属性；对原本横置的目标造成伤害后按最终伤害值摸牌。

`jyzhaxiang`（诈降）：锁定于首次进入濒死时将手牌补至体力上限并获得临时强化；随后使用
【杀】无距离限制，且每回合前两张牌无次数限制。旧 JavaScript 与临时子技能不进入运行时。

`jyzhouji`（舟济）：出牌阶段可发动当前体力值次，复用公共拼点流水线；仅目标严格获胜时
令其摸两张牌，平局不摸牌。

`jyboyan`（驳言）：出牌阶段限两次，与一名其他角色拼点；未获胜的一方手牌本回合不能
使用或打出并摸两张牌，平局时双方都执行。

`pshoufa`（后发）：每回合限一次，对座次更靠前的角色造成伤害前，可将手牌摸至其体力上限。

`pskuiji`（破试探）：出牌阶段限一次，选择一名有手牌的其他角色，从你与其的手牌中共选至多四张
花色互异的牌并弃置；若双方弃置数不等，少弃方失去 1 点体力。以新增 choice 续体原语
`choose_pskuiji_discard`（ABI 349）表达——镜像 `choose_scskuiji_discard` 并在续体 resolve 时按归属
计数、对少弃方施加失去体力。旧 JS 的 `rechouai` 授予在 upstream 中为 dangling（技能从未定义），
实为 no-op，故不建模；旧 JavaScript 不进入运行时。

`zj_fenxun`（奋迅）：出牌阶段限一次，无弃牌成本；选择一名其他角色，本回合对其使用牌
无距离限制。

`zj_duanbing`（短兵）：复用既有“为当前【杀】追加距离一目标”效果；ABI 353 新增通用的
“来源对目标本回合首次伤害”过滤器与“拥有者对距离一目标使用【杀】时闪数 +1”修正器。
智将丁奉因此形成完整武将闭包；旧 JavaScript 只作语义依据，不进入 npack 或运行时。

```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/offline/zj_guluo.skill \
  content/original/ported/offline/yjchenggong.skill \
  content/original/ported/offline/xy_zhangu.skill \
  content/original/ported/offline/spfuluan.skill \
  content/original/ported/offline/spfuluan2.skill \
  content/original/ported/offline/sptuji2.skill \
  content/original/ported/offline/psyuhuo.skill \
  content/original/ported/offline/jxxiongzi.skill \
  content/original/ported/offline/jun_jiang.skill \
  content/original/ported/offline/spshude.skill \
  content/original/ported/offline/jie.skill \
  content/original/ported/offline/yjqingjiao.skill \
  content/original/ported/offline/eu_xiru.skill \
  content/original/ported/offline/scls_yinshi.skill \
  content/original/ported/offline/gzsuishi2.skill \
  content/original/ported/offline/gzsuishi.skill \
  content/original/ported/offline/jylijun.skill \
  content/original/ported/offline/jytongbei.skill \
  content/original/ported/offline/eu_jiquan.skill \
  content/original/ported/offline/psdiwan.skill \
  content/original/ported/offline/hm_zhong_diamond_skill.skill \
  content/original/ported/offline/hm_yingzhan.skill \
  content/original/ported/offline/zj_hanci.skill \
  content/original/ported/offline/jdjizhi.skill \
  content/original/ported/offline/dragchengyuan.skill \
  content/original/ported/offline/sclwansha.skill \
  content/original/ported/offline/pskuiji.skill \
  content/original/ported/offline/yysudi.skill \
  content/original/ported/offline/pkwuku.skill \
  content/original/ported/offline/pszhiyi.skill \
  content/original/ported/offline/zymoucheng.skill \
  content/original/ported/offline/yjzhenlve.skill \
  content/original/ported/offline/nscesuan.skill \
  content/original/ported/offline/nsfeixiong.skill \
  content/original/ported/offline/goblin_lingruo.skill \
  content/original/ported/offline/goblin_shiqiang.skill \
  content/original/ported/offline/tyxiyu.skill \
  content/original/ported/offline/chixin1.skill \
  content/original/ported/offline/chixin2.skill \
  content/original/ported/offline/yyyazhen.skill \
  content/original/ported/offline/yyxuezhan.skill \
  content/original/ported/offline/vtbshuli.skill \
  content/original/ported/offline/tyfuli.skill \
  content/original/ported/offline/xktanlu.skill \
  content/original/ported/offline/xkjubian.skill \
  content/original/ported/offline/jyyuhuo.skill \
  content/original/ported/offline/jylianhuan.skill \
  content/original/ported/offline/jysuozhou.skill \
  content/original/ported/offline/yy_baimaxiaoqi_skill.skill \
  content/original/ported/offline/yyfuqi.skill \
  content/original/ported/offline/eu_tuxi.skill \
  content/original/ported/offline/tydangxian.skill \
  content/original/ported/offline/scls_qinzheng.skill \
  content/original/ported/offline/scls_zhanshen.skill \
  content/original/ported/offline/pshuoluan.skill \
  content/original/ported/offline/wn_dutan.skill \
  content/original/ported/offline/eu_xiaoguo.skill \
  content/original/ported/offline/zj_cuijian.skill \
  content/original/ported/offline/zyshangyi.skill \
  content/original/ported/offline/nsjianglie.skill \
  content/original/ported/offline/yjxuepin.skill \
  content/original/ported/offline/ylygqiangxi.skill \
  content/original/ported/offline/nslulve.skill \
  content/original/ported/offline/tyzhuan.skill \
  content/original/ported/offline/tybenxiang.skill \
  content/original/ported/offline/tyxianshou.skill \
  content/original/ported/offline/dragtaiguan.skill \
  content/original/ported/offline/tyyizhuang.skill \
  content/original/ported/offline/jdlianhuan.skill \
  content/original/ported/offline/jyliezhou.skill \
  content/original/ported/offline/jyzhaxiang.skill \
  content/original/ported/offline/yjluoshen.skill \
  content/original/ported/offline/jyzhouji.skill \
  content/original/ported/offline/jyboyan.skill \
  content/original/ported/offline/pshoufa.skill \
  content/original/ported/offline/zj_fenxun.skill \
  content/original/ported/offline/zj_duanbing.skill \
  content/original/ported/offline/wn_qifeng.skill \
  --characters content/original/ported/offline/characters.json \
  --metadata content/original/ported/offline/metadata.json \
  --out content/original/ported/offline/offline.npack.json \
  --package-id offline
```

`psmoyan`（魔魇·魔姜维）：锁定技，你造成和受到的伤害改为体力流失；有角色失去体力
后你摸一张牌（2026-09-09 批 186；`hp_lost owner_alive` + `damage_before
owner_is_source/owner_is_target` 三臂，prevent_current_damage + lose_hp 等量转化
（pxningzhou 模板）；自伤事件双臂同中为登记债）。
