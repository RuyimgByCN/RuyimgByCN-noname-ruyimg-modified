//! Typed query modifiers with explicit combine order and provenance.

use crate::content::ContentRegistry;
use crate::state::{Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_api::{PlayerId, SkillDefId, SkillId, Suit};
use rule_api::{ModifierSource, QueryResponse, RelationTag};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// What a modifier applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierTarget {
    /// Attack range of a player.
    AttackRange,
    /// Distance *from* owner to others (subtractive horse, etc.).
    DistanceFrom,
    /// Final distance from owner is one when the owner's HP is at least the target's HP.
    DistanceFromHpGteTarget,
    /// Distance *to* owner from others.
    DistanceTo,
    /// Hand limit.
    HandLimit,
    /// Base hand limit before ordinary hand-limit modifiers.
    HandLimitBase,
    /// Draw-phase card count (摸牌阶段摸牌数).
    DrawCount,
    /// Optional draw-phase bonus, confirmed once per owning skill.
    OptionalDrawCount,
    /// Bonus damage dealt by the source's 杀/决斗 (e.g. 裸衣 +1).
    ShaDamage,
    /// Number of proactive 杀 uses allowed during the owner's turn.
    ShaUseLimit,
    /// Maximum number of targets selected for one proactive 杀.
    ShaTargetLimit,
    /// Matching cards must target every otherwise-legal target.
    CardTargetsAllFor(CardSelector),
    /// Matching cards may not select the modifier owner as a target.
    CardTargetCancelFor(CardSelector),
    /// Matching cards from the current turn owner may not select the modifier owner.
    CardTargetCancelFromCurrentFor(CardSelector),
    /// Matching cards used by a turn-banned user may not select the modifier owner.
    CardTargetCancelTurnBannedFor(CardSelector),
    /// Cards used by the owner may not target another player.
    CardTargetsSelfOnly,
    /// Number of response cards required from a target (base 1; e.g. 无双 +1).
    ResponseCount,
    /// Skip the discard phase when the modifier owner used no 杀 this turn.
    SkipDiscardIfNoSha,
    /// Damage bonus offered in exchange for replacing the owner's draw phase.
    DrawReplacementDamage,
    /// Maximum draw-pile cards offered to a prepare-phase scry/reorder hook.
    PrepareScryCount,
    /// Permit the owner to replace a revealed judgment with a hand card.
    JudgmentRewrite,
    /// Generic pre-mutation damage amount intent.
    DamageAmount,
    /// Generic pre-mutation damage nature intent.
    DamageNature,
    /// Non-zero value or veto cancels a damage intent.
    DamageCancel,
    /// Non-zero value or veto cancels a card-use intent.
    CardUseCancel,
    /// Non-zero value or veto prevents use of cards matching the selector.
    CardUseCancelFor(CardSelector),
    /// Non-zero value or veto prevents the owner from responding with any card.
    CardResponseCancel,
    /// Non-zero value or veto prevents the owner from responding with matching cards.
    CardResponseCancelFor(CardSelector),
    /// Non-zero value or veto prevents the owner from using any card to save a dying player.
    CardSaveCancel,
    /// Non-zero value or veto prevents matching cards from saving a dying player.
    CardSaveCancelFor(CardSelector),
    /// Immunity: cannot be targeted by 顺手牵羊 (value > 0 = immune, e.g. 谦逊).
    ImmuneShunshou,
    /// Immunity: cannot be targeted by 乐不思蜀 (value > 0 = immune, e.g. 谦逊).
    ImmuneLe,
    /// Card attribute key.
    CardAttr(u32),
    /// Non-zero value or veto prevents the owner from proactively using 杀.
    ShaUseCancel,
    /// Non-zero value exempts proactive 杀 uses whose every target carries the
    /// given mark from the per-turn use counter (jsrgzhenglve② 不限次数).
    ShaUseUnlimitedTargetMark(u32),
    /// ABI 679 (batch 168): any player ignores distance limits toward a target that
    /// owns the modifier and carries the declared skill def (offline tanhu3 狼顾? no —
    /// 袁术 tanhu3: distance -Infinity toward tanhu2 holders).
    CardDistanceUnlimitedSkillOnOwner(u32),
    /// ABI content-layer (batch 170): tricks targeting a player who owns the declared
    /// skill def cannot be answered by 无懈 (offline tanhu3 wuxie-immunity face).
    CardUseUnrespondableSkillOnOwner(u32),
    /// Non-zero value makes maximum HP the raw hand-limit base instead of current HP.
    HandLimitBaseMaxHp,
    /// Non-zero value or veto ignores distance limits for cards used by the owner.
    CardDistanceUnlimited,
    /// Non-zero value ignores distance to targets that carry the given mark
    /// (jsrgzhenglve② 对有“猎”目标无距离限制).
    CardDistanceUnlimitedTargetMark(u32),
    /// Non-zero value or veto ignores distance limits for matching cards used by the owner.
    CardDistanceUnlimitedFor(CardSelector),
    /// Overrides the ordinary target-distance limit for matching cards used by the owner.
    CardDistanceLimitFor(CardSelector),
    /// Non-zero value or veto prevents the owner from responding with 杀.
    ShaResponseCancel,
    /// Non-zero value or veto excludes 杀 from hand-limit count and phase discard.
    ShaIgnoredHandLimit,
    /// Non-zero value or veto excludes cards matching a selector from hand-limit count and phase discard.
    CardIgnoredHandLimit(CardSelector),
    /// Non-zero value or veto removes ordinary per-turn use limits for all cards.
    CardUseUnlimited,
    /// Non-zero value or veto removes ordinary use limits for matching cards.
    CardUseUnlimitedFor(CardSelector),
    /// Non-zero value or veto cancels damage received by the owner.
    DamageTakenCancel,
    /// Non-zero value or veto cancels damage of the matching nature received by the owner.
    DamageTakenCancelForNature(u8),
    /// Matching cards ignore distance to targets whose hand size is at least the owner's.
    CardDistanceUnlimitedIfTargetHandGteOwnerFor(CardSelector),
    /// Non-zero value prevents the owner from using or responding with hand cards.
    HandCardUseResponseCancel,
    /// During Play phase, only physical hand cards gained this turn are playable/respondable.
    HandCardGainedThisTurnOnly,
    /// Non-zero value prevents targeting the owner when their hand lacks the card's suit.
    CardTargetCancelIfTargetLacksMatchingSuit,
    /// Matching cards cannot target an armorless owner from another player with no fewer hand cards.
    CardTargetCancelIfOtherUserHandGteTargetAndArmorEmptyFor(CardSelector),
    /// Override one physical suit while rules are evaluated for the skill owner.
    CardSuit(Suit),
    /// Override the rule category of matching physical cards for the skill owner.
    CardName(CardSelector),
    /// Matching cards used as Sha cannot be answered by the target.
    ShaUnrespondableFor(CardSelector),
    /// Adjust all non-lose-HP damage received by the owner.
    DamageTakenAmount,
    /// Adjust damage of one resolved nature received by the owner.
    DamageTakenAmountForNature(u8),
    /// Adjust damage dealt by a Sha matching the concrete card selector.
    ShaDamageFor(CardSelector),
    /// Sha targets within the configured final distance cannot respond.
    ShaUnrespondableWithinDistance,
    /// A Sha cannot be answered when the modifier owner is its target.
    ShaTargetUnrespondable,
    /// Source attack range threshold for incoming damage: below caps at one, above adds one.
    DamageTakenSourceRangeThreshold,
    /// Non-zero value skips the modifier owner's judgment phase.
    SkipJudgePhase,
    /// Initial hand size before the first turn begins.
    InitialHandCount,
    /// Adjust final non-lose-HP damage after equipment and ordinary damage modifiers.
    DamageTakenFinalAmount,
    /// Adjust final damage of one resolved nature after equipment and ordinary modifiers.
    DamageTakenFinalAmountForNature(u8),
    /// Override the resolved nature of non-lose-HP damage received by the owner.
    DamageTakenNature,
    /// Adjust damage dealt by the owner when the original nature matches.
    DamageAmountForNature(u8),
    /// Non-zero value replaces source-owned damage with equal HP loss.
    /// Appended to keep existing serialized enum ordinals stable.
    DamageAsHpLoss,
    /// Replace damage between the owner and another player with equal HP loss.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    DamageBetweenOwnerAndOtherAsHpLoss,
    /// Matching equipment in the owner's equipment zone cannot be discarded by another player.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardDiscardProtectedFor(CardSelector),
    /// Adjust damage received by the owner specifically from a logical Sha.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ShaDamageTaken,
    /// Adjust Sha damage dealt by the owner when the target cannot reach the owner.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ShaDamageIfOutsideTargetRange,
    /// Source attack-range threshold for logical Sha damage: below caps at one, above adds one.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ShaDamageSourceRangeThreshold,
    /// Non-zero value replaces source-owned logical Sha damage with equal HP loss.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    DamageAsHpLossForSha,
    /// Matching cards cannot target the owner while their armor effect is active.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ArmorCardTargetCancelFor(CardSelector),
    /// Adjust damage received by the owner while their armor effect is active.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ArmorDamageTakenAmountForNature(u8),
    /// Non-zero value makes the owner's source-side effects ignore armor.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    IgnoreArmor,
    /// Non-zero value disables armor effects owned by this player.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ArmorEffectDisabled,
    /// Offer the owner an optional draw-phase four-card reorder with exactly two cards sent
    /// to the bottom.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    DrawPhaseScryFourSplitTwo,
    /// Non-zero value skips the modifier owner's discard phase.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    SkipDiscardPhase,
    /// The whole Sha use is unrespondable when its source or one target owns this modifier and
    /// the other participant's final distance to that owner is within the configured threshold.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ShaUseUnrespondableWithinDistance,
    /// Cards used by the owner cannot be answered by another player whose final distance to the
    /// owner is within the configured threshold.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardUseUnrespondableWithinResponderToOwnerDistance,
    /// Cards used by the owner cannot be answered by another player within the configured final
    /// distance from the owner.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardUseUnrespondableWithinOwnerToResponderDistance,
    /// Matching cards used by the owner cannot be answered by another player.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardUseUnrespondableFor(CardSelector),
    /// Adjust damage dealt by the owner specifically through the logical Sha channel.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    DamageAmountForSha,
    /// Adjust every living player's attack range while the modifier owner is alive.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    GlobalAttackRange,
    /// Adjust ordinary incoming damage for every queried player while the modifier owner is alive.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    GlobalDamageTakenAmount,
    /// Adjust ordinary incoming damage of one resolved nature for every queried player while the
    /// modifier owner is alive.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    GlobalDamageTakenAmountForNature(u8),
    /// Adjust final incoming damage for every queried player while the modifier owner is alive.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    GlobalDamageTakenFinalAmount,
    /// Adjust final incoming damage of one resolved nature for every queried player while the
    /// modifier owner is alive.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    GlobalDamageTakenFinalAmountForNature(u8),
    /// Adjust every queried player's hand limit while the modifier owner is alive.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    GlobalHandLimit,
    /// Override the damage nature of matching cards used by the modifier owner.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardNatureFor(CardSelector),
    /// Adjust distance to a queried target while the modifier owner is alive.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    GlobalDistanceTo,
    /// Non-zero value prevents an unlinked owner from gaining the chain mark while their armor
    /// effect is active. Removing an existing chain mark remains legal.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ArmorLinkGainCancel,
    /// Count the owner's equipment together with their hand when resolving the ordinary
    /// discard-to-limit phase, and allow either zone to satisfy the mandatory discard.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    DiscardPhaseIncludesEquip,
    /// Adjust the maximum number of independently resolved targets for matching cards.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardTargetLimitFor(CardSelector),
    /// Reveal foreign hand-card faces to the owning observer.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ObserverForeignHandFacesVisible,
    /// Reveal identities that are normally hidden to the owning observer.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ObserverHiddenIdentitiesVisible,
    /// Permit the owner to replace a revealed judgment with a hand or equipment card.
    ///
    /// Appended to keep existing serialized enum ordinals stable. The existing
    /// `JudgmentRewrite` channel remains hand-only for classic 鬼才.
    JudgmentRewriteHandEquip,
    /// Cards used by the owner ignore distance only to the player encoded in this skill
    /// instance's bound-target state.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardDistanceUnlimitedForSkillBoundTarget,
    /// Proactive Sha uses beyond the ordinary use count remain legal only against the player
    /// encoded in this skill instance's bound-target state.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ShaUseUnlimitedForSkillBoundTarget,
    /// Source-side effects ignore armor only on the player encoded in this skill instance's
    /// bound-target state.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    IgnoreArmorForSkillBoundTarget,
    /// Prevent use, response, and save with cards whose effective red/black color is present in
    /// this skill instance's card-color-lock state.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardActionCancelForSkillColorMask,
    /// Source-side effects ignore armor only for cards recorded in this skill instance's policy
    /// state with the ignore-armor bit.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    IgnoreArmorForSkillCardPolicy,
    /// Cards recorded in this skill instance's policy state with the unrespondable bit cannot be
    /// answered by Shan or Wuxie.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardUseUnrespondableForSkillCardPolicy,
    /// Select the highest-scoring card in the draw pile for the owner's judgment.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    JudgmentBestFromDrawPile,
    /// Provide a virtual Bagua response while the owner's armor slot is enabled and empty.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    VirtualBaguaIfArmorSlotEmpty,
    /// Add to the Shan count for Sha when the modifier owner is one participant and the other
    /// participant has female sex.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ShaResponseCountIfOtherFemale,
    /// During the modifier owner's current turn, prevent other non-dying players from using or
    /// saving with Tao. Appended to keep existing serialized enum ordinals stable.
    TaoUseSaveCancelForNonDyingOthersDuringOwnerTurn,
    /// Prevent another player whose current HP is at least the modifier owner's HP from selecting
    /// the owner as a card target.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardTargetCancelIfOtherUserHpGteTarget,
    /// Prevent the owner from changing from face-up to face-down. A face-down owner may still
    /// turn face-up.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    TurnFaceDownCancel,
    /// Restore an owner phase after its ordinary skipped/cancelled audit event is recorded.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    RestoreSkippedPhase,
    /// Exact cards recorded in this skill instance's policy state ignore distance.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardDistanceUnlimitedForSkillCardPolicy,
    /// Prevent the owner from changing either face-up/face-down state.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    TurnOverCancel,
    /// Prevent matching cards from another player whose final distance to the owner is greater
    /// than one from selecting the owner.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardTargetCancelIfOtherUserDistanceGtOneFor(CardSelector),
    /// Add to the Sha count required from the modifier owner's opponent in each Duel exchange.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    DuelOpponentResponseCount,
    /// A logical Sha whose physical rank is below this skill's same-id mark ignores distance.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ShaDistanceUnlimitedIfRankLtSkillMark,
    /// A logical Sha whose physical rank is above this skill's same-id mark ignores and does not
    /// consume the ordinary use count.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ShaNoUseCountIfRankGtSkillMark,
    /// Set the owner's final distance to this skill instance's bound target.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    DistanceFromSkillBoundTarget,
    /// Prevent the linked owner from losing the chain mark.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    LinkLossCancel,
    /// Adjust every queried player's draw count while the modifier owner is alive.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    GlobalDrawCount,
    /// Adjust every queried player's Sha use limit while the modifier owner is alive.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    GlobalShaUseLimit,
    /// Permit a hand/equipment judgment rewrite while consuming the configured mark on success.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    JudgmentRewriteHandEquipCostMark(u32),
    /// A logical physical Sha ignores its target-distance limit when the final distance to that
    /// target is at most the card's physical rank.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ShaDistanceUnlimitedIfTargetDistanceLteRank,
    /// Matching cards in the owner's personal zones cannot be gained by another player.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardGainProtectedFor(CardSelector),
    /// Override or adjust the number rules see on the modifier owner's cards.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardNumber,
    /// Permit a red hand/equipment/expansion card to replace a revealed judgment.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    JudgmentRewriteRedHandEquipExpansion,
    /// Permit a black hand/equipment card to replace a revealed judgment (鬼道). ABI 550.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    JudgmentRewriteBlackHandEquip,
    /// The owner's non-locked skills are disabled while the owner has exactly one HP
    /// (禅缘). ABI 552.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    SkillsDisabledWhenHpOne,
    /// Cancel a 桃 save targeting a dying player whose distance from the saver exceeds 2.
    /// Powers 「泣梓」.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    TaoUseSaveCancelForDyingBeyondDistance,
    /// Add to the Shan count when the owner is the Sha source and the target is within distance 1.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ShaResponseCountIfSourceOwnerTargetDistanceLeOne,
    /// The first physical hand card of each suit used by the owner this turn ignores and does
    /// not consume ordinary per-turn use limits.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    FirstHandCardSuitUseUnlimited,
    /// 南蛮实体牌伤害的来源视为持有者（祸首）。ABI 541。
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    NanmanDamageSourceAsOwner,
}

