//! Classic generals effect definitions.

macro_rules! variants {
    (ResolveMassDamageDiscardTurnOver [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Consume marks, damage every other living player in seat order, then make those players
    /// discard all equipment and a bounded number of hand cards before turning the owner over.
    ///
    /// `stage` and `index` are engine-owned state-machine fields. Appended for serialized
    /// ordinal stability.
    ResolveMassDamageDiscardTurnOver {
        /// Skill owner and damage source.
        player: PlayerId,
        /// Mark definition consumed at stage zero.
        mark: MarkDefId,
        /// Number of marks consumed.
        mark_count: u32,
        /// Normal damage dealt to each target.
        damage_amount: u32,
        /// Maximum hand cards each target must discard.
        hand_count: u32,
        /// 0 = initialize, 1 = damage, 2 = equipment, 3 = hand, 4 = turn over.
        stage: u8,
        /// Next target index within the stable target snapshot.
        index: u32,
        /// Stable other-player snapshot in seat order.
        targets: Vec<PlayerId>,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Consume marks, damage every other living player in seat order, then make those players
    /// discard a bounded number of cards from their combined hand/equipment zones.
    ///
    /// `stage` and `index` are engine-owned state-machine fields. Appended for serialized
    /// ordinal stability.
    ResolveMassDamageDiscardHandEquip {
        /// Skill owner and damage source.
        player: PlayerId,
        /// Mark definition consumed at stage zero.
        mark: MarkDefId,
        /// Number of marks consumed.
        mark_count: u32,
        /// Normal damage dealt to each target.
        damage_amount: u32,
        /// Maximum hand/equipment cards each target must discard.
        discard_count: u32,
        /// 0 = initialize, 1 = damage, 2 = discard.
        stage: u8,
        /// Next target index within the stable target snapshot.
        index: u32,
        /// Stable other-player snapshot in seat order.
        targets: Vec<PlayerId>,
        /// Originating active skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveMarkParityControl [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Resolve a mark-parity target action, then optionally clear a large mark stack to damage
    /// every other living player in seat order.
    ///
    /// `stage`, `index`, `target`, and `targets` are engine-owned state-machine fields.
    /// Appended for serialized ordinal stability.
    ResolveMarkParityControl {
        /// Skill owner, chooser, and damage source.
        player: PlayerId,
        /// Mark definition controlling parity and the optional clear.
        mark: MarkDefId,
        /// Clear-all option is offered only above this count.
        mass_threshold: u32,
        /// Normal damage dealt by either damage branch.
        damage_amount: u32,
        /// 0 = choose target, 1 = parity action, 2 = discard card, 3 = clear option,
        /// 4 = mass damage.
        stage: u8,
        /// Next target index for mass damage.
        index: u32,
        /// Chosen parity-action target.
        target: PlayerId,
        /// Stable mass-damage target snapshot.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Discard all equipment from selected active-skill targets in seat order, then let the
    /// owner choose one surviving selected target to damage.
    ///
    /// `stage`, `index`, and `targets` are engine-owned state-machine fields. Appended for
    /// serialized ordinal stability.
    ResolveSelectedEquipmentDiscardThenChooseDamage {
        /// Skill owner, chooser, and damage source.
        player: PlayerId,
        /// Damage dealt to the final chosen target.
        damage_amount: u32,
        /// Damage nature tag.
        nature: u8,
        /// 0 = initialize, 1 = equipment discard, 2 = choose damage target.
        stage: u8,
        /// Next selected-target index.
        index: u32,
        /// Stable active-skill target snapshot.
        targets: Vec<PlayerId>,
        /// Originating active skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveYeyan [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Resolve the limited Yeyan choice between one-point fire damage and the four-suit
    /// three-point fire allocation.
    ///
    /// `mode`, `stage`, `index`, `targets`, and `data` are engine-owned state-machine fields.
    /// Appended for serialized ordinal stability.
    ResolveYeyan {
        /// Skill owner, chooser, and damage source.
        player: PlayerId,
        /// 0 = small fire, 1 = four-suit big fire, 255 = choose mode.
        mode: u8,
        /// 0 = initialize, 1 = choose cost/targets, 2 = choose big-fire targets,
        /// 3 = choose allocation, 4 = deal damage in seat order.
        stage: u8,
        /// Next target index for ordered damage.
        index: u32,
        /// Stable selected-target snapshot in seat order.
        targets: Vec<PlayerId>,
        /// Stage-specific scalar: single-target damage or boosted target raw id.
        data: u32,
        /// Originating active skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveQixing [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Initialize or exchange the current skill's public Qixing pile.
    ///
    /// `stage`, `hand_cards`, and `pile_cards` are engine-owned state-machine fields.
    /// Appended for serialized ordinal stability.
    ResolveQixing {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Whether stage zero first initializes seven draw-pile cards.
        initialize: bool,
        /// 0 = initialize/open hand choice, 1 = open hand choice, 2 = commit exchange.
        stage: u8,
        /// Current hand cards selected for exchange.
        hand_cards: Vec<CardId>,
        /// Current Qixing pile cards selected for exchange.
        pile_cards: Vec<CardId>,
        /// Originating Qixing skill instance.
        skill: SkillId,
    },
    /// Spend Qixing pile cards and grant a source-bound helper to equal-count living targets.
    ///
    /// `stage`, `cards`, and `targets` are engine-owned state-machine fields. Appended for
    /// serialized ordinal stability.
    ResolveQixingWeather {
        /// Skill owner, chooser, and weather source.
        player: PlayerId,
        /// Qixing definition owning the public pile.
        qixing_skill_def: SkillDefId,
        /// Target-owned weather helper definition.
        helper_skill_def: SkillDefId,
        /// Maximum number of pile cards and targets.
        max_targets: u8,
        /// 0 = choose cards, 1 = apply helpers after discard.
        stage: u8,
        /// Selected Qixing pile cards.
        cards: Vec<CardId>,
        /// Selected living targets.
        targets: Vec<PlayerId>,
        /// Originating weather skill instance.
        skill: SkillId,
    },
    /// Toggle Longnu, pay its branch cost, draw, then grant the matching phase helper.
    ///
    /// Appended for serialized ordinal stability.
    ResolveNzryLongnu {
        /// Skill owner.
        player: PlayerId,
        /// Red-card-to-fire-Sha helper definition.
        fire_skill_def: SkillDefId,
        /// Trick-card-to-thunder-Sha helper definition.
        thunder_skill_def: SkillDefId,
        /// 0 = toggle/pay cost, 1 = draw/grant helper.
        stage: u8,
        /// Originating Longnu skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveDrltPoxi [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose four distinct-suit cards across the owner and target hands, discard them, then
    /// resolve Poxi according to how many came from the owner.
    ///
    /// Appended for serialized ordinal stability.
    ResolveDrltPoxi {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Inspected active-skill target.
        target: PlayerId,
        /// Temporary hand-limit helper definition.
        hand_limit_skill_def: SkillDefId,
        /// 0 = choose cards, 1 = resolve the post-discard branch.
        stage: u8,
        /// Number of selected cards that belonged to the owner.
        own_count: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve one of Jieying's turn-begin, mark-transfer, or turn-end settlement branches.
    ///
    /// Appended for serialized ordinal stability.
    ResolveDrltJieying {
        /// Jieying owner.
        player: PlayerId,
        /// Player carried by the triggering event.
        event_player: PlayerId,
        /// Stable 营 mark definition.
        mark: MarkDefId,
        /// 0 = ensure mark, 1 = choose transfer, 2 = settle another marked player's turn.
        mode: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve Duorui's equipment-slot cost, skill borrowing, or expiry cleanup.
    ///
    /// Appended for serialized ordinal stability.
    ResolveDrltDuorui {
        /// Duorui owner for acquisition; helper owner for cleanup.
        player: PlayerId,
        /// Damaged target for acquisition; event player for cleanup.
        target: PlayerId,
        /// Target-side cleanup helper definition.
        helper_skill_def: SkillDefId,
        /// 0 = start, 1 = choose skill, 2 = acquire selected skill, 3 = cleanup.
        stage: u8,
        /// Selected target skill definition for stage 2.
        selected_skill_def: SkillDefId,
        /// Originating Duorui or helper skill instance.
        skill: SkillId,
    },
    /// Restore one disabled equipment slot when Zhiti's event condition is satisfied.
    ///
    /// Appended for serialized ordinal stability.
    ResolveDrltZhiti {
        /// Zhiti owner.
        player: PlayerId,
        /// Event player.
        event_player: PlayerId,
        /// Event source.
        event_source: PlayerId,
        /// Event scalar used to preserve a hand-comparison winner.
        event_amount: u32,
        /// 0 = hand comparison, 1 = Duel, 2 = received damage.
        mode: u8,
        /// Originating Zhiti skill instance.
        skill: SkillId,
    },
    /// Resolve Shen Simayi's shared mark, extra-turn, active, and draw branches.
    ///
    /// Appended for serialized ordinal stability.
    ResolveShenSimayi {
        /// Skill owner.
        player: PlayerId,
        /// Player carried by the triggering event.
        event_player: PlayerId,
        /// Scalar carried by the triggering event.
        event_amount: u32,
        /// Stable 忍 mark definition.
        mark: MarkDefId,
        /// 0=discard Renjie, 1=Lianpo, 2=Wansha, 3=Zhiheng, 4=Jizhi.
        mode: u8,
        /// Selected active cards or engine-owned continuation cards.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Selected legal targets for the chosen card at the final stage.
        #[serde(default)]
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve Cai Wenji's Beige cost, judgment, rewrite window, and final suit branch.
    ///
    /// Appended for serialized ordinal stability.
    ResolveBeige {
        /// Beige owner.
        player: PlayerId,
        /// Player damaged by Sha and judged by Beige.
        target: PlayerId,
        /// Damage source.
        source: PlayerId,
        /// 0=cost, 1=flip, 2=start rewrites, 3=apply suit branch, 4=continue rewrites.
        stage: u8,
        /// Current physical judgment result.
        #[serde(default = "no_card")]
        result_card: CardId,
        /// Current effective judgment suit.
        #[serde(default = "no_suit")]
        suit: Suit,
        /// Current physical judgment rank.
        #[serde(default)]
        rank: u8,
        /// Remaining judgment rewriters in seat order.
        #[serde(default)]
        rewriters: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveHiddenSkillPile [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Resolve a hidden skill-pile action shared by reveal, exchange, and matching-suit skills.
    ///
    /// Appended for serialized ordinal stability.
    ResolveHiddenSkillPile {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Definition that owns the expansion pile.
        pile_skill_def: SkillDefId,
        /// Stable action tag.
        action: u8,
        /// Current triggering card for target exclusion.
        card: CardId,
        /// Engine-owned continuation stage.
        #[serde(default)]
        stage: u8,
        /// Selected hand cards for an exchange.
        #[serde(default)]
        hand_cards: Vec<CardId>,
        /// Selected pile card(s).
        #[serde(default)]
        pile_cards: Vec<CardId>,
        /// Physical cards captured from the triggering movement event.
        #[serde(default)]
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveDiliYuheng [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Resolve EXTRA 帝力/驭衡 and its derived fate-line actions.
    ///
    /// Appended for serialized ordinal stability.
    ResolveDiliYuheng {
        /// Skill owner.
        player: PlayerId,
        /// Stable action tag.
        action: u8,
        /// Bound card for card-use or equipment actions.
        card: CardId,
        /// Physical cards captured from the triggering movement event.
        #[serde(default)]
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveDrltJueyan [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Resolve 决堰's equipment-slot choice and branch benefit.
    ResolveDrltJueyan {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Stable selected slot group (`1`, `2`, `3`, or `5`).
        slot: u8,
        /// Whether this is the first activation in the match.
        first: bool,
        /// Permanent +1 Sha-use helper.
        sha_skill_def: SkillDefId,
        /// Temporary +3 Sha-use helper.
        weapon_skill_def: SkillDefId,
        /// Unlimited-distance helper.
        mount_skill_def: SkillDefId,
        /// Hand-limit +3 helper.
        armor_skill_def: SkillDefId,
        /// 界集智 helper.
        treasure_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveGanglieThree [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Resolve 国战三版刚烈's enemy choice and judgment.
    ResolveGanglieThree {
        /// Skill owner.
        player: PlayerId,
        /// Selected enemy, or `NONE` before target choice.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 忠义 activation, damage bonus, or next-round cleanup.
    ResolveZhongyi {
        /// Skill owner.
        player: PlayerId,
        /// Stable action tag: 0=activate, 1=increase bound Sha damage, 2=cleanup.
        action: u8,
        /// Temporary helper definition.
        helper_skill_def: SkillDefId,
        /// Selected activation card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveTianxiang [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 天香（old/shenhua 小乔）：弃一张红桃手牌、选择一名其他角色，将当前伤害
    /// 重定向（取消后按同源/同量/同属性/同牌重新结算）并为其附加摸牌技能。
    ///
    /// VM 首发（`target == NONE`）时由 vm_host 从 damage_before 上下文捕获
    /// source/amount/nature/card；选择结算后经 `ResolveTianxiang` 重发应用。
    /// ABI 456（C 段 omp）。
    ResolveTianxiang {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Selected redirect target, or `NONE` before the target choice.
        target: PlayerId,
        /// Heart hand card discarded as the cost, or `NONE` before the card choice.
        discard_card: CardId,
        /// Current damage source (host-captured).
        source: PlayerId,
        /// Current damage amount (host-captured).
        amount: u32,
        /// Current damage nature (host-captured).
        nature: u8,
        /// Current damage card (host-captured).
        card: CardId,
        /// Current logical card kind (host-captured).
        card_kind: u8,
        /// Current card provenance flags (host-captured).
        card_flags: u8,
        /// Skill granted to the redirect target after the damage settles.
        draw_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveShuangxiong [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 双雄（shenhua 颜良文丑）：摸牌阶段可选改为判定，获得判定牌并记录判定颜色，
    /// 本回合非该颜色手牌可当【决斗】使用。ABI 511。
    ResolveShuangxiong {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 直谏（shenhua 张昭张纮）：出牌阶段限一次，将一张手牌区装备牌交给一名可装备
    /// 该牌的其他角色，然后摸一张。ABI 512。
    ResolveZhijian {
        /// Skill owner and giver.
        player: PlayerId,
        /// Exact selected equipment card from hand.
        card: CardId,
        /// Choice barrier stage: 0 choose target, 1 commit give.
        stage: u8,
        /// Chosen recipient.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 固政（shenhua 张昭张纮）：其他角色弃牌阶段结束后，归还其一张某阶段弃置且仍处
    /// 弃牌堆的牌，并可选获得其余弃牌。ABI 513。
    ResolveGuzheng {
        /// Skill owner.
        player: PlayerId,
        /// The player whose discard phase ended.
        event_player: PlayerId,
        /// Choice barrier stage: 0 choose return card, 1 choose take-rest.
        stage: u8,
        /// Snapshot of the discard-phase lost cards (player id + card id pairs).
        cards: Vec<u32>,
        /// Card chosen to return.
        return_card: CardId,
        /// Whether the owner takes the remaining phase discards.
        take_rest: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveDrltCongjian [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 从俭（shenhua 张绣）：成为多目标锦囊牌的目标后，交给其中一名其他目标一张
    /// 手牌/装备牌，然后摸一张（所给为装备牌则摸两张）。ABI 524。
    ResolveDrltCongjian {
        /// Skill owner.
        player: PlayerId,
        /// The trick's targets captured at trigger time.
        targets: Vec<PlayerId>,
        /// Card chosen to give.
        card: CardId,
        /// Chosen recipient.
        target: PlayerId,
        /// Choice barrier stage: 0 choose recipient, 1 choose card, 2 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 飞军（shenhua 王平）：出牌阶段限一次弃一张牌，按可用性选“手牌数大于你者交给
    /// 你一张牌”或“装备数大于你者弃置一张装备牌”分支；目标首次被选时置兵略旗标。
    /// ABI 526。
    ResolveNzryFeijun {
        /// Skill owner.
        player: PlayerId,
        /// Chosen target.
        target: PlayerId,
        /// Card the target chose to give or discard.
        card: CardId,
        /// Branch: 0 = hand-give, 1 = equip-discard.
        branch: u8,
        /// Choice barrier stage: 0 branch, 1 target, 2 target card, 3 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 庸肆（shenhua 袁术）：摸牌阶段摸牌数改为势力数；出牌阶段结束时按本回合是否
    /// 造成伤害：未造成伤害摸至体力值，否则授予既有庸肆弃牌阶段状态。ABI 527。
    ResolveDrltYongsi {
        /// Skill owner.
        player: PlayerId,
        /// Zero = draw-phase replacement, one = play-phase-end settlement.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 伪帝（shenhua 袁术，主公技）：弃牌阶段开始把至多 min(需弃置数, 其他群势力
    /// 角色数) 张手牌分别交给等量其他群势力角色。ABI 528。
    ResolveDrltWeidi {
        /// Skill owner.
        player: PlayerId,
        /// Cards chosen to give.
        cards: Vec<CardId>,
        /// Chosen recipients, paired with cards by index.
        targets: Vec<PlayerId>,
        /// Choice barrier stage: 0 cards, 1 targets, 2 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveLieren [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 烈刃（shenhua 祝融）：以固定受伤害角色为目标启动通用拼点选牌续体；胜者获得
    /// 绑定辅助技能。ABI 531。
    ResolveLieren {
        /// Skill owner and comparing player.
        player: PlayerId,
        /// The damaged player and compare target.
        target: PlayerId,
        /// Helper granted on a strict win.
        win_skill_def: SkillDefId,
        /// Helper granted on a tie or loss.
        lose_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveTuntian [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 屯田（shenhua 邓艾）：回合外失去牌后判定；判定牌不为红桃时移入本技能
    /// 牌堆（田）。ABI 533。
    ResolveTuntian {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 凿险（shenhua 邓艾）：准备阶段本技能牌堆至少 3 张时觉醒，减 1 点体力
    /// 上限并授予急袭。ABI 534。
    ResolveZaoxian {
        /// Skill owner.
        player: PlayerId,
        /// 急袭 definition granted on awakening.
        jixi_def: SkillDefId,
        /// 屯田 definition whose skill pile stores the 田 cards.
        tuntian_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 挑衅（shenhua 姜维）：出牌阶段限一次，选择一名攻击范围含持有者的其他角色，
    /// 其选择对持有者使用一张杀，或令持有者弃置其一张牌。ABI 537。
    ResolveTiaoxin {
        /// Skill owner.
        player: PlayerId,
        /// The challenged player; NONE while opening the target choice.
        target: PlayerId,
        /// Choice barrier stage: 0 open target, 1 open sha-or-refuse, 2 open sha source,
        /// 3 play the sha, 4 open discard card, 5 commit the discard.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 志继（shenhua 姜维）：觉醒技，准备阶段无手牌时摸两张牌、回复 1 点体力、
    /// 减 1 点体力上限并获得观星。ABI 538。
    ResolveZhiji {
        /// Skill owner.
        player: PlayerId,
        /// 观星 definition granted on awakening.
        guanxing_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 再起（shenhua 孟获）：摸牌阶段替换——已受伤时弃摸牌并翻亮牌堆顶 X 张
    /// （X=已损失体力+extra），红桃每张回复 1 点体力，其余获得。ABI 540。
    ResolveZaiqi {
        /// Skill owner.
        player: PlayerId,
        /// Extra revealed cards beyond the lost HP count.
        extra: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveHuangtian [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 黄天（shenhua 张角，主公技）：其他群势力角色出牌阶段限一次可将一张闪/闪电
    /// 交给持有者。ABI 551。
    ResolveHuangtian {
        /// Skill owner (lord).
        player: PlayerId,
        /// The giving Qun player.
        actor: PlayerId,
        /// Choice barrier stage: 0 open card choice, 1 commit the give.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveHaoshi [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 好施（shenhua 鲁肃）：摸牌阶段额外摸两张；摸牌阶段结束时手牌>5 则选手牌数
    /// 最少角色给 floor(手牌/2) 张。ABI 555。
    ResolveHaoshi {
        /// Skill owner.
        player: PlayerId,
        /// Choice barrier stage: 0 draw replacement, 1 open target, 2 open cards, 3 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 缔盟（shenhua 鲁肃）：出牌阶段限一次，选两名其他角色，弃置 X 张牌
    /// （X=两者手牌差）后交换其手牌。ABI 556。
    ResolveDimeng {
        /// Skill owner.
        player: PlayerId,
        /// The two chosen targets; NONE while opening the target choice.
        targets: Vec<PlayerId>,
        /// Choice barrier stage: 0 open targets, 1 open discard, 2 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveQiaobian [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 巧变（shenhua 张郃）：阶段开始时弃一张手牌并跳过该阶段；摸牌阶段可
    /// 获至多两名其他角色各一张手牌；出牌阶段可移动场上的一张牌。ABI 559。
    ResolveQiaobian {
        /// Skill owner.
        player: PlayerId,
        /// Triggered phase: 0 judge, 1 draw, 2 play, 3 discard.
        phase: u8,
        /// Choice barrier stage (0 discard, 1 benefits/skip, 2+ benefit flow).
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveXiangle [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 享乐（shenhua 刘禅）：使用者弃一张基本牌或此杀对持有者无效。ABI 560。
    ResolveXiangle {
        /// Skill owner (the Sha target).
        player: PlayerId,
        /// The Sha user.
        source: PlayerId,
        /// The Sha card.
        card: CardId,
        /// Choice barrier stage: 0 open discard, 1 commit/exclude.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 放权（shenhua 刘禅）：跳过本回合出牌阶段并标记结束阶段可授额外回合。
    /// ABI 560。
    ResolveFangquanSkip {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 放权（shenhua 刘禅）：结束阶段弃一张手牌并令一名其他角色进行一个额外回合。
    /// ABI 560。
    ResolveFangquanGrant {
        /// Skill owner.
        player: PlayerId,
        /// Choice barrier stage: 0 open discard, 1 open target, 2 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 若愚（shenhua 刘禅，主公技）：准备阶段体力值为全场最少时觉醒：加上限、
    /// 回复 1 并授予激将。ABI 560。
    ResolveRuoyu {
        /// Skill owner.
        player: PlayerId,
        /// 激将 granted skill definition.
        jijiang_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
}
pub(super) use variants;
