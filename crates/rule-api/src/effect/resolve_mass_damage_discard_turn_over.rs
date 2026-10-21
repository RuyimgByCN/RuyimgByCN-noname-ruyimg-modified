//! Ordered EffectRequest variants, starting with `ResolveMassDamageDiscardTurnOver`.

macro_rules! variants {
    ([$($tail:ident),*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail),*] [
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
    /// Record one selected target for an active skill declaring
    /// `unused_by_skill_this_turn`.
    ///
    /// Appended for serialized ordinal stability.
    RecordActiveTarget {
        /// Active-skill owner.
        player: PlayerId,
        /// Selected target to reserve for the current turn.
        target: PlayerId,
        /// Originating active skill instance.
        skill: SkillId,
    },
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
    /// Replace the pending normal draw with a public top-card reveal and an exact one-per-suit
    /// selection.
    ///
    /// `stage` and `cards` are engine-owned state-machine fields. Appended for serialized
    /// ordinal stability.
    ReplaceDrawWithDistinctSuitSelection {
        /// Draw-phase player and card recipient.
        player: PlayerId,
        /// Number of top cards revealed.
        count: u8,
        /// 0 = initialize, 1 = choose from the stable processing-zone cards.
        stage: u8,
        /// Stable revealed-card snapshot.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Inspect another player's hand, choose one heart card, then discard it or put it on top of
    /// the draw pile.
    ///
    /// `stage` and `card` are engine-owned state-machine fields. Appended for serialized ordinal
    /// stability.
    InspectTargetHandChooseHeartDiscardOrTop {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Inspected hand owner.
        target: PlayerId,
        /// 0 = reveal/choose heart, 1 = choose destination.
        stage: u8,
        /// Chosen heart card.
        card: CardId,
        /// Originating active skill instance.
        skill: SkillId,
    },
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
    /// Let Qinyin's owner cancel or make every currently living player lose or recover one HP.
    ///
    /// Appended for serialized ordinal stability.
    ChooseAllLoseHpOrRecover {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating trigger skill instance.
        skill: SkillId,
    },
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
    /// Force the skill owner to choose and link another currently unlinked living player.
    ///
    /// Appended for serialized ordinal stability.
    ChooseLinkOther {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Choose and pay either one discardable physical weapon from hand/equipment or one HP.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDiscardWeaponOrLoseHp {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Deal one ordinary self-damage, then discard the player's current judgment zone after the
    /// complete damage, dying, and save chain.
    ///
    /// Appended for serialized ordinal stability.
    DamageThenDiscardAllJudge {
        /// Player damaged and whose judgment zone is eventually emptied.
        player: PlayerId,
        /// Engine-owned continuation stage. Content emits false.
        #[serde(default)]
        ready_to_discard: bool,
    },
    /// Replace normal draw with two public top cards, gain them, then grant two temporary skills
    /// when their effective colors differ.
    ///
    /// Appended for serialized ordinal stability.
    ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
        /// Draw-phase player and card recipient.
        player: PlayerId,
        /// First temporary skill definition.
        first_skill_def: SkillDefId,
        /// Second temporary skill definition.
        second_skill_def: SkillDefId,
        /// 0 = reveal, 1 = gain, 2 = grant skills.
        #[serde(default)]
        stage: u8,
        /// Stable revealed-card snapshot.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose and gain one target hand/equipment card from a Play-phase zone not yet used by
    /// this skill, then let that target draw one card.
    ///
    /// `selected = NONE` opens the card choice; a concrete card resumes it. Appended for
    /// serialized ordinal stability.
    ChooseGainTargetCardAlternateZoneThenTargetDraw {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other living player losing one card, then drawing.
        target: PlayerId,
        /// Chosen current target card, or NONE before the choice.
        selected: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Mark a living player's next judgment phase to be skipped.
    ///
    /// Appended for serialized ordinal stability.
    SkipNextJudgePhase {
        /// Player whose next judgment phase is skipped.
        player: PlayerId,
    },
    /// Choose and discard one hand/equipment card from either endpoint of the current damage,
    /// then increase that damage if the card's owner-effective suit matches.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDiscardDamageEndpointCardIncreaseIfSuit {
        /// Damage source, skill owner, and chooser.
        player: PlayerId,
        /// Current damage target.
        target: PlayerId,
        /// Bound pre-resolution damage id. Content emits zero; the VM host binds it.
        damage_id: u32,
        /// Stable suit tag (`1` spade through `4` diamond).
        suit: u8,
        /// Damage added after a matching discard.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose and gain one hand card from a player supplied by the triggering event.
    ///
    /// `from = NONE` is bound by the VM host to the first event target.
    /// Appended for serialized ordinal stability.
    TakeOneHandCard {
        /// Card owner whose hand card is taken.
        from: PlayerId,
        /// Recipient of the taken card.
        to: PlayerId,
    },
    /// 选目标一张手牌置入拥有者技能牌堆（ABI 567,储元 chuyuan 等）。
    ///
    /// Appended to keep existing serialized variant ordinals stable.
    ChooseOtherHandStoreSkillPile {
        /// Skill owner whose skill pile receives the card.
        player: PlayerId,
        /// Player whose hand card is chosen.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 获得本技能牌堆全部牌（移入手牌,ABI 569,登极 dengji 觉醒取回等）。
    ///
    /// Appended to keep existing serialized variant ordinals stable.
    GainAllSkillPileCards {
        /// Skill owner whose skill pile is emptied into hand.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 获得其他角色指定技能牌堆的全部牌（移入自己手牌）。ABI 600,英逝 yingshi_die。
    ///
    /// Appended to keep existing serialized variant ordinals stable.
    GainOtherSkillPileCards {
        /// Player whose hand receives the pile cards.
        player: PlayerId,
        /// Player whose skill pile (keyed by `skill_def`) is emptied.
        target: PlayerId,
        /// Skill definition id keying the source skill pile.
        skill_def: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 弃置一张手牌/装备牌或失去等量体力。ABI 601,庸肆 xinyongsi2。
    ///
    /// Appended to keep existing serialized variant ordinals stable.
    ChooseDiscardOneHeOrLoseHp {
        /// Player making the discard-or-lose choice.
        player: PlayerId,
        /// Amount of HP lost if the player declines to discard.
        lose_amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 目标判定：黑桃 2-9 则其摸两张牌，否则对其使用两张雷【杀】。ABI 602,雷劫 releijie。
    ///
    /// Appended to keep existing serialized variant ordinals stable.
    JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
        /// Skill owner who uses the virtual thunder Sha.
        source: PlayerId,
        /// Target who is judged, draws, or receives the thunder Sha.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 死亡来源选择是否令拥有者失去体力。ABI 603,随势 suishi2。
    ///
    /// Appended to keep existing serialized variant ordinals stable.
    ChooseSourceConfirmOwnerLoseHp {
        /// Owner who loses HP when the source confirms.
        owner: PlayerId,
        /// Player (e.g. the death source) who confirms the HP loss.
        source: PlayerId,
        /// Amount of HP the owner loses on confirmation.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// At the current turn's completed End phase, choose an extra Draw or Play phase, or
    /// execute both when the distinct suits of cards used this turn reach current HP.
    ///
    /// Appended for serialized ordinal stability.
    ChooseEndExtraDrawOrPlayByUsedSuits {
        /// Current turn player and skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Lose HP equal to the bound damage, double that damage, then permanently replace the
    /// originating skill after the HP-loss resolution finishes.
    ///
    /// Appended for serialized ordinal stability.
    LoseHpDoubleCurrentDamageThenReplace {
        /// Damage source, HP-loss target, and skill owner.
        player: PlayerId,
        /// Bound pre-resolution damage id.
        damage_id: u32,
        /// Original damage amount used by the HP-loss cost and initial damage increase.
        amount: u32,
        /// Originating skill instance to replace.
        skill: SkillId,
        /// Permanent replacement skill definition.
        replacement_skill_def: SkillDefId,
        /// Internal second stage, executed after the HP-loss resolution.
        ready_to_replace: bool,
    },
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
    /// Enable or clear the originating skill as the current turn's non-locked-skill suppression
    /// source.
    ///
    /// Appended for serialized ordinal stability.
    SetNonLockedSkillSuppression {
        /// Skill owner and current turn player.
        player: PlayerId,
        /// Whether to enable or clear suppression.
        enabled: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Finish 调剂盐梅 after its forced discard and any resulting triggers.
    ///
    /// Appended for serialized ordinal stability.
    FinalizeTiaojiyanmei {
        /// Card user.
        actor: PlayerId,
        /// First original target.
        left: PlayerId,
        /// Second original target.
        right: PlayerId,
        /// Card discarded by the target above the original average.
        card: CardId,
    },
    /// Open 调剂盐梅's forced discard after the lower-hand target's draw and triggers finish.
    ///
    /// Appended for serialized ordinal stability.
    BeginTiaojiyanmeiDiscard {
        /// Card user.
        actor: PlayerId,
        /// First original target.
        left: PlayerId,
        /// Second original target.
        right: PlayerId,
        /// Target whose original hand count was above the average.
        higher: PlayerId,
    },
    /// Discard selected hand/equipment cards and draw the same count, plus one when all current
    /// hand cards were selected.
    ///
    /// Appended for serialized ordinal stability.
    DiscardSelectedDrawAllHandBonus {
        /// Skill owner and draw recipient.
        player: PlayerId,
        /// Cards selected by the active invocation.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 国战英魂: choose another player, choose one lost-HP-scaled draw/discard branch,
    /// then perform its forced discard after draw triggers finish.
    ///
    /// Appended for serialized ordinal stability.
    ChooseOtherYinghun {
        /// Wounded skill owner and chooser.
        player: PlayerId,
        /// Selected other player, or `NONE` before target choice.
        target: PlayerId,
        /// Lost HP captured when target choice opens.
        count: u32,
        /// Branch tag: zero before choice, one for draw-X/discard-one, two for
        /// draw-one/discard-X.
        branch: u8,
        /// Whether the selected target's draw and nested triggers have completed.
        ready_to_discard: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 弼政: choose another player to draw two, then sequentially make each participant
    /// above their own maximum HP discard up to two hand/equipment cards.
    ///
    /// Appended for serialized ordinal stability.
    ChooseOtherBizheng {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Selected other player, or `NONE` before target choice.
        target: PlayerId,
        /// Engine-owned stage: zero chooses, one draws, two checks owner, three checks target.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 霞姝: give all hand cards to another player, let that player reveal a nonempty
    /// subset, then let the owner gain the revealed or unrevealed partition.
    ///
    /// Appended for serialized ordinal stability.
    ChooseOtherXiashu {
        /// Skill owner and target chooser.
        player: PlayerId,
        /// Selected other player, or `NONE` before target choice.
        target: PlayerId,
        /// Engine-owned stage: zero chooses target, one gives, two chooses reveal, three chooses
        /// partition.
        stage: u8,
        /// Revealed hand cards captured for the partition choice.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve one 观潮 card rank against the previous Play-phase card, drawing on a valid
    /// selected direction or invalidating that direction after a violation.
    ///
    /// Appended for serialized ordinal stability.
    ResolveGuanchaoRank {
        /// Skill owner and card user.
        player: PlayerId,
        /// Current used physical or materialized card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 观微: discard one hand/equipment card, let the completed Play-phase player draw
    /// two, then insert one immediate extra Play phase for that player.
    ///
    /// Appended for serialized ordinal stability.
    ResolveGuanwei {
        /// Skill owner and discard chooser.
        player: PlayerId,
        /// Player whose Play phase just ended.
        target: PlayerId,
        /// Engine-owned stage: zero chooses discard, one discards, two draws, three inserts.
        stage: u8,
        /// Selected discard card after the choice.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 揽江's snapshotted seat-order draw offers, optional equal-hand damage, and
    /// post-damage forced draw.
    ///
    /// Appended for serialized ordinal stability.
    ResolveLanjiang {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Current seat-order snapshot index.
        index: u32,
        /// Selected target for the current stage, or `NONE`.
        target: PlayerId,
        /// Players whose hand count met the entry threshold, in seat order.
        #[serde(default)]
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Reveal 秉壹 owner's full hand, choose up to hand-count living players to draw when all
    /// cards share a color, and additionally draw for the owner when all ranks also match.
    ///
    /// Appended for serialized ordinal stability.
    ResolveRebingyi {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Record one 博图 use for the current round and queue an extra turn.
    ///
    /// Appended for serialized ordinal stability.
    ResolveRebotu {
        /// Skill owner and extra-turn recipient.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 旧缔盟's legal pair choice and atomic hand swap, or its Play-end forced discard.
    ///
    /// Appended for serialized ordinal stability.
    ResolveOldimeng {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned stage: zero chooses first, one chooses second/swaps, two settles discard.
        stage: u8,
        /// First selected other player.
        first: PlayerId,
        /// Second selected other player.
        second: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 界结姻's selected-card target/placement choice and ordered HP comparison benefit.
    ResolveRejieyin {
        /// Skill owner.
        player: PlayerId,
        /// Selected male target, or `NONE` before target choice.
        target: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Selected hand/equipment card.
        card: CardId,
        /// Whether the selected card should enter the target's equipment zone.
        equip: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 仁恤's two-target card transfer and ordered draw checks.
    ResolveReanxu {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Player losing the selected card.
        first: PlayerId,
        /// Player gaining the selected card.
        second: PlayerId,
        /// Selected card.
        card: CardId,
        /// Whether the selected card came from the first target's hand.
        from_hand: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Resolve 界集智's draw and optional discard of the drawn basic card.
    ResolveRejizhi {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Drawn basic card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 诫训's target choice, snapshotted draw/discard counts, and terminal replacement.
    ResolveJiexun {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Chosen other player.
        target: PlayerId,
        /// Snapshotted diamond count.
        draw_count: u32,
        /// Snapshotted existing 诫训 mark count.
        discard_count: u32,
        /// 复难 enhancement helper.
        funan_helper_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Grant 问卦's global active helper to every living player.
    ResolveWenguaSetup {
        /// 问卦 owner.
        player: PlayerId,
        /// Active helper definition.
        helper_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve one 问卦 active-helper invocation.
    ResolveWengua {
        /// Active actor.
        player: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Chosen 问卦 owner.
        target: PlayerId,
        /// Selected hand/equipment card.
        card: CardId,
        /// Top (`0`) or bottom (`1`) branch.
        branch: u8,
        /// 问卦 main definition.
        main_skill_def: SkillDefId,
        /// Per-Play-phase recipient marker.
        marker_skill_def: SkillDefId,
        /// Originating helper skill instance.
        skill: SkillId,
    },
    /// Resolve 新安国's ordered minimum-hand/HP/equipment actions.
    ResolveXinanguo {
        /// Skill owner.
        player: PlayerId,
        /// Selected other target.
        target: PlayerId,
        /// Engine-owned stage (`0` start, `1` target, `2` owner).
        stage: u8,
        /// Current action index (`0` hand, `1` HP, `2` equipment).
        index: u8,
        /// Actions not already performed for the target.
        remaining: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Discard every equipped card in one slot.
    DiscardEquipmentSlot {
        /// Equipment owner.
        player: PlayerId,
        /// Stable equipment-slot tag.
        slot: u8,
    },
    /// Resolve old 安恤's unequal-hand two-target transfer.
    ResolveOldAnxu {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Player with more hand cards.
        giver: PlayerId,
        /// Player with fewer hand cards.
        receiver: PlayerId,
        /// Selected hand card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Move active invocation cards into the originating skill's expansion pile.
    StoreSelectedInSkillPile {
        /// Skill owner.
        player: PlayerId,
        /// Selected hand cards.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// End an identity match for the player's camp when a skill pile reaches a threshold.
    EndMatchForCampIfSkillPileAtLeast {
        /// Camp representative.
        player: PlayerId,
        /// Definition owning the checked expansion pile.
        pile_skill_def: SkillDefId,
        /// Minimum pile size.
        count: u32,
    },
    /// Resolve old 寝情's target discard/draw sequence.
    ResolveOldQinqing {
        /// Skill owner.
        player: PlayerId,
        /// Selected target.
        target: PlayerId,
        /// Current lord.
        lord: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve old 明鉴's hand transfer and inserted play-only phase.
    ResolveOldMingjian {
        /// Skill owner.
        player: PlayerId,
        /// Selected recipient.
        target: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve old 旋风's virtual-Sha or distance-one damage branch.
    ResolveOldXuanfeng {
        /// Skill owner.
        player: PlayerId,
        /// Selected target.
        target: PlayerId,
        /// Materialized virtual Sha.
        card: CardId,
        /// Engine-owned stage.
        stage: u8,
        /// Branch tag: 1 virtual Sha, 2 direct damage.
        branch: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Discard two cards to prevent the bound damage for 剑柔.
    ResolveJianrou {
        /// Damaged skill owner.
        player: PlayerId,
        /// Engine-owned pre-resolution damage id.
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve old 贿生's reveal, source choice, and bound damage prevention.
    ResolveOldHuisheng {
        /// Damaged skill owner.
        player: PlayerId,
        /// Other living damage source.
        source: PlayerId,
        /// Engine-owned pre-resolution damage id.
        damage_id: u32,
        /// Engine-owned stage.
        stage: u8,
        /// Cards selected by the owner.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve old 解烦's prompted Sha and its bound damage replacement.
    ResolveOldJiefan {
        /// Skill owner and Sha user.
        player: PlayerId,
        /// Player being saved.
        dying: PlayerId,
        /// Current turn player who must be included as a Sha target.
        current: PlayerId,
        /// Selected physical or view-as Sha source.
        card: CardId,
        /// Engine-owned pre-resolution damage id.
        damage_id: u32,
        /// Engine-owned stage: 0 choose Sha, 1 use Sha, 2 replace its damage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 彻骨's target cancellation, branch, ordered target discards, and turn bonus.
    ResolveChegu {
        /// Skill owner and card user.
        player: PlayerId,
        /// Triggering physical card.
        card: CardId,
        /// Original card targets at stage zero; selected discard targets afterward.
        targets: Vec<PlayerId>,
        /// Cards discarded by earlier targets.
        cards: Vec<CardId>,
        /// Branch: 1 one target/many cards, 2 many targets/one card.
        branch: u8,
        /// Next selected target index.
        index: u8,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 归心's lord-skill acquisition or faction change.
    ResolveJunkguixin {
        /// Skill owner.
        player: PlayerId,
        /// Selected faction-change target.
        target: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Selected faction code at the commit stage.
        group: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 谶纬's alternating face-down-card branches.
    ResolveYaoChenwei {
        /// Skill owner.
        player: PlayerId,
        /// Selected player.
        target: PlayerId,
        /// Selected card.
        card: CardId,
        /// Conversion mode: 0 toggle owner's card, 1 gain and turn face-down.
        mode: u8,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let the current Sha user optionally add one legal target within a final-distance cap.
    ///
    /// Appended for serialized ordinal stability.
    ChooseAddCurrentShaTargetWithinDistance {
        /// Sha user and chooser.
        player: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Inclusive final-distance cap.
        max_distance: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Refund the currently saved proactive Sha's ordinary use count at most once.
    ///
    /// Appended for serialized ordinal stability.
    RefundCurrentShaUse {
        /// Current Sha user.
        player: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Damage each selected active-skill target, then let that target draw, in seat order.
    ///
    /// Appended for serialized ordinal stability.
    DamageThenDrawSelectedTargets {
        /// Damage source.
        source: PlayerId,
        /// Active-skill targets.
        targets: Vec<PlayerId>,
        /// Damage dealt to each target.
        damage: u32,
        /// Cards drawn by each target afterward.
        draw: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 游敌's target-owned discard and conditional counter-gain.
    ///
    /// Appended for serialized ordinal stability.
    ResolveYoudi {
        /// Skill owner.
        player: PlayerId,
        /// Selected other player.
        target: PlayerId,
        /// Selected card for the current stage.
        card: CardId,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 狂斧's selected equipment and move/discard branch.
    ///
    /// Appended for serialized ordinal stability.
    ResolveKuangfu {
        /// Skill owner.
        player: PlayerId,
        /// Damaged target.
        target: PlayerId,
        /// Selected equipment card.
        card: CardId,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 缮甲's draw, forced discard, and optional virtual Sha.
    ///
    /// Appended for serialized ordinal stability.
    ResolveShanjia {
        /// Skill owner.
        player: PlayerId,
        /// Draw/discard count, capped by the engine at seven.
        count: u32,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 新涯角's reveal, recipient, and category-mismatch discard.
    ///
    /// Appended for serialized ordinal stability.
    ResolveNewYajiao {
        /// Skill owner.
        player: PlayerId,
        /// Card just used or responded with.
        source_card: CardId,
        /// Revealed draw-pile card.
        revealed: CardId,
        /// Selected recipient.
        target: PlayerId,
        /// Snapshot of the source card's basic/trick/equipment category.
        source_category: u8,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 定品's judgment and color-dependent result.
    ///
    /// Appended for serialized ordinal stability.
    ResolveDingpin {
        /// Skill owner.
        player: PlayerId,
        /// Selected judging target.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Discard the shared processing area, abort the remaining current resolution, and finish the
    /// current phase without recording it as skipped.
    ///
    /// Appended for serialized ordinal stability.
    FinishCurrentPhaseAbortResolution {
        /// Skill owner that requested the abort.
        player: PlayerId,
        /// Engine-owned stage: 0 abort/discard, 1 finish the phase after move triggers.
        stage: u8,
    },
    /// Move the draw-pile top card into the originating skill's public pile.
    GainTopCardToSkillPile {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose and discard one card from the originating skill's public pile, then add one mark of
    /// a helper skill to the target.
    ChooseDiscardSkillPileCardAddMarkedSkill {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player receiving the helper skill.
        target: PlayerId,
        /// Helper skill definition and mark id.
        helper_skill_def: SkillDefId,
        /// Selected pile card, or `NONE` before choice.
        selected: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Gain one current hand card from each active-skill target in submitted order.
    GainOneHandFromSelectedTargets {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Active-skill targets in submitted order.
        targets: Vec<PlayerId>,
        /// Current target index.
        index: u32,
        /// Selected hand card, or `NONE` before choice.
        selected: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Optionally choose living 破虏 recipients, increment its mark once, then draw in seat order.
    ///
    /// The chooser may be dead because one authoritative trigger fires on the owner's death.
    /// Appended for serialized ordinal stability.
    ChooseRepoluTargetsDraw {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance; its definition is also the stable mark id.
        skill: SkillId,
    },
    /// Resolve 过论's reveal, optional exchange, and lower-rank draw.
    ///
    /// `player_card` and `target_card` are `NONE` before the first choice and both populated
    /// only for the atomic exchange. Appended for serialized ordinal stability.
    ResolveXinfuGuolun {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player whose hand card is selected.
        target: PlayerId,
        /// Owner's selected hand/equipment card.
        player_card: CardId,
        /// Target's selected hand card.
        target_card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Force one hand/equipment-card discard, refund the currently resolving proactive Sha's
    /// use count at most once, draw for the skill owner if that Sha is effectively red, and
    /// restore one skill definition when the paid card and Sha have equal effective ranks.
    ///
    /// This is a distinct appended request so the original Longyin request and its serialized
    /// shape remain stable.
    ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
        /// Skill owner choosing and paying the discard cost.
        player: PlayerId,
        /// Player who used the currently resolving Sha.
        sha_user: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Skill definition restored only when both effective ranks are equal.
        restore_skill_def: SkillDefId,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// Resolve Beige's existing cost, judgment, and rewrite pipeline with content-supplied
    /// Heart recovery and Diamond draw counts.
    ///
    /// This is a distinct appended request so the original `ResolveBeige` serialized shape and
    /// stable hash remain unchanged.
    ResolveBeigeConfigured {
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
        /// Amount recovered by the damaged player on a Heart result.
        heart_recover_amount: u32,
        /// Number of cards drawn by the damaged player on a Diamond result.
        diamond_draw_count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let a dead 追忆 owner optionally choose one living recipient other than the killer,
    /// then recover that recipient and draw cards in the continuation.
    ///
    /// Appended for serialized ordinal stability.
    ResolveZhuiyi {
        /// Dead skill owner and choice observer.
        player: PlayerId,
        /// Credited killer, or `NONE` for a source-free death.
        killer: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 界镇军 through an independent state machine while preserving legacy `rezhenjun`.
    ///
    /// Only stage zero is constructible by Rule VM content. Stage one is an engine-owned
    /// continuation after the selected target cards and their nested triggers have resolved.
    /// Appended for serialized ordinal stability.
    ResolveDecadezhenjun {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Selected target, or `NONE` before target selection.
        #[serde(default = "no_player")]
        target: PlayerId,
        /// 0=choose target/cards, 1=offer one owner discard or let the target draw.
        #[serde(default)]
        stage: u8,
        /// Number of all-non-equipment target cards discarded; also the fallback draw count.
        #[serde(default)]
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve SP 骁果 through an independent state machine while preserving Standard 骁果.
    ///
    /// Only stage zero is constructible by Rule VM content. Stages one through three are
    /// engine-owned continuations that run after the preceding discard and all nested
    /// card-movement triggers have resolved.
    /// Appended for serialized ordinal stability.
    ResolveSpXiaoguo {
        /// Skill owner.
        player: PlayerId,
        /// End-phase player.
        target: PlayerId,
        /// 0=choose owner cost, 1=offer target equipment, 2=damage fallback, 3=owner draw.
        #[serde(default)]
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Replace the pending normal draw with choosing and gaining one card from every other
    /// living player's hand/equipment/judgment zones in clockwise order.
    ///
    /// Only the initial empty target list is constructible by Rule VM content. The engine owns
    /// the stable remaining-target snapshot across nested card-movement triggers.
    /// Appended for serialized ordinal stability.
    ChooseGainCardFromEachOtherReplaceDraw {
        /// Skill owner, chooser, and card recipient.
        player: PlayerId,
        /// Whether the stable target snapshot has been initialized.
        #[serde(default)]
        initialized: bool,
        /// Engine-owned remaining targets in owner-relative clockwise order.
        #[serde(default)]
        remaining_targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Give selected hand cards in one batch, then recover the recipient after every movement
    /// event and nested trigger has completed.
    ///
    /// Appended for serialized ordinal stability.
    GiveSelectedThenRecoverTarget {
        /// Skill owner and giver.
        player: PlayerId,
        /// Gift recipient and recovery target.
        target: PlayerId,
        /// Selected hand cards.
        cards: Vec<CardId>,
        /// Recovery amount.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
        /// Engine-owned stage flag set after the gift batch has moved.
        #[serde(default)]
        ready_to_recover: bool,
    },
    /// Resolve SP 定叛's draw, target branch, equipment movement, and delayed damage.
    ///
    /// Only stage zero is constructible by Rule VM content. Stages one and two are
    /// engine-owned tails after draw or equipment-movement triggers finish.
    /// Appended for serialized ordinal stability.
    ResolveDingpan {
        /// Skill owner and possible damage source.
        player: PlayerId,
        /// Selected equipment-bearing target and branch chooser.
        target: PlayerId,
        /// 0=draw, 1=choose branch, 2=deal delayed damage.
        #[serde(default)]
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve OFFLINE 贪赂's frozen HP-gap branch, gift, damage, and delayed discard.
    ///
    /// Only stage zero is constructible by Rule VM content. Later stages are engine-owned
    /// continuations after the target's branch or the complete damage chain.
    /// Appended for serialized ordinal stability.
    ResolveTanlu {
        /// Skill owner, gift recipient, and possible damage source.
        player: PlayerId,
        /// Current-turn player making every forced choice.
        target: PlayerId,
        /// Frozen absolute signed HP difference.
        #[serde(default)]
        amount: u32,
        /// 0=freeze and branch, 1=damage, 2=post-damage discard.
        #[serde(default)]
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Judge repeatedly while results match the first judgment's effective color, gaining each
    /// matching result before optionally continuing.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeSameColorGainRepeat {
        /// Judged player and matching-card recipient.
        player: PlayerId,
        /// Matching judgment card awaiting gain, or `NONE` before a new judgment.
        #[serde(default = "no_card")]
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the gained-card trigger chain has completed and the repeat choice may open.
        #[serde(default)]
        ready_to_choose: bool,
        /// First effective judgment color: 0=unset, 1=red, 2=black.
        #[serde(default)]
        color: u8,
    },
    /// Let a skill owner choose one or more current targets of another player's suspended
    /// non-multitarget trick, then make that trick ineffective for those targets.
    ///
    /// Appended for serialized ordinal stability.
    ChooseCurrentTrickTargetsToNullify {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Current trick user.
        source: PlayerId,
        /// Physical card carrying the logical trick.
        card: CardId,
        /// Originating limited skill instance.
        skill: SkillId,
    },
    /// Judge repeatedly while results are black, gaining each black result and excluding that
    /// concrete card from this player's ordinary discard-to-limit accounting for the turn.
    ///
    /// Appended for serialized ordinal stability.
    JudgeBlackGainRepeatProtectHandLimit {
        /// Judged player and black-card recipient.
        player: PlayerId,
        /// Black judgment card awaiting gain, or `NONE` before a new judgment.
        #[serde(default = "no_card")]
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the gained-card trigger chain has completed and the repeat choice may open.
        #[serde(default)]
        ready_to_choose: bool,
    },
    /// Give one legal recipient every physical card consumed by the resolved Sha, then let a
    /// red gift use Sha and let a black gift draw one card.
    ///
    /// Appended for serialized ordinal stability.
    ChooseGainAllResolvedShaShanThenColorRewards {
        /// Skill owner and original Sha user.
        player: PlayerId,
        /// Physical Sha carrier.
        sha_card: CardId,
        /// Physical Shan source cards observed during this Sha use.
        shan_cards: Vec<CardId>,
        /// Original Sha targets, all excluded from receiving the gift.
        excluded_recipients: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose up to the current draw count of other players, gain one hand card from each, and
    /// reduce rather than replace the pending normal draw.
    ///
    /// Appended for serialized ordinal stability.
    ChooseOtherHandTargetsGainOneReduceDraw {
        /// Current Draw-phase player and chooser.
        player: PlayerId,
        /// Snapshotted selected targets in choice order.
        targets: Vec<PlayerId>,
        /// Current target index.
        index: u32,
        /// Selected target hand card, or NONE while opening/scanning choices.
        selected: CardId,
        /// Draw count captured before the target choice.
        draw_count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose and discard one hand/equipment card from the current Sha target.
    ///
    /// A non-basic selected card makes that target unable to respond, grants the supplied
    /// turn-scoped modifier skill, and increments its matching mark. A basic selected card gives
    /// the target the physical Sha if it remains gainable. Appended for serialized stability.
    ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
        /// Current Sha user and card chooser.
        source: PlayerId,
        /// Physical Sha currently resolving.
        card: CardId,
        /// Current target whose card may be discarded.
        target: PlayerId,
        /// Turn-scoped modifier skill definition and matching mark definition.
        bonus_skill_def: SkillDefId,
    },
    /// Resolve 趫猛's target choice, target-card discard, and equipment/trick branch.
    ///
    /// Only stage zero is constructible by Rule VM content. Later stages are reconstructed from
    /// authoritative choices. Appended for serialized ordinal stability.
    ResolveDcqiaomeng {
        /// Current black-card user, skill owner, and chooser.
        player: PlayerId,
        /// Physical card whose use is suspended.
        card: CardId,
        /// Ordered target snapshot captured from the triggering card event.
        targets: Vec<PlayerId>,
        /// Selected target, or NONE before target selection.
        target: PlayerId,
        /// Selected target card, or NONE before card selection.
        selected: CardId,
        /// Exact-card policy helper definition for the trick-card branch.
        helper_skill_def: SkillDefId,
        /// 0=choose target, 1=choose card, 2=resolve selected card.
        #[serde(default)]
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 界许褚「裸衣」: reveal the draw-pile top three, optionally replace the
    /// normal draw, gain every basic card, Duel, and weapon, discard the rest, and grant
    /// the configured damage helper until the owner's next turn begins.
    ///
    /// Only stage zero is constructible by Rule VM content. Later stages are engine-owned
    /// continuations after reveal and choice resolution. Appended for serialized stability.
    ResolveNewReluoyi {
        /// Current Draw-phase player and card recipient.
        player: PlayerId,
        /// Temporary Sha/Duel damage helper definition.
        buff_skill_def: SkillDefId,
        /// 0=reveal, 1=choose whether to replace draw, 2=move cards, 3=grant helper.
        #[serde(default)]
        stage: u8,
        /// Stable revealed-card snapshot.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Whether the player accepted the draw replacement.
        #[serde(default)]
        accepted: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve OL Huang Zhong's locked 没矢 placement or retrieval branch.
    ///
    /// The public expansion pile is keyed by the originating skill definition. Appended for
    /// serialized ordinal stability.
    ResolveRemoshi {
        /// Skill owner.
        player: PlayerId,
        /// Damaged player or player who lost equipment.
        target: PlayerId,
        /// Current physical Sha carrier for placement; NONE for retrieval.
        card: CardId,
        /// Presentation helper attached to a player holding 没矢 cards.
        helper_skill_def: SkillDefId,
        /// 0=place the physical Sha, 1=retrieve the target's 没矢 pile.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
}
pub(super) use variants;