/// Declarative card selector used by card-aware modifier channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardSelector {
    /// Any card.
    Any,
    /// Any kind of 杀.
    Sha,
    /// 桃.
    Tao,
    /// 兵粮寸断.
    Bingliang,
    /// Basic cards.
    Basic,
    /// Cards other than basic cards.
    NonBasic,
    /// Trick cards, including delayed tricks.
    Trick,
    /// Equipment cards.
    Equip,
    /// Spade cards.
    Spade,
    /// Heart cards.
    Heart,
    /// Club cards.
    Club,
    /// Diamond cards.
    Diamond,
    /// Red cards.
    Red,
    /// Black cards.
    Black,
    /// 决斗.
    Juedou,
    /// Black trick cards, including delayed tricks.
    BlackTrick,
    /// Damage-tagged trick cards.
    DamageTrick,
    /// Fire-nature 杀.
    ShaFire,
    /// 火攻.
    Huogong,
    /// 酒（追加在末尾以保持既有序列化变体索引稳定）.
    Jiu,
    /// A physical 酒, even when another modifier changes its effective name.
    PhysicalJiu,
    /// Red 杀 (ordinary/fire/thunder Sha whose effective suit is heart or diamond).
    ShaRed,
    /// 南蛮入侵（追加在末尾以保持既有序列化变体索引稳定）.
    Nanman,
    /// Non-delayed trick cards（普通锦囊；追加在末尾以保持既有序列化变体索引稳定）.
    OrdinaryTrick,
    /// Black Sha (ordinary/fire/thunder Sha whose effective suit is spade or club).
    ShaBlack,
    /// Fire- or thunder-nature Sha.
    ShaElemental,
    /// Equipment other than offensive/defensive mounts.
    EquipNonMount,
    /// Armor or treasure equipment.
    ArmorOrTreasure,
    /// 万箭齐发.
    Wanjian,
    /// Ordinary, non-elemental 杀.
    ShaNormal,
    /// Heart-suit 杀, including elemental variants.
    ShaHeart,
    /// Delayed judge-area trick cards.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    DelayedTrick,
    /// 无懈可击.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    Wuxie,
    /// 过河拆桥.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    Guohe,
    /// 顺手牵羊.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    Shunshou,
    /// Thunder-nature 杀.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ShaThunder,
    /// A/J/Q/K physical card in the queried player's hand.
    HandRankLetter,
    /// The queried player's sole physical hand card.
    SoleHandCard,
    /// Diamond-suit 杀, including elemental variants.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    ShaDiamond,
    /// Physical cards tracked by the modifier's own skill instance.
    ///
    /// Appended for serialized ordinal stability.
    SkillTracked,
    /// Any Sha or damage-tagged trick.
    DamageCard,
    /// 乐不思蜀.
    /// Appended for serialized ordinal stability.
    Lebu,
    /// A currently face-up physical card.
    FaceUp,
}

