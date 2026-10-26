//! Damage and recovery effect definitions.

macro_rules! variants {
    (DealDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (DealShaDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseDiscardOrFireDamageSkipPhases [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (PreventDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (DealCardDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ConvertCurrentShaToFireLoseHpIfDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseOtherTargetsBySkippedPhaseCountDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseOtherTargetsBySkippedPhaseCountDamageDraw [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Same target selection/damage as above, then draw one per target that actually took damage.
    /// Appended for serialized ordinal stability.
    ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
        /// Skill owner and damage source.
        player: PlayerId,
        /// Damage dealt to each selected target.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseDiscardEquipmentReduceCurrentDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (GiveAllEquipmentChooseRecoverOrDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ContinueChooseGainOneFromDamageTargetAndNeighbors [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseOtherDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (RevealUniformHandChooseDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseTargetDrawToMaxHpCap [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (DealDamageBanActiveIfDying [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (CommitDamageBanActiveIfDying [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (IncreaseCurrentShaBaseDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (RecoverIfMinCurrentHp [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseDrawOrIncreaseCurrentCardDamageOrBothMark [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (IncreaseCurrentShaTargetDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseTargetsDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseTargetRevealHandDamageIfSha [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (RecoverToMaxThenOptionalExchangeSkill [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseCompareSelectedHandCardSecondFailureSeatDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (RecordCompareFailureSeatDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (TargetChooseSuitGainOwnerHandDamageIfMismatch [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseDiscardBasicThenTargetDiscardEquipOrDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (LoseHpGainDiscardCardForOpeningExtraPlay [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (DamageThenDiscardAllJudge [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseDiscardDamageEndpointCardIncreaseIfSuit [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (LoseHpDoubleCurrentDamageThenReplace [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (DamageThenDrawSelectedTargets [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ChooseTargetDiscardDistanceHandOrIncreaseDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Let the bound damage target discard `max(1, 4-distance)` hand cards or increase the
    /// current damage by one.
    ///
    /// Appended for serialized ordinal stability.
    ChooseTargetDiscardDistanceHandOrIncreaseDamage {
        /// Damage source and skill owner.
        player: PlayerId,
        /// Bound damage target and discard chooser.
        target: PlayerId,
        /// Engine-owned damage event id.
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (GiveAllHandThenChooseUseOneOrDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Give every hand card to another player, who may use one of those exact cards; return the
    /// other gifted cards before that use, or deal one normal damage when the offer is declined.
    ///
    /// Appended for serialized ordinal stability.
    GiveAllHandThenChooseUseOneOrDamage {
        /// Original hand owner and damage source.
        player: PlayerId,
        /// Gift recipient and optional card user.
        target: PlayerId,
        /// Exact gifted cards captured at stage zero.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Selected legal targets for the gifted card at the final stage.
        #[serde(default)]
        targets: Vec<PlayerId>,
        /// Engine-owned stage: zero gives, one chooses a card, two chooses targets, three chooses
        /// a target pair.
        #[serde(default)]
        stage: u8,
        /// Selected gifted card after stage one.
        #[serde(default = "no_card")]
        card: CardId,
        /// Originating active skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Declare one logical card category; the damage target gives one matching hand/equipment
    /// card or the bound damage is increased by one. ABI 498 (`jytongbei`).
    ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
        /// Damage source and skill owner.
        player: PlayerId,
        /// Damage target.
        target: PlayerId,
        /// Bound pre-resolution damage id.
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ModifyCurrentDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 通用当前伤害修改（ABI 562）：damage_before 窗口内对当前伤害量增/减 delta。
    /// 正值走 increase_bound_damage，负值走 reduce_bound_damage（钳制到 0），零值为
    /// 空操作。解锁 lianhuo / dckanghui / renjia 等非卡牌来源伤害修改技能。
    ///
    /// Appended for serialized ordinal stability.
    ModifyCurrentDamage {
        /// Skill owner requesting the modification.
        player: PlayerId,
        /// Delta to apply to the current damage amount (may be negative).
        delta: i32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (PreventRecovery [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Cancel the recovery whose `recover_before` trigger queue is running. ABI 635.
    PreventRecovery {
        /// Engine-owned recovery id; content emits zero and the VM host binds it.
        #[serde(default)]
        recovery_id: u32,
    },
    /// Apply recovery after its `recover_before` trigger queue. Engine-owned, ABI 635.
    CommitRecovery {
        /// Engine-owned recovery id, or zero when no trigger matched.
        recovery_id: u32,
        /// Recovery source.
        source: PlayerId,
        /// Recovery target.
        player: PlayerId,
        /// Requested amount.
        amount: u32,
    },
        ] }
    };
    (ChooseTargetGainMaxHp [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 寿春 (sp/pxshouchun): the owner selects any living player (self included);
    /// the selected player gains 1 max hp. Appended for serialized ordinal stability.
    ChooseTargetGainMaxHp {
        /// Choosing player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
}
pub(super) use variants;
