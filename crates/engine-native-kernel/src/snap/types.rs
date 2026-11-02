use super::*;

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
    pub trick: Option<CardId>,
    pub judge_card: CardId,
    /// 鬼道/鬼才改判询问队列（座位序）。
    pub rewriters: Vec<PlayerId>,
    pub rewrite_provider: Option<crate::ids::SkillDefId>,
    pub used_rewrite_providers: Vec<(PlayerId, crate::ids::SkillDefId)>,
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
    /// 改命（T11 Boss 诸葛亮族批 116）：亮出的 7 张候选（Some=等待选一）。
    pub gaiming_cards: Option<Vec<u32>>,
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
    DamageFlow {
        root: crate::ids::EventId,
    },
    JudgeContinuation {
        owner: PlayerId,
        queue: Vec<CardId>,
        current: Option<Box<JudgementProgressSnap>>,
        ordinary: Option<crate::ids::EventId>,
        started: bool,
        payment: Option<JudgeReplacementSnap>,
    },
    NativeInvocation {
        owner: PlayerId,
        def: SkillDefId,
        window: crate::event::DamageWindowKind,
        progress: crate::event::NativeInvocationProgress,
    },
    NativeDamageSkill {
        owner: PlayerId,
        def: SkillDefId,
        window: crate::event::DamageWindowKind,
    },
    Sha {
        card: Option<CardId>,
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
        /// 狩猎（swd deferred 消解批 95）：本杀不可闪避（弃牌确认后置位）。
        shoulie_direct: bool,
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
        idx: usize,
        inner: Option<Box<MachineSnapshot>>,
        /// true=整牌调度层（逐目标无懈窗口 + 收尾弃置）；false=单目标叶子。
        dispatcher: bool,
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
    TriggerOrder {
        frame: crate::ids::EventId,
        owner: PlayerId,
        candidates: Vec<SkillDefId>,
        asked: bool,
        cursor: u16,
        done: Vec<(PlayerId, SkillDefId)>,
        owners: Vec<PlayerId>,
        normal_owners: u16,
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
    /// B7（规则 v324）：声明技能名、触发事件原文、限次清单与命中计数
    /// 旗标入快照——此前恢复从模块名重建技能名并把这些字段重置为默认，
    /// 续行 STORAGE_SET/once/hits 的键与未中断执行不一致。
    Rvm {
        owner: PlayerId,
        module: Vec<u8>,
        paused: Option<RvmPaused>,
        /// 声明技能名（目录 def 名；与模块名独立保存，恢复不再从模块名重建）。
        skill_name: String,
        /// 触发事件原文（恢复时经 `trigger_ctx_of` 重派生触发上下文）。
        event: crate::event::EventData,
        /// Halted 时写入 once 的 (键, 范围) 清单（空键表示用技能名）。
        consume: Vec<(String, u8)>,
        /// 命中计数旗标（伤害触发 Halted 时递增 `vm_aux.hits`）。
        count_hits: bool,
        /// J1（ADR 0786 §A，规则 v328）：提交期挂起——提交期开窗时的剩余
        /// 效果队列（rvm_bridge 自定义编码；未挂起为空）。
        commit_pending: Vec<u8>,
        /// J1：开窗参数（开窗座位, 弃置张数）；与 `paused` 互斥（VM 执行期
        /// 挂起）——同一时刻至多一种挂起。
        commit_pick: Option<(PlayerId, u32)>,
        /// Remaining handlers of the same logical skill, in package order.
        following: Vec<MachineSnapshot>,
        /// Content callback continuation, encoded at the Rvm tail.
        commit_state: Vec<u32>,
        committing: bool,
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
    RuleVersionMismatch { snapshot: u64, catalog: u64 },
    /// 越界/非法 ID 或结构关系；`what` 指明检查项。
    IdOutOfBounds { what: &'static str, got: u64 },
    /// 机器快照无法在当前目录下重建（未知机器类型/扩展缺包/坏标签）。
    UnknownMachine,
    /// 开放选择与挂起机器的承接关系不一致（N04-e）。
    OpenChoiceMismatch { observer: PlayerId },
    /// G1 装备技能生命周期：装备在场而技能实例缺失。
    EquipSkillMismatch { player: PlayerId },
    /// 机器快照载荷的坏标签。
    BadTag(&'static str, u64),
}

// ---------------------------------------------------------------------------
// 恢复边界与校验
// ---------------------------------------------------------------------------

/// 恢复边界：目录尺寸 + 座位数 + 规则版本的恢复期视图。
#[derive(Debug, Clone, Copy)]
pub struct RestoreBounds {
    pub vm_validate:
        fn(&rule_api::EffectRequest, &[u32], &crate::state::GameStateCore, usize) -> bool,
    pub seats: usize,
    pub cards: usize,
    pub skills: usize,
    pub generals: usize,
    pub ruleset_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JudgeReplacementSnap {
    pub provider: crate::ids::SkillDefId,
    pub player: PlayerId,
    pub old: CardId,
    pub discard_old: bool,
}
