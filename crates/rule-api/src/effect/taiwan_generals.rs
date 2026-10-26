//! Taiwan generals effect definitions.

macro_rules! variants {
    (ResolveTwfupan [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 复叛：按伤害点数摸牌，交出一张牌，并按目标的累计状态结算奖励或伤害。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwfupan {
        /// Skill owner.
        player: PlayerId,
        /// Damage amount captured by the trigger.
        damage_amount: u32,
        /// Selected card between the card and target choice stages.
        #[serde(default = "no_card")]
        card: CardId,
        /// Selected recipient between the gift and confirmation stages.
        #[serde(default = "no_player")]
        target: PlayerId,
        /// Engine-owned stage.
        #[serde(default)]
        stage: u8,
        /// Originating trigger skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveTwjiaohua [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 教化（TW）：摸牌后令摸牌角色获得一张本次未获得类别的牌。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwjiaohua {
        /// Skill owner.
        player: PlayerId,
        /// Player who drew the triggering cards.
        target: PlayerId,
        /// Physical cards captured by the draw event.
        event_cards: Vec<CardId>,
        /// Originating trigger skill instance.
        skill: SkillId,
    },
    /// 苦谏（TW）：赠予并追踪实体牌，或结算其使用/打出及其他失去分支。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwkujian {
        /// Skill owner.
        player: PlayerId,
        /// Gift recipient or player who used/lost tracked cards.
        target: PlayerId,
        /// Selected or event physical cards.
        cards: Vec<CardId>,
        /// Zero gifts; one rewards use/response; two penalizes another loss.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveTwruilian [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 睿敛（TW）：选择本轮关注角色、记录其弃牌，并在其回合结束按类别获得弃牌堆牌。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwruilian {
        /// Skill owner.
        player: PlayerId,
        /// Selected role or triggering event player.
        target: PlayerId,
        /// Physical cards captured by a discard event.
        #[serde(default)]
        event_cards: Vec<CardId>,
        /// Zero prompts for a target; one records a discard; two closes the tracked turn; three gains.
        action: u8,
        /// Selected category for action three: 1 basic, 2 trick, 3 equipment.
        #[serde(default)]
        category: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveTwjianwei [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 剑威（TW）：双向发起拼点并按胜负结算区域牌或武器。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwjianwei {
        /// Skill owner.
        player: PlayerId,
        /// Comparison initiator.
        actor: PlayerId,
        /// Comparison opponent.
        #[serde(default = "no_player")]
        target: PlayerId,
        /// Initiator's selected comparison card.
        #[serde(default = "no_card")]
        card: CardId,
        /// Comparison winner for settlement.
        #[serde(default = "no_player")]
        winner: PlayerId,
        /// Zero starts owner's branch; one starts another player's branch; two starts the
        /// comparison after both participants are fixed; three settles the finished comparison.
        #[serde(default)]
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 探锋（TW）：弃置目标区域牌后令其选择以牌当【杀】，否则受伤并跳过阶段。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwtanfeng {
        /// Skill owner.
        player: PlayerId,
        /// Selected other player.
        #[serde(default = "no_player")]
        target: PlayerId,
        /// Selected physical card used as Sha.
        #[serde(default = "no_card")]
        card: CardId,
        /// Resolver stage.
        #[serde(default)]
        stage: u8,
        /// Selected future phase tag for the skip branch.
        #[serde(default)]
        phase: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 直辩（TW）：发起拼点并结算移动对应区域牌、回复或背水。
    ResolveTwzhibian {
        /// Skill owner.
        player: PlayerId,
        /// Comparison opponent.
        #[serde(default = "no_player")]
        target: PlayerId,
        /// Comparison winner; NONE is a tie.
        #[serde(default = "no_player")]
        winner: PlayerId,
        /// Resolver stage.
        #[serde(default)]
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 御严（TW）：令当前 Sha 使用者交牌，否则取消拥有者这一目标。
    ResolveTwyuyan {
        /// Skill owner and current Sha target.
        player: PlayerId,
        /// Current Sha source.
        source: PlayerId,
        /// Current physical Sha carrier.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 果毅（TW）：目标选择禁用手牌或弃牌，并在满足条件时登记原牌重放。
    ResolveTwguoyi {
        /// Skill owner and current card user.
        player: PlayerId,
        /// Current card target making the branch choice.
        target: PlayerId,
        /// Current physical or materialized card.
        card: CardId,
        /// Temporary helper skill that disables hand-card use and responses.
        hand_lock_skill_def: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveTwchuhai [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 除害（TW）：完成使命后的判定区废除/座次收牌，或弃置本批从他人获得的牌。
    ResolveTwchuhai {
        /// Skill owner.
        player: PlayerId,
        /// Zero resolves mission success; one discards one triggering gained card.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Engine-owned seat queue index.
        index: u32,
        /// Physical cards captured by the triggering movement.
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 遗珠（TW）：埋置实体牌、修改他人的单目标用牌，或清除弃牌堆中的追踪牌。
    ResolveTwyizhu {
        /// Skill owner.
        player: PlayerId,
        /// Current card user for target adjustment.
        source: PlayerId,
        /// Current logical card.
        card: CardId,
        /// Zero plants pearls; one handles their use; two clears discarded pearls.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Physical cards captured by the triggering event.
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 乱筹/共患（TW）：重置共享技能持有者，或将当前伤害完整转移给技能拥有者。
    ResolveTwluanchou {
        /// Skill owner.
        player: PlayerId,
        /// Shared helper skill definition.
        helper_skill_def: u32,
        /// Zero resets the two selected holders; one redirects current damage.
        action: u8,
        /// Active invocation targets.
        targets: Vec<PlayerId>,
        /// Current damage source.
        source: PlayerId,
        /// Current damage target.
        target: PlayerId,
        /// Current damage amount.
        amount: u32,
        /// Current damage nature.
        nature: u8,
        /// Current damage card.
        card: CardId,
        /// Current logical card kind.
        card_kind: u8,
        /// Current card provenance flags.
        card_flags: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveTwxiawei [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 狭威/穷技（TW）：维护“威”牌堆、妄行代价与空堆伤害增量。
    ResolveTwxiawei {
        /// Skill owner.
        player: PlayerId,
        /// Temporary 妄行 settlement helper definition.
        wangxing_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 挽危/约俭（TW）：处理最低体力伤害保护、回合末牌堆操作与约俭置顶置底收益。
    ResolveTwbianfuren {
        /// Skill owner.
        player: PlayerId,
        /// Current damage target, or NONE outside 挽危伤害结算.
        target: PlayerId,
        /// 挽危临时技能或约俭手牌上限技能定义.
        helper_skill_def: u32,
        /// Zero handles 挽危 damage; one handles its end-phase effect; two handles 约俭.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected physical cards.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 生息/宽济（TW）：获得调剂盐梅/智囊，或转赠非使用进入弃牌堆的牌。
    ResolveTwfeiyi {
        /// Skill owner.
        player: PlayerId,
        /// Zero prepare Shengxi; one end Shengxi; two Kuanji.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected card.
        card: CardId,
        /// Physical cards carried by the trigger event.
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 睦约/察异（TW）：声明牌名获牌及察异的延迟分支切换。
    ResolveTwchenzhen {
        /// Skill owner or helper owner.
        player: PlayerId,
        /// Current event player when relevant.
        target: PlayerId,
        /// 睦约免弃技能或察异弃牌技能定义.
        helper_skill_def: u32,
        /// 察异延迟记录技能定义.
        record_skill_def: u32,
        /// Zero Muyue; one Chayi setup; two use-card discard; three delayed switch.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Declared physical card kind tag.
        card_kind: u8,
        /// Selected physical card.
        card: CardId,
        /// Branch/state byte.
        branch: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 前驱乐/倾袭（TW）：伤害后补装备，或结算首杀目标的装备选择。
    ResolveTwqianju {
        /// Skill owner.
        player: PlayerId,
        /// Damaged player.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 倾袭（TW）：首杀目标选择摸牌直击或双方弃装备并加伤。
    ResolveTwqingxi {
        /// Sha user and skill owner.
        player: PlayerId,
        /// Current Sha target.
        target: PlayerId,
        /// Current physical Sha.
        card: CardId,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Number of target equipment cards discarded by the branch.
        count: u8,
        /// Selected source equipment cards.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 躁厉（TW）：弃牌摸牌并按原装备槽补入装备。
    ResolveTwzaoli {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected discard cards.
        cards: Vec<CardId>,
        /// Distinct equipment-slot tags to refill.
        slots: Vec<u8>,
        /// Number of cards discarded by the skill.
        count: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 征建/众斥（TW）：选择惩罚条件、收牌切换并在累计后觉醒。
    ResolveTwzhengjian {
        /// Skill owner.
        player: PlayerId,
        /// Current Play-phase owner.
        target: PlayerId,
        /// 众斥伤害加值技能定义.
        helper_skill_def: u32,
        /// Zero setup; one phase-end settlement.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected physical card.
        card: CardId,
        /// Choice bitfield.
        branch: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 震袭/扬师（TW）：弃手牌、移动场上牌，或按攻击范围成长/获得【杀】。
    ResolveTwtianyu {
        /// Skill owner.
        player: PlayerId,
        /// Current Sha target for 震袭; selected move destination in the final stage.
        target: PlayerId,
        /// 扬师 attack-range mark skill definition.
        helper_skill_def: u32,
        /// Zero for 震袭; one for 扬师.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected field card.
        card: CardId,
        /// 震袭 branch bitfield: discard = 1, move = 2.
        branch: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 锋捍/从击（TW）：按牌目标数选择摸牌，或转赠事件中的红色弃牌。
    ResolveTwwujing {
        /// Skill owner.
        player: PlayerId,
        /// Declared target count for 锋捍.
        target_count: u8,
        /// Zero for 锋捍; one for 从击.
        action: u8,
        /// Cards carried by the triggering movement event.
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveTwshanghe [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 商和（TW）：濒死时其他角色依次交牌；无人交【酒】则回复至一点体力。
    ResolveTwshanghe {
        /// Dying skill owner.
        player: PlayerId,
        /// Zero initializes seat order; one continues an existing queue.
        stage: u8,
        /// Remaining contributors in authoritative seat order.
        remaining: Vec<PlayerId>,
        /// Whether any contributed card was logically Jiu for the dying owner.
        got_jiu: bool,
        /// Originating limited skill instance.
        skill: SkillId,
    },
    /// 开济/慑叛（TW 王昶）的目标选择、置顶与当前目标排除。
    ResolveTwwangchang {
        /// Skill owner.
        player: PlayerId,
        /// Card user for 慑叛; NONE for 开济.
        target: PlayerId,
        /// Triggering physical/virtual card carrier.
        card: CardId,
        /// Selected target-region card for 慑叛置顶.
        selected_card: CardId,
        /// Zero for 开济; one for 慑叛.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected 开济 draw targets.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 辅纂/宠齐/非服（TW 曹肇）的转换技切换、全场授予与单目标【杀】赠牌。
    ResolveTwcaozhao {
        /// Skill owner.
        player: PlayerId,
        /// Event target or selected player.
        target: PlayerId,
        /// Event source for 非服.
        source: PlayerId,
        /// Triggering or selected physical card.
        card: CardId,
        /// Selected conversion skill definition.
        selected_skill_def: u32,
        /// 辅纂 definition granted by 宠齐.
        twfuzuan_skill_def: u32,
        /// 非服 definition granted by 宠齐.
        twfeifu_skill_def: u32,
        /// Zero 辅纂; one 宠齐; two 非服.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 精策/御章（TW 郭淮）的摸牌蓄策、受伤分支与跳阶段结算。
    ResolveTwguohuai {
        /// Skill owner.
        player: PlayerId,
        /// Trigger event player.
        target: PlayerId,
        /// Damage source.
        source: PlayerId,
        /// Triggering card carrier.
        card: CardId,
        /// 御章 mark definition.
        mark_def: u32,
        /// Temporary no-card helper skill definition.
        helper_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 疠火/醇醪（TW 程普）的改火代价、置醇、增伤与救援结算。
    ResolveTwchengpu {
        /// Skill owner.
        player: PlayerId,
        /// Event or selected target.
        target: PlayerId,
        /// Card user or damage source.
        source: PlayerId,
        /// Triggering Sha carrier.
        card: CardId,
        /// 醇醪 skill definition and pile key.
        chunlao_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected field/hand/equipment card.
        selected_card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 孝廉（TW 曹昂）的单目标杀替换与“马”牌距离结算。
    ResolveTwcaoang {
        /// Skill owner.
        player: PlayerId,
        /// Original Sha target or damage target.
        target: PlayerId,
        /// Sha user or damage source.
        source: PlayerId,
        /// Triggering Sha carrier.
        card: CardId,
        /// Distance helper skill and pile definition.
        helper_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 挫锐/裂围（TW 牛金）的限定补牌、判定区废除与击杀奖励。
    ResolveTwniujin {
        /// Skill owner.
        player: PlayerId,
        /// 挫锐 limited skill definition.
        limited_skill_def: SkillDefId,
        /// Zero resolves 挫锐; one resolves 裂围.
        action: u8,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 谋溃（TW 伏完）的摸牌、弃目标牌与背水结算。
    ResolveTwmoukui {
        /// Sha user and skill owner.
        player: PlayerId,
        /// Current Sha target.
        target: PlayerId,
        /// Exact Sha carrier.
        card: CardId,
        /// Engine-owned stage; zero opens the branch choice.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 抚蛮（TW 马忠）的实体赠牌与技能实例追踪。
    ResolveTwfuman {
        /// Skill owner and card giver.
        player: PlayerId,
        /// Selected other recipient.
        target: PlayerId,
        /// Selected physical hand card.
        cards: Vec<CardId>,
        /// Zero gives selected cards; one clears the event card; two clears moved event cards.
        action: u8,
        /// Engine-owned stage; zero performs the gift.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveTwqingkou [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 清口（TW 冯习）的精确虚拟【决斗】结算。
    ResolveTwqingkou {
        /// Skill owner and Duel initiator.
        player: PlayerId,
        /// Duel damage source on completion; ignored while starting the Duel.
        winner: PlayerId,
        /// Exact materialized virtual Duel card; ignored while starting the Duel.
        card: CardId,
        /// Zero starts and tracks the virtual Duel; one applies its post-resolution reward.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 陈勋（幻刘封）的逐轮虚拟【决斗】链。
    ResolveTwchenxun {
        /// Skill owner and Duel initiator.
        player: PlayerId,
        /// Selected Duel target; none while opening a target choice.
        target: PlayerId,
        /// Exact materialized Duel card at post-resolution boundaries.
        card: CardId,
        /// Zero starts a round; one rewards and continues; two penalizes; three uses the
        /// selected target; four opens the next optional target choice.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 焚险（幻黄盖）的目标分支与实体牌转化结算。
    ResolveTwfenxian {
        /// Skill owner, except action four where this is the Duel user.
        player: PlayerId,
        /// Selected skill target, except action four where this is the Duel target.
        target: PlayerId,
        /// Selected field card for the Duel branch.
        card: CardId,
        /// Zero opens branches; one uses Huogong; two opens field cards; three opens Duel
        /// targets; four uses the selected field card as Duel.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveTwqiushou [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 求首（shenhua 孟获，主公技）：南蛮结算后目标死亡或受创目标数超过 3 时，
    /// 所有存活蜀/群角色各摸一张牌。ABI 542。
    ResolveTwqiushou {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveTwjuxiang [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 踞襄（shenhua 张绣，主公技）：其他群势力角色出牌阶段限一次，可将一张装备牌
    /// 置入你的装备区（栏位空闲/禁用时直接装备并恢复栏位），否则你获得之。ABI 590。
    ResolveTwjuxiang {
        /// Skill owner (the lord).
        player: PlayerId,
        /// 触发角色的出牌阶段（其他群势力角色）。
        event_player: PlayerId,
        /// 0=装备牌选择, 1=置入或获得.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveTwmouzhi [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 专属状态机 (批 245): twmouzhi 谋制 — mode 0: prevent same-color
    /// consecutive damage; mode 1: record the damage card's color.
    ResolveTwmouzhi {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// 0 = damage_before check/prevent, 1 = damage record.
        mode: u32,
        /// Damage event card (0 if none).
        event_card: u32,
    },
    /// 专属状态机 (批 247): twdaigui 待归 slice 1 — uniform-color hand gate,
    /// choose up to hand-count players, reveal deck-bottom cards.
    ResolveTwdaigui {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Machine stage (0 = entry/target selection, 2 = reveal).
        stage: u32,
        /// Number of revealed cards (target count bound at stage 2).
        count: u32,
    },
    /// 专属状态机 (批 249): twmitu 迷途 slice 1 — prepare phase, choose up to
    /// three players; each draws two 迷途-tagged cards and shows them.
    ResolveTwmitu {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Machine stage (0 = entry/target selection).
        stage: u32,
        /// Reserved.
        count: u32,
    },
        ] }
    };
    (ResolveTwxiongjun [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// every living player owning 雄军 draws one card.
    ResolveTwxiongjun {
        /// Damage source (skill owner).
        player: PlayerId,
    },
        ] }
    };
    (ResolveTwjieqiu [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 劫囚·装备栏编排（ABI 673，tw twjieqiu）：多路器——
    /// sub 0：选择一名其他角色，禁用其全部装备槽（现存装备入弃牌堆）并令其
    ///      摸 k 张（k=废除前装备区牌数），打上 BUFF 标记；
    /// sub 2：若目标仍有禁用槽，为持有者入队一个纯额外回合。
    ResolveTwjieqiu {
        /// 技能持有者。
        player: PlayerId,
        /// 被劫囚目标（sub 0 时忽略）。
        target: PlayerId,
        /// 子动作（0=废除+摸k，2=余栏判额外回合）。
        subaction: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
}
pub(super) use variants;
