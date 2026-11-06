//! 紧凑对局状态：只保存影响未来规则结果的数据。
//!
//! 三分结构中的「对局状态」部分：
//! - 只读规则目录在引擎侧 `Arc` 共享，不在此处；
//! - 宿主数据（展示日志、性能统计）在引擎侧，也不在此处；
//! - 执行位置（事件栈/挂起机器/开放选择）归 `resolution::Resolution`，
//!   为便于调度器拆借用而与核心字段并排放在 `GameState` 内。

use std::collections::{BTreeMap, BTreeSet};

use crate::event::Phase;
use crate::ids::{CardId, PlayerId, SkillDefId};
use crate::rng::RuleRng;
mod core_ops;

/// 单个玩家的实例级状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerState {
    pub alive: bool,
    pub hp: i32,
    pub max_hp: u32,
    pub hand: Vec<CardId>,
    pub equips: Vec<CardId>,
    /// 判定区：延时锦囊（后放入者先判定，pop 自尾部）。
    pub judge: Vec<CardId>,
    pub dying: bool,
    /// 连环（横置）状态：属性伤害结算时传导给其他被链玩家。
    pub linked: bool,
    /// 国战双将槽（STANDARD_GENERALS 下标；None = 未设置，M1 数据模型）。
    pub general_slots: [Option<u16>; 2],
    /// 国战明置状态（主将/副将各自）。
    pub revealed_generals: [bool; 2],
    pub kingdom_identity: KingdomIdentity,
    pub yexinjia_friends: Vec<PlayerId>,
    /// 五行属性（G1 wuxing：0=金 1=木 2=水 3=火 4=土；None = 未启用）。
    pub wunature: Option<u8>,
    /// 护甲（G1 古剑等扩展；受到伤害时 1:1 抵扣，先于体力扣减）。
    pub hujia: u8,
    /// 武将牌朝向（E03 欢乐【漩涡面具】原语：true = 背面朝上）。
    pub face_down: bool,
    /// 筑城（U07 玩点论杀刘馥，规则 v113）：筑城标记牌（跨回合持久；
    /// 空时阶段结束存牌、非空时收回/作为杀目标守卫筹码）。
    pub zhucheng: Vec<CardId>,
    /// 炮台等级（U05 ow 托比昂，规则 v123；跨回合持久；0=未布置）。
    pub paotai_lv: u8,
    /// 堡垒模式（U05 ow 堡垒，规则 v124；跨回合持久）：0=无、1=哨卫、
    /// 2=侦察（上游 bshaowei/zhencha storage 切换同口径）。
    pub baolei_mode: u8,
    /// 暴雪已发动（U05 ow 美，规则 v127）：limited 技每局限一次
    ///（上游 storage.baoxue + awakenSkill 同口径；跨回合持久）。
    pub baoxue_used: bool,
    /// 归汉已发动（玩点论杀批 17，规则 v280）：limited 技每局限一次
    ///（上游 awakenSkill 同口径，暴雪先例同构）。
    pub guihan_used: bool,
    /// 雪煌 limited 每局一次（swd 芝茵，武将面批 26）。
    pub xuehuang_used: bool,
    /// 雪霁冰雹标记（swd 卡牌面补切批②）：剩余摸牌阶段摸牌数 +1 次数。
    pub xuejibingbao: u8,
    /// 炼妖壶收纳仓（swd 卡牌面补切批③）：壶中暂存牌（特殊区等价——不
    /// 在任何玩家区域，跨回合持久；登记差异上游挂壶实体 storage）。
    pub lianyaohu_store: Vec<CardId>,
    /// 逆攻标记（swd 卡牌面补切批⑧龟灵之桃）：受伤积攒，上限 4。
    pub nigong_lv: u8,
    /// 急冻冰封（U05 ow 美，规则 v127）：hp=1 回合结束触发后的临时
    /// 冰封——受所有伤害无效 + 不能被其他角色的牌指定为目标；自己
    /// 回合结束（翻回后的跳过回合收束）解除（上游 addTempSkill
    /// "jidong2" 至 turnOverAfter 等价）。
    pub jidong_active: bool,
    /// 强固生效（U05 ow 奥丽莎，规则 v136）：他人对你使用杀需弃一张
    /// 基本牌否则杀无效；自己下回合判定阶段开始解除（上游 addTempSkill
    /// "qianggu2" 至 phaseBegin 等价）。
    pub qianggu_active: bool,
    /// 屏障本回合已对自己发动（U05 ow 查莉娅，规则 v138；上游空壳技
    /// pingzhang2 标志等价；自己回合判定阶段开始清）。
    pub pingzhang_self_used: bool,
    /// 屏障本回合已对他人发动（同上；上游 pingzhang3 标志等价）。
    pub pingzhang_other_used: bool,
    /// 上回合屏障次数（U05 ow 查莉娅，规则 v138）：应用（owliyong）摸
    /// 牌阶段 +X 的 X；自己回合判定阶段开始时由两标志转换（上游
    /// pingzhang_count phaseBegin silent 同口径）。
    pub pingzhang_count: u8,
    /// 潜行（U05 ow 黑影，规则 v139）：失去手牌基本牌且手牌无基本牌时
    /// 置位——不能成为其他角色的卡牌目标；自己回合判定阶段开始解除
    ///（上游 tempHide→addTempSkill "qianxing" 至 phaseBeginStart 等价）。
    pub yinshen_active: bool,
    /// 机甲血（U05 ow D.Va，规则 v141）：初始 4——受伤时 1:1 抵扣（体
    /// 力不损）；归零后照常扣体力（上游 storage.jijia changeHp 拦截同
    /// 口径；手牌上限 +4 同源）。
    pub jijia_hp: u8,
    /// 风/雅/颂计数器（U05 xianjian 雷元戈，规则 v150）：各累计 2 次后
    /// 第 3 次触发强化（上游 storage.feng/ya/song 同口径）。
    pub feng_count: u8,
    pub ya_count: u8,
    pub song_count: u8,
    /// 蛊舞剩余回合数（U05 xianjian 阿奴，规则 v154）：>0=蛊舞生效（出
    /// 牌上限=countUsed 门槛+手牌上限-1）；自己阶段结束 -1（上游 anwugu2
    /// phaseAfter 递减同口径）。
    pub anwugu_count: u8,
    /// 龙火已觉醒（U05 xianjian 姜城，规则 v157）：焚逝触发后置位=已获
    /// 得龙火（上游 addSkill("longhuo") 永久技同口径；觉醒一次性由本标
    /// 志门控）。
    pub jiangcheng_longhuo: bool,
    /// 月行对偶（U05 xianjian 龙幽，规则 v159）：非 None=距离基数交换
    /// 生效中（双向互指；自己出牌阶段结束解除——上游 yuexing2 临时技
    /// phaseUseAfter 同口径）。
    pub yuexing_pair: Option<u8>,
    /// 本回合已造成伤害次数（U05 xianjian 明绣，规则 v160 灵火门控；
    /// 上游 getStat("damage") 同口径，换回合清零）。
    pub turn_damage: u8,
    /// 灵火下次可用轮（U05 xianjian 明绣，规则 v160；上游 round:2 同族
    /// ——触发后 2 轮冷却，0=立即可用）。
    pub linghuo_ready_round: u32,
    /// 归烬下次可用轮（U05 xianjian 明绣，规则 v160；上游 round:3 同
    /// 族——触发后 3 轮冷却，0=立即可用）。
    pub guijin_ready_round: u32,
    /// 跳过下次摸牌阶段（U05 xianjian 明绣，规则 v160 归烬牌获得者；
    /// 上游 skip("phaseDraw") 同口径，Draw begin 消费）。
    pub skip_next_draw: bool,
    /// 贯日已发动（U05 xianjian 云天河，规则 v161；上游 limited 一次
    /// 性同口径，PlayerState 持久标志）。
    pub guanri_used: bool,
    /// 武将牌上置牌区（U05 xianjian 紫萱，规则 v162 离场区基建）：实体
    /// 牌离场存放（牌守恒——不入其他区、死亡时随 leftover 弃置由消费
    /// 技自理）。
    pub off_court: Vec<u32>,
    /// 水云已救人数（U05 xianjian 紫萱，规则 v162；上游 storage.
    /// shuiyun_count 同口径——小蛮 shenwu 觉醒链依赖）。
    pub shuiyun_count: u8,
    /// 忘忧标记回合戳（U05 xianjian 紫萱，规则 v162；上游 wangyou3 临
    /// 时技同口径——受伤时置当前轮，0=无标记，轮次推进自然过期）。
    pub wangyou3_round: u32,
    /// 全局用牌计数（U05 xianjian 小蛮，规则 v163 灵泉门控；上游
    /// getHistory("useCard").length 同口径，未被取消的使用全计）。
    pub total_used: u32,
    /// 灵泉已觉醒（U05 xianjian 小蛮，规则 v163；juexingji 一次性+持有
    /// 水云标记——觉醒时 runtime 授 shuiyun 实例）。
    pub lingquan_awakened: bool,
    /// 神雾已觉醒（U05 xianjian 小蛮，规则 v163；juexingji 一次性+持有
    /// 回梦标记——觉醒时 runtime 授 huimeng 实例）。
    pub shenwu_awakened: bool,
    /// 诚信免伤（U05 xianjian 明绣，规则 v164；上游 chengxin2 临时技
    /// damageBefore cancel 同口径——持续到本人 End 阶段结束，登记差异：
    /// 上游=最近阶段结束）。
    pub chengxin_active: bool,
    /// 诚信下次可用轮（U05 xianjian 明绣，规则 v164；上游 round:4 同族
    /// ——触发后 4 轮冷却，0=立即可用）。
    pub chengxin_ready_round: u32,
    /// 虎煞标记（U05 xianjian 王小虎，规则 v169；上游 storage.husha 同
    /// 口径——回合内伤人 +伤害量，上限 3）。
    pub husha_mark: u8,
    /// 魂身计数（swd 胡月，规则 v296）：使用/打出闪累计；准备阶段摸等量
    /// 并清零；他人算向自己距离 +计数。
    pub hyunshen_count: u8,
    /// 待行计数（swd 吕晓雪，规则 v297）：结束阶段弃牌换取的护甲量；
    /// 下回合准备阶段按剩余量移除；受伤吸收时递减。
    pub daixing_count: u8,
    /// 聚悉计数（swd 小黄龙，规则 v298）：非当前回合角色弃牌累计；
    /// 达存活数时可发动聚悉（扣存活数）令目标受伤或回复。
    pub juxi_count: u8,
    /// 破舟计数（swd 刘承宣，规则 v299）：受伤累计；出牌阶段可消耗 1
    /// 封印一名其他角色（全部技能禁用至其下一阶段开始）。
    pub pozhou_count: u8,
    /// 陷阱牌（swd 舒莞儿，规则 v300）：置出的陷阱牌实体；他人用同花色
    /// 牌指定你时触发（消耗陷阱 + 随机获得其一张牌）；你受伤时随机取回。
    pub sxianjing_traps: Vec<CardId>,
    /// 死亡拦截标记（规则 v171 死亡取消型基建；上游 dieBefore
    /// trigger.cancel 族——DeathMachine 直读跳过死亡执行）。
    pub death_cancelled: bool,
    /// 适盈已发动（U05 xianjian 月琦，规则 v171；limited 一次性）。
    pub shiying_used: bool,
    /// 花好已发动（U05 xianjian 苏媚，规则 v175；limited 一次性）。
    pub huahu_used: bool,
    /// 玄砚计数（U05 xianjian 慕容紫英，规则 v179；上游 storage.
    /// xuanning 计数标记——**获得路径上游缺失登记差异**：原生仅支持
    /// 调试/宿主注入）。
    pub xuanning_mark: u32,
    /// 蛇毒标记（Boss 相柳毒妒/邪酬，规则 v223）：受击者/用桃酒者累积，
    /// 其回合开始弃 X 或失 X 后递减。
    pub shedu: u8,
    /// 巨源（Boss 朱厌，规则 v226）：上次回合结束时的体力（跨回合持久；
    /// 出牌阶段体力低于该值时【杀】可额外指定 1 个目标）。
    pub juyuan_storage: i32,
    /// 封印（Boss 应昭封冻，规则 v228）：本回合无法使用手牌（应昭回合
    /// 开始授予他人，回合结束解除）。
    pub fengyin_active: bool,
    /// 灵渠（Boss 玄武，规则 v237）：受击计数（每次受伤 +1；手牌上限
    /// +X；跨回合持久）。
    pub lingqu_count: u8,
    /// 星囚（Boss 蓐收，规则 v239）：出牌阶段开始交替开关（每回合开始
    /// 取反；true 回合横置全部敌方；跨回合持久）。
    pub xingqiu_toggled: bool,
    /// 反噬 terra 旗（十殿阎罗转轮王，规则 v269）：本回合已受伤一次；
    /// 回合结束全清（封印解除同位）。
    pub zlfanshi_terra: bool,
    /// 往事封禁（十殿阎罗孟婆，规则 v270）：本回合不能使用或打出基本牌
    ///（上游 randomGet 类型→恒定基本牌近似；打出面未落=登记差异）；
    /// 回合结束全清。
    pub wangshi_ban: bool,
    /// 复仇标记（玩点论杀批 14，规则 v279）：以【杀】指向前弃牌取消并
    /// 标记的来源（跨回合持久；fuchou2 于自己回合结束逐个结算）。
    pub fuchou_marked: Vec<PlayerId>,
    /// 不臣（Boss 勾芒，规则 v240）：上次发动不臣的轮次（0=未发动过；
    /// 间隔 ≥2 轮可再发动；跨回合持久）。
    pub buchun_used_round: u32,
    /// 星夏（Boss 祝融，规则 v242）：上次发动星夏的轮次（0=未发动过；
    /// 间隔 ≥2 轮可再发动；跨回合持久）。
    pub xingxia_used_round: u32,
    /// 形态轮换（Boss 年兽系，规则 v249）：轮换交替旗标（false=处于初
    /// 始形态，true=已轮换至次形态；确定性互斥轮转登记）。
    pub nbianshen_toggle: bool,
    /// 天符（U05 xianjian 长卿，规则 v188）：挂附来源（雷伤来源）。
    pub ctianfu_source: Option<u8>,
    /// 天符：挂附闪实体（离场区中该牌；死亡清理/结算弃置用）。
    pub ctianfu_card: Option<u32>,
    /// 勾魂锣（U05 swd 机关牌第三切片，规则 v191）：倒计时轮数。
    pub gouhunluo_rounds: u32,
    /// 勾魂锣：挂附来源。
    pub gouhunluo_source: Option<u8>,
    /// 机关怨（U05 swd 机关牌第二十一批，规则 v196）：待获得牌实体
    /// （离场区中；其下一结束阶段由 SwdJiguanyuanGain 移入手牌）。
    pub jiguanyuan_cards: Vec<u32>,
    /// 紧急冷冻（U05 swd 灵剑，规则 v198）：冻结状态——不能使用牌也不
    /// 能成为牌的目标；翻回（TurnOvered）解除。
    pub jileng_active: bool,
    /// 圣休号角嘲讽（U05 swd 灵剑，规则 v198）：相邻角色只能以其为杀目
    /// 标；其下一阶段开始解除。
    pub chaofeng_active: bool,
    /// 星体改造（U05 swd 灵剑，规则 v198）：手牌上限 -N 累加；其下一结
    /// 束阶段重置。
    pub xingtigaizao_count: u8,
    /// 隐秘离场潜行（U05 swd 灵剑，规则 v198）：不能成为牌的目标；当前
    /// 回合结束解除（上游 tempHide 本回合近似）。
    pub yinmi_active: bool,
    /// 卡牌实例改造表（U05 swd 机关术批，规则 v199，ADR 0780）：装备实体
    /// → 附着灵剑实体（受控合成——目录不可变，改造以伴随表建模）。
    pub card_mods: std::collections::BTreeMap<u32, u32>,

    /// Boss 模式重整计数（P2/N08 模式入口批三-a，规则 v82）：死亡后
    /// 每次轮到其回合计数 +1，达阈值复活（仅 Boss 模式使用）。
    pub boss_chongzheng: u8,

    /// 混乱（Boss 郭嘉鬼谋，规则 v259）：疯狂状态标记（上游 addTempSkill
    /// "mad" 至该玩家 phaseAfter 同口径——其回合收束解除；上游 mad 仅
    /// 挂标记+AI 态度面，原生规则面消费=鬼谋去重过滤）。
    pub boss_mad: bool,
    /// 锦鳞（swd 司空蔷，swd deferred 消解批 114，规则 v302）：limited
    /// 一次性主动技已发动（上游 `awakenSkill` + `storage.jinlin`）。
    pub jinlin_used: bool,
    /// 禳星·星数（T11 Boss 诸葛亮族 / hlss 星系统，规则 v303）：上游
    /// `storage.hlss_xiangxing`，3→0；归零时觉醒（不再回满体力）。
    pub hlss_stars: u8,
    /// 禳星·累计伤害计数（上游 `storage.hlss_xiangxing_count`）：每受
    /// 1 点伤害/失去 1 点体力 +1，达 3 触发失星与雷伤分配。
    pub hlss_count: u8,
    /// 隐月·已受伤类型位图（上游 `storage.hlss_yueyin_damage`）：
    /// 1=雷属性伤害、2=火属性伤害、4=锦囊牌伤害；失星时清空。
    pub hlss_damage: u8,
    /// 起风·已用牌种码表（T11 Boss 诸葛亮族批 116，规则 v303 起同批）：
    /// 上游 `storage.hlss_fengqi_used`（牌名字符串表）——原生存目录牌种码。
    pub hlss_fengqi_used: Vec<u16>,
    /// 潜龙·道心值（D5 龙血玄黄规则面批 117，规则 v305）：上游
    /// `player.countMark('mbqianlong')`，0..=99；阈值 25/50/75/99 解锁
    /// 清正/酒诗/放逐/绝进。
    pub mbqianlong_marks: u8,
    /// 涅槃·限定技已发动（T11 Boss 庞统族批 118，规则 v306）：上游
    /// `player.storage.niepan`（awakenSkill 后 true）。
    pub niepan_used: bool,
    /// 神焰·限定技已发动（swd 批 121，规则 v308）：上游
    /// `player.storage.shenyan`（awakenSkill 后 true）。
    pub shenyan_used: bool,
    /// 神焰·待蔓延目标（同批）：上游 `storage.shenyan2`（原目标距离 ≤1 的
    /// 其他角色，座位序）——主伤害结算后逐个受 1 点无来源火伤。
    pub shenyan_pending: Vec<PlayerId>,
    /// 入梦·跳过弃牌阶段（swd 批 122，规则 v309）：上游 `skip('phaseDiscard')`
    /// ——弃牌阶段开始即跳过（本回合内一次性，用后清零）。
    pub skip_discard_phase: bool,
    /// 义山·最近失去的牌（swd 批 123，规则 v310）：上游 `storage.yishan`
    /// （`yishan2` loseEnd unshift 记录；新在前）——受伤后取回至多 2 张
    /// 仍在他处者。
    pub yishan_lost: Vec<CardId>,
    /// 守印·限定技已发动（swd 批 124，规则 v311）：上游
    /// `player.storage.shouyin`（awakenSkill 后 true）。
    pub shouyin_used: bool,
    /// 天书·技能来源角色（swd 批 142，规则 v317）：上游
    /// `storage.swdtianshu`（来源阵亡时移除所获技能）。
    pub swdtianshu_from: Option<PlayerId>,
    /// 天书·所获技能定义（同批）：移除时定位实例（上游
    /// `removeAdditionalSkill('swdtianshu')`）。
    pub swdtianshu_granted: Option<u16>,
    /// 脉轮·当前效果（swd 批 137，规则 v316）：上游 `storage.mailun`
    /// （1..8；0=未选）。每阶段开始重选（旧效果清除）。
    pub mailun_mode: u8,
    /// 脉轮·首次伤害（造成侧）已用（同批）：上游 `mailun31`/`mailun41`
    /// 一次性子技（用后移除）。
    pub mailun_out_used: bool,
    /// 脉轮·首次伤害（受到侧）已用（同批）：上游 `mailun32`/`mailun42`。
    pub mailun_in_used: bool,
    /// 极略·本出牌阶段已用次数（swd 批 136，规则 v315）：上游 `usable:3`
    /// （每出牌阶段 3 次；出牌阶段开始清零）。
    pub jilve_used: u8,
    /// 颂词·已发动过的目标（swd 批 129，规则 v313）：上游
    /// `storage.songci`（按座位序；全部角色均已记录时弃牌阶段末摸 1）。
    pub songci_targets: Vec<PlayerId>,
    /// 护天·寄存目标（swd 批 131，规则 v314）：上游 `storage.hutian`
    /// （寄存方视角；Some = 牌寄存于该角色处）。
    pub hutian_deposit: Option<PlayerId>,
    /// 护天·刚寄存位（同批）：上游 `hutian4` 临时技（默认 expire=
    /// phaseAfter/phaseBefore）——寄存只跨一个阶段边界，期间不归还。
    pub hutian_just_given: bool,
    /// 护天·寄存于此的牌（同批）：上游 `storage.hutian2`（被寄存方视角）。
    pub hutian_hold: Vec<CardId>,
    /// 护天·寄存方（同批）：被寄存方记录谁寄存（归还时用）。
    pub hutian_owner: Option<PlayerId>,
}

