// ---------------------------------------------------------------------------
// 首组官方技能（N08，ADR 0712）：判定主题——张角「鬼道」+ 郭嘉「天妒」
// 规则来源：标准包规则文本；鬼道改判流程对齐旧线 `query/mod.rs`
// `judgment_black_rewrite_cards`（黑色手牌/装备）与 `command/judge.rs`
// `judgment_rewriters`（自判定阶段拥有者起座位序、逐人一次）——
// 原生本批改判资格仅取黑色手牌（装备区牌打出未接，差异登记）。
// 天妒：旧线无实现（仅 `characters.rs::TIANDU` 常量），按标准包规则文本
// 「每当一名角色的判定牌生效后，你可以将之收入手牌」实现。
// ---------------------------------------------------------------------------

/// 张角「鬼道」：判定牌生效前，可打出一张黑色手牌代替之。
pub const GUI_DAO: &str = "鬼道";
/// 郭嘉「天妒」：一名角色的判定牌生效后，可将之收入手牌。
pub const TIAN_DU: &str = "天妒";

pub const SAN_CAI: &str = "散财";
pub const YI_SHANG: &str = "移伤";
pub const SUI_DUN: &str = "碎盾";
pub const SU_QING: &str = "肃清";
pub const SHOU_YI: &str = "授艺";
pub const JU_SHOU: &str = "据守";
/// 乐进「骁果」：其他角色的结束阶段开始时，可弃一张基本手牌，对其
/// 造成 1 点伤害（P2/N08 标准缺失将，2026-10-07）。
pub const XIAO_GUO: &str = "骁果";
/// 潘凤「狂斧」：出牌阶段用【杀】对目标造成伤害后，若其体力值小于你，
/// 你摸两张牌，否则你失去 1 点体力（每回合限一次）。
pub const KUANG_FU: &str = "狂斧";
/// 华雄「耀武」：锁定技，你受到红色【杀】的伤害时，伤害来源选择摸一
/// 张牌或回复 1 点体力。
pub const YAO_WU: &str = "耀武";
/// 甘夫人「淑慎」：回复体力后，可令一名其他角色摸牌（有手牌摸 1，
/// 无手牌摸 2）（P2/N08 标准缺失将第二批，2026-10-07）。
pub const SHU_SHEN: &str = "淑慎";
/// 甘夫人「神智」：准备阶段（原生锚点：判定阶段开始）可弃置全部手牌，
/// 若数量不少于体力值则回复 1 点体力。
pub const SHEN_ZHI: &str = "神智";
/// 李典「恂恂」：摸牌阶段开始时，观看牌堆顶四张，将其中两张置于牌堆顶
/// （先选先摸）其余置于牌堆底，然后正常摸牌（P2/N08 标准缺失将第二批）。
pub const XUN_XUN: &str = "恂恂";
/// 李典「忘隙」：对其他角色造成伤害或受到其伤害后，可与该角色各摸一张牌。
pub const WANG_XI: &str = "忘隙";
/// 公孙瓒「义从」（refresh 改版，P2/N08 标准缺失将第三批）：锁定技——
/// 你计算与其他角色的距离 -1；其他角色计算与你的距离 +1（你的体力值
/// 不大于 2 时）。
pub const YI_CONG: &str = "义从";
/// 伊籍「机捷」：出牌阶段限一次，取牌堆底一张牌交给一名角色
/// （P2/N08 标准缺失将第四批）。
pub const JI_JIE: &str = "机捷";
/// 伊籍「急援」：一名角色进入濒死状态时，可令其摸一张牌（登记差异：
/// 仅濒死臂——上游赠牌/失去牌臂需要 giver 跟踪面，未建模）。
pub const JI_YUAN: &str = "急援";
/// 丈八蛇矛（配套主动技，批③-b 第七把）：两张手牌当【杀】使用。
pub const ZHANGBA: &str = "丈八蛇矛";
/// 雌雄双股剑（批③-b 第八把，equip1 随牌种接线）：对异性目标出杀后
/// 目标弃一张手牌或令使用者摸一张牌。
pub const CI_XIONG: &str = "雌雄双股剑";
/// 袁术「妄尊」（P2/N08 标准缺失将收尾批）：锁定技——其他角色的回合
/// 开始时，若其体力值大于你，你摸一张牌；若该角色为主公，你再摸一张
/// 且其手牌上限 -1 直到回合结束（印记技「妄尊印」承载）。
pub const WANG_ZUN: &str = "妄尊";
/// 袁术「妄尊」的手牌上限印记（隐藏技能，弃牌阶段结算 -1）。
pub const WANG_ZUN_MARK: &str = "妄尊印";
/// 袁术「同疾」：锁定技——其他角色使用【杀】指定唯一目标时（你不在
/// 目标内、你在其攻击范围内），其须选择弃置一张牌或取消：弃置则该杀
/// 改为指向你（登记差异：目标合法域按上游 inRange=目标对你的攻击范围）。
pub const TONG_JI: &str = "同疾";

