//! Ordered StmtIr variants, starting with `EffectRecoverToMaxThenOptionalExchangeSkill`.

macro_rules! variants {
    ([$($tail:ident),*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail),*] [
            $($variants)*
    /// effect recover_to_max_then_optional_exchange_skill(player, removable_skill_def)
    ///
    /// Consume the originating limited skill, recover `player` to max HP, then, after recovery
    /// and nested triggers finish, optionally exchange `removable_skill_def` for a fresh instance
    /// of the originating skill. Appended for serialized ordinal stability.
    EffectRecoverToMaxThenOptionalExchangeSkill {
        /// Skill owner recovering and choosing the exchange.
        player: ExprIr,
        /// Skill definition sacrificed when the limited skill is restored.
        removable_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect reveal_hand_discard_kind(player, card_kind)
    ///
    /// Publicly reveal the player's current hand, then discard every discardable hand card whose
    /// effective kind matches `card_kind`. Appended for serialized ordinal stability.
    EffectRevealHandDiscardKind {
        /// Player revealing and discarding cards.
        player: ExprIr,
        /// Stable logical card-kind tag.
        kind: i32,
        /// Source span.
        span: Span,
    },
    /// effect choose_move_target_equipment_and_or_draw(player, target)
    ///
    /// Let the owner cancel, draw one, move one compatible equipment card from `target`, or
    /// perform both available actions in move-then-draw order.
    /// Appended for serialized ordinal stability.
    EffectChooseMoveTargetEquipmentAndOrDraw {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Other player whose turn ended.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(
    /// player, target)
    ///
    /// Let a direct-Sha damage target discard up to the source's current attack range from
    /// hand/equipment or increase that damage by one. A physical weapon forces both branches
    /// and is discarded afterward.
    /// Appended for serialized ordinal stability.
    EffectChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
        /// Skill owner and Sha source.
        player: ExprIr,
        /// Bound Sha damage target and chooser.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_draw_or_recast_add_skill(player, draw_skill_def, recast_skill_def)
    ///
    /// Let the player cancel, draw one and gain `draw_skill_def`, or recast one hand/equipment
    /// card and gain `recast_skill_def`.
    /// Appended for serialized ordinal stability.
    EffectChooseDrawOrRecastAddSkill {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Temporary skill granted before drawing.
        draw_skill_def: u32,
        /// Temporary skill granted before recasting.
        recast_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect choose_gain_sha_for_opening_extra_play(player)
    ///
    /// During the originating skill's turn-opening Play phase, let the owner gain the first Sha
    /// from the draw pile or discard pile and arm its phase-end consequence.
    /// Appended for serialized ordinal stability.
    EffectChooseGainShaForOpeningExtraPlay {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect finish_opening_extra_play_sha_gain(player)
    ///
    /// Clear the originating skill's armed exact-card policy and damage the owner when they dealt
    /// no damage since accepting the gain.
    /// Appended for serialized ordinal stability.
    EffectFinishOpeningExtraPlayShaGain {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect reveal_target_hand_gain_shan_armor_or_draw_two(player, target)
    ///
    /// Publicly reveal the target's hand. Give every effective Shan and physical armor to the
    /// skill owner; if none match, the owner draws two.
    /// Appended for serialized ordinal stability.
    EffectRevealTargetHandGainShanArmorOrDrawTwo {
        /// Skill owner and recipient.
        player: ExprIr,
        /// Other living player whose hand is revealed.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect give_all_hand(player, target)
    ///
    /// Give every current hand card from `player` to `target`.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGiveAllHand {
        /// Living player giving every hand card.
        player: ExprIr,
        /// Other living recipient.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect gain_card_if_discard(player, card)
    ///
    /// Move `card` from the shared discard pile into `player`'s hand if it is still there.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGainCardIfDiscard {
        /// Living recipient.
        player: ExprIr,
        /// Physical event card.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_draw_or_discard_target_card_record_sha(player, card, target)
    ///
    /// Let the current Sha user cancel, draw one, or discard one target hand/equipment card.
    /// Accepting either branch records the exact Sha/target for a later dodge consequence.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDrawOrDiscardTargetCardRecordSha {
        /// Sha user and chooser.
        player: ExprIr,
        /// Exact resolving Sha.
        card: ExprIr,
        /// Current Sha target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect reveal_hand_to_player(player, observer)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRevealHandToPlayer {
        /// Player whose current hand is revealed.
        player: ExprIr,
        /// Player authorized to see that hand.
        observer: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_optional_discard_target_card(
    ///     player,
    ///     target,
    ///     hand|equip|hand_equip|hand_equip_judge,
    /// )
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseOptionalDiscardTargetCard {
        /// Player choosing the card.
        player: ExprIr,
        /// Player whose scoped card may be discarded.
        target: ExprIr,
        /// Canonical zone scope.
        scope: TargetCardScopeIr,
        /// Source span.
        span: Span,
    },
    /// effect reveal_hand_choose_discard_color(player)
    ///
    /// Publicly reveal the player's current hand, then let that player choose one effective
    /// red/black color present and discard every discardable hand card of that color. A single
    /// present color resolves without a choice. Appended for serialized ordinal stability.
    EffectRevealHandChooseDiscardColor {
        /// Player revealing, choosing, and discarding.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_target_cards_recover_same_category(player, target, count, amount)
    ///
    /// Force `player` to discard up to `count` current hand/equipment cards from `target`.
    /// Recover `amount` HP after movement triggers only when exactly `count` cards were
    /// discarded and their effective categories for `target` are identical. Appended for
    /// serialized ordinal stability.
    EffectChooseDiscardTargetCardsRecoverSameCategory {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Player whose cards are discarded.
        target: ExprIr,
        /// Maximum discard count.
        count: ExprIr,
        /// Maximum HP recovered after a matching discard.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_scskuiji_discard(player, target, count): chooser picks up to `count`
    /// distinct-effective-suit cards from its own and `target`'s hands combined; all are discarded.
    EffectChooseScskuijiDiscard {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Other player whose hand also participates.
        target: ExprIr,
        /// Maximum number of cards chosen.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_pskuiji_discard(player, target, count): like `choose_scskuiji_discard`, but
    /// when owner and target discard different counts, the under-discarder loses 1 HP.
    EffectChoosePskuijiDiscard {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Other player whose hand also participates.
        target: ExprIr,
        /// Maximum number of cards chosen.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_compare_selected_hand_card_second_failure_seat_damage(
    ///     player, target, draw_count, failure_count, lose_hp, damage
    /// )
    ///
    /// The active selection supplies the owner's comparison card. Strict victory chooses between
    /// drawing and discarding one target hand/equipment card. At the configured non-win
    /// threshold, the owner loses HP and damages the shorter seat path between the first two
    /// failed targets. Appended for serialized ordinal stability.
    EffectChooseCompareSelectedHandCardSecondFailureSeatDamage {
        /// Skill owner and first comparison participant.
        player: ExprIr,
        /// Other comparison participant.
        target: ExprIr,
        /// Strict-win draw count.
        draw_count: ExprIr,
        /// Non-win threshold.
        failure_count: ExprIr,
        /// HP loss at the threshold.
        lose_hp: ExprIr,
        /// Damage per chosen seat-path player.
        damage: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_recover(player, amount)
    ///
    /// Force `player` to choose another living player, then recover that target's HP.
    /// Appended for serialized ordinal stability.
    EffectChooseOtherRecover {
        /// Player making the forced target choice.
        player: ExprIr,
        /// Maximum HP recovered by the selected target.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect enable_equip_slot(player, slot, source)
    ///
    /// Restore one disabled copy of an equipment slot for `player`, attributed to `source`.
    /// Appended for serialized ordinal stability.
    EffectEnableEquipSlot {
        /// Player whose equipment slot is restored.
        player: ExprIr,
        /// Stable slot tag (`1` weapon through `5` treasure).
        slot: ExprIr,
        /// Player causing the restoration.
        source: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect target_choose_suit_gain_owner_hand_damage_if_mismatch(player, target)
    ///
    /// `target` chooses a suit, gains one current hand card of `player`, then takes 1 damage
    /// from `player` when that card's owner-effective suit differs. Appended for serialized
    /// ordinal stability.
    EffectTargetChooseSuitGainOwnerHandDamageIfMismatch {
        /// Skill owner whose hand card is gained.
        player: ExprIr,
        /// Other living player making both choices.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect judge_source_discard_two_hand_or_damage(player, source)
    ///
    /// `player` judges. On a non-heart result, the living `source` may discard exactly two
    /// hand cards; otherwise `player` deals 1 normal damage to `source`. Appended for
    /// serialized ordinal stability.
    EffectJudgeSourceDiscardTwoHandOrDamage {
        /// Damaged skill owner performing the judgment.
        player: ExprIr,
        /// Damage source paying the discard or taking damage.
        source: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect give_selected_recover_after_phase_total_two(player, target)
    ///
    /// Give the active invocation's selected hand cards to `target`; when the current Play-phase
    /// total first reaches two cards, recover `player` by 1 after movement triggers. Appended
    /// for serialized ordinal stability.
    EffectGiveSelectedRecoverAfterPhaseTotalTwo {
        /// Skill owner giving selected hand cards.
        player: ExprIr,
        /// Other living recipient.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_hand_targets_gain_one_replace_draw(player)
    ///
    /// During the owner's Draw phase, choose one or two other players with hand cards, gain one
    /// hand card from each in selection order, and replace the normal draw. Appended for
    /// serialized ordinal stability.
    EffectChooseOtherHandTargetsGainOneReplaceDraw {
        /// Skill owner, target chooser, and card chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect distribute_top_two_cards(player)
    ///
    /// Move the draw-pile top two cards into public processing, then repeatedly assign a
    /// nonempty subset to any living player until all are distributed. Appended for serialized
    /// ordinal stability.
    EffectDistributeTopTwoCards {
        /// Skill owner choosing every subset and recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect judge_make_current_sha_target_unrespondable_on_red(player, card, target)
    ///
    /// Appended for serialized ordinal stability.
    EffectJudgeMakeCurrentShaTargetUnrespondableOnRed {
        /// Player performing the judgment.
        player: ExprIr,
        /// Physical Sha currently resolving.
        card: ExprIr,
        /// Current Sha target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_basic_then_target_discard_equip_or_damage(player, target)
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseDiscardBasicThenTargetDiscardEquipOrDamage {
        /// Skill owner paying the optional basic-card cost and dealing damage.
        player: ExprIr,
        /// End-phase player discarding equipment or taking damage.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_current_sha_target_discard_redirect_to_owner(owner, source, card, target)
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseCurrentShaTargetDiscardRedirectToOwner {
        /// Skill owner receiving the redirected Sha.
        owner: ExprIr,
        /// Current Sha user.
        source: ExprIr,
        /// Physical Sha currently resolving.
        card: ExprIr,
        /// Current target paying the optional discard cost.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_gain_bottom_card(player)
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseTargetGainBottomCard {
        /// Skill owner revealing and giving the bottom card.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect judge_black_gain_repeat(player)
    ///
    /// Appended for serialized ordinal stability.
    EffectJudgeBlackGainRepeat {
        /// Judged player and recipient of black results.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect use_virtual_juedou_second_on_first()
    ///
    /// Appended for serialized ordinal stability.
    EffectUseVirtualJuedouSecondOnFirst {
        /// Source span.
        span: Span,
    },
    /// effect recover_hp_as(source, player, amount)
    ///
    /// Appended for serialized ordinal stability.
    EffectRecoverAs {
        /// Recovery source.
        source: ExprIr,
        /// Player recovering HP.
        player: ExprIr,
        /// Amount expression.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_redirect_current_sha(player, source, card)
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseDiscardRedirectCurrentSha {
        /// Current Sha target and skill owner paying the discard cost.
        player: ExprIr,
        /// Current Sha user.
        source: ExprIr,
        /// Physical Sha currently resolving.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect judge_set_skill_mark_to_rank(player, skill)
    ///
    /// Appended for serialized ordinal stability.
    EffectJudgeSetSkillMarkToRank {
        /// Player performing the judgment.
        player: ExprIr,
        /// Originating skill instance whose definition id is used as the mark id.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_one_from_each_linked(player, skill)
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseDiscardOneFromEachLinked {
        /// Skill owner choosing cards.
        player: ExprIr,
        /// Originating skill instance.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect disable_skill_by_def_until_turn_end(player, skill_def)
    ///
    /// Appended for serialized ordinal stability.
    EffectDisableSkillByDefUntilTurnEnd {
        /// Skill owner.
        player: ExprIr,
        /// Skill definition to disable temporarily.
        skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect target_discard_then_duel_if_non_sha_and_hp_gte_owner(player, target)
    ///
    /// Appended for serialized ordinal stability.
    EffectTargetDiscardThenDuelIfNonShaAndHpGteOwner {
        /// Skill owner and virtual Duel user.
        player: ExprIr,
        /// Other player choosing and discarding one hand/equipment card.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_lose_hp_discard_hand_sha_if_basic(player)
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseTargetLoseHpDiscardHandShaIfBasic {
        /// Skill owner and virtual Sha user.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_optional_targets_draw(player, count, min_count, max_count, scope)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseOptionalTargetsDraw {
        /// Player making the target choice.
        player: ExprIr,
        /// Cards drawn by each selected target.
        count: ExprIr,
        /// Minimum number of distinct targets.
        min_count: u8,
        /// Maximum number of distinct targets.
        max_count: u8,
        /// Canonical target scope.
        scope: DrawTargetScopeIr,
        /// Source span.
        span: Span,
    },
    /// effect reveal_target_hand_sha_or_discard_bind(player, target, helper_skill_def)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRevealTargetHandShaOrDiscardBind {
        /// Skill owner and virtual Sha user.
        player: ExprIr,
        /// Selected other player.
        target: ExprIr,
        /// Temporary target-bound helper definition.
        helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect draw_remembered_hand_cards(player, count)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDrawRememberedHandCards {
        /// Player drawing and remembering the cards.
        player: ExprIr,
        /// Number of cards to draw.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect discard_remembered_hand_cards(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDiscardRememberedHandCards {
        /// Player discarding remembered cards still in hand.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Gain one topmost draw-pile card from each selector group whose paired divisor matches the
    /// player's match-lifetime card-action count.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGainDrawCardsForLifetimeCardActionDivisors {
        /// Player whose history is inspected and who gains the cards.
        player: ExprIr,
        /// Three positive divisors checked independently.
        divisors: [u32; 3],
        /// Three stable reward-group tags paired with the divisors.
        groups: [u8; 3],
        /// Source span.
        span: Span,
    },
    /// During the originating turn-opening extra Play phase, lose 1 HP and then gain the
    /// topmost matching discard-pile card. Owning the supplied skill definition makes the
    /// action mandatory; otherwise it is optional.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectLoseHpGainDiscardCardForOpeningExtraPlay {
        /// Skill owner, HP-loss target, and card recipient.
        player: ExprIr,
        /// Stable card-kind tag.
        kind: i32,
        /// Owning this definition makes the action mandatory.
        mandatory_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Choose and gain one physical card the player lost from hand/equipment this turn that
    /// still remains in the discard pile.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseGainOwnLostDiscardCardThisTurn {
        /// Skill owner, chooser, and card recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose one equipment or delayed-trick card from another living player's field and move it
    /// to a fixed living destination.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseMoveFieldCardTo {
        /// Skill owner and card chooser.
        player: ExprIr,
        /// Fixed destination.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Draw cards, then optionally distribute up to a bounded number of current hand cards among
    /// other living players.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDrawThenOptionallyDistributeHandCards {
        /// Skill owner, drawer, chooser, and giver.
        player: ExprIr,
        /// Cards drawn before distribution choices begin.
        draw_count: ExprIr,
        /// Maximum total number of hand cards that may be distributed.
        give_cap: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Give the selected hand cards, then optionally repeat for distinct other recipients and
    /// recover once when the activation's total first reaches two.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGiveSelectedThenContinueDistinctRecoverAtTwo {
        /// Skill owner and giver.
        player: ExprIr,
        /// First active-skill target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose one living other player tied for the highest positive mark, judge that target, and
    /// kill them unless the physical result is Tao or Taoyuan.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
        /// Dead skill owner and chooser.
        player: ExprIr,
        /// Mark definition used to rank living other players.
        mark_def: u32,
        /// Source span.
        span: Span,
    },
    /// Choose a zone priority, gain one deterministic-random card from every other living
    /// player's first nonempty preferred zone in seat order, then turn over.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
        /// Skill owner, chooser, and card recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose and gain one hand/equipment/judgment card from each other living player in seat
    /// order, then turn over.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseGainCardFromEachOtherThenTurnOver {
        /// Skill owner, chooser, and card recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Grant a skill until the current turn ends.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectAddSkillUntilTurnEnd {
        /// Skill recipient.
        player: ExprIr,
        /// Skill definition to grant.
        skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Consume marks and resolve ordered all-other damage/discard/turn-over semantics.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveMassDamageDiscardTurnOver {
        /// Skill owner and damage source.
        player: ExprIr,
        /// Mark definition consumed.
        mark_def: u32,
        /// Mark count consumed.
        mark_count: u32,
        /// Damage dealt to every other player.
        damage_amount: u32,
        /// Maximum hand cards discarded by every other player.
        hand_count: u32,
        /// Source span.
        span: Span,
    },
    /// Consume marks, damage every other living player, then make each discard a bounded
    /// number of hand/equipment cards.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveMassDamageDiscardHandEquip {
        /// Skill owner and damage source.
        player: ExprIr,
        /// Mark definition consumed.
        mark_def: u32,
        /// Mark count consumed.
        mark_count: u32,
        /// Damage dealt to every other player.
        damage_amount: u32,
        /// Maximum hand/equipment cards discarded by every other player.
        discard_count: u32,
        /// Source span.
        span: Span,
    },
    /// Record the selected target for a target-only turn uniqueness contract.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRecordActiveTarget {
        /// Skill owner.
        player: ExprIr,
        /// Selected target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose and pay either one discardable physical weapon or one HP.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDiscardWeaponOrLoseHp {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve a mark-parity target action and optional clear-all mass damage.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveMarkParityControl {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Mark definition controlling the effect.
        mark_def: u32,
        /// Clear-all option threshold.
        mass_threshold: u32,
        /// Normal damage amount.
        damage_amount: u32,
        /// Source span.
        span: Span,
    },
    /// Discard selected targets' equipment, then choose one selected survivor to damage.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSelectedEquipmentDiscardThenChooseDamage {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Damage amount.
        damage_amount: u32,
        /// Damage nature tag.
        nature: u8,
        /// Source span.
        span: Span,
    },
    /// Replace the pending normal draw with a top-card distinct-suit selection.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectReplaceDrawWithDistinctSuitSelection {
        /// Draw-phase player.
        player: ExprIr,
        /// Top-card reveal count.
        count: u8,
        /// Source span.
        span: Span,
    },
    /// Inspect a target hand and choose one heart card to discard or put on the draw pile.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectInspectTargetHandChooseHeartDiscardOrTop {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Inspected active-skill target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve the complete limited Yeyan choice and ordered fire damage.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveYeyan {
        /// Skill owner and damage source.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose whether every living player loses or recovers one HP, or cancel.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseAllLoseHpOrRecover {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Initialize or exchange the current skill's public Qixing pile.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveQixing {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Whether to initialize the pile from the draw pile before exchanging.
        initialize: bool,
        /// Source span.
        span: Span,
    },
    /// Spend cards from a Qixing pile and grant one source-bound weather helper per target.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveQixingWeather {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Qixing skill definition owning the public pile.
        qixing_skill_def: u32,
        /// Temporary target-owned helper skill definition.
        helper_skill_def: u32,
        /// Maximum number of cards and targets selected.
        max_targets: u8,
        /// Source span.
        span: Span,
    },
    /// Toggle Longnu, pay the matching HP cost, draw, and grant the phase helper.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveNzryLongnu {
        /// Skill owner.
        player: ExprIr,
        /// Red-card-to-fire-Sha helper.
        fire_skill_def: u32,
        /// Trick-card-to-thunder-Sha helper.
        thunder_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Force the owner to choose another living unlinked player and link them.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseLinkOther {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose and discard four distinct-suit cards across the owner and target hands, then
    /// resolve the owner-card-count branch.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDrltPoxi {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Inspected active-skill target.
        target: ExprIr,
        /// Temporary hand-limit helper definition.
        hand_limit_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve one Jieying lifecycle branch.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDrltJieying {
        /// Jieying owner.
        player: ExprIr,
        /// Player carried by the triggering event.
        event_player: ExprIr,
        /// Stable 营 mark definition.
        mark: u32,
        /// 0 = ensure mark, 1 = transfer, 2 = settle.
        mode: u8,
        /// Source span.
        span: Span,
    },
    /// Resolve one Duorui lifecycle branch.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDrltDuorui {
        /// Skill owner.
        player: ExprIr,
        /// Damaged target or cleanup event player.
        target: ExprIr,
        /// Target-side cleanup helper definition.
        helper_skill_def: u32,
        /// 0 = acquire, 3 = cleanup.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// Resolve one Zhiti restoration trigger.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDrltZhiti {
        /// Skill owner.
        player: ExprIr,
        /// Event player.
        event_player: ExprIr,
        /// Event source.
        event_source: ExprIr,
        /// Event scalar.
        event_amount: ExprIr,
        /// 0 = hand comparison, 1 = Duel, 2 = damage.
        mode: u8,
        /// Source span.
        span: Span,
    },
    /// Resolve one Shen Simayi shared branch.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveShenSimayi {
        /// Skill owner.
        player: ExprIr,
        /// Event player.
        event_player: ExprIr,
        /// Event scalar.
        event_amount: ExprIr,
        /// Stable 忍 mark definition.
        mark: u32,
        /// Package branch 0..=4.
        mode: u8,
        /// Source span.
        span: Span,
    },
    /// Resolve Cai Wenji's Beige cost and one four-suit judgment.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveBeige {
        /// Beige owner.
        player: ExprIr,
        /// Damaged player.
        target: ExprIr,
        /// Damage source.
        source: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect damage_then_discard_all_judge(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDamageThenDiscardAllJudge {
        /// Player damaged and whose judgment zone is eventually emptied.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect replace_draw_gain_top_two_add_skills_if_different_colors(player, first_skill_def, second_skill_def)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
        /// Draw-phase player and card recipient.
        player: ExprIr,
        /// First temporary skill definition.
        first_skill_def: u32,
        /// Second temporary skill definition.
        second_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect choose_gain_target_card_alternate_zone_then_target_draw(player, target)
    ///
    /// Gain one current hand/equipment card from `target`, excluding the zone already used by
    /// this skill in the current Play phase, then let `target` draw one card. Appended to keep
    /// existing serialized enum ordinals stable.
    EffectChooseGainTargetCardAlternateZoneThenTargetDraw {
        /// Skill owner gaining the card.
        player: ExprIr,
        /// Other living target losing one card, then drawing.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Mark a living player's next judgment phase to be skipped.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectSkipNextJudgePhase {
        /// Player whose next judgment phase is skipped.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose and discard one hand/equipment card from either endpoint of the current damage;
    /// increase that damage when the card's owner-effective suit matches.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDiscardDamageEndpointCardIncreaseIfSuit {
        /// Damage source, skill owner, and chooser.
        player: ExprIr,
        /// Current damage target.
        target: ExprIr,
        /// Stable suit tag (`1` spade through `4` diamond).
        suit: u8,
        /// Damage added after the discard resolves.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose and gain one hand card from the first player bound to the current event.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectTakeOneEventTargetHand {
        /// Player choosing and gaining the card.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose an extra Draw or Play phase after End, or execute both when used-card suits reach HP.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseEndExtraDrawOrPlayByUsedSuits {
        /// Current turn player and skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Lose HP equal to the bound damage, double that damage, then replace this skill.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectLoseHpDoubleCurrentDamageThenReplace {
        /// Damage source, HP-loss target, and skill owner.
        player: ExprIr,
        /// Permanent replacement skill definition.
        replacement_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve one reusable hidden skill-pile action.
    ///
    /// Actions: 0 initialize two hidden top cards; 1 reveal one and exclude the current card
    /// target; 2 exchange hand cards with hidden pile cards; 3 reveal one and draw the number of
    /// revealed pile cards; 4 draw for triggering lost cards whose suits match revealed cards.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveHiddenSkillPile {
        /// Skill owner.
        player: ExprIr,
        /// Definition that owns the expansion pile.
        pile_skill_def: u32,
        /// Stable action tag.
        action: u8,
        /// Current event card; ignored by actions other than target exclusion.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Enable or clear the current skill as the turn's non-locked-skill suppression source.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectSetNonLockedSkillSuppression {
        /// Skill owner and current turn player.
        player: ExprIr,
        /// Whether to enable (`true`) or clear (`false`) suppression.
        enabled: bool,
        /// Source span.
        span: Span,
    },
    /// Resolve EXTRA 帝力/驭衡 and its derived fate-line actions.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDiliYuheng {
        /// Skill owner.
        player: ExprIr,
        /// Stable action tag.
        action: u8,
        /// Bound card for card-use or equipment actions.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Discard the active invocation's selected hand/equipment cards, then draw that many cards
    /// plus one if every current hand card was selected.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDiscardSelectedDrawAllHandBonus {
        /// Skill owner and draw recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose another player for 国战英魂's lost-HP draw/discard branch.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseOtherYinghun {
        /// Wounded skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose another player for 弼政's draw and post-draw over-limit discards.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseOtherBizheng {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 霞姝's give-all, reveal-partition, and gain-partition choices.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseOtherXiashu {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve one 观潮 card rank against the preceding Play-phase card and invalidate the mode
    /// after the first monotonicity violation.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveGuanchaoRank {
        /// Skill owner.
        player: ExprIr,
        /// Current used card.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 观微's discard, draw-two, and immediate extra Play phase.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveGuanwei {
        /// Skill owner.
        player: ExprIr,
        /// Player whose Play phase ended.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 揽江's snapshotted sequential choices.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveLanjiang {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 秉壹's full-hand reveal and conditional draws.
    EffectResolveRebingyi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Queue 博图's round-bounded extra turn.
    EffectResolveRebotu {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 旧缔盟's pair selection/swap or Play-end discard.
    EffectResolveOldimeng {
        /// Skill owner.
        player: ExprIr,
        /// Zero starts the active swap; one settles the Play-end discard.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// Resolve 界结姻 using the active invocation's selected card.
    EffectResolveRejieyin {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 仁恤's ordered two-target card transfer and draw checks.
    EffectResolveReanxu {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 决堰's equipment-slot choice and permanent/temporary branch benefit.
    EffectResolveDrltJueyan {
        /// Skill owner.
        player: ExprIr,
        /// Permanent +1 Sha-use helper.
        sha_skill_def: u32,
        /// Temporary +3 Sha-use helper.
        weapon_skill_def: u32,
        /// Unlimited-distance helper.
        mount_skill_def: u32,
        /// Hand-limit +3 helper.
        armor_skill_def: u32,
        /// 界集智 helper.
        treasure_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve 界集智's draw and optional basic-card discard.
    EffectResolveRejizhi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 诫训's snapshotted draw/discard counts and terminal replacement.
    EffectResolveJiexun {
        /// Skill owner.
        player: ExprIr,
        /// 复难 enhancement helper.
        funan_helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Grant 问卦's active helper to every living player.
    EffectResolveWenguaSetup {
        /// 问卦 owner.
        player: ExprIr,
        /// Global active helper.
        helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve one 问卦 helper activation.
    EffectResolveWengua {
        /// Active actor.
        player: ExprIr,
        /// 问卦 main skill.
        main_skill_def: u32,
        /// Per-Play-phase recipient marker.
        marker_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve 新安国's target-first and owner-second minimum-state actions.
    EffectResolveXinanguo {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 国战三版刚烈's enemy choice and judgment.
    EffectResolveGanglieThree {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 忠义 activation, damage bonus, or round cleanup.
    EffectResolveZhongyi {
        /// Skill owner.
        player: ExprIr,
        /// Stable action tag.
        action: u8,
        /// Temporary helper definition.
        helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Discard every equipped card in one slot.
    EffectDiscardEquipmentSlot {
        /// Equipment owner.
        player: ExprIr,
        /// Stable equipment-slot tag.
        slot: u8,
        /// Source span.
        span: Span,
    },
    /// Resolve old 安恤's unequal-hand two-target transfer.
    EffectResolveOldAnxu {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Move active invocation cards into the originating skill's expansion pile.
    EffectStoreSelectedInSkillPile {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// End an identity match for the player's camp when a skill pile reaches a threshold.
    EffectEndMatchForCampIfSkillPileAtLeast {
        /// Camp representative.
        player: ExprIr,
        /// Definition owning the checked expansion pile.
        pile_skill_def: u32,
        /// Minimum pile size.
        count: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve old 寝情's target discard/draw sequence.
    EffectResolveOldQinqing {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve old 明鉴's hand transfer and inserted play-only phase.
    EffectResolveOldMingjian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve old 旋风's virtual-Sha or distance-one damage branch.
    EffectResolveOldXuanfeng {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Discard two cards to prevent the bound damage for 剑柔.
    EffectResolveJianrou {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve old 贿生's reveal and source choice.
    EffectResolveOldHuisheng {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve old 解烦's prompted Sha or bound recovery.
    EffectResolveOldJiefan {
        /// Skill owner.
        player: ExprIr,
        /// Engine-owned stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// Resolve 彻骨's cancellation and discard branches.
    EffectResolveChegu {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 归心's lord-skill acquisition or faction change.
    EffectResolveJunkguixin {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 谶纬's alternating face-down-card branches.
    EffectResolveYaoChenwei {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Offer one additional current-Sha target within a final-distance cap.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseAddCurrentShaTargetWithinDistance {
        /// Current Sha user and chooser.
        player: ExprIr,
        /// Physical Sha currently resolving.
        card: ExprIr,
        /// Inclusive final-distance cap.
        max_distance: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Refund the currently saved proactive Sha's ordinary use count.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRefundCurrentShaUse {
        /// Current Sha user.
        player: ExprIr,
        /// Physical Sha currently resolving.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Damage each selected active target, then let that target draw, in seat order.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDamageThenDrawSelectedTargets {
        /// Damage source.
        source: ExprIr,
        /// Damage dealt to each target.
        damage: ExprIr,
        /// Cards drawn by each target afterward.
        draw: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 游敌's target-owned discard and conditional counter-gain.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveYoudi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 狂斧's equipment selection and move/discard branch.
    EffectResolveKuangfu {
        /// Skill owner.
        player: ExprIr,
        /// Damaged target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 缮甲's draw, forced discard, and optional virtual Sha.
    EffectResolveShanjia {
        /// Skill owner.
        player: ExprIr,
        /// Lifetime equipment-use count.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 新涯角's reveal, recipient, and category-mismatch discard.
    EffectResolveNewYajiao {
        /// Skill owner.
        player: ExprIr,
        /// Card just used or responded with.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 定品's target judgment and color-dependent result.
    EffectResolveDingpin {
        /// Skill owner.
        player: ExprIr,
        /// Selected target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose living recipients for 破虏, increase its stable activation mark once, then make
    /// every selected recipient draw the previous mark plus one card in seat order.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseRepoluTargetsDraw {
        /// Skill owner and chooser, who may already be dead.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 过论's hidden target-hand selection, optional owner-card exchange, and draw.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveXinfuGuolun {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Selected other player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Discard one card for the current Sha refund/red draw, restoring a skill on equal ranks.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
        /// Skill owner choosing the discard cost and receiving the possible draw.
        player: ExprIr,
        /// Player who used the currently resolving Sha.
        sha_user: ExprIr,
        /// Physical card carrying the logical Sha.
        card: ExprIr,
        /// Skill definition restored only when the paid card and Sha have equal ranks.
        restore_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve Beige's cost, judgment, rewrite window, and final suit branch with configurable
    /// Heart recovery and Diamond draw counts.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveBeigeConfigured {
        /// Beige owner.
        player: ExprIr,
        /// Damaged player.
        target: ExprIr,
        /// Damage source.
        source: ExprIr,
        /// Heart recovery amount.
        heart_recover_amount: ExprIr,
        /// Diamond draw count.
        diamond_draw_count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Let a dead 追忆 owner optionally choose a living non-killer recipient.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveZhuiyi {
        /// Dead skill owner and choice observer.
        player: ExprIr,
        /// Credited killer, or no player for a source-free death.
        killer: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 界镇军 without changing the legacy `rezhenjun` host effect or continuation stages.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDecadezhenjun {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve SP 骁果 without changing Standard `stdxiaoguo`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpXiaoguo {
        /// Skill owner and chooser.
        player: ExprIr,
        /// End-phase player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Replace the pending normal draw by gaining one card from each other living player.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseGainCardFromEachOtherReplaceDraw {
        /// Skill owner, chooser, and card recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Give the active invocation's selected hand cards, then recover its target after movement
    /// events and nested triggers finish.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGiveSelectedThenRecoverTarget {
        /// Skill owner and giver.
        player: ExprIr,
        /// Selected living recipient.
        target: ExprIr,
        /// Recovery amount.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Active selection: give chosen cards to the target, then the chooser recovers
    /// (ABI 679).
    EffectGiveSelectedThenRecoverSelf {
        /// Chooser and giver.
        player: ExprIr,
        /// Gift recipient.
        target: ExprIr,
        /// Chooser recovery amount.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Third-party hand comparison with winner-vs-loser virtual Sha (ABI 680).
    EffectChooseHandCompareBetween {
        /// Skill owner and originating observer.
        player: ExprIr,
        /// First comparison participant (picks the opponent).
        first: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Watch and re-order the top `count` cards one by one (ABI 681).
    EffectGuanxingTop {
        /// Viewer.
        player: ExprIr,
        /// Number of top cards.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Falu-marked guanxing: N = non-zero count among four declared marks (ABI 683).
    EffectGuanxingTopByFaluMarks {
        /// Viewer.
        player: ExprIr,
        /// Four falu mark defs.
        marks: [u32; 4],
        /// Source span.
        span: Span,
    },
    /// Choose a giver who stashes one hand card to the chooser, then recovers (ABI 682).
    EffectChooseOtherGiveThenRecover {
        /// Chooser and gift recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve SP 定叛's draw, target branch, equipment movement, and delayed damage.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDingpan {
        /// Skill owner and possible damage source.
        player: ExprIr,
        /// Selected equipment-bearing target and branch chooser.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve OFFLINE 贪赂's frozen HP-gap branch, gift, damage, and delayed discard.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveTanlu {
        /// Skill owner, gift recipient, and possible damage source.
        player: ExprIr,
        /// Current-turn player making every forced choice.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Judge repeatedly while results match the first judgment's effective color.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectJudgeSameColorGainRepeat {
        /// Judged player and successful-card recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
        ] }
    };
}
pub(super) use variants;