/// 无限体力哨兵（上游 `Infinity`，D5/T11-BOSS-ZHUGE-LIANG 前置原语）：
/// `max_hp` 等于该值即视为无限体力——伤害/失去体力/回复**都不改变**体力
/// （保持 hp == max_hp，上游 Infinity 同口径），濒死判定永不进入，手牌上限
/// 基数取哨兵值（等效无上限）。投影直接暴露哨兵数值，宿主可映射为 ∞
///（登记差异：不新增视图字段）。
pub const INFINITE_HP: u32 = i32::MAX as u32;

impl PlayerState {
    /// 该玩家是否持有无限体力（哨兵判定）。
    pub fn infinite_hp(&self) -> bool {
        self.max_hp == INFINITE_HP
    }

    pub fn new(hp: u32) -> Self {
        Self {
            alive: true,
            hp: hp as i32,
            max_hp: hp,
            hand: Vec::new(),
            equips: Vec::new(),
            judge: Vec::new(),
            dying: false,
            linked: false,
            hujia: 0,
            face_down: false,
            zhucheng: Vec::new(),
            paotai_lv: 0,
            baolei_mode: 0,
            baoxue_used: false,
            guihan_used: false,
            xuehuang_used: false,
            xuejibingbao: 0,
            lianyaohu_store: Vec::new(),
            nigong_lv: 0,
            jidong_active: false,
            qianggu_active: false,
            pingzhang_self_used: false,
            pingzhang_other_used: false,
            pingzhang_count: 0,
            yinshen_active: false,
            jijia_hp: 0,
            feng_count: 0,
            ya_count: 0,
            song_count: 0,
            anwugu_count: 0,
            jiangcheng_longhuo: false,
            yuexing_pair: None,
            turn_damage: 0,
            linghuo_ready_round: 0,
            guijin_ready_round: 0,
            skip_next_draw: false,
            guanri_used: false,
            off_court: Vec::new(),
            shuiyun_count: 0,
            wangyou3_round: 0,
            total_used: 0,
            lingquan_awakened: false,
            shenwu_awakened: false,
            chengxin_active: false,
            chengxin_ready_round: 0,
            husha_mark: 0,
            hyunshen_count: 0,
            daixing_count: 0,
            juxi_count: 0,
            pozhou_count: 0,
            sxianjing_traps: Vec::new(),
            death_cancelled: false,
            shiying_used: false,
            huahu_used: false,
            xuanning_mark: 0,
            shedu: 0,
            juyuan_storage: 0,
            fengyin_active: false,
            lingqu_count: 0,
            xingqiu_toggled: false,
            zlfanshi_terra: false,
            wangshi_ban: false,
            fuchou_marked: Vec::new(),
            buchun_used_round: 0,
            nbianshen_toggle: false,
            xingxia_used_round: 0,
            ctianfu_source: None,
            ctianfu_card: None,
            gouhunluo_rounds: 0,
            gouhunluo_source: None,
            jiguanyuan_cards: Vec::new(),
            jileng_active: false,
            chaofeng_active: false,
            xingtigaizao_count: 0,
            yinmi_active: false,
            card_mods: Default::default(),
            general_slots: [None, None],
            revealed_generals: [false, false],
            kingdom_identity: KingdomIdentity::Unknown,
            yexinjia_friends: Vec::new(),
            wunature: None,
            boss_chongzheng: 0,
            boss_mad: false,
            jinlin_used: false,
            hlss_stars: 0,
            hlss_count: 0,
            hlss_damage: 0,
            hlss_fengqi_used: Vec::new(),
            mbqianlong_marks: 0,
            niepan_used: false,
            shenyan_used: false,
            shenyan_pending: Vec::new(),
            skip_discard_phase: false,
            yishan_lost: Vec::new(),
            shouyin_used: false,
            swdtianshu_from: None,
            swdtianshu_granted: None,
            mailun_mode: 0,
            mailun_out_used: false,
            mailun_in_used: false,
            jilve_used: 0,
            songci_targets: Vec::new(),
            hutian_deposit: None,
            hutian_just_given: false,
            hutian_hold: Vec::new(),
            hutian_owner: None,
        }
    }
}

