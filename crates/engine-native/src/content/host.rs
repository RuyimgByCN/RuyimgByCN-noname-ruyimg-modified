//! G1 规则宿主原型（ADR 0776）：扩展包以**数据声明行为**，宿主以
//! 有界原语执行——扩展机器载荷的独立装载与显式续行状态。
//!（2026-10-10 事故重建：臂库自 db.part 重放+挖掘恢复；部分行为臂
//! 为占位待重写——见 file-loss-recovery-20261010 记忆）

use crate::catalog::{CardKind, QueryCtx, SkillBehavior};
use crate::catalog::TriggerMeta;
use crate::event::{EventKind, Timing};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::EventData;
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::snap::MachineSnapshot;


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
    /// 疾影（tag 311，批㉑ 赛特）：锁定 sha 无视距离（marker，sha_range_
    /// free 链 skill 查询）。
    SwdJiyingHeld,
    /// 沐风（tag 312，批㉑ 桓远之）：结束阶段本回合用过基本牌则摸 1
    ///（discoverCard 确定性=牌堆末摸 1）。
    SwdMufengHeld,
    /// 望尘（tag 313，批㉑ 古月圣）：弃牌阶段结束弃置过基本牌则令一名角
    /// 色翻面（End 帧入场确认门）。
    SwdWangchenHeld,
    /// 玉蝉换形（tag 310，批⑮ equipSkill）：自己用牌结算后，手牌中玉蝉
    /// 实体轮换为八卦序下一 kind 的实体（确定性发现；登记差异上游随机）。
    SwdYuchanSwap,
    /// 鬼蛇旗（tag 297，批⑦ trick 卡牌机）：目标 +1 护甲。
    GuisheqiCard,
    /// 仙露回（tag 298，批⑦ trick 卡牌机）：全体受伤目标各 +1 护甲。
    XianluhuiCard,
}


fn u_matches_lw2(
    event: &EventData,
    owner: u8,
    ctx: &QueryCtx<'_>,
) -> bool {
    matches!(event,
        EventData::CardUsed(u)
        if u.player == owner
            && u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card))
                == crate::catalog::CardKind::Shuigong
            && !u.targets.is_empty())
}

impl HostBehavior {
    /// u16（批 11 扩容）：u8 值域已满（256 变体全占用）；编号只增不改
    /// ——ABI 兼容对包数据无影响（包内为 `host:*` 字符串键，tag 仅内部
    /// 编号与存档快照）。
    pub const fn tag(self) -> u16 {
        match self {
            HostBehavior::YunchouXicai => 241,
            HostBehavior::YunchouDiykuanggu => 242,
            HostBehavior::YunchouXiongzi => 243,
            HostBehavior::YunchouYaliang => 244,
            HostBehavior::YunchouZonghuo => 245,
            HostBehavior::YunchouShaoying => 246,
            HostBehavior::YunchouDiyguhuo => 247,
            HostBehavior::YunchouJiaoxia => 248,
            HostBehavior::YunchouHonglian => 249,
            HostBehavior::YunchouLuweiyan => 250,
            HostBehavior::YunchouLuweiyan2 => 251,
            HostBehavior::YunchouJieyan => 252,
            HostBehavior::YunchouYiesheng => 253,
            HostBehavior::YunchouDiychanyuan => 254,
            HostBehavior::WudianDiyjuntun => 255,
            // Jinyan 用历史空洞 96（u8 值域已满——256 越界；from_tag 同步；
            // 批 11 起 tag/from_tag 已扩 u16，新变体顺序编号）
            HostBehavior::WudianJinyan => 96,
            HostBehavior::WudianLiduan => 257,
            HostBehavior::WudianJianbi => 258,
            HostBehavior::WudianFuchou => 259,
            HostBehavior::WudianFuchou2 => 260,
            HostBehavior::WudianLiangceRemain => 261,
            HostBehavior::WudianGuihan => 262,
            HostBehavior::YunchouBeige => 263,
            HostBehavior::SwdTiebi => 264,
            HostBehavior::SwdDatong => 265,
            HostBehavior::SwdChengjian => 266,
            HostBehavior::SwdJufu => 267,
            HostBehavior::SwdHuajing => 268,
            HostBehavior::SwdHudun => 269,
            HostBehavior::SwdPoxing => 270,
            HostBehavior::SwdJiefen => 271,
            HostBehavior::SwdLiuhong => 272,
            HostBehavior::SwdMojian => 273,
            HostBehavior::SwdDaofa => 274,
            HostBehavior::SwdYulin => 275,
            HostBehavior::SwdNingshuang => 276,
            HostBehavior::SwdGongshen => 277,
            HostBehavior::SwdPingxu => 278,
            HostBehavior::SwdHuanling => 279,
            HostBehavior::SwdDuanyue => 280,
            HostBehavior::SwdTuzhen => 281,
            HostBehavior::SwdLuomei => 282,
            HostBehavior::SwdXingdian => 283,
            HostBehavior::SwdXielv => 284,
            HostBehavior::SwdZhuyu => 285,
            HostBehavior::SwdXuehuang => 286,
            HostBehavior::SwdHuanxia => 287,
            HostBehavior::SwdPoyue => 288,
            HostBehavior::SwdLianyaohuShouna => 289,
            HostBehavior::SwdLianyaohuLianhua => 290,
            HostBehavior::SwdHujing => 291,
            HostBehavior::SwdShennongdingCard => 292,
            HostBehavior::SwdNvwashiGuard => 293,
            HostBehavior::SwdKunlunjingcCard => 294,
            HostBehavior::SwdDonghuangzhongCard => 295,
            HostBehavior::SwdKongdongyinGuard => 296,
            HostBehavior::SwdNigongAccumulate => 299,
            HostBehavior::SwdNigongStrike => 300,
            HostBehavior::SwdSifeizhenmianCard => 303,
            HostBehavior::SwdQinglongzhiguiHeld => 304,
            HostBehavior::SwdZhuquezhizhangHeld => 305,
            HostBehavior::SwdBaishouzhihuHeld => 306,
            HostBehavior::SwdXuanwuzhihuangHeld => 307,
            HostBehavior::SwdHuanglinzhicongHeld => 308,
            HostBehavior::SwdShencaojieGuard => 309,
            HostBehavior::SwdYuchanSwap => 310,
            HostBehavior::SwdJiyingHeld => 312,
            HostBehavior::SwdMufengHeld => 313,
            HostBehavior::SwdWangchenHeld => 314,
            HostBehavior::GuisheqiCard => 301,
            HostBehavior::XianluhuiCard => 302,
            HostBehavior::ShaDamageForceDiscard => 0,
            HostBehavior::PhaseEndRecoverOrDraw => 1,
            HostBehavior::TargetDiscardEquipOrDamage => 2,
            HostBehavior::ShengkeCheck => 3,
            HostBehavior::ChangeCardNature => 4,
            HostBehavior::MaxRangeWeaponDamage => 5,
            HostBehavior::WangmeiZhikeCard => 6,
            HostBehavior::SijiDiscardDraw => 7,
            HostBehavior::SanjianWeaponDamage => 8,
            HostBehavior::MoshouImmunity => 9,
            HostBehavior::ShuigongCard => 10,
            HostBehavior::FudichouxinCompare => 11,
            HostBehavior::ChenhuodajieCard => 12,
            HostBehavior::TouliangSwap => 13,
            HostBehavior::LinghunzhihuoCard => 14,
            HostBehavior::ZhaomingdanCard => 15,
            HostBehavior::JihuocardDraw => 16,
            HostBehavior::GeanguanhuoCompare => 17,
            HostBehavior::CaochuanCard => 18,
            HostBehavior::ShezhanqunruCompare => 19,
            HostBehavior::GujianShujinsanCard => 20,
            HostBehavior::GujianMianliCard => 21,
            HostBehavior::GujianFeibiaoCard => 22,
            HostBehavior::JinlianzhuCancel => 23,
            HostBehavior::GujianFoodTanhuadong => 24,
            HostBehavior::GujianFoodYougeng => 25,
            HostBehavior::GujianFoodGrant => 26,
            HostBehavior::GujianFoodQingtuan => 27,
            HostBehavior::GujianFoodYuanbaorou => 28,
            HostBehavior::GujianFoodGudonggeng => 29,
            HostBehavior::GujianFoodMapodoufu => 30,
            HostBehavior::GujianFoodChunbing => 31,
            HostBehavior::GujianFoodLiyutang => 32,
            HostBehavior::GujianFoodMolicha => 33,
            HostBehavior::GujianFoodXiajiao => 34,
            HostBehavior::GujianFoodMizhilianou => 35,
            HostBehavior::GujianFoodLuyugeng => 36,
            HostBehavior::JiguanBingpoCard => 37,
            HostBehavior::JiguanQiankunCard => 38,
            HostBehavior::JiguanLongxuCard => 39,
            HostBehavior::JiguanShenhuoCard => 40,
            HostBehavior::GujianFoodZiyangdan => 41,
            HostBehavior::GujianYunvyuanshen => 42,
            HostBehavior::GujianFoodShihuifenBan => 43,
            HostBehavior::GujianShihuifenCard => 44,
            HostBehavior::GujianLiufengsanCard => 45,
            HostBehavior::JiguanShatangCard => 46,
            HostBehavior::JiguanHeilongCard => 47,
            HostBehavior::JiguanLiutougeCard => 48,
            HostBehavior::GujianMutouEquipSkill => 49,
            HostBehavior::JiguanDinvCard => 50,
            HostBehavior::GujianYuhengEquipSkill => 51,
            HostBehavior::HearthDunpaiCard => 52,
            HostBehavior::HearthZhiliaoboCard => 53,
            HostBehavior::HearthYuansuCard => 54,
            HostBehavior::HearthXingjieCard => 55,
            HostBehavior::HearthChuansongCard => 56,
            HostBehavior::HearthTansheCard => 57,
            HostBehavior::HearthShijieshuCard => 58,
            HostBehavior::GwentYanzhiCard => 59,
            HostBehavior::GwentAozuCard => 60,
            HostBehavior::GwentWenyiCard => 61,
            HostBehavior::GwentZhihuanCard => 62,
            HostBehavior::GwentBaoxueCard => 63,
            HostBehavior::GwentQinpenState => 64,
            HostBehavior::GwentBirinState => 65,
            HostBehavior::GwentCiguState => 66,
            HostBehavior::GwentWeatherGrant => 67,
            HostBehavior::GwentKunenState => 68,
            HostBehavior::GwentKunenCard => 69,
            HostBehavior::GwentShanbengCard => 70,
            HostBehavior::GwentPoxiaoCard => 71,
            HostBehavior::GwentFengyinState => 72,
            HostBehavior::GwentZumoCard => 73,
            HostBehavior::HuanleMonkeyCard => 74,
            HostBehavior::HuanleXuelunyangCard => 75,
            HostBehavior::HuanleMianjuCard => 76,
            HostBehavior::ZhangbaSha => 77,
            HostBehavior::SwdLiuxinghuoyuCard => 78,
            HostBehavior::SwdShentouEquipSkill => 79,
            HostBehavior::SwdYufuluEquipSkill => 80,
            HostBehavior::SwdYangpijuanCard => 81,
            HostBehavior::SwdLangeguaiyiEquipSkill => 82,
            HostBehavior::MtgLongshanfengLand => 83,
            HostBehavior::MtgHaidaoLand => 84,
            HostBehavior::MtgShuimomuxueLand => 85,
            HostBehavior::MtgFeixuLand => 86,
            HostBehavior::MtgYixialanLand => 87,
            HostBehavior::MtgYixialanRecast => 88,
            HostBehavior::MtgShamolvzhouLand => 89,
            HostBehavior::MtgLinzhongjiantaLand => 90,
            HostBehavior::MtgLinzhongjiantaStrike => 91,
            HostBehavior::MtgBingheyaosaiLand => 92,
            HostBehavior::MtgYoulinLand => 93,
            HostBehavior::MtgYoulinDiscard => 94,
            HostBehavior::MtgLindixiliuLand => 95,
            HostBehavior::WudianCiqiu => 97,
            HostBehavior::WudianShuaiyan => 98,
            HostBehavior::WudianZhucheng => 99,
            HostBehavior::WudianZhuchengGuard => 100,
            HostBehavior::WudianKangyin => 101,
            HostBehavior::WudianChezhen => 102,
            HostBehavior::WudianJuedao => 103,
            HostBehavior::WudianGeju => 104,
            HostBehavior::WudianTuji => 105,
            HostBehavior::WudianMujing => 106,
            HostBehavior::WudianGuangshu => 107,
            HostBehavior::WudianGuangshuHeart => 108,
            HostBehavior::WudianGuangshuDiamond => 109,
            HostBehavior::WudianGuangshuClub => 110,
            HostBehavior::WudianGuangshuSpade => 111,
            HostBehavior::WudianZiyu => 112,
            HostBehavior::WudianShouhu => 113,
            HostBehavior::WudianDuwen => 114,
            HostBehavior::WudianPaotai => 115,
            HostBehavior::WudianPaotaiFire => 116,
            HostBehavior::WudianBaoleiSwitch => 117,
            HostBehavior::WudianZhenchaDraw => 118,
            HostBehavior::WudianYuedong => 119,
            HostBehavior::WudianKuoyin => 120,
            HostBehavior::WudianHuhuan => 121,
            HostBehavior::WudianJidong => 122,
            HostBehavior::WudianBaoxue => 123,
            HostBehavior::WudianLiangou => 124,
            HostBehavior::WudianXiyang => 125,
            HostBehavior::WudianFeiren => 126,
            HostBehavior::WudianFeidan => 127,
            HostBehavior::WudianZhiyuan => 128,
            HostBehavior::WudianShenqiang => 129,
            HostBehavior::WudianShihuo => 130,
            HostBehavior::WudianMengji => 131,
            HostBehavior::WudianYinbo => 132,
            HostBehavior::WudianQianggu => 133,
            HostBehavior::WudianQiangguGuard => 134,
            HostBehavior::WudianFeitiao => 135,
            HostBehavior::WudianPingzhang => 136,
            HostBehavior::WudianOwliyong => 137,
            HostBehavior::WudianYinshen => 138,
            HostBehavior::WudianXiandan => 139,
            HostBehavior::WudianJijia => 140,
            HostBehavior::XianjianLingxue => 141,
            HostBehavior::XianjianBinxin => 142,
            HostBehavior::XianjianHuimeng => 143,
            HostBehavior::XianjianTianshe => 144,
            HostBehavior::XianjianTianshe2 => 145,
            HostBehavior::XianjianFanling => 146,
            HostBehavior::WudianLongxiang => 147,
            HostBehavior::XianjianGuiyuan => 148,
            HostBehavior::XianjianTannang => 149,
            HostBehavior::XianjianFeixu => 150,
            HostBehavior::XianjianFeng => 151,
            HostBehavior::XianjianYa => 152,
            HostBehavior::XianjianSong => 153,
            HostBehavior::XianjianXjyufeng => 154,
            HostBehavior::XianjianXuanyan2 => 155,
            HostBehavior::XianjianXuanyan3 => 156,
            HostBehavior::XianjianNingbin => 157,
            HostBehavior::XianjianXuanyan => 158,
            HostBehavior::XianjianDiewu => 159,
            HostBehavior::XianjianLingyu => 160,
            HostBehavior::WudianLueying => 161,
            HostBehavior::XianjianAnwugu => 162,
            HostBehavior::XianjianAnwugu2 => 163,
            HostBehavior::XianjianTxianqu => 164,
            HostBehavior::XianjianFenshi => 165,
            HostBehavior::XianjianLonghuo => 166,
            HostBehavior::XianjianSajin => 167,
            HostBehavior::XianjianJtjubao => 168,
            HostBehavior::XianjianYuexing => 169,
            HostBehavior::XianjianMinsha => 170,
            HostBehavior::XianjianLinghuo => 171,
            HostBehavior::XianjianGuijin => 172,
            HostBehavior::XianjianZhuyue => 173,
            HostBehavior::XianjianGuanri => 174,
            HostBehavior::XianjianShuiyun => 175,
            HostBehavior::XianjianShuiyun5 => 176,
            HostBehavior::XianjianWangyou => 177,
            HostBehavior::XianjianWangyou2 => 178,
            HostBehavior::XianjianLingquan => 179,
            HostBehavior::XianjianShenwu => 180,
            HostBehavior::XianjianChengxin => 181,
            HostBehavior::XianjianShenmu => 182,
            HostBehavior::XianjianDuci => 183,
            HostBehavior::XianjianRunxin => 184,
            HostBehavior::XianjianXjzhimeng => 185,
            HostBehavior::XianjianXjzhimeng2 => 186,
            HostBehavior::XianjianSheling => 187,
            HostBehavior::XianjianHushaCount => 188,
            HostBehavior::XianjianHusha => 189,
            HostBehavior::XianjianChangnian => 190,
            HostBehavior::XianjianChangnian2 => 191,
            HostBehavior::XianjianShiying => 192,
            HostBehavior::XianjianZhaoyao => 193,
            HostBehavior::XianjianLiguang => 194,
            HostBehavior::XianjianXuanmo => 195,
            HostBehavior::XianjianSheying => 196,            HostBehavior::XianjianPoyun => 202,

            HostBehavior::XianjianHuahu => 197,
            HostBehavior::XianjianXtanxi => 198,
            HostBehavior::XianjianFenglue => 199,
            HostBehavior::XianjianLiefeng => 200,
            HostBehavior::XianjianTianjian => 201,
            HostBehavior::XianjianXshuangren => 203,
            HostBehavior::XianjianXshuangren2 => 204,
            HostBehavior::XianjianZhangmu => 205,
            HostBehavior::XianjianQianfang => 206,
            HostBehavior::XianjianXunying => 207,
            HostBehavior::XianjianTianxian => 208,
            HostBehavior::XianjianXjyingfeng => 209,
            HostBehavior::XianjianXiepan => 210,
            HostBehavior::XianjianLuanjian => 211,
            HostBehavior::XianjianLuanjian2 => 212,
            HostBehavior::XianjianCtianfu => 213,
            HostBehavior::XianjianCtianfu2 => 214,
            HostBehavior::XianjianCtianfu3 => 215,
            HostBehavior::SwdZhiluxiaohuCard => 216,
            HostBehavior::SwdMujiarenCard => 217,
            HostBehavior::SwdGouhunluoCard => 218,
            HostBehavior::SwdGouhunluoSub => 219,
            HostBehavior::SwdJiguantongCard => 220,
            HostBehavior::SwdJiguanfengCard => 221,
            HostBehavior::SwdFengyinState => 222,
            HostBehavior::SwdJiguanyuanCard => 223,
            HostBehavior::SwdJiguanyuanGain => 224,
            HostBehavior::SwdJiguanyaoshuCard => 225,
            HostBehavior::SwdJiguanyaoState => 226,
            HostBehavior::SwdXuanfengzhirenCard => 227,
            HostBehavior::SwdZhongxinghujiaCard => 228,
            HostBehavior::SwdXingtigaizaoCard => 229,
            HostBehavior::SwdShijianhuisuCard => 230,
            HostBehavior::SwdJinjilengdongCard => 231,
            HostBehavior::SwdShengxiuhaojiaoCard => 232,
            HostBehavior::SwdYinmilichangCard => 233,
            HostBehavior::SwdJiguanyaoGain => 234,
            HostBehavior::SwdJiguanshuCard => 235,
            HostBehavior::SwdTianfuzhenCard => 236,
            HostBehavior::SwdQixingzhenCard => 237,
            HostBehavior::SwdNiaoxiangzhenCard => 238,
            HostBehavior::SwdChangshezhenCard => 239,
            HostBehavior::SwdPozhenjueCard => 240,
        }
    }

    pub fn from_tag(t: u16) -> Option<Self> {
        Some(match t {
            0 => HostBehavior::ShaDamageForceDiscard,
            1 => HostBehavior::PhaseEndRecoverOrDraw,
            2 => HostBehavior::TargetDiscardEquipOrDamage,
            3 => HostBehavior::ShengkeCheck,
            4 => HostBehavior::ChangeCardNature,
            5 => HostBehavior::MaxRangeWeaponDamage,
            6 => HostBehavior::WangmeiZhikeCard,
            7 => HostBehavior::SijiDiscardDraw,
            8 => HostBehavior::SanjianWeaponDamage,
            9 => HostBehavior::MoshouImmunity,
            10 => HostBehavior::ShuigongCard,
            11 => HostBehavior::FudichouxinCompare,
            12 => HostBehavior::ChenhuodajieCard,
            13 => HostBehavior::TouliangSwap,
            14 => HostBehavior::LinghunzhihuoCard,
            15 => HostBehavior::ZhaomingdanCard,
            16 => HostBehavior::JihuocardDraw,
            17 => HostBehavior::GeanguanhuoCompare,
            18 => HostBehavior::CaochuanCard,
            19 => HostBehavior::ShezhanqunruCompare,
            20 => HostBehavior::GujianShujinsanCard,
            21 => HostBehavior::GujianMianliCard,
            22 => HostBehavior::GujianFeibiaoCard,
            23 => HostBehavior::JinlianzhuCancel,
            24 => HostBehavior::GujianFoodTanhuadong,
            25 => HostBehavior::GujianFoodYougeng,
            26 => HostBehavior::GujianFoodGrant,
            27 => HostBehavior::GujianFoodQingtuan,
            28 => HostBehavior::GujianFoodYuanbaorou,
            29 => HostBehavior::GujianFoodGudonggeng,
            30 => HostBehavior::GujianFoodMapodoufu,
            31 => HostBehavior::GujianFoodChunbing,
            32 => HostBehavior::GujianFoodLiyutang,
            33 => HostBehavior::GujianFoodMolicha,
            34 => HostBehavior::GujianFoodXiajiao,
            35 => HostBehavior::GujianFoodMizhilianou,
            36 => HostBehavior::GujianFoodLuyugeng,
            37 => HostBehavior::JiguanBingpoCard,
            38 => HostBehavior::JiguanQiankunCard,
            39 => HostBehavior::JiguanLongxuCard,
            40 => HostBehavior::JiguanShenhuoCard,
            41 => HostBehavior::GujianFoodZiyangdan,
            42 => HostBehavior::GujianYunvyuanshen,
            43 => HostBehavior::GujianFoodShihuifenBan,
            44 => HostBehavior::GujianShihuifenCard,
            45 => HostBehavior::GujianLiufengsanCard,
            46 => HostBehavior::JiguanShatangCard,
            47 => HostBehavior::JiguanHeilongCard,
            48 => HostBehavior::JiguanLiutougeCard,
            49 => HostBehavior::GujianMutouEquipSkill,
            50 => HostBehavior::JiguanDinvCard,
            51 => HostBehavior::GujianYuhengEquipSkill,
            52 => HostBehavior::HearthDunpaiCard,
            53 => HostBehavior::HearthZhiliaoboCard,
            54 => HostBehavior::HearthYuansuCard,
            55 => HostBehavior::HearthXingjieCard,
            56 => HostBehavior::HearthChuansongCard,
            57 => HostBehavior::HearthTansheCard,
            58 => HostBehavior::HearthShijieshuCard,
            59 => HostBehavior::GwentYanzhiCard,
            60 => HostBehavior::GwentAozuCard,
            61 => HostBehavior::GwentWenyiCard,
            62 => HostBehavior::GwentZhihuanCard,
            63 => HostBehavior::GwentBaoxueCard,
            64 => HostBehavior::GwentQinpenState,
            65 => HostBehavior::GwentBirinState,
            66 => HostBehavior::GwentCiguState,
            67 => HostBehavior::GwentWeatherGrant,
            68 => HostBehavior::GwentKunenState,
            69 => HostBehavior::GwentKunenCard,
            70 => HostBehavior::GwentShanbengCard,
            71 => HostBehavior::GwentPoxiaoCard,
            72 => HostBehavior::GwentFengyinState,
            73 => HostBehavior::GwentZumoCard,
            74 => HostBehavior::HuanleMonkeyCard,
            75 => HostBehavior::HuanleXuelunyangCard,
            76 => HostBehavior::HuanleMianjuCard,
            77 => HostBehavior::ZhangbaSha,
            78 => HostBehavior::SwdLiuxinghuoyuCard,
            79 => HostBehavior::SwdShentouEquipSkill,
            80 => HostBehavior::SwdYufuluEquipSkill,
            81 => HostBehavior::SwdYangpijuanCard,
            82 => HostBehavior::SwdLangeguaiyiEquipSkill,
            83 => HostBehavior::MtgLongshanfengLand,
            84 => HostBehavior::MtgHaidaoLand,
            85 => HostBehavior::MtgShuimomuxueLand,
            86 => HostBehavior::MtgFeixuLand,
            87 => HostBehavior::MtgYixialanLand,
            88 => HostBehavior::MtgYixialanRecast,
            89 => HostBehavior::MtgShamolvzhouLand,
            90 => HostBehavior::MtgLinzhongjiantaLand,
            91 => HostBehavior::MtgLinzhongjiantaStrike,
            92 => HostBehavior::MtgBingheyaosaiLand,
            93 => HostBehavior::MtgYoulinLand,
            94 => HostBehavior::MtgYoulinDiscard,
            95 => HostBehavior::MtgLindixiliuLand,
            97 => HostBehavior::WudianCiqiu,
            98 => HostBehavior::WudianShuaiyan,
            99 => HostBehavior::WudianZhucheng,
            100 => HostBehavior::WudianZhuchengGuard,
            101 => HostBehavior::WudianKangyin,
            102 => HostBehavior::WudianChezhen,
            103 => HostBehavior::WudianJuedao,
            104 => HostBehavior::WudianGeju,
            105 => HostBehavior::WudianTuji,
            106 => HostBehavior::WudianMujing,
            107 => HostBehavior::WudianGuangshu,
            108 => HostBehavior::WudianGuangshuHeart,
            109 => HostBehavior::WudianGuangshuDiamond,
            110 => HostBehavior::WudianGuangshuClub,
            111 => HostBehavior::WudianGuangshuSpade,
            112 => HostBehavior::WudianZiyu,
            113 => HostBehavior::WudianShouhu,
            114 => HostBehavior::WudianDuwen,
            115 => HostBehavior::WudianPaotai,
            116 => HostBehavior::WudianPaotaiFire,
            117 => HostBehavior::WudianBaoleiSwitch,
            118 => HostBehavior::WudianZhenchaDraw,
            119 => HostBehavior::WudianYuedong,
            120 => HostBehavior::WudianKuoyin,
            121 => HostBehavior::WudianHuhuan,
            122 => HostBehavior::WudianJidong,
            123 => HostBehavior::WudianBaoxue,
            124 => HostBehavior::WudianLiangou,
            125 => HostBehavior::WudianXiyang,
            126 => HostBehavior::WudianFeiren,
            127 => HostBehavior::WudianFeidan,
            128 => HostBehavior::WudianZhiyuan,
            129 => HostBehavior::WudianShenqiang,
            130 => HostBehavior::WudianShihuo,
            131 => HostBehavior::WudianMengji,
            132 => HostBehavior::WudianYinbo,
            133 => HostBehavior::WudianQianggu,
            134 => HostBehavior::WudianQiangguGuard,
            135 => HostBehavior::WudianFeitiao,
            136 => HostBehavior::WudianPingzhang,
            137 => HostBehavior::WudianOwliyong,
            138 => HostBehavior::WudianYinshen,
            139 => HostBehavior::WudianXiandan,
            140 => HostBehavior::WudianJijia,
            141 => HostBehavior::XianjianLingxue,
            142 => HostBehavior::XianjianBinxin,
            143 => HostBehavior::XianjianHuimeng,
            144 => HostBehavior::XianjianTianshe,
            145 => HostBehavior::XianjianTianshe2,
            146 => HostBehavior::XianjianFanling,
            147 => HostBehavior::WudianLongxiang,
            148 => HostBehavior::XianjianGuiyuan,
            149 => HostBehavior::XianjianTannang,
            150 => HostBehavior::XianjianFeixu,
            151 => HostBehavior::XianjianFeng,
            152 => HostBehavior::XianjianYa,
            153 => HostBehavior::XianjianSong,
            154 => HostBehavior::XianjianXjyufeng,
            155 => HostBehavior::XianjianXuanyan2,
            156 => HostBehavior::XianjianXuanyan3,
            157 => HostBehavior::XianjianNingbin,
            158 => HostBehavior::XianjianXuanyan,
            159 => HostBehavior::XianjianDiewu,
            160 => HostBehavior::XianjianLingyu,
            161 => HostBehavior::WudianLueying,
            162 => HostBehavior::XianjianAnwugu,
            163 => HostBehavior::XianjianAnwugu2,
            164 => HostBehavior::XianjianTxianqu,
            165 => HostBehavior::XianjianFenshi,
            166 => HostBehavior::XianjianLonghuo,
            167 => HostBehavior::XianjianSajin,
            168 => HostBehavior::XianjianJtjubao,
            169 => HostBehavior::XianjianYuexing,
            170 => HostBehavior::XianjianMinsha,
            171 => HostBehavior::XianjianLinghuo,
            172 => HostBehavior::XianjianGuijin,
            173 => HostBehavior::XianjianZhuyue,
            174 => HostBehavior::XianjianGuanri,
            175 => HostBehavior::XianjianShuiyun,
            176 => HostBehavior::XianjianShuiyun5,
            177 => HostBehavior::XianjianWangyou,
            178 => HostBehavior::XianjianWangyou2,
            179 => HostBehavior::XianjianLingquan,
            180 => HostBehavior::XianjianShenwu,
            181 => HostBehavior::XianjianChengxin,
            182 => HostBehavior::XianjianShenmu,
            183 => HostBehavior::XianjianDuci,
            184 => HostBehavior::XianjianRunxin,
            185 => HostBehavior::XianjianXjzhimeng,
            186 => HostBehavior::XianjianXjzhimeng2,
            187 => HostBehavior::XianjianSheling,
            188 => HostBehavior::XianjianHushaCount,
            189 => HostBehavior::XianjianHusha,
            190 => HostBehavior::XianjianChangnian,
            191 => HostBehavior::XianjianChangnian2,
            192 => HostBehavior::XianjianShiying,
            193 => HostBehavior::XianjianZhaoyao,
            194 => HostBehavior::XianjianLiguang,
            195 => HostBehavior::XianjianXuanmo,
            196 => HostBehavior::XianjianSheying,
            197 => HostBehavior::XianjianHuahu,
            198 => HostBehavior::XianjianXtanxi,
            199 => HostBehavior::XianjianFenglue,
            200 => HostBehavior::XianjianLiefeng,
            201 => HostBehavior::XianjianTianjian,
            202 => HostBehavior::XianjianPoyun,
            203 => HostBehavior::XianjianXshuangren,
            204 => HostBehavior::XianjianXshuangren2,
            205 => HostBehavior::XianjianZhangmu,
            206 => HostBehavior::XianjianQianfang,
            207 => HostBehavior::XianjianXunying,
            208 => HostBehavior::XianjianTianxian,
            209 => HostBehavior::XianjianXjyingfeng,
            210 => HostBehavior::XianjianXiepan,
            211 => HostBehavior::XianjianLuanjian,
            212 => HostBehavior::XianjianLuanjian2,
            213 => HostBehavior::XianjianCtianfu,
            214 => HostBehavior::XianjianCtianfu2,
            215 => HostBehavior::XianjianCtianfu3,
            216 => HostBehavior::SwdZhiluxiaohuCard,
            217 => HostBehavior::SwdMujiarenCard,
            218 => HostBehavior::SwdGouhunluoCard,
            219 => HostBehavior::SwdGouhunluoSub,
            220 => HostBehavior::SwdJiguantongCard,
            221 => HostBehavior::SwdJiguanfengCard,
            222 => HostBehavior::SwdFengyinState,
            223 => HostBehavior::SwdJiguanyuanCard,
            224 => HostBehavior::SwdJiguanyuanGain,
            225 => HostBehavior::SwdJiguanyaoshuCard,
            226 => HostBehavior::SwdJiguanyaoState,
            227 => HostBehavior::SwdXuanfengzhirenCard,
            228 => HostBehavior::SwdZhongxinghujiaCard,
            229 => HostBehavior::SwdXingtigaizaoCard,
            230 => HostBehavior::SwdShijianhuisuCard,
            231 => HostBehavior::SwdJinjilengdongCard,
            232 => HostBehavior::SwdShengxiuhaojiaoCard,
            233 => HostBehavior::SwdYinmilichangCard,
            234 => HostBehavior::SwdJiguanyaoGain,
            235 => HostBehavior::SwdJiguanshuCard,
            236 => HostBehavior::SwdTianfuzhenCard,
            237 => HostBehavior::SwdQixingzhenCard,
            238 => HostBehavior::SwdNiaoxiangzhenCard,
            239 => HostBehavior::SwdChangshezhenCard,
            240 => HostBehavior::SwdPozhenjueCard,
            241 => HostBehavior::YunchouXicai,
            242 => HostBehavior::YunchouDiykuanggu,
            243 => HostBehavior::YunchouXiongzi,
            244 => HostBehavior::YunchouYaliang,
            245 => HostBehavior::YunchouZonghuo,
            246 => HostBehavior::YunchouShaoying,
            247 => HostBehavior::YunchouDiyguhuo,
            248 => HostBehavior::YunchouJiaoxia,
            249 => HostBehavior::YunchouHonglian,
            250 => HostBehavior::YunchouLuweiyan,
            251 => HostBehavior::YunchouLuweiyan2,
            252 => HostBehavior::YunchouJieyan,
            253 => HostBehavior::YunchouYiesheng,
            254 => HostBehavior::YunchouDiychanyuan,
            255 => HostBehavior::WudianDiyjuntun,
            96 => HostBehavior::WudianJinyan,
            257 => HostBehavior::WudianLiduan,
            258 => HostBehavior::WudianJianbi,
            259 => HostBehavior::WudianFuchou,
            260 => HostBehavior::WudianFuchou2,
            261 => HostBehavior::WudianLiangceRemain,
            262 => HostBehavior::WudianGuihan,
            263 => HostBehavior::YunchouBeige,
            264 => HostBehavior::SwdTiebi,
            265 => HostBehavior::SwdDatong,
            266 => HostBehavior::SwdChengjian,
            267 => HostBehavior::SwdJufu,
            268 => HostBehavior::SwdHuajing,
            269 => HostBehavior::SwdHudun,
            270 => HostBehavior::SwdPoxing,
            271 => HostBehavior::SwdJiefen,
            272 => HostBehavior::SwdLiuhong,
            273 => HostBehavior::SwdMojian,
            274 => HostBehavior::SwdDaofa,
            275 => HostBehavior::SwdYulin,
            276 => HostBehavior::SwdNingshuang,
            277 => HostBehavior::SwdGongshen,
            278 => HostBehavior::SwdPingxu,
            279 => HostBehavior::SwdHuanling,
            280 => HostBehavior::SwdDuanyue,
            281 => HostBehavior::SwdTuzhen,
            282 => HostBehavior::SwdLuomei,
            283 => HostBehavior::SwdXingdian,
            284 => HostBehavior::SwdXielv,
            285 => HostBehavior::SwdZhuyu,
            286 => HostBehavior::SwdXuehuang,
            287 => HostBehavior::SwdHuanxia,
            288 => HostBehavior::SwdPoyue,
            289 => HostBehavior::SwdLianyaohuShouna,
            290 => HostBehavior::SwdLianyaohuLianhua,
            291 => HostBehavior::SwdHujing,
            292 => HostBehavior::SwdShennongdingCard,
            293 => HostBehavior::SwdNvwashiGuard,
            294 => HostBehavior::SwdKunlunjingcCard,
            295 => HostBehavior::SwdDonghuangzhongCard,
            296 => HostBehavior::SwdKongdongyinGuard,
            299 => HostBehavior::SwdNigongAccumulate,
            300 => HostBehavior::SwdNigongStrike,
            303 => HostBehavior::SwdSifeizhenmianCard,
            304 => HostBehavior::SwdQinglongzhiguiHeld,
            305 => HostBehavior::SwdZhuquezhizhangHeld,
            306 => HostBehavior::SwdBaishouzhihuHeld,
            307 => HostBehavior::SwdXuanwuzhihuangHeld,
            308 => HostBehavior::SwdHuanglinzhicongHeld,
            309 => HostBehavior::SwdShencaojieGuard,
            312 => HostBehavior::SwdJiyingHeld,
            313 => HostBehavior::SwdMufengHeld,
            314 => HostBehavior::SwdWangchenHeld,
            310 => HostBehavior::SwdYuchanSwap,
            301 => HostBehavior::GuisheqiCard,
            302 => HostBehavior::XianluhuiCard,
            _ => return None,
        })
    }

    pub fn from_key(key: &str) -> Option<Self> {
        HostBehavior::ALL.into_iter().find(|b| b.key() == key)
    }
    /// AOE 目标排除语义标记（目标排除原语首例——坚壁；catalog 通用查询
    /// `aoe_excludes_target` 消费，内核不点名扩展技能）。
    pub const fn aoe_exclude(self) -> bool {
        matches!(self, Self::WudianJianbi)
    }
    /// 五谷剩余归获语义标记（玩点论杀批 15——良策；catalog 通用查询
    /// `wugu_remain_beneficiary` 消费）。
    pub const fn wugu_remain_gain(self) -> bool {
        matches!(self, Self::WudianLiangceRemain)
    }
    pub const ALL: [HostBehavior; 311] = [
        HostBehavior::ShaDamageForceDiscard,
        HostBehavior::PhaseEndRecoverOrDraw,
        HostBehavior::TargetDiscardEquipOrDamage,
        HostBehavior::ShengkeCheck,
        HostBehavior::ChangeCardNature,
        HostBehavior::MaxRangeWeaponDamage,
        HostBehavior::WangmeiZhikeCard,
        HostBehavior::SijiDiscardDraw,
        HostBehavior::SanjianWeaponDamage,
        HostBehavior::MoshouImmunity,
        HostBehavior::ShuigongCard,
        HostBehavior::FudichouxinCompare,
        HostBehavior::ChenhuodajieCard,
        HostBehavior::TouliangSwap,
        HostBehavior::LinghunzhihuoCard,
        HostBehavior::ZhaomingdanCard,
        HostBehavior::JihuocardDraw,
        HostBehavior::GeanguanhuoCompare,
        HostBehavior::CaochuanCard,
        HostBehavior::ShezhanqunruCompare,
        HostBehavior::GujianShujinsanCard,
        HostBehavior::GujianMianliCard,
        HostBehavior::GujianFeibiaoCard,
        HostBehavior::JinlianzhuCancel,
        HostBehavior::GujianFoodTanhuadong,
        HostBehavior::GujianFoodYougeng,
        HostBehavior::GujianFoodGrant,
        HostBehavior::GujianFoodQingtuan,
        HostBehavior::GujianFoodYuanbaorou,
        HostBehavior::GujianFoodGudonggeng,
        HostBehavior::GujianFoodMapodoufu,
        HostBehavior::GujianFoodChunbing,
        HostBehavior::GujianFoodLiyutang,
        HostBehavior::GujianFoodMolicha,
        HostBehavior::GujianFoodXiajiao,
        HostBehavior::GujianFoodMizhilianou,
        HostBehavior::GujianFoodLuyugeng,
        HostBehavior::JiguanBingpoCard,
        HostBehavior::JiguanQiankunCard,
        HostBehavior::JiguanLongxuCard,
        HostBehavior::JiguanShenhuoCard,
        HostBehavior::GujianFoodZiyangdan,
        HostBehavior::GujianYunvyuanshen,
        HostBehavior::GujianFoodShihuifenBan,
        HostBehavior::GujianShihuifenCard,
        HostBehavior::GujianLiufengsanCard,
        HostBehavior::JiguanShatangCard,
        HostBehavior::JiguanHeilongCard,
        HostBehavior::JiguanLiutougeCard,
        HostBehavior::GujianMutouEquipSkill,
        HostBehavior::JiguanDinvCard,
        HostBehavior::GujianYuhengEquipSkill,
        HostBehavior::HearthDunpaiCard,
        HostBehavior::HearthZhiliaoboCard,
        HostBehavior::HearthYuansuCard,
        HostBehavior::HearthXingjieCard,
        HostBehavior::HearthChuansongCard,
        HostBehavior::HearthTansheCard,
        HostBehavior::HearthShijieshuCard,
        HostBehavior::GwentYanzhiCard,
        HostBehavior::GwentAozuCard,
        HostBehavior::GwentWenyiCard,
        HostBehavior::GwentZhihuanCard,
        HostBehavior::GwentBaoxueCard,
        HostBehavior::GwentQinpenState,
        HostBehavior::GwentBirinState,
        HostBehavior::GwentCiguState,
        HostBehavior::GwentWeatherGrant,
        HostBehavior::GwentKunenState,
        HostBehavior::GwentKunenCard,
        HostBehavior::GwentShanbengCard,
        HostBehavior::GwentPoxiaoCard,
        HostBehavior::GwentFengyinState,
        HostBehavior::GwentZumoCard,
        HostBehavior::HuanleMonkeyCard,
        HostBehavior::HuanleXuelunyangCard,
        HostBehavior::HuanleMianjuCard,
        HostBehavior::ZhangbaSha,
        HostBehavior::SwdLiuxinghuoyuCard,
        HostBehavior::SwdShentouEquipSkill,
        HostBehavior::SwdYufuluEquipSkill,
        HostBehavior::SwdYangpijuanCard,
        HostBehavior::SwdLangeguaiyiEquipSkill,
        HostBehavior::MtgLongshanfengLand,
        HostBehavior::MtgHaidaoLand,
        HostBehavior::MtgShuimomuxueLand,
        HostBehavior::MtgFeixuLand,
        HostBehavior::MtgYixialanLand,
        HostBehavior::MtgYixialanRecast,
        HostBehavior::MtgShamolvzhouLand,
        HostBehavior::MtgLinzhongjiantaLand,
        HostBehavior::MtgLinzhongjiantaStrike,
        HostBehavior::MtgBingheyaosaiLand,
        HostBehavior::MtgYoulinLand,
        HostBehavior::MtgYoulinDiscard,
        HostBehavior::MtgLindixiliuLand,
        HostBehavior::WudianCiqiu,
        HostBehavior::WudianShuaiyan,
        HostBehavior::WudianZhucheng,
        HostBehavior::WudianZhuchengGuard,
        HostBehavior::WudianKangyin,
        HostBehavior::WudianChezhen,
        HostBehavior::WudianJuedao,
        HostBehavior::WudianGeju,
        HostBehavior::WudianTuji,
        HostBehavior::WudianMujing,
        HostBehavior::WudianGuangshu,
        HostBehavior::WudianGuangshuHeart,
        HostBehavior::WudianGuangshuDiamond,
        HostBehavior::WudianGuangshuClub,
        HostBehavior::WudianGuangshuSpade,
        HostBehavior::WudianZiyu,
        HostBehavior::WudianShouhu,
        HostBehavior::WudianDuwen,
        HostBehavior::WudianPaotai,
        HostBehavior::WudianPaotaiFire,
        HostBehavior::WudianBaoleiSwitch,
        HostBehavior::WudianZhenchaDraw,
        HostBehavior::WudianYuedong,
        HostBehavior::WudianKuoyin,
        HostBehavior::WudianHuhuan,
        HostBehavior::WudianJidong,
        HostBehavior::WudianBaoxue,
        HostBehavior::WudianLiangou,
        HostBehavior::WudianXiyang,
        HostBehavior::WudianFeiren,
        HostBehavior::WudianFeidan,
        HostBehavior::WudianZhiyuan,
        HostBehavior::WudianShenqiang,
        HostBehavior::WudianShihuo,
        HostBehavior::WudianMengji,
        HostBehavior::WudianYinbo,
        HostBehavior::WudianQianggu,
        HostBehavior::WudianQiangguGuard,
        HostBehavior::WudianFeitiao,
        HostBehavior::WudianPingzhang,
        HostBehavior::WudianOwliyong,
        HostBehavior::WudianYinshen,
        HostBehavior::WudianXiandan,
        HostBehavior::WudianJijia,
        HostBehavior::XianjianLingxue,
        HostBehavior::XianjianBinxin,
        HostBehavior::XianjianHuimeng,
        HostBehavior::XianjianTianshe,
        HostBehavior::XianjianTianshe2,
        HostBehavior::XianjianFanling,
        HostBehavior::WudianLongxiang,
        HostBehavior::XianjianGuiyuan,
        HostBehavior::XianjianTannang,
        HostBehavior::XianjianFeixu,
        HostBehavior::XianjianFeng,
        HostBehavior::XianjianYa,
        HostBehavior::XianjianSong,
        HostBehavior::XianjianXjyufeng,
        HostBehavior::XianjianXuanyan2,
        HostBehavior::XianjianXuanyan3,
        HostBehavior::XianjianNingbin,
        HostBehavior::XianjianXuanyan,
        HostBehavior::XianjianDiewu,
        HostBehavior::XianjianLingyu,
        HostBehavior::WudianLueying,
        HostBehavior::XianjianAnwugu,
        HostBehavior::XianjianAnwugu2,
        HostBehavior::XianjianTxianqu,
        HostBehavior::XianjianFenshi,
        HostBehavior::XianjianLonghuo,
        HostBehavior::XianjianSajin,
        HostBehavior::XianjianJtjubao,
        HostBehavior::XianjianYuexing,
        HostBehavior::XianjianMinsha,
        HostBehavior::XianjianLinghuo,
        HostBehavior::XianjianGuijin,
        HostBehavior::XianjianZhuyue,
        HostBehavior::XianjianGuanri,
        HostBehavior::XianjianShuiyun,
        HostBehavior::XianjianShuiyun5,
        HostBehavior::XianjianWangyou,
        HostBehavior::XianjianWangyou2,
        HostBehavior::XianjianLingquan,
        HostBehavior::XianjianShenwu,
        HostBehavior::XianjianChengxin,
        HostBehavior::XianjianShenmu,
        HostBehavior::XianjianDuci,
        HostBehavior::XianjianRunxin,
        HostBehavior::XianjianXjzhimeng,
        HostBehavior::XianjianXjzhimeng2,
        HostBehavior::XianjianSheling,
        HostBehavior::XianjianHushaCount,
        HostBehavior::XianjianHusha,
        HostBehavior::XianjianChangnian,
        HostBehavior::XianjianChangnian2,
        HostBehavior::XianjianShiying,
        HostBehavior::XianjianZhaoyao,
        HostBehavior::XianjianLiguang,
        HostBehavior::XianjianXuanmo,
        HostBehavior::XianjianSheying,
        HostBehavior::XianjianHuahu,
        HostBehavior::XianjianXtanxi,
        HostBehavior::XianjianFenglue,
        HostBehavior::XianjianLiefeng,
        HostBehavior::XianjianTianjian,
        HostBehavior::XianjianPoyun,
        HostBehavior::XianjianXshuangren,
        HostBehavior::XianjianXshuangren2,
        HostBehavior::XianjianZhangmu,
        HostBehavior::XianjianQianfang,
        HostBehavior::XianjianXunying,
        HostBehavior::XianjianTianxian,
        HostBehavior::XianjianXjyingfeng,
        HostBehavior::XianjianXiepan,
        HostBehavior::XianjianLuanjian,
        HostBehavior::XianjianLuanjian2,
        HostBehavior::XianjianCtianfu,
        HostBehavior::XianjianCtianfu2,
        HostBehavior::XianjianCtianfu3,
        HostBehavior::SwdZhiluxiaohuCard,
        HostBehavior::SwdMujiarenCard,
        HostBehavior::SwdGouhunluoCard,
        HostBehavior::SwdGouhunluoSub,
        HostBehavior::SwdJiguantongCard,
        HostBehavior::SwdJiguanfengCard,
        HostBehavior::SwdFengyinState,
        HostBehavior::SwdJiguanyuanCard,
        HostBehavior::SwdJiguanyuanGain,
        HostBehavior::SwdJiguanyaoshuCard,
        HostBehavior::SwdJiguanyaoState,
        HostBehavior::SwdXuanfengzhirenCard,
        HostBehavior::SwdZhongxinghujiaCard,
        HostBehavior::SwdXingtigaizaoCard,
        HostBehavior::SwdShijianhuisuCard,
        HostBehavior::SwdJinjilengdongCard,
        HostBehavior::SwdShengxiuhaojiaoCard,
        HostBehavior::SwdYinmilichangCard,
        HostBehavior::SwdJiguanyaoGain,
        HostBehavior::SwdJiguanshuCard,
        HostBehavior::SwdTianfuzhenCard,
        HostBehavior::SwdQixingzhenCard,
        HostBehavior::SwdNiaoxiangzhenCard,
        HostBehavior::SwdChangshezhenCard,
        HostBehavior::SwdPozhenjueCard,
        HostBehavior::YunchouXicai,
        HostBehavior::YunchouDiykuanggu,
        HostBehavior::YunchouXiongzi,
        HostBehavior::YunchouYaliang,
        HostBehavior::YunchouZonghuo,
        HostBehavior::YunchouShaoying,
        HostBehavior::YunchouDiyguhuo,
        HostBehavior::YunchouJiaoxia,
        HostBehavior::YunchouHonglian,
        HostBehavior::YunchouLuweiyan,
        HostBehavior::YunchouLuweiyan2,
        HostBehavior::YunchouJieyan,
        HostBehavior::YunchouYiesheng,
        HostBehavior::YunchouDiychanyuan,
        HostBehavior::WudianDiyjuntun,
        HostBehavior::WudianJinyan,
        HostBehavior::WudianLiduan,
        HostBehavior::WudianJianbi,
        HostBehavior::WudianFuchou,
        HostBehavior::WudianFuchou2,
        HostBehavior::WudianLiangceRemain,
        HostBehavior::WudianGuihan,
        HostBehavior::YunchouBeige,
        HostBehavior::SwdTiebi,
        HostBehavior::SwdDatong,
        HostBehavior::SwdChengjian,
        HostBehavior::SwdJufu,
        HostBehavior::SwdHuajing,
        HostBehavior::SwdHudun,
        HostBehavior::SwdPoxing,
        HostBehavior::SwdJiefen,
        HostBehavior::SwdLiuhong,
        HostBehavior::SwdMojian,
        HostBehavior::SwdDaofa,
        HostBehavior::SwdYulin,
        HostBehavior::SwdNingshuang,
        HostBehavior::SwdGongshen,
        HostBehavior::SwdPingxu,
        HostBehavior::SwdHuanling,
        HostBehavior::SwdDuanyue,
        HostBehavior::SwdTuzhen,
        HostBehavior::SwdLuomei,
        HostBehavior::SwdXingdian,
        HostBehavior::SwdXielv,
        HostBehavior::SwdZhuyu,
        HostBehavior::SwdXuehuang,
        HostBehavior::SwdHuanxia,
        HostBehavior::SwdPoyue,
        HostBehavior::SwdLianyaohuShouna,
        HostBehavior::SwdLianyaohuLianhua,
        HostBehavior::SwdHujing,
        HostBehavior::SwdShennongdingCard,
        HostBehavior::SwdNvwashiGuard,
        HostBehavior::SwdKunlunjingcCard,
        HostBehavior::SwdDonghuangzhongCard,
        HostBehavior::SwdKongdongyinGuard,
        HostBehavior::SwdNigongAccumulate,
        HostBehavior::SwdNigongStrike,
        HostBehavior::SwdSifeizhenmianCard,
        HostBehavior::SwdQinglongzhiguiHeld,
        HostBehavior::SwdZhuquezhizhangHeld,
        HostBehavior::SwdBaishouzhihuHeld,
        HostBehavior::SwdXuanwuzhihuangHeld,
        HostBehavior::SwdHuanglinzhicongHeld,
        HostBehavior::SwdShencaojieGuard,
        HostBehavior::SwdJiyingHeld,
        HostBehavior::SwdMufengHeld,
        HostBehavior::SwdWangchenHeld,
        HostBehavior::SwdYuchanSwap,
        HostBehavior::GuisheqiCard,
        HostBehavior::XianluhuiCard,
    ];

    /// 扩展包数据 `mapping` 键（宿主 ABI：与 tag 一一对应，只增不改）。
    pub const fn key(self) -> &'static str {
        match self {
            HostBehavior::YunchouXicai => "host:xicai",
            HostBehavior::YunchouDiykuanggu => "host:diykuanggu",
            HostBehavior::YunchouXiongzi => "host:xiongzi",
            HostBehavior::YunchouYaliang => "host:yaliang",
            HostBehavior::YunchouZonghuo => "host:zonghuo",
            HostBehavior::YunchouShaoying => "host:shaoying",
            HostBehavior::YunchouDiyguhuo => "host:diyguhuo",
            HostBehavior::YunchouJiaoxia => "host:jiaoxia",
            HostBehavior::YunchouHonglian => "host:honglian",
            HostBehavior::YunchouLuweiyan => "host:luweiyan",
            HostBehavior::YunchouLuweiyan2 => "host:luweiyan2",
            HostBehavior::YunchouJieyan => "host:jieyan",
            HostBehavior::YunchouYiesheng => "host:yiesheng",
            HostBehavior::YunchouDiychanyuan => "host:diychanyuan",
            HostBehavior::WudianDiyjuntun => "host:wudian_diyjuntun",
            HostBehavior::WudianJinyan => "host:wudian_jinyan",
            HostBehavior::WudianLiduan => "host:wudian_liduan",
            HostBehavior::WudianJianbi => "host:wudian_jianbi",
            HostBehavior::WudianFuchou => "host:fuchou",
            HostBehavior::WudianFuchou2 => "host:fuchou2",
            HostBehavior::WudianLiangceRemain => "host:liangce_remain",
            HostBehavior::WudianGuihan => "host:guihan",
            HostBehavior::YunchouBeige => "host:beige",
            HostBehavior::SwdTiebi => "host:tiebi",
            HostBehavior::SwdDatong => "host:datong",
            HostBehavior::SwdChengjian => "host:chengjian",
            HostBehavior::SwdJufu => "host:jufu",
            HostBehavior::SwdHuajing => "host:huajing",
            HostBehavior::SwdHudun => "host:hudun",
            HostBehavior::SwdPoxing => "host:poxing",
            HostBehavior::SwdLiuhong => "host:swdliuhong",
            HostBehavior::SwdMojian => "host:mojian",
            HostBehavior::SwdDaofa => "host:daofa",
            HostBehavior::SwdYulin => "host:yulin",
            HostBehavior::SwdNingshuang => "host:ningshuang",
            HostBehavior::SwdGongshen => "host:gongshen",
            HostBehavior::SwdPingxu => "host:pingxu",
            HostBehavior::SwdHuanling => "host:huanling",
            HostBehavior::SwdDuanyue => "host:duanyue",
            HostBehavior::SwdTuzhen => "host:tuzhen",
            HostBehavior::SwdLuomei => "host:luomei",
            HostBehavior::SwdXingdian => "host:xingdian",
            HostBehavior::SwdXielv => "host:xielv",
            HostBehavior::SwdZhuyu => "host:zhuyu",
            HostBehavior::SwdXuehuang => "host:xuehuang",
            HostBehavior::SwdHuanxia => "host:huanxia",
            HostBehavior::SwdPoyue => "host:poyue",
            HostBehavior::SwdLianyaohuShouna => "host:lianyaohu_shouna",
            HostBehavior::SwdLianyaohuLianhua => "host:lianyaohu_lianhua",
            HostBehavior::SwdHujing => "host:hujing",
            HostBehavior::SwdShennongdingCard => "host:shennongding",
            HostBehavior::SwdNvwashiGuard => "host:nvwashi",
            HostBehavior::SwdKunlunjingcCard => "host:kunlunjingc",
            HostBehavior::SwdDonghuangzhongCard => "host:donghuangzhong",
            HostBehavior::SwdKongdongyinGuard => "host:kongdongyin",
            HostBehavior::SwdNigongAccumulate => "nigong_accumulate",
            HostBehavior::SwdNigongStrike => "nigong_strike",
            HostBehavior::SwdSifeizhenmianCard => "yiluan",
            HostBehavior::SwdQinglongzhiguiHeld => "qinglongzhigui",
            HostBehavior::SwdZhuquezhizhangHeld => "zhuquezhizhang",
            HostBehavior::SwdBaishouzhihuHeld => "baishouzhihu",
            HostBehavior::SwdXuanwuzhihuangHeld => "xuanwuzhihuang",
            HostBehavior::SwdHuanglinzhicongHeld => "huanglinzhicong",
            HostBehavior::SwdShencaojieGuard => "shencaojie",
            HostBehavior::SwdJiyingHeld => "host:jiying",
            HostBehavior::SwdMufengHeld => "host:mufeng",
            HostBehavior::SwdWangchenHeld => "host:wangchen",
            HostBehavior::SwdYuchanSwap => "yuchan_swap",
            HostBehavior::GuisheqiCard => "guisheqi_card",
            HostBehavior::XianluhuiCard => "xianluhui_card",
            HostBehavior::SwdJiefen => "host:jiefen",
            HostBehavior::ShaDamageForceDiscard => "host:sha_damage_force_discard",
            HostBehavior::PhaseEndRecoverOrDraw => "host:phase_end_recover_or_draw",
            HostBehavior::TargetDiscardEquipOrDamage => "host:target_discard_equip_or_damage",
            HostBehavior::ShengkeCheck => "host:shengke_check",
            HostBehavior::ChangeCardNature => "host:change_card_nature",
            HostBehavior::MaxRangeWeaponDamage => "host:max_range_weapon_damage",
            HostBehavior::WangmeiZhikeCard => "host:wangmeizhike_card",
            HostBehavior::SijiDiscardDraw => "host:siji_discard_draw",
            HostBehavior::SanjianWeaponDamage => "host:sanjian_weapon",
            HostBehavior::MoshouImmunity => "host:moshou_immunity",
            HostBehavior::ShuigongCard => "host:shuigong_card",
            HostBehavior::FudichouxinCompare => "host:fudichouxin_compare",
            HostBehavior::ChenhuodajieCard => "host:chenhuodajie_card",
            HostBehavior::TouliangSwap => "host:touliang_swap",
            HostBehavior::LinghunzhihuoCard => "host:linghunzhihuo_card",
            HostBehavior::ZhaomingdanCard => "host:zhaomingdan_card",
            HostBehavior::JihuocardDraw => "host:jihuocard_draw",
            HostBehavior::GeanguanhuoCompare => "host:geanguanhuo_compare",
            HostBehavior::CaochuanCard => "host:caochuan_card",
            HostBehavior::ShezhanqunruCompare => "host:shezhanqunru_compare",
            HostBehavior::GujianShujinsanCard => "host:gujian_shujinsan_card",
            HostBehavior::GujianMianliCard => "host:gujian_mianli_card",
            HostBehavior::GujianFeibiaoCard => "host:gujian_feibiao_card",
            HostBehavior::JinlianzhuCancel => "host:jinlianzhu_cancel",
            HostBehavior::GujianFoodTanhuadong => "host:gujian_food_tanhuadong",
            HostBehavior::GujianFoodYougeng => "host:gujian_food_yougeng",
            HostBehavior::GujianFoodGrant => "host:gujian_food_grant",
            HostBehavior::GujianFoodQingtuan => "host:gujian_food_qingtuan",
            HostBehavior::GujianFoodYuanbaorou => "host:gujian_food_yuanbaorou",
            HostBehavior::GujianFoodGudonggeng => "host:gujian_food_gudonggeng",
            HostBehavior::GujianFoodMapodoufu => "host:gujian_food_mapodoufu",
            HostBehavior::GujianFoodChunbing => "host:gujian_food_chunbing",
            HostBehavior::GujianFoodLiyutang => "host:gujian_food_liyutang",
            HostBehavior::GujianFoodMolicha => "host:gujian_food_molicha",
            HostBehavior::GujianFoodXiajiao => "host:gujian_food_xiajiao",
            HostBehavior::GujianFoodMizhilianou => "host:gujian_food_mizhilianou",
            HostBehavior::GujianFoodLuyugeng => "host:gujian_food_luyugeng",
            HostBehavior::JiguanBingpoCard => "host:jiguan_bingpo_card",
            HostBehavior::JiguanQiankunCard => "host:jiguan_qiankun_card",
            HostBehavior::JiguanLongxuCard => "host:jiguan_longxu_card",
            HostBehavior::JiguanShenhuoCard => "host:jiguan_shenhuo_card",
            HostBehavior::GujianFoodZiyangdan => "host:gujian_food_ziyangdan",
            HostBehavior::GujianYunvyuanshen => "host:gujian_yunvyuanshen",
            HostBehavior::GujianFoodShihuifenBan => "host:gujian_food_shihuifen_ban",
            HostBehavior::GujianShihuifenCard => "host:gujian_shihuifen_card",
            HostBehavior::GujianLiufengsanCard => "host:gujian_liufengsan_card",
            HostBehavior::JiguanShatangCard => "host:jiguan_shatang_card",
            HostBehavior::JiguanHeilongCard => "host:jiguan_heilong_card",
            HostBehavior::JiguanLiutougeCard => "host:jiguan_liutouge_card",
            HostBehavior::GujianMutouEquipSkill => "host:gujian_mutou_equip_skill",
            HostBehavior::JiguanDinvCard => "host:jiguan_dinv_card",
            HostBehavior::GujianYuhengEquipSkill => "host:gujian_yuheng_equip_skill",
            HostBehavior::HearthDunpaiCard => "host:hearth_dunpai_card",
            HostBehavior::HearthZhiliaoboCard => "host:hearth_zhiliaobo_card",
            HostBehavior::HearthYuansuCard => "host:hearth_yuansu_card",
            HostBehavior::HearthXingjieCard => "host:hearth_xingjie_card",
            HostBehavior::HearthChuansongCard => "host:hearth_chuansong_card",
            HostBehavior::HearthTansheCard => "host:hearth_tanshe_card",
            HostBehavior::HearthShijieshuCard => "host:hearth_shijieshu_card",
            HostBehavior::GwentYanzhiCard => "host:gwent_yanzhi_card",
            HostBehavior::GwentAozuCard => "host:gwent_aozu_card",
            HostBehavior::GwentWenyiCard => "host:gwent_wenyi_card",
            HostBehavior::GwentZhihuanCard => "host:gwent_zhihuan_card",
            HostBehavior::GwentBaoxueCard => "host:gwent_baoxue_card",
            HostBehavior::GwentQinpenState => "host:gwent_qinpen_state",
            HostBehavior::GwentBirinState => "host:gwent_birin_state",
            HostBehavior::GwentCiguState => "host:gwent_cigu_state",
            HostBehavior::GwentWeatherGrant => "host:gwent_weather_grant",
            HostBehavior::GwentKunenState => "host:gwent_kunen_state",
            HostBehavior::GwentKunenCard => "host:gwent_kunen_card",
            HostBehavior::GwentShanbengCard => "host:gwent_shanbeng_card",
            HostBehavior::GwentPoxiaoCard => "host:gwent_poxiao_card",
            HostBehavior::GwentFengyinState => "host:gwent_fengyin_state",
            HostBehavior::GwentZumoCard => "host:gwent_zumo_card",
            HostBehavior::HuanleMonkeyCard => "host:huanle_monkey",
            HostBehavior::HuanleXuelunyangCard => "host:huanle_xuelunyang",
            HostBehavior::HuanleMianjuCard => "host:huanle_mianju",
            HostBehavior::ZhangbaSha => "host:zhangba_sha",
            HostBehavior::SwdLiuxinghuoyuCard => "host:swd_liuxinghuoyu_card",
            HostBehavior::SwdShentouEquipSkill => "host:swd_shentou_equip_skill",
            HostBehavior::SwdYufuluEquipSkill => "host:swd_yufulu_equip_skill",
            HostBehavior::SwdYangpijuanCard => "host:swd_yangpijuan_card",
            HostBehavior::SwdLangeguaiyiEquipSkill => "host:swd_langeguaiyi_equip_skill",
            HostBehavior::MtgLongshanfengLand => "host:mtg_longshanfeng_land",
            HostBehavior::MtgHaidaoLand => "host:mtg_haidao_land",
            HostBehavior::MtgShuimomuxueLand => "host:mtg_shuimomuxue_land",
            HostBehavior::MtgFeixuLand => "host:mtg_feixu_land",
            HostBehavior::MtgYixialanLand => "host:mtg_yixialan_land",
            HostBehavior::MtgYixialanRecast => "host:mtg_yixialan_recast",
            HostBehavior::MtgShamolvzhouLand => "host:mtg_shamolvzhou_land",
            HostBehavior::MtgLinzhongjiantaLand => "host:mtg_linzhongjianta_land",
            HostBehavior::MtgLinzhongjiantaStrike => "host:mtg_linzhongjianta_strike",
            HostBehavior::MtgBingheyaosaiLand => "host:mtg_bingheyaosai_land",
            HostBehavior::MtgYoulinLand => "host:mtg_youlin_land",
            HostBehavior::MtgYoulinDiscard => "host:mtg_youlin_discover",
            HostBehavior::MtgLindixiliuLand => "host:mtg_lindixiliu_land",
            HostBehavior::WudianCiqiu => "host:wudian_ciqiu",
            HostBehavior::WudianShuaiyan => "host:wudian_shuaiyan",
            HostBehavior::WudianZhucheng => "host:wudian_zhucheng",
            HostBehavior::WudianZhuchengGuard => "host:wudian_zhucheng_guard",
            HostBehavior::WudianKangyin => "host:wudian_kangyin",
            HostBehavior::WudianChezhen => "host:wudian_chezhen",
            HostBehavior::WudianJuedao => "host:wudian_juedao",
            HostBehavior::WudianGeju => "host:wudian_geju",
            HostBehavior::WudianTuji => "host:wudian_tuji",
            HostBehavior::WudianMujing => "host:wudian_mujing",
            HostBehavior::WudianGuangshu => "host:wudian_guangshu",
            HostBehavior::WudianGuangshuHeart => "host:wudian_guangshu_heart",
            HostBehavior::WudianGuangshuDiamond => "host:wudian_guangshu_diamond",
            HostBehavior::WudianGuangshuClub => "host:wudian_guangshu_club",
            HostBehavior::WudianGuangshuSpade => "host:wudian_guangshu_spade",
            HostBehavior::WudianZiyu => "host:wudian_ziyu",
            HostBehavior::WudianShouhu => "host:wudian_shouhu",
            HostBehavior::WudianDuwen => "host:wudian_duwen",
            HostBehavior::WudianPaotai => "host:wudian_paotai",
            HostBehavior::WudianPaotaiFire => "host:wudian_paotai_fire",
            HostBehavior::WudianBaoleiSwitch => "host:wudian_baolei_switch",
            HostBehavior::WudianZhenchaDraw => "host:wudian_zhencha_draw",
            HostBehavior::WudianYuedong => "host:wudian_yuedong",
            HostBehavior::WudianKuoyin => "host:wudian_kuoyin",
            HostBehavior::WudianHuhuan => "host:wudian_huhuan",
            HostBehavior::WudianJidong => "host:wudian_jidong",
            HostBehavior::WudianBaoxue => "host:wudian_baoxue",
            HostBehavior::WudianLiangou => "host:wudian_liangou",
            HostBehavior::WudianXiyang => "host:wudian_xiyang",
            HostBehavior::WudianFeiren => "host:wudian_feiren",
            HostBehavior::WudianFeidan => "host:wudian_feidan",
            HostBehavior::WudianZhiyuan => "host:wudian_zhiyuan",
            HostBehavior::WudianShenqiang => "host:wudian_shenqiang",
            HostBehavior::WudianShihuo => "host:wudian_shihuo",
            HostBehavior::WudianMengji => "host:wudian_mengji",
            HostBehavior::WudianYinbo => "host:wudian_yinbo",
            HostBehavior::WudianQianggu => "host:wudian_qianggu",
            HostBehavior::WudianQiangguGuard => "host:wudian_qianggu_guard",
            HostBehavior::WudianFeitiao => "host:wudian_feitiao",
            HostBehavior::WudianPingzhang => "host:wudian_pingzhang",
            HostBehavior::WudianOwliyong => "host:wudian_owliyong",
            HostBehavior::WudianYinshen => "host:wudian_yinshen",
            HostBehavior::WudianXiandan => "host:wudian_xiandan",
            HostBehavior::WudianJijia => "host:wudian_jijia",
            HostBehavior::XianjianLingxue => "host:xj_lingxue",
            HostBehavior::XianjianBinxin => "host:xj_binxin",
            HostBehavior::XianjianHuimeng => "host:xj_huimeng",
            HostBehavior::XianjianTianshe => "host:xj_tianshe",
            HostBehavior::XianjianTianshe2 => "host:xj_tianshe2",
            HostBehavior::XianjianFanling => "host:xj_fanling",
            HostBehavior::WudianLongxiang => "host:xj_longxiang",
            HostBehavior::XianjianGuiyuan => "host:xj_guiyuan",
            HostBehavior::XianjianTannang => "host:xj_tannang",
            HostBehavior::XianjianFeixu => "host:xj_feixu",
            HostBehavior::XianjianFeng => "host:xj_feng",
            HostBehavior::XianjianYa => "host:xj_ya",
            HostBehavior::XianjianSong => "host:xj_song",
            HostBehavior::XianjianXjyufeng => "host:xj_xjyufeng",
            HostBehavior::XianjianXuanyan2 => "host:xj_xuanyan2",
            HostBehavior::XianjianXuanyan3 => "host:xj_xuanyan3",
            HostBehavior::XianjianNingbin => "host:xj_ningbin",
            HostBehavior::XianjianXuanyan => "host:xj_xuanyan",
            HostBehavior::XianjianDiewu => "host:xj_diewu",
            HostBehavior::XianjianLingyu => "host:xj_lingyu",
            HostBehavior::WudianLueying => "host:xj_lueying",
            HostBehavior::XianjianAnwugu => "host:xj_anwugu",
            HostBehavior::XianjianAnwugu2 => "host:xj_anwugu2",
            HostBehavior::XianjianTxianqu => "host:xj_txianqu",
            HostBehavior::XianjianFenshi => "host:xj_fenshi",
            HostBehavior::XianjianLonghuo => "host:xj_longhuo",
            HostBehavior::XianjianSajin => "host:xj_sajin",
            HostBehavior::XianjianJtjubao => "host:xj_jtjubao",
            HostBehavior::XianjianYuexing => "host:xj_yuexing",
            HostBehavior::XianjianMinsha => "host:xj_minsha",
            HostBehavior::XianjianLinghuo => "host:xj_linghuo",
            HostBehavior::XianjianGuijin => "host:xj_guijin",
            HostBehavior::XianjianZhuyue => "host:xj_zhuyue",
            HostBehavior::XianjianGuanri => "host:xj_guanri",
            HostBehavior::XianjianShuiyun => "host:xj_shuiyun",
            HostBehavior::XianjianShuiyun5 => "host:xj_shuiyun5",
            HostBehavior::XianjianWangyou => "host:xj_wangyou",
            HostBehavior::XianjianWangyou2 => "host:xj_wangyou2",
            HostBehavior::XianjianLingquan => "host:xj_lingquan",
            HostBehavior::XianjianShenwu => "host:xj_shenwu",
            HostBehavior::XianjianChengxin => "host:xj_chengxin",
            HostBehavior::XianjianShenmu => "host:xj_shenmu",
            HostBehavior::XianjianDuci => "host:xj_duci",
            HostBehavior::XianjianRunxin => "host:xj_runxin",
            HostBehavior::XianjianXjzhimeng => "host:xj_xjzhimeng",
            HostBehavior::XianjianXjzhimeng2 => "host:xj_xjzhimeng2",
            HostBehavior::XianjianSheling => "host:xj_sheling",
            HostBehavior::XianjianHushaCount => "host:xj_husha_count",
            HostBehavior::XianjianHusha => "host:xj_husha",
            HostBehavior::XianjianChangnian => "host:xj_changnian",
            HostBehavior::XianjianChangnian2 => "host:xj_changnian2",
            HostBehavior::XianjianShiying => "host:xj_shiying",
            HostBehavior::XianjianZhaoyao => "host:xj_zhaoyao",
            HostBehavior::XianjianLiguang => "host:xj_liguang",
            HostBehavior::XianjianXuanmo => "host:xj_xuanmo",
            HostBehavior::XianjianSheying => "host:xj_sheying",
            HostBehavior::XianjianHuahu => "host:xj_huahu",
            HostBehavior::XianjianXtanxi => "host:xj_xtanxi",
            HostBehavior::XianjianFenglue => "host:xj_fenglue",
            HostBehavior::XianjianLiefeng => "host:xj_liefeng",
            HostBehavior::XianjianTianjian => "host:xj_tianjian",
            HostBehavior::XianjianPoyun => "host:xj_poyun",
            HostBehavior::XianjianXshuangren => "host:xj_xshuangren",
            HostBehavior::XianjianXshuangren2 => "host:xj_xshuangren2",
            HostBehavior::XianjianZhangmu => "host:xj_zhangmu",
            HostBehavior::XianjianQianfang => "host:xj_qianfang",
            HostBehavior::XianjianXunying => "host:xj_xunying",
            HostBehavior::XianjianTianxian => "host:xj_tianxian",
            HostBehavior::XianjianXjyingfeng => "host:xj_xjyingfeng",
            HostBehavior::XianjianXiepan => "host:xj_xiepan",
            HostBehavior::XianjianLuanjian => "host:xj_luanjian",
            HostBehavior::XianjianLuanjian2 => "host:xj_luanjian2",
            HostBehavior::XianjianCtianfu => "host:xj_ctianfu",
            HostBehavior::XianjianCtianfu2 => "host:xj_ctianfu2",
            HostBehavior::XianjianCtianfu3 => "host:xj_ctianfu3",
            HostBehavior::SwdZhiluxiaohuCard => "host:swd_zhiluxiaohu_card",
            HostBehavior::SwdMujiarenCard => "host:swd_mujiaren_card",
            HostBehavior::SwdGouhunluoCard => "host:swd_gouhunluo_card",
            HostBehavior::SwdGouhunluoSub => "host:swd_gouhunluo_sub",
            HostBehavior::SwdJiguantongCard => "host:swd_jiguantong_card",
            HostBehavior::SwdJiguanfengCard => "host:swd_jiguanfeng_card",
            HostBehavior::SwdFengyinState => "host:swd_fengyin_state",
            HostBehavior::SwdJiguanyuanCard => "host:swd_jiguanyuan_card",
            HostBehavior::SwdJiguanyuanGain => "host:swd_jiguanyuan_gain",
            HostBehavior::SwdJiguanyaoshuCard => "host:swd_jiguanyaoshu_card",
            HostBehavior::SwdJiguanyaoState => "host:swd_jiguanyao_state",
            HostBehavior::SwdXuanfengzhirenCard => "host:swd_xuanfengzhiren_card",
            HostBehavior::SwdZhongxinghujiaCard => "host:swd_zhongxinghujia_card",
            HostBehavior::SwdXingtigaizaoCard => "host:swd_xingtigaizao_card",
            HostBehavior::SwdShijianhuisuCard => "host:swd_shijianhuisu_card",
            HostBehavior::SwdJinjilengdongCard => "host:swd_jinjilengdong_card",
            HostBehavior::SwdShengxiuhaojiaoCard => "host:swd_shengxiuhaojiao_card",
            HostBehavior::SwdYinmilichangCard => "host:swd_yinmilichang_card",
            HostBehavior::SwdJiguanyaoGain => "host:swd_jiguanyao_gain",
            HostBehavior::SwdJiguanshuCard => "host:swd_jiguanshu_card",
            HostBehavior::SwdTianfuzhenCard => "host:swd_tianfuzhen_card",
            HostBehavior::SwdQixingzhenCard => "host:swd_qixingzhen_card",
            HostBehavior::SwdNiaoxiangzhenCard => "host:swd_niaoxiangzhen_card",
            HostBehavior::SwdChangshezhenCard => "host:swd_changshezhen_card",
            HostBehavior::SwdPozhenjueCard => "host:swd_pozhenjue_card",
        }
    }
}

/// 全部行为（tag 序；完整性由 `a0_tag_registry` 的穷尽守卫锁定）。

impl HostBehavior {
    pub fn trigger_meta(&self) -> Option<TriggerMeta> {
    match self {
            HostBehavior::ShaDamageForceDiscard => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::PhaseEndRecoverOrDraw => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::YunchouXicai => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::YunchouDiykuanggu => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::YunchouXiongzi => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::YunchouYaliang => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 纵火：伤害前置窗（nature 写回须在 commit 前生效）
            HostBehavior::YunchouZonghuo => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::YunchouShaoying => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::YunchouLuweiyan => None,
            HostBehavior::YunchouJieyan => None,
            HostBehavior::YunchouYiesheng => None,
            HostBehavior::WudianDiyjuntun => None,
            HostBehavior::WudianJinyan => None,
            // 理断：获得结算后（CardsMoved after）
            HostBehavior::WudianLiduan => Some(TriggerMeta {
                event: EventKind::CardsMoved,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 坚壁：锁定查询面（AOE 目标段查询，无触发帧）
            HostBehavior::WudianJianbi => None,
            // 复仇：目录锚（取消逻辑在 ShaResolveMachine 内联）
            HostBehavior::WudianFuchou => None,
            // 良策·余获：锁定查询面（五谷机器内联查询，无触发帧）
            HostBehavior::WudianLiangceRemain => None,
            // 归汉：目录锚（limited 确认逻辑在 DyingSaveMachine 内联）
            HostBehavior::WudianGuihan => None,
            // 铁壁：成为杀目标时（CardUsed 帧等价承载，登记差异：上游
            // target shaBefore，原生以 CardUsed+targets 等价承载）
            HostBehavior::SwdTiebi => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 大同：任意玩家阶段结束（global；frequent 自动无门）
            HostBehavior::SwdDatong => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 承剑：其他角色受伤结算后（global；确认门=optional）
            HostBehavior::SwdChengjian => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 缚斧：伤害前置窗（num 直写）
            HostBehavior::SwdJufu => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 花镜：自己造成伤害结算后（frequent 自动无门）
            HostBehavior::SwdHuajing => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 护盾：自己造成伤害结算后（forced 无门）
            HostBehavior::SwdHudun => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 破星：伤害前置窗（num 直写）
            HostBehavior::SwdPoxing => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 流虹：使用杀（frequent 自动无门）
            HostBehavior::SwdLiuhong => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 墨剑：自己使用杀（可选确认门；上游 shaBegin）
            HostBehavior::SwdMojian => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 道法：其他角色受伤结算后（global；确认门=optional）
            HostBehavior::SwdDaofa => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 玉鳞：受伤前置窗口（可选；弃装备牌免伤）
            HostBehavior::SwdYulin => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 凝霜：成为黑牌目标（可选；上游 useCardToBegin）
            HostBehavior::SwdNingshuang => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 工神：其他角色用基本/锦囊牌（可选；上游 global useCard）
            HostBehavior::SwdGongshen => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 平恤：无触发面（computed_distance 字段消费）
            HostBehavior::SwdPingxu => None,
            // 还灵：自己阶段结束（可选确认门；上游 player phaseEnd）
            HostBehavior::SwdHuanling => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 断月：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdDuanyue => None,
            // 屠阵：自己造成伤害结算后（可选；上游 source damageAfter）
            HostBehavior::SwdTuzhen => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 落梅：使用或打出（频率技；上游 useCard+respond 双事件）
            HostBehavior::SwdLuomei => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: vec![EventKind::CardResponded],
            }),
            // 星点：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdXingdian => None,
            // 协律：弃牌阶段结束=End 帧入场（可选；上游 phaseDiscardEnd）
            HostBehavior::SwdXielv => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 雪煌：出牌主动技（ActivateSkill 通道，limited bool 门）
            HostBehavior::SwdXuehuang => None,
            // 还侠：无触发面（PlayCard 转化臂+kernel 跟踪钩承载）
            HostBehavior::SwdHuanxia => None,
            // 破月：无触发面（PlayCard 臂+kernel 计数/闪询问钩承载）
            HostBehavior::SwdPoyue => None,
            // 收纳：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdLianyaohuShouna => None,
            // 炼化：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdLianyaohuLianhua => None,
            // 神农鼎：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdShennongdingCard => None,
            // 昆仑镜/东皇钟：出牌主动技/阶段末触发（东皇钟 direct forced）
            HostBehavior::SwdKunlunjingcCard => None,
            HostBehavior::SwdDonghuangzhongCard => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 鬼蛇旗/仙露回：trick 卡牌机（元数据不进触发计划）
            HostBehavior::GuisheqiCard => None,
            HostBehavior::XianluhuiCard => None,
            // 青龙之圭/黄麟之琮：准备阶段（Judge 帧入场 forced）
            HostBehavior::SwdQinglongzhiguiHeld => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdHuanglinzhicongHeld => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 朱雀之璋：受伤结算后 forced
            HostBehavior::SwdZhuquezhizhangHeld => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 白兽之琥：弃牌阶段结束 direct（蝶舞口径）
            HostBehavior::SwdBaishouzhihuHeld => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 玉蝉换形：自己用牌结算后 silent forced
            HostBehavior::SwdYuchanSwap => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Last,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 疾影：锁定 mod 无触发面（sha_range_free 链查询）
            HostBehavior::SwdJiyingHeld => None,
            // 沐风：结束阶段确认门（frequent 上游→确认近似）
            HostBehavior::SwdMufengHeld => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 望尘：弃牌阶段结束确认门（Discard→End 入场近似）
            HostBehavior::SwdWangchenHeld => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 神草结：锦囊伤害前置窗口（确认门；上游 chooseToUse）
            HostBehavior::SwdShencaojieGuard => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 玄武之璜：自己造成伤害结算后 forced
            HostBehavior::SwdXuanwuzhihuangHeld => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 逆攻积攒：受伤结算后 forced
            HostBehavior::SwdNigongAccumulate => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 逆攻发动：出牌主动技（标记门在 activate）
            HostBehavior::SwdNigongStrike => None,
            // 四非珍面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdSifeizhenmianCard => None,
            // 空洞印：死亡前置窗口（forced；上游 dieBefore cancel 同口径）
            HostBehavior::SwdKongdongyinGuard => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::First,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 女娲石：他人濒死（确认门；上游 global dying priority 6）
            HostBehavior::SwdNvwashiGuard => Some(TriggerMeta {
                event: EventKind::Dying,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 壶境：准备阶段（Judge 帧入场 forced；上游 phaseBegin 锁定）
            HostBehavior::SwdHujing => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 朱羽：横置者受伤前置窗口（可选；上游 global damageBegin）
            HostBehavior::SwdZhuyu => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 分舵：出牌主动技（ActivateSkill 通道，每回合限一次走 limits）
            HostBehavior::SwdJiefen => None,
            // 悲歌：任意角色受伤结算后（global；确认门=optional）
            HostBehavior::YunchouBeige => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 复仇·清算：自己回合结束（标记逐个结算）
            HostBehavior::WudianFuchou2 => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 蝉缘·变体：死亡结算后（来源减上限）
            HostBehavior::YunchouDiychanyuan => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::YunchouLuweiyan2 => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 蛊惑·变体：回合（判定阶段）开始
            HostBehavior::YunchouDiyguhuo => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 狡黠：成为牌目标（CardUsed+targets，frequent 自动）
            HostBehavior::YunchouJiaoxia => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::YunchouHonglian => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::TargetDiscardEquipOrDamage => None,
            HostBehavior::ShengkeCheck => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::ChangeCardNature => None,

            HostBehavior::MaxRangeWeaponDamage => Some(TriggerMeta {
                event: EventKind::Equipped,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WangmeiZhikeCard => None,
            HostBehavior::SijiDiscardDraw => Some(TriggerMeta {
                event: EventKind::CardsDiscarded,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::SanjianWeaponDamage => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::MoshouImmunity => None,
            HostBehavior::ShuigongCard => None,
            HostBehavior::FudichouxinCompare => None,
            HostBehavior::ChenhuodajieCard => None,
            HostBehavior::TouliangSwap => None,
            HostBehavior::LinghunzhihuoCard => None,
            HostBehavior::ZhaomingdanCard => None,
            HostBehavior::JihuocardDraw => None,
            HostBehavior::GeanguanhuoCompare => None,
            HostBehavior::CaochuanCard => None,
            HostBehavior::ShezhanqunruCompare => None,
            HostBehavior::GujianShujinsanCard => None,
            HostBehavior::GujianMianliCard => None,
            HostBehavior::GujianFeibiaoCard => None,
            HostBehavior::JinlianzhuCancel => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodTanhuadong => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodYougeng => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodGrant => None,
            HostBehavior::GujianFoodQingtuan => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodYuanbaorou => None,
            HostBehavior::GujianFoodGudonggeng => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodMapodoufu => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodChunbing => None,
            HostBehavior::GujianFoodLiyutang => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodMolicha => None,
            HostBehavior::GujianFoodXiajiao => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodMizhilianou => None,
            HostBehavior::GujianFoodLuyugeng => None,
            HostBehavior::JiguanBingpoCard => None,
            HostBehavior::JiguanQiankunCard => None,
            HostBehavior::JiguanLongxuCard => None,
            HostBehavior::JiguanShenhuoCard => None,
            HostBehavior::GujianFoodZiyangdan => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianYunvyuanshen => Some(TriggerMeta {
                event: EventKind::Dying,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodShihuifenBan => None,
            HostBehavior::GujianShihuifenCard => None,
            HostBehavior::GujianLiufengsanCard => None,
            HostBehavior::JiguanShatangCard => None,
            HostBehavior::JiguanHeilongCard => None,
            HostBehavior::JiguanLiutougeCard => None,
            HostBehavior::GujianMutouEquipSkill => None,
            HostBehavior::JiguanDinvCard => None,
            HostBehavior::GujianYuhengEquipSkill => None,
            HostBehavior::HearthDunpaiCard => None,
            HostBehavior::HearthZhiliaoboCard => None,
            HostBehavior::HearthYuansuCard => None,
            HostBehavior::HearthXingjieCard => None,
            HostBehavior::HearthChuansongCard => None,
            HostBehavior::HearthTansheCard => None,
            HostBehavior::HearthShijieshuCard => None,
            HostBehavior::GwentYanzhiCard => None,
            HostBehavior::GwentAozuCard => None,
            HostBehavior::GwentWenyiCard => None,
            HostBehavior::GwentZhihuanCard => None,
            HostBehavior::GwentBaoxueCard => None,
            HostBehavior::GwentQinpenState => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GwentBirinState => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GwentCiguState => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GwentWeatherGrant => None,
            HostBehavior::GwentKunenState => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GwentKunenCard => None,
            HostBehavior::GwentShanbengCard => None,
            HostBehavior::GwentPoxiaoCard => None,
            HostBehavior::GwentFengyinState => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 机关凤封印：拥有者下一阶段开始解除（上游 addTempSkill
            // {player:'phaseBegin'} 同口径）
            HostBehavior::SwdFengyinState => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 机关怨待获得：拥有者结束阶段强制获得置牌
            HostBehavior::SwdJiguanyuanGain => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdJiguanfengCard => None,
            HostBehavior::SwdJiguanyuanCard => None,
            // 机关妖术防重标记：纯状态，无触发
            HostBehavior::SwdJiguanyaoshuCard => None,
            HostBehavior::SwdJiguanyaoState => None,
            HostBehavior::SwdJiguanshuCard => None,
            HostBehavior::SwdTianfuzhenCard => None,
            HostBehavior::SwdQixingzhenCard => None,
            HostBehavior::SwdNiaoxiangzhenCard => None,
            HostBehavior::SwdChangshezhenCard => None,
            HostBehavior::SwdPozhenjueCard => None,
            // 机关妖术补偿：弃牌事件（含装备牌时触发，登记近似差异）
            HostBehavior::SwdJiguanyaoGain => Some(TriggerMeta {
                event: EventKind::CardsDiscarded,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdXuanfengzhirenCard => None,
            HostBehavior::SwdZhongxinghujiaCard => None,
            HostBehavior::SwdXingtigaizaoCard => None,
            HostBehavior::SwdShijianhuisuCard => None,
            HostBehavior::SwdJinjilengdongCard => None,
            HostBehavior::SwdShengxiuhaojiaoCard => None,
            HostBehavior::SwdYinmilichangCard => None,
            HostBehavior::GwentZumoCard => None,
            HostBehavior::HuanleMonkeyCard => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::HuanleXuelunyangCard => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::HuanleMianjuCard => Some(TriggerMeta {
                event: EventKind::TurnOverBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::ZhangbaSha => None,
            HostBehavior::SwdLiuxinghuoyuCard => None,
            HostBehavior::SwdShentouEquipSkill => None,
            HostBehavior::SwdYufuluEquipSkill => None,
            HostBehavior::SwdYangpijuanCard => None,
            HostBehavior::SwdLangeguaiyiEquipSkill => None,
            HostBehavior::MtgLongshanfengLand => None,
            HostBehavior::MtgHaidaoLand => None,
            HostBehavior::MtgShuimomuxueLand => None,
            HostBehavior::MtgFeixuLand => None,
            HostBehavior::MtgYixialanLand => None,
            HostBehavior::MtgYixialanRecast => None,
            HostBehavior::MtgShamolvzhouLand => None,
            HostBehavior::MtgLinzhongjiantaLand => None,
            HostBehavior::MtgLinzhongjiantaStrike => None,
            HostBehavior::MtgBingheyaosaiLand => None,
            HostBehavior::MtgYoulinLand => None,
            HostBehavior::MtgYoulinDiscard => None,
            HostBehavior::MtgLindixiliuLand => None,
            HostBehavior::WudianCiqiu => None,
            HostBehavior::WudianShuaiyan => Some(TriggerMeta {
                event: EventKind::Recover,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianZhucheng => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianZhuchengGuard => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianKangyin => None,
            HostBehavior::WudianChezhen => None,
            HostBehavior::WudianJuedao => None,
            HostBehavior::WudianGeju => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianTuji => None,
            HostBehavior::WudianMujing => None,
            HostBehavior::WudianGuangshu => None,
            HostBehavior::WudianGuangshuHeart => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianGuangshuDiamond => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianGuangshuClub => None,
            HostBehavior::WudianGuangshuSpade => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianZiyu => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianShouhu => None,
            HostBehavior::WudianDuwen => None,
            HostBehavior::WudianPaotai => None,
            HostBehavior::WudianPaotaiFire => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianBaoleiSwitch => None,
            HostBehavior::WudianZhenchaDraw => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianYuedong => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianKuoyin => None,
            HostBehavior::WudianHuhuan => None,
            HostBehavior::WudianJidong => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianBaoxue => None,
            HostBehavior::WudianLiangou => None,
            HostBehavior::WudianXiyang => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianFeiren => None,
            HostBehavior::WudianFeidan => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianZhiyuan => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianShenqiang => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianShihuo => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianMengji => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianYinbo => None,
            HostBehavior::WudianQianggu => None,
            HostBehavior::WudianQiangguGuard => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianFeitiao => None,
            HostBehavior::WudianPingzhang => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianOwliyong => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianYinshen => Some(TriggerMeta {
                event: EventKind::CardsDiscarded,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianXiandan => None,
            HostBehavior::WudianJijia => None,
            HostBehavior::XianjianLingxue => Some(TriggerMeta {
                event: EventKind::Recover,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianBinxin => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianHuimeng => Some(TriggerMeta {
                event: EventKind::Recover,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianTianshe => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianTianshe2 => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianFanling => Some(TriggerMeta {
                event: EventKind::LoseHp,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianLongxiang => None,
            HostBehavior::XianjianGuiyuan => None,
            HostBehavior::XianjianTannang => None,
            HostBehavior::XianjianFeixu => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: vec![EventKind::CardResponded],
            }),
            HostBehavior::XianjianFeng => Some(TriggerMeta {
                event: EventKind::CardsDrawn,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianYa => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianSong => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianXjyufeng => Some(TriggerMeta {
                event: EventKind::CardsDiscarded,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianXuanyan2 => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianXuanyan3 => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianNingbin => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianXuanyan => None,
            HostBehavior::XianjianDiewu => None,
            HostBehavior::XianjianLingyu => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::WudianLueying => None,
            HostBehavior::XianjianAnwugu => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianAnwugu2 => None,
            // 甜香去：伤害前置窗口（上游 source damageBefore check 询问式）
            HostBehavior::XianjianTxianqu => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianFenshi => Some(TriggerMeta {
                event: EventKind::DyingSaved,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianLonghuo => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianSajin => None,
            // 聚宝：global 弃牌（上游 discardAfter frequent=自动无确认）
            HostBehavior::XianjianJtjubao => Some(TriggerMeta {
                event: EventKind::CardsDiscarded,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianYuexing => None,
            HostBehavior::XianjianMinsha => None,
            // 聚宝：global 弃牌（上游 discardAfter frequent=自动无确认）
            HostBehavior::XianjianLinghuo => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianGuijin => None,
            HostBehavior::XianjianZhuyue => None,
            HostBehavior::XianjianGuanri => None,
            HostBehavior::XianjianShuiyun => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianShuiyun5 => Some(TriggerMeta {
                event: EventKind::Dying,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianWangyou => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianWangyou2 => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianLingquan => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianShenwu => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianChengxin => Some(TriggerMeta {
                event: EventKind::Dying,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianShenmu => Some(TriggerMeta {
                event: EventKind::Dying,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianDuci => Some(TriggerMeta {
                event: EventKind::CardsDiscarded,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianRunxin => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: vec![EventKind::CardResponded],
            }),
            HostBehavior::XianjianXjzhimeng => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianXjzhimeng2 => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianSheling => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: vec![EventKind::CardResponded, EventKind::CardsDiscarded],
            }),
            HostBehavior::XianjianHushaCount => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianHusha => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianChangnian => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianChangnian2 => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianShiying => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianZhaoyao => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianLiguang => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianXuanmo => None,
            HostBehavior::XianjianSheying => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianHuahu => None,
            HostBehavior::XianjianXtanxi => None,
            HostBehavior::XianjianFenglue => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianLiefeng => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianTianjian => None,
            HostBehavior::XianjianPoyun => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianXshuangren => Some(TriggerMeta {
                event: EventKind::CardsDiscarded,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianXshuangren2 => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianZhangmu => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianQianfang => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianXunying => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianTianxian => None,
            HostBehavior::XianjianXjyingfeng => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianXiepan => Some(TriggerMeta {
                event: EventKind::CardsDiscarded,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianLuanjian => None,
            // 乱剑·增伤：杀伤害前置窗（上游 source damageBegin 同口径）
            HostBehavior::XianjianLuanjian2 => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianCtianfu => None,
            // 天符·反噬：伤害结算后（上游 source damageAfter=Damage 帧
            // AfterTriggers）
            HostBehavior::XianjianCtianfu2 => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::XianjianCtianfu3 => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            // swd 机关牌族（v189-v192）：卡牌机器（无懈窗包装，intrinsic 分派——无触发）
            HostBehavior::SwdZhiluxiaohuCard => None,
            HostBehavior::SwdMujiarenCard => None,
            HostBehavior::SwdGouhunluoCard => None,
            // 勾魂锣·倒计时：来源回合开始（global phaseBegin 由 filter 定向）
            HostBehavior::SwdGouhunluoSub => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdJiguantongCard => None,
        }
    }
}

/// 相克表：`KE[nature]` 为 `nature` 所克的属性（金克木/木克土/水克火/
/// 火克金/土克水，编号 0=金 1=木 2=水 3=火 4=土）。

pub const KE: [u8; 5] = [1, 4, 3, 0, 2];
/// 相生表：`SHENG[nature]` 为 `nature` 所生的属性（金生水/木生火/
/// 水生木/火生土/土生金）。
pub const SHENG: [u8; 5] = [2, 3, 1, 4, 0];

// ==== 提示语等常量（事故恢复：操作史重放挖掘） ====
pub const GJ_ZIYANGDAN_HUJIA: u8 = 3;
pub const GJ_FOOD_ROUNDS_CHUNBING: u8 = 5;
pub const GJ_FOOD_ROUNDS_DEFAULT: u8 = 3;
pub const GJ_FOOD_ROUNDS_LIYUTANG: u8 = 2;
pub const GJ_FOOD_ROUNDS_MIZHLIANOU: u8 = 4;
pub const GJ_FOOD_ROUNDS_TANHUADONG: u8 = 3;
pub const GJ_FOOD_ROUNDS_XIAJIAO: u8 = 4;
pub const GJ_FOOD_ROUNDS_YOUGENG: u8 = 3;
pub const D3_JINGGOU_TARGET: &str = "d3_jinggou_target";
pub const GJ_BINGPOTONG_CARD_KEY: &str = "gj:bingpotong";
pub const GJ_BINGPO_SHOW_OWN: &str = "gj_bingpo_show_own";
pub const GJ_BINGPO_SHOW_TARGET: &str = "gj_bingpo_show_target";
pub const GJ_CHUNBING_CARD_KEY: &str = "gj:chunbing";
pub const GJ_DINVXUANSHUANG_CARD_KEY: &str = "gj:dinvxuanshuang";
pub const GJ_DINVXUANSHUANG_RIDER: &str = "gj_dinvxuanshuang_rider";
pub const GJ_FEIBIAO_DISCARD: &str = "gj_feibiao_discard";
pub const GJ_FEIBIAO_KEY: &str = "gj:feibiao";
pub const GJ_FOOD_CHUNBING_KEY: &str = "gj_food_chunbing";
pub const GJ_FOOD_GUDONGGENG_KEY: &str = "gj_food_gudonggeng";
pub const GJ_FOOD_LIYUTANG_KEY: &str = "gj_food_liyutang";
pub const GJ_FOOD_LUYUGENG_KEY: &str = "gj_food_luyugeng";
pub const GJ_FOOD_MAPODOUFU_KEY: &str = "gj_food_mapodoufu";
pub const GJ_FOOD_MIZHLIANOU_KEY: &str = "gj_food_mizhilianou";
pub const GJ_FOOD_MOLICHA_KEY: &str = "gj_food_molicha";
pub const GJ_FOOD_QINGTUAN_KEY: &str = "gj_food_qingtuan";
pub const GJ_FOOD_SHIHUIFEN_BAN_KEY: &str = "gj_food_shihuifen_ban";
pub const GJ_FOOD_TANHUADONG_KEY: &str = "gj_food_tanhuadong";
pub const GJ_FOOD_XIAJIAO_KEY: &str = "gj_food_xiajiao";
pub const GJ_FOOD_YOUGENG_KEY: &str = "gj_food_yougeng";
pub const GJ_FOOD_YUANBAOROU_KEY: &str = "gj_food_yuanbaorou";
pub const GJ_FOOD_ZIYANGDAN_KEY: &str = "gj_food_ziyangdan";
pub const GJ_GUDONGGENG_CARD_KEY: &str = "gj:gudonggeng";
pub const GJ_HEILONGLINPIAN_CARD_KEY: &str = "gj:heilonglinpian";
pub const GJ_JINLIANZHU_CONFIRM: &str = "gj_jinlianzhu_confirm";
pub const GJ_JINLIANZHU_KEY: &str = "gj_jinlianzhu_listener";
pub const GJ_LIUFENGSAN_CARD_KEY: &str = "gj:liufengsan";
pub const GJ_LIUTOUGE_CARD_KEY: &str = "gj:liutouge";
pub const GJ_LIYUTANG_CARD_KEY: &str = "gj:liyutang";
pub const GJ_LONGXUGOU_CARD_KEY: &str = "gj:longxugou";
pub const GJ_LONGXU_PICK: &str = "gj_longxu_pick";
pub const GJ_LUYUGENG_CARD_KEY: &str = "gj:luyugeng";
pub const GJ_LUYUGENG_COST: &str = "gj_luyugeng_cost";
pub const GJ_MAPODOUFU_CARD_KEY: &str = "gj:mapodoufu";
pub const GJ_MIANLI_KEY: &str = "gj:mianlijinzhen";
pub const GJ_MIZHLIANOU_CARD_KEY: &str = "gj:mizhilianou";
pub const GJ_MOLICHA_CARD_KEY: &str = "gj:molicha";
pub const GJ_MUTOU_SKILL_KEY: &str = "gj_mutoumianju_skill";
pub const GJ_QIANKUNBIAO_CARD_KEY: &str = "gj:qiankunbiao";
pub const GJ_QINGTUAN_CARD_KEY: &str = "gj:qingtuan";
pub const GJ_SHATANG_CARD_KEY: &str = "gj:shatang";
pub const GJ_SHENHUOFEIYA_CARD_KEY: &str = "gj:shenhuofeiya";
pub const GJ_SHIHUIFEN_CARD_KEY: &str = "gj:shihuifen";
pub const GJ_SHUJINSAN_DISCARD: &str = "gj_shujinsan_discard";
pub const GJ_SHUJINSAN_KEY: &str = "gj:shujinsan";
pub const GJ_TANHUADONG_CARD_KEY: &str = "gj:tanhuadong";
pub const GJ_XIAJIAO_CARD_KEY: &str = "gj:xiajiao";
pub const GJ_XIAJIAO_DISCARD: &str = "gj_xiajiao_discard";
pub const GJ_YOUGENG_CARD_KEY: &str = "gj:yougeng";
pub const GJ_YUANBAOROU_CARD_KEY: &str = "gj:yuanbaorou";
pub const GJ_YUHENG_SKILL_KEY: &str = "gj_yuheng_skill";
pub const GJ_YUNVYUANSHEN_CARD_KEY: &str = "gj:yunvyuanshen";
pub const GJ_YUNVYUANSHEN_LISTENER_KEY: &str = "gj_yunvyuanshen_listener";
pub const GJ_ZIYANGDAN_CARD_KEY: &str = "gj:ziyangdan";
pub const GW_AOZU_CARD_KEY: &str = "gw:aozuzhilei";
pub const GW_BAOXUE_CARD_KEY: &str = "gw:baoxueyaoshui";
pub const GW_BAOXUE_DISCARD: &str = "gw_baoxue_discard";
pub const GW_BIRIN_CARD_KEY: &str = "gw:birinongwu";
pub const GW_BIRIN_STATE_KEY: &str = "gw_birin_state";
pub const GW_CIGU_CARD_KEY: &str = "gw:ciguhanshuang";
pub const GW_CIGU_STATE_KEY: &str = "gw_cigu_state";
pub const GW_FENGYIN_STATE_KEY: &str = "gw_fengyin_state";
pub const GW_KUNEN_CARD_KEY: &str = "gw:kunenfayin";
pub const GW_KUNEN_STATE_KEY: &str = "gw_kunen_state";
pub const GW_POXIAO_BRANCH: &str = "gw_poxiao_branch";
pub const GW_POXIAO_CARD_KEY: &str = "gw:poxiao";
pub const GW_QINPEN_CARD_KEY: &str = "gw:qinpendayu";
pub const GW_QINPEN_STATE_KEY: &str = "gw_qinpen_state";
pub const GW_SHANBENG_CARD_KEY: &str = "gw:shanbengshu";
pub const GW_WEATHER_GRANT_KEY: &str = "gw:weather_grant";
pub const GW_WENYI_CARD_KEY: &str = "gw:wenyi";
pub const GW_YANZHI_CARD_KEY: &str = "gw:yanziyaoshui";
pub const GW_ZIHUAN_CARD_KEY: &str = "gw:zhihuanjun";
pub const GW_ZUMO_CARD_KEY: &str = "gw:zumoshoukao";
pub const HH_CHUANSONG_CARD_KEY: &str = "hh:chuansongmen";
pub const HH_DUNPAI_CARD_KEY: &str = "hh:dunpaigedang";
pub const HH_DUNPAI_DISCARD: &str = "hh_dunpai_discard";
pub const HH_SHIJIE_CARD_KEY: &str = "hh:shijieshu";
pub const HH_TANSHE_CARD_KEY: &str = "hh:tanshezhiren";
pub const HH_XINGJIE_CARD_KEY: &str = "hh:xingjiegoutong";
pub const HH_YUANSU_CARD_KEY: &str = "hh:yuansuhuimie";
pub const HH_YUANSU_DISCARD: &str = "hh_yuansu_discard";
pub const HH_ZHILIAOBO_CARD_KEY: &str = "hh:zhiliaobo";
pub const HOST_SHOULIJIAN_KEY: &str = "host:shoulijian";
pub const HOST_TARGET_DISCARD_EQUIP: &str = "host_target_discard_equip";
pub const HT_HEARTH_KEY_PREFIX: &str = "ht:";
pub const HT_JIHUO_KEY: &str = "ht:jihuocard";
pub const HT_LINGHUN_KEY: &str = "ht:linghunzhihuo";
pub const HT_ZHAOMING_KEY: &str = "ht:zhaomingdan";
pub const HUANLE_MIANJI_KEY: &str = "huanle:mianju";
pub const HUANLE_MONKEY_KEY: &str = "huanle:monkey";
pub const HUANLE_XUELUNYANG_KEY: &str = "huanle:xuelunyang";
pub const MTG_BINGHEYAOSAI_DISCARD: &str = "mtg_bingheyaosai_discard";
pub const MTG_BINGHEYAOSAI_KEY: &str = "mtg_bingheyaosai_land";
pub const MTG_FEIXU_DISCOVER: &str = "mtg_feixu_discover";
pub const MTG_FEIXU_KEY: &str = "mtg_feixu_land";
pub const MTG_HAIDAO_DISCARD: &str = "mtg_haidao_discard";
pub const MTG_HAIDAO_KEY: &str = "mtg_haidao_land";
pub const MTG_LINDIXILIU_DISCARD: &str = "mtg_lindixiliu_discard";
pub const MTG_LINDIXILIU_KEY: &str = "mtg_lindixiliu_land";
pub const MTG_LINZHONGJIANTA_CARD: &str = "mtg_linzhongjianta_card";
pub const MTG_LINZHONGJIANTA_DISCOVER: &str = "mtg_linzhongjianta_discover";
pub const MTG_LINZHONGJIANTA_KEY: &str = "mtg_linzhongjianta_land";
pub const MTG_LINZHONGJIANTA_SKILL_KEY: &str = "mtg_linzhongjianta_strike";
pub const MTG_LINZHONGJIANTA_TARGET: &str = "mtg_linzhongjianta_target";
pub const MTG_LONGSHANFENG_GAIN: &str = "mtg_longshanfeng_gain";
pub const MTG_LONGSHANFENG_KEY: &str = "mtg_longshanfeng_land";
pub const MTG_SHAMOLVZHOU_DISCOVER: &str = "mtg_shamolvzhou_discover";
pub const MTG_SHAMOLVZHOU_KEY: &str = "mtg_shamolvzhou_land";
pub const MTG_SHUIMOMUXUE_DISCARD: &str = "mtg_shuimomuxue_discard";
pub const MTG_SHUIMOMUXUE_KEY: &str = "mtg_shuimomuxue_land";
pub const MTG_YIXIALAN_GAIN: &str = "mtg_yixialan_gain";
pub const MTG_YIXIALAN_KEY: &str = "mtg_yixialan_land";
pub const MTG_YIXIALAN_RECAST_CARD: &str = "mtg_yixialan_recast_card";
pub const MTG_YIXIALAN_RECAST_KEY: &str = "mtg_yixialan_recast";
pub const MTG_YOULIN_DISCARD: &str = "mtg_youlin_trick";
pub const MTG_YOULIN_DISCOVER_KEY: &str = "mtg_youlin_discover";
pub const MTG_YOULIN_KEY: &str = "mtg_youlin_land";
pub const MTG_YOULIN_PICK: &str = "mtg_youlin_pick";
pub const SWD_GOUHUNLUO_KEY: &str = "swd_gouhunluo";
pub const SWD_GOUHUNLUO_SUB_KEY: &str = "swd_gouhunluo_sub";
pub const SWD_JIGUANTONG_DISCARD: &str = "swd_jiguantong_discard";
pub const SWD_JIGUANTONG_KEY: &str = "swd_jiguantong";
pub const SWD_JIGUANFENG_KEY: &str = "swd_jiguanfeng";
pub const SWD_FENGYIN_STATE_KEY: &str = "swd_fengyin_state";
pub const SWD_JIGUANFENG_SHAN: &str = "swd_jiguanfeng_shan";
pub const SWD_JIGUANYUAN_KEY: &str = "swd_jiguanyuan";
pub const SWD_JIGUANYUAN_GAIN_KEY: &str = "swd_jiguanyuan_gain";
pub const SWD_JIGUANYUAN_CHOOSE: &str = "swd_jiguanyuan_choose";
pub const SWD_JIGUANYAOSHU_KEY: &str = "swd_jiguanyaoshu";
pub const SWD_JIGUANYAOSHU_STATE_KEY: &str = "swd_jiguanyaoshu_state";
pub const SWD_XUANFENG_KEY: &str = "swd_xuanfengzhiren";
pub const SWD_ZHONGXING_KEY: &str = "swd_zhongxinghujia";
pub const SWD_XINGTIGAIZAO_KEY: &str = "swd_xingtigaizao";
pub const SWD_SHIJIANHUISU_KEY: &str = "swd_shijianhuisu";
pub const SWD_JINJILENGDONG_KEY: &str = "swd_jinjilengdong";
pub const SWD_SHENGXIUHAOJIAO_KEY: &str = "swd_shengxiuhaojiao";
pub const SWD_YINMILICHANG_KEY: &str = "swd_yinmilichang";
pub const SWD_JIGUANYAO_GAIN_KEY: &str = "swd_jiguanyao_gain";
pub const SWD_JIGUANSHU_KEY: &str = "swd_jiguanshu";
pub const SWD_TIANFUZHEN_KEY: &str = "swd_tianfuzhen";
pub const SWD_QIXINGZHEN_KEY: &str = "swd_qixingzhen";
pub const SWD_NIAOXIANGZHEN_KEY: &str = "swd_niaoxiangzhen";
pub const SWD_CHANGSHEZHEN_KEY: &str = "swd_changshezhen";
pub const SWD_POZHENJUE_KEY: &str = "swd_pozhenjue";
pub const SWD_LANGEGUAIYI_SKILL_KEY: &str = "swd_langeguaiyi_skill";
pub const SWD_LIUXING_DISCARD: &str = "swd_liuxing_discard";
pub const SWD_LIUXING_KEY: &str = "gj:swd_liuxinghuoyu";
pub const SWD_MUJIAREN_KEY: &str = "swd_mujiaren";
pub const SWD_SHENTOU_COST: &str = "swd_shentou_cost";
pub const SWD_SHENTOU_SKILL_KEY: &str = "swd_shentou_skill";
pub const SWD_YANGPIJUAN_CATEGORY: &str = "swd_yangpijuan_category";
pub const SWD_YANGPIJUAN_KEY: &str = "swd_yangpijuan";
pub const SWD_YANGPIJUAN_PICK: &str = "swd_yangpijuan_pick";
pub const SWD_YUFULU_SKILL_KEY: &str = "swd_yufulu_skill";
pub const SWD_YUFULU_TARGET: &str = "swd_yufulu_target";
pub const SWD_YUFULU_WEAPON: &str = "swd_yufulu_weapon";
pub const SWD_ZHILUXIAOHU_KEY: &str = "swd_zhiluxiaohu";
pub const WD_CAOCHUAN_BRANCH: &str = "wd_caochuan_branch";
pub const WD_CAOCHUAN_KEY: &str = "wd:caochuanjiejian";
pub const WD_CHENHUO_KEY: &str = "wd:chenhuodajie";
pub const WD_FUDI_COMPARE: &str = "wd_fudi_compare";
pub const WD_FUDI_KEY: &str = "wd:fudichouxin";
pub const WD_FUDI_PLAYER_PICK: &str = "wd_fudi_player_pick";
pub const WD_FUDI_TARGET_PICK: &str = "wd_fudi_target_pick";
pub const WD_SANJIAN_DISCARD: &str = "wd_sanjian_discard";
pub const WD_SANJIAN_TARGET: &str = "wd_sanjian_target";
pub const WD_SHEZHAN_KEY: &str = "wd:shezhanqunru";
pub const WD_SHUIGONG_KEY: &str = "wd:shuigong";
pub const WD_TOULIANG_KEY: &str = "wd:toulianghuanzhu";
pub const WD_TOULIANG_OWN: &str = "wd_touliang_own";
pub const WD_TOULIANG_TARGET: &str = "wd_touliang_target";
pub const WD_WANGMEIZHIKE_KEY: &str = "wd:wangmeizhike";
pub const WUDIAN_BAOLEI_CONFIRM: &str = "wudian_baolei_confirm";
pub const WUDIAN_BAOLEI_KEY: &str = "bshaowei";
pub const WUDIAN_BAOXUE_TARGETS: &str = "wudian_baoxue_targets";
pub const WUDIAN_CHEZHEN_KEY: &str = "chezhen";
pub const WUDIAN_CIQIU_KEY: &str = "ciqiu";
pub const WUDIAN_DUWEN_KEY: &str = "duwen";
pub const WUDIAN_FEIDAN_DISCARD: &str = "wudian_feidan_discard";
pub const WUDIAN_FEIDAN_KEY: &str = "feidan";
pub const WUDIAN_FEIREN_KEY: &str = "feiren";
pub const WUDIAN_FEITIAO_CARDS: &str = "wudian_feitiao_cards";
pub const WUDIAN_FEITIAO_TARGET: &str = "wudian_feitiao_target";
pub const WUDIAN_GEJU_KEY: &str = "geju";
pub const WUDIAN_GUANGSHU_CARD: &str = "wudian_guangshu_card";
pub const WUDIAN_GUANGSHU_CLUB_KEY: &str = "guangshu_club";
pub const WUDIAN_GUANGSHU_DIAMOND_KEY: &str = "guangshu_diamond";
pub const WUDIAN_GUANGSHU_HEART_KEY: &str = "guangshu_heart";
pub const WUDIAN_GUANGSHU_KEY: &str = "guangshu";
pub const WUDIAN_GUANGSHU_SPADE_KEY: &str = "guangshu_spade";
pub const WUDIAN_GUANGSHU_TARGET: &str = "wudian_guangshu_target";
pub const WUDIAN_HUHUAN_CARDS: &str = "wudian_huhuan_cards";
pub const WUDIAN_JIJIA_KEY: &str = "jijia";
pub const WUDIAN_JUEDAO_KEY: &str = "juedao";
pub const WUDIAN_KANGYIN_DISCARD: &str = "wudian_kangyin_discard";
pub const WUDIAN_KANGYIN_DRAW: &str = "wudian_kangyin_draw";
pub const WUDIAN_KANGYIN_KEY: &str = "kangyin";
/// 禁言（玩点论杀续批）：PlayCard 门查询锚。
pub const WUDIAN_JINYAN_KEY: &str = "jinyan";
/// 复仇（玩点论杀批 14）：ShaResolveMachine 内联取消查询锚。
pub const WUDIAN_FUCHOU_KEY: &str = "fuchou";
/// 归汉（玩点论杀批 17）：DyingSaveMachine 内联确认查询锚。
pub const WUDIAN_GUIHAN_KEY: &str = "ycnguihan";
pub const WUDIAN_KANGYIN_TARGET: &str = "wudian_kangyin_target";
pub const WUDIAN_KUOYIN_CARDS: &str = "wudian_kuoyin_cards";
pub const WUDIAN_LIANGOU_CARDS: &str = "wudian_liangou_cards";
pub const WUDIAN_LIANGOU_TARGET: &str = "wudian_liangou_target";
pub const WUDIAN_LONGXIANG_KEY: &str = "longxiang";
pub const WUDIAN_LUEYING_KEY: &str = "lueying";
pub const WUDIAN_MUJING_CARD: &str = "wudian_mujing_card";
pub const WUDIAN_MUJING_KEY: &str = "mujing";
pub const WUDIAN_MUJING_TARGET: &str = "wudian_mujing_target";
pub const WUDIAN_PAOTAI_CARD: &str = "wudian_paotai_card";
pub const WUDIAN_PAOTAI_FIRE_KEY: &str = "paotai_fire";
pub const WUDIAN_PAOTAI_KEY: &str = "paotai";
pub const WUDIAN_PINGZHANG_CARDS: &str = "wudian_pingzhang_cards";
pub const WUDIAN_QIANGGU_CARDS: &str = "wudian_qianggu_cards";
pub const WUDIAN_QIANGGU_GUARD_ASK: &str = "wudian_qianggu_guard_ask";
pub const WUDIAN_SHOUHU_CARD: &str = "wudian_shouhu_card";
pub const WUDIAN_SHOUHU_KEY: &str = "shouhu";
pub const WUDIAN_SHOUHU_TARGET: &str = "wudian_shouhu_target";
pub const WUDIAN_SHUAIYAN_CHOICE: &str = "wudian_shuaiyan_choice";
pub const WUDIAN_SHUAIYAN_DISCARD: &str = "wudian_shuaiyan_discard";
pub const WUDIAN_SHUAIYAN_KEY: &str = "nsshuaiyan";
pub const WUDIAN_TUJI_KEY: &str = "tuji";
pub const WUDIAN_XIANDAN_KEY: &str = "xiandan";
pub const WUDIAN_XJ_CHANGNIAN_TARGET: &str = "xj_changnian_target";
pub const WUDIAN_XJ_CTIANFU_CARD: &str = "xj_ctianfu_card";
pub const WUDIAN_XJ_CTIANFU_TARGET: &str = "xj_ctianfu_target";
pub const WUDIAN_XJ_DIEWU_CARDS: &str = "xj_diewu_cards";
pub const WUDIAN_XJ_DIEWU_TARGET: &str = "xj_diewu_target";
pub const WUDIAN_XJ_DUCI_TARGET: &str = "xj_duci_target";
pub const WUDIAN_XJ_FENGLUE_TARGET: &str = "xj_fenglue_target";
pub const WUDIAN_XJ_GUANRI_CARDS: &str = "xj_guanri_cards";
pub const WUDIAN_XJ_GUANRI_TARGET: &str = "xj_guanri_target";
pub const WUDIAN_XJ_GUIJIN_CARDS: &str = "xj_guijin_cards";
pub const WUDIAN_XJ_GUIJIN_TARGET: &str = "xj_guijin_target";
pub const WUDIAN_XJ_GUIYUAN_CARDS: &str = "xj_guiyuan_cards";
pub const WUDIAN_XJ_HUAHU_TARGETS: &str = "xj_huahu_targets";
pub const WUDIAN_XJ_HUSHA_MODE: &str = "xj_husha_mode";
pub const WUDIAN_XJ_HUSHA_TARGET: &str = "xj_husha_target";
pub const WUDIAN_XJ_LIGUANG_CARDS: &str = "xj_liguang_cards";
pub const WUDIAN_XJ_LINGQUAN_CARDS: &str = "xj_lingquan_cards";
pub const WUDIAN_XJ_LINGYU_TARGET: &str = "xj_lingyu_target";
pub const WUDIAN_XJ_LUANJIAN_CARDS: &str = "xj_luanjian_cards";
pub const WUDIAN_XJ_LUANJIAN_TARGETS: &str = "xj_luanjian_targets";
pub const WUDIAN_XJ_MINSHA_CARDS: &str = "xj_minsha_cards";
pub const WUDIAN_XJ_MINSHA_TARGET: &str = "xj_minsha_target";
pub const WUDIAN_XJ_POYUN_CARDS: &str = "xj_poyun_cards";
pub const WUDIAN_XJ_QIANFANG_CARDS: &str = "xj_qianfang_cards";
pub const WUDIAN_XJ_RUNXIN_TARGET: &str = "xj_runxin_target";
pub const WUDIAN_XJ_SAJIN_CARDS: &str = "xj_sajin_cards";
pub const WUDIAN_XJ_SAJIN_TARGETS: &str = "xj_sajin_targets";
pub const WUDIAN_XJ_SHUIYUN5_CARDS: &str = "xj_shuiyun5_cards";
pub const WUDIAN_XJ_SHUIYUN_CARDS: &str = "xj_shuiyun_cards";
pub const WUDIAN_XJ_TIANJIAN_CARDS: &str = "xj_tianjian_cards";
pub const WUDIAN_XJ_TIANXIAN_KEY: &str = "tianxian";
pub const WUDIAN_XJ_WANGYOU_CARDS: &str = "xj_wangyou_cards";
pub const WUDIAN_XJ_XIEPAN_CARDS: &str = "xj_xiepan_cards";
pub const WUDIAN_XJ_XTANXI_CARDS: &str = "xj_xtanxi_cards";
pub const WUDIAN_XJ_XTANXI_TARGET: &str = "xj_xtanxi_target";
pub const WUDIAN_XJ_XUANMO_CARDS: &str = "xj_xuanmo_cards";
pub const WUDIAN_XJ_YINGFENG_KEY: &str = "xjyingfeng";
pub const WUDIAN_XJ_ZHAOYAO_CARDS: &str = "xj_zhaoyao_cards";
pub const WUDIAN_XJ_ZHIMENG_TARGET: &str = "xj_zhimeng_target";
pub const WUDIAN_XJ_ZHUYUE_CARDS: &str = "xj_zhuyue_cards";
pub const WUDIAN_XJ_ZHUYUE_TARGETS: &str = "xj_zhuyue_targets";
pub const WUDIAN_YINBO_CARDS: &str = "wudian_yinbo_cards";
pub const WUDIAN_YUEDONG_TARGETS: &str = "wudian_yuedong_targets";
pub const WUDIAN_ZHENCHA_CHOICE: &str = "wudian_zhencha_choice";
pub const WUDIAN_ZHENCHA_DRAW_KEY: &str = "zhencha";
pub const WUDIAN_ZHUCHENG_CONFIRM: &str = "wudian_zhucheng_confirm";
pub const WUDIAN_ZHUCHENG_GUARD_ASK: &str = "wudian_zhucheng_guard";
pub const WUDIAN_ZHUCHENG_GUARD_KEY: &str = "zhucheng2";
pub const WUDIAN_ZHUCHENG_KEY: &str = "zhucheng";
pub const WUDIAN_ZIYU_CHOICE: &str = "wudian_ziyu_choice";
pub const WUDIAN_ZIYU_KEY: &str = "ziyu";
pub const WUXING_SELECT_CARD: &str = "wuxing_select_card";
pub const WUXING_SELECT_NATURE: &str = "wuxing_select_nature";
pub const XJ_ANWUGU2_KEY: &str = "anwugu2";
pub const XJ_FANLING_KEY: &str = "fanling";
pub const XJ_TANNANG_KEY: &str = "tannang";
pub const XJ_YUEXING_KEY: &str = "yuexing";
pub const YC_GEAN_KEY: &str = "yc:geanguanhuo";
pub const YC_GEAN_T1_PICK: &str = "yc_gean_t1_pick";
pub const YC_GEAN_T2_PICK: &str = "yc_gean_t2_pick";

/// 羊皮卷发现类别池（swd 第十三批 v97）：上游 `get.inpile(type)` 按通用
/// type 枚举；原生按目录牌种枚举——gwent/炉石/古剑机关/食物/欢乐等
/// 专用牌种不入池（登记边界）。Tag：0=基本 1=锦囊 2=装备。
pub fn yangpijuan_class(kind: CardKind) -> Option<u8> {
    use CardKind as K;
    match kind {
        K::Sha | K::FireSha | K::LeiSha | K::Shan | K::Tao | K::Jiu | K::Hufu
        | K::Caoyao | K::Tianxianjiu => Some(0),
        K::SavageAttack
        | K::ArcheryAttack
        | K::Duel
        | K::Nullify
        | K::Le
        | K::Bing
        | K::ShanDian
        | K::Dismantle
        | K::Snatch
        | K::Chain
        | K::Wuzhong
        | K::Taoyuan
        | K::Wugu
        | K::Jiedao
        | K::WangmeiZhike
        | K::Shuigong
        | K::Fudichouxin
        | K::Chenhuodajie
        | K::HongShui
        | K::Toulianghuanzhu
        | K::Youdishenru
        | K::Geanguanhuo
        | K::Caochuanjiejian
        | K::Guiyoujie
        | K::Zhufangshenshi
        | K::Liuxinghuoyu => Some(1),
        _ if kind.equip_slot().is_some() => Some(2),
        _ => None,
    }
}

/// 宿主技能续执行机器：显式类型化状态（挂起/恢复载荷）。
#[derive(Debug, Clone)]
pub struct HostMachine {
    /// 定义键（目录技能名；String 便于快照编解码，无泄漏风险）。
    pub def_key: String,
    pub behavior: HostBehavior,
    pub owner: PlayerId,
    pub target: Option<PlayerId>,
    /// 关联卡牌（手里剑：伤害归因）。
    pub card: Option<CardId>,
    /// 关联卡牌组（思继：本次弃置的牌）。
    pub cards: Vec<CardId>,
    /// 多目标行为的目标列表（水攻）。
    pub targets: Vec<PlayerId>,
    /// 显式续行阶段（入快照；恢复后从该阶段继续）。
    pub stage: HostStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostStage {
    /// 执行行为效果。
    Act,
    /// 手里剑：等待目标选择（弃装备 / 受伤）。
    TargetChoice,
    /// 五行盘：等待选择要改属性的手牌。
    SelectCard,
    /// 五行盘：等待选择新属性（PickTag 0..=4）。
    SelectNature,
    /// 拼点：等待目标选牌。
    CompareTargetPick,
    /// 拼点：等待己方选牌。
    ComparePlayerPick,
    /// 丈八蛇矛：等待选择两张手牌。
    CardsAsk,
    /// 分舵（习嫣 jiefen）：等待选交牌目标。
    JiefenTarget,
    /// 分舵：等待目标选交出的手牌。
    JiefenTargetDiscard,
    /// 分舵：等待自己选交出的手牌。
    JiefenOwnerDiscard,
}

impl HostStage {
    pub const fn tag(self) -> u8 {
        match self {
            HostStage::Act => 0,
            HostStage::TargetChoice => 1,
            HostStage::SelectCard => 2,
            HostStage::SelectNature => 3,
            HostStage::CompareTargetPick => 4,
            HostStage::ComparePlayerPick => 5,
            HostStage::CardsAsk => 6,
            HostStage::JiefenTarget => 7,
            HostStage::JiefenTargetDiscard => 8,
            HostStage::JiefenOwnerDiscard => 9,
        }
    }
    pub fn from_tag(t: u8) -> Option<Self> {
        Some(match t {
            0 => HostStage::Act,
            1 => HostStage::TargetChoice,
            2 => HostStage::SelectCard,
            3 => HostStage::SelectNature,
            4 => HostStage::CompareTargetPick,
            5 => HostStage::ComparePlayerPick,
            6 => HostStage::CardsAsk,
            7 => HostStage::JiefenTarget,
            8 => HostStage::JiefenTargetDiscard,
            9 => HostStage::JiefenOwnerDiscard,
            _ => return None,
        })
    }
}

/// 宿主行为实现：打包为 SkillDef 的行为载体。
pub struct HostSkillBehavior {
    pub name: String,
    pub behavior: HostBehavior,
}

#[derive(Debug, Clone, Copy)]
pub struct HostSkillDecl {
    /// 技能名（目录内唯一；桥接期固定为 'static）。
    pub name: &'static str,
    pub behavior: HostBehavior,
}


fn ctianfu_clear(ctx: &mut crate::machine::Ctx<'_>, owner: u8) {
    if let Some(cid) = ctx.core.player(owner).ctianfu_card {
        let cid = cid as u32;
        if let Some(pos) = ctx
            .core
            .player(owner)
            .off_court
            .iter()
            .position(|x| *x == cid)
        {
            ctx.core.player_mut(owner).off_court.remove(pos);
            ctx.to_discard(cid);
            ctx.log(LogEvent::CardDiscarded {
                player: owner,
                cards: vec![cid],
            });
        }
    }
    let pl = ctx.core.player_mut(owner);
    pl.ctianfu_source = None;
    pl.ctianfu_card = None;
}

impl HostSkillBehavior {
    /// 墨守：延时锦囊（乐/兵）不能指定持有者为目标。
    fn blocks_delayed_trick(&self) -> bool {
        self.behavior == HostBehavior::MoshouImmunity
    }

    fn machine(&self, owner: PlayerId, event: &EventData) -> HostMachine {
        let (target, card) = match self.behavior {
            HostBehavior::ShaDamageForceDiscard => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::ShengkeCheck => match event {
                EventData::CardUsed(u) => (None, Some(u.card)),
                _ => (None, None),
            },
            HostBehavior::MaxRangeWeaponDamage => match event {
                EventData::Equipped { card, .. } => (None, Some(*card)),
                _ => (None, None),
            },
            HostBehavior::WangmeiZhikeCard => match event {
                EventData::CardUsed(u) => (u.targets.first().copied(), Some(u.card)),
                _ => (None, None),
            },
            // 迅影：同一目标（杀结算后对同目标再杀）
            HostBehavior::XianjianXunying => match event {
                EventData::CardUsed(u) => (u.targets.first().copied(), Some(u.card)),
                _ => (None, None),
            },
            // 破云：受伤者（被弃者自选弃牌）
            HostBehavior::XianjianPoyun => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
                        HostBehavior::SijiDiscardDraw => match event {
                EventData::CardsDiscarded { cards: _, .. } => (None, None),
                _ => (None, None),
            },
            HostBehavior::SanjianWeaponDamage => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::MoshouImmunity => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::ShuigongCard => match event {
                EventData::CardUsed(u) => (None, Some(u.card)),
                _ => (None, None),
            },
            HostBehavior::FudichouxinCompare => match event {
                EventData::CardUsed(u) => (
                    u.targets.first().copied(),
                    Some(u.card),
                ),
                _ => (None, None),
            },
            HostBehavior::ChenhuodajieCard => match event {
                EventData::CardUsed(u) => (
                    u.targets.first().copied(),
                    Some(u.card),
                ),
                _ => (None, None),
            },
            HostBehavior::TouliangSwap => match event {
                EventData::CardUsed(u) => (
                    u.targets.first().copied(),
                    Some(u.card),
                ),
                _ => (None, None),
            },
            HostBehavior::LinghunzhihuoCard => match event {
                EventData::CardUsed(u) => (
                    u.targets.first().copied(),
                    Some(u.card),
                ),
                _ => (None, None),
            },
            HostBehavior::ZhaomingdanCard => match event {
                EventData::CardUsed(u) => (
                    u.targets.first().copied(),
                    Some(u.card),
                ),
                _ => (None, None),
            },
            HostBehavior::JihuocardDraw => match event {
                EventData::CardUsed(u) => (Some(u.player), Some(u.card)),
                _ => (None, None),
            },
            HostBehavior::GeanguanhuoCompare => match event {
                EventData::CardUsed(u) => (None, Some(u.card)),
                _ => (None, None),
            },
            HostBehavior::CaochuanCard => match event {
                EventData::CardUsed(u) => (
                    u.targets.first().copied(),
                    Some(u.card),
                ),
                _ => (None, None),
            },
            HostBehavior::ShezhanqunruCompare => match event {
                EventData::CardUsed(u) => (None, Some(u.card)),
                _ => (None, None),
            },
            HostBehavior::JinlianzhuCancel => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::WudianShuaiyan => match event {
                EventData::Recover { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::WudianZhuchengGuard => match event {
                EventData::CardUsed(u) => (Some(u.player), Some(u.card)),
                _ => (None, None),
            },
            HostBehavior::WudianMujing => match event {
                EventData::CardUsed(u) => (Some(u.player), Some(u.card)),
                _ => (None, None),
            },
            HostBehavior::WudianGuangshuHeart => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::WudianGuangshuDiamond => match event {
                EventData::Damage(d) => (Some(d.source.unwrap_or(owner)), None),
                _ => (None, None),
            },
            HostBehavior::WudianFeidan => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::WudianZhiyuan => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::WudianQiangguGuard => match event {
                EventData::CardUsed(u) => (Some(u.player), Some(u.card)),
                _ => (None, None),
            },
            HostBehavior::WudianPingzhang => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::XianjianBinxin => match event {
                EventData::Phase { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::XianjianFeixu => match event {
                EventData::CardUsed(u) => (Some(u.player), None),
                EventData::CardResponded { card: _, player } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::XianjianAnwugu => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::XianjianLinghuo => match event {
                EventData::Phase { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::XianjianShuiyun5 => match event {
                EventData::Dying { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::XianjianWangyou2 => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::XianjianChengxin => match event {
                EventData::Dying { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::XianjianShenmu => match event {
                EventData::Dying { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::XianjianHushaCount => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            HostBehavior::XianjianShiying => match event {
                EventData::Death { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::XianjianZhaoyao => match event {
                EventData::Phase { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::XianjianCtianfu3 => match event {
                EventData::Death { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            HostBehavior::SwdGouhunluoCard => match event {
                EventData::CardUsed(u) => (u.targets.first().copied(), Some(u.card)),
                _ => (None, None),
            },
            // 雄姿：自身（补摸量在效果内按装备数计算）
            HostBehavior::YunchouXiongzi => match event {
                EventData::Phase { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            // 雅量：伤害目标（双方经 targets=[source,target] 提取摸牌）
            HostBehavior::YunchouYaliang => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 戏才：伤害目标+伤害牌（获得面）
            HostBehavior::YunchouXicai => match event {
                EventData::Damage(d) => (Some(d.target), d.card),
                _ => (None, None),
            },
            // 狂骨·变体：伤害目标（距离判定面）
            HostBehavior::YunchouDiykuanggu => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 纵火：伤害目标（帧直写在 step）
            HostBehavior::YunchouZonghuo => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 烧营：受伤者（蔓延目标在 step 内按距离自算）
            HostBehavior::YunchouShaoying => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 蛊惑·变体：自身
            HostBehavior::YunchouDiyguhuo => match event {
                EventData::Phase { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            // 狡黠：自身（摸 1）
            HostBehavior::YunchouJiaoxia => (Some(owner), None),
            // 复仇：目录锚（无提取）
            HostBehavior::WudianFuchou => (None, None),
            // 归汉：目录锚（无提取）
            HostBehavior::WudianGuihan => (None, None),
            // 铁壁：杀的首个非持有者目标（活/距/护甲校验在 filter）
            HostBehavior::SwdTiebi => match event {
                EventData::CardUsed(u) => (u.targets.first().copied(), None),
                _ => (None, None),
            },
            // 大同：自己（摸 2）
            HostBehavior::SwdDatong => (Some(owner), None),
            // 承剑：伤害来源（摸 1 面在效果）
            HostBehavior::SwdChengjian => match event {
                EventData::Damage(d) => (d.source, None),
                _ => (None, None),
            },
            // 缚斧：伤害目标（效果直写帧）
            HostBehavior::SwdJufu => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 花镜：伤害目标
            HostBehavior::SwdHuajing => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 护盾：自己（+1 护甲）
            HostBehavior::SwdHudun => (Some(owner), None),
            // 破星：伤害目标（效果直写帧）
            HostBehavior::SwdPoxing => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 流虹：自己（摸 1）
            HostBehavior::SwdLiuhong => (Some(owner), None),
            // 墨剑：杀的首个目标（摸 1；回 1 面在效果）
            HostBehavior::SwdMojian => match event {
                EventData::CardUsed(u) => (u.targets.first().copied(), None),
                _ => (None, None),
            },
            // 道法：伤害来源（弃其手牌）
            HostBehavior::SwdDaofa => match event {
                EventData::Damage(d) => (d.source, None),
                _ => (None, None),
            },
            // 玉鳞：无目标（弃自己装备牌 + 免伤）
            HostBehavior::SwdYulin => (None, None),
            // 凝霜：黑牌使用者（阶梯对象）
            HostBehavior::SwdNingshuang => match event {
                EventData::CardUsed(u) => (Some(u.player), None),
                _ => (None, None),
            },
            // 工神：用牌者（取消其使用）
            HostBehavior::SwdGongshen => match event {
                EventData::CardUsed(u) => (Some(u.player), None),
                _ => (None, None),
            },
            // 屠阵：受伤者（弃其非基本手牌）
            HostBehavior::SwdTuzhen => match event {
                EventData::Damage(d) => (Some(d.target), None),
                _ => (None, None),
            },
            // 女娲石：濒死者（回复其 1）
            HostBehavior::SwdNvwashiGuard => match event {
                EventData::Dying { player, .. } => (Some(*player), None),
                _ => (None, None),
            },
            // 悲歌：targets=[来源, 受伤者]
            HostBehavior::YunchouBeige => match event {
                EventData::Damage(d) => (
                    Some(d.target),
                    None,
                ),
                _ => (None, None),
            },
            // 蝉缘·变体：死亡来源（减其上限）
            HostBehavior::YunchouDiychanyuan => match event {
                EventData::Death { killer: Some(k), .. } => (Some(*k), None),
                _ => (None, None),
            },
            // 理断：获得者+所获装备（第 9 触点：card 必须提取）
            HostBehavior::WudianLiduan => match event {
                EventData::CardsMoved { cards, to, .. } => {
                    (Some(*to), cards.first().copied())
                }
                _ => (None, None),
            },
            // 厉焰·链杀：用水攻的持有者（目标取 targets 首）
            HostBehavior::YunchouLuweiyan2 => (Some(owner), None),
            // 红莲：伤害来源（弃其红牌）
            HostBehavior::YunchouHonglian => match event {
                EventData::Damage(d) => (d.source, None),
                _ => (None, None),
            },
        _ => (None, None),
        };
        let stage = match self.behavior {
            HostBehavior::TargetDiscardEquipOrDamage => HostStage::TargetChoice,
            HostBehavior::MaxRangeWeaponDamage => HostStage::TargetChoice,
            HostBehavior::SanjianWeaponDamage => HostStage::SelectCard,
            HostBehavior::FudichouxinCompare => HostStage::CompareTargetPick,
            HostBehavior::CaochuanCard => HostStage::SelectNature,
            HostBehavior::ChangeCardNature => HostStage::SelectCard,
            HostBehavior::TouliangSwap => HostStage::SelectCard,
            HostBehavior::GeanguanhuoCompare => HostStage::CompareTargetPick,
            HostBehavior::GujianShujinsanCard => HostStage::SelectCard,
            HostBehavior::YunchouYiesheng => HostStage::SelectCard,
            HostBehavior::WudianDiyjuntun => HostStage::SelectCard,
            HostBehavior::GujianFeibiaoCard => HostStage::SelectCard,
            HostBehavior::JiguanBingpoCard => HostStage::ComparePlayerPick,
            HostBehavior::JiguanLongxuCard => HostStage::SelectCard,
            HostBehavior::GwentBaoxueCard => HostStage::SelectCard,
            HostBehavior::WudianKuoyin => HostStage::SelectCard,
            HostBehavior::WudianHuhuan => HostStage::SelectCard,
            HostBehavior::WudianBaoxue => HostStage::SelectCard,
            HostBehavior::WudianLiangou => HostStage::SelectCard,
            HostBehavior::WudianYinbo => HostStage::SelectCard,
            HostBehavior::WudianQianggu => HostStage::SelectCard,
            HostBehavior::WudianFeitiao => HostStage::SelectCard,
            HostBehavior::XianjianGuiyuan => HostStage::SelectCard,
            HostBehavior::XianjianDiewu => HostStage::SelectCard,
            HostBehavior::XianjianSajin => HostStage::SelectCard,
            HostBehavior::XianjianYuexing => HostStage::Act,
            HostBehavior::XianjianMinsha => HostStage::SelectCard,
            HostBehavior::XianjianZhuyue => HostStage::SelectCard,
            HostBehavior::XianjianGuanri => HostStage::SelectCard,
            HostBehavior::XianjianShuiyun => HostStage::SelectCard,
            HostBehavior::XianjianWangyou => HostStage::SelectCard,
            HostBehavior::XianjianXuanmo => HostStage::SelectCard,
            HostBehavior::XianjianHuahu => HostStage::TargetChoice,
            HostBehavior::XianjianXtanxi => HostStage::SelectCard,
            HostBehavior::XianjianTianjian => HostStage::SelectCard,
            HostBehavior::SwdDonghuangzhongCard => HostStage::SelectCard,
            _ => HostStage::Act,
        };
        let cards = match self.behavior {
            HostBehavior::SijiDiscardDraw => match event {
                EventData::CardsDiscarded { cards, .. } => cards.clone(),
                _ => Vec::new(),
            },
            // 聚宝：本次弃入弃堆的牌（随机回收候选池）
            HostBehavior::XianjianJtjubao => match event {
                EventData::CardsDiscarded { cards, .. } => cards.clone(),
                _ => Vec::new(),
            },
            // 摄灵：本次事件入弃堆的牌（回收候选）
            HostBehavior::XianjianSheling => match event {
                EventData::CardUsed(u) => vec![u.card],
                EventData::CardResponded { card, .. } => vec![*card],
                EventData::CardsDiscarded { cards, .. } => cards.clone(),
                _ => Vec::new(),
            },
            // 双刃：本次弃入弃堆的牌（武器识别候选）
            HostBehavior::XianjianXshuangren => match event {
                EventData::CardsDiscarded { cards, .. } => cards.clone(),
                _ => Vec::new(),
            },
            // 射影：伤害牌（判定黑回收目标）
            HostBehavior::XianjianSheying => match event {
                EventData::Damage(d) => d.card.map(|c| vec![c]).unwrap_or_default(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        let targets = match self.behavior {
            HostBehavior::ShuigongCard => match event {
                EventData::CardUsed(u) => u.targets.clone(),
                _ => Vec::new(),
            },
            HostBehavior::JinlianzhuCancel => match event {
                EventData::Damage(d) => d.source.map(|s| vec![s]).unwrap_or_default(),
                _ => Vec::new(),
            },
            // 雅量：伤害双方（座位序在效果内排序）
            HostBehavior::YunchouYaliang => match event {
                EventData::Damage(d) => vec![d.source.unwrap_or(d.target), d.target],
                _ => Vec::new(),
            },
            // 厉焰·链杀：水攻目标集
            HostBehavior::YunchouLuweiyan2 => match event {
                EventData::CardUsed(u) => u.targets.clone(),
                _ => Vec::new(),
            },
            // 悲歌：targets=[来源, 受伤者]
            HostBehavior::YunchouBeige => match event {
                EventData::Damage(d) => vec![d.source.unwrap_or(d.target), d.target],
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        HostMachine {
            def_key: self.name.clone(),
            behavior: self.behavior,
            owner,
            target,
            card,
            cards,
            targets,
            stage,
        }
    }
}

impl SkillBehavior for HostSkillBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        match self.behavior {
            // 厉焰·链杀：持有者以水攻用牌（使用者=持有者；kind 按**结算
            // 种**（use_as 转化声明优先，目录种兜底）——厉焰转化的牌目录
            // 种非水攻）
                        // 厉焰·链杀：持有者以水攻用牌（结算种 use_as 优先；语义集
            // 中在 helper）
            HostBehavior::YunchouLuweiyan2 => u_matches_lw2(event, owner, ctx),
            HostBehavior::YunchouJieyan => false,
            HostBehavior::YunchouYiesheng => false,
            HostBehavior::WudianDiyjuntun => false,
            HostBehavior::WudianJianbi => false,
            // 复仇·清算：自己 End 阶段结束（登记差异：上游 global
            // phaseAfter，原生自己回合结束）
            HostBehavior::WudianFuchou2 => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player == owner),
            HostBehavior::WudianFuchou => false,
            HostBehavior::WudianLiangceRemain => false,
            HostBehavior::WudianGuihan => false,
            // 铁壁：杀指定了与持有者距离 ≤1 的目标，持有者有黑牌
            HostBehavior::SwdTiebi => {
                let hit = matches!(event,
                    EventData::CardUsed(u)
                    if ctx.catalog.card_kind(u.card).is_sha_like()
                        && u.targets.iter().any(|&t| {
                            t != owner
                                && ctx.state.player(t).alive
                                && ctx.state.player(t).hujia == 0
                                && ctx.state.seat_distance(t, owner) <= 1
                        })
                        && ctx.state.player(owner).hand.iter().any(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                            )
                        }));
                if std::env::var("TB_DBG").is_ok() {
                    if let EventData::CardUsed(u) = event {
                        eprintln!(
                            "DBG tb owner={owner} kind={:?} targets={:?} hit={hit}",
                            ctx.catalog.card_kind(u.card),
                            u.targets
                        );
                    }
                }
                hit
            },
            // 大同：任意角色任意阶段结束（global phaseEnd）全场极差 ≤1
            HostBehavior::SwdDatong => matches!(event,
                EventData::Phase { begin: false, .. }
                if {
                    let counts: Vec<usize> = ctx.state.players.iter()
                        .filter(|p| p.alive)
                        .map(|p| p.hand.len())
                        .collect();
                    let max = counts.iter().copied().max().unwrap_or(0);
                    let min = counts.iter().copied().min().unwrap_or(0);
                    max - min <= 1
                }),
            // 承剑：其他角色受到杀伤害（来源≠持有者）
            HostBehavior::SwdChengjian => matches!(event,
                EventData::Damage(d)
                if !d.prevented && d.amount > 0
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)
                    && d.source.is_some()
                    && d.source != Some(owner)
                    && ctx.state.player(d.source.unwrap()).alive),
            // 缚斧：持有者对目标用杀伤害前置（有武器）
            HostBehavior::SwdJufu => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)
                    && ctx.state.player(owner).equips.iter().any(|c| {
                        ctx.catalog.card_kind(*c).equip_slot()
                            == Some(crate::catalog::EquipSlot::Weapon)
                    })),
            // 花镜：持有者造成的锦囊牌伤害
            HostBehavior::SwdHuajing => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0
                    && d.card.map(|c| {
                        let k = ctx.catalog.card_kind(c);
                        !k.is_basic_kind()
                            && k.equip_slot().is_none()
                            && k.is_basic_trick_or_equip()
                    }).unwrap_or(false)),
            // 护盾：持有者造成伤害后自己无护甲
            HostBehavior::SwdHudun => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0
                    && ctx.state.player(owner).hujia == 0),
            // 破星：持有者对体力高于自己的目标造成伤害
            HostBehavior::SwdPoxing => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0
                    && d.target != owner
                    && ctx.state.player(d.target).hp > ctx.state.player(owner).hp),
            // 流虹：持有者使用杀
            HostBehavior::SwdLiuhong => matches!(event,
                EventData::CardUsed(u)
                if u.player == owner
                    && ctx.catalog.card_kind(u.card).is_sha_like()),
            // 墨剑：持有者使用杀且指定了目标（上游 shaBegin=自己用杀）
            HostBehavior::SwdMojian => matches!(event,
                EventData::CardUsed(u)
                if u.player == owner
                    && ctx.catalog.card_kind(u.card).is_sha_like()
                    && !u.targets.is_empty()),
            // 道法：其他角色受伤且来源有牌
            HostBehavior::SwdDaofa => matches!(event,
                EventData::Damage(d)
                if !d.prevented && d.amount > 0
                    && d.source.is_some()
                    && d.source != Some(owner)
                    && !ctx.state.player(d.source.unwrap()).hand.is_empty()),
            // 玉鳞：持有者即将受伤且手/装备区有装备牌
            HostBehavior::SwdYulin => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0
                    && (!ctx.state.player(owner).equips.is_empty()
                        || ctx.state.player(owner).hand.iter().any(|c| {
                            ctx.catalog.card_kind(*c).equip_slot().is_some()
                        }))),
            // 平恤：无触发面
            HostBehavior::SwdPingxu => false,
            // 还灵：自己任一阶段结束（上游 player phaseEnd 逐阶段均触发）
            HostBehavior::SwdHuanling => matches!(event,
                EventData::Phase { player, begin, .. }
                if *player == owner && !*begin),
            // 屠阵：自己伤害他人结算后且其有非基本手牌
            HostBehavior::SwdTuzhen => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0
                    && d.target != owner
                    && ctx.state.player(d.target).alive
                    && ctx.state.player(d.target).hand.iter().any(|c| {
                        !ctx.catalog.card_kind(*c).is_basic_kind()
                    })),
            // 协律：自己弃牌阶段结束（End 入场）且手牌≥2 同色
            HostBehavior::SwdXielv => matches!(event,
                EventData::Phase { player, phase, begin }
                if *player == owner
                    && *phase == crate::event::Phase::End
                    && *begin
                    && {
                        let hand = &ctx.state.player(owner).hand;
                        hand.len() >= 2
                            && {
                                let first =
                                    ctx.catalog.cards[hand[0] as usize].suit;
                                hand.iter().all(|c| {
                                    let red = |su| {
                                        matches!(
                                            su,
                                            crate::catalog::Suit::Heart
                                                | crate::catalog::Suit::Diamond
                                        )
                                    };
                                    red(ctx.catalog.cards[*c as usize].suit)
                                        == red(first)
                                })
                            }
                    }),
            // 雪煌：无触发面
            HostBehavior::SwdXuehuang => false,
            // 还侠：无触发面
            HostBehavior::SwdHuanxia => false,
            // 破月：无触发面
            HostBehavior::SwdPoyue => false,
            // 女娲石：濒死者 hp≤0 且自己 hp>1（上游 global dying 同口径；
    // 含自身濒死时 owner hp>1 恒否自然排除）
            HostBehavior::SwdNvwashiGuard => matches!(event,
                EventData::Dying { player, .. }
                if ctx.state.player(*player).hp <= 0
                    && ctx.state.player(owner).hp > 1),
            // 收纳/炼化/神农鼎：无触发面
            HostBehavior::SwdLianyaohuShouna => false,
            HostBehavior::SwdLianyaohuLianhua => false,
            HostBehavior::SwdShennongdingCard => false,
            // 昆仑镜：无触发面
            HostBehavior::SwdKunlunjingcCard => false,
            // 鬼蛇旗/仙露回：无触发面（CardUsed 内建直派）
            HostBehavior::GuisheqiCard => false,
            HostBehavior::XianluhuiCard => false,
            // 手持门控 helper（四象五件共用）：持有者手牌含对应器族实体
            // 青龙之圭：自己准备阶段且手持
            HostBehavior::SwdQinglongzhiguiHeld => matches!(event,
                EventData::Phase { player, phase, begin }
                if *player == owner
                    && *phase == crate::event::Phase::Judge
                    && *begin
                    && hand_has_kind(ctx, owner, CardKind::Qinglongzhigui)),
            // 逆攻积攒：自己受到真实伤害
            HostBehavior::SwdNigongAccumulate => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0),
            // 逆攻发动：无触发面
            HostBehavior::SwdNigongStrike => false,
            // 四非珍面：无触发面
            HostBehavior::SwdSifeizhenmianCard => false,
            // 朱雀之璋：受他人伤害结算后且手持（来源存活且非自己）
            HostBehavior::SwdZhuquezhizhangHeld => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0
                    && d.source.is_some()
                    && d.source != Some(owner)
                    && ctx.state.player(d.source.unwrap()).alive
                    && hand_has_kind(ctx, owner, CardKind::Zhuquezhizhang)),
            // 白兽之琥：自己弃牌阶段结束且手持且有红牌
            HostBehavior::SwdBaishouzhihuHeld => matches!(event,
                EventData::Phase { player, begin, .. }
                if *player == owner
                    && *begin
                    && ctx.state.player(owner).hand.iter().any(|c| {
                        matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Heart
                                | crate::catalog::Suit::Diamond
                        )
                    })
                    && hand_has_kind(ctx, owner, CardKind::Baishouzhihu)),
            // 玄武之璜：自己造成伤害结算后且手持且受伤
            HostBehavior::SwdXuanwuzhihuangHeld => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0
                    && ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
                    && hand_has_kind(ctx, owner, CardKind::Xuanwuzhihuang)),
            // 玉蝉换形：自己用牌（非玉蝉本体）结算后且手牌有玉蝉
            HostBehavior::SwdYuchanSwap => matches!(event,
                EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && ctx.state.player(owner).hand.iter().any(|c| {
                        ctx.catalog.card_kind(*c).yuchan_auto_view_as().is_some()
                    })),
            // 疾影：无触发面
            HostBehavior::SwdJiyingHeld => false,
            // 沐风：结束阶段且本回合用过基本牌
            HostBehavior::SwdMufengHeld => matches!(event,
                EventData::Phase { player, begin, .. }
                if *player == owner && !*begin
                    && ctx.state.turn.mufeng_used_basic),
            // 望尘：弃牌阶段结束（End 入场）且弃置过基本牌
            HostBehavior::SwdWangchenHeld => matches!(event,
                EventData::Phase { player, phase, begin }
                if *player == owner
                    && *phase == crate::event::Phase::End
                    && *begin
                    && ctx.state.turn.wangchen_discard_basic),
            // 神草结：锦囊伤害且自己是受伤者或来源、手持神草结
            HostBehavior::SwdShencaojieGuard => matches!(event,
                EventData::Damage(d)
                if !d.prevented && d.amount > 0
                    && d.card.map(|c| {
                        let k = ctx.catalog.card_kind(c);
                        !k.is_basic_kind() && k.equip_slot().is_none()
                    })
                    .unwrap_or(false)
                    && (d.target == owner || d.source == Some(owner))
                    && hand_has_kind(ctx, owner, CardKind::Shencaojie)),
            // 黄麟之琮：自己准备阶段无护甲且手持
            HostBehavior::SwdHuanglinzhicongHeld => matches!(event,
                EventData::Phase { player, phase, begin }
                if *player == owner
                    && *phase == crate::event::Phase::Judge
                    && *begin
                    && ctx.state.player(owner).hujia == 0
                    && hand_has_kind(ctx, owner, CardKind::Huanglinzhicong)),
            // 空洞印：自己死亡（maxHp>0 gate 恒真登记）
            HostBehavior::SwdKongdongyinGuard => matches!(event,
                EventData::Death { player, .. } if *player == owner),
            // 东皇钟：自己任一阶段结束且手牌有红牌（蝶舞口径）
            HostBehavior::SwdDonghuangzhongCard => matches!(event,
                EventData::Phase { player, begin, .. }
                if *player == owner && !*begin
                    && ctx.state.player(owner).hand.iter().any(|c| {
                        matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Heart
                                | crate::catalog::Suit::Diamond
                        )
                    })),
            // 壶境：自己准备阶段入场且弃牌堆有炼妖壶
            HostBehavior::SwdHujing => matches!(event,
                EventData::Phase { player, phase, begin }
                if *player == owner
                    && *phase == crate::event::Phase::Judge
                    && *begin
                    && ctx.state.discard.iter().any(|c| {
                        ctx.catalog.card_kind(*c) == CardKind::Lianyaohu
                    })),
            // 朱羽：横置角色受到非传导伤害且持有者有红牌（手/装备区）
            HostBehavior::SwdZhuyu => matches!(event,
                EventData::Damage(d)
                if !d.prevented && d.amount > 0
                    && !d.from_chain
                    && ctx.state.player(d.target).linked
                    && ctx.state.player(owner).hand.iter()
                        .chain(ctx.state.player(owner).equips.iter())
                        .any(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Heart
                                    | crate::catalog::Suit::Diamond
                            )
                        })),
            // 落梅：持有者使用或打出梅花牌
            HostBehavior::SwdLuomei => match event {
                EventData::CardUsed(u) => {
                    u.player == owner
                        && matches!(ctx.catalog.cards[u.card as usize].suit,
                            crate::catalog::Suit::Club)
                }
                EventData::CardResponded { card, player } => {
                    *player == owner
                        && matches!(ctx.catalog.cards[*card as usize].suit,
                            crate::catalog::Suit::Club)
                }
                _ => false,
            },
            // 凝霜：成为黑色牌的目标（使用者≠持有者、未「横置且翻面」、
            // 持有者手/装备区有黑牌）
            HostBehavior::SwdNingshuang => matches!(event,
                EventData::CardUsed(u)
                if u.player != owner
                    && u.targets.contains(&owner)
                    && !u.cancelled
                    && matches!(ctx.catalog.cards[u.card as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club)
                    && !(ctx.state.player(u.player).linked
                        && ctx.state.player(u.player).face_down)
                    && ctx.state.player(owner).hand.iter()
                        .chain(ctx.state.player(owner).equips.iter())
                        .any(|c| matches!(ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club))),
            // 工神：其他角色使用基本/锦囊牌且有目标（持有者手/装备区有
            // 装备牌；地/机关等异类种不计）
            HostBehavior::SwdGongshen => matches!(event,
                EventData::CardUsed(u)
                if u.player != owner
                    && !u.cancelled
                    && !u.targets.is_empty()
                    && {
                        let k = ctx.catalog.card_kind(u.card);
                        k.is_basic_kind()
                            || (k.is_basic_trick_or_equip() && k.equip_slot().is_none())
                    }
                    && (!ctx.state.player(owner).equips.is_empty()
                        || ctx.state.player(owner).hand.iter().any(|c| {
                            ctx.catalog.card_kind(*c).equip_slot().is_some()
                        }))),
            // 悲歌：伤害由杀造成、受伤者在场、持有者有牌
            HostBehavior::YunchouBeige => matches!(event,
                EventData::Damage(d)
                if !d.prevented && d.amount > 0
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)
                    && ctx.state.player(d.target).alive
                    && (!ctx.state.player(owner).hand.is_empty()
                        || !ctx.state.player(owner).equips.is_empty())),
            HostBehavior::WudianLiduan => matches!(event,
                EventData::CardsMoved { cards, from, to }
                if *to != owner
                    && from != to
                    && cards.len() == 1
                    && ctx.state.player(*to).alive
                    && ctx.state.turn.current != *to
                    && ctx.state.player(*to).hand.contains(&cards[0])
                    && ctx.catalog.card_kind(cards[0]).equip_slot().is_some()),
            // 蝉缘·变体：有来源的死亡
            HostBehavior::YunchouDiychanyuan => matches!(event,
                EventData::Death { killer: Some(_), .. }),
            // 蛊惑·变体：自己回合开始且区域有牌
            HostBehavior::YunchouDiyguhuo => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Judge, begin: true }
                if *player == owner
                    && (!ctx.state.player(owner).hand.is_empty()
                        || !ctx.state.player(owner).equips.is_empty()
                        || !ctx.state.player(owner).judge.is_empty())),
            // 狡黠：成为红色牌的目标（使用者不限）
            HostBehavior::YunchouJiaoxia => matches!(event,
                EventData::CardUsed(u)
                if u.targets.contains(&owner)
                    && matches!(
                        ctx.catalog.cards[u.card as usize].suit,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    )),
            // 红莲：受到来源为其他角色的伤害且来源有红牌
            HostBehavior::YunchouHonglian => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0
                    && d.source.is_some() && d.source != Some(owner)
                    && {
                        let src = ctx.state.player(d.source.unwrap());
                        src.hand.iter().chain(src.equips.iter()).any(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                            )
                        })
                    }),
            // 纵火：自己造成的非火伤害（前置窗）
            HostBehavior::YunchouZonghuo => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.nature != crate::event::DamageNature::Fire),
            // 烧营：自己造成的火伤害
            HostBehavior::YunchouShaoying => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.nature == crate::event::DamageNature::Fire),
            // 雄姿：自己摸牌阶段开始（固定摸 2 后补摸）
            HostBehavior::YunchouXiongzi => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Draw, begin: true }
                if *player == owner),
            // 雅量：自己造成或受到伤害，双方存活且异人
            HostBehavior::YunchouYaliang => matches!(event,
                EventData::Damage(d)
                if !d.prevented && d.amount > 0
                    && (d.source == Some(owner) || d.target == owner)
                    && d.source.is_some()
                    && d.source != Some(d.target)
                    && ctx.state.player(d.source.unwrap()).alive
                    && ctx.state.player(d.target).alive),
            // 戏才：自己受到有牌伤害（可选确认门走 trigger_meta optional）
            HostBehavior::YunchouXicai => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0 && d.card.is_some()),
            // 狂骨·变体：自己造成伤害；满血时要求目标攻击距离 >1（受伤
            // 恒真——回复分支在效果内自检）
            HostBehavior::YunchouDiykuanggu => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0 && d.target != owner
                    && (ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
                        || ctx.state.seat_distance(d.target, owner) > 1)),
            HostBehavior::ShaDamageForceDiscard => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)),
            HostBehavior::PhaseEndRecoverOrDraw => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: true }
                if *player == owner),
            HostBehavior::TargetDiscardEquipOrDamage => false,
            HostBehavior::ShengkeCheck => matches!(event,
                EventData::CardUsed(u)
                if u.targets.contains(&owner)
                    && ctx.state.card_natures.contains_key(&u.card)
                    && ctx.state.player(owner).wunature.is_some()),
            HostBehavior::ChangeCardNature => false,
            HostBehavior::WangmeiZhikeCard => false,
            HostBehavior::ShuigongCard => false,
            HostBehavior::FudichouxinCompare => false,
            HostBehavior::ChenhuodajieCard => false,
            HostBehavior::TouliangSwap => false,
            HostBehavior::LinghunzhihuoCard => false,
            HostBehavior::ZhaomingdanCard => false,
            HostBehavior::JihuocardDraw => false,
            HostBehavior::GeanguanhuoCompare => false,
            HostBehavior::CaochuanCard => false,
            HostBehavior::ShezhanqunruCompare => false,
            HostBehavior::GujianShujinsanCard => false,
            HostBehavior::GujianMianliCard => false,
            HostBehavior::GujianFeibiaoCard => false,
            HostBehavior::JinlianzhuCancel => matches!(event,
                EventData::Damage(d)
                if !d.prevented && d.amount > 0
                    && ctx.state.player(owner).hand.iter()
                        .any(|c| ctx.catalog.card_kind(*c) == CardKind::Jinlianzhu)),
            HostBehavior::MoshouImmunity => false,
            HostBehavior::SanjianWeaponDamage => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)
                    && !ctx.state.player(owner).hand.is_empty()),
            HostBehavior::SijiDiscardDraw => matches!(event,
                EventData::CardsDiscarded { player, cards }
                if *player == owner
                    && cards.iter().any(|c| ctx.catalog.card_kind(*c).is_sha_like())),
            HostBehavior::MaxRangeWeaponDamage => matches!(event,
                EventData::Equipped { player, card, .. }
                if *player == owner
                    && ctx.catalog.card_kind(*card) == CardKind::Weapon
                    && {
                        // 攻击范围全场最大（严格大于其余所有存活角色）
                        let my = ctx.state.attack_range(ctx.catalog, owner);
                        ctx.state.alive_players().filter(|p| *p != owner).all(|p| {
                            ctx.state.attack_range(ctx.catalog, p) < my
                        })
                    }),
            HostBehavior::XianjianXiepan => matches!(event,
                EventData::CardsDiscarded { player, cards }
                if *player == owner
                    && cards.iter().any(|c| {
                        matches!(
                            ctx.catalog.card_kind(*c),
                            CardKind::Sha | CardKind::Shan | CardKind::Tao
                        )
                    })
                    && !ctx.state.player(owner).hand.is_empty()
                    && !ctx.state.player(owner).hand.iter().any(|c| {
                        matches!(
                            ctx.catalog.card_kind(*c),
                            CardKind::Sha | CardKind::Shan | CardKind::Tao
                        )
                    })),
            HostBehavior::XianjianXjyingfeng => {
                // 影锋（U05 xianjian 越今朝，规则 v185）：自己用杀+链挡
                // 旗标未置+候选存在（其他角色-本杀目标，存活+距离内）
                // ——上游 card=sha && parent.name!=="xjyingfeng" &&
                // canUse 的登记差异等价。
                if let EventData::CardUsed(u) = event {
                    u.player == owner
                        && ctx.catalog.card_kind(u.card)
                            == crate::catalog::CardKind::Sha
                        && !ctx.state.turn.yingfeng_chain
                        && ctx.state.seat_order_from(owner).into_iter().any(|t| {
                            t != owner
                                && !u.targets.contains(&t)
                                && ctx.state.player(t).alive
                                && ctx
                                    .state
                                    .computed_distance(ctx.catalog, owner, t)
                                    <= ctx.state.attack_range(ctx.catalog, owner)
                        })
                } else {
                    false
                }
            }
            HostBehavior::XianjianXjzhimeng2 => {
                if let EventData::CardUsed(u) = event {
                    u.player == owner && {
                        let k =
                            yangpijuan_class(ctx.catalog.card_kind(u.card));
                        ctx.state.player(owner).off_court.iter().any(|o| {
                            yangpijuan_class(ctx.catalog.card_kind(*o)) == k
                        })
                    }
                } else {
                    false
                }
            },
            HostBehavior::XianjianSheling => {
                let in_my_turn = ctx.state.turn.current == owner;
                let others = match event {
                    EventData::CardUsed(u) => u.player != owner,
                    EventData::CardResponded { player, .. } => *player != owner,
                    EventData::CardsDiscarded { player, .. } => *player != owner,
                    _ => false,
                };
                in_my_turn
                    && others
                    && ctx.state.turn.sheling_used < 3
                    && match event {
                        EventData::CardUsed(u) => ctx.state.discard.contains(&u.card),
                        EventData::CardResponded { card, .. } => ctx.state.discard.contains(card),
                        EventData::CardsDiscarded { cards, .. } => cards
                            .iter()
                            .any(|c| ctx.state.discard.contains(c)),
                        _ => false,
                    }
            },
            HostBehavior::WudianZhenchaDraw => matches!(event,
                EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && u.use_as.unwrap_or(ctx.catalog.card_kind(u.card)) == CardKind::Sha
                    && ctx.state.player(owner).baolei_mode == 2),
            HostBehavior::WudianZiyu => matches!(event,
                EventData::Phase { player: _, phase: crate::event::Phase::End, begin: true }
                if ctx.state.turn.phase_number > 0
                    && ctx.state.turn.phase_number % 4 == 0),
            HostBehavior::XianjianFanling => matches!(event,
                EventData::LoseHp { player, amount }
                if *player != owner
                    && *amount > 0
                    && !ctx.state.turn.fanling_used
                    && (ctx.state.player(owner).hp as u32) < ctx.state.player(owner).max_hp),
            HostBehavior::XianjianNingbin => matches!(event,
                EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == crate::event::DamageNature::Thunder),
            HostBehavior::XianjianLingyu => {
                matches!(event,
                    EventData::Phase { player, begin: false, .. }
                    if *player == owner)
                    && ctx.state.alive_players().any(|p| {
                        p != owner && (ctx.state.player(p).hp as u32) < ctx.state.player(p).max_hp
                    })
            }
            HostBehavior::XianjianTxianqu => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.turn.current == owner
                    && ctx.state.turn.phase == crate::event::Phase::Play
                    && !ctx.state.turn.txianqu_used),
            HostBehavior::XianjianCtianfu3 => matches!(event,
                EventData::Death { player, .. }
                if *player == owner
                    && ctx.state.player(owner).ctianfu_source.is_some()),
            HostBehavior::WudianGuangshuSpade => matches!(event,
                EventData::Phase { player, begin: false, .. } if *player == owner),
            HostBehavior::GujianFoodGudonggeng => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)),
            HostBehavior::GujianFoodXiajiao => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Draw, begin: true }
                if *player == owner),
            HostBehavior::GujianFoodLiyutang => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: true }
                if *player == owner),
            HostBehavior::GujianFoodMapodoufu => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: true }
                if *player == owner),
            HostBehavior::GujianFoodQingtuan => matches!(event,
                EventData::CardUsed(u)
                if u.player == owner
                    && ctx.catalog.card_kind(u.card).is_sha_like()
                    && ctx.state.turn.current == owner
                    && ctx.state.turn.phase == crate::event::Phase::Play),
            HostBehavior::GujianFoodTanhuadong => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Play, begin: false }
                if *player == owner),
            HostBehavior::GujianFoodYougeng => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Judge, begin: true }
                if *player == owner),
            HostBehavior::GujianFoodZiyangdan => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: true }
                if *player == owner),
            HostBehavior::GujianYunvyuanshen => matches!(event,
                EventData::Dying { player, .. } if *player == owner),
            HostBehavior::GwentBirinState => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Play, begin: false }
                if *player == owner),
            HostBehavior::GwentCiguState => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Draw, begin: false }
                if *player == owner),
            HostBehavior::GwentKunenState => matches!(event,
                EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == crate::event::DamageNature::Normal),
            HostBehavior::GwentQinpenState => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Discard, begin: false }
                if *player == owner),
            HostBehavior::HuanleMonkeyCard => matches!(event,
                EventData::CardUsed(u)
                if u.player != owner
                    && !u.cancelled
                    && ctx.catalog.card_kind(u.card) == crate::catalog::CardKind::Tao
                    && ctx.state.player(owner).equips.iter().any(|c| {
                        ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Monkey
                    })),
            HostBehavior::HuanleXuelunyangCard => {
                let armed_and_targets = matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Judge, begin: true }
                if *player == owner
                    && ctx.state.player(owner).equips.iter().any(|c| {
                        ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Xuelunyang
                    })) && ctx.state.players.iter().enumerate().any(|(i, p)| {
                    i as PlayerId != owner
                        && p.alive
                        && ctx
                            .state
                            .skills
                            .range((i as PlayerId, 0)..(i as PlayerId, u16::MAX))
                            .next()
                            .is_some()
                });
                armed_and_targets
            },
            HostBehavior::XianjianChangnian => matches!(event,
                EventData::Death { player, .. } if *player == owner),
            // 长念·思：自己阶段末（runtime 授技实例持有者）,
            HostBehavior::XianjianChangnian2 => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player == owner),
            HostBehavior::XianjianChengxin => matches!(event,
                EventData::Dying { player, .. } if *player != owner)
                && ctx.state.turn.round >= ctx.state.player(owner).chengxin_ready_round,
            HostBehavior::XianjianCtianfu2 => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && ctx.state.player(owner).ctianfu_source.is_some()),
            // 天符·清挂：挂附者死亡,
            HostBehavior::XianjianDuci => matches!(event,
                EventData::CardsDiscarded { player, cards }
                if *player == owner
                    && cards.iter().any(|c| {
                        yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(2)
                    }))
                && ctx.state.alive_players().into_iter().any(|p| {
                    p != owner
                        && ctx.state.computed_distance(ctx.catalog, owner, p) <= 1
                }),
            HostBehavior::XianjianFenglue => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Play, begin: true }
                if *player == owner),
            HostBehavior::XianjianFenshi => matches!(event,
                EventData::DyingSaved { player, .. } if *player == owner)
                && !ctx.state.player(owner).jiangcheng_longhuo,
            // 龙火：自己阶段结束且已觉醒（持有龙火）,
            HostBehavior::XianjianHusha => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player == owner)
                && ctx.state.player(owner).husha_mark > 0
                && ctx.state.alive_players().into_iter().any(|p| p != owner),
            HostBehavior::XianjianHushaCount => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.turn.current == owner
                    && ctx.state.player(owner).husha_mark < 3),
            // 虎煞：有标记+（可虚拟杀：有其他存活目标）或（可虚拟南蛮：
            // 标记≥2）,
            HostBehavior::XianjianJtjubao => matches!(event,
                EventData::CardsDiscarded { player, cards }
                if *player != owner
                    && ctx.state.turn.current != owner
                    && !ctx.state.turn.jtjubao_used
                    && cards.iter().any(|c| {
                        yangpijuan_class(ctx.catalog.card_kind(*c)) != Some(0)
                    })),
            HostBehavior::XianjianLiefeng => matches!(event,
                EventData::CardUsed(u)
                if u.player == owner
                    && ctx.state.turn.current == owner
                    && !u.cancelled
                    && matches!(ctx.state.turn.cards_used_play_phase, 3 | 4)),
            HostBehavior::XianjianLiguang => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player == owner)
                && !ctx.state.player(owner).hand.is_empty()
                && ctx.state.alive_players().into_iter().any(|src| {
                    let eq_ok = ctx.state.player(src).equips.iter().any(|c| {
                        let slot = ctx.catalog.card_kind(*c).equip_slot();
                        ctx.state.alive_players().into_iter().any(|dst| {
                            dst != src
                                && ctx.state.player(dst).equips.iter().all(|o| {
                                    ctx.catalog.card_kind(*o).equip_slot() != slot
                                })
                        })
                    });
                    let jg_ok = ctx.state.player(src).judge.iter().any(|c| {
                        ctx.state.alive_players().into_iter().any(|dst| {
                            dst != src
                                && ctx.state.player(dst).judge.iter().all(|o| {
                                    ctx.catalog.card_kind(*o)
                                        != ctx.catalog.card_kind(*c)
                                })
                        })
                    });
                    eq_ok || jg_ok
                }),
            HostBehavior::XianjianLinghuo => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player != owner
                    && ctx.state.player(*player).turn_damage > 0
                    && ctx.state.turn.round >= ctx.state.player(owner).linghuo_ready_round),
            HostBehavior::XianjianLingquan => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player == owner)
                && !ctx.state.player(owner).lingquan_awakened
                && ctx.state.turn.round >= 3
                && ctx.state.player(owner).total_used
                    > ctx.state.player(owner).hp as u32,
            // 神雾：水云计数≥3+未觉醒（juexingji 一次性）,
            HostBehavior::XianjianLonghuo => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player == owner)
                && ctx.state.player(owner).jiangcheng_longhuo,
            HostBehavior::XianjianPoyun => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && ctx.state.player(owner).xuanning_mark > 0
                    && (!ctx.state.player(d.target).hand.is_empty()
                        || !ctx.state.player(d.target).equips.is_empty())),
            HostBehavior::XianjianQianfang => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Judge, begin: true }
                if *player == owner)
                && ctx.state.player(owner).xuanning_mark > 0
                && ((ctx.state.player(owner).hand.len()
                    + ctx.state.player(owner).equips.len()) as u32)
                    + ctx.state.player(owner).xuanning_mark
                    >= 3,
            HostBehavior::XianjianRunxin => {
                let heart = match event {
                    EventData::CardUsed(u) => u.player == owner && {
                        ctx.catalog.cards[u.card as usize].suit
                            == crate::catalog::Suit::Heart
                    },
                    EventData::CardResponded { card, player } => {
                        *player == owner
                            && ctx.catalog.cards[*card as usize].suit
                                == crate::catalog::Suit::Heart
                    }
                    _ => false,
                };
                heart
                    && ctx.state.alive_players().into_iter().any(|p| {
                        (ctx.state.player(p).hp as u32) < ctx.state.player(p).max_hp
                    })
            },
            HostBehavior::XianjianShenmu => matches!(event,
                EventData::Dying { .. })
                && ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Heart
                            | crate::catalog::Suit::Diamond
                    )
                }),
            // 毒刺：自己弃牌含装备 kind+距离 1 内有其他存活角色（登记差
            // 异：仅弃置路径；被获得/替换不触发）,
            HostBehavior::XianjianShenwu => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if ctx.state.player(owner).shuiyun_count >= 3)
                && !ctx.state.player(owner).shenwu_awakened,
            HostBehavior::XianjianSheying => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && !ctx.state.turn.sheying_used
                    && d.card.is_some()
                    && ctx.state.discard.contains(d.card.as_ref().unwrap())),
            // 花好：任意其他存活角色（多目标 [1,∞]）+未发动,
            HostBehavior::XianjianShiying => matches!(event,
                EventData::Death { player, .. } if *player != owner)
                && !ctx.state.player(owner).shiying_used,
            HostBehavior::XianjianShuiyun5 => matches!(event,
                EventData::Dying { player, .. } if *player != owner)
                && !ctx.state.player(owner).off_court.is_empty(),
            // 忘忧：他人阶段末+自己有牌+本回合存在受伤标记者,
            HostBehavior::XianjianWangyou => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player != owner),
            HostBehavior::XianjianWangyou2 => matches!(event,
                EventData::Damage(d)
                if !d.prevented && d.amount > 0 && ctx.state.player(d.target).alive),
            HostBehavior::XianjianXjzhimeng => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player == owner)
                && !ctx.state.deck.is_empty()
                && ctx.state.alive_players().into_iter().any(|p| p != owner),
            // 织梦·梦：暗牌持有者被使用同类型（三分类）牌（targets 含
            // 持有者+暗牌保留=上游 storage 不清除同口径）,
            HostBehavior::XianjianXshuangren => matches!(event,
                EventData::CardsDiscarded { player, cards }
                if *player == owner
                    && cards.iter().any(|c| {
                        ctx.catalog.card_kind(*c).equip_slot()
                            == Some(crate::catalog::EquipSlot::Weapon)
                    })),
            // 双刃·殁：自己死亡帧,
            HostBehavior::XianjianXshuangren2 => matches!(event,
                EventData::Death { player, .. } if *player == owner),
            HostBehavior::XianjianXunying => {
                if let EventData::CardUsed(u) = event {
                    u.player == owner
                        && ctx.catalog.card_kind(u.card)
                            == crate::catalog::CardKind::Sha
                        && !u.targets.is_empty()
                        && u.targets.iter().all(|t| {
                            ctx.state.player(*t).alive
                                && ctx.state.computed_distance(ctx.catalog, owner, *t)
                                    <= ctx.state.attack_range(ctx.catalog, owner)
                        })
                        && ctx.state.player(owner).hand.iter().any(|c| {
                            ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Sha
                        })
                } else {
                    false
                }
            },
            HostBehavior::XianjianZhangmu => matches!(event,
                EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.card.is_some()
                    && ctx.catalog.card_kind(d.card.as_ref().unwrap().clone())
                        == crate::catalog::CardKind::Sha
                    && !ctx.state.turn.zhangmu_used
                    && ctx.state.player(owner).hand.iter().any(|c| {
                        ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Shan
                    })),
            // 双刃·殁：自己死亡帧,
            HostBehavior::GwentFengyinState => matches!(event,
                EventData::Phase { player, begin: false, .. } if *player == owner),
            // 机关凤封印：拥有者任一阶段开始即解除（上游
            // {player:'phaseBegin'} 同口径）
            HostBehavior::SwdFengyinState => matches!(event,
                EventData::Phase { player, begin: true, .. } if *player == owner),
            HostBehavior::SwdJiguanyuanGain => matches!(event,
                EventData::Phase { player, begin: false, .. }
                    if *player == owner && ctx.state.player(owner).alive
                        && !ctx.state.player(owner).jiguanyuan_cards.is_empty()),
            HostBehavior::SwdJiguanfengCard => false,
            HostBehavior::SwdJiguanyuanCard => false,
            HostBehavior::SwdJiguanyaoshuCard => false,
            HostBehavior::SwdJiguanyaoState => false,
            HostBehavior::SwdJiguanshuCard => false,
            HostBehavior::SwdTianfuzhenCard => false,
            HostBehavior::SwdQixingzhenCard => false,
            HostBehavior::SwdNiaoxiangzhenCard => false,
            HostBehavior::SwdChangshezhenCard => false,
            HostBehavior::SwdPozhenjueCard => false,
            // 机关妖术补偿：非自己回合、弃牌含装备 Kind 牌（以弃装备牌近
            // 似"失去装备区牌"，登记差异：替换/获得离去路径不触发）
            HostBehavior::SwdJiguanyaoGain => matches!(event,
                EventData::CardsDiscarded { player, cards }
                    if *player != ctx.state.turn.current
                        && ctx.state.player(*player).alive
                        && cards.iter().any(|c| {
                            matches!(
                                ctx.catalog.card_kind(*c).equip_slot(),
                                Some(_)
                            )
                        })),
            HostBehavior::SwdXuanfengzhirenCard => false,
            HostBehavior::SwdZhongxinghujiaCard => false,
            HostBehavior::SwdXingtigaizaoCard => false,
            HostBehavior::SwdShijianhuisuCard => false,
            HostBehavior::SwdJinjilengdongCard => false,
            HostBehavior::SwdShengxiuhaojiaoCard => false,
            HostBehavior::SwdYinmilichangCard => false,
            HostBehavior::HuanleMianjuCard => matches!(event,
                EventData::TurnOverBefore { player, .. }
                if ctx.state.player(*player).equips.iter().any(|c| {
                    ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Mianju
                })),
            HostBehavior::ZhangbaSha => false,
            HostBehavior::SwdGouhunluoSub => matches!(event,
                EventData::Phase { player, begin: true, .. }
                if ctx.state.player(owner).gouhunluo_source == Some(*player)),
            HostBehavior::XianjianShuiyun => {
                let base = matches!(event,
                    EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                    if *player == owner);
                let dedup = ctx.state.player(owner).hand.iter().any(|c| {
                    let k = yangpijuan_class(ctx.catalog.card_kind(*c));
                    !ctx.state.player(owner).off_court.iter().any(|o| {
                        yangpijuan_class(ctx.catalog.card_kind(*o)) == k
                    })
                });
                base && ctx.state.player(owner).off_court.len() < 3 && dedup
            },
            HostBehavior::XianjianZhaoyao => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Draw, begin: true }
                if *player != owner
                    && !ctx.state.player(owner).hand.is_empty()
                    && !ctx.state.player(*player).hand.is_empty()),
            HostBehavior::WudianShuaiyan => matches!(event,
                EventData::Recover { player, .. }
                if *player != owner
                    && ctx.state.turn.current != owner
                    && ctx.state.player(*player).alive),
            HostBehavior::WudianGeju => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Judge, begin: true }
                if *player == owner),
            HostBehavior::WudianFeidan => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)
                    && ctx.state.player(d.target).alive
                    && ctx.state.alive_players().into_iter().any(|p| {
                        p != d.target
                            && ctx.state.computed_distance(ctx.catalog, d.target, p) <= 1
                    })),
            HostBehavior::WudianGuangshuDiamond => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0),
            HostBehavior::WudianGuangshuHeart => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0),
            HostBehavior::WudianJidong => matches!(event,
                EventData::Phase { begin: false, .. }
                if ctx.state.player(owner).hp == 1
                    && !ctx.state.player(owner).face_down),
            HostBehavior::WudianMengji => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && ctx.state.player(owner).hujia == 0
                    && !d.prevented
                    && !d.from_chain
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)),
            HostBehavior::WudianOwliyong => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Draw, begin: true }
                if *player == owner && ctx.state.player(owner).pingzhang_count > 0),
            HostBehavior::WudianPingzhang => matches!(event,
                EventData::Damage(d)
                if !d.prevented && d.amount > 0
                    && ctx.state.player(owner).alive
                    && (if d.target == owner {
                        !ctx.state.player(owner).pingzhang_self_used
                            && ctx.state.player(owner).hand.iter().chain(ctx.state.player(owner).equips.iter())
                                .any(|c| matches!(ctx.catalog.cards[*c as usize].suit, crate::catalog::Suit::Heart))
                    } else {
                        !ctx.state.player(owner).pingzhang_other_used
                            && ctx.state.player(owner).hand.iter().chain(ctx.state.player(owner).equips.iter())
                                .any(|c| matches!(ctx.catalog.cards[*c as usize].suit, crate::catalog::Suit::Spade))
                    })),
            // 应用：自己摸牌阶段开始+上回合有屏障次数,
            HostBehavior::WudianQiangguGuard => matches!(event,
                EventData::CardUsed(u)
                if !u.cancelled
                    && {
                        let k = u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card));
                        k == CardKind::Sha
                    }
                    && u.targets.contains(&owner)
                    && ctx.state.player(owner).qianggu_active),
            HostBehavior::WudianShenqiang => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)
                    && ctx.state.turn.current == owner),
            HostBehavior::WudianShihuo => matches!(event,
                EventData::Damage(d)
                if d.nature == crate::event::DamageNature::Fire
                    && !d.prevented
                    && d.amount > 0),
            // 猛击：你无护甲时的杀伤害（非传导；上游 filter storage.
            // zhongdun && !hujia && sha && notLink 同口径——zhongdun 置
            // 位以持技等价）,
            HostBehavior::WudianXiyang => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player == owner
                    && (ctx.state.player(owner).hp as u32) < ctx.state.player(owner).max_hp
                    && !ctx.state.player(owner).face_down),
            HostBehavior::WudianYuedong => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Play, begin: false }
                if *player == owner),
            HostBehavior::WudianZhiyuan => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0),
            HostBehavior::WudianZhucheng => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: true }
                if *player == owner),
            HostBehavior::WudianZhuchengGuard => matches!(event,
                EventData::CardUsed(u)
                if !u.cancelled
                    && {
                        let k = u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card));
                        k == CardKind::Sha
                    }
                    && u.targets.contains(&owner)
                    && !ctx.state.player(owner).zhucheng.is_empty()),
            HostBehavior::XianjianAnwugu => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && d.target != owner
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.player(d.target).alive
                    && ctx.state.player(d.target).anwugu_count == 0),
            HostBehavior::XianjianBinxin => matches!(event,
                EventData::Phase { player, begin: false, .. }
                if ctx.state.player(*player).alive
                    && ctx.state.player(*player).hp == 1),
            // 回梦：你回复体力（灵雪同款 filter）,
            HostBehavior::XianjianFeixu => matches!(event,
                EventData::CardUsed(u)
                    if ctx.catalog.card_kind(u.card) == CardKind::Shan)
                || matches!(event,
                EventData::CardResponded { card, .. }
                    if ctx.catalog.card_kind(*card) == CardKind::Shan),
            // 风：自己摸牌（有量）,
            HostBehavior::XianjianFeng => matches!(event,
                EventData::CardsDrawn { player, .. }
                if *player == owner),
            // 雅：自己受伤（未防止有量）,
            HostBehavior::XianjianHuimeng => matches!(event,
                EventData::Recover { player, amount, .. }
                if *player == owner && *amount > 0),
            // 蛇蜕：属性伤害对你（有属性=非普通；传导事件同免疫=上游
            // hasNature 同口径）,
            HostBehavior::XianjianLingxue => matches!(event,
                EventData::Recover { player, amount, .. }
                if *player == owner && *amount > 0),
            // 冰心：任意体力为 1 的角色阶段结束（含自己）,
            HostBehavior::XianjianSong => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0),
            // 御风：失去手牌（弃置路径含手牌）且手牌<2、本回合<2 次,
            HostBehavior::XianjianTianshe => matches!(event,
                EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.nature != crate::event::DamageNature::Normal),
            // 蛇蜕·息：你造成属性伤害（未防止有量）且已受伤,
            HostBehavior::XianjianTianshe2 => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.nature != crate::event::DamageNature::Normal
                    && (ctx.state.player(owner).hp as u32) < ctx.state.player(owner).max_hp),
            // 返灵：他人失去体力（有量）且自己已受伤、本回合未发动,
            HostBehavior::XianjianXjyufeng => matches!(event,
                EventData::CardsDiscarded { player, cards }
                if *player == owner
                    && ctx.state.turn.xjyufeng_used < 2
                    && !cards.is_empty()
                    && ctx.state.player(owner).hand.len() < 2),
            // 玄炎·燃：你的火属性伤害（非传导）,
            HostBehavior::XianjianXuanyan2 => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == crate::event::DamageNature::Fire
                    && !d.from_chain),
            // 玄炎·烬：你造成火属性伤害（未防止有量）,
            HostBehavior::XianjianXuanyan3 => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == crate::event::DamageNature::Fire),
            // 凝冰：你受到雷属性伤害（未防止有量）
            HostBehavior::XianjianYa => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0),
            // 颂：自己造成伤害（未防止有量）
            HostBehavior::WudianYinshen => matches!(event,
                EventData::CardsDiscarded { player, cards }
                if *player == owner
                    && !ctx.state.player(owner).yinshen_active
                    && cards.iter().any(|c| {
                        matches!(
                            ctx.catalog.card_kind(*c),
                            CardKind::Sha | CardKind::Shan | CardKind::Tao | CardKind::Jiu
                        )
                    })
                    && !ctx.state.player(owner).hand.iter().any(|c| {
                        matches!(
                            ctx.catalog.card_kind(*c),
                            CardKind::Sha | CardKind::Shan | CardKind::Tao | CardKind::Jiu
                        )
                    })),
            HostBehavior::WudianPaotaiFire => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: true }
                if *player == owner
                    && ctx.state.player(owner).paotai_lv > 0),
            HostBehavior::XianjianLuanjian2 => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.card.is_some()
                    && ctx.state.turn.luanjian_card == d.card.map(|c| c as u32)),
            _ => false,
        }
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(self.machine(owner, event))
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn crate::catalog::SkillMachine>> {
        match self.behavior {
            // 分舵：存在手牌多于自己的存活角色才可发动
            HostBehavior::SwdJiefen => {
                let my = ctx.state.player(owner).hand.len();
                let has = ctx.state.alive_players().any(|p| {
                    p != owner && ctx.state.player(p).hand.len() > my
                });
                if !has {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::JiefenTarget,
                }))
            }
            // 五行盘：出牌阶段限一次（阶段门控在命令边界），需手牌非空
            HostBehavior::ChangeCardNature => {
                if ctx.state.player(owner).hand.is_empty() {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            }
            HostBehavior::GujianFoodLuyugeng => {
                let has_basic = ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.card_kind(*c),
                        CardKind::Sha | CardKind::Shan | CardKind::Tao | CardKind::Jiu
                    )
                });
                if !has_basic {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            HostBehavior::GujianYuhengEquipSkill => {
                let has = ctx.state.alive_players().any(|p| {
                    p != owner
                        && ctx.state.player(p).hand.iter().any(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Spade
                            )
                        })
                });
                if !has {
                    return None;
                }
                // 潜偷同款校验已在 SwdShentouEquipSkill 臂单独实现
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            HostBehavior::SwdLangeguaiyiEquipSkill => {
                // 蓝格怪衣（swd 第十五批 v99）：判定技无前置条件（上游
                // 无 filter；onLose 清 getStat 为上游计数簿记，原生
                // once_per_turn 等价）
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            HostBehavior::SwdYufuluEquipSkill => {
                // 御夫录（swd 第十二批 v96）：需有武器牌（手牌/装备区，
                // 上游 filter he+equip1 同口径）且存在其他存活角色
                let is_weapon = |c: CardId| {
                    ctx.catalog.card_kind(c).equip_slot()
                        == Some(crate::catalog::EquipSlot::Weapon)
                };
                let has_weapon = ctx.state.player(owner)
                    .hand
                    .iter()
                    .chain(ctx.state.player(owner).equips.iter())
                    .copied()
                    .any(is_weapon);
                let others = ctx.state.alive_players().any(|p| p != owner);
                if !has_weapon || !others {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            HostBehavior::SwdShentouEquipSkill => {
                // 潜偷（swd 第三批 v92）：需有可弃手牌，且存在手牌更多的
                // 其他角色（上游 filter 同口径）
                let nh = ctx.state.player(owner).hand.len();
                let has = ctx.state.alive_players().any(|p| {
                    p != owner && ctx.state.player(p).hand.len() > nh
                });
                if nh == 0 || !has {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            // 断月：有装备牌（手/装备区）且存在其他存活角色
            HostBehavior::SwdDuanyue => {
                let has = !ctx.state.player(owner).equips.is_empty()
                    || ctx.state.player(owner).hand.iter().any(|c| {
                        ctx.catalog.card_kind(*c).equip_slot().is_some()
                    });
                let others = ctx.state.alive_players().any(|p| p != owner);
                if !has || !others {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            }
            // 星点：有手牌且存在其他有牌角色
            HostBehavior::SwdXingdian => {
                let has = !ctx.state.player(owner).hand.is_empty();
                let others = ctx.state.alive_players().any(|p| {
                    p != owner
                        && (!ctx.state.player(p).hand.is_empty()
                            || !ctx.state.player(p).equips.is_empty())
                });
                if !has || !others {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            }
            // 雪煌：limited 未发动、手牌全红且非空
            HostBehavior::SwdXuehuang => {
                let p = ctx.state.player(owner);
                let all_red = !p.hand.is_empty()
                    && p.hand.iter().all(|c| {
                        matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        )
                    });
                if p.xuehuang_used || !all_red {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            // 收纳：有手牌且存在其他有手牌角色（装备炼妖壶在 ActivateSkill
            // 命令层由 equip 技能实例承载——实例随 grant_equip_skills 授予）
            HostBehavior::SwdLianyaohuShouna => {
                let has = !ctx.state.player(owner).hand.is_empty();
                let others = ctx.state.alive_players().any(|p| {
                    p != owner && !ctx.state.player(p).hand.is_empty()
                });
                if !has || !others {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            }
            // 炼化：壶仓 ≥2
            HostBehavior::SwdLianyaohuLianhua => {
                if ctx.state.player(owner).lianyaohu_store.len() < 2 {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            }
            // 昆仑镜：牌堆 ≥3 且有手牌
            HostBehavior::SwdKunlunjingcCard => {
                if ctx.state.deck.len() < 3 || ctx.state.player(owner).hand.is_empty()
                {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            // 四非珍面：存在其他有手牌角色
            HostBehavior::SwdSifeizhenmianCard => {
                let others = ctx.state.alive_players().any(|p| {
                    p != owner && !ctx.state.player(p).hand.is_empty()
                });
                if !others {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::TargetChoice,
                }))
            }
            // 逆攻发动：标记 ≥1；lv≥2 打伤需选目标，lv==1 仅摸 1（直发）
            HostBehavior::SwdNigongStrike => {
                let lv = ctx.state.player(owner).nigong_lv;
                if lv == 0 {
                    return None;
                }
                if lv == 1 {
                    Some(Box::new(HostMachine {
                        def_key: self.name.clone(),
                        behavior: self.behavior,
                        owner,
                        target: None,
                        card: None,
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: HostStage::Act,
                    }))
                } else {
                    Some(Box::new(HostMachine {
                        def_key: self.name.clone(),
                        behavior: self.behavior,
                        owner,
                        target: None,
                        card: None,
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: HostStage::TargetChoice,
                    }))
                }
            }
            // 神农鼎：受伤且手牌 ≥2
            HostBehavior::SwdShennongdingCard => {
                let p = ctx.state.player(owner);
                if p.hp >= p.max_hp as i32 || p.hand.len() < 2 {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            }
            // 悲歌：确认门由 optional 承担（activate 恒供机）
            HostBehavior::SwdTiebi => Some(Box::new(HostMachine {
                def_key: self.name.clone(),
                behavior: self.behavior,
                owner,
                target: None,
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            })),
            HostBehavior::SwdDatong => Some(Box::new(HostMachine {
                def_key: self.name.clone(),
                behavior: self.behavior,
                owner,
                target: None,
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            })),
            HostBehavior::YunchouBeige => {
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            // 屯田·DIY：有装备牌（手/场）才可发动
            HostBehavior::WudianDiyjuntun => {
                let has = !ctx.state.player(owner).equips.is_empty()
                    || ctx.state.player(owner).hand.iter().any(|c| {
                        ctx.catalog.card_kind(*c).equip_slot().is_some()
                    });
                if !has {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            }
            // 隙焰：有红手牌才可发动
            HostBehavior::YunchouJieyan => {
                let has = ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    )
                });
                if !has {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            // 野升：有黑手牌才可发动（自选多张）
            HostBehavior::YunchouYiesheng => {
                let has = ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    )
                });
                if !has {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            }
            HostBehavior::ZhangbaSha => {
                // 丈八蛇矛：两张手牌当杀的主动技（机器见 HostMachine）。
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            HostBehavior::WudianPaotai => {
                // 炮台（U05 ow v123）：手牌有杀且等级 <3 才可发动
                if !ctx.state.player(owner).hand.iter().any(|c| {
                    ctx.catalog.card_kind(*c).is_sha_like()
                }) || ctx.state.player(owner).paotai_lv >= 3 {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            },
            HostBehavior::WudianShouhu => {
                // 守护（U07 ow v121）：手牌有杀才可发动
                if !ctx.state.player(owner).hand.iter().any(|c| {
                    ctx.catalog.card_kind(*c).is_sha_like()
                }) {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            },
            HostBehavior::WudianBaoleiSwitch => {
                // 堡垒/侦察切换（U05 ow v124）：无前置（上游 filter 仅
                // zhencha2 互斥锁——原生以机器推进天然互斥）
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            },
            HostBehavior::XianjianXtanxi => {
                if ctx.state.player(owner).hand.is_empty()
                    || !ctx.state.alive_players().any(|t| {
                        t != owner && !ctx.state.player(t).hand.is_empty()
                    })
                {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            },
            HostBehavior::XianjianHuahu => {
                if ctx.state.player(owner).huahu_used {
                    return None;
                }
                let has_other = ctx.state.alive_players().any(|t| t != owner);
                if !has_other {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::TargetChoice,
                }))
            },
            HostBehavior::XianjianLuanjian => {
                let sha_count = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .filter(|c| {
                        ctx.catalog.card_kind(**c) == crate::catalog::CardKind::Sha
                    })
                    .count();
                if sha_count < 2 {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            },
            HostBehavior::XianjianCtianfu => {
                let has_shan = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|c| {
                        ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Shan
                    });
                let Some(ctianfu2) = ctx.catalog.skill_by_name("ctianfu2") else {
                    return None;
                };
                let has_target = ctx.state.alive_players().any(|t| {
                    t != owner
                        && !ctx.state.skills.contains_key(&(t, ctianfu2))
                });
                if !has_shan || !has_target {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            },
            HostBehavior::XianjianTianjian => {
                let has_sha = ctx.state.player(owner).hand.iter().any(|c| {
                    ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Sha
                });
                if !has_sha {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            },
            HostBehavior::XianjianSajin => {
                let p = ctx.state.player(owner);
                let wounded = ctx
                    .state
                    .alive_players()
                    .any(|t| (ctx.state.player(t).hp as u32) < ctx.state.player(t).max_hp);
                if p.hand.is_empty() || !wounded {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            },
            HostBehavior::XianjianMinsha => {
                let enough = ctx.state.player(owner).hand.len()
                    + ctx.state.player(owner).equips.len()
                    >= 2;
                let has_target = ctx.state.alive_players().any(|t| {
                    t != owner && ctx.state.player(t).hp > 1
                });
                if !enough || !has_target {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            },
            HostBehavior::XianjianZhuyue => {
                let has_nonbasic = ctx.state.player(owner).hand.iter().any(|c| {
                    yangpijuan_class(ctx.catalog.card_kind(*c)) != Some(0)
                });
                let has_target = ctx.state.alive_players().any(|t| {
                    t != owner
                        && (!ctx.state.player(t).hand.is_empty()
                            || !ctx.state.player(t).equips.is_empty())
                });
                if !has_nonbasic || !has_target {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            },
            HostBehavior::XianjianGuanri => {
                if ctx.state.player(owner).guanri_used {
                    return None;
                }
                let reds = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .filter(|c| {
                        matches!(
                            ctx.catalog.cards[**c as usize].suit,
                            crate::catalog::Suit::Heart
                                | crate::catalog::Suit::Diamond
                        )
                    })
                    .count();
                let has_target = ctx.state.alive_players().any(|t| {
                    t != owner
                        && ctx.state.player(t).hp >= ctx.state.player(owner).hp
                });
                if reds < 2 || !has_target {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            },
            HostBehavior::XianjianDiewu => {
                let has_sha = ctx.state.player(owner).hand.iter().any(|c| {
                    ctx.catalog.card_kind(*c).is_sha_like()
                });
                if !has_sha {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            },
            HostBehavior::XianjianGuiyuan => {
                let p = ctx.state.player(owner);
                let has_sha = p
                    .hand
                    .iter()
                    .chain(p.equips.iter())
                    .any(|c| ctx.catalog.card_kind(*c).is_sha_like());
                if !has_sha || (p.hp as u32) >= p.max_hp {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            },
            HostBehavior::XianjianGuijin => {
                if ctx.state.turn.round < ctx.state.player(owner).guijin_ready_round {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            },
            // 玄墨（U05 xianjian 夏侯瑾轩，规则 v174）：手牌中有三类牌
            //（usable:1 走命令层 once_used）
            HostBehavior::XianjianXuanmo => {
                let has = ctx.state.player(owner).hand.iter().any(|c| {
                    yangpijuan_class(ctx.catalog.card_kind(*c)).is_some()
                });
                if !has {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            }
            HostBehavior::XianjianYuexing => {
                let has_other = ctx
                    .state
                    .alive_players()
                    .any(|t| t != owner);
                if !has_other {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            },
            HostBehavior::WudianKangyin => {
                // 抗yin（U07 批四 v114）：存在其他存活且 he 非空目标
                if !ctx.state.alive_players().any(|p| {
                    p != owner
                        && (!ctx.state.player(p).hand.is_empty()
                            || !ctx.state.player(p).equips.is_empty())
                }) {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            },
            HostBehavior::WudianJuedao => {
                // 绝道（U07 批六 v116）：未横置才可发动（上游 filter
                // isLinked()===false 同口径）
                if ctx.state.player(owner).linked {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            },
            HostBehavior::WudianMujing => {
                // 目镜（U07 ow v118）：手牌有黑色牌才可发动（上游
                // filterCard 黑色 + viewAsFilter 同口径）
                if !ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    )
                }) {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            },
            HostBehavior::WudianGuangshu => {
                // 光书（U07 ow v119）：he 有牌才可发动
                if ctx.state.player(owner).hand.is_empty()
                    && ctx.state.player(owner).equips.is_empty()
                {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            },
            HostBehavior::WudianZiyu => {
                // 自愈（U07 ow v120）：触发技经 OptionalGate 确认
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            },
            HostBehavior::WudianKuoyin => {
                let t = &ctx.state.turn;
                if t.yuedong_eff > 0 && t.yuedong_num > 0 {
                    return None;
                }
                let p = ctx.state.player(owner);
                if p.hand.is_empty() && p.equips.is_empty() {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            },
            HostBehavior::WudianHuhuan => {
                let p = ctx.state.player(owner);
                if p.hand.len() + p.equips.len() < 2 || ctx.state.turn.yuedong_recover {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            },
            HostBehavior::WudianBaoxue => {
                if ctx.state.player(owner).baoxue_used {
                    return None;
                }
                let p = ctx.state.player(owner);
                let black = p
                    .hand
                    .iter()
                    .chain(p.equips.iter())
                    .filter(|c| {
                        matches!(
                            ctx.catalog.cards[**c as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        )
                    })
                    .count();
                if black == 0 {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            },
            HostBehavior::WudianLiangou => {
                let p = ctx.state.player(owner);
                if p.hand.is_empty() && p.equips.is_empty() {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            },
            HostBehavior::WudianYinbo => {
                let p = ctx.state.player(owner);
                let has_spade = p
                    .hand
                    .iter()
                    .chain(p.equips.iter())
                    .any(|c| matches!(ctx.catalog.cards[*c as usize].suit, crate::catalog::Suit::Spade));
                if !has_spade {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            },
            HostBehavior::WudianQianggu => {
                let p = ctx.state.player(owner);
                if p.hand.len() + p.equips.len() < 2 {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            },
            HostBehavior::WudianFeitiao => {
                let p = ctx.state.player(owner);
                if p.hand.is_empty() && p.equips.is_empty() {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            },
            HostBehavior::MtgYixialanRecast => {
                // 依夏兰重铸（mtg 第五批 v104）：地附主动技——仅当地图
                // 在场（current_land=依夏兰，全局技同口径）且手牌有基本
                // 牌（上游 filterCard basic + canRecast；canRecast 为引
                // 擎内置归档不可核验，原生取手牌基本牌并登记）
                if ctx.state.current_land != Some(CardKind::Yixialan) {
                    return None;
                }
                let has_basic = ctx.state.player(owner).hand.iter().any(|c| {
                    yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(0)
                });
                if !has_basic {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            },
            HostBehavior::MtgYoulinDiscard => {
                // 幽林地附技（mtg 第九批 v108）：current_land=幽林 + 手
                // 牌有锦囊（yangpijuan_class==1）
                if ctx.state.current_land != Some(CardKind::Youlin) {
                    return None;
                }
                let has_trick = ctx.state.player(owner).hand.iter().any(|c| {
                    yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(1)
                });
                if !has_trick {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            },
            HostBehavior::MtgLinzhongjiantaStrike => {
                // 塔击（mtg 第七批 v106）：地附主动技——current_land=林
                // 中尖塔 + 装备区有武器（上游 viewAsFilter getEquip(1)
                // 同口径）+ 手牌有基本牌
                if ctx.state.current_land != Some(CardKind::Linzhongjianta) {
                    return None;
                }
                let has_weapon = ctx.state.player(owner).equips.iter().any(|c| {
                    ctx.catalog.card_kind(*c).equip_slot()
                        == Some(crate::catalog::EquipSlot::Weapon)
                });
                let has_basic = ctx.state.player(owner).hand.iter().any(|c| {
                    yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(0)
                });
                if !has_weapon || !has_basic {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            },
            _ => None,
        }
    }

    fn as_host_behavior(&self) -> Option<HostBehavior> {
        Some(self.behavior)
    }

    fn blocks_delayed_trick_target(&self) -> bool {
        self.blocks_delayed_trick()
    }
}
impl HostMachine {

    /// 手里剑目标询问规格（机器与恢复派生共用同一形状来源）。
    fn target_choice_spec(ctx: &Ctx<'_>, target: PlayerId) -> ChoiceSpec {
        let mut options = vec![ChoiceOption::Pass];
        options.extend(
            ctx.core
                .player(target)
                .equips
                .iter()
                .copied()
                .map(ChoiceOption::Card),
        );
        ChoiceSpec {
            kind: ChoiceKind::RespondCard,
            prompt: HOST_TARGET_DISCARD_EQUIP,
            options,
            min: 1,
            max: 1,
            allow_cancel: false,
        }
    }
}

impl crate::catalog::SkillMachine for HostMachine {
    fn fork(&self) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::Host {
            def_key: self.def_key.clone(),
            behavior: self.behavior.tag(),
            stage: self.stage.tag(),
            owner: self.owner,
            target: self.target,
            card: self.card,
            cards: self.cards.clone(),
            targets: self.targets.clone(),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 平恤：无效果步（computed_distance 字段消费）
            HostBehavior::SwdPingxu => Flow::Done,
            // 疾影：无效果步（sha_range_free 链查询）
            HostBehavior::SwdJiyingHeld => Flow::Done,
            // 沐风：确认门后摸 1（discoverCard 确定性=牌堆末摸 1）
            HostBehavior::SwdMufengHeld => {
                ctx.draw_cards(self.owner, 1);
                Flow::Done
            }
            // 望尘：确认门后 TargetChoice 选任意角色翻面
            HostBehavior::SwdWangchenHeld => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = (0..ctx.core.players.len())
                        .filter(|&i| ctx.core.player(i as PlayerId).alive)
                        .map(|i| ChoiceOption::Player(i as u8))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "wangchen_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    self.target = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive {
                        let flipped = {
                            let pl = ctx.core.player_mut(t);
                            pl.face_down = !pl.face_down;
                            pl.face_down
                        };
                        ctx.emit(EventData::TurnOvered {
                            player: t,
                            face_down: flipped,
                        });
                    }
                    ctx.core.turn.wangchen_discard_basic = false;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 还侠：无效果步（PlayCard 转化臂+kernel 跟踪钩承载）
            HostBehavior::SwdHuanxia => Flow::Done,
            // 破月：无效果步
            HostBehavior::SwdPoyue => Flow::Done,
            // 收纳：SelectCard 弃 1 手牌 → TargetChoice 其他有手牌角色 →
            // Act 目标首张手牌移入壶仓（登记差异上游随机取）
            HostBehavior::SwdLianyaohuShouna => match self.stage {
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "shouna_cost",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(cost) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    self.cards = vec![cost];
                    let options: Vec<ChoiceOption> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            i as PlayerId != self.owner
                                && ctx.core.player(i as PlayerId).alive
                                && !ctx.core.player(i as PlayerId).hand.is_empty()
                        })
                        .map(|i| ChoiceOption::Player(i as u8))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "shouna_target",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    self.target = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let (Some(cost), Some(t)) =
                        (self.cards.first().copied(), self.target)
                    else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&cost) {
                        ctx.discard_from_hand(self.owner, cost);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![cost],
                        });
                    }
                    if ctx.core.player(t).alive {
                        if let Some(c) =
                            ctx.core.player(t).hand.first().copied()
                        {
                            ctx.take_hand_card(t, c);
                            ctx.core
                                .player_mut(self.owner)
                                .lianyaohu_store
                                .push(c);
                            ctx.log(LogEvent::CardsGiven {
                                from: t,
                                to: self.owner,
                                cards: vec![c],
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 炼化：SelectCard 壶仓选 2 → Act 弃 2+牌堆顶首张类别均不同入手
            HostBehavior::SwdLianyaohuLianhua => match self.stage {
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .lianyaohu_store
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "lianhua_pick",
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.len() < 2 {
                        return Flow::Done;
                    }
                    let mut types: Vec<u8> = Vec::new();
                    for c in &picked {
                        if let Some(pos) = ctx
                            .core
                            .player(self.owner)
                            .lianyaohu_store
                            .iter()
                            .position(|x| x == c)
                        {
                            ctx.core
                                .player_mut(self.owner)
                                .lianyaohu_store
                                .remove(pos);
                        }
                        types.push(
                            ctx.catalog.cards[*c as usize].kind as u8,
                        );
                        ctx.to_discard(*c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![*c],
                        });
                    }
                    // 牌堆序扫描首张类别均不同（上游 ui.cardPile 顺序同口径）
                    for i in 0..ctx.core.deck.len() {
                        let c = ctx.core.deck[i];
                        let k = ctx.catalog.cards[c as usize].kind as u8;
                        if !types.contains(&k) {
                            ctx.core.deck.remove(i);
                            ctx.core.player_mut(self.owner).hand.push(c);
                            ctx.log(LogEvent::CardsDrawn {
                                player: self.owner,
                                cards: vec![c],
                            });
                            break;
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 壶境：弃牌堆回收炼妖壶入装备区（宝物栏）
            HostBehavior::SwdHujing => {
                if let Some(pos) = ctx
                    .core
                    .discard
                    .iter()
                    .position(|c| ctx.catalog.card_kind(*c) == CardKind::Lianyaohu)
                {
                    let hu = ctx.core.discard.remove(pos);
                    // 同栏已有宝物则先卸下（commit_equip 同栏替换口径）
                    if let Some(epos) = ctx
                        .core
                        .player(self.owner)
                        .equips
                        .iter()
                        .position(|c| {
                            ctx.catalog.card_kind(*c).equip_slot()
                                == Some(crate::catalog::EquipSlot::Treasure)
                        })
                    {
                        let old = ctx.core.player_mut(self.owner).equips.remove(epos);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            old,
                        );
                        ctx.to_discard(old);
                    }
                    ctx.core.player_mut(self.owner).equips.push(hu);
                    crate::kernel::grant_equip_skills(
                        ctx,
                        self.owner,
                        hu,
                    );
                    ctx.log(LogEvent::CardsGiven {
                        from: self.owner,
                        to: self.owner,
                        cards: vec![hu],
                    });
                }
                Flow::Done
            }
            // 神农鼎：SelectCard 弃 2 手牌 → Act 回复 1（受伤时；门控在
            // activate 臂 hp<maxHp+手牌≥2）
            HostBehavior::SwdShennongdingCard => match self.stage {
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "shennongding_cost",
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.len() < 2 {
                        return Flow::Done;
                    }
                    for c in picked {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        }
                    }
                    let p = ctx.core.player(self.owner);
                    if p.alive && p.hp < p.max_hp as i32 {
                        ctx.recover(self.owner, 1, None);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 鬼蛇旗：目标 +1 护甲（change_hujia=受伤抵 1 伤语义）
            HostBehavior::GuisheqiCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        ctx.change_hujia(t, 1);
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            // 仙露回：全体受伤目标各 +1 护甲（targets 已过滤受伤）
            HostBehavior::XianluhuiCard => {
                for t in self.targets.clone() {
                    if ctx.core.player(t).alive {
                        ctx.change_hujia(t, 1);
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            // 空洞印：死亡取消——hp=1、摸 1、弃全部宝物（death_cancelled
            // 基建消费，DeathMachine 开头拦截跳过死亡执行）
            HostBehavior::SwdKongdongyinGuard => {
                ctx.core.player_mut(self.owner).death_cancelled = true;
                ctx.core.player_mut(self.owner).hp = 1;
                ctx.draw_cards(self.owner, 1);
                let baowu: Vec<CardId> = ctx.core.player(self.owner)
                    .equips
                    .iter()
                    .filter(|c| {
                        ctx.catalog.card_kind(**c).equip_slot()
                            == Some(crate::catalog::EquipSlot::Treasure)
                    })
                    .copied()
                    .collect();
                for c in baowu {
                    if let Some(pos) = ctx
                        .core
                        .player(self.owner)
                        .equips
                        .iter()
                        .position(|x| *x == c)
                    {
                        let old = ctx.core.player_mut(self.owner).equips.remove(pos);
                        // 弃宝物同步撤销装备技实例（上游 onLose 同口径——
                        // 空洞印自身守卫技亦随之移除=一次性）
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            old,
                        );
                        ctx.to_discard(old);
                    }
                }
                Flow::Done
            }
            // 逆攻积攒：标记 +amount（上限 4）
            HostBehavior::SwdNigongAccumulate => {
                if let Some(EventData::Damage(d)) =
                    ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data)
                {
                    let lv = ctx.core.player(self.owner).nigong_lv;
                    ctx.core.player_mut(self.owner).nigong_lv =
                        (lv + d.amount as u8).min(4);
                }
                Flow::Done
            }
            // 四非珍面：TargetChoice 选其他有手牌角色 → Act 判定（非红桃
            // 则目标用首张手牌指向座位序首个其他存活者——登记差异上游
            // 随机牌/随机目标）
            HostBehavior::SwdSifeizhenmianCard => match self.stage {
                HostStage::TargetChoice => {
                    let options: Vec<ChoiceOption> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            i as PlayerId != self.owner
                                && ctx.core.player(i as PlayerId).alive
                                && !ctx.core.player(i as PlayerId).hand.is_empty()
                        })
                        .map(|i| ChoiceOption::Player(i as u8))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "yiluan_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    // 已执行守卫：子帧 CardUsed 结算期间机器可能被再次
                    // 步进（阶段未推进），cards 非空即完成
                    if !self.cards.is_empty() {
                        return Flow::Done;
                    }
                    self.target = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let judged = ctx.draw_judge_card();
                    if matches!(
                        ctx.catalog.cards[judged as usize].suit,
                        crate::catalog::Suit::Heart
                    ) {
                        return Flow::Done;
                    }
                    let Some(c) = ctx.core.player(t).hand.first().copied() else {
                        return Flow::Done;
                    };
                    self.cards = vec![c];
                    let kind = ctx.catalog.cards[c as usize].kind;
                    let targets: Vec<PlayerId> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            let p = i as PlayerId;
                            p != t && ctx.core.player(p).alive
                        })
                        .map(|i| i as PlayerId)
                        .take(1)
                        .collect();
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    // 真实牌自动使用（v177 同口径）：机器侧不预取，卡留在
                    // 目标手牌由 CardUsed 提交段自取
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        c, t, targets, Some(kind),
                    )));
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 青龙之圭：摸 2 → 弃 1 张 he（SelectCard min1）
            HostBehavior::SwdQinglongzhiguiHeld => match self.stage {
                HostStage::Act => {
                    ctx.draw_cards(self.owner, 2);
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "qinglongzhigui_discard",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let Some(c) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                        } else if let Some(pos) = ctx
                            .core
                            .player(self.owner)
                            .equips
                            .iter()
                            .position(|x| *x == c)
                        {
                            let old = ctx.core.player_mut(self.owner).equips.remove(pos);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                old,
                            );
                            ctx.to_discard(old);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 朱雀之璋：来源火 1 伤（上游 fire；native 火属性同壳）
            HostBehavior::SwdZhuquezhizhangHeld => {
                if let Some(EventData::Damage(d)) =
                    ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data)
                {
                    if let Some(src) = d.source {
                        if ctx.core.player(src).alive {
                            ctx.emit(EventData::Damage(
                                crate::machine::damage_data_with(
                                    Some(self.owner),
                                    src,
                                    1,
                                    crate::event::DamageNature::Fire,
                                    None,
                                ),
                            ));
                        }
                    }
                }
                Flow::Done
            }
            // 白兽之琥：TargetChoice 选其他有牌角色 → Act 其首张 he 弃置
            //（确定性首张，登记差异上游随机）
            HostBehavior::SwdBaishouzhihuHeld => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            let p = i as PlayerId;
                            p != self.owner
                                && ctx.core.player(p).alive
                                && (!ctx.core.player(p).hand.is_empty()
                                    || !ctx.core.player(p).equips.is_empty())
                        })
                        .map(|i| ChoiceOption::Player(i as u8))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "baishouzhihu_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    self.target = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    // 首张 he 弃置（确定性首张，登记差异上游随机）
                    if let Some(c) = ctx
                        .core
                        .player(t)
                        .hand
                        .first()
                        .copied()
                        .or_else(|| ctx.core.player(t).equips.first().copied())
                    {
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                        } else if let Some(pos) = ctx
                            .core
                            .player(t)
                            .equips
                            .iter()
                            .position(|x| *x == c)
                        {
                            let old = ctx.core.player_mut(t).equips.remove(pos);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                t,
                                old,
                            );
                            ctx.to_discard(old);
                        }
                        ctx.log(LogEvent::CardDiscarded { player: t, cards: vec![c] });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 玄武之璜：回复等量体力（伤害结算帧读 amount，hp<max 门在 filter）
            HostBehavior::SwdXuanwuzhihuangHeld => {
                if let Some(EventData::Damage(d)) =
                    ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data)
                {
                    let p = ctx.core.player(self.owner);
                    if p.alive && p.hp < p.max_hp as i32 {
                        ctx.recover(self.owner, d.amount, None);
                    }
                }
                Flow::Done
            }
            // 黄麟之琮：+1 护甲（hujia==0 门在 filter）
            HostBehavior::SwdHuanglinzhicongHeld => {
                ctx.change_hujia(self.owner, 1);
                Flow::Done
            }
            // 神草结：确认门后弃一张神草结——受伤者是自己则伤-1、是自己
            // 造成则伤+1
            HostBehavior::SwdShencaojieGuard => {
                if let Some(EventData::Damage(dm)) =
                    ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data)
                {
                    let toward_self = dm.target == self.owner;
                    if let Some(pos) = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .position(|c| {
                            ctx.catalog.card_kind(*c) == CardKind::Shencaojie
                        })
                    {
                        let c = ctx
                            .core
                            .player_mut(self.owner)
                            .hand
                            .remove(pos);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                        let frame = ctx
                            .res
                            .frame_mut(ctx.current_frame_id())
                            .expect("frame");
                        if let EventData::Damage(d2) = &mut frame.data {
                            if toward_self {
                                d2.amount = d2.amount.saturating_sub(1);
                            } else {
                                d2.amount += 1;
                            }
                        }
                    }
                }
                Flow::Done
            }
            // 玉蝉换形: 手牌中每张玉蝉实体轮换为八卦序下一 kind 的实体
            //（牌堆/弃堆确定性发现首张；登记差异上游随机+init 原地变形）
            HostBehavior::SwdYuchanSwap => {
                const ORDER: [CardKind; 8] = [
                    CardKind::Yuchanqian,
                    CardKind::Yuchankun,
                    CardKind::Yuchanzhen,
                    CardKind::Yuchanxun,
                    CardKind::Yuchankan,
                    CardKind::Yuchanli,
                    CardKind::Yuchangen,
                    CardKind::Yuchandui,
                ];
                let mut swaps: Vec<(usize, CardId, CardId)> = Vec::new();
                for (i, c) in ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .enumerate()
                    .rev()
                {
                    let Some(pos) = ctx
                        .catalog
                        .card_kind(*c)
                        .yuchan_auto_view_as()
                        .and_then(|_| {
                            ORDER.iter().position(|k| k == &ctx.catalog.card_kind(*c))
                        })
                    else {
                        continue;
                    };
                    let want = ORDER[(pos + 1) % 8];
                    if let Some(newc) = ctx
                        .core
                        .deck
                        .iter()
                        .chain(ctx.core.discard.iter())
                        .copied()
                        .find(|x| ctx.catalog.card_kind(*x) == want)
                    {
                        swaps.push((i, *c, newc));
                    }
                }
                for (i, oldc, newc) in swaps {
                    // 旧实体入弃堆（守恒），新实体从来源区移除后入手
                    ctx.to_discard(oldc);
                    if let Some(dpos) = ctx.core.deck.iter().position(|x| *x == newc) {
                        ctx.core.deck.remove(dpos);
                    } else if let Some(dpos) =
                        ctx.core.discard.iter().position(|x| *x == newc)
                    {
                        ctx.core.discard.remove(dpos);
                    }
                    let hand = &mut ctx.core.player_mut(self.owner).hand;
                    hand[i] = newc;
                    ctx.log(LogEvent::CardsGiven {
                        from: self.owner,
                        to: self.owner,
                        cards: vec![newc],
                    });
                }
                Flow::Done
            }
            // 逆攻发动：TargetChoice 选目标 → Act 弃全部标记——奇数摸 1+
            // floor(标记/2) 伤；标记=1 时仅摸 1（activate 分流两 activate 面）
            HostBehavior::SwdNigongStrike => match self.stage {
                HostStage::TargetChoice => {
                    let options: Vec<ChoiceOption> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            i as PlayerId != self.owner
                                && ctx.core.player(i as PlayerId).alive
                        })
                        .map(|i| ChoiceOption::Player(i as u8))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "nigong_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    self.target = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let lv = ctx.core.player(self.owner).nigong_lv;
                    ctx.core.player_mut(self.owner).nigong_lv = 0;
                    if lv % 2 == 1 {
                        ctx.draw_cards(self.owner, 1);
                    }
                    if self.target.is_none() {
                        // lv==1 面：无目标仅摸 1 清零
                        return Flow::Done;
                    }
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let dmg = lv / 2;
                    if dmg > 0 && ctx.core.player(t).alive {
                        ctx.emit(EventData::Damage(
                            crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                dmg as u32,
                                crate::event::DamageNature::Normal,
                                None,
                            ),
                        ));
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 女娲石：确认门后濒死者回复 1、自己失去 1（hp>1 门在 filter）
            HostBehavior::SwdNvwashiGuard => {
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive || ctx.core.player(t).hp <= 0 {
                    ctx.recover(t, 1, None);
                }
                let p = ctx.core.player(self.owner);
                if p.alive && p.hp > 1 {
                    ctx.emit(EventData::LoseHp {
                        player: self.owner,
                        amount: 1,
                    });
                }
                Flow::Done
            }
            // 昆仑镜：Act 亮牌堆顶 3 选 1 → SelectCard 手牌替换（弃 1 手
            // 牌，登记差异上游替换牌入特殊区）→ TargetChoice 获得选中牌
            HostBehavior::SwdKunlunjingcCard => match self.stage {
                HostStage::Act => {
                    if ctx.core.deck.len() < 3 {
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx.core.deck
                        .iter()
                        .take(3)
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "kunlunjingc_pick",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(picked) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    self.cards = vec![picked];
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "kunlunjingc_replace",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(replace) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let Some(picked) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&replace) {
                        ctx.discard_from_hand(self.owner, replace);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![replace],
                        });
                    }
                    if let Some(pos) = ctx.core.deck.iter().position(|x| *x == picked) {
                        ctx.core.deck.remove(pos);
                        ctx.core.player_mut(self.owner).hand.push(picked);
                        ctx.log(LogEvent::CardsDrawn {
                            player: self.owner,
                            cards: vec![picked],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 东皇钟：SelectCard 弃 1 红牌 → TargetChoice 选目标 → Act 从
            // 牌堆发现延时锦囊实体（Le/Bing/ShanDian 且目标判定区未有）置
            // 入判定区（实体发现替代 createCard 凭空造；登记差异随机→首张）
            HostBehavior::SwdDonghuangzhongCard => match self.stage {
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Heart
                                    | crate::catalog::Suit::Diamond
                            )
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "donghuangzhong_cost",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(cost) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    self.cards = vec![cost];
                    let options: Vec<ChoiceOption> = (0..ctx.core.players.len())
                        .filter(|&i| i as PlayerId != self.owner)
                        .map(|i| ChoiceOption::Player(i as u8))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "donghuangzhong_target",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    self.target = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let (Some(cost), Some(t)) =
                        (self.cards.first().copied(), self.target)
                    else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&cost) {
                        ctx.discard_from_hand(self.owner, cost);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![cost],
                        });
                    }
                    let delay_kinds = [
                        CardKind::Le,
                        CardKind::Bing,
                        CardKind::ShanDian,
                    ];
                    let mut placed = None;
                    for i in 0..ctx.core.deck.len() {
                        let c = ctx.core.deck[i];
                        let k = ctx.catalog.cards[c as usize].kind;
                        if !delay_kinds.contains(&k) {
                            continue;
                        }
                        if ctx.core.player(t).judge.iter().any(|jc| {
                            ctx.catalog.cards[*jc as usize].kind == k
                        }) {
                            continue;
                        }
                        placed = Some(c);
                        break;
                    }
                    if let Some(dc) = placed {
                        ctx.core.deck.retain(|x| *x != dc);
                        ctx.core.player_mut(t).judge.push(dc);
                        ctx.log(LogEvent::CardsGiven {
                            from: self.owner,
                            to: t,
                            cards: vec![dc],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 分舵（习嫣 jiefen）：三段——选交牌目标（手>自己）→ 目标选
            // 手牌交出 → 自己选 1 张手牌交给手牌最少的其他存活角色（登
            // 记差异：上游 chooseCardTarget 同时选牌与目标，原生确定性
            // 目标两段询问）。
            HostBehavior::SwdJiefen => match self.stage {
                HostStage::JiefenTarget => {
                    let my = ctx.core.player(self.owner).hand.len();
                    let options: Vec<ChoiceOption> = (0
                        ..ctx.core.players.len() as u8)
                        .filter(|&i| {
                            i != self.owner
                                && ctx.core.player(i).alive
                                && ctx.core.player(i).hand.len() > my
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::JiefenTargetDiscard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "swd_jiefen_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::JiefenTargetDiscard => {
                    let Some(t) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    self.target = Some(t);
                    self.stage = HostStage::JiefenOwnerDiscard;
                    let options: Vec<ChoiceOption> = ctx.core.player(t)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "swd_jiefen_give",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::JiefenOwnerDiscard => {
                    eprintln!("DBG jiefen owner_discard entered");
                    let Some(t) = self.target else { return Flow::Done };
                    let Some(c) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        eprintln!("DBG jiefen owner_discard no card");
                        return Flow::Done;
                    };
                    eprintln!("DBG jiefen owner gives card={c}");
                    ctx.core.player_mut(t).hand.retain(|x| *x != c);
                    ctx.core.player_mut(self.owner).hand.push(c);
                    ctx.emit(EventData::CardsMoved {
                        cards: vec![c],
                        from: t,
                        to: self.owner,
                    });
                    ctx.log(LogEvent::CardsGiven {
                        from: t,
                        to: self.owner,
                        cards: vec![c],
                    });
                    // 自己交 1 张手牌给手牌最少的其他存活角色（确定性）
                    let dest = (0..ctx.core.players.len() as u8)
                        .filter(|&i| {
                            i != self.owner && ctx.core.player(i).alive
                        })
                        .min_by_key(|&i| ctx.core.player(i).hand.len());
                    if let Some(d) = dest {
                        if let Some(c2) =
                            ctx.core.player(self.owner).hand.first().copied()
                        {
                            ctx.core
                                .player_mut(self.owner)
                                .hand
                                .retain(|x| *x != c2);
                            ctx.core.player_mut(d).hand.push(c2);
                            ctx.emit(EventData::CardsMoved {
                                cards: vec![c2],
                                from: self.owner,
                                to: d,
                            });
                            ctx.log(LogEvent::CardsGiven {
                                from: self.owner,
                                to: d,
                                cards: vec![c2],
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 复仇：目录锚（取消逻辑在 ShaResolveMachine 内联）
            HostBehavior::WudianFuchou => Flow::Done,
            // 坚壁：锁定查询面（AOE 目标段查询），无独立效果步
            HostBehavior::WudianJianbi => Flow::Done,
            // 厉焰：目录锚（转化逻辑在 PlayCard 门，无独立效果步）
            HostBehavior::YunchouLuweiyan => Flow::Done,
            // 雄姿：补摸 1+⌊装备数/2⌋
            HostBehavior::YunchouXiongzi => {
                let n = 1 + ctx.core.player(self.owner).equips.len() / 2;
                ctx.draw_cards(self.owner, n);
                Flow::Done
            },
            // 雅量：伤害双方各摸 1（座位序逐个）
            HostBehavior::YunchouYaliang => {
                let mut seats = self.targets.clone();
                seats.sort_unstable();
                seats.dedup();
                for s in seats {
                    if ctx.core.player(s).alive {
                        ctx.draw_cards(s, 1);
                    }
                }
                Flow::Done
            },
            // 蝉缘·变体：来源体力上限 -1（体力钳上限）
            HostBehavior::YunchouDiychanyuan => {
                if let Some(src) = self.target {
                    ctx.lose_max_hp(src, 1);
                }
                Flow::Done
            },
            // 理断：同栏已有装备 → 其交首张手牌（登记差异：上游自选）；
            // 否则直接装备该牌（不走 Equipped 帧防双 grant）
            HostBehavior::WudianLiduan => {
                let to = match self.target {
                    Some(t) => t,
                    None => return Flow::Done,
                };
                let Some(card) = self.card else { return Flow::Done };
                let Some(slot) = ctx.catalog.card_kind(card).equip_slot() else {
                    return Flow::Done;
                };
                let same_slot = ctx.core.player(to).equips.iter().any(|e| {
                    ctx.catalog.card_kind(*e).equip_slot() == Some(slot)
                });
                if same_slot {
                    // 交首张手牌给持有者
                    if let Some(c) = ctx.core.player(to).hand.first().copied() {
                        ctx.core.player_mut(to).hand.retain(|x| *x != c);
                        ctx.core.player_mut(self.owner).hand.push(c);
                        ctx.emit(EventData::CardsMoved {
                            cards: vec![c],
                            from: to,
                            to: self.owner,
                        });
                        ctx.log(LogEvent::CardsGiven {
                            from: to,
                            to: self.owner,
                            cards: vec![c],
                        });
                    }
                } else {
                    ctx.core.player_mut(to).hand.retain(|x| *x != card);
                    let old_equip = ctx.core.player(to).equips.iter().copied().find(
                        |e| ctx.catalog.card_kind(*e).equip_slot() == Some(slot),
                    );
                    if let Some(old) = old_equip {
                        ctx.core.player_mut(to).equips.retain(|x| *x != old);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            to,
                            old,
                        );
                        ctx.to_discard(old);
                    }
                    ctx.core.player_mut(to).equips.push(card);
                    crate::kernel::grant_equip_skills(ctx, to, card);
                    ctx.log(LogEvent::Equipped {
                        player: to,
                        card,
                        replaced: old_equip,
                    });
                }
                Flow::Done
            },
            // 复仇·清算：标记来源逐个摸 1+虚拟杀强制指定（死亡移除）
            HostBehavior::WudianFuchou2 => {
                let mut marked = std::mem::take(
                    &mut ctx.core.player_mut(self.owner).fuchou_marked,
                );
                marked.retain(|src| ctx.core.player(*src).alive);
                for src in marked {
                    ctx.draw_cards(self.owner, 1);
                    ctx.use_virtual_card(
                        self.owner,
                        crate::catalog::CardKind::Sha,
                        vec![src],
                    );
                }
                Flow::Done
            },
            // 悲歌（refresh 变体）：判定→弃 1 张（可取消）→按判定花色
            // 分支+收回奖励。登记差异：草花来源弃 2 取首两张确定性；
            // addExpose（AI 面）不做。
            HostBehavior::YunchouBeige => match self.stage {
                HostStage::Act => {
                    // 受伤者=targets[1]，来源=targets[0]
                    let judged = ctx.draw_judge_card();
                    ctx.to_discard(judged);
                    self.cards = vec![judged];
                    self.stage = HostStage::SelectCard;
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "beige_discard",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let [src, target] = [self.targets[0], self.targets[1]];
                    let judged = self.cards[0];
                    let judged_suit = ctx.catalog.cards[judged as usize].suit;
                    let judged_num = ctx.catalog.cards[judged as usize].rank;
                    let picked: Option<CardId> = ctx.answer().iter().find_map(
                        |o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        },
                    );
                    let Some(discarded) = picked else {
                        // 取消：判定牌已在弃堆，无效果
                        return Flow::Done;
                    };
                    let disc_suit = ctx.catalog.cards[discarded as usize].suit;
                    let disc_num = ctx.catalog.cards[discarded as usize].rank;
                    // 弃置所选（手/装备）
                    if ctx.core.player(self.owner).hand.contains(&discarded) {
                        ctx.discard_from_hand(self.owner, discarded);
                    } else {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != discarded);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            discarded,
                        );
                        ctx.to_discard(discarded);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![discarded],
                    });
                    // 按判定花色分支
                    match judged_suit {
                        crate::catalog::Suit::Heart => {
                            let p = ctx.core.player(target);
                            if p.alive && p.hp < p.max_hp as i32 {
                                ctx.recover(target, 1, None);
                            }
                        }
                        crate::catalog::Suit::Diamond => {
                            if ctx.core.player(target).alive {
                                ctx.draw_cards(target, 2);
                            }
                        }
                        crate::catalog::Suit::Spade => {
                            if ctx.core.player(src).alive {
                                let flipped = {
                                    let p = ctx.core.player_mut(src);
                                    p.face_down = !p.face_down;
                                    p.face_down
                                };
                                ctx.emit(EventData::TurnOvered {
                                    player: src,
                                    face_down: flipped,
                                });
                            }
                        }
                        crate::catalog::Suit::Club => {
                            // 来源弃 he 前 2 张（确定性首两张，登记差异）
                            if ctx.core.player(src).alive {
                                let drop: Vec<CardId> = ctx.core.player(src)
                                    .hand
                                    .iter()
                                    .chain(ctx.core.player(src).equips.iter())
                                    .copied()
                                    .take(2)
                                    .collect();
                                for c in &drop {
                                    if ctx.core.player(src).hand.contains(c) {
                                        ctx.discard_from_hand(src, *c);
                                    } else {
                                        ctx.core.player_mut(src).equips.retain(|x| x != c);
                                        crate::kernel::revoke_equip_skills(
                                            ctx.core,
                                            ctx.catalog,
                                            src,
                                            *c,
                                        );
                                        ctx.to_discard(*c);
                                    }
                                    ctx.log(LogEvent::CardDiscarded {
                                        player: src,
                                        cards: vec![*c],
                                    });
                                }
                            }
                        }
                    }
                    // 收回奖励：花色相同→判定牌归持有者；点数相同→所弃牌归持有者
                    let mut gains: Vec<CardId> = Vec::new();
                    if disc_suit == judged_suit {
                        if let Some(pos) =
                            ctx.core.discard.iter().position(|x| *x == judged)
                        {
                            ctx.core.discard.remove(pos);
                            gains.push(judged);
                        }
                    }
                    if disc_num == judged_num && discarded != judged {
                        if let Some(pos) =
                            ctx.core.discard.iter().position(|x| *x == discarded)
                        {
                            ctx.core.discard.remove(pos);
                            gains.push(discarded);
                        }
                    }
                    if !gains.is_empty() {
                        ctx.core.player_mut(self.owner).hand.extend(gains.iter().copied());
                        ctx.log(LogEvent::CardsGiven {
                            from: self.owner,
                            to: self.owner,
                            cards: gains,
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 铁壁：弃首张黑牌 → 目标 +1 护甲（登记差异：上游玩家选弃牌）
            HostBehavior::SwdTiebi => {
                let Some(t) = self.target else { return Flow::Done };
                let black = ctx.core.player(self.owner).hand.iter().copied().find(
                    |c| {
                        matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        )
                    },
                );
                if let Some(c) = black {
                    ctx.discard_from_hand(self.owner, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    ctx.change_hujia(t, 1);
                }
                Flow::Done
            }
            // 流虹：使用杀后摸 1
            HostBehavior::SwdLiuhong => {
                ctx.draw_cards(self.owner, 1);
                Flow::Done
            }
            // 墨剑：目标摸 1 + 自己回 1（未满血门）
            HostBehavior::SwdMojian => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        ctx.draw_cards(t, 1);
                    }
                }
                let p = ctx.core.player(self.owner);
                if p.hp < p.max_hp as i32 {
                    ctx.recover(self.owner, 1, None);
                }
                Flow::Done
            }
            // 道法：来源弃首张手牌（登记差异：上游玩家自选）
            HostBehavior::SwdDaofa => {
                if let Some(src) = self.target {
                    if let Some(c) =
                        ctx.core.player(src).hand.first().copied()
                    {
                        ctx.discard_from_hand(src, c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: src,
                            cards: vec![c],
                        });
                    }
                }
                Flow::Done
            }
            // 玉鳞：Act 问选装备牌（手/装备区）→ SelectCard 弃置+免伤；
            // 取消=不免伤（照常伤害）
            HostBehavior::SwdYulin => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .filter(|c| {
                            ctx.catalog.card_kind(**c).equip_slot().is_some()
                        })
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "yulin_discard",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Option<CardId> = ctx.answer().iter().find_map(
                        |o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        },
                    );
                    let Some(discarded) = picked else {
                        // 取消：伤害照常
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&discarded) {
                        ctx.discard_from_hand(self.owner, discarded);
                    } else {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != discarded);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            discarded,
                        );
                        ctx.to_discard(discarded);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![discarded],
                    });
                    ctx.current_damage_mut().prevented = true;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 凝霜：Act 问选黑牌 → SelectCard 弃置+使用者阶梯
            // （已翻面→失 1 体力；已横置→翻面；否则→横置且自己摸 1）
            HostBehavior::SwdNingshuang => match self.stage {
                HostStage::Act => {
                    let is_black = |c: CardId| {
                        matches!(
                            ctx.catalog.cards[c as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        )
                    };
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .filter(|c| is_black(*c))
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "ningshuang_discard",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Option<CardId> = ctx.answer().iter().find_map(
                        |o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        },
                    );
                    let Some(discarded) = picked else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&discarded) {
                        ctx.discard_from_hand(self.owner, discarded);
                    } else {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != discarded);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            discarded,
                        );
                        ctx.to_discard(discarded);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![discarded],
                    });
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    if ctx.core.player(t).face_down {
                        // 已翻面：失 1 体力（不摸牌）
                        ctx.emit(EventData::LoseHp { player: t, amount: 1 });
                    } else if ctx.core.player(t).linked {
                        // 已横置：翻面（不摸牌）
                        let flipped = {
                            let p = ctx.core.player_mut(t);
                            p.face_down = !p.face_down;
                            p.face_down
                        };
                        ctx.emit(EventData::TurnOvered {
                            player: t,
                            face_down: flipped,
                        });
                    } else {
                        // 横置（仅此支自己摸 1，上游代码原样）
                        ctx.emit(EventData::Linked { player: t, linked: true });
                        ctx.draw_cards(self.owner, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 工神：Act 问选装备牌 → SelectCard 弃置+取消该使用（筑城
            // cancelled 写回同款；取消=使用照常）
            HostBehavior::SwdGongshen => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .filter(|c| {
                            ctx.catalog.card_kind(**c).equip_slot().is_some()
                        })
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "gongshen_discard",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Option<CardId> = ctx.answer().iter().find_map(
                        |o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        },
                    );
                    let Some(discarded) = picked else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&discarded) {
                        ctx.discard_from_hand(self.owner, discarded);
                    } else {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != discarded);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            discarded,
                        );
                        ctx.to_discard(discarded);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![discarded],
                    });
                    {
                        let (used, user) = {
                            let u = ctx.current_card_use_mut();
                            (u.card, u.player)
                        };
                        ctx.current_card_use_mut().cancelled = true;
                        // 取消的使用：牌从桌面归还使用者手牌（上游
                        // trigger.cancel 同口径；提交段按 cancelled 跳过）
                        ctx.core.table.retain(|x| *x != used);
                        ctx.core.player_mut(user).hand.push(used);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 还灵：Act 问选其他角色 → TargetChoice 自己与目标同时翻面
            //（取消=不发动；上游 turnOver×2）
            HostBehavior::SwdHuanling => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            i as PlayerId != self.owner
                                && ctx.core.player(i as PlayerId).alive
                        })
                        .map(|i| ChoiceOption::Player(i as u8))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "huanling_target",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let picked: Option<PlayerId> = ctx.answer().iter().find_map(
                        |o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        },
                    );
                    let Some(t) = picked else {
                        // 取消：双方都不翻
                        return Flow::Done;
                    };
                    for p in [self.owner, t] {
                        if ctx.core.player(p).alive {
                            let flipped = {
                                let pl = ctx.core.player_mut(p);
                                pl.face_down = !pl.face_down;
                                pl.face_down
                            };
                            ctx.emit(EventData::TurnOvered {
                                player: p,
                                face_down: flipped,
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 断月：SelectCard 选装备牌（手/装备区）→ TargetChoice 选其
            // 他角色 → Act 弃牌+造成 1 点伤害（牌留区到末段）
            HostBehavior::SwdDuanyue => match self.stage {
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .filter(|c| {
                            ctx.catalog.card_kind(**c).equip_slot().is_some()
                        })
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "duanyue_card",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    // 收牌应答：存 cards 后前移 Act 问目标（每臂一答铁律）
                    let picked: Option<CardId> = ctx.answer().iter().find_map(
                        |o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        },
                    );
                    let Some(c) = picked else {
                        return Flow::Done;
                    };
                    self.cards = vec![c];
                    let options: Vec<ChoiceOption> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            i as PlayerId != self.owner
                                && ctx.core.player(i as PlayerId).alive
                        })
                        .map(|i| ChoiceOption::Player(i as u8))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "duanyue_target",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    // 收目标应答后结算（牌在 SelectCard 已存 cards）
                    self.target = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let (Some(discarded), Some(t)) =
                        (self.cards.first().copied(), self.target)
                    else {
                        return Flow::Done;
                    };
                    // 取消任一段=无效果（牌留区）
                    if !ctx.core.player(self.owner).hand.contains(&discarded)
                        && !ctx.core.player(self.owner).equips.contains(&discarded)
                    {
                        return Flow::Done;
                    }
                    if ctx.core.player(self.owner).hand.contains(&discarded) {
                        ctx.discard_from_hand(self.owner, discarded);
                    } else {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != discarded);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            discarded,
                        );
                        ctx.to_discard(discarded);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![discarded],
                    });
                    if ctx.core.player(t).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 屠阵：弃置受伤者全部非基本手牌（确定性整组）
            HostBehavior::SwdTuzhen => {
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let drop: Vec<CardId> = ctx.core.player(t)
                    .hand
                    .iter()
                    .filter(|c| !ctx.catalog.card_kind(**c).is_basic_kind())
                    .copied()
                    .collect();
                for c in drop {
                    ctx.discard_from_hand(t, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: t,
                        cards: vec![c],
                    });
                }
                Flow::Done
            }
            // 落梅：使用或打出梅花牌摸 1
            HostBehavior::SwdLuomei => {
                ctx.draw_cards(self.owner, 1);
                Flow::Done
            }
            // 星点：SelectCard 弃 1 张手牌 → Act 确定性两名其他存活角色
            // 各弃手/装备区首张（登记差异：上游随机 2 名敌人各随机弃 1）
            HostBehavior::SwdXingdian => match self.stage {
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "xingdian_card",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    // 收牌应答（SelectCard 直入 Act，每臂一答铁律）
                    let picked: Option<CardId> = ctx.answer().iter().find_map(
                        |o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        },
                    );
                    let Some(discarded) = picked else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&discarded) {
                        ctx.discard_from_hand(self.owner, discarded);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![discarded],
                        });
                    }
                    let mut victims: Vec<PlayerId> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            i as PlayerId != self.owner
                                && ctx.core.player(i as PlayerId).alive
                                && (!ctx.core.player(i as PlayerId).hand.is_empty()
                                    || !ctx.core.player(i as PlayerId).equips.is_empty())
                        })
                        .map(|i| i as PlayerId)
                        .collect();
                    victims.truncate(2);
                    for v in victims {
                        let Some(c) = ctx.core.player(v)
                            .hand
                            .iter()
                            .chain(ctx.core.player(v).equips.iter())
                            .copied()
                            .next()
                        else {
                            continue;
                        };
                        if ctx.core.player(v).hand.contains(&c) {
                            ctx.discard_from_hand(v, c);
                        } else {
                            ctx.core.player_mut(v).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                v,
                                c,
                            );
                            ctx.to_discard(c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: v,
                            cards: vec![c],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 协律：确认门后回 1（未满血门）+ 弃置场上全部判定牌（座位序）
            HostBehavior::SwdXielv => {
                let p = ctx.core.player(self.owner);
                if p.alive && p.hp < p.max_hp as i32 {
                    ctx.recover(self.owner, 1, None);
                }
                let judge_cards: Vec<(PlayerId, CardId)> = (0..ctx
                    .core
                    .players
                    .len())
                    .flat_map(|i| {
                        let pl = ctx.core.player(i as PlayerId);
                        pl.judge
                            .iter()
                            .map(|c| (i as PlayerId, *c))
                            .collect::<Vec<_>>()
                    })
                    .collect();
                for (pl, c) in judge_cards {
                    ctx.core.player_mut(pl).judge.retain(|x| *x != c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: pl,
                        cards: vec![c],
                    });
                }
                Flow::Done
            }
            // 朱羽：Act 问选红牌 → SelectCard 弃置+伤害 +1 且改火属性
            //（取消=伤害照常）
            HostBehavior::SwdZhuyu => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Heart
                                    | crate::catalog::Suit::Diamond
                            )
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "zhuyu_discard",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Option<CardId> = ctx.answer().iter().find_map(
                        |o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        },
                    );
                    let Some(discarded) = picked else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&discarded) {
                        ctx.discard_from_hand(self.owner, discarded);
                    } else {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != discarded);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            discarded,
                        );
                        ctx.to_discard(discarded);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![discarded],
                    });
                    let frame = ctx.res.frame_mut(ctx.current_frame_id()).expect("frame");
                    if let EventData::Damage(dm) = &mut frame.data {
                        dm.amount += 1;
                        dm.nature = crate::event::DamageNature::Fire;
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 雪煌：limited 置位 → 弃全部手牌 → 逐张视为对（至多）两名
            // 其他存活角色使用火杀（座位序确定性；登记差异上游随机 2 名
            // 敌人；目标中途死亡即终止该张）
            HostBehavior::SwdXuehuang => {
                ctx.core.player_mut(self.owner).xuehuang_used = true;
                let cards: Vec<CardId> = ctx.core.player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .collect();
                let num = cards.len();
                for c in cards {
                    ctx.discard_from_hand(self.owner, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                }
                for _ in 0..num {
                    let targets: Vec<PlayerId> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            i as PlayerId != self.owner
                                && ctx.core.player(i as PlayerId).alive
                        })
                        .map(|i| i as PlayerId)
                        .take(2)
                        .collect();
                    if targets.is_empty() {
                        break;
                    }
                    ctx.use_virtual_card(
                        self.owner,
                        crate::catalog::CardKind::FireSha,
                        targets,
                    );
                }
                Flow::Done
            }
            // 大同：全场极差 ≤1 → 摸 2
            HostBehavior::SwdDatong => {
                ctx.draw_cards(self.owner, 2);
                Flow::Done
            }
            // 承剑：来源摸 1
            HostBehavior::SwdChengjian => {
                if let Some(src) = self.target {
                    if ctx.core.player(src).alive {
                        ctx.draw_cards(src, 1);
                    }
                }
                Flow::Done
            }
            // 缚斧：DamageBefore 帧直写 num+1（乱剑先例）
            HostBehavior::SwdJufu => {
                let frame = ctx.res.frame_mut(ctx.current_frame_id()).expect("frame");
                if let EventData::Damage(dm) = &mut frame.data {
                    dm.amount += 1;
                }
                Flow::Done
            }
            // 花镜：回 1 + 摸 1（受伤者判定面：上游回 1 无判定——纯回摸）
            HostBehavior::SwdHuajing => {
                let p = ctx.core.player(self.owner);
                if p.hp < p.max_hp as i32 {
                    ctx.recover(self.owner, 1, None);
                }
                ctx.draw_cards(self.owner, 1);
                Flow::Done
            }
            // 护盾：自己无护甲 → +1
            HostBehavior::SwdHudun => {
                if ctx.core.player(self.owner).hujia == 0 {
                    ctx.change_hujia(self.owner, 1);
                }
                Flow::Done
            }
            // 破星：目标体力 > 持有者 → 伤害 +1（帧直写）
            HostBehavior::SwdPoxing => {
                eprintln!("b41 dbg poxing step target={:?} stage={:?}", self.target, self.stage);
                if let Some(t) = self.target {
                    eprintln!("b41 dbg hp t={} owner={}", ctx.core.player(t).hp, ctx.core.player(self.owner).hp);
                    if ctx.core.player(t).hp > ctx.core.player(self.owner).hp {
                        let frame =
                            ctx.res.frame_mut(ctx.current_frame_id()).expect("frame");
                        if let EventData::Damage(dm) = &mut frame.data {
                            dm.amount += 1;
                            eprintln!("b41 dbg amount now {}", dm.amount);
                        }
                    }
                }
                Flow::Done
            }
            // 良策·余获：锁定查询面（五谷机器内联查询），无独立效果步
            HostBehavior::WudianLiangceRemain => Flow::Done,
            // 归汉：目录锚（无独立效果步）
            HostBehavior::WudianGuihan => Flow::Done,
            // 禁言：查询锚（PlayCard 门），无独立效果步
            HostBehavior::WudianJinyan => Flow::Done,
            // 屯田·DIY：自选一张装备牌（手/场）弃之，牌堆随机获得一张
            // 装备牌（实体发现模式；deck+discard 无装备则仅弃置）
            HostBehavior::WudianDiyjuntun => match self.stage {
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let mut options: Vec<ChoiceOption> = p
                        .equips
                        .iter()
                        .chain(p.hand.iter())
                        .copied()
                        .filter(|c| {
                            ctx.catalog.card_kind(*c).equip_slot().is_some()
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    options.dedup();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "wudian_diyjuntun_recast",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let picked: Option<CardId> = ctx.answer().iter().find_map(
                        |o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        },
                    );
                    let Some(c) = picked else { return Flow::Done };
                    // 弃置（装备栏走 revoke）
                    if ctx.core.player(self.owner).equips.contains(&c) {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            c,
                        );
                        ctx.to_discard(c);
                    } else {
                        ctx.discard_from_hand(self.owner, c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 实体发现：牌堆随机一张装备牌入手（弃堆兜底）
                    let mut pool: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .chain(ctx.core.discard.iter())
                        .copied()
                        .filter(|x| {
                            ctx.catalog.card_kind(*x).equip_slot().is_some()
                        })
                        .collect();
                    if !pool.is_empty() {
                        let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let got = pool.swap_remove(idx);
                        if let Some(pos) =
                            ctx.core.deck.iter().position(|x| *x == got)
                        {
                            ctx.core.deck.remove(pos);
                        } else if let Some(pos) = ctx
                            .core
                            .discard
                            .iter()
                            .position(|x| *x == got)
                        {
                            ctx.core.discard.remove(pos);
                        }
                        ctx.core.player_mut(self.owner).hand.push(got);
                        ctx.log(LogEvent::CardsDrawn {
                            player: self.owner,
                            cards: vec![got],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 隙焰：弃首张红手牌（登记差异）→ 全体其他存活座位序各 1 火
            HostBehavior::YunchouJieyan => {
                let red = ctx.core.player(self.owner).hand.iter().copied().find(
                    |c| {
                        matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Heart
                                | crate::catalog::Suit::Diamond
                        )
                    },
                );
                let Some(c) = red else { return Flow::Done };
                ctx.discard_from_hand(self.owner, c);
                ctx.log(LogEvent::CardDiscarded {
                    player: self.owner,
                    cards: vec![c],
                });
                let seats: Vec<u8> = (0..ctx.core.players.len() as u8)
                    .filter(|&i| i != self.owner && ctx.core.player(i).alive)
                    .collect();
                for t in seats {
                    ctx.emit(crate::event::EventData::Damage(
                        crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            None,
                        ),
                    ));
                }
                Flow::Done
            },
            // 野升：自选任意张黑手牌 → 弃所选摸等量（舒筋散机器同构）
            HostBehavior::YunchouYiesheng => match self.stage {
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> =
                        ctx.core.player(self.owner).hand.iter().copied()
                            .filter(|c| {
                                matches!(
                                    ctx.catalog.cards[*c as usize].suit,
                                    crate::catalog::Suit::Spade
                                        | crate::catalog::Suit::Club
                                )
                            })
                            .map(ChoiceOption::Card)
                            .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let n = options.len();
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "yiesheng_discard",
                            options,
                            min: 1,
                            max: n,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if !picked.is_empty() {
                        for c in &picked {
                            ctx.core
                                .player_mut(self.owner)
                                .hand
                                .retain(|x| x != c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: picked.clone(),
                        });
                        ctx.draw_cards(self.owner, picked.len());
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 厉焰·链杀：对座位序首目标视为用杀（虚拟杀直发；登记差异：
            // 上游玩家选目标+可取消，原生自动首目标）
            HostBehavior::YunchouLuweiyan2 => {
                let mut seats = self.targets.clone();
                seats.sort_unstable();
                seats.dedup();
                if let Some(t) = seats.into_iter().find(|&t| {
                    t != self.owner && ctx.core.player(t).alive
                }) {
                    ctx.use_virtual_card(
                        self.owner,
                        crate::catalog::CardKind::Sha,
                        vec![t],
                    );
                }
                Flow::Done
            },
            // 蛊惑·变体：摸 2 然后弃自己区域 2 张（判定区→装备→手牌首张序）
            HostBehavior::YunchouDiyguhuo => {
                ctx.draw_cards(self.owner, 2);
                for _ in 0..2 {
                    let next = if let Some(c) =
                        ctx.core.player(self.owner).judge.first().copied()
                    {
                        ctx.core.player_mut(self.owner).judge.retain(|x| *x != c);
                        Some(c)
                    } else if let Some(c) =
                        ctx.core.player(self.owner).equips.first().copied()
                    {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            c,
                        );
                        Some(c)
                    } else {
                        ctx.core.player(self.owner).hand.first().copied()
                    };
                    if let Some(c) = next {
                        ctx.core.player_mut(self.owner).hand.retain(|x| *x != c);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                    }
                }
                Flow::Done
            },
            // 狡黠：摸 1
            HostBehavior::YunchouJiaoxia => {
                ctx.draw_cards(self.owner, 1);
                Flow::Done
            },
            // 红莲：来源弃其手牌与装备区全部红牌（装备走 revoke 同款）
            HostBehavior::YunchouHonglian => {
                let src = match self.target {
                    Some(t) => t,
                    None => return Flow::Done,
                };
                let reds: Vec<_> = {
                    let p = ctx.core.player(src);
                    p.hand
                        .iter()
                        .chain(p.equips.iter())
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Heart
                                    | crate::catalog::Suit::Diamond
                            )
                        })
                        .collect()
                };
                for c in reds {
                    if ctx.core.player(src).hand.contains(&c) {
                        ctx.discard_from_hand(src, c);
                    } else {
                        ctx.core.player_mut(src).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            src,
                            c,
                        );
                        ctx.to_discard(c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: src,
                        cards: vec![c],
                    });
                }
                Flow::Done
            },
            // 纵火：弃首张手牌（登记差异）→ 直写帧 nature=Fire
            HostBehavior::YunchouZonghuo => {
                let hand = ctx.core.player(self.owner).hand.clone();
                if let Some(c) = hand.first().copied() {
                    ctx.discard_from_hand(self.owner, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    let frame = ctx.res.frame_mut(ctx.current_frame_id()).expect("frame");
                    if let EventData::Damage(dm) = &mut frame.data {
                        dm.nature = crate::event::DamageNature::Fire;
                    }
                }
                Flow::Done
            },
            // 烧营：座位序首合法目标（受伤者距离 ≤1 且非受伤者）→ 弃堆顶
            // 牌亮出，红色则其受 1 点火伤（登记差异：上游玩家选目标）
            HostBehavior::YunchouShaoying => {
                let hurt = match self.target {
                    Some(t) => t,
                    None => return Flow::Done,
                };
                let target = (0..ctx.core.players.len())
                    .map(|i| i as u8)
                    .find(|&i| {
                        i != hurt
                            && i != self.owner
                            && ctx.core.player(i).alive
                            && ctx.core.seat_distance(hurt, i) <= 1
                    });
                let Some(t) = target else { return Flow::Done };
                let Some(card) = ctx.core.deck.pop() else {
                    return Flow::Done;
                };
                let red = matches!(
                    ctx.catalog.cards[card as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                );
                ctx.to_discard(card);
                ctx.log(LogEvent::CardDiscarded {
                    player: self.owner,
                    cards: vec![card],
                });
                if red {
                    ctx.emit(crate::event::EventData::Damage(
                        crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            None,
                        ),
                    ));
                }
                Flow::Done
            },
            // 戏才：获得伤害牌（table 优先、弃堆兜底；登记近似：不校验
            // 上游 position 'o' 处理区位）
            HostBehavior::YunchouXicai => {
                if let Some(c) = self.card {
                    let taken = if let Some(pos) =
                        ctx.core.table.iter().position(|x| *x == c)
                    {
                        ctx.core.table.remove(pos);
                        true
                    } else if let Some(pos) =
                        ctx.core.discard.iter().position(|x| *x == c)
                    {
                        ctx.core.discard.remove(pos);
                        true
                    } else {
                        false
                    };
                    if taken {
                        ctx.core.player_mut(self.owner).hand.push(c);
                        ctx.log(LogEvent::CardsGiven {
                            from: self.owner,
                            to: self.owner,
                            cards: vec![c],
                        });
                    }
                }
                Flow::Done
            },
            // 狂骨·变体：目标攻击距离 >1 摸 1，否则已受伤回复 1
            HostBehavior::YunchouDiykuanggu => {
                if let Some(t) = self.target {
                    if ctx.core.seat_distance(t, self.owner) > 1 {
                        ctx.draw_cards(self.owner, 1);
                    } else {
                        let p = ctx.core.player(self.owner);
                        if p.hp < p.max_hp as i32 {
                            ctx.recover(self.owner, 1, None);
                        }
                    }
                }
                Flow::Done
            },
            HostBehavior::ShaDamageForceDiscard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        if let Some(c) = ctx.core.player(t).hand.first().copied() {
                            ctx.discard_from_hand(t, c);
                            ctx.log(LogEvent::CardDiscarded { player: t, cards: vec![c] });
                        }
                    }
                }
                Flow::Done
            },
            HostBehavior::PhaseEndRecoverOrDraw => {
                let p = ctx.core.player(self.owner);
                if p.hp < p.max_hp as i32 {
                    ctx.recover(self.owner, 1, None);
                } else {
                    ctx.draw_cards(self.owner, 1);
                }
                Flow::Done
            },
            HostBehavior::TargetDiscardEquipOrDamage => match self.stage {
                HostStage::TargetChoice => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    self.stage = HostStage::Act;
                    let spec = Self::target_choice_spec(ctx, t);
                    Flow::WaitChoice {
                        observer: t,
                        spec,
                    }
                }
                HostStage::Act => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    match ctx.answer_first() {
                        ChoiceOption::Card(equip) => {
                            if ctx.core.player(t).equips.contains(&equip) {
                                ctx.core.player_mut(t).equips.retain(|x| *x != equip);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, equip);
                                ctx.to_discard(equip);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: t,
                                    cards: vec![equip],
                                });
                            }
                        }
                        _ => {
                            // 弃权：受 1 点伤害（来源 = 使用者，牌归因 = 手里剑）
                            let source = Some(self.owner);
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                source,
                                t,
                                1,
                                crate::event::DamageNature::Normal,
                                self.card,
                            )));
                        }
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::ShengkeCheck => {
                // 被克 → 弃一张手牌（确定性首张，登记差异：上游为目标
                // 自选「he」）；被生 → 摸一张；无关系 → 无事
                let card = match self.card {
                    Some(c) => c,
                    None => return Flow::Done,
                };
                let cnat = match ctx.core.card_natures.get(&card).copied() {
                    Some(n) => n,
                    None => return Flow::Done,
                };
                let onat = match ctx.core.player(self.owner).wunature {
                    Some(n) => n as usize,
                    None => return Flow::Done,
                };
                if KE[cnat as usize] == onat as u8 {
                    if let Some(c) = ctx.core.player(self.owner).hand.first().copied() {
                        ctx.discard_from_hand(self.owner, c);
                        ctx.log(LogEvent::CardDiscarded { player: self.owner, cards: vec![c] });
                    }
                } else if SHENG[cnat as usize] == onat as u8 {
                    ctx.draw_cards(self.owner, 1);
                }
                Flow::Done
            },
            HostBehavior::ChangeCardNature => match self.stage {
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUXING_SELECT_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectNature => {
                    self.card = match ctx.answer().first() {
                        Some(ChoiceOption::Card(c)) => Some(*c),
                        _ => return Flow::Done, // 弃权
                    };
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: WUXING_SELECT_NATURE,
                            options: (0..=4).map(ChoiceOption::Tag).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    if let Some(ChoiceOption::Tag(n)) = ctx.answer().first().copied() {
                        if n <= 4 && ctx.core.player(self.owner).hand.contains(&card) {
                            ctx.core.card_natures.insert(card, n);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MaxRangeWeaponDamage => match self.stage {
                HostStage::TargetChoice => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: D3_JINGGOU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                        if ctx.core.player(t).alive {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                1,
                                crate::event::DamageNature::Normal,
                                None,
                            )));
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WangmeiZhikeCard => {
                // 条件增益（上游 content）：最小体力且已受伤 → 回复 1；
                // 最小手牌数 → 摸（已回复 ? 1 : 2）张
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                let mut min_hp = i32::MAX;
                let mut min_hand = usize::MAX;
                for p in ctx.core.alive_players() {
                    let pl = ctx.core.player(p);
                    min_hp = min_hp.min(pl.hp);
                    min_hand = min_hand.min(pl.hand.len());
                }
                let tp = ctx.core.player(t);
                let damaged = tp.hp < tp.max_hp as i32;
                let is_min_hp = tp.hp == min_hp;
                let is_min_hand = tp.hand.len() == min_hand;
                let mut rec = false;
                if is_min_hp && damaged {
                    ctx.recover(t, 1, self.card);
                    rec = true;
                }
                if is_min_hand {
                    ctx.draw_cards(t, if rec { 1 } else { 2 });
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::SijiDiscardDraw => {
                let n = self
                    .cards
                    .iter()
                    .filter(|c| ctx.catalog.card_kind(**c).is_sha_like())
                    .count() as u32;
                if n > 0 {
                    ctx.draw_cards(self.owner, 2 * n as usize);
                }
                Flow::Done
            },
            HostBehavior::SanjianWeaponDamage => match self.stage {
                HostStage::SelectCard => {
                    let Some(damaged) = self.target else {
                        return Flow::Done;
                    };
                    // 合法目标：与受损者距离 ≤1 的其他存活角色
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && *p != damaged
                                && ctx.core.computed_distance(ctx.catalog, damaged, *p) <= 1
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() || ctx.core.player(self.owner).hand.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WD_SANJIAN_DISCARD,
                            options: ctx.core.player(self.owner)
                                .hand
                                .iter()
                                .copied()
                                .map(ChoiceOption::Card)
                                .collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectNature => {
                    // 记住所弃手牌（复用 cards[0]）
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        self.cards = vec![c];
                    }
                    self.stage = HostStage::Act;
                    let Some(damaged) = self.target else {
                        return Flow::Done;
                    };
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && *p != damaged
                                && ctx.core.computed_distance(ctx.catalog, damaged, *p) <= 1
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WD_SANJIAN_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                        if let Some(&c) = self.cards.first() {
                            if ctx.core.player(self.owner).hand.contains(&c) {
                                ctx.discard_from_hand(self.owner, c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: self.owner,
                                    cards: vec![c],
                                });
                            }
                            if ctx.core.player(t).alive {
                                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                    Some(self.owner),
                                    t,
                                    1,
                                    crate::event::DamageNature::Normal,
                                    None,
                                )));
                            }
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MoshouImmunity => {
                // 墨守（玩点论杀/费诗，规则 v36）：锁定——延迟锦囊（乐不思蜀、
                // 兵粮寸断）不能指定持有者为目标。上游仅 mod.targetEnabled；
                // 效果在 `blocks_delayed_trick`（同文件 2755 行）与延迟锦囊目标
                // 过滤，宿主无机器步骤。
                Flow::Done
            },
            HostBehavior::ShuigongCard => {
                // 多目标强制弃装备（登记差异：上游为目标自选，原生取首张）
                let mut any = false;
                for t in &self.targets {
                    if ctx.core.player(*t).alive {
                        if let Some(c) = ctx.core.player(*t).equips.first().copied() {
                            ctx.core.player_mut(*t).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, *t, c);
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded { player: *t, cards: vec![c] });
                            any = true;
                        }
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                let _ = any;
                Flow::Done
            },
            HostBehavior::FudichouxinCompare => match self.stage {
                HostStage::CompareTargetPick => {
                    // 目标选牌（拼点牌来自手牌）
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).hand.is_empty() || ctx.core.player(self.owner).hand.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::ComparePlayerPick;
                    let options: Vec<ChoiceOption> = ctx.core.player(t)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WD_FUDI_TARGET_PICK,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::ComparePlayerPick => {
                    // 消费目标拼点牌 → 挂起己方选牌（点数大者赢）
                    let t_card = match ctx.answer_first() {
                        ChoiceOption::Card(c) => c,
                        _ => return Flow::Done,
                    };
                    self.cards = vec![t_card];
                    self.stage = HostStage::Act;
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WD_FUDI_PLAYER_PICK,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    // 消费己方拼点牌：大者赢，赢方获得两张拼点牌
                    let Some(ChoiceOption::Card(p_card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let t_card = self.cards.first().copied();
                    let pr = ctx.catalog.cards[p_card as usize].rank;
                    if let Some(t_card) = t_card {
                        let tr = ctx.catalog.cards[t_card as usize].rank;
                        if pr > tr {
                            let t = self.target.expect("target set");
                            for c in [p_card, t_card] {
                                ctx.core.player_mut(t).hand.retain(|x| *x != c);
                                ctx.core.player_mut(self.owner).hand.push(c);
                            }
                        }
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::ChenhuodajieCard => {
                // 获得目标一张手牌（登记差异：确定性首张）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if let Some(c) = ctx.core.player(t).hand.first().copied() {
                    ctx.core.player_mut(t).hand.retain(|x| *x != c);
                    ctx.core.player_mut(self.owner).hand.push(c);
                    ctx.log(LogEvent::CardsGiven { from: t, to: self.owner, cards: vec![c] });
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::TouliangSwap => match self.stage {
                // 三段：己方选牌（SelectCard）→ 目标选牌（SelectNature）→
                // 交换 + 同色摸 1（Act）
                HostStage::SelectCard => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() || ctx.core.player(t).hand.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WD_TOULIANG_OWN,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectNature => {
                    let own = match ctx.answer_first() {
                        ChoiceOption::Card(c) => c,
                        _ => return Flow::Done,
                    };
                    self.cards = vec![own];
                    self.stage = HostStage::Act;
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let options: Vec<ChoiceOption> = ctx.core.player(t)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WD_TOULIANG_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let target_card = match ctx.answer_first() {
                        ChoiceOption::Card(c) => c,
                        _ => return Flow::Done,
                    };
                    let own_card = self.cards.first().copied();
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let own_color = own_card
                        .map(|c| ctx.catalog.cards[c as usize].suit);
                    let t_color = ctx.catalog.cards[target_card as usize].suit;
                    // 交换：己方失去 own_card 获得目标牌；目标反之
                    if let Some(oc) = own_card {
                        ctx.core.player_mut(self.owner).hand.retain(|x| *x != oc);
                        ctx.core.player_mut(t).hand.push(oc);
                    }
                    ctx.core.player_mut(t).hand.retain(|x| *x != target_card);
                    ctx.core.player_mut(self.owner).hand.push(target_card);
                    // 同色 → 摸 1
                    if own_card.is_some() && own_color == Some(t_color) {
                        ctx.draw_cards(self.owner, 1);
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::LinghunzhihuoCard => {
                // 目标受 1 点火焰伤害，使用者弃一张手牌（确定性首张）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(self.owner),
                        t,
                        1,
                        crate::event::DamageNature::Fire,
                        self.card,
                    )));
                }
                if let Some(c) = ctx.core.player(self.owner).hand.first().copied() {
                    ctx.discard_from_hand(self.owner, c);
                    ctx.log(LogEvent::CardDiscarded { player: self.owner, cards: vec![c] });
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::ZhaomingdanCard => {
                // 弃目标一张手牌（确定性首张），目标与使用者各摸 1
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if let Some(c) = ctx.core.player(t).hand.first().copied() {
                    ctx.discard_from_hand(t, c);
                    ctx.log(LogEvent::CardDiscarded { player: t, cards: vec![c] });
                }
                ctx.draw_cards(t, 1);
                ctx.draw_cards(self.owner, 1);
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::JihuocardDraw => {
                ctx.draw_cards(self.owner, 1);
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::GeanguanhuoCompare => match self.stage {
                HostStage::CompareTargetPick => {
                    // 目标一选牌（观察者=目标一）
                    let targets = &self.targets;
                    if targets.len() != 2 {
                        return Flow::Done;
                    }
                    let t1 = targets[0];
                    if ctx.core.player(t1).hand.is_empty() || ctx.core.player(targets[1]).hand.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::ComparePlayerPick;
                    let options: Vec<ChoiceOption> = ctx.core.player(t1)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    Flow::WaitChoice {
                        observer: t1,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: YC_GEAN_T1_PICK,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::ComparePlayerPick => {
                    // 目标二选牌
                    let targets = self.targets.clone();
                    if targets.len() != 2 {
                        return Flow::Done;
                    }
                    self.cards = vec![match ctx.answer_first() {
                        ChoiceOption::Card(c) => c,
                        _ => return Flow::Done,
                    }];
                    self.stage = HostStage::Act;
                    let t2 = targets[1];
                    let options: Vec<ChoiceOption> = ctx.core.player(t2)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    Flow::WaitChoice {
                        observer: t2,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: YC_GEAN_T2_PICK,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    // 结算：大者赢，赢方获得输方一张手牌（确定性首张）；
                    // 平局：目标一弃一张手牌（不计入任何人的收益）
                    let targets = self.targets.clone();
                    if targets.len() != 2 {
                        return Flow::Done;
                    }
                    let (t1, t2) = (targets[0], targets[1]);
                    let Some(ChoiceOption::Card(t2_card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(t1_card) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    let r1 = ctx.catalog.cards[t1_card as usize].rank;
                    let r2 = ctx.catalog.cards[t2_card as usize].rank;
                    if r1 > r2 {
                        if let Some(c) = ctx.core.player(t2).hand.first().copied() {
                            ctx.core.player_mut(t2).hand.retain(|x| *x != c);
                            ctx.core.player_mut(t1).hand.push(c);
                            ctx.log(LogEvent::CardsGiven { from: t2, to: t1, cards: vec![c] });
                        }
                    } else if r2 > r1 {
                        if let Some(c) = ctx.core.player(t1).hand.first().copied() {
                            ctx.core.player_mut(t1).hand.retain(|x| *x != c);
                            ctx.core.player_mut(t2).hand.push(c);
                            ctx.log(LogEvent::CardsGiven { from: t1, to: t2, cards: vec![c] });
                        }
                    } else {
                        // 平局：目标一弃一张手牌
                        if let Some(c) = ctx.core.player(t1).hand.first().copied() {
                            ctx.discard_from_hand(t1, c);
                            ctx.log(LogEvent::CardDiscarded { player: t1, cards: vec![c] });
                        }
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::CaochuanCard => match self.stage {
                // 分支选择（PickTag：0=交杀+反噬 1=展示+被弃一张）
                HostStage::SelectNature => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let has_sha = ctx.core.player(t)
                        .hand
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c) == CardKind::Sha);
                    if !has_sha {
                        // 无杀：直接走展示+被弃分支
                        self.stage = HostStage::Act;
                        self.cards = vec![1];
                        let options = vec![ChoiceOption::Tag(1)];
                        return Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::PickTag,
                                prompt: WD_CAOCHUAN_BRANCH,
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    self.stage = HostStage::Act;
                    let options = vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)];
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: WD_CAOCHUAN_BRANCH,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    match ctx.answer().first() {
                        Some(ChoiceOption::Tag(0)) => {
                            // 交出所有手牌杀给使用者；使用者受 1 点反噬杀
                            //（登记差异：上游走完整杀结算）
                            let shas: Vec<CardId> = ctx.core.player(t)
                                .hand
                                .iter()
                                .copied()
                                .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Sha)
                                .collect();
                            for c in &shas {
                                ctx.core.player_mut(t).hand.retain(|x| x != c);
                                ctx.core.player_mut(self.owner).hand.push(*c);
                            }
                            if !shas.is_empty() {
                                ctx.log(LogEvent::CardsGiven { from: t, to: self.owner, cards: shas });
                            }
                            if ctx.core.player(self.owner).alive {
                                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                    Some(t),
                                    self.owner,
                                    1,
                                    crate::event::DamageNature::Normal,
                                    None,
                                )));
                            }
                        }
                        _ => {
                            // 展示手牌 + 被弃一张（确定性首张）
                            if let Some(c) = ctx.core.player(t).hand.first().copied() {
                                ctx.discard_from_hand(t, c);
                                ctx.log(LogEvent::CardDiscarded { player: t, cards: vec![c] });
                            }
                        }
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::ShezhanqunruCompare => {
                // 与所有有手牌的响应者拼点（确定性首张）：
                // 响应者首张点数 > 使用者首张点数 → 响应者摸 1；否则使用者摸 1
                let own_rank = ctx.core.player(self.owner)
                    .hand
                    .first()
                    .map(|c| ctx.catalog.cards[*c as usize].rank);
                let Some(own_r) = own_rank else {
                    return Flow::Done;
                };
                for t in &self.targets {
                    let Some(t_card) = ctx.core.player(*t).hand.first().copied() else {
                        continue;
                    };
                    let t_r = ctx.catalog.cards[t_card as usize].rank;
                    if t_r > own_r {
                        ctx.draw_cards(*t, 1);
                    } else {
                        ctx.draw_cards(self.owner, 1);
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::GujianShujinsanCard => match self.stage {
                HostStage::SelectCard => {
                    // 目标选任意张手牌（至少 1，可取消；取消则无事发生）
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let options: Vec<ChoiceOption> = ctx.core.player(t)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() || !ctx.core.player(t).alive {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    let n = options.len();
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: GJ_SHUJINSAN_DISCARD,
                            options,
                            min: 1,
                            max: n,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if !picked.is_empty() {
                        for c in &picked {
                            ctx.core.player_mut(t).hand.retain(|x| *x != *c);
                        }
                        ctx.log(LogEvent::CardDiscarded { player: t, cards: picked.clone() });
                        ctx.draw_cards(t, picked.len());
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::GujianMianliCard => {
                // 目标摸一张，然后受 1 点伤害（登记差异：上游为失去体力）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    ctx.draw_cards(t, 1);
                    // 失去体力（逐技对照批三 v77 消除「伤害替代」登记差异）
                    ctx.emit(EventData::LoseHp { player: t, amount: 1 });
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::GujianFeibiaoCard => match self.stage {
                HostStage::SelectCard => {
                    // 有黑色手牌 → 选一张弃置（可取消，取消则受伤）；
                    // 否则直接受伤
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let finish = |ctx: &mut Ctx<'_>, card: Option<CardId>| {
                        if let Some(c) = card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                    };
                    if !ctx.core.player(t).alive {
                        finish(ctx, self.card);
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                            )
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            self.card,
                        )));
                        finish(ctx, self.card);
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: GJ_FEIBIAO_DISCARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    // 空应答 = 取消（allow_cancel）→ 失去 1 点体力
                    //（v77 原语对齐：上游 loseHp 同口径）
                    let chosen = ctx.answer().first().copied();
                    match chosen {
                        Some(ChoiceOption::Card(c)) => {
                            ctx.core.player_mut(t).hand.retain(|x| *x != c);
                            ctx.log(LogEvent::CardDiscarded { player: t, cards: vec![c] });
                        }
                        _ => {
                            ctx.emit(EventData::LoseHp { player: t, amount: 1 });
                        }
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::JinlianzhuCancel => match self.stage {
                HostStage::Act => {
                    // 伤害前置：持有者确认是否打出金莲烛
                    if !ctx.core.player(self.owner).alive {
                        return Flow::Done;
                    }
                    let has = ctx.core.player(self.owner).hand.iter()
                        .any(|c| ctx.catalog.card_kind(*c) == CardKind::Jinlianzhu);
                    if !has {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::Confirm,
                            prompt: GJ_JINLIANZHU_CONFIRM,
                            options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    // 确认：弃一张金莲烛（确定性首张）+ 防止此伤害 +
                    // 来源摸 1；拒绝：本持有者放弃（其余持有者由各自
                    // 监听技实例继续询问）
                    if matches!(ctx.answer_first(), ChoiceOption::Confirm) {
                        let candle = ctx.core.player(self.owner).hand.iter()
                            .copied()
                            .find(|c| ctx.catalog.card_kind(*c) == CardKind::Jinlianzhu);
                        if let Some(c) = candle {
                            ctx.core.player_mut(self.owner).hand.retain(|x| *x != c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        }
                        ctx.current_damage_mut().prevented = true;
                        let source = self.targets.first().copied();
                        if let Some(src) = source {
                            if ctx.core.player(src).alive {
                                ctx.draw_cards(src, 1);
                            }
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::GujianFoodTanhuadong => {
                // 出牌阶段结束：摸 1；倒计时扣减，归零移除状态
                ctx.draw_cards(self.owner, 1);
                food_tick(ctx, self.owner, &self.def_key);
                Flow::Done
            },
            HostBehavior::GujianFoodYougeng => {
                // 阶段开始（判定阶段替位准备阶段）：已受伤且体力值为
                // 全场最少或之一 → 回复 1；倒计时扣减（无论是否触发
                // 回复，登记差异：与上游同点扣减）
                let hp = ctx.core.player(self.owner).hp;
                let max_hp = ctx.core.player(self.owner).max_hp as i32;
                let min_hp = ctx
                    .core
                    .alive_players()
                    .all(|p| ctx.core.player(p).hp >= hp);
                if hp < max_hp && min_hp {
                    ctx.recover(self.owner, 1, None);
                }
                food_tick(ctx, self.owner, &self.def_key);
                Flow::Done
            },
            HostBehavior::GujianFoodGrant => {
                // 使用食物牌：给目标安装对应监听技实例（rounds=剩余
                // 回合）并弃置食物牌（食物无懈不可响应，无窗口）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let Some(c) = self.card else {
                    return Flow::Done;
                };
                let (listener, rounds) = match ctx.catalog.card_kind(c) {
                    CardKind::Tanhuadong => {
                        (GJ_FOOD_TANHUADONG_KEY, GJ_FOOD_ROUNDS_TANHUADONG)
                    }
                    CardKind::Yougeng => (GJ_FOOD_YOUGENG_KEY, GJ_FOOD_ROUNDS_YOUGENG),
                    CardKind::Qingtuan => (GJ_FOOD_QINGTUAN_KEY, GJ_FOOD_ROUNDS_DEFAULT),
                    CardKind::Yuanbaorou => (GJ_FOOD_YUANBAOROU_KEY, GJ_FOOD_ROUNDS_DEFAULT),
                    CardKind::Gudonggeng => (GJ_FOOD_GUDONGGENG_KEY, GJ_FOOD_ROUNDS_DEFAULT),
                    CardKind::Mapodoufu => (GJ_FOOD_MAPODOUFU_KEY, GJ_FOOD_ROUNDS_DEFAULT),
                    CardKind::Chunbing => (GJ_FOOD_CHUNBING_KEY, GJ_FOOD_ROUNDS_CHUNBING),
                    CardKind::Liyutang => (GJ_FOOD_LIYUTANG_KEY, GJ_FOOD_ROUNDS_LIYUTANG),
                    CardKind::Molicha => (GJ_FOOD_MOLICHA_KEY, GJ_FOOD_ROUNDS_DEFAULT),
                    CardKind::Xiajiao => (GJ_FOOD_XIAJIAO_KEY, GJ_FOOD_ROUNDS_XIAJIAO),
                    CardKind::Mizhilianou => {
                        (GJ_FOOD_MIZHLIANOU_KEY, GJ_FOOD_ROUNDS_MIZHLIANOU)
                    }
                    CardKind::Luyugeng => (GJ_FOOD_LUYUGENG_KEY, GJ_FOOD_ROUNDS_DEFAULT),
                    CardKind::Ziyangdan => (GJ_FOOD_ZIYANGDAN_KEY, GJ_FOOD_ROUNDS_DEFAULT),
                    CardKind::Yunvyuanshen => {
                        (GJ_YUNVYUANSHEN_LISTENER_KEY, GJ_FOOD_ROUNDS_DEFAULT)
                    }
                    _ => return Flow::Done,
                };
                let Some(def) = ctx.catalog.skill_by_name(listener) else {
                    return Flow::Done;
                };
                ctx.core.skills.insert(
                    (t, def),
                    crate::state::SkillInstance {
                        owner: t,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds,
                    },
                );
                // 紫阳丹：授予同时获得 3 点护甲（可叠加，登记差异：
                // 上游每阶段衰减，原生为每回合结束 -1）
                if ctx.catalog.card_kind(c) == CardKind::Ziyangdan {
                    ctx.change_hujia(t, GJ_ZIYANGDAN_HUJIA as i32);
                }
                ctx.to_discard(c);
                ctx.core.table.retain(|x| *x != c);
                Flow::Done
            },
            HostBehavior::GujianFoodQingtuan => {
                // 回合内使用首张杀（once_per_turn 门控）：摸 1
                ctx.draw_cards(self.owner, 1);
                food_tick(ctx, self.owner, &self.def_key);
                Flow::Done
            },
            HostBehavior::GujianFoodYuanbaorou => {
                // 元宝肉（古剑食物，规则 v48）：出牌阶段可额外使用一张杀
                // （每回合一次）。上游 cardUsable num+1 与 mark 回合递减；效果
                // 在杀次数校验（engine.rs 2339 行读 once_used、2501 行用后置位）
                // 与目标校验（engine.rs 3329 行），宿主无机器步骤。
                Flow::Done
            },
            HostBehavior::GujianFoodGudonggeng => {
                // 受到杀伤害：伤害 -1；一次性状态，触发即移除
                let d = ctx.current_damage_mut();
                d.amount = d.amount.saturating_sub(1);
                food_remove(ctx, self.owner, &self.def_key);
                Flow::Done
            },
            HostBehavior::GujianFoodMapodoufu => {
                // 结束阶段：随机一名有牌其他角色随机弃一张牌（手牌+装备，
                // 上游 list.randomGet() + he.randomGet()；逐技对照批四
                // v78 种子 RNG 均匀抽取；敌我阵营面按原型惯例取其他角色）
                let cands: Vec<crate::ids::PlayerId> = ctx
                    .core
                    .seat_order_from(ctx.core.turn.current)
                    .into_iter()
                    .filter(|p| {
                        *p != self.owner
                            && ctx.core.player(*p).alive
                            && (!ctx.core.player(*p).hand.is_empty()
                                || !ctx.core.player(*p).equips.is_empty())
                    })
                    .collect();
                let victim = if cands.is_empty() {
                    None
                } else {
                    Some(cands[ctx.core.rng.gen_range(cands.len() as u32) as usize])
                };
                if let Some(v) = victim {
                    let he_len = ctx.core.player(v).hand.len()
                        + ctx.core.player(v).equips.len();
                    let pick = ctx.core.rng.gen_range(he_len as u32) as usize;
                    let hand_card = ctx.core.player(v).hand.get(pick).copied();
                    match hand_card {
                        Some(c) => {
                            ctx.discard_from_hand(v, c);
                            ctx.log(LogEvent::CardDiscarded { player: v, cards: vec![c] });
                        }
                        None => {
                            // pick 落在装备区：下标减去手牌数
                            let ei = pick - ctx.core.player(v).hand.len();
                            if let Some(c) = ctx.core.player(v).equips.get(ei).copied() {
                                ctx.core.player_mut(v).equips.retain(|x| *x != c);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, v, c);
                                ctx.to_discard(c);
                                ctx.log(LogEvent::CardDiscarded { player: v, cards: vec![c] });
                            }
                        }
                    }
                }
                food_tick(ctx, self.owner, &self.def_key);
                Flow::Done
            },
            HostBehavior::GujianFoodChunbing => {
                Flow::Done
            },
            HostBehavior::GujianFoodLiyutang => {
                // 阶段开始：体力值为全场最少或之一 → 获得 1 点护甲
                let hp = ctx.core.player(self.owner).hp;
                let min_hp = ctx
                    .core
                    .alive_players()
                    .all(|p| ctx.core.player(p).hp >= hp);
                if min_hp {
                    ctx.change_hujia(self.owner, 1);
                }
                food_tick(ctx, self.owner, &self.def_key);
                Flow::Done
            },
            HostBehavior::GujianFoodMolicha => Flow::Done,
            HostBehavior::GujianFoodXiajiao => match self.stage {
                HostStage::Act => {
                    // 摸牌阶段：额外摸 1（固定摸牌后），然后弃 1
                    ctx.draw_cards(self.owner, 1);
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() || !ctx.core.player(self.owner).alive {
                        food_tick(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: GJ_XIAJIAO_DISCARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    // 弃 1（必选）后倒计时扣减
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        ctx.discard_from_hand(self.owner, c);
                        ctx.log(LogEvent::CardDiscarded { player: self.owner, cards: vec![c] });
                    }
                    food_tick(ctx, self.owner, &self.def_key);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::GujianFoodMizhilianou => Flow::Done,
            HostBehavior::GujianFoodLuyugeng => match self.stage {
                HostStage::Act => {
                    // 出牌阶段限一次：弃一张基本牌，获得牌堆顶牌
                    //（登记差异：上游为发现牌随机五选一）
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.card_kind(*c),
                                CardKind::Sha
                                    | CardKind::Shan
                                    | CardKind::Tao
                                    | CardKind::Jiu
                            )
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: GJ_LUYUGENG_COST,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        ctx.discard_from_hand(self.owner, c);
                        ctx.log(LogEvent::CardDiscarded { player: self.owner, cards: vec![c] });
                        ctx.draw_cards(self.owner, 1);
                    }
                    food_tick(ctx, self.owner, &self.def_key);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::JiguanBingpoCard => loop {
                // 逐目标双段自选展示（逐技对照批五 v79 对齐上游 chooseCard
                // ×2）：使用者自选展示一张 → 目标自选展示一张 → 同色则
                // 使用者弃展示牌、目标失去 1 点体力（规则 v77 原语）；
                // 异色无事。展示不移动牌（仅同色弃使用者展示牌）。
                match self.stage {
                    HostStage::ComparePlayerPick => {
                        let Some(t) = self.targets.first().copied() else {
                            if let Some(c) = self.card {
                                ctx.to_discard(c);
                                ctx.core.table.retain(|x| *x != c);
                            }
                            return Flow::Done;
                        };
                        if !ctx.core.player(t).alive
                            || ctx.core.player(t).hand.is_empty()
                            || ctx.core.player(self.owner).hand.is_empty()
                        {
                            // 上游 step0：任一方无手牌该目标直接结束
                            self.targets.remove(0);
                            continue;
                        }
                        let options: Vec<ChoiceOption> = ctx
                            .core
                            .player(self.owner)
                            .hand
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card)
                            .collect();
                        self.stage = HostStage::CompareTargetPick;
                        return Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: GJ_BINGPO_SHOW_OWN,
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    HostStage::CompareTargetPick => {
                        let Some(t) = self.targets.first().copied() else {
                            if let Some(c) = self.card {
                                ctx.to_discard(c);
                                ctx.core.table.retain(|x| *x != c);
                            }
                            return Flow::Done;
                        };
                        // 暂存使用者展示牌（cards 入快照，恢复可续）
                        self.cards = ctx
                            .answer()
                            .iter()
                            .filter_map(|o| match o {
                                ChoiceOption::Card(c) => Some(*c as u32),
                                _ => None,
                            })
                            .collect();
                        let options: Vec<ChoiceOption> = ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card)
                            .collect();
                        self.stage = HostStage::Act;
                        return Flow::WaitChoice {
                            observer: t,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: GJ_BINGPO_SHOW_TARGET,
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    HostStage::Act => {
                        let Some(t) = self.targets.first().copied() else {
                            if let Some(c) = self.card {
                                ctx.to_discard(c);
                                ctx.core.table.retain(|x| *x != c);
                            }
                            return Flow::Done;
                        };
                        let tc = ctx.answer().first().and_then(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        });
                        let own = self.cards.first().copied().map(|c| c as CardId);
                        self.cards.clear();
                        self.targets.remove(0);
                        self.stage = HostStage::ComparePlayerPick;
                        if let (Some(own), Some(tc)) = (own, tc) {
                            let color = |c: CardId| {
                                matches!(
                                    ctx.catalog.cards[c as usize].suit,
                                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                                )
                            };
                            if color(own) == color(tc) {
                                ctx.core
                                    .player_mut(self.owner)
                                    .hand
                                    .retain(|x| *x != own);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: self.owner,
                                    cards: vec![own],
                                });
                                ctx.emit(EventData::LoseHp { player: t, amount: 1 });
                            }
                        }
                        continue;
                    }
                    _ => return Flow::Done,
                }
            },
            HostBehavior::JiguanQiankunCard => {
                // 目标 + 距离 1 邻座：各弃首张手牌（无手牌弃首张装备）
                //（登记差异：随机牌改确定性首张）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let mut victims = vec![t];
                for p in ctx.core.alive_players() {
                    if p != t
                        && p != self.owner
                        && ctx.core.computed_distance(ctx.catalog, t, p) == 1
                    {
                        victims.push(p);
                    }
                }
                for v in victims {
                    if !ctx.core.player(v).alive {
                        continue;
                    }
                    // 上游 he.randomGet()：手牌+装备合集随机一张
                    //（逐技对照批四 v78 种子 RNG 均匀抽取；changeTarget
                    // 只把 he 非空者入列——此处跳过无牌者同口径）
                    let he_len = ctx.core.player(v).hand.len()
                        + ctx.core.player(v).equips.len();
                    if he_len == 0 {
                        continue;
                    }
                    let pick = ctx.core.rng.gen_range(he_len as u32) as usize;
                    if let Some(c) = ctx.core.player(v).hand.get(pick).copied() {
                        ctx.discard_from_hand(v, c);
                        ctx.log(LogEvent::CardDiscarded { player: v, cards: vec![c] });
                    } else {
                        let ei = pick - ctx.core.player(v).hand.len();
                        if let Some(c) = ctx.core.player(v).equips.get(ei).copied() {
                            ctx.core.player_mut(v).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                v,
                                c,
                            );
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded { player: v, cards: vec![c] });
                        }
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::JiguanLongxuCard => match self.stage {
                HostStage::SelectCard => {
                    // 使用者自选目标装备区一张获得（上游
                    // choosePlayerCard("e", target)；逐技对照批五 v79
                    // 消除「首张」登记差异；装备区明牌，向使用者询问）
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .equips
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() || !ctx.core.player(t).alive {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: GJ_LONGXU_PICK,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let pick = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    if let Some(c) = pick.filter(|c| ctx.core.player(t).equips.contains(c)) {
                        ctx.core.player_mut(t).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                        ctx.core.player_mut(self.owner).hand.push(c);
                        ctx.log(LogEvent::CardsGiven {
                            from: t,
                            to: self.owner,
                            cards: vec![c],
                        });
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::JiguanShenhuoCard => loop {
                // 目标 + 距离 1 邻座：需出闪否则受 1 点火焰伤害
                match self.stage {
                    HostStage::Act => {
                        let Some(t) = self.targets.first().copied() else {
                            if let Some(c) = self.card {
                                ctx.to_discard(c);
                                ctx.core.table.retain(|x| *x != c);
                            }
                            return Flow::Done;
                        };
                        self.targets.remove(0);
                        if !ctx.core.player(t).alive {
                            continue;
                        }
                        self.target = Some(t);
                        let shans: Vec<ChoiceOption> = ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .copied()
                            .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Shan)
                            .map(ChoiceOption::Card)
                            .collect();
                        if shans.is_empty() {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                1,
                                crate::event::DamageNature::Fire,
                                self.card,
                            )));
                            continue;
                        }
                        let mut options = vec![ChoiceOption::Pass];
                        options.extend(shans);
                        self.stage = HostStage::TargetChoice;
                        return Flow::WaitChoice {
                            observer: t,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::RespondCard,
                                prompt: "aoe_response",
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    HostStage::TargetChoice => {
                        self.stage = HostStage::Act;
                        let answered = ctx.answer().first().copied();
                        let responder = self.target;
                        if let (Some(ChoiceOption::Card(c)), Some(t)) = (answered, responder) {
                            ctx.discard_from_hand(t, c);
                            ctx.emit(EventData::CardResponded { card: c, player: t });
                        } else if let Some(t) = responder {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                1,
                                crate::event::DamageNature::Fire,
                                self.card,
                            )));
                        }
                        continue;
                    }
                    _ => return Flow::Done,
                }
            },
            HostBehavior::GujianFoodZiyangdan => {
                // 结束阶段：护甲 -1（登记差异：上游每阶段开始衰减）；
                // 倒计时扣减，归零移除状态
                ctx.change_hujia(self.owner, -1);
                food_tick(ctx, self.owner, &self.def_key);
                Flow::Done
            },
            HostBehavior::GujianYunvyuanshen => {
                // 下次濒死：回复 1 点体力（recover 清 dying 标记，
                // 濒死救援机器检测 hp>0 直接退出）并移除状态
                ctx.recover(self.owner, 1, None);
                food_remove(ctx, self.owner, &self.def_key);
                Flow::Done
            },
            HostBehavior::GujianFoodShihuifenBan => Flow::Done,
            HostBehavior::GujianShihuifenCard => match self.stage {
                HostStage::Act => {
                    // 目标有闪 → 出闪询问；无闪 → 置禁用状态
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let shans: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Shan)
                        .map(ChoiceOption::Card)
                        .collect();
                    if shans.is_empty() || !ctx.core.player(t).alive {
                        self.grant_shihuifen_ban(ctx, t);
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    let mut options = vec![ChoiceOption::Pass];
                    options.extend(shans);
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::RespondCard,
                            prompt: "respond_shan",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    match ctx.answer().first().copied() {
                        Some(ChoiceOption::Card(c)) => {
                            ctx.discard_from_hand(t, c);
                            ctx.emit(EventData::CardResponded { card: c, player: t });
                        }
                        _ => self.grant_shihuifen_ban(ctx, t),
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::GujianLiufengsanCard => {
                // 目标从牌堆获得两张实体闪（登记差异：虚拟牌生成改
                // 实体牌获得；牌堆无闪则无事）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    let mut gained = Vec::new();
                    for _ in 0..2 {
                        let pos = ctx
                            .core
                            .deck
                            .iter()
                            .rposition(|c| ctx.catalog.card_kind(*c) == CardKind::Shan);
                        match pos {
                            Some(i) => {
                                let c = ctx.core.deck.remove(i);
                                ctx.core.player_mut(t).hand.push(c);
                                gained.push(c);
                            }
                            None => break,
                        }
                    }
                    if !gained.is_empty() {
                        ctx.emit(EventData::CardsDrawn { player: t, cards: gained });
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::JiguanShatangCard => match self.stage {
                HostStage::Act => {
                    // 目标受 1 点火焰伤害——emit 后 Yield，待伤害帧
                    // 提交后再给护甲（否则新护甲吸收本次伤害）
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            self.card,
                        )));
                        self.stage = HostStage::SelectNature;
                        return Flow::Yield;
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                HostStage::SelectNature => {
                    // 伤害已提交：获得 1 点护甲并收尾
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive {
                        ctx.change_hujia(t, 1);
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::JiguanHeilongCard => {
                // 获得 1 点护甲（登记差异：防御距离 +1 临时技暂缓）
                ctx.change_hujia(self.owner, 1);
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::JiguanLiutougeCard => {
                // 自己 → 摸 1（增益简化）；其他角色 → 弃首张手牌
                //（减益简化；登记差异：上游随机效果池）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    if t == self.owner {
                        ctx.draw_cards(t, 1);
                    } else if let Some(c) = ctx.core.player(t).hand.first().copied() {
                        ctx.discard_from_hand(t, c);
                        ctx.log(LogEvent::CardDiscarded { player: t, cards: vec![c] });
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::GujianMutouEquipSkill => Flow::Done,
            HostBehavior::JiguanDinvCard => match self.stage {
                HostStage::Act => {
                    // 回复 1 点体力，然后可弃任意张手牌摸等量（可取消）
                    ctx.recover(self.owner, 1, None);
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    let n = options.len();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: GJ_DINVXUANSHUANG_RIDER,
                            options,
                            min: 0,
                            max: n,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if !picked.is_empty() {
                        for c in &picked {
                            ctx.core.player_mut(self.owner).hand.retain(|x| *x != *c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: picked.clone(),
                        });
                        ctx.draw_cards(self.owner, picked.len());
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::GujianYuhengEquipSkill => {
                // 随机一名有黑桃手牌的其他角色 → 其随机一张黑桃手牌交给你
                //（上游 list.randomGet() + randomGet()；逐技对照批四 v78
                // 种子 RNG 均匀抽取；升级机制仍登记暂缓）
                let cands: Vec<crate::ids::PlayerId> = ctx
                    .core
                    .alive_players()
                    .filter(|p| {
                        *p != self.owner
                            && ctx
                                .core
                                .player(*p)
                                .hand
                                .iter()
                                .any(|c| {
                                    ctx.catalog.cards[*c as usize].suit
                                        == crate::catalog::Suit::Spade
                                })
                    })
                    .collect();
                if !cands.is_empty() {
                    let v = cands[ctx.core.rng.gen_range(cands.len() as u32) as usize];
                    let spades: Vec<usize> = ctx
                        .core
                        .player(v)
                        .hand
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| {
                            ctx.catalog.cards[**c as usize].suit == crate::catalog::Suit::Spade
                        })
                        .map(|(i, _)| i)
                        .collect();
                    let i = spades[ctx.core.rng.gen_range(spades.len() as u32) as usize];
                    let c = ctx.core.player_mut(v).hand.remove(i);
                    ctx.core.player_mut(self.owner).hand.push(c);
                    ctx.log(LogEvent::CardsGiven {
                        from: v,
                        to: self.owner,
                        cards: vec![c],
                    });
                }
                Flow::Done
            },
            HostBehavior::HearthDunpaiCard => match self.stage {
                HostStage::Act => {
                    // +1 护甲、摸 1
                    ctx.change_hujia(self.owner, 1);
                    ctx.draw_cards(self.owner, 1);
                    let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    // 弃 1（登记差异：确定性首张）
                    let c0 = match options.first() {
                        Some(ChoiceOption::Card(c)) => *c,
                        _ => return Flow::Done,
                    };
                    ctx.core.player_mut(self.owner).hand.retain(|x| *x != c0);
                    ctx.log(LogEvent::CardDiscarded { player: self.owner, cards: vec![c0] });
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::HearthZhiliaoboCard => {
                // 判定红色 → 回复 1；否则 +1 护甲
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let judged = ctx.draw_judge_card();
                let red = matches!(
                    ctx.catalog.cards[judged as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                );
                ctx.to_discard(judged);
                if red {
                    ctx.recover(t, 1, None);
                } else {
                    ctx.change_hujia(t, 1);
                }
                Flow::Done
            },
            HostBehavior::HearthYuansuCard => loop {
                // 逐目标：可弃 1–2 张（可取消）→ 每少弃一张受 1 雷伤
                match self.stage {
                    HostStage::Act => {
                        let Some(t) = self.targets.first().copied() else {
                            if let Some(c) = self.card {
                                ctx.to_discard(c);
                                ctx.core.table.retain(|x| *x != c);
                            }
                            return Flow::Done;
                        };
                        self.targets.remove(0);
                        if !ctx.core.player(t).alive {
                            continue;
                        }
                        self.target = Some(t);
                        let hand: Vec<ChoiceOption> = ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .chain(ctx.core.player(t).equips.iter())
                            .copied()
                            .map(ChoiceOption::Card)
                            .collect();
                        if hand.is_empty() {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                2,
                                crate::event::DamageNature::Thunder,
                                self.card,
                            )));
                            continue;
                        }
                        let n = hand.len().min(2);
                        self.stage = HostStage::TargetChoice;
                        return Flow::WaitChoice {
                            observer: t,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: HH_YUANSU_DISCARD,
                                options: hand,
                                min: 1,
                                max: n,
                                allow_cancel: true,
                            },
                        };
                    }
                    HostStage::TargetChoice => {
                        self.stage = HostStage::Act;
                        let Some(t) = self.target else {
                            return Flow::Done;
                        };
                        let picked: Vec<CardId> = ctx
                            .answer()
                            .iter()
                            .filter_map(|o| match o {
                                ChoiceOption::Card(c) => Some(*c),
                                _ => None,
                            })
                            .collect();
                        for c in &picked {
                            ctx.core.player_mut(t).hand.retain(|x| *x != *c);
                            ctx.core.player_mut(t).equips.retain(|x| *x != *c);
                            ctx.to_discard(*c);
                        }
                        if !picked.is_empty() {
                            ctx.log(LogEvent::CardDiscarded { player: t, cards: picked.clone() });
                        }
                        let dmg = 2 - picked.len() as u32;
                        if dmg > 0 {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                dmg,
                                crate::event::DamageNature::Thunder,
                                self.card,
                            )));
                        }
                        continue;
                    }
                    _ => return Flow::Done,
                }
            },
            HostBehavior::HearthXingjieCard => {
                // 体力上限 +1、回复 1、弃全部手牌
                ctx.core.player_mut(self.owner).max_hp += 1;
                ctx.recover(self.owner, 1, None);
                let hand: Vec<CardId> = ctx.core.player(self.owner).hand.clone();
                for c in &hand {
                    ctx.core.player_mut(self.owner).hand.retain(|x| x != c);
                }
                if !hand.is_empty() {
                    ctx.log(LogEvent::CardDiscarded { player: self.owner, cards: hand });
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::HearthChuansongCard => {
                // 获得牌堆顶一张牌（登记差异：立即使用链暂缓）
                ctx.draw_cards(self.owner, 1);
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::HearthTansheCard => {
                // 座位序链式弃首张手牌，共「存活数取奇」次
                //（登记差异：随机敌随机牌改确定性座位链）
                let mut current = self.owner;
                let mut times = {
                    let alive = ctx.core.alive_count();
                    if alive % 2 == 0 { alive - 1 } else { alive }
                };
                while times > 0 {
                    let next = ctx
                        .core
                        .seat_order_from(current)
                        .into_iter()
                        .find(|p| *p != current && ctx.core.player(*p).alive && !ctx.core.player(*p).hand.is_empty());
                    match next {
                        Some(v) => {
                            let c = ctx.core.player(v).hand.first().copied();
                            if let Some(c) = c {
                                ctx.discard_from_hand(v, c);
                                ctx.log(LogEvent::CardDiscarded { player: v, cards: vec![c] });
                            }
                            current = v;
                            times -= 1;
                        }
                        None => break,
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::HearthShijieshuCard => {
                // 牌堆自顶搜索两张不同槽位的装备，目标依次穿戴
                //（同槽旧装备撤销技能后置入弃牌堆）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let slot_of = |c: CardId| ctx.catalog.cards[c as usize]
                    .kind
                    .equip_slot();
                let mut picked: Vec<CardId> = Vec::new();
                let mut seen_slots: Vec<_> = Vec::new();
                // 牌堆自顶（向量尾）向底扫描
                let mut i = ctx.core.deck.len();
                while i > 0 && picked.len() < 2 {
                    i -= 1;
                    let c = ctx.core.deck[i];
                    let Some(slot) = slot_of(c) else { continue };
                    if seen_slots.contains(&slot) {
                        continue;
                    }
                    seen_slots.push(slot);
                    picked.push(c);
                    ctx.core.deck.remove(i);
                }
                for c in picked {
                    let slot = slot_of(c);
                    // 同槽旧装备替换
                    let replaced = ctx.core.player(t).equips.iter().copied().find(|e| {
                        ctx.catalog.cards[*e as usize].kind.equip_slot() == slot
                    });
                    if let Some(old) = replaced {
                        ctx.core.player_mut(t).equips.retain(|x| *x != old);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, old);
                        ctx.to_discard(old);
                    }
                    ctx.core.player_mut(t).equips.push(c);
                    crate::kernel::grant_equip_skills(ctx, t, c);
                    ctx.log(LogEvent::Equipped {
                        player: t,
                        card: c,
                        replaced,
                    });
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::GwentYanzhiCard => {
                // 目标为手牌数最少者（之一）→ 摸 2；否则摸 1
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    let min_hand = ctx
                        .core
                        .alive_players()
                        .all(|p| ctx.core.player(p).hand.len() >= ctx.core.player(t).hand.len());
                    let n = if min_hand { 2 } else { 1 };
                    ctx.draw_cards(t, n);
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::GwentAozuCard => {
                // 1 点雷电伤害，若存活摸 1
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(self.owner),
                        t,
                        1,
                        crate::event::DamageNature::Thunder,
                        self.card,
                    )));
                    if ctx.core.player(t).alive {
                        ctx.draw_cards(t, 1);
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::GwentWenyiCard => {
                // 有手牌 → 弃首张（登记差异：随机改首张）；无手牌 →
                // 失去 1 点体力（规则 v77 原语）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    // 上游 target.randomDiscard("h")：随机一张手牌
                    //（逐技对照批四 v78 种子 RNG 均匀抽取）
                    let hand_len = ctx.core.player(t).hand.len();
                    if hand_len > 0 {
                        let i = ctx.core.rng.gen_range(hand_len as u32) as usize;
                        let c = ctx.core.player(t).hand[i];
                        ctx.discard_from_hand(t, c);
                        ctx.log(LogEvent::CardDiscarded { player: t, cards: vec![c] });
                    } else {
                        // 失去 1 点体力（逐技对照批三 v77 消除登记差异）
                        ctx.emit(EventData::LoseHp { player: t, amount: 1 });
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::GwentZhihuanCard => {
                // 反复失去 1 点体力上限，直至未受伤或手牌数 ≥ 体力上限
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    loop {
                        let p = ctx.core.player(t);
                        let damaged = p.hp < p.max_hp as i32;
                        if !(damaged && (p.hand.len() as u32) < p.max_hp) {
                            break;
                        }
                        let p = ctx.core.player_mut(t);
                        p.max_hp -= 1;
                        if p.hp > p.max_hp as i32 {
                            p.hp = p.max_hp as i32;
                        }
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::GwentBaoxueCard => match self.stage {
                HostStage::SelectCard => {
                    // 上游 chooseToDiscard("h",2,true)：目标自选两张手牌
                    //（不足则全选，逐技对照批五 v79 消除「前两张」登记差异）
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let n = ctx.core.player(t).hand.len().min(2);
                    if n == 0 || !ctx.core.player(t).alive {
                        // 无手牌：跳过弃牌；上游 step1 draw 恒执行
                        if ctx.core.player(t).alive {
                            ctx.draw_cards(t, 1);
                        }
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: GW_BAOXUE_DISCARD,
                            options,
                            min: n,
                            max: n,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let picks: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    let mut discarded = Vec::new();
                    for c in picks {
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.core.player_mut(t).hand.retain(|x| *x != c);
                            discarded.push(c);
                        }
                    }
                    if !discarded.is_empty() {
                        ctx.log(LogEvent::CardDiscarded { player: t, cards: discarded });
                    }
                    ctx.draw_cards(t, 1);
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            }
            HostBehavior::GwentQinpenState
            | HostBehavior::GwentBirinState => {
                // 天气到期移除（效果为被动钩子，无步骤）
                food_remove(ctx, self.owner, &self.def_key);
                Flow::Done
            },
            HostBehavior::GwentCiguState => {
                // 摸牌阶段结束：弃首张手牌（若手牌非空，登记差异：
                // 等价净 -1）并移除状态
                if let Some(c) = ctx.core.player(self.owner).hand.first().copied() {
                    ctx.discard_from_hand(self.owner, c);
                    ctx.log(LogEvent::CardDiscarded { player: self.owner, cards: vec![c] });
                }
                food_remove(ctx, self.owner, &self.def_key);
                Flow::Done
            },
            HostBehavior::GwentWeatherGrant => {
                // 对 targets（目标 + 距离 1 邻座）逐个安装对应天气状态
                //（已持有者跳过，对齐上游 filterTarget）
                let kind = self.card.map(|c| ctx.catalog.card_kind(c));
                let listener = match kind {
                    Some(CardKind::Qinpendayu) => Some(GW_QINPEN_STATE_KEY),
                    Some(CardKind::Birinongwu) => Some(GW_BIRIN_STATE_KEY),
                    Some(CardKind::Ciguhanshuang) => Some(GW_CIGU_STATE_KEY),
                    _ => None,
                };
                let Some(listener) = listener else {
                    return Flow::Done;
                };
                let Some(def) = ctx.catalog.skill_by_name(listener) else {
                    return Flow::Done;
                };
                let targets = self.targets.clone();
                for t in targets {
                    if !ctx.core.player(t).alive
                        || ctx.core.skills.contains_key(&(t, def))
                    {
                        continue;
                    }
                    ctx.core.skills.insert(
                        (t, def),
                        crate::state::SkillInstance {
                            owner: t,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::GwentKunenState => {
                // 防止此次普通伤害并按次数扣减计数，归零移除状态
                ctx.current_damage_mut().prevented = true;
                food_tick(ctx, self.owner, &self.def_key);
                Flow::Done
            },
            HostBehavior::GwentKunenCard => {
                // 授予昆恩状态：rounds = min(5, 存活数)
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let Some(def) = ctx.catalog.skill_by_name(GW_KUNEN_STATE_KEY) else {
                    return Flow::Done;
                };
                let rounds = (ctx.core.alive_count() as u8).min(5);
                ctx.core.skills.insert(
                    (t, def),
                    crate::state::SkillInstance {
                        owner: t,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds,
                    },
                );
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::GwentShanbengCard => {
                // 所有其他存活角色装备区合集均匀随机取两张，恰两张才弃置
                //（上游 equips.randomGets(2) + length===2 门控；逐技对照
                // 批四 v78 种子 RNG，无序对均匀：i + j 越过法）
                let pool: Vec<(crate::ids::PlayerId, CardId)> = ctx
                    .core
                    .alive_players()
                    .filter(|v| *v != self.owner)
                    .flat_map(|v| {
                        ctx.core
                            .player(v)
                            .equips
                            .iter()
                            .copied()
                            .map(move |c| (v, c))
                            .collect::<Vec<_>>()
                    })
                    .collect();
                if pool.len() >= 2 {
                    let n = pool.len() as u32;
                    let i = ctx.core.rng.gen_range(n) as usize;
                    let mut j = ctx.core.rng.gen_range(n - 1) as usize;
                    if j >= i {
                        j += 1;
                    }
                    for (v, c) in [pool[i], pool[j]] {
                        ctx.core.player_mut(v).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, v, c);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded { player: v, cards: vec![c] });
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::GwentPoxiaoCard => match self.stage {
                HostStage::Act => {
                    self.stage = HostStage::SelectCard;
                    let options = vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)];
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: GW_POXIAO_BRANCH,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    // 分支 0：清除所有天气状态并弃置全部判定区牌
                    // 分支 1：获得牌堆中首张天气/暴雪法术（登记差异：
                    // 随机改牌堆序首张；上游分支一为自选角色）
                    match ctx.answer_first() {
                        ChoiceOption::Tag(0) => {
                            let weather_defs: Vec<_> = [
                                GW_QINPEN_STATE_KEY,
                                GW_BIRIN_STATE_KEY,
                                GW_CIGU_STATE_KEY,
                            ]
                            .iter()
                            .filter_map(|k| ctx.catalog.skill_by_name(k))
                            .collect();
                            let mut to_remove = Vec::new();
                            for ((o, def), _) in ctx.core.skills.iter() {
                                if weather_defs.contains(def) {
                                    to_remove.push((*o, *def));
                                }
                            }
                            for key in to_remove {
                                ctx.core.skills.remove(&key);
                            }
                            for p in ctx.core.alive_players().collect::<Vec<_>>() {
                                let judge: Vec<CardId> =
                                    ctx.core.player(p).judge.clone();
                                for c in judge {
                                    ctx.core.player_mut(p).judge.retain(|x| *x != c);
                                    ctx.to_discard(c);
                                }
                            }
                        }
                        ChoiceOption::Tag(1) => {
                            // 随机获得一张铜卡法术（上游随机；逐技对照批四
                            // v78 种子 RNG 均匀抽取）
                            let bronze: Vec<CardKind> = vec![
                                CardKind::Qinpendayu,
                                CardKind::Birinongwu,
                                CardKind::Ciguhanshuang,
                                CardKind::Baoxueyaoshui,
                            ];
                            let positions: Vec<usize> = ctx
                                .core
                                .deck
                                .iter()
                                .enumerate()
                                .filter(|(_, c)| bronze.contains(&ctx.catalog.card_kind(**c)))
                                .map(|(i, _)| i)
                                .collect();
                            if positions.is_empty() {
                                ctx.draw_cards(self.owner, 1);
                            } else {
                                let i = positions
                                    [ctx.core.rng.gen_range(positions.len() as u32) as usize];
                                let c = ctx.core.deck.remove(i);
                                ctx.core.player_mut(self.owner).hand.push(c);
                            }
                        }
                        _ => {}
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::GwentFengyinState => {
                // 拥有者任意阶段结束：解除封印——全部技能实例重新
                // 启用，并移除封印实例自身
                for ((p, _), inst) in ctx.core.skills.iter_mut() {
                    if *p == self.owner {
                        inst.enabled = true;
                    }
                }
                if let Some(def) = ctx.catalog.skill_by_name(GW_FENGYIN_STATE_KEY) {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            },
            HostBehavior::GwentZumoCard => {
                // 目标技能全部失效（授予封印实例 + 禁用其余实例；
                // 登记差异：原生无锁定技区分）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let Some(fdef) = ctx.catalog.skill_by_name(GW_FENGYIN_STATE_KEY) else {
                    return Flow::Done;
                };
                let keys: Vec<(PlayerId, crate::ids::SkillDefId)> = ctx
                    .core
                    .skills
                    .iter()
                    .map(|((p, d), _)| (*p, *d))
                    .filter(|(p, d)| *p == t && *d != fdef)
                    .collect();
                for k in keys {
                    if let Some(inst) = ctx.core.skills.get_mut(&k) {
                        inst.enabled = false;
                    }
                }
                ctx.core.skills.insert(
                    (t, fdef),
                    crate::state::SkillInstance {
                        owner: t,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::HuanleMonkeyCard => {
                // 确认后（OptionalGate 通过）：取消桃的结算——
                // ① 使用者手牌中的桃移入自己手牌；② 弃掉自己的猴子
                // （离场撤销装备技）；③ 写回取消位，提交阶段跳过结算。
                let (used_card, user) = {
                    let u = ctx.current_card_use_mut();
                    (u.card, u.player)
                };
                // ① 夺桃：使用者手牌 → 自己手牌。
                let user_hand = &mut ctx.core.players[user as usize].hand;
                let pos = user_hand
                    .iter()
                    .position(|c| *c == used_card)
                    .expect("被取消使用的桃应在使用者手牌");
                let peach = user_hand.remove(pos);
                ctx.core.players[self.owner as usize].hand.push(peach);
                ctx.log(LogEvent::CardsGiven {
                    from: user,
                    to: self.owner,
                    cards: vec![peach],
                });
                // ② 弃猴子（装备区 → 弃牌堆 + 离场撤销装备技）。
                if let Some(pos) = ctx.core.players[self.owner as usize]
                    .equips
                    .iter()
                    .position(|c| {
                        ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Monkey
                    })
                {
                    let monkey = ctx.core.players[self.owner as usize].equips.remove(pos);
                    crate::kernel::revoke_equip_skills(
                        &mut ctx.core,
                        ctx.catalog,
                        self.owner,
                        monkey,
                    );
                    ctx.core.discard.push(monkey);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![monkey],
                    });
                }
                // ③ 写回取消位：提交阶段跳过桃的结算。
                ctx.current_card_use_mut().cancelled = true;
                Flow::Done
            },
            HostBehavior::HuanleXuelunyangCard => {
                // 写轮眼（E03）多步：确认后 ① 选目标（其他有技能者）→
                // ② 选其一个技能（Tag=列表下标，answer 时重枚举映射）→
                // 临时授予至回合结束。
                match self.stage {
                    HostStage::Act => {
                        let candidates: Vec<ChoiceOption> = (0..ctx.core.players.len())
                            .filter(|&i| {
                                i as PlayerId != self.owner
                                    && ctx.core.player(i as PlayerId).alive
                                    && ctx
                                        .core
                                        .skills
                                        .keys()
                                        .any(|(o, _)| *o == i as PlayerId)
                            })
                            .map(|i| ChoiceOption::Player(i as PlayerId))
                            .collect();
                        if candidates.is_empty() {
                            return Flow::Done;
                        }
                        self.stage = HostStage::CompareTargetPick;
                        Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectPlayers,
                                prompt: "huanle_xuelunyang_target",
                                options: candidates,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        }
                    }
                    HostStage::CompareTargetPick => {
                        // 从 SelectPlayers 应答填充窃取目标。
                        let ChoiceOption::Player(target) =
                            ctx.answer().first().copied().unwrap_or(ChoiceOption::Pass)
                        else {
                            return Flow::Done;
                        };
                        self.target = Some(target);
                        // 目标的可窃技能（目录注册且非本窃取技能自身），
                        // 按目录序确定性枚举。
                        let owned: Vec<&str> = ctx
                            .core
                            .skills
                            .range((target, 0)..(target, u16::MAX))
                            .map(|((_, d), _)| ctx.catalog.skills[*d as usize].name)
                            .collect();
                        let tags: Vec<ChoiceOption> = (0..owned.len())
                            .map(|i| ChoiceOption::Tag(i as u8))
                            .collect();
                        if tags.is_empty() {
                            return Flow::Done;
                        }
                        self.stage = HostStage::ComparePlayerPick;
                        // 记录选项数供应答映射：借助 cards 字段存选项数。
                        self.cards.clear();
                        self.cards.push(owned.len() as CardId);
                        Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::PickTag,
                                prompt: "huanle_xuelunyang_skill",
                                options: tags,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        }
                    }
                    HostStage::ComparePlayerPick => {
                        let picked = ctx
                            .answer()
                            .first()
                            .copied()
                            .expect("PickTag 应答存在");
                        let ChoiceOption::Tag(idx) = picked else {
                            return Flow::Done;
                        };
                        let target = self.target.expect("已选目标");
                        let owned: Vec<crate::ids::SkillDefId> = ctx
                            .core
                            .skills
                            .range((target, 0)..(target, u16::MAX))
                            .map(|((_, d), _)| *d)
                            .collect();
                        let Some(def) = owned.get(idx as usize).copied() else {
                            return Flow::Done;
                        };
                        ctx.add_skill_until_turn_end(self.owner, def);
                        Flow::Done
                    }
                    _ => Flow::Done,
                }
            },
            HostBehavior::HuanleMianjuCard => {
                // 强制锁定：写回翻面取消位，提交阶段跳过翻转。
                let (_p, c) = ctx.current_turn_over_before_mut();
                *c = true;
                Flow::Done
            },
            HostBehavior::ZhangbaSha => match self.stage {
                // 丈八蛇矛：两张手牌当杀。Stage Act 已由 activate 设置
                // CardsAsk；恢复顺序：选两张 → 选目标 → 发射 CardUsed。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::CardsAsk;
                    return Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "zhangba_pick_two",
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: false,
                        },
                    };
                }
                HostStage::CardsAsk => {
                    let picked: Vec<crate::ids::CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.len() != 2 {
                        return Flow::Done;
                    }
                    self.cards = picked;
                    // 目标选择：其他存活且在射程内（丈八射程 3）。
                    let candidates: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx
                                    .core
                                    .computed_distance(ctx.catalog, self.owner, *p)
                                    <= ctx.core.attack_range(ctx.catalog, self.owner)
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if candidates.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "zhangba_sha_target",
                            options: candidates,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let (Some(ChoiceOption::Player(target)), [c1, c2]) =
                        (ctx.answer().first(), self.cards.as_slice())
                    else {
                        return Flow::Done;
                    };
                    let target = *target;
                    if !ctx.core.player(target).alive {
                        return Flow::Done;
                    }
                    // 两张手牌弃为转化素材；首张作 CardUsed 载荷牌
                    //（declared Sha，join 判定与伤害归属走杀结算）。
                    ctx.discard_from_hand(self.owner, *c2);
                    let c1_in_hand =
                        ctx.core.player(self.owner).hand.contains(c1);
                    if !c1_in_hand {
                        return Flow::Done;
                    }
                    ctx.core.declared_kinds
                        .insert(*c1, crate::catalog::CardKind::Sha);
                    ctx.emit(crate::event::EventData::CardUsed(
                        crate::machine::card_use_as(
                            *c1,
                            self.owner,
                            vec![target],
                            Some(crate::catalog::CardKind::Sha),
                        ),
                    ));
                    return Flow::Yield; // CardUsed 帧挂起结算（出闪/伤害）
                }
                _ => Flow::Done,
            },
            HostBehavior::SwdLiuxinghuoyuCard => match self.stage {
                HostStage::Act => {
                    // 流星火雨（swd 首批，规则 v86）：目标自选弃两张 he 牌
                    //（可取消）——取消/不足两张则受 1 点火伤（上游 step1
                    // directfalse || !result.bool → damage("fire")）。
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    options.extend(
                        ctx.core
                            .player(t)
                            .equips
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card),
                    );
                    if options.len() < 2 {
                        // he 不足两张：directfalse → 直接火伤
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            self.card,
                        )));
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: SWD_LIUXING_DISCARD,
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let picks: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    let valid: Vec<CardId> = picks
                        .into_iter()
                        .filter(|c| {
                            ctx.core.player(t).hand.contains(c)
                                || ctx.core.player(t).equips.contains(c)
                        })
                        .collect();
                    if valid.len() == 2 {
                        let mut discarded = Vec::new();
                        for c in &valid {
                            if ctx.core.player(t).hand.contains(c) {
                                ctx.core.player_mut(t).hand.retain(|x| x != c);
                            } else {
                                ctx.core.player_mut(t).equips.retain(|x| x != c);
                                crate::kernel::revoke_equip_skills(
                                    ctx.core,
                                    ctx.catalog,
                                    t,
                                    *c,
                                );
                            }
                            ctx.to_discard(*c);
                            discarded.push(*c);
                        }
                        ctx.log(LogEvent::CardDiscarded { player: t, cards: discarded });
                    } else {
                        // 取消/无效 → 1 点火伤
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            self.card,
                        )));
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::SwdShentouEquipSkill => match self.stage {
                HostStage::Act => {
                    // 潜偷（swd 第三批，规则 v92）：出牌阶段限一次，弃一张
                    // 手牌为代价 → 判定梅花则失败 → 成功则获得手牌数比你多
                    // 的目标一张随机手牌（上游 shentou chooseCard cost +
                    // judge 梅花失败 + randomGet 同口径）
                    let nh = ctx.core.player(self.owner).hand.len();
                    let richer = ctx.core.alive_players().any(|p| {
                        p != self.owner && ctx.core.player(p).hand.len() > nh
                    });
                    if nh == 0 || !richer {
                        // 无代价可付或无人手牌更多：技能空发（上游 filter）
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: SWD_SHENTOU_COST,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    // 消费代价牌 → 判定（梅花失败）→ 成功偷牌
                    let cost = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    if let Some(c) =
                        cost.filter(|c| ctx.core.player(self.owner).hand.contains(c))
                    {
                        ctx.discard_from_hand(self.owner, c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                    }
                    let judged = ctx.draw_judge_card();
                    let club = matches!(
                        ctx.catalog.cards[judged as usize].suit,
                        crate::catalog::Suit::Club
                    );
                    ctx.log(LogEvent::Judged {
                        player: self.owner,
                        card: judged,
                    });
                    ctx.to_discard(judged);
                    if club {
                        return Flow::Done;
                    }
                    // 成功：随机一名手牌更多的其他角色 → 随机一张手牌
                    let cands: Vec<crate::ids::PlayerId> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.player(*p).hand.len()
                                    > ctx.core.player(self.owner).hand.len()
                        })
                        .collect();
                    if !cands.is_empty() {
                        let v = cands[ctx.core.rng.gen_range(cands.len() as u32) as usize];
                        let n = ctx.core.player(v).hand.len();
                        let i = ctx.core.rng.gen_range(n as u32) as usize;
                        let c = ctx.core.player_mut(v).hand.remove(i);
                        ctx.core.player_mut(self.owner).hand.push(c);
                        ctx.log(LogEvent::CardsGiven {
                            from: v,
                            to: self.owner,
                            cards: vec![c],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::SwdYufuluEquipSkill => match self.stage {
                // 御夫录（swd 第十二批，规则 v96）：三段——Act 问武器
                //（手牌/装备区 equip1）→ SelectCard 消费并问目标 →
                // TargetChoice 目标受 1 点伤害并获得武器牌（上游
                // damage + gain 同序；dieThen 仍获得登记差异）。
                HostStage::Act => {
                    let is_weapon = |c: CardId| {
                        ctx.catalog.card_kind(c).equip_slot()
                            == Some(crate::catalog::EquipSlot::Weapon)
                    };
                    let has_weapon = ctx.core.player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .any(is_weapon);
                    let others = ctx.core.alive_players().any(|p| p != self.owner);
                    if !has_weapon || !others {
                        return Flow::Done;
                    }
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| is_weapon(*c))
                        .map(ChoiceOption::Card)
                        .collect();
                    options.extend(
                        ctx.core
                            .player(self.owner)
                            .equips
                            .iter()
                            .copied()
                            .filter(|c| is_weapon(*c))
                            .map(ChoiceOption::Card),
                    );
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: SWD_YUFULU_WEAPON,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    // 记录所选武器（保留在原区域，上游 prepare:"give" 同
                    // 口径——卡牌在 content 执行时才移动）→ 问目标
                    let w = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let w = w.filter(|c| {
                        ctx.core.player(self.owner).hand.contains(c)
                            || ctx.core.player(self.owner).equips.contains(c)
                    });
                    let Some(weapon) = w else {
                        return Flow::Done;
                    };
                    self.card = Some(weapon);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: SWD_YUFULU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(weapon) = self.card.take() else {
                        return Flow::Done;
                    };
                    let target = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let Some(t) = target.filter(|p| ctx.core.player(*p).alive) else {
                        return Flow::Done;
                    };
                    // 此刻才移动武器（手牌→直接取出；装备区→取出并撤销
                    // 装备技），保持存档核算各询问挂起态区域完整
                    if ctx.core.player(self.owner).hand.contains(&weapon) {
                        ctx.core
                            .player_mut(self.owner)
                            .hand
                            .retain(|x| *x != weapon);
                    } else if ctx.core.player(self.owner).equips.contains(&weapon) {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|x| *x != weapon);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            weapon,
                        );
                    } else {
                        return Flow::Done;
                    }
                    // 上游 content：damage 后 gain
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(self.owner),
                        t,
                        1,
                        crate::event::DamageNature::Normal,
                        None,
                    )));
                    ctx.core.player_mut(t).hand.push(weapon);
                    ctx.log(LogEvent::CardsGiven {
                        from: self.owner,
                        to: t,
                        cards: vec![weapon],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::SwdYangpijuanCard => match self.stage {
                // 羊皮卷（swd 第十三批，规则 v97）：选类别（0=基本
                // 1=锦囊 2=装备）→ 牌堆随机亮该类别 ≤3 张 → 选一张入手。
                // 上游 createCard 凭空造牌，原生从牌堆发现（登记差异）；
                // 亮出的牌择定前留在牌堆（挂起态区域完整，御夫录同口径）。
                HostStage::Act => {
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: SWD_YANGPIJUAN_CATEGORY,
                            options: (0..3).map(ChoiceOption::Tag).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let class = match ctx.answer().first() {
                        Some(ChoiceOption::Tag(t @ 0..=2)) => *t,
                        _ => return Flow::Done,
                    };
                    // 牌堆部分 Fisher-Yates 取前 ≤3 张（RNG 真消除口径）
                    let mut pool: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .copied()
                        .filter(|c| {
                            yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(class)
                        })
                        .collect();
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    let n = pool.len().min(3);
                    for i in 0..n {
                        let j = i
                            + ctx.core.rng.gen_range((pool.len() - i) as u32) as usize;
                        pool.swap(i, j);
                    }
                    self.cards = pool[..n].to_vec();
                    let options: Vec<ChoiceOption> = self
                        .cards
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: SWD_YANGPIJUAN_PICK,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(picked) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    // 择定此刻才离牌堆入手（五谷丰登同款日志口径）
                    let Some(pos) =
                        ctx.core.deck.iter().position(|c| *c == picked)
                    else {
                        return Flow::Done;
                    };
                    ctx.core.deck.remove(pos);
                    ctx.core.player_mut(self.owner).hand.push(picked);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: vec![picked],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::SwdLangeguaiyiEquipSkill => match self.stage {
                // 蓝格怪衣「龙帆」（swd 第十五批，规则 v99）：判定按花色
                // 分支——红桃回 1 体力（ctx.recover 满血饱和为既有路径，
                // 上游 recover 同钳制）、方片摸 1、梅花随机敌人随机弃
                // 一张、黑桃无事。上游 judge 的 suit 提示函数为 AI 排序
                // （不迁移）；判定牌入弃堆（Shentou 同款）。
                HostStage::Act => {
                    let judged = ctx.draw_judge_card();
                    let suit = ctx.catalog.cards[judged as usize].suit;
                    ctx.log(LogEvent::Judged {
                        player: self.owner,
                        card: judged,
                    });
                    ctx.to_discard(judged);
                    use crate::catalog::Suit;
                    match suit {
                        Suit::Heart => {
                            ctx.recover(self.owner, 1, None);
                        }
                        Suit::Diamond => {
                            ctx.draw_cards(self.owner, 1);
                        }
                        Suit::Club => {
                            // 候选=其他存活且（手牌∪装备区）非空（上游
                            // he filter 同口径）；随机取一。上游
                            // randomDiscard() 默认区不可从归档核验（现有
                            // 调用点均显式 "h"）——登记差异：手牌优先、
                            // 无手牌退随机装备（离装撤销装备技）。
                            let mut cands: Vec<crate::ids::PlayerId> = ctx
                                .core
                                .alive_players()
                                .filter(|p| {
                                    *p != self.owner
                                        && (!ctx.core.player(*p).hand.is_empty()
                                            || !ctx.core.player(*p).equips.is_empty())
                                })
                                .collect();
                            if !cands.is_empty() {
                                let i = ctx
                                    .core
                                    .rng
                                    .gen_range(cands.len() as u32)
                                    as usize;
                                let t = cands.swap_remove(i);
                                if !ctx.core.player(t).hand.is_empty() {
                                    let n = ctx.core.player(t).hand.len();
                                    let j = ctx
                                        .core
                                        .rng
                                        .gen_range(n as u32) as usize;
                                    let c = ctx.core.player_mut(t).hand.remove(j);
                                    ctx.to_discard(c);
                                    ctx.log(LogEvent::CardDiscarded {
                                        player: t,
                                        cards: vec![c],
                                    });
                                } else {
                                    let n = ctx.core.player(t).equips.len();
                                    let j = ctx
                                        .core
                                        .rng
                                        .gen_range(n as u32) as usize;
                                    let c = ctx.core.player_mut(t).equips.remove(j);
                                    crate::kernel::revoke_equip_skills(
                                        ctx.core,
                                        ctx.catalog,
                                        t,
                                        c,
                                    );
                                    ctx.to_discard(c);
                                    ctx.log(LogEvent::CardDiscarded {
                                        player: t,
                                        cards: vec![c],
                                    });
                                }
                            }
                        }
                        Suit::Spade => {}
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgLongshanfengLand => match self.stage {
                // 龙颅山峰（mtg 首切片，规则 v100）：Act 换地（覆盖
                // current_land；land_used 本回合门在 PlayCard 门控）→
                // 问获得杀/闪 → SelectCard 从牌堆随机取一张该类牌入手
                //（上游 createCard 凭空造牌→牌堆取，羊皮卷同款登记差异；
                // 牌堆无该类牌则放弃该臂）。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Longlushanfeng);
                    ctx.core.current_land = Some(crate::catalog::CardKind::Longlushanfeng);
                    ctx.core.turn.land_used = true;
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: MTG_LONGSHANFENG_GAIN,
                            options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let want_sha = match ctx.answer().first() {
                        Some(ChoiceOption::Tag(0)) => true,
                        Some(ChoiceOption::Tag(1)) => false,
                        _ => return Flow::Done,
                    };
                    let want = if want_sha {
                        crate::catalog::CardKind::Sha
                    } else {
                        crate::catalog::CardKind::Shan
                    };
                    let Some(pos) = ctx
                        .core
                        .deck
                        .iter()
                        .position(|c| ctx.catalog.card_kind(*c) == want)
                    else {
                        return Flow::Done;
                    };
                    // 上游 get.cardPile 取牌堆顶侧；原生取牌堆内首张该类
                    //（确定性；登记差异与羊皮卷 RNG 口径并注）
                    let c = ctx.core.deck.remove(pos);
                    ctx.core.player_mut(self.owner).hand.push(c);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: vec![c],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgHaidaoLand => match self.stage {
                // 海岛（mtg 第二批，规则 v101）：Act 换地 → 满血直接 +1
                // 护甲（不受伤，转换不触发）；受伤问弃一张（he，可取消）
                // → SelectCard 弃则回 1 / 取消得 1 护甲（上游 _temp 标记
                // 口径：此路线绕过转换原语，路径级抑制）。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Haidao);
                    ctx.core.current_land = Some(crate::catalog::CardKind::Haidao);
                    ctx.core.turn.land_used = true;
                    let p = ctx.core.player(self.owner);
                    if p.hp >= p.max_hp as i32 {
                        ctx.change_hujia(self.owner, 1);
                        return Flow::Done;
                    }
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    options.extend(
                        ctx.core
                            .player(self.owner)
                            .equips
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card),
                    );
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_HAIDAO_DISCARD,
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    // 取消=空选择（allow_cancel 既有口径）→ +1 护甲
                    let picked = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    match picked {
                        Some(c) => {
                            // 弃一张（手牌直弃；装备离装撤销装备技）
                            if ctx.core.player(self.owner).hand.contains(&c) {
                                ctx.discard_from_hand(self.owner, c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: self.owner,
                                    cards: vec![c],
                                });
                            } else if ctx.core.player(self.owner).equips.contains(&c) {
                                ctx.core
                                    .player_mut(self.owner)
                                    .equips
                                    .retain(|x| *x != c);
                                crate::kernel::revoke_equip_skills(
                                    ctx.core,
                                    ctx.catalog,
                                    self.owner,
                                    c,
                                );
                                ctx.to_discard(c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: self.owner,
                                    cards: vec![c],
                                });
                            }
                            ctx.recover(self.owner, 1, None);
                        }
                        None => {
                            // 路径级抑制（上游 _temp_mtg_haidao 同口径）：
                            // 直写护甲字段不经转换原语
                            let cur = ctx.core.player(self.owner).hujia;
                            ctx.core.player_mut(self.owner).hujia = cur + 1;
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgShuimomuxueLand => match self.stage {
                // 水没墓穴（mtg 第三批，规则 v102）：Act 换地 → he 有牌
                // 则问弃一张（可取消）→ SelectCard 弃则摸二/取消无事
                //（上游 chooseToDiscard result.bool 同口径）。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Shuimomuxue);
                    ctx.core.current_land = Some(crate::catalog::CardKind::Shuimomuxue);
                    ctx.core.turn.land_used = true;
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    options.extend(
                        ctx.core
                            .player(self.owner)
                            .equips
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card),
                    );
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_SHUIMOMUXUE_DISCARD,
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    if let Some(c) = picked {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        } else if ctx.core.player(self.owner).equips.contains(&c) {
                            ctx.core
                                .player_mut(self.owner)
                                .equips
                                .retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                c,
                            );
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        }
                        ctx.draw_cards(self.owner, 2);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgFeixuLand => match self.stage {
                // 废墟（mtg 第四批，规则 v103）：Act 换地 → 弃牌堆随机
                // 亮 ≤3 张（留弃堆，羊皮卷同口径）→ 问三选一 → SelectCard
                // 选定从弃堆入手；弃堆空则空发。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Feixu);
                    ctx.core.current_land = Some(crate::catalog::CardKind::Feixu);
                    ctx.core.turn.land_used = true;
                    let mut pool: Vec<CardId> =
                        ctx.core.discard.iter().copied().collect();
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    let n = pool.len().min(3);
                    for i in 0..n {
                        let j = i
                            + ctx.core.rng.gen_range((pool.len() - i) as u32)
                                as usize;
                        pool.swap(i, j);
                    }
                    self.cards = pool[..n].to_vec();
                    let options: Vec<ChoiceOption> = self
                        .cards
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_FEIXU_DISCOVER,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(picked) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    // 择定此刻才离弃堆（亮出态仍在弃堆，区域完整）
                    let Some(pos) =
                        ctx.core.discard.iter().position(|c| *c == picked)
                    else {
                        return Flow::Done;
                    };
                    ctx.core.discard.remove(pos);
                    ctx.core.player_mut(self.owner).hand.push(picked);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: vec![picked],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgYixialanLand => match self.stage {
                // 依夏兰（mtg 第五批，规则 v104）：Act 换地 + 同步地附
                // 技实例 → 问类型（0=基本 1=锦囊）→ SelectCard 牌堆随机
                // 取该类一张入手（上游 createCard(inpile.randomGet())→
                // 牌堆实体取，羊皮卷登记差异口径；无该类牌则空发）。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Yixialan);
                    ctx.core.current_land = Some(CardKind::Yixialan);
                    ctx.core.turn.land_used = true;
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: MTG_YIXIALAN_GAIN,
                            options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let class = match ctx.answer().first() {
                        Some(ChoiceOption::Tag(t @ 0..=1)) => *t,
                        _ => return Flow::Done,
                    };
                    let pool: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .copied()
                        .filter(|c| {
                            yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(class)
                        })
                        .collect();
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    let i = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                    let c = pool[i];
                    let pos = ctx.core.deck.iter().position(|x| *x == c).expect("deck contains");
                    ctx.core.deck.remove(pos);
                    ctx.core.player_mut(self.owner).hand.push(c);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: vec![c],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgYixialanRecast => match self.stage {
                // 重铸（mtg 第五批，规则 v104）：Act 问手牌基本牌 →
                // SelectCard 放回牌堆 → 获得牌堆一张锦囊（无锦囊摸一，
                // 上游 recast 回调补偿同口径）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(0)
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_YIXIALAN_RECAST_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(basic) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&basic) {
                        return Flow::Done;
                    }
                    // 放回牌堆（顶侧惯例，上游 recast 洗回同义）
                    ctx.core
                        .player_mut(self.owner)
                        .hand
                        .retain(|x| *x != basic);
                    ctx.core.deck.push(basic);
                    // 牌堆找一张锦囊获得；无则摸一补偿
                    let trick = ctx
                        .core
                        .deck
                        .iter()
                        .copied()
                        .find(|c| {
                            yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(1)
                        });
                    match trick {
                        Some(t) => {
                            let pos = ctx
                                .core
                                .deck
                                .iter()
                                .position(|x| *x == t)
                                .expect("deck contains");
                            ctx.core.deck.remove(pos);
                            ctx.core.player_mut(self.owner).hand.push(t);
                            ctx.log(LogEvent::CardsDrawn {
                                player: self.owner,
                                cards: vec![t],
                            });
                        }
                        None => {
                            ctx.draw_cards(self.owner, 1);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgShamolvzhouLand => match self.stage {
                // 沙漠绿洲（mtg 第六批，规则 v105）：Act 换地 → 牌堆基本
                // 牌随机亮 ≤3 张（留牌堆）→ 问三选一 → SelectCard 选定
                // 从牌堆入手；牌堆无基本牌空发。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Shamolvzhou);
                    ctx.core.current_land = Some(CardKind::Shamolvzhou);
                    ctx.core.turn.land_used = true;
                    let mut pool: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .copied()
                        .filter(|c| {
                            yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(0)
                        })
                        .collect();
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    let n = pool.len().min(3);
                    for i in 0..n {
                        let j = i
                            + ctx.core.rng.gen_range((pool.len() - i) as u32)
                                as usize;
                        pool.swap(i, j);
                    }
                    self.cards = pool[..n].to_vec();
                    let options: Vec<ChoiceOption> = self
                        .cards
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_SHAMOLVZHOU_DISCOVER,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(picked) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let Some(pos) = ctx.core.deck.iter().position(|c| *c == picked)
                    else {
                        return Flow::Done;
                    };
                    ctx.core.deck.remove(pos);
                    ctx.core.player_mut(self.owner).hand.push(picked);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: vec![picked],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgLinzhongjiantaLand => match self.stage {
                // 林中尖塔（mtg 第七批，规则 v106）：Act 换地 → 牌堆武器
                // 亮 ≤3 三选一 → SelectCard 选定 emit Equipped（走完整装
                // 配：替换/装备技授予）；无武器空发。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Linzhongjianta);
                    ctx.core.current_land = Some(CardKind::Linzhongjianta);
                    ctx.core.turn.land_used = true;
                    let mut pool: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .copied()
                        .filter(|c| {
                            ctx.catalog.card_kind(*c).equip_slot()
                                == Some(crate::catalog::EquipSlot::Weapon)
                        })
                        .collect();
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    let n = pool.len().min(3);
                    for i in 0..n {
                        let j = i
                            + ctx.core.rng.gen_range((pool.len() - i) as u32)
                                as usize;
                        pool.swap(i, j);
                    }
                    self.cards = pool[..n].to_vec();
                    let options: Vec<ChoiceOption> = self
                        .cards
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_LINZHONGJIANTA_DISCOVER,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(picked) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let Some(pos) = ctx.core.deck.iter().position(|c| *c == picked)
                    else {
                        return Flow::Done;
                    };
                    ctx.core.deck.remove(pos);
                    // 实体牌入手后 emit Equipped（commit_equip 从手牌取
                    // 牌的适配；走完整装配：同栏替换/装备技授予）
                    ctx.core.player_mut(self.owner).hand.push(picked);
                    ctx.emit(EventData::Equipped {
                        player: self.owner,
                        card: picked,
                        replaced: None,
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgLinzhongjiantaStrike => match self.stage {
                // 塔击（mtg 第七批，规则 v106）：Act 问手牌基本牌 →
                // SelectCard 问目标 → declared-Sha 进入杀结算（丈八蛇矛
                // TargetRead 同构；登记差异：上游响应/出牌双入口，原生
                // 仅出牌入口）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(0)
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_LINZHONGJIANTA_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(basic) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&basic) {
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.cards = vec![basic];
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: MTG_LINZHONGJIANTA_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(basic) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive
                        || !ctx.core.player(self.owner).hand.contains(&basic)
                    {
                        return Flow::Done;
                    }
                    // 素材牌留手牌由杀结算消费（丈八 TargetRead 同构；
                    // commit 从手牌取牌）
                    ctx.core.declared_kinds.insert(basic, CardKind::Sha);
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        basic,
                        self.owner,
                        vec![t],
                        Some(CardKind::Sha),
                    )));
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgBingheyaosaiLand => match self.stage {
                // 冰河要塞（mtg 第八批，规则 v107）：Act 换地 + 摸 2 →
                // 问弃两张（he=手牌∪装备，强制 min2 max2；摸 2 保证手牌
                // 恒 ≥2）→ SelectCard 弃所选项。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Bingheyaosai);
                    ctx.core.current_land = Some(CardKind::Bingheyaosai);
                    ctx.core.turn.land_used = true;
                    ctx.draw_cards(self.owner, 2);
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    options.extend(
                        ctx.core
                            .player(self.owner)
                            .equips
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card),
                    );
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_BINGHEYAOSAI_DISCARD,
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    for c in picked {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        } else if ctx.core.player(self.owner).equips.contains(&c) {
                            ctx.core
                                .player_mut(self.owner)
                                .equips
                                .retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                c,
                            );
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgYoulinLand => match self.stage {
                // 幽林（mtg 第九批，规则 v108）：Act 换地 → 池=牌堆中的
                // 决斗/无中生有（登记差异：上游全普通锦囊池+正收益 AI
                // 判定；原生收窄为内核结算已确证者，无懈窗口照常）→ 随
                // 机一张随机其他存活目标（无中无目标）→ emit 结算；池
                // 空空发。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Youlin);
                    ctx.core.current_land = Some(CardKind::Youlin);
                    ctx.core.turn.land_used = true;
                    let pool: Vec<(CardId, bool)> = ctx
                        .core
                        .deck
                        .iter()
                        .copied()
                        .filter_map(|c| match ctx.catalog.card_kind(c) {
                            CardKind::Duel => Some((c, true)),
                            CardKind::Wuzhong => Some((c, false)),
                            _ => None,
                        })
                        .collect();
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    let i = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                    let (card, has_target) = pool[i];
                    let pos =
                        ctx.core.deck.iter().position(|x| *x == card).expect("deck contains");
                    ctx.core.deck.remove(pos);
                    // 实体牌入手后 emit CardUsed（commit_card_used 从手
                    // 牌取牌的适配；内核各锦囊结算/无懈窗口自动复用）
                    ctx.core.player_mut(self.owner).hand.push(card);
                    let targets: Vec<PlayerId> = if has_target {
                        let cands: Vec<PlayerId> = ctx
                            .core
                            .alive_players()
                            .filter(|p| *p != self.owner)
                            .collect();
                        if cands.is_empty() {
                            ctx.core.player_mut(self.owner).hand.retain(|x| *x != card);
                            ctx.to_discard(card);
                            return Flow::Done;
                        }
                        let j = ctx.core.rng.gen_range(cands.len() as u32) as usize;
                        vec![cands[j]]
                    } else {
                        Vec::new()
                    };
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        card,
                        self.owner,
                        targets,
                        None,
                    )));
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgYoulinDiscard => match self.stage {
                // 幽林地附技（mtg 第九批，规则 v108）：Act 问手牌锦囊 →
                // SelectCard 弃 → 牌堆随机亮 ≤3（全类别，留牌堆）→ 问三
                // 选一 → TargetChoice 选定入手。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(1)
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_YOULIN_DISCARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(trick) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&trick) {
                        return Flow::Done;
                    }
                    ctx.discard_from_hand(self.owner, trick);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![trick],
                    });
                    let mut pool: Vec<CardId> =
                        ctx.core.deck.iter().copied().collect();
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    let n = pool.len().min(3);
                    for i in 0..n {
                        let j = i
                            + ctx.core.rng.gen_range((pool.len() - i) as u32)
                                as usize;
                        pool.swap(i, j);
                    }
                    self.cards = pool[..n].to_vec();
                    let options: Vec<ChoiceOption> = self
                        .cards
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_YOULIN_PICK,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(picked) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let Some(pos) = ctx.core.deck.iter().position(|c| *c == picked)
                    else {
                        return Flow::Done;
                    };
                    ctx.core.deck.remove(pos);
                    ctx.core.player_mut(self.owner).hand.push(picked);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: vec![picked],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::MtgLindixiliuLand => match self.stage {
                // 邻地溪流（mtg 第十批，规则 v109）：Act 换地 → 牌堆♥
                // 首张入手（get.cardPile2 顺序同义）→ 手牌∪装备♥非空则
                // 问弃任意张（可取消）→ SelectCard 逐张弃置，每张随机
                // 敌方判定区置入其未持有的延时锦囊（牌堆 Le→Bing→
                // ShanDian 顺序，上游 inpile("delay") 同序；判定区直接
                // push，JudgePhase 阶段取队；专用日志暂缓登记）。
                HostStage::Act => {
                    sync_land_skill(ctx, self.owner, CardKind::Lindixiliu);
                    ctx.core.current_land = Some(CardKind::Lindixiliu);
                    ctx.core.turn.land_used = true;
                    // 牌堆♥首张入手
                    if let Some(pos) = ctx
                        .core
                        .deck
                        .iter()
                        .position(|c| {
                            ctx.catalog.cards[*c as usize].suit
                                == crate::catalog::Suit::Heart
                        })
                    {
                        let c = ctx.core.deck.remove(pos);
                        ctx.core.player_mut(self.owner).hand.push(c);
                        ctx.log(LogEvent::CardsDrawn {
                            player: self.owner,
                            cards: vec![c],
                        });
                    }
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            ctx.catalog.cards[*c as usize].suit
                                == crate::catalog::Suit::Heart
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    // 装备区♥并入（chooseToDiscard "he"）
                    let equip_hearts: Vec<CardId> = ctx
                        .core
                        .player(self.owner)
                        .equips
                        .iter()
                        .copied()
                        .filter(|c| {
                            ctx.catalog.cards[*c as usize].suit
                                == crate::catalog::Suit::Heart
                        })
                        .collect();
                    options.extend(equip_hearts.iter().copied().map(ChoiceOption::Card));
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.cards = equip_hearts;
                    self.stage = HostStage::SelectCard;
                    let max_n = options.len();
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: MTG_LINDIXILIU_DISCARD,
                            options,
                            min: 0,
                            max: max_n,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    for c in picked {
                        // 弃置（手牌直弃/装备离装撤销技）
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        } else if ctx.core.player(self.owner).equips.contains(&c) {
                            ctx.core
                                .player_mut(self.owner)
                                .equips
                                .retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                c,
                            );
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        }
                        // 随机敌方 → 置入其未持有的延时锦囊（Le→Bing→
                        // ShanDian 顺序，牌堆实体）
                        let cands: Vec<crate::ids::PlayerId> = ctx
                            .core
                            .alive_players()
                            .filter(|p| *p != self.owner)
                            .collect();
                        if cands.is_empty() {
                            continue;
                        }
                        let j =
                            ctx.core.rng.gen_range(cands.len() as u32) as usize;
                        let t = cands[j];
                        let want = [
                            CardKind::Le,
                            CardKind::Bing,
                            CardKind::ShanDian,
                        ]
                        .into_iter()
                        .find(|k| {
                            !ctx.core.player(t).judge.iter().any(|jc| {
                                ctx.catalog.card_kind(*jc) == *k
                            })
                        });
                        let Some(kind) = want else { continue };
                        let Some(pos) = ctx.core.deck.iter().position(|dc| {
                            ctx.catalog.card_kind(*dc) == kind
                        }) else {
                            continue;
                        };
                        let dc = ctx.core.deck.remove(pos);
                        if ctx.core.player(t).alive {
                            ctx.core.player_mut(t).judge.push(dc);
                        } else {
                            ctx.to_discard(dc);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianCiqiu => {
                // 刺酋（玩点论杀/韩龙，规则 v111）：锁定——你的杀对满血目标
                // 伤害 +1；加成后达到目标体力则其本回合濒死即死、本技能移除。
                // 上游 damageBegin1；效果在 commit_damage（kernel.rs 142 行）
                // 与濒死处理（kernel.rs 3568 行），宿主无机器步骤。
                Flow::Done
            },
            HostBehavior::WudianShuaiyan => match self.stage {
                // 率言（U07 批二，规则 v112）：Act 问回复者二选一 →
                // SelectCard：Tag0 费诗摸 1 / Tag1 问回复者弃一张 →
                // TargetChoice 弃置。
                HostStage::Act => {
                    let Some(r) = self.target else {
                        return Flow::Done;
                    };
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: r,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: WUDIAN_SHUAIYAN_CHOICE,
                            options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(r) = self.target else {
                        return Flow::Done;
                    };
                    match ctx.answer().first() {
                        Some(ChoiceOption::Tag(0)) => {
                            ctx.draw_cards(self.owner, 1);
                            Flow::Done
                        }
                        Some(ChoiceOption::Tag(1)) => {
                            if ctx.core.player(r).hand.is_empty() {
                                return Flow::Done;
                            }
                            let options: Vec<ChoiceOption> = ctx
                                .core
                                .player(r)
                                .hand
                                .iter()
                                .copied()
                                .map(ChoiceOption::Card)
                                .collect();
                            self.stage = HostStage::TargetChoice;
                            Flow::WaitChoice {
                                observer: r,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectCards,
                                    prompt: WUDIAN_SHUAIYAN_DISCARD,
                                    options,
                                    min: 1,
                                    max: 1,
                                    allow_cancel: false,
                                },
                            }
                        }
                        _ => Flow::Done,
                    }
                }
                HostStage::TargetChoice => {
                    let Some(r) = self.target else {
                        return Flow::Done;
                    };
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        if ctx.core.player(r).hand.contains(&c) {
                            ctx.discard_from_hand(r, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: r,
                                cards: vec![c],
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianZhucheng => match self.stage {
                // 筑城开关（U07 批三 v113）：确认（OptionalGate）后——无
                // 标记 → 牌堆顶取 max(1, maxHp-体力) 张存标记；有标记 →
                // 收回入手并清空。
                HostStage::Act => {
                    if ctx.core.player(self.owner).zhucheng.is_empty() {
                        let n = 1.max(
                            ctx.core.player(self.owner).max_hp as i32
                                - ctx.core.player(self.owner).hp,
                        ) as usize;
                        for _ in 0..n {
                            if ctx.core.deck.is_empty() {
                                break;
                            }
                            let c =
                                ctx.core.deck.remove(ctx.core.deck.len() - 1);
                            ctx.core.player_mut(self.owner).zhucheng.push(c);
                        }
                    } else {
                        let cards = std::mem::take(
                            &mut ctx.core.player_mut(self.owner).zhucheng,
                        );
                        ctx.core.player_mut(self.owner).hand.extend(cards);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianZhuchengGuard => match self.stage {
                // 筑城守卫（U07 批三 v113）：Act 问使用者——选一张=弃
                // 筑城张数张（登记差异：上游一次性选 n 张，原生首张自选
                // +其余随机代抽）；Pass=杀无效（帧 cancelled + 杀归还手
                // 牌，Intrinsic 阶段按 cancelled 跳过结算）。
                HostStage::Act => {
                    let Some(attacker) = self.target else {
                        return Flow::Done;
                    };
                    let mut options = vec![ChoiceOption::Pass];
                    options.extend(
                        ctx.core
                            .player(attacker)
                            .hand
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card),
                    );
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: attacker,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_ZHUCHENG_GUARD_ASK,
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(attacker) = self.target else {
                        return Flow::Done;
                    };
                    let answered = ctx.answer().first().copied();
                    match answered {
                        Some(ChoiceOption::Card(c)) => {
                            let n = ctx.core.player(self.owner).zhucheng.len();
                            if ctx.core.player(attacker).hand.contains(&c) {
                                ctx.discard_from_hand(attacker, c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: attacker,
                                    cards: vec![c],
                                });
                            }
                            let mut extra = n.saturating_sub(1);
                            while extra > 0
                                && !ctx.core.player(attacker).hand.is_empty()
                            {
                                let idx = ctx.core.rng.gen_range(
                                    ctx.core.player(attacker).hand.len() as u32,
                                ) as usize;
                                let c = ctx.core.player(attacker).hand[idx];
                                ctx.discard_from_hand(attacker, c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: attacker,
                                    cards: vec![c],
                                });
                                extra -= 1;
                            }
                            Flow::Done
                        }
                        _ => {
                            // 杀无效：帧 cancelled + 杀归还手牌（Intrinsic
                            // 阶段按 cancelled 跳过建 ShaResolve）
                            if let Some(c) = self.card {
                                ctx.core.table.retain(|x| *x != c);
                                ctx.core.player_mut(attacker).hand.push(c);
                            }
                            let fid = ctx.current_frame_id();
                            if let Some(f) = ctx.res.frame_mut(fid) {
                                if let EventData::CardUsed(u) = &mut f.data {
                                    u.cancelled = true;
                                }
                            }
                            Flow::Done
                        }
                    }
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianKangyin => match self.stage {
                // 抗yin（U07 批四，规则 v114）：Act 问目标 → SelectCard
                // 失去 1 体力 + 被弃者选弃一张 → TargetChoice 分支——已
                // 受伤：基本=选至多 X 名各摸 1（CardsAsk），非基本=授
                // kangyin2（回合内）；未受伤结束。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && (!ctx.core.player(*p).hand.is_empty()
                                    || !ctx.core.player(*p).equips.is_empty())
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_KANGYIN_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    self.target = Some(t);
                    // 失去 1 点体力（上游 loseHp 同口径）
                    let hp0 = ctx.core.player(self.owner).hp;
                    ctx.core.player_mut(self.owner).hp = hp0 - 1;
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    options.extend(
                        ctx.core
                            .player(t)
                            .equips
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card),
                    );
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_KANGYIN_DISCARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let Some(c) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).hand.contains(&c) {
                        ctx.discard_from_hand(t, c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    } else if ctx.core.player(t).equips.contains(&c) {
                        ctx.core.player_mut(t).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            t,
                            c,
                        );
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    }
                    let p = ctx.core.player(self.owner);
                    if p.hp >= p.max_hp as i32 {
                        return Flow::Done;
                    }
                    let x = (p.max_hp - p.hp as u32).max(1);
                    let is_basic =
                        yangpijuan_class(ctx.catalog.card_kind(c)) == Some(0);
                    if is_basic {
                        let options: Vec<ChoiceOption> = ctx
                            .core
                            .alive_players()
                            .map(ChoiceOption::Player)
                            .collect();
                        self.cards = vec![x as CardId];
                        self.stage = HostStage::CardsAsk;
                        Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectPlayers,
                                prompt: WUDIAN_KANGYIN_DRAW,
                                options,
                                min: 1,
                                max: x as usize,
                                allow_cancel: false,
                            },
                        }
                    } else {
                        ctx.core.turn.kangyin2 = Some((self.owner, x));
                        Flow::Done
                    }
                }
                HostStage::CardsAsk => {
                    let picked: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    for p in picked {
                        ctx.draw_cards(p, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianChezhen => {
                // 车阵（玩点论杀/田豫，规则 v115）：锁定——持有者装备区有牌
                // 时其到他人距离 -1；目标装备区为空时其被算到 +1。上游仅
                // mod.globalFrom/globalTo；效果在距离计算（state.rs 700 行），
                // 宿主无机器步骤。
                Flow::Done
            },
            HostBehavior::WudianJuedao => {
                // 绝道（U07 批六 v116）：出牌阶段横置（自连环，已横置
                // 不可发动=activate 层互斥）；横置距离双向 +1 由
                // computed_distance 直读 linked（无机器步）。
                if !ctx.core.player(self.owner).linked {
                    ctx.core.player_mut(self.owner).linked = true;
                    ctx.log(LogEvent::Linked {
                        player: self.owner,
                        linked: true,
                    });
                }
                Flow::Done
            },
            HostBehavior::WudianGeju => {
                // 割据（U07 批七 v117）：摸「攻击范围外其他势力种数」张
                // （势力=effective_faction；unknown/无主将排除；杀范围内
                // 他势力不计入——上游 geju 同口径）
                let mut factions: Vec<u8> = Vec::new();
                for q in ctx.core.alive_players() {
                    if q == self.owner {
                        continue;
                    }
                    if let Some(f) =
                        ctx.core.effective_faction(ctx.catalog, q)
                    {
                        let fb = f as u8;
                        if !factions.contains(&fb) {
                            factions.push(fb);
                        }
                    }
                }
                let range = ctx.core.attack_range(ctx.catalog, self.owner);
                for q in ctx.core.alive_players() {
                    if q == self.owner {
                        continue;
                    }
                    let in_range = ctx
                        .core
                        .computed_distance(ctx.catalog, q, self.owner)
                        <= range;
                    if in_range {
                        if let Some(f) =
                            ctx.core.effective_faction(ctx.catalog, q)
                        {
                            factions.retain(|fb| *fb != f as u8);
                        }
                    }
                }
                if !factions.is_empty() {
                    ctx.draw_cards(self.owner, factions.len());
                }
                Flow::Done
            },
            HostBehavior::WudianTuji => {
                // 突击（ow/士兵76，规则 v118）：锁定——持有者回合内每已使用
                // 一张牌，其进攻距离 -1。上游仅 mod.globalFrom；效果在距离
                // 计算（state.rs 663 行），宿主无机器步骤。
                Flow::Done
            },
            HostBehavior::WudianMujing => match self.stage {
                // 目镜（U07 ow，规则 v118）：Act 问黑色牌 → SelectCard 问
                // 目标 → declared-Sha 结算（塔击同构）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Spade
                                    | crate::catalog::Suit::Club
                            )
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_MUJING_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(basic) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&basic) {
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.cards = vec![basic];
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_MUJING_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(basic) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive
                        || !ctx.core.player(self.owner).hand.contains(&basic)
                    {
                        return Flow::Done;
                    }
                    // 素材牌留手牌由杀结算消费（丈八 TargetRead 同构）
                    ctx.core.declared_kinds.insert(basic, CardKind::Sha);
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        basic,
                        self.owner,
                        vec![t],
                        Some(CardKind::Sha),
                    )));
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianGuangshu => match self.stage {
                // 光书（U05 ow 批二 v119）：出牌阶段弃 he 一张并指定一名
                // 角色，按弃牌花色授对应子技（♥受伤回1/♦伤害后摸2/
                // ♣至下阶段结束不能用杀/♠下阶段结束受1无源雷伤；各一次
                // 性移除；已有任意子技者不可指定）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_GUANGSHU_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied()
                    else {
                        return Flow::Done;
                    };
                    self.cards = vec![c];
                    // 目标：四子技同查排除（已有任意子技者不可指定）
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .into_iter()
                        .filter(|t| {
                            *t != self.owner
                                && ![WUDIAN_GUANGSHU_HEART_KEY, WUDIAN_GUANGSHU_DIAMOND_KEY, WUDIAN_GUANGSHU_CLUB_KEY, WUDIAN_GUANGSHU_SPADE_KEY]
                                    .iter()
                                    .any(|k| {
                                        ctx.catalog
                                            .skill_by_name(k)
                                            .map(|d| ctx.core.skills.contains_key(&(*t, d)))
                                            .unwrap_or(false)
                                    })
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_GUANGSHU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied()
                    else {
                        return Flow::Done;
                    };
                    let Some(c) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    // 按弃牌花色授对应子技实例
                    let suit = ctx.catalog.cards[c as usize].suit;
                    let key = match suit {
                        crate::catalog::Suit::Heart => WUDIAN_GUANGSHU_HEART_KEY,
                        crate::catalog::Suit::Diamond => WUDIAN_GUANGSHU_DIAMOND_KEY,
                        crate::catalog::Suit::Club => WUDIAN_GUANGSHU_CLUB_KEY,
                        crate::catalog::Suit::Spade => WUDIAN_GUANGSHU_SPADE_KEY,
                    };
                    if let Some(def) = ctx.catalog.skill_by_name(key) {
                        ctx.core.skills.insert(
                            (t, def),
                            crate::state::SkillInstance {
                                owner: t,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 0,
                            },
                        );
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianGuangshuHeart => {
                // ♥光书：受伤后回复 1 点体力（Damage 触发臂 filter 已
                // 收窄；一次性移除实例）。
                ctx.recover(self.owner, 1, None);
                if let Some(def) = ctx.catalog.skill_by_name(WUDIAN_GUANGSHU_HEART_KEY) {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            }
            HostBehavior::WudianGuangshuDiamond => {
                // ♦光书：造成伤害后摸 2 张牌（一次性移除实例）。
                ctx.draw_cards(self.owner, 2);
                if let Some(def) = ctx.catalog.skill_by_name(WUDIAN_GUANGSHU_DIAMOND_KEY) {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            }

            HostBehavior::WudianGuangshuClub => {
                // ♣光井：mod 钩在杀门控（下阶段结束移除——PhaseEnd 触发臂）
                let self_def = ctx
                    .catalog
                    .skill_by_name(WUDIAN_GUANGSHU_CLUB_KEY)
                    .expect("光井在目录");
                ctx.core.skills.remove(&(self.owner, self_def));
                Flow::Done
            },
            HostBehavior::WudianGuangshuSpade => {
                // 光塔·♠（ow，规则 v119）：锁定——持有者于下一个阶段结束时受
                // 1 点无来源雷伤并移除本子技。上游 guangshu_spade trigger
                // phaseEnd；效果在 commit_phase 的阶段结束直推（kernel.rs 768
                // 行），宿主无机器步骤（TriggerMeta 仍注册 PhaseEnd 供过滤）。
                Flow::Done
            },
            HostBehavior::WudianZiyu => match self.stage {
                // 自愈（U07 ow，规则 v120）：摸一张或回复 1 点体力（可放
                // 弃；满血时无回复选项——上游 chooseDrawRecover 同口径）。
                HostStage::Act => {
                    let p = ctx.core.player(self.owner);
                    let mut options = vec![ChoiceOption::Tag(0)];
                    if (p.hp as u32) < p.max_hp {
                        options.push(ChoiceOption::Tag(1));
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: WUDIAN_ZIYU_CHOICE,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => match ctx.answer().first() {
                    Some(ChoiceOption::Tag(1)) => {
                        ctx.recover(self.owner, 1, None);
                        Flow::Done
                    }
                    Some(ChoiceOption::Tag(0)) => {
                        ctx.draw_cards(self.owner, 1);
                        Flow::Done
                    }
                    _ => Flow::Done,
                },
                _ => Flow::Done,
            },
            HostBehavior::WudianShouhu => match self.stage {
                // 守护（U07 ow，规则 v121）：Act 弃一张杀 → SelectCard 选
                // 其他受伤角色 → 回复 1 点体力（上游 target.recover 同口
                // 径）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c).is_sha_like())
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_SHOUHU_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(c) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&c)
                        || !ctx.catalog.card_kind(c).is_sha_like()
                    {
                        return Flow::Done;
                    }
                    ctx.discard_from_hand(self.owner, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && (ctx.core.player(*p).hp as u32)
                                    < ctx.core.player(*p).max_hp
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_SHOUHU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive {
                        ctx.recover(t, 1, None);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianDuwen => {
                // 毒吻（ow/黑白合，规则 v122）：锁定——造成伤害时若来源与受
                // 伤害角色手牌数相等，伤害 +1。上游 duwen damageBegin；效果
                // 在 commit_damage（kernel.rs 167 行），宿主无机器步骤。
                Flow::Done
            },
            HostBehavior::WudianPaotai => match self.stage {
                // 炮台（U05 ow，规则 v123）：Act 弃一张杀 → 布置/升级
                //（≤3；层数跨回合持久入 PlayerState）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c).is_sha_like())
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_PAOTAI_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(c) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&c)
                        || !ctx.catalog.card_kind(c).is_sha_like()
                    {
                        return Flow::Done;
                    }
                    ctx.discard_from_hand(self.owner, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    if ctx.core.player(self.owner).paotai_lv < 3 {
                        ctx.core.player_mut(self.owner).paotai_lv += 1;
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianPaotaiFire => {
                // 炮台射击（U05 ow，规则 v123）：End 阶段开始强制触发——
                // 概率 30%×层（RuleRng）→ 随机一名存活敌人 1 点火焰伤害
                //（无敌人则不触发；上游 paotai2 forced 同口径）。
                let lv = ctx.core.player(self.owner).paotai_lv;
                if lv > 0 {
                    let roll = ctx.core.rng.gen_range(100);
                    if roll < 30 * lv as u32 {
                        let cands: Vec<crate::ids::PlayerId> = ctx
                            .core
                            .alive_players()
                            .filter(|p| *p != self.owner)
                            .collect();
                        if !cands.is_empty() {
                            let i =
                                ctx.core.rng.gen_range(cands.len() as u32) as usize;
                            let t = cands[i];
                            ctx.emit(EventData::Damage(
                                crate::machine::damage_data_with(
                                    Some(self.owner),
                                    t,
                                    1,
                                    crate::event::DamageNature::Fire,
                                    None,
                                ),
                            ));
                        }
                    }
                }
                Flow::Done
            },
            HostBehavior::WudianBaoleiSwitch => match self.stage {
                // 堡垒/侦察切换（U05 ow，规则 v124）：三态翻转——0→1
                //（哨卫）、1→2（侦察）、2→1（回哨卫；上游双技各 phaseEnd
                // 确认切换的互斥语义合并为一个确认式翻转）。
                HostStage::Act => {
                    let cur = ctx.core.player(self.owner).baolei_mode;
                    let next = match cur {
                        2 => 1,
                        _ => cur + 1,
                    };
                    ctx.core.player_mut(self.owner).baolei_mode = next;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianZhenchaDraw => match self.stage {
                // 侦察摸回（U05 ow，规则 v125）：摸一张或回复 1 点体力
                //（chooseDrawRecover 可放弃；满血仅摸牌）。
                HostStage::Act => {
                    let p = ctx.core.player(self.owner);
                    let mut options = vec![ChoiceOption::Tag(0)];
                    if (p.hp as u32) < p.max_hp {
                        options.push(ChoiceOption::Tag(1));
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: WUDIAN_ZHENCHA_CHOICE,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => match ctx.answer().first() {
                    Some(ChoiceOption::Tag(1)) => {
                        ctx.recover(self.owner, 1, None);
                        Flow::Done
                    }
                    Some(ChoiceOption::Tag(0)) => {
                        ctx.draw_cards(self.owner, 1);
                        Flow::Done
                    }
                    _ => Flow::Done,
                },
                _ => Flow::Done,
            },
            HostBehavior::WudianYuedong => match self.stage {
                // 乐动（U05 ow 卢西奥，规则 v126）：出牌阶段结束时选至多
                // 1+扩音增量名目标各摸 1+扩音量张；呼唤后改为各回复等量
                //（回复模式目标须未满体力；上游 direct + chooseTarget
                // [1,num] 可整体放弃——空选项直接跳过）。
                HostStage::Act => {
                    let num = 1 + ctx.core.turn.yuedong_num as usize;
                    let recover = ctx.core.turn.yuedong_recover;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            !recover
                                || (ctx.core.player(*p).hp as u32)
                                    < ctx.core.player(*p).max_hp
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_YUEDONG_TARGETS,
                            options,
                            min: 1,
                            max: num,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let targets: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if targets.is_empty() {
                        return Flow::Done; // 整体放弃
                    }
                    let eff = 1 + ctx.core.turn.yuedong_eff as usize;
                    if ctx.core.turn.yuedong_recover {
                        // 上游按座位序逐个回复（sort seat；原生座位序同构）
                        let mut sorted = targets.clone();
                        sorted.sort_by_key(|p| ctx.core.seat_distance(ctx.core.turn.current, *p));
                        for p in sorted {
                            ctx.recover(p, eff as u32, None);
                        }
                    } else {
                        for p in targets {
                            ctx.draw_cards(p, eff);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianKuoyin => match self.stage {
                // 扩音（U05 ow 卢西奥，规则 v126）：弃 1 张→乐动目标数
                // +2；弃 2 张→乐动摸/回复量 +1。选择张数由存量决定：
                // 量已 +1→恰 1 张；数已 +2→恰 2 张；否则 1–2 张（上游
                // selectCard 同口径；he 区域=手牌+装备）。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let mut options: Vec<ChoiceOption> =
                        p.hand.iter().copied().map(ChoiceOption::Card).collect();
                    options.extend(p.equips.iter().copied().map(ChoiceOption::Card));
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let (min, max) = if ctx.core.turn.yuedong_eff > 0 {
                        (1, 1)
                    } else if ctx.core.turn.yuedong_num > 0 {
                        (2, 2)
                    } else {
                        (1, 2)
                    };
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_KUOYIN_CARDS,
                            options,
                            min,
                            max,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let sel: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if sel.is_empty() {
                        return Flow::Done; // 弃权
                    }
                    for c in &sel {
                        ctx.take_owned_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: sel.clone(),
                    });
                    if sel.len() == 1 {
                        ctx.core.turn.yuedong_num = ctx.core.turn.yuedong_num.saturating_add(2);
                    } else {
                        ctx.core.turn.yuedong_eff = ctx.core.turn.yuedong_eff.saturating_add(1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianHuhuan => match self.stage {
                // 呼唤（U05 ow 卢西奥，规则 v126）：弃 2 张牌，本回合乐动
                // 改为回复模式（限一次——recover 已置位时 activate 拒绝）。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let mut options: Vec<ChoiceOption> =
                        p.hand.iter().copied().map(ChoiceOption::Card).collect();
                    options.extend(p.equips.iter().copied().map(ChoiceOption::Card));
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_HUHUAN_CARDS,
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let sel: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if sel.len() < 2 {
                        return Flow::Done; // 弃权
                    }
                    for c in &sel {
                        ctx.take_owned_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: sel.clone(),
                    });
                    ctx.core.turn.yuedong_recover = true;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianJidong => match self.stage {
                // 急冻（U05 ow 美，规则 v127）：翻面+回复 2+冰封。登记
                // 差异：翻面内联（风雪刃 v95 先例，不经 turnOverBefore
                // 窗口）；冰封解除=翻回时点（commit_turn_over_before 翻
                // 至正面清位，上游 turnOverAfter 到期同口径）。
                HostStage::Act => {
                    let flipped = {
                        let p = ctx.core.player_mut(self.owner);
                        p.face_down = !p.face_down;
                        p.face_down
                    };
                    ctx.emit(EventData::TurnOvered {
                        player: self.owner,
                        face_down: flipped,
                    });
                    ctx.recover(self.owner, 2, None);
                    if flipped {
                        ctx.core.player_mut(self.owner).jidong_active = true;
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianBaoxue => match self.stage {
                // 暴雪（U05 ow 美，规则 v127）：选至多「黑色 he 牌数」
                // 名其他角色（可整体放弃=limited 未消耗）→ 弃全部黑色
                // he 牌 → 逐目标其 he 区随机弃 1 张+翻至背面 → 自己翻
                // 至背面（turnOver(true) 幂等同口径）。登记差异：
                // showHandcards 表现层不建模；翻面内联无窗口。
                HostStage::SelectCard => {
                    let black = {
                        let p = ctx.core.player(self.owner);
                        p.hand
                            .iter()
                            .chain(p.equips.iter())
                            .filter(|c| {
                                matches!(
                                    ctx.catalog.cards[**c as usize].suit,
                                    crate::catalog::Suit::Spade
                                        | crate::catalog::Suit::Club
                                )
                            })
                            .count()
                    };
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() || black == 0 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_BAOXUE_TARGETS,
                            options,
                            min: 1,
                            max: black,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let targets: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if targets.is_empty() {
                        return Flow::Done; // 放弃：limited 未消耗
                    }
                    ctx.core.player_mut(self.owner).baoxue_used = true;
                    // contentBefore：弃全部黑色 he 牌
                    let black_cards: Vec<CardId> = {
                        let p = ctx.core.player(self.owner);
                        p.hand
                            .iter()
                            .copied()
                            .chain(p.equips.iter().copied())
                            .filter(|c| {
                                matches!(
                                    ctx.catalog.cards[*c as usize].suit,
                                    crate::catalog::Suit::Spade
                                        | crate::catalog::Suit::Club
                                )
                            })
                            .collect()
                    };
                    for c in &black_cards {
                        ctx.take_owned_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: black_cards.clone(),
                    });
                    // content：逐目标 he 区随机弃 1 张+翻至背面
                    for t in targets {
                        let pool: Vec<CardId> = {
                            let p = ctx.core.player(t);
                            p.hand
                                .iter()
                                .copied()
                                .chain(p.equips.iter().copied())
                                .collect()
                        };
                        if !pool.is_empty() {
                            let j = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                            let c = pool[j];
                            ctx.take_owned_card(t, c);
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: vec![c],
                            });
                        }
                        let flipped = {
                            let p = ctx.core.player_mut(t);
                            let f = !p.face_down;
                            p.face_down = true;
                            f
                        };
                        if flipped {
                            ctx.emit(EventData::TurnOvered {
                                player: t,
                                face_down: true,
                            });
                        }
                    }
                    // contentAfter：自己翻至背面（幂等）
                    let flipped = {
                        let p = ctx.core.player_mut(self.owner);
                        let f = !p.face_down;
                        p.face_down = true;
                        f
                    };
                    if flipped {
                        ctx.emit(EventData::TurnOvered {
                            player: self.owner,
                            face_down: true,
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianLiangou => match self.stage {
                // 链钩（U05 ow 路霸，规则 v128）：弃 1 张 he（必选）→
                // 选目标（必选）→ 弃置+判定（非红桃成功）→ 挂钩
                // TurnState.liangou_hook（本回合内：距离恒 1+其首次受
                // 伤 +1；换回合随 TurnState 清零解除）。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let mut options: Vec<ChoiceOption> =
                        p.hand.iter().copied().map(ChoiceOption::Card).collect();
                    options.extend(p.equips.iter().copied().map(ChoiceOption::Card));
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_LIANGOU_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    // 收「牌选择」应答：存入 self.cards，前移 Act 问目标
                    let cards: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if cards.len() != 1 {
                        return Flow::Done;
                    }
                    self.cards = cards;
                    self.stage = HostStage::Act;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_LIANGOU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    // 收「目标选择」应答：弃牌+判定+挂钩
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let sel = std::mem::take(&mut self.cards);
                    for c in &sel {
                        ctx.take_owned_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: sel,
                    });
                    // 判定：非红桃成功（上游 judge suit!=="heart"）
                    let judged = ctx.draw_judge_card();
                    let suit = ctx.catalog.cards[judged as usize].suit;
                    ctx.log(LogEvent::Judged {
                        player: self.owner,
                        card: judged,
                    });
                    ctx.to_discard(judged);
                    if suit != crate::catalog::Suit::Heart {
                        ctx.core.turn.liangou_hook = Some(t);
                        ctx.core.turn.liangou_bonus_used = false;
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianXiyang => match self.stage {
                // 吸阳（U05 ow 路霸，规则 v128）：确认后翻面+回复 2。
                // 登记差异：翻面内联（风雪刃 v95 先例，不经窗口——与
                // 急冻同口径）。
                HostStage::Act => {
                    let flipped = {
                        let p = ctx.core.player_mut(self.owner);
                        p.face_down = !p.face_down;
                        p.face_down
                    };
                    ctx.emit(EventData::TurnOvered {
                        player: self.owner,
                        face_down: flipped,
                    });
                    ctx.recover(self.owner, 2, None);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianFeiren => {
                // 飞刃（ow/源氏，规则 v129）：锁定——黑桃杀伤害 +1；杀无视
                // 距离；梅花杀可多指定一个目标。上游 feiren damageBegin 与
                // mod.targetInRange/selectTarget；效果在 commit_damage
                // （kernel.rs 121 行）与出杀目标数/距离校验（engine.rs 2377、
                // 2447 行），宿主无机器步骤。
                Flow::Done
            },
            HostBehavior::WudianFeidan => match self.stage {
                // 飞弹（U05 ow 法老之鹰，规则 v130）：Act 建弹射目标与
                // 弃牌选项→SelectCard 读弃牌应答（取消=不弃不弹）→按
                // 座位序逐目标各受 1 点无牌伤害（source=持有者；上游
                // damage() 无参同口径）。
                HostStage::Act => {
                    // 受伤者从触发事件带入 target 字段（machine() 分流）
                    let Some(hurt) = self.target else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(hurt).alive {
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != hurt
                                && ctx.core.computed_distance(ctx.catalog, hurt, *p) <= 1
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.targets = options
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    let he: Vec<ChoiceOption> = {
                        let p = ctx.core.player(self.owner);
                        p.hand
                            .iter()
                            .copied()
                            .chain(p.equips.iter().copied())
                            .map(ChoiceOption::Card)
                            .collect()
                    };
                    if he.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_FEIDAN_DISCARD,
                            options: he,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let disc = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let Some(c) = disc else {
                        return Flow::Done; // 放弃：不弃牌不弹射
                    };
                    ctx.take_owned_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 按座位序逐目标伤害（上游 sort seat + shift damage）
                    let hurt = self.target.unwrap_or(self.owner);
                    let mut sorted = std::mem::take(&mut self.targets);
                    sorted.sort_by_key(|p| ctx.core.seat_distance(hurt, *p));
                    for t in sorted {
                        if ctx.core.player(t).alive {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                1,
                                crate::event::DamageNature::Normal,
                                None,
                            )));
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianZhiyuan => match self.stage {
                // 支援（U05 ow 安娜，规则 v131）：防止此伤害，受伤者回
                // 复等量体力（上游 trigger.cancel()+recover(num) 同口
                // 径；确认门由触发路径 OptionalGate 承担）。
                HostStage::Act => {
                    let (num, hurt) = {
                        let d = ctx.current_damage_mut();
                        d.prevented = true;
                        (d.amount, d.target)
                    };
                    if num > 0 && ctx.core.player(hurt).alive {
                        ctx.recover(hurt, num, None);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianShenqiang => match self.stage {
                // 神枪（U05 ow 麦克雷，规则 v132）：杀使用计数回退——
                // 本回合可再出一张杀（上游 getStat().card.sha-- 同口径；
                // 地图效果加成杀的 land_sha_used 不回退，登记）。
                HostStage::Act => {
                    ctx.core.turn.sha_used_play_phase = false;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianShihuo => match self.stage {
                // 拾火（U05 ow 狂鼠，规则 v133）：摸一张牌（每次火伤各
                // 触发；铁索传导的独立火伤事件照常）。
                HostStage::Act => {
                    ctx.draw_cards(self.owner, 1);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianMengji => match self.stage {
                // 猛击（U05 ow 莱因哈特，规则 v134）：伤害 +1（前置窗口
                // 直写帧量——伊籍改目标同机制；上游 trigger.num++ 同口径）。
                HostStage::Act => {
                    ctx.current_damage_mut().amount += 1;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianYinbo => match self.stage {
                // 音波（U05 ow 半藏，规则 v135）：弃 1 张黑桃→随机 ≤3
                // 名有牌的其他角色各随机弃 1 张（上游 randomGets(3)+
                // sort seat+randomGet 同口径；登记差异：原型无阵营——
                // enemies=其他存活玩家）。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let options: Vec<ChoiceOption> = p
                        .hand
                        .iter()
                        .copied()
                        .chain(p.equips.iter().copied())
                        .filter(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Spade
                            )
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_YINBO_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    let Some(c) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    ctx.take_owned_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 候选=其他存活且 he 非空；随机取 ≤3（龙帆 swap_remove
                    // 抽取式）
                    let mut cands: Vec<PlayerId> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && (!ctx.core.player(*p).hand.is_empty()
                                    || !ctx.core.player(*p).equips.is_empty())
                        })
                        .collect();
                    let take = cands.len().min(3);
                    for _ in 0..take {
                        let j = ctx.core.rng.gen_range(cands.len() as u32) as usize;
                        let t = cands.swap_remove(j);
                        // he 池随机弃 1（暴雪同口径：手牌+装备）
                        let pool: Vec<CardId> = {
                            let p = ctx.core.player(t);
                            p.hand
                                .iter()
                                .copied()
                                .chain(p.equips.iter().copied())
                                .collect()
                        };
                        let k = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let dc = pool[k];
                        ctx.take_owned_card(t, dc);
                        ctx.to_discard(dc);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![dc],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianQianggu => match self.stage {
                // 强固（U05 ow 奥丽莎，规则 v136）：弃 2 张 he（必选）→
                // +2 护甲+强固置位（自己下回合判定阶段开始解除）。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let mut options: Vec<ChoiceOption> =
                        p.hand.iter().copied().map(ChoiceOption::Card).collect();
                    options.extend(p.equips.iter().copied().map(ChoiceOption::Card));
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_QIANGGU_CARDS,
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    let sel: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if sel.len() < 2 {
                        return Flow::Done;
                    }
                    for c in &sel {
                        ctx.take_owned_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: sel,
                    });
                    ctx.change_hujia(self.owner, 2);
                    ctx.core.player_mut(self.owner).qianggu_active = true;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianQiangguGuard => match self.stage {
                // 强固守卫（U05 ow 奥丽莎，规则 v136）：Act 问使用者——
                // 选一张基本牌=弃置放行；Pass/无基本牌=杀无效（帧
                // cancelled，Intrinsic 按 cancelled 跳过结算——筑城守卫
                // 同款）。
                HostStage::Act => {
                    let Some(attacker) = self.target else {
                        return Flow::Done;
                    };
                    let mut options = vec![ChoiceOption::Pass];
                    options.extend(
                        ctx.core
                            .player(attacker)
                            .hand
                            .iter()
                            .copied()
                            .filter(|c| {
                                ctx.catalog.card_kind(*c).aoe_need().is_none()
                                    && matches!(
                                        ctx.catalog.card_kind(*c),
                                        CardKind::Sha
                                            | CardKind::Shan
                                            | CardKind::Tao
                                            | CardKind::Jiu
                                    )
                            })
                            .map(ChoiceOption::Card),
                    );
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: attacker,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_QIANGGU_GUARD_ASK,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    match ctx.answer().first().copied() {
                        Some(ChoiceOption::Card(c)) => {
                            ctx.discard_from_hand(self.target.expect("attacker"), c);
                        }
                        _ => {
                            // 未弃基本牌：杀对持有者无效
                            ctx.current_card_use_mut().cancelled = true;
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianFeitiao => match self.stage {
                // 跳跃（U05 ow 温斯顿，规则 v137）：弃 1 张 he（必选）→
                // 选一名其他角色（必选）→ 弃置+其随机弃 1+置跳跃目标
                //（本回合自己算到其距离恒 1；换回合解除）。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let options: Vec<ChoiceOption> = p
                        .hand
                        .iter()
                        .copied()
                        .chain(p.equips.iter().copied())
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_FEITIAO_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    // 收「牌选择」应答：存 cards 后前移 Act 问目标
                    let cards: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if cards.len() != 1 {
                        return Flow::Done;
                    }
                    self.cards = cards;
                    self.stage = HostStage::Act;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_FEITIAO_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    // 收「目标选择」应答：结算
                    let Some(t) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let sel = std::mem::take(&mut self.cards);
                    for c in &sel {
                        ctx.take_owned_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: sel,
                    });
                    // 目标随机弃 1（he 池）
                    let pool: Vec<CardId> = {
                        let p = ctx.core.player(t);
                        p.hand
                            .iter()
                            .copied()
                            .chain(p.equips.iter().copied())
                            .collect()
                    };
                    if !pool.is_empty() {
                        let k = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let dc = pool[k];
                        ctx.take_owned_card(t, dc);
                        ctx.to_discard(dc);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![dc],
                        });
                    }
                    ctx.core.turn.feitiao_target = Some(t);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianPingzhang => match self.stage {
                // 屏障（U05 ow 查莉娅，规则 v138）：Act 建对应花色弃牌选
                // 项（自己受伤=红桃/他人受伤=黑桃；可取消=不发动）→
                // SelectCard 弃置+伤害 -1+置对应标志（各每回合一次）。
                HostStage::Act => {
                    let hurt = self.target.unwrap_or(self.owner);
                    let self_case = hurt == self.owner;
                    let used = if self_case {
                        ctx.core.player(self.owner).pingzhang_self_used
                    } else {
                        ctx.core.player(self.owner).pingzhang_other_used
                    };
                    if used || !ctx.core.player(hurt).alive {
                        return Flow::Done;
                    }
                    let want = if self_case {
                        crate::catalog::Suit::Heart
                    } else {
                        crate::catalog::Suit::Spade
                    };
                    let options: Vec<ChoiceOption> = {
                        let p = ctx.core.player(self.owner);
                        p.hand
                            .iter()
                            .copied()
                            .chain(p.equips.iter().copied())
                            .filter(|c| ctx.catalog.cards[*c as usize].suit == want)
                            .map(ChoiceOption::Card)
                            .collect()
                    };
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.card = Some(crate::ids::CardId::from(self_case as u8));
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_PINGZHANG_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(c) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done; // 放弃：不发动
                    };
                    ctx.take_owned_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 伤害 -1（前置窗口直写帧量；上游 trigger.num--）
                    let d = ctx.current_damage_mut();
                    d.amount = d.amount.saturating_sub(1);
                    let self_case = self.target == Some(self.owner);
                    if self_case {
                        ctx.core.player_mut(self.owner).pingzhang_self_used = true;
                    } else {
                        ctx.core.player_mut(self.owner).pingzhang_other_used = true;
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianOwliyong => match self.stage {
                // 应用（U05 ow 查莉娅，规则 v138）：摸牌阶段额外摸上回合
                // 屏障次数张。
                HostStage::Act => {
                    let n = ctx.core.player(self.owner).pingzhang_count;
                    if n > 0 {
                        ctx.draw_cards(self.owner, n as usize);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianYinshen => match self.stage {
                // 隐匿（U05 ow 黑影，规则 v139）：置潜行（不能成为其他角
                // 色的卡牌目标；自己回合判定阶段开始解除）。
                HostStage::Act => {
                    ctx.core.player_mut(self.owner).yinshen_active = true;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianXiandan => {
                // 霰弹（ow/死神，规则 v140）：出杀时可弃一张 he——红牌使该杀
                // 不可闪避，黑牌使该杀伤害 +1。上游 shaBegin direct 与
                // xiandan2 damageBegin；效果在 ShaResolve 内联询问（kernel.rs
                // 3152 行）与黑牌加伤折算（kernel.rs 2222 行），宿主无机器
                // 步骤。
                Flow::Done
            },
            HostBehavior::WudianJijia => {
                // 机甲（ow/D.Va，规则 v141）：初始 4 点机甲血，受伤 1:1 抵扣；
                // 机甲血 >0 时手牌上限 +4。上游 jijia changeHp 拦截与
                // mod.maxHandcard；效果在 commit_damage（kernel.rs 197 行）与
                // 弃牌阶段手牌上限（kernel.rs 2006 行），宿主无机器步骤。
                Flow::Done
            },
            HostBehavior::XianjianLingxue => match self.stage {
                // 灵雪（U05 xianjian 星璇，规则 v142）：+1 护甲（change_
                // hujia 原语；上游 changeHujia() 同口径）。
                HostStage::Act => {
                    ctx.change_hujia(self.owner, 1);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianBinxin => match self.stage {
                // 冰心（U05 xianjian 沈欺霜，规则 v143）：体力 1 的阶段
                // 结束者 +1 护甲（change_hujia 原语；上游 changeHujia()
                // 同口径；事件者带入 target）。
                HostStage::Act => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive && ctx.core.player(t).hp == 1 {
                        ctx.change_hujia(t, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianHuimeng => match self.stage {
                // 回梦（U05 xianjian 赵灵儿，规则 v144）：摸两张牌。
                HostStage::Act => {
                    ctx.draw_cards(self.owner, 2);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianTianshe => match self.stage {
                // 蛇蜕（U05 xianjian 赵灵儿，规则 v144）：属性伤害无效
                //（prevented 直写=碎盾/急冻先例）。
                HostStage::Act => {
                    ctx.current_damage_mut().prevented = true;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianTianshe2 => match self.stage {
                // 蛇蜕·息（U05 xianjian 赵灵儿，规则 v144）：回复 1 点体
                // 力（满血饱和；filter 已收窄已受伤）。
                HostStage::Act => {
                    ctx.recover(self.owner, 1, None);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianFanling => match self.stage {
                // 返灵（U05 xianjian 瑕，规则 v145）：回复 1 点体力+置
                // 本回合已发动（换回合复位）。
                HostStage::Act => {
                    ctx.core.turn.fanling_used = true;
                    ctx.recover(self.owner, 1, None);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianLongxiang => {
                // 龙翔（xianjian/温慧，规则 v146）：你使用杀时可令目标随机弃至
                // 与你手牌数相同。上游 shaBegin（有 check，非强制）；原生在
                // ShaResolve 内联、不询问（v146 已登记口径，kernel.rs 3047
                // 行），宿主无机器步骤。
                Flow::Done
            },
            HostBehavior::XianjianGuiyuan => match self.stage {
                // 归元（U05 xianjian 林月如，规则 v147）：弃 1 张杀（必
                // 选）→回复 1 点体力+摸一张牌（已受伤 filter 收窄）。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let options: Vec<ChoiceOption> = p
                        .hand
                        .iter()
                        .copied()
                        .chain(p.equips.iter().copied())
                        .filter(|c| ctx.catalog.card_kind(*c).is_sha_like())
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() || (ctx.core.player(self.owner).hp as u32)
                        >= ctx.core.player(self.owner).max_hp
                    {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_GUIYUAN_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    let Some(c) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    ctx.take_owned_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    ctx.recover(self.owner, 1, None);
                    ctx.draw_cards(self.owner, 1);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianTannang => {
                // 探囊（xianjian/韩菱纱，规则 v148）：每回合限一次，梅花手牌当
                // 顺手牵羊使用；此顺手牵羊无距离限制。上游 viewAs 与
                // mod.targetInRange；效果在出牌转化（engine.rs 2165 行）与距离
                // 豁免（engine.rs 2623 行），usable 走命令层 once_used，宿主无
                // 机器步骤。
                Flow::Done
            },
            HostBehavior::XianjianFeixu => match self.stage {
                // 废墟（U05 xianjian 王蓬絮，规则 v149）：使用者/打出者
                // 摸一张牌（事件者带入 target）。
                HostStage::Act => {
                    if let Some(t) = self.target {
                        if ctx.core.player(t).alive {
                            ctx.draw_cards(t, 1);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianFeng => match self.stage {
                // 风（U05 xianjian 雷元戈，规则 v150）：计数累计至 2 后
                // 本次额外摸 1 并清零（上游 drawBegin num++ 的 after 补
                // 摸等价，登记差异）。
                HostStage::Act => {
                    let c = ctx.core.player(self.owner).feng_count;
                    if c < 2 {
                        ctx.core.player_mut(self.owner).feng_count = c + 1;
                    } else {
                        ctx.core.player_mut(self.owner).feng_count = 0;
                        ctx.draw_cards(self.owner, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianYa => match self.stage {
                // 雅（U05 xianjian 雷元戈，规则 v150）：计数累计至 2 后
                // 本次受伤 -1 并清零（前置窗口直写帧量=猛击同机制）。
                HostStage::Act => {
                    let c = ctx.core.player(self.owner).ya_count;
                    if c < 2 {
                        ctx.core.player_mut(self.owner).ya_count = c + 1;
                    } else {
                        ctx.core.player_mut(self.owner).ya_count = 0;
                        let d = ctx.current_damage_mut();
                        d.amount = d.amount.saturating_sub(1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianSong => match self.stage {
                // 颂（U05 xianjian 雷元戈，规则 v150）：计数累计至 2 后
                // 本次伤害 +1 并清零（前置窗口直写帧量）。
                HostStage::Act => {
                    let c = ctx.core.player(self.owner).song_count;
                    if c < 2 {
                        ctx.core.player_mut(self.owner).song_count = c + 1;
                    } else {
                        ctx.core.player_mut(self.owner).song_count = 0;
                        ctx.current_damage_mut().amount += 1;
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXjyufeng => match self.stage {
                // 御风（U05 xianjian 李逍遥，规则 v151）：摸（2-手牌数）
                // 张+置本回合已发动次数。
                HostStage::Act => {
                    let used = ctx.core.turn.xjyufeng_used;
                    ctx.core.turn.xjyufeng_used = used + 1;
                    let hand = ctx.core.player(self.owner).hand.len();
                    if hand < 2 {
                        ctx.draw_cards(self.owner, 2 - hand);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXuanyan2 => match self.stage {
                // 玄炎·燃（U05 xianjian 玄霄，规则 v152）：火伤 +1（前置
                // 窗口直写帧量=猛击同机制）。
                HostStage::Act => {
                    ctx.current_damage_mut().amount += 1;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXuanyan3 => match self.stage {
                // 玄炎·烬（U05 xianjian 玄霄，规则 v152）：失去 1 点体力
                //（LoseHp 帧独立事件=返灵同位）。
                HostStage::Act => {
                    let p = ctx.core.player(self.owner);
                    if p.alive {
                        let hp = p.hp;
                        let _ = hp;
                        let amount = 1u32;
                        // LoseHp 事件（经 commit_lose_hp；可触发返灵等）
                        ctx.emit(EventData::LoseHp {
                            player: self.owner,
                            amount,
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianNingbin => match self.stage {
                // 凝冰（U05 xianjian 玄霄，规则 v152）：回复 1 点体力（满
                // 血饱和）。
                HostStage::Act => {
                    ctx.recover(self.owner, 1, None);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXuanyan => {
                // 玄炎（xianjian/玄霄，规则 v152）：锁定——你造成火属性伤害时
                // +1；受到雷属性伤害后回复 1。上游仅 group:[xuanyan2,
                // xuanyan3]；子技效果在 XianjianXuanyan2/3（同文件 10930、
                // 10939 行），本壳分支无机器步骤。
                Flow::Done
            },
            HostBehavior::XianjianDiewu => match self.stage {
                // 蝶舞（U05 xianjian 龙葵，规则 v153）：选 1 张杀+1 目标
                // →直接转移（discard:false give 同口径）→本回合首次摸 1。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let options: Vec<ChoiceOption> = p
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c).is_sha_like())
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_DIEWU_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    // 收「牌选择」应答
                    if self.cards.is_empty() {
                        let cards: Vec<CardId> = ctx
                            .answer()
                            .iter()
                            .filter_map(|o| match o {
                                ChoiceOption::Card(c) => Some(*c),
                                _ => None,
                            })
                            .collect();
                        if cards.len() != 1 {
                            return Flow::Done;
                        }
                        self.cards = cards;
                        let options: Vec<ChoiceOption> = ctx
                            .core
                            .alive_players()
                            .filter(|p| *p != self.owner)
                            .map(ChoiceOption::Player)
                            .collect();
                        if options.is_empty() {
                            return Flow::Done;
                        }
                        Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectPlayers,
                                prompt: WUDIAN_XJ_DIEWU_TARGET,
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        }
                    } else {
                        // 收「目标选择」应答：转移+首次摸 1
                        let Some(t) = ctx.answer().first().and_then(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        }) else {
                            return Flow::Done;
                        };
                        let sel = std::mem::take(&mut self.cards);
                        for c in &sel {
                            ctx.give_hand_card(self.owner, t, *c);
                        }
                        if !ctx.core.turn.diewu_draw_used {
                            ctx.core.turn.diewu_draw_used = true;
                            ctx.draw_cards(self.owner, 1);
                        }
                        Flow::Done
                    }
                }
                _ => Flow::Done,
            },
HostBehavior::XianjianLingyu => match self.stage {
                // 灵愈（U05 xianjian 龙葵，规则 v153）：令一名其他已受伤
                // 角色回复 1 点体力（可放弃=空目标）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && (ctx.core.player(*p).hp as u32)
                                    < ctx.core.player(*p).max_hp
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_LINGYU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first() {
                        let t = *t;
                        if ctx.core.player(t).alive && (ctx.core.player(t).hp as u32) < ctx.core.player(t).max_hp {
                            ctx.recover(t, 1, None);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianLueying => {
                // 掠影（xianjian/慕长兰，规则 v154）：你使用杀时获得目标 he 随机
                // 一张；若另有其他有牌角色，目标另选一人供你随机弃其一张。
                // 上游 shaBegin usable:1（可选发动）；效果在 ShaResolve 内联
                // （kernel.rs 3087 行），宿主无机器步骤。差异：上游要求另有
                // 其他有牌角色且可选发动，原生自动获得一张——登记见 T11。
                Flow::Done
            },
            HostBehavior::XianjianAnwugu => match self.stage {
                // 蛊舞（U05 xianjian 阿奴，规则 v154）：受害者获蛊 3 阶段
                //（上游 addSkill anwugu2+init storage=3 同口径）。
                HostStage::Act => {
                    if let Some(t) = self.target {
                        if ctx.core.player(t).alive {
                            ctx.core.player_mut(t).anwugu_count = 3;
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianAnwugu2 => match self.stage {
                // 蛊舞·缚（无机器——三钩直读：PlayCard 出牌上限/弃牌阶段
                // limit/自己阶段结束递减）。
                HostStage::Act => Flow::Done,
                _ => Flow::Done,
            },
            HostBehavior::XianjianTxianqu => match self.stage {
                // 甜香去（U05 xianjian 唐雨柔，规则 v156）：防止伤害（安
                // 娜支援同款 prevented 直写）+摸 2+回复 1+置本回合已发动。
                HostStage::Act => {
                    ctx.current_damage_mut().prevented = true;
                    ctx.core.turn.txianqu_used = true;
                    ctx.draw_cards(self.owner, 2);
                    ctx.recover(self.owner, 1, None);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianFenshi => match self.stage {
                // 焚逝（U05 xianjian 姜城，规则 v157）：觉醒——+2 护甲、
                // 摸两张、获得龙火（上游 changeHujia(2)/draw(2)/
                // addSkill("longhuo") 同口径；置位兼一次性门控）。
                HostStage::Act => {
                    ctx.core.player_mut(self.owner).jiangcheng_longhuo = true;
                    ctx.change_hujia(self.owner, 2);
                    ctx.draw_cards(self.owner, 2);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianLonghuo => loop {
                // 龙火（U05 xianjian 姜城，规则 v157）：座位序对所有存活
                // 角色各 1 点火伤（含自己；上游 redo 逐目标结算——逐目标
                // Yield 后重查存活，传导致死者不再向后结算）。target 兼
                // 作装填标记（None=未装填）。
                if self.target.is_none() {
                    self.target = Some(self.owner);
                    self.targets = (0..ctx.core.players.len() as u8).collect();
                }
                let Some(t) = self.targets.first().copied() else {
                    return Flow::Done;
                };
                self.targets.remove(0);
                if !ctx.core.player(t).alive {
                    continue;
                }
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(self.owner),
                    t,
                    1,
                    crate::event::DamageNature::Fire,
                    None,
                )));
                return Flow::Yield;
            },
            HostBehavior::XianjianSajin => match self.stage {
                // 洒金（U05 xianjian 景天，规则 v158）：弃 1 张手牌（必
                // 选）→ 指定任意名已受伤角色（上游 selectTarget
                // [1,Infinity] 同口径）→ 逐目标判定（Yield 后重查），
                // 判定颜色与所弃牌相同者回复 1 点体力。每臂只收一个应答：
                // SelectCard 发问、TargetChoice 收卡+发问、Act 收目标
                //（self.card 兼首入标记）+弃牌+纯循环。
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_SAJIN_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first() {
                        self.card = Some(*c);
                    }
                    let Some(c) = self.card else {
                        return Flow::Done;
                    };
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|t| {
                            (ctx.core.player(*t).hp as u32)
                                < ctx.core.player(*t).max_hp
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        // 无可指定目标：牌照弃（已进入技能，上游同弃）
                        ctx.take_owned_card(self.owner, c);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                        return Flow::Done;
                    }
                    let max = options.len();
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_SAJIN_TARGETS,
                            options,
                            min: 1,
                            max,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    // 首入（self.card 尚存）：收目标应答并弃牌（弃牌色
                    // 弃后仍可读目录）；后续 Yield 重入只走逐目标循环
                    if let Some(c) = self.card.take() {
                        let picked: Vec<PlayerId> = ctx
                            .answer()
                            .iter()
                            .filter_map(|o| match o {
                                ChoiceOption::Player(p) => Some(*p),
                                _ => None,
                            })
                            .collect();
                        ctx.take_owned_card(self.owner, c);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                        self.targets = picked;
                        self.cards = vec![c];
                    }
                    // 逐目标：判定→同色回复（Yield 后重查存活）
                    let Some(t) = self.targets.first().copied() else {
                        return Flow::Done;
                    };
                    self.targets.remove(0);
                    if !ctx.core.player(t).alive {
                        return Flow::Yield;
                    }
                    let judged = ctx.draw_judge_card();
                    let suit = ctx.catalog.cards[judged as usize].suit;
                    ctx.log(LogEvent::Judged { player: t, card: judged });
                    ctx.to_discard(judged);
                    if let Some(c) = self.cards.first().copied() {
                        let is_red = |su: crate::catalog::Suit| {
                            matches!(
                                su,
                                crate::catalog::Suit::Heart
                                    | crate::catalog::Suit::Diamond
                            )
                        };
                        let disc_suit = ctx.catalog.cards[c as usize].suit;
                        if is_red(suit) == is_red(disc_suit)
                            && ctx.core.player(t).alive
                        {
                            ctx.recover(t, 1, None);
                        }
                    }
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianJtjubao => {
                // 聚宝（U05 xianjian 景天，规则 v158）：随机回收一张非基
                // 本弃牌（候选=filter 已验；取牌此刻离弃堆，羊皮卷同款）
                // +置本回合已回收（上游 jtjubao2 临时技等价）。
                ctx.core.turn.jtjubao_used = true;
                let cands: Vec<CardId> = self
                    .cards
                    .iter()
                    .copied()
                    .filter(|c| {
                        yangpijuan_class(ctx.catalog.card_kind(*c)) != Some(0)
                            && ctx.core.discard.contains(c)
                    })
                    .collect();
                if !cands.is_empty() {
                    let i = ctx.core.rng.gen_range(cands.len() as u32) as usize;
                    let picked = cands[i];
                    let Some(pos) =
                        ctx.core.discard.iter().position(|c| *c == picked)
                    else {
                        return Flow::Done;
                    };
                    ctx.core.discard.remove(pos);
                    ctx.core.player_mut(self.owner).hand.push(picked);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: vec![picked],
                    });
                }
                Flow::Done
            },
            HostBehavior::XianjianYuexing => match self.stage {
                // 月行（U05 xianjian 龙幽，规则 v159）：指定一名其他角色
                // → 双向互指距离基数交换，自己出牌阶段结束解除（commit_
                // phase Play 尾钩；computed_distance 座位基数代换）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "wudian_xj_yuexing_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first() {
                        let t = *t;
                        if t != self.owner && ctx.core.player(t).alive {
                            ctx.core.player_mut(self.owner).yuexing_pair = Some(t);
                            ctx.core.player_mut(t).yuexing_pair = Some(self.owner);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
                        HostBehavior::XianjianMinsha => match self.stage {
                // 敏杀（U05 xianjian 龙幽，规则 v159）：弃两张 he 牌（必
                // 选）→ 指定体力>1 的其他角色受 1 雷伤 → 距其 1 内其他
                // 角色（除自己）按座位序各随机弃一张（手牌优先退随机装
                // 备，蓝格怪衣同款登记差异；逐目标 Yield 重查存活）。
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_MINSHA_CARDS,
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.len() < 2 {
                        return Flow::Done;
                    }
                    // 此刻离区弃置（SelectCards 应答已校验归属）
                    for c in &picked {
                        ctx.take_owned_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: picked,
                    });
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|t| {
                            *t != self.owner && ctx.core.player(*t).hp > 1
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let max = options.len();
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_MINSHA_TARGET,
                            options,
                            min: 1,
                            max,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    // 首入：收目标应答
                    if self.target.is_none() {
                        self.target = Some(self.owner);
                        let picked: Vec<PlayerId> = ctx
                            .answer()
                            .iter()
                            .filter_map(|o| match o {
                                ChoiceOption::Player(p) => Some(*p),
                                _ => None,
                            })
                            .collect();
                        self.targets = picked;
                    } else if let Some(t) = self.card.map(|x| x as PlayerId) {
                        // 阶段 B：上一主目标已结算——其距 1 内随机弃队列
                        self.card = None;
                        let neighbors: Vec<PlayerId> = ctx
                            .core
                            .seat_order_from(t)
                            .into_iter()
                            .filter(|p| {
                                *p != t
                                    && *p != self.owner
                                    && ctx.core.player(*p).alive
                                    && ctx.core.computed_distance(ctx.catalog, t, *p) <= 1
                            })
                            .collect();
                        for x in neighbors {
                            if !ctx.core.player(x).alive {
                                continue;
                            }
                            if !ctx.core.player(x).hand.is_empty() {
                                let n2 = ctx.core.player(x).hand.len();
                                let i2 = ctx.core.rng.gen_range(n2 as u32) as usize;
                                let c = ctx.core.player(x).hand[i2];
                                ctx.discard_from_hand(x, c);
                            } else if !ctx.core.player(x).equips.is_empty() {
                                let n2 = ctx.core.player(x).equips.len();
                                let j2 = ctx.core.rng.gen_range(n2 as u32) as usize;
                                let c = ctx.core.player_mut(x).equips.remove(j2);
                                crate::kernel::revoke_equip_skills(
                                    ctx.core,
                                    ctx.catalog,
                                    x,
                                    c,
                                );
                                ctx.to_discard(c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: x,
                                    cards: vec![c],
                                });
                            }
                        }
                    }
                    // 阶段 A：下一主目标雷伤
                    let Some(t) = self.targets.first().copied() else {
                        return Flow::Done;
                    };
                    self.targets.remove(0);
                    if !ctx.core.player(t).alive {
                        return Flow::Yield;
                    }
                    self.card = Some(t as CardId);
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(self.owner),
                        t,
                        1,
                        crate::event::DamageNature::Thunder,
                        None,
                    )));
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianLinghuo => {
                // 灵火（U05 xianjian 明绣，规则 v160）：确认后置 2 轮冷
                // 却并对阶段归属者 1 火伤（target=机器分流带入）。
                ctx.core.player_mut(self.owner).linghuo_ready_round =
                    ctx.core.turn.round + 2;
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            None,
                        )));
                    }
                }
                Flow::Done
            },
            HostBehavior::XianjianGuijin => match self.stage {
                // 归烬（U05 xianjian 明绣，规则 v160）：亮牌堆顶 4 张，
                // 逐张交付任意角色（获得者跳过下次摸牌阶段；取消=弃置
                // 剩余；登记差异：上游 chooseCardButton 可批量交付）。
                // 首臂发问（activate 创建首步无应答）。
                HostStage::Act => {
                    if self.target.is_none() {
                        self.target = Some(self.owner); // 装填标记
                        for _ in 0..4 {
                            if ctx.core.deck.is_empty() {
                                let n = ctx.core.discard.len();
                                if n == 0 {
                                    break;
                                }
                                let i =
                                    ctx.core.rng.gen_range(n as u32) as usize;
                                let c = ctx.core.discard.remove(i);
                                ctx.core.deck.push(c);
                            }
                            let c = ctx.core.deck.pop().expect("非空");
                            self.cards.push(c);
                        }
                        ctx.core.player_mut(self.owner).guijin_ready_round =
                            ctx.core.turn.round + 3;
                    }
                    if self.cards.is_empty() {
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = self
                        .cards
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_GUIJIN_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    // 取消=弃置剩余亮牌收束（上游 result.bool false 同径）
                    let picked = match ctx.answer().first() {
                        Some(ChoiceOption::Card(c)) => Some(*c),
                        _ => None,
                    };
                    let Some(c) = picked else {
                        for r in self.cards.drain(..) {
                            ctx.to_discard(r);
                        }
                        return Flow::Done;
                    };
                    self.cards.retain(|x| *x != c);
                    self.card = Some(c);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .map(ChoiceOption::Player)
                        .collect();
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_GUIJIN_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(c) = self.card.take() else {
                        return Flow::Done;
                    };
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first() {
                        let t = *t;
                        if ctx.core.player(t).alive {
                            ctx.core.player_mut(t).hand.push(c);
                            ctx.core.player_mut(t).skip_next_draw = true;
                            ctx.log(LogEvent::CardsGiven {
                                from: self.owner,
                                to: t,
                                cards: vec![c],
                            });
                        } else {
                            ctx.to_discard(c);
                        }
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianZhuyue => match self.stage {
                // 逐月（U05 xianjian 云天河，规则 v161）：弃一张非基本牌
                //（必选）→ 指定至多两名有牌其他角色 → 各随机弃一张（手
                // 牌优先退随机装备，蓝格怪衣同款登记差异）→ 白名单入
                // TurnState（出牌阶段结束清）。每臂只收一个应答。
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            yangpijuan_class(ctx.catalog.card_kind(*c)) != Some(0)
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_ZHUYUE_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first() else {
                        return Flow::Done;
                    };
                    let c = *c;
                    ctx.take_owned_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|t| {
                            *t != self.owner
                                && (!ctx.core.player(*t).hand.is_empty()
                                    || !ctx.core.player(*t).equips.is_empty())
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let max = options.len().min(2);
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_ZHUYUE_TARGETS,
                            options,
                            min: 1,
                            max,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    // 首入（target 为空）：收目标应答装填
                    if self.target.is_none() {
                        self.target = Some(self.owner); // 装填标记
                        let picked: Vec<PlayerId> = ctx
                            .answer()
                            .iter()
                            .filter_map(|o| match o {
                                ChoiceOption::Player(p) => Some(*p),
                                _ => None,
                            })
                            .collect();
                        let picked2 = picked.clone();
                        self.targets = picked;
                        for t in &picked2 {
                            if !ctx.core.turn.zhuyue_targets.contains(t) {
                                ctx.core.turn.zhuyue_targets.push(*t);
                            }
                        }
                    }
                    // 逐目标随机弃一张（存活重查；无牌跳过）
                    let Some(t) = self.targets.first().copied() else {
                        return Flow::Done;
                    };
                    self.targets.remove(0);
                    if !ctx.core.player(t).alive {
                        return Flow::Yield;
                    }
                    if !ctx.core.player(t).hand.is_empty() {
                        let n = ctx.core.player(t).hand.len();
                        let i = ctx.core.rng.gen_range(n as u32) as usize;
                        let c = ctx.core.player(t).hand[i];
                        ctx.discard_from_hand(t, c);
                    } else if !ctx.core.player(t).equips.is_empty() {
                        let n = ctx.core.player(t).equips.len();
                        let j = ctx.core.rng.gen_range(n as u32) as usize;
                        let c = ctx.core.player_mut(t).equips.remove(j);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    }
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianGuanri => match self.stage {
                // 贯日（U05 xianjian 云天河，规则 v161）：弃两张红手牌
                //（必选）→ 失去 1 点体力 → 指定体力≥自己的其他角色受
                // 2 火伤，存活则弃其装备区所有牌。limited 一次性
                //（guanri_used）。每臂只收一个应答。
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Heart
                                    | crate::catalog::Suit::Diamond
                            )
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_GUANRI_CARDS,
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.len() < 2 {
                        return Flow::Done;
                    }
                    for c in &picked {
                        ctx.take_owned_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: picked,
                    });
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|t| {
                            *t != self.owner
                                && ctx.core.player(*t).hp
                                    >= ctx.core.player(self.owner).hp
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_GUANRI_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    // 首入（target 为空）：置一次性标记+失 1 体力+对目标
                    // 2 火伤；Yield 后结「存活则弃其装备区所有牌」
                    if self.target.is_none() {
                        self.target = Some(self.owner); // 装填标记
                        ctx.core.player_mut(self.owner).guanri_used = true;
                        let t = match ctx.answer().first() {
                            Some(ChoiceOption::Player(p)) => Some(*p),
                            _ => None,
                        };
                        self.cards.clear();
                        if let Some(t) = t {
                            self.cards = vec![t as CardId];
                        }
                        ctx.emit(EventData::LoseHp { player: self.owner, amount: 1 });
                        if let Some(t) = self.cards.first().map(|x| *x as PlayerId) {
                            if ctx.core.player(t).alive {
                                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                    Some(self.owner),
                                    t,
                                    2,
                                    crate::event::DamageNature::Fire,
                                    None,
                                )));
                            }
                        }
                        return Flow::Yield;
                    }
                    // 结算臂：目标存活则弃其装备区所有牌
                    if let Some(t) = self.cards.first().map(|x| *x as PlayerId) {
                        self.cards.clear();
                        if ctx.core.player(t).alive && !ctx.core.player(t).equips.is_empty() {
                            let equips: Vec<CardId> =
                                ctx.core.player(t).equips.clone();
                            for c in equips {
                                ctx.core.player_mut(t).equips.retain(|x| *x != c);
                                crate::kernel::revoke_equip_skills(
                                    ctx.core,
                                    ctx.catalog,
                                    t,
                                    c,
                                );
                                ctx.to_discard(c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: t,
                                    cards: vec![c],
                                });
                            }
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianShuiyun => match self.stage {
                // 水云（U05 xianjian 紫萱，规则 v162）：阶段末从手中选一
                // 张与离场区类型（基本/锦囊/装备，yangpijuan_class 三分
                // 类）不重复的牌置入离场区（≤3）；取消=放弃（上游
                // direct+chooseCard 同口径）。首臂发问。
                HostStage::SelectCard => {
                    let used: Vec<Option<u8>> = ctx
                        .core
                        .player(self.owner)
                        .off_court
                        .iter()
                        .map(|o| yangpijuan_class(ctx.catalog.card_kind(*o)))
                        .collect();
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            let k =
                                yangpijuan_class(ctx.catalog.card_kind(*c));
                            !used.contains(&k)
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() || used.len() >= 3 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_SHUIYUN_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first() {
                        let c = *c;
                        if ctx.core.player(self.owner).hand.contains(&c)
                            && ctx.core.player(self.owner).off_court.len() < 3
                        {
                            ctx.hand_to_off_court(self.owner, c);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianShuiyun5 => match self.stage {
                // 水云·救（U05 xianjian 紫萱，规则 v162）：濒死确认门后
                // 必选离场区一张牌弃置，濒死者回复 1（shuiyun_count++——
                // 小蛮 shenwu 链依赖）。target=机器分流带入的濒死者。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .off_court
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_SHUIYUN5_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first()
                    else {
                        return Flow::Done;
                    };
                    let c = *c;
                    ctx.off_court_discard(self.owner, c);
                    let t = self.target.unwrap_or(self.owner);
                    if ctx.core.player(t).alive || ctx.core.player(t).hp <= 0 {
                        ctx.recover(t, 1, None);
                    }
                    ctx.core.player_mut(self.owner).shuiyun_count += 1;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianWangyou => match self.stage {
                // 忘忧（U05 xianjian 紫萱，规则 v162）：他人阶段末可弃一
                // 张手牌令本回合受伤标记者（wangyou3_round=当前轮）各摸
                // 1；取消=放弃。首臂发问；逐目标 Yield。
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_WANGYOU_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    if self.target.is_none() {
                        self.target = Some(self.owner); // 装填标记
                        let Some(ChoiceOption::Card(c)) =
                            ctx.answer().first().cloned()
                        else {
                            return Flow::Done; // 取消=放弃
                        };
                        let c = c;
                        ctx.discard_from_hand(self.owner, c);
                        self.targets = ctx
                            .core
                            .seat_order_from(self.owner)
                            .into_iter()
                            .filter(|p| {
                                *p != self.owner
                                    && ctx.core.player(*p).alive
                                    && ctx.core.player(*p).wangyou3_round
                                        == ctx.core.turn.round
                            })
                            .collect();
                    }
                    let Some(t) = self.targets.first().copied() else {
                        return Flow::Done;
                    };
                    self.targets.remove(0);
                    if ctx.core.player(t).alive {
                        ctx.draw_cards(t, 1);
                    }
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianWangyou2 => {
                // 忘忧·标（U05 xianjian 紫萱，规则 v162）：受伤者获本回
                // 合戳（wangyou3_round=当前轮；轮次推进自然过期）。
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        ctx.core.player_mut(t).wangyou3_round =
                            ctx.core.turn.round;
                    }
                }
                Flow::Done
            },
            HostBehavior::XianjianLingquan => match self.stage {
                // 灵泉（U05 xianjian 小蛮，规则 v163）：觉醒——摸 3+
                // runtime 授水云实例（上游 addSkill("shuiyun") 同口径）
                // +立即执行一次水云置牌（内联询问=上游 createTrigger 同
                // 效）。首臂做效果并发问（类型去重选项，可放弃）。
                HostStage::Act => {
                    ctx.core.player_mut(self.owner).lingquan_awakened = true;
                    ctx.draw_cards(self.owner, 3);
                    if let Some(def) = ctx.catalog.skill_by_name("shuiyun") {
                        ctx.core.skills.insert(
                            (self.owner, def),
                            crate::state::SkillInstance {
                                owner: self.owner,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 0,
                            },
                        );
                    }
                    let used: Vec<Option<u8>> = ctx
                        .core
                        .player(self.owner)
                        .off_court
                        .iter()
                        .map(|o| yangpijuan_class(ctx.catalog.card_kind(*o)))
                        .collect();
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            ctx.catalog.cards[*c as usize].kind
                                != CardKind::Tao // 摸 3 的牌含刚入手牌末——全部参与去重即可
                                || true
                        })
                        .filter(|c| {
                            let k =
                                yangpijuan_class(ctx.catalog.card_kind(*c));
                            !used.contains(&k)
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() || used.len() >= 3 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_LINGQUAN_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first() {
                        let c = *c;
                        if ctx.core.player(self.owner).hand.contains(&c)
                            && ctx.core.player(self.owner).off_court.len() < 3
                        {
                            ctx.hand_to_off_court(self.owner, c);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianShenwu => {
                // 神雾（U05 xianjian 小蛮，规则 v163）：觉醒——体力上限
                // +1、回复 1、runtime 授回梦实例（上游 gainMaxHp/recover/
                // addSkill("huimeng") 同口径）。
                ctx.core.player_mut(self.owner).shenwu_awakened = true;
                ctx.core.player_mut(self.owner).max_hp += 1;
                ctx.recover(self.owner, 1, None);
                if let Some(def) = ctx.catalog.skill_by_name("huimeng") {
                    ctx.core.skills.insert(
                        (self.owner, def),
                        crate::state::SkillInstance {
                            owner: self.owner,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                }
                Flow::Done
            },
            HostBehavior::XianjianChengxin => {
                // 诚信（U05 xianjian 明绣，规则 v164）：确认后冷却置位+
                // 濒死者回复至 1（recover 量=1-当前体力）+免伤至其 End
                // 阶段结束（kernel 直钩）。target=分流带入的濒死者。
                ctx.core.player_mut(self.owner).chengxin_ready_round =
                    ctx.core.turn.round + 4;
                if let Some(t) = self.target {
                    let hp = ctx.core.player(t).hp;
                    if hp <= 0 {
                        let amount = (1 - hp) as u32;
                        ctx.recover(t, amount, None);
                        ctx.core.player_mut(t).chengxin_active = true;
                    }
                }
                Flow::Done
            },
            HostBehavior::XianjianShenmu => {
                // 神暮（U05 xianjian 雪见，规则 v165）：确认后弃全部红手
                // 牌，濒死者回复 1 并摸等量张（上游 showHandcards 纯表现
                // 不迁移）。target=分流带入的濒死者。
                let reds: Vec<CardId> = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Heart
                                | crate::catalog::Suit::Diamond
                        )
                    })
                    .collect();
                let n = reds.len();
                for c in reds {
                    ctx.discard_from_hand(self.owner, c);
                }
                if let Some(t) = self.target {
                    if ctx.core.player(t).hp <= 0 {
                        ctx.recover(t, 1, None);
                        ctx.draw_cards(t, n);
                    }
                }
                Flow::Done
            },
            HostBehavior::XianjianDuci => match self.stage {
                // 毒刺（U05 xianjian 雪见，规则 v165）：弃装备牌入弃堆后
                // 令距离 1 内一名其他角色受 1 伤害（chooseTarget 取消=放
                // 弃；登记差异：仅弃置路径触发）。首臂发问。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.player(*p).alive
                                && ctx
                                    .core
                                    .computed_distance(ctx.catalog, self.owner, *p)
                                    <= 1
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_DUCI_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first() {
                        let t = *t;
                        if t != self.owner && ctx.core.player(t).alive {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                1,
                                crate::event::DamageNature::Normal,
                                None,
                            )));
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianRunxin => match self.stage {
                // 润心（U05 xianjian 柳梦璃，规则 v166）：使用/打出红桃
                // 牌后令一名已受伤角色回复 1（chooseTarget 取消=放弃）。
                // 首臂发问。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            (ctx.core.player(*p).hp as u32)
                                < ctx.core.player(*p).max_hp
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_RUNXIN_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first() {
                        let t = *t;
                        if (ctx.core.player(t).hp as u32)
                            < ctx.core.player(t).max_hp
                        {
                            ctx.recover(t, 1, None);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXjzhimeng => match self.stage {
                // 织梦（U05 xianjian 柳梦璃，规则 v167）：阶段末选一名其
                // 他角色→牌堆顶一张暗置其武将牌上（off_court）+runtime
                // 授织梦·梦实例（上游 addSkill 同口径）。取消=放弃。
                // 首臂发问。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() || ctx.core.deck.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_ZHIMENG_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first() {
                        let t = *t;
                        if t != self.owner && ctx.core.player(t).alive {
                            if let Some(c) = ctx.core.deck.pop() {
                                ctx.core.player_mut(t).off_court.push(c);
                            }
                            if let Some(def) =
                                ctx.catalog.skill_by_name("xjzhimeng2")
                            {
                                ctx.core.skills.insert(
                                    (t, def),
                                    crate::state::SkillInstance {
                                        owner: t,
                                        enabled: true,
                                        once_used: false,
                                        expires_end_of_turn: false,
                                        rounds: 0,
                                    },
                                );
                            }
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXjzhimeng2 => {
                // 织梦·梦（U05 xianjian 柳梦璃，规则 v167）：被使用同类型
                // 牌时摸 1（frequent silent 自动；暗牌保留=上游 storage
                // 不清除同口径）。
                ctx.draw_cards(self.owner, 1);
                Flow::Done
            },
            HostBehavior::XianjianSheling => {
                // 摄灵（U05 xianjian 南宫煌，规则 v168）：回收事件牌中在
                // 弃堆者（羊皮卷同款离堆入手）+回合次数 +1（每回合 3 次
                // =上游 usable:3 同口径）。
                ctx.core.turn.sheling_used += 1;
                for c in self.cards.clone() {
                    if let Some(pos) =
                        ctx.core.discard.iter().position(|x| *x == c)
                    {
                        ctx.core.discard.remove(pos);
                        ctx.core.player_mut(self.owner).hand.push(c);
                        ctx.log(LogEvent::CardsDrawn {
                            player: self.owner,
                            cards: vec![c],
                        });
                    }
                }
                Flow::Done
            },
            HostBehavior::XianjianHushaCount => {
                // 虎煞·集（U05 xianjian 王小虎，规则 v169）：+伤害量标记
                //（≤3；上游 husha_count forced 同口径）。
                let amount = match ctx.res.frame(ctx.current_frame_id()) {
                    Some(f) => match &f.data {
                        EventData::Damage(d) => d.amount,
                        _ => 0,
                    },
                    None => 0,
                } as u8;
                let cur = ctx.core.player(self.owner).husha_mark;
                ctx.core.player_mut(self.owner).husha_mark =
                    (cur + amount).min(3);
                Flow::Done
            },
            HostBehavior::XianjianHusha => match self.stage {
                // 虎煞（U05 xianjian 王小虎，规则 v169）：阶段末模式选择
                //（Tag0=1 枚虚拟杀+选目标；Tag1=2 枚虚拟南蛮 AOE；取消=
                // 放弃；元素毁灭牌种不存在登记不转）→ 虚拟用牌原语
                //（use_virtual_card 首例：牌堆底载体+use_as 声明）。
                HostStage::Act => {
                    let mark = ctx.core.player(self.owner).husha_mark;
                    let mut options: Vec<ChoiceOption> = Vec::new();
                    let has_sha_target = ctx
                        .core
                        .alive_players()
                        .any(|p| p != self.owner);
                    if mark >= 1 && has_sha_target {
                        options.push(ChoiceOption::Tag(0));
                    }
                    if mark >= 2 {
                        options.push(ChoiceOption::Tag(1));
                    }
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: WUDIAN_XJ_HUSHA_MODE,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let mode = match ctx.answer().first() {
                        Some(ChoiceOption::Tag(t)) => *t,
                        _ => {
                            return Flow::Done; // 取消=放弃
                        }
                    };
                    match mode {
                        0 => {
                            // 虚拟杀：扣 1 枚+选目标
                            ctx.core.player_mut(self.owner).husha_mark -= 1;
                            let options: Vec<ChoiceOption> = ctx
                                .core
                                .alive_players()
                                .filter(|p| *p != self.owner)
                                .map(ChoiceOption::Player)
                                .collect();
                            if options.is_empty() {
                                return Flow::Done;
                            }
                            self.stage = HostStage::SelectNature; // 复用为出杀目标段
                            Flow::WaitChoice {
                                observer: self.owner,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectPlayers,
                                    prompt: WUDIAN_XJ_HUSHA_TARGET,
                                    options,
                                    min: 1,
                                    max: 1,
                                    allow_cancel: false,
                                },
                            }
                        }
                        1 => {
                            // 虚拟南蛮：扣 2 枚+AOE（全其他存活）
                            ctx.core.player_mut(self.owner).husha_mark -= 2;
                            let targets: Vec<PlayerId> = ctx
                                .core
                                .seat_order_from(self.owner)
                                .into_iter()
                                .filter(|p| *p != self.owner)
                                .collect();
                            ctx.use_virtual_card(
                                self.owner,
                                CardKind::SavageAttack,
                                targets,
                            );
                            Flow::Done
                        }
                        _ => Flow::Done,
                    }
                }
                // 复用段：虚拟杀目标选择应答 → use_virtual_card(Sha)
                HostStage::SelectNature => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first() {
                        let t = *t;
                        if t != self.owner && ctx.core.player(t).alive {
                            ctx.use_virtual_card(
                                self.owner,
                                CardKind::Sha,
                                vec![t],
                            );
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianChangnian => match self.stage {
                // 长念（U05 xianjian 柳梦璃，规则 v170）：死亡前置窗口遗
                // 言——选一名其他角色移交 hej 全部牌（全部入手）+runtime
                // 授长念·思实例；随后 Commit 正常死亡。首臂发问。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_CHANGNIAN_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(t)) = ctx.answer().first()
                    else {
                        return Flow::Done;
                    };
                    let t = *t;
                    if t == self.owner || !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    // 移交 hej 全部牌（上游 gain 全部入手同口径；装备撤
                    // 技后入手）
                    let equips: Vec<CardId> =
                        ctx.core.player(self.owner).equips.clone();
                    for c in equips {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            c,
                        );
                        ctx.core.player_mut(t).hand.push(c);
                    }
                    let judge: Vec<CardId> =
                        ctx.core.player(self.owner).judge.clone();
                    for c in judge {
                        ctx.core.player_mut(self.owner).judge.retain(|x| *x != c);
                        ctx.core.player_mut(t).hand.push(c);
                    }
                    let hand: Vec<CardId> =
                        ctx.core.player(self.owner).hand.clone();
                    for c in hand {
                        ctx.take_hand_card(self.owner, c);
                        ctx.core.player_mut(t).hand.push(c);
                    }
                    if let Some(def) =
                        ctx.catalog.skill_by_name("changnian2")
                    {
                        ctx.core.skills.insert(
                            (t, def),
                            crate::state::SkillInstance {
                                owner: t,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 0,
                            },
                        );
                    }
                    ctx.log(LogEvent::CardsGiven {
                        from: self.owner,
                        to: t,
                        cards: {
                            let mut all = Vec::new();
                            all.extend(ctx.core.player(t).hand.iter().copied());
                            all
                        },
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianChangnian2 => {
                // 长念·思（U05 xianjian 柳梦璃，规则 v170）：结束阶段摸
                // 1（forced；runtime 授技实例持有者）。
                ctx.draw_cards(self.owner, 1);
                Flow::Done
            },
            HostBehavior::XianjianShiying => {
                // 适盈（U05 xianjian 月琦，规则 v171）：死亡取消型首例——
                // 写拦截标记（DeathMachine 直读跳过死亡执行）+双方体力
                // 上限/体力置 3+clearSkills 双方+弃 hej 全部+各摸 3。
                ctx.core.player_mut(self.owner).shiying_used = true;
                let Some(dying) = self.target else { return Flow::Done };
                ctx.core.player_mut(dying).death_cancelled = true;
                for p in [self.owner, dying] {
                    if !ctx.core.player(p).alive && p != dying {
                        continue;
                    }
                    let pl = ctx.core.player_mut(p);
                    pl.max_hp = 3;
                    pl.hp = 3;
                    pl.dying = false;
                }
                // clearSkills 双方（上游 clearSkills 同口径；shiying 实例
                // 自身也被清=上游同款）
                for owner in [self.owner, dying] {
                    let removed: Vec<crate::ids::SkillDefId> = ctx
                        .core
                        .skills
                        .keys()
                        .filter(|(o, _)| *o == owner)
                        .map(|(_, d)| *d)
                        .collect();
                    for def in removed {
                        ctx.core.skills.remove(&(owner, def));
                        ctx.log(LogEvent::SkillToggled {
                            player: owner,
                            skill: ctx.catalog.skill_name(def),
                            enabled: false,
                        });
                    }
                    // 弃 hej 全部
                    let hand: Vec<CardId> =
                        ctx.core.player(owner).hand.clone();
                    for c in hand {
                        ctx.take_hand_card(owner, c);
                        ctx.to_discard(c);
                    }
                    let equips: Vec<CardId> =
                        ctx.core.player(owner).equips.clone();
                    for c in equips {
                        ctx.core.player_mut(owner).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            owner,
                            c,
                        );
                        ctx.to_discard(c);
                    }
                    let judge: Vec<CardId> =
                        ctx.core.player(owner).judge.clone();
                    for c in judge {
                        ctx.core.player_mut(owner).judge.retain(|x| *x != c);
                        ctx.to_discard(c);
                    }
                    ctx.draw_cards(owner, 3);
                }
                Flow::Done
            },
            HostBehavior::XianjianZhaoyao => match self.stage {
                // 照妖（U05 xianjian 南宫煌，规则 v172）：确定性首张拼点
                //（舌战群儒同款登记差异）——赢→摸 2+选两张手牌置牌堆顶
                //（先选在上=后选的先压栈）。输/平→无事。首臂比点并发问。
                HostStage::Act => {
                    let own_r = ctx.core.player(self.owner)
                        .hand
                        .first()
                        .map(|c| ctx.catalog.cards[*c as usize].rank);
                    let t_r = self.target.and_then(|t| {
                        ctx.core.player(t)
                            .hand
                            .first()
                            .map(|c| ctx.catalog.cards[*c as usize].rank)
                    });
                    match (own_r, t_r) {
                        (Some(o), Some(t)) if o > t => {
                            ctx.draw_cards(self.owner, 2);
                            let hand: Vec<CardId> =
                                ctx.core.player(self.owner).hand.clone();
                            if hand.len() < 2 {
                                return Flow::Done;
                            }
                            let options: Vec<ChoiceOption> = hand
                                .iter()
                                .copied()
                                .map(ChoiceOption::Card)
                                .collect();
                            self.stage = HostStage::SelectCard;
                            Flow::WaitChoice {
                                observer: self.owner,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectCards,
                                    prompt: WUDIAN_XJ_ZHAOYAO_CARDS,
                                    options,
                                    min: 2,
                                    max: 2,
                                    allow_cancel: false,
                                },
                            }
                        }
                        _ => Flow::Done,
                    }
                }
                HostStage::SelectCard => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.len() == 2 {
                        for c in &picked {
                            ctx.take_hand_card(self.owner, *c);
                        }
                        // 先选在上：后选的先压栈
                        for c in picked.iter().rev() {
                            ctx.core.deck.push(*c);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianLiguang => match self.stage {
                // 离光（U05 xianjian 月琦，规则 v173）：弃一张手牌（必选，
                // 可取消=放弃）→移动场上第一张可移牌至首个合法目的地
                //（确定性，moveCard(true) AI 选取同款登记差异）。装备迁
                // 移 revoke+grant 技能；判定区直接迁移。首臂发问。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_LIGUANG_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first()
                    else {
                        return Flow::Done; // 取消=放弃
                    };
                    let c = *c;
                    ctx.take_hand_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 确定性移动：首张可移牌+首个合法目的地
                    'scan: for src in ctx.core.alive_players().collect::<Vec<_>>() {
                        for eq in ctx.core.player(src).equips.clone() {
                            let slot = ctx.catalog.card_kind(eq).equip_slot();
                            let dst = ctx.core.alive_players().find(|dst| {
                                *dst != src
                                    && ctx.core.player(*dst).equips.iter().all(|o| {
                                        ctx.catalog.card_kind(*o).equip_slot() != slot
                                    })
                            });
                            if let Some(dst) = dst {
                                ctx.core.player_mut(src).equips.retain(|x| *x != eq);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, src, eq);
                                ctx.core.player_mut(dst).equips.push(eq);
                                crate::kernel::grant_equip_skills(ctx, dst, eq);
                                ctx.log(LogEvent::Equipped {
                                    player: dst,
                                    card: eq,
                                    replaced: None,
                                });
                                break 'scan;
                            }
                        }
                        for jg in ctx.core.player(src).judge.clone() {
                            let k = ctx.catalog.card_kind(jg);
                            let dst = ctx.core.alive_players().find(|dst| {
                                *dst != src
                                    && ctx.core.player(*dst).judge.iter().all(|o| {
                                        ctx.catalog.card_kind(*o) != k
                                    })
                            });
                            if let Some(dst) = dst {
                                ctx.core.player_mut(src).judge.retain(|x| *x != jg);
                                ctx.core.player_mut(dst).judge.push(jg);
                                break 'scan;
                            }
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXuanmo => match self.stage {
                // 玄墨（U05 xianjian 夏侯瑾轩，规则 v174）：弃一张三类手
                // 牌置牌堆顶→随机获得牌堆+弃堆中同类型（三分类）实体牌
                // 2 张入手（实体发现基建首例=羊皮卷同款登记差异：上游
                // createCard 凭空造牌，原生实体池发现；不足给余量）。
                // 首臂发问。
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            yangpijuan_class(ctx.catalog.card_kind(*c)).is_some()
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_XUANMO_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first()
                    else {
                        return Flow::Done;
                    };
                    let c = *c;
                    let k = yangpijuan_class(ctx.catalog.card_kind(c));
                    ctx.take_hand_card(self.owner, c);
                    ctx.core.deck.push(c); // 置牌堆顶
                    ctx.log(LogEvent::CardPlayed {
                        card: c,
                        player: self.owner,
                        targets: vec![],
                    });
                    // 实体发现：牌堆+弃堆中同类型实体（排除置顶牌）随机
                    // 2 张入手（RNG；不足给余量）
                    let mut pool: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .chain(ctx.core.discard.iter())
                        .copied()
                        .filter(|x| {
                            *x != c
                                && yangpijuan_class(ctx.catalog.card_kind(*x))
                                    == k
                        })
                        .collect();
                    // RNG 洗牌取前 2
                    let n = pool.len();
                    for i in (1..n).rev() {
                        let j = ctx.core.rng.gen_range(i as u32 + 1) as usize;
                        pool.swap(i, j);
                    }
                    let take = pool.into_iter().take(2).collect::<Vec<_>>();
                    for x in take {
                        if let Some(pos) =
                            ctx.core.deck.iter().position(|y| *y == x)
                        {
                            ctx.core.deck.remove(pos);
                        } else if let Some(pos) = ctx
                            .core
                            .discard
                            .iter()
                            .position(|y| *y == x)
                        {
                            ctx.core.discard.remove(pos);
                        }
                        ctx.core.player_mut(self.owner).hand.push(x);
                        ctx.log(LogEvent::CardsDrawn {
                            player: self.owner,
                            cards: vec![x],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianSheying => {
                // 射影（U05 xianjian 苏媚，规则 v175）：判定黑色→获得伤
                // 害牌（direct 无询问；每回合 1 次=TurnState.sheying_used）。
                ctx.core.turn.sheying_used = true;
                let judged = ctx.draw_judge_card();
                let suit = ctx.catalog.cards[judged as usize].suit;
                ctx.log(LogEvent::Judged { player: self.owner, card: judged });
                ctx.to_discard(judged);
                let is_black = matches!(
                    suit,
                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                );
                if is_black {
                    // target 未用——伤害牌从 cards[0] 取
                    if let Some(c) = self.cards.first().copied() {
                        if let Some(pos) =
                            ctx.core.discard.iter().position(|x| *x == c)
                        {
                            ctx.core.discard.remove(pos);
                            ctx.core.player_mut(self.owner).hand.push(c);
                            ctx.log(LogEvent::CardsDrawn {
                                player: self.owner,
                                cards: vec![c],
                            });
                        }
                    }
                }
                Flow::Done
            },
            HostBehavior::XianjianHuahu => match self.stage {
                // 花好（U05 xianjian 苏媚，规则 v175）：多目标选择→置一次
                // 性+失 1 体力上限+clearSkills 自己→逐目标 Yield：各回复
                // 1+获 1 护甲+摸 1。首臂发问。
                HostStage::TargetChoice => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let max = options.len();
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_HUAHU_TARGETS,
                            options,
                            min: 1,
                            max,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    if self.target.is_none() {
                        self.target = Some(self.owner); // 装填标记
                        let picked: Vec<PlayerId> = ctx
                            .answer()
                            .iter()
                            .filter_map(|o| match o {
                                ChoiceOption::Player(p) => Some(*p),
                                _ => None,
                            })
                            .collect();
                        self.targets = picked;
                        ctx.core.player_mut(self.owner).huahu_used = true;
                        let pl = ctx.core.player_mut(self.owner);
                        pl.max_hp = pl.max_hp.saturating_sub(1);
                        let mh = pl.max_hp as i32;
                        if pl.hp > mh {
                            pl.hp = mh;
                        }
                        let removed: Vec<crate::ids::SkillDefId> = ctx
                            .core
                            .skills
                            .keys()
                            .filter(|(o, _)| *o == self.owner)
                            .map(|(_, d)| *d)
                            .collect();
                        for def in removed {
                            ctx.core.skills.remove(&(self.owner, def));
                            ctx.log(LogEvent::SkillToggled {
                                player: self.owner,
                                skill: ctx.catalog.skill_name(def),
                                enabled: false,
                            });
                        }
                    }
                    let Some(t) = self.targets.first().copied() else {
                        return Flow::Done;
                    };
                    self.targets.remove(0);
                    if ctx.core.player(t).alive {
                        ctx.recover(t, 1, None);
                        ctx.change_hujia(t, 1);
                        ctx.draw_cards(t, 1);
                    }
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXtanxi => match self.stage {
                // 探息（U05 xianjian 仙卿，规则 v176）：弃一张手牌→持有
                // 同名牌手牌的其他角色中随机一名→随机获得其一张手牌
                //（亮牌纯表现不迁移；无人持有同名牌则牌照弃）。
                // 首臂发问。
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_XTANXI_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first()
                    else {
                        return Flow::Done; // 取消=放弃
                    };
                    let c = *c;
                    let name_kind = ctx.catalog.card_kind(c);
                    ctx.take_hand_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 候选：持有同（牌）种手牌的其他角色——同名牌判定按
                    // 牌种（sha/shan 各只有一种实体）
                    let cands: Vec<PlayerId> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.player(*p).hand.iter().any(|h| {
                                    ctx.catalog.card_kind(*h) == name_kind
                                })
                        })
                        .collect();
                    if cands.is_empty() {
                        return Flow::Done;
                    }
                    let i = ctx.core.rng.gen_range(cands.len() as u32) as usize;
                    let t = cands[i];
                    self.target = Some(t);
                    let hand: Vec<CardId> = ctx.core.player(t).hand.clone();
                    let n = hand.len();
                    let j = ctx.core.rng.gen_range(n as u32) as usize;
                    let taken = hand[j];
                    ctx.core.player_mut(t).hand.retain(|x| *x != taken);
                    ctx.core.player_mut(self.owner).hand.push(taken);
                    ctx.log(LogEvent::CardsGiven {
                        from: t,
                        to: self.owner,
                        cards: vec![taken],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianFenglue => match self.stage {
                // 凤略（U05 xianjian 罗招嫣，规则 v177）：出牌阶段开始选
                // 一名其他角色→逐张自动对其使用手中可指定其的杀与决斗
                //（真实牌 take 后 emit CardUsed=虚拟管线复用真实牌变体；
                // **登记差异**：其余锦囊不自动使用）。首臂发问，取消=放
                // 弃；逐张 Yield。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner && ctx.core.player(*p).alive
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_FENGLUE_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    if self.target.is_none() {
                        if let Some(ChoiceOption::Player(t)) =
                            ctx.answer().first().cloned()
                        {
                            if t != self.owner && ctx.core.player(t).alive {
                                self.target = Some(t);
                            }
                        }
                        if self.target.is_none() {
                            return Flow::Done;
                        }
                        // 收集可对其使用的杀（距离内）与决斗
                        let t = self.target.expect("checked");
                        let usable: Vec<CardId> = ctx
                            .core
                            .player(self.owner)
                            .hand
                            .iter()
                            .copied()
                            .filter(|c| {
                                let k = ctx.catalog.card_kind(*c);
                                match k {
                                    CardKind::Sha => ctx
                                        .core
                                        .computed_distance(ctx.catalog, self.owner, t)
                                        <= ctx.core.attack_range(ctx.catalog, self.owner),
                                    CardKind::Duel => true,
                                    _ => false,
                                }
                            })
                            .collect();
                        self.cards = usable;
                    }
                    // 逐张使用（emit CardUsed 由提交阶段 take 真实牌）
                    let Some(c) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    self.cards.remove(0);
                    let t = self.target.expect("checked");
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: c,
                        player: self.owner,
                        targets: vec![t],
                        nature: crate::event::DamageNature::Normal,
                        use_as: None,
                        cancelled: false,
                    }));
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianLiefeng => {
                // 疾风（U05 xianjian 姜云凡，规则 v178）：第 3/4 张使用
                // 牌时 runtime 授天剑/御风实例（第 2 张授炎杀 yanzhan
                // viewAs 暂缓不授；上游 xjyufeng 授予时补牌由御风自身
                // filter 兜底，此处仅授技）。
                let n = ctx.core.turn.cards_used_play_phase;
                let skill_name = match n {
                    3 => Some("tianjian"),
                    4 => Some("xjyufeng"),
                    _ => None,
                };
                if let Some(name) = skill_name {
                    if let Some(def) = ctx.catalog.skill_by_name(name) {
                        if !ctx.core.skills.contains_key(&(self.owner, def)) {
                            ctx.core.skills.insert(
                                (self.owner, def),
                                crate::state::SkillInstance {
                                    owner: self.owner,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: false,
                                    rounds: 0,
                                },
                            );
                        }
                    }
                }
                Flow::Done
            },
            HostBehavior::XianjianTianjian => match self.stage {
                // 天剑（U05 xianjian 姜云凡，规则 v178）：弃一张手牌杀
                //（必选）→视为使用万箭齐发（虚拟 AOE 原语：全其他存活）
                // →伤害目标随机弃 he 一张（discard 子技直连=登记差异：
                // 上游为 damageEnd 子技）。
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            ctx.catalog.card_kind(*c)
                                == crate::catalog::CardKind::Sha
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_TIANJIAN_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first()
                    else {
                        return Flow::Done;
                    };
                    let c = *c;
                    ctx.take_hand_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 虚拟万箭 AOE
                    let targets: Vec<PlayerId> = ctx
                        .core
                        .seat_order_from(self.owner)
                        .into_iter()
                        .filter(|p| *p != self.owner)
                        .collect();
                    ctx.use_virtual_card(
                        self.owner,
                        CardKind::ArcheryAttack,
                        targets.clone(),
                    );
                    // 伤害目标随机弃 he 一张（直连）
                    for p in targets {
                        if !ctx.core.player(p).alive {
                            continue;
                        }
                        let mut he: Vec<CardId> = ctx
                            .core
                            .player(p)
                            .hand
                            .iter()
                            .copied()
                            .collect();
                        he.extend(ctx.core.player(p).equips.iter().copied());
                        if he.is_empty() {
                            continue;
                        }
                        let i = ctx.core.rng.gen_range(he.len() as u32) as usize;
                        let dc = he[i];
                        if ctx.core.player(p).hand.contains(&dc) {
                            ctx.discard_from_hand(p, dc);
                        } else {
                            ctx.core.player_mut(p).equips.retain(|x| *x != dc);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, dc);
                            ctx.to_discard(dc);
                            ctx.log(LogEvent::CardDiscarded { player: p, cards: vec![dc] });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianPoyun => match self.stage {
                // 破云（U05 xianjian 慕容紫英，规则 v179）：玄砚计数>0
                // 时令受伤目标自选弃 he 1-2 张，计数 -1（direct 无确
                // 认；计数获得路径上游缺失=调试/宿主注入登记差异）。
                // 首臂发问（被弃者自选）。
                HostStage::Act => {
                    let Some(t) = self.target else { return Flow::Done };
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .chain(ctx.core.player(t).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let max = options.len().min(2);
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_POYUN_CARDS,
                            options,
                            min: 1,
                            max,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    let Some(t) = self.target else { return Flow::Done };
                    for c in &picked {
                        if ctx.core.player(t).hand.contains(c) {
                            ctx.discard_from_hand(t, *c);
                        } else if ctx.core.player(t).equips.contains(c) {
                            ctx.core.player_mut(t).equips.retain(|x| x != c);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, *c);
                            ctx.to_discard(*c);
                        }
                    }
                    ctx.core.player_mut(self.owner).xuanning_mark =
                        ctx.core.player(self.owner).xuanning_mark.saturating_sub(1);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXshuangren => {
                // 双刃（U05 xianjian 雪见，规则 v180）：弃牌中的武器从弃
                // 堆回收置离场区+grant 该武器技能（上游 addAdditionalSkill
                // 同口径；**登记差异**：仅弃置路径触发）。
                for c in self.cards.clone() {
                    let is_weapon = ctx.catalog.card_kind(c).equip_slot()
                        == Some(crate::catalog::EquipSlot::Weapon);
                    let Some(pos) =
                        ctx.core.discard.iter().position(|x| *x == c)
                    else {
                        continue;
                    };
                    if is_weapon {
                        ctx.core.discard.remove(pos);
                        ctx.core.player_mut(self.owner).off_court.push(c);
                        crate::kernel::grant_equip_skills(
                            ctx,
                            self.owner,
                            c,
                        );
                    }
                }
                Flow::Done
            },
            HostBehavior::XianjianXshuangren2 => {
                // 双刃·殁（U05 xianjian 雪见，规则 v180）：死亡时弃置离
                // 场区全部牌（通用区清空=登记差异）。
                let oc: Vec<CardId> =
                    ctx.core.player(self.owner).off_court.clone();
                for c in oc {
                    ctx.core.player_mut(self.owner).off_court.retain(|x| *x != c);
                    ctx.to_discard(c);
                }
                Flow::Done
            },
            HostBehavior::XianjianZhangmu => {
                // 张目（U05 xianjian 南宫煌，规则 v181）：需要打出闪响应
                // 时弃一张手牌闪直接视为已响应（杀不命中=前置直防；弃哪
                // 张闪确定性取首张闪=登记差异；每回合 1 次）。
                ctx.core.turn.zhangmu_used = true;
                let Some(shan) = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .find(|c| {
                        ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Shan
                    })
                else {
                    return Flow::Done;
                };
                ctx.discard_from_hand(self.owner, shan);
                // 当前帧=杀伤害前置窗口：直写 prevented（诚信同款）
                {
                    let frame = ctx.res.frame_mut(ctx.current_frame_id()).expect("frame");
                    if let EventData::Damage(d) = &mut frame.data {
                        d.prevented = true;
                    }
                }
                ctx.log(LogEvent::DamagePrevented { target: self.owner });
                Flow::Done
            },
            HostBehavior::XianjianQianfang => match self.stage {
                // 千方（U05 xianjian 慕容紫英，规则 v182）：回合开始时
                // 玄砚计数换弃牌（num=3-计数 clamp≥0）后清零，视为对全
                // 部其他角色使用万箭齐发（虚拟 AOE 原语）；被伤敌人每名
                // 紫英摸 1（qianfang_draw 直连=登记差异：上游 damageEnd
                // 逐目标）。首臂发问弃牌（可取消=放弃；num=0 时直接
                // AOE）。
                HostStage::Act => {
                    let mark = ctx.core.player(self.owner).xuanning_mark;
                    let num = 3usize.saturating_sub(mark as usize);
                    let targets: Vec<PlayerId> = ctx
                        .core
                        .seat_order_from(self.owner)
                        .into_iter()
                        .filter(|p| *p != self.owner)
                        .collect();
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    self.targets = targets.clone();
                    self.cards = vec![CardId::from(num as u32)]; // num 暂存
                    if num == 0 {
                        // 纯 AOE（chooseBool 确认=direct 无询问直接发动）
                        ctx.core.player_mut(self.owner).xuanning_mark = 0;
                        ctx.use_virtual_card(
                            self.owner,
                            CardKind::ArcheryAttack,
                            targets,
                        );
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.len() < num {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_QIANFANG_CARDS,
                            options,
                            min: num,
                            max: num,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    let num = 3usize
                        .saturating_sub(ctx.core.player(self.owner).xuanning_mark as usize);
                    if picked.len() != num {
                        return Flow::Done; // 取消=放弃
                    }
                    for c in &picked {
                        if ctx.core.player(self.owner).hand.contains(c) {
                            ctx.discard_from_hand(self.owner, *c);
                        } else {
                            ctx.core.player_mut(self.owner).equips.retain(|x| x != c);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, *c);
                            ctx.to_discard(*c);
                        }
                    }
                    ctx.core.player_mut(self.owner).xuanning_mark = 0;
                    let targets = self.targets.clone();
                    ctx.use_virtual_card(
                        self.owner,
                        CardKind::ArcheryAttack,
                        targets.clone(),
                    );
                    // 被伤敌人每名紫英摸 1（qianfang_draw 直连=登记差异：
                    // 上游 damageEnd 逐目标即时）
                    let hurt: Vec<PlayerId> = targets
                        .into_iter()
                        .filter(|p| ctx.core.player(*p).alive)
                        .collect();
                    let n = hurt.len();
                    ctx.draw_cards(self.owner, n);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXunying => {
                // 迅影（U05 xianjian 姜云凡，规则 v183）：杀结算后自动
                // 再出一张手牌杀对同一目标（**登记差异**：上游
                // chooseToUse 玩家选牌，原生自动取首张杀；酒状态保留不
                // 迁移；sha 限次不适用=CardUsed 直发不经 PlayCard 校验）。
                let Some(t) = self.target else { return Flow::Done };
                if !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                let Some(sha) = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .find(|c| {
                        ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Sha
                    })
                else {
                    return Flow::Done;
                };
                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                    card: sha,
                    player: self.owner,
                    targets: vec![t],
                    nature: crate::event::DamageNature::Normal,
                    use_as: None,
                    cancelled: false,
                }));
                Flow::Done
            },
            HostBehavior::XianjianTianxian => {
                // 天弦（锁定旗标行为）：无机器步
                Flow::Done
            },
            HostBehavior::XianjianXjyingfeng => {
                // 影锋（U05 xianjian 越今朝，规则 v185）：随机候选虚拟
                // 杀（use_virtual_card=牌堆底载体完整管线；虚拟杀
                // use_as=Some(Sha) 不再命中本 filter=防递归）。
                // 影锋虚拟杀结算后窗拒绝再触发：emit 前置链挡旗标（上游
                // parent.name!=="xjyingfeng" 的登记差异等价——同回合多张
                // 真实杀仅首次触发，Judge begin 清）
                ctx.core.turn.yingfeng_chain = true;
                let exclude: Vec<PlayerId> = match ctx.current_card_use() {
                    Ok(u) => u.targets.clone(),
                    Err(_) => Vec::new(),
                };
                if !exclude.is_empty() {
                    let candidates: Vec<PlayerId> = ctx
                        .core
                        .seat_order_from(self.owner)
                        .into_iter()
                        .filter(|t| {
                            *t != self.owner
                                && !exclude.contains(t)
                                && ctx.core.player(*t).alive
                                && ctx
                                    .core
                                    .computed_distance(ctx.catalog, self.owner, *t)
                                    <= ctx.core.attack_range(ctx.catalog, self.owner)
                        })
                        .collect();
                    if let Some(&pick) = candidates.get(
                        ctx.core.rng.gen_range(candidates.len() as u32) as usize,
                    ) {
                        ctx.use_virtual_card(
                            self.owner,
                            crate::catalog::CardKind::Sha,
                            vec![pick],
                        );
                    }
                }
                Flow::Done
            },
            HostBehavior::XianjianXiepan => match self.stage {
                // 械磐（U05 xianjian 居十方，规则 v186）：弃最后一张基本
                // 牌后可弃一张手牌获一件随机装备（可取消=放弃）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_XIEPAN_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) =
                        ctx.answer().first().copied()
                    else {
                        return Flow::Done; // 取消=放弃
                    };
                    ctx.discard_from_hand(self.owner, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 实体发现：牌堆+弃堆装备实体随机 1 张入手（RNG；
                    // 不足无=实体池余量口径）
                    let mut pool: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .chain(ctx.core.discard.iter())
                        .copied()
                        .filter(|x| {
                            *x != c
                                && yangpijuan_class(ctx.catalog.card_kind(*x))
                                    == Some(2)
                        })
                        .collect();
                    let n = pool.len();
                    for i in (1..n).rev() {
                        let j =
                            ctx.core.rng.gen_range(i as u32 + 1) as usize;
                        pool.swap(i, j);
                    }
                    if let Some(x) = pool.first().copied() {
                        if let Some(pos) =
                            ctx.core.deck.iter().position(|y| *y == x)
                        {
                            ctx.core.deck.remove(pos);
                        } else if let Some(pos) = ctx
                            .core
                            .discard
                            .iter()
                            .position(|y| *y == x)
                        {
                            ctx.core.discard.remove(pos);
                        }
                        ctx.core.player_mut(self.owner).hand.push(x);
                        ctx.log(LogEvent::CardsDrawn {
                            player: self.owner,
                            cards: vec![x],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianLuanjian => match self.stage {
                // 乱剑（U05 xianjian 长卿，规则 v187）：两张杀当一张杀
                // 使用（虚拟杀=天剑同款弃素材；机器直发不经 PlayCard
                // 校验=无视距离+任意多目标登记差异）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Sha
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_LUANJIAN_CARDS,
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.len() != 2 {
                        return Flow::Done; // 取消=放弃
                    }
                    for c in &picked {
                        ctx.discard_from_hand(self.owner, *c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: picked.clone(),
                    });
                    self.cards = picked;
                    // 目标多选（任意名其他存活角色；allow_cancel=false=
                    // 上游 selectTarget [1,∞] 必选，素材已弃不退）
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .seat_order_from(self.owner)
                        .into_iter()
                        .filter(|p| *p != self.owner && ctx.core.player(*p).alive)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let max = options.len();
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_LUANJIAN_TARGETS,
                            options,
                            min: 1,
                            max,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let picked: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if picked.is_empty() {
                        return Flow::Done;
                    }
                    // 载体牌锚定（DamageBefore 增伤判定=普通杀/其他虚拟
                    // 技载体不命中）——预读牌堆底=use_virtual_card 载体
                    let carrier = ctx.core.deck.first().copied();
                    ctx.core.turn.luanjian_card =
                        carrier.map(|c| c as u32);
                    ctx.use_virtual_card(
                        self.owner,
                        crate::catalog::CardKind::Sha,
                        picked,
                    );
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianLuanjian2 => {
                // 乱剑·增伤：50% 伤害+1（上游 Math.random()<0.5 同 RNG
                // 流；DamageBefore 帧直写 amount=张目 prevented 同款机
                // 制）；每目标独立 roll（filter 已过=载体锚定成立）。
                if ctx.core.rng.gen_range(100) < 50 {
                    let frame = ctx.res.frame_mut(ctx.current_frame_id()).expect("frame");
                    if let EventData::Damage(dm) = &mut frame.data {
                        dm.amount += 1;
                    }
                }
                Flow::Done
            },
            HostBehavior::XianjianCtianfu => match self.stage {
                // 天符（U05 xianjian 长卿，规则 v188）：弃一张闪置一名
                // 其他角色武将牌上（discard:false=挂附不弃置；离场区承
                // 载+授 ctianfu2 实例+记来源）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Shan
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_CTIANFU_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) =
                        ctx.answer().first().copied()
                    else {
                        return Flow::Done; // 取消=放弃
                    };
                    self.cards = vec![c];
                    // 目标：其他角色且无挂附（上游 filterTarget
                    // !hasSkill("ctianfu2") 同口径）
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .seat_order_from(self.owner)
                        .into_iter()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.player(*p).alive
                                && !ctx.core.skills.contains_key(&(
                                    *p,
                                    ctx.catalog
                                        .skill_by_name("ctianfu2")
                                        .expect("ctianfu2"),
                                ))
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        // 闪不消耗（未置出）——留手牌
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_CTIANFU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(t)) =
                        ctx.answer().first().copied()
                    else {
                        return Flow::Done;
                    };
                    let Some(c) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    // 挂附：闪离自己手牌 → 置目标武将牌上（离场区）
                    ctx.take_hand_card(self.owner, c);
                    ctx.core.player_mut(t).off_court.push(c);
                    for sub in ["ctianfu2", "ctianfu3"] {
                        if let Some(def) = ctx.catalog.skill_by_name(sub) {
                            ctx.core.skills.insert(
                                (t, def),
                                crate::state::SkillInstance {
                                    owner: t,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: false,
                                    rounds: 0,
                                },
                            );
                        }
                    }
                    let pl = ctx.core.player_mut(t);
                    pl.ctianfu_source = Some(self.owner);
                    pl.ctianfu_card = Some(c as u32);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianCtianfu2 => {
                // 天符·反噬（ctianfu2，规则 v188）：挂附者造成伤害后——
                // 受来自挂附来源的 1 点雷伤（来源亡=无来源）+随机弃 he
                // 一张+挂附闪入弃堆+清挂附（触发即清=链天然终止）。
                let src = ctx.core.player(self.owner).ctianfu_source;
                let Some(src) = src else { return Flow::Done };
                let src_alive = src < ctx.core.players.len() as u8
                    && ctx.core.player(src).alive;
                let nature = crate::event::DamageNature::Thunder;
                let (sid, dmg_src) = if src_alive {
                    (Some(src), Some(src))
                } else {
                    (None, None)
                };
                let _ = sid;
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    dmg_src,
                    self.owner,
                    1,
                    nature,
                    None,
                )));
                // 随机弃 he 一张（上游 he.randomGet 同 RNG）
                let pool: Vec<CardId> = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .chain(ctx.core.player(self.owner).equips.iter())
                    .copied()
                    .collect();
                if let Some(x) = pool.get(
                    ctx.core.rng.gen_range(pool.len() as u32) as usize,
                ) {
                    let x = *x;
                    if ctx.core.player(self.owner).hand.contains(&x) {
                        ctx.discard_from_hand(self.owner, x);
                    } else {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|y| *y != x);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            x,
                        );
                        ctx.to_discard(x);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![x],
                    });
                }
                // 挂附闪入弃堆+清挂附+清实例
                ctianfu_clear(ctx, self.owner);
                if let Some(def) = ctx.catalog.skill_by_name("ctianfu2") {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            },
            HostBehavior::XianjianCtianfu3 => {
                // 天符·清挂（ctianfu3，规则 v188）：挂附者死亡——挂附闪
                // 入弃堆（**登记差异**：上游凭空消失）+清挂附+清两实例。
                ctianfu_clear(ctx, self.owner);
                for sub in ["ctianfu2", "ctianfu3"] {
                    if let Some(def) = ctx.catalog.skill_by_name(sub) {
                        ctx.core.skills.remove(&(self.owner, def));
                    }
                }
                Flow::Done
            },
            HostBehavior::SwdZhiluxiaohuCard => {
                // 指路小虎（U05 swd 机关牌首切片，规则 v189）：随机对一
                // 名其他存活角色使用虚拟杀（无懈窗已在外层包装；虚拟杀
                // 完整管线走闪询问/伤害）。
                let candidates: Vec<PlayerId> = ctx
                    .core
                    .seat_order_from(self.owner)
                    .into_iter()
                    .filter(|p| *p != self.owner && ctx.core.player(*p).alive)
                    .collect();
                if let Some(&pick) = candidates.get(
                    ctx.core.rng.gen_range(candidates.len() as u32) as usize,
                ) {
                    ctx.use_virtual_card(
                        self.owner,
                        crate::catalog::CardKind::Sha,
                        vec![pick],
                    );
                }
                // 本体弃置（卡牌机器惯例=流星火雨同款）
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::SwdMujiarenCard => {
                // 木甲人（U05 swd 机关牌第二切片，规则 v190）：弃全部
                // 非基本手牌（静默=上游 lose _triggered=null 同口径）→
                // 随机机关牌 1+N 张入手（实体发现，排除本体）。
                ctx.core.turn.mujiaren_used = true;
                let dropped: Vec<CardId> = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        crate::content::host::yangpijuan_class(
                            ctx.catalog.card_kind(*c),
                        ) != Some(0)
                    })
                    .collect();
                for c in &dropped {
                    ctx.take_hand_card(self.owner, *c);
                    ctx.to_discard(*c);
                }
                if !dropped.is_empty() {
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: dropped.clone(),
                    });
                }
                let num = 1 + dropped.len();
                // 实体发现：牌堆+弃堆 Jiguan 池随机（排除本体；不足给余量）
                let mut pool: Vec<CardId> = ctx
                    .core
                    .deck
                    .iter()
                    .chain(ctx.core.discard.iter())
                    .copied()
                    .filter(|x| {
                        Some(*x) != self.card
                            && ctx.catalog.card_kind(*x)
                                == crate::catalog::CardKind::Jiguan
                    })
                    .collect();
                let n = pool.len();
                for i in (1..n).rev() {
                    let j = ctx.core.rng.gen_range(i as u32 + 1) as usize;
                    pool.swap(i, j);
                }
                for x in pool.into_iter().take(num) {
                    if let Some(pos) =
                        ctx.core.deck.iter().position(|y| *y == x)
                    {
                        ctx.core.deck.remove(pos);
                    } else if let Some(pos) = ctx
                        .core
                        .discard
                        .iter()
                        .position(|y| *y == x)
                    {
                        ctx.core.discard.remove(pos);
                    }
                    ctx.core.player_mut(self.owner).hand.push(x);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: vec![x],
                    });
                }
                // 本体弃置（卡牌机器惯例）
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::SwdGouhunluoCard => {
                // 勾魂锣（U05 swd 机关牌第三切片，规则 v191）：目标挂
                // 3 轮倒计时（storage=3+来源）+授倒计时实例（kind 级
                // def=card_kind_host_skills）。
                let Some(t) = self.target else { return Flow::Done };
                if !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                let pl = ctx.core.player_mut(t);
                pl.gouhunluo_rounds = 3;
                pl.gouhunluo_source = Some(self.owner);
                if let Some(def) =
                    ctx.catalog.skill_by_name(SWD_GOUHUNLUO_SUB_KEY)
                {
                    ctx.core.skills.insert(
                        (t, def),
                        crate::state::SkillInstance {
                            owner: t,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                }
                // 本体弃置（卡牌机器惯例）
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            },
            HostBehavior::SwdGouhunluoSub => {
                // 勾魂锣·倒计时（规则 v191）：来源回合开始-1；归 0=
                // 失去 1 体力+弃全部手牌（真实弃置事件）+清挂附+清实例。
                let Some(src) = ctx.core.player(self.owner).gouhunluo_source
                else {
                    return Flow::Done;
                };
                if !ctx.core.player(src).alive {
                    return Flow::Done;
                }
                let rounds = ctx.core.player(self.owner).gouhunluo_rounds;
                if rounds > 1 {
                    ctx.core.player_mut(self.owner).gouhunluo_rounds = rounds - 1;
                    return Flow::Done;
                }
                // 归 0：失去 1 体力
                ctx.emit(EventData::LoseHp {
                    player: self.owner,
                    amount: 1,
                                    });
                // 弃全部手牌（真实弃置事件=械磐等可触发）
                let hand: Vec<CardId> = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .collect();
                if !hand.is_empty() {
                    for c in &hand {
                        ctx.take_hand_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: hand,
                    });
                }
                // 清挂附+清实例
                let pl = ctx.core.player_mut(self.owner);
                pl.gouhunluo_rounds = 0;
                pl.gouhunluo_source = None;
                if let Some(def) =
                    ctx.catalog.skill_by_name(SWD_GOUHUNLUO_SUB_KEY)
                {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            },
            HostBehavior::SwdJiguantongCard => match self.stage {
                // 机关桶（U05 swd 机关牌第四切片，规则 v192；v193 补齐
                // contentAfter 与逆序）：AOE 逐目标「弃一张手牌或受 1 火
                // 伤」（上游 ♦4/♣7，reverseOrder 逆序由 kernel 工厂臂在
                // 目标表反转）。无手牌目标直接火伤。
                HostStage::Act => {
                    // 本体已弃置 = AOE 已完成；targets 空 = 结算完 →
                    // contentAfter（无人受伤 → 使用者摸 1）→ 弃本体
                    if self.card.is_none() {
                        return Flow::Done;
                    }
                    let Some(t) = self.targets.first().copied() else {
                        let Some(body) = self.card else {
                            return Flow::Done;
                        };
                        // 上游 contentAfter !preResult：本次结算中无以本
                        // 牌归因的实际伤害（无懈抵消/取消弃牌不算；弃牌
                        // 免伤不算）→ 使用者摸 1（按日志实伤判定）
                        let damaged = ctx.out.iter().any(|l| {
                            matches!(l, LogEvent::DamageDealt { card: Some(c), .. } if *c == body)
                        });
                        if !damaged {
                            ctx.draw_cards(self.owner, 1);
                        }
                        // 本体弃置（卡牌机器惯例）
                        ctx.to_discard(body);
                        ctx.core.table.retain(|x| *x != body);
                        self.card = None;
                        return Flow::Done;
                    };
                    self.targets.remove(0);
                    if !ctx.core.player(t).alive {
                        return Flow::Yield;
                    }
                    if ctx.core.player(t).hand.is_empty() {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            self.card,
                        )));
                        return Flow::Yield;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.targets.insert(0, t); // 未决目标回队首（SelectCard 段消费）
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: SWD_JIGUANTONG_DISCARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                        // 取消=受 1 火伤
                        let Some(t) = self.targets.first().copied() else {
                            return Flow::Done;
                        };
                        self.targets.remove(0);
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            self.card,
                        )));
                        self.stage = HostStage::Act;
                        return Flow::Yield;
                    };
                    let Some(t) = self.targets.first().copied() else {
                        return Flow::Done;
                    };
                    self.targets.remove(0);
                    ctx.discard_from_hand(t, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: t,
                        cards: vec![c],
                    });
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 机关凤（U05 swd 第二十批，规则 v195）：单体——目标选择打出
            // 一张闪（无闪/取消=未响应）；未响应→目标获得"封印"（全部技
            // 能实例禁用，其下一阶段开始解除[SwdFengyinState]，登记差异：
            // 原生无锁定技区分）+受 1 点伤害；响应则无事。本体入弃堆。
            HostBehavior::SwdJiguanfengCard => match self.stage {
                HostStage::Act => {
                    let Some(t) = self.target else {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Shan)
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        // 无闪：上游 chooseToRespond 自动失败 → 未响应路径
                        self.seal_and_damage(ctx);
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: SWD_JIGUANFENG_SHAN,
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let responded = matches!(ctx.answer().first(), Some(ChoiceOption::Card(_)));
                    if responded {
                        // 打出闪响应：闪入弃牌堆（上游 chooseToRespond 同）
                        if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                            if let Some(t) = self.target {
                                ctx.discard_from_hand(t, c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: t,
                                    cards: vec![c],
                                });
                            }
                        }
                    } else {
                        // 未响应：封印 + 1 点伤害（上游 addTempSkill fengyin
                        // + damage 同序）
                        self.seal_and_damage(ctx);
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                        self.card = None;
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 机关凤封印状态（规则 v195）：拥有者任一阶段开始→全部技能
            // 实例重新启用并移除自身（上游 addTempSkill {player:
            // 'phaseBegin'} 移除同口径）
            HostBehavior::SwdFengyinState => {
                for ((p, _), inst) in ctx.core.skills.iter_mut() {
                    if *p == self.owner {
                        inst.enabled = true;
                    }
                }
                if let Some(def) = ctx.catalog.skill_by_name(SWD_FENGYIN_STATE_KEY) {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            }
            // 机关怨（U05 swd 第二十一批，规则 v196）：单体——使用者选一
            // 张手牌，与本体一起置于目标的武将牌上（离场区），其下一结束
            // 阶段强制获得（SwdJiguanyuanGain）；使用者摸 1。本体不进弃
            // 堆（随置牌走）。
            HostBehavior::SwdJiguanyuanCard => match self.stage {
                HostStage::Act => {
                    let Some(t) = self.target else {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive || !ctx.core.player(self.owner).alive {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    if ctx.core.player(self.owner).hand.is_empty() {
                        // 上游 step0 else：无手牌 event.finish（本体入弃堆）
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: SWD_JIGUANYUAN_CHOOSE,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        // 使用者手牌 + 本体 → 目标离场区；记录待获得实体
                        ctx.take_hand_card(self.owner, c);
                        ctx.core.player_mut(t).off_court.push(c);
                        if let Some(body) = self.card {
                            ctx.core.table.retain(|x| *x != body);
                            ctx.core.player_mut(t).off_court.push(body);
                            self.card = None;
                        }
                        let moved: Vec<crate::ids::CardId> =
                            ctx.core.player(t).off_court.iter().copied().collect();
                        ctx.core.player_mut(t).jiguanyuan_cards = moved;
                        if let Some(gdef) =
                            ctx.catalog.skill_by_name(SWD_JIGUANYUAN_GAIN_KEY)
                        {
                            ctx.core.skills.insert(
                                (t, gdef),
                                crate::state::SkillInstance {
                                    owner: t,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: false,
                                    rounds: 0,
                                },
                            );
                        }
                        ctx.log(LogEvent::CardsGiven {
                            from: self.owner,
                            to: t,
                            cards: ctx.core.player(t).jiguanyuan_cards.clone(),
                        });
                        ctx.draw_cards(self.owner, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 机关怨待获得（规则 v196）：拥有者结束阶段——离场区待获得牌
            // 移入手牌并移除自身（上游 jiguanyuan 技 phaseEnd forced 同口径）
            HostBehavior::SwdJiguanyuanGain => {
                let pending = ctx.core.player(self.owner).jiguanyuan_cards.clone();
                if !pending.is_empty() {
                    let off = &mut ctx.core.player_mut(self.owner).off_court;
                    let mut gained = Vec::new();
                    for c in &pending {
                        if let Some(pos) = off.iter().position(|x| x == c) {
                            gained.push(off.remove(pos));
                        }
                    }
                    ctx.core.player_mut(self.owner).hand.extend(gained);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: pending.clone(),
                    });
                    ctx.core.player_mut(self.owner).jiguanyuan_cards.clear();
                }
                if let Some(def) = ctx.catalog.skill_by_name(SWD_JIGUANYUAN_GAIN_KEY) {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            }
            // 旋风之刃（灵剑，规则 v198）：目标随机弃 1 张 he；本体入弃堆。
            HostBehavior::SwdXuanfengzhirenCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        let mut pool: Vec<CardId> = ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .copied()
                            .chain(ctx.core.player(t).equips.iter().copied())
                            .collect();
                        if !pool.is_empty() {
                            let k = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                            let c = pool.swap_remove(k);
                            if ctx.core.player(t).hand.contains(&c) {
                                ctx.discard_from_hand(t, c);
                            } else if let Some(pos) =
                                ctx.core.player(t).equips.iter().position(|x| *x == c)
                            {
                                ctx.core.player_mut(t).equips.remove(pos);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                                ctx.to_discard(c);
                                // 装备弃置也发弃置事件（上游 loseEnd 含
                                // 装备区牌；机关妖术补偿依赖此事件）
                                ctx.emit(EventData::CardsDiscarded {
                                    player: t,
                                    cards: vec![c],
                                });
                            }
                            ctx.log(LogEvent::CardDiscarded { player: t, cards: vec![c] });
                        }
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 中行虎甲（灵剑，规则 v198）：目标非 isMin——从牌堆+弃堆发现防
            // 具 Kind 实体装配（登记差异：上游装备定义池凭空造），再随机弃
            // 1 张手牌；本体入弃堆。
            HostBehavior::SwdZhongxinghujiaCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        let mut pool: Vec<CardId> = ctx
                            .core
                            .deck
                            .iter()
                            .chain(ctx.core.discard.iter())
                            .copied()
                            .filter(|x| ctx.catalog.card_kind(*x) == CardKind::Armor)
                            .collect();
                        let n = pool.len();
                        for i in (1..n).rev() {
                            let j = ctx.core.rng.gen_range(i as u32 + 1) as usize;
                            pool.swap(i, j);
                        }
                        if let Some(x) = pool.first().copied() {
                            if let Some(pos) = ctx.core.deck.iter().position(|y| *y == x) {
                                ctx.core.deck.remove(pos);
                            } else if let Some(pos) =
                                ctx.core.discard.iter().position(|y| *y == x)
                            {
                                ctx.core.discard.remove(pos);
                            }
                            ctx.core.player_mut(t).hand.push(x);
                            ctx.emit(EventData::Equipped { player: t, card: x, replaced: None });
                        }
                        let hand = ctx.core.player(t).hand.clone();
                        if !hand.is_empty() {
                            let k = ctx.core.rng.gen_range(hand.len() as u32) as usize;
                            let c = hand[k];
                            ctx.discard_from_hand(t, c);
                            ctx.log(LogEvent::CardDiscarded { player: t, cards: vec![c] });
                        }
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 星体改造（灵剑，规则 v198）：使用者（自目标）摸 1 + 手牌上限
            // -1 累加（其下一结束阶段重置，kernel 相位钩）；本体入弃堆。
            HostBehavior::SwdXingtigaizaoCard => {
                let p = ctx.core.player_mut(self.owner);
                p.xingtigaizao_count = p.xingtigaizao_count.saturating_add(1);
                ctx.draw_cards(self.owner, 1);
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 时间回溯（灵剑，规则 v198）：目标获得其装备区全部牌（装备技
            // 随迁撤销）；本体入弃堆。
            HostBehavior::SwdShijianhuisuCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        let es: Vec<CardId> = ctx.core.player(t).equips.iter().copied().collect();
                        for c in es {
                            if let Some(pos) =
                                ctx.core.player(t).equips.iter().position(|x| *x == c)
                            {
                                ctx.core.player_mut(t).equips.remove(pos);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                                ctx.core.player_mut(t).hand.push(c);
                            }
                        }
                        let hand: Vec<CardId> = ctx.core.player(t).hand.iter().copied().collect();
                        ctx.log(LogEvent::CardsDrawn { player: t, cards: hand });
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 紧急冷冻（灵剑，规则 v198）：目标≠使用者且未翻面——+2 护甲、
            // 翻面；仍翻面则冻结（不能使用牌也不能成为目标，翻回解除，
            // kernel TurnOvered 钩）；本体入弃堆。
            HostBehavior::SwdJinjilengdongCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive && !ctx.core.player(t).face_down {
                        // 护甲直写字段（灵雪同口径，无日志原语）
                        ctx.core.player_mut(t).hujia += 2;
                        ctx.core.player_mut(t).face_down = true;
                        ctx.emit(EventData::TurnOvered {
                            player: t,
                            face_down: true,
                        });
                        if ctx.core.player(t).face_down {
                            ctx.core.player_mut(t).jileng_active = true;
                        }
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 圣休号角（灵剑，规则 v198）：目标获嘲讽（相邻角色只能以其为
            // 杀目标；其下一阶段开始解除，kernel 相位钩）；本体入弃堆。
            HostBehavior::SwdShengxiuhaojiaoCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        ctx.core.player_mut(t).chaofeng_active = true;
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 隐秘离场（灵剑，规则 v198）：目标≠使用者——潜行（不能成为牌
            // 的目标；当前回合结束解除，kernel 回合收束钩）；本体入弃堆。
            HostBehavior::SwdYinmilichangCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        ctx.core.player_mut(t).yinmi_active = true;
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 机关妖术补偿（规则 v198）：非自己回合弃牌含装备 Kind 牌（近
            // 似失去装备区牌，登记差异：替换/获得离去不触发）→ 随机补获
            // 等量灵剑（牌堆+弃堆发现；无则不补，登记差异：上游全定义池
            // 凭空造）。状态不移除（持久防重面不变）。
            HostBehavior::SwdJiguanyaoGain => {
                // 触发帧即弃牌事件帧（栈顶）：读取其 CardsDiscarded 载荷
                let lost: Vec<CardId> = match ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data) {
                    Some(EventData::CardsDiscarded { cards, .. }) => cards.clone(),
                    _ => Vec::new(),
                };
                let num = lost
                    .iter()
                    .filter(|c| matches!(ctx.catalog.card_kind(**c).equip_slot(), Some(_)))
                    .count();
                if num > 0 {
                    let mut pool: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .chain(ctx.core.discard.iter())
                        .copied()
                        .filter(|x| ctx.catalog.card_kind(*x) == CardKind::Hslingjian)
                        .collect();
                    let n = pool.len();
                    for i in (1..n).rev() {
                        let j = ctx.core.rng.gen_range(i as u32 + 1) as usize;
                        pool.swap(i, j);
                    }
                    let take = num.min(pool.len());
                    let mut got = Vec::new();
                    for x in pool.into_iter().take(take) {
                        if let Some(pos) = ctx.core.deck.iter().position(|y| *y == x) {
                            ctx.core.deck.remove(pos);
                        } else if let Some(pos) =
                            ctx.core.discard.iter().position(|y| *y == x)
                        {
                            ctx.core.discard.remove(pos);
                        }
                        ctx.core.player_mut(self.owner).hand.push(x);
                        got.push(x);
                    }
                    if !got.is_empty() {
                        ctx.log(LogEvent::CardsDrawn { player: self.owner, cards: got });
                    }
                }
                Flow::Done
            }
            // 机关术（U05 swd 机关术批，规则 v199/ADR 0780）：自目标——
            // 每张有效装备受控合成：随机附着一张灵剑实体（PlayerState.
            // card_mods 表）。登记差异：①名称合成/数值升级仅入改造表（目
            // 录不可变）；②jiqi/yuchan 池暂缓，全部附着灵剑；③演示呈现暂
            // 缓。本体入弃堆。
            HostBehavior::SwdJiguanshuCard => {
                let valid: Vec<CardId> =
                    ctx.core.player(self.owner).equips.iter().copied().collect();
                let lingjian: Vec<CardId> = ctx
                    .catalog
                    .cards
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.kind == CardKind::Hslingjian)
                    .map(|(i, _)| i as CardId)
                    .collect();
                for equip in valid {
                    if lingjian.is_empty() || ctx.core.player(self.owner).card_mods.contains_key(&equip) {
                        continue;
                    }
                    let k = ctx.core.rng.gen_range(lingjian.len() as u32) as usize;
                    let attached = lingjian[k];
                    ctx.core
                        .player_mut(self.owner)
                        .card_mods
                        .insert(equip, attached);
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 天覆阵（zhenfa，规则 v200/T19a）：AOE 逐大国目标强制弃 1 张
            // he（targets 队列；目标自选牌 min 1 不可取消；self.target=当
            // 前询问目标）。全部结算完后本体入弃堆。
            HostBehavior::SwdTianfuzhenCard => match self.stage {
                HostStage::Act => {
                    if self.targets.is_empty() {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    let t = self.targets.remove(0);
                    self.target = Some(t);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .chain(ctx.core.player(t).equips.iter().copied())
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Yield;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "swd_tianfuzhen_discard",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        let t = self.target.unwrap_or(self.owner);
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                        } else if let Some(pos) =
                            ctx.core.player(t).equips.iter().position(|x| *x == c)
                        {
                            ctx.core.player_mut(t).equips.remove(pos);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                            ctx.to_discard(c);
                        }
                        ctx.log(LogEvent::CardDiscarded { player: t, cards: vec![c] });
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 七星阵（zhenfa，规则 v200/T19a）：阶段一 Act——逐围攻者弃 1
            //（SelectCard 询问，self.target=围攻者）；队列空后阶段二——
            // 视为对围攻者用杀（响应闪或 1 伤；登记差异：虚拟杀简化，无
            // 加成/不可无懈额外面），阶段二队列存 self.cards。本体入弃堆。
            HostBehavior::SwdQixingzhenCard => match self.stage {
                HostStage::Act => {
                    if !self.targets.is_empty() {
                        let t = self.targets.remove(0);
                        self.target = Some(t);
                        let options: Vec<ChoiceOption> = ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .copied()
                            .chain(ctx.core.player(t).equips.iter().copied())
                            .map(ChoiceOption::Card)
                        .collect();
                        if options.is_empty() {
                            return Flow::Yield;
                        }
                        self.stage = HostStage::SelectCard;
                        return Flow::WaitChoice {
                            observer: t,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: "swd_qixingzhen_discard",
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    if self.cards.is_empty() {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    let t = self.cards.remove(0) as u8;
                    self.target = Some(t);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Shan)
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        // 无闪：直接结算 1 点伤害（机关凤同款收口）
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                        self.stage = HostStage::Act;
                        return Flow::Yield;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "swd_qixingzhen_shan",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        let t = self.target.unwrap_or(self.owner);
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                        } else if let Some(pos) =
                            ctx.core.player(t).equips.iter().position(|x| *x == c)
                        {
                            ctx.core.player_mut(t).equips.remove(pos);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                            ctx.to_discard(c);
                        }
                        ctx.log(LogEvent::CardDiscarded { player: t, cards: vec![c] });
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                HostStage::TargetChoice => {
                    let t = self.target.unwrap_or(self.owner);
                    if matches!(ctx.answer().first(), Some(ChoiceOption::Card(_))) {
                        if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                            ctx.discard_from_hand(t, c);
                            ctx.log(LogEvent::CardDiscarded { player: t, cards: vec![c] });
                        }
                    } else {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 鸟翔阵（zhenfa，规则 v200/T19a）：逐合法目标响应闪或受 1 伤
            //（targets 队列；TargetChoice 询问，取消=伤害）。本体入弃堆。
            HostBehavior::SwdNiaoxiangzhenCard => match self.stage {
                HostStage::Act => {
                    if self.targets.is_empty() {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    let t = self.targets.remove(0);
                    self.target = Some(t);
                    self.stage = HostStage::TargetChoice;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Shan)
                        .map(ChoiceOption::Card)
                        .collect();
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "swd_niaoxiangzhen_shan",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let t = self.target.unwrap_or(self.owner);
                    if matches!(ctx.answer().first(), Some(ChoiceOption::Card(_))) {
                        if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                            ctx.discard_from_hand(t, c);
                            ctx.log(LogEvent::CardDiscarded { player: t, cards: vec![c] });
                        }
                    } else {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 长蛇阵（zhenfa，规则 v201/T19b）：队列中→全队列（含自己）各
            // 摸 1；否则与距离最近的友方换座成队列（swapSeat）；本体入弃堆。
            HostBehavior::SwdChangshezhenCard => {
                let inline_now = ctx.core.inline(ctx.catalog, self.owner);
                let prev_of_owner = ctx.core.alive_neighbors(self.owner).1;
                if inline_now {
                    for q in 0..ctx.core.players.len() as u8 {
                        if ctx.core.inline_with(ctx.catalog, self.owner, q) {
                            ctx.draw_cards(q, 1);
                        }
                    }
                } else if prev_of_owner.is_some() {
                    // 友方按与自己的座位距离排序取最近（上游 absolute 距离
                    // 排序同口径；seat_distance 即环上最短距离）
                    let mut friends: Vec<(u8, u8)> = (0..ctx.core.players.len() as u8)
                        .filter(|q| {
                            *q != self.owner
                                && ctx.core.player(*q).alive
                                && ctx.core.is_friend_of(ctx.catalog, self.owner, *q)
                        })
                        .map(|q| (q, ctx.core.seat_distance(self.owner, q)))
                        .collect();
                    friends.sort_by_key(|(_, d)| *d);
                    if let Some((nearest, _)) = friends.first().copied() {
                        let next_of_owner = ctx.core.next_alive(self.owner);
                        ctx.core.swap_seat(nearest, next_of_owner);
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 破阵诀（zhenfa，规则 v201/T19b）：n 次随机座位交换（n=存活
            // 数，上游 swapSeat(random, random) ×n 同口径）；本体入弃堆。
            HostBehavior::SwdPozhenjueCard => {
                let n = ctx.core.alive_players().count();
                for _ in 0..n {
                    let seats: Vec<u8> = ctx.core.alive_players().collect();
                    if seats.len() < 2 {
                        break;
                    }
                    let i = ctx.core.rng.gen_range(seats.len() as u32) as usize;
                    let j = ctx.core.rng.gen_range(seats.len() as u32) as usize;
                    ctx.core.swap_seat(seats[i], seats[j]);
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 机关妖术（U05 swd 第二十二批，规则 v197）：单体（range.global
            // 1）——从牌堆+弃堆装备实体中发现一张「目标空装备栏」子类型的
            // 装备，置入其手牌后 emit Equipped 走完整装配（commit_equip 同
            // 栏替换/装备技授予），并授予防重标记（登记差异：①发现池=牌
            // 堆+弃堆可见实体，上游=全部已定义装备牌凭空造牌，无候选则不
            // 装备；②上游 jiguanyaoshu_skill 的补偿触发面——失去装备区牌
            // 时补获 hslingjian 灵剑——依赖灵剑类型池，暂缓登记至机关术
            // 批，本切片仅保留防重面）。本体入弃堆。
            HostBehavior::SwdJiguanyaoshuCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        let occupied: Vec<Option<crate::catalog::EquipSlot>> = ctx
                            .core
                            .player(t)
                            .equips
                            .iter()
                            .map(|c| ctx.catalog.card_kind(*c).equip_slot())
                            .collect();
                        let mut pool: Vec<CardId> = ctx
                            .core
                            .deck
                            .iter()
                            .chain(ctx.core.discard.iter())
                            .copied()
                            .filter(|x| {
                                let slot = ctx.catalog.card_kind(*x).equip_slot();
                                matches!(slot, Some(_))
                                    && !occupied.contains(&slot)
                            })
                            .collect();
                        let n = pool.len();
                        for i in (1..n).rev() {
                            let j = ctx.core.rng.gen_range(i as u32 + 1) as usize;
                            pool.swap(i, j);
                        }
                        if let Some(x) = pool.first().copied() {
                            if let Some(pos) = ctx.core.deck.iter().position(|y| *y == x) {
                                ctx.core.deck.remove(pos);
                            } else if let Some(pos) =
                                ctx.core.discard.iter().position(|y| *y == x)
                            {
                                ctx.core.discard.remove(pos);
                            }
                            ctx.core.player_mut(t).hand.push(x);
                            ctx.emit(EventData::Equipped {
                                player: t,
                                card: x,
                                replaced: None,
                            });
                        }
                        for gkey in [SWD_JIGUANYAOSHU_STATE_KEY, SWD_JIGUANYAO_GAIN_KEY] {
                            if let Some(sdef) = ctx.catalog.skill_by_name(gkey) {
                                ctx.core.skills.insert(
                                    (t, sdef),
                                    crate::state::SkillInstance {
                                        owner: t,
                                        enabled: true,
                                        once_used: false,
                                        expires_end_of_turn: false,
                                        rounds: 0,
                                    },
                                );
                            }
                        }
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 机关妖术防重标记：纯状态无触发面（补偿触发暂缓）
            HostBehavior::SwdJiguanyaoState => Flow::Done,
        }
    }
}

/// 目录侧注册：行为 → 触发元数据 + 宿主行为实现（打包为 SkillDef）。
pub fn host_skill_def(
    name: String,
    behavior: HostBehavior,
) -> crate::catalog::SkillDef {
    // 目录技能名为 'static：经符号表 intern，同名名称全局只分配一次
    // （重复开局/启停循环内存恒定，见 content::names）。
    let name: &'static str = crate::content::names::intern(name);
    let limits = match behavior {
        HostBehavior::GujianFoodQingtuan | HostBehavior::GujianFoodLuyugeng => {
            crate::catalog::SkillLimits {
                once_per_turn: true,
                ..Default::default()
            }
        }
        // swd 装备主动技 usable:1（潜偷 v92 补执行/御夫录 v96）：出牌
        // 阶段限一次（上游 usable:1 同口径）
        HostBehavior::SwdShentouEquipSkill
        | HostBehavior::SwdYufuluEquipSkill
        | HostBehavior::SwdLangeguaiyiEquipSkill
        | HostBehavior::MtgYixialanRecast => {
            crate::catalog::SkillLimits {
                once_per_turn: true,
                ..Default::default()
            }
        }
        HostBehavior::MtgLinzhongjiantaStrike
        | HostBehavior::MtgYoulinDiscard
        | HostBehavior::WudianKangyin
        | HostBehavior::WudianLiangou
        | HostBehavior::WudianQianggu
        | HostBehavior::WudianYinbo
        | HostBehavior::XianjianGuiyuan
        | HostBehavior::SwdDuanyue
        | HostBehavior::SwdXingdian
        | HostBehavior::SwdLianyaohuShouna
        | HostBehavior::SwdLianyaohuLianhua
        | HostBehavior::SwdShennongdingCard
        | HostBehavior::SwdKunlunjingcCard
        | HostBehavior::SwdSifeizhenmianCard => {
            crate::catalog::SkillLimits {
                once_per_turn: true,
                ..Default::default()
            }
        }
        _ => crate::catalog::SkillLimits::default(),
    };
    crate::catalog::SkillDef {
        name,
        trigger: behavior.trigger_meta(),
        limits,
        behavior: Box::new(HostSkillBehavior { name: name.to_string(), behavior }),
    }
}

impl HostMachine {
    /// 机关凤未响应路径（规则 v195）：目标全部技能实例禁用 + 授予封印
    /// 状态（其下一阶段开始由 SwdFengyinState 解除）+ 受 1 点伤害（上游
    /// addTempSkill fengyin + damage 同序；登记差异：原生无锁定技区分）。
    fn seal_and_damage(&self, ctx: &mut Ctx<'_>) {
        let Some(t) = self.target else { return; };
        if let Some(fdef) = ctx.catalog.skill_by_name(SWD_FENGYIN_STATE_KEY) {
            let keys: Vec<(PlayerId, crate::ids::SkillDefId)> = ctx
                .core
                .skills
                .iter()
                .map(|((p, d), _)| (*p, *d))
                .filter(|(p, d)| *p == t && *d != fdef)
                .collect();
            for k in keys {
                if let Some(inst) = ctx.core.skills.get_mut(&k) {
                    inst.enabled = false;
                }
            }
            ctx.core.skills.insert(
                (t, fdef),
                crate::state::SkillInstance {
                    owner: t,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
        }
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            Some(self.owner),
            t,
            1,
            crate::event::DamageNature::Normal,
            self.card,
        )));
    }
    /// 石会粉：置目标禁用状态（本回合无法对其他角色用牌；
    /// expires_end_of_turn 于其回合结束清除）。
    fn grant_shihuifen_ban(&mut self, ctx: &mut Ctx<'_>, target: PlayerId) {
        if let Some(def) = ctx.catalog.skill_by_name(GJ_FOOD_SHIHUIFEN_BAN_KEY) {
            ctx.core.skills.insert(
                (target, def),
                crate::state::SkillInstance {
                    owner: target,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: true,
                    rounds: 0,
                },
            );
        }
    }
}

/// 食物一次性状态：立即移除监听技实例（咕咚羹「下次」语义）。
fn food_remove(ctx: &mut Ctx<'_>, owner: PlayerId, def_key: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(def_key) {
        ctx.core.skills.remove(&(owner, def));
    }
}

/// 食物状态倒计时：效果触发点扣减，归零移除监听技实例。
fn food_tick(ctx: &mut Ctx<'_>, owner: PlayerId, def_key: &str) {
    let Some(def) = ctx.catalog.skill_by_name(def_key) else {
        return;
    };
    let remove = match ctx.core.skills.get_mut(&(owner, def)) {
        Some(inst) => {
            inst.rounds = inst.rounds.saturating_sub(1);
            inst.rounds == 0
        }
        None => return,
    };
    if remove {
        ctx.core.skills.remove(&(owner, def));
        ctx.core.skills.remove(&(owner, def));
    }
}
/// 地附主动技同步（mtg 第五批 v104）：换地时撤销旧地附技、授予新地附
/// 技（当前仅依夏兰附重铸技；上游 addGlobalSkill+temp 簿记的实例化对
/// 应物——once_used 随拥有者回合开始重置为既有路径）。
pub(crate) fn sync_land_skill(ctx: &mut Ctx<'_>, owner: PlayerId, new_kind: CardKind) {
    // 地附技映射（当前三例）：依夏兰→重铸、林中尖塔→塔击、幽林→发现
    let key_of = |k: CardKind| match k {
        CardKind::Yixialan => Some(MTG_YIXIALAN_RECAST_KEY),
        CardKind::Linzhongjianta => Some(MTG_LINZHONGJIANTA_SKILL_KEY),
        CardKind::Youlin => Some(MTG_YOULIN_DISCOVER_KEY),
        _ => None,
    };
    let old_key = ctx.core.current_land.and_then(key_of);
    let new_key = key_of(new_kind);
    if let Some(k) = old_key {
        if old_key != new_key {
            if let Some(d) = ctx.catalog.skill_by_name(k) {
                ctx.core.skills.remove(&(owner, d));
            }
        }
    }
    if let Some(k) = new_key {
        if let Some(d) = ctx.catalog.skill_by_name(k) {
            ctx.core.skills.insert(
                (owner, d),
                crate::state::SkillInstance {
                    owner,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
        }
    }
}





/// 牌种固有的装备技能（引擎按固定键查找）；包数据声明的装备技追加其后。

pub fn card_kind_host_skills(kind: crate::catalog::CardKind) -> &'static [HostSkillDecl] {
    match kind {
        CardKind::Jinlianzhu => &[HostSkillDecl { name: GJ_JINLIANZHU_KEY, behavior: HostBehavior::JinlianzhuCancel }],
        CardKind::Tanhuadong => &[HostSkillDecl { name: GJ_FOOD_TANHUADONG_KEY, behavior: HostBehavior::GujianFoodTanhuadong }],
        CardKind::Yougeng => &[HostSkillDecl { name: GJ_FOOD_YOUGENG_KEY, behavior: HostBehavior::GujianFoodYougeng }],
        CardKind::Qingtuan => &[HostSkillDecl { name: GJ_FOOD_QINGTUAN_KEY, behavior: HostBehavior::GujianFoodQingtuan }],
        CardKind::Yuanbaorou => &[HostSkillDecl { name: GJ_FOOD_YUANBAOROU_KEY, behavior: HostBehavior::GujianFoodYuanbaorou }],
        CardKind::Gudonggeng => &[HostSkillDecl { name: GJ_FOOD_GUDONGGENG_KEY, behavior: HostBehavior::GujianFoodGudonggeng }],
        CardKind::Mapodoufu => &[HostSkillDecl { name: GJ_FOOD_MAPODOUFU_KEY, behavior: HostBehavior::GujianFoodMapodoufu }],
        CardKind::Chunbing => &[HostSkillDecl { name: GJ_FOOD_CHUNBING_KEY, behavior: HostBehavior::GujianFoodChunbing }],
        CardKind::Liyutang => &[HostSkillDecl { name: GJ_FOOD_LIYUTANG_KEY, behavior: HostBehavior::GujianFoodLiyutang }],
        CardKind::Molicha => &[HostSkillDecl { name: GJ_FOOD_MOLICHA_KEY, behavior: HostBehavior::GujianFoodMolicha }],
        CardKind::Xiajiao => &[HostSkillDecl { name: GJ_FOOD_XIAJIAO_KEY, behavior: HostBehavior::GujianFoodXiajiao }],
        CardKind::Mizhilianou => &[HostSkillDecl { name: GJ_FOOD_MIZHLIANOU_KEY, behavior: HostBehavior::GujianFoodMizhilianou }],
        CardKind::Luyugeng => &[HostSkillDecl { name: GJ_FOOD_LUYUGENG_KEY, behavior: HostBehavior::GujianFoodLuyugeng }],
        CardKind::Ziyangdan => &[HostSkillDecl { name: GJ_FOOD_ZIYANGDAN_KEY, behavior: HostBehavior::GujianFoodZiyangdan }],
        CardKind::Yunvyuanshen => &[HostSkillDecl { name: GJ_YUNVYUANSHEN_LISTENER_KEY, behavior: HostBehavior::GujianYunvyuanshen }],
        CardKind::Shihuifen => &[HostSkillDecl { name: GJ_FOOD_SHIHUIFEN_BAN_KEY, behavior: HostBehavior::GujianFoodShihuifenBan }],
        CardKind::Mutoumianju => &[HostSkillDecl { name: GJ_MUTOU_SKILL_KEY, behavior: HostBehavior::GujianMutouEquipSkill }],
        CardKind::Gjyuheng => &[HostSkillDecl { name: GJ_YUHENG_SKILL_KEY, behavior: HostBehavior::GujianYuhengEquipSkill }],
        CardKind::Qinpendayu => &[HostSkillDecl { name: GW_QINPEN_STATE_KEY, behavior: HostBehavior::GwentQinpenState }],
        CardKind::Birinongwu => &[HostSkillDecl { name: GW_BIRIN_STATE_KEY, behavior: HostBehavior::GwentBirinState }],
        CardKind::Ciguhanshuang => &[HostSkillDecl { name: GW_CIGU_STATE_KEY, behavior: HostBehavior::GwentCiguState }],
        CardKind::Zumoshoukao => &[HostSkillDecl { name: GW_FENGYIN_STATE_KEY, behavior: HostBehavior::GwentFengyinState }],
        CardKind::Kunenfayin => &[HostSkillDecl { name: GW_KUNEN_STATE_KEY, behavior: HostBehavior::GwentKunenState }],
        CardKind::Mianju => &[HostSkillDecl { name: HUANLE_MIANJI_KEY, behavior: HostBehavior::HuanleMianjuCard }],
        CardKind::Monkey => &[HostSkillDecl { name: HUANLE_MONKEY_KEY, behavior: HostBehavior::HuanleMonkeyCard }],
        CardKind::Xuelunyang => &[HostSkillDecl { name: HUANLE_XUELUNYANG_KEY, behavior: HostBehavior::HuanleXuelunyangCard }],
        CardKind::Langeguaiyi => &[HostSkillDecl { name: SWD_LANGEGUAIYI_SKILL_KEY, behavior: HostBehavior::SwdLangeguaiyiEquipSkill }],
        CardKind::Shentoumianju => &[HostSkillDecl { name: SWD_SHENTOU_SKILL_KEY, behavior: HostBehavior::SwdShentouEquipSkill }],
        CardKind::Yufulu => &[HostSkillDecl { name: SWD_YUFULU_SKILL_KEY, behavior: HostBehavior::SwdYufuluEquipSkill }],
        CardKind::Longlushanfeng => &[HostSkillDecl { name: MTG_LONGSHANFENG_KEY, behavior: HostBehavior::MtgLongshanfengLand }],
        CardKind::Haidao => &[HostSkillDecl { name: MTG_HAIDAO_KEY, behavior: HostBehavior::MtgHaidaoLand }],
        CardKind::Shuimomuxue => &[HostSkillDecl { name: MTG_SHUIMOMUXUE_KEY, behavior: HostBehavior::MtgShuimomuxueLand }],
        CardKind::Feixu => &[HostSkillDecl { name: MTG_FEIXU_KEY, behavior: HostBehavior::MtgFeixuLand }],
        CardKind::Yixialan => &[
            HostSkillDecl { name: MTG_YIXIALAN_KEY, behavior: HostBehavior::MtgYixialanLand },
            HostSkillDecl { name: MTG_YIXIALAN_RECAST_KEY, behavior: HostBehavior::MtgYixialanRecast },
        ],
        CardKind::Shamolvzhou => &[HostSkillDecl { name: MTG_SHAMOLVZHOU_KEY, behavior: HostBehavior::MtgShamolvzhouLand }],
        CardKind::Linzhongjianta => &[
            HostSkillDecl { name: MTG_LINZHONGJIANTA_KEY, behavior: HostBehavior::MtgLinzhongjiantaLand },
            HostSkillDecl { name: MTG_LINZHONGJIANTA_SKILL_KEY, behavior: HostBehavior::MtgLinzhongjiantaStrike },
        ],
        CardKind::Bingheyaosai => &[HostSkillDecl { name: MTG_BINGHEYAOSAI_KEY, behavior: HostBehavior::MtgBingheyaosaiLand }],
        CardKind::Youlin => &[
            HostSkillDecl { name: MTG_YOULIN_KEY, behavior: HostBehavior::MtgYoulinLand },
            HostSkillDecl { name: MTG_YOULIN_DISCOVER_KEY, behavior: HostBehavior::MtgYoulinDiscard },
        ],
        CardKind::Lindixiliu => &[HostSkillDecl { name: MTG_LINDIXILIU_KEY, behavior: HostBehavior::MtgLindixiliuLand }],
        CardKind::Jiguan => &[HostSkillDecl { name: SWD_GOUHUNLUO_SUB_KEY, behavior: HostBehavior::SwdGouhunluoSub }, HostSkillDecl { name: SWD_FENGYIN_STATE_KEY, behavior: HostBehavior::SwdFengyinState }, HostSkillDecl { name: SWD_JIGUANYUAN_GAIN_KEY, behavior: HostBehavior::SwdJiguanyuanGain }, HostSkillDecl { name: SWD_JIGUANYAOSHU_STATE_KEY, behavior: HostBehavior::SwdJiguanyaoState }, HostSkillDecl { name: SWD_JIGUANYAO_GAIN_KEY, behavior: HostBehavior::SwdJiguanyaoGain }],
        CardKind::Lianyaohu => &[HostSkillDecl { name: "lianyaohu_shouna", behavior: HostBehavior::SwdLianyaohuShouna }, HostSkillDecl { name: "lianyaohu_lianhua", behavior: HostBehavior::SwdLianyaohuLianhua }],
        CardKind::Shennongding => &[HostSkillDecl { name: "shennongding", behavior: HostBehavior::SwdShennongdingCard }],
        CardKind::Nvwashi => &[HostSkillDecl { name: "nvwashi", behavior: HostBehavior::SwdNvwashiGuard }],
        CardKind::Kongdongyin => &[HostSkillDecl { name: "kongdongyin", behavior: HostBehavior::SwdKongdongyinGuard }],
        CardKind::Guilingzhitao => &[HostSkillDecl { name: "nigong_accumulate", behavior: HostBehavior::SwdNigongAccumulate }, HostSkillDecl { name: "nigong_strike", behavior: HostBehavior::SwdNigongStrike }],
        CardKind::Sifeizhenmian => &[HostSkillDecl { name: "yiluan", behavior: HostBehavior::SwdSifeizhenmianCard }],
        CardKind::Qinglongzhigui => &[HostSkillDecl { name: "qinglongzhigui", behavior: HostBehavior::SwdQinglongzhiguiHeld }],
        CardKind::Zhuquezhizhang => &[HostSkillDecl { name: "zhuquezhizhang", behavior: HostBehavior::SwdZhuquezhizhangHeld }],
        CardKind::Baishouzhihu => &[HostSkillDecl { name: "baishouzhihu", behavior: HostBehavior::SwdBaishouzhihuHeld }],
        CardKind::Xuanwuzhihuang => &[HostSkillDecl { name: "xuanwuzhihuang", behavior: HostBehavior::SwdXuanwuzhihuangHeld }],
        CardKind::Huanglinzhicong => &[HostSkillDecl { name: "huanglinzhicong", behavior: HostBehavior::SwdHuanglinzhicongHeld }],
        CardKind::Shencaojie => &[HostSkillDecl { name: "shencaojie", behavior: HostBehavior::SwdShencaojieGuard }],
        CardKind::Yuchanqian
        | CardKind::Yuchankun
        | CardKind::Yuchanzhen
        | CardKind::Yuchanxun
        | CardKind::Yuchankan
        | CardKind::Yuchanli
        | CardKind::Yuchangen
        | CardKind::Yuchandui => &[HostSkillDecl { name: "yuchan_swap", behavior: HostBehavior::SwdYuchanSwap }],
        CardKind::Kunlunjingc => &[HostSkillDecl { name: "kunlunjingc", behavior: HostBehavior::SwdKunlunjingcCard }],
        CardKind::Donghuangzhong => &[HostSkillDecl { name: "donghuangzhong", behavior: HostBehavior::SwdDonghuangzhongCard }],
        _ => &[],
    }
}

/// 牌种固有的装备技能（引擎按固定键查找）；包数据声明的装备技追加其后。
pub fn card_kind_equip_skills(kind: crate::catalog::CardKind) -> &'static [&'static str] {
    match kind {
        CardKind::Mutoumianju => &[GJ_MUTOU_SKILL_KEY],
        CardKind::Gjyuheng => &[GJ_YUHENG_SKILL_KEY],
        CardKind::Monkey => &[HUANLE_MONKEY_KEY],
        CardKind::Xuelunyang => &[HUANLE_XUELUNYANG_KEY],
        CardKind::Mianju => &[HUANLE_MIANJI_KEY],
        _ => &[],
    }
}





/// 器族手持门控（swd 卡牌面补切批⑩）：持有者手牌含对应 kind 实体。
fn hand_has_kind(ctx: &crate::catalog::QueryCtx<'_>, owner: u8, kind: CardKind) -> bool {
    ctx.state
        .player(owner)
        .hand
        .iter()
        .any(|c| ctx.catalog.card_kind(*c) == kind)
}
