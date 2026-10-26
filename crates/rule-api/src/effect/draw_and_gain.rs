//! Draw and gain effect definitions.

macro_rules! variants {
    (DrawCards [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Draw `count` cards for `player`.
    DrawCards {
        /// Recipient.
        player: PlayerId,
        /// Number of cards.
        count: u32,
    },
        ] }
    };
    (DrawCardsFromSkill [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Draw cards while preserving the originating skill instance for trigger filters.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    DrawCardsFromSkill {
        /// Recipient.
        player: PlayerId,
        /// Number of cards.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (DrawEachBySeat [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Draw for two players in seat order.
    ///
    /// `skill` is filled by the engine VM host so draw-trigger provenance remains exact.
    DrawEachBySeat {
        /// First participant.
        first: PlayerId,
        /// Second participant.
        second: PlayerId,
        /// Cards drawn by each participant.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Draw cards, then force the drawer to give one eligible hand/equipment card.
    ///
    /// The progress fields are engine-owned continuation state. Content emits them empty.
    DrawThenGiveOne {
        /// Drawer and giver.
        player: PlayerId,
        /// Recipient.
        target: PlayerId,
        /// Cards to draw.
        count: u32,
        /// Restrict the gift to cards drawn by this effect.
        drawn_only: bool,
        /// Let the recipient optionally use the gifted card when it is equipment.
        #[serde(default)]
        use_given_equipment: bool,
        /// Originating skill instance.
        skill: SkillId,
        /// Cards drawn by the initial stage.
        #[serde(default)]
        drawn: Vec<CardId>,
        /// False for draw stage, true for give stage.
        #[serde(default)]
        ready_to_give: bool,
    },
    /// Draw cards, then let another living player take one hand/equipment card.
    ///
    /// The progress field is engine-owned continuation state. Content emits it false.
    DrawThenTakeOne {
        /// Drawer and card owner.
        player: PlayerId,
        /// Player choosing and taking the card.
        taker: PlayerId,
        /// Cards to draw.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
        /// False for draw stage, true for take stage.
        #[serde(default)]
        ready_to_take: bool,
    },
        ] }
    };
    (AdjustHandTo [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Adjust a player's hand to an exact count: draw when below it, otherwise
    /// force that player to choose the excess hand cards to discard.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    AdjustHandTo {
        /// Player whose hand is adjusted.
        player: PlayerId,
        /// Exact target hand count.
        count: u32,
    },
        ] }
    };
    (DrawByPlayPhaseCardCategoryCount [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Draw cards equal to the number of distinct logical card categories the player used in
    /// the current play phase (basic, trick, equipment).
    ///
    /// The bounded history is maintained by the engine only when loaded content requests it.
    /// Appended for serialized ordinal stability.
    DrawByPlayPhaseCardCategoryCount {
        /// Player whose play-phase history is queried and who draws the cards.
        player: PlayerId,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
        ] }
    };
    (DrawThenGiveExactRecordPhaseTarget [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Draw cards, then give exactly `count` hand/equipment cards to the other current
    /// Play-phase player and record that player for a delayed phase consequence.
    /// Appended for serialized ordinal stability.
    DrawThenGiveExactRecordPhaseTarget {
        /// Skill owner drawing and giving cards.
        player: PlayerId,
        /// Other player's current Play phase.
        target: PlayerId,
        /// Draw and gift count.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
        /// Internal continuation flag set after the draw and nested draw triggers finish.
        ready_to_give: bool,
    },
        ] }
    };
    (IncreaseCurrentDrawCount [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Increase the pending card count for the current draw phase. The draw-phase command
    /// consumes the accumulated value exactly once.
    /// Appended for serialized ordinal stability.
    IncreaseCurrentDrawCount {
        /// Current draw-phase player.
        player: PlayerId,
        /// Additional cards.
        count: u32,
    },
        ] }
    };
    (DrawThenChooseGainOneFromDamageTargetAndNeighbors [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Draw one card, then choose and gain one card in legacy seat order from the damaged
    /// player, that player's previous living seat, and that player's next living seat.
    /// Content emits `ready_to_choose = false`; the engine-owned second stage runs only after
    /// the draw and all nested draw triggers have finished.
    /// Appended for serialized ordinal stability.
    DrawThenChooseGainOneFromDamageTargetAndNeighbors {
        /// Damage source, skill owner, drawer, and card chooser.
        player: PlayerId,
        /// Player damaged by the originating damage event.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// False for the draw stage and true for the post-draw seat-list stage.
        #[serde(default)]
        ready_to_choose: bool,
    },
        ] }
    };
    (GiveSelectedThenRecoverSelf [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Active selection: give the chosen hand/equip cards to `target`, then the **chooser**
    /// recovers `amount` HP. Appended for serialized stability (ABI 679, batch 172 jun_renwang).
    GiveSelectedThenRecoverSelf {
        /// Chooser and giver; recovers after the gift.
        player: PlayerId,
        /// Gift recipient.
        target: PlayerId,
        /// Chosen cards (injected from the active selection).
        cards: Vec<CardId>,
        /// Recovery amount for the chooser.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (DrawForLifetimeCardActionDivisors [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Draw one card for each supplied divisor of the player's match-lifetime number of cards
    /// used or played as a response. Appended for serialized stability.
    DrawForLifetimeCardActionDivisors {
        /// Player whose action history is inspected and who draws the cards.
        player: PlayerId,
        /// Three positive divisors checked independently.
        divisors: [u32; 3],
    },
        ] }
    };
    (DrawSelectedTargets [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Draw cards for the distinct living targets selected by the active-skill command.
    ///
    /// The VM snapshots `targets` from the authoritative active invocation. The engine resolves
    /// them in seat order. Appended for serialized stability.
    DrawSelectedTargets {
        /// Active-skill targets.
        targets: Vec<PlayerId>,
        /// Cards drawn by each target.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (DrawThenChooseGiveOrDiscardHand [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Draw cards, wait for the draw's nested triggers, then choose either to give an exact
    /// number of hand cards to another living player or discard that many hand cards.
    ///
    /// Appended for serialized ordinal stability.
    DrawThenChooseGiveOrDiscardHand {
        /// Skill owner drawing and paying cards.
        player: PlayerId,
        /// Cards drawn before the branch choice.
        draw_count: u32,
        /// Exact hand-card payment count when enough cards remain.
        payment_count: u32,
        /// Originating skill instance.
        skill: SkillId,
        /// Internal continuation flag set after draw events and nested triggers finish.
        ready_to_choose: bool,
    },
    /// Gain every card currently equipped by `target` in one equipment slot.
    ///
    /// The target zone and card slot are revalidated when this request resolves. Appended for
    /// serialized ordinal stability.
    GainTargetEquipment {
        /// Player gaining the equipment.
        player: PlayerId,
        /// Player whose matching equipment is gained.
        target: PlayerId,
        /// Stable equipment-slot tag (`1` weapon through `5` treasure).
        slot: u8,
    },
        ] }
    };
    (GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Give one active-skill-selected hand card to another living player. After the movement and
    /// nested triggers complete, that recipient chooses either to reveal their current hand and
    /// discard every discardable hand/equipment card matching the given card's physical suit, or
    /// lose 1 HP.
    ///
    /// `ready_to_choose` is false only for the VM-emitted gift stage. The engine captures `suit`
    /// from `card` before moving it and schedules the true stage behind the movement barrier.
    /// Appended for serialized ordinal stability.
    GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
        /// Skill owner giving the selected card.
        player: PlayerId,
        /// Other living recipient who makes the follow-up choice.
        target: PlayerId,
        /// Exact active-skill hand-card input.
        card: CardId,
        /// Captured physical suit of the given card.
        suit: Suit,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the gift and its nested triggers have completed.
        ready_to_choose: bool,
    },
    /// Gain the topmost equipment card in the shared draw pile.
    ///
    /// This request never searches the discard pile. Appended for serialized ordinal stability.
    GainEquipmentFromDrawPile {
        /// Player gaining the equipment card.
        player: PlayerId,
    },
    /// Give every current hand card from `player` to `target`, then recover `target` to exactly
    /// one HP after the movement and all nested triggers finish.
    ///
    /// Content emits `ready_to_recover = false`; the engine-owned second stage runs behind the
    /// movement barrier. Appended for serialized ordinal stability.
    GiveAllHandRecoverToOne {
        /// Living skill owner giving every hand card.
        player: PlayerId,
        /// Other living dying recipient.
        target: PlayerId,
        /// Whether the gift and all nested movement triggers have completed.
        ready_to_recover: bool,
    },
        ] }
    };
    (FinishOpeningExtraPlayShaGain [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Finish the originating turn-opening Sha-gain policy and apply its no-damage consequence.
    ///
    /// Appended for serialized ordinal stability.
    FinishOpeningExtraPlayShaGain {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (GiveAllHand [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Give every current hand card from `player` to `target`.
    ///
    /// Appended for serialized ordinal stability.
    GiveAllHand {
        /// Living player giving every hand card.
        player: PlayerId,
        /// Other living recipient.
        target: PlayerId,
    },
        ] }
    };
    (DrawRememberedHandCards [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Draw cards and remember their exact physical ids in the originating skill instance.
    ///
    /// Remembered ids are removed as soon as those cards leave the owner's hand.
    /// Appended for serialized ordinal stability.
    DrawRememberedHandCards {
        /// Skill owner and drawer.
        player: PlayerId,
        /// Number of cards to draw.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (DrawThenOptionallyDistributeHandCards [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Draw cards, then optionally plan and give up to a bounded number of current hand cards to
    /// one or more other living players.
    ///
    /// Appended for serialized ordinal stability.
    DrawThenOptionallyDistributeHandCards {
        /// Skill owner, drawer, chooser, and giver.
        player: PlayerId,
        /// Cards drawn before distribution choices begin.
        draw_count: u32,
        /// Maximum total number of hand cards that may be distributed.
        give_cap: u32,
        /// Planned distinct hand cards, paired positionally with `targets`.
        cards: Vec<CardId>,
        /// Planned recipients, paired positionally with `cards`.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
        /// Internal flag set after draw events and nested triggers finish.
        ready_to_choose: bool,
        /// Internal flag requesting atomic validation and transfer of the complete plan.
        finish: bool,
    },
    /// Give the active invocation's selected hand cards, then optionally repeat for previously
    /// unused other recipients; recover once when the cumulative gift count first reaches two.
    ///
    /// Appended for serialized ordinal stability.
    GiveSelectedThenContinueDistinctRecoverAtTwo {
        /// Skill owner, chooser, giver, and recovery target.
        player: PlayerId,
        /// Current distinct gift recipient.
        target: PlayerId,
        /// Current hand cards to give.
        cards: Vec<CardId>,
        /// Recipients already used by this activation.
        used_targets: Vec<PlayerId>,
        /// Cards given by this activation before the current batch.
        total_given: u32,
        /// Originating active skill instance.
        skill: SkillId,
        /// Internal flag that opens the next optional card choice without giving a batch.
        continue_only: bool,
    },
        ] }
    };
    (ReplaceDrawWithDistinctSuitSelection [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (GainOneHandFromSelectedTargets [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (GiveSelectedThenRecoverTarget [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (DrawLinkedPlayers [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Draw `count` cards for every currently linked living player in seat order.
    ///
    /// Appended for serialized ordinal stability.
    DrawLinkedPlayers {
        /// Cards drawn by each linked player.
        count: u32,
    },
        ] }
    };
    (GiveSelectedThenTargetMayUseDrawOwner [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Give one selected non-equipment hand card, let the recipient optionally use it, and draw
    /// one for the owner after a successful use. ABI 497 (`yuanlve`).
    GiveSelectedThenTargetMayUseDrawOwner {
        /// Skill owner and giver.
        player: PlayerId,
        /// Other living recipient.
        target: PlayerId,
        /// Exact selected card.
        card: CardId,
        /// Selected targets for the recipient's use.
        targets: Vec<PlayerId>,
        /// Resolution stage: 0 gift, 1 offer use, 2 commit use.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (DrawCardsFromBottom [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 寸目（shenhua 许攸，主公技）：本回合摸牌改为从牌堆底摸。ABI 589。
    /// Appended for serialized ordinal stability.
    DrawCardsFromBottom {
        /// Player drawing.
        player: PlayerId,
        /// Card count.
        count: u32,
    },
        ] }
    };
    (GuozhanUseZhulianBihe [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Guozhan: spend a 珠联璧合 (perfect pair) boon mark to draw two cards, emulating a
    /// 【无中生有】 effect. The mark is consumed from the player's `marks` list.
    /// ABI 641 (ADR 0604).
    GuozhanUseZhulianBihe {
        /// Player spending the mark.
        player: PlayerId,
    },
        ] }
    };
    (DrawAllEventTargets [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Draw `count` cards into the hand of each listed player. Implements upstream
    /// `game.asyncDraw([player, ...event.targets])`. The VM host fills the player list with the
    /// event player + the declared event targets. Used by clanxsyingxiang's cross-player draw.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    DrawAllEventTargets {
        /// Players to draw (event player + declared targets, in seat order).
        players: Vec<PlayerId>,
        /// Count per player.
        count: u32,
    },
        ] }
    };
    (ChooseClanNotMaxHandLimitGain [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 穆荫 (clan/clanmuyin): the owner selects a living player who is the owner
    /// themself or shares a clan with the owner and whose hand limit is NOT the
    /// field maximum; that player permanently gains +1 hand limit.
    /// Appended for serialized ordinal stability.
    ChooseClanNotMaxHandLimitGain {
        /// Choosing player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
}
pub(super) use variants;