/// 技能实例：同一定义可被不同玩家持有，或被动态获得/移除。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillInstance {
    pub owner: PlayerId,
    pub enabled: bool,
    /// `once_per_turn` 技能本回合是否已发动。
    pub once_used: bool,
    /// 临时技能：到该玩家回合结束移除。
    pub expires_end_of_turn: bool,
    /// G1 古剑食物状态：剩余回合数（0 = 非计时技能；效果触发点扣减，
    /// 归零移除实例）。
    pub rounds: u8,
}

/// 回合状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnState {
    pub current: PlayerId,
    pub phase: Phase,
    pub round: u32,
    /// 出牌阶段是否已使用过杀。
    pub sha_used_play_phase: bool,
    /// 沐风（swd 桓远之，批㉑）：本回合已使用基本牌。
    pub mufeng_used_basic: bool,
    /// 望尘（swd 古月圣，批㉑）：弃牌阶段弃置过基本牌。
    pub wangchen_discard_basic: bool,
    /// 神秘果（swd 卡牌面窗口机批）：本阶段已再结算过（上游 shenmiguo2
    /// 临时技等价——限一次与循环守卫共用；换回合清）。
    pub shenmiguo_used: bool,
    /// 鼠槌（swd 卡牌面窗口机批）：多杀循环链武装中（commit_damage 钩
    /// 消费；机器收束清，换回合清）。
    pub shuchui_armed: bool,
    /// 鼠槌：链上任一杀已造成真实伤害（终局摸 1 判据）。
    pub shuchui_hit: bool,
    /// 连打（swd 武将面批㉔ 敷衍）：本回合已链过（上游 lianda2 临时技等
    /// 价——循环守卫；换回合清）。
    pub lianda_used: bool,
    /// mtg 龙颅山峰（规则 v100）：本回合已使用地（上游 land_used 临时技
    /// 同口径——仅当前回合玩家可出牌，故回合级布尔等价）。
    pub land_used: bool,
    /// 龙颅山峰地图效果（全局出杀上限 +1）下本回合已用的额外杀。
    pub land_sha_used: bool,
    /// 水没墓穴地图效果（mtg 第三批，规则 v102）：本弃牌阶段已弃张数
    ///（阶段开始清零、阶段结束消费后清零；入存档/哈希）。
    pub discard_phase_discarded: u32,
    /// 刺猬（U07 玩点论杀韩龙，规则 v111）：本回合濒死即死目标（上游
    /// addTempSkill("ciqiu_dying") 回合级临时技等价；换回合清空）。
    pub ciqiu_dying: Vec<PlayerId>,
    /// 抗yin2（U07 玩点论杀留赞，规则 v114）：回合内临时增益（上游
    /// addTempSkill("kangyin2") 等价；换回合清空）——到其他角色距离
    /// -X、杀额外目标上限 +X（X=发动时 maxHp-体力）。
    pub kangyin2: Option<(PlayerId, u32)>,
    /// 还侠跟踪（swd 屈娴，批 27）：本回合已提交的还侠转化杀（牌, 是否
    /// 已造成伤害）——阶段末收回未造成伤害者（换回合清空）。
    pub huanxia_track: Vec<(CardId, bool)>,
    /// 突击（U07 ow 士兵76，规则 v118）：本出牌阶段已使用牌数（上游
    /// countUsed 同口径；Play 阶段开始清零、换回合自然清零）。
    pub cards_used_play_phase: u32,
    /// 本回合已使用牌数（swd 灵武批 120，规则 v307；上游 `countUsed()`
    /// 无参口径——本回合全部使用；换回合清零）。
    pub cards_used_turn: u32,
    /// 燎原·本次【杀】的额外闪需求/加伤数（swd 批 145，规则 v319）：
    /// 上游 `storage.liaoyuan`（杀结算开始置数、伤害时加伤、杀结束清零）。
    pub liaoyuan_count: u8,
    /// 轩辕剑加伤标记（swd 批 146，规则 v320）：上游 `event._xuanyuanjian`
    /// （加伤生效后反噬子技据此失去体力）。
    pub xuanyuanjian_marked: bool,
    /// 昆仑镜·快照（swd 批 149，规则 v321）：逐玩家（手牌/装备/判定区）。
    pub kunlunjing_snapshot: Vec<(PlayerId, Vec<CardId>, Vec<CardId>, Vec<CardId>)>,
    /// 昆仑镜·快照时体力（同批）。
    pub kunlunjing_hp: i32,
    /// 额外阶段队列（同批）：`insertPhase` 原语——阶段结束时若非空则弹出
    /// 该阶段作为下一阶段（码同 `Phase`：0 准备/1 摸牌/2 出牌/3 弃牌/4 结束）。
    pub extra_phases: Vec<u8>,
    /// 当前阶段是否由灵武插入（同批）：上游 `event.skill != 'lingwu'`
    /// 防递归——插入阶段的收束不再触发灵武。
    pub extra_phase_lingwu: bool,
    /// 出牌阶段已使用的基本/锦囊/装备牌数（T8 增量 14，DSL filter
    /// `*_play_phase_card_category_count_positive` 承载；上游三类计数
    /// 同口径——按结算牌种判类，地/机关牌异类不计；Play 阶段开始清零）。
    pub bte_used_play_phase: u32,
    /// 全局阶段计数（U07 ow 自愈，规则 v120；上游 game.phaseNumber 同
    /// 口径——每阶段开始 +1，自愈 %4 门控用；入存档/哈希）。
    pub phase_number: u32,
    /// 仁德（刘备）本出牌阶段累计给出的手牌数（跨发动累计，
    /// 跨过 2 张回复 1 点；回合状态随新回合自然清零）。
    pub rende_given: u8,
    /// 本回合是否已使用过【酒】（每回合限一次）。
    pub jiu_used: bool,
    /// 木甲人（U05 swd 机关牌第二切片，规则 v190）：usable:1 每回合
    /// 一次（Judge begin 清）。
    pub mujiaren_used: bool,
    /// 酒效果待消耗：本回合使用的下一张杀伤害 +1
    ///（于杀结算机派发时捕获并清零）。
    pub jiu_active: bool,
    /// 回合开始标记：用于重置 `once_per_turn`。
    pub turn_opened: bool,
    /// 本回合跳过摸牌阶段（兵粮寸断判定生效）。
    pub skip_draw_phase: bool,
    /// 本回合跳过出牌阶段（乐不思蜀判定生效）。
    pub skip_play_phase: bool,
    /// 孤行（swd 慕容诗，规则 v301）：本回合孤行杀已发出（guxing3 计数门）。
    pub guxing_armed: bool,
    /// 孤行：本回合孤行杀造成的伤害数（结算后摸等量）。
    pub guxing_count: u8,
    /// 猛态（Boss 年兽，规则 v248）：本回合准备阶段已过而摸牌阶段未发
    /// 生（true=结束阶段补摸 3）。
    pub mengtai_draw_pending: bool,
    /// 猛态：本回合准备阶段已过而出牌阶段未发生（true=跳过弃牌阶段）。
    pub mengtai_use_pending: bool,
    /// 贴房绅视（U05 swd，规则 v94）：本回合无视距离指定的目标
    ///（对【杀】等距离校验豁免；回合结束随 TurnState 更替自然清零）。
    pub zhufangshenshi_target: Option<PlayerId>,
    /// 乐动目标数增量（U07 ow 卢西奥，规则 v126）：扩音弃 1 张 +2
    ///（上游 storage.yuedong_num；仅自己出牌阶段积攒/消费，换回合
    /// 清空等价上游 kuoyin2 phaseBegin 静默重置）。
    pub yuedong_num: u8,
    /// 乐动摸/回复量增量（U07 ow 卢西奥，规则 v126）：扩音弃 2 张 +1
    ///（上游 storage.yuedong_eff；本回合限一次）。
    pub yuedong_eff: u8,
    /// 乐动改回复模式（U07 ow 卢西奥，规则 v126）：呼唤弃 2 张置位，
    /// 乐动结算由摸牌改为回复（上游 storage.yuedong_recover）。
    pub yuedong_recover: bool,
    /// 链钩被钩者（U05 ow 路霸，规则 v128）：链钩判定成功后置位——
    /// 本回合内路霸算到其距离恒 1、其首次受伤害 +1（上游
    /// target.storage.liangou2=addTempSkill 至回合结束同口径；仅路霸
    /// 出牌阶段可置位，换回合随 TurnState 自然解除）。
    pub liangou_hook: Option<PlayerId>,
    /// 链钩伤害增幅已消耗（同批）：被钩者本回合首次受伤 +1 后置位
    ///（上游 liangou2 damageBegin usable:1 同口径）。
    pub liangou_bonus_used: bool,
    /// 跳跃目标（U05 ow 温斯顿，规则 v137）：跳跃弃牌后置位——本回合
    /// 自己算到其距离恒 1（上游 storage.feitiao2+addTempSkill "feitiao2"
    /// 至回合结束同口径；换回合随 TurnState 复位）。
    pub feitiao_target: Option<PlayerId>,
    /// 返灵已发动（U05 xianjian 瑕，规则 v145）：他人失去体力后回复 1
    /// 每回合限一次（上游 fanling usable:1 同口径；换回合随 TurnState
    /// 复位）。
    pub fanling_used: bool,
    /// 蝶舞已摸牌（U05 xianjian 龙葵，规则 v153）：蝶舞给出杀后首次摸 1
    ///（上游 diewu2 临时技等价；换回合随 TurnState 复位）。
    pub diewu_draw_used: bool,
    /// 掠影已发动（U05 xianjian 慕长兰，规则 v154）：每回合限一次（上游
    /// lueying usable:1 同口径；换回合随 TurnState 复位）。
    pub lueying_used: bool,
    /// 甜香去已发动（U05 xianjian 唐雨柔，规则 v156）：每回合限一次（上
    /// 游 txianqu2 临时技等价；换回合随 TurnState 复位）。
    pub txianqu_used: bool,
    /// 聚宝已回收（U05 xianjian 景天，规则 v158）：他人回合外首次弃非
    /// 基本牌门控（上游 jtjubao2 临时技等价；换回合随 TurnState 复位）。
    pub jtjubao_used: bool,
    /// 逐月目标（U05 xianjian 云天河，规则 v161）：本出牌阶段使用杀须
    /// 以其为目标（白名单+距离豁免——上游 zhuyue mod 同口径；出牌阶段
    /// 结束清空，随 TurnState 复位）。
    pub zhuyue_targets: Vec<u8>,
    /// 摄灵本回合已触发次数（U05 xianjian 南宫煌，规则 v168；上游
    /// usable:3 同口径——Judge begin 清，随 TurnState 复位）。
    pub sheling_used: u8,
    /// 射影已发动（U05 xianjian 苏媚，规则 v175；上游 usable:1 同口径
    /// ——Judge begin 清，随 TurnState 复位）。
    pub sheying_used: bool,
    /// 张目已发动（U05 xianjian 南宫煌，规则 v181；上游 usable:1 同口
    /// 径——Judge begin 清，随 TurnState 复位）。
    pub zhangmu_used: bool,
    /// 影锋（U05 xianjian 越今朝，规则 v185）：链挡旗标——影锋虚拟杀
    /// 结算后窗拒绝再触发（上游 parent.name 检查的登记差异等价；
    /// Judge begin 清）。
    pub yingfeng_chain: bool,
    /// 乱剑（U05 xianjian 长卿，规则 v187）：虚拟杀载体牌锚定——
    /// DamageBefore 增伤判定（Judge begin 清）。
    pub luanjian_card: Option<u32>,
    /// 冥顽（Boss 梼杌，规则 v219）：本回合内用牌伤害过的目标锁定集
    /// （空=未激活；回合更替自然清零=上游 phaseAfter clear 同窗）。
    pub minwan_targets: Vec<PlayerId>,
    /// 炎狱（Boss 毕方，规则 v229）：本回合已触发的炎狱判定次数（每回
    /// 合至多 3；回合更替清零）。
    pub yanyu_count: u8,
    /// 御风已发动次数（U05 xianjian 李逍遥，规则 v151）：每回合限两次
    ///（上游 xjyufeng usable:2 同口径；换回合随 TurnState 复位）。
    pub xjyufeng_used: u8,
    /// 奇佐（Boss 郭嘉，规则 v259）：副本用牌载体槽（上游 event.parent
    /// name 递归守卫同口径——filter 拒绝载体自身的 CardUsed；任一其他
    /// 卡牌提交即清，回合收束兜底清）。
    pub qizuo_copy: Option<u32>,
    /// 极点（Boss 张角，规则 v263）：链挡旗标——极点造成的雷伤不再触发
    /// 极点（登记差异：上游 damageAfter 真递归，原生确定性判定可能循环）。
    pub jidian_chain: bool,
}

