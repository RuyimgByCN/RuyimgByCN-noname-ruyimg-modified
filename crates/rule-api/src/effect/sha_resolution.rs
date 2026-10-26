//! Sha resolution effect definitions.

macro_rules! variants {
    (MakeCurrentShaTargetUnrespondable [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (RefundCurrentShaUseAndDrawIfRed [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (CommitAddCurrentShaTarget [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ForceOthersUseShaOnNearestOrLoseHp [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (SpjungongDiscardOrLoseThenSha [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (TargetDiscardThenDuelIfNonShaAndHpGteOwner [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (RefundCurrentShaUse [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Refund the currently saved proactive Sha's ordinary use count at most once.
    ///
    /// Appended for serialized ordinal stability.
    RefundCurrentShaUse {
        /// Current Sha user.
        player: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ExcludeSavedShaTarget [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 将当前正在结算的【杀】的指定目标从保存目标中移除（此杀对该目标无效）。
    /// ABI 560。
    ExcludeSavedShaTarget {
        /// The Sha card currently resolving.
        card: CardId,
        /// Target to exclude from the Sha's saved target list.
        target: PlayerId,
    },
        ] }
    };
    (RecordCurrentShaAllTargets [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Bind all targets of the exact currently resolving Sha to the originating skill. ABI 580.
    /// Appended for serialized ordinal stability.
    RecordCurrentShaAllTargets {
        /// Sha user.
        player: PlayerId,
        /// Exact logical Sha carrier.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ContinueYoudishenru [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Engine-owned continuation for the recoverable 诱敌深入 Sha loop. ABI 613.
    ContinueYoudishenru {
        /// 0=start virtual response; 1=inspect the forced Sha; 2=resume the cancelled incoming Sha.
        action: u8,
        /// Protected player who used 诱敌深入.
        protected: PlayerId,
        /// Original Sha user forced to continue attacking.
        source: PlayerId,
        /// Exact forced Sha carrier for action 1.
        forced_card: CardId,
        /// Encoded incoming-Sha continuation.
        saved: Vec<u32>,
    },
        ] }
    };
}
pub(super) use variants;
