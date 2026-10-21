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

/// Active modifier index (derived; not hashed).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveModifierIndex {
    entries: Vec<ActiveModifier>,
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

mod bootstrap_modifier_defs;
pub use bootstrap_modifier_defs::*;

mod matches_kind_and_suit;