/// 对局结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub winner: Winner,
}

/// 胜利方：原型模式为存活玩家；身份局为阵营。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Winner {
    /// 原型模式：唯一存活者。
    Player(PlayerId),
    /// 身份局：主公与忠臣阵营胜（反贼与内奸全灭）。
    LordSide,
    /// 身份局：反贼阵营胜（主公死亡且内奸非唯一存活）。
    Rebels,
    /// 身份局：内奸胜（主公死亡时场上只剩内奸）。
    Traitor,
    /// 国战：势力胜（场上仅剩该势力角色）。
    Faction(crate::catalog::Faction),
    /// 对决：队伍胜（index 入 GameMode::Versus.teams）。
    Versus(usize),
    /// 斗地主：`true`=地主胜（农民全灭），`false`=农民胜（地主死亡）。
    Doudizhu { lord_won: bool },
    /// Boss：`true`=boss 胜（挑战者全灭），`false`=挑战者胜（boss 死亡）。
    Boss { boss_won: bool },
}

/// 身份（身份局）：座位 0 固定为主公。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Lord,
    Loyalist,
    Rebel,
    Traitor,
}

/// 斗地主开局变体（T11 模式变体，规则 v205）：仅影响建局初始发牌数，
/// 不入 `GameMode` 编码（开局手牌数已进状态与哈希，存档无需变体位）。
/// 上游 `doudizhu_mode` 配置的 online 变体（选将后 infoHp 加成+地骑虚拟
/// 装备）不建模，登记差异。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoudizhuVariant {
    /// 标准：各 4 张。
    Standard,
    /// binglin：地主 5 张、农民 4 张（上游 `next.num` 覆盖同口径）。
    Binglin,
}