/// 司马懿「鬼才」：一名角色的判定牌生效前，可打出一张**任意花色**手牌代替之
/// （区别于鬼道的黑色手牌/装备牌）。
pub const GUI_CAI: &str = "鬼才";
/// 司马懿「反馈」：受到 1 点伤害后，可获得伤害来源的一张牌（手牌或装备）。
pub const FAN_KUI: &str = "反馈";
/// 曹操「奸雄」：受到伤害后，可获得造成此伤害的牌。
pub const JIAN_XIONG: &str = "奸雄";
/// 貂蝉「闭月」：结束阶段开始时，摸一张牌。
pub const BI_YUE: &str = "闭月";
/// 甄姬「洛神」：判定阶段开始时，可进行判定——黑色则获得此牌并再次判定
/// （循环至红桃或放弃）。
pub const LUO_SHEN: &str = "洛神";
/// 关羽「武圣」：可将任意红色手牌当杀使用。
pub const WU_SHENG: &str = "武圣";
/// 赵云「龙胆」：可将杀当闪、闪当杀使用。
pub const LONG_DAN: &str = "龙胆";
/// 大乔「国色」：可将方块牌当【乐不思蜀】使用。
pub const GUO_SE: &str = "国色";
/// 吕布「无双」：你的杀需连续两张闪抵消；与你决斗的角色每次需打出两张杀。
pub const WU_SHUANG: &str = "无双";
/// 周瑜「英姿」：摸牌阶段摸牌时额外摸一张牌。
pub const YING_ZI: &str = "英姿";
/// 孙尚香「枭姬」：失去装备区里的一张牌时，摸两张牌。
pub const XIAO_JI: &str = "枭姬";
/// 华佗「青囊」：出牌阶段限一次，弃一张手牌并令一名已受伤的其他角色回复 1 点体力。
pub const QING_NANG: &str = "青囊";
/// 黄月英「集智」：使用普通锦囊时摸一张牌。
pub const JI_ZHI: &str = "集智";
/// 吕蒙「克己」：若本回合未使用杀，则无需弃牌。
pub const KE_JI: &str = "克己";
/// 张飞「咆哮」：出牌阶段使用杀没有次数限制。
pub const PAO_XIAO: &str = "咆哮";
/// 甘宁「奇袭」：可将黑色牌当【过河拆桥】使用。
pub const QI_XI: &str = "奇袭";
/// 黄盖「苦肉」：出牌阶段，可失去 1 点体力，摸两张牌（每阶段限一次）。
pub const KU_ROU: &str = "苦肉";
/// 孙权「制衡」：出牌阶段限一次，可弃置任意张手牌，然后摸等量的牌。
pub const ZHI_HENG: &str = "制衡";
/// 郭嘉「遗计」：受到 1 点伤害后，摸两张牌，然后可将至多两张手牌交给其他玩家。
pub const JI_JI: &str = "遗计";
/// 夏侯惇「刚烈」：受到伤害后，可进行判定：若判定牌不为红桃，伤害来源受到你造成的 1 点伤害。
pub const GANG_LIE: &str = "刚烈";
/// 张辽「突袭」：摸牌阶段开始时，可放弃摸牌，改为获得至多两名其他角色的各一张手牌。
pub const TU_XI: &str = "突袭";
/// 许褚「裸衣」：摸牌阶段开始时，可少摸一张牌，然后本回合【杀】或【决斗】造成的伤害 +1。
pub const LUO_YI: &str = "裸衣";
/// 甄姬「倾国」：可将装备区里的黑色牌当【闪】使用（响应杀）。
pub const QING_GUO: &str = "倾国";
/// 黄月英「奇才」：锁定技，使用任何锦囊牌无距离限制。
pub const QI_CAI: &str = "奇才";
/// 孙权「救援」：主公技，锁定技，处于濒死状态时，其他角色对你使用的【桃】回复量 +1。
pub const JIU_YUAN: &str = "救援";
/// 华佗「急救」：锁定技，濒死结算时可将红色手牌当【桃】使用。
pub const JI_JIU: &str = "急救";
/// 曹操「护驾」：主公技，需要出闪时，其他存活角色的闪可代出（原生简化：跨玩家借闪响应）。
pub const HU_JIA: &str = "护驾";
/// 大乔「流离」：成为【杀】的目标时，可弃一张手牌将此杀转移给另一名其他角色。
pub const LIU_LI: &str = "流离";
/// 孙尚香「结姻」：出牌阶段限一次，弃两张手牌，与一名受伤的其他男性角色各回复 1 点体力
/// （v202 对齐上游 filterTarget；旧「已婚=装备区有牌」登记撤销）。
pub const JIE_YIN: &str = "结姻";
/// 周瑜「反间」：出牌阶段限一次，一名其他角色选择你的一张手牌获得之，然后受到你造成的 1 点伤害
/// （v203 对齐上游：目标先选花色，花色不符才受伤；旧登记撤销）。
pub const FAN_JIAN: &str = "反间";
/// 貂蝉「离间」：出牌阶段限一次，弃一张手牌，令两名其他角色决斗
/// （原生简化：不限男性角色，见 ADR 0770 登记表）。
pub const LI_JIAN: &str = "离间";
/// 诸葛亮「观星」：回合开始（判定阶段）时，可观看牌库顶 X 张（X=存活数且至多 5），
/// 任意张以任意顺序置于牌库顶，其余置于牌库底。
pub const GUAN_XING: &str = "观星";
/// 诸葛亮「空城」：锁定技，没有手牌时不能成为【杀】和【决斗】的目标。
pub const KONG_CHENG: &str = "空城";
/// 陆逊「谦逊」：锁定技，不能成为【乐不思蜀】的目标（上游无【顺手牵羊】，挂账清单）。
pub const QIAN_XUN: &str = "谦逊";
/// 陆逊「连营」：失去最后一张手牌后（出牌/响应/给出路径），可摸一张牌。
pub const LIAN_YING: &str = "连营";
/// 刘备「仁德」：出牌阶段限一次，可将任意张手牌交给一名其他角色，
/// 若给出的牌不少于两张，你回复 1 点体力。
pub const REN_DE: &str = "仁德";
/// 刘备「激将」主公技：出牌阶段限一次，可请求一名已声明蜀势力的其他角色
/// 打出一张杀（经其确认后置入你的手牌，视为由你使用——原生简化：获得后
/// 走正常出杀流程，见 ADR 0760）。
pub const JI_JIANG: &str = "激将";
/// 马超「马术」：锁定技，计算与其他角色的距离时始终 -1。
pub const MA_SHU: &str = "马术";
/// 凶兽（Boss 模式共有技，规则 v208）：锁定技三面——你使用【杀】对体力
/// 值小于你的角色伤害 +1；你与其他角色距离 -1；你不能被翻面。
pub const BOSS_XIONGSHOU: &str = "凶兽";
/// 无脏（Boss 混沌，规则 v209）：锁定技两面——摸牌阶段的摸牌基数为
/// max(5, 体力值一半向上取整前的整除值)（即 2+max(5,floor(hp/2))-2）；
/// 手牌上限基数为 0（体力贡献清零，保留其余加成）。
pub const BOSS_WUZANG: &str = "无脏";
/// 相德（Boss 混沌，规则 v210）：锁定技——其他角色（装备区有武器）对
/// 你造成伤害时，此伤害 +1。
pub const BOSS_XIANGDE: &str = "相德";
/// 隐贼（Boss 混沌，规则 v210）：锁定技，体力值首次减至一半或更少时激
/// 活；你无手牌时，其他角色对你造成伤害后弃置其一张手牌。
pub const BOSS_YINZEI: &str = "隐贼";
/// 逐厄（Boss 穷奇，规则 v211）：锁定技——任何人受到伤害后（来源存活
/// 且非你），你与来源各摸一张牌。
pub const BOSS_ZHUE: &str = "逐厄";
/// 覆泰（Boss 穷奇，规则 v212）：锁定技——准备阶段（原生锚点：判定阶
/// 段开始，神智同口径）开始时，所有受伤角色按座位序各回复 1 点体力。
pub const BOSS_FUTAI: &str = "覆泰";
/// 延妒（Boss 穷奇，规则 v213）：锁定技，体力值首次减至一半或更少时激
/// 活；其他角色结束阶段开始时，若其本回合未造成伤害且有牌，你获得其
/// 一张手牌。
pub const BOSS_YANDU: &str = "延妒";
/// 贪欲（Boss 饕餮，规则 v215）：锁定技两面——跳过你的弃牌阶段；结束
/// 阶段若你的手牌数为全场最多（不少于任何其他存活角色），失去 1 点体
/// 力。
pub const BOSS_TANYU: &str = "贪欲";
/// 藏目（Boss 饕餮，规则 v216）：锁定技——摸牌阶段的摸牌基数改为存活
/// 角色数（固定摸 2 后补额外 存活数-2）。
pub const BOSS_CANGMU: &str = "藏目";
/// 积财（Boss 饕餮，规则 v216）：锁定技，体力值首次减至一半或更少时激
/// 活；一名角色回复体力后，你与其各摸一张牌（自己回复则自己摸两张）。
pub const BOSS_JICAI: &str = "积财";
/// 拟态（Boss 梼杌，规则 v217）：锁定技两面——防止你于回合内受到的伤
/// 害；你于回合外受到的火属性伤害 +1。
pub const BOSS_NITAI: &str = "拟态";
/// 冥顽（Boss 梼杌，规则 v219）：锁定技——你于回合内使用牌对其他角色
/// 造成伤害后，你于此回合内使用牌只能指定你与这些角色为目标，且你每
/// 使用一张牌，摸一张牌。
pub const BOSS_MINWAN: &str = "冥顽";
/// 乱常（Boss 梼杌，规则 v218）：锁定技，体力值首次减至一半或更少时激
/// 活；回合开始（判定阶段开始锚点）视为使用【南蛮入侵】，回合结束视为
/// 使用【万箭齐发】（目标=全部其他存活角色）。
pub const BOSS_LUANCHANG: &str = "乱常";
/// 催毒（Boss 书精，规则 v220）：锁定技——你对敌方角色造成伤害后，若
/// 其没有「中毒」，你令其获得「中毒」（登记：敌方以受伤者≠持技者近似，
/// 木神勾芒联动不在场跳过）。
pub const BOSS_CUIDUI: &str = "催毒";
/// 中毒（Boss 书精授技，规则 v220）：锁定技——回合开始（判定阶段开始
/// 锚点）判定，非红桃受 1 点无来源伤害，非黑桃则失去此技。
pub const BOSS_ZHONGDU: &str = "中毒";
/// 妖兽（Boss 神兽共有技，规则 v221）：锁定技——你与其他角色计算距离
/// 时 -2（上游 boss_yaoshou mod globalFrom distance-2 同口径）。
pub const BOSS_YAOSHOU: &str = "妖兽";
/// 毒妒（Boss 相柳，规则 v223）：锁定技——你受到伤害时，伤害来源获得
/// 1 枚「蛇毒」标记（你自身不会拥有）；【桃】视为【杀】（mod 面暂缓
/// 登记）。蛇毒标记：其回合开始弃 X 张牌或失去 X 点体力后递减。
pub const BOSS_DUQU: &str = "毒妒";
/// 邪酬（Boss 相柳，规则 v223）：锁定技，体力值首次减至一半或更少时激
/// 活；除你之外的角色使用「桃」或「酒」时，获得 1 枚「蛇毒」标记。
pub const BOSS_ECHOU: &str = "邪酬";
/// 蛇毒标记结算（毒妒/邪酬授予的标记，规则 v223）：锁定技——回合开始
/// 弃 X 张牌或失去 X 点体力后标记-1（X=标记数）。
pub const BOSS_SHEDU: &str = "蛇毒";
/// 兽九（Boss 相柳，规则 v222）：锁定技——手牌上限恒为 9；出牌阶段与
/// 回合结束时手牌补至 9；跳过摸牌阶段（摸牌基数 0）。
pub const BOSS_JIUSHOU: &str = "兽九";
/// 兵先（Boss 朱厌，规则 v224）：锁定技——其他角色的回合结束时，若其
/// 本回合内没有使用【杀】（登记：出牌阶段口径），视为对其使用一张
/// 【杀】。
pub const BOSS_BINGXIAN: &str = "兵先";
/// 巨源（Boss 朱厌，规则 v226）：锁定技——体力上限+5（由 boss_zhuyan
/// maxHp30 表意，登记同穷奇）；出牌阶段体力低于上次回合结束时体力时，
/// 【杀】可额外指定 1 个目标。
pub const BOSS_JUYUAN: &str = "巨源";
/// 招祸（Boss 毕方，规则 v227）：锁定技两面——你造成的所有伤害均视为
/// 火属性；你免疫所有火属性伤害（登记差异：你的回合中其他角色防具无
/// 效面挂点分散，未建模）。
pub const BOSS_ZHAOHUO: &str = "招祸";
/// 封冻（Boss 应昭，规则 v228）：锁定技——你的回合开始时，所有其他角
/// 色获得「封印」（本回合无法使用手牌，回合结束解除）。
pub const BOSS_FENGDONG: &str = "封冻";
/// 巡游（Boss 应昭，规则 v231）：锁定技——其他角色回合开始时，你获得
/// 座位序首名有手牌的其他角色的一张手牌（登记：hej 全区域/随机以座位
/// 序+手牌首张确定性近似）；若为装备牌则你自动使用之。
pub const BOSS_XUNYOU: &str = "巡游";
/// 魔道（Boss 夜叉，规则 v253）：锁定技——准备阶段摸 2 张牌。
pub const BOSS_MODAO: &str = "魔道";
/// 魔剑（Boss 夜叉，规则 v253）：锁定技——出牌阶段开始视为对全部敌方
/// 使用【万箭齐发】（虚拟牌，机器直发不经 PlayCard 校验；上游 useCard
/// canUse 过滤同口径=异侧存活）。
pub const BOSS_MOJIAN: &str = "魔剑";
/// 鬼计（Boss 黑无常，规则 v256）：锁定技——准备阶段（判定阶段锚）
/// 弃置自己判定区一张牌（随机→首张确定性近似登记）。
pub const BOSS_GUIJI: &str = "鬼计";
/// 太平（Boss 黑无常，规则 v256）：锁定技——摸牌阶段额外摸 2 张牌。
pub const BOSS_TAIPING: &str = "太平";
/// 索命（Boss 黑无常，规则 v256）：锁定技——结束阶段横置（链接）全部
/// 未链接的其他存活角色（[1,num] 自选→全部确定性近似登记）。
pub const BOSS_SUOMING: &str = "索命";
/// 吸行（Boss 黑无常，规则 v256）：锁定技——准备阶段对一名已链接的其
/// 他角色造成 1 点雷伤并回复 1 点体力（目标自选→座位序首个已链接确定
/// 性近似登记）。
pub const BOSS_XIXING: &str = "吸行";
/// 爆敛（Boss 白无常/牛头，规则 v255）：锁定技——结束阶段摸 2 张牌。
pub const BOSS_BAOLIAN: &str = "爆敛";
/// 强征（Boss 白无常，规则 v255）：锁定技——结束阶段从每名其他有手牌
/// 角色处随机获得一张手牌（随机→手牌首张确定性近似登记）。
pub const BOSS_QIANGZHENG: &str = "强征";
/// 醉酒（Boss 白无常，规则 v255）：锁定技——你使用的【杀】造成的伤害
/// +1（上游 damageBegin1 sha+shaParent 同口径）。
pub const BOSS_ZUIJIU: &str = "醉酒";
/// 枭首（Boss 牛头，规则 v255）：锁定技——结束阶段对一名体力值不小于你
/// 的其他角色造成 3 点伤害（目标自选可放弃→恒发动座位序首个 hp≥self 其
/// 他存活确定性近似登记）。
pub const BOSS_XIAOSHOU: &str = "枭首";
/// 御兽（Boss 罗刹，规则 v254）：锁定技——出牌阶段开始视为对全部敌方
/// 使用【南蛮入侵】（虚拟牌机器直发；异侧存活全目标）。
pub const BOSS_YUSHOU: &str = "御兽";
/// 魔炎（Boss 罗刹，规则 v254）：锁定技——非当前回合失去牌后判定红色 →
/// 对一名其他角色造成 2 点火焰伤害（目标自选→座位序首个其他存活确定性
/// 近似登记；loseEnd 全牌损失面→CardsDiscarded 面登记）。
pub const BOSS_MOYANY: &str = "魔炎";
/// 胆术（Boss 夜叉，规则 v252）：锁定技——失去手牌后（非当前回合且已
/// 受伤）判定红色回 1（CardsDiscarded 面；gain/addJudge/equip 等其余卡
/// 牌移动面登记差异）。
pub const BOSS_DANSHU: &str = "胆术";
/// 精觉（年兽精觉形态，规则 v252）：锁定技——继承胆术（同内容共用
/// commit_cards_discarded 臂，按名双查）。
pub const BOSS_JINGJUE: &str = "精觉";
/// 绝策（Boss 马面，规则 v258，一将包李儒旧绝策）：锁定技——自己回合
/// 内其他角色失去最后一手牌（弃后空手）→ 对其造成 1 点伤害（chooseBool
/// AI→恒发动确定性近似；lose/equip/gain 全事件族→CardsDiscarded 面登记）。
pub const BOSS_JUECE: &str = "绝策";
/// 酒池（Boss 董卓，规则 v258，群雄乱入）：锁定技——黑桃手牌可当【酒】
/// 使用（viewAs 转化：奇袭/武圣同族 PlayCard kind 改写臂；与奇袭[黑→拆]
/// 上游同持时由玩家择技——native if/else 链奇袭优先，登记）。
pub const BOSS_JIUCHI: &str = "酒池";
/// 鬼谋（Boss 郭嘉，规则 v259，群雄乱入）：锁定技——结束阶段随机一名
/// 其他未混乱角色进入混乱状态，其回合结束时解除（随机→座位序首个确定性
/// 近似登记；上游 mad 为纯标记技+AI 态度面，原生规则面消费=鬼谋去重过滤）。
pub const BOSS_GUIMOU: &str = "鬼谋";
/// 远策（Boss 郭嘉，规则 v259，群雄乱入）：锁定技——每当其他角色对另一
/// 其他角色造成伤害后，你选择一项：①判定，黑则受伤角色失 1 体力否则来
/// 源失 1 体力；②判定，红则受伤角色回 1 体力否则来源回 1 体力（chooseControl
/// AI 态度面+rejudge stop 检查→恒选选项一确定性近似登记；判定修正回调
/// 为 AI 选牌面不落，判定=翻牌取色）。
pub const BOSS_YUANCE: &str = "远策";
/// 奇佐（Boss 郭嘉，规则 v259，群雄乱入）：锁定技——你使用锦囊牌结算
/// 结束后，你使用一张同名牌（不可用则不发动）——副本以牌堆底实体为载体
/// 走 use_virtual_card 完整管线（v169 原语；上游凭空造牌→载体弃置登记）；
/// 触发时机 useCardAfter→CardUsed AfterTriggers 窗口近似（副本效果先于
/// 原牌效果）；铁索不复制（上游 AI check 排除同口径）；无懈不可复制。
pub const BOSS_QIZUO: &str = "奇佐";
/// 绝情（Boss 张春华，规则 v260，群雄乱入；一将包张春华旧绝情）：锁定
/// 技——你造成的伤害均改为目标失去等量体力（damageBefore cancel+loseHp
/// (num)；num=增幅前原始值——原生 commit 入口直读取 d.amount 同口径；
/// 取消后伤害事件不再结算=受击/加成类不触发）。
pub const JUE_QING: &str = "绝情";
/// 无心（Boss 张春华，规则 v260，群雄乱入）：锁定技两半——①你不能使用
/// 延时锦囊指定其他角色为目标（mod targetEnabled；原生乐/兵路由本就只
/// 能置他人→等价于持有者不可用乐/兵；闪电自置不受限）；②你受到的伤害
/// 改为失去 1 点体力（damageBefore priority 10 cancel+loseHp() 无参=1）。
pub const BOSS_WUXIN: &str = "无心";
/// 伤逝（Boss 张春华，规则 v260，群雄乱入）：锁定技——每当你的牌失去后
/// 或体力变化后，若手牌数小于 4，你将手牌补至 4 张（loseEnd/changeHp 双
/// 触发→原生 CardsDiscarded/Damage/LoseHp/Recover 四 commit 臂）；子技
/// shangshix2（group）：结束阶段若你的体力大于 1，全体存活角色（含自己）
/// 各失去 1 点体力。
pub const BOSS_SHANGSHIX: &str = "伤逝";
/// 枭雄（Boss 刘备，规则 v261，群雄乱入）：锁定技两半——①其他角色使用
/// 基本牌或锦囊牌结算结束后，你获得该牌的副本（上游 createCard 凭空造牌
/// →牌堆底实体入手登记差异：花色点数随载体+消耗库顶实体）；②子技
/// xiaoxiong_damage（group）：其他角色的结束阶段，若其本回合未使用过牌
///（上游 countUsed()==0→原生出牌阶段用牌计数==0 登记差异：响应/装备不
/// 计），你对其造成 1 点伤害。
pub const BOSS_XIAOXIONG: &str = "枭雄";
/// 章武（Boss 刘备，规则 v261，群雄乱入）：锁定技——你受到伤害后（来源
/// 存活且非自己），你弃置一张牌，来源选择：弃置一张牌，或受到 1 点伤害
///（上游刘备 [1,∞] 自选+来源 chooseToDiscard/取消双 AI 态度面→双方恒弃
/// 首张确定性近似登记；上游 num=刘备弃牌数可多张登记）。
pub const BOSS_ZHANGWU: &str = "章武";
/// 凌波（Boss 甄姬，规则 v262，群雄乱入）：锁定技——你使用或打出【闪】
/// 后摸两张牌（frequent 无询问；上游 [respond, useCard] 双触发→原生
/// CardResponded 触发+CardUsed extra 触发；viewAs 闪按声明牌种）。
pub const BOSS_LINGBO: &str = "凌波";
/// 娇孝（Boss 甄姬，规则 v262，群雄乱入）：锁定技——你成为红色牌的目标
/// 后摸一张牌（frequent；上游 target useCardToTargeted→原生 CardUsed
/// commit 臂 targets 含持有者；红色=实体花色 Heart/Diamond）。
pub const BOSS_JIAOXIA: &str = "娇孝";
/// 芳华（Boss 甄姬，规则 v262，群雄乱入）：锁定技——结束阶段若存在翻面
/// 角色，全部翻面角色各失去 1 点体力。
pub const BOSS_FANGHUA: &str = "芳华";
/// 踏水（Boss 甄姬，规则 v262，群雄乱入）：锁定技——你使用或打出黑色牌
/// 后，将一名其他角色翻面（chooseTarget AI 态度面→座位序首个其他存活
/// 确定性近似登记；经 TurnOverBefore 走翻面提交=虚失同款通道）。
pub const BOSS_TASHUI: &str = "踏水";
/// 火神（Boss 周瑜，规则 v263，群雄乱入）：锁定技——火属性伤害对你无效，
/// 你回复 1 点体力（上游 damageBegin1 cancel+recover() 无参=1 同口径）。
pub const BOSS_HUOSHEN: &str = "火神";
/// 红莲·周瑜（Boss 周瑜，规则 v263，群雄乱入；上游 boss_honglian 与毕方
/// boss_honglianx 显示名同为「红莲」，按迅影批改名映射先例以「红莲·周瑜」
/// 入目录登记）：锁定技——结束阶段你摸两张牌，并对所有敌人各造成 1 点
/// 火焰伤害。
pub const BOSS_ZYHONGLIAN: &str = "红莲·周瑜";
/// 仙音（Boss 周瑜，规则 v263，群雄乱入）：锁定技——非你的回合内，你每
/// 失去一次牌后进行判定：红色则一名其他角色失去 1 点体力（chooseTarget
/// AI 态度面→座位序首个其他存活确定性近似登记）。
pub const BOSS_XIANYIN: &str = "仙音";
/// 天道（Boss 张角，规则 v263，群雄乱入）：锁定技——判定牌确定前，你可
/// 打出一张手牌或装备牌替换判定牌，并获得原判定牌（改判机 rewriters 扩
/// 展：hes 任意牌，鬼道/鬼才同族）。
pub const BOSS_TIANDAO: &str = "天道";
/// 雷击·张角（Boss 张角，规则 v263，群雄乱入；上游 boss_leiji 与青龙
/// boss_releiji 显示名同为「雷击」，按迅影批改名映射先例以「雷击·张角」
/// 入目录登记）：锁定技——你使用或打出【闪】后，一名其他角色判定：黑则
/// 其受到 1 点雷伤且你摸一张牌（chooseTarget AI→座位序首个确定性近似）。
pub const BOSS_ZJLEIJI: &str = "雷击·张角";
/// 极点（Boss 张角，规则 v263，群雄乱入）：锁定技——你造成伤害后，距离
/// 受伤者 1 以内的一名其他角色判定：黑则其受到 1 点雷伤（chooseTarget
/// AI→座位序首个确定性近似；不自我递归=链挡旗标登记差异）。
pub const BOSS_JIDIAN: &str = "极点";
/// 枳枉（十殿阎罗五官王，规则 v265，群雄乱入）：锁定技——其他角色获得
/// 牌结算后（非摸牌阶段摸牌），其随机弃置一张手牌（随机→首张确定性近
/// 似登记；隐藏彩蛋分支 planetarian 孙策化不落登记）。
pub const BOSS_ZHIWANG: &str = "枳枉";
/// 公正（十殿阎罗五官王，规则 v265，群雄乱入）：锁定技——准备阶段（原生
/// 判定阶段开始锚点）若你的判定区有牌，你随机弃置其中一张（随机→首张
/// 确定性近似登记；鬼计同款臂）。
pub const BOSS_GONGZHENG: &str = "公正";
/// 血池（十殿阎罗五官王，规则 v265，群雄乱入）：锁定技——结束阶段随机
/// 一名其他角色失去 2 点体力（随机→座位序首个确定性近似登记）。
pub const BOSS_XUECHI: &str = "血池";
/// 黑绳（十殿阎罗宋帝王，规则 v266，群雄乱入）：锁定技——你死亡时，全
/// 部其他角色横置（上游 die forceDie 逐个 link() 同口径；死亡前置窗口触
/// 发=changan 氏遗言同款；若死亡被取消仍已横置=登记差异）。
pub const BOSS_HEISHENG: &str = "黑绳";
/// 胜负（十殿阎罗宋帝王，规则 v266，群雄乱入）：锁定技——结束阶段，一
/// 名装备区有坐骑牌的其他角色弃置其中一张（随机目标/随机牌→首个有牌者
/// /首张确定性近似登记；宝物 equip6 面随宝物栏机制登记）。
pub const BOSS_SHENGFU: &str = "胜负";
/// 雷舟（十殿阎罗卞城王，规则 v267，群雄乱入）：锁定技——准备阶段，随
/// 机一名其他角色受 1 点雷伤（randomGet→座位序首个确定性近似登记）。
pub const BOSS_LEIZHOU: &str = "雷舟";
/// 雷斧（十殿阎罗卞城王，规则 v267，群雄乱入）：锁定技——结束阶段，随
/// 机一名其他角色横置态切换（上游 link() 非恒链同口径；randomGet→首个
/// 确定性近似登记）。
pub const BOSS_LEIFU: &str = "雷斧";
/// 雷诛（十殿阎罗卞城王，规则 v267，群雄乱入）：锁定技——你死亡时，按
/// 座位序对全部其他角色各造成 1 点雷伤（die forceDie 遗言面，黑绳同位）。
pub const BOSS_LEIZHU: &str = "雷诛";
/// 服毒（十殿阎罗泰山王，规则 v268，群雄乱入）：锁定技——其他角色使用
/// 【桃】时，随机另一名其他角色失去 1 点体力（randomGet→座位序首个确
/// 定性近似登记；勘误——勘验存量误记「复读」）。
pub const BOSS_FUDU: &str = "服毒";
/// 苦酒（十殿阎罗泰山王，规则 v268，群雄乱入）：锁定技——其他角色准备
/// 阶段，其失去 1 点体力，然后视为使用一张【酒】（jiu_active 置位，不
/// 占 jiu_used；无 useCard 事件面=登记差异）。
pub const BOSS_KUJIU: &str = "苦酒";
/// 热恼（十殿阎罗泰山王，规则 v268，群雄乱入）：锁定技——你死亡时，随
/// 机一名其他角色受 3 点火焰伤害（die forceDie 遗言面，雷诛同位）。
pub const BOSS_RENAO: &str = "热恼";
/// 轮回（十殿阎罗转轮王，规则 v269，群雄乱入）：锁定技——准备阶段，若
/// 你的体力不大于 2，你与体力最高且大于 2 的一名其他角色交换体力值
///（randomGet→座位序首个确定性近似登记；交换直写 min(maxHp) 封顶）。
pub const BOSS_LUNHUI: &str = "轮回";
/// 往生（十殿阎罗转轮王，规则 v269，群雄乱入）：锁定技——出牌阶段开始
/// 时，视为随机使用一张【南蛮入侵】/【万箭齐发】（randomGet→恒定取南
/// 蛮近似登记；use_virtual_card 载体管线，目标=全部其他存活）。
pub const BOSS_WANGSHENG: &str = "往生";
/// 反噬（十殿阎罗转轮王，规则 v269，群雄乱入）：锁定技——每回合第一次
/// 受伤后，若再次受到伤害，对随机一名其他角色造成 1 点伤害（上游
/// addTempSkill boss_zlfanshi_terra 交替同口径；随机→首个确定性近似）。
pub const BOSS_ZLFANSHI: &str = "反噬";
/// 拾忧（十殿阎罗孟婆，规则 v270，群雄乱入）：锁定化近似（上游可选技
/// 恒发动确定性近似登记）——其他角色弃牌阶段弃置牌后，获得其中花色各
/// 不相同的牌（随机/自选→按弃牌序贪心取首个未见花色确定性近似；显示
/// 名勘误——勘验存量误记「拾又」）。
pub const BOSS_SHIYOU: &str = "拾忧";
/// 往事（十殿阎罗孟婆，规则 v270，群雄乱入）：锁定技——其他角色的回合
/// 开始时，其本回合不能使用或打出一种类型的牌（上游 randomGet 基本锦
/// 囊装备→恒定基本牌确定性近似登记；敌方面→全部其他近似；打出面响应
/// 窗未落=登记差异；显示名勘误——勘验存量误记「忘世」）。
pub const BOSS_WANGSHI: &str = "往事";
/// 忘魂（十殿阎罗孟婆，规则 v270，群雄乱入）：锁定遗言（Death 触发）——
/// 清全场往事封禁，随机两名敌方角色各随机失去一个技能（随机敌将/随机
/// 技→座位序前两个其他存活各首个启用技确定性近似登记；回魂牌堆注入与
/// 回收面、shidianyanluo 模式旗未落随 boss 模式扩展批）。
pub const BOSS_WANGHUN: &str = "忘魂";
/// 铁面（十殿阎罗阎罗王，规则 v271）：锁定技——防具区没有牌时，视为
/// 装备【仁王盾】（sha 机仁王块扩展：持有者防具槽空 → 黑杀无效；青釭/
/// 铡刀穿透天然覆盖）。
pub const BOSS_TIEMIAN: &str = "铁面";
/// 铡刀（十殿阎罗阎罗王，规则 v271）：锁定技——你使用【杀】指定目标后，
/// 令目标角色防具无效（上游 inherit qinggang_skill；sha 机青釭穿透链
/// 扩展：来源持有者即穿透）。
pub const BOSS_ZHADAO: &str = "铡刀";
/// 诛心（十殿阎罗阎罗王，规则 v271）：锁定遗言（Death 触发）——你死亡
/// 时，令场上血量最少的一名其他角色受到 2 点伤害（上游 chooseTarget
/// 最低体力池自选→座位序首个最低体力确定性近似登记）。
pub const BOSS_ZHUXIN: &str = "诛心";
/// 锁足（十殿阎罗平等王，规则 v272）：锁定技——准备阶段，全部其他存活
/// 角色横置（上游 boss_suozu content link 同口径；不分阵营）。
pub const BOSS_SUOZU: &str = "锁足";
/// 阿鼻（十殿阎罗平等王，规则 v272）：锁定技——你受到有来源的伤害后，
/// 来源受 1 点火焰伤害（上游 randomGet fire/thunder→恒定火确定性近似
/// 登记；无来源伤害不触发）。
pub const BOSS_ABI: &str = "阿鼻";
/// 平等（十殿阎罗平等王，规则 v272）：锁定遗言（Death 触发）——两波：
/// 最高体力其他存活（座位序首个）先受 2 点火焰伤害、重算后再受 1 点火
/// 焰伤害（上游两波 randomGet×lib.linked randomGet→座位序首个+恒定火
/// 确定性近似登记）。
pub const BOSS_PINGDENG: &str = "平等";
/// 热闷（十殿阎罗都市王，规则 v273）：锁定技——防具区没有牌时，【杀】
///（无属性，plain Sha）/【南蛮入侵】/【万箭齐发】对其无效（上游
/// useCardToBefore cancel 同口径；PlayCard 拒绝位+AOE 机窗口前跳过；
/// 属性杀不覆盖）。
pub const BOSS_REMEN: &str = "热闷";
/// 炙焚（十殿阎罗都市王，规则 v273）：锁定技——准备阶段随机一名其他
/// 角色获得其一张随机手牌（若有）并受 1 点火焰伤害（上游
/// gainPlayerCard randomGet→座位序首个+首张确定性近似登记；伤害无来
/// 源同上游）。
pub const BOSS_ZHIFEN: &str = "炙焚";
/// 火刑（十殿阎罗都市王，规则 v273）：锁定遗言（Death 触发）——全部
/// 其他存活角色按座位序各受 1 点火焰伤害（上游 sort seat 逐发同口径；
/// 中途死亡由存活过滤跳过）。
pub const BOSS_HUOXING: &str = "火刑";
/// 不佛（十殿阎罗地藏王，规则 v274）：锁定技两面——受到大于 1 点的伤
/// 害时，改为其中 1 点（上游 damageBegin4 num>1→num-1 同口径，灵渠入
/// 口同区）；准备阶段距离 1 以内的其他存活角色受 1 点火焰伤害（有来源
/// 同上游）。
pub const BOSS_BUFO: &str = "不佛";
/// 无量（十殿阎罗地藏王，规则 v274）：锁定技——结束阶段摸 2 张牌；准
/// 备阶段体力小于 3 时回复至 3 点（上游 phaseJieshu draw 2 + recover
/// 同口径）。建局摸 3 面（gameDrawAfter/enterGame）未落随装配批登记。
pub const BOSS_WULIANG: &str = "无量";
/// 谛听（十殿阎罗地藏王，规则 v274）：锁定技——你计算与其他角色的距离
/// -1；其他角色计算与你的距离 +1（上游 mod globalFrom/-1+globalTo/+1
/// 同口径；义从同位）。
pub const BOSS_DITING: &str = "谛听";
/// 帷幕（十殿阎罗楚江王，规则 v275，共用新落）：锁定技——黑色锦囊牌对
/// 你无效（显式目标黑锦囊 PlayCard 拒绝位+黑色 AOE 机窗口前跳过；无指
/// 向锦囊无指向面天然不适用=登记；红杀/属性杀不属锦囊不覆盖）。
pub const BOSS_WEIMU: &str = "帷幕";
/// 冰封（十殿阎罗楚江王，规则 v275）：锁定遗言（Death 触发）——伤害来
/// 源存活且未翻面时，将其翻面（上游 turnOver 同口径；已翻面不重复翻）。
pub const BOSS_BINGFENG: &str = "冰封";
/// 判官（十殿阎罗秦广王，规则 v276）：锁定技——延时锦囊对你无效
///（上游 mod targetEnabled delay→目标不可指定近似登记；乐/闪电；兵粮
/// 寸断不在原生目录）。
pub const BOSS_PANGUAN: &str = "判官";
/// 拘魂（十殿阎罗秦广王，规则 v276）：锁定技——结束阶段随机一名其他
/// 角色翻面或横置（上游 ["turnOver","link"] randomGet→恒定翻面+座位序
/// 首个确定性近似登记）。
pub const BOSS_JUHUN: &str = "拘魂";
/// 望乡（十殿阎罗秦广王，规则 v276）：锁定遗言（Death 触发）——全部
/// 有装备的其他角色装备全弃（撤技能+入弃堆；绝红同款拆装管线）。
pub const BOSS_WANGXIANG: &str = "望乡";
/// 天狱（群雄乱入 Boss 庞统[涅槃凤雏]，规则 v277）：锁定技——结束阶段
/// 自己已横置则先解除，再将全部未横置的其他存活横置（上游
/// ["turnOver"...boss_tianyu link 同口径；炼狱/锁足同款臂）。
pub const BOSS_TIANYU: &str = "天狱";
/// 栖梧（群雄乱入 Boss 庞统，规则 v277）：锁定技——使用梅花牌后若已受
/// 伤则回复 1 点体力（上游 useCard club+hp<maxHp recover 同口径；事件面
/// 与娇孝臂同族）。
pub const BOSS_QIWU: &str = "栖梧";
/// 涅槃（Boss 庞统[涅槃凤雏]，T11 批 118，规则 v306）：限定技——濒死时或
/// 出牌阶段发动：弃置全部牌（手/装/判）、解除横置、翻回正面、摸 3、
/// 回复至 3 点体力（上游 shenhua `niepan` 同口径）。
pub const BOSS_NIEPAN: &str = "涅槃";
/// 浴火（Boss 庞统，批 118）：涅槃后授予看破/神威/朱羽并置关卡循环型 1
/// （上游 `boss_yuhuo` `niepanAfter` 触发；原生链式执行，登记差异）。
pub const BOSS_YUHOU: &str = "浴火";
/// 神威（Boss 庞统浴火授予，批 118）：锁定——摸牌阶段多摸
/// min(3, 其他角色数)，手牌上限同量加成（上游 `shenwei` 同口径）。
pub const BOSS_SHENWEI: &str = "神威";
/// 炼狱（Boss 马面，规则 v257）：锁定技——结束阶段对全部其他存活角色
/// 各造成 1 点火焰伤害（无询问直伤）。
pub const BOSS_LIANYU: &str = "炼狱";
/// 守翼（年兽守翼形态，规则 v251）：锁定技——你使用所有牌无距离限制
///（上游 mod targetInRange恒 true 同口径；sha/顺手两距离链追加）。
pub const BOSS_SHOUYI: &str = "守翼";
/// 睿智（年兽睿智形态，规则 v250）：锁定技——准备阶段其余角色（手牌
/// >1 或装备 >1）须保留一张手牌和一张装备区内的牌，弃置其余（保留哪张
/// 由目标自选——原生保留首张确定性近似登记）。
pub const BOSS_RUIZHI: &str = "睿智";
/// 形态轮换（Boss 年兽系，规则 v249）：锁定技——回合开始交替轮换至另
/// 一「形态轮换」携带将（assembly 重挂+hp/手牌/装备/判定区保留；首次置
/// 位不换将）。随机序列→确定性互斥轮转登记。
pub const BOSS_NBIANSHEN: &str = "形态轮换";
/// 忍性（年兽忍性形态，规则 v249）：锁定技——持有者非当前回合角色时，
/// 任意角色受到伤害或回复后其摸 1 张牌。
pub const BOSS_RENXING: &str = "忍性";
/// 年瑞（Boss 年兽，规则 v248）：锁定技——摸牌阶段额外摸 2 张牌。
pub const BOSS_NIANRUI: &str = "年瑞";
/// 猛态（Boss 年兽，规则 v248）：锁定技——准备阶段置位双旗标：本回合
/// 未进入摸牌阶段 → 结束阶段摸 3；未进入出牌阶段 → 跳过弃牌阶段
/// （TurnState.mengtai_draw_pending/use_pending 承载）。形态轮换
/// （boss_nbianshen）未落随下批。
pub const BOSS_MENGTAI: &str = "猛态";
/// 自润（Boss 玄冥，规则 v247）：锁定技——准备阶段全体角色按座位序各
/// 摸牌：有装备者摸 2、无装备者摸 1。
pub const BOSS_ZIRUN: &str = "自润";
/// 造异（Boss 颛顼，规则 v246）：锁定技——玄冥/共工死亡且场上无存活
/// 同名将 → 摸 4 并获「造异异变」（运行时授予技：回合开始敌方中体力最
/// 低者全体失去全部体力）；玄冥在场 → 敌方不可为锦囊目标、共工在场 →
/// 敌方不可为基本牌目标（targetEnabled 双门控；玄冥未入库随自润批）。
pub const BOSS_ZAOYI: &str = "造异";
/// 造异异变（造异运行时授予技，规则 v246）：回合开始——敌方中体力最低
/// 者全体失去全部体力。
pub const BOSS_ZAOYI_HP: &str = "造异异变";
/// 庆祝（Boss 明星，规则 v245）：锁定技两面——跳过弃牌阶段；自己回合
/// 内出牌阶段不能用【杀】，直到获得急叮（上游 mod cardEnabled 同口径）。
pub const BOSS_QINGZHU: &str = "庆祝";
/// 夹击（Boss 明星，规则 v245）：锁定技——准备阶段相邻（环上下家）敌方
/// 中有坐骑（+1/-1 马槽）者 → 坐骑全弃。
pub const BOSS_JIAZU: &str = "夹击";
/// 急叮（Boss 明星，规则 v245）：锁定技——友方（同侧非己）受敌方来源伤
/// 害后，对来源用虚拟雷杀（机器直发不经 PlayCard 校验；蓐收加成尾面登
/// 记差异未建模）。**授予面**：由星囚（蓐收）addSkills 授予——不随明星
/// 条目装配（庆祝禁杀窗口依赖初始缺席）。
pub const BOSS_JIDING: &str = "急叮";
/// 绝红（Boss 共工，规则 v244）：锁定技——准备阶段敌方全体逐个：有牌
/// 者，有装备 → 装备全弃（撤技能+入弃堆）；无装备 → 随机弃一张手牌
/// （随机→手牌首张确定性近似登记先例族）。
pub const BOSS_JUEHONG: &str = "绝红";
/// 星夏（Boss 祝融，规则 v242）：出牌阶段主动技——对场上焰灵造成 2 点
/// 火焰伤害；随后可选一名敌方：其弃一张红色牌（手牌+装备）或受 1 点火
/// 焰伤害（可放弃追击）。间隔 ≥2 轮可再发动
/// （PlayerState.xingxia_used_round 登记）。
pub const BOSS_XINGXIA: &str = "星夏";
/// 回火（Boss 焰灵，规则 v241）：锁定技两面——【杀】使用上限 +1（每
/// 回合一次额外额度，SkillInstance.once_used 随本人回合开始重置）；死
/// 亡反噬——死亡结算完成后其敌方全体各受 3 点无来源火伤（上游
/// boss_huihuo2 global dieAfter 同口径）。
pub const BOSS_HUIHUO: &str = "回火";
/// 复燃（Boss 焰灵，规则 v241）：焰灵濒死时其敌方可用红色牌当【桃】
/// 救（上游 boss_furan2 dying chooseToUse viewAs tao 同口径）。
pub const BOSS_FURAN: &str = "复燃";
/// 不臣（Boss 勾芒，规则 v240）：出牌阶段主动技两线——场上有已死亡的
/// 书精 → 失去 1 点体力并把它们全部复活（回 1 体、各摸 2）；否则令一
/// 名己方受伤角色回复 2 点体力。间隔 ≥2 轮可再发动
/// （PlayerState.buchun_used_round 登记）。
pub const BOSS_BUCHUN: &str = "不臣";
/// 星囚（Boss 蓐收，规则 v239）：锁定技——出牌阶段开始交替横置全部敌
/// 方（每回合开始开关取反，true 回合把未横置的敌方全部横置；只链不解，
/// 已横置保持）。对 boss_mingxingzhu 授 boss_jiding 面依赖武将未入库，
/// 登记差异。
pub const BOSS_XINGQIU: &str = "星囚";
/// 神恩（Boss 太昊/炎帝 光环，规则 v238）：全局光环——boss 方用牌无视
/// 距离；挑战者侧手牌上限 +1、摸牌阶段 +1（上游 boss_shenen2 global
/// 三 mod 同口径：targetInRange side→true / maxHandcard !side→num+1 /
/// phaseDrawBegin !side→num++；光环激活=场上存在神恩持有者）。
pub const BOSS_SHENEN: &str = "神恩";
/// 灵渠（Boss 玄武，规则 v237）：锁定技两面——每次受到伤害后摸 1 张牌
/// 且受击计数 +1（手牌上限 +X，跨回合持久）；受到的大于 1 点的伤害视为
/// 0 点（上游 boss_lingqu cancel damageBegin4 num>1 → num=0 同口径，
/// 0 点伤害仍完成事件→摸 1/计数照常）。boss_xuanlin2 玄麟=关卡流程面，
/// 登记不落内核。
pub const BOSS_LINGQU: &str = "灵渠";
/// 焚天（Boss 朱雀，规则 v236）：锁定技多面——你造成的伤害视为火属性
/// （与招祸同构，复用 ZhaoHuo 机器/快照变体）；红色牌使用无次数限制且
/// 无距离限制（上游 mod cardUsable Infinity + targetInRange=true）；红
/// 色杀无视防具（boss_fentian2 directHit 全场——原生防具=八卦阵，跳防
/// 具判定承载）。wuxieRespondable 红色牌禁他人无懈面未挂钩，登记差异。
pub const BOSS_FENTIAN: &str = "焚天";
/// 狂笑（Boss 白虎，规则 v235）：锁定技两面——你的【杀】无距离限制且
/// 目标数上限无限（上游 mod targetInRange=true + selectTarget
/// range[0..1]=-1/-1 同口径）；不能指定友方为杀目标（playerEnabled）——
/// boss 侧同队关系网未入原生 PlayCard 校验，登记差异。
pub const BOSS_KUANGXIAO: &str = "狂笑";
/// 雷击（Boss 青龙，规则 v234，上游 releiji 界雷击）：你使用/打出闪后
/// 选一名其他角色判定——黑桃其受 2 点雷伤、梅花你回 1 其受 1 点雷伤、
/// 其他无事（原生=杀/万箭两闪消费位内联臂；目标选择以座位序首名确定
/// 性近似登记；判定=铁骑内联口径无鬼道/天妒改判窗；闪无主动使用面，
/// useCard 触发半面由模型承载）。
pub const BOSS_RELEIJI: &str = "雷击";
/// 青木（Boss 青龙，规则 v292，D4/B1）：死亡前置隐藏 + 配套转阶技。
pub const BOSS_QINGMU2: &str = "青木";
/// 青木·转阶（Boss 青龙，规则 v292，D4/B1）：死亡前置晚窗→勾芒+书精。
pub const BOSS_QINGMU2X: &str = "青木·转阶";
/// 青木3（Boss 勾芒，规则 v292，D4/B1）：死亡前置三人隐藏。
pub const BOSS_QINGMU3: &str = "青木3";
/// 青木3·转阶（Boss 勾芒，规则 v292，D4/B1）：→太昊+侧座勾芒/书精。
pub const BOSS_QINGMU3X: &str = "青木3·转阶";
/// 白芒（Boss 白虎，规则 v293，D4/B2）：死亡前置隐藏 + 配套转阶技。
pub const BOSS_BAIMANG2: &str = "白芒";
/// 白芒·转阶（Boss 白虎，规则 v293，D4/B2）：→蓐收+两明星+全场回满。
pub const BOSS_BAIMANG2X: &str = "白芒·转阶";
/// 赤炎（Boss 朱雀，规则 v293，D4/B2）：死亡前置隐藏 + 配套转阶技。
pub const BOSS_CHIYAN2: &str = "赤炎";
/// 赤炎·转阶（Boss 朱雀，规则 v293，D4/B2）：→祝融+两焰灵+全场回满。
pub const BOSS_CHIYAN2X: &str = "赤炎·转阶";
/// 玄麟（Boss 玄武，规则 v293，D4/B2）：死亡前置隐藏 + 配套转阶技。
pub const BOSS_XUANLIN2: &str = "玄麟";
/// 玄麟·转阶（Boss 玄武，规则 v293，D4/B2）：→共工+一玄冥+全场回满。
pub const BOSS_XUANLIN2X: &str = "玄麟·转阶";
/// 白芒3（Boss 蓐收，规则 v294，D4/B3）：死亡前置三人隐藏。
pub const BOSS_BAIMANG3: &str = "白芒3";
/// 白芒3·转阶（Boss 蓐收，规则 v294，D4/B3）：→少昊+侧座蓐收/明星+明星 fellow。
pub const BOSS_BAIMANG3X: &str = "白芒3·转阶";
/// 赤炎3（Boss 祝融，规则 v294，D4/B3）：死亡前置三人隐藏。
pub const BOSS_CHIYAN3: &str = "赤炎3";
/// 赤炎3·转阶（Boss 祝融，规则 v294，D4/B3）：→炎帝+侧座祝融/焰灵。
pub const BOSS_CHIYAN3X: &str = "赤炎3·转阶";
/// 玄麟3（Boss 共工，规则 v294，D4/B3）：死亡前置三人隐藏。
pub const BOSS_XUANLIN3: &str = "玄麟3";
/// 玄麟3·转阶（Boss 共工，规则 v294，D4/B3）：→颛顼+共工 fellow+次座玄冥。
pub const BOSS_XUANLIN3X: &str = "玄麟3·转阶";
/// 后援3（Boss 牛头/马面，规则 v295）：死亡前置隐藏。
pub const BOSS_BIANSHEN3: &str = "后援3";
/// 后援3·转阶（规则 v295）：种子化换白无常/黑无常。
pub const BOSS_BIANSHEN3X: &str = "后援3·转阶";
/// 后援4（Boss 黑无常/白无常，规则 v295）：死亡前置隐藏。
pub const BOSS_BIANSHEN4: &str = "后援4";
/// 后援4·转阶（规则 v295）：种子化换夜叉/罗刹。
pub const BOSS_BIANSHEN4X: &str = "后援4·转阶";
/// 还人（Boss 秦广王，规则 v295）：开局禁将与孟婆乱入。
pub const BOSS_HUANREN: &str = "还人";
/// 还人·转关（规则 v295）：死亡选将换关。换将后重新授予，链不随条目丢失。
pub const BOSS_HUANREN_DIE: &str = "还人·转关";
/// 青翼（Boss 太昊，规则 v295）：轮次 3/5/7。
pub const BOSS_QINGYI: &str = "青翼";
/// 赤翼（Boss 炎帝，规则 v295）：轮次 3/5/7。
pub const BOSS_CHIYI: &str = "赤翼";
/// 赤仪2（规则 v295）：赤翼第 3 轮授予，受到的伤害 +1。
pub const BOSS_CHIYI2: &str = "赤仪2";
/// 白翼（Boss 少昊，规则 v295）：第 5 轮敌方弃 2；雷伤见配套技。
pub const BOSS_BAIYI: &str = "白翼";
/// 白翼·雷（规则 v295）：第 7 轮前雷伤无效。
pub const BOSS_BAIYI_THUNDER: &str = "白翼·雷";
/// 神裔（Boss 青龙，四象系共有技，规则 v233）：锁定技两面——判定阶段
/// 的延时锦囊判定恒不生效（上游 mod judge `event.type=="phase"`：bool
/// true→false、false→null，对延时锦囊消费面等价恒失效；牌去向不变）；
/// 武将牌不能翻至背面（上游 turnOverBefore filter !isTurnedOver——仅
/// 正→背取消，背→正照常翻回）。洪荒之力禁用交互（honghuangzhili）未
/// 落，登记差异。
pub const BOSS_SHENYI: &str = "神裔";
/// 司圃（Boss 应昭，规则 v232）：锁定技，体力值首次减至一半或更少时激
/// 活；你的出牌阶段内已使用牌数≤2 期间，其他角色无法使用牌。cardEnabled/
/// cardUsable 使用面在原生回合模型下由「仅回合玩家可提交使用」天然承载；
/// cardRespondable/cardSavable 按响应/救选项排除建模（无懈等其余响应窗
/// 未纳入禁令，登记差异）。
pub const BOSS_SIPU: &str = "司圃";
/// 红莲（Boss 毕方，规则 v228）：锁定技两面——你的红色牌不计入手牌上
/// 限（弃牌阶段红色牌不可弃）；回合开始随机获得牌堆 0–3 张红色牌（确
/// 定性近似：顺序取牌堆前缀红牌至多 3），并随机对 3–0 名其他角色各造
/// 成 1 点火属性伤害（确定性近似：座位序前至多 3 名）。
pub const BOSS_HONGLIANX: &str = "红莲";
/// 炎狱（Boss 毕方，规则 v229）：锁定技，体力值首次减至一半或更少时激
/// 活；其他角色回合开始（判定阶段开始锚点）判定，红色 → 其受 1 点来
/// 自你的火伤（每回合至多 3 次）。
pub const BOSS_YANYU: &str = "炎狱";
/// 虚失（Boss 朱厌，规则 v225）：锁定技，体力值首次减至一半或更少时激
/// 活；你的出牌阶段结束时翻面；武将牌翻回正面时对所有其他角色各造成
/// 1 点伤害（上游 [1,2] 随机——恒 1 登记差异）。
pub const BOSS_XUSHI: &str = "虚失";
/// 马超「铁骑」：锁定式判定——使用杀指定目标后进行判定，若判定牌为红色，
/// 该角色不能使用闪响应此杀。
pub const TIE_JI: &str = "铁骑";