impl CardSelector {
    pub(crate) fn matches_kind_and_suit(self, kind: CardKind, suit: Suit) -> bool {
        let basic = kind.is_basic();
        match self {
            Self::Any => true,
            Self::Sha => kind.is_sha(),
            Self::Tao => kind == CardKind::Tao,
            Self::Bingliang => kind == CardKind::TrickBingliang,
            Self::Basic => basic,
            Self::NonBasic => !basic,
            Self::Trick => kind.is_trick(),
            Self::Equip => kind.is_equip(),
            Self::Spade => suit == Suit::Spade,
            Self::Heart => suit == Suit::Heart,
            Self::Club => suit == Suit::Club,
            Self::Diamond => suit == Suit::Diamond,
            Self::Red => matches!(suit, Suit::Heart | Suit::Diamond),
            Self::Black => matches!(suit, Suit::Spade | Suit::Club),
            Self::Juedou => kind == CardKind::TrickJuedou,
            Self::BlackTrick => kind.is_trick() && matches!(suit, Suit::Spade | Suit::Club),
            Self::DamageTrick => kind.is_damage_trick(),
            Self::ShaFire => kind == CardKind::ShaFire,
            Self::Huogong => kind == CardKind::TrickHuogong,
            Self::Jiu => kind == CardKind::Jiu,
            Self::PhysicalJiu => kind == CardKind::Jiu,
            Self::ShaRed => kind.is_sha() && matches!(suit, Suit::Heart | Suit::Diamond),
            Self::Nanman => kind == CardKind::TrickNanman,
            Self::OrdinaryTrick => kind.is_trick() && !kind.is_delayed_trick(),
            Self::ShaBlack => kind.is_sha() && matches!(suit, Suit::Spade | Suit::Club),
            Self::ShaElemental => matches!(kind, CardKind::ShaFire | CardKind::ShaThunder),
            Self::EquipNonMount => {
                matches!(
                    kind,
                    CardKind::Weapon | CardKind::Armor | CardKind::Treasure
                )
            }
            Self::ArmorOrTreasure => matches!(kind, CardKind::Armor | CardKind::Treasure),
            Self::Wanjian => kind == CardKind::TrickWanjian,
            Self::ShaNormal => kind == CardKind::Sha,
            Self::ShaHeart => kind.is_sha() && suit == Suit::Heart,
            Self::DelayedTrick => kind.is_delayed_trick(),
            Self::Wuxie => kind == CardKind::TrickWuxie,
            Self::Guohe => kind == CardKind::TrickGuohe,
            Self::Shunshou => kind == CardKind::TrickShunshou,
            Self::ShaThunder => kind == CardKind::ShaThunder,
            Self::ShaDiamond => kind.is_sha() && suit == Suit::Diamond,
            Self::DamageCard => kind.is_sha() || kind.is_damage_trick(),
            Self::Lebu => kind == CardKind::TrickLe,
            Self::HandRankLetter | Self::SoleHandCard | Self::SkillTracked | Self::FaceUp => false,
        }
    }

    pub(crate) fn matches_physical_card(
        self,
        state: &GameState,
        player: PlayerId,
        card: &Card,
        suit: Suit,
    ) -> bool {
        match self {
            Self::SkillTracked => false,
            Self::HandRankLetter => {
                card.owner == player
                    && state
                        .zones
                        .get(card.zone)
                        .is_some_and(|zone| zone.kind == ZoneKind::Hand)
                    && matches!(card.rank, 1 | 11 | 12 | 13)
            }
            Self::SoleHandCard => {
                card.owner == player
                    && state.hand_count(player) == 1
                    && state
                        .zones
                        .get(card.zone)
                        .is_some_and(|zone| zone.kind == ZoneKind::Hand)
            }
            Self::FaceUp => card.face_up,
            _ => self.matches_kind_and_suit(card.kind, suit),
        }
    }

    pub(crate) fn matches_card(
        self,
        physical_kind: CardKind,
        effective_kind: CardKind,
        effective_suit: Suit,
    ) -> bool {
        match self {
            Self::PhysicalJiu => physical_kind == CardKind::Jiu,
            _ => self.matches_kind_and_suit(effective_kind, effective_suit),
        }
    }

    pub(crate) fn tag(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Sha => "sha",
            Self::Tao => "tao",
            Self::Bingliang => "bingliang",
            Self::Basic => "basic",
            Self::NonBasic => "non_basic",
            Self::Trick => "trick",
            Self::Equip => "equip",
            Self::Spade => "spade",
            Self::Heart => "heart",
            Self::Club => "club",
            Self::Diamond => "diamond",
            Self::Red => "red",
            Self::Black => "black",
            Self::Juedou => "juedou",
            Self::BlackTrick => "black_trick",
            Self::DamageTrick => "damage_trick",
            Self::ShaFire => "sha_fire",
            Self::Huogong => "huogong",
            Self::Jiu => "jiu",
            Self::PhysicalJiu => "physical_jiu",
            Self::ShaRed => "sha_red",
            Self::Nanman => "nanman",
            Self::OrdinaryTrick => "ordinary_trick",
            Self::ShaBlack => "sha_black",
            Self::ShaElemental => "sha_elemental",
            Self::EquipNonMount => "equip_non_mount",
            Self::ArmorOrTreasure => "armor_or_treasure",
            Self::Wanjian => "wanjian",
            Self::ShaNormal => "sha_normal",
            Self::ShaHeart => "sha_heart",
            Self::DelayedTrick => "delayed_trick",
            Self::Wuxie => "wuxie",
            Self::Guohe => "guohe",
            Self::Shunshou => "shunshou",
            Self::ShaThunder => "sha_thunder",
            Self::HandRankLetter => "hand_rank_letter",
            Self::SoleHandCard => "sole_hand_card",
            Self::ShaDiamond => "sha_diamond",
            Self::SkillTracked => "skill_tracked",
            Self::DamageCard => "damage_card",
            Self::Lebu => "lebu",
            Self::FaceUp => "face_up",
        }
    }
}

/// How modifiers combine (architecture §9.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombineMode {
    /// Add to running total.
    Add,
    /// Replace running total.
    Override,
    /// Running = min(running, value).
    Min,
    /// Running = max(running, value).
    Max,
    /// Multiply (integer).
    Multiply,
    /// If value != 0, veto query as false / invalid.
    Veto,
}

/// Query-time source for a modifier contribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModifierValueSource {
    /// Owner's current HP, clamped to zero.
    OwnerHp,
    /// Owner's maximum HP.
    OwnerMaxHp,
    /// Owner's current HP minus one, clamped to zero.
    OwnerHpMinusOne,
    /// Owner's current lost HP, clamped to zero.
    OwnerLostHp,
    /// Owner's current lost HP minus one, clamped to zero.
    OwnerLostHpMinusOne,
    /// Owner's current lost HP, with a minimum contribution of one.
    OwnerLostHpMinOne,
    /// Owner's mark whose id equals this modifier's skill definition id.
    OwnerSkillMark,
    /// Owner's skill-pile (expansion) card count for this modifier's skill definition id.
    OwnerSkillPileCount,
    /// Number of distinct groups among alive players' characters.
    AliveGroupCount,
    /// Number of alive players whose character sex is female.
    AliveFemaleCount,
    /// Number of other alive players, capped at four.
    AliveOtherCountCappedFour,
    /// One while the owner's equipment zone contains no mount.
    OwnerNoMount,
    /// One while the owner's hand is empty.
    OwnerHandEmpty,
    /// One while the owner's equipment zone contains a treasure.
    OwnerHasTreasure,
    /// Number of alive players with at least one mount equipped, with a minimum of one.
    AliveMountedPlayerCountMinOne,
    /// One while the owner's equipment-card count is odd.
    OwnerEquipCountOdd,
    /// One while the owner's equipment-card count is even (including zero).
    OwnerEquipCountEven,
    /// Half the owner's equipment-card count, rounded down.
    OwnerEquipCountHalfFloor,
    /// One while the owner's current HP is greater than two.
    OwnerHpGtTwo,
    /// One while the owner's current HP is at most two.
    OwnerHpLeTwo,
    /// One while the owner's lost HP is at least two.
    OwnerLostHpAtLeastTwo,
    /// One while every other alive player is within the owner's attack range.
    OwnerAllOthersInAttackRange,
    /// One before the first Sha has been used in the current turn.
    OwnerShaUsedZero,
    /// Distinct basic/trick/equipment categories among the owner's non-damage hand cards.
    OwnerNonDamageHandTypeCount,
    /// Damage dealt by the owner during their current turn.
    OwnerTurnDamageDealt,
    /// One while the owner's equipment zone contains an effective heart card.
    OwnerHasHeartEquip,
    /// One after the owner has used at least one Sha in the current turn.
    OwnerShaUsedNonzero,
    /// Number of other alive players whose character group is `qun`.
    AliveOtherQunCount,
    /// One while the owner's equipment zone contains a weapon.
    OwnerHasWeapon,
    /// Current round number, capped at five.
    RoundNumberCappedFive,
    /// One while the round number is at least the alive `qun` character count.
    RoundAtLeastAliveQunCount,
    /// One while the round number is below the alive `qun` character count.
    RoundBelowAliveQunCount,
    /// Number of alive players with at least one rank-K card in equip or judge.
    AliveFieldRankThirteenPlayerCount,
    /// Half the mount cards on living players' fields, rounded up.
    FieldMountCardCountCeilHalf,
    /// One while the owner uniquely has the most hand cards among living players.
    OwnerUniqueMaxHand,
    /// One while the owner's equipment zone contains no armor.
    OwnerArmorEmpty,
    /// One while the owner is at full health.
    OwnerHealthy,
    /// Owner's mark with the specified stable definition id.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerMark(u32),
    /// One after the owner has used exactly one Sha in their current turn.
    OwnerShaUsedExactlyOne,
    /// One after the owner has used exactly two Sha in their current turn.
    OwnerShaUsedExactlyTwo,
    /// One while the owner has maximum hand size and this skill has not fired this turn.
    OwnerMaxHandBeforeSkillFiresThisTurn,
    /// One while the owner's equipment-card count is exactly one.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerEquipCountExactlyOne,
    /// One while no alive player is wounded or at least four alive players are wounded.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    AliveWoundedCountZeroOrAtLeastFour,
    /// One while at least the configured number of living players are wounded.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    AliveWoundedCountAtLeast(u32),
    /// One while the owner's hand count is greater than the other damage participant's.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerHandGreaterThanOther,
    /// One while the owner's hand count is less than the other damage participant's.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerHandLessThanOther,
    /// Number of cards in the owner's equipment zone.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerEquipCount,
    /// One while the owner's face-up/face-down state differs from the other damage participant's.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerTurnedOverDiffOther,
    /// One while the owner's hand and equipment counts are each no smaller than the other
    /// damage participant's corresponding count.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerHandEquipGteOther,
    /// One before the owner has dealt any logical-Sha damage event in the current turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerShaDamageDealtZeroThisTurn,
    /// Positive one when the queried player's hand count exceeds HP, otherwise negative one.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    QueriedPlayerHandGtHpSigned,
    /// One while the owner and the other damage participant have different linked states.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerLinkedDiffOther,
    /// One while the queried living player differs from the owner and has a judge-zone card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    QueriedPlayerJudgeNonemptyOtherThanOwner,
    /// One while the queried living player differs from the owner and is linked.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    QueriedPlayerLinkedOtherThanOwner,
    /// One while the queried living player differs from the modifier owner.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    QueriedPlayerOtherThanOwner,
    /// One when the queried distance target is the owner's ally, the distance source is that
    /// target's enemy, and the target is not the owner.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    QueriedSourceEnemyOwnerAllyTargetOtherThanOwner,
    /// One when the owner's equipment count meets the configured minimum; otherwise zero.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerEquipCountAtLeast(u32),
    /// The equip count itself once it reaches the threshold, else 0.
    OwnerEquipCountFloor(u32),
    /// Number of distinct effective suits used from physical cards in the owner's hand during
    /// the current turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerDistinctHandCardSuitsUsedThisTurn,
    /// Number of cards proactively used by the owner during the current turn, or zero outside
    /// that owner's turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerCurrentTurnCardUseCount,
    /// One while the owner is a living `qun` character and the other damage participant is a
    /// living `shu` character.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerQunOtherShu,
    /// One before the owner has proactively used a card in their current Play phase; otherwise
    /// zero.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerPlayPhaseCardUseCountZero,
    /// Originating active skill's use count in the current turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSkillUseCount,
    /// Living wounded players, plus other living Wu players while the owner is the Lord and
    /// has enabled 归命; clamped to a minimum of one.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    AliveWoundedOrGuimingWuCountMinOne,
    /// One when the queried living player is linked and this modifier belongs to the first
    /// living enabled owner of its skill definition.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    QueriedPlayerLinkedFirstLivingSkillOwner,
    /// One while the queried living player has the configured mark.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    QueriedPlayerMarkPositive(u32),
    /// One while the queried wounded other player is in the owner's attack range.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    QueriedPlayerWoundedInOwnerAttackRangeOther,
    /// One during the owner's Play phase while this skill instance has fired fewer than the
    /// configured number of times in that phase.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSkillPlayPhaseFireCountBelow(u32),
    /// One during the owner's current turn while every other living player's final distance
    /// from the owner is at most one.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerCurrentTurnAllOthersDistanceAtMostOne,
    /// Negative of the card count in the owner's numbered skill pile. ABI 533 (屯田).
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSkillPileCountNegative(u32),
    /// Positive skill-pile count gated by a specific skill definition.
    OwnerSkillPileCountOf(u32),
    /// First little-endian `u32` in this skill instance's state blob.
    ///
    /// Appended to keep existing serialized enum ordinals stable. ABI 522.
    OwnerSkillStateU32,
    /// One for a queried living Qun player while this modifier's living owner is the Lord.
    /// Appended to keep existing serialized enum ordinals stable. ABI 553.
    QueriedPlayerQunOwnerLordAlive,
    /// One while the owner's character card is linked. ABI 611.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerLinked,
    /// Negative of the first little-endian `u32` in this skill instance's state blob. ABI 612.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSkillStateU32Negative,
    /// Number of Sha cards in the owner's hand (任城 pxrencheng: hand_limit +X, X=杀数).
    /// Appended to keep existing serialized enum ordinals stable. ABI 613.
    OwnerShaHandCount,
    /// One while the owner's equipment zone lacks a weapon (shahai pingxu 出向距离).
    /// Appended to keep existing serialized enum ordinals stable. Value-source ladder.
    OwnerNoWeapon,
    /// One while the owner's equipment zone contains an armor (shahai 中兴护甲/平叙).
    /// Appended to keep existing serialized enum ordinals stable. Value-source ladder.
    OwnerHasArmorEquip,
}