/// Boss 关卡流程种类（ADR 0783 §3.1；B0 默认 `None`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum BossStageKind {
    /// 未启用关卡流程面（现有 Boss 局默认）。
    #[default]
    None = 0,
    /// 十殿阎罗还人链（B3）。
    ShidianYanluo = 1,
    /// 四象隐藏链（B1）。
    SixiangChain = 2,
    /// 青/赤/白翼轮次门控（B4）。
    WingGate = 3,
}

/// Boss 关卡流程挂起询问（B0 仅占位编码；写入留给后续批）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BossStagePending {
    /// 选将询问：候选为目录武将 def 下标。
    SelectGeneral { candidates: Vec<u16> },
}

/// Boss 关卡流程态（ADR 0783；入快照/哈希/存档 v79）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BossStageState {
    pub kind: BossStageKind,
    /// 十殿：0..3 对应关卡说明四关；其他 kind 自解释。
    pub level: u8,
    /// `_status.shidianyanluo_mengpo`：孟婆已乱入。
    pub mengpo_spawned: bool,
    /// `_status.shidianyanluo_mengpodie`：孟婆已死。
    pub mengpo_died: bool,
    /// 还人禁将表（目录武将 def 下标；B0 恒空，写入留给 B3）。
    pub banned_generals: Vec<u16>,
    /// 青/赤/白翼本局已武装的 round 门（位旗；B0 恒 0）。
    pub wing_armed: u8,
    /// 选将/延迟换将挂起（B0 恒 None）。
    pub pending: Option<BossStagePending>,
    /// 浴火·关卡循环型（T11 Boss 庞统族批 118，规则 v306）：上游
    /// `game.bossinfo.loopType = 1`（浴火后置位；0=未置位）。
    pub loop_type: u8,
}

