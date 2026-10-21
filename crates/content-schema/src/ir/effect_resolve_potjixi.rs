//! Ordered StmtIr variants, starting with `EffectResolvePotjixi`.

macro_rules! variants {
    ([$($tail:ident),*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail),*] [
            $($variants)*
    /// 势邓艾「急袭」记录/结算效果。ABI 543。
    EffectResolvePotjixi {
        /// Skill owner.
        player: ExprIr,
        /// Event target for record actions.
        target: ExprIr,
        /// Public DSL action (0..=2).
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 势陈矫「策断」复合效果。ABI 544。
    EffectResolvePotceduan {
        /// Skill owner.
        player: ExprIr,
        /// Selected active target.
        target: ExprIr,
        /// Public DSL action (0..=1).
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 势太史慈「酣战/战烈/振锋」复合效果。ABI 548。
    EffectResolvePottaishici {
        /// Skill owner.
        player: ExprIr,
        /// Selected/event target.
        target: ExprIr,
        /// Public DSL action (0..=5).
        action: u8,
        /// 酣战 definition.
        hanzhan_def: u32,
        /// 战烈 definition.
        zhanlie_def: u32,
        /// Source span.
        span: Span,
    },
    /// 势钟会「俟资/挟志/狱难/克昌」复合效果。ABI 553。
    EffectResolveMbzhonghui {
        /// Skill owner.
        player: ExprIr,
        /// Event participant.
        target: ExprIr,
        /// Public DSL action (0..=7).
        action: u8,
        /// Event amount.
        amount: ExprIr,
        /// 俟资 definition.
        mbsizi_def: u32,
        /// 克昌 definition.
        mbkechang_def: u32,
        /// Source span.
        span: Span,
    },
    /// 势辛宪英「诫节」复合效果。ABI 554。
    EffectResolvePotjiejie {
        /// Skill owner.
        player: ExprIr,
        /// Current Play-phase actor.
        actor: ExprIr,
        /// 诫节已用 helper definition.
        used_def: u32,
        /// Suit-unlimited helper definitions in spade/heart/club/diamond order.
        suit_defs: [u32; 4],
        /// 清识 definition.
        qingshi_def: u32,
        /// Source span.
        span: Span,
    },
    /// 势辛宪英「清识」复合效果。ABI 554。
    EffectResolvePotqingshi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 势魏延【忠傲/壮誓/饮战/狂骨】复合效果。ABI 558。
    EffectResolvePotweiyan {
        /// Skill owner.
        player: ExprIr,
        /// Event target.
        target: ExprIr,
        /// Bound card.
        card: ExprIr,
        /// Public DSL action 0..=10.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 势周瑜【炽沄/焰洄/焚涛/雄姿】复合效果。ABI 561。
    EffectResolvePotzhouyu {
        /// Skill owner.
        player: ExprIr,
        /// Event target.
        target: ExprIr,
        /// Bound card.
        card: ExprIr,
        /// Public DSL action 0..=7.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 势吕壹【隳律/颂谳/势施】复合效果。ABI 564。
    EffectResolvePotlvyi {
        /// Skill owner.
        player: ExprIr,
        /// Event actor/source.
        target: ExprIr,
        /// Triggering physical card.
        card: ExprIr,
        /// Public DSL action 0..=9.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 势曹爽【典易/奢权/渐专/溺尊】复合效果。ABI 568。
    EffectResolvePotcaoshuang {
        /// Skill owner.
        player: ExprIr,
        /// Event source/target.
        target: ExprIr,
        /// Triggering physical card.
        card: ExprIr,
        /// Public DSL action 0..=11.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// Choose another living player and deal normal damage to that player.
    EffectChooseOtherDamage {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Damage amount.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose another living player, then choose and discard one of that player's cards.
    EffectChooseOtherDiscardTargetCard {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Canonical target-card zone scope.
        scope: TargetCardScopeIr,
        /// Source span.
        span: Span,
    },
    /// Gain the topmost card of one rule kind from the discard pile only.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGainDiscardCardByKind {
        /// Player gaining the card.
        player: ExprIr,
        /// Stable card-kind tag.
        kind: i32,
        /// Source span.
        span: Span,
    },
    /// Reveal all hand cards and, when their effective color is uniform, choose any living
    /// player and deal normal damage to that player.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRevealUniformHandChooseDamage {
        /// Skill owner, hand owner, damage source, and chooser.
        player: ExprIr,
        /// Damage amount.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose and gain one matching card from the discard pile.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseGainDiscardCardByKind {
        /// Player choosing and gaining the card.
        player: ExprIr,
        /// Stable card-kind tag.
        kind: i32,
        /// Source span.
        span: Span,
    },
    /// Draw one card for each supplied divisor of the player's match-lifetime card-action count.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDrawForLifetimeCardActionDivisors {
        /// Player whose action history is inspected and who draws the cards.
        player: ExprIr,
        /// Three positive divisors checked independently.
        divisors: [u32; 3],
        /// Source span.
        span: Span,
    },
    /// Force the owner to choose one definition they do not yet own, then grant it.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseAddMissingSkill {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Three candidate skill definitions.
        skill_defs: [u32; 3],
        /// Source span.
        span: Span,
    },
    /// Force a player to choose a fixed number of distinct living targets, then each draws.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseTargetsDraw {
        /// Player making the target choice.
        player: ExprIr,
        /// Cards drawn by each selected target.
        count: ExprIr,
        /// Exact number of distinct targets.
        target_count: u8,
        /// Canonical target scope.
        scope: DrawTargetScopeIr,
        /// Source span.
        span: Span,
    },
    /// Gain every card captured by the triggering event that is still in the discard pile.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGainEventCards {
        /// Player gaining the cards.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_draw_to_max_hp_cap(player, cap)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseTargetDrawToMaxHpCap {
        /// Player making the target choice.
        player: ExprIr,
        /// Inclusive cap applied to the selected target's maximum HP.
        cap: u32,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_draw_owner_lost_hp_turn_over(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseOtherDrawOwnerLostHpTurnOver {
        /// Skill owner making the target choice and supplying the lost-HP draw count.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect judge_recover_other_on_suit(judge_player, recover_player, suit, amount)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectJudgeRecoverOtherOnSuit {
        /// Player performing the judgment.
        judge_player: ExprIr,
        /// Player receiving any recovery.
        recover_player: ExprIr,
        /// Stable suit tag: 1=spade, 2=heart, 3=club, 4=diamond.
        suit: u8,
        /// HP recovered when the effective judgment suit matches.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect force_others_use_sha_on_nearest_or_lose_hp(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectForceOthersUseShaOnNearestOrLoseHp {
        /// Skill owner excluded from the ordered forced-use queue.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect restore_character_state(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRestoreCharacterState {
        /// Player whose character card is restored to unlinked and face-up.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect discard_all_hand_equip_judge(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDiscardAllHandEquipJudge {
        /// Player whose hand, equipment, and judgment zones are emptied.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_target_card_resolve_current_sha(source, card, target)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDiscardTargetCardResolveCurrentSha {
        /// Current Sha user and chooser.
        source: ExprIr,
        /// Physical Sha currently resolving.
        card: ExprIr,
        /// Current Sha target whose card may be discarded.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect use_virtual_card_on_selected_targets(player, card_kind)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectUseVirtualCardOnSelectedTargets {
        /// Virtual card user.
        player: ExprIr,
        /// Stable card-kind tag.
        kind: i32,
        /// Source span.
        span: Span,
    },
    /// effect deal_damage_ban_active_if_dying(source, target, amount, skill)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDealDamageBanActiveIfDying {
        /// Damage source.
        source: ExprIr,
        /// Damage target.
        target: ExprIr,
        /// Base damage amount.
        amount: ExprIr,
        /// Active skill instance banned when this damage enters dying.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_reclaim_equipment_draw_if_range_reduced(player, target, skill)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseTargetReclaimEquipmentDrawIfRangeReduced {
        /// Skill owner and equipment-card chooser.
        player: ExprIr,
        /// Other player reclaiming one equipped card into their own hand.
        target: ExprIr,
        /// Originating skill instance.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_use_virtual_card(player, card_kind, skill)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseUseVirtualCard {
        /// Virtual card user.
        player: ExprIr,
        /// Stable card-kind tag.
        kind: i32,
        /// Originating skill instance.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect increase_current_sha_base_damage(player, card, amount)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectIncreaseCurrentShaBaseDamage {
        /// Current Sha user.
        player: ExprIr,
        /// Physical card backing the current Sha.
        card: ExprIr,
        /// Amount added to the saved base damage.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect skip_next_play_phase(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectSkipNextPlayPhase {
        /// Current turn player whose upcoming Play phase is skipped.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_draw_or_recover(player, draw_count, recover_count, times)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseOtherDrawOrRecover {
        /// Player choosing a different living player on each iteration.
        player: ExprIr,
        /// Cards drawn by the selected target's draw branch.
        draw_count: ExprIr,
        /// HP recovered by the selected target's recovery branch.
        recover_count: ExprIr,
        /// Number of independently cancellable target choices.
        times: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_hand_equip_then_remove_self_skill(player, count)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDiscardHandEquipThenRemoveSelfSkill {
        /// Player making the forced card choice.
        player: ExprIr,
        /// Requested discard count before capping to available cards.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect draw_selected_targets(count)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDrawSelectedTargets {
        /// Cards drawn by each active-skill target.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect recover_if_min_current_hp(player, amount)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRecoverIfMinCurrentHp {
        /// Potential recovery recipient.
        player: ExprIr,
        /// Maximum HP recovered.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect increase_current_card_base_damage(player, card, amount)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectIncreaseCurrentCardBaseDamage {
        /// Current damaging-card user.
        player: ExprIr,
        /// Physical card backing the current use.
        card: ExprIr,
        /// Amount added to the saved base damage.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect lose_hp_then_damage_other_lose_max_hp_if_dying(player, target, amount, skill)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectLoseHpThenDamageOtherLoseMaxHpIfDying {
        /// Skill owner who first loses HP and later sources the damage.
        player: ExprIr,
        /// Other player damaged after the owner's dying window resolves.
        target: ExprIr,
        /// Shared HP-loss and damage amount.
        amount: ExprIr,
        /// Originating active skill instance.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect recover_selected_targets(amount)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRecoverSelectedTargets {
        /// HP recovered by each active-skill target.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect record_selected_play_phase_category(skill)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRecordSelectedPlayPhaseCategory {
        /// Originating active skill instance.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect take_target_hand_then_repeat_until_max(player, target, skill)
    ///
    /// The first target is mandatory and every later larger-hand target choice is optional.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectTakeTargetHandThenRepeatUntilMax {
        /// Skill owner choosing and gaining cards.
        player: ExprIr,
        /// Mandatory target selected by the active-skill command.
        target: ExprIr,
        /// Originating active skill instance.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_draw_or_increase_current_card_damage_or_both_mark(
    /// player, card, draw_count, damage_amount, mark_def, mark_count)
    ///
    /// The third branch performs both earlier branches and then adds the requested mark.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
        /// Current card user and skill owner.
        player: ExprIr,
        /// Physical card backing the current use.
        card: ExprIr,
        /// Cards drawn by the first and third branches.
        draw_count: ExprIr,
        /// Base damage added by the second and third branches.
        damage_amount: ExprIr,
        /// Stable mark definition added by the third branch.
        mark_def: u32,
        /// Mark count added by the third branch.
        mark_count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect draw_then_choose_give_or_discard_hand(player, draw_count, payment_count)
    ///
    /// Draw first, then choose to give the exact hand-card count to another living player or
    /// discard that many hand cards. Appended to keep existing serialized enum ordinals stable.
    EffectDrawThenChooseGiveOrDiscardHand {
        /// Skill owner drawing and paying cards.
        player: ExprIr,
        /// Cards drawn before the branch choice.
        draw_count: ExprIr,
        /// Exact hand-card payment count when available.
        payment_count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect gain_target_equipment(player, target, slot)
    ///
    /// Gain every card currently occupying the requested equipment slot on `target`.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGainTargetEquipment {
        /// Player gaining the equipment.
        player: ExprIr,
        /// Player whose matching equipment is gained.
        target: ExprIr,
        /// Stable equipment-slot tag (`1` weapon through `5` treasure).
        slot: u8,
        /// Source span.
        span: Span,
    },
    /// effect discard_all_target_equipment(actor, target)
    ///
    /// Discard every equipment card that `actor` may currently discard from `target`.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDiscardAllTargetEquipment {
        /// Player causing the discard.
        actor: ExprIr,
        /// Player whose discardable equipment is discarded.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect increase_current_sha_target_damage(source, card, target, amount)
    ///
    /// Increase only the specified target's damage for the currently resolving Sha.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectIncreaseCurrentShaTargetDamage {
        /// Current Sha user.
        source: ExprIr,
        /// Physical card backing the current Sha.
        card: ExprIr,
        /// Current Sha target.
        target: ExprIr,
        /// Additional damage for this target.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_draw_then_owner_if_below_max_hp(player, count)
    ///
    /// Choose any living player to draw `count` cards, then draw one for the chooser if the
    /// selected target's hand remains below their maximum HP.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseTargetDrawThenOwnerIfBelowMaxHp {
        /// Skill owner making the target choice.
        player: ExprIr,
        /// Cards drawn by the selected target.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_bound_source_attack_range_damage(player, source, amount)
    ///
    /// Let `player` choose one living player in the bound `source` player's attack range, then
    /// deal normal damage from `source` to the selected player.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseBoundSourceAttackRangeDamage {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Bound damage source whose attack range limits the target choice.
        source: ExprIr,
        /// Damage amount.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_draw_max_hp_cap_then_discard_excess(player, cap)
    ///
    /// Choose any living player with positive maximum HP. That target draws the lesser of their
    /// current maximum HP and `cap`, then discards excess hand cards down to that same
    /// freshly-evaluated limit after nested draw triggers finish.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseTargetDrawMaxHpCapThenDiscardExcess {
        /// Skill owner making the target choice, including a dead owner resolving a death trigger.
        player: ExprIr,
        /// Inclusive upper bound applied to the selected target's maximum HP.
        cap: u32,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_draw_to_max_hp_cap_then_add_skill(player, cap, skill_def)
    ///
    /// Choose any living player with positive maximum HP. That target draws the lesser of their
    /// current maximum HP and `cap`, then gains the skill defined by `skill_def` after the draw
    /// finishes. ABI 455 (C 段 omp, `beizhan`). Appended to keep existing ordinals stable.
    EffectChooseTargetDrawMaxHpCapThenAddSkill {
        /// Skill owner making the target choice.
        player: ExprIr,
        /// Inclusive upper bound applied to the selected target's maximum HP.
        cap: u32,
        /// Skill definition granted to the selected target after the draw.
        skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_heart_redirect_current_damage(player, draw_skill_def)
    ///
    /// In a `damage_before` trigger, discard a heart hand card, choose another living player and
    /// redirect the current damage to them (cancel + re-deal with the same source/amount/nature/
    /// card), granting `draw_skill_def` to the new target. ABI 456 (C 段 omp, `tianxiang`).
    EffectResolveTianxiang {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Skill granted to the redirect target after the damage settles.
        draw_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_heart_cancel_damage_branch(player)
    ///
    /// In a `damage_before` trigger, discard a heart hand card, choose another living player,
    /// cancel the current damage, then the owner chooses: the target takes 1 damage from the
    /// damage source and draws min(5, lost HP), or loses 1 HP and gains the discarded heart card.
    /// ABI 457 (C 段 omp, `retianxiang`).
    EffectResolveRetianxiang {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_link_other_same_hp(player)
    ///
    /// In the owner's prepare phase, choose any number of other living players with the same HP
    /// as the owner who are not linked, and link them. ABI 458 (C 段 omp, `lianzhou`).
    EffectResolveLianzhou {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_give_hand_equip_to_event_player(player)
    ///
    /// Choose any number of discardable hand/equipment cards and give them to the triggering
    /// event player. ABI 459 (C 段 omp, `stddechong`).
    EffectResolveStddechong {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect discard_hand_equip_count(player, count)
    ///
    /// Choose exactly `count` discardable hand/equipment cards and discard them (generic
    /// discard-cost primitive). ABI 460 (C 段 omp, `yangzhong`).
    EffectDiscardHandEquipCount {
        /// Player discarding the cards.
        player: ExprIr,
        /// Number of cards to discard.
        count: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_jieliang_gain(player)
    ///
    /// When the current phase player discards cards during their discard phase, choose one
    /// of those cards still in the discard pile and gain it. ABI 461 (C 段 omp, `jieliang`).
    EffectResolveJieliangGain {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_gain_event_cards(player)
    ///
    /// Choose another living player; they gain the triggering event's used cards. ABI 462
    /// (C 段 omp, `stdjujian`).
    EffectResolveStdjujian {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_lulve(player)
    ///
    /// Choose another player with fewer hand cards; they choose: give all hand cards and flip
    /// the owner, or flip themselves and use a virtual Sha on the owner. ABI 463 (C 段 omp,
    /// `lulve`).
    EffectResolveLulve {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dctingxian(player)
    ///
    /// When the owner's Sha is resolving, draw (equipment count + 1) cards and exclude up to
    /// that many of the Sha's targets. ABI 464 (C 段 omp, `dctingxian`).
    EffectResolveDctingxian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_add_skill(player, skill_def)
    ///
    /// Choose another living player and grant them the given skill definition. ABI 465
    /// (C 段 omp, `hfjieying`).
    EffectResolveHfjieying {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Temp skill definition granted to the recipient.
        grant_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcnuchen(player)
    ///
    /// Reveal one of a target's hand cards, then discard same-suit cards for equal damage
    /// or gain the target's same-suit hand. ABI 466 (C 段 omp, `dcnuchen`).
    EffectResolveDcnuchen {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcdanji(player, skill)
    ///
    /// Awakening resolution: remove the skill, lose 1 maxHp, then recover+draw to the new
    /// maxHp and gain mashu + dcnuchen. ABI 466 (C 段 omp, `dcdanji`).
    EffectResolveDcdanji {
        /// Skill owner.
        player: ExprIr,
        /// Awakening skill instance to remove.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dczhiheng(player)
    ///
    /// Discard any number of hand/equip cards, draw the same count, plus one if every hand
    /// card was discarded. ABI 467 (C 段 omp, `dczhiheng`).
    EffectResolveDczhiheng {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_stdgaobian(player)
    ///
    /// At another player's end phase, when exactly one player took damage this turn, that
    /// player uses a virtual Sha or resets the owner's Anran counter. ABI 468
    /// (C 段 omp, `stdgaobian`).
    EffectResolveStdgaobian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spjiedao(player)
    ///
    /// Before the owner's first damage of the turn, optionally boost the damage by X and
    /// discard X hand/equip cards. ABI 469 (C 段 omp, `spjiedao`).
    EffectResolveSpjiedao {
        /// Skill owner and damage source.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_sibian(player)
    ///
    /// At the draw phase, reveal the top 4 pile cards, gain the max/min-numbered ones and
    /// give the rest to a minimum-hand player; the draw is skipped. ABI 470
    /// (C 段 omp, `sibian`).
    EffectResolveSibian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect reveal_choose_category_distribute_self_threshold_hp_loss(player, reveal_count, recipient_cap)
    ///
    /// Reveal draw-pile cards, choose one card category, distribute every matching card with a
    /// per-recipient cap, then make recipients whose share is at least the owner's lose 1 HP.
    /// ABI 491 (TW `twxingluan`).
    EffectRevealChooseCategoryDistributeSelfThresholdHpLoss {
        /// Skill owner.
        player: ExprIr,
        /// Number of cards to reveal.
        reveal_count: u8,
        /// Maximum number of distributed cards each recipient may receive.
        recipient_cap: u8,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_random_gain_region_then_draw_compare_color_lose_hp(player)
    ///
    /// Choose another living player with a card in hand/equipment/judgment, gain one
    /// authoritative-random card from those regions, draw one, then lose 1 HP when the two
    /// cards have different colors. ABI 492 (DIY `nsdingzhou`).
    EffectChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_refuhai(player)
    ///
    /// Collect every other living player's private binary choice in seat order, reveal the
    /// results together, and draw for the initial equal run. ABI 493 (mobile `refuhai`).
    EffectResolveRefuhai {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_gain_target_hand_ceil_half_difference(player, target)
    ///
    /// Discard the active invocation's selected hand card, then visibly gain half (rounded up)
    /// of the target's post-cost hand-count advantage. ABI 494 (offline `nsyangwu`).
    EffectChooseGainTargetHandCeilHalfDifference {
        /// Skill owner.
        player: ExprIr,
        /// Other target whose hand was larger before paying the cost.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_gzjinfa(player, target)
    ///
    /// Pay the selected hand/equipment card, then let the target give equipment or lose one
    /// gainable card; a spade gift makes the target use a virtual Sha. ABI 495.
    EffectResolveGzjinfa {
        /// Skill owner.
        player: ExprIr,
        /// Other target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wn_qifeng(player, target)
    ///
    /// Use a distance/count-free virtual Sha on the ending-turn player with damage equal to the
    /// number of cards the owner lost this turn. ABI 496.
    EffectResolveWnQifeng {
        /// Skill owner.
        player: ExprIr,
        /// Ending-turn player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 杨彪「举讹」 from the current End-phase player. ABI 583.
    EffectResolveDcjue {
        /// 结算「厥」的角色。
        player: ExprIr,
        /// 结束阶段角色。
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 阮籍「昭文/酒遁」. ABI 584.
    EffectResolveRuanji {
        /// Skill owner.
        player: ExprIr,
        /// Event source, or NONE.
        source: ExprIr,
        /// Triggering card, or NONE.
        card: ExprIr,
        /// Public action: 0 昭文 setup, 1 red-card draw, 2 酒 rewrite, 3 酒遁 response.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 崔琰毛玠「征辟」. ABI 585.
    EffectResolveZhengbi {
        /// Skill owner.
        player: ExprIr,
        /// 征辟目标。
        target: ExprIr,
        /// Public action stage.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 崔琰毛玠「奉迎」. ABI 585.
    EffectResolveFengying {
        /// Skill owner.
        player: ExprIr,
        /// 奉迎辅助技能定义。
        helper_def: ExprIr,
        /// Public action stage.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect set_draw_phase_bottom(player)：寸目（许攸）当前摸牌阶段从牌堆底摸牌。ABI 589。
    EffectSetDrawPhaseFromBottom {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect draw_cards_from_bottom(player, count)：寸目（许攸）从牌堆底摸牌。ABI 589。
    EffectDrawFromBottom {
        /// Player drawing.
        player: ExprIr,
        /// Card count.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_chenglve(player, stage)：成略（许攸）转换技状态机。ABI 589。
    EffectResolveChenglve {
        /// Skill owner.
        player: ExprIr,
        /// Choice barrier stage: 0 flip+draw, 1 discard, 2 commit suits.
        stage: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_shicai(player, card, stage)：恃才（许攸）放置牌堆顶状态机。ABI 589。
    EffectResolveShicai {
        /// Skill owner.
        player: ExprIr,
        /// 使用牌。
        card: ExprIr,
        /// Choice barrier stage: 0 check+order, 1 place+draw.
        stage: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_sbbiyue(player)：闭月（界貂蝉）结束阶段摸牌。ABI 599。
    EffectResolveSbbiyue {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_sxrmqiewang(player, event_player)：窃王（sxrm 严畯）距离内摸牌。ABI 596。
    EffectResolveSxrmqiewang {
        /// Skill owner.
        player: ExprIr,
        /// 受伤角色。
        event_player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_sxrmqianliu(player, stage)：潜流（sxrm 伏皇后）牌堆底四张。ABI 596。
    EffectResolveSxrmqianliu {
        /// Skill owner.
        player: ExprIr,
        /// 使用者。
        source: ExprIr,
        /// 0=取牌堆底判定.
        stage: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_gzbuqu(player, delta, stage)：旧不屈（旧周泰）创牌状态机。ABI 591。
    EffectResolveGzbuqu {
        /// Skill owner.
        player: ExprIr,
        /// 本次体力变化量（负数 = 流失）。
        delta: ExprIr,
        /// 0=置入创牌并判定免死.
        stage: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_gzbuqu_recover(player, stage)：旧不屈恢复移除创牌。ABI 591。
    EffectResolveGzbuquRecover {
        /// Skill owner.
        player: ExprIr,
        /// 0=选择移除, 1=移除+救回判定.
        stage: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xiongluan(player, target, stage)：雄乱（张绣）限定技状态机。ABI 590。
    EffectResolveXiongluan {
        /// Skill owner.
        player: ExprIr,
        /// 限定目标。
        target: ExprIr,
        /// 1=施加限制（废除装备栏与判定区、目标禁用手牌）。
        stage: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twjuxiang(player, event_player, stage)：踞襄（张绣）主公技状态机。ABI 590。
    EffectResolveTwjuxiang {
        /// Skill owner.
        player: ExprIr,
        /// 触发角色（其他群势力角色出牌阶段）。
        event_player: ExprIr,
        /// 0=装备牌选择, 1=置入或获得.
        stage: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 傅干「巧谏」. ABI 586.
    EffectResolveQiaojian {
        /// Skill owner.
        player: ExprIr,
        /// 巧谏目标。
        target: ExprIr,
        /// 巧谏辅助技能定义。
        helper_def: ExprIr,
        /// Public action stage.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 傅干「析察」. ABI 586.
    EffectResolveXicha {
        /// Skill owner.
        player: ExprIr,
        /// 事件来源。
        source: ExprIr,
        /// 触发牌。
        card: ExprIr,
        /// Public action stage.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 胡遵「斩涛 / 安境」. ABI 587.
    EffectResolveHuzun {
        /// Skill owner.
        player: ExprIr,
        /// Damage source for 斩涛; none for 安境.
        source: ExprIr,
        /// Damage card for 斩涛; none for 安境.
        card: ExprIr,
        /// 0=斩涛, 1=安境.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 诸葛梦雪「寄春 / 寒英」. ABI 588.
    EffectResolveZhugemengxue {
        /// Skill owner.
        player: ExprIr,
        /// 0=寄春, 10=寒英.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 柏灵筠「灵慧 / 黠策」. ABI 593.
    EffectResolveBailingyun {
        /// Skill owner.
        player: ExprIr,
        /// 0=灵慧, 10=造成伤害后黠策, 11=受到伤害后黠策.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 威关银屏「绍威」. ABI 594.
    EffectResolveDcshaowei {
        /// Skill owner.
        player: ExprIr,
        /// 0=失牌花色分支，1=回复后恢复装备栏.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 威关银屏「涤仇」. ABI 594.
    EffectResolveDcdichou {
        /// Skill owner.
        player: ExprIr,
        /// 0=回合开始，10=回合结束.
        action: ExprIr,
        /// Temporary modifier skill definition.
        helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 凌操「独锋」. ABI 595.
    EffectResolveDcdufeng {
        /// Skill owner.
        player: ExprIr,
        /// 0=choose costs, 1=draw and grant turn modifiers.
        action: ExprIr,
        /// Temporary modifier skill definition.
        helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect skip_next_discard_phase(player)
    ///
    /// Skip the target's discard phase this turn. ABI 471 (C 段 omp, `spfengyin`).
    EffectSkipNextDiscardPhase {
        /// Target whose discard phase is skipped.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spfengyin(player, target)
    ///
    /// Give one Sha from the owner's hand to the target and skip its play + discard
    /// phases. ABI 471 (C 段 omp, `spfengyin`).
    EffectResolveSpfengyin {
        /// Skill owner.
        player: ExprIr,
        /// Other player whose turn is being sealed.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xinfu_lianpian(player)
    ///
    /// When the owner uses a card sharing a target with the previous card this play phase,
    /// draw one and optionally give it to a shared target. ABI 472 (C 段 omp,
    /// `xinfu_lianpian`).
    EffectResolveXinfuLianpian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_gongjian(player)
    ///
    /// For every target shared with the previous Sha this turn, discard 1-2 of its
    /// hand/equip cards, then gain the discarded Sha cards. ABI 473 (C 段 omp,
    /// `gongjian`).
    EffectResolveGongjian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xinfu_qinguo(player)
    ///
    /// After using an equipment card, optionally use a virtual Sha against another player.
    /// ABI 474 (C 段 omp, `xinfu_qinguo`).
    EffectResolveXinfuQinguo {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcmanzhi(player)
    ///
    /// At Prepare/End, choose one of two branches: give (target hands over up to two cards
    /// and uses a virtual Sha), or gain (gain up to two zone cards, give back equal, draw).
    /// Each branch once per turn via skill marks. ABI 475 (C 段 omp, `dcmanzhi`).
    EffectResolveDcmanzhi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xinxingluan(player)
    ///
    /// After a play-phase card use, once per turn, choose one of three rank-6 branches.
    /// ABI 476 (C 段 omp, `xinxingluan`).
    EffectResolveXinxingluan {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dckuizhen(player)
    ///
    /// Choose a target with >= hand or >= hp; the target uses a Duel on the owner; on
    /// owner damage gain the target's hand Shas, else the target loses 1 HP. ABI 477
    /// (C 段 omp, `dckuizhen`).
    EffectResolveDckuizhen {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dckuizhen_resolve(player, winner, loser)
    ///
    /// DuelResolved outcome for 溃阵: owner lost -> gain the initiator's hand Shas;
    /// owner won -> the initiator loses 1 HP. ABI 477 (C 段 omp, `dckuizhen`).
    EffectResolveDckuizhenResolve {
        /// Skill owner.
        player: ExprIr,
        /// Duel winner.
        winner: ExprIr,
        /// Duel loser.
        loser: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_nsdingzhou(player)
    ///
    /// Gain one of a target's hand/equip/judge cards (first, deterministic), draw one,
    /// lose 1 HP if the colors differ. ABI 478 (C 段 omp, `nsdingzhou`).
    EffectResolveNsdingzhou {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_sxrmlanjiao(player)
    ///
    /// Loop: view two of a target's hand cards; the target may lose 1 HP and draw to
    /// retry; on stop the owner gains the last viewed cards. ABI 479 (C 段 omp,
    /// `sxrmlanjiao`).
    EffectResolveSxrmlanjiao {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_rekuanshi(player)
    ///
    /// End-phase target choice marking the watched player. ABI 480 (C 段 omp,
    /// `rekuanshi`).
    EffectResolveRekuanshi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_rekuanshi_resolve(player, damaged)
    ///
    /// When the watched player takes their second damage this turn, recover 1 HP.
    /// ABI 480 (C 段 omp, `rekuanshi`).
    EffectResolveRekuanshiResolve {
        /// Skill owner.
        player: ExprIr,
        /// The damaged (watched) player.
        damaged: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcmoyu(player)
    ///
    /// Gain one (or two) of a target's hand/equip/judge cards; the target uses a virtual
    /// Sha; on owner damage ban the skill this turn, else the next gain is bigger. ABI 481
    /// (C 段 omp, `dcmoyu`).
    EffectResolveDcmoyu {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xizhen(player)
    ///
    /// At play-phase start, choose a target and use a virtual Sha or Duel on it.
    /// ABI 482 (C 段 omp, `xizhen`).
    EffectResolveXizhen {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xuxie(player)
    ///
    /// Lose 1 maxHp at play start; adjacent (distance <= 1) players either each discard
    /// one card or each draw one. ABI 483 (C 段 omp, `xuxie`).
    EffectResolveXuxie {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xuxie_add(player)
    ///
    /// Restore 1 maxHp at play end when someone's maxHp exceeds the owner's.
    /// ABI 483 (C 段 omp, `xuxie`).
    EffectResolveXuxieAdd {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcchiying(player)
    ///
    /// Let the selected target repeatedly discard cards from distinct players in its attack
    /// range, then gain the discarded basic cards. ABI 484 (C 段 omp, `dcchiying`).
    EffectResolveDcchiying {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_jsrgchiying(player, target)
    ///
    /// Every other player in the target's attack range discards one hand/equipment card in
    /// seat order. If the discarded basic-card count does not exceed the target's HP, the
    /// target gains the discarded basic cards. ABI 501 (`jsrgchiying`).
    EffectResolveJsrgchiying {
        /// Skill owner.
        player: ExprIr,
        /// Active-skill target whose attack range determines the discarders.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xinkuangfu(player)
    ///
    /// Discard one equipment of a chosen player (self allowed), use a virtual Sha, then
    /// branch on damage: self-equip + damage -> draw 2; other-equip + no damage ->
    /// discard 2 hand cards. ABI 485 (C 段 omp, `xinkuangfu`).
    EffectResolveXinkuangfu {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcxiangshu(player, target)
    ///
    /// At another player's play start (hand >= hp), choose a secret number 0-5; at their
    /// play end, resolve |hand - number|: <=1 gain one card, ==0 deal 1 damage.
    /// ABI 486 (C 段 omp, `dcxiangshu`).
    EffectResolveDcxiangshu {
        /// Skill owner.
        player: ExprIr,
        /// The observed player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcxiangshu_lottery(player, target)
    ///
    /// Second phase: the number-guess payoff at the observed player's play end.
    /// ABI 486 (C 段 omp, `dcxiangshu`).
    EffectResolveDcxiangshuLottery {
        /// Skill owner.
        player: ExprIr,
        /// The observed player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_shuangxiong(player)
    ///
    /// Draw-phase replacement: judge, gain the judgment card, record its color, and arm the
    /// opposite-color hand-card view-as Duel for the turn. ABI 511 (C 段 omp, `shuangxiong`).
    EffectResolveShuangxiong {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_zhijian(player)
    ///
    /// Play-phase: give one selected hand equipment card to another player who can equip it,
    /// then draw one. ABI 512 (C 段 omp, `zhijian`).
    EffectResolveZhijian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_guzheng(player, event_player)
    ///
    /// After another player's discard phase: return one card they discarded into the discard
    /// pile and optionally take the rest. ABI 513 (C 段 omp, `guzheng`).
    EffectResolveGuzheng {
        /// Skill owner.
        player: ExprIr,
        /// The player whose discard phase ended.
        event_player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_zhiba(player, actor)
    ///
    /// Lord skill: a Wu-faction player may start a hand comparison with the owner; the owner
    /// may refuse when awakened and may take both comparison cards when not winning.
    /// ABI 523 (C 段 omp, `zhiba`).
    EffectResolveZhiba {
        /// Skill owner and lord.
        player: ExprIr,
        /// The Wu-faction comparison initiator.
        actor: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_drlt_congjian(player)
    ///
    /// As a target of a multi-target trick, give one hand/equipment card to another target
    /// and draw. ABI 524 (C 段 omp, `drlt_congjian`).
    EffectResolveDrltCongjian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_nzry_feijun(player)
    ///
    /// Play-phase: discard one hand/equipment card, then either a hand-more player gives the
    /// owner one card or an equip-more player discards one equipment; arms 兵略 for a first
    /// time target. ABI 526 (C 段 omp, `nzry_feijun`).
    EffectResolveNzryFeijun {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_drlt_yongsi(player, stage)
    ///
    /// Draw-phase replacement with the living faction count, and play-phase-end settlement
    /// by turn damage. ABI 527 (C 段 omp, `drlt_yongsi`).
    EffectResolveDrltYongsi {
        /// Skill owner.
        player: ExprIr,
        /// Zero = draw replacement, one = play-phase-end settlement.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_drlt_weidi(player)
    ///
    /// Lord skill: at the start of the owner's Discard phase, give up to
    /// min(needed, other-Qun count) hand cards to equal other Qun players.
    /// ABI 528 (C 段 omp, `drlt_weidi`).
    EffectResolveDrltWeidi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect give_selected_then_target_choose_discard_same_suit_or_lose_hp(player, target)
    ///
    /// Give the single active-skill hand-card input to `target`. After the movement and nested
    /// triggers finish, that target chooses either to reveal their hand and discard every
    /// discardable hand/equipment card matching the given card's physical suit, or lose 1 HP.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
        /// Skill owner giving the selected hand card.
        player: ExprIr,
        /// Other living recipient who makes the follow-up choice.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect gain_equipment_from_draw_pile(player)
    ///
    /// Move the topmost equipment card in the shared draw pile into `player`'s hand.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGainEquipmentFromDrawPile {
        /// Player gaining the equipment card.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect give_all_hand_recover_to_one(player, target)
    ///
    /// Give every current hand card from `player` to `target`, then recover `target` to exactly
    /// one HP after the movement and all nested triggers finish.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGiveAllHandRecoverToOne {
        /// Living skill owner giving every hand card.
        player: ExprIr,
        /// Other living dying recipient.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(player)
    ///
    /// Let `player` choose any living target able to discard one of their own hand/equipment
    /// cards. That target chooses and discards exactly one such card. After the movement and all
    /// nested triggers finish, `player` loses one HP when the target's hand count differs from
    /// their HP. Appended to keep existing serialized enum ordinals stable.
    EffectChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
        /// Living skill owner making the target choice.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_give_one_hand_equip_or_recover(owner, giver, target)
    ///
    /// Let `giver` optionally give one current hand/equipment card to `target`; if no card is
    /// given, recover `target` by one HP. `owner` identifies the originating skill owner.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseGiveOneHandEquipOrRecover {
        /// Originating skill owner.
        owner: ExprIr,
        /// Player making the card choice and giving the card.
        giver: ExprIr,
        /// Gift recipient or recovery target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect replace_skill_with_bound_after_resolution_if_target_alive(
    ///     player, skill, skill_def, target
    /// )
    ///
    /// After the current effect, trigger, and dying/save resolution finishes, replace `skill`
    /// with a target-bound helper only when `target` is still alive. Appended to keep existing
    /// serialized enum ordinals stable.
    EffectReplaceSkillWithBoundAfterResolutionIfTargetAlive {
        /// Skill owner.
        player: ExprIr,
        /// Skill instance being replaced.
        skill: ExprIr,
        /// Target-bound helper skill definition.
        skill_def: u32,
        /// Target whose life state is checked after the resolution barrier.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(player)
    ///
    /// Force `player` to choose a nonempty hand or equipment zone, discard every
    /// discardable card in that zone, draw the discarded count plus current lost HP after
    /// movement triggers finish, then lose 1 HP after draw triggers finish. Appended to keep
    /// existing serialized enum ordinals stable.
    EffectChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_hand_equip_then_recover(player, count, amount)
    ///
    /// Force `player` to choose and discard exactly `count` current hand/equipment cards,
    /// then recover `amount` HP after movement triggers finish. Appended to keep existing
    /// serialized enum ordinals stable.
    EffectChooseDiscardHandEquipThenRecover {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Exact discard count.
        count: ExprIr,
        /// Maximum HP recovered after the discard resolves.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_targets_damage(player, min_count, max_count, scope, amount)
    ///
    /// Force `player` to choose a bounded number of distinct living targets in `scope`, then
    /// deal `amount` normal damage to each in seat order. Appended to keep existing serialized
    /// enum ordinals stable.
    EffectChooseTargetsDamage {
        /// Player making the target choice and dealing the damage.
        player: ExprIr,
        /// Minimum number of targets.
        min_count: u8,
        /// Maximum number of targets.
        max_count: u8,
        /// Canonical target scope.
        scope: DrawTargetScopeIr,
        /// Damage dealt to each selected target.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect discard_target_equipment_draw_weapon_range(player, target)
    ///
    /// Force `player` to choose and discard one current discardable equipment card from
    /// `target`; after movement triggers finish, a discarded weapon makes `target` draw cards
    /// equal to its physical attack range. Appended for serialized ordinal stability.
    EffectDiscardTargetEquipmentDrawWeaponRange {
        /// Skill owner choosing the equipment card.
        player: ExprIr,
        /// Living target whose equipment is discarded and who may draw.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_reveal_hand_damage_if_sha(player, amount)
    ///
    /// Force `player` to choose a living target with hand cards, publicly reveal that target's
    /// complete current hand, then deal normal damage from `player` iff at least one revealed
    /// card is an effective Sha for the target. Appended for serialized ordinal stability.
    EffectChooseTargetRevealHandDamageIfSha {
        /// Skill owner choosing the target and dealing damage.
        player: ExprIr,
        /// Damage amount when the revealed hand contains an effective Sha.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_give_one_hand_equip(player)
    ///
    /// Force `player` to choose another living player, then give that target exactly one current
    /// hand/equipment card. Appended for serialized ordinal stability.
    EffectChooseOtherGiveOneHandEquip {
        /// Skill owner choosing the target and card.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_reveal_hand_card_bind_category_target(player, target)
    ///
    /// Force `player` to choose and publicly reveal one current hand card from `target`, then
    /// remember both that target and the card's logical category for the current Play phase.
    /// Appended for serialized ordinal stability.
    EffectChooseRevealHandCardBindCategoryTarget {
        /// Skill owner choosing and revealing the card.
        player: ExprIr,
        /// Current Play-phase player whose hand card is revealed.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
        ] }
    };
}
pub(super) use variants;