/// Static modifier definition (content).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModifierDefinition {
    /// Definition id.
    pub id: u32,
    /// Skill that grants this.
    pub skill_def: SkillDefId,
    /// Target channel.
    pub target: ModifierTarget,
    /// Combine mode.
    pub mode: CombineMode,
    /// Optional query-time value source.
    pub value_source: Option<ModifierValueSource>,
    /// Fixed value (Phase 3; formulas in Phase 4).
    pub value: i32,
    /// Apply order within same mode (lower first).
    pub order: i32,
    /// Label for explain.
    pub label_key: String,
}

/// Active modifier instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveModifier {
    /// Instance skill.
    pub skill_id: SkillId,
    /// Skill def.
    pub skill_def: SkillDefId,
    /// Owner (source of modifier).
    pub owner: PlayerId,
    /// Definition id.
    pub def_id: u32,
    /// Target.
    pub target: ModifierTarget,
    /// Mode.
    pub mode: CombineMode,
    /// Optional query-time value source.
    pub value_source: Option<ModifierValueSource>,
    /// Value.
    pub value: i32,
    /// Order.
    pub order: i32,
    /// Enabled.
    pub enabled: bool,
    /// Label.
    pub label_key: String,
}

/// Bootstrap modifier defs.
pub fn bootstrap_modifier_defs() -> Vec<ModifierDefinition> {
    use crate::trigger::{BOOTSTRAP_MASHU, BOOTSTRAP_QICAO};
    vec![
        ModifierDefinition {
            id: 1,
            skill_def: SkillDefId::new(BOOTSTRAP_MASHU),
            target: ModifierTarget::DistanceFrom,
            mode: CombineMode::Add,
            value_source: None,
            value: -1, // 马术: 计算距离时 -1
            order: 0,
            label_key: "skill.mashu".into(),
        },
        ModifierDefinition {
            id: 2,
            skill_def: SkillDefId::new(BOOTSTRAP_QICAO),
            target: ModifierTarget::AttackRange,
            mode: CombineMode::Add,
            value_source: None,
            value: 1,
            order: 0,
            label_key: "skill.qicao_range".into(),
        },
    ]
}

/// Active modifier index (derived; not hashed).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveModifierIndex {
    entries: Vec<ActiveModifier>,
}

impl ActiveModifierIndex {
    /// Empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// Len.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Clear.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Register skill modifiers (catalog from content registry).
    pub fn register_skill(
        &mut self,
        skill_id: SkillId,
        skill_def: SkillDefId,
        owner: PlayerId,
        content: &ContentRegistry,
    ) {
        for d in content.modifier_defs_for_skill(skill_def) {
            self.entries.push(ActiveModifier {
                skill_id,
                skill_def,
                owner,
                def_id: d.id,
                target: d.target,
                mode: d.mode,
                value_source: d.value_source,
                value: d.value,
                order: d.order,
                enabled: true,
                label_key: d.label_key,
            });
        }
        self.entries.sort_by_key(|e| (e.skill_id.raw(), e.def_id));
    }

    /// Unregister.
    pub fn unregister_skill(&mut self, skill_id: SkillId) {
        self.entries.retain(|e| e.skill_id != skill_id);
    }

    /// Set enabled flag for all rows of a skill instance.
    pub fn set_skill_enabled(&mut self, skill_id: SkillId, enabled: bool) {
        for entry in &mut self.entries {
            if entry.skill_id == skill_id {
                entry.enabled = enabled;
            }
        }
    }

    /// Rebuild.
    pub fn rebuild_from<'a, I>(&mut self, skills: I, content: &ContentRegistry)
    where
        I: IntoIterator<Item = (SkillId, SkillDefId, PlayerId, bool)>,
    {
        self.clear();
        for (sid, sdef, owner, enabled) in skills {
            for d in content.modifier_defs_for_skill(sdef) {
                self.entries.push(ActiveModifier {
                    skill_id: sid,
                    skill_def: sdef,
                    owner,
                    def_id: d.id,
                    target: d.target,
                    mode: d.mode,
                    value_source: d.value_source,
                    value: d.value,
                    order: d.order,
                    enabled,
                    label_key: d.label_key,
                });
            }
        }
        self.entries.sort_by_key(|e| (e.skill_id.raw(), e.def_id));
    }