impl BossStageState {
    /// 默认空态：`kind=None`、等级 0、旗 false、禁将空、翼未武装、无挂起。
    pub fn default_none() -> Self {
        Self {
            kind: BossStageKind::None,
            level: 0,
            mengpo_spawned: false,
            mengpo_died: false,
            banned_generals: Vec::new(),
            wing_armed: 0,
            pending: None,
            loop_type: 0,
        }
    }
}

/// 对局模式。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameMode {
    /// 原型模式：存活数 ≤ 1 即终局，无阵营与奖惩。
    Prototype,
    /// 身份局：主公体力上限 +1；反贼/内奸全灭主公方胜、主公死按阵营判负，
    /// 杀反贼摸三、主公杀忠臣弃全部（正式模式胜负规则，见覆盖清单）。
    Identity { roles: Vec<Role> },
    /// 国战（N11 M2/M3）：势力模式——场上仅剩同一势力时该势力胜。
    Kingdoms,
    /// 对决 2v2 基线（P2/N08 模式入口批一，规则 v80）：两队对抗——
    /// 消灭敌方全队胜；奖惩：杀死敌方摸三张、杀死友方弃掉自己全部
    /// 手牌与装备（上游 versus 2v2 基线）。`substitutes`=每队替补武将
    /// 池（与 `teams` 同队序；规则 v206 替补面：死亡座位从本队池弹队首
    /// 换将复活，空池照常死亡）。
    Versus {
        teams: Vec<Vec<PlayerId>>,
        #[allow(missing_docs)]
        substitutes: Vec<Vec<u16>>,
    },
    /// 斗地主（P2/N08 模式入口批二，规则 v81）：地主 1 名对农民若干——
    /// 地主死亡农民胜、农民全灭地主胜；无击杀奖惩（上游 dieAfter 仅
    /// UI+判负；特殊开局摸牌变体未建模）。
    Doudizhu { lord: PlayerId },
    /// Boss（P2/N08 模式入口批三-a，规则 v82；批三-b v84 增 fellow；
    /// D4/ADR 0783 批 B0 增 `stage` 关卡流程态）：
    /// 单体对抗 + 重整复活回环——死亡玩家轮到回合时计数 +1 并回血/摸牌，
    /// 达阈值复活；boss 死亡挑战者胜、挑战者（含助战 fellow 为 boss 方）
    /// 全灭 boss 胜。fellows=中途入队的助战座位（队尾追加，规则 v84）。
    /// `stage`=关卡流程面（默认 `kind=None`，B0 无对局可达语义）。
    Boss {
        boss: PlayerId,
        chongzheng: u8,
        fellows: Vec<PlayerId>,
        stage: BossStageState,
    },
    /// KOF 引擎层（P2/N08 模式入口批四，规则 v83）：单败会话状态入
    /// 引擎——双方将池（GENERAL 下标）、比分、局数；单局终局时推进
    /// 会话（败者队首淘汰、胜者队首轮转、比分/局数 +1）。
    Kof {
        pools: [Vec<u16>; 2],
        wins: [u32; 2],
        games: u32,
    },
}

