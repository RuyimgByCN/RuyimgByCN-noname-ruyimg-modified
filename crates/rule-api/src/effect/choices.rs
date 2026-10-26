//! Choices effect definitions.

macro_rules! variants {
    (RequestChoice [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseDiscardOneHandOrEquip [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Ask `player` to discard exactly one card from hand or equipment.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseDiscardOneHandOrEquip {
        /// Player making the forced choice.
        player: PlayerId,
    },
        ] }
    };
    (ChooseDiscardHandAddSkillByColor [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseDrawOrRecover [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseUseVirtualSha [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseUseVirtualShaOrDraw [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseOtherDrawByHandCount [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseDiscardOneForShaRefundAndDrawIfRed [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseAddCurrentShaTarget [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseStoreHandShaInSkillPile [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseDiscardMatchingEquipColorAddSkill [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseDrawOrDiscardAddSkill [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseGainResolvedShaOrShanThenOptionalUseSha [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseMoveTargetEquipmentOrDraw [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseHandCountDifferentTargetDrawOrRandomDiscard [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose a living player whose hand count differs from the owner. The target draws one when
    /// lower, or discards one authoritative deterministic-random hand card when higher.
    /// Appended for serialized ordinal stability.
    ChooseHandCountDifferentTargetDrawOrRandomDiscard {
        /// Skill owner and target chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseHandCompare [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseHandCompareBetween [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseOtherGiveThenRecover [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose one other living player with hand cards: they give one hand card to the
    /// chooser, then **they** recover 1 HP (ABI 682, batch 180 psdaohe 踏鹤).
    ChooseOtherGiveThenRecover {
        /// Chooser and gift recipient.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseAddMissingSkill [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Force the owner to choose and gain one candidate skill definition they do not yet own.
    /// Appended for serialized stability.
    ChooseAddMissingSkill {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Three distinct candidate skill definitions.
        skill_defs: [SkillDefId; 3],
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Force a player to choose a fixed number of distinct living targets, then each draws.
    /// Appended for serialized stability.
    ChooseTargetsDraw {
        /// Player making the target choice.
        player: PlayerId,
        /// Cards drawn by each selected target.
        count: u32,
        /// Exact number of distinct targets.
        target_count: u8,
        /// Canonical target scope.
        scope: DrawTargetScope,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseOtherDrawOwnerLostHpTurnOver [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Force the owner to choose another living player. The target draws cards equal to the
    /// owner's current lost HP, then turns over. Appended for serialized stability.
    ChooseOtherDrawOwnerLostHpTurnOver {
        /// Skill owner making the target choice.
        player: PlayerId,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseTargetReclaimEquipmentDrawIfRangeReduced [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose one of the target's equipped cards and move it into that target's own hand.
    ///
    /// After nested movement triggers finish, the owner draws one card only if the number of
    /// living players inside the target's attack range decreased. Appended for serialized
    /// stability.
    ChooseTargetReclaimEquipmentDrawIfRangeReduced {
        /// Skill owner and equipment-card chooser.
        player: PlayerId,
        /// Other player reclaiming the selected equipment.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-owned committed equipment reclamation after the card choice is resolved.
    ///
    /// Content bytecode cannot construct this request. Appended for serialized stability.
    CommitTargetReclaimEquipmentDrawIfRangeReduced {
        /// Skill owner and equipment-card chooser.
        player: PlayerId,
        /// Player reclaiming the selected equipment.
        target: PlayerId,
        /// Selected card still expected in the target's equipment zone.
        card: CardId,
        /// Attack-range population captured before the equipment leaves play.
        before: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-owned post-movement range comparison for
    /// [`EffectRequest::ChooseTargetReclaimEquipmentDrawIfRangeReduced`].
    ///
    /// Content bytecode cannot construct this request. Appended for serialized stability.
    FinishTargetReclaimEquipmentDrawIfRangeReduced {
        /// Skill owner who may draw.
        player: PlayerId,
        /// Player whose attack-range population is compared.
        target: PlayerId,
        /// Attack-range population captured before the equipment left play.
        before: u32,
        /// Originating skill instance for draw attribution.
        skill: SkillId,
    },
        ] }
    };
    (ChooseOtherDrawOrRecover [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Repeatedly let one player choose another living player, then force the selected target
    /// to choose between drawing cards and recovering HP.
    ///
    /// Every target choice is independently cancellable. Appended for serialized stability.
    ChooseOtherDrawOrRecover {
        /// Player making each target choice.
        player: PlayerId,
        /// Cards drawn by the selected target's draw branch.
        draw_count: u32,
        /// HP recovered by the selected target's recovery branch.
        recover_count: u32,
        /// Remaining independently cancellable target choices.
        times: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Force a player to discard the requested number of hand/equipment cards, capped by the
    /// currently available cards, then remove the originating one-shot skill.
    ///
    /// Cleanup is deferred until movement triggers and any nested choices finish. Appended for
    /// serialized stability.
    ChooseDiscardHandEquipThenRemoveSelfSkill {
        /// Player making the forced card choice.
        player: PlayerId,
        /// Requested discard count before capping to available cards.
        count: u32,
        /// Originating one-shot skill instance removed after the discard resolves.
        skill: SkillId,
    },
        ] }
    };
    (ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose a living target who can self-discard one hand/equipment card, let that target
    /// discard exactly one, then make the owner lose one HP if the target's hand count differs
    /// from their HP after all nested movement triggers finish.
    ///
    /// Content emits `target = NONE` and `ready_to_check = false`; the engine owns the target,
    /// card-choice, and post-movement stages. Appended for serialized ordinal stability.
    ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
        /// Living skill owner choosing the target.
        player: PlayerId,
        /// Selected target, or NONE before target selection.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the target's discard and all nested movement triggers have completed.
        ready_to_check: bool,
    },
    /// Let `giver` choose whether to give one current hand/equipment card to `target`; if the
    /// giver declines or has no eligible card, recover the target by one HP.
    ///
    /// `owner` is the originating skill owner and may differ from both choice observer and
    /// recipient. Appended for serialized ordinal stability.
    ChooseGiveOneHandEquipOrRecover {
        /// Living owner of the originating skill.
        owner: PlayerId,
        /// Living player who observes the card choice and gives the selected card.
        giver: PlayerId,
        /// Living recipient, normally the current dying player.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Force the skill owner to choose a nonempty hand or equipment zone, discard every
    /// discardable card there, draw the discarded count plus current lost HP after movement
    /// triggers finish, then lose 1 HP after draw triggers finish.
    ///
    /// `zone` is 0 before the zone branch, 1 for hand, or 2 for equipment.
    /// `ready_to_draw` is false until the discard and all nested movement triggers complete.
    /// Appended for serialized ordinal stability.
    ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
        /// Living skill owner and forced chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Chosen zone tag: 0 = unresolved, 1 = hand, 2 = equipment.
        zone: u8,
        /// Discarded-card count captured before the movement.
        discarded_count: u32,
        /// Whether the discard and its nested triggers have completed.
        ready_to_draw: bool,
    },
    /// Force a player to discard exactly `count` current hand/equipment cards, then recover
    /// `amount` HP after movement triggers and nested choices finish.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDiscardHandEquipThenRecover {
        /// Skill owner and forced chooser.
        player: PlayerId,
        /// Exact number of cards required.
        count: u32,
        /// Maximum HP recovered after the discard resolves.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseOtherGiveOneHandEquip [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Force the skill owner to choose another living player, then give that target exactly one
    /// current hand/equipment card.
    ///
    /// Appended for serialized ordinal stability.
    ChooseOtherGiveOneHandEquip {
        /// Skill owner choosing the target and card.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseMoveTargetEquipmentAndOrDraw [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Let the skill owner cancel, draw one, move one compatible equipment card from `target`,
    /// or perform both available actions in move-then-draw order.
    ///
    /// Appended for serialized ordinal stability.
    ChooseMoveTargetEquipmentAndOrDraw {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player whose turn just ended.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseDrawOrRecastAddSkill [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Let the player cancel, draw one and gain one temporary skill, or recast one
    /// hand/equipment card and gain another temporary skill.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDrawOrRecastAddSkill {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Temporary skill granted before drawing one card.
        draw_skill_def: SkillDefId,
        /// Temporary skill granted before recasting one card.
        recast_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let the owner gain the first Sha from the draw or discard pile during this originating
    /// skill's turn-opening extra Play phase.
    ///
    /// Appended for serialized ordinal stability.
    ChooseGainShaForOpeningExtraPlay {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseScskuijiDiscard [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// `player` chooses up to `count` cards from its own and `target`'s hands combined, each of a
    /// distinct effective suit, then all chosen cards are discarded (split by owner). Powers
    /// 「试探」-style active skills.
    ///
    /// Appended for serialized ordinal stability.
    ChooseScskuijiDiscard {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player whose hand also participates.
        target: PlayerId,
        /// Maximum number of cards chosen (and discarded).
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Like `ChooseScskuijiDiscard`, but when the chooser and target discard different numbers of
    /// cards, the side that discarded fewer loses 1 HP (the legacy `rechouai` grant is a dangling
    /// no-op in upstream and is intentionally not modeled). Powers 「破试探」.
    ///
    /// Appended for serialized ordinal stability.
    ChoosePskuijiDiscard {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player whose hand also participates.
        target: PlayerId,
        /// Maximum number of cards chosen (and discarded).
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseOtherRecover [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Force one player to choose another living player, then recover the selected target.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseOtherRecover {
        /// Player making the forced target choice.
        player: PlayerId,
        /// Maximum HP recovered by the selected target.
        amount: u32,
    },
        ] }
    };
    (ChooseOtherHandTargetsGainOneReplaceDraw [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose one or two other players with hand cards, gain one hand card from each in order,
    /// and replace the current Draw-phase draw.
    ///
    /// An empty `targets` list opens the target choice. A concrete `selected` card gains that
    /// card from `targets[index]`; `NONE` scans for the next card choice. Appended so existing
    /// serialized effect variant ordinals remain stable.
    ChooseOtherHandTargetsGainOneReplaceDraw {
        /// Current Draw-phase player and chooser.
        player: PlayerId,
        /// Snapshotted selected targets in choice order.
        targets: Vec<PlayerId>,
        /// Current target index.
        index: u32,
        /// Selected target hand card, or NONE while opening/scanning choices.
        selected: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseCurrentShaTargetDiscardRedirectToOwner [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Let the current Sha target optionally discard one hand/equipment card, then redirect that
    /// target slot to the skill owner after the discard's nested triggers finish.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseCurrentShaTargetDiscardRedirectToOwner {
        /// Skill owner receiving the redirected Sha.
        owner: PlayerId,
        /// Current Sha user.
        source: PlayerId,
        /// Physical Sha currently resolving.
        card: CardId,
        /// Current target paying the discard cost.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// False opens the cost choice; true commits the redirect after the discard barrier.
        ready_to_redirect: bool,
    },
    /// Let the current Sha target discard one hand/equipment card and redirect that target slot
    /// to another legal player in the skill owner's attack range.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseDiscardRedirectCurrentSha {
        /// Current Sha target and skill owner paying the discard cost.
        player: PlayerId,
        /// Current Sha user.
        source: PlayerId,
        /// Physical Sha currently resolving.
        card: CardId,
        /// Chosen replacement target, or `NONE` before target selection.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseDiscardOneFromEachLinked [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// In seat order, choose and discard one hand/equipment card from every linked player.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseDiscardOneFromEachLinked {
        /// Skill owner choosing each discard.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseTargetLoseHpDiscardHandShaIfBasic [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose a player with a discardable hand card, lose 1 HP, then make that target discard.
    /// A basic card produces a distance-free virtual Sha after movement triggers finish.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseTargetLoseHpDiscardHandShaIfBasic {
        /// Skill owner and virtual Sha user.
        player: PlayerId,
        /// Selected target, or NONE before target selection.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// 0=choose target, 1=choose discard after HP loss, 2=post-discard check.
        stage: u8,
        /// Effective category snapshot for the discarded card.
        discarded_basic: bool,
    },
    /// Optionally choose a bounded number of living targets, then each draws in seat order.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseOptionalTargetsDraw {
        /// Player making the target choice.
        player: PlayerId,
        /// Cards drawn by each selected target.
        count: u32,
        /// Minimum number of distinct targets.
        min_count: u8,
        /// Maximum number of distinct targets.
        max_count: u8,
        /// Canonical target scope.
        scope: DrawTargetScope,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Let a dead skill owner choose one living other player tied for the highest positive mark,
    /// judge that target, and kill them unless the physical result is Tao or Taoyuan.
    ///
    /// Appended for serialized ordinal stability.
    ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
        /// Dead skill owner and chooser.
        player: PlayerId,
        /// Mark definition used to rank living other players.
        mark: MarkDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseAllLoseHpOrRecover [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Let Qinyin's owner cancel or make every currently living player lose or recover one HP.
    ///
    /// Appended for serialized ordinal stability.
    ChooseAllLoseHpOrRecover {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating trigger skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseLinkOther [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Force the skill owner to choose and link another currently unlinked living player.
    ///
    /// Appended for serialized ordinal stability.
    ChooseLinkOther {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseDiscardWeaponOrLoseHp [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose and pay either one discardable physical weapon from hand/equipment or one HP.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDiscardWeaponOrLoseHp {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseOtherHandStoreSkillPile [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseDiscardOneHeOrLoseHp [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseSourceConfirmOwnerLoseHp [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseOtherYinghun [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseAddCurrentShaTargetWithinDistance [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseRepoluTargetsDraw [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseCurrentTrickTargetsToNullify [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseGainAllResolvedShaShanThenColorRewards [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseDrawViewTwoGainOrMoveEquipment [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose to draw one card, privately inspect up to two sampled target hand cards and gain
    /// one, or move one compatible target equipment card.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDrawViewTwoGainOrMoveEquipment {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player whose turn ended.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseGiveTwoDifferentColorsOrRecover [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Let `giver` give exactly two differently colored hand/equipment cards to `target`; if
    /// the giver declines or has no valid pair, recover the target by one HP.
    ///
    /// `owner` identifies the originating skill owner and may differ from the choice observer.
    /// Appended for serialized ordinal stability.
    ChooseGiveTwoDifferentColorsOrRecover {
        /// Living owner of the originating skill.
        owner: PlayerId,
        /// Living player who chooses and gives the cards.
        giver: PlayerId,
        /// Living recipient, normally the current dying player.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseRecoverTarget [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Let one player cancel or recover a specified target.
    ///
    /// Appended for serialized ordinal stability.
    ChooseRecoverTarget {
        /// Living player making the choice.
        chooser: PlayerId,
        /// Living wounded recovery target.
        target: PlayerId,
        /// Maximum HP recovered.
        amount: u32,
    },
        ] }
    };
    (ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose one logical card category, then let the current Sha target give the source one
    /// matching hand/equipment card or make that Sha unrespondable for the target.
    ///
    /// Appended for serialized ordinal stability.
    ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
        /// Current Sha user and skill owner.
        player: PlayerId,
        /// Current physical Sha.
        card: CardId,
        /// Current Sha target.
        target: PlayerId,
        /// Originating trigger skill instance.
        skill: SkillId,
    },
    /// Choose targets for an ignore-distance virtual Sha, then lose 1 HP and use it. Its base
    /// damage is increased when the user played multiple distinct basic-card variants this turn.
    ///
    /// Appended for serialized ordinal stability.
    ChooseLoseHpUseVirtualShaByBasicVariants {
        /// Skill owner and virtual Sha user.
        player: PlayerId,
        /// Materialized virtual Sha after target selection begins.
        #[serde(default = "no_card")]
        card: CardId,
        /// Selected legal Sha targets at the post-cost stage.
        #[serde(default)]
        targets: Vec<PlayerId>,
        /// Zero chooses targets; one uses the saved virtual Sha after the HP-loss barrier.
        #[serde(default)]
        stage: u8,
        /// Captured virtual Sha damage bonus.
        #[serde(default)]
        damage_bonus: u32,
        /// Originating trigger skill.
        skill: SkillId,
    },
        ] }
    };
    (ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose another player with a region card, gain one authoritative-random card, draw one,
    /// and lose 1 HP when the gained and drawn cards have different colors. ABI 492.
    ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseGainTargetHandCeilHalfDifference [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Discard the selected active-skill hand card, then visibly gain half (rounded up) of the
    /// target's post-cost hand-count advantage. ABI 494.
    ChooseGainTargetHandCeilHalfDifference {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other target.
        target: PlayerId,
        /// Exact selected heart hand card paid as the cost.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the discard and its nested triggers have completed.
        ready_to_choose: bool,
    },
        ] }
    };
    (ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose another living player other than the damage source, gain one deterministic-random
    /// gainable hand/equipment card, reveal it, and use it when it is an equipment card. ABI 499.
    ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
        /// Damaged skill owner.
        player: PlayerId,
        /// Damage source excluded from the target choice; may be `NONE`.
        source: PlayerId,
        /// Randomly gained equipment awaiting ordinary use.
        card: CardId,
        /// Resolution stage: 0 = choose target; 1 = use gained equipment if still legal.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseGiveOrDiscardHand [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Choose either to give an exact number of hand cards to one living target in `scope`, or
    /// discard that many hand cards. ABI 581.
    /// Appended for serialized ordinal stability.
    ChooseGiveOrDiscardHand {
        /// Skill owner choosing and paying cards.
        player: PlayerId,
        /// Exact hand-card payment count.
        payment_count: u32,
        /// Recipient target scope.
        scope: DrawTargetScope,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let the chooser cancel or discard an exact number of hand/equipment cards, then recover
    /// the specified target after movement triggers finish. ABI 582.
    /// Appended for serialized ordinal stability.
    ChooseDiscardHandEquipThenRecoverTarget {
        /// Skill owner and chooser.
        chooser: PlayerId,
        /// Living wounded recovery target.
        target: PlayerId,
        /// Exact number of cards required.
        count: u32,
        /// Maximum HP recovered after the discard resolves.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseDiscardHandRange [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Force a player to choose and discard between `min` and `max` discardable hand cards.
    /// ABI 596. Appended for serialized ordinal stability.
    ChooseDiscardHandRange {
        /// Player choosing and discarding cards.
        player: PlayerId,
        /// Inclusive minimum selection count.
        min: u32,
        /// Inclusive maximum selection count.
        max: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseOtherGainOneOfTwoSkills [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 择一加技能：玩家在两个技能定义间二选一，将所选技能置入目标（gwent 呼朋
    /// hupeng 择一杜达元帅 1/2 置入目标手牌）。
    /// ABI 644 (ADR 0612).
    ChooseOtherGainOneOfTwoSkills {
        /// Chooser (skill owner).
        player: PlayerId,
        /// Recipient of the chosen skill.
        target: PlayerId,
        /// First skill definition option.
        first_def: SkillDefId,
        /// Second skill definition option.
        second_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseUseVirtualNanman [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Use a suitless, source-free virtual 南蛮入侵 through the ordinary card pipeline.
    ///
    /// Unlike `ChooseUseVirtualCard`, this variant is phase-agnostic (no phase-end
    /// history suppression): 南蛮 resolves against its complete legal target set
    /// immediately. Appended so existing serialized effect variant ordinals remain stable.
    ChooseUseVirtualNanman {
        /// Virtual card user.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseOneFromSkillPileDiscard [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 选堆内单卡弃置 (expansion S1-B, ADR 0641 §8.7): pick one card from the
    /// owner's own (player, pile_def) skill pile and discard it.
    ChooseOneFromSkillPileDiscard {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Skill definition owning the pile.
        pile_def: u32,
    },
        ] }
    };
    (ChooseGroup [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 选择势力并变更（ABI 664，势力族动态）：选现存在且不同于自己的势力，
    /// 写入 group_overrides（chijie/延祚/wuchang 动态势力）。
    ChooseGroup {
        /// 变更者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// §7 技能夺取设施 S0（ABI 675）：选一名其他存活角色，再从其可获得技能
    /// 名单中获得一个（potyinhui 音洄增益臂）。
    ChooseGainOtherSkill {
        /// 获得者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseOtherAddMark [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Select one other alive player and adjust one of their marks (ABI 652).
    ChooseOtherAddMark {
        /// Choosing player.
        player: PlayerId,
        /// Mark definition raw id.
        mark: u32,
        /// Delta (positive or negative).
        delta: i32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChoosePlaceHandXumou [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 蓄谋·选牌（ABI 666，jsrg 遏前①）：选一张手牌移入判定区作为蓄谋牌
    /// （cancel = 停止蓄谋）。
    ChoosePlaceHandXumou {
        /// 蓄谋者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseOthersInsertExtraTurn [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 目标插相（ABI 671，shenhua 放权）：选择一名其他角色，将其排入
    /// extra_turn_queue；check 非零时同时登记尾随失去体力 check 标记。
    ChooseOthersInsertExtraTurn {
        /// 发动者。
        player: PlayerId,
        /// 非零：为目标额外回合登记尾随 check。
        check: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseOthersAddMark [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 多目标授标（ABI 670，jsrg 政略①「至多两名」）：选择 1..max_count 名其他
    /// 存活角色，各获得 delta 点给定标记（cancel = 放弃授标）。
    ChooseOthersAddMark {
        /// 选择者（授标者）。
        player: PlayerId,
        /// Mark definition raw id.
        mark: u32,
        /// Delta per target.
        delta: i32,
        /// Max selectable targets.
        max_count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseGiveOrRecastOneHand [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Let the skill owner choose one hand card, then either give it to a living other player or
    /// recast it. Appended for serialized ordinal stability.
    ChooseGiveOrRecastOneHand {
        /// Living skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Required owned skill def (0 = no reward).
        require_skill_def: u32,
        /// Skill def granted to the recipient on give (0 = none).
        grant_skill_def: u32,
        /// Mark def granted to the recipient on give (0 = none).
        grant_mark_def: u32,
    },
        ] }
    };
    (ResolveOlJieDanshou [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// OL界朱然「胆守」：四选项每阶段各一次，选择目标执行效果后弃置序号张牌。
    ResolveOlJieDanshou {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance (used for per-option storage).
        skill: SkillId,
    },
        ] }
    };
}
pub(super) use variants;
