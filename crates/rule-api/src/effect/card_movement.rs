//! Card movement effect definitions.

macro_rules! variants {
    (MoveCards [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Move cards to a zone.
    MoveCards {
        /// Cards in order.
        cards: Vec<CardId>,
        /// Destination zone.
        to: ZoneId,
        /// Optional explicit source (validated by core).
        from: Option<ZoneId>,
    },
    /// Use a card (play-as-use semantics).
    UseCard {
        /// User.
        player: PlayerId,
        /// Card.
        card: CardId,
        /// Targets.
        targets: Vec<PlayerId>,
    },
    /// Use and consume a physical card from an equipment or judgment zone.
    UseFieldCard {
        /// User (which may differ from the field card's owner).
        player: PlayerId,
        /// Physical field card.
        card: CardId,
        /// Targets.
        targets: Vec<PlayerId>,
    },
        ] }
    };
    (RevealCards [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Reveal cards to observers.
    RevealCards {
        /// Cards.
        cards: Vec<CardId>,
        /// Recipients; empty = public.
        to: Vec<PlayerId>,
    },
        ] }
    };
    (DiscardFromHand [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Discard up to `count` cards from hand (deterministic: hand zone order).
    DiscardFromHand {
        /// Player.
        player: PlayerId,
        /// Max cards to discard.
        count: u32,
    },
    /// Deterministically discard up to `count` cards from `player`'s hand, then equipment (he
    /// scope, in legacy seat/id order). Powers skills like 「惊澜」.
    ///
    /// Appended for serialized ordinal stability.
    DiscardFromHandEquip {
        /// Player.
        player: PlayerId,
        /// Max cards to discard.
        count: u32,
    },
    /// Discard an explicit set of cards owned by `player`.
    DiscardCards {
        /// Card owner.
        player: PlayerId,
        /// Selected cards, in command order.
        cards: Vec<CardId>,
    },
    /// Take the first card from `from`'s hand into `to`'s hand (deterministic).
    TakeFromHand {
        /// Source hand owner.
        from: PlayerId,
        /// Destination hand owner.
        to: PlayerId,
    },
        ] }
    };
    (RemoveCardsByKind [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Move every card of one rule kind from live play zones into the shared
    /// removed-from-game zone.
    RemoveCardsByKind {
        /// Stable rule card-kind tag.
        kind: u8,
    },
        ] }
    };
    (RevealDiscardSuitGainOrUse [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Reveal draw-pile cards, then perform the discard-by-suit gain option and/or
    /// use-one-revealed-card option in either order.
    ///
    /// The progress fields are engine-owned continuation state. Content emits them empty.
    /// Appended so existing serialized effect variant ordinals remain stable.
    RevealDiscardSuitGainOrUse {
        /// Player performing the choices.
        player: PlayerId,
        /// Mark whose current count expands the reveal and whose stack rewards both options.
        mark: MarkDefId,
        /// Base number of cards revealed.
        base_count: u8,
        /// Maximum counted/rewarded mark stacks.
        max_mark: u8,
        /// Revealed cards still tracked by this resolution.
        #[serde(default)]
        revealed: Vec<CardId>,
        /// Completed option bitset: 1=discard/gain, 2=use.
        #[serde(default)]
        completed: u8,
        /// Internal resume step: 0=choose/finish, 1=discard suit captured.
        #[serde(default)]
        step: u8,
        /// Step-specific stable scalar (currently the discarded card's effective suit tag).
        #[serde(default)]
        data: u32,
    },
        ] }
    };
    (DiscardAllHandAndEquip [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Discard every card in `player`'s hand and equipment zones.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    DiscardAllHandAndEquip {
        /// Player losing all hand and equipment cards.
        player: PlayerId,
    },
        ] }
    };
    (TakeOneCard [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Let one player choose and take one card from another player's allowed zones.
    TakeOneCard {
        /// Card owner.
        from: PlayerId,
        /// Player choosing and taking the card.
        to: PlayerId,
        /// Restrict the choice to the equipment zone.
        equip_only: bool,
    },
        ] }
    };
    (RevealTopGainNonBasicOrSwapBasic [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Reveal the draw-pile top card, gain it when non-basic, or optionally
    /// replace a basic card with one hand card placed on top of the draw pile.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    RevealTopGainNonBasicOrSwapBasic {
        /// Player revealing, gaining, and optionally swapping the card.
        player: PlayerId,
    },
        ] }
    };
    (GainCardByKind [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Gain the topmost card of one rule kind, searching the draw pile before
    /// the discard pile.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    GainCardByKind {
        /// Player gaining the card.
        player: PlayerId,
        /// Stable rule card-kind tag.
        kind: u8,
    },
        ] }
    };
    (GiveCards [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Move cards as a voluntary transfer from one player to another.
    ///
    /// This engine-level primitive preserves the semantic movement reason for
    /// content triggers. Appended so existing serialized effect ordinals remain stable.
    GiveCards {
        /// Cards in transfer order.
        cards: Vec<CardId>,
        /// Recipient hand zone.
        to: ZoneId,
        /// Explicit source personal zone.
        from: ZoneId,
    },
    /// Materialize and use a suitless virtual card against one explicit target.
    ///
    /// The card traverses the ordinary use/nullification/resolution pipeline and
    /// is moved to the removed zone after use. Appended for ordinal stability.
    UseVirtualCardOn {
        /// User.
        player: PlayerId,
        /// Stable logical card-kind tag.
        kind: u8,
        /// Explicit target.
        target: PlayerId,
    },
        ] }
    };
    (RecastCards [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Recast selected personal-zone cards, then draw the same count.
    ///
    /// Appended for serialized ordinal stability.
    RecastCards {
        /// Player recasting the cards and drawing replacements.
        player: PlayerId,
        /// Physical cards selected by the active invocation.
        cards: Vec<CardId>,
    },
    /// Disable the requested currently enabled equipment slots.
    ///
    /// Slot tags are stable (`1` weapon through `5` treasure). Equipment occupying a newly
    /// disabled slot is discarded. Appended for serialized ordinal stability.
    DisableEquipSlots {
        /// Affected player.
        player: PlayerId,
        /// Stable equipment-slot tags in deterministic request order.
        slots: Vec<u8>,
    },
    /// Let `player` move one equipment card to another legal living player, excluding
    /// `player` as the destination, then let the equipment's original owner draw one card.
    ///
    /// The card and destination choices are resolved against canonical state and revalidated
    /// at both choice barriers. Appended for serialized ordinal stability.
    MoveEquipmentThenDraw {
        /// Skill owner making both choices and forbidden as the destination.
        player: PlayerId,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
        ] }
    };
    (DiscardEquipmentPairWithArmor [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Discard one equipment card from each of two distinct selected players, requiring at
    /// least one selected card to be armor.
    ///
    /// Target and card choices are revalidated against canonical discard-protection rules at
    /// every barrier. Appended for serialized ordinal stability.
    DiscardEquipmentPairWithArmor {
        /// Skill owner choosing the two equipment cards.
        player: PlayerId,
        /// Two distinct living targets selected by the active invocation.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let `player` choose and discard one equipment card from `target`; if that card is a
    /// mount and remains in the discard pile, move it into `player`'s hand.
    ///
    /// The forced card choice and discard protection are revalidated at the continuation
    /// barrier. Appended for serialized ordinal stability.
    DiscardTargetEquipmentGainMount {
        /// Player choosing the equipment card and receiving a discarded mount.
        player: PlayerId,
        /// Living damaged player whose equipment may be discarded.
        target: PlayerId,
    },
    /// Let `player` choose and discard one card from `target` in a typed zone scope; if that
    /// card is a mount and remains in the discard pile, move it into `player`'s hand.
    ///
    /// Appended for serialized ordinal stability. ABI-33 packages keep using
    /// [`Self::DiscardTargetEquipmentGainMount`]; ABI-34 content uses this generalized form.
    DiscardTargetCardGainMount {
        /// Player choosing the card and receiving a discarded mount.
        player: PlayerId,
        /// Living damaged player whose card may be discarded.
        target: PlayerId,
        /// Canonical zones offered to the chooser.
        scope: TargetCardScope,
    },
    /// Discard concrete cards from another player's typed target-card scope.
    ///
    /// This preserves semantic `Discard` movement events for hand/equipment/judgment effects
    /// while reusing the canonical discard-protection query. Appended for ordinal stability.
    DiscardTargetCards {
        /// Player causing the discard.
        actor: PlayerId,
        /// Player who owns the selected cards.
        target: PlayerId,
        /// Concrete cards selected from the target.
        cards: Vec<CardId>,
        /// Canonical zones allowed by the effect.
        scope: TargetCardScope,
    },
        ] }
    };
    (ChooseDiscardTargetCard [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Let `player` choose and discard one card from `target` in a typed zone scope.
    ///
    /// The forced choice and canonical discard protection are revalidated at the continuation
    /// barrier. Appended for serialized ordinal stability.
    ChooseDiscardTargetCard {
        /// Player choosing the card.
        player: PlayerId,
        /// Living player whose card may be discarded.
        target: PlayerId,
        /// Canonical zones offered to the chooser.
        scope: TargetCardScope,
    },
        ] }
    };
    (ChooseGainTopCardsBySuit [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Privately view draw-pile top cards, then optionally reveal and gain any matching-suit
    /// subset while leaving every unselected card on top in its original relative order.
    ///
    /// Appended for serialized ordinal stability.
    ChooseGainTopCardsBySuit {
        /// Player viewing and choosing cards.
        player: PlayerId,
        /// Maximum number of top cards viewed.
        count: u8,
        /// Required effective suit.
        suit: Suit,
    },
        ] }
    };
    (ChooseCompareSelectedHandCard [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Compare the active invocation's selected hand card with one hand card chosen by `target`.
    /// The two cards are revealed and discarded; strict victory grants a target-bound helper,
    /// while a tie or loss grants the ordinary losing helper.
    /// Appended for serialized ordinal stability.
    ChooseCompareSelectedHandCard {
        /// Skill owner and first comparison participant.
        player: PlayerId,
        /// Other comparison participant.
        target: PlayerId,
        /// Owner's card captured from the active invocation's selected-card input.
        card: CardId,
        /// Helper skill granted on strict victory and bound to `target`.
        win_skill_def: SkillDefId,
        /// Helper skill granted on tie or loss.
        lose_skill_def: SkillDefId,
        /// Originating active skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseAddSkillBoundCardTarget [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Optionally append the current skill instance's bound target to the suspended one-target
    /// Sha or ordinary-trick use.
    /// Appended for serialized ordinal stability.
    ChooseAddSkillBoundCardTarget {
        /// Current card user and chooser.
        player: PlayerId,
        /// Physical card whose use is suspended in a CardUsed trigger.
        card: CardId,
        /// Originating target-bound skill instance.
        skill: SkillId,
    },
    /// Engine-internal commit of a selected bound extra target.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    CommitAddSkillBoundCardTarget {
        /// Current card user.
        player: PlayerId,
        /// Physical card whose use is suspended.
        card: CardId,
        /// Revalidated bound target.
        target: PlayerId,
        /// Originating target-bound skill instance.
        skill: SkillId,
    },
        ] }
    };
    (UseVirtualShaFromCardColorLockSourceIfNoSha [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// If the helper owner used no logical Sha in the current Play phase, make the source stored
    /// in this skill instance use a virtual Sha on them without distance or use-count limits.
    /// Appended for serialized ordinal stability.
    UseVirtualShaFromCardColorLockSourceIfNoSha {
        /// Helper owner and retaliation target.
        player: PlayerId,
        /// Originating color-lock helper instance.
        skill: SkillId,
    },
    /// Force the user of the suspended one-target Sha or ordinary trick to choose one or two
    /// reusable enhancements for that exact card use.
    /// Appended for serialized ordinal stability.
    ChooseCurrentCardUseEnhancements {
        /// Current card user and chooser.
        player: PlayerId,
        /// Physical card whose use is suspended.
        card: CardId,
        /// Temporary helper definition that stores exact-card policies.
        helper_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (DiscardSelectedRecordUniqueCategoryTarget [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Discard one selected hand/equipment card after atomically recording its logical category
    /// and the chosen target in the originating skill's turn-scoped uniqueness state.
    /// Appended for serialized ordinal stability.
    DiscardSelectedRecordUniqueCategoryTarget {
        /// Skill owner and card owner.
        player: PlayerId,
        /// Other player selected by the active skill.
        target: PlayerId,
        /// Selected physical cost card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseRevealOtherHandCardRememberCategory [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Let `player` choose another living player with hand cards, choose one of those cards,
    /// reveal it publicly, and remember its logical category for the current Play phase.
    /// Appended for serialized ordinal stability.
    ChooseRevealOtherHandCardRememberCategory {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (RevealUniformHandDrawSelfChooseOthers [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Publicly reveal the owner's current hand. If every card has the same effective color,
    /// let the owner choose up to the hand count of other living players, then draw one card for
    /// every chosen player and the owner in canonical seat order.
    /// Appended for serialized ordinal stability.
    RevealUniformHandDrawSelfChooseOthers {
        /// Skill owner, hand owner, and target chooser.
        player: PlayerId,
        /// Originating skill instance for validation and draw provenance.
        skill: SkillId,
    },
    /// Choose another legal target for a suspended single-target Sha/ordinary trick. That
    /// player may give one hand card and become the user, or become an additional target.
    /// Appended for serialized ordinal stability.
    ChooseCardUserOrAddTarget {
        /// Current card user, skill owner, and first chooser.
        player: PlayerId,
        /// Suspended physical card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal user transfer after the selected hand card and all nested movement
    /// triggers finish. Rule VM bytecode never emits this directly.
    /// Appended for serialized ordinal stability.
    CommitTransferSavedCardUser {
        /// Previous card user and skill owner.
        previous: PlayerId,
        /// New card user who paid the hand-card gift.
        new_user: PlayerId,
        /// Suspended physical card.
        card: CardId,
        /// Originating skill instance for provenance.
        skill: SkillId,
    },
        ] }
    };
    (ContinueChooseCompareTargetHandCard [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Engine-internal continuation that opens or resolves the second participant's hand-card
    /// selection after every `hand_compare_started` trigger has completed.
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    ContinueChooseCompareTargetHandCard {
        /// First comparison participant.
        player: PlayerId,
        /// Second comparison participant.
        target: PlayerId,
        /// Initially selected card of the first participant.
        card: CardId,
        /// Helper granted on strict victory.
        win_skill_def: SkillDefId,
        /// Helper granted on tie or loss.
        lose_skill_def: SkillDefId,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Make the other participant's card in the currently starting hand comparison be selected
    /// by authoritative deterministic RNG. Appended for serialized ordinal stability.
    RandomizeCurrentHandCompareOpponent {
        /// Skill owner and one comparison participant.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (GainHighestRankShaFromEventCards [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Gain every highest-rank logical Sha among the physical cards captured by the triggering
    /// domain event and still present in the discard pile. Appended for serialized stability.
    GainHighestRankShaFromEventCards {
        /// Skill owner receiving the cards.
        player: PlayerId,
        /// Physical event cards copied into the typed request by the Rule VM host.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Gain the legacy `zongshi` result card from a completed two-player hand comparison when
    /// that physical card is still in the discard pile. Appended for serialized stability.
    GainHandCompareResultCard {
        /// Skill owner receiving the card.
        player: PlayerId,
        /// First comparison participant.
        initiator: PlayerId,
        /// Second comparison participant.
        target: PlayerId,
        /// Strict winner, or `NONE` for a tie.
        winner: PlayerId,
        /// Physical comparison cards in `[initiator, target]` order.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose whether to add one legal target to, or remove one existing target from, the
    /// currently resolving basic or trick card. Appended for serialized stability.
    ChooseAddOrRemoveCurrentCardTarget {
        /// Card user, skill owner, and chooser.
        player: PlayerId,
        /// Physical card whose saved target list may be adjusted.
        card: CardId,
        /// Originating one-shot skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseOtherDiscardTargetCard [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose another living player, then choose and discard one card in the supplied zone scope.
    /// Appended for serialized stability.
    ChooseOtherDiscardTargetCard {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Canonical zones offered for the selected target.
        scope: TargetCardScope,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Gain the topmost card of one rule kind from the discard pile only.
    /// Appended for serialized stability.
    GainDiscardCardByKind {
        /// Player gaining the card.
        player: PlayerId,
        /// Stable rule card-kind tag.
        kind: u8,
    },
        ] }
    };
    (ChooseGainDiscardCardByKind [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Force the player to choose and gain one matching card from the discard pile.
    /// Appended for serialized stability.
    ChooseGainDiscardCardByKind {
        /// Player choosing and gaining the card.
        player: PlayerId,
        /// Stable rule card-kind tag.
        kind: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (GainEventCards [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Gain every card captured by the triggering event that remains in the discard pile.
    /// Appended for serialized stability.
    GainEventCards {
        /// Player gaining the cards.
        player: PlayerId,
        /// Stable event-card snapshot supplied by the engine host.
        cards: Vec<CardId>,
    },
        ] }
    };
    (DiscardAllHandEquipJudge [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Force every card in a player's hand, equipment, and judgment zones into discard.
    ///
    /// Appended for serialized stability.
    DiscardAllHandEquipJudge {
        /// Player whose three public/private character zones are emptied.
        player: PlayerId,
    },
    /// Choose and discard one hand/equipment card from the current Sha target.
    ///
    /// If the selected card is equipment, that target cannot respond to this Sha. Otherwise,
    /// the target gains the physical Sha if it remains in the discard pile. Appended for
    /// serialized stability.
    ChooseDiscardTargetCardResolveCurrentSha {
        /// Current Sha user and card chooser.
        source: PlayerId,
        /// Physical Sha currently resolving.
        card: CardId,
        /// Current target whose card may be discarded.
        target: PlayerId,
    },
    /// Materialize one source-free virtual card and use it on the active skill's selected targets.
    ///
    /// Appended for serialized stability.
    UseVirtualCardOnSelectedTargets {
        /// Virtual card user.
        player: PlayerId,
        /// Stable logical card-kind tag.
        kind: u8,
        /// Distinct active-skill targets in submitted order.
        targets: Vec<PlayerId>,
    },
        ] }
    };
    (ChooseUseVirtualCard [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Use a suitless, source-free virtual Wanjian or Juedou through the ordinary card pipeline.
    ///
    /// Wanjian resolves against its complete legal target set immediately. Juedou opens a
    /// mandatory target choice. Appended for serialized stability.
    ChooseUseVirtualCard {
        /// Virtual card user.
        player: PlayerId,
        /// Stable card-kind tag; ABI 98 accepts Wanjian and Juedou.
        kind: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-owned tail that ends phase-history suppression after a phase-end virtual card
    /// finishes every nested choice and response.
    ///
    /// Content bytecode cannot construct this request. Appended for serialized stability.
    FinishPhaseEndVirtualCard,
        ] }
    };
    (TakeTargetHandThenRepeatUntilMax [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Take one hand card from the mandatory larger-hand target, then optionally repeat with
    /// freshly chosen larger-hand targets until the owner ties for the largest living hand.
    ///
    /// Appended for serialized stability.
    TakeTargetHandThenRepeatUntilMax {
        /// Skill owner choosing and gaining cards.
        player: PlayerId,
        /// Mandatory first target selected by the active-skill command.
        target: PlayerId,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Engine-owned scan for the next optional larger-hand target after movement triggers finish.
    ///
    /// Rule VM bytecode cannot construct this request. Appended for serialized stability.
    ContinueTakeLargerHandUntilMax {
        /// Skill owner choosing and gaining cards.
        player: PlayerId,
        /// Originating active skill instance.
        skill: SkillId,
    },
        ] }
    };
    (DiscardAllTargetEquipment [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Discard every equipment card that `actor` may currently discard from `target`.
    ///
    /// The target's living state, equipment zone, and discard protection are revalidated when
    /// this request resolves. Appended for serialized ordinal stability.
    DiscardAllTargetEquipment {
        /// Player causing the discard.
        actor: PlayerId,
        /// Player whose discardable equipment is discarded.
        target: PlayerId,
    },
        ] }
    };
    (DiscardTargetEquipmentDrawWeaponRange [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Force the skill owner to choose and discard one current equipment card from a target;
    /// when the selected card is a weapon, the target draws its physical attack range after
    /// movement triggers finish.
    ///
    /// Appended for serialized ordinal stability.
    DiscardTargetEquipmentDrawWeaponRange {
        /// Skill owner choosing and causing the discard.
        player: PlayerId,
        /// Living target whose equipment is discarded and who may draw.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseRevealHandCardBindCategoryTarget [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Force the skill owner to choose and publicly reveal one current hand card from the
    /// specified Play-phase player, then bind that player and the card's effective category to
    /// this skill instance until the phase changes.
    ///
    /// Appended for serialized ordinal stability.
    ChooseRevealHandCardBindCategoryTarget {
        /// Skill owner choosing and revealing the card.
        player: PlayerId,
        /// Current Play-phase player whose hand card is revealed.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (RevealHandDiscardKind [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Publicly reveal a player's complete current hand, then discard every discardable hand
    /// card whose effective kind matches `kind`.
    ///
    /// Appended for serialized ordinal stability.
    RevealHandDiscardKind {
        /// Player revealing and discarding cards.
        player: PlayerId,
        /// Stable logical card-kind tag.
        kind: u8,
    },
        ] }
    };
    (RevealTargetHandGainShanArmorOrDrawTwo [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Reveal another player's hand, gain every effective Shan and physical armor, or draw two
    /// when no card matches.
    ///
    /// Appended for serialized ordinal stability.
    RevealTargetHandGainShanArmorOrDrawTwo {
        /// Skill owner and recipient.
        player: PlayerId,
        /// Other living player whose hand is revealed.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (GainCardIfDiscard [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Gain `card` if it is still in the shared discard pile.
    ///
    /// Appended for serialized ordinal stability.
    GainCardIfDiscard {
        /// Living recipient.
        player: PlayerId,
        /// Physical card to gain.
        card: CardId,
    },
    /// Let the Sha user cancel, draw one card, or discard one target hand/equipment card.
    /// Accepting either branch records this skill and target on the exact resolving Sha.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDrawOrDiscardTargetCardRecordSha {
        /// Sha user and chooser.
        player: PlayerId,
        /// Exact resolving Sha.
        card: CardId,
        /// Current Sha target.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Reveal `player`'s complete current hand only to `observer`.
    ///
    /// Appended for serialized ordinal stability.
    RevealHandToPlayer {
        /// Player whose hand is revealed.
        player: PlayerId,
        /// Living player authorized to see the hand.
        observer: PlayerId,
    },
    /// Let `player` optionally choose and discard one card from `target` in a typed zone scope.
    ///
    /// Cancelling is a successful no-op. The scope and card legality are revalidated at the
    /// continuation barrier. Appended for serialized ordinal stability.
    ChooseOptionalDiscardTargetCard {
        /// Player choosing the card.
        player: PlayerId,
        /// Living player whose card may be discarded.
        target: PlayerId,
        /// Canonical zones offered to the chooser.
        scope: TargetCardScope,
    },
    /// Publicly reveal `player`'s complete current hand, then let that player choose one
    /// effective red/black color present and discard every discardable hand card of that color.
    /// A single present color resolves without opening a choice.
    ///
    /// Appended for serialized ordinal stability.
    RevealHandChooseDiscardColor {
        /// Player revealing, choosing, and discarding.
        player: PlayerId,
    },
    /// Force `player` to discard up to `count` current hand/equipment cards from `target`.
    /// Recover `amount` HP after movement triggers only when exactly `count` cards were
    /// discarded and their effective categories for `target` are identical.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDiscardTargetCardsRecoverSameCategory {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Player whose cards are discarded.
        target: PlayerId,
        /// Maximum discard count.
        count: u32,
        /// Maximum HP recovered after a matching discard.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseDrawOrDiscardTargetCard [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose between drawing cards and discarding one target hand/equipment card.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDrawOrDiscardTargetCard {
        /// Living chooser.
        player: PlayerId,
        /// Other player whose card may be discarded.
        target: PlayerId,
        /// Draw-branch count.
        draw_count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (EnableEquipSlot [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Restore one disabled copy of an equipment slot.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    EnableEquipSlot {
        /// Player whose equipment slot is restored.
        player: PlayerId,
        /// Stable slot tag (`1` weapon through `5` treasure).
        slot: u8,
        /// Player causing the restoration.
        source: PlayerId,
    },
        ] }
    };
    (DistributeTopTwoCards [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Move the draw-pile top two cards into public processing and distribute all of them.
    ///
    /// `ready_to_choose = false` performs the top-card movement. The engine-owned stages use
    /// `cards` for the remaining processing cards, `selected` for the current subset, and
    /// `target` for its recipient. Appended so existing serialized effect variant ordinals
    /// remain stable.
    DistributeTopTwoCards {
        /// Skill owner choosing every subset and recipient.
        player: PlayerId,
        /// Remaining cards to distribute.
        cards: Vec<CardId>,
        /// Current selected subset.
        selected: Vec<CardId>,
        /// Current subset recipient, or NONE before target choice.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the top cards have reached processing.
        ready_to_choose: bool,
    },
        ] }
    };
    (ChooseTargetGainBottomCard [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Reveal the draw-pile bottom card, then force the skill owner to give it to a living
    /// player.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseTargetGainBottomCard {
        /// Skill owner and attributed giver.
        player: PlayerId,
        /// Revealed card in processing, or `NONE` before it is staged.
        card: CardId,
        /// Chosen recipient, or `NONE` before target selection.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (UseVirtualJuedouSecondOnFirst [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Make the active skill's second selected target use a suitless virtual Duel on the first
    /// without opening a Wuxie window.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    UseVirtualJuedouSecondOnFirst {
        /// Active-skill targets in submitted order.
        targets: Vec<PlayerId>,
        /// Materialized virtual card after its CardUsed trigger chain, or `NONE` initially.
        card: CardId,
    },
        ] }
    };
    (RevealTargetHandShaOrDiscardBind [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Resolve the legacy Weikui HP cost and hand branch, then bind a temporary distance helper.
    ///
    /// Stage zero loses HP. Later stages run only after its complete dying window, then branch
    /// on effective Shan, attempt the virtual Sha, and conditionally grant the helper.
    /// Appended for serialized ordinal stability.
    RevealTargetHandShaOrDiscardBind {
        /// Skill owner, HP-loss payer, observer, and virtual Sha user.
        player: PlayerId,
        /// Selected other player.
        target: PlayerId,
        /// Temporary target-bound helper skill definition.
        helper_skill_def: SkillDefId,
        /// Originating active skill instance.
        skill: SkillId,
        /// Engine-owned stage: 0=HP loss, 1=branch, 2=virtual Sha, 3=helper grant.
        stage: u8,
    },
        ] }
    };
    (DiscardRememberedHandCards [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Discard every remembered card that remains discardable in the owner's hand, then clear
    /// the remembered set.
    ///
    /// Appended for serialized ordinal stability.
    DiscardRememberedHandCards {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Gain one topmost draw-pile card from each selector group whose paired divisor matches the
    /// player's match-lifetime card-action count.
    ///
    /// Appended for serialized ordinal stability.
    GainDrawCardsForLifetimeCardActionDivisors {
        /// Player whose history is inspected and who gains the cards.
        player: PlayerId,
        /// Three positive divisors checked independently.
        divisors: [u32; 3],
        /// Three stable reward-group tags paired with the divisors.
        groups: [u8; 3],
    },
        ] }
    };
    (ChooseGainOwnLostDiscardCardThisTurn [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose and gain one physical card the player lost from hand/equipment this turn that
    /// still remains in the discard pile.
    ///
    /// Appended for serialized ordinal stability.
    ChooseGainOwnLostDiscardCardThisTurn {
        /// Player choosing and gaining the card.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose one equipment or delayed-trick card from another living player's field and move
    /// it into `target`'s matching empty equipment slot or judgment area.
    ///
    /// Appended for serialized ordinal stability.
    ChooseMoveFieldCardTo {
        /// Skill owner and card chooser.
        player: PlayerId,
        /// Fixed living destination.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose a zone priority, gain one deterministic-random card from the first nonempty
    /// preferred zone of each other living player in seat order, then turn over.
    ///
    /// Appended for serialized ordinal stability.
    ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
        /// Skill owner, chooser, and card recipient.
        player: PlayerId,
        /// 0 = hand/equip/judge, 1 = equip/hand/judge, 2 = judge/hand/equip, 255 = choose.
        priority: u8,
        /// Engine-owned remaining targets in seat order.
        remaining_targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose and gain one card from every other living player's hand/equipment/judgment zones
    /// in seat order, then turn over.
    ///
    /// Appended for serialized ordinal stability.
    ChooseGainCardFromEachOtherThenTurnOver {
        /// Skill owner, chooser, and card recipient.
        player: PlayerId,
        /// Whether the stable target snapshot has been initialized.
        initialized: bool,
        /// Engine-owned remaining targets in seat order.
        remaining_targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseGainTargetCardAlternateZoneThenTargetDraw [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (TakeOneHandCard [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (GainAllSkillPileCards [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (FinalizeTiaojiyanmei [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (DiscardEquipmentSlot [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Discard every equipped card in one slot.
    DiscardEquipmentSlot {
        /// Equipment owner.
        player: PlayerId,
        /// Stable equipment-slot tag.
        slot: u8,
    },
        ] }
    };
    (GainTopCardToSkillPile [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseGainCardFromEachOtherReplaceDraw [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (TakeOneScopedCard [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Let one player choose and gain one card from another player's typed zone scope.
    ///
    /// Appended for serialized ordinal stability. Unlike the legacy [`Self::TakeOneCard`],
    /// this request supports the judgment zone and uses the shared gain-protection query.
    TakeOneScopedCard {
        /// Living player who owns the offered cards.
        from: PlayerId,
        /// Living player choosing and gaining the card.
        to: PlayerId,
        /// Canonical zones offered to the chooser.
        scope: TargetCardScope,
    },
    /// Publicly reveal one player's complete current hand.
    ///
    /// Appended for serialized ordinal stability.
    RevealHand {
        /// Player whose hand is revealed.
        player: PlayerId,
    },
        ] }
    };
    (RevealTopGainIfColor [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Publicly reveal the draw-pile top card, then gain it when its printed color matches.
    /// A nonmatching card enters the discard pile.
    ///
    /// Appended for serialized ordinal stability.
    RevealTopGainIfColor {
        /// Living player revealing and possibly gaining the card.
        player: PlayerId,
        /// Printed-color mask: `1` red or `2` black.
        color_mask: u8,
    },
        ] }
    };
    (MoveCardsWithReason [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Move cards with an engine-owned semantic reason.
    ///
    /// Appended separately so existing `MoveCards` producers keep their wire shape and ordinals.
    MoveCardsWithReason {
        /// Cards in order.
        cards: Vec<CardId>,
        /// Destination zone.
        to: ZoneId,
        /// Optional explicit source (validated by core).
        from: Option<ZoneId>,
        /// Authoritative movement cause.
        reason: engine_api::CardMoveReason,
    },
        ] }
    };
    (EquipWeaponFromDrawPile [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Randomly equip one physical weapon from the shared draw pile when the slot is empty.
    ///
    /// Appended for serialized ordinal stability.
    EquipWeaponFromDrawPile {
        /// Equipment recipient.
        player: PlayerId,
    },
        ] }
    };
    (ReplayVirtualCardOnTargets [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Replay a virtual card on its original targets after the source card has fully settled.
    ReplayVirtualCardOnTargets {
        /// Card user.
        player: PlayerId,
        /// Stable logical card-kind tag.
        kind: u8,
        /// Original targets in declaration order.
        targets: Vec<PlayerId>,
        /// Skill provenance used to prevent recursive replay.
        skill: SkillId,
    },
        ] }
    };
    (DiscardHandEquipCount [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 通用弃牌数量代价（`yangzhong` 殃众等）：选择恰好 `count` 张可弃置手牌/装备
    /// 并弃置。
    ///
    /// `count == 0` 是 VM 首发（打开选牌）；选择结算后重发弃置。ABI 460。
    DiscardHandEquipCount {
        /// Player discarding the cards.
        player: PlayerId,
        /// Number of cards to discard.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (RevealChooseCategoryDistributeSelfThresholdHpLoss [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Reveal cards, choose one category, distribute all matching cards up to a per-recipient cap,
    /// then make recipients with at least the owner's share lose 1 HP. ABI 491.
    RevealChooseCategoryDistributeSelfThresholdHpLoss {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Number of draw-pile cards to reveal.
        reveal_count: u8,
        /// Maximum matching cards each recipient may receive.
        recipient_cap: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ContinueDiaobingqianjiang [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Engine-owned recoverable 调兵遣将 settlement. ABI 615.
    ContinueDiaobingqianjiang {
        /// 0=reveal after user draw; 1=advance target swaps.
        action: u8,
        /// Card user, who chooses the final top-deck order.
        actor: PlayerId,
        /// Physical or materialized card carrier.
        source_card: CardId,
        /// Targets still waiting to resolve in seat order.
        targets: Vec<PlayerId>,
        /// Currently revealed cards remaining in the processing zone.
        revealed: Vec<CardId>,
    },
        ] }
    };
    (ContinueWugu [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Engine-owned recoverable【五谷丰登】settlement. ABI 616.
    ContinueWugu {
        /// 0=reveal; 1=start target queue; 2=discard leftovers.
        action: u8,
        /// Card user.
        actor: PlayerId,
        /// Physical or materialized card carrier.
        source_card: CardId,
        /// Original targets, kept for reveal count and seat order.
        targets: Vec<PlayerId>,
        /// Targets excluded by skills before their individual resolution.
        excluded: Vec<PlayerId>,
        /// Currently revealed cards remaining in processing.
        revealed: Vec<CardId>,
    },
        ] }
    };
    (CancelIncomingCardUse [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 取消使用牌：成为其他角色使用牌的目标时，取消该牌对当前角色生效，并获取对
    /// 其使用的实体牌（gwent 杜达元帅1 特殊卡机制）。
    /// ABI 643 (ADR 0608).
    CancelIncomingCardUse {
        /// Card holder being targeted.
        player: PlayerId,
        /// Card user whose use is cancelled.
        source: PlayerId,
        /// The used card being cancelled.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (DiscardAllOwnJudge [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Discard every card currently in the player's judgment zone.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    DiscardAllOwnJudge {
        /// Player whose judgment zone is cleared.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Store the triggering event's used card (if it ended in the discard pile) onto the
    /// owner's skill pile (武将牌). Implements 灵杉 (sp/huamu).
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    StoreEventCardInSkillPile {
        /// Skill owner who stores the card.
        player: PlayerId,
        /// The triggering event's used card.
        card: CardId,
        /// Originating skill instance (defines the pile).
        skill: SkillId,
    },
        ] }
    };
    (GainTargetHandCard [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 肃州 (sp/pxsizhou): the owner selects another living player, then gains one
    /// of that player's hand cards (the owner picks the card). Two-stage choice:
    /// target selection, then card selection. Appended for serialized ordinal stability.
    GainTargetHandCard {
        /// Gaining player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ExtraResolveCurrentTrick [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 额外结算 (clanfenshi family, upstream effectCount++): queue one extra
    /// effect pass for the currently suspended trick. Must run inside the
    /// CardUsed trigger window (WuxieWindow continuation).
    ExtraResolveCurrentTrick {
        /// Owning player.
        player: PlayerId,
        /// Suspended trick source card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 置入 (expansion P2, ADR 0641): the owner selects one hand card (any kind)
    /// and places it onto their skill pile (武将牌上). Reuses the
    /// CommitStoreHandShaInSkillPile pile-insertion machinery.
    PlaceHandCardToExpansion {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (DiscardSkillPile [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 弃置 (expansion S1-B, ADR 0641 §8.2): move every card of the owner's own
    /// skill pile (武将牌上) into the discard pile.
    DiscardSkillPile {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Skill definition owning the pile (u32 literal; may be a sibling skill).
        pile_def: u32,
    },
    /// 置入 (expansion P2-var, ADR 0641 §8.7): move the top `count` cards of the
    /// draw pile into the owner's own (player, pile_def) skill pile.
    PlaceTopDeckToExpansion {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Skill definition owning the pile (u32 literal; may be a sibling skill).
        pile_def: u32,
        /// Number of top deck cards to place.
        count: u32,
    },
        ] }
    };
    (ChooseTargetHandCardToPile [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 置入他人手牌 (expansion A1, ADR 0641 §8.7): pick another player with hand
    /// cards, then one of their hand cards, into the (player, pile_def) pile.
    ChooseTargetHandCardToPile {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Skill definition owning the pile.
        pile_def: u32,
    },
    /// 堆重排 (chooseToMove family, ADR 0641 §8.8): select `count` cards from the
    /// own (player, pile_def) skill pile and place them on top of the draw pile.
    RearrangePileToDeckTop {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Skill definition owning the pile.
        pile_def: u32,
        /// Number of cards to rearrange.
        count: u32,
    },
        ] }
    };
    (ChooseHandCardsToSkillPile [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 变张置入 (A0 置入面, ADR 0641 §8.12): select between `min_count` and
    /// `max_count` cards from the own hand and place them into the own
    /// originating skill pile.
    ChooseHandCardsToSkillPile {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance (pile def source).
        skill: SkillId,
        /// Minimum number of cards to place.
        min_count: u32,
        /// Maximum number of cards to place.
        max_count: u32,
    },
    /// 堆重排 (chooseToMove family, ADR 0641 §8.8): select `count` cards from the
    /// player's hand and place them on top of the draw pile (in selection order).
    PlaceHandCardsToDeckTop {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Number of cards to select.
        count: u32,
    },
        ] }
    };
    (DiscardBasicDrawEqual [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 玉蝉引擎（ABI 657）：弃任意张基本手牌并摸等量（选择续体）。
    DiscardBasicDrawEqual {
        /// 发动者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (GainShadowCards [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 获得【影】（ABI 660，jsrg 影族）：生成 count 张影牌（黑桃 1 虚拟
    /// 基本牌）置入手牌。
    GainShadowCards {
        /// 获得者。
        player: PlayerId,
        /// 张数。
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (DistributeShadowCards [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 分发【影】（ABI 662，jsrg funi②）：选 1..N 角色共计 count 张影均分给予。
    DistributeShadowCards {
        /// 分发者。
        player: PlayerId,
        /// 总张数。
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (PlaceXumou [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 蓄谋（ABI 665，jsrg 蓄谋族）：将一张牌作为蓄谋牌置入判定区
    /// （特殊延时虚拟牌，可重复）。
    PlaceXumou {
        /// 蓄谋者。
        player: PlayerId,
        /// 置入的实体牌。
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (PlaceDeckTopXumou [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 蓄谋·牌堆顶（ABI 666，jsrg 殆谋）：将牌堆顶实体牌移入判定区作为蓄谋牌
    /// （上游 addJudge({name:"xumou_jsrg"}, get.cards())）。
    PlaceDeckTopXumou {
        /// 蓄谋者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (DiscardOneOwnXumou [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 弃置一张蓄谋牌（ABI 666，jsrg 殆谋「我」分支）：确定性弃置判定区
    /// 最后一张蓄谋牌（无 tagged 牌则静默 no-op）。
    DiscardOneOwnXumou {
        /// 弃置者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 筑城·堆顶置入扩张区（ABI 667，jsrg 沙城 build 子技）：将牌堆顶 count 张
    /// 实体牌明置移入自己的「城」扩张堆。
    PlaceDeckTopExpansion {
        /// 筑城者。
        player: PlayerId,
        /// 置入张数（常量）。
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 城族消耗（ABI 668，jsrg 沙城②）：移去一张「城」并令一名本回合失去过
    /// 牌的其他角色摸 min(5, 失去牌数) 张。
    CityConsume {
        /// 发动者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
}
pub(super) use variants;
