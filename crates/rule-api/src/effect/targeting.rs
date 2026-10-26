//! Targeting effect definitions.

macro_rules! variants {
    (RecordActiveTarget [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (InspectTargetHandChooseHeartDiscardOrTop [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (BanUseAgainst [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 回合内禁用指定角色对另一角色使用牌。ABI 571。
    BanUseAgainst {
        /// Banned card user.
        user: PlayerId,
        /// Banned target.
        target: PlayerId,
    },
        ] }
    };
    (ResetLinkedSelectedTargets [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 主动原语 (批 270): duansuo 断索 — reset every selected linked target,
    /// then deal 1 fire damage to each (two passes, upstream order).
    ResetLinkedSelectedTargets {
        /// Damage source (skill owner).
        source: PlayerId,
        /// Selected linked targets.
        targets: Vec<PlayerId>,
    },
        ] }
    };
}
pub(super) use variants;
