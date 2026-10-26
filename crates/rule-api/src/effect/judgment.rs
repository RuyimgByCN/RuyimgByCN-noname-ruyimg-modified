//! Judgment effect definitions.

macro_rules! variants {
    (JudgeGainEquipmentByColor [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Perform a judgment, then gain the topmost equipment from the draw pile
    /// for red or from the discard pile for black.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeGainEquipmentByColor {
        /// Player performing the judgment and gaining the equipment.
        player: PlayerId,
    },
        ] }
    };
    (JudgeCancelCurrentShaTargetOnRed [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (JudgeRecoverOnSuit [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (JudgeDamageOtherOnSuit [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (JudgePreventDamageLoseMaxHpOnNonHeart [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (GuanxingTop [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
    (GuanxingTopByFaluMarks [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (JudgeRecoverOtherOnSuit [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (JudgeSourceDiscardTwoHandOrDamage [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (JudgeMakeCurrentShaTargetUnrespondableOnRed [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (JudgeBlackGainRepeat [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (JudgeSetSkillMarkToRank [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Perform a judgment and replace the originating skill's same-id mark with its rank.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeSetSkillMarkToRank {
        /// Player performing the judgment and receiving the mark.
        player: PlayerId,
        /// Originating skill instance whose definition id is used as the mark id.
        skill: SkillId,
    },
        ] }
    };
    (JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 目标判定：黑桃 2-9 则其摸两张牌，否则对其使用两张雷【杀】。ABI 602,雷劫 releijie。
    ///
    /// Appended to keep existing serialized variant ordinals stable.
    JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
        /// Skill owner who uses the virtual thunder Sha.
        source: PlayerId,
        /// Target who is judged, draws, or receives the thunder Sha.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (JudgeSameColorGainRepeat [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Judge repeatedly while results match the first judgment's effective color, gaining each
    /// matching result before optionally continuing.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeSameColorGainRepeat {
        /// Judged player and matching-card recipient.
        player: PlayerId,
        /// Matching judgment card awaiting gain, or `NONE` before a new judgment.
        #[serde(default = "no_card")]
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the gained-card trigger chain has completed and the repeat choice may open.
        #[serde(default)]
        ready_to_choose: bool,
        /// First effective judgment color: 0=unset, 1=red, 2=black.
        #[serde(default)]
        color: u8,
    },
        ] }
    };
    (JudgeBlackGainRepeatProtectHandLimit [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Judge repeatedly while results are black, gaining each black result and excluding that
    /// concrete card from this player's ordinary discard-to-limit accounting for the turn.
    ///
    /// Appended for serialized ordinal stability.
    JudgeBlackGainRepeatProtectHandLimit {
        /// Judged player and black-card recipient.
        player: PlayerId,
        /// Black judgment card awaiting gain, or `NONE` before a new judgment.
        #[serde(default = "no_card")]
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the gained-card trigger chain has completed and the repeat choice may open.
        #[serde(default)]
        ready_to_choose: bool,
    },
        ] }
    };
}
pub(super) use variants;
