/// 宿主原语行为（数据声明的最小完备集）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostBehavior {
    /// 苦无：你对其他角色造成【杀】伤害后，其弃一张手牌（强制；
    /// 登记差异：上游为目标选择弃牌，原生取首张手牌确定性弃置）。
    /// [ShaDamageForceDiscard]（tag 0）。
    ShaDamageForceDiscard,
    /// 酒瓶：结束阶段开始时，已受伤回复 1 点，否则摸 1 张（可选触发，
    /// 确认门由框架 OptionalGate 承担）。
    /// [PhaseEndRecoverOrDraw]（tag 1）。
    PhaseEndRecoverOrDraw,
    /// 手里剑：出牌目标弃 1 张装备或受 1 点伤害（嵌套选择恢复样本）。
    /// [TargetDiscardEquipOrDamage]（tag 2）。
    TargetDiscardEquipOrDamage,
    /// 五行生克（wuxing）：成为有属性牌的目标时——被克弃一张手牌
    /// （确定性首张，登记差异：上游为目标自选），被生摸一张（强制）。
    /// [ShengkeCheck]（tag 3）。
    ShengkeCheck,
    /// 五行盘（wuxing）：出牌阶段限一次——改变一张手牌的五行属性
    ///（选牌 → 选属性，两段选择）。
    /// [ChangeCardNature]（tag 4）。
    ChangeCardNature,
    /// 精彀（3D精选·刘冲）：使用武器牌结算结束后，若攻击范围为全场
    /// 最大，对一名其他角色造成 1 点伤害（强制触发 + 目标选择）。
    /// [MaxRangeWeaponDamage]（tag 5）。
    MaxRangeWeaponDamage,
    /// 望梅止渴（玩点论杀）：目标为最小体力且已受伤回复 1 点；
    /// 目标为最小手牌数摸两张（已回复则摸一张）。
    /// [WangmeiZhikeCard]（tag 6）。
    WangmeiZhikeCard,
    /// 思继（玩点论杀·韩龙）：弃牌阶段结束时，本次弃置了几张【杀】
    /// 就摸 2×张数的牌（可选触发；登记差异：火/雷杀计入）。
    /// [SijiDiscardDraw]（tag 7）。
    SijiDiscardDraw,
    /// 三尖（玩点论杀·武器）：你对其他角色造成【杀】伤害后，可弃一张
    /// 手牌，对距受损者距离 1 以内的另一名角色造成 1 点伤害
    ///（两段选择：弃牌 → 选目标）。
    /// [SanjianWeaponDamage]（tag 8）。
    SanjianWeaponDamage,
    /// 墨守（玩点论杀·费诗）：延时锦囊（乐/兵）不能指定你为目标
    ///（目标过滤原语，命令边界生效）。
    /// [MoshouImmunity]（tag 9）。
    MoshouImmunity,
    /// 水攻（运筹帷幄）：所有有装备的目标各弃一张装备（强制；
    /// 登记差异：上游为目标自选，原生取首张确定性弃置）。
    /// [ShuigongCard]（tag 10）。
    ShuigongCard,
    /// 釜底抽薪（运筹帷幄）：与目标拼点，赢则获得两张拼点牌
    ///（两段选择：目标选牌 → 己方选牌；平局目标胜）。
    /// [FudichouxinCompare]（tag 11）。
    FudichouxinCompare,
    /// 趁火打劫（运筹帷幄）：获得目标一张手牌（登记差异：确定性首张）。
    /// [ChenhuodajieCard]（tag 12）。
    ChenhuodajieCard,
    /// 偷梁换柱（运筹帷幄）：与目标各选一张手牌交换，同色则摸 1
    ///（登记差异：目标临时技暂缓）。
    /// [TouliangSwap]（tag 13）。
    TouliangSwap,
    /// 釜底抽薪（运筹帷幄）：与目标拼点，赢则获得两张拼点牌
    ///（两段选择：目标选牌 → 己方选牌；平局目标胜）。
    /// 灵魂之火（炉石）：目标受 1 点火焰伤害，使用者弃一张手牌
    ///（登记差异：确定性首张）。
    /// [LinghunzhihuoCard]（tag 14）。
    LinghunzhihuoCard,
    /// 照明弹（炉石）：弃目标一张手牌，目标与使用者各摸 1
    ///（登记差异：确定性首张）。
    /// [ZhaomingdanCard]（tag 15）。
    ZhaomingdanCard,
    /// 急耗卡（炉石）：摸一张牌（临时技暂缓）。
    /// [JihuocardDraw]（tag 16）。
    JihuocardDraw,
    /// 隔岸观火（运筹帷幄）：双目标拼点，赢方获得输方一张手牌，
    /// 平局目标一弃牌交由使用者（三段：目标一选牌 → 目标二选牌 → 结算）。
    /// [GeanguanhuoCompare]（tag 17）。
    GeanguanhuoCompare,
    /// 草船借箭（运筹帷幄）：目标选分支——交出所有手牌杀且受 1 点
    /// 反噬杀（简化：直接 1 点伤害），或展示手牌并被弃一张
    ///（登记差异：上游"视为对使用者出杀"走完整杀结算）。
    /// [CaochuanCard]（tag 18）。
    CaochuanCard,
    /// 舌战群儒（运筹帷幄）：与所有有手牌的其他角色拼点
    ///（确定性首轮拼点——各取首张手牌，点数大者摸 1；
    /// 登记差异：上游为多轮 Bool 响应链+随机牌）。
    /// [ShezhanqunruCompare]（tag 19）。
    ShezhanqunruCompare,
    /// 舒筋散（古剑）：目标弃置任意张手牌（至少 1，可取消）并摸等量的牌。
    /// [GujianShujinsanCard]（tag 20）。
    GujianShujinsanCard,
    /// 面里金针（古剑）：目标摸一张牌然后受 1 点伤害
    ///（取消分支为目标失去 1 点体力，规则 v77 原语）。
    /// [GujianMianliCard]（tag 21）。
    GujianMianliCard,
    /// 飞镖（古剑）：目标弃一张黑色手牌或受 1 点伤害
    ///（登记差异：上游为失去 1 点体力）。
    /// [GujianFeibiaoCard]（tag 22）。
    GujianFeibiaoCard,
    /// 金莲烛（古剑）：伤害前置事件取消——持有者打出金莲烛防止此
    /// 伤害，来源摸 1（监听技授予全体角色，按持有过滤）。
    /// [JinlianzhuCancel]（tag 23）。
    JinlianzhuCancel,
    /// 摊黄冻监听技（古剑食物）：出牌阶段结束摸 1，倒计时扣减。
    /// [GujianFoodTanhuadong]（tag 24）。
    GujianFoodTanhuadong,
    /// 有更监听技（古剑食物）：阶段开始已受伤且体力最少回 1，倒计时
    /// 扣减。
    /// [GujianFoodYougeng]（tag 25）。
    GujianFoodYougeng,
    /// 食物授予机（古剑）：使用食物牌时给目标安装监听技实例（rounds
    /// = 剩余回合）并弃置食物牌。
    /// [GujianFoodGrant]（tag 26）。
    GujianFoodGrant,
    /// 青团监听技（古剑食物）：回合内使用首张杀摸 1（once_per_turn）。
    /// [GujianFoodQingtuan]（tag 27）。
    GujianFoodQingtuan,
    /// 元宝肉监听技（古剑食物）：被动状态——杀次数校验读取（额外一张
    /// 杀，每回合一次，校验侧置 once_used）。
    /// [GujianFoodYuanbaorou]（tag 28）。
    GujianFoodYuanbaorou,
    /// 咕咚羹监听技（古剑食物）：受到杀伤害时伤害 -1（一次性移除）。
    /// [GujianFoodGudonggeng]（tag 29）。
    GujianFoodGudonggeng,
    /// 麻婆豆腐监听技（古剑食物）：结束阶段弃其他角色一张牌（确定性）。
    /// [GujianFoodMapodoufu]（tag 30）。
    GujianFoodMapodoufu,
    /// 春饼监听技（古剑食物）：被动状态——手牌上限 +1（弃牌校验读取）。
    /// [GujianFoodChunbing]（tag 31）。
    GujianFoodChunbing,
    /// 鲤鱼汤监听技（古剑食物）：阶段开始体力最少或之一得 1 护甲。
    /// [GujianFoodLiyutang]（tag 32）。
    GujianFoodLiyutang,
    /// 麻茶监听技（古剑食物）：被动状态——不能成为其他角色黑色牌的
    /// 目标（出牌使用路径过滤）。
    /// [GujianFoodMolicha]（tag 33）。
    GujianFoodMolicha,
    /// 菱角监听技（古剑食物）：摸牌阶段额外摸 1 后弃 1（选择机）。
    /// [GujianFoodXiajiao]（tag 34）。
    GujianFoodXiajiao,
    /// 蜜链藕监听技（古剑食物）：被动状态——红桃手牌当桃（出牌转化
    /// 与濒死响应选项并入处读取）。
    /// [GujianFoodMizhilianou]（tag 35）。
    GujianFoodMizhilianou,
    /// 卤玉羹监听技（古剑食物）：出牌阶段限一次，弃基本牌获得牌堆顶。
    /// [GujianFoodLuyugeng]（tag 36）。
    GujianFoodLuyugeng,
    /// 冰魄通（古剑机关）：逐目标双方自选展示一张手牌比对——同色使用
    /// 者弃展示牌、目标失去 1 点体力（登记差异：双方自选展示改首张，
    /// 自选类；失去体力为规则 v77 原语）。
    /// [JiguanBingpoCard]（tag 37）。
    JiguanBingpoCard,
    /// 乾坤镖（古剑机关）：目标及距离 1 邻座各弃首张手牌/装备
    ///（随机牌已对齐：he 合集随机一张，规则 v78 种子 RNG）。
    /// [JiguanQiankunCard]（tag 38）。
    JiguanQiankunCard,
    /// 龙须钩（古剑机关）：获得目标首张装备（登记差异：自选改首张）。
    /// [JiguanLongxuCard]（tag 39）。
    JiguanLongxuCard,
    /// 神火飞鸦（古剑机关）：目标及距离 1 邻座需出闪否则受 1 火伤
    ///（逐目标响应循环，复用 aoe_response 提示词）。
    /// [JiguanShenhuoCard]（tag 40）。
    JiguanShenhuoCard,
    /// 紫阳丹监听技（古剑）：结束阶段护甲 -1，倒计时扣减。
    /// [GujianFoodZiyangdan]（tag 41）。
    GujianFoodZiyangdan,
    /// 玉女元神监听技（古剑）：下次濒死回复 1 体力（一次性移除）。
    /// [GujianYunvyuanshen]（tag 42）。
    GujianYunvyuanshen,
    /// 石会粉禁用状态（古剑）：被动——本回合无法对其他角色用牌
    ///（出牌校验读取；expires_end_of_turn）。
    /// [GujianFoodShihuifenBan]（tag 43）。
    GujianFoodShihuifenBan,
    /// 石会粉（古剑）：目标需出闪否则置禁用状态（单目标响应机）。
    /// [GujianShihuifenCard]（tag 44）。
    GujianShihuifenCard,
    /// 六风散（古剑）：目标从牌堆获得两张实体闪（登记差异：虚拟牌
    /// 生成改实体牌获得，牌堆无闪则无事）。
    /// [GujianLiufengsanCard]（tag 45）。
    GujianLiufengsanCard,
    /// 砂糖（古剑机关）：目标受 1 火伤然后获得 1 护甲。
    /// [JiguanShatangCard]（tag 46）。
    JiguanShatangCard,
    /// 黑龙鳞片（古剑机关）：获得 1 护甲（登记差异：防御距离临时技
    /// 暂缓）。
    /// [JiguanHeilongCard]（tag 47）。
    JiguanHeilongCard,
    /// 硫头阁（古剑机关）：自己摸 1；其他角色弃首张手牌（确定性
    /// 简化，登记差异：上游随机效果池）。
    /// [JiguanLiutougeCard]（tag 48）。
    JiguanLiutougeCard,
    /// 木头面具装备技（古剑）：被动——view-as 转化处读取（once_used
    /// 每回合一次）。
    /// [GujianMutouEquipSkill]（tag 49）。
    GujianMutouEquipSkill,
    /// 帝女玄霜（古剑）：回复 1 体力后可弃任意张摸等量（rider 选择机，
    /// 主动使用路径；濒死路径由 DyingSaveMachine rider 承担）。
    /// [JiguanDinvCard]（tag 50）。
    JiguanDinvCard,
    /// 玉衡装备技（古剑）：出牌阶段限一次——一名有黑桃手牌的其他角色
    /// 将其首张黑桃手牌交给你（确定性简化）。
    /// [GujianYuhengEquipSkill]（tag 51）。
    GujianYuhengEquipSkill,
    /// 盾牌格挡（炉石）：+1 护甲、摸 1、弃 1（确定性首张）。
    /// [HearthDunpaiCard]（tag 52）。
    HearthDunpaiCard,
    /// 治疗波（炉石）：判定红 → 回复 1，否则 +1 护甲。
    /// [HearthZhiliaoboCard]（tag 53）。
    HearthZhiliaoboCard,
    /// 元素毁灭（炉石）：逐目标可弃 1–2 张，每少弃一张受 1 雷伤。
    /// [HearthYuansuCard]（tag 54）。
    HearthYuansuCard,
    /// 星界沟通（炉石）：体力上限 +1、回 1、弃全部手牌。
    /// [HearthXingjieCard]（tag 55）。
    HearthXingjieCard,
    /// 传送门（炉石）：获得牌堆顶一张牌（登记差异：立即使用链暂缓）。
    /// [HearthChuansongCard]（tag 56）。
    HearthChuansongCard,
    /// 弹射之刃（炉石）：座位序链式弃首张手牌（登记差异：敌方-友方
    /// 交替阵营链未实现，保留登记）
    /// 性）。
    /// [HearthTansheCard]（tag 57）。
    HearthTansheCard,
    /// 世界树（炉石）：牌堆搜索两张不同槽位装备，目标依次穿戴。
    /// [HearthShijieshuCard]（tag 58）。
    HearthShijieshuCard,
    /// 燕子药水（gwent）：最小手牌数摸 2，否则摸 1。
    /// [GwentYanzhiCard]（tag 59）。
    GwentYanzhiCard,
    /// 奥祖之雷（gwent）：1 点雷伤后摸 1。
    /// [GwentAozuCard]（tag 60）。
    GwentAozuCard,
    /// 瘟疫（gwent）：有手牌弃首张，无手牌受 1 伤害。
    /// [GwentWenyiCard]（tag 61）。
    GwentWenyiCard,
    /// 置换军（gwent）：反复失去 1 体力上限直至未受伤或手牌≥上限。
    /// [GwentZhihuanCard]（tag 62）。
    GwentZhihuanCard,
    /// 暴雪药水（gwent）：弃前两张手牌然后摸 1。
    /// [GwentBaoxueCard]（tag 63）。
    GwentBaoxueCard,
    /// 倾盆大雨状态（gwent 天气）：手牌上限 -1（弃牌校验读取），
    /// 弃牌阶段结束移除。
    /// [GwentQinpenState]（tag 64）。
    GwentQinpenState,
    /// 碧日浓雾状态（gwent 天气）：不能使用杀（杀次数校验读取），
    /// 出牌阶段结束移除。
    /// [GwentBirinState]（tag 65）。
    GwentBirinState,
    /// 刺骨寒霜状态（gwent 天气）：摸牌阶段结束时弃首张手牌并移除
    ///（登记差异：等价净 -1）。
    /// [GwentCiguState]（tag 66）。
    GwentCiguState,
    /// 天气授予机（gwent）：对 targets 逐个安装对应天气状态。
    /// [GwentWeatherGrant]（tag 67）。
    GwentWeatherGrant,
    /// 昆恩法印状态（gwent）：防止普通伤害（按次数计数，初始
    /// min(5, 存活数)）。
    /// [GwentKunenState]（tag 68）。
    GwentKunenState,
    /// 昆恩法印（gwent）：授予昆恩状态实例。
    /// [GwentKunenCard]（tag 69）。
    GwentKunenCard,
    /// 山崩术（gwent）：弃置其他角色的前两件装备（座位序）。
    /// [GwentShanbengCard]（tag 70）。
    GwentShanbengCard,
    /// 破晓（gwent）：分支选择——清除天气+判定区 / 获得天气法术。
    /// [GwentPoxiaoCard]（tag 71）。
    GwentPoxiaoCard,
    /// [GwentFengyinState]（tag 72）。
    GwentFengyinState,
    /// [GwentZumoCard]（tag 73）。
    GwentZumoCard,
    /// [HuanleMonkeyCard]（tag 74）。
    HuanleMonkeyCard,
    /// [HuanleXuelunyangCard]（tag 75）。
    HuanleXuelunyangCard,
    /// [HuanleMianjuCard]（tag 76）。
    HuanleMianjuCard,
    /// [ZhangbaSha]（tag 77）。
    ZhangbaSha,
    /// [SwdLiuxinghuoyuCard]（tag 78）。
    SwdLiuxinghuoyuCard,
    /// [SwdShentouEquipSkill]（tag 79）。
    SwdShentouEquipSkill,
    /// [SwdYufuluEquipSkill]（tag 80）。
    SwdYufuluEquipSkill,
    /// [SwdYangpijuanCard]（tag 81）。
    SwdYangpijuanCard,
    /// [SwdLangeguaiyiEquipSkill]（tag 82）。
    SwdLangeguaiyiEquipSkill,
    /// [MtgLongshanfengLand]（tag 83）。
    MtgLongshanfengLand,
    /// [MtgHaidaoLand]（tag 84）。
    MtgHaidaoLand,
    /// [MtgShuimomuxueLand]（tag 85）。
    MtgShuimomuxueLand,
    /// [MtgFeixuLand]（tag 86）。
    MtgFeixuLand,
    /// [MtgYixialanLand]（tag 87）。
    MtgYixialanLand,
    /// [MtgYixialanRecast]（tag 88）。
    MtgYixialanRecast,
    /// [MtgShamolvzhouLand]（tag 89）。
    MtgShamolvzhouLand,
    /// [MtgLinzhongjiantaLand]（tag 90）。
    MtgLinzhongjiantaLand,
    /// [MtgLinzhongjiantaStrike]（tag 91）。
    MtgLinzhongjiantaStrike,
    /// [MtgBingheyaosaiLand]（tag 92）。
    MtgBingheyaosaiLand,
    /// [MtgYoulinLand]（tag 93）。
    MtgYoulinLand,
    /// [MtgYoulinDiscard]（tag 94）。
    MtgYoulinDiscard,
    /// [MtgLindixiliuLand]（tag 95）。
    MtgLindixiliuLand,
    /// [WudianCiqiu]（tag 97）。
    WudianCiqiu,
    /// [WudianShuaiyan]（tag 98）。
    WudianShuaiyan,
    /// [WudianZhucheng]（tag 99）。
    WudianZhucheng,
    /// [WudianZhuchengGuard]（tag 100）。
    WudianZhuchengGuard,
    /// [WudianKangyin]（tag 101）。
    WudianKangyin,
    /// [WudianChezhen]（tag 102）。
    WudianChezhen,
    /// [WudianJuedao]（tag 103）。
    WudianJuedao,
    /// [WudianGeju]（tag 104）。
    WudianGeju,
    /// [WudianTuji]（tag 105）。
    WudianTuji,
    /// [WudianMujing]（tag 106）。
    WudianMujing,
    /// [WudianGuangshu]（tag 107）。
    WudianGuangshu,
    /// [WudianGuangshuHeart]（tag 108）。
    WudianGuangshuHeart,
    /// [WudianGuangshuDiamond]（tag 109）。
    WudianGuangshuDiamond,
    /// [WudianGuangshuClub]（tag 110）。
    WudianGuangshuClub,
    /// [WudianGuangshuSpade]（tag 111）。
    WudianGuangshuSpade,
    /// [WudianZiyu]（tag 112）。
    WudianZiyu,
    /// [WudianShouhu]（tag 113）。
    WudianShouhu,
    /// [WudianDuwen]（tag 114）。
    WudianDuwen,
    /// [WudianPaotai]（tag 115）。
    WudianPaotai,
    /// [WudianPaotaiFire]（tag 116）。
    WudianPaotaiFire,
    /// [WudianBaoleiSwitch]（tag 117）。
    WudianBaoleiSwitch,
    /// [WudianZhenchaDraw]（tag 118）。
    WudianZhenchaDraw,
    /// [WudianYuedong]（tag 119）。
    WudianYuedong,
    /// [WudianKuoyin]（tag 120）。
    WudianKuoyin,
    /// [WudianHuhuan]（tag 121）。
    WudianHuhuan,
    /// [WudianJidong]（tag 122）。
    WudianJidong,
    /// [WudianBaoxue]（tag 123）。
    WudianBaoxue,
    /// [WudianLiangou]（tag 124）。
    WudianLiangou,
    /// [WudianXiyang]（tag 125）。
    WudianXiyang,
    /// [WudianFeiren]（tag 126）。
    WudianFeiren,
    /// [WudianFeidan]（tag 127）。
    WudianFeidan,
    /// [WudianZhiyuan]（tag 128）。
    WudianZhiyuan,
    /// [WudianShenqiang]（tag 129）。
    WudianShenqiang,
    /// [WudianShihuo]（tag 130）。
    WudianShihuo,
    /// [WudianMengji]（tag 131）。
    WudianMengji,
    /// [WudianYinbo]（tag 132）。
    WudianYinbo,
    /// [WudianQianggu]（tag 133）。
    WudianQianggu,
    /// [WudianQiangguGuard]（tag 134）。
    WudianQiangguGuard,
    /// [WudianFeitiao]（tag 135）。
    WudianFeitiao,
    /// [WudianPingzhang]（tag 136）。
    WudianPingzhang,
    /// [WudianOwliyong]（tag 137）。
    WudianOwliyong,
    /// [WudianYinshen]（tag 138）。
    WudianYinshen,
    /// [WudianXiandan]（tag 139）。
    WudianXiandan,
    /// [WudianJijia]（tag 140）。
    WudianJijia,
    /// [XianjianLingxue]（tag 141）。
    XianjianLingxue,
    /// [XianjianBinxin]（tag 142）。
    XianjianBinxin,
    /// [XianjianHuimeng]（tag 143）。
    XianjianHuimeng,
    /// [XianjianTianshe]（tag 144）。
    XianjianTianshe,
    /// [XianjianTianshe2]（tag 145）。
    XianjianTianshe2,
    /// [XianjianFanling]（tag 146）。
    XianjianFanling,
    /// [WudianLongxiang]（tag 147）。
    WudianLongxiang,
    /// [XianjianGuiyuan]（tag 148）。
    XianjianGuiyuan,
    /// [XianjianTannang]（tag 149）。
    XianjianTannang,
    /// [XianjianFeixu]（tag 150）。
    XianjianFeixu,
    /// [XianjianFeng]（tag 151）。
    XianjianFeng,
    /// [XianjianYa]（tag 152）。
    XianjianYa,
    /// [XianjianSong]（tag 153）。
    XianjianSong,
    /// [XianjianXjyufeng]（tag 154）。
    XianjianXjyufeng,
    /// [XianjianXuanyan2]（tag 155）。
    XianjianXuanyan2,
    /// [XianjianXuanyan3]（tag 156）。
    XianjianXuanyan3,
    /// [XianjianNingbin]（tag 157）。
    XianjianNingbin,
    /// [XianjianXuanyan]（tag 158）。
    XianjianXuanyan,
    /// [XianjianDiewu]（tag 159）。
    XianjianDiewu,
    /// [XianjianLingyu]（tag 160）。
    XianjianLingyu,
    /// [WudianLueying]（tag 161）。
    WudianLueying,
    /// [XianjianAnwugu]（tag 162）。
    XianjianAnwugu,
    /// [XianjianAnwugu2]（tag 163）。
    XianjianAnwugu2,
    /// [XianjianTxianqu]（tag 164）。
    XianjianTxianqu,
    /// [XianjianFenshi]（tag 165）。
    XianjianFenshi,
    /// [XianjianLonghuo]（tag 166）。
    XianjianLonghuo,
    /// [XianjianSajin]（tag 167）。
    XianjianSajin,
    /// [XianjianJtjubao]（tag 168）。
    XianjianJtjubao,
    /// [XianjianYuexing]（tag 169）。
    XianjianYuexing,
    /// [XianjianMinsha]（tag 170）。
    XianjianMinsha,
    /// [XianjianLinghuo]（tag 171）。
    XianjianLinghuo,
    /// [XianjianGuijin]（tag 172）。
    XianjianGuijin,
    /// [XianjianZhuyue]（tag 173）。
    XianjianZhuyue,
    /// [XianjianGuanri]（tag 174）。
    XianjianGuanri,
    /// [XianjianShuiyun]（tag 175）。
    XianjianShuiyun,
    /// [XianjianShuiyun5]（tag 176）。
    XianjianShuiyun5,
    /// [XianjianWangyou]（tag 177）。
    XianjianWangyou,
    /// [XianjianWangyou2]（tag 178）。
    XianjianWangyou2,
    /// [XianjianLingquan]（tag 179）。
    XianjianLingquan,
    /// [XianjianShenwu]（tag 180）。
    XianjianShenwu,
    /// [XianjianChengxin]（tag 181）。
    XianjianChengxin,
    /// [XianjianShenmu]（tag 182）。
    XianjianShenmu,
    /// [XianjianDuci]（tag 183）。
    XianjianDuci,
    /// [XianjianRunxin]（tag 184）。
    XianjianRunxin,
    /// [XianjianXjzhimeng]（tag 185）。
    XianjianXjzhimeng,
    /// [XianjianXjzhimeng2]（tag 186）。
    XianjianXjzhimeng2,
    /// [XianjianSheling]（tag 187）。
    XianjianSheling,
    /// [XianjianHushaCount]（tag 188）。
    XianjianHushaCount,
    /// [XianjianHusha]（tag 189）。
    XianjianHusha,
    /// [XianjianChangnian]（tag 190）。
    XianjianChangnian,
    /// [XianjianChangnian2]（tag 191）。
    XianjianChangnian2,
    /// [XianjianShiying]（tag 192）。
    XianjianShiying,
    /// [XianjianZhaoyao]（tag 193）。
    XianjianZhaoyao,
    /// [XianjianLiguang]（tag 194）。
    XianjianLiguang,
    /// [XianjianXuanmo]（tag 195）。
    XianjianXuanmo,
    /// [XianjianSheying]（tag 196）。
    XianjianSheying,
    /// [XianjianHuahu]（tag 197）。
    XianjianHuahu,
    /// [XianjianXtanxi]（tag 198）。
    XianjianXtanxi,
    /// [XianjianFenglue]（tag 199）。
    XianjianFenglue,
    /// [XianjianLiefeng]（tag 200）。
    XianjianLiefeng,
    /// [XianjianTianjian]（tag 201）。
    XianjianTianjian,
    /// [XianjianPoyun]（tag 202）。
    XianjianPoyun,
    /// [XianjianXshuangren]（tag 203）。
    XianjianXshuangren,
    /// [XianjianXshuangren2]（tag 204）。
    XianjianXshuangren2,
    /// [XianjianZhangmu]（tag 205）。
    XianjianZhangmu,
    /// [XianjianQianfang]（tag 206）。
    XianjianQianfang,
    /// [XianjianXunying]（tag 207）。
    XianjianXunying,
    /// [XianjianTianxian]（tag 208）。
    XianjianTianxian,
    /// [XianjianXjyingfeng]（tag 209）。
    XianjianXjyingfeng,
    /// [XianjianXiepan]（tag 210）。
    XianjianXiepan,
    /// [XianjianLuanjian]（tag 211）。
    XianjianLuanjian,
    /// [XianjianLuanjian2]（tag 212）。
    XianjianLuanjian2,
    /// [XianjianCtianfu]（tag 213）。
    XianjianCtianfu,
    /// [XianjianCtianfu2]（tag 214）。
    XianjianCtianfu2,
    /// [XianjianCtianfu3]（tag 215）。
    XianjianCtianfu3,
    /// [SwdZhiluxiaohuCard]（tag 216）。
    SwdZhiluxiaohuCard,
    /// [SwdMujiarenCard]（tag 217）。
    SwdMujiarenCard,
    /// [SwdGouhunluoCard]（tag 218）。
    SwdGouhunluoCard,
    /// [SwdGouhunluoSub]（tag 219）。
    SwdGouhunluoSub,
    /// [SwdJiguantongCard]（tag 220）。
    SwdJiguantongCard,
    /// [SwdJiguanfengCard]（tag 221）。
    SwdJiguanfengCard,
    /// [SwdFengyinState]（tag 222）——机关凤封印状态（上游 fengyin=非锁定
    /// 技失效；登记差异：原生无锁定技区分=全部技能失效，gwent 封印同口径）。
    SwdFengyinState,
    /// [SwdJiguanyuanCard]（tag 223）。
    SwdJiguanyuanCard,
    /// [SwdJiguanyuanGain]（tag 224）——机关怨待获得状态（拥有者结束阶
    /// 段强制获得置牌）。
    SwdJiguanyuanGain,
    /// [SwdJiguanyaoshuCard]（tag 225）。
    SwdJiguanyaoshuCard,
    /// [SwdJiguanyaoState]（tag 226）——机关妖术防重标记（上游
    /// jiguanyaoshu_skill 防重面；其补偿触发面暂缓登记至机关术批）。
    SwdJiguanyaoState,
    /// 旋风之刃（tag 227）：目标随机弃 1 张 he。
    SwdXuanfengzhirenCard,
    /// 中行虎甲（tag 228）：防具发现装配 + 目标随机弃 1 张手牌。
    SwdZhongxinghujiaCard,
    /// 星体改造（tag 229）：使用者摸 1 + 手牌上限 -1 累加（结束阶段重置）。
    SwdXingtigaizaoCard,
    /// 时间回溯（tag 230）：目标获得其装备区全部牌。
    SwdShijianhuisuCard,
    /// 紧急冷冻（tag 231）：目标 +2 护甲、翻面；仍翻面则冻结。
    SwdJinjilengdongCard,
    /// 圣休号角（tag 232）：目标获嘲讽（相邻只能以其为杀目标）。
    SwdShengxiuhaojiaoCard,
    /// 隐秘离场（tag 233）：目标获潜行（不能成为牌目标，本回合）。
    SwdYinmilichangCard,
    /// 机关妖术补偿（tag 234，规则 v198 补齐）：非自己回合失去装备区牌
    /// 时随机补获等量灵剑（弃装备事件近似）。
    SwdJiguanyaoGain,
    /// 机关术（tag 235，规则 v199/ADR 0780）：每张有效装备受控合成——附
    /// 着一张随机灵剑（CardMod 表；登记差异：名称合成/数值升级仅入改
    /// 造表，目录不可变；jiqi/yuchan 池暂缓；演示呈现暂缓）。
    SwdJiguanshuCard,
    /// 天覆阵（tag 236，规则 v200/T19a）：大国 AOE 逐目标强制弃 1。
    SwdTianfuzhenCard,
    /// 七星阵（tag 237，规则 v200/T19a）：围攻者逐个弃 1+视为对其用杀
    ///（响应闪或 1 伤；登记差异：虚拟杀简化，无加成/不可响应额外面）。
    SwdQixingzhenCard,
    /// 鸟翔阵（tag 238，规则 v200/T19a）：逐合法目标响应闪或受 1 伤。
    SwdNiaoxiangzhenCard,
    /// 长蛇阵（tag 239，规则 v201/T19b）：队列中→全队列摸 1；否则与最近
    /// 友方换座成队列（swapSeat）。
    SwdChangshezhenCard,
    /// 破阵诀（tag 240，规则 v201/T19b）：随机座位重排（swapSeat 若干次）。
    SwdPozhenjueCard,
    /// 戏才（tag 241，运筹武将面首切片）：受到伤害后获得造成伤害的牌
    ///（可选触发；登记差异：上游校验伤害牌在处理区，原生按 d.card 有值
    /// 等价；获得路径 table 优先、弃堆兜底）。
    YunchouXicai,
    /// 狂骨·变体（tag 242，运筹武将面首切片）：你对其他角色造成伤害后，
    /// 其到你攻击距离大于 1 摸 1 张，否则你已受伤回复 1 点（强制；登记
    /// 差异：攻击距离按座位基础距离，坐骑修正不参与）。
    YunchouDiykuanggu,
    /// 雄姿（tag 243，运筹武将面批 2）：摸牌阶段开始额外摸 1+⌊装备数/2⌋
    /// 张（强制；固定摸 2 后补摸——阶段技后执行口径）。
    YunchouXiongzi,
    /// 雅量（tag 244，运筹武将面批 2）：你造成或受到伤害后（可选），伤害
    /// 双方各摸 1 张（双方存活且异人；登记差异：摸牌序按座位序逐个）。
    YunchouYaliang,
    /// 纵火（tag 245，运筹武将面批 3）：你造成非火属性伤害时（可选）弃
    /// 1 张手牌将该伤害改为火属性（DamageBefore 帧直写 nature=乱剑增伤
    /// amount 直写同机制；登记差异：上游玩家自选弃牌，原生取首张手牌）。
    YunchouZonghuo,
    /// 烧营（tag 246，运筹武将面批 3）：你造成火属性伤害后（可选），弃
    /// 牌堆顶牌亮出——红色则你与受伤者距离 1 以内的一名其他角色受 1 点
    /// 火属性伤害（登记差异：上游玩家选目标，原生取座位序首合法目标；
    /// 链式火伤再触发与上游一致不挡）。
    YunchouShaoying,
    /// 蛊惑·变体（tag 247，运筹武将面批 4）：回合开始摸 2 然后弃自己区
    /// 域 2 张牌（强制；登记差异：上游 AI 择弃优先判定区，原生确定性
    /// 判定区→装备→手牌首张序）。
    YunchouDiyguhuo,
    /// 狡黠（tag 248，运筹武将面批 4）：成为红色牌的目标时摸 1（frequent
    /// 自动触发，无确认门）。
    YunchouJiaoxia,
    /// 红莲（tag 249，运筹武将面批 4）：受到来源为其他角色的伤害后（可
    /// 选），来源弃其手牌与装备区的全部红色牌。
    YunchouHonglian,
    /// 厉焰（tag 250，运筹武将面批 6）：目录锚（非触发面）——转化逻辑在
    /// PlayCard 门（非基本手牌当水攻，国色先例同位）。
    YunchouLuweiyan,
    /// 厉焰·链杀（tag 251，运筹武将面批 6）：持有者以水攻用牌后（可选
    /// 确认），对座位序首目标视为用杀（登记差异：上游限「厉焰转化的水
    /// 攻」且玩家选目标，原生按持有者全部水攻+自动首目标）。
    YunchouLuweiyan2,
    /// 隙焰（tag 252，运筹武将面批 7）：出牌主动技——弃 1 张红手牌，全
    /// 体其他存活角色各受 1 点火属性伤害（座位序逐个；登记差异：上游玩
    /// 家选弃哪张红牌，原生取首张红手牌；上游 multiline 呈现面不做）。
    YunchouJieyan,
    /// 野升（tag 253，运筹武将面批 7）：出牌主动技——弃任意张黑手牌并
    /// 摸等量的牌（自选多张，舒筋散机器同构）。
    YunchouYiesheng,
    /// 蝉缘·变体（tag 254，运筹武将面批 9）：死亡时来源体力上限 -1
    ///（强制；max_hp 减面承载——来源不存在则不触发）。
    YunchouDiychanyuan,
    /// 屯田·DIY（tag 255，玩点论杀续批）：出牌主动技——重铸一张装备牌：
    /// 弃之并从牌堆随机获得一张装备牌（实体发现模式，v187 先例；登记
    /// 差异：上游 canRecast 的换置面不做）。
    WudianDiyjuntun,
    /// 禁言（tag 256，玩点论杀续批）：目录锚+查询面——体力 ≤2 时黑色
    /// 锦囊禁用（PlayCard 门查询，往事封禁先例；登记差异：上游四面
    /// cardEnabled/Usable/Respondable/Savable，原生仅使用面）。
    WudianJinyan,
    /// 理断（tag 257，玩点论杀批 12）：他人回合外获得一张装备牌到手后
    /// （可选确认）：同栏已有装备则其交 1 张手牌（首张，登记差异：上游
    /// 玩家自选），否则直接装备该牌（不走 Equipped 帧防双 grant）。
    WudianLiduan,
    /// 坚壁（tag 258，玩点论杀批 13）：锁定查询面——AOE（南蛮/万箭）对
    /// 持有者无效（目标排除原语首例，帷幕 weimu_blocks_aoe 同构；登记
    /// 差异：上游可选确认且覆盖全部多目标锦囊，原生锁定仅 AOE 面）。
    WudianJianbi,
    /// 复仇（tag 259，玩点论杀批 14）：目录锚——取消逻辑在
    /// ShaResolveMachine 内联（杀目标响应窗前选择阶段）。
    WudianFuchou,
    /// 复仇·清算（tag 260，玩点论杀批 14）：自己回合结束时对复仇标记的
    /// 来源逐个结算——摸 1 张并以虚拟杀强制指定（登记差异：上游
    /// chooseToUse 玩家选杀实体且可逐个取消，原生虚拟杀直发；死亡标记
    /// 一并移除）。
    WudianFuchou2,
    /// 良策·余获（tag 261，玩点论杀批 15）：锁定查询面——五谷丰登结算
    /// 后剩余牌归持有者（wuguRemained 子事件承载，登记差异：上游玩家
    /// 选目标，原生归持有者本人；viewAs 转化面在 PlayCard 门）。
    WudianLiangceRemain,
    /// 归汉（tag 262，玩点论杀批 17）：目录锚——limited 濒死自救逻辑在
    /// DyingSaveMachine 内联（确认+回摸，性别目标面内联）。
    WudianGuihan,
    /// 悲歌（tag 263，运筹批 18，refresh 变体）：受到杀伤害后（可选确
    /// 认）——受伤者判定，持有者弃 1 张牌（可取消）按判定花色执行：
    /// 红桃受伤者回 1/方块受伤者摸 2/黑桃来源翻面/草花来源弃 2（强制，
    /// 登记差异：上游玩家自选）；收回奖励：弃牌花色=判定花色收回判定
    /// 牌、点数=判定点数收回所弃牌。
    YunchouBeige,
    /// 铁壁（tag 264，swd 武将面批 18）：成为杀目标且与持有者距离 ≤1
    /// 时弃 1 张黑牌给目标 +1 护甲（CardUsed 帧等价承载，登记差异：上游
    /// target shaBefore 玩家自选弃牌，原生首张黑牌确定性）。
    SwdTiebi,
    /// 大同（tag 265，swd 武将面批 18）：任意角色任意阶段结束全场手牌
    /// 极差 ≤1 则自己摸 2（frequent 自动无门）。
    SwdDatong,
    /// 承剑（tag 266，swd 武将面批 19）：其他角色受到杀伤害后（可选确
    /// 认）来源摸 1。
    SwdChengjian,
    /// 缚斧（tag 267，swd 武将面批 19）：你对目标使用杀伤害 +1（强制，
    /// 需装备武器）。
    SwdJufu,
    /// 花镜（tag 268，swd 武将面批 19）：你造成的锦囊牌伤害后回 1 摸 1
    ///（frequent 自动）。
    SwdHuajing,
    /// 护盾（tag 269，swd 武将面批 19）：你造成伤害后自己无护甲则 +1
    /// 护甲（强制）。
    SwdHudun,
    /// 破星（tag 270，swd 武将面批 19）：你对体力高于你的角色造成伤害
    /// 时伤害 +1（强制）。
    SwdPoxing,
    /// 分舵（tag 271，swd 武将面批 20，习嫣 jiefen）：出牌主动技（每回
    /// 合限一次）——手牌多于自己的角色交 1 张手牌给自己；然后自己交
    /// 1 张手牌给手牌数更少的其他角色（登记差异：上游 chooseCardTarget
    /// 玩家同时选牌与目标，原生两段询问+确定性目标）。
    SwdJiefen,
    /// 流虹（tag 272，swd 武将面批 21，拓跋月儿）：使用杀后摸 1
    ///（frequent 自动）。
    SwdLiuhong,
    /// 墨剑（tag 273，swd 武将面批 21，水镜/曲贤）：成为杀目标时（可选
    /// 确认）目标摸 1、自己回 1（未满血门；登记差异：上游 check 为 AI
    /// 决策面）。
    SwdMojian,
    /// 道法（tag 274，swd 武将面批 21，壶中仙）：其他角色受到伤害后（可
    /// 选确认）来源弃 1 张牌（登记差异：上游 chooseToDiscard 玩家自选，
    /// 原生首张确定性）。
    SwdDaofa,
    /// 玉鳞（tag 275，批 22，兰茵 yulin）：受伤前置窗口（可选）弃 1 张
    /// 装备牌（手/装备区）免此伤害。
    SwdYulin,
    /// 凝霜（tag 276，批 22，芝茵 ningshuang）：成为黑色牌目标（可选）
    /// 弃 1 张黑色牌，对使用者阶梯：已翻面→失 1 体力；已横置→翻面；
    /// 否则→横置（仅横置支自己摸 1，上游代码原样；登记差异：上游
    /// useCardToBegin 帧等价接入 CardUsed）。
    SwdNingshuang,
    /// 工神（tag 277，批 22，黄雷 gongshen）：其他角色使用基本/锦囊牌
    /// 且有目标（可选）弃 1 张装备牌令其失效（cancelled 写回，筑城同款）。
    SwdGongshen,
    /// 平恤（tag 278，批 23，秦叔宝 pingxu）：无触发面——全局距离字段技
    /// （computed_distance 消费：场上有持技者时，攻击方无武器距离 -1、
    /// 防守方无防具距离 +1；上游 mod globalFrom/globalTo 同口径）。
    SwdPingxu,
    /// 还灵（tag 279，批 23，轩辕剑侍 huanling）：自己阶段结束（可选）
    /// 选一名其他角色，自己与其同时翻面（上游 phaseEnd chooseTarget
    /// turnOver×2 同口径；原生 Phase begin:false 帧等价接入）。
    SwdHuanling,
    /// 断月（tag 280，批 24，水镜 duanyue）：出牌阶段限一次主动技——弃
    /// 1 张装备牌（手/装备区）对一名其他角色造成 1 点伤害（两段机）。
    SwdDuanyue,
    /// 屠阵（tag 281，批 24，水镜 tuzhen）：自己造成伤害结算后（可选）
    /// 弃置受伤者手牌中的全部非基本牌（确定性整组）。
    SwdTuzhen,
    /// 落梅（tag 282，批 24，兰茵 luomei）：使用或打出梅花牌（频率技）
    /// 摸 1——CardUsed+CardResponded 双事件（飞虚先例）。
    SwdLuomei,
    /// 星点（tag 283，批 24，兰茵 xingdian）：出牌阶段限一次主动技——
    /// 弃 1 张手牌，确定性弃两名其他存活角色各手/装备区首张（登记差异：
    /// 上游随机 2 名敌人各随机弃 1）。
    SwdXingdian,
    /// 协律（tag 284，批 25，壶中仙 xielv）：弃牌阶段结束（End 帧入场）
    /// 手牌≥2 且同色（可选）回复 1 点体力并弃置场上全部判定牌。
    SwdXielv,
    /// 朱羽（tag 285，批 25，芝茵 zhuyu）：横置角色即将受到非传导伤害
    /// （可选）弃 1 张红色牌，伤害 +1 且改为火属性。
    SwdZhuyu,
    /// 雪煌（tag 286，批 26，芝茵 xuehuang）：limited 每局一次主动技——
    /// 手牌全红时发动：弃全部手牌，逐张视为对（至多）两名其他存活角色
    /// 使用火杀（use_virtual_card；登记差异上游随机 2 名敌人）。
    SwdXuehuang,
    /// 还侠（tag 287，批 27，屈娴 huanxia）：目录标记技——viewAs 红牌当
    /// 杀（engine PlayCard 转化臂）+TurnState 跟踪未造成伤害的转化杀，
    /// 持有者阶段末从弃牌堆收回（kernel 钩；登记差异上游经特殊区暂存）。
    SwdHuanxia,
    /// 破月（tag 288，批 28，拓跋玉儿 poyue）：目录标记技——黑杀无视距
    /// 离、红杀不计入出牌阶段限一次、红杀不可被闪避（poyue2 group 子技
    /// 同承载；PlayCard 臂+kernel 计数豁免+闪询问过滤三钩；niepan=外部
    /// 包技依赖登记不落）。
    SwdPoyue,
    /// 收纳（tag 289，批③，炼妖壶 equipSkill）：出牌限一次——弃 1 手牌
    /// 随机获目标 1 手牌移入壶仓（确定性首张；登记差异上游随机）。
    SwdLianyaohuShouna,
    /// 炼化（tag 290，批③，炼妖壶 equipSkill）：出牌限一次——壶仓 ≥2
    /// 弃 2 张，从牌堆顶获得第一张类别均不同的牌（上游牌堆序扫描同口径）。
    SwdLianyaohuLianhua,
    /// 壶境（tag 291，批③，壶中仙 hujing）：锁定——准备阶段弃牌堆有炼
    /// 妖壶则装备之；装备时手牌上限 +2（kernel 手牌上限钩+弃堆回收钩）。
    SwdHujing,
    /// 神农鼎（tag 292，批④ equipSkill）：出牌限一次弃 2 张手牌回复 1
    /// （受伤时；ActivateSkill 装备技实例通道）。
    SwdShennongdingCard,
    /// 女娲石（tag 293，批④ equipSkill）：其他角色濒死（其 hp≤0）且自
    /// 己 hp>1：目标回复 1、自己失去 1（Dying 触发确认门）。
    SwdNvwashiGuard,
    /// 昆仑镜（tag 294，批⑤ equipSkill）：出牌限一次亮牌堆顶 3 张选 1
    /// 获得、以 1 张手牌替换（三段机）。
    SwdKunlunjingcCard,
    /// 东皇钟（tag 295，批⑤ equipSkill）：自己阶段结束弃 1 张红牌，从
    /// 牌堆发现延时锦囊实体置入目标判定区（实体发现替代 createCard）。
    SwdDonghuangzhongCard,
    /// 空洞印（tag 296，批⑥ equipSkill）：锁定——死亡前置窗口取消死亡：
    /// hp=1、摸 1、弃全部宝物（death_cancelled 基建消费）。
    SwdKongdongyinGuard,
    /// 逆攻积攒（tag 297，批⑧龟灵之桃 equipSkill）：受伤后标记
    /// +amount（上限 4，forced）。
    SwdNigongAccumulate,
    /// 逆攻发动（tag 298，批⑧龟灵之桃 equipSkill）：出牌弃全部标记——
    /// 奇数摸 1+对一名其他角色造成 floor(标记/2) 伤；标记=1 时仅摸 1。
    SwdNigongStrike,
    /// 四非珍面（tag 303，批⑨ equipSkill）：出牌限一次令其他有手牌角色
    /// 判定，非红桃则其用首张手牌指向座位序首个其他存活者（登记差异
    /// 上游随机牌/随机目标）。
    SwdSifeizhenmianCard,
    /// 青龙之圭（tag 304，批⑩ 器族手持技）：准备阶段摸 2 弃 1。
    SwdQinglongzhiguiHeld,
    /// 朱雀之璋（tag 305，批⑩ 器族手持技）：受他人伤害后来源雷 1 伤
    ///（上游 fire，native 火属性同壳）。
    SwdZhuquezhizhangHeld,
    /// 白兽之琥（tag 306，批⑩ 器族手持技）：自己弃牌阶段结束弃置一名
    /// 其他角色一张牌（确定性首张）。
    SwdBaishouzhihuHeld,
    /// 玄武之璜（tag 307，批⑩ 器族手持技）：自己造成伤害后回复等量体
    /// 力（hp<max 门）。
    SwdXuanwuzhihuangHeld,
    /// 黄麟之琮（tag 308，批⑩ 器族手持技）：准备阶段无护甲则 +1 护甲。
    SwdHuanglinzhicongHeld,
    /// 神草结（tag 309，批⑬ trick 手持交互）：锦囊伤害前——受伤者是自
    /// 己则伤-1、来源是自己则伤+1（弃一张神草结；确认门）。
    SwdShencaojieGuard,
    /// 疾影（tag 312，批㉑ 赛特）：锁定 sha 无视距离（marker，sha_range_
    /// free 链 skill 查询）。
    SwdJiyingHeld,
    /// 沐风（tag 313，批㉑ 桓远之）：结束阶段本回合用过基本牌则摸 1
    ///（discoverCard 确定性=牌堆末摸 1）。
    SwdMufengHeld,
    /// 望尘（tag 314，批㉑ 古月圣）：弃牌阶段结束弃置过基本牌则令一名角
    /// 色翻面（End 帧入场确认门）。
    SwdWangchenHeld,
    /// 玉蝉换形（tag 310，批⑮ equipSkill）：自己用牌结算后，手牌中玉蝉
    /// 实体轮换为八卦序下一 kind 的实体（确定性发现；登记差异上游随机）。
    SwdYuchanSwap,
    /// 神秘果窗口（tag 315，卡牌面窗口机批）：出牌阶段用出基本/普通锦
    /// 囊结算后（手持神秘果、每阶段限一次）确认令该牌再结算一次（原体
    /// 自弃堆复用登记差异上游凭空复制；view-as 转化用牌收窄不可再结算）。
    SwdShenmiguoWindow,
    /// 鼠槌主动技（tag 316，equip5）：出牌阶段限一次——指定攻击范围内
    /// 角色依次自动用出手中至多 3 张杀（凤略 Yield 循环同型）。
    SwdShuchuiUse,
    /// 连舞（tag 317，武将面批㉒ 尚章）：锁定 marker——杀目标上限 +1
    ///（engine max_targets 链查询）+红杀不可闪避（kernel 破月 directHit
    /// 臂同位查询）。
    SwdLianwu,
    /// 浴尘（tag 318，武将面批㉒ 游兆）：回合外使用/打出黑色牌后弃置一
    /// 名角色一张牌（确认门+目标选择；随机代弃登记差异上游自选）。
    SwdYuchen,
    /// 挟雷（tag 319，武将面批㉒ 宇文拓）：使用/打出杀后弃一张牌并对目
    /// 标以外一名角色 1 点雷伤（双事件；>2 人局）。
    SwdXielei,
    /// 对诗（tag 320，武将面批㉒ 王思月）：出牌阶段弃一张手牌指定角色：
    /// 弃同花色手牌或令你获得其一张牌（usable:1 收敛上游 duishi2/3
    /// 临时技面）。
    SwdDuishi,
    /// 连打（tag 321，武将面批㉔ 敷衍）：杀结算完毕后可弃一张牌视为对
    /// 目标再使用一张杀（虚拟杀；每回合限一次=TurnState.lianda_used）。
    SwdLianda,
    /// 隳延（tag 322，武将面批㉔ 荣霜）：造成伤害后可弃一张牌弃置对方
    /// 全部装备牌。
    SwdSuiyan,
    /// 断行（tag 323，武将面批㉔ 荣霜）：装备牌入装后可视为对杀合法目
    /// 标使用一张虚拟杀。
    SwdDuanxing,
    /// 破浪（tag 324，武将面批㉔ 慕云）：造成伤害后可弃置对方一张装备
    /// 牌（自选面保留——装备区公开信息）。
    SwdPolang,
    /// 诱引（tag 325，武将面批㉔ 陈靖仇）：其他角色弃置过含非基本牌的
    /// 牌且手牌<5 → 摸 1（frequent 锁定式；移花 defer）。
    SwdYouyin,
    /// 湛卢（tag 326，武将面批㉕ 屠维）：出牌阶段限一次弃一张黑桃牌令
    /// 至多 3 名角色各回复 1 点体力。
    SwdZhanlu,
    /// 苏生（tag 327，武将面批㉕ 屠维）：任意角色死亡前置窗口——可弃一
    /// 张手牌防止其死亡并体力回至 1（每回合限一次走 limits；空洞印取消
    /// 基建同款）。
    SwdSusheng,
    /// 相惠（tag 328，武将面批㉕ 夏柔）：出牌阶段限一次弃一张红色手牌
    /// 令场上体力最少且已受伤角色各回复 1 点体力。
    SwdXianghui,
    /// 挥琪（tag 329，武将面批㉕ 夏柔）：受伤后可令一名其他角色摸 X 张
    ///（X=自己已损失体力值）。
    SwdHuiqi,
    /// 万钧（tag 330，武将面批㉖ 浊山铸）：锁定 marker——装备牌当南蛮
    /// 入侵使用（engine PlayCard viewAs 链查询）。
    SwdWanjun,
    /// 森罗（tag 331，武将面批㉖ 莫耶）：出牌阶段限一次且手牌数为全场
    /// 最少——其他有手牌角色各弃 2 摸 1（弃牌确定性首两张登记差异）。
    SwdSenluo,
    /// 火扑（tag 332，武将面批㉖ 夜鸦熙）：锁定 marker——红桃牌当流星
    /// 火雨使用（engine PlayCard viewAs 链查询）。
    SwdHuopu,
    /// 夜影（tag 333，武将面批㉗ 小黄龙）：锁定 marker——黑色牌当乾坤镖
    /// 使用（engine PlayCard viewAs 链查询）。
    SwdYeying,
    /// 归影（tag 334，武将面批㉗ 虎魄）：锁定 marker——黑色牌当偷梁换柱
    /// 使用（engine PlayCard viewAs 链查询）。
    SwdGuiying,
    /// 凤泽（tag 335，武将面批㉗ 司徒强）：锁定 marker——黑色牌当桃园结
    /// 义使用（engine PlayCard viewAs 链查询）。
    SwdFengze,
    /// 玲月（tag 336，武将面批㉗ 司徒强）：锁定——杀指定目标后目标弃置
    /// 一张牌（确定性首张登记差异上游自选；锦鳞 limited defer）。
    SwdLingyue,
    /// 暗莲映（tag 337，武将面批㉗ 暗卡）：手牌被弃置空后摸 2（frequent
    /// 锁定式；登记差异上游 loseEnd 全丢失面≈弃置面；颂词 defer）。
    SwdAnlianying,
    /// 疾风（tag 338，武将面批㉘ 卢承轩）：锁定 marker——杀目标上限 +X、
    /// 与他人距离 -X（X=已损失体力；engine max_targets 链+state
    /// computed_distance 链查询）。
    SwdLjifeng,
    /// 断义（tag 339，武将面批㉘ 慕容诗）：出牌阶段限一次弃两张杀——对
    /// 一名角色造成 1 点伤害并其随机弃 X 张 he（X=其已损失体力；孤兴
    /// defer）。
    SwdDuanyi,
    /// 乐跃（tag 340，武将面批㉘ 小雪）：任意角色回复体力后（确认门）——
    /// 回复者非自己且其手牌≥自己则各摸 1，否则回复者摸 1（唤魂/待行
    /// defer）。
    SwdYinyue,
    /// 灵心（tag 341，武将面批㉙ 玉澧）：阶段结束 frequent——观看牌堆顶
    /// 三张，红桃收入手牌其余弃置（天香非原生 defer）。
    SwdLingxin,
    /// 渔人（tag 342，武将面喀玛批㉙）：锁定——杀指定目标后判定：黑→弃
    /// 目标 he 随机一张；红→摸 1（登记差异：上游收回此杀实体，原生实体
    /// 回流与在结机器持有冲突改摸 1；每杀一次=!asked 门）。
    SwdYueren,
    /// 幻剑（tag 343，武将面批㉚ 妮可）：锁定 marker——黑色牌当冰魄针使
    /// 用（engine PlayCard viewAs 链查询；灵武额外阶段=引擎缺口 defer）。
    SwdHuanjian,
    /// 虚影（tag 344，武将面批㉚ 淳于衡）：锁定——伤害前置窗口取消伤害，
    /// 手牌非空则失去 1 点体力（DamageBefore 帧写回；绝情 builtin 白名
    /// 单引用；笙笙 NOT FOUND defer）。
    SwdXuying,
    /// 乖戾（tag 345，武将面批㉙ 子巧）：锁定双事件——杀伤害 +1（非铁索
    /// 传导）+ 造成杀伤害后弃两张手牌（敷衍=受伤后令无护甲角色 +1 护甲
    /// 同将第二技独立 tag 348）。
    SwdGuaili,
    /// 无邪（tag 346，武将面批㉙ 莉莲）：锁定 marker——延时锦囊只能以自
    /// 己为目标（engine PlayCard 延时校验臂查询；青城判定循环 defer）。
    SwdWuxie,
    /// 观虎（tag 347，武将面批㉙ 吉挺）：杀造成伤害后可弃目标一张手牌和
    /// 一张装备牌（确定性首张登记差异上游自选；灵视多面 defer）。
    SwdGuanhu,
    /// 敷衍·护甲面（tag 348，武将面批㉙ 子巧）：受伤后可令一名无护甲角色
    /// 获得 1 点护甲（确认门）。
    SwdFuyan,
    /// 凌波·恒爱面（tag 349，武将面批㉜ 恒爱）：闪响应/使用后 frequent 摸
    /// 2（双事件 CardResponded+CardUsed；funiao/ningxian defer）。
    SwdHlingbo,
    /// 血裔·Miles 面（tag 350，武将面批㉜ Miles）：锁定——伤害前置取消+
    /// 失去 1 点体力（无条件版虚影；aojian/mohua2 defer）。
    SwdMilesXueyi,
    /// 鸣凤（tag 351，武将面批㉝ 胡月）：出牌阶段限一次弃一张装备牌——
    /// 一名角色回复 1 并摸 1（hyunshen 计数存储 defer）。
    SwdFengming,
    /// 小莫雨·山小小面（tag 352，武将面批㉝ 山小小）：锁定——造成伤害后
    /// 回复 1（满血改摸 1），每回合一次（inst.once_used；shehun defer）。
    SwdXiaomoyu,
    /// 冰箭（tag 353，武将面批㉞ 海大胖）：出牌阶段限一次弃黑色杀——目标
    /// 闪全弃或受 1 雷伤（rumeng 阶段跳过 defer）。
    SwdBingjian,
    /// 神鼎·麦星面（tag 354，武将面批㉞ 麦星）：出牌阶段限一次判定——红桃
    /// 回 1/方块摸 1/梅花随机敌弃 1/黑桃无（toudan defer）。
    SwdShending,
    /// 鬼眼（tag 355，武将面批㉞ 冯天凌）：出牌阶段限一次观看目标手牌取其
    /// 一张梅花（guiyan2 濒死复活 defer）。
    SwdGuiyan,
    /// 袭西·凤钰面（tag 356，武将面批㉟ 凤钰）：出牌阶段开始时弃一张牌——
    /// 攻击距离内且体力不小于自己的角色受 1 伤（fzhenwei defer）。
    SwdShangxi,
    /// 花间·剑仙面（tag 357，武将面批㉟ 轩辕剑仙）：出牌阶段结束时弃一张
    /// 牌视为对杀合法目标使用一张虚拟杀（pozhou/xuanyuan defer）。
    SwdHuajian,
    /// 星坠·慕月面（tag 358，武将面批㉟ 慕月）：出牌阶段限一次弃一张牌——
    /// 目标弃同类别牌或受 1 伤（lingxian/shouyin defer）。
    SwdXingzhui,
    /// 焰羽·少微面（tag 359，武将面批㊱ 少微）：出牌阶段限一次弃两张红色
    /// 牌视为使用炽羽袭（jianji 联动 defer）。
    SwdHuangyu,
    /// 摄引（tag 360，武将面批㊱ 伽兰多）：出牌阶段——依次弃尽判定区有牌
    /// 者的判定牌并令其交一张手牌（mailun 轮盘 defer）。
    SwdXianyin,
    /// 破茧·齐面（tag 361，武将面批㊱ 齐）：手牌空后弃置——从牌堆/弃堆寻
    /// 首张装备牌装备之（yaotong/heihuo defer）。
    SwdPojian,
    /// 横刀·慧彦面（tag 362，武将面批㊲ 慧彦）：回合外用牌/响应结算后——
    /// 当前回合角色弃同花色牌或令自己获得其一张牌（hxunzhi defer）。
    SwdHwendao,
    /// 凌锋（tag 363，武将面批㊲ 慧彦）：阶段结束 frequent 出牌数达标——
    /// 攻击距离内目标受 1 伤或取消自获 1 护甲。
    SwdLingfeng,
    /// 天火·秦壬面（tag 364，武将面批㊲ 秦壬）：出牌阶段限一次——判定区
    /// 有牌的其他角色各弃判定牌并受等量火伤（huanyin/xuanzhou defer）。
    SwdTianhuo,
    /// 运筹·姬亮面（tag 365，武将面批㊳ 姬亮）：出牌阶段限一次弃一张手牌
    /// ——目标随机弃一张手牌，同色自己摸 1 否则目标摸 1（gongxin/jqimou
    /// defer）。
    SwdYunchou,
    /// 破晓·张烈面（tag 366，武将面批㊞ 张烈）：锁定 marker 三查询——装备
    /// 牌当杀 viewAs 链+无武器距离 -1（computed_distance 链）+无武器杀目
    /// 标上限 +1（max_targets 链）（huajin 授技面 defer）。
    SwdPoxiao,
    /// 疾风·司空宇面（tag 367，武将面批㊟ 司空宇）：锁定 marker——体力值
    /// 不大于自己的角色视为在攻击范围（engine 杀距离校验臂查询；
    /// linyun 两牌当杀/hutian 护天 defer）。
    SwdSliufeng,
    /// 乱击·多鹏面（tag 368，武将面批 81 多鹏）：出牌将两张同花色手牌
    /// 视为使用【万箭齐发】（ActivateSkill 双段选牌机 + use_virtual_card
    /// 完整 AOE 管线；上游 viewAs 双卡复杂选牌面同口径）。
    SwdLuanji,
    /// 聒噪·疾鹏面（tag 369，武将面批 81 疾鹏）：锁定——距离 1 以内的
    /// 角色受到伤害结算后（伤害来源非自己），观看牌堆顶三张并令距离 2
    /// 以内一名有手牌角色以手牌与其交换（登记差异：上游不足三张时先摸
    /// 三张再弃置的补牌面收敛为洗弃堆）。
    SwdGuozao,
    /// 英姿·刷新（tag 370，武将面批 81 疾鹏/多鹏 reyingzi）：锁定——
    /// 摸牌阶段额外摸一张牌，且手牌上限为体力上限（上游 refresh
    /// reyingzi 双面；标准目录「英姿」只有摸牌面，故独立行为承载）。
    SwdReyingzi,
    /// 愚钝·福面（tag 371，武将面批 82 福）：锁定——无法使用锦囊牌；
    /// 可将两张锦囊手牌当一张不计入出杀次数的【杀】使用（上游 yudun
    /// mod cardEnabled/cardRespondable/cardSavable 全禁锦囊 +
    /// viewAs sha + useCard 后 stat 回退同口径）。
    SwdYudun,
    /// 占星·肯迪面（tag 372，武将面批 82 肯迪）：出牌阶段限一次——弃
    /// 任意张手牌，亮出等量牌堆顶牌：方块摸 2/红桃受伤回 1 否则 +1 护
    /// 甲/梅花各敌人随机弃 1/黑桃令一名角色受 1 无来源雷伤（上游
    /// zhanxing 同口径；敌对关系按阵营 friend 判定取反）。
    SwdZhanxing,
    /// 乐学·久悠面（tag 373，武将面批 83 久悠）：锁定——自己回合开始/
    /// 结束随机更换临时技能：回合内面授 {制衡/集智}（上游池含缔盟/驱虎
    /// ——原生未实现=登记差异，池取已实现子集）、回合外面授 {遗计/急救/
    /// 鬼道/反馈}（上游 lexue1/lexue2 同口径；授前清四候选）。
    SwdLexue,
    /// 枕流·临月面（tag 374，武将面批 83 临月）：出牌阶段限一次——将一
    /// 张红色手牌置于一名其他角色的武将牌上（其 off_court 挂附、来源
    /// 牌通道），其下一个准备阶段回复 1 并获得此牌（上游 zhenjiu/
    /// zhenjiu2 同口径；触发面=独立 tag 375）。
    SwdZhenjiu,
    /// 枕流·临月触发面（tag 375）：自己准备阶段开始时——off_court 红牌
    /// 入手并回复 1（登记差异：经花色过滤取牌，目标若有其他红色
    /// off_court 牌可能误取）。
    SwdZhenjiu2,
    /// 麻醉·临月面（tag 376，武将面批 83 临月）：出牌阶段限一次——将一
    /// 张黑色手牌置于一名其他角色的武将牌上（off_court 挂附），其造成
    /// 的下一次伤害 -1 然后获得此牌（上游 lmazui/lmazui2/lmazui3 同口
    /// 径；减伤触发面=独立 tag 377）。
    SwdLmazui,
    /// 麻醉·临月触发面（tag 377）：伤害前置——off_court 黑牌返还（登记
    /// 差异：上游伤害结算后归还，原生伤前归还）并令本次伤害 -1。
    SwdLmazui2,
    /// 激昂·凤天凌面（tag 378，武将面批 84 凤天凌）：frequent——成为
    /// 【决斗】或红色【杀】的目标、或使用红色【杀】/【决斗】指定唯一
    /// 目标时摸 1（上游 jiang useCardToPlayered/useCardToTargeted 双面
    /// 同口径；登记差异：原生按 CardUsed+targets 单事件面表达）。
    SwdJiang,
    /// 圣手·淳于恒面（tag 379，武将面批 84 淳于恒）：锁定 marker——
    /// 黑色手牌当草药使用（engine PlayCard viewAs 链查询）。
    SwdShengshou,
    /// 活弹·舒莞儿面（tag 380，武将面批 85 舒莞儿）：出牌阶段限一次——
    /// 弃一张红色手牌并失去 1 点体力：一名目标受 2 点火伤或两名目标各
    /// 受 1 点火伤（上游 huodan selectTarget [1,2] 同口径）。
    SwdHuodan,
    /// 义绝（tag 381，武将面批 86 疆梧）：出牌阶段限一次——与有手牌的
    /// 其他角色拼点：赢=其本回合内基本/锦囊用牌失效（封印）且受 1 伤；
    /// 输=其若已受伤回复 1（上游 yijue 同口径；登记差异：赢面附加伤害
    /// 与封印合并压一窗）。
    SwdYijue,
    /// 荡平（tag 382，武将面批 86 疆梧）：造成伤害后（每回合限一次）——
    /// 可弃一张 he → 受害者的邻近（距离 ≤1）另一名角色受 1 伤（上游
    /// dangping 弃牌+目标窗同口径；addTempSkill dangping2 换回合等价
    /// once_per_turn）。
    SwdDangping,
    /// 幻形（tag 383，武将面批 87 尉迟嫣红）：准备阶段——弃一张 he 并
    /// 指定一名男性其他角色：获得其（非 unique 且自己未持有）武将技
    /// 实例，自己受伤后移除（上游 huanxing/huanxing2 同口径；登记差异：
    /// 上游复制标记面收敛为实例授予）。
    SwdHuanxing,
    /// 幻形·移除面（tag 384）：自己受伤结算后——移除幻形授予的技能实例
    ///（上游 huanxing2 damageAfter 同口径；惑乱 meihuo=装备离场事件缺口
    /// 登记 defer）。
    SwdHuanxing2,
    /// 命符·姜子牙面（tag 385，武将面批 88 姜子牙）：出牌阶段限一次——
    /// 弃一张梅花手牌并令一名角色获得【鬼幽结】延时牌（实体发现置其判
    /// 定区；上游 mingfu prepare throw + addJudge guiyoujie 同口径；
    /// 天命 tianlun=判定改判窗缺口登记 defer）。
    SwdMingfu,
    /// 仙匠·车芸面（tag 386，武将面批 89 车芸）：使用牌指定唯一其他目
    /// 标时（每回合每目标一次）——可选其装备区一张自己未持有的同类牌，
    /// 实体发现复制并装备（上游 cyxianjiang choosePlayerCard+createCard
    /// equip 同口径；登记差异：实体发现替代 createCard；巧械 cyqiaoxie
    /// 挂后续批）。
    SwdCyxianjiang,
    /// 旋影·端蒙面（tag 387）：父技授予 sha/use，不授予 shan/damage。
    /// 原生保留未横置时的主动虚拟杀；loseEnd-use 的完整时机仍待接线。
    SwdXuanying,
    /// 玄影伤害子技（`xuanying_damage`）：父技 group 的子定义，只在父技授予时可达。
    SwdXuanyingDamage,
    /// 烈刃（tag 388，武将面批 90 端蒙）：杀造成伤害后（每触发与目标
    /// 拼点，赢则获得其一张 he；上游 lieren chooseToCompare+gainPlayerCard
    /// 同口径）。
    SwdLieren,
    /// 极谋·姬良面（tag 389，武将面批 91 姬良）：回合外受伤结算后
    /// frequent——摸 1（上游 jqimou damageEnd + 回合外门 + draw 同口径；
    /// 使用牌续段=主动选择面登记差异不收）。
    SwdJqimou,
    /// 剑击·少微面（tag 390，武将面批 91 少微）：出牌阶段限一次——弃一
    /// 张装备牌视为对杀合法目标使用一张虚拟杀并摸 1（上游 jianji
    /// discard equip + useCard sha + draw 同口径；unequip 附加技面登记
    /// 差异不收）。
    SwdJianji,
    /// 头弹·麦星面（tag 391，武将面批 91 麦星）：出牌阶段限一次——弃一
    /// 张黑桃 he，目标受 1 火伤；随后自受害者起（含自己）座位序各弃一
    /// 张 he（上游 toudan 链式弃置同口径；登记差异：机器直发确定性弃
    /// 首张）。
    SwdToudan,
    /// 黑火（tag 392，deferred 消解批 齐）：出牌阶段弃一张装备牌——手牌
    /// 数加倍；若结果 ≥8 则自受 3 点火伤（每回合限一次走 limits）。
    SwdHeihuo,
    /// 真威（tag 393，deferred 消解批 凤钰）：自己回合内其他角色打出的
    /// 牌（响应入弃堆）→ 可交给另一名角色（确认门+目标选择）。
    SwdFzhenwei,
    /// 遁行（tag 394，deferred 消解批 虎魄）：成为其他角色卡牌目标时——
    /// 可弃一张牌判定，非红桃则取消该牌（幻音取消写回同款）。
    SwdDunxing,
    /// 孵鸟（tag 395，deferred 消解批 恒爱）：出牌阶段限一次将一张手牌
    /// 交给一名其他角色然后摸 1（观看手牌=展示面登记差异无隐藏信息）。
    SwdFuniao,
    /// 凝弦（tag 396，deferred 消解批 恒爱）：受伤后可弃任意张黑色牌并
    /// 选等量其他角色各受 1 伤。
    SwdNingxian,
    /// 妖童（tag 397，deferred 消解批 齐）：锁定 marker——手牌数为奇数时
    /// 任意手牌当杀使用（engine viewAs 链查询；闪响应面/无懈偶数面
    /// 登记 defer）。
    SwdYaotong,
    /// 破阵（tag 398，deferred 消解批 赵云）：受伤后若与伤害来源手牌数
    /// 不同——多于时弃差值张对其 1 伤；少于时弃其差值张（确认门）。
    SwdPozhen,
    /// 鬼眼·复活面（tag 399，deferred 消解批 冯天凌）：濒死时强制回 1
    /// 并移除鬼眼双技（鬼眼使用时授予本实例）。
    SwdGuiyan2,
    /// 不死（tag 400，deferred 消解批 康那里士）：濒死时判定——黑桃则
    /// 照常死亡；否则体力回至 1 并翻面（unique 一次后移除实例）。
    SwdBusi,
    /// 狩猎（tag 401，deferred 消解批 子达书）：可选——杀指定后弃一张
    /// 手牌令此杀不可闪避（Sha 机器 shoulie_direct 旗标承载）。
    SwdShoulie,
    /// 唤魂（tag 402，deferred 消解批 小雪）：任意角色濒死——弃一张牌
    /// 令其判定：红→回 1；黑→其获得弃置牌。
    SwdHuanhun,
    /// 倾城（tag 403，deferred 消解批 莉莲）：结束阶段判定循环（至多
    /// 3 次）——红判定牌收入手牌，非红弃置终止。
    SwdLqingcheng,
    /// 玄咒（tag 404，deferred 消解批 陈辅）：出牌阶段限一次置出一张普
    /// 通锦囊——视为任意延时锦囊置入任一角色判定区（kind 确定性规则
    /// 登记：目标手牌≤1→兵、余→乐；闪电自置无意义排除）。
    SwdXuanzhou,
    /// 因果（tag 405，deferred 消解批 康那里士）：他人即将受到有来源伤
    /// 害时——可弃一张牌将伤害来源和目标对调（unique 一次）。
    SwdYinguo,
    /// 拷贝栏（tag 406，deferred 消解批 肯迪）：锁定——摸牌阶段额外摸
    /// 1，随后将一张手牌置于牌堆顶（确定性首张登记差异）。
    SwdKbolan,
    /// 奥剑（tag 407，deferred 消解批 Miles）：出牌阶段弃 X 张手牌——
    /// 攻击距离内至多 X 名角色各 1 伤（X=已损失体力；usable:1）。
    SwdAojian,
    /// 化金（tag 408，deferred 消解批 张烈）：出牌阶段限一次弃一张 he——
    /// 授予化金触发面，令你造成的伤害 +1 直到你的下一回合开始。
    SwdHuajin,
    /// 化金触发面（tag 409）：自己造成伤害前置 +1；自己准备阶段开始移除。
    SwdHuajin2,
    /// 明镜（tag 410，deferred 消解批 妮可）：无防具时视为光纱天衣——
    /// 即将受到伤害时 1/3 概率伤害 -1（RNG 同上游 filter）。
    SwdMinjing,
    /// 兰芷（tag 439，deferred 消解批 环婉之）：使用梅花牌后，体力不大于
    /// 自己的已受伤角色（含自己）按座位序各回复 1。上游含 `check`（可放弃）
    /// ——原生恒发动近似登记。
    SwdLanzhi,
    /// 改天道（tag 440，deferred 消解批 孤月）：全局判定——hes 任意牌改判
    /// 并获得原判定牌（同 Boss 张角「天道」语义）。查询型能力：判定流程经
    /// `skill_by_name` 查询持有者，无独立触发时机。
    SwdGTiandao,
    /// 热血（tag 441，deferred 消解批 墨耶/夜鸦熙）：任意角色阶段开始时，
    /// 若其在你杀范围内且你有杀，视为对其使用一张杀，然后摸 1。
    /// 登记差异：虚拟杀（不消耗实体杀）+ 恒发动近似（上游 chooseToUse 可放弃）。
    SwdRexue,
    /// 幻音（tag 442，deferred 消解批 琴儿）：他人对你使用牌时，判定——
    /// 黑桃取消该牌、红桃摸 1、其余无事。
    SwdHuanyin,
    /// 灵弦（tag 443，deferred 消解批 慕月）：非你回合时，你使用或打出牌后，
    /// 与一名攻击距离大于 1 的角色各摸 1。登记差异：自选目标→座位序首个
    /// 确定性近似；可放弃→恒发动近似。
    SwdLingxian,
    /// 拼点（tag 444，deferred 消解批 独孤宁珂）：出牌阶段限一次，弃一张手牌，
    /// 令两名角色拼点：赢者获得该牌、输者受赢者 1 伤。登记差异：拼点牌由
    /// 两目标自选→确定性与座位序近似。
    SwdNlianji,
    /// 蛇魂（tag 445，deferred 消解批 山小小）：出牌阶段限一次，弃 1–4 张
    /// 花色互异的手牌并选一名有牌角色：其弃等量 he 牌，若其中有与弃牌同花色
    /// 者则其受 1 伤。登记差异：花色互异约束未强制（任意 1–4 张）。
    SwdShehun,
    /// 魂身（tag 446，deferred 消解批 胡月）：使用/打出闪后计数 +1。
    SwdHyunshen,
    /// 魂身·收束（tag 447）：准备阶段若计数 >0，摸等量张并清零。
    SwdHyunshen2,
    /// 待行（tag 448，deferred 消解批 吕晓雪）：结束阶段弃 1..张 he 换取等量护甲。
    SwdDaixing,
    /// 待行·清甲（tag 449）：下回合准备阶段按剩余计数移除护甲。
    SwdDaixing2,
    /// 待行·吸收（tag 450）：受伤后递减计数（近似护甲吸收量）。
    SwdDaixing3,
    /// 聚悉（tag 451，deferred 消解批 小黄龙）：出牌阶段，计数达存活数时可发动：
    /// 扣存活数并选一名角色——其已受伤则自选受伤/回复，否则受伤。
    SwdJuxi,
    /// 聚悉·计数（tag 452）：非当前回合角色弃牌后计数 +1。
    SwdJuxiCount,
    /// 破舟（tag 453，deferred 消解批 刘承宣）：受伤后计数累加伤害值。
    SwdPozhou,
    /// 破舟·封印（tag 454）：出牌阶段消耗 1 计数，封印一名其他角色（全部
    /// 技能禁用至其下一阶段开始）。
    SwdPozhou2,
    /// 陷阱（tag 455，deferred 消解批 舒莞儿）：出牌阶段置出一张花色未在
    /// 陷阱表中的手牌为陷阱。
    SwdSxianjing,
    /// 陷阱·触发（tag 456）：他人以同花色牌指定你时，弃陷阱并随机获得其一张牌。
    SwdSxianjingGain,
    /// 陷阱·取回（tag 457）：你受伤时随机取回一张陷阱。
    SwdSxianjingDamage,
    /// 孤行（tag 458，deferred 消解批 慕容诗）：出牌阶段限一次，若手牌数不
    /// 大于 max(1,已损失体力)，将全部手牌当杀使用（1–3 目标）；结算后摸
    /// 该杀造成的伤害数。
    SwdGuxing,
    /// 孤行·计数（tag 459）：孤行杀造成伤害时计数 +1。
    SwdGuxing3,
    /// 灵矢（tag 460，swd deferred 消解批 113 计亭）：锁定——装备区每有
    /// 一张牌攻击范围 +2；有武器或防具时【杀】不可闪避；有马时摸牌阶段
    /// 额外摸一张；有宝物时回合内可额外使用一张【杀】。四条效果全部内联
    /// （距离/杀限次/直中/摸牌四臂），无机器步（天弦同款旗标行为）。
    SwdLingshi,
    /// 锦鳞（tag 461，swd deferred 消解批 114 司空蔷）：limited 一次性主动
    /// 技——选任意名角色各 +3 护甲并授予「锦鳞·退」（每阶段开始 -1 护甲、
    /// 计数 -1，护甲归零或计数归零时移除；计数 3）。
    SwdJinlin,
    /// 锦鳞·退（tag 462）：持有者阶段开始递减护甲与计数。
    SwdJinlin2,
    /// 禳星（tag 463，T11 Boss 诸葛亮族批 115，赫拉西斯 `hlss_xiangxing`）：
    /// 受伤害/失去体力累计 3 点 → 失一枚星（3→0）并向其他角色分配 3 点
    /// 雷伤；星归零即觉醒（不再回满体力）。隐月满足免除条件时不失星。
    HlssXiangxing,
    /// 隐月（tag 464，`hlss_yueyin`）：受到伤害 +1 并记录伤害类型
    ///（雷/火/锦囊位图），供禳星免除条件使用。
    HlssYueyin,
    /// 隐月·判（tag 465，`hlss_yueyin2`）：判定区的牌对持有者不生效
    ///（上游 mod judge 反转）+ 不能被翻面。
    HlssYueyin2,
    /// 起风（tag 466，T11 Boss 诸葛亮族批 116，`hlss_fengqi`）：准备阶段
    /// 限一次——从可用**非延时锦囊**牌名中选一张并使用（本技已用过的
    /// 牌名不再出现）。
    HlssFengqi,
    /// 起风·闭（tag 467，`hlss_fengqi2`）：锁定——你使用的牌不可被无懈。
    HlssFengqi2,
    /// 改命（tag 468，`hlss_gaiming`）：锁定旗标——判定前亮牌堆顶 7 张，
    /// 选 1 作为判定结果（其余弃置，改判/天妒窗跳过）；逻辑内联于判定机。
    HlssGaiming,
    /// 潜龙·道心值（tag 469，D5 龙血玄黄批 117，`mbqianlong`）：开局
    /// 20 点（另有魏势力持「卫统」者 60 点）；获得牌 +5、受伤 +10×伤害、
    /// 造成伤害 +15×伤害（上限 99）；阈值 25/50/75/99 解锁清正/酒诗/
    /// 放逐/绝进（外部技能存在才授予）。
    MbQianlong,
    /// 卫统（tag 470，`mbweitong`，主公技）：开局登记「修改潜龙初始道心值」
    /// （实际 +60 由潜龙内容按同条件判定）。
    MbWeitong,
    /// 涅槃（tag 471，T11 Boss 庞统族批 118，`niepan`）：限定技——濒死时
    /// 或出牌阶段发动：弃置全部牌（手/装/判）、解除横置、翻回正面、摸 3、
    /// 回复至 3 点体力；发动后触发浴火。
    Niepan,
    /// 浴火（tag 472，`boss_yuhuo`）：涅槃后授予看破/神威/朱羽（目录存在
    /// 才授予）并置关卡循环型 1（逻辑内联于涅槃收束）。
    Yuhuo,
    /// 神威（tag 473，`shenwei`）：锁定——摸牌阶段多摸 min(3, 其他角色数)，
    /// 手牌上限同量加成。
    Shenwei,
    /// 烈弓（tag 535，L20 标记补充批 3，`fz_liegong`）：锁定——对目标使用
    /// 【杀】时，若目标手牌数 ≥ 自己体力或 ≤ 自己攻击范围，则该目标**不可被
    /// 闪避**（内联杀机器）。
    FzLiegong,
    /// 狂骨（tag 536，L20 批 3，`fz_xinkuanggu`）：自己造成伤害后可选（距离
    /// ≤1）——回复 1 点体力或摸 1 张牌。
    FzXinKuanggu,
    /// 苦肉（tag 533，L20 标记补充批 2，`kurou`）：出牌阶段主动——失去 1 点
    /// 体力并摸 2 张牌。
    Kurou,
    /// 主亡忠继（tag 534，L20 批 2，`_xigua_zhuwanzhongji_`）：锁定——主公
    /// 阵亡时，忠臣**继位为主公**（身份互换）。
    ZhuWanzhongJi,
    /// 摸牌翻倍（tag 531，L20 标记补充批 1，`_xigua_mopaifanbei_`）：锁定——
    /// 摸牌阶段额外摸 2 张（上游配置 `mpfbmoshi` 门控；原生以实例启用承载）。
    XiguaMopaiFanbei,
    /// 血量翻倍（tag 532，L20 批 1，`_xigua_xueliangfanbei_`）：锁定——开局
    /// 体力上限 + 当前体力并回复等量（上游配置 `xueliangfanbei` 门控；原生以
    /// 首次阶段开始的一次性标记承载 `gameStart`，登记差异）。
    XiguaXueliangFanbei,
    /// 门神（tag 528，U08 英雄杀批 2，`yxs_menshen`）：自己结束阶段可选——
    /// 选一名其他角色，令其获得「门神」标记（`yxs_menshen2` 实例 + 标记=自己
    /// 座位）。
    YxsMenshen,
    /// 门神·清除（tag 529，U08 批 2，`yxs_menshen3`）：锁定——任意阶段开始或
    /// 任意角色死亡时，清除全场「门神」标记与实例。
    YxsMenshen3,
    /// 门神·标记持有（tag 530，U08 批 2，`yxs_menshen2`）：被动占位（主体
    /// 70 KB 效果待后续批次，登记差异）。
    YxsMenshen2,
    /// 反击（tag 526，U08 英雄杀批 1，`yxs_fanji`）：自己受到【杀】/【决斗】
    /// 伤害后可选（手牌有【杀】）——获得临时「反击·红杀直伤」并对伤害来源
    /// 使用一张【杀】。
    YxsFanji,
    /// 反击·红杀直伤（tag 527，U08 批 1，`yxs_fanji2`）：临时——自己使用的
    /// **红色**【杀】不可被闪避（内联杀机器）。
    YxsFanji2,
    /// 灾变（tag 525，L19 末日乱斗批 2，`zaibian`）：锁定——自己出牌阶段
    /// 开始时摸「忠 + 2 - 反 - 内」（下限 0）张牌。
    Zaibian,
    /// 誓师（tag 523，L19 末日乱斗批 1，`shishi`）：锁定——自己击杀角色后
    /// 体力上限 +1 并回复 1 点。
    Shishi,
    /// 循梦（tag 524，L19 末日乱斗批 1，`xunmeng`）：锁定——自己以【杀】
    /// 造成的伤害 +1（非铁索传导），且自己体力 >1 时失去 1 点体力。
    Xunmeng,
    /// 集舍（tag 522，L27 选将美化批 5，`jishe`）：出牌阶段主动（手牌上限
    /// >0）——摸一张牌并累计标记（上游 `jishe2` 的手牌上限削减面因该技在
    /// 上游被注释而**不生效**，登记差异）。
    Jishe,
    /// 诈降（tag 521，L27 选将美化批 4，`sbzhaxiang`）：锁定——本阶段已用
    /// 牌数 < 已损失体力时：用牌**无距离/次数限制**且【杀】**不可被闪避**；
    /// 摸牌阶段额外摸「已损失体力」张。
    SbZhaXiang,
    /// 每步（tag 519，L27 选将美化批 3，`new_meibu`）：任意角色出牌阶段
    /// 开始时可选——弃一张牌（手/装备），令其本回合**止息**（不能用牌）；
    /// 若弃牌非【杀】非锦囊且非黑，另使其到你的距离视为 1。
    NewMeibu,
    /// 每步·距离（tag 520，L27 批 3，`new_meibu_range`）：临时——到「每步」
    /// 弃牌者的距离视为 1（内联 `computed_distance`）。
    NewMeibuRange,
    /// 止息（tag 518，L27 选将美化批 2，`new_zhixi`）：锁定——使用牌累计
    /// 标记；标记 ≥ 体力或本局已使用过**锦囊**时，不能使用/响应牌（状态
    /// 相关封锁）；标记显示「体力 - 计数」。
    NewZhixi,
    /// 撷芳（tag 517，L27 选将美化批 1，`xiefang`）：锁定——距离 -存活女性
    /// 数（内联于 `computed_distance`）；并在死亡/摸牌阶段结束时刷新
    /// 「女性数」标记（0 时移除）。
    Xiefang,
    /// 让节（tag 516，史诗卡牌批 2，`rangjie`）：自己受到伤害后可选（按
    /// 伤害点数逐次）——「获得牌」（摸 1 + 从牌堆取一张指定类型牌）或
    /// 「移动牌」（把场上可见区一张牌移到一名角色处）或取消。
    Rangjie,
    /// 蛊惑（tag 515，史诗卡牌批 1，`reguhuo`）：出牌阶段限一次——弃一张
    /// 手牌并声明任意**基本牌/锦囊牌**名，以该牌使用（转化声明；原生实体
    /// 花色点数保留=登记差异）。
    Reguhuo,
    /// 攻心（tag 514，swd 批 155，`gongxin`）：出牌阶段限一次——看一名
    /// 其他角色的手牌，选其一张**红桃**牌：弃置之，或将其置于牌堆顶。
    Gongxin,
    /// 魔化（tag 513，swd 批 154，`mohua2`）：自己进入濒死时**锁定**——
    /// 移除旧三技（血裔/傲剑/魔化）、获得撒旦四技（魔焰/灭道/疾风/修罗）、
    /// 体力上限改 4、回复 2、摸 2。
    Mohua2,
    /// 修罗（tag 512，swd 批 153，`swd_xiuluo`）：准备阶段可选——弃一张
    /// 手牌，弃置判定区一张**同花色**延时锦囊。
    Xiuluo,
    /// 灭道（tag 511，swd 批 152，`miedao`）：组技两面——摸牌阶段多摸
    /// 「已损失体力」张（内联于摸牌收束）；弃牌阶段须额外弃等量张（手/装备，
    /// 强制）。
    Miedao,
    /// 魔焰（tag 510，swd 批 151，`moyan`）：出牌阶段限一次——弃 N 张红色
    /// 手牌（N ≤ min(其他存活数, 体力上限-体力)），对 N 名其他角色各造成
    /// 1 点**火焰**伤害。
    Moyan,
    /// 疾风（tag 509，swd 批 150，`jifeng`）：单目标牌可指定任意数量目标
    /// （Sha/单目标锦囊）；多目标使用时按概率「失误」——整张牌取消。
    Jifeng,
    /// 昆仑镜（tag 507，swd deferred 消解批 149，`kunlunjing`）：组技两
    /// 面——阶段收束记录全场（手牌/装备/判定区）快照与当时体力；之后自己
    /// 阶段开始时若体力低于快照值，可**回滚**：把快照中的牌拉回各自原位。
    Kunlunjing,
    /// 神弓（tag 506，swd deferred 消解批 148，`shengong`）：需要打出
    /// 【闪】/【杀】时，可借用**其他角色**的防具/武器作为响应（上游
    /// `chooseToRespondBegin` 借用装备；原生在响应窗追加借用候选）。
    Shengong,
    /// 轩辕剑·伤增（tag 503，swd 批 146，`xuanyuanjian2`）：装备者造成
    /// 伤害时 +1 并转为雷属性（未横置伤害）。
    XuanyuanjianDamage,
    /// 轩辕剑·反噬（tag 504，同批，`xuanyuanjian3`）：上项加伤生效后，
    /// 装备者若无「轩辕」则失去 1 点体力。
    XuanyuanjianBacklash,
    /// 轩辕（tag 505，同批，`xuanyuan`）：结束阶段限定技——弃一张黑桃牌，
    /// 从弃牌堆取【轩辕剑】装备。
    Xuanyuan,
    /// 燎原（tag 502，swd deferred 消解批 145，`liaoyuan`）：自己使用【杀】
    /// 时可选——弃置任意张与该杀同花色的牌（手/装备），令目标额外需打出
    /// 等量张【闪】；未能全部打出则该杀**直接命中**且伤害 +张数。
    Liaoyuan,
    /// 改造（tag 501，swd deferred 消解批 143，`gaizao`）：自己使用装备牌
    /// 且该牌固有栏位已被占用时可选——改放到一个**空栏位**（牌种不变故
    /// 装备技能保留；`declared_slots` 栏位覆盖）。
    Gaizao,
    /// 天书（tag 500，swd deferred 消解批 142，`swdtianshu`）：出牌阶段
    /// 限定技——弃一张锦囊手牌，选一名角色，获取其武将的**一个未持有
    /// 技能**（永久）；来源角色阵亡时移除（内联臂）。
    SwdTianshu,
    /// 李香龙（tag 499，swd deferred 消解批 141，`lxianglong`）：自己的
    /// 【杀】被闪避后可选——对攻击范围内一名角色使用一张虚拟【杀】。
    Lxianglong,
    /// 移花（tag 498，swd deferred 消解批 140，`yihua`）：成为**单目标**牌
    /// 的目标时可选——弃 2 张手牌，把该牌的「使用者/目标」对调（反弹给原
    /// 使用者，且视为自己使用）。
    Yihua,
    /// 贞烈（tag 497，swd deferred 消解批 139，`zhenlie`）：成为其他角色
    /// 【杀】或锦囊牌的目标时可选——失去 1 点体力，**将该牌从自己目标中
    /// 移除**（结算前写回帧 targets），并弃置使用者一张牌（手/装备）。
    Zhenlie,
    /// 天伦（tag 496，swd deferred 消解批 138，`tianlun`）：任意判定时可选
    /// ——从「本次判定牌 + 全场判定区牌」中选一张替换本次判定牌，旧判定牌
    /// 弃置（判定机改判窗驱动；无触发时机）。
    Tianlun,
    /// 脉轮（tag 495，swd deferred 消解批 137，`mailun`）：任意阶段开始时
    /// 可选——从 8 种效果中选一种（1 换体力上限/2 反向/3 首次伤害-1/4 首次
    /// 伤害+1/5 少摸一张且上限+1/6 多摸一张且上限-1/7 双向距离-1/8 双向
    /// 距离+1），旧效果在重选时清除；3/4 为一次性（造成侧与受到侧各一次）。
    Mailun,
    /// 极略（tag 494，swd deferred 消解批 136，`jilve`）：出牌阶段限三次
    /// ——观看牌堆顶 3 张，从中选一张**基本牌或锦囊牌**使用（非手牌用牌；
    /// 目标按通用目标规则表选定）。
    Jilve,
    /// 革新印（tag 493，swd deferred 消解批 135，`gxianyin`）：出牌阶段限
    /// 一次——弃置自己一种花色的全部手牌（**不触发失去牌类事件**），再从
    /// 牌堆/弃牌堆获得等量张另一种花色的牌。
    Gxianyin,
    /// 冰封（tag 491，swd deferred 消解批 134，`bingfeng`）：出牌阶段限定
    /// 技——翻面自己与至多 3 名未翻面其他角色，失去 1 点体力上限，并给
    /// 双方授予「冰封·封」（tag 492：出牌/响应/求桃全封锁，翻回正面时移除）。
    Bingfeng,
    /// 冰封·封（tag 492，`bingfeng2`）：出牌封锁旗标（翻回正面自移除）。
    Bingfeng2,
    /// 此桥邪（tag 490，swd deferred 消解批 133，`cyqiaoxie`）：自己失去
    /// 装备牌后，按失去张数逐次——从牌堆/弃牌堆可得的**机关牌名**中选一张
    /// 使用（自目标/单目标/AOE 按牌名规则；取消即止）。
    Cyqiaoxie,
    /// 魂殉志（tag 488，swd deferred 消解批 132，`hxunzhi`）：出牌阶段限定
    /// 技——视为对全部其他角色使用【万箭齐发】，并获得武圣/咆哮；出牌阶段
    /// 结束时死亡（组技 `hxunzhi2`，tag 489 旗标 + 内联臂）。
    Hxunzhi,
    /// 魂殉志·殉（tag 489，`hxunzhi2`）：出牌阶段结束死亡（旗标 + 内联臂）。
    Hxunzhi2,
    /// 护天（tag 487，swd deferred 消解批 131，`hutian`）：结束阶段可选——
    /// 选任意张牌（手/装备）寄存于一名体力上限不低于张数的其他角色处；
    /// 其体力低于寄存数时回复至该数；寄存期间其受到的伤害/失去体力以
    /// 「体力降至寄存数」为上限；寄存只跨一个阶段边界，随后归还（归还由
    /// 阶段收束/死亡内联承载）。
    Hutian,
    /// 天香（tag 485，swd deferred 消解批 130，`tianxiang`）：受到伤害前
    /// 可选——弃一张红桃手牌，把该伤害转移给一名其他角色；其承受后按其
    /// 实际失去的体力摸等量牌（组技 `tianxiang2`，运行时授予，tag 486）。
    Tianxiang,
    /// 天香·受（tag 486，`tianxiang2`）：伤害结算后按其失去体力摸牌并移除。
    Tianxiang2,
    /// 颂词（tag 484，swd deferred 消解批 129，`songci`）：出牌阶段主动
    /// （可重复）——选一名未发动过的角色（含自己）：其手牌多于体力则弃 2 张
    /// （手/装备），否则摸 2 张；全部角色均已记录时弃牌阶段末摸 1（组技
    /// `songci_draw`，内联于阶段收束）。
    Songci,
    /// 诛害（tag 483，swd deferred 消解批 128，`zhuhai`）：任意角色结束
    /// 阶段开始时——若其本回合造成过伤害、自己手中有【杀】且其在你攻击
    /// 距离内，可对其使用一张【杀】（实体牌）。
    SwdZhuhai,
    /// 林韵（tag 482，swd deferred 消解批 127，`linyun`）：出牌阶段主动——
    /// 两张牌（手牌或装备）当【杀】使用；转化杀照常结算（响应闪/伤害）。
    SwdLinyun,
    /// 集控（tag 481，swd deferred 消解批 126，`jikong`）：每回合限一次——
    /// 自己阶段开始，或自己失去手牌致手牌为空时，可对攻击距离内一名其他
    /// 角色使用虚拟【雷杀】；取消不消耗次数（上游 `counttrigger--` 同口径）。
    SwdJikong,
    /// 魅惑（tag 480，swd deferred 消解批 125，`meihuo`）：自己失去装备
    /// 且装备区已空时可选——选一名其他有牌角色，获得其一张牌（装备牌则
    /// 直接装备之）。
    SwdMeihuo,
    /// 守印（tag 479，swd deferred 消解批 124，`shouyin`）：限定技——任意
    /// 角色濒死时（自己未翻面）可发动：翻面自己，随后按座位序将全部已受伤
    /// 角色回复至满体力。
    SwdShouyin,
    /// 义山（tag 478，swd deferred 消解批 123，`yishan`）：自己受伤后，
    /// 取回「最近失去的牌」中仍在他处者至多 2 张（新在前）。
    SwdYishan,
    /// 入梦（tag 477，swd deferred 消解批 122，`rumeng`）：他人出牌阶段
    /// 开始前可选——弃一张非基本牌（手/装备），令其弃一张基本牌，否则
    /// 跳过其出牌与弃牌阶段。
    SwdRumeng,
    /// 神焰（tag 476，swd deferred 消解批 121，`shenyan`）：限定技——你
    /// 造成的**火属性**伤害 +1，并在结算后令原目标距离 ≤1 的其他角色
    /// （座位序，不含你与原目标）各受 1 点无来源火伤。
    SwdShenyan,
    /// 灵武（tag 475，swd deferred 消解批 120，`lingwu`）：频繁技——阶段
    /// 结束时本回合已用牌数 ≥ 体力则插入一个额外出牌阶段（效果内联于
    /// 阶段收束；本行为仅登记与触发面）。
    SwdLingwu,
    /// 清正（tag 474，T11 Boss 庞统族后继批 119，`mbcmqingzheng`）：出牌
    /// 阶段开始可选——弃置自己一种花色的全部手牌，再令一名其他角色弃置
    /// 其一种花色的全部手牌；其弃牌数少于自己时受 1 点伤害（无来源）。
    MbcmQingzheng,
    /// 青木（tag 411，D4/B1）：Boss 死亡前置隐藏（dieBegin；filter=本人是 boss）。
    BossQingmu2,
    /// 青木·转阶（tag 412，D4/B1）：Boss 死亡前置晚窗转阶→勾芒+书精 fellow
    /// （上游 dieAfter priority -10；原生 Death 仅 BeforeTriggers，见 ADR 0783）。
    BossQingmu2x,
    /// 青木3（tag 413，D4/B1）：Boss 死亡前置三人隐藏（本体+环邻）。
    BossQingmu3,
    /// 青木3·转阶（tag 414，D4/B1）：转阶→太昊，并侧座重挂勾芒/书精。
    BossQingmu3x,
    /// 白芒（tag 415，D4/B2）：Boss 死亡前置隐藏（dieBegin；filter=本人是 boss）。
    BossBaimang2,
    /// 白芒·转阶（tag 416，D4/B2）：→蓐收 + 两明星 fellow + 全场回满。
    BossBaimang2x,
    /// 赤炎（tag 417，D4/B2）：Boss 死亡前置隐藏。
    BossChiyan2,
    /// 赤炎·转阶（tag 418，D4/B2）：→祝融 + 两焰灵 fellow + 全场回满。
    BossChiyan2x,
    /// 玄麟（tag 419，D4/B2）：Boss 死亡前置隐藏。
    BossXuanlin2,
    /// 玄麟·转阶（tag 420，D4/B2）：→共工 + 一玄冥 fellow + 全场回满。
    BossXuanlin2x,
    /// 白芒3（tag 421，D4/B3）：Boss 死亡前置三人隐藏（本体+环邻）。
    BossBaimang3,
    /// 白芒3·转阶（tag 422，D4/B3）：→少昊 + 侧座蓐收/明星 + 一明星 fellow。
    BossBaimang3x,
    /// 赤炎3（tag 423，D4/B3）：Boss 死亡前置三人隐藏。
    BossChiyan3,
    /// 赤炎3·转阶（tag 424，D4/B3）：→炎帝 + 侧座祝融/焰灵。
    BossChiyan3x,
    /// 玄麟3（tag 425，D4/B3）：Boss 死亡前置三人隐藏（上游同侧邻座条件近似）。
    BossXuanlin3,
    /// 玄麟3·转阶（tag 426，D4/B3）：→颛顼 + 共工 fellow + 次座玄冥。
    BossXuanlin3x,
    /// 后援3（tag 427，D4/B4）：Boss 死亡前置隐藏（bianshen3；牛头/马面）。
    BossBianshen3,
    /// 后援3·转阶（tag 428，D4/B4）：种子化换 Boss → 白无常/黑无常池。
    BossBianshen3x,
    /// 后援4（tag 429，D4/B4）：Boss 死亡前置隐藏（bianshen4；黑/白无常）。
    BossBianshen4,
    /// 后援4·转阶（tag 430，D4/B4）：种子化换 Boss → 夜叉/罗刹池。
    BossBianshen4x,
    /// 还人（tag 431，D4/B4）：十殿开局——禁将表、40% 孟婆乱入。
    BossHuanren,
    /// 还人·转关（tag 432，D4/B4）：Boss 死亡选将换关。
    BossHuanrenDie,
    /// 青翼（tag 433，D4/B4）：轮次 3/5/7 编排。
    BossQingyi,
    /// 赤翼（tag 434，D4/B4）：轮次 3/5/7 编排。
    BossChiyi,
    /// 赤仪2（tag 435，D4/B4）：受到的伤害 +1。
    BossChiyiMark,
    /// 白翼（tag 436，D4/B4）：第 5 轮敌方弃 2。
    BossBaiyi,
    /// 白翼·雷（tag 437，D4/B4）：第 7 轮前雷伤无效。
    BossBaiyiThunder,
    /// 末焱（tag 438，3D精选·刘冲）：一轮内首次判定阶段开始展示至多 3 张
    /// 牌库顶牌，一名角色获得其中至少一张；该角色存活且手牌数不大于获得
    /// 张数时，技能持有者受到的伤害 +1。标记进 `vm_aux.flags`，标准局不写。
    D3Moyan,
    /// 鬼蛇旗（tag 297，批⑦ trick 卡牌机）：目标 +1 护甲。
    GuisheqiCard,
    /// 仙露回（tag 298，批⑦ trick 卡牌机）：全体受伤目标各 +1 护甲。
    XianluhuiCard,
    /// 苍螭之璧（tag 538，swd 裁决 33）：手持法器，回合开始可选 1–3 名角色横置/重置。
    SwdCangchizhibiHeld,
    /// 穿云（tag 539，太虚幻境·extension_skill.js:3806）：结束阶段开始，若场上有体力值
    /// 大于等于你的其他角色，须选择其中一名造成 1 点伤害。
    BossChuanyun,
    /// 天陨（tag 540，太虚幻境·extension_skill.js:4085）：结束阶段开始，可失去 1 点体力并
    /// 选择一名其他角色，对其造成 2 点火焰伤害并弃置其所有装备牌。
    BossTianyun,
    /// 侠行（tag 541，3D精选·character/skill.js:59）：一轮一次，任意角色准备阶段或濒死时，
    /// 可弃置一张牌令其随机获得牌堆中一张闪/桃（本轮「侠」标记下可视为无距离限制使用【杀】）。
    DddXiaxing,
    /// 夺气（tag 542，玩点论杀·character/skill.js:109）：其他角色出牌阶段弃置牌后，
    /// 可移去一张「筑」牌并令当前角色结束出牌阶段。
    WudianDuoqi,
    /// 诱战（tag 543，玩点论杀·character/skill.js:422）：成为【杀】目标（或你攻击范围内至多 1
    /// 距离角色成为【杀】目标）时，可弃置一张装备牌令此【杀】无效并视为对其执行【诱敌深入】。
    WudianYouzhan,
    /// 筹度（tag 544，玩点论杀·character/skill.js:586）：出牌阶段限一次，可弃置一张牌并指定
    /// 一名角色视为使用一张【调兵遣将】。
    WudianChoudu,
    /// 调兵遣将（tag 545，玩点论杀·card/card.js:4）：使用者摸一张牌，亮出牌堆顶 ceil(存活数/2)
    /// 张牌，各目标可依次用一张手牌交换其中一张亮出牌。
    WudianDiaobingCard,
    /// 酒诗（tag 546，L29 龙血玄黄·mobile/skill.js:14399，`mbcmjiushi`）：
    /// 出牌阶段若正面朝上且本回合未用过【酒】，可翻面视为使用【酒】并可随机获得一张锦囊牌；
    /// 背面朝上受到非本技能强化【杀】造成的伤害后可翻回正面；翻面时可随机获得一张锦囊牌。
    MbcmJiushi,
    /// 放逐（tag 547，L29 龙血玄黄·mobile/skill.js:14559，`mbcmfangzhu`）：
    /// 出牌阶段限一次，可选择一名非上次指定的其他角色并选择限制其使用非锦囊手牌或令其技能失效至其下回合结束。
    MbcmFangzhu,
    /// 决进（tag 548，L29 龙血玄黄·mobile/skill.js:14751，`mbjuejin`）：
    /// 限定技，出牌阶段可令所有角色将体力调整为 1（减少体力转化为护甲，发动者额外 +2 护甲），
    /// 并将全场（牌堆、弃牌堆、手牌、装备区、判定区）及后续进入弃牌堆的【闪】、【桃】、【酒】移出游戏。
    MbJuejin,
    /// 党锢（tag 549，L10 史诗卡牌·extension.js:2276，`k_danggu`）：
    /// 锁定技，开局首阶段与修整归来准备阶段（摸两张牌）进行结党，随机抽出首名常侍并从无冲突候选常侍中选择结党对象以获得两者伴生技。
    KDanggu,
    /// 殁亡（tag 550，L10 史诗卡牌·extension.js:2677，`k_mowang`）：
    /// 锁定技，濒死或回合结束时若仍有未亮出常侍且体力上限大于 0 则进入修整（复原武将、弃置全区域牌、撤销当前结党伴生技），下回合准备阶段满血归来并触发党锢。
    KMowang,
    /// U01 3D精选 Batch A（tag 551，character/skill.js 1–15）：
    /// 远振、治束、聚贤、均归、势烈、业身、巧断、跌宕、暗流、归萤、并肩、统御。
    U01DddBatchA,
    /// U01 3D精选 Batch B（tag 552，character/skill.js 16–31）。
    U01DddBatchB,
    /// U01 3D精选 Batch C（tag 553，character/skill.js 32–47）。
    U01DddBatchC,
    /// U01 3D精选 Batch D（tag 554，character/skill.js 48–64）。
    U01DddBatchD,
    /// L28 雷霆万钧 Batch A（tag 555，雷霆万钧/extension.js 武将 1–9）。
    L28ThunderBatchA,
    /// L28 雷霆万钧 Batch B（tag 556，雷霆万钧/extension.js 武将 10–14）。
    L28ThunderBatchB,
    /// L27 选将美化 Batch A（tag 557，选将美化/extension.js 前 16 个完整技能 + 3 个标记/屏蔽桩）。
    L27BeautifyBatchA,
    /// L27 选将美化 Batch B（tag 558，选将美化/extension.js 后 15 个完整技能 + 4 个辅助/界面桩）。
    L27BeautifyBatchB,
    /// L19 末日乱斗 Batch A（tag 559，末日乱斗/extension.js 模式 1–3 核心技能 + 20 注释/VFX 桩）。
    L19DoomsdayBatchA,
    /// L19 末日乱斗 Batch B（tag 560，末日乱斗/extension.js 模式 4–10 RPG/祖安桥接技能）。
    L19DoomsdayBatchB,
    /// L20 标记补充 Batch A（tag 561，标记补充/extension.js 国战标记/调虎离山/问卦及 41 个 UI/标记桩）。
    L20MarkBatchA,
    /// L20 标记补充 Batch B（tag 562，标记补充/extension.js 整肃/贞仪/破围/清玉等 20 个核心技能）。
    L20MarkBatchB,
    /// U08 英雄杀 Batch A（tag 563，英雄杀/character/skill.js 23 技能：兰陵王/妺喜/魏忠贤/南丁格尔/张三丰/唐伯虎/杨广/赵雍/鲁智深/小乔）。
    U08YxsBatchA,
    /// U08 英雄杀 Batch B（tag 564，英雄杀/character/skill.js 27 技能：华佗/吕雉/岳飞/项羽/程咬金/孙武/狄仁杰/霍去病/王昭君/虞姬）。
    U08YxsBatchB,
    /// U08 英雄杀 Batch C（tag 565，英雄杀/character/skill.js 27 技能：成吉思汗/范蠡/李师师/刘邦/秦始皇/武则天/杨玉环/褒姒）。
    U08YxsBatchC,
    /// U08 英雄杀 Batch D（tag 566，英雄杀/character/skill.js 27 技能：伯乐/曹操/貂蝉/勾践/花木兰/墨子/拿破仑/孙膑/赢政/荆轲/李白/鲁班）。
    U08YxsBatchD,
}
