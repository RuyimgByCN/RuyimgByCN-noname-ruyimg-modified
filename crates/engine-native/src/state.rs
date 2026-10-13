//! 紧凑对局状态：只保存影响未来规则结果的数据。
//!
//! 三分结构中的「对局状态」部分：
//! - 只读规则目录在引擎侧 `Arc` 共享，不在此处；
//! - 宿主数据（展示日志、性能统计）在引擎侧，也不在此处；
//! - 执行位置（事件栈/挂起机器/开放选择）归 `resolution::Resolution`，
//!   为便于调度器拆借用而与核心字段并排放在 `GameState` 内。

use std::collections::BTreeMap;

use crate::event::Phase;
use crate::ids::{CardId, PlayerId, SkillDefId};
use crate::rng::RuleRng;

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
}

impl PlayerState {
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
            death_cancelled: false,
            shiying_used: false,
            huahu_used: false,
            xuanning_mark: 0,
            shedu: 0,
            juyuan_storage: 0,
            fengyin_active: false,
            lingqu_count: 0,
            xingqiu_toggled: false,
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
            wunature: None,
            boss_chongzheng: 0,
            boss_mad: false,
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
    /// 突击（U07 ow 士兵76，规则 v118）：本出牌阶段已使用牌数（上游
    /// countUsed 同口径；Play 阶段开始清零、换回合自然清零）。
    pub cards_used_play_phase: u32,
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
    /// Boss（P2/N08 模式入口批三-a，规则 v82；批三-b v84 增 fellow）：
    /// 单体对抗 + 重整复活回环——死亡玩家轮到回合时计数 +1 并回血/摸牌，
    /// 达阈值复活；boss 死亡挑战者胜、挑战者（含助战 fellow 为 boss 方）
    /// 全灭 boss 胜。fellows=中途入队的助战座位（队尾追加，规则 v84）。
    Boss { boss: PlayerId, chongzheng: u8, fellows: Vec<PlayerId> },
    /// KOF 引擎层（P2/N08 模式入口批四，规则 v83）：单败会话状态入
    /// 引擎——双方将池（GENERAL 下标）、比分、局数；单局终局时推进
    /// 会话（败者队首淘汰、胜者队首轮转、比分/局数 +1）。
    Kof { pools: [Vec<u16>; 2], wins: [u32; 2], games: u32 },
}

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
    /// 结算中的牌（出牌阶段使用、尚未进弃牌堆）。
    pub table: Vec<CardId>,
    pub rng: RuleRng,
    pub turn: TurnState,
    /// 技能实例表：键 =（拥有者，定义号）。
    pub skills: BTreeMap<(PlayerId, SkillDefId), SkillInstance>,
    /// view-as 延时锦囊声明（国色）：牌 id → 声明牌种。
    /// 判定生效时按声明牌种执行（目录牌种不再可靠）。
    pub declared_kinds: BTreeMap<CardId, crate::catalog::CardKind>,
    /// 五行属性表（G1 wuxing：牌 id → 属性 0=金 1=木 2=水 3=火 4=土；
    /// 仅启用 wuxing 时非空）。
    pub card_natures: BTreeMap<CardId, u8>,
    /// 当前地（mtg land 首切片，规则 v100）：使用地时覆盖（上游
    /// changeLand 全局唯一 ui.land 同口径；效果全局生效）。
    pub current_land: Option<crate::catalog::CardKind>,
    pub outcome: Option<Outcome>,
}

impl GameStateCore {
    pub fn player(&self, p: PlayerId) -> &PlayerState {
        &self.players[p as usize]
    }

    pub fn player_mut(&mut self, p: PlayerId) -> &mut PlayerState {
        &mut self.players[p as usize]
    }

