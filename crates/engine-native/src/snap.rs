//! 快照与恢复：状态规范形态 + 信任边界校验 + 开放选择承接派生。
//!
//! 三分职责（N04/N05，ADR 0774）：
//! - **规范快照类型**：`StateSnapshot` = 核心状态 + 执行位置 + 段数据；
//!   挂起机器以 `MachineSnapshot` 全字段导出（字节存档与规范摘要共用）；
//! - **恢复期校验**：`validate_core` / `validate_card_ownership` /
//!   `validate_resolution` 在任何恢复路径上拒绝越界 ID、破损帧链、
//!   牌归属不闭合与承接关系不一致的快照（恶意快照不得产出引擎）；
//! - **承接派生**：`active_ask` 从挂起机器的 stage/asked 状态派生
//!   「当前应有的开放选择询问」，与快照内的 `open_choice` 逐字段核对
//!   （N04-e）——机器字段篡改即失去承接。

use crate::catalog::{CardKind, Catalog, SkillMachine};
use crate::choice::{ChoiceKind, ChoiceOption, OpenChoice, Selection};
use crate::content::host::{HostBehavior, HostMachine, HostStage};
use crate::content::standard::{general_gender, Gender, GUI_CAI, JI_JIU, LONG_DAN, QING_GUO};
use crate::content::host::GJ_FOOD_MIZHLIANOU_KEY;
use crate::event::{DamageNature, EventData, Frame};
use crate::ids::{CardId, PlayerId, SkillDefId};
use crate::resolution::{Slot, NO_FRAME};
use crate::state::{GameMode, GameStateCore, Outcome, Winner};

// ---------------------------------------------------------------------------
// 阶段快照枚举（内容机器 stage 的规范形态；运行时枚举 ↔ 快照枚举一一映射）
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SanCaiStageSnap {
    Draw,
    Target,
    GiveCard,
    Finish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YiShangStageSnap {
    Target,
    Apply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JiJiStageSnap {
    Draw,
    Target,
    Retarget,
    GiveCard,
    GiveChooseCard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZhiHengStageSnap {
    Select,
    Draw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KuRouStageSnap {
    Lose,
    Draw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QingNangStageSnap {
    TargetAsk,
    TargetRead,
    CardRead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuXiStageSnap {
    Ask,
    Read,
}

/// 骁果阶段（弃牌选择/读应答）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XiaoGuoStageSnap {
    Ask,
    Read,
}

/// 耀武阶段（来源二选/读应答）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YaoWuStageSnap {
    Ask,
    Read,
}

/// 淑慎阶段（目标选择/读应答）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShuShenStageSnap {
    Ask,
    Read,
}

/// 恂恂阶段（还牌观看/调整摸牌）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XunXunStageSnap {
    Undo,
    Arrange,
}

/// 机捷阶段（目标选择/读应答）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JiJieStageSnap {
    Ask,
    Read,
}

/// 同疾阶段（目标弃牌/转移）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TongJiStageSnap {
    Ask,
    Read,
}

/// 丈八蛇矛阶段（选牌/选目标/读目标）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZhangbaStageSnap {
    Cards,
    Target,
    TargetRead,
}

