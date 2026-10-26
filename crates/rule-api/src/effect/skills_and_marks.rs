//! Skills and marks effect definitions.

macro_rules! variants {
    (AddSkill [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (RemoveSkillByDef [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Remove every skill instance with one definition from a player.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    RemoveSkillByDef {
        /// Skill owner.
        player: PlayerId,
        /// Definition to remove.
        skill_def: SkillDefId,
    },
        ] }
    };
    (CommitStoreHandShaInSkillPile [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (AddBoundSkill [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (AddCardColorLockSkill [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (AddCardUsePolicySkill [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ReplaceSkillWithBoundAfterResolutionIfTargetAlive [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (RemoveAllSkills [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Remove every skill currently owned by one player.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    RemoveAllSkills {
        /// Player losing all current skills.
        player: PlayerId,
    },
        ] }
    };
    (DisableSkillByDefUntilTurnEnd [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Disable every instance of one skill definition for a player until the next turn begins.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    DisableSkillByDefUntilTurnEnd {
        /// Skill owner.
        player: PlayerId,
        /// Skill definition to disable temporarily.
        skill_def: SkillDefId,
    },
        ] }
    };
    (AddSkillUntilTurnEnd [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
    (ReplaceDrawGainTopTwoAddSkillsIfDifferentColors [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (SetNonLockedSkillSuppression [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (StoreSelectedInSkillPile [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (SuppressNonLockedSkillsUntilTurnEnd [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Suppress one player's non-locked skills until the current turn ends.
    ///
    /// Appended for serialized ordinal stability.
    SuppressNonLockedSkillsUntilTurnEnd {
        /// Living player being suppressed.
        player: PlayerId,
    },
        ] }
    };
    (SetSkillContentFlag [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Set or clear one persistent content flag on the originating skill instance.
    SetSkillContentFlag {
        /// Skill owner.
        player: PlayerId,
        /// Flag bit in 0..64.
        bit: u8,
        /// New flag value.
        value: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (AddFirstMissingSkill [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Permanently grant the first configured skill definition the player does not own. ABI 575.
    /// Appended for serialized ordinal stability.
    AddFirstMissingSkill {
        /// Skill owner.
        player: PlayerId,
        /// Three ordered candidate skill definitions.
        skill_defs: [SkillDefId; 3],
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (RefreshSkill [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Reset one skill instance's per-turn / per-phase usage and trigger counters, implementing
    /// upstream `player.refreshSkill()`. Used by locked clan skills such as 中流 (clanzhongliu).
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    RefreshSkill {
        /// Skill instance to refresh.
        skill: SkillId,
    },
    /// Attach a persistent content tag (gaintag) to each card. The tag value is a stable id
    /// (typically the owning skill's definition id) so multiple skills' tags never collide.
    /// Implements upstream `player.addGaintag(cards, tag)`.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    TagCard {
        /// Cards to tag.
        cards: Vec<CardId>,
        /// Stable tag id (usually a skill definition id).
        tag: u32,
    },
    /// Remove a persistent content tag from each card.
    /// Implements upstream `player.removeGaintag(tag, cards)`.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    UntagCard {
        /// Cards to untag.
        cards: Vec<CardId>,
        /// Stable tag id to remove.
        tag: u32,
    },
        ] }
    };
    (StorageSet [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Write `value` under `key` in one skill instance's keyed content storage map.
    /// Implements upstream `player.storage.<skill>[key] = value`.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    StorageSet {
        /// Skill instance whose storage is updated.
        skill: SkillId,
        /// Storage key (content-defined, e.g. a packed suit+category code).
        key: u32,
        /// Storage value.
        value: u32,
    },
        ] }
    };
    (GainFromSkillPile [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 取用 (expansion P1, ADR 0641): the owner selects one card from their own
    /// skill pile (武将牌上) and gains it into hand.
    GainFromSkillPile {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (GainWholeSkillPile [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 整取 (expansion S1-B 深化, ADR 0641 §8.7): move every card of the owner's
    /// own (player, pile_def) skill pile into the owner's hand.
    GainWholeSkillPile {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Skill definition owning the pile.
        pile_def: u32,
    },
        ] }
    };
    (ChangeGroup [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 变更势力（ABI 661，jsrg/tw 势力族）：将 player 势力覆盖为指定势力。
    ChangeGroup {
        /// 变更者。
        player: PlayerId,
        /// 目标势力名（如 "wei"/"shu"/"qun"）。
        group: String,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (LoseGainedSkill [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// §7 技能夺取设施 S1（ABI 676）：从本设施获得清单中移除一个技能
    /// （potyinhui 音洄失去臂）。
    LoseGainedSkill {
        /// 移除者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
}
pub(super) use variants;
