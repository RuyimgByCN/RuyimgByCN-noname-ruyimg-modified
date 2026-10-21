//! Ordered EffectRequest variants, starting with `ChooseOtherGiveThenRecover`.

macro_rules! variants {
    ([$($tail:ident),*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail),*] [
            $($variants)*
    /// Choose one other living player with hand cards: they give one hand card to the
    /// chooser, then **they** recover 1 HP (ABI 682, batch 180 psdaohe 踏鹤).
    ChooseOtherGiveThenRecover {
        /// Chooser and gift recipient.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Watch the top N cards of the draw pile, where N is the number of non-zero stacks
    /// among the four declared falu marks; pick top/bottom per card as ABI 681 (batch 183
    /// xinfu_dianhua 花色化法箓).
    GuanxingTopByFaluMarks {
        /// Viewer.
        player: PlayerId,
        /// Four falu mark defs (spade, heart, club, diamond).
        marks: [SkillDefId; 4],
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose one other living player and deal fixed normal damage. Appended for serialized
    /// stability.
    ChooseOtherDamage {
        /// Skill owner, damage source, and chooser.
        player: PlayerId,
        /// Damage amount.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Reveal all hand cards and, when their effective color is uniform, force the owner to
    /// choose any living player and deal fixed normal damage. Appended for serialized stability.
    RevealUniformHandChooseDamage {
        /// Skill owner, hand owner, damage source, and chooser.
        player: PlayerId,
        /// Damage amount.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Draw one card for each supplied divisor of the player's match-lifetime number of cards
    /// used or played as a response. Appended for serialized stability.
    DrawForLifetimeCardActionDivisors {
        /// Player whose action history is inspected and who draws the cards.
        player: PlayerId,
        /// Three positive divisors checked independently.
        divisors: [u32; 3],
    },
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
    /// Gain every card captured by the triggering event that remains in the discard pile.
    /// Appended for serialized stability.
    GainEventCards {
        /// Player gaining the cards.
        player: PlayerId,
        /// Stable event-card snapshot supplied by the engine host.
        cards: Vec<CardId>,
    },
    /// Force the chooser to select any living player, then draw that target up to the lesser of
    /// their maximum HP and a fixed cap. Appended for serialized stability.
    ChooseTargetDrawToMaxHpCap {
        /// Player making the target choice.
        player: PlayerId,
        /// Inclusive upper bound applied to the selected target's maximum HP.
        cap: u32,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// Force the owner to choose another living player. The target draws cards equal to the
    /// owner's current lost HP, then turns over. Appended for serialized stability.
    ChooseOtherDrawOwnerLostHpTurnOver {
        /// Skill owner making the target choice.
        player: PlayerId,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// Perform a judgment for one player and recover another player if the effective suit
    /// matches. Appended for serialized stability.
    JudgeRecoverOtherOnSuit {
        /// Recovery source, normally the skill owner.
        source: PlayerId,
        /// Player performing the judgment.
        judge_player: PlayerId,
        /// Player receiving any recovery.
        recover_player: PlayerId,
        /// Required effective judgment suit.
        suit: Suit,
        /// Recovery amount.
        amount: u32,
    },
    /// In clockwise seat order, force every other snapshotted living player to use a physical
    /// or view-as Sha on a nearest player, or lose one HP. Appended for serialized stability.
    ForceOthersUseShaOnNearestOrLoseHp {
        /// Skill owner excluded from the forced-use queue.
        player: PlayerId,
        /// Remaining players in the activation-time clockwise snapshot.
        remaining: Vec<PlayerId>,
        /// False only for the Rule VM's initial request; prevents an empty completed queue from
        /// being mistaken for a fresh activation.
        started: bool,
        /// Originating once-per-game skill instance.
        skill: SkillId,
    },
    /// Continue a deterministic multi-player dying-save queue. Engine-internal and appended for
    /// serialized stability.
    ContinueDyingSave {
        /// Player currently dying.
        player: PlayerId,
        /// Damage source used if the player dies.
        source: PlayerId,
        /// Remaining responders in clockwise order.
        responders: Vec<PlayerId>,
    },
    /// Restore a living character card to its normal state: unlinked and face-up.
    ///
    /// Appended for serialized stability.
    RestoreCharacterState {
        /// Player whose character card state is restored.
        player: PlayerId,
    },
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
    /// Deal ordinary damage and ban one active skill for the current turn if this exact damage
    /// makes the target enter dying.
    ///
    /// Appended for serialized stability.
    DealDamageBanActiveIfDying {
        /// Damage source and skill owner.
        source: PlayerId,
        /// Damage target.
        target: PlayerId,
        /// Base damage amount.
        amount: u32,
        /// Active skill instance to ban.
        skill: SkillId,
    },
    /// 峻攻：弃置至多 `use_count + 1` 张牌或失去等量体力，然后对目标使用虚拟【杀】。
    ///
    /// 虚拟【杀】经普通结算队列，命中目标时由配套触发禁用该主动技至回合结束。
    SpjungongDiscardOrLoseThenSha {
        /// 峻攻 owner；弃牌/失体力者与虚拟【杀】使用者。
        player: PlayerId,
        /// 峻攻虚拟【杀】目标。
        target: PlayerId,
        /// 来源主动技实例。
        skill: SkillId,
    },
    /// 把技能实例的 `active_turn_banned` 置位，禁用该主动技直到当前回合结束。
    BanActiveUntilTurnEnd {
        /// 技能拥有者。
        player: PlayerId,
        /// 被禁用的主动技实例。
        skill: SkillId,
    },
    /// 挽危（主动阶段）：目标回复 player.hp 点体力，拥有者失去 player.hp 点体力。
    SpwanweiActive {
        /// 挽危拥有者。
        player: PlayerId,
        /// 回复体力的目标角色。
        target: PlayerId,
    },
    /// 挽危（濒死救援）：目标回复 max(player.hp + 1, 1 - target.hp) 体力，拥有者失去 player.hp 点体力。
    SpwanweiDying {
        /// 挽危拥有者。
        player: PlayerId,
        /// 处于濒死状态的目标角色。
        target: PlayerId,
    },
    /// Engine-owned prepared form of [`EffectRequest::DealDamageBanActiveIfDying`].
    ///
    /// Content bytecode cannot construct this request. Appended for serialized stability.
    CommitDamageBanActiveIfDying {
        /// Engine-owned damage event id, or zero when no pre-trigger matched.
        damage_id: u32,
        /// Damage source and skill owner.
        source: PlayerId,
        /// Damage target.
        target: PlayerId,
        /// Final amount after ordinary damage modifiers.
        amount: u32,
        /// Amount propagated through an elemental chain before target-side modifiers.
        base_amount: u32,
        /// Damage nature after damage-before rewrites.
        #[serde(default)]
        nature: u8,
        /// Active skill instance to ban if the target enters dying.
        skill: SkillId,
    },
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
    /// Increase the base damage of the currently suspended proactive Sha use.
    ///
    /// The source/card pair is revalidated against the saved continuation. Appended for
    /// serialized stability.
    IncreaseCurrentShaBaseDamage {
        /// Current Sha user.
        player: PlayerId,
        /// Physical card backing the current Sha.
        card: CardId,
        /// Amount added to the saved base damage.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Mark the current living player's upcoming Play phase to be skipped.
    ///
    /// The request is valid only before Play in that player's current turn. Appended for
    /// serialized stability.
    SkipNextPlayPhase {
        /// Current turn player.
        player: PlayerId,
    },
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
    /// Recover HP only when the player is tied for the lowest current HP among living players.
    ///
    /// The condition is evaluated when the effect resolves, not while bytecode is emitted.
    /// Appended for serialized stability.
    RecoverIfMinCurrentHp {
        /// Potential recovery recipient.
        player: PlayerId,
        /// Maximum HP recovered.
        amount: u32,
    },
    /// Increase the base damage of the currently suspended damaging card use.
    ///
    /// ABI 104 accepts proactive Sha, Nanman, and Wanjian; ABI 229 also accepts Juedou.
    /// The source/card pair is revalidated against the saved continuation. Appended for
    /// serialized stability.
    IncreaseCurrentCardBaseDamage {
        /// Current card user.
        player: PlayerId,
        /// Physical card backing the current use.
        card: CardId,
        /// Amount added to the saved base damage.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Lose HP, reduce maximum HP only if that exact loss first enters dying, then damage another
    /// player after the complete dying/save window.
    ///
    /// Appended for serialized stability.
    LoseHpThenDamageOtherLoseMaxHpIfDying {
        /// Skill owner who loses HP and later sources the damage.
        player: PlayerId,
        /// Other player damaged after the owner's dying window.
        target: PlayerId,
        /// Shared HP-loss and damage amount.
        amount: u32,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Engine-owned prepared form of
    /// [`EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying`].
    ///
    /// Content bytecode cannot construct this request. Appended for serialized stability.
    CommitLoseHpThenDamageOtherLoseMaxHpIfDying {
        /// Skill owner whose HP loss is committed.
        player: PlayerId,
        /// Other player damaged after the owner's dying window.
        target: PlayerId,
        /// Shared HP-loss and damage amount.
        amount: u32,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Recover HP for the distinct living targets selected by the active-skill command.
    ///
    /// The VM snapshots `targets` from the authoritative active invocation and the engine
    /// resolves them in seat order. Appended for serialized stability.
    RecoverSelectedTargets {
        /// Skill owner credited as the recovery source.
        source: PlayerId,
        /// Active-skill targets.
        targets: Vec<PlayerId>,
        /// HP recovered by each target.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Record the selected card's effective category for the current play phase.
    ///
    /// Appended for serialized stability.
    RecordSelectedPlayPhaseCategory {
        /// Selected physical card, captured from the authoritative active invocation.
        card: CardId,
        /// Originating active skill instance.
        skill: SkillId,
    },
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
    /// Choose between drawing cards, increasing the current card's base damage, or performing
    /// both and then adding a mark.
    ///
    /// Content emits the current event card and no skill id; the engine VM host binds the
    /// originating skill. Appended for serialized ordinal stability.
    ChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
        /// Current card user, skill owner, and chooser.
        player: PlayerId,
        /// Physical card backing the current use.
        card: CardId,
        /// Cards drawn by the first and third branches.
        draw_count: u32,
        /// Base damage added by the second and third branches.
        damage_amount: u32,
        /// Stable mark definition added by the third branch.
        mark_def: MarkDefId,
        /// Mark count added by the third branch.
        mark_count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Increase only one target's damage for the currently resolving physical Sha.
    ///
    /// The source/card/target tuple is revalidated against the saved proactive Sha continuation.
    /// Appended for serialized ordinal stability.
    IncreaseCurrentShaTargetDamage {
        /// Current Sha user.
        source: PlayerId,
        /// Physical card backing the current Sha.
        card: CardId,
        /// Current Sha target.
        target: PlayerId,
        /// Additional damage for this target.
        amount: u32,
    },
    /// Choose any living target to draw cards. After the draw and all nested triggers finish,
    /// the owner draws one card if that target's hand remains below their maximum HP.
    ///
    /// `target == NONE` is the VM-emitted target-choice stage. `ready_to_check` is set only by
    /// the engine after the selected target's draw has completed.
    /// Appended for serialized ordinal stability.
    ChooseTargetDrawThenOwnerIfBelowMaxHp {
        /// Skill owner and target chooser.
        player: PlayerId,
        /// Selected draw target, or `NONE` before the choice.
        target: PlayerId,
        /// Cards drawn by the selected target.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the target draw and its nested triggers have completed.
        ready_to_check: bool,
    },
    /// Choose one living player in a bound source's attack range, then deal normal damage from
    /// that source. Appended for serialized ordinal stability.
    ChooseBoundSourceAttackRangeDamage {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Skill-bound damage source.
        source: PlayerId,
        /// Damage amount.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose any living target with positive maximum HP. The target draws cards equal to the
    /// lesser of their current maximum HP and `cap`; after nested draw triggers finish, they
    /// discard excess hand cards down to the freshly evaluated limit.
    ///
    /// `target == NONE` is the VM-emitted target-choice stage. `ready_to_discard` is set only by
    /// the engine after the selected target's draw has completed. The chooser may already be dead
    /// so that death-trigger skills can finish. Appended for serialized ordinal stability.
    ChooseTargetDrawMaxHpCapThenDiscardExcess {
        /// Skill owner and target chooser.
        player: PlayerId,
        /// Selected draw target, or `NONE` before the choice.
        target: PlayerId,
        /// Inclusive upper bound applied to the target's maximum HP.
        cap: u32,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the draw and its nested triggers have completed.
        ready_to_discard: bool,
    },
    /// Choose any living target with positive maximum HP. The target draws cards equal to the
    /// lesser of their current maximum HP and `cap`; after the draw finishes, the target gains
    /// the skill from `add_def` (the DSL side adds/removes the temp skill on its own triggers).
    ///
    /// `target == NONE` is the VM-emitted target-choice stage; the choice resolver re-emits with
    /// the selected target. Appended for serialized ordinal stability. ABI 455 (C 段 omp,
    /// `beizhan`).
    ChooseTargetDrawMaxHpCapThenAddSkill {
        /// Skill owner and target chooser.
        player: PlayerId,
        /// Selected draw target, or `NONE` before the choice.
        target: PlayerId,
        /// Inclusive upper bound applied to the target's maximum HP.
        cap: u32,
        /// Originating skill instance.
        skill: SkillId,
        /// Skill definition granted to the selected target after the draw.
        add_def: SkillDefId,
    },
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
    /// Replace an active skill with a target-bound helper after the current resolution barrier,
    /// but only if the target is still alive.
    ///
    /// Content emits `ready_to_replace = false`; the engine-owned second stage runs after
    /// damage triggers and any dying/save window. Appended for serialized ordinal stability.
    ReplaceSkillWithBoundAfterResolutionIfTargetAlive {
        /// Owner of both the replaced skill and the target-bound helper.
        player: PlayerId,
        /// Exact skill instance to replace.
        skill: SkillId,
        /// Helper skill definition to add.
        skill_def: SkillDefId,
        /// Target encoded into the helper instance when still alive.
        bound_target: PlayerId,
        /// Whether the current resolution barrier has completed.
        ready_to_replace: bool,
    },
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
    /// Force a player to choose a bounded number of living targets in a canonical scope, then
    /// deal normal damage to each selected target in seat order.
    ///
    /// Appended for serialized ordinal stability.
    ChooseTargetsDamage {
        /// Player making the choice and dealing the damage.
        player: PlayerId,
        /// Minimum number of distinct targets.
        min_count: u8,
        /// Maximum number of distinct targets.
        max_count: u8,
        /// Canonical target scope.
        scope: DrawTargetScope,
        /// Damage dealt to each selected target.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Force the skill owner to choose a living target with hand cards, publicly reveal that
    /// target's complete current hand, and deal normal damage when it contains an effective Sha.
    ///
    /// Appended for serialized ordinal stability.
    ChooseTargetRevealHandDamageIfSha {
        /// Skill owner choosing the target and dealing the damage.
        player: PlayerId,
        /// Damage dealt when the revealed hand contains an effective Sha.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Consume a limited skill, recover its owner to max HP, then optionally sacrifice another
    /// skill to restore a fresh instance of the limited skill after recovery fully resolves.
    ///
    /// Appended for serialized ordinal stability.
    RecoverToMaxThenOptionalExchangeSkill {
        /// Skill owner recovering and choosing the exchange.
        player: PlayerId,
        /// Originating limited skill instance; consumed in the first stage.
        skill: SkillId,
        /// Skill definition that may be sacrificed.
        removable_skill_def: SkillDefId,
        /// Originating definition captured before the instance is consumed.
        restore_skill_def: SkillDefId,
        /// Whether recovery and all nested triggers have completed.
        ready_to_choose: bool,
    },
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
    /// Let a bound direct-Sha damage target discard up to the source's current attack range
    /// from hand/equipment or increase that damage by one. When the source has a physical
    /// weapon, both branches are forced in discard-then-increase order and the weapon is
    /// discarded afterward.
    ///
    /// Content emits `damage_id = 0`; the engine VM host binds the current pre-damage event.
    /// Appended for serialized ordinal stability.
    ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
        /// Sha source and originating skill owner.
        player: PlayerId,
        /// Bound damage target and discard/branch chooser.
        target: PlayerId,
        /// Engine-owned pre-resolution damage id.
        #[serde(default)]
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Finish the originating turn-opening Sha-gain policy and apply its no-damage consequence.
    ///
    /// Appended for serialized ordinal stability.
    FinishOpeningExtraPlayShaGain {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Give every current hand card from `player` to `target`.
    ///
    /// Appended for serialized ordinal stability.
    GiveAllHand {
        /// Living player giving every hand card.
        player: PlayerId,
        /// Other living recipient.
        target: PlayerId,
    },
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
    /// Compare the selected hand card with one unused target. A strict win chooses between
    /// drawing and discarding one target hand/equipment card; the configured non-win threshold
    /// loses HP, damages the shorter seat path between the first two failures, and bans the
    /// active skill for the turn.
    ///
    /// Appended for serialized ordinal stability.
    ChooseCompareSelectedHandCardSecondFailureSeatDamage {
        /// Skill owner and first comparison participant.
        player: PlayerId,
        /// Other comparison participant.
        target: PlayerId,
        /// Owner's selected hand card.
        card: CardId,
        /// Cards drawn after choosing the strict-win draw branch.
        draw_count: u32,
        /// Number of non-wins that activates the failure branch.
        failure_count: u32,
        /// HP lost when the failure threshold is reached.
        lose_hp: u32,
        /// Damage dealt to each chosen seat-path player.
        damage: u32,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Resume the comparison after `hand_compare_started` triggers finish.
    ///
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    ContinueChooseCompareTargetHandCardSecondFailureSeatDamage {
        /// Skill owner and first comparison participant.
        player: PlayerId,
        /// Other comparison participant.
        target: PlayerId,
        /// Owner's selected hand card.
        card: CardId,
        /// Cards drawn on the strict-win draw branch.
        draw_count: u32,
        /// Non-win threshold.
        failure_count: u32,
        /// HP loss at the threshold.
        lose_hp: u32,
        /// Damage per seat-path player.
        damage: u32,
        /// Originating active skill instance.
        skill: SkillId,
    },
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
    /// Record one comparison non-win and, at the threshold, schedule HP loss followed by
    /// seat-path damage.
    ///
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    RecordCompareFailureSeatDamage {
        /// Skill owner.
        player: PlayerId,
        /// Current failed comparison target.
        target: PlayerId,
        /// Non-win threshold.
        failure_count: u32,
        /// HP loss at the threshold.
        lose_hp: u32,
        /// Damage per seat-path player.
        damage: u32,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Choose an equal-length seat path, or automatically use the shorter path, then damage
    /// its currently living players in order.
    ///
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    ChooseSeatPathDamage {
        /// Damage source and chooser.
        player: PlayerId,
        /// First failed comparison target.
        first: PlayerId,
        /// Second failed comparison target.
        second: PlayerId,
        /// Damage per player.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Damage the remaining snapshotted players one by one, resuming only after each dying/save
    /// interruption has completed.
    ///
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    DealDamageEachInOrder {
        /// Damage source.
        source: PlayerId,
        /// Remaining targets in exact resolution order.
        targets: Vec<PlayerId>,
        /// Damage per target.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Remove every skill currently owned by one player.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    RemoveAllSkills {
        /// Player losing all current skills.
        player: PlayerId,
    },
    /// Force one player to choose another living player, then recover the selected target.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseOtherRecover {
        /// Player making the forced target choice.
        player: PlayerId,
        /// Maximum HP recovered by the selected target.
        amount: u32,
    },
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
    /// Let `target` choose a suit and one current hand card from `player`; move that card to
    /// `target`, then deal 1 damage from `player` if its owner-effective suit differs.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    TargetChooseSuitGainOwnerHandDamageIfMismatch {
        /// Skill owner whose hand card is gained.
        player: PlayerId,
        /// Other living player making both choices.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Perform Ganglie's judgment, then let a living damage source discard exactly two hand
    /// cards or take 1 normal damage from the judging player.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeSourceDiscardTwoHandOrDamage {
        /// Damaged player performing the judgment.
        player: PlayerId,
        /// Damage source paying the discard or taking damage.
        source: PlayerId,
    },
    /// Give selected hand cards and recover the giver when their current Play-phase gift total
    /// first reaches two.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    GiveSelectedRecoverAfterPhaseTotalTwo {
        /// Skill owner giving selected cards.
        player: PlayerId,
        /// Other living recipient.
        target: PlayerId,
        /// Selected hand cards in input order.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether movement and nested triggers have completed.
        ready_to_recover: bool,
    },
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
    /// Perform a judgment and, on a red result, make one target unable to respond to the
    /// currently resolving Sha.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeMakeCurrentShaTargetUnrespondableOnRed {
        /// Player performing the judgment and using the Sha.
        player: PlayerId,
        /// Physical Sha whose target is being resolved.
        card: CardId,
        /// Current Sha target whose response window is disabled on success.
        target: PlayerId,
        /// Resolve the full 界铁骑 judgment, suppression, and same-suit gift branch.
        #[serde(default)]
        retieji: bool,
        /// Originating skill instance for the 界铁骑 branch.
        #[serde(default = "no_skill")]
        skill: SkillId,
        /// Whether judgment triggers have finished and the target choice may open.
        #[serde(default)]
        ready_to_choose: bool,
        /// Captured effective judgment suit tag for the 界铁骑 branch.
        #[serde(default)]
        judgment_suit: u8,
    },
    /// Let the skill owner optionally discard one basic hand card, then let the target discard
    /// one equipment card from hand/equipment or take 1 normal damage from the owner.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseDiscardBasicThenTargetDiscardEquipOrDamage {
        /// Skill owner paying the cost and dealing damage.
        player: PlayerId,
        /// End-phase player choosing the consequence.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Judge repeatedly while results are black, gaining each black result before optionally
    /// continuing.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeBlackGainRepeat {
        /// Judged player and black-card recipient.
        player: PlayerId,
        /// Black judgment card awaiting gain, or `NONE` before a new judgment.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the gained-card trigger chain has completed and the repeat choice may open.
        ready_to_choose: bool,
    },
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
    /// Perform a judgment and replace the originating skill's same-id mark with its rank.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeSetSkillMarkToRank {
        /// Player performing the judgment and receiving the mark.
        player: PlayerId,
        /// Originating skill instance whose definition id is used as the mark id.
        skill: SkillId,
    },
    /// In seat order, choose and discard one hand/equipment card from every linked player.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseDiscardOneFromEachLinked {
        /// Skill owner choosing each discard.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Disable every instance of one skill definition for a player until the next turn begins.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    DisableSkillByDefUntilTurnEnd {
        /// Skill owner.
        player: PlayerId,
        /// Skill definition to disable temporarily.
        skill_def: SkillDefId,
    },
    /// Force `target` to discard one hand/equipment card, then let `player` use a virtual Duel
    /// on that target when the discarded card is not Sha and the target's HP is not lower.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    TargetDiscardThenDuelIfNonShaAndHpGteOwner {
        /// Skill owner and virtual Duel user.
        player: PlayerId,
        /// Other player choosing and discarding a card.
        target: PlayerId,
        /// Originating active skill instance.
        skill: SkillId,
    },
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
    /// During the originating turn-opening extra Play phase, lose 1 HP and then gain the
    /// topmost matching discard-pile card. The action is forced while `mandatory_skill_def`
    /// remains owned and optional after that definition has been removed.
    ///
    /// Appended for serialized ordinal stability.
    LoseHpGainDiscardCardForOpeningExtraPlay {
        /// Skill owner, HP-loss target, and card recipient.
        player: PlayerId,
        /// Stable rule card-kind tag.
        kind: u8,
        /// Owning this definition makes the action mandatory.
        mandatory_skill_def: SkillDefId,
        /// Originating extra-phase skill instance.
        skill: SkillId,
    },
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
    /// Grant a skill only until the current turn ends.
    ///
    /// An already-owned permanent or temporary instance is left unchanged. Appended for
    /// serialized ordinal stability.
    AddSkillUntilTurnEnd {
        /// Skill recipient.
        player: PlayerId,
        /// Skill definition to grant.
        skill_def: SkillDefId,
    },
        ] }
    };
}
pub(super) use variants;