use crate::event::DetachedKind;

/// 核心（无执行位置）的对局状态字段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameStateCore {
    pub revision: u64,
    /// 规则版本（开目录期固定入状态）：入摘要、随快照做恢复绑定，
    /// 保证跨规则版本的哈希与恢复不互认。
    pub ruleset_version: u64,
    pub mode: GameMode,
    pub players: Vec<PlayerState>,
    /// 座位环（T19b，规则 v201/ADR 0781）：ring[pos]=PlayerId，恒为恒等
    /// 排列（建局 0..n 初始化，swapSeat 重排）；座位语义全部经由
    /// ring_next/ring_prev/ring_pos，禁止 +1%n 直算。
    pub seat_ring: Vec<PlayerId>,
    pub deck: Vec<CardId>,
    pub discard: Vec<CardId>,
    /// R05（规则 333）：抽牌 gain 取消/死亡且源未指定返还时的实体去处；
    /// `Unclaimed`/`Destroyed` 均不可洗回，`Destroyed` 不能普通 gain。
    pub detached_cards: Vec<(CardId, DetachedKind)>,
    /// 结算中的牌（出牌阶段使用、尚未进弃牌堆）。
    pub table: Vec<CardId>,
    pub rng: RuleRng,
    pub turn: TurnState,
    /// 技能实例表：键 =（拥有者，定义号）。
    pub skills: BTreeMap<(PlayerId, SkillDefId), SkillInstance>,
    /// view-as 延时锦囊声明（国色）：牌 id → 声明牌种。
    /// 判定生效时按声明牌种执行（目录牌种不再可靠）。
    pub declared_kinds: BTreeMap<CardId, crate::catalog::CardKind>,
    /// 装备栏改写（swd 批 143 改造，规则 v318）：牌 → 实际占用的装备栏
    /// （上游 `gaizao` 把牌改名为 `name_gaizao_equipN`、`subtype` 改槽但
    /// **保留原装备技能**；原生以栏位覆盖承载——牌种不变故技能保留）。
    pub declared_slots: BTreeMap<CardId, crate::catalog::EquipSlot>,
    /// 五行属性表（G1 wuxing：牌 id → 属性 0=金 1=木 2=水 3=火 4=土；
    /// 仅启用 wuxing 时非空）。
    pub card_natures: BTreeMap<CardId, u8>,
    /// 当前地（mtg land 首切片，规则 v100）：使用地时覆盖（上游
    /// changeLand 全局唯一 ui.land 同口径；效果全局生效）。
    pub current_land: Option<crate::catalog::CardKind>,
    pub outcome: Option<Outcome>,
    /// T8 具名 filter 原语面（标记 / 技能牌堆 / 本回合丢失 / once / 旗标）。
    /// 空表不写入确定性摘要，标准局面哈希与 v295 golden 一致。
    pub vm_aux: VmAux,
}