    pub fn alive_players(&self) -> impl Iterator<Item = PlayerId> + '_ {
        self.players
            .iter()
            .enumerate()
            .filter(|(_, p)| p.alive)
            .map(|(i, _)| i as PlayerId)
    }

    pub fn alive_count(&self) -> usize {
        self.players.iter().filter(|p| p.alive).count()
    }

    /// 身份局的身份表（非身份局为 None）。
    pub fn identity_roles(&self) -> Option<&[Role]> {
        match &self.mode {
            GameMode::Identity { roles } => Some(roles),
            GameMode::Prototype | GameMode::Kingdoms | GameMode::Versus { .. } | GameMode::Doudizhu { .. } | GameMode::Boss { .. } | GameMode::Kof { .. } => None,
        }
    }

    /// KOF 会话状态（非 KOF 为 None）。
    pub fn kof_session(&self) -> Option<(&[Vec<u16>], &[u32; 2], u32)> {
        match &self.mode {
            GameMode::Kof { pools, wins, games } => Some((pools, wins, *games)),
            _ => None,
        }
    }

    /// Boss 模式参数：boss 座位与重整阈值（非 Boss 为 None）。
    pub fn boss_mode(&self) -> Option<(PlayerId, u8)> {
        match &self.mode {
            GameMode::Boss { boss, chongzheng, .. } => Some((*boss, *chongzheng)),
            _ => None,
        }
    }

    /// Boss 模式阵营：boss 座位与助战 fellow 为 boss 方，其余为挑战者
    /// 方（非 Boss 模式一律 false——神恩光环仅 Boss 场生效）。
    pub fn is_boss_side(&self, p: PlayerId) -> bool {
        match &self.mode {
            GameMode::Boss { boss, fellows, .. } => p == *boss || fellows.contains(&p),
            _ => false,
        }
    }

    /// Boss 模式助战座位表（非 Boss 为 None）。
    pub fn boss_fellows(&self) -> Option<&[PlayerId]> {
        match &self.mode {
            GameMode::Boss { fellows, .. } => Some(fellows),
            _ => None,
        }
    }

    /// 斗地主模式的地主座位（非斗地主为 None）。
    pub fn doudizhu_lord(&self) -> Option<PlayerId> {
        match &self.mode {
            GameMode::Doudizhu { lord } => Some(*lord),
            _ => None,
        }
    }

    /// 对决模式的队伍表（非对决为 None）。队伍互斥且并集=全部座位。
    pub fn versus_teams(&self) -> Option<&[Vec<PlayerId>]> {
        match &self.mode {
            GameMode::Versus { teams, .. } => Some(teams),
            _ => None,
        }
    }

    /// 对决模式的替补武将池（非对决为 None；与队伍同队序，可为空池）。
    pub fn versus_substitutes(&self) -> Option<&[Vec<u16>]> {
        match &self.mode {
            GameMode::Versus { substitutes, .. } => Some(substitutes),
            _ => None,
        }
    }

    /// 对决模式：座位 `p` 的队伍下标（非对决/不在队中为 None）。
    pub fn versus_team_of(&self, p: PlayerId) -> Option<usize> {
        self.versus_teams()?.iter().position(|t| t.contains(&p))
    }

    /// 国战（N11 M3 门控）：座位 `p` 的**已声明**势力——
    /// 主将明置按主将势力；主将暗置而副将明置按副将势力
    /// （半明置）；双暗置或槽空 → None（势力未定）。
    pub fn declared_faction(
        &self,
        catalog: &crate::catalog::Catalog,
        p: PlayerId,
    ) -> Option<crate::catalog::Faction> {
        let pl = self.player(p);
        if pl.revealed_generals[0] {
            catalog.general_meta(pl.general_slots[0]?).map(|(_, f, ..)| f)
        } else if pl.revealed_generals[1] {
            catalog.general_meta(pl.general_slots[1]?).map(|(_, f, ..)| f)
        } else {
            None
        }
    }

    /// 对局内可见势力：国战取明置势力（暗置不出势力）；其余模式
    /// （身份/原型）主将势力直接可见——标准玩法无暗置概念，护驾/
    /// 激将等势力资格按此判定。
    pub fn effective_faction(
        &self,
        catalog: &crate::catalog::Catalog,
        p: PlayerId,
    ) -> Option<crate::catalog::Faction> {
        match self.mode {
            GameMode::Kingdoms => self.declared_faction(catalog, p),
            _ => {
                let pl = self.player(p);
                pl.general_slots[0]
                    .and_then(|g| catalog.general_meta(g).map(|(_, f, ..)| f))
            }
        }
    }

    /// 大势力判定（zhenfa 地载阵首卡，规则 v110；上游国战 isMajor 同
    /// 口径简化）：effective_faction 非空者按势力统计存活数，人数最大
    /// 的势力（并列同算）为大势力；未明置/无主将者不属大势力。上游
    /// forceMajor 技能标特例未建模（登记）。
    pub fn is_major_faction(
        &self,
        catalog: &crate::catalog::Catalog,
        p: PlayerId,
    ) -> bool {
        let mut counts: std::collections::BTreeMap<u8, u32> = Default::default();
        for q in self.alive_players() {
            if let Some(f) = self.effective_faction(catalog, q) {
                *counts.entry(f as u8).or_insert(0) += 1;
            }
        }
        let Some(max) = counts.values().copied().max() else {
            return false;
        };
        match self.effective_faction(catalog, p) {
            Some(f) => counts.get(&(f as u8)) == Some(&max) && max >= 1,
            None => false,
        }
    }

    /// 主公资格（zhu 技能：护驾/激将）：身份局要求主公身份；
    /// 无身份体系的模式持技即可。
    pub fn is_lord(&self, p: PlayerId) -> bool {
        match self.identity_roles() {
            Some(roles) => roles.get(p as usize) == Some(&Role::Lord),
            None => true,
        }
    }

    /// 座位最短距离（顺/逆时针取小）。
    pub fn seat_distance(&self, a: PlayerId, b: PlayerId) -> u8 {
        let n = self.players.len() as u8;
        let (a, b) = (a % n, b % n);
        let cw = b.wrapping_sub(a) % n;
        let ccw = a.wrapping_sub(b) % n;
        cw.min(ccw)
    }

    // ---- 座位环（T19b/ADR 0781）----

    /// p 在座位环中的位置（环长度与玩家数不符时退回恒等位）。
    pub fn ring_pos(&self, p: PlayerId) -> usize {
        if self.seat_ring.len() == self.players.len() {
            self.seat_ring.iter().position(|x| *x == p).unwrap_or(p as usize)
        } else {
            p as usize
        }
    }

    /// 座位环中 p 的下一位（不计存活；出界退恒等）。
    pub fn ring_next(&self, p: PlayerId) -> PlayerId {
        let n = self.players.len() as PlayerId;
        let pos = self.ring_pos(p);
        self.seat_ring.get((pos + 1) % (n as usize)).copied().unwrap_or((p + 1) % n)
    }

    /// 座位环中 p 的上一位（不计存活）。
    pub fn ring_prev(&self, p: PlayerId) -> PlayerId {
        let n = self.players.len() as PlayerId;
        let pos = self.ring_pos(p);
        if pos == 0 {
            *self.seat_ring.last().unwrap_or(&(n - 1))
        } else {
            self.seat_ring.get(pos - 1).copied().unwrap_or((p + n - 1) % n)
        }
    }

    /// swapSeat 座位重排（T19b，规则 v201）：交换 a/b 的环位置（PlayerId
    /// 身份不变，座位语义即时生效；上游 game.swapSeat 同口径）。
    pub fn swap_seat(&mut self, a: PlayerId, b: PlayerId) {
        if a == b || self.seat_ring.len() != self.players.len() {
            return;
        }
        let pa = self.ring_pos(a);
        let pb = self.ring_pos(b);
        self.seat_ring.swap(pa, pb);
    }

    /// 计算距离（含 ±1 马修饰，下限 1；座位距离只计存活玩家）。
    /// 上游/旧线一致性待核对，见覆盖清单行为差异表。
    pub fn computed_distance(
        &self,
        catalog: &crate::catalog::Catalog,
        a: PlayerId,
        b: PlayerId,
    ) -> u32 {
        use crate::catalog::CardKind;
        // 链钩（U05 ow 路霸，规则 v128）：本回合被钩者——路霸算到其
        // 距离恒 1（上游 liangou2 mod globalTo -Infinity 经 get.distance
        // 下限钳制同口径；仅路霸出牌阶段可钩住，换回合解除）
        if a == self.turn.current && self.turn.liangou_hook == Some(b) {
            return 1;
        }
        // 跳跃（U05 ow 温斯顿，规则 v137）：本回合跳跃目标——自己算到
        // 其距离恒 1（上游 feitiao2 mod globalFrom -Infinity 同口径；
        // 仅自己回合内置位，换回合解除）
        if a == self.turn.current && self.turn.feitiao_target == Some(b) {
            return 1;
        }
        // 月行（U05 xianjian 龙幽，规则 v159）：距离基数交换——持对偶
        // 者以配对者座位为基数（上游 yuexing2 mod globalFrom 的 pure 距
        // 离代换同口径；登记差异：代换仅座位基数，马匹等修饰照常叠加）
        let base = match self.player(a).yuexing_pair {
            Some(p) if p != a => self.seat_distance_alive(p, b) as u32,
            _ => self.seat_distance_alive(a, b) as u32,
        };
        let plus = self
            .player(b)
            .equips
            .iter()
            .filter(|c| catalog.cards[**c as usize].kind == CardKind::HorsePlus)
            .count() as u32;
        let minus = self
            .player(a)
            .equips
            .iter()
            .filter(|c| catalog.cards[**c as usize].kind == CardKind::HorseMinus)
            .count() as u32;
        // 马术（马超）：锁定技——计算与他人的距离时始终 -1（下限 1）
        let mashu = catalog
            .skill_by_name(crate::content::standard::MA_SHU)
            .map(|d| self.skills.get(&(a, d)).map(|i| i.enabled).unwrap_or(false))
            .unwrap_or(false);
        let minus = minus + u32::from(mashu);
        // 凶兽（Boss 模式共有技，规则 v208）：锁定技——你与其他角色
        // 距离 -1（上游 boss_xiongshou mod globalFrom distance-1 同口径）
        let xiongshou = catalog
            .skill_by_name(crate::content::standard::BOSS_XIONGSHOU)
            .map(|d| self.skills.get(&(a, d)).map(|i| i.enabled).unwrap_or(false))
            .unwrap_or(false);
        let minus = minus + u32::from(xiongshou);
        // 妖兽（Boss 神兽共有技，规则 v221）：锁定技——你与其他角色
        // 距离 -2（上游 boss_yaoshou mod globalFrom distance-2 同口径）
        let yaoshou = catalog
            .skill_by_name(crate::content::standard::BOSS_YAOSHOU)
            .map(|d| self.skills.get(&(a, d)).map(|i| i.enabled).unwrap_or(false))
            .unwrap_or(false);
        let minus = minus + 2 * u32::from(yaoshou);
        // 义从（公孙瓒）：锁定技——你计算与其他角色的距离 -1；其他角色
        // 计算与你的距离 +1（你的体力值不大于 2 时）。
        let yicong_from = catalog
            .skill_by_name(crate::content::standard::YI_CONG)
            .map(|d| self.skills.get(&(a, d)).map(|i| i.enabled).unwrap_or(false))
            .unwrap_or(false);
        let yicong_to = catalog
            .skill_by_name(crate::content::standard::YI_CONG)
            .map(|d| {
                self.skills.get(&(b, d)).map(|i| i.enabled).unwrap_or(false)
                    && self.player(b).hp <= 2
            })
            .unwrap_or(false);
        let mut minus = minus + u32::from(yicong_from);
        let mut plus = plus + u32::from(yicong_to);
        // 抗yin2（U07 玩点论杀留赞，规则 v114）：回合内临时——持有者算
        // 到其他角色距离 -X（上游 kangyin2 mod globalFrom 同口径）
        if let Some((o, x)) = self.turn.kangyin2 {
            if o == a {
                minus += x;
            }
        }
        // 突击（U07 ow 士兵76，规则 v118）：锁定——持有者回合内每已使
        // 用一张牌，进攻距离 -1（上游 tuji mod globalFrom countUsed 同
        // 口径）
        if let Some(def) = catalog.skill_by_name(crate::content::host::WUDIAN_TUJI_KEY) {
            let tuji = self
                .skills
                .get(&(a, def))
                .map(|i| i.enabled)
                .unwrap_or(false);
            if tuji
                && self.turn.current == a
                && self.turn.cards_used_play_phase > 0
            {
                minus += self.turn.cards_used_play_phase;
            }
        }
        // 绝道（U07 玩点论杀刘焉，规则 v116）：锁定——**持技者**横置时
        // 双向距离 +1（上游 mod 挂在 juedao 技能上：globalFrom 查 from
        // 的技能、globalTo 查 to 的技能——非持技者的横置不影响距离）
        if let Some(def) = catalog.skill_by_name(crate::content::host::WUDIAN_JUEDAO_KEY) {
            let juedao_a = self
                .skills
                .get(&(a, def))
                .map(|i| i.enabled)
                .unwrap_or(false);
            let juedao_b = self
                .skills
                .get(&(b, def))
                .map(|i| i.enabled)
                .unwrap_or(false);
            if juedao_a && self.player(a).linked {
                plus += 1;
            }
            if juedao_b && self.player(b).linked {
                plus += 1;
            }
        }
        // 车阵（U07 玩点论杀田豫，规则 v115）：锁定——有装备时你算到
        // 他人 -1；目标无装备时其被算到 +1（上游 chezhen mod globalFrom/
        // globalTo 同口径）
        if let Some(def) = catalog.skill_by_name(crate::content::host::WUDIAN_CHEZHEN_KEY) {
            let chezhen_a = self
                .skills
                .get(&(a, def))
                .map(|i| i.enabled)
                .unwrap_or(false);
            let chezhen_b = self
                .skills
                .get(&(b, def))
                .map(|i| i.enabled)
                .unwrap_or(false);
            if chezhen_a && !self.player(a).equips.is_empty() {
                minus += 1;
            }
            if chezhen_b && self.player(b).equips.is_empty() {
                plus += 1;
            }
        }
        // 带火风轮/霞云彩绫（U05 swd 第十四批，规则 v98）：命名距离
        // 修饰（上游 distance globalFrom/globalTo 同口径）——
        // 带火风轮：持有者算到他人 -2、他人算到持有者 -1；
        // 霞云彩绫：持有者算到他人 +1、他人算到持有者 +2。
        let count_named = |equips: &[crate::ids::CardId], kind: CardKind| -> u32 {
            equips
                .iter()
                .filter(|c| catalog.cards[**c as usize].kind == kind)
                .count() as u32
        };
        let minus = minus
            + 2 * count_named(&self.player(a).equips, CardKind::Daihuofenglun)
            + count_named(&self.player(b).equips, CardKind::Daihuofenglun);
        let plus = plus
            + count_named(&self.player(a).equips, CardKind::Xiayuncailing)
            + 2 * count_named(&self.player(b).equips, CardKind::Xiayuncailing);
        (base + plus).saturating_sub(minus).max(1)
    }

    /// 攻击范围：武器射程（泛型 Weapon 走 CardDef.range，命名武器走
    /// 牌种固定射程），无武器为 1。
    pub fn attack_range(&self, catalog: &crate::catalog::Catalog, p: PlayerId) -> u32 {
        use crate::catalog::CardKind;
        self.player(p)
            .equips
            .iter()
            .filter_map(|c| {
                let def = &catalog.cards[*c as usize];
                if def.kind == CardKind::Weapon {
                    Some(def.range)
                } else {
                    def.kind.named_weapon_range()
                }
            })
            .max()
            .unwrap_or(1)
    }

    /// 存活玩家间的座位最短距离（死亡玩家的座位不计数）。
    pub fn seat_distance_alive(&self, a: PlayerId, b: PlayerId) -> u8 {
        let alive: Vec<PlayerId> = self.alive_players().collect();
        let n = alive.len() as i16;
        let ia = alive.iter().position(|p| *p == a).expect("a must be alive") as i16;
        let ib = alive.iter().position(|p| *p == b).expect("b must be alive") as i16;
        let cw = (ib - ia).rem_euclid(n) as u8;
        let ccw = (ia - ib).rem_euclid(n) as u8;
        cw.min(ccw)
    }

    /// 有效性别（批③-b 第八把 雌雄）：主将槽有武将时取武将性别，
    /// 无武将默认男性（「性别有槽时筛选、无槽默认」惯例）。
    pub fn effective_gender(
        &self,
        p: PlayerId,
    ) -> crate::content::standard::Gender {
        self.player(p).general_slots[0]
            .and_then(crate::content::standard::general_gender)
            .unwrap_or(crate::content::standard::Gender::Male)
    }

    // ---- 国战阵法派生判定（T19a，规则 v200；上游 guozhan patch
    // player.js 简化口径：构逆/野心家结盟/undist 不建模，登记差异）----

    /// 国战友方判定（上游 isFriendOf 简化）：双方明置势力已知且相同；
    /// 同一角色恒真。
    pub fn is_friend_of(&self, catalog: &crate::catalog::Catalog, a: PlayerId, b: PlayerId) -> bool {
        if a == b {
            return true;
        }
        match (
            self.effective_faction(catalog, a),
            self.effective_faction(catalog, b),
        ) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    }

    /// 存活左右邻居（座位环跳过死亡座位——登记差异：上游 getNext 含死
    /// 亡座位时语义不明，保守取存活）。
    pub fn alive_neighbors(&self, p: PlayerId) -> (Option<PlayerId>, Option<PlayerId>) {
        let n = self.players.len() as PlayerId;
        if n <= 1 {
            return (None, None);
        }
        let alive = |q: PlayerId| self.player(q).alive;
        let mut next = self.ring_next(p);
        let mut guard = 0;
        while next != p && !alive(next) && guard < n {
            next = self.ring_next(next);
            guard += 1;
        }
        let next = if next == p || !alive(next) { None } else { Some(next) };
        let mut prev = self.ring_prev(p);
        guard = 0;
        while prev != p && !alive(prev) && guard < n {
            prev = self.ring_prev(prev);
            guard += 1;
        }
        let prev = if prev == p || !alive(prev) { None } else { Some(prev) };
        (next, prev)
    }

    /// 被围攻（上游 sieged 简化）：己方明置势力已知，存活左右邻均已知
    /// 且互异、非己方、且互为友方（同势力夹击）。
    pub fn sieged(&self, catalog: &crate::catalog::Catalog, p: PlayerId) -> bool {
        if self.effective_faction(catalog, p).is_none() {
            return false;
        }
        let (Some(next), Some(prev)) = self.alive_neighbors(p) else {
            return false;
        };
        if next == prev {
            return false;
        }
        if self.effective_faction(catalog, next).is_none() {
            return false;
        }
        if self.is_friend_of(catalog, next, p) {
            return false;
        }
        self.is_friend_of(catalog, next, prev)
    }

    /// q 围攻 p（q 为 p 的存活邻且 p 处于被围攻）。
    pub fn sieges(&self, catalog: &crate::catalog::Catalog, q: PlayerId, p: PlayerId) -> bool {
        let (next, prev) = self.alive_neighbors(p);
        (next == Some(q) || prev == Some(q)) && self.sieged(catalog, p)
    }

    /// 队列判定（上游 inline）：自己回合外有 ≥1 连续同势力友邻。
    pub fn inline(&self, catalog: &crate::catalog::Catalog, p: PlayerId) -> bool {
        if self.effective_faction(catalog, p).is_none() {
            return false;
        }
        let (next, prev) = self.alive_neighbors(p);
        next.is_some_and(|q| self.is_friend_of(catalog, p, q))
            || prev.is_some_and(|q| self.is_friend_of(catalog, p, q))
    }

    /// q 是否处于 p 的连续同势力友方段内（或 q==p 且 p 处于队列）。
    pub fn inline_with(&self, catalog: &crate::catalog::Catalog, p: PlayerId, q: PlayerId) -> bool {
        if !self.inline(catalog, p) {
            return false;
        }
        if q == p {
            return true;
        }
        if !self.is_friend_of(catalog, p, q) {
            return false;
        }
        // forward/backward 沿座位环（T19b）
        let mut cur = p;
        loop {
            let nxt = self.ring_next(cur);
            if nxt == p || nxt == cur || !self.player(nxt).alive
                || !self.is_friend_of(catalog, p, nxt)
            {
                break;
            }
            if nxt == q {
                return true;
            }
            cur = nxt;
        }
        let mut cur = p;
        loop {
            let prv = self.ring_prev(cur);
            if prv == p || prv == cur || !self.player(prv).alive
                || !self.is_friend_of(catalog, p, prv)
            {
                break;
            }
            if prv == q {
                return true;
            }
            cur = prv;
        }
        false
    }

    pub fn hand_limit(&self, catalog: &crate::catalog::Catalog, p: PlayerId) -> usize {
        // 星体改造（U05 swd 灵剑，规则 v198）：手牌上限 -N 累加
        let mut limit = (self.player(p).hp.max(0) as usize)
            .saturating_sub(self.player(p).xingtigaizao_count as usize);
        // 无脏（Boss 混沌，规则 v209）：手牌上限基数 0——体力贡献清零
        // （上游 boss_wuzang mod maxHandcard `num - player.hp` 同口径：
        // 基数 hp-hp=0；其余加成修饰保留）
        if let Some(def) = catalog.skill_by_name(crate::content::standard::BOSS_WUZANG) {
            if self.has_skill(p, def) {
                limit = limit.saturating_sub(self.player(p).hp.max(0) as usize);
            }
        }
        // 兽九（Boss 相柳，规则 v222）：手牌上限恒 9（上游 boss_jiushou
        // mod maxHandcard `num - hp + 9`：基数 hp-hp+9=9 同口径）
        if let Some(def) = catalog.skill_by_name(crate::content::standard::BOSS_JIUSHOU) {
            if self.has_skill(p, def) {
                limit = 9;
            }
        }
        // 灵渠（Boss 玄武，规则 v237）：手牌上限 +受击计数（上游
        // boss_lingqu mod maxHandcard `num + storage` 同口径）
        limit += self.player(p).lingqu_count as usize;
        // 神恩（Boss 太昊/炎帝 光环，规则 v238）：挑战者侧手牌上限 +1
        //（上游 boss_shenen2 mod maxHandcard `!side → num+1` 同口径；
        // 光环激活=场上存在神恩持有者）
        {
            let shenen = catalog.skill_by_name(crate::content::standard::BOSS_SHENEN);
            let active = shenen
                .map(|d| self.skills.keys().any(|(_, dd)| *dd == d))
                .unwrap_or(false);
            if active && !self.is_boss_side(p) {
                limit += 1;
            }
        }
        limit
    }

    /// 持有判定（**含未启用**实例）——隐贼激活门控用（装配即禁用，
    /// 体力条件达标时翻启用）。
    pub fn has_skill_any_state(&self, p: PlayerId, def: SkillDefId) -> bool {
        self.skills.contains_key(&(p, def))
    }

    pub fn has_skill(&self, p: PlayerId, def: SkillDefId) -> bool {
        self.skills
            .get(&(p, def))
            .map(|i| i.enabled)
            .unwrap_or(false)
    }

    /// 当前玩家的下一位存活玩家（座位环顺时针）。
    pub fn next_alive(&self, from: PlayerId) -> PlayerId {
        let n = self.players.len() as PlayerId;
        let mut i = self.ring_next(from);
        let mut guard = 0;
        while guard < n {
            if self.player(i).alive {
                return i;
            }
            i = self.ring_next(i);
            guard += 1;
        }
        i
    }

    /// 从 `start` 起按座位顺序（含 start，若存活）的存活玩家队列。
    /// 起点可为已死亡座位（问询链构造时 turn.current 可能已死）：
    /// 固定扫一圈座位数，避免「cur 只会落在存活座位、永不再等于
    /// 死亡起点」的死循环。
    pub fn seat_order_from(&self, start: PlayerId) -> Vec<PlayerId> {
        let n = self.players.len() as PlayerId;
        let mut out = Vec::new();
        let mut cur = self.ring_pos(start) as PlayerId;
        // 从 ring 位置起遍历（起点按环位置而非恒等位）
        let pos_start = self.ring_pos(start);
        let _ = cur;
        cur = *self.seat_ring.get(pos_start).unwrap_or(&start);
        for _ in 0..n {
            if self.player(cur).alive {
                out.push(cur);
            }
            cur = self.ring_next(cur);
        }
        out
    }
}

/// 完整对局状态 = 核心字段 + 执行位置。
#[derive(Debug, Clone)]
pub struct GameState {
    pub core: GameStateCore,
    pub resolution: crate::resolution::Resolution,
}