    /// Collect modifiers for a target channel, stable order.
    pub fn for_target(&self, target: ModifierTarget) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|e| e.enabled && e.target == target)
            .collect();
        // Combine order: mode class then order then skill_id
        v.sort_by_key(|e| (mode_rank(e.mode), e.order, e.skill_id.raw(), e.def_id));
        v
    }

    /// First configured mark cost for this player's enabled judgment rewrite.
    pub fn judgment_rewrite_cost_mark(&self, player: PlayerId) -> Option<u32> {
        self.entries.iter().find_map(|entry| match entry.target {
            ModifierTarget::JudgmentRewriteHandEquipCostMark(mark)
                if entry.enabled && entry.owner == player =>
            {
                Some(mark)
            }
            _ => None,
        })
    }

    /// Collect owner-local and living-owner global attack-range modifiers in one stable order.
    pub fn for_attack_range(
        &self,
        player: PlayerId,
        global_owner_alive: impl Fn(PlayerId) -> bool,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::AttackRange => entry.owner == player,
                        ModifierTarget::GlobalAttackRange => global_owner_alive(entry.owner),
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect owner-local and living-owner global hand-limit modifiers in stable order.
    pub fn for_hand_limit(
        &self,
        player: PlayerId,
        global_owner_alive: impl Fn(PlayerId) -> bool,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::HandLimit => entry.owner == player,
                        ModifierTarget::GlobalHandLimit => global_owner_alive(entry.owner),
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect owner-local and living-owner global draw-count modifiers in stable order.
    pub fn for_draw_count(
        &self,
        player: PlayerId,
        global_owner_alive: impl Fn(PlayerId) -> bool,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::DrawCount => entry.owner == player,
                        ModifierTarget::GlobalDrawCount => global_owner_alive(entry.owner),
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect owner-local and living-owner global Sha-use-limit modifiers in stable order.
    pub fn for_sha_use_limit(
        &self,
        player: PlayerId,
        global_owner_alive: impl Fn(PlayerId) -> bool,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::ShaUseLimit => entry.owner == player,
                        ModifierTarget::GlobalShaUseLimit => global_owner_alive(entry.owner),
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect active hand-limit exclusions matching one concrete card.
    pub fn for_hand_limit_card(
        &self,
        state: &GameState,
        player: PlayerId,
        card: &Card,
        kind: CardKind,
        suit: Suit,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::ShaIgnoredHandLimit => kind.is_sha(),
                        ModifierTarget::CardIgnoredHandLimit(selector) => {
                            if selector == CardSelector::SkillTracked {
                                entry.owner == player
                                    && state.skills.get(entry.skill_id).is_some_and(|skill| {
                                        skill.tracked_cards().contains(&card.id)
                                    })
                            } else {
                                selector.matches_physical_card(state, player, card, suit)
                            }
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect distance-limit exemptions matching one concrete card.
    pub fn for_card_distance(&self, kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::CardDistanceUnlimited => true,
                        ModifierTarget::CardDistanceUnlimitedFor(selector) => {
                            selector.matches_kind_and_suit(kind, suit)
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect target-owned discard protections matching one physical equipment card.
    pub fn for_card_discard_protection(&self, kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardDiscardProtectedFor(selector)
                            if selector.matches_kind_and_suit(kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect target-owned gain protections matching one physical card.
    pub fn for_card_gain_protection(&self, kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardGainProtectedFor(selector)
                            if selector.matches_kind_and_suit(kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect target-aware distance exemptions matching one concrete card.
    pub fn for_card_distance_if_target_hand_gte_owner(
        &self,
        kind: CardKind,
        suit: Suit,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardDistanceUnlimitedIfTargetHandGteOwnerFor(selector)
                            if selector.matches_kind_and_suit(kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect ordinary use-limit exemptions matching one concrete card.
    pub fn for_card_use_limit(
        &self,
        physical_kind: CardKind,
        effective_kind: CardKind,
        effective_suit: Suit,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::CardUseUnlimited => true,
                        ModifierTarget::CardUseUnlimitedFor(selector) => {
                            selector.matches_card(physical_kind, effective_kind, effective_suit)
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect target-count modifiers matching one concrete card use.
    pub fn for_card_target_limit(
        &self,
        physical_kind: CardKind,
        effective_kind: CardKind,
        effective_suit: Suit,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardTargetLimitFor(selector)
                            if selector.matches_card(
                                physical_kind,
                                effective_kind,
                                effective_suit,
                            )
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect outgoing Sha response locks matching one concrete effective card.
    pub fn for_sha_unrespondable(&self, kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::ShaUnrespondableFor(selector)
                            if selector.matches_kind_and_suit(kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect source-owned response locks matching the current logical card.
    pub fn for_card_use_unrespondable(&self, kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardUseUnrespondableFor(selector)
                            if selector.matches_kind_and_suit(kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect generic and card-matching outgoing Sha damage modifiers.
    pub fn for_sha_damage(&self, kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::ShaDamage => true,
                        ModifierTarget::ShaDamageFor(selector) => {
                            selector.matches_kind_and_suit(kind, suit)
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect bounded distance-limit overrides matching one concrete card.
    pub fn for_card_distance_limit(&self, kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardDistanceLimitFor(selector)
                            if selector.matches_kind_and_suit(kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect rule-category overrides matching a physical card category and effective suit.
    pub fn for_card_name(
        &self,
        state: &GameState,
        player: PlayerId,
        card: &Card,
        suit: Suit,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardName(selector)
                            if if selector == CardSelector::SkillTracked {
                                state.skills.get(entry.skill_id).is_some_and(|skill| {
                                    skill.tracked_cards().contains(&card.id)
                                })
                            } else {
                                selector.matches_physical_card(state, player, card, suit)
                            }
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect damage-nature overrides matching one physical card category and effective suit.
    pub fn for_card_nature(&self, kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardNatureFor(selector)
                            if selector.matches_kind_and_suit(kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect card-use cancellation modifiers matching a logical card kind.
    pub fn for_card_use(&self, logical_kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::CardUseCancel => true,
                        ModifierTarget::CardUseCancelFor(selector) => {
                            selector.matches_kind_and_suit(logical_kind, suit)
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect response-cancellation modifiers matching one logical card kind and suit.
    pub fn for_card_response(&self, logical_kind: CardKind, suit: Suit) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::CardResponseCancel => true,
                        ModifierTarget::CardResponseCancelFor(selector) => {
                            selector.matches_kind_and_suit(logical_kind, suit)
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect target-side cancellation modifiers matching a logical card kind.
    pub fn for_card_target(
        &self,
        user: PlayerId,
        target: PlayerId,
        logical_kind: CardKind,
        suit: Suit,
        from_current: bool,
        use_ban_pairs: &[(PlayerId, PlayerId)],
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::CardTargetCancelFor(selector) => {
                            selector.matches_kind_and_suit(logical_kind, suit)
                        }
                        ModifierTarget::CardTargetCancelFromCurrentFor(selector) => {
                            from_current && selector.matches_kind_and_suit(logical_kind, suit)
                        }
                        ModifierTarget::CardTargetCancelTurnBannedFor(selector) => {
                            user != target
                                && use_ban_pairs.contains(&(user, target))
                                && selector.matches_kind_and_suit(logical_kind, suit)
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect armor-owned target cancellation modifiers matching a logical card kind.
    pub fn for_armor_card_target(
        &self,
        logical_kind: CardKind,
        suit: Suit,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::ArmorCardTargetCancelFor(selector)
                            if selector.matches_kind_and_suit(logical_kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect hand-count/armor-conditional target cancellation modifiers.
    pub fn for_card_target_if_other_user_hand_gte_target_and_armor_empty(
        &self,
        logical_kind: CardKind,
        suit: Suit,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardTargetCancelIfOtherUserHandGteTargetAndArmorEmptyFor(selector)
                            if selector.matches_kind_and_suit(logical_kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect distance-conditional target cancellation modifiers matching one logical card.
    pub fn for_card_target_if_other_user_distance_gt_one(
        &self,
        logical_kind: CardKind,
        suit: Suit,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardTargetCancelIfOtherUserDistanceGtOneFor(selector)
                            if selector.matches_kind_and_suit(logical_kind, suit)
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect target-owned damage cancellation modifiers for the resolved nature.
    pub fn for_damage_taken_cancel(&self, nature: u8) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::DamageTakenCancel => true,
                        ModifierTarget::DamageTakenCancelForNature(value) => value == nature,
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect source-owned damage amount modifiers for the original nature.
    pub fn for_damage_amount(&self, nature: u8, logical_sha: bool) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::DamageAmount => true,
                        ModifierTarget::DamageAmountForNature(value) => value == nature,
                        ModifierTarget::DamageAmountForSha => logical_sha,
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect target-owned and living-owner global damage amount modifiers.
    pub fn for_damage_taken_amount(
        &self,
        nature: u8,
        target: PlayerId,
        global_owner_alive: impl Fn(PlayerId) -> bool,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::DamageTakenAmount => entry.owner == target,
                        ModifierTarget::DamageTakenAmountForNature(value) => {
                            entry.owner == target && value == nature
                        }
                        ModifierTarget::GlobalDamageTakenAmount => global_owner_alive(entry.owner),
                        ModifierTarget::GlobalDamageTakenAmountForNature(value) => {
                            value == nature && global_owner_alive(entry.owner)
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect armor-owned damage amount modifiers for the resolved nature.
    pub fn for_armor_damage_taken_amount(&self, nature: u8) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::ArmorDamageTakenAmountForNature(value)
                            if value == nature
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect target-owned and living-owner global final damage amount modifiers.
    pub fn for_damage_taken_final_amount(
        &self,
        nature: u8,
        target: PlayerId,
        global_owner_alive: impl Fn(PlayerId) -> bool,
    ) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && match entry.target {
                        ModifierTarget::DamageTakenFinalAmount => entry.owner == target,
                        ModifierTarget::DamageTakenFinalAmountForNature(value) => {
                            entry.owner == target && value == nature
                        }
                        ModifierTarget::GlobalDamageTakenFinalAmount => {
                            global_owner_alive(entry.owner)
                        }
                        ModifierTarget::GlobalDamageTakenFinalAmountForNature(value) => {
                            value == nature && global_owner_alive(entry.owner)
                        }
                        _ => false,
                    }
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }

    /// Collect forced-all-target modifiers matching one concrete card kind.
    pub fn for_card_targets_all(&self, selector: CardSelector) -> Vec<&ActiveModifier> {
        let mut v: Vec<&ActiveModifier> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.enabled
                    && matches!(
                        entry.target,
                        ModifierTarget::CardTargetsAllFor(found) if found == selector
                    )
            })
            .collect();
        v.sort_by_key(|entry| {
            (
                mode_rank(entry.mode),
                entry.order,
                entry.skill_id.raw(),
                entry.def_id,
            )
        });
        v
    }
}

fn mode_rank(m: CombineMode) -> u8 {
    match m {
        CombineMode::Override => 0,
        CombineMode::Add => 1,
        CombineMode::Multiply => 2,
        CombineMode::Min => 3,
        CombineMode::Max => 4,
        CombineMode::Veto => 5,
    }
}

/// Apply modifiers to a base integer; return final value + sources.
pub fn apply_int_modifiers(
    base: i32,
    mods: &[&ActiveModifier],
    owner_filter: Option<PlayerId>,
) -> (i32, Vec<ModifierSource>, bool) {
    apply_int_modifiers_by(base, mods, owner_filter, |modifier| modifier.value)
}

/// Apply modifiers after resolving query-time values from the current state.
pub fn apply_int_modifiers_for_state(
    state: &GameState,
    base: i32,
    mods: &[&ActiveModifier],
    owner_filter: Option<PlayerId>,
) -> (i32, Vec<ModifierSource>, bool) {
    apply_int_modifiers_for_state_with_other(state, base, mods, owner_filter, None)
}

/// Apply modifiers with the other participant of a contextual query.
pub fn apply_int_modifiers_for_state_with_other(
    state: &GameState,
    base: i32,
    mods: &[&ActiveModifier],
    owner_filter: Option<PlayerId>,
    other: Option<PlayerId>,
) -> (i32, Vec<ModifierSource>, bool) {
    apply_int_modifiers_for_state_with_context(state, base, mods, owner_filter, None, other)
}

/// Apply modifiers with an explicit queried subject and optional other participant.
pub fn apply_int_modifiers_for_state_with_context(
    state: &GameState,
    base: i32,
    mods: &[&ActiveModifier],
    owner_filter: Option<PlayerId>,
    queried: Option<PlayerId>,
    other: Option<PlayerId>,
) -> (i32, Vec<ModifierSource>, bool) {
    apply_int_modifiers_by(base, mods, owner_filter, |modifier| {
        resolved_value_with_context(state, modifier, queried, other)
    })
}

fn apply_int_modifiers_by(
    base: i32,
    mods: &[&ActiveModifier],
    owner_filter: Option<PlayerId>,
    value_of: impl Fn(&ActiveModifier) -> i32,
) -> (i32, Vec<ModifierSource>, bool) {
    let mut value = base;
    let mut sources = Vec::new();
    let mut vetoed = false;

    // Pass 1: overrides (last override wins among filtered)
    for m in mods.iter().filter(|m| m.mode == CombineMode::Override) {
        if let Some(o) = owner_filter {
            if m.owner != o {
                continue;
            }
        }
        let contribution = value_of(m);
        value = contribution;
        sources.push(src(m, contribution));
    }

    // Pass 2: add
    for m in mods.iter().filter(|m| m.mode == CombineMode::Add) {
        if let Some(o) = owner_filter {
            // DistanceFrom: only owner's outgoing distance mods
            if m.owner != o {
                continue;
            }
        }
        let contribution = value_of(m);
        value = value.saturating_add(contribution);
        sources.push(src(m, contribution));
    }

    // Pass 3: multiply
    for m in mods.iter().filter(|m| m.mode == CombineMode::Multiply) {
        if let Some(o) = owner_filter {
            if m.owner != o {
                continue;
            }
        }
        let contribution = value_of(m);
        value = value.saturating_mul(contribution);
        sources.push(src(m, contribution));
    }

    // Pass 4: min / max
    for m in mods.iter().filter(|m| m.mode == CombineMode::Min) {
        if let Some(o) = owner_filter {
            if m.owner != o {
                continue;
            }
        }
        let contribution = value_of(m);
        value = value.min(contribution);
        sources.push(src(m, contribution));
    }
    for m in mods.iter().filter(|m| m.mode == CombineMode::Max) {
        if let Some(o) = owner_filter {
            if m.owner != o {
                continue;
            }
        }
        let contribution = value_of(m);
        value = value.max(contribution);
        sources.push(src(m, contribution));
    }

    // Pass 5: veto
    for m in mods.iter().filter(|m| m.mode == CombineMode::Veto) {
        if let Some(o) = owner_filter {
            if m.owner != o {
                continue;
            }
        }
        let contribution = value_of(m);
        if contribution != 0 {
            vetoed = true;
            sources.push(src(m, contribution));
        }
    }

    (value, sources, vetoed)
}

/// Apply distance modifiers and return an optional final distance override.
pub fn apply_distance_modifiers(
    state: &GameState,
    base: i32,
    from: PlayerId,
    to: PlayerId,
) -> (i32, Vec<ModifierSource>, Option<i32>) {
    let mut sources = Vec::new();
    let mut value = base;

    let from_mods = state.modifiers.for_target(ModifierTarget::DistanceFrom);
    for m in from_mods {
        if m.owner != from {
            continue;
        }
        if m.mode == CombineMode::Add {
            let contribution = resolved_value(state, m);
            value += contribution;
            sources.push(src(m, contribution));
        }
    }
    let to_mods = state.modifiers.for_target(ModifierTarget::DistanceTo);
    for m in to_mods {
        if m.owner != to {
            continue;
        }
        if m.mode == CombineMode::Add {
            let contribution = resolved_value(state, m);
            value += contribution;
            sources.push(src(m, contribution));
        }
    }
    let global_to_mods = state.modifiers.for_target(ModifierTarget::GlobalDistanceTo);
    for m in global_to_mods {
        if !state.players.get(m.owner).is_some_and(|owner| owner.alive)
            || m.mode != CombineMode::Add
        {
            continue;
        }
        let contribution = resolved_value_with_context(state, m, Some(to), Some(from));
        value += contribution;
        sources.push(src(m, contribution));
    }
    let mut final_override = None;
    if matches!(
        (state.players.get(from), state.players.get(to)),
        (Some(from_player), Some(to_player)) if from_player.hp >= to_player.hp
    ) {
        for m in state
            .modifiers
            .for_target(ModifierTarget::DistanceFromHpGteTarget)
        {
            if m.owner != from || m.mode != CombineMode::Override {
                continue;
            }
            let value = resolved_value(state, m);
            final_override = Some(value);
            sources.push(src(m, value));
        }
    }
    for m in state
        .modifiers
        .for_target(ModifierTarget::DistanceFromSkillBoundTarget)
    {
        if m.mode != CombineMode::Override
            || !super::skill_bound_target_modifier_applies(state, &m, from, to)
        {
            continue;
        }
        let value = resolved_value(state, m);
        final_override = Some(value);
        sources.push(src(m, value));
    }
    // Distance never below 1 when different players and both alive (caller clamps)
    (value, sources, final_override)
}

fn resolved_value(state: &GameState, modifier: &ActiveModifier) -> i32 {
    resolved_value_with_context(state, modifier, None, None)
}

fn resolved_value_with_context(
    state: &GameState,
    modifier: &ActiveModifier,
    queried: Option<PlayerId>,
    other: Option<PlayerId>,
) -> i32 {
    let source = match modifier.value_source {
        None => return modifier.value,
        Some(source) => source,
    };
    let player = state.players.get(modifier.owner);
    let value = match source {
        ModifierValueSource::OwnerHp => player.map(|player| player.hp.max(0)).unwrap_or(0),
        ModifierValueSource::OwnerMaxHp => player
            .map(|player| player.max_hp.min(i32::MAX as u32) as i32)
            .unwrap_or(0),
        ModifierValueSource::OwnerHpMinusOne => player
            .map(|player| player.hp.saturating_sub(1).max(0))
            .unwrap_or(0),
        ModifierValueSource::OwnerLostHp => player
            .map(|player| (player.max_hp.min(i32::MAX as u32) as i32 - player.hp).max(0))
            .unwrap_or(0),
        ModifierValueSource::OwnerLostHpMinusOne => player
            .map(|player| {
                (player.max_hp.min(i32::MAX as u32) as i32)
                    .saturating_sub(player.hp)
                    .saturating_sub(1)
                    .max(0)
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerLostHpMinOne => player
            .map(|player| (player.max_hp.min(i32::MAX as u32) as i32 - player.hp).max(1))
            .unwrap_or(1),
        ModifierValueSource::OwnerSkillMark => player
            .and_then(|player| {
                player
                    .marks
                    .iter()
                    .find(|(mark, _)| *mark == modifier.skill_def.raw())
            })
            .map(|(_, count)| *count)
            .unwrap_or(0),
        ModifierValueSource::OwnerSkillPileCount => state
            .skill_pile_cards(modifier.owner, modifier.skill_def.raw())
            .len()
            .min(i32::MAX as usize) as i32,
        ModifierValueSource::OwnerSkillStateU32 => state
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == modifier.owner && skill.def == modifier.skill_def)
            .and_then(|skill| skill.state_blob.get(..4))
            .map(|raw| u32::from_le_bytes(raw.try_into().unwrap()).min(i32::MAX as u32) as i32)
            .unwrap_or(0),
        ModifierValueSource::OwnerSkillStateU32Negative => state
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == modifier.owner && skill.def == modifier.skill_def)
            .and_then(|skill| skill.state_blob.get(..4))
            .map(|raw| -(u32::from_le_bytes(raw.try_into().unwrap()).min(i32::MAX as u32) as i32))
            .unwrap_or(0),
        ModifierValueSource::QueriedPlayerQunOwnerLordAlive => queried
            .filter(|queried| {
                state
                    .players
                    .get(*queried)
                    .is_some_and(|player| player.alive)
                    && state.player_group(*queried) == Some("qun")
                    && state
                        .players
                        .get(modifier.owner)
                        .is_some_and(|owner| owner.alive && owner.identity == Identity::Lord)
            })
            .map_or(0, |_| 1),
        ModifierValueSource::OwnerSkillUseCount => state
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == modifier.owner && skill.def == modifier.skill_def)
            .map(|skill| skill.use_count.min(i32::MAX as u32) as i32)
            .unwrap_or(0),
        ModifierValueSource::OwnerLinked => i32::from(player.is_some_and(|participant| {
            participant.alive
                && participant
                    .marks
                    .iter()
                    .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
        })),
        ModifierValueSource::AliveWoundedOrGuimingWuCountMinOne => {
            alive_wounded_or_guiming_wu_count(state, modifier.owner).max(1)
        }
        ModifierValueSource::OwnerMark(mark) => player
            .and_then(|player| player.marks.iter().find(|(id, _)| *id == mark))
            .map(|(_, count)| *count)
            .unwrap_or(0),
        ModifierValueSource::AliveGroupCount => super::alive_group_count(state),
        ModifierValueSource::AliveFemaleCount => state
            .players
            .iter_live()
            .filter(|player| player.alive)
            .filter_map(|player| state.content.character(player.character_def))
            .filter(|character| character.sex == "female")
            .count()
            .min(i32::MAX as usize) as i32,
        ModifierValueSource::AliveOtherCountCappedFour => state
            .players
            .iter_live()
            .filter(|player| player.alive && player.id != modifier.owner)
            .count()
            .min(4) as i32,
        ModifierValueSource::OwnerNoMount => player
            .map(|_| {
                let has_mount = state
                    .zones
                    .find(ZoneKind::Equip, modifier.owner)
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| {
                        zone.cards.iter().any(|card| {
                            state.cards.get(*card).is_some_and(|card| {
                                matches!(card.kind, CardKind::HorseOff | CardKind::HorseDef)
                            })
                        })
                    });
                i32::from(!has_mount)
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerHandEmpty => player
            .map(|_| i32::from(state.hand_count(modifier.owner) == 0))
            .unwrap_or(0),
        ModifierValueSource::OwnerHasTreasure => player
            .map(|_| {
                i32::from(
                    state
                        .zones
                        .find(ZoneKind::Equip, modifier.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| {
                            zone.cards.iter().any(|card| {
                                state
                                    .cards
                                    .get(*card)
                                    .is_some_and(|card| card.kind == CardKind::Treasure)
                            })
                        }),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::AliveMountedPlayerCountMinOne => state
            .players
            .iter_live()
            .filter(|player| player.alive)
            .filter(|player| {
                state
                    .zones
                    .find(ZoneKind::Equip, player.id)
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| {
                        zone.cards.iter().any(|card| {
                            state.cards.get(*card).is_some_and(|card| {
                                matches!(card.kind, CardKind::HorseOff | CardKind::HorseDef)
                            })
                        })
                    })
            })
            .count()
            .max(1)
            .min(i32::MAX as usize)
            as i32,
        ModifierValueSource::OwnerEquipCountOdd => player
            .map(|_| i32::from(owner_equip_count(state, modifier.owner) % 2 == 1))
            .unwrap_or(0),
        ModifierValueSource::OwnerEquipCountEven => player
            .map(|_| i32::from(owner_equip_count(state, modifier.owner) % 2 == 0))
            .unwrap_or(0),
        ModifierValueSource::OwnerEquipCountHalfFloor => player
            .map(|_| {
                owner_equip_count(state, modifier.owner)
                    .saturating_div(2)
                    .min(i32::MAX as usize) as i32
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerEquipCount => player
            .map(|_| owner_equip_count(state, modifier.owner).min(i32::MAX as usize) as i32)
            .unwrap_or(0),
        ModifierValueSource::OwnerSkillPileCountNegative(skill_def) => {
            -(state
                .skill_pile_cards(modifier.owner, skill_def)
                .len()
                .min(i32::MAX as usize) as i32)
        }
        ModifierValueSource::OwnerSkillPileCountOf(skill_def) => state
            .skill_pile_cards(modifier.owner, skill_def)
            .len()
            .min(i32::MAX as usize)
            as i32,
        ModifierValueSource::OwnerEquipCountAtLeast(threshold) => player
            .map(|_| i32::from(owner_equip_count(state, modifier.owner) >= threshold as usize))
            .unwrap_or(0),
        // Count-gated floor: the equip count itself when it reaches the threshold,
        // otherwise 0 (upstream 白虎骁骑 cardUsable/globalFrom/draw thresholds).
        ModifierValueSource::OwnerEquipCountFloor(threshold) => player
            .map(|_| {
                let count = owner_equip_count(state, modifier.owner);
                if count >= threshold as usize {
                    count.min(i32::MAX as usize) as i32
                } else {
                    0
                }
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerEquipCountExactlyOne => player
            .map(|_| i32::from(owner_equip_count(state, modifier.owner) == 1))
            .unwrap_or(0),
        ModifierValueSource::OwnerHpGtTwo => {
            player.map(|player| i32::from(player.hp > 2)).unwrap_or(0)
        }
        ModifierValueSource::OwnerHpLeTwo => {
            player.map(|player| i32::from(player.hp <= 2)).unwrap_or(0)
        }
        ModifierValueSource::OwnerLostHpAtLeastTwo => player
            .map(|player| {
                i32::from((player.max_hp.min(i32::MAX as u32) as i32 - player.hp).max(0) >= 2)
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerAllOthersInAttackRange => player
            .map(|_| {
                i32::from(super::all_other_players_in_attack_range(
                    state,
                    modifier.owner,
                ))
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerShaUsedZero => {
            i32::from(state.turn.flag(crate::state::turn_flag::SHA_PLAYED) == 0)
        }
        ModifierValueSource::OwnerShaUsedNonzero => i32::from(
            modifier.owner == state.turn.current
                && state.turn.flag(crate::state::turn_flag::SHA_PLAYED) != 0,
        ),
        ModifierValueSource::OwnerShaUsedExactlyOne => i32::from(
            modifier.owner == state.turn.current
                && state.turn.flag(crate::state::turn_flag::SHA_PLAYED) == 1,
        ),
        ModifierValueSource::OwnerShaUsedExactlyTwo => i32::from(
            modifier.owner == state.turn.current
                && state.turn.flag(crate::state::turn_flag::SHA_PLAYED) == 2,
        ),
        ModifierValueSource::OwnerMaxHandBeforeSkillFiresThisTurn => player
            .filter(|player| player.alive)
            .map(|_| {
                let owner_hand = state.hand_count(modifier.owner);
                i32::from(
                    state
                        .skills
                        .get(modifier.skill_id)
                        .is_some_and(|skill| skill.turn_fire_count == 0)
                        && state
                            .players
                            .iter_live()
                            .filter(|other| other.alive && other.id != modifier.owner)
                            .all(|other| state.hand_count(other.id) <= owner_hand),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerNonDamageHandTypeCount => state
            .hand_cards(modifier.owner)
            .into_iter()
            .filter_map(|card| state.cards.get(card))
            .filter(|card| !card.kind.is_sha() && !card.kind.is_damage_trick())
            .filter_map(|card| {
                if card.kind.is_equip() {
                    Some(2u8)
                } else if card.kind.is_trick() {
                    Some(1u8)
                } else if matches!(
                    card.kind,
                    CardKind::Sha
                        | CardKind::ShaFire
                        | CardKind::ShaThunder
                        | CardKind::Tao
                        | CardKind::Shan
                        | CardKind::Jiu
                ) {
                    Some(0u8)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>()
            .len()
            .min(i32::MAX as usize)
            as i32,
        ModifierValueSource::OwnerShaHandCount => state
            .hand_cards(modifier.owner)
            .into_iter()
            .filter(|card| state.cards.get(*card).is_some_and(|card| card.kind.is_sha()))
            .count()
            .min(i32::MAX as usize) as i32,
        ModifierValueSource::OwnerTurnDamageDealt => {
            if modifier.owner == state.turn.current {
                state
                    .turn
                    .flag(crate::state::turn_flag::DAMAGE_DEALT)
                    .max(0)
            } else {
                0
            }
        }
        ModifierValueSource::OwnerDistinctHandCardSuitsUsedThisTurn => state
            .turn
            .distinct_hand_card_suits_used(modifier.owner)
            .min(i32::MAX as u32)
            as i32,
        ModifierValueSource::OwnerCurrentTurnCardUseCount => {
            if modifier.owner == state.turn.current {
                state
                    .turn
                    .flag(crate::state::turn_flag::card_used_by_player(modifier.owner))
                    .max(0)
            } else {
                0
            }
        }
        ModifierValueSource::OwnerPlayPhaseCardUseCountZero => i32::from(
            modifier.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && state
                    .turn
                    .flag(crate::state::turn_flag::play_phase_card_used_by_player(
                        modifier.owner,
                    ))
                    == 0,
        ),
        ModifierValueSource::OwnerSkillPlayPhaseFireCountBelow(limit) => i32::from(
            limit > 0
                && modifier.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && state
                    .skills
                    .get(modifier.skill_id)
                    .is_some_and(|skill| skill.play_phase_fire_count < limit),
        ),
        ModifierValueSource::OwnerCurrentTurnAllOthersDistanceAtMostOne => i32::from(
            modifier.owner == state.turn.current
                && owner_all_others_distance_at_most_one(state, modifier.owner),
        ),
        ModifierValueSource::OwnerHasHeartEquip => player
            .map(|_| {
                i32::from(
                    state
                        .zones
                        .find(ZoneKind::Equip, modifier.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| {
                            zone.cards.iter().any(|card| {
                                state.cards.get(*card).is_some_and(|card| {
                                    super::effective_card_suit(state, modifier.owner, card)
                                        == Suit::Heart
                                })
                            })
                        }),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::AliveOtherQunCount => state
            .players
            .iter_live()
            .filter(|other| other.alive && other.id != modifier.owner)
            .filter(|other| state.player_group(other.id) == Some("qun"))
            .count()
            .min(i32::MAX as usize) as i32,
        ModifierValueSource::OwnerHasWeapon => player
            .map(|_| {
                i32::from(
                    state
                        .zones
                        .find(ZoneKind::Equip, modifier.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| {
                            zone.cards.iter().any(|card| {
                                state
                                    .cards
                                    .get(*card)
                                    .is_some_and(|card| card.kind == CardKind::Weapon)
                            })
                        }),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerNoWeapon => player
            .map(|_| {
                i32::from(
                    !state
                        .zones
                        .find(ZoneKind::Equip, modifier.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| {
                            zone.cards.iter().any(|card| {
                                state
                                    .cards
                                    .get(*card)
                                    .is_some_and(|card| card.kind == CardKind::Weapon)
                            })
                        }),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerHasArmorEquip => player
            .map(|_| {
                i32::from(
                    state
                        .zones
                        .find(ZoneKind::Equip, modifier.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| {
                            zone.cards.iter().any(|card| {
                                state
                                    .cards
                                    .get(*card)
                                    .is_some_and(|card| card.kind == CardKind::Armor)
                            })
                        }),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::RoundNumberCappedFive => state.turn.round.min(5) as i32,
        ModifierValueSource::RoundAtLeastAliveQunCount => {
            let qun = alive_qun_count(state);
            i32::from(state.turn.round as usize >= qun)
        }
        ModifierValueSource::RoundBelowAliveQunCount => {
            let qun = alive_qun_count(state);
            i32::from((state.turn.round as usize) < qun)
        }
        ModifierValueSource::AliveFieldRankThirteenPlayerCount => state
            .players
            .iter_live()
            .filter(|other| other.alive)
            .filter(|other| {
                [ZoneKind::Equip, ZoneKind::Judge].into_iter().any(|kind| {
                    state
                        .zones
                        .find(kind, other.id)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| {
                            zone.cards.iter().any(|card| {
                                state.cards.get(*card).is_some_and(|card| card.rank == 13)
                            })
                        })
                })
            })
            .count()
            .min(i32::MAX as usize)
            as i32,
        ModifierValueSource::FieldMountCardCountCeilHalf => {
            let mounts = state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .filter_map(|player| state.zones.find(ZoneKind::Equip, player.id))
                .filter_map(|zone| state.zones.get(zone))
                .flat_map(|zone| zone.cards.iter())
                .filter(|card| {
                    state.cards.get(**card).is_some_and(|card| {
                        matches!(card.kind, CardKind::HorseOff | CardKind::HorseDef)
                    })
                })
                .count();
            mounts.div_ceil(2).min(i32::MAX as usize) as i32
        }
        ModifierValueSource::OwnerUniqueMaxHand => player
            .filter(|player| player.alive)
            .map(|_| {
                let owner_hand = state.hand_count(modifier.owner);
                i32::from(
                    state
                        .players
                        .iter_live()
                        .filter(|other| other.alive && other.id != modifier.owner)
                        .all(|other| state.hand_count(other.id) < owner_hand),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerArmorEmpty => player
            .map(|_| {
                i32::from(!super::has_equip_kind(
                    state,
                    modifier.owner,
                    CardKind::Armor,
                ))
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerHealthy => player
            .map(|player| i32::from(player.hp >= player.max_hp as i32))
            .unwrap_or(0),
        ModifierValueSource::AliveWoundedCountZeroOrAtLeastFour => {
            let wounded = super::alive_wounded_count(state);
            i32::from(wounded == 0 || wounded >= 4)
        }
        ModifierValueSource::AliveWoundedCountAtLeast(threshold) => {
            i32::from(super::alive_wounded_count(state) >= threshold as usize)
        }
        ModifierValueSource::OwnerHandGreaterThanOther => other
            .filter(|other| other.is_some())
            .filter(|other| {
                player.is_some_and(|player| player.alive)
                    && state.players.get(*other).is_some_and(|player| player.alive)
            })
            .map(|other| i32::from(state.hand_count(modifier.owner) > state.hand_count(other)))
            .unwrap_or(0),
        ModifierValueSource::OwnerHandLessThanOther => other
            .filter(|other| other.is_some())
            .filter(|other| {
                player.is_some_and(|player| player.alive)
                    && state.players.get(*other).is_some_and(|player| player.alive)
            })
            .map(|other| i32::from(state.hand_count(modifier.owner) < state.hand_count(other)))
            .unwrap_or(0),
        ModifierValueSource::OwnerQunOtherShu => other
            .filter(|other| other.is_some())
            .filter(|other| {
                player.is_some_and(|player| player.alive)
                    && state.players.get(*other).is_some_and(|player| player.alive)
            })
            .map(|other| {
                i32::from(
                    state.player_group(modifier.owner) == Some("qun")
                        && state.player_group(other) == Some("shu"),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerTurnedOverDiffOther => other
            .filter(|other| other.is_some())
            .filter(|other| {
                player.is_some_and(|player| player.alive)
                    && state.players.get(*other).is_some_and(|player| player.alive)
            })
            .map(|other| {
                i32::from(
                    player.is_some_and(|player| player.turned_over)
                        != state
                            .players
                            .get(other)
                            .is_some_and(|player| player.turned_over),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerHandEquipGteOther => other
            .filter(|other| other.is_some())
            .filter(|other| {
                player.is_some_and(|player| player.alive)
                    && state.players.get(*other).is_some_and(|player| player.alive)
            })
            .map(|other| {
                i32::from(
                    state.hand_count(modifier.owner) >= state.hand_count(other)
                        && owner_equip_count(state, modifier.owner)
                            >= owner_equip_count(state, other),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerShaDamageDealtZeroThisTurn => i32::from(
            state
                .turn
                .flag(crate::state::turn_flag::sha_damage_dealt_by_player(
                    modifier.owner,
                ))
                == 0,
        ),
        ModifierValueSource::QueriedPlayerHandGtHpSigned => queried
            .and_then(|queried| state.players.get(queried).map(|player| (queried, player)))
            .map(|(queried, player)| {
                if state.hand_count(queried).min(i32::MAX as u32) as i32 > player.hp {
                    1
                } else {
                    -1
                }
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerLinkedDiffOther => other
            .filter(|other| other.is_some())
            .filter(|other| {
                player.is_some_and(|player| player.alive)
                    && state.players.get(*other).is_some_and(|player| player.alive)
            })
            .map(|other| {
                let linked = |player: PlayerId| {
                    state.players.get(player).is_some_and(|player| {
                        player
                            .marks
                            .iter()
                            .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                    })
                };
                i32::from(linked(modifier.owner) != linked(other))
            })
            .unwrap_or(0),
        ModifierValueSource::QueriedPlayerJudgeNonemptyOtherThanOwner => queried
            .filter(|queried| *queried != modifier.owner)
            .filter(|queried| {
                state
                    .players
                    .get(*queried)
                    .is_some_and(|player| player.alive)
            })
            .map(|queried| {
                i32::from(
                    state
                        .zones
                        .find(ZoneKind::Judge, queried)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| !zone.cards.is_empty()),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::QueriedPlayerLinkedOtherThanOwner => queried
            .filter(|queried| *queried != modifier.owner)
            .filter(|queried| {
                state
                    .players
                    .get(*queried)
                    .is_some_and(|player| player.alive)
            })
            .map(|queried| {
                i32::from(state.players.get(queried).is_some_and(|player| {
                    player
                        .marks
                        .iter()
                        .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                }))
            })
            .unwrap_or(0),
        ModifierValueSource::QueriedPlayerLinkedFirstLivingSkillOwner => {
            let first_owner = state
                .skills
                .sorted_live()
                .into_iter()
                .filter(|skill| {
                    skill.def == modifier.skill_def
                        && state.skill_effectively_enabled(skill)
                        && state
                            .players
                            .get(skill.owner)
                            .is_some_and(|owner| owner.alive)
                })
                .min_by_key(|skill| {
                    state
                        .players
                        .get(skill.owner)
                        .map(|owner| owner.seat.raw())
                        .unwrap_or(u8::MAX)
                })
                .map(|skill| skill.owner);
            if first_owner != Some(modifier.owner) {
                0
            } else {
                queried
                    .filter(|queried| {
                        state
                            .players
                            .get(*queried)
                            .is_some_and(|player| player.alive)
                    })
                    .map(|queried| {
                        i32::from(state.players.get(queried).is_some_and(|player| {
                            player.marks.iter().any(|(mark, count)| {
                                *mark == crate::content::MARK_CHAIN && *count > 0
                            })
                        }))
                    })
                    .unwrap_or(0)
            }
        }
        ModifierValueSource::QueriedPlayerMarkPositive(mark) => queried
            .and_then(|queried| state.players.get(queried))
            .filter(|player| player.alive)
            .and_then(|player| {
                player
                    .marks
                    .iter()
                    .find(|(found, _)| *found == mark)
                    .map(|(_, count)| i32::from(*count > 0))
            })
            .unwrap_or(0)
            .max(0),
        ModifierValueSource::QueriedPlayerOtherThanOwner => queried
            .filter(|queried| *queried != modifier.owner)
            .filter(|queried| {
                state
                    .players
                    .get(*queried)
                    .is_some_and(|player| player.alive)
            })
            .map(|_| 1)
            .unwrap_or(0),
        ModifierValueSource::QueriedPlayerWoundedInOwnerAttackRangeOther => queried
            .filter(|queried| *queried != modifier.owner)
            .filter(|queried| {
                state
                    .players
                    .get(*queried)
                    .is_some_and(|player| player.alive && player.hp < player.max_hp as i32)
            })
            .filter(|queried| crate::query::player_in_attack_range(state, modifier.owner, *queried))
            .map(|_| 1)
            .unwrap_or(0),
        ModifierValueSource::QueriedSourceEnemyOwnerAllyTargetOtherThanOwner => queried
            .filter(|target| *target != modifier.owner)
            .zip(other)
            .filter(|(target, source)| {
                state
                    .players
                    .get(*target)
                    .is_some_and(|player| player.alive)
                    && state
                        .players
                        .get(*source)
                        .is_some_and(|player| player.alive)
            })
            .map(|(target, source)| {
                let owner_allied_with_target = matches!(
                    super::relation(state, modifier.owner, target),
                    QueryResponse::Relation(RelationTag::Ally)
                );
                let source_enemy_of_target = matches!(
                    super::relation(state, source, target),
                    QueryResponse::Relation(RelationTag::Enemy)
                );
                i32::from(owner_allied_with_target && source_enemy_of_target)
            })
            .unwrap_or(0),
    };
    modifier.value.saturating_mul(value)
}

/// Positive optional draw bonuses grouped by owning skill in stable skill-id order.
pub fn optional_draw_bonuses_for(state: &GameState, owner: PlayerId) -> Vec<u32> {
    let mut grouped = BTreeMap::<SkillId, i32>::new();
    for modifier in state
        .modifiers
        .for_target(ModifierTarget::OptionalDrawCount)
        .into_iter()
        .filter(|modifier| modifier.owner == owner && modifier.mode == CombineMode::Add)
    {
        let total = grouped.entry(modifier.skill_id).or_default();
        *total = total.saturating_add(resolved_value(state, modifier));
    }
    grouped
        .into_values()
        .filter(|bonus| *bonus > 0)
        .map(|bonus| bonus as u32)
        .collect()
}

/// Count living wounded players, plus other living Wu players while the owner is the Lord and
/// has enabled Guiming.
pub fn alive_wounded_or_guiming_wu_count(state: &GameState, owner: PlayerId) -> i32 {
    const GUIMING: u32 = 3_828_399_214;
    let guiming_enabled = state
        .players
        .get(owner)
        .is_some_and(|player| player.alive && player.identity == Identity::Lord)
        && state.skills.sorted_live().into_iter().any(|skill| {
            skill.owner == owner
                && skill.def.raw() == GUIMING
                && state.skill_effectively_enabled(skill)
        });
    state
        .players
        .iter_live()
        .filter(|other| {
            other.alive
                && (other.hp < other.max_hp as i32
                    || (guiming_enabled
                        && other.id != owner
                        && state.player_group(other.id) == Some("wu")))
        })
        .count()
        .min(i32::MAX as usize) as i32
}

fn owner_all_others_distance_at_most_one(state: &GameState, owner: PlayerId) -> bool {
    state
        .players
        .iter_live()
        .filter(|player| player.alive && player.id != owner)
        .all(
            |player| match super::distance_explain(state, owner, player.id).response {
                rule_api::QueryResponse::Int(value) => value <= 1,
                _ => false,
            },
        )
}

fn owner_equip_count(state: &GameState, owner: PlayerId) -> usize {
    state
        .zones
        .find(ZoneKind::Equip, owner)
        .and_then(|zone| state.zones.get(zone))
        .map(|zone| zone.cards.len())
        .unwrap_or(0)
}

fn alive_qun_count(state: &GameState) -> usize {
    state
        .players
        .iter_live()
        .filter(|player| player.alive)
        .filter(|player| state.player_group(player.id) == Some("qun"))
        .count()
}

fn src(m: &ActiveModifier, value: i32) -> ModifierSource {
    ModifierSource {
        source_id: m.skill_id.raw(),
        label_key: m.label_key.clone(),
        value,
    }
}

/// Debug helper: group sources by label.
pub fn sources_by_label(sources: &[ModifierSource]) -> BTreeMap<String, i32> {
    let mut map = BTreeMap::new();
    for s in sources {
        *map.entry(s.label_key.clone()).or_insert(0) += s.value;
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::bootstrap_match;
    use crate::state::GameState;
    use crate::transaction::Transaction;
    use rule_api::TargetCardScope;

    #[test]
    fn gain_protected_card_is_not_gainable_by_another_player() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 20_260_730, "bootstrap").unwrap();
        let players = tx.state().players.alive_by_seat();
        let actor = players[0];
        let target = players[1];
        let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
        let equip = tx.state().zones.find(ZoneKind::Equip, target).unwrap();
        let protected = tx.state().hand_cards(target)[0];
        tx.state_mut()
            .zones
            .get_mut(hand)
            .unwrap()
            .cards
            .retain(|card| *card != protected);
        tx.state_mut()
            .zones
            .get_mut(equip)
            .unwrap()
            .cards
            .push(protected);
        let card = tx.state_mut().cards.get_mut(protected).unwrap();
        card.kind = CardKind::Weapon;
        card.zone = equip;
        card.face_up = true;

        tx.state_mut().modifiers.entries.push(ActiveModifier {
            skill_id: SkillId::new(90_001),
            skill_def: SkillDefId::new(90_002),
            owner: target,
            def_id: 1,
            target: ModifierTarget::CardGainProtectedFor(CardSelector::Equip),
            mode: CombineMode::Override,
            value_source: None,
            value: 1,
            order: 0,
            enabled: true,
            label_key: "test.card_gain_protected".into(),
        });

        let gainable = super::super::gainable_target_cards(
            tx.state(),
            actor,
            target,
            TargetCardScope::HandEquipJudge,
        );
        assert!(!gainable.contains(&protected));
        assert!(!gainable.is_empty());
        assert!(super::super::card_gain_protected_from(
            tx.state(),
            actor,
            target,
            protected
        ));

        assert!(super::super::gainable_target_cards(
            tx.state(),
            target,
            target,
            TargetCardScope::HandEquipJudge,
        )
        .contains(&protected));
    }
}