/// 借刀阶段（出杀或交械选择/读应答）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JiedaoStageSnap {
    Ask,
    Read,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuanXingStageSnap {
    Arrange,
    Place,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendeStageSnap {
    SelectCardsAsk,
    TargetAsk,
    TargetRead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JijiangStageSnap {
    HelperAsk,
    TargetAsk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JieYinStageSnap {
    CardsAsk,
    TargetAsk,
    TargetRead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanJianStageSnap {
    TargetAsk,
    SuitAsk,
    PickAsk,
    Consume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiJianStageSnap {
    DiscardAsk,
    TargetsAsk,
    Launch,
}

/// 单次判定的进行态快照（与 kernel `JudgementProgress` 字段一一对应）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JudgementProgressSnap {
    pub trick: CardId,
    pub judge_card: CardId,
    /// 鬼道/鬼才改判询问队列（座位序）。
    pub rewriters: Vec<PlayerId>,
    /// 当前改判者（或天妒询问）是否已被询问。
    pub asked: bool,
    /// 天妒持有人队列。
    pub tian_du: Vec<PlayerId>,
    /// 判定牌是否已被天妒获得。
    pub gained: bool,
    /// 是否处于改判询问阶段（false = 天妒阶段）。
    pub in_rewrite: bool,
    /// 判定确定已记（效果生效前恰好一次）。
    pub finalized: bool,
    /// 判定牌是否仍由机器持有。
    pub judge_held: bool,
    /// 昊天塔（swd 卡牌面窗口机批）：插窗持有者（Some=等待强选）。
    pub haotian_holder: Option<PlayerId>,
    /// 昊天塔：未选中的候选牌（选中另一张后弃置）。
    pub haotian_other: Option<u32>,
}

// ---------------------------------------------------------------------------
// 机器规范快照：判别符 + 全字段（字节存档 / 摘要 / 恢复共用）
// ---------------------------------------------------------------------------

/// 挂起机器的规范快照。字段清单与 kernel / content 机器结构一一对应；
/// 编解码（archive.rs）与摘要（engine.rs canonical_hash）按同一字段集。
/// T8 增量 2：VM 机挂起态快照（continuation 帧 + 选项 + 排队效果）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RvmPaused {
    /// Serialized operand stack + locals。
    pub frames: Vec<u8>,
    pub pc: u32,
    pub fuel_remaining: u32,
    /// Continuation identity within a match.
    pub id: u64,
    /// Module logic hash（内容锁：续跑时与模块哈希核对）。
    pub module_hash: [u8; 32],
    /// 选项 tokens（emission 序）。
    pub options: Vec<u32>,
    /// 排队未落账效果（rvm_bridge 自定义编码；映射面仅三种）。
    pub pending: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachineSnapshot {
    Sha {
        card: CardId,
        source: PlayerId,
        target: PlayerId,
        nature: DamageNature,
        asked: bool,
        /// 八卦阵判定是否已做（每杀至多一次）。
        armor_judged: bool,
        /// 已使用的闪数（无双需两张）。
        dodges_used: u8,
        /// 铁骑判定结果：None=未判定；Some(true)=红色（不能出闪）。
        tieji_red: Option<bool>,
        /// 流离（大乔）状态。
        liuli_done: bool,
        /// 流离两段询问（逐技对照批二 v76）：1=弃牌选择挂起，2=转移目标挂起。
        liuli_stage: u8,
        /// 流离已选定的待弃牌（stage 2 期间暂存）。
        liuli_card: Option<u32>,
        /// 护驾逐人询问队列：None=未开始；Some(q)=剩余待问。
        hujia_queue: Option<Vec<PlayerId>>,
        /// 护驾当前被询问者。
        hujia_holder: Option<PlayerId>,
        /// 酒效果（派发时捕获）：伤害 +1。
        jiu_bonus: bool,
        /// 麒麟弓弃马询问挂起中。
        asking_qilin: bool,
        qilin_horse: Option<u32>,
        /// 寒冰剑防止伤害确认挂起中。
        asking_hanbing: bool,
        /// 风雪刃命中后翻面+摸牌询问挂起中（规则 v95）。
        asking_fengxueren: bool,
        /// 贯石斧弃二强伤询问挂起中 + 待弃牌池。
        asking_guanshi: bool,
        guanshi_cards: Vec<u32>,
        /// 青龙偃月刀再杀询问挂起中。
        asking_qinglong: bool,
        /// 方天画戟剩余目标队列。
        extra_targets: Vec<u32>,
        /// 雌雄双股剑：弃牌/令摸询问挂起中（本杀已问过）。
        asking_cixiong: bool,
        cixiong_done: bool,
        /// 寒冰剑目标自选弃牌剩余次数（逐技对照批一，规则 v75）。
        hanbing_pending: u8,
        /// 霰弹（U05 ow 死神，规则 v140）：弃牌询问挂起/已问/黑牌加成。
        asking_xiandan: bool,
        xiandan_done: bool,
        xiandan_bonus: bool,
        /// 掠影（U05 xianjian 慕长兰，规则 v154）：目标选人挂起/受害者。
        asking_lueying: bool,
        lueying_victim: Option<u32>,
        /// 掠影：本杀已询问且被放弃（v194，阻止重入再问）。
        lueying_declined: bool,
        asking_fuchou: bool,
        fuchou_card: Option<u32>,
        /// 萨登荆环（swd 卡牌面窗口机批）：被闪后判定确认/额外出闪询问。
        asking_sadeng: bool,
        asking_sadeng_shan: bool,
    },
    /// 丈八蛇矛（批③-b 第七把）：两张手牌当杀（牌池 + 阶段）。
    Zhangba {
        owner: PlayerId,
        cards: Vec<u32>,
        stage: ZhangbaStageSnap,
    },
    DyingSave {
        dying: PlayerId,
        source: Option<PlayerId>,
        responders: Vec<PlayerId>,
        asked: bool,
        round_used: bool,
        guihan_asked: bool,
    },
    Duel {
        card: CardId,
        source: PlayerId,
        target: PlayerId,
        on_target: bool,
        asked: bool,
        second_pending: bool,
    },
    NullifyWindow {
        card: CardId,
        source: PlayerId,
        nullified: bool,
        queue: Vec<PlayerId>,
        asked: bool,
        window_open: bool,
        owns_disposal: bool,
        effect: Option<Box<MachineSnapshot>>,
    },
    JudgePhase {
        owner: PlayerId,
        queue: Vec<CardId>,
        current: Option<Box<JudgementProgressSnap>>,
    },
    Noop,
    PlaceDelayed {
        card: CardId,
        target: PlayerId,
        placed: bool,
    },
    Aoe {
        card: CardId,
        source: PlayerId,
        need: CardKind,
        targets: Vec<PlayerId>,
        idx: usize,
        inner: Option<Box<MachineSnapshot>>,
        nature: crate::event::DamageNature,
    },
    AoeTarget {
        source: PlayerId,
        target: PlayerId,
        need: CardKind,
        asked: bool,
        card: CardId,
        nature: crate::event::DamageNature,
    },
    Chain {
        card: CardId,
        source: PlayerId,
        targets: Vec<PlayerId>,
        emitted: usize,
    },
    DiscardToLimit {
        player: PlayerId,
        need: usize,
        asked: bool,
    },
    Death {
        player: PlayerId,
        killer: Option<PlayerId>,
    },
    OptionalGate {
        owner: PlayerId,
        def: SkillDefId,
        event: EventData,
        asked: bool,
        inner: Option<Box<MachineSnapshot>>,
    },
    SanCai {
        owner: PlayerId,
        stage: SanCaiStageSnap,
        target: Option<PlayerId>,
    },
    YiShang {
        owner: PlayerId,
        stage: YiShangStageSnap,
    },
    SuiDun,
    SuQing {
        target: PlayerId,
    },
    ShouYi {
        owner: PlayerId,
    },
    FanKui {
        owner: PlayerId,
        source: Option<PlayerId>,
        asked: bool,
    },
    JianXiong {
        owner: PlayerId,
        card: CardId,
        asked: bool,
    },
    BiYue {
        owner: PlayerId,
    },
    JiZhi {
        owner: PlayerId,
    },
    ZhiHeng {
        owner: PlayerId,
        stage: ZhiHengStageSnap,
    },
    KuRou {
        owner: PlayerId,
        stage: KuRouStageSnap,
    },
    LuoShen {
        owner: PlayerId,
    },
    YingZi {
        owner: PlayerId,
    },
    Dismantle {
        card: CardId,
        source: PlayerId,
        target: PlayerId,
        asked: bool,
    },
    /// E03 猴子偷桃：使用前取消窗口（确认门逐持有者询问）。
    MonkeySteal {
        card: CardId,
        user: PlayerId,
        holders: Vec<PlayerId>,
        asked: bool,
    },
    XiaoJi {
        owner: PlayerId,
    },
    QingNang {
        owner: PlayerId,
        stage: QingNangStageSnap,
        healed: Option<PlayerId>,
    },
    JiJi {
        owner: PlayerId,
        stage: JiJiStageSnap,
        target: Option<PlayerId>,
        given: u8,
    },
    GangLie {
        owner: PlayerId,
        source: PlayerId,
        asked: bool,
    },
    /// 骁果（乐进）：弃基本牌对结束阶段玩家造成 1 伤。
    XiaoGuo {
        owner: PlayerId,
        target: PlayerId,
        stage: XiaoGuoStageSnap,
    },
    /// 狂斧（潘凤）：杀伤害后摸 2 或失去 1 体力（单发）。
    KuangFu {
        owner: PlayerId,
        target: PlayerId,
    },
    /// 耀武（华雄）：红杀伤害来源摸 1 或回 1。
    YaoWu {
        owner: PlayerId,
        source: PlayerId,
        stage: YaoWuStageSnap,
    },
    /// 淑慎（甘夫人）：回复后令其他角色摸 1/2。
    ShuShen {
        owner: PlayerId,
        target: Option<PlayerId>,
        stage: ShuShenStageSnap,
    },
    /// 神智（甘夫人）：弃全部手牌，数量≥体力则回 1（单发）。
    ShenZhi {
        owner: PlayerId,
    },
    /// 恂恂（李典）：观看顶四张、两置顶两置底（top=观看的牌）。
    XunXun {
        owner: PlayerId,
        stage: XunXunStageSnap,
        top: Vec<u32>,
    },
    /// 忘隙（李典）：伤害双方各摸 1（单发）。
    WangXi {
        owner: PlayerId,
        other: PlayerId,
    },
    /// 机捷（伊籍）：牌堆底牌交任一角色（card 挂起期持有）。
    JiJie {
        owner: PlayerId,
        card: Option<u32>,
        stage: JiJieStageSnap,
    },
    /// 急援（伊籍）：濒死者摸 1（单发）。
    JiYuan {
        owner: PlayerId,
    },
    /// 妄尊（袁术）：回合开始摸 1/2 + 主公印记（单发）。
    WangZun {
        owner: PlayerId,
        target: PlayerId,
    },
    /// 同疾（袁术）：目标弃牌则杀转移（改写帧载荷）。
    TongJi {
        owner: PlayerId,
        stage: TongJiStageSnap,
    },
    /// 无中生有（P2/N08 缺牌种批①）：使用后摸 2（单发）。
    Wuzhong {
        owner: PlayerId,
    },
    /// 桃园结义（P2/N08 缺牌种批①）：所有其他角色回 1（单发）。
    Taoyuan {
        owner: PlayerId,
    },
    /// 五谷丰登（批①-2）：亮牌池 + 剩余择牌队列。
    Wugu {
        owner: PlayerId,
        cards: Vec<u32>,
        pickers: Vec<PlayerId>,
    },
    /// 借刀杀人（批①-2）：武器持有者出杀/交械。
    Jiedao {
        owner: PlayerId,
        holder: PlayerId,
        victim: PlayerId,
        stage: JiedaoStageSnap,
    },
    TuXi {
        owner: PlayerId,
        stage: TuXiStageSnap,
    },
    LuoYi {
        owner: PlayerId,
    },
    GuanXing {
        owner: PlayerId,
        stage: GuanXingStageSnap,
        top: Vec<CardId>,
    },
    LianYing {
        owner: PlayerId,
    },
    Rende {
        owner: PlayerId,
        stage: RendeStageSnap,
        cards: Vec<CardId>,
    },
    Jijiang {
        owner: PlayerId,
        stage: JijiangStageSnap,
        queue: Option<Vec<PlayerId>>,
        helper: Option<PlayerId>,
        sha: Option<CardId>,
        inner: Option<Box<MachineSnapshot>>,
    },
    Snatch {
        card: CardId,
        source: PlayerId,
        target: PlayerId,
        asked: bool,
    },
    JieYin {
        owner: PlayerId,
        stage: JieYinStageSnap,
        cards: Vec<CardId>,
    },
    FanJian {
        owner: PlayerId,
        stage: FanJianStageSnap,
        target: Option<PlayerId>,
        /// 目标所选花色 tag（0=♥ 1=♦ 2=♣ 3=♠；v203）。
        chosen_suit: Option<u8>,
    },
    /// 相德（Boss 混沌，规则 v210）：无挂起一步机，快照仅身份。
    BossXiangde {
        owner: PlayerId,
    },
    /// 覆泰（Boss 穷奇，规则 v212）：无挂起一步机，快照仅身份。
    BossFutai {
        owner: PlayerId,
    },
    /// 乱常（Boss 梼杌，规则 v218）：无挂起一步机，快照仅身份。
    BossLuanchang {
        owner: PlayerId,
    },
    /// 招祸（Boss 毕方，规则 v227）：无挂起一步机，快照仅身份。
    BossZhaohuo {
        owner: PlayerId,
    },
    /// 奇佐（Boss 郭嘉，规则 v259）：无挂起一步机，快照仅身份。
    BossQizuo {
        owner: PlayerId,
    },
    /// 枭雄（Boss 刘备，规则 v261）：无挂起一步机，快照仅身份。
    BossXiaoxiong {
        owner: PlayerId,
    },
    /// 凌波（Boss 甄姬，规则 v262）：无挂起一步机，快照仅身份。
    BossLingbo {
        owner: PlayerId,
    },
    /// 踏水（Boss 甄姬，规则 v262）：无挂起一步机，快照仅身份。
    BossTashui {
        owner: PlayerId,
    },
    /// 雷击·张角（Boss 张角，规则 v263）：无挂起一步机，快照仅身份。
    BossZjLeiji {
        owner: PlayerId,
    },
    /// 黑绳（宋帝王，规则 v266）：无挂起一步机，快照仅身份。
    BossHeisheng {
        owner: PlayerId,
    },
    /// 雷诛（卞城王，规则 v267）：逐发雷伤遗言机，emitted=已发目标数
    ///（窗口内挂起恢复用）。
    BossLeizhu {
        owner: PlayerId,
        emitted: u8,
    },
    /// 热恼（泰山王，规则 v268）：单发 3 火伤遗言机，emitted∈{0,1}。
    BossRenao {
        owner: PlayerId,
        emitted: u8,
    },
    /// 忘魂（孟婆，规则 v270）：直写一步遗言机，快照仅身份。
    BossWanghun {
        owner: PlayerId,
    },
    /// 诛心（阎罗王，规则 v271）：直写一步遗言机，emitted 防重入
    ///（热恼同构）。
    BossZhuxin {
        owner: PlayerId,
        emitted: u8,
    },
    /// 平等（平等王，规则 v272）：直写两波遗言机，emitted 波次进度
    ///（诛心同构）。
    BossPingdeng {
        owner: PlayerId,
        emitted: u8,
    },
    /// 火刑（都市王，规则 v273）：直写逐座遗言机，emitted=下一座位
    /// 索引。
    BossHuoxing {
        owner: PlayerId,
        emitted: u8,
    },
    /// 冰封（楚江王，规则 v275）：直写一步遗言机，快照仅身份。
    BossBingfeng {
        owner: PlayerId,
    },
    /// 望乡（秦广王，规则 v276）：直写一步遗言机（全场装备全弃）。
    BossWangxiang {
        owner: PlayerId,
    },
    /// T8 增量 2：rule-vm 技能机（数据驱动扩展包技能）。快照自包含：
    /// 模块载体字节（from_bytes 还原并验证）+ 挂起态（continuation/
    /// 选项/排队效果——排队效果保留至续跑后落账）。
    Rvm {
        owner: PlayerId,
        module: Vec<u8>,
        paused: Option<RvmPaused>,
    },
    /// 星夏（Boss 祝融，规则 v242）：主动技机。stage：0=损焰灵（2 火
    /// 已结算）、1=询问敌方挂起、2=敌方已选、3=弃牌已答待结算（弃红询
    /// 问由阶段 2 直接挂出）；actor=当前焦点（焰灵/敌方）。
    BossXingxia {
        owner: PlayerId,
        stage: u8,
        actor: u32,
    },
    /// 不臣（Boss 勾芒，规则 v240）：主动技两线机。stage：0=失体
    /// （复活线已结算失体）、1=逐个复活中、2=目标询问挂起（恢复线）、
    /// 3=目标已答待结算；targets=复活队列（复活线）/空（恢复线）。
    BossBuchun {
        owner: PlayerId,
        stage: u8,
        targets: Vec<u32>,
    },
    /// 蛇毒标记结算（Boss 相柳，规则 v223）：两阶段挂起机。
    Shedu {
        owner: PlayerId,
    },
    LiJian {
        owner: PlayerId,
        stage: LiJianStageSnap,
        card: Option<CardId>,
        inner: Option<Box<MachineSnapshot>>,
    },
    Host {
        def_key: String,
        behavior: u16,
        stage: u8,
        owner: PlayerId,
        target: Option<PlayerId>,
        card: Option<CardId>,
        cards: Vec<CardId>,
        targets: Vec<PlayerId>,
    },
}

// ---------------------------------------------------------------------------
// 执行位置 / 顶层快照
// ---------------------------------------------------------------------------

/// 挂起的机器与归属槽位的规范快照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParkSnapshot {
    /// 栈深回到该值时恢复。
    pub resume_depth: usize,
    pub slot: Slot,
    pub machine: MachineSnapshot,
}

/// 执行位置的规范快照（与 `resolution::Resolution` 字段一一对应）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionSnapshot {
    pub next_event_id: u32,
    pub next_choice_id: u64,
    pub stack: Vec<Frame>,
    pub parks: Vec<ParkSnapshot>,
    pub open_choice: Option<OpenChoice>,
    /// 已校验、待机器读取的应答。
    pub answer: Option<Selection>,
    pub budget: u32,
}

/// 完整对局快照（规则版本绑定）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateSnapshot {
    pub ruleset_version: u64,
    pub core: GameStateCore,
    /// 命令去重高水位（会话数据，随快照延续）。
    pub last_dedup: u64,
    /// N11-a 令牌代次（恢复时 +1）。
    pub auth_generation: u64,
    /// 段暂停中标志（必须携带检查点）。
    pub segment_open: bool,
    /// 段检查点（至多一层嵌套）。
    pub segment_checkpoint: Option<Box<StateSnapshot>>,
    pub resolution: ResolutionSnapshot,
}

// ---------------------------------------------------------------------------
// 快照错误
// ---------------------------------------------------------------------------

/// 恢复期校验错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    /// 规则版本不符（快照与目录/核心状态不互认）。
    RuleVersionMismatch {
        snapshot: u64,
        catalog: u64,
    },
    /// 越界/非法 ID 或结构关系；`what` 指明检查项。
    IdOutOfBounds {
        what: &'static str,
        got: u64,
    },
    /// 机器快照无法在当前目录下重建（未知机器类型/扩展缺包/坏标签）。
    UnknownMachine,
    /// 开放选择与挂起机器的承接关系不一致（N04-e）。
    OpenChoiceMismatch {
        observer: PlayerId,
    },
    /// G1 装备技能生命周期：装备在场而技能实例缺失。
    EquipSkillMismatch {
        player: PlayerId,
    },
    /// 机器快照载荷的坏标签。
    BadTag(&'static str, u64),
}

// ---------------------------------------------------------------------------
// 恢复边界与校验
// ---------------------------------------------------------------------------

/// 恢复边界：目录尺寸 + 座位数 + 规则版本的恢复期视图。
#[derive(Debug, Clone, Copy)]
pub struct RestoreBounds {
    pub seats: usize,
    pub cards: usize,
    pub skills: usize,
    pub generals: usize,
    pub ruleset_version: u64,
}

/// 由目录与座位数导出恢复边界。
pub fn bounds_from(catalog: &Catalog, seats: usize) -> RestoreBounds {
    RestoreBounds {
        seats,
        cards: catalog.cards.len(),
        skills: catalog.skills.len(),
        generals: catalog.general_count(),
        ruleset_version: catalog.ruleset_version,
    }
}

fn oob(what: &'static str, got: u64) -> SnapshotError {
    SnapshotError::IdOutOfBounds { what, got }
}

fn check_pid(p: PlayerId, b: &RestoreBounds) -> Result<(), SnapshotError> {
    if p as usize >= b.seats {
        return Err(oob("player", p as u64));
    }
    Ok(())
}

fn check_card(c: CardId, b: &RestoreBounds) -> Result<(), SnapshotError> {
    if c as usize >= b.cards {
        return Err(oob("card", c as u64));
    }
    Ok(())
}

fn check_def(d: SkillDefId, b: &RestoreBounds) -> Result<(), SnapshotError> {
    if d as usize >= b.skills {
        return Err(oob("skill_def", d as u64));
    }
    Ok(())
}

fn check_opt_pid(p: Option<PlayerId>, b: &RestoreBounds) -> Result<(), SnapshotError> {
    if let Some(p) = p {
        check_pid(p, b)?;
    }
    Ok(())
}

fn check_opt_card(c: Option<CardId>, b: &RestoreBounds) -> Result<(), SnapshotError> {
    if let Some(c) = c {
        check_card(c, b)?;
    }
    Ok(())
}

fn check_pids(ps: &[PlayerId], b: &RestoreBounds) -> Result<(), SnapshotError> {
    for p in ps {
        check_pid(*p, b)?;
    }
    Ok(())
}

fn check_cards(cs: &[CardId], b: &RestoreBounds) -> Result<(), SnapshotError> {
    for c in cs {
        check_card(*c, b)?;
    }
    Ok(())
}

fn check_event_data(data: &EventData, b: &RestoreBounds) -> Result<(), SnapshotError> {
    match data {
        EventData::Damage(d) => {
            check_opt_pid(d.source, b)?;
            check_pid(d.target, b)?;
            check_pids(&d.redirect_chain, b)?;
            check_opt_card(d.card, b)?;
        }
        EventData::LoseHp { player, .. } => {
            check_pid(*player, b)?;
        }
        EventData::Recover { player, by_card, .. } => {
            check_pid(*player, b)?;
            check_opt_card(*by_card, b)?;
        }
        EventData::CardsDrawn { player, cards } => {
            check_pid(*player, b)?;
            check_cards(cards, b)?;
        }
        EventData::CardsMoved { cards, from, to } => {
            check_cards(cards, b)?;
            check_pid(*from, b)?;
            check_pid(*to, b)?;
        }
        EventData::CardUsed(u) => {
            check_card(u.card, b)?;
            check_pid(u.player, b)?;
            check_pids(&u.targets, b)?;
        }
        EventData::CardResponded { card, player } => {
            check_card(*card, b)?;
            check_pid(*player, b)?;
        }
        EventData::Dying { player, source } => {
            check_pid(*player, b)?;
            check_opt_pid(*source, b)?;
        }
        EventData::DyingSaved { player, source } => {
            check_pid(*player, b)?;
            check_opt_pid(*source, b)?;
        }
        EventData::Death { player, killer } => {
            check_pid(*player, b)?;
            check_opt_pid(*killer, b)?;
        }
        EventData::Phase { player, .. } => check_pid(*player, b)?,
        EventData::TurnEnded { player } => check_pid(*player, b)?,
        EventData::SkillToggled { player, skill, .. } => {
            check_pid(*player, b)?;
            check_def(*skill, b)?;
        }
        EventData::Equipped { player, card, replaced } => {
            check_pid(*player, b)?;
            check_card(*card, b)?;
            check_opt_card(*replaced, b)?;
        }
        EventData::Linked { player, .. } => check_pid(*player, b)?,
        EventData::CardsDiscarded { player, cards } => {
            check_pid(*player, b)?;
            check_cards(cards, b)?;
        }
        EventData::TurnOverBefore { player, .. } => {
            check_pid(*player, b)?;
        }
        EventData::TurnOvered { player, .. } => {
            check_pid(*player, b)?;
        }
    }
    Ok(())
}

fn check_options(opts: &[ChoiceOption], b: &RestoreBounds) -> Result<(), SnapshotError> {
    for o in opts {
        match o {
            ChoiceOption::Card(c) => check_card(*c, b)?,
            ChoiceOption::Player(p) => check_pid(*p, b)?,
            _ => {}
        }
    }
    Ok(())
}

/// 机器快照树内全部 ID 的边界校验（含嵌套机器与判定进行态关系）。
fn check_machine(m: &MachineSnapshot, b: &RestoreBounds) -> Result<(), SnapshotError> {
    match m {
        MachineSnapshot::Sha {
            card,
            source,
            target,
            hujia_queue,
            hujia_holder,
            qilin_horse,
            asking_guanshi,
            guanshi_cards,
            extra_targets,
            ..
        } => {
            check_card(*card, b)?;
            check_pid(*source, b)?;
            check_pid(*target, b)?;
            if let Some(q) = hujia_queue {
                check_pids(q, b)?;
            }
            check_opt_pid(*hujia_holder, b)?;
            if let Some(c) = qilin_horse {
                check_card(*c, b)?;
            }
            if *asking_guanshi {
                for c in guanshi_cards.iter() {
                    check_card(*c, b)?;
                }
            }
            for c in extra_targets.iter() {
                check_card(*c, b)?;
            }
        }
        MachineSnapshot::DyingSave {
            dying,
            source,
            responders,
            ..
        } => {
            check_pid(*dying, b)?;
            check_opt_pid(*source, b)?;
            check_pids(responders, b)?;
        }
        MachineSnapshot::Duel { card, source, target, .. } => {
            check_card(*card, b)?;
            check_pid(*source, b)?;
            check_pid(*target, b)?;
        }
        MachineSnapshot::NullifyWindow {
            card,
            source,
            queue,
            effect,
            ..
        } => {
            check_card(*card, b)?;
            check_pid(*source, b)?;
            check_pids(queue, b)?;
            if let Some(e) = effect {
                check_machine(e, b)?;
            }
        }
        MachineSnapshot::JudgePhase { owner, queue, current } => {
            check_pid(*owner, b)?;
            check_cards(queue, b)?;
            if let Some(c) = current {
                check_card(c.trick, b)?;
                check_card(c.judge_card, b)?;
                check_pids(&c.rewriters, b)?;
                check_pids(&c.tian_du, b)?;
                // N04-d：改判阶段 asked 与非空队列同置——asked=true 而队列
                // 已空会在恢复后应答时 panic，属非法快照。
                if c.in_rewrite && c.asked && c.rewriters.is_empty() {
                    return Err(oob("rewrite_asked_without_rewriter", 0));
                }
            }
        }
        MachineSnapshot::Noop => {}
        MachineSnapshot::PlaceDelayed { card, target, .. } => {
            check_card(*card, b)?;
            check_pid(*target, b)?;
        }
        MachineSnapshot::Aoe {
            card,
            source,
            targets,
            inner,
            ..
        } => {
            check_card(*card, b)?;
            check_pid(*source, b)?;
            check_pids(targets, b)?;
            if let Some(e) = inner {
                check_machine(e, b)?;
            }
        }
        MachineSnapshot::AoeTarget {
            source, target, card, ..
        } => {
            check_pid(*source, b)?;
            check_pid(*target, b)?;
            check_card(*card, b)?;
        }
        MachineSnapshot::Chain {
            card,
            source,
            targets,
            ..
        } => {
            check_card(*card, b)?;
            check_pid(*source, b)?;
            check_pids(targets, b)?;
        }
        MachineSnapshot::DiscardToLimit { player, .. } => check_pid(*player, b)?,
        MachineSnapshot::Death { player, killer } => {
            check_pid(*player, b)?;
            check_opt_pid(*killer, b)?;
        }
        MachineSnapshot::OptionalGate {
            owner, def, event, inner, ..
        } => {
            check_pid(*owner, b)?;
            check_def(*def, b)?;
            check_event_data(event, b)?;
            if let Some(e) = inner {
                check_machine(e, b)?;
            }
        }
        MachineSnapshot::SanCai { owner, target, .. } => {
            check_pid(*owner, b)?;
            check_opt_pid(*target, b)?;
        }
        MachineSnapshot::YiShang { owner, .. } => check_pid(*owner, b)?,
        MachineSnapshot::SuiDun => {}
        MachineSnapshot::SuQing { target } => check_pid(*target, b)?,
        MachineSnapshot::ShouYi { owner } => check_pid(*owner, b)?,
        MachineSnapshot::FanKui { owner, source, .. } => {
            check_pid(*owner, b)?;
            check_opt_pid(*source, b)?;
        }
        MachineSnapshot::JianXiong { owner, card, .. } => {
            check_pid(*owner, b)?;
            check_card(*card, b)?;
        }
        MachineSnapshot::BiYue { owner } => check_pid(*owner, b)?,
        MachineSnapshot::JiZhi { owner } => check_pid(*owner, b)?,
        MachineSnapshot::ZhiHeng { owner, .. } => check_pid(*owner, b)?,
        MachineSnapshot::KuRou { owner, .. } => check_pid(*owner, b)?,
        MachineSnapshot::LuoShen { owner } => check_pid(*owner, b)?,
        MachineSnapshot::YingZi { owner } => check_pid(*owner, b)?,
        MachineSnapshot::Dismantle {
            card,
            source,
            target,
            ..
        } => {
            check_card(*card, b)?;
            check_pid(*source, b)?;
            check_pid(*target, b)?;
        }
        MachineSnapshot::MonkeySteal { card, user, .. } => {
            check_card(*card, b)?;
            check_pid(*user, b)?;
        }
        MachineSnapshot::XiaoJi { owner } => check_pid(*owner, b)?,
        MachineSnapshot::QingNang { owner, healed, .. } => {
            check_pid(*owner, b)?;
            check_opt_pid(*healed, b)?;
        }
        MachineSnapshot::JiJi { owner, target, .. } => {
            check_pid(*owner, b)?;
            check_opt_pid(*target, b)?;
        }
        MachineSnapshot::GangLie { owner, source, .. } => {
            check_pid(*owner, b)?;
            check_pid(*source, b)?;
        }
        MachineSnapshot::XiaoGuo { owner, target, .. } => {
            check_pid(*owner, b)?;
            check_pid(*target, b)?;
        }
        MachineSnapshot::KuangFu { owner, target } => {
            check_pid(*owner, b)?;
            check_pid(*target, b)?;
        }
        MachineSnapshot::YaoWu { owner, source, .. } => {
            check_pid(*owner, b)?;
            check_pid(*source, b)?;
        }
        MachineSnapshot::ShuShen { owner, target, .. } => {
            check_pid(*owner, b)?;
            check_opt_pid(*target, b)?;
        }
        MachineSnapshot::ShenZhi { owner } => check_pid(*owner, b)?,
        MachineSnapshot::XunXun { owner, top, .. } => {
            check_pid(*owner, b)?;
            for c in top {
                check_card(*c, b)?;
            }
        }
        MachineSnapshot::WangXi { owner, other } => {
            check_pid(*owner, b)?;
            check_pid(*other, b)?;
        }
        MachineSnapshot::JiJie { owner, card, .. } => {
            check_pid(*owner, b)?;
            if let Some(c) = card {
                check_card(*c, b)?;
            }
        }
        MachineSnapshot::JiYuan { owner } => check_pid(*owner, b)?,
        MachineSnapshot::Zhangba { owner, cards, .. } => {
            check_pid(*owner, b)?;
            for c in cards {
                check_card(*c, b)?;
            }
        }
        MachineSnapshot::WangZun { owner, target } => {
            check_pid(*owner, b)?;
            check_pid(*target, b)?;
        }
        MachineSnapshot::TongJi { owner, .. } => check_pid(*owner, b)?,
        MachineSnapshot::Wuzhong { owner } => check_pid(*owner, b)?,
        MachineSnapshot::Taoyuan { owner } => check_pid(*owner, b)?,
        MachineSnapshot::Wugu { owner, cards, pickers } => {
            check_pid(*owner, b)?;
            for c in cards {
                check_card(*c, b)?;
            }
            check_pids(pickers, b)?;
        }
        MachineSnapshot::Jiedao { owner, holder, victim, .. } => {
            check_pid(*owner, b)?;
            check_pid(*holder, b)?;
            check_pid(*victim, b)?;
        }
        MachineSnapshot::TuXi { owner, .. } => check_pid(*owner, b)?,
        MachineSnapshot::LuoYi { owner } => check_pid(*owner, b)?,
        MachineSnapshot::GuanXing { owner, top, .. } => {
            check_pid(*owner, b)?;
            check_cards(top, b)?;
        }
        MachineSnapshot::LianYing { owner } => check_pid(*owner, b)?,
        MachineSnapshot::Rende { owner, cards, .. } => {
            check_pid(*owner, b)?;
            check_cards(cards, b)?;
        }
        MachineSnapshot::Jijiang {
            owner,
            queue,
            helper,
            sha,
            inner,
            ..
        } => {
            check_pid(*owner, b)?;
            if let Some(q) = queue {
                check_pids(q, b)?;
            }
            check_opt_pid(*helper, b)?;
            check_opt_card(*sha, b)?;
            if let Some(e) = inner {
                check_machine(e, b)?;
            }
        }
        MachineSnapshot::Snatch {
            card,
            source,
            target,
            ..
        } => {
            check_card(*card, b)?;
            check_pid(*source, b)?;
            check_pid(*target, b)?;
        }
        MachineSnapshot::JieYin { owner, cards, .. } => {
            check_pid(*owner, b)?;
            check_cards(cards, b)?;
        }
        MachineSnapshot::FanJian { owner, target, .. } => {
            check_pid(*owner, b)?;
            check_opt_pid(*target, b)?;
            // chosen_suit 为定界 tag（0..=3），非实体引用，无需界校验
        }
        MachineSnapshot::BossXiangde { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossFutai { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossLuanchang { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossZhaohuo { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossQizuo { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossXiaoxiong { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossLingbo { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossTashui { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossZjLeiji { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossHeisheng { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossLeizhu { owner, emitted, .. } => {
            check_pid(*owner, b)?;
            let _ = emitted;
        }
        MachineSnapshot::BossRenao { owner, emitted, .. } => {
            check_pid(*owner, b)?;
            let _ = emitted;
        }
        MachineSnapshot::BossWanghun { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossZhuxin { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossPingdeng { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossHuoxing { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossBingfeng { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossWangxiang { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::Rvm {
            owner,
            module,
            paused,
        } => {
            check_pid(*owner, b)?;
            if rule_vm::BytecodeModule::from_bytes(module).is_err() {
                return Err(SnapshotError::UnknownMachine);
            }
            if let Some(p) = paused {
                if p.module_hash
                    != rule_vm::BytecodeModule::from_bytes(module)
                        .expect("checked")
                        .logic_hash
                {
                    return Err(SnapshotError::UnknownMachine);
                }
                let _ = &p.frames;
            }
        }
        MachineSnapshot::Shedu { owner, .. } => {
            check_pid(*owner, b)?;
        }
        MachineSnapshot::BossBuchun { owner, targets, .. } => {
            check_pid(*owner, b)?;
            for t in targets {
                check_pid(*t as u8, b)?;
            }
        }
        MachineSnapshot::BossXingxia { owner, actor, .. } => {
            check_pid(*owner, b)?;
            check_pid(*actor as u8, b)?;
        }
        MachineSnapshot::LiJian {
            owner, card, inner, ..
        } => {
            check_pid(*owner, b)?;
            check_opt_card(*card, b)?;
            if let Some(e) = inner {
                check_machine(e, b)?;
            }
        }
        MachineSnapshot::Host {
            owner,
            target,
            card,
            cards,
            targets,
            ..
        } => {
            check_pid(*owner, b)?;
            check_opt_pid(*target, b)?;
            check_opt_card(*card, b)?;
            check_cards(cards, b)?;
            check_pids(targets, b)?;
        }
    }
    Ok(())
}

/// 核心状态校验：规则版本绑定、牌区/技能实例 ID 界、身份局结构、
/// 武将槽界、胜负与模式一致性。
pub fn validate_core(core: &GameStateCore, b: &RestoreBounds) -> Result<(), SnapshotError> {
    // 规则版本（核心副本与目录绑定；顶层一致不豁免核心篡改）
    if core.ruleset_version != b.ruleset_version {
        return Err(SnapshotError::RuleVersionMismatch {
            snapshot: core.ruleset_version,
            catalog: b.ruleset_version,
        });
    }
    // 牌区 ID 界、国战武将槽界、五行属性界
    for p in &core.players {
        check_cards(&p.hand, b)?;
        check_cards(&p.equips, b)?;
        check_cards(&p.judge, b)?;
        check_cards(&p.off_court, b)?;
        for g in p.general_slots.iter().flatten() {
            if *g as usize >= b.generals {
                return Err(oob("general_slot_index", *g as u64));
            }
        }
        // 明置必须引用已配置的武将槽
        for slot in 0..2 {
            if p.revealed_generals[slot] && p.general_slots[slot].is_none() {
                return Err(oob("reveal_without_general", 0));
            }
        }
        if let Some(n) = p.wunature {
            if n > 4 {
                return Err(oob("wunature", n as u64));
            }
        }
    }
    // 回合当前玩家在界
    if core.turn.current as usize >= core.players.len() {
        return Err(oob("player", core.turn.current as u64));
    }
    // 技能实例：键在界、值 owner 与键一致
    for ((owner, def), inst) in &core.skills {
        check_pid(*owner, b)?;
        check_def(*def, b)?;
        if inst.owner != *owner {
            return Err(oob("skill_instance_owner", inst.owner as u64));
        }
    }
    // 对决模式结构：两队互斥、并集覆盖全部座位
    if let GameMode::Versus { teams, substitutes } = &core.mode {
        if teams.len() != 2 {
            return Err(oob("versus_team_count", teams.len() as u64));
        }
        let mut seen = vec![false; b.seats];
        for t in teams {
            for p in t {
                if (*p as usize) >= b.seats || seen[*p as usize] {
                    return Err(oob("versus_team_member", *p as u64));
                }
                seen[*p as usize] = true;
            }
        }
        if seen.iter().any(|s| !s) {
            return Err(oob("versus_team_coverage", 0));
        }
        // 替补池（v206）：武将 def 界校验（池数与队数一致，条目为目录将）
        if substitutes.len() != teams.len() {
            return Err(oob("versus_substitute_pools", substitutes.len() as u64));
        }
        for pool in substitutes {
            for g in pool {
                check_def(*g, b)?;
            }
        }
    }
    // Boss 结构：boss/助战座位有效
    if let GameMode::Boss { boss, fellows, .. } = &core.mode {
        if (*boss as usize) >= b.seats {
            return Err(oob("boss_seat", *boss as u64));
        }
        for f in fellows {
            if (*f as usize) >= b.seats {
                return Err(oob("boss_fellow_seat", *f as u64));
            }
        }
    }
    // KOF 结构：2 人局（会话将池内容为 GENERAL 下标，越界由运行期授予校验）
    if let GameMode::Kof { .. } = &core.mode {
        if b.seats != 2 {
            return Err(oob("kof_seats", b.seats as u64));
        }
    }
    // 斗地主结构：地主座位有效
    if let GameMode::Doudizhu { lord } = &core.mode {
        if (*lord as usize) >= b.seats {
            return Err(oob("doudizhu_lord", *lord as u64));
        }
    }
    // 身份局结构
    if let GameMode::Identity { roles } = &core.mode {
        if roles.len() != b.seats {
            return Err(oob("roles_len", roles.len() as u64));
        }
        if roles.first() != Some(&crate::state::Role::Lord) {
            return Err(oob("lord_seat", 0));
        }
        if !roles.contains(&crate::state::Role::Traitor) {
            return Err(oob("roles_composition", 0));
        }
        // 人数表：4–6 人局 1 忠，7–8 人局 2 忠
        let loyalists = if (4..=6).contains(&b.seats) { 1 } else { 2 };
        let got = roles
            .iter()
            .filter(|r| **r == crate::state::Role::Loyalist)
            .count();
        if got != loyalists {
            return Err(oob("roles_loyalist_count", got as u64));
        }
    }
    // 胜负与模式一致性
    if let Some(Outcome { winner }) = &core.outcome {
        let ok = match (&core.mode, winner) {
            (GameMode::Prototype, Winner::Player(_)) => true,
            (GameMode::Identity { .. }, Winner::LordSide | Winner::Rebels | Winner::Traitor) => {
                true
            }
            (GameMode::Kingdoms, Winner::Faction(_)) => true,
            (GameMode::Versus { .. }, Winner::Versus(_)) => true,
            (GameMode::Doudizhu { .. }, Winner::Doudizhu { .. }) => true,
            (GameMode::Boss { .. }, Winner::Boss { .. }) => true,
            (GameMode::Kof { .. }, Winner::Player(_)) => true,
            _ => false,
        };
        if !ok {
            return Err(oob("outcome_mode_mismatch", 0));
        }
    }
    Ok(())
}

/// 机器持牌收集（N04-f：判定进行态/观星顶牌由机器持有，不在任何牌区）。
fn machine_held_cards(m: &MachineSnapshot, out: &mut Vec<CardId>) {
    match m {
        MachineSnapshot::JudgePhase { queue, current, .. } => {
            out.extend(queue.iter().copied());
            if let Some(c) = current {
                // finalized（效果已套用）后锦囊已离机，trick 仅为簿记
                if !c.finalized {
                    out.push(c.trick);
                }
                if c.judge_held {
                    out.push(c.judge_card);
                }
            }
        }
        MachineSnapshot::GuanXing { top, .. } => out.extend(top.iter().copied()),
        MachineSnapshot::OptionalGate { inner, .. }
        | MachineSnapshot::NullifyWindow { effect: inner, .. }
        | MachineSnapshot::Aoe { inner, .. }
        | MachineSnapshot::Jijiang { inner, .. }
        | MachineSnapshot::LiJian { inner, .. } => {
            if let Some(m) = inner {
                machine_held_cards(m, out);
            }
        }
        _ => {}
    }
}

/// N04-f 牌归属多重集核算：目录全集 ⊆ 恰一牌区 ∪ 机器暂离。
/// 尾端缺失 / 中段缺失 / 全量缺失 / 重复归属一律拒绝。
pub fn validate_card_ownership(
    core: &GameStateCore,
    resolution: &ResolutionSnapshot,
    b: &RestoreBounds,
) -> Result<(), SnapshotError> {
    let mut seen = vec![false; b.cards];
    fn mark(seen: &mut [bool], b: &RestoreBounds, c: CardId) -> Result<(), SnapshotError> {
        check_card(c, b)?;
        if seen[c as usize] {
            return Err(oob("card_in_multiple_zones", c as u64));
        }
        seen[c as usize] = true;
        Ok(())
    }
    for p in &core.players {
        for c in &p.hand {
            mark(&mut seen, b, *c)?;
        }
        for c in &p.equips {
            mark(&mut seen, b, *c)?;
        }
        for c in &p.judge {
            mark(&mut seen, b, *c)?;
        }
        // 离场区（机关怨挂附等；T19a 修复：此前未计入所有权校验）
        for c in &p.off_court {
            mark(&mut seen, b, *c)?;
        }
    }
    for c in &core.deck {
        mark(&mut seen, b, *c)?;
    }
    for c in &core.discard {
        mark(&mut seen, b, *c)?;
    }
    for c in &core.table {
        mark(&mut seen, b, *c)?;
    }
    let mut held: Vec<CardId> = Vec::new();
    for park in &resolution.parks {
        machine_held_cards(&park.machine, &mut held);
    }
    for c in held {
        mark(&mut seen, b, c)?;
    }
    for (id, ok) in seen.iter().enumerate() {
        if !ok {
            return Err(oob("card_missing_from_all_zones", id as u64));
        }
    }
    Ok(())
}

/// 执行位置校验：帧父子链 / 帧号水位与去重 / 挂起深度与槽位帧引用 /
/// 选择应答 ID 水位 / 机器树 ID 界。
pub fn validate_resolution(
    resolution: &ResolutionSnapshot,
    b: &RestoreBounds,
) -> Result<(), SnapshotError> {
    // 栈帧：唯一性、水位、父子链（父必须存在且是更早的帧）
    let mut ids: std::collections::BTreeMap<u32, usize> = Default::default();
    for (i, f) in resolution.stack.iter().enumerate() {
        if f.id as u64 >= resolution.next_event_id as u64 {
            return Err(oob("event_id", f.id as u64));
        }
        if ids.insert(f.id, i).is_some() {
            return Err(oob("event_id_dup", f.id as u64));
        }
        if let Some(p) = f.parent {
            match ids.get(&p) {
                Some(&pi) if pi < i => {}
                _ => return Err(oob("frame_parent", p as u64)),
            }
        }
        check_event_data(&f.data, b)?;
        for q in &f.queue {
            check_def(q.def, b)?;
            check_pid(q.owner, b)?;
        }
    }
    // 挂起：深度与槽位帧引用、机器树 ID 界
    for p in &resolution.parks {
        if p.resume_depth > resolution.stack.len() {
            return Err(oob("resume_depth", p.resume_depth as u64));
        }
        let frame = match p.slot {
            Slot::Intrinsic { frame } => frame,
            Slot::Trigger { frame } => frame,
        };
        // 会话级挂起槽（NO_FRAME 哨兵）豁免帧存在性检查
        if frame != NO_FRAME && !ids.contains_key(&frame) {
            return Err(oob("park_frame", frame as u64));
        }
        check_machine(&p.machine, b)?;
    }
    // 应答选项 ID 界
    if let Some(ans) = &resolution.answer {
        check_options(ans, b)?;
    }
    // 开放选择：观察者/选项在界、ID 水位、必须有承接机器
    if let Some(open) = &resolution.open_choice {
        check_pid(open.observer, b)?;
        check_options(&open.spec.options, b)?;
        if open.id == 0 || open.id >= resolution.next_choice_id {
            return Err(oob("choice_id", open.id));
        }
        if resolution.parks.is_empty() {
            return Err(oob("choice_without_machine", 0));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 开放选择承接派生（N04-e）
// ---------------------------------------------------------------------------

/// 开放选择的期望形状（由挂起机器的 stage/asked 派生）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedAsk {
    pub observer: PlayerId,
    pub prompt: &'static str,
    pub kind: ChoiceKind,
    pub options: Vec<ChoiceOption>,
    pub min: usize,
    pub max: usize,
    pub allow_cancel: bool,
}

fn ask(
    observer: PlayerId,
    prompt: &'static str,
    kind: ChoiceKind,
    options: Vec<ChoiceOption>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> ExpectedAsk {
    ExpectedAsk {
        observer,
        prompt,
        kind,
        options,
        min,
        max,
        allow_cancel,
    }
}

fn confirm_ask(observer: PlayerId, prompt: &'static str) -> ExpectedAsk {
    ask(
        observer,
        prompt,
        ChoiceKind::Confirm,
        vec![ChoiceOption::Confirm],
        1,
        1,
        true,
    )
}

fn hand_card_options(hand: &[CardId]) -> Vec<ChoiceOption> {
    hand.iter().copied().map(ChoiceOption::Card).collect()
}

fn other_player_options(core: &GameStateCore, owner: PlayerId) -> Vec<ChoiceOption> {
    core.alive_players()
        .filter(|p| *p != owner)
        .map(ChoiceOption::Player)
        .collect()
}

/// 濒死救援的桃判定扩面（与 kernel `is_tao_like` 一致）。
fn is_tao_like(
    core: &GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    card: CardId,
    dying: PlayerId,
) -> bool {
    if catalog.card_kind(card) == CardKind::Hufu {
        return responder == dying;
    }
    let kind = catalog.card_kind(card);
    if matches!(kind, CardKind::Tao | CardKind::Jiu | CardKind::Dinvxuanshuang) {
        return true;
    }
    let suit = catalog.cards[card as usize].suit;
    let jijiu = catalog
        .skill_by_name(JI_JIU)
        .map(|d| core.has_skill(responder, d))
        .unwrap_or(false);
    if jijiu && matches!(suit, crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond) {
        return true;
    }
    let mizhilianou = catalog
        .skill_by_name(GJ_FOOD_MIZHLIANOU_KEY)
        .map(|d| core.has_skill(responder, d))
        .unwrap_or(false);
    if mizhilianou && suit == crate::catalog::Suit::Heart {
        return true;
    }
    false
}

/// 性别门控（与 standard.rs `seat_is_male` 一致）：双将槽全空 → 不适用。
fn seat_is_male_core(core: &GameStateCore, p: PlayerId) -> bool {
    let slots = core.player(p).general_slots;
    if slots.iter().all(|s| s.is_none()) {
        return true;
    }
    slots
        .iter()
        .flatten()
        .any(|g| general_gender(*g) == Some(Gender::Male))
}

/// 从挂起机器派生当前应有的开放选择询问。
///
/// 返回 `Some` 当且仅当机器处于「已发出询问、等待应答」的静止状态；
/// 派生形状与机器创建该询问时的 `ChoiceSpec` 逐字段一致（选项含顺序）。
/// 恢复期以本函数核对 `open_choice` 的承接关系（N04-e）。
pub fn active_ask(
    machine: &MachineSnapshot,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match machine {
        // ---- 内核机器 ----
        MachineSnapshot::MonkeySteal {
            holders,
            asked,
            ..
        } => {
            if !*asked {
                return None;
            }
            let holder = *holders.first()?;
            Some(ask(
                holder,
                "huanle_monkey_confirm",
                ChoiceKind::RespondCard,
                vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                1,
                1,
                false,
            ))
        }
        MachineSnapshot::Sha {
            target,
            source,
            asked,
            tieji_red,
            liuli_stage,
            hujia_holder,
            ..
        } => {
            if *liuli_stage == 1 {
                // 流离弃牌自选（逐技对照批二 v76）：手牌+装备选一张，可取消
                let mut options: Vec<ChoiceOption> = core
                    .player(*target)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                options.extend(
                    core.player(*target)
                        .equips
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card),
                );
                return Some(ask(
                    *target,
                    "liuli_discard_choose",
                    ChoiceKind::SelectCards,
                    options,
                    0,
                    1,
                    true,
                ));
            }
            if *liuli_stage == 2 {
                // 流离转移目标询问：在（被转移者）攻击范围内、非来源非己、
                // 非空城（kernel liuli_candidate_ok 同口径）
                let candidates: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| {
                        *p != *target
                            && *p != *source
                            && {
                                let kc = catalog
                                    .skill_by_name(crate::content::standard::KONG_CHENG)
                                    .map(|d| core.has_skill(*p, d))
                                    .unwrap_or(false);
                                !(kc && core.player(*p).hand.is_empty())
                            }
                            && core.computed_distance(catalog, *target, *p)
                                <= core.attack_range(catalog, *target)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                return Some(ask(
                    *target,
                    "liuli_transfer",
                    ChoiceKind::SelectPlayers,
                    candidates,
                    1,
                    1,
                    false,
                ));
            }
            if let Some(h) = hujia_holder {
                // v204：持牌者自选一张闪（含倾国转化黑手牌）；空选=拒绝
                let qingguo_h = catalog
                    .skill_by_name(QING_GUO)
                    .map(|d| core.has_skill(*h, d))
                    .unwrap_or(false);
                let options: Vec<ChoiceOption> = core
                    .player(*h)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        let kind = catalog.card_kind(*c);
                        kind == CardKind::Shan
                            || (qingguo_h
                                && matches!(
                                    catalog.cards[*c as usize].suit,
                                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                                ))
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                return Some(ask(
                    *h,
                    "hujia_respond",
                    ChoiceKind::SelectCards,
                    options,
                    0,
                    1,
                    false, // 空选=拒绝本位（非取消）
                ));
            }
            if *asked {
                // 出闪响应：Pass + 目标的闪（龙胆并入杀、倾国并入装备黑牌、
                // 铁骑红色判定封锁常规闪；诱敌深入独立并入一张）
                let tieji_blocks = *tieji_red == Some(true);
                let longdan = catalog
                    .skill_by_name(LONG_DAN)
                    .map(|d| core.has_skill(*target, d))
                    .unwrap_or(false);
                let qingguo = catalog
                    .skill_by_name(QING_GUO)
                    .map(|d| core.has_skill(*target, d))
                    .unwrap_or(false);
                let mut options: Vec<ChoiceOption> = vec![ChoiceOption::Pass];
                for c in core.player(*target).hand.iter().copied() {
                    let kind = catalog.card_kind(c);
                    if !tieji_blocks
                        && (kind == CardKind::Shan || (longdan && kind == CardKind::Sha))
                    {
                        options.push(ChoiceOption::Card(c));
                    }
                }
                for c in core.player(*target).hand.iter().copied() {
                    if catalog.card_kind(c) == CardKind::Youdishenru {
                        options.push(ChoiceOption::Card(c));
                        break;
                    }
                }
                if qingguo && !tieji_blocks {
                    // 倾国（v202 对齐上游 position "hs"）：黑色手牌并入
                    for c in core.player(*target).hand.iter().copied() {
                        let kind = catalog.card_kind(c);
                        let already = kind == CardKind::Shan || (longdan && kind == CardKind::Sha);
                        let black = matches!(
                            catalog.cards[c as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        );
                        if black && !already {
                            options.push(ChoiceOption::Card(c));
                        }
                    }
                }
                return Some(ask(
                    *target,
                    "respond_shan",
                    ChoiceKind::RespondCard,
                    options,
                    1,
                    1,
                    false,
                ));
            }
            None
        }
        MachineSnapshot::DyingSave {
            responders, asked, dying, ..
        } => {
            if !*asked {
                return None;
            }
            let responder = *responders.first()?;
            let mut options = vec![ChoiceOption::Pass];
            options.extend(
                core.player(responder)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| is_tao_like(core, catalog, responder, *c, *dying))
                    .map(ChoiceOption::Card),
            );
            Some(ask(
                responder,
                "dying_save",
                ChoiceKind::RespondCard,
                options,
                1,
                1,
                false,
            ))
        }
        MachineSnapshot::Duel {
            source,
            target,
            on_target,
            asked,
            ..
        } => {
            if !*asked {
                return None;
            }
            let responder = if *on_target { *target } else { *source };
            let mut options = vec![ChoiceOption::Pass];
            options.extend(
                core.player(responder)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| catalog.card_kind(*c) == CardKind::Sha)
                    .map(ChoiceOption::Card),
            );
            Some(ask(
                responder,
                "duel_sha",
                ChoiceKind::RespondCard,
                options,
                1,
                1,
                false,
            ))
        }
        MachineSnapshot::NullifyWindow {
            queue,
            asked,
            window_open,
            effect,
            ..
        } => {
            if *window_open {
                if !*asked {
                    return None;
                }
                let responder = *queue.first()?;
                let mut options = vec![ChoiceOption::Pass];
                options.extend(
                    core.player(responder)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| catalog.card_kind(*c) == CardKind::Nullify)
                        .map(ChoiceOption::Card),
                );
                Some(ask(
                    responder,
                    "nullify_window",
                    ChoiceKind::Confirm,
                    options,
                    1,
                    1,
                    false,
                ))
            } else {
                // 窗口已关闭：委托效果机器
                let e = effect.as_deref()?;
                active_ask(e, core, catalog)
            }
        }
        MachineSnapshot::JudgePhase { current, .. } => {
            let c = current.as_deref()?;
            if c.in_rewrite {
                if !c.asked {
                    return None;
                }
                let rewriter = *c.rewriters.first()?;
                // 黑色手牌 + 黑色装备；鬼才持有者全部手牌可打
                let gui_cai = catalog
                    .skill_by_name(GUI_CAI)
                    .map(|d| core.has_skill(rewriter, d))
                    .unwrap_or(false);
                let is_black = |id: CardId| {
                    matches!(
                        catalog.cards[id as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    )
                };
                let mut options = vec![ChoiceOption::Pass];
                for c in core.player(rewriter).hand.iter().copied() {
                    if gui_cai || is_black(c) {
                        options.push(ChoiceOption::Card(c));
                    }
                }
                for c in core.player(rewriter).equips.iter().copied() {
                    if is_black(c) {
                        options.push(ChoiceOption::Card(c));
                    }
                }
                return Some(ask(
                    rewriter,
                    "gui_dao_rewrite",
                    ChoiceKind::RespondCard,
                    options,
                    1,
                    1,
                    false,
                ));
            }
            // 天妒询问（判定确定后、获得前、判定牌仍由机器持有）
            if c.finalized && !c.tian_du.is_empty() && c.asked && !c.gained && c.judge_held {
                return Some(confirm_ask(c.tian_du[0], "tian_du_gain"));
            }
            None
        }
        MachineSnapshot::PlaceDelayed { .. } => None,
        MachineSnapshot::Aoe { inner, .. } => {
            let e = inner.as_deref()?;
            active_ask(e, core, catalog)
        }
        MachineSnapshot::AoeTarget {
            target, need, asked, ..
        } => {
            if !*asked {
                return None;
            }
            let mut options = vec![ChoiceOption::Pass];
            options.extend(
                core.player(*target)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| catalog.card_kind(*c) == *need)
                    .map(ChoiceOption::Card),
            );
            Some(ask(
                *target,
                "aoe_response",
                ChoiceKind::RespondCard,
                options,
                1,
                1,
                false,
            ))
        }
        MachineSnapshot::Chain { .. } => None,
        MachineSnapshot::DiscardToLimit { player, need, asked } => {
            if !*asked {
                return None;
            }
            let options = hand_card_options(&core.player(*player).hand);
            Some(ask(
                *player,
                "discard_to_limit",
                ChoiceKind::SelectCards,
                options,
                *need,
                *need,
                false,
            ))
        }
        MachineSnapshot::Death { .. } => None,
        MachineSnapshot::OptionalGate { owner, asked, inner, .. } => {
            if let Some(e) = inner {
                return active_ask(e, core, catalog);
            }
            if !*asked {
                return None;
            }
            Some(confirm_ask(*owner, "trigger_confirm"))
        }
        // ---- 内容机器 ----
        MachineSnapshot::SanCai { owner, stage, .. } => match stage {
            SanCaiStageSnap::GiveCard => Some(ask(
                *owner,
                "sancai_target",
                ChoiceKind::SelectPlayers,
                other_player_options(core, *owner),
                1,
                1,
                false,
            )),
            SanCaiStageSnap::Finish => Some(ask(
                *owner,
                "sancai_card",
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(*owner).hand),
                1,
                1,
                false,
            )),
            _ => None,
        },
        MachineSnapshot::YiShang { owner, stage } => match stage {
            YiShangStageSnap::Apply => Some(ask(
                *owner,
                "yishang_target",
                ChoiceKind::SelectPlayers,
                other_player_options(core, *owner),
                1,
                1,
                false,
            )),
            _ => None,
        },
        MachineSnapshot::TuXi { owner, stage } => match stage {
            TuXiStageSnap::Read => {
                let candidates: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| *p != *owner && !core.player(*p).hand.is_empty())
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(
                    *owner,
                    "tuxi_target",
                    ChoiceKind::SelectPlayers,
                    candidates,
                    1,
                    2,
                    true,
                ))
            }
            _ => None,
        },
        MachineSnapshot::Rende { owner, stage, .. } => match stage {
            RendeStageSnap::TargetAsk => {
                let hand = &core.player(*owner).hand;
                let max = hand.len();
                Some(ask(
                    *owner,
                    "rende_cards",
                    ChoiceKind::SelectCards,
                    hand_card_options(hand),
                    0,
                    max,
                    true,
                ))
            }
            RendeStageSnap::TargetRead => Some(ask(
                *owner,
                "rende_target",
                ChoiceKind::SelectPlayers,
                other_player_options(core, *owner),
                1,
                1,
                true,
            )),
            _ => None,
        },
        MachineSnapshot::Jijiang {
            owner,
            stage,
            helper,
            sha,
            inner,
            ..
        } => {
            if let Some(e) = inner {
                return active_ask(e, core, catalog);
            }
            match stage {
                JijiangStageSnap::HelperAsk => {
                    let h = (*helper)?;
                    Some(ask(
                        h,
                        "jijiang_respond",
                        ChoiceKind::Confirm,
                        vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                        1,
                        1,
                        false,
                    ))
                }
                JijiangStageSnap::TargetAsk => {
                    // 目标询问仅在借杀成功后存在
                    if helper.is_none() || sha.is_none() {
                        return None;
                    }
                    let candidates: Vec<ChoiceOption> = core
                        .alive_players()
                        .filter(|p| {
                            *p != *owner
                                && core.computed_distance(catalog, *owner, *p)
                                    <= core.attack_range(catalog, *owner)
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    Some(ask(
                        *owner,
                        "jijiang_target",
                        ChoiceKind::SelectPlayers,
                        candidates,
                        1,
                        1,
                        true,
                    ))
                }
            }
        }
        MachineSnapshot::JieYin { owner, stage, .. } => match stage {
            JieYinStageSnap::TargetAsk => {
                let hand = &core.player(*owner).hand;
                Some(ask(
                    *owner,
                    "jieyin_cards",
                    ChoiceKind::SelectCards,
                    hand_card_options(hand),
                    2,
                    2,
                    true,
                ))
            }
            JieYinStageSnap::TargetRead => {
                let candidates: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|t| {
                        *t != *owner
                            && !core.player(*t).equips.is_empty()
                            && seat_is_male_core(core, *t)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(
                    *owner,
                    "jieyin_target",
                    ChoiceKind::SelectPlayers,
                    candidates,
                    1,
                    1,
                    true,
                ))
            }
            _ => None,
        },
        MachineSnapshot::BossXiangde { .. } => {
            // 相德：无挂起一步机（永不等待应答），恢复无询问
            None
        }
        MachineSnapshot::BossFutai { .. } => {
            // 覆泰：无挂起一步机，恢复无询问
            None
        }
        MachineSnapshot::BossLuanchang { .. } => {
            // 乱常：无挂起一步机，恢复无询问
            None
        }
        MachineSnapshot::BossZhaohuo { .. } => {
            // 招祸：无挂起一步机，恢复无询问
            None
        }
        MachineSnapshot::BossQizuo { .. } => {
            // 奇佐：无挂起一步机，恢复无询问
            None
        }
        MachineSnapshot::BossXiaoxiong { .. } => {
            // 枭雄：无挂起一步机，恢复无询问
            None
        }
        MachineSnapshot::BossLingbo { .. } => {
            // 凌波：无挂起一步机，恢复无询问
            None
        }
        MachineSnapshot::BossTashui { .. } => {
            // 踏水：无挂起一步机，恢复无询问
            None
        }
        MachineSnapshot::BossZjLeiji { .. } => {
            // 雷击·张角：无挂起一步机，恢复无询问
            None
        }
        MachineSnapshot::BossHeisheng { .. } => {
            // 黑绳：无挂起一步机，恢复无询问
            None
        }
        MachineSnapshot::BossLeizhu { .. } => {
            // 雷诛：逐发机器不产生询问（救援询问属濒死机），恢复无询问
            None
        }
        MachineSnapshot::BossRenao { .. } => {
            // 热恼：同雷诛，恢复无询问
            None
        }
        MachineSnapshot::BossWanghun { .. } => {
            // 忘魂：直写一步机，恢复无询问
            None
        }
        MachineSnapshot::BossZhuxin { .. } => {
            // 诛心：直写一步机，恢复无询问
            None
        }
        MachineSnapshot::BossPingdeng { .. } => {
            // 平等：直写两波机，恢复无询问
            None
        }
        MachineSnapshot::BossHuoxing { .. } => {
            // 火刑：直写逐座机，恢复无询问
            None
        }
        MachineSnapshot::BossBingfeng { .. } => {
            // 冰封：直写一步机，恢复无询问
            None
        }
        MachineSnapshot::BossWangxiang { .. } => {
            // 望乡：直写一步机，恢复无询问
            None
        }
        MachineSnapshot::Rvm {
            owner,
            paused: Some(p),
            ..
        } => {
            // T8 增量 2：VM 选择挂起——选项 tokens 派生（PickTag）
            let options: Vec<ChoiceOption> =
                p.options.iter().map(|t| ChoiceOption::Tag(*t as u8)).collect();
            Some(ask(
                *owner,
                "rvm_choice",
                ChoiceKind::PickTag,
                options,
                1,
                1,
                false,
            ))
        }
        MachineSnapshot::Rvm { .. } => {
            // 非挂起态：无询问
            None
        }
        MachineSnapshot::BossBuchun { owner, stage, .. } if *stage == 3 => {
            // 不臣恢复线目标询问挂起：同阵营受伤角色逐字段派生
            let seats = core.players.len() as u8;
            let candidates: Vec<ChoiceOption> = (0..seats)
                .filter(|&t| {
                    let p = core.player(t);
                    p.alive
                        && p.hp < p.max_hp as i32
                        && core.is_boss_side(t) == core.is_boss_side(*owner)
                })
                .map(ChoiceOption::Player)
                .collect();
            Some(ask(
                *owner,
                "buchun_target",
                ChoiceKind::SelectPlayers,
                candidates,
                1,
                1,
                false,
            ))
        }
        MachineSnapshot::BossBuchun { .. } => {
            // 复活线/恢复线非挂起阶段：无询问
            None
        }
        MachineSnapshot::BossXingxia { owner, stage, .. } if *stage == 1 => {
            // 星夏敌方询问挂起：异侧存活角色派生
            let seats = core.players.len() as u8;
            let candidates: Vec<ChoiceOption> = (0..seats)
                .filter(|&t| {
                    core.player(t).alive
                        && core.is_boss_side(t) != core.is_boss_side(*owner)
                })
                .map(ChoiceOption::Player)
                .collect();
            Some(ask(
                *owner,
                "xingxia_enemy",
                ChoiceKind::SelectPlayers,
                candidates,
                0,
                1,
                true,
            ))
        }
        MachineSnapshot::BossXingxia { actor, stage, .. } if *stage == 3 => {
            // 星夏弃红询问挂起：焦点敌方的手牌+装备红色牌派生
            let t = *actor as u8;
            let mut candidates: Vec<ChoiceOption> = core.player(t)
                .hand
                .iter()
                .copied()
                .filter(|c| {
                    matches!(
                        catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    )
                })
                .map(ChoiceOption::Card)
                .collect();
            for c in core.player(t).equips.iter().copied() {
                if matches!(
                    catalog.cards[c as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                ) {
                    candidates.push(ChoiceOption::Card(c));
                }
            }
            Some(ask(
                t,
                "xingxia_discard",
                ChoiceKind::SelectCards,
                candidates,
                0,
                1,
                true,
            ))
        }
        MachineSnapshot::BossXingxia { .. } => {
            // 非挂起阶段：无询问
            None
        }
        MachineSnapshot::Shedu { owner, .. } => {
            // 蛇毒：挂起态询问重挂（弃 X 或失 X）
            Some(ask(
                *owner,
                "shedu_discard_or_lose",
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(*owner).hand),
                0,
                core.player(*owner).shedu as usize,
                true,
            ))
        }
        MachineSnapshot::FanJian { owner, stage, target, .. } => match stage {
            // stage 名 = 消费者：SuitAsk 消费 fanjian_target 应答、
            // PickAsk 消费 fanjian_suit、Consume 消费 fanjian_pick（v203）
            FanJianStageSnap::SuitAsk => Some(ask(
                *owner,
                "fanjian_target",
                ChoiceKind::SelectPlayers,
                other_player_options(core, *owner),
                1,
                1,
                true,
            )),
            FanJianStageSnap::PickAsk => {
                let t = (*target)?;
                Some(ask(
                    t,
                    "fanjian_suit",
                    ChoiceKind::PickTag,
                    (0..=3).map(crate::choice::ChoiceOption::Tag).collect(),
                    1,
                    1,
                    false,
                ))
            }
            FanJianStageSnap::Consume => {
                let t = (*target)?;
                Some(ask(
                    t,
                    "fanjian_pick",
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(*owner).hand),
                    1,
                    1,
                    false,
                ))
            }
            FanJianStageSnap::TargetAsk => None,
        },
        MachineSnapshot::LiJian {
            owner, stage, inner, ..
        } => {
            if let Some(e) = inner {
                return active_ask(e, core, catalog);
            }
            match stage {
                LiJianStageSnap::TargetsAsk => {
                    let hand = &core.player(*owner).hand;
                    Some(ask(
                        *owner,
                        "lijian_discard",
                        ChoiceKind::SelectCards,
                        hand_card_options(hand),
                        1,
                        1,
                        true,
                    ))
                }
                LiJianStageSnap::Launch => {
                    let candidates: Vec<ChoiceOption> = core
                        .alive_players()
                        .filter(|p| *p != *owner && seat_is_male_core(core, *p))
                        .map(ChoiceOption::Player)
                        .collect();
                    Some(ask(
                        *owner,
                        "lijian_targets",
                        ChoiceKind::SelectPlayers,
                        candidates,
                        2,
                        2,
                        true,
                    ))
                }
                _ => None,
            }
        }
        MachineSnapshot::GuanXing { owner, stage, top } => match stage {
            GuanXingStageSnap::Place => {
                let take = top.len();
                Some(ask(
                    *owner,
                    "guanxing_arrange",
                    ChoiceKind::SelectCards,
                    hand_card_options(top),
                    0,
                    take,
                    false,
                ))
            }
            _ => None,
        },
        MachineSnapshot::QingNang { owner, stage, .. } => match stage {
            QingNangStageSnap::TargetRead => {
                let wounded: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| {
                        *p != *owner && {
                            let pl = core.player(*p);
                            pl.alive && (pl.hp as i64) < (pl.max_hp as i64)
                        }
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(
                    *owner,
                    "qingnang_target",
                    ChoiceKind::SelectPlayers,
                    wounded,
                    1,
                    1,
                    true,
                ))
            }
            QingNangStageSnap::CardRead => Some(ask(
                *owner,
                "qingnang_card",
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(*owner).hand),
                1,
                1,
                true,
            )),
            _ => None,
        },
        MachineSnapshot::JiJi { owner, stage, .. } => match stage {
            JiJiStageSnap::GiveCard => Some(ask(
                *owner,
                "jiji_target",
                ChoiceKind::SelectPlayers,
                other_player_options(core, *owner),
                0,
                1,
                true,
            )),
            JiJiStageSnap::GiveChooseCard => Some(ask(
                *owner,
                "jiji_card",
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(*owner).hand),
                1,
                1,
                true,
            )),
            _ => None,
        },
        MachineSnapshot::ZhiHeng { owner, stage } => match stage {
            ZhiHengStageSnap::Draw => {
                let hand = &core.player(*owner).hand;
                let max = hand.len();
                Some(ask(
                    *owner,
                    "zhiheng_select",
                    ChoiceKind::SelectCards,
                    hand_card_options(hand),
                    0,
                    max,
                    true,
                ))
            }
            _ => None,
        },
        // ---- G1 规则宿主机器 ----
        MachineSnapshot::Host {
            behavior, stage, owner, target, targets, cards, ..
        } => {
            active_ask_host(
                u16::from(*behavior),
                *stage,
                *owner,
                *target,
                targets,
                cards,
                core,
                catalog,
            )
        }
        // ---- 无询问机器 ----
        _ => None,
    }
}

/// 宿主机器的询问派生：按行为 + 阶段还原 host.rs 各 step 中
/// 创建该 `WaitChoice` 时的 `ChoiceSpec`（挂起时 stage 已前移到下一阶段）。
#[allow(clippy::too_many_arguments)]
fn active_ask_host(
    behavior: u16,
    stage: u8,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    use crate::content::host::{
        D3_JINGGOU_TARGET, GJ_BINGPO_SHOW_OWN, GJ_BINGPO_SHOW_TARGET, GJ_DINVXUANSHUANG_RIDER,
        GJ_FEIBIAO_DISCARD, GJ_JINLIANZHU_CONFIRM, GJ_LONGXU_PICK, GJ_LUYUGENG_COST,
        GJ_SHUJINSAN_DISCARD, GJ_XIAJIAO_DISCARD, GW_BAOXUE_DISCARD, HOST_TARGET_DISCARD_EQUIP,
        WD_CAOCHUAN_BRANCH, WD_FUDI_PLAYER_PICK, WD_FUDI_TARGET_PICK, WD_SANJIAN_DISCARD,
        WD_SANJIAN_TARGET, WD_TOULIANG_OWN, WD_TOULIANG_TARGET, WUDIAN_BAOXUE_TARGETS,
        WUDIAN_FEIDAN_DISCARD, WUDIAN_HUHUAN_CARDS, WUDIAN_KUOYIN_CARDS,
        WUDIAN_FEITIAO_CARDS, WUDIAN_FEITIAO_TARGET, WUDIAN_LIANGOU_CARDS,
        WUDIAN_LIANGOU_TARGET, WUDIAN_PINGZHANG_CARDS, WUDIAN_QIANGGU_CARDS,
        WUDIAN_QIANGGU_GUARD_ASK, WUDIAN_XJ_DIEWU_CARDS, WUDIAN_XJ_DIEWU_TARGET,
        WUDIAN_XJ_GUIYUAN_CARDS, WUDIAN_XJ_LINGYU_TARGET, WUDIAN_YINBO_CARDS,
        WUDIAN_YUEDONG_TARGETS, WUXING_SELECT_CARD, WUXING_SELECT_NATURE,
        YC_GEAN_T1_PICK, YC_GEAN_T2_PICK,
    };
    let behavior = HostBehavior::from_tag(behavior)?; // behavior: u16
    let stage = HostStage::from_tag(stage)?;
    match behavior {
        // 五行盘：改属性手牌 → 新属性
        HostBehavior::ChangeCardNature => match stage {
            HostStage::SelectNature => Some(ask(
                owner,
                WUXING_SELECT_CARD,
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(owner).hand),
                1,
                1,
                true,
            )),
            HostStage::Act => Some(ask(
                owner,
                WUXING_SELECT_NATURE,
                ChoiceKind::PickTag,
                (0..=4).map(ChoiceOption::Tag).collect(),
                1,
                1,
                true,
            )),
            _ => None,
        },
        // 手里剑：目标弃装备或受伤
        HostBehavior::TargetDiscardEquipOrDamage => {
            if stage != HostStage::Act {
                return None;
            }
            let t = target?;
            let mut options = vec![ChoiceOption::Pass];
            options.extend(
                core.player(t)
                    .equips
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card),
            );
            Some(ask(
                t,
                HOST_TARGET_DISCARD_EQUIP,
                ChoiceKind::RespondCard,
                options,
                1,
                1,
                false,
            ))
        }
        // 潜偷（swd 第三批 v92）：代价牌选择询问（owner）
        HostBehavior::SwdShentouEquipSkill => {
            if stage != HostStage::Act {
                return None;
            }
            let nh = core.player(owner).hand.len();
            if nh == 0 {
                return None;
            }
            Some(ask(
                owner,
                crate::content::host::SWD_SHENTOU_COST,
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(owner).hand),
                1,
                1,
                false,
            ))
        }
        // 御夫录（swd 第十二批 v96）：武器选择（挂起时 stage 已前移到
        // SelectCard）→ 目标选择（stage=TargetChoice）
        HostBehavior::SwdYufuluEquipSkill => match stage {
            HostStage::SelectCard => {
                let mut options: Vec<ChoiceOption> = core
                    .player(owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        catalog.card_kind(*c).equip_slot()
                            == Some(crate::catalog::EquipSlot::Weapon)
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                options.extend(
                    core.player(owner)
                        .equips
                        .iter()
                        .copied()
                        .filter(|c| {
                            catalog.card_kind(*c).equip_slot()
                                == Some(crate::catalog::EquipSlot::Weapon)
                        })
                        .map(ChoiceOption::Card),
                );
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    crate::content::host::SWD_YUFULU_WEAPON,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            HostStage::TargetChoice => Some(ask(
                owner,
                crate::content::host::SWD_YUFULU_TARGET,
                ChoiceKind::SelectPlayers,
                other_player_options(core, owner),
                1,
                1,
                false,
            )),
            _ => None,
        },
        // 屏障（U05 ow 查莉娅，规则 v138）：花色弃牌询问（挂起时 stage
        // 已前移到 SelectCard；按受伤者归属定花色）
        HostBehavior::WudianPingzhang => match stage {
            HostStage::SelectCard => {
                let hurt = target.unwrap_or(owner);
                let want = if hurt == owner {
                    crate::catalog::Suit::Heart
                } else {
                    crate::catalog::Suit::Spade
                };
                let p = core.player(owner);
                let options: Vec<ChoiceOption> = p
                    .hand
                    .iter()
                    .copied()
                    .chain(p.equips.iter().copied())
                    .filter(|c| catalog.cards[*c as usize].suit == want)
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_PINGZHANG_CARDS,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    true,
                ))
            }
            _ => None,
        },
        // 蝶舞（U05 xianjian 龙葵，规则 v153）：杀牌（挂起时 stage 已前
        // 移到 TargetChoice）→目标（挂起时 stage 已前移到 Act）
        HostBehavior::XianjianDiewu => match stage {
            HostStage::TargetChoice => {
                let p = core.player(owner);
                let options: Vec<ChoiceOption> = p
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| catalog.card_kind(*c).is_sha_like())
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_XJ_DIEWU_CARDS,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            HostStage::Act => {
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| *p != owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_XJ_DIEWU_TARGET,
                    ChoiceKind::SelectPlayers,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 灵愈（U05 xianjian 龙葵，规则 v153）：目标询问（挂起时 stage
        // 已前移到 TargetChoice；仅其他已受伤角色）
        HostBehavior::XianjianLingyu => match stage {
            HostStage::TargetChoice => {
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| {
                        *p != owner
                            && (core.player(*p).hp as u32) < core.player(*p).max_hp
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_XJ_LINGYU_TARGET,
                    ChoiceKind::SelectPlayers,
                    options,
                    1,
                    1,
                    true,
                ))
            }
            _ => None,
        },
        // 归元（U05 xianjian 林月如，规则 v147）：杀弃牌询问（挂起时
        // stage 已前移到 Act）
        HostBehavior::XianjianGuiyuan => match stage {
            HostStage::Act => {
                let p = core.player(owner);
                let options: Vec<ChoiceOption> = p
                    .hand
                    .iter()
                    .copied()
                    .chain(p.equips.iter().copied())
                    .filter(|c| catalog.card_kind(*c).is_sha_like())
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_XJ_GUIYUAN_CARDS,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 跳跃（U05 ow 温斯顿，规则 v137）：弃牌（挂起时 stage 已前移
        // 到 TargetChoice）→ 目标（挂起时 stage 已前移到 Act）
        HostBehavior::WudianFeitiao => match stage {
            HostStage::TargetChoice => {
                let p = core.player(owner);
                let options: Vec<ChoiceOption> = p
                    .hand
                    .iter()
                    .copied()
                    .chain(p.equips.iter().copied())
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_FEITIAO_CARDS,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            HostStage::Act => {
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| *p != owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_FEITIAO_TARGET,
                    ChoiceKind::SelectPlayers,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 强固（U05 ow 奥丽莎，规则 v136）：弃 2 张 he（挂起时 stage 已
        // 前移到 Act）
        HostBehavior::WudianQianggu => match stage {
            HostStage::Act => {
                let p = core.player(owner);
                let options: Vec<ChoiceOption> = p
                    .hand
                    .iter()
                    .copied()
                    .chain(p.equips.iter().copied())
                    .map(ChoiceOption::Card)
                    .collect();
                if options.len() < 2 {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_QIANGGU_CARDS,
                    ChoiceKind::SelectCards,
                    options,
                    2,
                    2,
                    false,
                ))
            }
            _ => None,
        },
        // 强固守卫（U05 ow 奥丽莎，规则 v136）：来源基本牌询问（挂起时
        // stage 已前移到 SelectCard；observer=target=杀使用者）
        HostBehavior::WudianQiangguGuard => match stage {
            HostStage::SelectCard => {
                let Some(attacker) = target else {
                    return None;
                };
                let mut options = vec![ChoiceOption::Pass];
                options.extend(
                    core.player(attacker)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            matches!(
                                catalog.card_kind(*c),
                                crate::catalog::CardKind::Sha
                                    | crate::catalog::CardKind::Shan
                                    | crate::catalog::CardKind::Tao
                                    | crate::catalog::CardKind::Jiu
                            )
                        })
                        .map(ChoiceOption::Card),
                );
                Some(ask(
                    attacker,
                    WUDIAN_QIANGGU_GUARD_ASK,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 音波（U05 ow 半藏，规则 v135）：黑桃弃牌询问（挂起时 stage
        // 已前移到 Act）
        HostBehavior::WudianYinbo => match stage {
            HostStage::Act => {
                let p = core.player(owner);
                let options: Vec<ChoiceOption> = p
                    .hand
                    .iter()
                    .copied()
                    .chain(p.equips.iter().copied())
                    .filter(|c| {
                        matches!(catalog.cards[*c as usize].suit, crate::catalog::Suit::Spade)
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_YINBO_CARDS,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 飞弹（U05 ow 法老之鹰，规则 v130）：弃牌询问（挂起时 stage
        // 已前移到 SelectCard；he 区域=手牌+装备）
        HostBehavior::WudianFeidan => match stage {
            HostStage::SelectCard => {
                let p = core.player(owner);
                let options: Vec<ChoiceOption> = p
                    .hand
                    .iter()
                    .copied()
                    .chain(p.equips.iter().copied())
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_FEIDAN_DISCARD,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    true,
                ))
            }
            _ => None,
        },
        // 链钩（U05 ow 路霸，规则 v128）：弃牌（挂起时 stage 已前移到
        // TargetChoice）→ 目标（挂起时 stage 已前移到 Act）
        HostBehavior::WudianLiangou => match stage {
            HostStage::TargetChoice => {
                let p = core.player(owner);
                let mut options: Vec<ChoiceOption> =
                    p.hand.iter().copied().map(ChoiceOption::Card).collect();
                options.extend(p.equips.iter().copied().map(ChoiceOption::Card));
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_LIANGOU_CARDS,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            HostStage::Act => {
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| *p != owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_LIANGOU_TARGET,
                    ChoiceKind::SelectPlayers,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 暴雪（U05 ow 美，规则 v127）：目标选择（挂起时 stage 已前移
        // 到 TargetChoice；上限=黑色 he 牌数，仅其他存活角色）
        HostBehavior::WudianBaoxue => match stage {
            HostStage::TargetChoice => {
                let p = core.player(owner);
                let black = p
                    .hand
                    .iter()
                    .chain(p.equips.iter())
                    .filter(|c| {
                        matches!(
                            catalog.cards[**c as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        )
                    })
                    .count();
                if black == 0 {
                    return None;
                }
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|q| *q != owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_BAOXUE_TARGETS,
                    ChoiceKind::SelectPlayers,
                    options,
                    1,
                    black,
                    true,
                ))
            }
            _ => None,
        },
        // 乐动（U05 ow 卢西奥，规则 v126）：目标选择（挂起时 stage 已
        // 前移到 TargetChoice；选项按回复模式收窄未满体力者）
        HostBehavior::WudianYuedong => match stage {
            HostStage::TargetChoice => {
                let num = 1 + core.turn.yuedong_num as usize;
                let recover = core.turn.yuedong_recover;
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| {
                        !recover
                            || (core.player(*p).hp as u32) < core.player(*p).max_hp
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_YUEDONG_TARGETS,
                    ChoiceKind::SelectPlayers,
                    options,
                    1,
                    num,
                    true,
                ))
            }
            _ => None,
        },
        // 扩音（U05 ow 卢西奥，规则 v126）：弃牌选择（挂起时 stage 已前
        // 移到 Act；1/2 张由存量决定，he 区域=手牌+装备）
        HostBehavior::WudianKuoyin => match stage {
            HostStage::Act => {
                let p = core.player(owner);
                let mut options: Vec<ChoiceOption> =
                    p.hand.iter().copied().map(ChoiceOption::Card).collect();
                options.extend(p.equips.iter().copied().map(ChoiceOption::Card));
                if options.is_empty() {
                    return None;
                }
                let (min, max) = if core.turn.yuedong_eff > 0 {
                    (1, 1)
                } else if core.turn.yuedong_num > 0 {
                    (2, 2)
                } else {
                    (1, 2)
                };
                Some(ask(
                    owner,
                    WUDIAN_KUOYIN_CARDS,
                    ChoiceKind::SelectCards,
                    options,
                    min,
                    max,
                    true,
                ))
            }
            _ => None,
        },
        // 呼唤（U05 ow 卢西奥，规则 v126）：弃 2 张（挂起时 stage 已前移
        // 到 Act）
        HostBehavior::WudianHuhuan => match stage {
            HostStage::Act => {
                let p = core.player(owner);
                let mut options: Vec<ChoiceOption> =
                    p.hand.iter().copied().map(ChoiceOption::Card).collect();
                options.extend(p.equips.iter().copied().map(ChoiceOption::Card));
                if options.len() < 2 {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_HUHUAN_CARDS,
                    ChoiceKind::SelectCards,
                    options,
                    2,
                    2,
                    true,
                ))
            }
            _ => None,
        },
        // 羊皮卷（swd 第十三批 v97）：类别询问（挂起时 stage 已前移到
        // SelectCard，静态 Tag 0–2）→ 发现三选一（stage=TargetChoice，
        // 选项读快照 cards——亮出的牌仍在牌堆）
        HostBehavior::SwdYangpijuanCard => match stage {
            HostStage::SelectCard => Some(ask(
                owner,
                crate::content::host::SWD_YANGPIJUAN_CATEGORY,
                ChoiceKind::PickTag,
                (0..3).map(ChoiceOption::Tag).collect(),
                1,
                1,
                false,
            )),
            HostStage::TargetChoice => {
                if cards.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    crate::content::host::SWD_YANGPIJUAN_PICK,
                    ChoiceKind::SelectCards,
                    cards.iter().copied().map(ChoiceOption::Card).collect(),
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 龙颅山峰（mtg 首切片 v100）：获得杀/闪二选一（挂起时 stage 已
        // 前移到 SelectCard，静态 Tag 0–1）
        HostBehavior::MtgLongshanfengLand => {
            if stage != HostStage::SelectCard {
                return None;
            }
            Some(ask(
                owner,
                crate::content::host::MTG_LONGSHANFENG_GAIN,
                ChoiceKind::PickTag,
                vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                1,
                1,
                false,
            ))
        }
        // 海岛（mtg 第二批 v101）：受伤分支弃牌/取消（挂起时 stage 已
        // 前移到 SelectCard；选项=手牌∪装备区，可取消）
        HostBehavior::MtgHaidaoLand => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let mut options = hand_card_options(&core.player(owner).hand);
            options.extend(
                core.player(owner)
                    .equips
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card),
            );
            Some(ask(
                owner,
                crate::content::host::MTG_HAIDAO_DISCARD,
                ChoiceKind::SelectCards,
                options,
                0,
                1,
                true,
            ))
        }
        // 水没墓穴（mtg 第三批 v102）：弃一摸二（挂起时 stage 已前移到
        // SelectCard；选项=手牌∪装备区，可取消）
        // 邻地溪流（mtg 第十批 v109）：弃任意张♥（挂起时 stage 已前移
        // 到 SelectCard；选项=手牌∪装备♥，可取消）
        HostBehavior::MtgLindixiliuLand => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let mut options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|c| catalog.cards[*c as usize].suit == crate::catalog::Suit::Heart)
                .map(ChoiceOption::Card)
                .collect();
            options.extend(
                core.player(owner)
                    .equips
                    .iter()
                    .copied()
                    .filter(|c| {
                        catalog.cards[*c as usize].suit == crate::catalog::Suit::Heart
                    })
                    .map(ChoiceOption::Card),
            );
            if options.is_empty() {
                return None;
            }
            let n = options.len();
            Some(ask(
                owner,
                crate::content::host::MTG_LINDIXILIU_DISCARD,
                ChoiceKind::SelectCards,
                options,
                0,
                n,
                true,
            ))
        }
        // 幽林地附技（mtg 第九批 v108）：锦囊选择（stage=SelectCard，
        // 手牌锦囊）+ 发现三选一（stage=TargetChoice，选项读快照 cards）
        HostBehavior::MtgYoulinDiscard => match stage {
            HostStage::SelectCard => {
                let options: Vec<ChoiceOption> = core
                    .player(owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        crate::content::host::yangpijuan_class(catalog.card_kind(*c))
                            == Some(1)
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    crate::content::host::MTG_YOULIN_DISCARD,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            HostStage::TargetChoice => {
                if cards.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    crate::content::host::MTG_YOULIN_PICK,
                    ChoiceKind::SelectCards,
                    cards.iter().copied().map(ChoiceOption::Card).collect(),
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 率言（U07 批二 v112）：回复者二选一（stage=SelectCard，观察者
        // =target）→ 被弃者选牌（stage=TargetChoice）
        HostBehavior::WudianShuaiyan => {
            let Some(r) = target else {
                return None;
            };
            match stage {
                HostStage::SelectCard => Some(ask(
                    r,
                    crate::content::host::WUDIAN_SHUAIYAN_CHOICE,
                    ChoiceKind::PickTag,
                    vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                    1,
                    1,
                    false,
                )),
                HostStage::TargetChoice => Some(ask(
                    r,
                    crate::content::host::WUDIAN_SHUAIYAN_DISCARD,
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(r).hand),
                    1,
                    1,
                    false,
                )),
                _ => None,
            }
        }
        // 抗yin（U07 批四 v114）：目标选择（stage=SelectCard，owner）→
        // 被弃者选牌（stage=TargetChoice，observer=target）→ 基本分支多
        // 角色摸牌（stage=CardsAsk，owner）
        HostBehavior::WudianKangyin => match stage {
            HostStage::SelectCard => Some(ask(
                owner,
                crate::content::host::WUDIAN_KANGYIN_TARGET,
                ChoiceKind::SelectPlayers,
                other_player_options(core, owner),
                1,
                1,
                false,
            )),
            HostStage::TargetChoice => {
                let Some(t) = target else {
                    return None;
                };
                let mut options = hand_card_options(&core.player(t).hand);
                options.extend(
                    core.player(t)
                        .equips
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card),
                );
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    t,
                    crate::content::host::WUDIAN_KANGYIN_DISCARD,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            HostStage::CardsAsk => {
                let x = core.player(owner)
                    .max_hp
                    .saturating_sub(core.player(owner).hp as u32)
                    .max(1) as usize;
                Some(ask(
                    owner,
                    crate::content::host::WUDIAN_KANGYIN_DRAW,
                    ChoiceKind::SelectPlayers,
                    core.alive_players().map(ChoiceOption::Player).collect(),
                    1,
                    x,
                    false,
                ))
            }
            _ => None,
        },
        // 目镜（U07 ow v118）：黑色牌选择（stage=SelectCard）→ 目标选择
        // （stage=TargetChoice，塔击同构）
        HostBehavior::WudianMujing => match stage {
            HostStage::SelectCard => {
                let options: Vec<ChoiceOption> = core
                    .player(owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        matches!(
                            catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        )
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    crate::content::host::WUDIAN_MUJING_CARD,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            HostStage::TargetChoice => Some(ask(
                owner,
                crate::content::host::WUDIAN_MUJING_TARGET,
                ChoiceKind::SelectPlayers,
                other_player_options(core, owner),
                1,
                1,
                false,
            )),
            _ => None,
        },
        // 冰河要塞（mtg 第八批 v107）：摸二弃二（挂起时 stage 已前移到
        // SelectCard；选项=手牌∪装备区，强制两张）
        HostBehavior::MtgBingheyaosaiLand => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let mut options = hand_card_options(&core.player(owner).hand);
            options.extend(
                core.player(owner)
                    .equips
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card),
            );
            if options.len() < 2 {
                return None;
            }
            Some(ask(
                owner,
                crate::content::host::MTG_BINGHEYAOSAI_DISCARD,
                ChoiceKind::SelectCards,
                options,
                2,
                2,
                false,
            ))
        }
        // 林中尖塔（mtg 第七批 v106）：武器发现三选一（stage=SelectCard，
        // 选项读快照 cards）+ 塔击基本牌选择（stage=SelectCard）+ 目标
        // 选择（stage=TargetChoice）
        HostBehavior::MtgLinzhongjiantaLand => {
            if stage != HostStage::SelectCard || cards.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                crate::content::host::MTG_LINZHONGJIANTA_DISCOVER,
                ChoiceKind::SelectCards,
                cards.iter().copied().map(ChoiceOption::Card).collect(),
                1,
                1,
                false,
            ))
        }
        HostBehavior::MtgLinzhongjiantaStrike => match stage {
            HostStage::SelectCard => {
                let options: Vec<ChoiceOption> = core
                    .player(owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        crate::content::host::yangpijuan_class(catalog.card_kind(*c))
                            == Some(0)
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    crate::content::host::MTG_LINZHONGJIANTA_CARD,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            HostStage::TargetChoice => Some(ask(
                owner,
                crate::content::host::MTG_LINZHONGJIANTA_TARGET,
                ChoiceKind::SelectPlayers,
                other_player_options(core, owner),
                1,
                1,
                false,
            )),
            _ => None,
        },
        // 沙漠绿洲（mtg 第六批 v105）：基本牌发现三选一（挂起时 stage
        // 已前移到 SelectCard；选项读快照 cards——亮出的牌仍在牌堆）
        HostBehavior::MtgShamolvzhouLand => {
            if stage != HostStage::SelectCard || cards.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                crate::content::host::MTG_SHAMOLVZHOU_DISCOVER,
                ChoiceKind::SelectCards,
                cards.iter().copied().map(ChoiceOption::Card).collect(),
                1,
                1,
                false,
            ))
        }
        // 依夏兰（mtg 第五批 v104）：类型选择（挂起时 stage 已前移到
        // SelectCard，静态 Tag 0–1）
        HostBehavior::MtgYixialanLand => {
            if stage != HostStage::SelectCard {
                return None;
            }
            Some(ask(
                owner,
                crate::content::host::MTG_YIXIALAN_GAIN,
                ChoiceKind::PickTag,
                vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                1,
                1,
                false,
            ))
        }
        // 依夏兰重铸（mtg 第五批 v104）：基本牌选择（stage=SelectCard；
        // 选项=手牌基本牌）
        HostBehavior::MtgYixialanRecast => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|c| {
                    crate::content::host::yangpijuan_class(catalog.card_kind(*c))
                        == Some(0)
                })
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                crate::content::host::MTG_YIXIALAN_RECAST_CARD,
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                false,
            ))
        }
        // 废墟（mtg 第四批 v103）：弃牌堆发现三选一（挂起时 stage 已
        // 前移到 SelectCard；选项读快照 cards——亮出的牌仍在弃堆）
        HostBehavior::MtgFeixuLand => {
            if stage != HostStage::SelectCard || cards.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                crate::content::host::MTG_FEIXU_DISCOVER,
                ChoiceKind::SelectCards,
                cards.iter().copied().map(ChoiceOption::Card).collect(),
                1,
                1,
                false,
            ))
        }
        HostBehavior::MtgShuimomuxueLand => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let mut options = hand_card_options(&core.player(owner).hand);
            options.extend(
                core.player(owner)
                    .equips
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card),
            );
            Some(ask(
                owner,
                crate::content::host::MTG_SHUIMOMUXUE_DISCARD,
                ChoiceKind::SelectCards,
                options,
                0,
                1,
                true,
            ))
        }
        // 暴雪药水：目标自选两张手牌（逐技对照批五 v79）
        HostBehavior::GwentBaoxueCard => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let t = target?;
            let n = core.player(t).hand.len().min(2);
            if n == 0 {
                return None;
            }
            Some(ask(
                t,
                GW_BAOXUE_DISCARD,
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(t).hand),
                n,
                n,
                false,
            ))
        }
        // 冰魄通：双段自选展示（逐技对照批五 v79）
        HostBehavior::JiguanBingpoCard => match stage {
            HostStage::ComparePlayerPick => Some(ask(
                owner,
                GJ_BINGPO_SHOW_OWN,
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(owner).hand),
                1,
                1,
                false,
            )),
            HostStage::CompareTargetPick => {
                let t = target?;
                Some(ask(
                    t,
                    GJ_BINGPO_SHOW_TARGET,
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(t).hand),
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 龙须钩：使用者自选目标装备（逐技对照批五 v79）
        HostBehavior::JiguanLongxuCard => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let t = target?;
            Some(ask(
                owner,
                GJ_LONGXU_PICK,
                ChoiceKind::SelectCards,
                core.player(t)
                    .equips
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect(),
                1,
                1,
                false,
            ))
        }
        // 最远射程武器（精彀）：指定一名其他角色受 1 点伤害
        HostBehavior::MaxRangeWeaponDamage => {
            if stage != HostStage::Act {
                return None;
            }
            Some(ask(
                owner,
                D3_JINGGOU_TARGET,
                ChoiceKind::SelectPlayers,
                other_player_options(core, owner),
                1,
                1,
                false,
            ))
        }
        // 三尖武器技：弃一张手牌 → 选转移伤害目标
        HostBehavior::SanjianWeaponDamage => match stage {
            HostStage::SelectNature => Some(ask(
                owner,
                WD_SANJIAN_DISCARD,
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(owner).hand),
                1,
                1,
                true,
            )),
            HostStage::Act => {
                let damaged = target?;
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| {
                        *p != owner
                            && *p != damaged
                            && core.computed_distance(catalog, damaged, *p) <= 1
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(
                    owner,
                    WD_SANJIAN_TARGET,
                    ChoiceKind::SelectPlayers,
                    options,
                    1,
                    1,
                    true,
                ))
            }
            _ => None,
        },
        // 偷梁换柱：己方选牌 → 目标选牌（交换）
        HostBehavior::TouliangSwap => {
            let t = target?;
            match stage {
                HostStage::SelectNature => Some(ask(
                    owner,
                    WD_TOULIANG_OWN,
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(owner).hand),
                    1,
                    1,
                    true,
                )),
                HostStage::Act => Some(ask(
                    owner,
                    WD_TOULIANG_TARGET,
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(t).hand),
                    1,
                    1,
                    true,
                )),
                _ => None,
            }
        }
        // 釜底抽薪：目标选拼点牌 → 己方选拼点牌
        HostBehavior::FudichouxinCompare => {
            let t = target?;
            match stage {
                HostStage::ComparePlayerPick => Some(ask(
                    t,
                    WD_FUDI_TARGET_PICK,
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(t).hand),
                    1,
                    1,
                    false,
                )),
                HostStage::Act => Some(ask(
                    owner,
                    WD_FUDI_PLAYER_PICK,
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(owner).hand),
                    1,
                    1,
                    false,
                )),
                _ => None,
            }
        }
        // 隔岸观火：目标一选牌 → 目标二选牌
        HostBehavior::GeanguanhuoCompare => {
            if targets.len() != 2 {
                return None;
            }
            match stage {
                HostStage::ComparePlayerPick => Some(ask(
                    targets[0],
                    YC_GEAN_T1_PICK,
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(targets[0]).hand),
                    1,
                    1,
                    false,
                )),
                HostStage::Act => Some(ask(
                    targets[1],
                    YC_GEAN_T2_PICK,
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(targets[1]).hand),
                    1,
                    1,
                    false,
                )),
                _ => None,
            }
        }
        // 草船借箭：分支选择（有杀 → 目标双选；无杀 → 使用者单选）
        HostBehavior::CaochuanCard => {
            if stage != HostStage::Act {
                return None;
            }
            let t = target?;
            let has_sha = core
                .player(t)
                .hand
                .iter()
                .any(|c| catalog.card_kind(*c) == CardKind::Sha);
            if has_sha {
                Some(ask(
                    t,
                    WD_CAOCHUAN_BRANCH,
                    ChoiceKind::PickTag,
                    vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                    1,
                    1,
                    false,
                ))
            } else {
                Some(ask(
                    owner,
                    WD_CAOCHUAN_BRANCH,
                    ChoiceKind::PickTag,
                    vec![ChoiceOption::Tag(1)],
                    1,
                    1,
                    false,
                ))
            }
        }
        // 舒筋散：目标弃任意张手牌摸等量
        HostBehavior::GujianShujinsanCard => {
            if stage != HostStage::Act {
                return None;
            }
            let t = target?;
            let hand = &core.player(t).hand;
            let n = hand.len();
            Some(ask(
                t,
                GJ_SHUJINSAN_DISCARD,
                ChoiceKind::SelectCards,
                hand_card_options(hand),
                1,
                n,
                true,
            ))
        }
        // 飞镖：目标弃一张黑色手牌（可取消）
        HostBehavior::GujianFeibiaoCard => {
            if stage != HostStage::Act {
                return None;
            }
            let t = target?;
            let options: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .filter(|c| {
                    matches!(
                        catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    )
                })
                .map(ChoiceOption::Card)
                .collect();
            Some(ask(
                t,
                GJ_FEIBIAO_DISCARD,
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                true,
            ))
        }
        // 金莲烛：持有者确认打出
        HostBehavior::JinlianzhuCancel => {
            if stage != HostStage::TargetChoice {
                return None;
            }
            Some(ask(
                owner,
                GJ_JINLIANZHU_CONFIRM,
                ChoiceKind::Confirm,
                vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                1,
                1,
                false,
            ))
        }
        // 菱角：摸牌阶段额外摸 1 后弃 1
        HostBehavior::GujianFoodXiajiao => {
            if stage != HostStage::SelectCard {
                return None;
            }
            Some(ask(
                owner,
                GJ_XIAJIAO_DISCARD,
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(owner).hand),
                1,
                1,
                false,
            ))
        }
        // 卤玉羹：弃一张基本牌获得牌堆顶
        HostBehavior::GujianFoodLuyugeng => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|c| {
                    matches!(
                        catalog.card_kind(*c),
                        CardKind::Sha | CardKind::Shan | CardKind::Tao | CardKind::Jiu
                    )
                })
                .map(ChoiceOption::Card)
                .collect();
            Some(ask(
                owner,
                GJ_LUYUGENG_COST,
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                false,
            ))
        }
        // 帝女玄霜：回复后可弃任意张手牌摸等量
        HostBehavior::JiguanDinvCard => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let hand = &core.player(owner).hand;
            let n = hand.len();
            Some(ask(
                owner,
                GJ_DINVXUANSHUANG_RIDER,
                ChoiceKind::SelectCards,
                hand_card_options(hand),
                0,
                n,
                true,
            ))
        }
        // 元素毁灭：逐目标弃 1–2（可取消）
        HostBehavior::HearthYuansuCard => {
            if stage != HostStage::TargetChoice {
                return None;
            }
            let t = target?;
            let mut options: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .chain(core.player(t).equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            options.truncate(2);
            if options.is_empty() {
                return None;
            }
            let n = options.len();
            Some(ask(
                t,
                crate::content::host::HH_YUANSU_DISCARD,
                ChoiceKind::SelectCards,
                options,
                1,
                n,
                true,
            ))
        }
        // 石会粉：目标出闪否则禁用
        HostBehavior::GujianShihuifenCard => {
            if stage != HostStage::TargetChoice {
                return None;
            }
            let t = target?;
            let mut options = vec![ChoiceOption::Pass];
            options.extend(
                core.player(t)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| catalog.card_kind(*c) == CardKind::Shan)
                    .map(ChoiceOption::Card),
            );
            Some(ask(
                t,
                "respond_shan",
                ChoiceKind::RespondCard,
                options,
                1,
                1,
                false,
            ))
        }
        // 神火飞鸦：逐目标需闪
        HostBehavior::JiguanShenhuoCard => {
            if stage != HostStage::TargetChoice {
                return None;
            }
            let t = target?;
            let mut options = vec![ChoiceOption::Pass];
            options.extend(
                core.player(t)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| catalog.card_kind(*c) == CardKind::Shan)
                    .map(ChoiceOption::Card),
            );
            Some(ask(
                t,
                "aoe_response",
                ChoiceKind::RespondCard,
                options,
                1,
                1,
                false,
            ))
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// 机器重建（MachineSnapshot → Box<dyn SkillMachine>）
// ---------------------------------------------------------------------------

impl MachineSnapshot {
    /// 由机器快照重建机器实例（内核机器就地分派；内容机器委托
    /// `content::standard::restore_content_machine`；宿主机器按标签重建）。
    pub fn restore(&self) -> Result<Box<dyn SkillMachine>, SnapshotError> {
        // 内容侧分派（标准内容机器 + 委托 kernel 的拆桥/牵羊）
        if let Some(m) = crate::content::standard::restore_content_machine(self) {
            return Ok(m);
        }
        match self {
            MachineSnapshot::Sha {
                card,
                source,
                target,
                nature,
                asked,
                armor_judged,
                dodges_used,
                tieji_red,
                liuli_done,
                liuli_stage,
            liuli_card,
                hujia_queue,
                hujia_holder,
                jiu_bonus,
                asking_qilin,
                qilin_horse,
                asking_hanbing,
                asking_fengxueren,
                asking_guanshi,
                guanshi_cards,
                asking_qinglong,
                asking_cixiong,
                cixiong_done,
                extra_targets,
                hanbing_pending,
                asking_xiandan,
                xiandan_done,
                xiandan_bonus,
                asking_lueying,
                lueying_victim,
                lueying_declined,
                asking_fuchou,
                fuchou_card,
                asking_sadeng,
                asking_sadeng_shan,
            } => Ok(Box::new(crate::kernel::ShaResolveMachine::new(
                *card,
                *source,
                *target,
                *nature,
                *asked,
                *armor_judged,
                *dodges_used,
                *tieji_red,
                *liuli_done,
                *liuli_stage,
                liuli_card.map(|c| c as crate::ids::CardId),
                hujia_queue.clone(),
                *hujia_holder,
                *jiu_bonus,
                *asking_qilin,
                qilin_horse.map(|c| c as crate::ids::CardId),
                *asking_hanbing,
                *asking_fengxueren,
                *asking_guanshi,
                guanshi_cards.iter().copied().map(|c| c as crate::ids::CardId).collect(),
                *asking_qinglong,
                *asking_cixiong,
                *cixiong_done,
                extra_targets.iter().copied().map(|c| c as crate::ids::PlayerId).collect(),
                *hanbing_pending,
                *asking_xiandan,
                *xiandan_done,
                *xiandan_bonus,
                *asking_lueying,
                lueying_victim.map(|c| c as crate::ids::PlayerId),
                *lueying_declined,
                *asking_fuchou,
                fuchou_card.map(|c| c as u32),
                *asking_sadeng,
                *asking_sadeng_shan,
            ))),
            MachineSnapshot::DyingSave {
                dying,
                source,
                responders,
                asked,
                round_used,
                guihan_asked: _,
            } => Ok(Box::new(crate::kernel::DyingSaveMachine::new(
                *dying,
                *source,
                responders.clone(),
                *asked,
                *round_used,
            ))),
            MachineSnapshot::Duel {
                card,
                source,
                target,
                on_target,
                asked,
                second_pending,
            } => Ok(Box::new(crate::kernel::DuelMachine::new(
                *card,
                *source,
                *target,
                *on_target,
                *asked,
                *second_pending,
            ))),
            MachineSnapshot::NullifyWindow {
                card,
                source,
                nullified,
                queue,
                asked,
                window_open,
                owns_disposal,
                effect,
            } => {
                let effect = match effect {
                    Some(e) => match e.restore() {
                        Ok(m) => Some(m),
                        Err(_) => return Err(SnapshotError::UnknownMachine),
                    },
                    None => None,
                };
                Ok(Box::new(crate::kernel::NullifyWindowMachine::new(
                    *card,
                    *source,
                    *nullified,
                    queue.clone(),
                    *asked,
                    *window_open,
                    *owns_disposal,
                    effect,
                )))
            }
            MachineSnapshot::JudgePhase { owner, queue, current } => {
                Ok(Box::new(crate::kernel::JudgePhaseMachine::new(
                    *owner,
                    queue.clone(),
                    current
                        .as_deref()
                        .map(crate::kernel::JudgementProgress::from_snap),
                )))
            }
            MachineSnapshot::Noop => Ok(Box::new(crate::kernel::NoopMachine::new())),
            MachineSnapshot::PlaceDelayed { card, target, placed } => Ok(Box::new(
                crate::kernel::PlaceDelayedMachine::new(*card, *target, *placed),
            )),
            MachineSnapshot::Aoe {
                card,
                source,
                need,
                targets,
                idx,
                inner,
                nature,
            } => {
                let inner = match inner {
                    Some(e) => match e.restore() {
                        Ok(m) => Some(m),
                        Err(_) => return Err(SnapshotError::UnknownMachine),
                    },
                    None => None,
                };
                Ok(Box::new(crate::kernel::AoeResolveMachine::new(
                    *card,
                    *source,
                    *need,
                    targets.clone(),
                    *idx,
                    inner,
                    *nature,
                )))
            }
            MachineSnapshot::AoeTarget {
                source,
                target,
                need,
                asked,
                card,
                nature,
            } => Ok(Box::new(crate::kernel::AoeTargetMachine::new(
                *source,
                *target,
                *need,
                *asked,
                *card,
                *nature,
            ))),
            MachineSnapshot::Chain {
                card,
                source,
                targets,
                emitted,
            } => Ok(Box::new(crate::kernel::ChainMachine::new(
                *card,
                *source,
                targets.clone(),
                *emitted,
            ))),
            MachineSnapshot::DiscardToLimit { player, need, asked } => Ok(Box::new(
                crate::kernel::DiscardToLimitMachine::new(*player, *need, *asked),
            )),
            MachineSnapshot::Death { player, killer } => {
                Ok(Box::new(crate::kernel::DeathMachine::new(*player, *killer)))
            }
            MachineSnapshot::BossLeizhu { owner, emitted } => Ok(Box::new(
                crate::content::standard::LeizhuMachine::new(*owner, *emitted),
            )),
            MachineSnapshot::BossRenao { owner, emitted } => Ok(Box::new(
                crate::content::standard::RenaoMachine::new(*owner, *emitted),
            )),
            MachineSnapshot::Rvm {
                owner,
                module,
                paused,
            } => {
                let m = rule_vm::BytecodeModule::from_bytes(module)
                    .map_err(|_| SnapshotError::UnknownMachine)?;
                let machine = if let Some(mut p) = paused.clone() {
                    crate::rvm_bridge::VmMachine::restore_paused(
                        *owner,
                        m,
                        std::mem::take(&mut p.frames),
                        p.pc,
                        p.fuel_remaining,
                        p.id,
                        p.module_hash,
                        std::mem::take(&mut p.options),
                        std::mem::take(&mut p.pending),
                    )
                    .map_err(|_| SnapshotError::UnknownMachine)?
                } else {
                    crate::rvm_bridge::VmMachine::from_module(*owner, m)
                };
                Ok(Box::new(machine))
            }
            MachineSnapshot::OptionalGate {
                owner,
                def,
                event,
                asked,
                inner,
            } => {
                let inner = match inner {
                    Some(e) => match e.restore() {
                        Ok(m) => Some(m),
                        Err(_) => return Err(SnapshotError::UnknownMachine),
                    },
                    None => None,
                };
                Ok(Box::new(crate::resolution::OptionalGate::new(
                    *owner,
                    *def,
                    event.clone(),
                    *asked,
                    inner,
                )))
            }
            MachineSnapshot::Host {
                def_key,
                behavior,
                stage,
                owner,
                target,
                card,
                cards,
                targets,
            } => {
                let behavior =
                    HostBehavior::from_tag(*behavior).ok_or(SnapshotError::UnknownMachine)?;
                let stage = HostStage::from_tag(*stage).ok_or(SnapshotError::UnknownMachine)?;
                Ok(Box::new(HostMachine {
                    def_key: def_key.clone(),
                    behavior,
                    stage,
                    owner: *owner,
                    target: *target,
                    card: *card,
                    cards: cards.clone(),
                    targets: targets.clone(),
                }))
            }
            _ => Err(SnapshotError::UnknownMachine),
        }
    }
}
