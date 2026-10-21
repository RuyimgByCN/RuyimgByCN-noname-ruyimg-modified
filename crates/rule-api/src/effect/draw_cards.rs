//! Ordered EffectRequest variants, starting with `DrawCards`.

macro_rules! variants {
    ([$($tail:ident),*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail),*] [
            $($variants)*
    /// Draw `count` cards for `player`.
    DrawCards {
        /// Recipient.
        player: PlayerId,
        /// Number of cards.
        count: u32,
    },
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
    /// Deal damage.
    DealDamage {
        /// Source.
        source: PlayerId,
        /// Target.
        target: PlayerId,
        /// Amount.
        amount: u32,
        /// Lose-HP channel (skips armor etc. per rules).
        is_lose_hp: bool,
        /// Damage nature: 0=normal, 1=fire, 2=thunder, 3=divine (see `damage_nature`).
        #[serde(default)]
        nature: u8,
    },
    /// Recover HP.
    RecoverHp {
        /// Recovery source (`NONE` for unattributed recovery).
        #[serde(default = "no_player")]
        source: PlayerId,
        /// Target.
        player: PlayerId,
        /// Amount.
        amount: u32,
    },
    /// Change max HP.
    ChangeMaxHp {
        /// Target.
        player: PlayerId,
        /// Delta (may be negative).
        delta: i32,
    },
    /// Add skill instance from definition.
    AddSkill {
        /// Owner.
        player: PlayerId,
        /// Skill definition.
        skill_def: SkillDefId,
    },
    /// Remove skill instance.
    RemoveSkill {
        /// Skill instance.
        skill: SkillId,
    },
    /// Add / set mark stack.
    AddMark {
        /// Player.
        player: PlayerId,
        /// Mark def.
        mark: MarkDefId,
        /// Delta.
        delta: i32,
    },
    /// Remove mark stacks.
    RemoveMark {
        /// Player.
        player: PlayerId,
        /// Mark def.
        mark: MarkDefId,
        /// Amount to remove; 0 means all.
        amount: u32,
    },
    /// Reveal cards to observers.
    RevealCards {
        /// Cards.
        cards: Vec<CardId>,
        /// Recipients; empty = public.
        to: Vec<PlayerId>,
    },
    /// Request a player choice (pauses resolution).
    RequestChoice {
        /// Observer.
        observer: PlayerId,
        /// Min selections.
        min_count: u8,
        /// Max selections.
        max_count: u8,
        /// Allow cancel.
        allow_cancel: bool,
        /// Option tokens.
        option_tokens: Vec<u32>,
    },
    /// Enter dying.
    EnterDying {
        /// Player.
        player: PlayerId,
    },
    /// Kill player.
    KillPlayer {
        /// Victim.
        player: PlayerId,
        /// Killer if any.
        killer: PlayerId,
    },
    /// Set turn phase / current player (system).
    SetTurnState {
        /// Current player.
        current: PlayerId,
        /// Phase ordinal (maps to TurnPhaseTag in core).
        phase: u8,
    },
    /// Continue normal/extra turn selection after all End-phase triggers settle.
    ///
    /// Engine-owned; content does not emit this request.
    AdvanceTurn,
    /// Engine-internal: emit a DebateResolved event (ADR 0644 P2 resolvers only).
    EmitDebateResolved {
        /// Initiator.
        initiator: PlayerId,
        /// Red-opinion players.
        red: Vec<PlayerId>,
        /// Black-opinion players.
        black: Vec<PlayerId>,
        /// Result code.
        result: u8,
    },
    /// Emit a rule note event.
    EmitRuleEvent {
        /// Event code.
        code: u32,
        /// Payload.
        payload: Vec<u32>,
    },
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
    /// Record a logical response card after its cost has been paid.
    CardResponded {
        /// Responding player.
        player: PlayerId,
        /// Player whose card is being answered; none when no response relation exists.
        #[serde(default = "no_player")]
        respond_to: PlayerId,
        /// Physical card; none for a virtual response.
        card: CardId,
        /// Logical card-kind tag after view-as conversion.
        response_kind: u8,
    },
    /// Move every card of one rule kind from live play zones into the shared
    /// removed-from-game zone.
    RemoveCardsByKind {
        /// Stable rule card-kind tag.
        kind: u8,
    },
    /// Deal damage produced by a resolved logical Sha.
    ///
    /// Appended separately so card-specific late damage policies do not affect generic damage
    /// and existing serialized effect variant ordinals remain stable.
    DealShaDamage {
        /// Source.
        source: PlayerId,
        /// Target.
        target: PlayerId,
        /// Physical card carrying the logical Sha, or none for a cardless Sha.
        #[serde(default = "no_card")]
        card: CardId,
        /// Amount after ordinary Sha-use modifiers.
        amount: u32,
        /// Damage nature: 0=normal, 1=fire, 2=thunder.
        #[serde(default)]
        nature: u8,
        /// Card-use provenance flags; see [`damage_card_flag`].
        #[serde(default)]
        card_flags: u8,
    },
    /// Ask `player` to discard exactly one card from hand or equipment.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseDiscardOneHandOrEquip {
        /// Player making the forced choice.
        player: PlayerId,
    },
    /// Perform a judgment, then gain the topmost equipment from the draw pile
    /// for red or from the discard pile for black.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeGainEquipmentByColor {
        /// Player performing the judgment and gaining the equipment.
        player: PlayerId,
    },
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
    /// Choose the discard and/or fire-damage branches, applying their conditional phase skips.
    ///
    /// The progress fields are engine-owned continuation state. Content emits them empty.
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseDiscardOrFireDamageSkipPhases {
        /// Current-turn player performing the choices.
        player: PlayerId,
        /// Maximum cards discarded from the chosen target's hand/equipment zones.
        max_discard: u8,
        /// Fire damage dealt by the damage branch.
        damage: u8,
        /// Selected branch bitset: 1=discard, 2=damage.
        #[serde(default)]
        selected: u8,
        /// Internal resume step: 0=choose branches, 1=after discard.
        #[serde(default)]
        step: u8,
        /// Step-specific target.
        #[serde(default = "no_player")]
        target: PlayerId,
    },
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
    /// Discard every card in `player`'s hand and equipment zones.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    DiscardAllHandAndEquip {
        /// Player losing all hand and equipment cards.
        player: PlayerId,
    },
    /// Open the deterministic multi-rescuer save queue for a dying player.
    ///
    /// Appended so forced dying-trigger choices can suspend this request without losing its
    /// continuation metadata.
    RequestDyingSave {
        /// Player currently dying.
        player: PlayerId,
        /// Damage source used if the player dies.
        source: PlayerId,
    },
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
    /// Let one player choose and take one card from another player's allowed zones.
    TakeOneCard {
        /// Card owner.
        from: PlayerId,
        /// Player choosing and taking the card.
        to: PlayerId,
        /// Restrict the choice to the equipment zone.
        equip_only: bool,
    },
    /// Toggle a player's character card between face-up and face-down.
    ///
    /// A face-down player skips their next whole turn and returns face-up.
    TurnOver {
        /// Player whose character card is toggled.
        player: PlayerId,
    },
    /// Remove every skill instance with one definition from a player.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    RemoveSkillByDef {
        /// Skill owner.
        player: PlayerId,
        /// Definition to remove.
        skill_def: SkillDefId,
    },
    /// Optionally discard one hand card, then add a skill selected by its effective color.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseDiscardHandAddSkillByColor {
        /// Player making the discard choice and receiving the skill.
        player: PlayerId,
        /// Skill added after discarding a red card.
        red_skill_def: SkillDefId,
        /// Skill added after discarding a black card.
        black_skill_def: SkillDefId,
    },
    /// Reveal the draw-pile top card, gain it when non-basic, or optionally
    /// replace a basic card with one hand card placed on top of the draw pile.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    RevealTopGainNonBasicOrSwapBasic {
        /// Player revealing, gaining, and optionally swapping the card.
        player: PlayerId,
    },
    /// Prevent the damage event whose pre-resolution trigger is currently running.
    ///
    /// Content emits `damage_id = 0`; the engine VM host binds the current event id.
    /// Appended so existing serialized effect variant ordinals remain stable.
    PreventDamage {
        /// Engine-owned damage event id.
        #[serde(default)]
        damage_id: u32,
    },
    /// Apply damage after its `damage_before` trigger queue has completed.
    ///
    /// Engine-owned; content bytecode cannot construct this request.
    /// Appended so existing serialized effect variant ordinals remain stable.
    CommitDamage {
        /// Engine-owned damage event id, or zero when no pre-trigger matched.
        damage_id: u32,
        /// Source.
        source: PlayerId,
        /// Target.
        target: PlayerId,
        /// Final amount after ordinary damage modifiers.
        amount: u32,
        /// Amount propagated through an elemental chain before target-side modifiers.
        base_amount: u32,
        /// Lose-HP channel.
        is_lose_hp: bool,
        /// Damage nature.
        nature: u8,
        /// Physical card that caused the damage, or none.
        #[serde(default = "no_card")]
        card: CardId,
        /// Logical card-kind tag, or zero when the damage was not card-linked.
        #[serde(default)]
        card_kind: u8,
        /// Card-use provenance flags; see [`damage_card_flag`].
        #[serde(default)]
        card_flags: u8,
    },
    /// Perform a judgment and, on a red result, exclude one target from the
    /// currently resolving Sha.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeCancelCurrentShaTargetOnRed {
        /// Player performing the judgment.
        player: PlayerId,
        /// Physical Sha whose target is being resolved.
        card: CardId,
        /// Target excluded on a red judgment.
        target: PlayerId,
    },
    /// Let a specified player choose between drawing cards and recovering HP.
    /// Full-HP players draw immediately because recovery is unavailable.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseDrawOrRecover {
        /// Player making the choice and receiving the result.
        player: PlayerId,
        /// Cards drawn by the draw branch.
        draw_count: u32,
        /// HP recovered by the recovery branch.
        recover_count: u32,
        /// Whether declining both branches is legal.
        allow_cancel: bool,
    },
    /// Damage linked to a logical card without reapplying card-specific source modifiers.
    /// Used internally for elemental chain propagation.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    DealCardDamage {
        /// Source.
        source: PlayerId,
        /// Target.
        target: PlayerId,
        /// Amount before target-side chain modifiers.
        amount: u32,
        /// Lose-HP channel.
        is_lose_hp: bool,
        /// Damage nature.
        nature: u8,
        /// Physical source card, or none.
        card: CardId,
        /// Logical card-kind tag.
        card_kind: u8,
        /// Card-use provenance flags; see [`damage_card_flag`].
        #[serde(default)]
        card_flags: u8,
    },
    /// Materialize a suitless virtual Sha and let its user choose legal targets.
    ///
    /// The card still traverses the ordinary use/response/damage pipeline and is
    /// moved to the removed zone after use. Appended so existing serialized
    /// effect variant ordinals remain stable.
    ChooseUseVirtualSha {
        /// Player using the virtual Sha.
        player: PlayerId,
        /// Whether range is ignored while target legality is otherwise preserved.
        ignore_distance: bool,
    },
    /// Use a virtual Sha with a base-damage bonus (国战克复中原：蜀势力目标出杀的
    /// 伤害值基数 +1). Appended for serialized ordinal stability.
    ChooseUseVirtualShaWithBonus {
        /// Player using the virtual Sha.
        player: PlayerId,
        /// Whether range is ignored while target legality is otherwise preserved.
        ignore_distance: bool,
        /// Base-damage bonus applied to the used Sha.
        damage_bonus: u32,
    },
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
    /// Perform a judgment and recover HP when its effective suit matches.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeRecoverOnSuit {
        /// Skill owner credited as the recovery source.
        source: PlayerId,
        /// Player performing the judgment and receiving any recovery.
        player: PlayerId,
        /// Effective suit required for recovery.
        suit: Suit,
        /// HP recovered on a matching result.
        amount: u32,
    },
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
    /// Emit the semantic completion of one Duel after its damage has resolved.
    ///
    /// Engine-owned; appended for serialized ordinal stability.
    EmitDuelResolved {
        /// Physical or materialized virtual card that carried the Duel.
        card: CardId,
        /// Player who used the Duel.
        initiator: PlayerId,
        /// Original Duel target.
        target: PlayerId,
        /// Participant who dealt Duel damage.
        winner: PlayerId,
        /// Participant who received Duel damage.
        loser: PlayerId,
    },
    /// Emit a completed judgment after every rewrite opportunity has settled.
    ///
    /// Engine-owned; appended for serialized ordinal stability.
    EmitJudgmentResolved {
        /// Player whose judgment completed.
        player: PlayerId,
        /// Physical card carrying the final judgment result.
        result_card: CardId,
        /// Final effective suit for the judged player.
        suit: Suit,
        /// Final rank.
        rank: u8,
        /// Delayed trick/equipment that caused the judgment, or `NONE`.
        cause_card: CardId,
    },
    /// Perform a judgment, then force the judging player to damage another
    /// living player when the effective suit matches.
    ///
    /// Appended for serialized ordinal stability.
    JudgeDamageOtherOnSuit {
        /// Player performing the judgment and dealing any resulting damage.
        player: PlayerId,
        /// Effective suit required for the target choice.
        suit: Suit,
        /// Damage dealt on a matching result.
        amount: u32,
        /// Stable damage nature tag.
        nature: u8,
    },
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
    /// Let `player` use a suitless virtual Sha without consuming the ordinary Sha-use count;
    /// if no legal use is selected, draw one card instead.
    ///
    /// The target choice still applies ordinary distance and target-legality rules. Appended for
    /// serialized ordinal stability.
    ChooseUseVirtualShaOrDraw {
        /// Player choosing between a legal virtual Sha use and drawing one card.
        player: PlayerId,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
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
    /// Make one target unable to respond to the currently resolving physical Sha.
    ///
    /// The request mutates only the saved in-flight Sha continuation. It never installs a
    /// persistent modifier, and is rejected when the source/card/target tuple does not match
    /// that continuation. Appended for serialized ordinal stability.
    MakeCurrentShaTargetUnrespondable {
        /// Sha user and skill owner.
        source: PlayerId,
        /// Physical Sha currently resolving.
        card: CardId,
        /// Current Sha target whose response window is disabled.
        target: PlayerId,
    },
    /// Record that one target fully dodged the currently resolving logical Sha.
    ///
    /// This engine-internal intent emits the typed `ShaDodged` domain event only after every
    /// required Shan has resolved. Appended for serialized ordinal stability.
    ShaDodged {
        /// Sha user.
        source: PlayerId,
        /// Target who dodged the Sha.
        target: PlayerId,
        /// Current logical Sha carrier (which may be a view-as source card), or `NONE` for a
        /// source-free virtual Sha.
        card: CardId,
    },
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
    /// Perform a judgment and, on a non-heart result, prevent the currently resolving damage
    /// and reduce the damaged player's maximum HP by one.
    ///
    /// Content emits `damage_id = 0`; the engine VM host binds the current pre-damage event.
    /// Appended for serialized ordinal stability.
    JudgePreventDamageLoseMaxHpOnNonHeart {
        /// Skill owner performing the judgment.
        player: PlayerId,
        /// Damaged player whose maximum HP may be reduced.
        target: PlayerId,
        /// Engine-owned damage event id.
        #[serde(default)]
        damage_id: u32,
    },
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
    /// Repeatedly let one player choose another living player, then draw for the selected
    /// target using one of two counts according to whether that target's hand is empty.
    ///
    /// Every iteration is independently cancellable. Appended for serialized ordinal stability.
    ChooseOtherDrawByHandCount {
        /// Player making each target choice.
        player: PlayerId,
        /// Cards drawn when the target has at least one hand card.
        normal_count: u32,
        /// Cards drawn when the target has no hand cards.
        empty_count: u32,
        /// Remaining independently cancellable choices.
        times: u32,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// Optionally discard one hand/equipment card, then gain every equipped card in one slot
    /// from `target` that still exists when the cost resolves.
    ///
    /// The discard selection and target equipment are revalidated at the continuation barrier.
    /// Appended for serialized ordinal stability.
    ChooseDiscardOneThenGainTargetEquipment {
        /// Player choosing and paying the discard cost.
        player: PlayerId,
        /// Player whose matching equipment is gained.
        target: PlayerId,
        /// Stable equipment-slot tag (`1` weapon through `5` treasure).
        slot: u8,
    },
    /// Let one player choose a living target from a typed scope, then make that target draw.
    ///
    /// Appended for serialized ordinal stability.
    ChooseTargetDraw {
        /// Player making the target choice.
        player: PlayerId,
        /// Cards drawn by the selected target.
        count: u32,
        /// Canonical target scope.
        scope: DrawTargetScope,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
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
    /// Refund the currently resolving proactive Sha's use count at most once, then let the
    /// skill owner draw if that Sha is effectively red.
    ///
    /// The current saved Sha continuation, source, and physical card are revalidated by the
    /// engine. Appended for serialized ordinal stability.
    RefundCurrentShaUseAndDrawIfRed {
        /// Skill owner receiving the possible draw.
        player: PlayerId,
        /// Player who used the currently resolving Sha.
        sha_user: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// Force one hand/equipment-card discard, then refund the currently resolving proactive
    /// Sha's use count at most once and draw for the skill owner if that Sha is effectively red.
    ///
    /// The engine rechecks the cost options when this effect starts and revalidates the selected
    /// card at the choice barrier. The refund/draw follow-up is scheduled only after payment.
    /// Appended for serialized ordinal stability.
    ChooseDiscardOneForShaRefundAndDrawIfRed {
        /// Skill owner choosing and paying the discard cost.
        player: PlayerId,
        /// Player who used the currently resolving Sha.
        sha_user: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// Queue one extra Play phase at the opening of the specified player's current turn.
    ///
    /// The engine executes queued phases before Prepare and gives every Play phase an independent
    /// Sha-use allowance and play-phase counters. Appended for serialized ordinal stability.
    QueueOpeningExtraPlayPhase {
        /// Player receiving the extra Play phase.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal turn-opening boundary. Appended for serialized ordinal stability.
    BeginTurn {
        /// Player whose turn begins.
        player: PlayerId,
    },
    /// Engine-internal continuation after all turn-begin triggers resolve.
    FinishTurnBegin {
        /// Current player.
        player: PlayerId,
    },
    /// Engine-internal transition after an opening extra Play phase ends.
    AdvanceOpeningExtraPlay {
        /// Current player.
        player: PlayerId,
    },
    /// Convert the currently saved ordinary Sha to fire and attach its post-use HP-loss rule.
    ///
    /// The source/card tuple must match the proactive Sha continuation. Appended for serialized
    /// ordinal stability.
    ConvertCurrentShaToFireLoseHpIfDamage {
        /// Sha user.
        player: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let the Sha user optionally add one legal living target to the currently saved Sha.
    ///
    /// The original target list, selected target, range, and card policies are revalidated at the
    /// choice barrier. Appended for serialized ordinal stability.
    ChooseAddCurrentShaTarget {
        /// Sha user and chooser.
        player: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal commit after the optional Sha target choice is revalidated.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    CommitAddCurrentShaTarget {
        /// Sha user.
        player: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Newly selected target.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose one or more logical Sha from hand and store them in this skill's public pile.
    /// Appended for serialized ordinal stability.
    ChooseStoreHandShaInSkillPile {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose one card from this skill's public pile, discard it, then make the dying player use
    /// one suitless virtual Jiu on themselves. Appended for serialized ordinal stability.
    ChooseSpendSkillPileForDyingJiu {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Current dying player who will use Jiu.
        dying: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal commit of selected hand Sha into a skill pile.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    CommitStoreHandShaInSkillPile {
        /// Skill owner.
        player: PlayerId,
        /// Revalidated selected hand Sha.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal commit of one skill-pile card for a dying player's virtual Jiu.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    CommitSpendSkillPileForDyingJiu {
        /// Skill owner.
        player: PlayerId,
        /// Current dying player.
        dying: PlayerId,
        /// Selected skill-pile card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Add a skill whose versioned instance state binds it to one player.
    /// Appended for serialized ordinal stability.
    AddBoundSkill {
        /// Skill owner.
        player: PlayerId,
        /// Skill definition.
        skill_def: SkillDefId,
        /// Bound target encoded in the new instance state.
        bound_target: PlayerId,
    },
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
    /// Optionally discard one non-basic hand/equipment card whose color matches one of the
    /// chooser's equipment cards, then grant a color-lock helper to `target`.
    /// Appended for serialized ordinal stability.
    ChooseDiscardMatchingEquipColorAddSkill {
        /// Skill owner choosing and paying the cost.
        player: PlayerId,
        /// Other player's current Play phase receiving the helper.
        target: PlayerId,
        /// Temporary color-lock helper definition.
        skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Add or update a color-lock helper. Repeated grants merge color masks and replace the
    /// stored source with the latest grant.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    AddCardColorLockSkill {
        /// Player receiving the helper.
        player: PlayerId,
        /// Latest skill source used by the helper's follow-up.
        source: PlayerId,
        /// Bit 0 = red, bit 1 = black.
        color_mask: u8,
        /// Temporary helper definition.
        skill_def: SkillDefId,
    },
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
    /// Add or merge exact-card policies in a temporary helper skill.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    AddCardUsePolicySkill {
        /// Helper owner and card user.
        player: PlayerId,
        /// Physical card receiving the policies.
        card: CardId,
        /// Bit 0 = ignore armor, bit 1 = unrespondable, bit 2 = draw per damage event.
        policy_mask: u8,
        /// Temporary helper definition.
        skill_def: SkillDefId,
    },
    /// Force a choice between drawing one card and discarding one hand/equipment card, then
    /// grant the branch-specific temporary skill.
    /// Appended for serialized ordinal stability.
    ChooseDrawOrDiscardAddSkill {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Temporary skill granted before drawing one card.
        draw_skill_def: SkillDefId,
        /// Temporary skill granted before discarding one card.
        discard_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
        /// Offer the cancellable plain-draw-one / draw-two / discard three-way contract.
        #[serde(default)]
        three_way: bool,
    },
    /// Repeatedly let the owner use one black basic card from the triggering discard batch as
    /// a distance-unlimited Bingliang. Each iteration is optional and revalidated.
    /// Appended for serialized ordinal stability.
    ChooseDiscardedBlackBasicAsBingliang {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Physical cards captured from the triggering move batch by the engine VM host.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Optionally discard one judgment card from `target`, then make `player` use a source-free
    /// Sha on that target and draw one card if that exact Sha deals no damage.
    /// Appended for serialized ordinal stability.
    ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player whose judgment phase is beginning.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal commit of one fixed virtual Sha with a skill-attributed no-damage draw.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    UseVirtualShaDrawIfNoDamage {
        /// Virtual Sha user.
        player: PlayerId,
        /// Fixed Sha target.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Let the owner choose whether the target draws or discards the originating skill's current
    /// turn use count, then link the owner if the target is wounded.
    /// Appended for serialized ordinal stability.
    ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Player affected by the branch.
        target: PlayerId,
        /// Toggle linked state instead of only linking the owner.
        #[serde(default)]
        toggle_link: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal finalization after the selected draw/discard branch and its nested triggers.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    FinishLinkIfTargetWounded {
        /// Player to link if currently unlinked.
        player: PlayerId,
        /// Player whose wounded state is tested after the branch.
        target: PlayerId,
        /// Toggle linked state instead of only linking the owner.
        #[serde(default)]
        toggle_link: bool,
        /// Originating skill instance for attribution.
        skill: SkillId,
    },
    /// Let `player` choose another living player with hand cards, choose one of those cards,
    /// reveal it publicly, and remember its logical category for the current Play phase.
    /// Appended for serialized ordinal stability.
    ChooseRevealOtherHandCardRememberCategory {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Clear this skill instance's delayed Play-phase gift target.
    /// Appended for serialized ordinal stability.
    ClearPhaseGiftTarget {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Emit the engine-owned boundary for one fully resolved logical Sha use.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    EmitShaResolved {
        /// Sha user.
        source: PlayerId,
        /// Physical Sha carrier or engine materialization.
        card: CardId,
        /// Complete declared target list.
        targets: Vec<PlayerId>,
        /// Physical Shan source cards in response order.
        response_cards: Vec<CardId>,
        /// Stable card-use flags carried to the post-resolution trigger context.
        card_flags: u8,
    },
    /// Choose a legal recipient, give that recipient the resolved Sha or its physical Shan
    /// sources still in discard, then optionally use one physical Sha source if the gift is red.
    /// Appended for serialized ordinal stability.
    ChooseGainResolvedShaOrShanThenOptionalUseSha {
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
    /// Engine-internal continuation after the gifted cards and their movement triggers finish.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    ChooseUseShaAfterResolvedGift {
        /// Original skill owner whose attack range constrains the follow-up Sha.
        owner: PlayerId,
        /// Gift recipient who may use a Sha.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Skip the owner's current non-idle phase and enter the next phase.
    /// Appended for serialized ordinal stability.
    SkipCurrentPhase {
        /// Current phase owner.
        player: PlayerId,
    },
    /// Choose up to the current turn's skipped-phase count of other players and deal damage to
    /// them in the submitted target order.
    /// Appended for serialized ordinal stability.
    ChooseOtherTargetsBySkippedPhaseCountDamage {
        /// Skill owner and damage source.
        player: PlayerId,
        /// Damage dealt to each selected target.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Choose and discard one effective equipment card from hand/equipment, then reduce the
    /// bound pre-resolution damage by one.
    /// Appended for serialized ordinal stability.
    ChooseDiscardEquipmentReduceCurrentDamage {
        /// Damaged skill owner and chooser.
        player: PlayerId,
        /// Engine-owned pre-resolution damage id.
        #[serde(default)]
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose and discard one effective equipment card from hand/equipment, then prevent the
    /// entire bound pre-resolution damage segment.
    ChooseDiscardEquipmentPreventCurrentDamage {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Engine-owned pre-resolution damage id.
        #[serde(default)]
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose to move one compatible equipment card from `target` to `player`, draw one card,
    /// or cancel. Appended for serialized ordinal stability.
    ChooseMoveTargetEquipmentOrDraw {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player whose turn just ended.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Give all of the owner's current equipment to `target`, then let that target choose the
    /// count-bound recover-or-damage branch. Appended for serialized ordinal stability.
    GiveAllEquipmentChooseRecoverOrDamage {
        /// Limited active skill owner.
        player: PlayerId,
        /// Recipient and later branch chooser.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal branch choice after the equipment gift and nested movement triggers.
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    ChooseRecoverOwnerOrDamageByGivenEquipment {
        /// Original equipment owner and possible recovery target.
        owner: PlayerId,
        /// Equipment recipient, branch chooser, recovery source, and damage source.
        recipient: PlayerId,
        /// Number of equipment cards captured before the gift.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let the target discard exactly the source's current physical weapon range from hand. If
    /// the target cannot or declines, increase the bound pre-resolution Sha damage by one.
    /// Appended for serialized ordinal stability.
    ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
        /// Sha source and originating skill owner.
        player: PlayerId,
        /// Bound damage target and discard chooser.
        target: PlayerId,
        /// Engine-owned pre-resolution damage id.
        #[serde(default)]
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose a living player whose hand count differs from the owner. The target draws one when
    /// lower, or discards one authoritative deterministic-random hand card when higher.
    /// Appended for serialized ordinal stability.
    ChooseHandCountDifferentTargetDrawOrRandomDiscard {
        /// Skill owner and target chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Increase the pending card count for the current draw phase. The draw-phase command
    /// consumes the accumulated value exactly once.
    /// Appended for serialized ordinal stability.
    IncreaseCurrentDrawCount {
        /// Current draw-phase player.
        player: PlayerId,
        /// Additional cards.
        count: u32,
    },
    /// During the owner's Prepare phase, choose a living target and discard the target's
    /// hand-minus-HP amount (at least one) from hand/equipment. The discarded non-equipment
    /// count becomes the optional self-discard cost; declining makes the target draw instead.
    /// Appended for serialized ordinal stability.
    ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
        /// Skill owner and target/card chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal follow-up after the target discard and its nested movement triggers.
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    ChooseSelfDiscardOrTargetDrawAfterTargetDiscard {
        /// Skill owner who may pay the discard cost.
        player: PlayerId,
        /// Original target who draws when the owner declines or cannot pay.
        target: PlayerId,
        /// Number of non-equipment cards discarded from the target.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Engine-internal sequential gain continuation. `selected = NONE` scans for the next
    /// eligible seat and opens a choice; a concrete card performs the gain and defers the next
    /// scan until all nested movement triggers finish. Rule VM bytecode never emits this.
    /// Appended for serialized ordinal stability.
    ContinueChooseGainOneFromDamageTargetAndNeighbors {
        /// Skill owner and card chooser.
        player: PlayerId,
        /// Snapshotted legacy order, including intentional duplicate seats.
        targets: Vec<PlayerId>,
        /// Current index in `targets`.
        index: u32,
        /// Selected card to gain, or NONE while scanning/opening the next choice.
        selected: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Force the damage source to choose between preventing the bound damage or losing one
    /// maximum HP and increasing that same damage by the requested amount.
    /// Content emits damage_id = 0; the engine VM host binds the current pre-damage event.
    /// Appended for serialized ordinal stability.
    ChoosePreventDamageOrLoseMaxHpIncreaseDamage {
        /// Damage source, skill owner, and chooser.
        player: PlayerId,
        /// Engine-owned pre-resolution damage id.
        #[serde(default)]
        damage_id: u32,
        /// Amount added to both the pending damage and its base amount.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal continuation that increases a still-bound pending damage after the
    /// maximum-HP loss and all nested max-HP-change triggers have resolved.
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    ContinueIncreaseBoundDamage {
        /// Original damage source.
        player: PlayerId,
        /// Engine-owned pre-resolution damage id.
        damage_id: u32,
        /// Amount added to both the pending damage and its base amount.
        amount: u32,
        /// Originating skill instance for provenance.
        skill: SkillId,
    },
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
    /// Emit the completed hand-comparison boundary after both physical cards have been moved.
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    EmitHandCompareFinished {
        /// First comparison participant.
        player: PlayerId,
        /// Second comparison participant.
        target: PlayerId,
        /// First participant's comparison card.
        player_card: CardId,
        /// Second participant's comparison card.
        target_card: CardId,
        /// Strict winner, or `NONE` for a tie.
        winner: PlayerId,
    },
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
    /// Choose another living player with hand cards, then let both participants choose one hand
    /// card for a comparison. Appended for serialized stability.
    ChooseHandCompare {
        /// First participant, skill owner, and chooser.
        player: PlayerId,
        /// Helper granted on strict victory.
        win_skill_def: SkillDefId,
        /// Helper granted on tie or loss.
        lose_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose one other living player for a hand comparison; on the chooser's strict win the
    /// **target** is granted `win_grant_def`, on tie or loss nothing happens. Appended for
    /// serialized stability (ABI 678, batch 166 tanhu).
    ChooseHandCompareGrantTargetOnWin {
        /// First participant, skill owner, and chooser.
        player: PlayerId,
        /// Helper granted to the target on the chooser's strict victory.
        win_grant_def: SkillDefId,
        /// Reserved for symmetry with `ChooseHandCompare`; unused in the grant flow.
        lose_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Third-party hand comparison (ABI 680, batch 174 jun_zhaoshou): `first` picks an
    /// opponent (excluding `player`), both stash one hand card, and on a non-tie result the
    /// winner uses a distance-free virtual Sha against the loser.
    ChooseHandCompareBetween {
        /// Skill owner / originating observer.
        player: PlayerId,
        /// First comparison participant (also picks the opponent).
        first: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Watch the top `count` cards of the draw pile and re-order them one by one: each
    /// round the player chooses top-sequence or bottom-sequence for the current card
    /// (ABI 681, batch 176 guanxing family). Multi-round state lives in the continuation
    /// payload.
    GuanxingTop {
        /// Viewer.
        player: PlayerId,
        /// Number of top cards to watch and re-order.
        count: u32,
        /// Round cursor (0-based); carried through the continuation payload.
        round: u32,
        /// Flipped cards in draw order.
        flipped: Vec<CardId>,
        /// Cards placed into the top sequence (in pick order).
        top: Vec<CardId>,
        /// Cards placed into the bottom sequence (in pick order).
        bottom: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
}
pub(super) use variants;