/// VM 扩展技能的可观察状态。空表编码为零字节（哈希）或长度前缀 0（存档 v80）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VmAux {
    /// (座位, 标记定义) → 层数。
    pub marks: BTreeMap<(u8, u32), i32>,
    /// (座位, 技能名) → 牌堆。
    pub piles: BTreeMap<(u8, String), Vec<CardId>>,
    /// 本回合离开该座位的牌（弃置或被移走）。
    pub lost: BTreeMap<u8, Vec<CardId>>,
    /// (座位, 键, 范围)：1 回合 / 2 轮 / 3 整局。在集合中 = 已消费。
    pub once: BTreeSet<(u8, String, u8)>,
    /// 效果写入的具名旗标（pending / assigned）。
    pub flags: BTreeMap<(u8, String), u32>,
    /// 本回合各技能已触发的伤害次数（first/second damage）。
    pub hits: BTreeMap<(u8, String), u32>,
    /// 技能私有存储（PA12 批⑤-3）：键 "技能名#storage_key"（VM 桥无实例
    /// id，按执行中技能名；上游 player.storage.<skill>[key] 同语义——
    /// phase_play 触发段发 StorageSet(key,0) 清零）。
    pub storage: BTreeMap<String, u32>,
    /// 本回合摸入的张数。
    pub drawn: BTreeMap<u8, u32>,
}

impl VmAux {
    pub fn is_empty(&self) -> bool {
        self.marks.is_empty()
            && self.piles.is_empty()
            && self.lost.is_empty()
            && self.once.is_empty()
            && self.flags.is_empty()
            && self.hits.is_empty()
            && self.drawn.is_empty()
            && self.storage.is_empty()
    }

    /// 换回合：丢掉回合范围的 once、丢失史、命中与摸牌计数。
    pub fn clear_turn(&mut self) {
        self.lost.clear();
        self.hits.clear();
        self.drawn.clear();
        self.once.retain(|(_, _, scope)| *scope != 1);
        let turn_skills: Vec<(u8, String)> = self
            .flags
            .keys()
            .filter_map(|(p, k)| {
                k.strip_prefix("skill_turn:")
                    .map(|raw| (*p, format!("skill:{raw}")))
            })
            .collect();
        for key in turn_skills {
            self.flags.remove(&key);
        }
        self.flags.retain(|(_, key), _| {
            key != "vm_skipped_phase_count"
                && key != "draw_phase_bonus"
                && key != "play_cat_mask"
                && key != "draw_phase_bottom"
                && key != "non_locked_suppression_source"
                && key != "suppress_non_locked_turn"
                && key != "ban_active"
                && key != "card_policy"
                && key != "unique_cat_target"
                && key != "unique_cat_mask"
                && key != "selected_cards_len"
                && key != "selected_card"
                && key != "selected_targets_len"
                && key != "selected_target"
                && !key.starts_with("selected_card:")
                && !key.starts_with("selected_target:")
                && !key.starts_with("card_policy:")
                && !key.starts_with("unique_cat_target:")
                && !key.starts_with("unique_cat_mask:")
                && !key.starts_with("skill_turn:")
                && !key.starts_with("skill_disabled_turn:")
                && !key.starts_with("skill_disabled_raw:")
                && !key.starts_with("skill_banned_turn:")
                && !key.starts_with("card_damage_bonus:")
                && !key.starts_with("sha_target_damage_bonus:")
                && !key.starts_with("sha_unrespondable:")
        });
    }

    /// 轮次推进：丢掉轮范围的 once。
    pub fn clear_round(&mut self) {
        self.once.retain(|(_, _, scope)| *scope != 2);
    }

    /// 登记 `owner` 本次 VM 技能调用的预选牌列表（`CardId` 序列，存于 `flags` 以免污染实体 `piles`）。
    pub fn set_selected_cards(&mut self, owner: PlayerId, cards: &[CardId]) {
        self.flags.retain(|(p, k), _| {
            *p != owner
                || (k != "selected_cards_len"
                    && k != "selected_card"
                    && !k.starts_with("selected_card:"))
        });
        if cards.is_empty() {
            return;
        }
        self.flags
            .insert((owner, "selected_cards_len".to_string()), cards.len() as u32);
        self.flags
            .insert((owner, "selected_card".to_string()), cards[0]);
        for (i, &c) in cards.iter().enumerate() {
            self.flags
                .insert((owner, format!("selected_card:{i}")), c);
        }
    }

    /// 读取 `owner` 登记的预选牌列表。
    pub fn selected_cards(&self, owner: PlayerId) -> Vec<u32> {
        if let Some(&len) = self.flags.get(&(owner, "selected_cards_len".to_string())) {
            return (0..len)
                .filter_map(|i| self.flags.get(&(owner, format!("selected_card:{i}"))).copied())
                .collect();
        }
        if let Some(&single) = self.flags.get(&(owner, "selected_card".to_string())) {
            if single != 0 {
                return vec![single];
            }
        }
        Vec::new()
    }

    /// 登记 `owner` 本次 VM 技能调用的预选目标列表（1-based `rule_api::PlayerId` 裸值）。
    pub fn set_selected_targets(&mut self, owner: PlayerId, targets: &[u32]) {
        self.flags.retain(|(p, k), _| {
            *p != owner
                || (k != "selected_targets_len"
                    && k != "selected_target"
                    && !k.starts_with("selected_target:"))
        });
        if targets.is_empty() {
            return;
        }
        self.flags.insert(
            (owner, "selected_targets_len".to_string()),
            targets.len() as u32,
        );
        self.flags
            .insert((owner, "selected_target".to_string()), targets[0]);
        for (i, &t) in targets.iter().enumerate() {
            self.flags
                .insert((owner, format!("selected_target:{i}")), t);
        }
    }

    /// 按 0-based 座位号登记 `owner` 的预选目标列表（内部自动转为 1-based `rule_api::PlayerId` 裸值）。
    pub fn set_selected_target_seats(&mut self, owner: PlayerId, seats: &[PlayerId]) {
        let raw: Vec<u32> = seats.iter().map(|&s| u32::from(s) + 1).collect();
        self.set_selected_targets(owner, &raw);
    }

    /// 读取 `owner` 登记的预选目标列表（1-based `rule_api::PlayerId` 裸值）。
    pub fn selected_targets(&self, owner: PlayerId) -> Vec<u32> {
        if let Some(&len) = self.flags.get(&(owner, "selected_targets_len".to_string())) {
            return (0..len)
                .filter_map(|i| {
                    self.flags
                        .get(&(owner, format!("selected_target:{i}")))
                        .copied()
                })
                .collect();
        }
        if let Some(&single) = self.flags.get(&(owner, "selected_target".to_string())) {
            if single != 0 {
                return vec![single];
            }
        }
        Vec::new()
    }
}

/// 完整对局状态 = 核心字段 + 执行位置。
#[derive(Debug, Clone)]
pub struct GameState {
    pub core: GameStateCore,
    pub resolution: crate::resolution::Resolution,
}

mod general_assembly;
pub use general_assembly::apply_general_assembly;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum KingdomIdentity {
    #[default]
    Unknown,
    Ye,
    Declared(String),
}
