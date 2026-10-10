//! Static trigger definitions (content-compiler will populate later).

use super::{EventKind, TimingBand};
use content_schema::TriggerChoiceActor;
use engine_api::SkillDefId;
use serde::{Deserialize, Serialize};

/// Trigger definition id (stable across content packages once assigned).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct TriggerDefId(pub u32);

impl TriggerDefId {
    /// Raw value.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Who must match for the trigger to fire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerFilter {
    /// Owner is the damage target / dying player / killed player.
    OwnerIsEventPlayer,
    /// Owner is the event player and the event source is still alive.
    OwnerIsEventPlayerSourceAlive,
    /// Owner is the damage source / card user.
    OwnerIsEventSource,
    /// Owner is the wounded damage source and is within distance 1 of the target.
    OwnerSourceWoundedDistanceLeOne,
    /// Always (global), still ordered by owner's seat.
    Always,
    /// Owner is event player and their hand is empty (after the event).
    OwnerHandEmpty,
    /// Cards left owner's hand zone (`CardsMoved` from Hand).
    OwnerFromHand,
    /// Cards left owner's equip zone (`CardsMoved` from Equip).
    OwnerFromEquip,
    /// Cards left owner's equip zone; repeat once per moved card (枭姬).
    OwnerFromEquipPerCard,
    /// Left hand and hand is now empty (连营).
    OwnerFromHandEmpty,
    /// Owner is card user and the used card is a 锦囊 (集智).
    OwnerSourceTrick,
    /// Fire at most once per turn (uses `SkillInstance.turn_fire_count`).
    OncePerTurn,
    /// Living owner, current Play-phase player used at least two valid cards all of one suit,
    /// and this skill has not fired this turn.
    OwnerAliveEventPlayerPlayPhaseSameSuitAtLeastTwoOncePerTurn,
    /// Owner is the End-phase player, all four physical suits entered the discard pile this
    /// turn, and the skill's current-round extra-turn count is below the living-player cap.
    OwnerIsEventPlayerDiscardPileFourSuitsBelowAliveCap,
    /// Owner is event source and once per turn.
    OwnerSourceOncePerTurn,
    /// Owner is event source and currently at full HP.
    OwnerSourceHealthy,
    /// Owner is the source of damage whose amount is greater than one.
    OwnerSourceAmountGtOne,
    /// Owner is the source of fire damage.
    OwnerSourceFire,
    /// Owner is the source of thunder damage.
    OwnerSourceThunder,
    /// Owner is event source, target is another player, at most twice per turn.
    OwnerSourceOtherTwicePerTurn,
    /// Skill owner is alive after the event (global living-owner trigger).
    OwnerAlive,
    /// Skill owner is alive and the event source/target share a sex tag.
    OwnerAliveSourceTargetSameSex,
    /// Skill owner is alive and shares a group tag with the event player.
    OwnerAliveEventPlayerSameGroup,
    /// Skill owner and event player are alive, and the event player has this group tag.
    OwnerAliveEventPlayerGroup(String),
    /// Skill owner is the event player, no *other* living member of `clan` lost hand cards this
    /// turn. Implements clan 中流 (clanzhongliu): `owner_is_event_player_no_<CLAN>_lost_hand_this_turn`.
    /// Per-turn approximation of upstream's per-useEvent correlation.
    OwnerIsEventPlayerNoClanLostHandThisTurn(String),
    /// Skill owner and another judged player are alive, the player has this group tag, and the
    /// completed judgment result is black.
    OwnerAliveOtherJudgmentBlackEventPlayerGroup(String),
    /// Skill owner and event player are alive, and the event player's equipment zone is empty.
    OwnerAliveEventPlayerEquipEmpty,
    /// 神君: the owner is alive, the event is a card use, and the owner's hand
    /// holds a card sharing the used card's effective kind (同名牌).
    OwnerAliveHandHasSameKindAsEventCard,
    /// 神君: the owner is alive and holds a hand card tagged with the active
    /// skill's definition id (revealed by 神君 earlier this phase).
    OwnerAliveHasSkillTaggedCards,
    /// 影响附近 (clanxsyingxiang): the owner is alive and a card carrying the active skill's
    /// definition-id tag was consumed (left its holder) this turn. Per-turn approximation of
    /// upstream's per-useEvent `hasHistory("lose", ... gaintag_map ...)` correlation.
    OwnerAliveTaggedCardLostThisTurn,
    /// Skill owner is wounded and was not the event source.
    OwnerWoundedNotEventSource,
    /// Skill owner is the event player and is wounded.
    OwnerIsEventPlayerWounded,
    /// Skill owner is wounded and the event player has this group tag.
    OwnerWoundedEventPlayerGroup(String),
    /// Skill owner is alive and is not the event player.
    OwnerAliveNotEventPlayer,
    /// Skill owner is alive, is not the event player, and shares a group with the event source.
    OwnerAliveNotEventPlayerSourceSameGroup,
    /// Owner and the other living event player have the same hand-card count.
    OwnerAliveNotEventPlayerEqualHand,
    /// Living non-event-player owner, at most once per turn.
    OwnerAliveNotEventPlayerOncePerTurn,
    /// Skill owner is alive and the logical event card is any 杀.
    OwnerAliveCardIsSha,
    /// A living owner observed another player respond with a gainable physical 杀.
    OwnerAliveOtherRespondedShaGainable,
    /// Skill owner is alive and the used card declared more than one target.
    OwnerAliveCardTargetsGtOne,
    /// Skill owner is tied for the lowest HP among living players.
    OwnerMinHp,
    /// Owner is the event player and tied for the fewest hand cards.
    OwnerIsEventPlayerMinHand,
    /// Owner is the event player but is not tied for the fewest hand cards.
    OwnerIsEventPlayerNotMinHand,
    /// Owner is ending their play phase after using at least two proactive 杀.
    OwnerShaUsedAtLeastTwo,
    /// Owner has max HP above one and either their hand or equip zone is empty.
    OwnerMaxHpGtOneHandOrEquipEmpty,
    /// A card left the owner's equip/judge zone and both zones are now empty.
    OwnerFromFieldEmpty,
    /// Owner was targeted by a trick while their equipment zone was empty.
    OwnerTargetedByTrickEquipEmpty,
    /// Owner was targeted by Sha while their hand count was below their HP.
    OwnerTargetedByShaHandBelowHp,
    /// Damaged owner was targeted by Sha/Juedou by a player with at least as much HP; once per turn.
    OwnerTargetedByShaOrJuedouDamagedHpLeSourceOnce,
    /// Owner was targeted outside their turn by Sha/trick while their hand was empty.
    OwnerTargetedOutsideTurnByShaOrTrickHandEmpty,
    /// Owner was targeted while their hand count was below their maximum HP.
    OwnerTargetedHandBelowMaxHp,
    /// Owner used or was targeted by Juedou or a red Sha.
    OwnerSourceOrTargetedByJuedouOrRedSha,
    /// In a per-target context, owner is source or target and both have equal HP for Sha.
    OwnerSourceOrTargetedByShaEqualHp,
    /// Owner used/responded with a diamond card, or another player targeted them with one.
    OwnerSourceOrTargetedByOtherDiamond,
    /// Owner used a red card.
    OwnerSourceRed,
    /// Owner is the event player and has fewer hand cards than maximum HP.
    OwnerIsEventPlayerHandBelowMaxHp,
    /// Owner is the event player and has dealt no damage during the current turn.
    OwnerIsEventPlayerNoTurnDamage,
    /// Owner is the event player and has used at least as many cards as their current HP this turn.
    OwnerIsEventPlayerCardUsedAtLeastHp,
    /// Owner is ending their turn after damaging another Qun character.
    OwnerIsEventPlayerDamagedOtherQun,
    /// Owner is either the damage source or the damaged player.
    OwnerIsEventSourceOrPlayer,
    /// Owner is the player receiving fire damage.
    OwnerTargetFire,
    /// Living owner may fire up to current HP times per turn, or without limit during own turn.
    OwnerAliveCurrentOrTurnFiresBelowHp,
    /// Owner was the sole target of another player's trick card.
    OwnerTargetedByOtherSingleTargetTrick,
    /// Owner used a card while basic and non-basic hand-card counts are equal.
    OwnerSourceHandBasicEqualsNonBasic,
    /// Owner used a logical Sha.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceSha,
    /// Owner used a Sha and is the first target's namer — the current card_targeted
    /// event player is the Sha's first target (ABI 410 霆威).
    OwnerSourceShaFirstTarget,
    /// Owner used a Sha or Juedou against exactly one target (ABI 420 曲迪).
    OwnerSourceShaOrJuedouSingleTarget,
    /// Owner alive; a play-phase player with the mark targeted another marked player
    /// (ABI 425 谏喻). Parameter = the 喻 mark def.
    OwnerAliveCardTargetedMarkedPeer(u32),
    /// Owner alive; another player used a single-target Juedou or red Sha
    /// (ABI 427 合击).
    OwnerAliveOtherSingleTargetJuedouOrRedSha,
    /// Owner has at least one hand card gained this turn and can compare with another
    /// player (ABI 428 压军).
    OwnerAliveGainedThisTurnCanCompareOther,
    /// Owner is the living event player and has odd current HP.
    OwnerIsEventPlayerHpOdd,
    /// Owner is the living event player, wounded, and has even current HP.
    OwnerIsEventPlayerHpEvenWounded,
    /// Skill owner and event player are both alive.
    OwnerAliveEventPlayerAlive,
    /// Living owner and another living event player are both not unique maximum-hand players.
    OwnerAliveNotEventPlayerNeitherUniqueMaxHand,
    /// Owner is the event player and a different event source is still alive.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerOtherSourceAlive,
    /// Owner used a card whose color differs from their previous card this turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceAlternatesColor,
    /// During play, owner used a card matching the previous play-phase card's suit or rank.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceMatchesPreviousSuitOrRank,
    /// Owner used an ordinary (non-delayed) trick card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceOrdinaryTrick,
    /// A living owner observes a player's first used card this turn, and it is an ordinary trick.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveSourceFirstCardOrdinaryTrick,
    /// Owner used a card matching their previous card in the whole game by suit or rank.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceMatchesLastUsedSuitOrRank,
    /// Living event player is within the living owner's current HP distance from the owner.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerDistanceLeOwnerHp,
    /// A living owner observes an equipment user tied for the most equipped cards.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEquipSourceMaxEquip,
    /// A living owner observes another current-turn player repeat the previous card name.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherCurrentSourceRepeatsCardName,
    /// Skill owner and event player are alive, and the event player's hand is empty.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerHandEmpty,
    /// Living owner is not the event player and has fewer hand cards than current HP.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveNotEventPlayerHandBelowHp,
    /// Owner has at least one stack of the specified stable mark definition.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerMarkPositive(u32),
    /// Living owner has dealt damage to the event player earlier in this match.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveDamagedEventPlayer,
    /// Owner and event player are alive, and the event player is the owner or has female sex.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerOwnerOrFemale,
    /// Living owner is not the event player, whose raw HP is below the owner's HP clamped at zero.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveNotEventPlayerNonnegativeHpGtEventPlayer,
    /// Living owner is the current turn player and is not the event player.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveCurrentTurnOtherEventPlayer,
    /// Living owner gains multiple cards in one event outside their own draw phase.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerGainsMultipleOutsideOwnDraw,
    /// Living owner gains or loses at least two cards in one event.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerGainedOrLostAtLeastTwo,
    /// A player whose attack range includes the owner responded to the owner's card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerCardRespondedToByPlayerInRange,
    /// Living owner observes the first non-draw card move into any hand this turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveFirstCardsMovedToHand,
    /// Living owner observes an equipment card use while this skill's mark is below three.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEquipUsedSkillMarkBelowThree,
    /// Living owner observes the first sourced damage event they handle this round.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveSourceAliveOncePerRound,
    /// Owner used a logical 酒.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceJiu,
    /// Living owner gains cards outside their turn while the specified mark is below living count.
    /// Fires at most once per turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerGainsOutsideTurnMarkBelowAliveOncePerTurn(u32),
    /// Living owner observes another player's equipment loss during the owner's turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerCurrentTurnOtherEquipmentLost,
    /// Living owner observes a living non-current player lose their last hand card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerOutsideTurnFromHandEmpty,
    /// Living current-turn owner observes another living player lose their last hand card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveCurrentTurnOtherEventPlayerFromHandEmpty,
    /// Living owner is the sole target of another player's card and can self-discard from hand/equip.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveTargetedByOtherSingleCardDiscardable,
    /// Living owner is targeted by a trick; multi-target uses require a self-discardable card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveTargetedByTrickSingleOrDiscardable,
    /// Living owner is targeted by another player's card whose raw suit matches an equipped card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveTargetedByOtherMatchingEquipSuit,
    /// Living owner; fires at most once in the current round.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOncePerRound,
    /// Owner draws during their own play phase, excluding draws from this skill instance.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerDrawsDuringOwnPlayNotSelf,
    /// Owner used a non-equipment card with targets while holding no basic hand cards.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceNonEquipNoBasicHandHasTargets,
    /// An equipment card entered or left the living owner's equipment zone while at least two
    /// living players were wounded.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerEquipZoneChangedAtLeastTwoWounded,
    /// Owner is one endpoint of sourced damage between two distinct living players.
    /// The trigger planner emits one activation per damage point.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceOrTargetOtherAlivePerDamagePoint,
    /// Owner is the event player; fires at most once per turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerOncePerTurn,
    /// Owner caused damage to a living player who has at least one equipment card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceTargetAliveHasEquip,
    /// Owner is targeted by another player's Sha; fires at most once per turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerTargetedByOtherShaOncePerTurn,
    /// Owner used or responded with the card that makes their turn action count equal attack range.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerCardActionCountEqualsAttackRange,
    /// Owner used or was targeted by Juedou.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceOrTargetedByJuedou,
    /// Owner used a physical ordinary trick rather than a view-as conversion.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourcePhysicalOrdinaryTrick,
    /// Owner's hand is below lost HP after an owner HP/max-HP change or a move
    /// touching one of the owner's personal card zones.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerHandBelowLostHpAfterRelevantChange,
    /// Owner is the player receiving thunder damage.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerTargetThunder,
    /// Owner is the living event player and has more hand cards than current HP.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerHandAboveHp,
    /// Owner is the living target of another player's Sha.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerTargetedByOtherSha,
    /// Owner is about to receive damage from a red logical Sha whose source is alive.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerRedShaSourceAlive,
    /// Owner is about to receive damage from a non-red logical Sha.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerNonRedSha,
    /// Owner received damage from a red logical Sha or one boosted by Jiu.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerRedOrJiuSha,
    /// Living owner has fewer hand cards than min(7, living player count).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveHandBelowAliveCountCappedSeven,
    /// Owner is the event source and has fewer hand cards than min(7, living player count).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceHandBelowAliveCountCappedSeven,
    /// Owner is about to receive damage from a red card whose source is alive.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerRedCardSourceAlive,
    /// Owner is about to receive damage from a non-red card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerNonRedCard,
    /// Owner is the event player and this is the skill's first damage trigger this turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerFirstDamagePerTurn,
    /// Owner is the event player and this skill already handled damage this turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerAfterFirstDamagePerTurn,
    /// Owner dealt damage within distance one; emit one activation per damage point.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceDistanceLeOnePerDamagePoint,
    /// Owner is the event player and this is the skill's second damage trigger this turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerSecondDamagePerTurn,
    /// Owner is the damaged player, the living source is different, and the trigger repeats once per damage point.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerOtherSourceAlivePerDamagePoint,
    /// Owner caused damage to a living player who has at least one hand or equipment card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceTargetAliveHasHandOrEquip,
    /// Living owner observes the first declared target of a virtual card or a card whose logical
    /// kind differs from its physical kind.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveFirstTargetVirtualOrConvertedCard,
    /// Living owner observes another living player deal Sha damage; fires at most twice per turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherSourceShaTwicePerTurn,
    /// Owner is the event player and is not the unique current-HP leader.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerNotUniqueMaxCurrentHp,
    /// Owner is the event player and currently has at least one hand card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerHandNonempty,
    /// Owner is the event player and has a positive stable mark count.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerMarkPositive(u32),
    /// Owner is the event player and dealt damage during the current turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerTurnDamage,
    /// Owner is the event player and dealt at least three damage during the current turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerTurnDamageAtLeastThree,
    /// Owner used a card on the current per-target event player, whose final distance from the
    /// owner is greater than one.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceTargetDistanceGtOne,
    /// Living owner observes another player's basic card discarded outside the owner's turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherDiscardedBasicOutsideTurn,
    /// Living owner observes another player's basic card discarded outside the owner's turn;
    /// fires at most once during each current character's turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherDiscardedBasicOutsideTurnOncePerTurn,
    /// Living owner observes their first hand-card discard event of the turn, and at least one
    /// physical Sha from that event still remains in the shared discard pile.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerFirstHandDiscardHasShaInDiscard,
    /// Living owner dealt or received damage during the turn that just ended.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveDealtOrReceivedDamageThisTurn,
    /// During the owner's play phase, the owner dealt logical Sha damage to another living
    /// player; fires at most once in that play phase.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourcePlayShaOtherAliveOncePerPlayPhase,
    /// Living owner received cards voluntarily given by another player.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerReceivedGiftFromOther,
    /// Owner initiated a resolved Duel, its original target is alive, and the loser can give
    /// one hand/equipment card to the winner.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerInitiatedDuelTargetAliveLoserHasHandOrEquip,
    /// Owner is the living event player and their current HP is below the threshold.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerHpBelow(u32),
    /// Owner is alive and has at least one enabled equipment slot.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerHasEnabledEquipSlot,
    /// Owner is the equipment-slot event player, has no enabled slot left, and this skill
    /// instance has not fired previously in the match.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerNoEnabledEquipSlotOncePerGame,
    /// Owner is ending their turn after at least current-HP physical cards entered the discard
    /// pile because any player used or responded with them during this turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerUseResponseDiscardedAtLeastHp,
    /// Owner took sourced damage, the living source has a card the owner may take, and the
    /// trigger repeats once per damage point. Self-sourced damage exposes equipment only;
    /// damage from another player exposes hand and equipment cards.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerSourceAliveGainableCardPerDamagePoint,
    /// Living owner observes a living non-current player lose their last hand card; fires at
    /// most once during each current character's turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerOutsideTurnFromHandEmptyOncePerTurn,
    /// Living owner observes any turn-over transition or a transition into the linked state.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerTurnChangedOrBecameLinked,
    /// Living owner observes a transition to face-up or into the linked state.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerBecameFaceUpOrLinked,
    /// Living owner observes a living player lose one or more equipment cards and leave their
    /// equipment zone empty.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerFromEquipEmpty,
    /// Living owner observes a living Sha target that no distinct red card targeted earlier in
    /// the current turn. The current physical card is excluded from the history comparison.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveShaTargetNoOtherRedCardThisTurn,
    /// Owner is the preparation-phase event player and at least one equipment card can be moved
    /// to a different living player other than the owner.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerHasMovableEquipment,
    /// Owner is alive and used at least one logical Sha during the current character's turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveUsedShaThisTurn,
    /// Owner caused damage with a physical black logical Sha to a living player who has at
    /// least one equipment card the owner may discard.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceBlackShaTargetAliveHasDiscardableEquip,
    /// Owner caused logical-Sha damage to a living player who has at least one discardable card
    /// in their hand, equipment, or judgment zones.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceShaTargetAliveHasDiscardableRegionCard,
    /// Living owner is the user of a logical Sha and the current living target's hand count is
    /// at least the owner's HP or at most the owner's current attack range.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceShaTargetHandGteHpOrLteAttackRange,
    /// Living owner used the fully dodged Sha and the living target has at least one
    /// discardable hand or equipment card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceShaDodgedTargetAliveHasDiscardableHandEquip,
    /// Living owner is the source of logical-Sha damage to a living target at final distance one.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceShaTargetAliveDistanceLeOne,
    /// A living owner observes a living logical-Sha target at final distance at most one.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveShaTargetDistanceLeOne,
    /// Owner received logical-Sha damage from an identified source and can discard one card
    /// from hand or equipment.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerShaSourceHasDiscardableHandEquip,
    /// Owner is ending their play phase after invoking this active skill at least the threshold
    /// number of times in that same play phase.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerActiveUsedAtLeastThisPlayPhase(u32),
    /// Owner is ending their play phase after using at least one logical basic, trick, or
    /// equipment card during that play phase.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerPlayPhaseCardCategoryCountPositive,
    /// A living player is using a logical Sha during that player's play phase, and the living
    /// owner can currently discard a hand/equipment card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAlivePlayPhaseShaUserHasDiscardableHandEquip,
    /// Owner used an ordinary-nature logical Sha.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceNormalSha,
    /// Owner used a fire-nature logical Sha.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceFireSha,
    /// Owner is entering the end phase with at least one logical hand Sha and this skill's
    /// public pile is empty.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerHandShaSkillPileEmpty,
    /// Living owner observes a player at non-positive HP while this skill's public pile is
    /// nonempty.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveDyingHasSkillPileCards,
    /// During their own turn, the owner used a Sha or ordinary trick with exactly one target.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceSingleTargetShaOrOrdinaryTrickCurrentTurn,
    /// The owner caused damage with a card whose draw-on-damage policy is recorded in this exact
    /// skill instance.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceEventCardPolicyDraw,
    /// Owner discarded at least one effective black basic card from hand/equipment into the
    /// discard pile in the triggering movement batch.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerDiscardedBlackBasicToDiscard,
    /// Another living player's judgment phase began while that player is in the living owner's
    /// attack range and has a judgment card the owner may discard.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherJudgePhaseInRangeHasJudgeCard,
    /// Owner is the current phase event player and at least one other living player has hand cards.
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerOtherHandNonempty,
    /// Owner used a card during their Play phase whose logical category matches this skill's
    /// remembered category.
    /// Appended for serialized ordinal stability.
    OwnerSourcePlayCardMatchesRememberedCategory,
    /// The event source matches this skill's delayed Play-phase gift target.
    /// Appended for serialized ordinal stability.
    OwnerPhaseGiftTargetIsEventSource,
    /// The event player matches this skill's delayed Play-phase gift target.
    /// Appended for serialized ordinal stability.
    OwnerPhaseGiftTargetIsEventPlayer,
    /// The living owner used the fully resolved Sha, at least one physical Sha/Shan group is
    /// still gainable from discard, and a living non-owner/non-target recipient exists.
    /// Appended for serialized ordinal stability.
    OwnerSourceShaResolvedHasGainableCardsAndRecipient,
    /// Owner is the phase event player and ties for the most hand cards among living players.
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerTiedMaxHand,
    /// Owner is the phase event player and ties for the highest current HP among living players.
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerTiedMaxHp,
    /// Owner is the phase event player, has equipment, and ties for the most equipment among
    /// living players. Appended for serialized ordinal stability.
    OwnerIsEventPlayerTiedMaxEquipNonempty,
    /// Owner is the phase event player and at least one phase was skipped this turn.
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerSkippedPhaseCountPositive,
    /// The living owner discarded at least one hand/equipment card, still has hand cards, and
    /// has not accepted this trigger in the current phase.
    /// Appended for serialized ordinal stability.
    OwnerDiscardedHasHandOncePerPhase,
    /// During the owner's Play phase, they used a single-target Sha or black ordinary trick and
    /// this skill has not completed in the current phase.
    /// Appended for serialized ordinal stability.
    OwnerSourceSingleTargetShaOrBlackOrdinaryTrickCurrentPlayPhaseUnused,
    /// The owner is about to take damage from a male/double-sex source and can discard an
    /// effective equipment card from hand/equipment.
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerMaleSourceHasDiscardableEquipment,
    /// A living owner observes another living player's turn end after that player used no card
    /// targeting anyone other than themselves during the turn.
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherTurnEndedNoCardTargetedOther,
    /// The living owner is the source of bound pre-resolution Sha damage, the living event
    /// player is its target, and the owner has a physical weapon equipped.
    /// Appended for serialized ordinal stability.
    OwnerSourceShaHasWeapon,
    /// The living owner is the source of bound direct logical-Sha damage before resolution.
    /// Chain-propagated damage is excluded.
    /// Appended for serialized ordinal stability.
    OwnerSourceShaDirectDamageBefore,
    /// After damage involving the living owner as source or target, at least one living player
    /// has a different hand count, and this skill has not fired in the current turn.
    /// Appended for serialized ordinal stability.
    OwnerSourceOrTargetHandCountDiffersOncePerTurn,
    /// The living owner is the Prepare-phase event player and at least one living player has a
    /// nonempty hand. Appended for serialized ordinal stability.
    OwnerIsEventPlayerAnyAliveHandNonempty,
    /// The living owner is either participant in the current hand-comparison event.
    /// Appended for serialized ordinal stability.
    OwnerHandCompareParticipant,
    /// The living owner is either participant in a completed hand comparison and at least one
    /// compared physical card is a logical Sha. Appended for serialized ordinal stability.
    OwnerHandCompareParticipantHasSha,
    /// The living owner participated in a completed hand comparison and the exact result card
    /// selected by legacy `zongshi` semantics is still in the discard pile.
    /// Appended for serialized ordinal stability.
    OwnerHandCompareParticipantResultCardInDiscard,
    /// The owner used a basic or trick card. Appended for serialized ordinal stability.
    OwnerSourceBasicOrTrick,
    /// The owner enters their Play phase and can compare with another living player.
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerCanCompareOther,
    /// The owner enters a phase with hand count at least current HP and another living player.
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerHandAtLeastHpOtherAlive,
    /// Cards left the owner's hand, it is now empty, and another living player has a discardable
    /// hand/equipment card. Appended for serialized ordinal stability.
    OwnerFromHandEmptyOtherDiscardableHandEquip,
    /// Owner proactively used the numbered card in the current play phase.
    /// Appended for serialized ordinal stability.
    OwnerSourcePlayCardCountEquals(u32),
    /// Owner proactively used the card whose ordinal equals their current HP in this Play phase.
    /// Appended for serialized ordinal stability.
    OwnerSourcePlayCardCountEqualsHp,
    /// Owner is the damaged event player, source is alive, and the numbered mark is positive.
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerSourceAliveMarkPositive(u32),
    /// Owner enters Prepare, some living field has a card, and no living player has this pile.
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerAnyFieldCardNoSkillPile(u32),
    /// Living owner observes a Sha user with hand/equipment and that user's numbered pile.
    /// Appended for serialized ordinal stability.
    OwnerAliveSourceShaHandEquipSkillPile(u32),
    /// Living owner observes a dying player with cards in the numbered pile.
    /// Appended for serialized ordinal stability.
    OwnerAliveDyingSkillPile(u32),
    /// Living owner may replace another user's single-target Sha target with themselves.
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherSourceSingleShaTargetOther,
    /// Owner took damage from the exact Sha currently remembered by this skill.
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerTwxiaolianRedirectedDamage,
    /// Owner's Prepare phase, another living player has more hand cards, and this limited skill is unused.
    OwnerIsEventPlayerOtherHigherHandOncePerGame,
    /// A living owner observes another living, unlinked player about to receive normal damage.
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherNormalDamageTargetUnlinked,
    /// The living owner used a card or played a card as a response. This filter also enables
    /// match-lifetime card-action tracking for the loaded content set.
    /// Appended for serialized ordinal stability.
    OwnerLifetimeCardAction,
    /// Living owner observes an event carrying at least one captured physical card.
    /// Appended for serialized ordinal stability.
    OwnerAliveEventCardsNonempty,
    /// Living owner is targeted by another player's card.
    /// Appended for serialized ordinal stability.
    OwnerAliveTargetedByOther,
    /// Owner is the damaged player; emit one activation per damage point, regardless of source.
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerPerDamagePoint,
    /// Owner is the event player and at least one other player remains alive.
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerOtherAlive,
    /// Living owner observes damage from another living player in the requested character group.
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherDamageSourceGroup(String),
    /// Dying owner is being saved with Tao by another living responder in the requested group.
    /// Appended for serialized ordinal stability.
    OwnerDyingOtherTaoResponderGroup(String),
    /// Living owner is the event player and is not tied for the lowest current HP.
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerNotMinHp,
    /// Living owner used a logical Sha and the current living target has at least one
    /// hand/equipment card the owner may discard.
    /// Appended for serialized ordinal stability.
    OwnerSourceShaTargetAliveHasDiscardableHandEquip,
    /// Living owner was targeted during the ending phase, their HP changed during that phase,
    /// and they can legally use a source-free Wanjian.
    /// Appended for serialized ordinal stability.
    OwnerAlivePhaseTargetedHpChangedCanUseWanjian,
    /// Living owner was targeted during the ending phase, their HP did not change during that
    /// phase, and they can legally use a source-free Juedou.
    /// Appended for serialized ordinal stability.
    OwnerAlivePhaseTargetedHpUnchangedCanUseJuedou,
    /// The owner used a logical basic card.
    /// Appended for serialized ordinal stability.
    OwnerSourceBasic,
    /// The owner used a logical non-basic card.
    /// Appended for serialized ordinal stability.
    OwnerSourceNonBasic,
    /// The wounded owner used Nanman, Wanjian, or an ordinary-nature spade Sha.
    /// Appended for serialized ordinal stability.
    OwnerSourceWoundedNanmanWanjianOrNormalSpadeSha,
    /// A living owner observes another player's Play phase end after that player used at least
    /// two logical cards of one basic/trick/equipment category; the owner can discard a
    /// hand/equipment card. Appended for serialized ordinal stability.
    OwnerAliveOtherPlayPhaseDuplicateCardCategoryHasHandEquip,
    /// A living owner observes another player's End phase begin after that player dealt more
    /// than one total damage this turn; the owner can discard a hand/equipment card.
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherPhaseEndDamageGtOneHasHandEquip,
    /// The living owner is the event player and current Play-phase player.
    /// Appended for serialized ordinal stability.
    OwnerAliveIsEventPlayerCurrentPlayPhase,
    /// A living owner is targeted by another player's logical trick card.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveTargetedByOtherTrick,
    /// A living owner observes another player's raw black physical cards enter the discard pile
    /// through a discard movement. The VM receives only the matching black cards as event cards.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherDiscardedBlackToDiscard,
    /// A living, wounded owner used a card that declared more than one target.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveWoundedCardTargetsGtOne,
    /// The owner used a logical Sha or trick during their current Play phase.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaOrTrickCurrentPlayPhase,
    /// The owner is the damaged player and the damage source still has at least one card in
    /// their weapon equipment slot.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerSourceHasWeapon,
    /// The living owner dealt damage to another living player and may currently discard at
    /// least one card from that target's hand or equipment zone.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceOtherTargetAliveHasDiscardableHandEquip,
    /// The living owner used a logical Sha on a living target whose attack range does not
    /// currently include the owner.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaTargetAliveOwnerOutsideTargetAttackRange,
    /// The living owner observes another living player enter dying while the owner has at least
    /// one hand card, and this skill instance has not fired previously in the match.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherDyingHandNonemptyOncePerGame,
    /// A living owner observes Sha or Duel damage whose living source and target collectively
    /// have at least one hand/equipment card.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveShaOrDuelDamageSourceTargetAliveHandEquipNonempty,
    /// The living owner is the event player and at least one living player can discard one of
    /// their own hand/equipment cards.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerAnyAliveSelfDiscardableHandEquip,
    /// A living owner observes a living sourced event whose event player is still dying, at
    /// most once in the current round.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveSourceAliveEventPlayerDyingOncePerRound,
    /// During the owner's current turn, the owner used a red/black card matching the immediately
    /// previous used card's color and can discard a hand/equipment card.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceRepeatsColorHasDiscardableHandEquip,
    /// The living owner observes an event for the player stored in this skill instance's shared
    /// target-bound state.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveSkillBoundTargetIsEventPlayer,
    /// The living owner is the damaged event player, the damage source belongs to Qun, and the
    /// owner can currently discard at least two hand/equipment cards.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerSourceGroupQunDiscardableHandEquipGteTwo,
    /// The living owner is the event player of a positive damage event, owns at least one
    /// hand/equipment card, and another living player exists.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerHandEquipNonemptyOtherAlive,
    /// The living owner gained at least three physical cards during this turn and another living
    /// player exists.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveGainedThreeThisTurnOtherAlive,
    /// Living owner observes the start of a living player's Play phase while that player has at
    /// least one hand card; fires at most once in the current round.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveEventPlayerHandNonemptyOncePerRound,
    /// The current Play-phase player is using a card whose effective category matches this skill
    /// instance's bound category; fires at most three times in that Play phase.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceBoundCategoryFirstThreePlayPhase,
    /// Living owner used a logical Sha or damage trick; fires at most once per turn.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceDamageCardOncePerTurn,
    /// The living owner is not the event player and owns at least one hand/equipment card.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveNotEventPlayerHandEquipNonempty,
    /// The living owner used a logical Sha or ordinary trick on a wounded living target and
    /// can discard one hand/equipment card.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaOrOrdinaryTrickTargetWoundedHasDiscardableHandEquip,
    /// A living owner observes another living player's turn end after that player targeted
    /// another player with a card but dealt no damage to another player during the turn.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherTurnEndedCardTargetedOtherNoDamageOther,
    /// A living owner observes another player's fully resolved Sha, was not one of its targets,
    /// and may gain its physical carrier from discard once per turn.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherSourceShaResolvedNotTargetGainableOncePerTurn,
    /// Living owner used the fully dodged exact Sha, armed this skill for the current target,
    /// and the target can discard one owner hand/equipment card.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaDodgedTargetRecordedBySkillHasDiscardableHandEquip,
    /// Owner's currently saved Sha was dodged.
    OwnerSourceShaDodged,
    /// Owner's source-free Sha recorded by this skill instance dealt damage to the
    /// current `event_player`; used by 峻攻 to ban its active until the turn ends.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaDamageRecordedBySkill,
    /// Living owner observes another player enter dying once per round.
    OwnerAliveOtherDyingOncePerRound,
    /// Living event player can discard at least two of their own hand/equipment cards.
    OwnerIsTargetDiscardableHandEquipGteTwo,
    /// Living owner used a logical Sha on a current living target with at least one hand card.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaTargetAliveHandNonempty,
    /// A living owner observes any logical equipment-card use.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveEquipUsed,
    /// Owner is the damaged player and the living damage source has at least one disabled
    /// weapon slot.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerSourceAliveDisabledWeapon,
    /// The living owner is the phase event player and at least one living player has no hand
    /// cards.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerAnyAliveHandEmpty,
    /// Owner is the End-phase player and another living player has no more hand cards.
    OwnerIsEventPlayerOtherHandLteOwner,
    /// The living owner is the Prepare-phase event player, is wounded, and the draw pile has
    /// never been reshuffled.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerDrawReshuffleZeroWounded,
    /// The living owner is the Prepare-phase event player after at least one draw-pile
    /// reshuffle.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerDrawReshufflePositive,
    /// Owner is the event player, the event source is alive, and this skill has not fired
    /// during the current turn.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerSourceAliveOncePerTurn,
    /// Owner took sourced damage and the living source has a card the owner may take.
    /// Self-sourced damage exposes equipment only; damage from another player exposes hand
    /// and equipment cards.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerSourceAliveGainableCard,
    /// Owner is the damaged player and the current damage card is still in the processing zone.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerCardInProcessing,
    /// A living owner observes another living player's End phase and can discard at least one
    /// effective basic card from hand.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherPhaseEndHasDiscardableBasicHand,
    /// A living owner observes a living player's Prepare phase and that player's current HP is
    /// greater than the owner's.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAlivePreparePlayerHpGtOwner,
    /// The HP comparison above also requires the Prepare-phase player to hold the Lord identity.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAlivePrepareLordHpGtOwner,
    /// A living owner is not among the current Sha targets; another current target can discard a
    /// hand/equipment card and has the owner in that target's attack range.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherShaTargetCanRedirectToOwner,
    /// A living owner is the attributed giver and the living recipient is another player.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveGaveCardsToOtherAlive,
    /// The living owner is the current Sha target, can discard a hand/equipment card, and can
    /// redirect that target slot to another legal player in the owner's attack range.
    ///
    /// Appended for serialized ordinal stability.
    OwnerShaTargetCanDiscardRedirectToOtherInAttackRange,
    /// 批 271：告援——成为【杀】目标，存在持「诤」标的合法转移候选。
    OwnerShaTargetCanDiscardRedirectToZhengjianMarked,
    /// 批 272：度断——成为【杀】目标（每回合一次），owner 有可弃牌。
    OwnerIsTargetShaSourceAliveHandEquipNonemptyOncePerTurn,
    /// 批 276：镇南——成为锦囊目标且目标数>max(1,使用者体力)，owner 有可弃牌。
    OwnerIsTargetTrickTargetsGtSourceHpHandEquipNonempty,
    /// 批 277：姝勇——owner 使用【杀】（card_used 臂）。
    OwnerIsSourceShaUsed,
    /// 批 277：姝勇——owner 打出【杀】（card_responded 臂）。
    OwnerIsResponderSha,
    /// 批 278：羌勇——owner 的【杀】造成伤害（damage_before）。
    OwnerIsSourceShaCardDamageBefore,
    /// 批 288：义襄——成为使用牌目标且来源体力>自身。
    OwnerIsTargetSourceHpGtOwnerPerRoleTurn,
    /// 批 279：镇抚——own 结束阶段且本回合失去过牌（card_loss_count>0）。
    OwnerPhaseEndLostCards,
    /// 批 280：照月狮子盔——owner 为伤害目标且伤害量>1。
    OwnerIsTargetDamageAmountGtOne,
    /// 批 293：spolaige 觉醒——全局第二次濒死结算后（DyingAfter）。
    OwnerAliveSecondGlobalDying,
    /// 批 294：直谏——任意角色结束阶段，owner 本回合用过/打出过牌、
    /// 被指定为目标过且射程内有角色。
    OwnerUsedAndWasTargetedThisTurnInRange,
    /// 批 304：威临——owner 为伤害来源、自己回合内、目标本局未受过伤、
    /// owner 本局用牌数 ≥ 目标体力。
    OwnerSourceInTurnTargetUnharmedCardActionsGeTargetHp,
    /// 批 305：粗遐——owner 为使用牌目标，source 体力更高或伤害过 owner。
    OwnerIsTargetSourceHpGtOwnerOrSourceDamagedOwner,
    /// 批 305：未命——owner 为使用牌目标，owner 体力更高或 source 伤害过 owner。
    OwnerIsTargetOwnerHpGtSourceOrSourceDamagedOwner,
    /// 批 307：三首——owner 为伤害目标（携带类型位图 track 信号）。
    OwnerIsTargetSanshouTypeTracked,
    /// The living owner is the event player and is currently linked.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerLinked,
    /// The living owner is the draw-phase player and at least one living wounded player, or an
    /// eligible other Wu player under enabled Lord skill Guiming, exists.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerAliveWoundedOrGuimingWuExists,
    /// The living owner used a logical Sha or ordinary trick and can discard one hand/equipment
    /// card.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaOrOrdinaryTrickHasDiscardableHandEquip,
    /// The living owner dealt logical-Sha damage boosted by Jiu and still has the configured
    /// effectively enabled skill.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceJiuBoostedShaDamageSkillEnabled(SkillDefId),
    /// The living owner is the event player and at least one living player can discard one of
    /// their own hand cards.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerAnyAliveSelfDiscardableHand,
    /// The living owner is the event player and at least one living player is wounded.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerAnyAliveWounded,
    /// A living owner observes another living player of the configured group use or respond
    /// with Shan outside that player's turn; fires at most once in the current turn.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherShanOutsideTurnGroupOncePerTurn(String),
    /// The living owner is ending their turn after losing one to current-HP hand/equipment
    /// cards, and at least one uniquely lost physical card remains in discard.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerLostCardsAtMostHpGainable,
    /// The living owner is the damaged event player, the source is living, and at least one
    /// compatible field card can move from another living player to that source.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerSourceMovableFieldCard,
    /// The living owner is the damaged player and at least one other living player has a hand,
    /// equipment, or judgment card. Emits once per damage point.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerPerDamagePointOtherHandEquipJudgeNonempty,
    /// The living owner is ending their discard phase after discarding at least two physical
    /// cards from their hand/equipment region during that phase.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerDiscardedAtLeastTwoInDiscardPhase,
    /// A living owner killed at least one character during the turn that is ending.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveKilledThisTurn,
    /// The owner used an ordinary trick and has the configured stable mark.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceOrdinaryTrickMarkPositive(u32),
    /// A living owner observes sourced Sha damage to a living player and can pay one
    /// discardable hand/equipment card.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveShaDamageSourceTargetAliveOwnerHandEquipNonempty,
    /// During the owner's Play phase, one of the first two cards they use targets another player.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceCardHasOtherTargetCurrentPlayPhaseFirstTwo,
    /// The living owner used a logical Sha and the current living target has no more hand cards
    /// than the owner or at least as much HP as the owner.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceShaTargetHandLteOwnerOrHpGteOwner,
    /// The living owner is entering their Prepare phase with at least one judgment-zone card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerJudgeNonempty,
    /// The living owner is entering an unreplaced Draw phase.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerDrawNotReplaced,
    /// Living owner has fewer hand cards than min(8, living player count).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveHandBelowAliveCountCappedEight,
    /// The living owner is the current target of a delayed trick and has no pending judgment
    /// phase skip.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveDelayedTrickTargetWithoutPendingJudgeSkip,
    /// The living owner used their first logical Sha or first Juedou of the current turn and
    /// specified exactly one target.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceShaOrJuedouSingleTargetFirstSameNameThisTurn,
    /// The living owner used a logical Sha or Juedou, specified exactly one living target, and
    /// that target currently has at least one hand card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceShaOrJuedouSingleTargetHandNonempty,
    /// The living owner is the source of pending logical-Sha damage to a living target and may
    /// discard at least one hand/equipment card from either endpoint.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceShaDamageBeforeEndpointDiscardableHandEquip,
    /// The living owner used or responded with a Sha/Shan converted by a Longdan/Fanghun-named
    /// skill; the current player is the first counterpart and has at least one hand card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerLongdanViewAsFirstCounterpartHandNonempty,
    /// The living owner is about to deal damage to another living player.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceOtherDamageBefore,
    /// The living owner is about to deal direct, non-chain damage to another living player.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceOtherDirectDamageBefore,
    /// Living owner is the damage target and can discard a hand/equipment card.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsTargetDiscardableHandEquip,
    /// Living owner is targeted by another player and has a face-down card in the named skill
    /// pile.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveTargetedByOtherHiddenSkillPile(u32),
    /// Event player is the owner, who has hand cards and a face-down card in the named skill pile.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerHandNonemptyHiddenSkillPile(u32),
    /// Owner lost at least one hand/equipment/judgment card whose suit matches a face-up card in
    /// the named skill pile.
    ///
    /// Appended for serialized ordinal stability.
    OwnerLostMatchingRevealedSkillPileSuit(u32),
    /// Living owner observes an event after represented living groups fall to two or fewer.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveGroupCountAtMostTwo,
    /// Living owner is the current turn player.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveCurrentTurn,
    /// Owner lost a package-defined equipment card whose leave handler is this skill.
    ///
    /// Appended for serialized ordinal stability.
    OwnerLostPackageCardFromEquip,
    /// Debate resolved and the owner is the initiator (ADR 0644 P2).
    DebateOwnerIsInitiator,
    /// Debate resolved and the owner revealed an opinion (red or black).
    DebateOwnerParticipated,
    /// Debate resolved; owner and initiator both participated with different colors.
    DebateOwnerOpinionDiffersInitiator,
    /// Debate settled red (majority) and the owner initiated it (ADR 0644 P2).
    DebateOwnerInitiatorResultRed,
    /// Debate settled black and the owner initiated it.
    DebateOwnerInitiatorResultBlack,
    /// Damage dealt by a source other than the owner (bystander view; ADR 0597 gap
    /// family daofa/chengjian/xuezhou, threshold met 2026-08-22).
    OwnerAliveOtherSourceDamage,
    /// Owner used a card during their current Play phase while the named skill mark is mode one
    /// or two.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourcePlaySkillMarkMode(u32),
    /// A living owner observes pending logical-Sha damage from self or an allied source.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveSourceAllyShaDamageBefore,
    /// The wounded living owner enters Prepare after at least one allied player has died.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerWoundedDeadAlly,
    /// The living owner is about to take damage, can discard at least two hand/equipment
    /// cards, and has not accepted this trigger in the current round.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerDiscardableHandEquipGteTwoOncePerRound,
    /// The living owner is about to take damage from another living source, owns at least one
    /// hand/equipment card, and this skill has not recorded that source.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerOtherSourceAliveHandEquipNonemptyOldHuishengUnused,
    /// A different living player is dying outside the owner's turn, and the owner can supply a
    /// legal Sha whose targets include the current turn player.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherDyingOutsideTurnCanShaCurrent,
    /// Damage belongs to the exact old 解烦 Sha used by this owner.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceOldJiefanSha,
    /// The owner is the current turn player and first target context of a non-equipment card use.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceCurrentTurnFirstTargetNonEquip,
    /// The owner used a tagged face-down view-as card and the current 谶纬 branch has a target.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceFaceDownViewAsChenweiAvailable,
    /// The living owner is the End-phase event player, owns at least one hand/equipment card,
    /// and another living player exists.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerPhaseEndHandEquipNonemptyOtherAlive,
    /// The living owner caused logical-Sha damage to a living target with equipment the owner
    /// may discard.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaDamageTargetAliveHasDiscardableEquip,
    /// The living owner used an equipment card while this skill's lifetime counter is below seven.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceEquipUsedSkillMarkBelowSeven,
    /// The living owner used or responded with a card outside their own turn.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceOutsideTurnCardUsedOrResponded,
    /// Living owner observes a logical Shan used by themselves, or by anyone during their turn.
    OwnerAliveShanUsedByOwnerOrDuringOwnerTurn,
    /// Living owner observes another living player enter Play phase and has a nonempty public
    /// pile for this skill definition.
    OwnerAliveOtherPhasePlaySkillPileNonempty,
    /// Skill owner is alive after the event and is its source.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveIsEventSource,
    /// A living owner used at least one logical basic, trick, or equipment card during the
    /// exact Play phase that is ending.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAlivePlayPhaseCardCategoryCountPositive,
    /// The living owner is the phase event player and may discard at least one hand/equipment
    /// card from some living player, including themselves.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerAnyAliveDiscardableByOwnerHandEquip,
    /// A living owner observes another living player's End phase and has at least one hand card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherPhaseEndHandNonempty,
    /// The living owner is entering an unfixed Draw phase while at one HP or with no hand cards.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerDrawNotFixedHpOneOrHandEmpty,
    /// The living owner used a logical Tiesuo on a living, unlinked target whose hand or
    /// equipment region is nonempty.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceTiesuoTargetAliveUnlinkedHandEquipNonempty,
    /// The living owner is the damage source and the target was linked immediately before that
    /// damage segment committed.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveSourceTargetWasLinked,
    /// A living owner observes a non-fixed-multitarget ordinary trick with at least two declared
    /// targets.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveNonMultitargetTrickTargetsGtOne,
    /// The living owner enters an unfixed Draw phase with a positive pending draw count and at
    /// least one other living player has a hand card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventPlayerPositiveDrawOtherHandNonempty,
    /// The living owner used a black card and this is its first target event; at least one
    /// non-owner target in the full event target list has a discardable hand/equipment card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceFirstTargetBlackCardAnyTargetOtherDiscardableHandEquip,
    /// Living owner dealt direct Sha damage to a living target with armor or a mount, and the
    /// physical Sha carrier remains in processing or discard.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceDirectShaDamageTargetArmorOrMountCardGainable,
    /// A living owner observes a living event player lose armor or a mount while that player has
    /// cards in the public pile keyed by the originating skill definition.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerLostArmorOrMountHasOriginatingSkillPile,
    /// The living owner is a damage endpoint and still has a legal unused 界邀名 action.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsEventSourceOrPlayerXinyaomingAvailable,
    /// Once per current turn, the living owner is about to deal damage to another living player.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceOtherDamageBeforeOncePerTurn,
    /// A living owner observes another living player's turn end after that player dealt no
    /// damage to another player during the turn.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherTurnEndedNoDamageOther,
    /// A living owner gained at least two physical cards this turn and both the owner and
    /// another living player have a hand card for comparison.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveGainedTwoThisTurnCanCompareOther,
    /// The living owner used a logical Sha during their own current Play phase.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaCurrentPlayPhase,
    /// The owner is the event player on their first actual HP change this turn, and trick cards
    /// are the sole most numerous basic/trick/equipment category in their current hand.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerFirstHpChangeTurnUniqueTrickMajority,
    /// Owner is the event player and the event's physical card is an effective Sha.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerEventCardSha,
    /// Owner is the event player and the event's physical card is an effective Sha or trick.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerEventCardShaOrTrick,
    /// Owner is the card user or within attack range of the user, and the card is Sha or Juedou.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceOrWithinDistanceOneShaOrJuedou,
    /// Once per turn, the living owner is about to damage an earlier-seated living target while
    /// holding fewer cards than that target's max HP.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceTargetEarlierSeatHandBelowTargetMaxHpOncePerTurn,
    /// The living owner used or was targeted by an effective-club card, at least one living
    /// player is linked, and this skill has not fired in the current turn.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceOrTargetedClubLinkedExistsOncePerTurn,
    /// The living owner used a logical Sha backed by exactly one effective trick source card.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaSingleTrickSourceCard,
    /// The living owner used a logical Sha backed by exactly one effective equipment source card.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaSingleEquipSourceCard,
    /// Owner is the living event player with more hand cards than HP, and the living Lord's
    /// normalized character name is not Liu Bei.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerHandAboveHpNoLiubeiLord,
    /// The living owner observes another player enter dying for the first time seen by this
    /// skill instance.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherDyingFirstPerTarget,
    /// The living owner is the event player (e.g. at the start of their own discard phase) and has
    /// used zero logical trick cards so far this turn. At the discard-phase start this turn flag
    /// only reflects cards the owner has used, because discards happen later inside the same phase.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerTrickUsedZero,
    /// The living owner is the event player and has at most one HP. Used by awakening skills such
    /// as 孙坚「魂姿」 that fire at the owner's prepare phase while at death's door. Reads only the
    /// owner's intrinsic HP, so it cannot perturb canonical state hashes.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerHpLeOne,
    /// The living owner is the event player and has at most two HP. Used by awakening skills such
    /// as 界孙坚「界魂姿」 that fire at the owner's prepare phase while low on HP. Reads only the
    /// owner's intrinsic HP, so it cannot perturb canonical state hashes.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerHpLeTwo,
    /// The living owner is either the damaged event player or the damage source, and the event
    /// player is taking their second applied damage of the current turn. Powers skills like
    /// 「凝厄」 that react to anyone's second damage in a turn. Reads the ABI 347 opt-in per-player
    /// damage-taken counter; packages that do not opt in keep their canonical state hashes.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceOrTargetEventPlayerSecondDamageThisTurn,
    /// The living owner is either the damaged event player or the damage source, and the event
    /// player is taking their first applied damage of the current turn. Reads the ABI 347 opt-in
    /// per-player damage-taken counter.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceOrTargetEventPlayerFirstDamageThisTurn,
    /// The living owner is the source of a used logical Sha or ordinary trick card. Bare variant
    /// (no extra hand/equipment precondition) for skills like 「扼睨」 that react to any such use.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaOrOrdinaryTrick,
    /// The living owner used a 无懈可击 (CardKind::TrickWuxie), once per turn.
    /// ABI 462 (C 段 omp, `stdjujian`).
    OwnerSourceWuxieUsed,
    /// The living owner is the event player at Prepare and holds more hand+equip+judge
    /// cards than their HP (dcdanji 单骑 awakening). ABI 466 (C 段 omp).
    OwnerIsEventPlayerHandEquipJudgeAboveHp,
    /// The living owner used a card during their own play phase whose targets overlap the
    /// previous card's targets (xinfu_lianpian 连篇), at most three times per turn.
    /// ABI 472 (C 段 omp).
    OwnerSourcePlayCardRepeatsPreviousTargets,
    /// The living owner used a Sha whose targets overlap the previous Sha's targets this
    /// turn, once per turn (gongjian 拱坚). ABI 473 (C 段 omp).
    OwnerSourceShaRepeatsPreviousShaTargets,
    /// The living owner's equipment zone gained or lost a card (xinfu_qinguo 勤国).
    /// ABI 474 (C 段 omp).
    OwnerEquipZoneChanged,
    /// The living owner targeted a player with a Sha, once per turn (dctingxian 铤险).
    /// ABI 464 (C 段 omp).
    OwnerSourceShaOncePerTurn,
    /// The living owner is the event player and also the current turn player.
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsEventPlayerCurrentTurn,
    /// The living owner dealt this turn's first damage event to the event player, who is within
    /// final distance one.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceTargetDistanceLeOneFirstDamageToTargetThisTurn,
    /// Living owner used a logical Sha converted by a skill from at least one physical source
    /// card; limited to the first matching use in each turn.
    ///
    /// Appended for serialized ordinal stability.
    OwnerFirstTransformedShaUsedThisTurn,
    /// Living owner is about to take elemental or ordinary-trick damage while their hand is empty.
    ///
    /// Appended for serialized ordinal stability.
    OwnerTargetHandEmptyElementalOrTrickDamageBefore,
    /// A living owner observes themselves or a tied-lowest-HP living player draw cards while at
    /// least one basic/trick/equipment category remains available for TW 教化 this turn.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveDrawPlayerOwnerOrMinHpTwjiaohuaAvailable,
    /// The living owner lost at least one hand/equipment card during the current turn.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveLostHandThisTurn,
    /// PhaseEnd (any seat): owner alive and lost ≥2 cards this turn (turn-level loss counter).
    /// Used by 精练 `dcsbjinglian` (global phaseEnd + getHistory lose length > 1).
    OwnerAlivePhaseEndLostCardsAtLeastTwo,
    /// Another living player used/responded with a tracked TW 苦谏 card, including view-as
    /// source-card movement.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherTwkujianUsedOrResponded,
    /// Another living player lost a tracked TW 苦谏 card from hand for any other reason.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherTwkujianLost,
    /// Living owner has not yet opened TW 睿敛's target choice this round.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveTwruilianRoundUnprompted,
    /// The selected TW 睿敛 player discarded cards during their own turn.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveTwruilianSelectedTargetDiscarded,
    /// The selected TW 睿敛 player's whole turn ended.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveTwruilianSelectedTargetTurnEnded,
    /// The living owner lost a physical weapon from equipment outside their own turn, excluding
    /// movement into their own hand.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveLostWeaponOutsideTurn,
    /// Another living player's prepare phase began and both they and the owner can compare.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherPhasePrepareCanCompareOwner,
    /// Living owner is targeted by another living player's physical Sha while that source has
    /// strictly more HP.
    ///
    /// Appended for serialized ordinal stability.
    OwnerTargetPhysicalShaSourceHpGtOwner,
    /// Owner targets another player with Sha or an ordinary trick and 果毅's HP/hand condition
    /// holds; replays attributed to this exact skill instance are excluded.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceOtherTargetShaOrOrdinaryTrickTwguoyiCondition,
    /// At a phase end, the living owner has made at least two distinct other players enter dying
    /// through damage during the current turn.
    OwnerAlivePhaseEndCausedTwoDistinctDying,
    /// Cards moved from another player's personal zone into the owner's hand.
    OwnerGainedEventCardsFromOtherPersonalZone,
    /// Another player used exactly one marked 遗珠 card on a single target.
    OwnerAliveOtherSingleTargetUsedTrackedTwyizhu,
    /// At least one marked 遗珠 card entered the discard pile.
    OwnerTrackedTwyizhuEnteredDiscard,
    /// Another 共患 holder with no more HP than the owner is about to take unredirected damage.
    OwnerAliveOtherTwgonghuanDamageBeforeHpLteOwnerUnused,
    /// Owner's turn begins while at least one playable skill pile is nonempty.
    OwnerPhaseBeginPlayableSkillPileNonempty,
    /// Owner used or responded with a physical card consumed from a playable skill pile this turn.
    OwnerUsedPlayableSkillPileCardUnused,
    /// Owner is about to take damage while every playable skill pile they own is empty.
    OwnerTargetDamageBeforePlayableSkillPilesEmpty,
    /// A living owner observes damage about to hit a living player tied for minimum HP; unused.
    OwnerAliveDamageBeforeTargetMinHpUnused,
    /// Any living owner at the beginning of any End phase.
    OwnerAliveGlobalPhaseEnd,
    /// Owner's End phase after using at least one card and dealing no damage this turn.
    OwnerAlivePhaseEndUsedCardNoDamage,
    /// Owner's personal cards entered discard for a reason other than use/response; unused.
    OwnerCardsEnteredDiscardNotUseUnused,
    /// The living owner used the first logical Sha of their current Play phase.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceFirstShaThisPlayPhase,
    /// Owner dealt damage to a target within distance one and has an enabled empty equipment slot.
    OwnerSourceDamageTargetDistanceAtMostOneEmptyEquipSlotUnused,
    /// A target of the owner's first logical Sha in the current turn.
    OwnerSourceFirstShaTargetThisTurn,
    /// Another player's Play phase ended without satisfying the selected 征建 condition.
    OwnerAliveOtherPhasePlayEndTwzhengjianCondition,
    /// The living owner used a card and the card-targeted event player carries the mark.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceTargetMarkPositive(u32),
    /// Owner's once-per-turn Sha target has discardable hand cards or a movable field card.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaTargetTwzhenxiAvailableOncePerTurn,
    /// First target event of the owner's Sha or damage trick; once per turn.
    OwnerSourceFirstTargetDamageCardOncePerTurn,
    /// Owner discarded red cards into discard outside their own turn.
    OwnerOutsideTurnDiscardedRedCardsInDiscard,
    /// Owner is dying; another living player has a hand/equipment card; unused lifetime skill.
    ///
    /// Appended for serialized ordinal stability.
    OwnerDyingOtherHandEquipNonemptyOncePerGame,
    /// Owner is targeted by any card used by another player; once per turn.
    ///
    /// Appended for serialized ordinal stability.
    OwnerTargetedByOtherCardOncePerTurn,
    /// Conversion Feifu branch: owner is the sole Sha target of another player, or owner used a
    /// sole-target Sha; that target still has a hand/equipment card.
    ///
    /// Appended for serialized ordinal stability.
    OwnerTwfeifuSingleShaPartyTargetHasHandEquip,
    /// Living owner can replace another in-range user's sole Sha target with themselves.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherSourceSingleShaTargetOtherOwnerInAttackRange,
    /// The exact other-user Sha redirected by this skill finished and its living user has H/E.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherSourceShaResolvedTwxiaolianRedirectHasHandEquip,
    /// Another player fully resolved a Sha carried by this skill instance's tracked card and it
    /// dealt damage.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherShaResolvedTrackedSkillCardDealtDamage,
    /// Another player fully resolved a Sha carried by this skill instance's tracked card without
    /// dealing damage.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherShaResolvedTrackedSkillCardNoDamage,
    /// A tracked physical card left a hand for a reason other than use or response.
    ///
    /// Appended for serialized ordinal stability.
    OwnerTrackedSkillCardLeftHandNotUseOrRespond,
    /// Owner or a living player within the owner's attack range used Sha while the owner can
    /// discard a hand/equipment card.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveShaUsedBySelfOrInRangeDiscardableHandEquip,
    /// The exact Sha tracked by this skill fully resolved without damage and its user is alive.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveTrackedCuijinShaResolvedNoDamage,
    /// The exact Sha tracked by this skill fully resolved after dealing damage.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveTrackedCuijinShaResolvedDealtDamage,
    /// The exact materialized virtual Duel tracked by this skill finished resolving.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveTrackedTwqingkouDuelResolved,
    /// The first phase-begin event observed by this skill in the current round.
    OwnerAliveTwchenxunRoundUnprompted,
    /// The tracked Chenxun Duel actually damaged its selected target.
    OwnerAliveTrackedTwchenxunDuelTargetDamaged,
    /// The tracked Chenxun Duel did not damage its selected target.
    OwnerAliveTrackedTwchenxunDuelTargetNotDamaged,
    /// The tracked Chenxun virtual card left processing without a Duel completion event.
    OwnerAliveTrackedTwchenxunCardLeftProcessing,
    /// A living owner lost cards this turn and another living player's turn is ending.
    ///
    /// Appended for serialized ordinal stability. ABI 496.
    OwnerAliveGlobalPhaseEndLostCardsTargetOtherAlive,
    /// A living owner observes an End phase while at least one card discarded this turn remains
    /// in the discard pile and there is a legal fixed/other target. ABI 583.
    OwnerAliveGlobalPhaseEndDiscardedCardsAvailable,
    /// Owner is the damage source and the damage target does not have the configured group tag.
    ///
    /// Appended to keep existing serialized enum ordinals stable. ABI 498.
    OwnerSourceTargetNotGroup(String),
    /// A neighboring living player gained cards into hand outside that player's Draw phase, and
    /// the owner may gain a hand/equipment card from them; once per turn. ABI 487.
    OwnerAliveNeighborGainedCardsOutsideOwnDrawOncePerTurn,
    /// Another player's End phase begins and the owner can resolve Mobile Danshou from the
    /// exact current-turn count of cards that player used targeting the owner. ABI 500.
    OwnerAliveOtherPhaseEndMobiledanshou,
    /// 批 220：其他角色结束阶段，且其本回合未对拥有者造成过伤害（伤害对计数）。
    OwnerAliveOtherPhaseEndNotDamagedOwnerThisTurn,
    /// 批 220：其他角色结束阶段，其本回合对拥有者用牌且拥有者本回合未受伤。
    OwnerAliveOtherPhaseEndUsedCardOnOwnerUndamagedThisTurn,
    /// Another player fully resolved a damage card targeting the living owner, and this owner
    /// received damage from that exact use. ABI 503.
    OwnerAliveOtherSourceDamageCardResolvedTargetedDamaged,
    /// Same boundary, but this owner received no damage from the exact use. ABI 503.
    OwnerAliveOtherSourceDamageCardResolvedTargetedNotDamaged,
    /// Another living player is dying and at least one Chen She role has a discardable card.
    /// ABI 503.
    OwnerAliveOtherDyingMbchensheAvailable,
    /// Owner's End phase begins with at least one discardable hand/equipment card; once per round.
    /// ABI 504.
    OwnerIsEventPlayerDiscardableHandEquipOncePerRound,
    /// First discard event outside the Discard phase this turn; 任行 has a legal branch. ABI 504.
    OwnerAliveFirstNonDiscardPhaseDiscardMbrenxingAvailable,
    /// Owner used a hand damage card, this is its first target, and can discard hand cards. ABI 505.
    OwnerSourceFirstTargetHandDamageCardDiscardable,
    /// Owner lost a physical Sha from hand outside use/response, once per turn. ABI 505.
    OwnerLostHandOwnerlessShaNotUseOncePerTurn,
    /// Any phase ended while the owner's originating skill pile is nonempty. ABI 505.
    OwnerAlivePhaseAnyEndOriginatingSkillPileNonempty,
    /// Owner dealt damage with the Sha currently tracked by 断鞅; target has HEJ cards. ABI 505.
    OwnerSourceDamageTrackedMbduanyangTargetRecastable,
    /// 润微's armed loss threshold observes a qualifying owner card-loss batch. ABI 506.
    OwnerMbrunweiLossThreshold,
    /// Owner's Play phase ended with tracked 润微 cards still in hand. ABI 506.
    OwnerPhasePlayEndMbrunweiTrackedHand,
    /// Damage is about to hit a role within final distance one of the owner, once per turn. ABI 506.
    OwnerAliveDamageBeforeTargetDistanceLeOneOncePerTurn,
    /// Owner dealt damage while 凶图's next-damage reward is armed. ABI 507.
    OwnerSourceDamageMbxiongtuArmed,
    /// Owner's Play phase ended while 凶图 has transient state. ABI 507.
    OwnerPhasePlayEndMbxiongtuState,
    /// Owner's turn End phase with a living target remembered by 刚鲠. ABI 508.
    OwnerPhaseEndMbganggengTargetAlive,
    /// Owner entered dying or became handless by losing hand cards; at most twice per turn. ABI 508.
    OwnerFromHandEmptyOrDyingMbsijianTwicePerTurn,
    /// Owner's Prepare phase or post-damage window can move a field card; at most twice per turn. ABI 509.
    OwnerIsTargetCanMoveFieldCardMbyuanmoTwicePerTurn,
    /// Owner used a single-target Sha and this is its first target; at most twice per turn. ABI 510.
    OwnerSourceFirstSingleTargetShaMbfeijingTwicePerTurn,
    /// Damage from the bound Sha is about to hit a 飞径-added target. ABI 510.
    OwnerSourceDamageBeforeMbfeijingExtra,
    /// Owner's originally single-target Sha finished resolving. ABI 510.
    OwnerSourceShaResolvedOriginalSingleMbxiaoge,
    /// A living owner observes another player's Discard phase ending after that player
    /// discarded at least one card into the discard pile during the phase. ABI 513 (固政).
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherDiscardedInDiscardPhase,
    /// A living damage target ties for minimum hand count; once per turn. ABI 514.
    OwnerAliveDamageTargetMinHandMbxuyeOncePerTurn,
    /// A tracked 狂想 role lost its final received exchange card from hand. ABI 514.
    OwnerMbkuangxiangReceivedCardsExhausted,
    /// The owner's next Play phase begins while 狂想 tracking is active. ABI 514.
    OwnerPhasePlayMbkuangxiangActive,
    /// The owner gained cards outside the recursive 督作 movement. ABI 515.
    OwnerGainedCardsPotduzuo,
    /// 批 226：Potduzuo 增益判定的每回合一次变体（防自触发递归）。
    OwnerGainedCardsPotduzuoOncePerTurn,
    /// Sha damage is pending and owner or target has hand count above HP. ABI 515.
    OwnerShaDamageBeforePotbihanAvailable,
    /// Living owner is targeted by a card while at full HP. ABI 516.
    OwnerAliveCardTargetedHealthy,
    /// Current turn player finished a card and has this skill's expansion pile. ABI 517.
    OwnerAliveCardResolvedSourceCurrentPotsifengPile,
    /// A phase-ending player has this skill's expansion pile. ABI 517.
    OwnerAlivePhaseEndPlayerPotsifengPile,
    /// A living owner sees an End phase after at least one living player lost cards this turn;
    /// once per round. ABI 518.
    OwnerAlivePotdingpinAvailableOncePerRound,
    /// Owner finished resolving the exact physical card armed in this skill's phase policy. ABI 519.
    OwnerSourceCardResolvedPhasePolicy,
    /// Owner's Play phase ended with hand/equipment cards and unused 灌沙. ABI 520.
    OwnerPhasePlayEndPotguanshaAvailableOncePerGame,
    /// Owner discarded at least one physical Sha into the shared discard pile. ABI 522.
    ///
    /// Appended for serialized ordinal stability.
    OwnerDiscardedShaToDiscard,
    /// A living owner is the lord of a Wu-faction player's Play phase where both can start a
    /// hand comparison. ABI 523 (制霸).
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherWuPlayPhaseCanCompareOwner,
    /// The living owner is among the targets of a trick that has more than one target and
    /// holds at least one hand/equipment card. ABI 524 (从俭).
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsTargetMultiTargetTrickHasHandEquip,
    /// Another player begins their Play phase, misses at least one 整肃 type, and is
    /// within the owner's attack range. ABI 432 (侯封).
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherPlayPhaseZhengsuMissingInRange,
    /// The owner begins their own Play phase missing at least one 整肃 type. ABI 432 (燕集).
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsTargetMissingZhengsu,
    /// Owner used a club card. ABI 435 (彰名).
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceClub,
    /// Another player begins their Play phase holding a hand card, and the owner
    /// can discard at least as many hand/equip cards as their 凶竖 mark. ABI 436.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherPlayPhaseDiscardableGteMark,
    /// Another player uses a trick whose first target is the owner, and that
    /// player's hand count is tied for the highest among living players. ABI 438.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherTrickFirstTargetSourceMaxHand,
    /// Another player recovered HP while holding the 檄 mark and is still alive
    /// with positive HP. ABI 441 (索命).
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherMarkedXi,
    /// The owner became the single target of another player's card, once per
    /// turn. ABI 442 (通夺).
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsTargetSingleTargetOtherSourceOncePerTurn,
    /// The owner lost exactly one hand/equip/judge card while another player is
    /// the current-phase actor, once per turn. ABI 447 (孤影).
    ///
    /// Appended for serialized ordinal stability.
    OwnerLostOneHandEquipJudgeOtherCurrentPhaseOnce,
    /// Another player's turn ended after a recorded 伏暗 suit entered discard. ABI 525.
    OwnerAliveOtherTurnDiscardedRecordedPotfuanSuit,
    /// The owner's first 伏暗 virtual Sha resolved after dealing damage. ABI 525.
    OwnerSourceShaResolvedPotfuanDealtDamage,
    /// The living owner ends their Play phase after 飞军 first chose a new target this turn.
    /// ABI 526 (兵略).
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsTargetFeijunNewTarget,
    /// The living owner enters their Discard phase owing more cards than their hand limit
    /// while another Qun-faction player is alive. ABI 528 (伪帝).
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsTargetDiscardPhaseOweLimitOtherQun,
    /// The living owner caused Sha damage to a living target and both hold at least one hand
    /// card. ABI 530 (烈刃).
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaDamageTargetAliveBothHand,
    /// A living owner observes another player using Nanman whose physical cards are still in
    /// the discard pile. ABI 530 (巨象).
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherNanmanCardsDiscard,
    /// The living owner's bound skill target is the current event's second comparison
    /// participant. ABI 530 (烈刃胜负辅助).
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveSkillBoundTargetIsEventSource,
    /// The owner lost cards out of their own hand/equip/judge zones while it is not the
    /// owner's turn. ABI 533 (屯田).
    ///
    /// Appended for serialized ordinal stability.
    OwnerLostCardsOutsideTurn,
    /// The owner's prepare phase begins with at least three cards in the named skill's
    /// public pile (田 ≥ 3). ABI 534 (凿险).
    ///
    /// Appended for serialized ordinal stability.
    OwnerPhaseBeginSkillPileGteThree(u32),
    /// The owner's prepare phase begins with no hand cards. ABI 538 (志继).
    ///
    /// Appended for serialized ordinal stability.
    OwnerPhaseBeginHandEmpty,
    /// A Nanman card the owner observes resolves with at least one dead target or more
    /// than three damaged targets. ABI 542 (求首).
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveNanmanResolvedTargetDeadOrDamagedGtThree,
    /// The owner's damage-tagged basic/trick names a first target while some target has
    /// hand count at least the owner's and a discardable hand/equipment card. ABI 545
    /// (征荣).
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceFirstDamageCardTargetHandGteOwner,
    /// The owner just responded with a 闪 (shan). ABI 549 (雷击).
    ///
    /// Appended for serialized ordinal stability.
    OwnerRespondedShan,
    /// Another living Qun player's play phase begins while the owner is alive and that
    /// player holds a 闪/闪电. ABI 551 (黄天, once per round).
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherQunPlayPhaseCanGiveOwner,
    /// Another living player's Play phase begins with a nonempty hand before that actor has
    /// invoked any 诫节 this phase. ABI 554.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAlivePlayPhaseActorHandNonemptyPotjiejieUnused,
    /// The owner is the source of pending Sha damage and at least one 饮战 comparison holds.
    /// ABI 558.
    OwnerSourceShaDamageBeforePotyinzhanCondition,
    /// The owner dealt positive damage to a target at current distance at most one. ABI 558.
    OwnerSourceDamageWithinOne,
    /// Owner gained cards; 势周瑜【炽沄】handles first-gain-per-phase state internally. ABI 561.
    OwnerGainedCardsPotzhouyu,
    /// Owner used a card and this is its first target event; at least one target has hand cards.
    /// ABI 561.
    OwnerSourceFirstTargetHasHandPotyanhui,
    /// A living owner observes fire damage to another currently linked role. ABI 561.
    OwnerAliveFireDamagePotfentao,
    /// 势周瑜【焚涛】has a pending post-damage relink for this event target. ABI 561.
    OwnerAlivePotfentaoPendingDamage,
    /// The owner lost at least one non-damage card from a personal zone, excluding an equipment
    /// card consumed by using that equipment, while 势屯田's charge is below its stored maximum.
    /// ABI 543.
    ///
    /// Appended for serialized ordinal stability.
    OwnerLostNonDamageCardPottuntianChargeNotFull,
    /// A Sha created by this exact 策断 skill instance resolved after dealing damage. ABI 544.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaResolvedTrackedPotceduanDealtDamage,
    /// Living owner saw one or more Sha enter the discard pile while 战烈 can still gain 烈.
    OwnerAliveShaEnteredDiscardPotzhanlieAvailable,
    /// Owner's tracked 战烈 Sha resolved and requested the post-use draw.
    OwnerSourceShaResolvedTrackedPotzhanlieDraw,
    /// Another living player's Play phase begins and 隳律 still has an unused round color.
    /// ABI 564.
    OwnerAliveOtherPlayPhasePothuilvAvailable,
    /// Owner is the event player and can legally use a source-free Wuzhong. ABI 564.
    OwnerIsTargetCanUseWuzhong,
    /// A non-empty discard batch has a living personal-zone owner. ABI 564.
    OwnerAliveDiscardBatch,
    /// Another player responded to a card used by the living owner. ABI 564.
    OwnerAliveOtherRespondedToOwner,
    /// Owner dealt damage at an unassigned 渐专 timing, or all timings await awakening. ABI 568.
    OwnerSourceDamagePotjianzhuanAvailable,
    /// Owner took damage at an unassigned 渐专 timing, or all timings await awakening. ABI 568.
    OwnerTargetDamagePotjianzhuanAvailable,
    /// Someone entered dying at an unassigned 渐专 timing, or all timings await awakening. ABI 568.
    OwnerAliveDyingPotjianzhuanAvailable,
    /// This 典易/奢权 instance owns the damage-source timing and has not fired it this round.
    OwnerSourceDamagePotcaoshuangAssignedUntriggered,
    /// This 典易/奢权 instance owns the damage-target timing and has not fired it this round.
    OwnerTargetDamagePotcaoshuangAssignedUntriggered,
    /// This 典易/奢权 instance owns the dying timing and has not fired it this round.
    OwnerAliveDyingPotcaoshuangAssignedUntriggered,
    /// Owner just used/responded with the exact card from 溺尊's playable pile. ABI 568.
    OwnerUsedPotnizunPileCard,
    /// Owner is about to take damage after using at least one 溺尊 pile card this round.
    OwnerTargetDamageBeforePotnizunUsed,
    /// A turn ended after owner used a card and tagged cards remain in discard.
    OwnerAliveTurnEndedUsedCardTaggedDiscard,
    /// Living owner has a pending 横江 settlement for this ended turn.
    /// Appended for serialized ordinal stability (ABI 572).
    OwnerAliveTurnEndedRehengjiangPending,
    /// Owner used a logical Sha whose effective suit is heart. ABI 580.
    /// Appended for serialized ordinal stability.
    OwnerSourceHeartSha,
    /// Owner used a red physical card tracked by the active 昭文 instance. ABI 584.
    /// Appended for serialized ordinal stability.
    OwnerSourceActiveDczhaowenTrackedRed,
    /// Owner is a target of Jiu, whose ordinary effect 酒遁 rewrites. ABI 584.
    /// Appended for serialized ordinal stability.
    OwnerIsTargetEventCardJiu,
    /// Another player targeted the owner with a black card and 酒遁 has a legal branch. ABI 584.
    /// Appended for serialized ordinal stability.
    OwnerIsTargetOtherSourceBlackDcjiudunAvailable,
    /// Owner's Play phase ended with a pending 征辟 target. ABI 585.
    /// Appended for serialized ordinal stability.
    OwnerAlivePhasePlayEndZhengbiPending,
    /// Owner took damage from a source with hand cards; repeat once per damage point. ABI 586.
    OwnerIsTargetSourceHandNonemptyPerDamagePoint,
    /// The active 巧谏 helper sees its owner's first still-unconsumed Sha/trick category. ABI 586.
    OwnerSourceQiaojianExtraAvailable,
    /// A player watched by 析察 used their next card. ABI 586.
    OwnerXichaWatchedSourceCardUsed,
    /// A physical card tagged by 巧谏 left the owner's hand. ABI 586.
    OwnerQiaojianTrackedCardLeftHand,
    /// Owner or a character in the owner's attack range took numeric card damage from another
    /// source. ABI 587.
    OwnerAliveDamageTargetSelfOrInRangeOtherSourceNumericCard,
    /// Owner dealt damage, at least one living character is wounded, and this skill has not fired
    /// this turn. ABI 587.
    OwnerSourceWoundedExistsOncePerTurn,
    /// Living owner and event player share a group, and the owner currently has at least one
    /// discardable hand card. ABI 596. Appended for serialized ordinal stability.
    OwnerAliveEventPlayerSameGroupHandNonempty,
    /// Owner dealt damage to the original target of the exact tracked virtual Duel, and that
    /// target's attack range contains a wounded living character other than the owner. ABI 597.
    /// Appended for serialized ordinal stability.
    OwnerTrackedJuedouOriginalTargetDamageRecoverableInRange,
    /// The living owner is the End-phase player, has hand cards, and at least one other living
    /// character also has hand cards. ABI 598. Appended for serialized ordinal stability.
    OwnerIsTargetPhaseEndHandNonemptyOtherHandNonempty,
    /// The living owner is targeted by an ordinary trick and still has a 辞虑 option. ABI 600.
    OwnerIsTargetOrdinaryTrickCilvAvailable,
    /// The damage source card is tracked by this 辞虑 instance's prevention option. ABI 600.
    CilvTrackedCardDamageBefore,
    /// The resolved card has pending 辞虑 cleanup or material gain. ABI 600.
    CilvTrackedCardResolved,
    /// The living owner is dying and this 痛悼 instance has never fired. ABI 600.
    OwnerDyingZhupeilanOncePerGame,
    /// Another tied-most-hand player's Play phase began and 附袭 has a legal branch. ABI 601.
    OwnerAliveOtherPlayPhaseTiedMaxHandDcsbfuxiAvailable,
    /// The owner's End phase began with a damage card that entered discard this turn without
    /// dealing damage. ABI 601.
    OwnerPhaseEndDcsbhaoyiAvailable,
    /// A living owner observes completed fire damage and arms non-stacking 融火. ABI 602.
    OwnerAliveGlobalFireDamageDcsbronghuo,
    /// The living owner is using a fire Sha or Huogong. ABI 602.
    OwnerSourceFireShaOrHuogongDcsbronghuo,
    /// The living owner fully resolved a card with at least one other living declared target;
    /// 英谋 may fire once per turn and must have a legal current conversion branch. ABI 602.
    OwnerSourceCardResolvedHasOtherTargetDcsbyingmouOncePerTurn,
    /// A card currently tracked by an in-flight 英谋 sequence fully resolved. ABI 602.
    DcsbyingmouTrackedCardResolved,
    /// A hand-to-hand transfer involved the owner and the current 盟谋 branch is unused this
    /// turn. ABI 603.
    OwnerHandTransferOtherDcsbmengmouBranchAvailable,
    /// Damage belongs to the exact Sha currently used by 盟谋's Yang branch. ABI 603.
    DcsbmengmouTrackedShaDamage,
    /// The exact Sha currently used by 盟谋's Yang branch fully resolved. ABI 603.
    DcsbmengmouTrackedShaResolved,
    /// The owner is about to take damage associated with a logical or physical card. ABI 604.
    OwnerIsTargetCardDamageBefore,
    /// The owner fully resolved a single-target Sha or trick using only hand materials and may
    /// use an ignore-distance Sha on the surviving target. ABI 606.
    OwnerSourceSingleTargetShaOrTrickAllMaterialsFromHandCanSha,
    /// The exact source-less Sha currently tracked by 寅君 fully resolved. ABI 606.
    DcyinjunTrackedShaResolved,
    /// The owner's own view-as card targeted a player carrying the requested skill pile. ABI 607.
    OwnerSourceOwnViewAsCardTargetedTargetSkillPile(u32),
    /// A living owner observes another player recover outside the owner's turn. ABI 608.
    OwnerAliveOtherRecoveredOutsideOwnerTurn,
    /// The owner ended their Discard phase after discarding at least one logical Sha. ABI 609.
    OwnerIsTargetDiscardedShaInDiscardPhase,
    /// The owner is dealing Sha damage to a currently healthy target. ABI 609.
    OwnerSourceShaDamageBeforeTargetHealthy,
    /// The owner entered End phase with the originating skill's public pile empty. ABI 610.
    OwnerPhaseEndOriginatingSkillPileEmpty,
    /// The owner is a current Sha target and the named public skill pile is nonempty. ABI 610.
    OwnerIsTargetShaSkillPileNonempty(u32),
    /// Another player's Play phase produced a discard while the named public pile is nonempty.
    /// ABI 610.
    OwnerAliveOtherPlayDiscardSkillPileNonempty(u32),
    /// The owner begins Prepare phase and at least one known other group has no member whose
    /// attack range contains the owner. ABI 611.
    OwnerPrepareGroupOutsideAttackRangeExists,
    /// A living owner observes a living Sha target at distance at most one and can discard an
    /// equipment card from hand or equipment. ABI 613.
    OwnerAliveShaTargetDistanceLeOneHasEquipCard,
    /// Owner is the current Sha target, its source is alive, and owner has a hand/equipment card. ABI 614.
    OwnerIsEventPlayerShaSourceAliveHandEquipNonempty,
    /// Living owner has at least one recorded living 负仇 source in this skill instance. ABI 614.
    OwnerAliveWdbFuchouPendingLiving,
    /// Another living player gained exactly one equipment card into hand outside that player's
    /// turn, and that equipment remains immediately usable. ABI 615.
    OwnerAliveOtherGainedSingleEquipToHandOutsideTurnUsable,
    /// 五谷丰登剩余牌刚以专用原因进入弃牌堆且仍可获得。ABI 616.
    OwnerAliveWuguRemainedInDiscard,
    /// Owner is one of at least two targets of a non-multitarget ordinary trick. ABI 616.
    OwnerIsTargetNonMultitargetTrickMultipleTargets,
    /// Owner used a Sha in their Play phase and at least one legal non-target remains when only
    /// ordinary distance is ignored. ABI 617.
    OwnerSourcePlayPhaseShaHasAdditionalTargetIgnoringDistance,
    /// The owner's exact Sha previously extended by 险袭 has fully resolved. ABI 617.
    OwnerSourceCardResolvedTrackedDddXianxi,
    /// Owner dealt card-caused damage to a living target that can receive at least one of the
    /// owner's Bingliang judgment cards. ABI 621.
    OwnerSourceCardDamageTargetCanReceiveOwnerBingliang,
    /// A living owner observes any discard batch that leaves an effective black basic card in
    /// the discard pile. ABI 622.
    OwnerAliveDiscardedBlackBasicToDiscard,
    /// Owner's previous 虚誉 activation is waiting for this owner's next resolved card. ABI 623.
    OwnerSourceCardResolvedDddXuyuPending,
    /// A 虚誉 debt is waiting to expire at the current turn boundary. ABI 623.
    DddXuyuPending,
    /// Another player completed their second Play-phase card while owner can give a card. ABI 623.
    OwnerAliveOtherSourceSecondPlayCardResolvedHandEquipNonempty,
    /// The player watched by 实荐 used their next card with at least one target. ABI 623.
    DddShijianWatchedSourceCardUsedTargeted,
    /// A 实荐 watch is waiting to expire at the current turn boundary. ABI 623.
    DddShijianPending,
    /// A living owner changed HP this turn and a turn-entered discard card remains. ABI 624.
    OwnerAliveHpChangedThisTurnEnteredDiscardAvailable,
    /// Another living player's turn begins while both them and owner have hand cards. ABI 625.
    OwnerAliveOtherPhaseBeginBothHandNonempty,
    /// Owner has at least one card in the 洞察 skill pile at PhaseEnd. ABI 625.
    OwnerAliveDddDongchaPileNonempty,
    /// Another living player's turn begins and owner has at least that player's HP worth of
    /// face-down hand cards. ABI 626.
    OwnerAliveOtherPhaseBeginUnshownHandGteHp,
    /// Owner is in their Play phase, HP category is unused, and the changed player's HP now
    /// equals owner's HP. ABI 627.
    OwnerPlayDddZhengjunHpAvailable,
    /// Owner is in their Play phase and a changed hand/equipment count now equals owner's
    /// corresponding count in an unused category. ABI 627.
    OwnerPlayDddZhengjunCardsMovedAvailable,
    /// Another living player's Play phase begins while owner can discard a hand/equipment card.
    /// ABI 628.
    OwnerAliveOtherPhasePlayDddChashiAvailable,
    /// The watched Play-phase player used a card matching 察势's discarded suit or category.
    /// ABI 628.
    DddChashiMatchingCard,
    /// The living owner's 齐策 persistent lock is currently clear. ABI 628.
    OwnerAliveDddQiceReady,
    /// The Play phase watched by 察势 is ending, so its pending comparison expires. ABI 628.
    DddChashiPendingPhasePlayEnd,
    /// At global phase end, exactly two living players used/responded this turn: owner and one
    /// other male or double-sex player. ABI 629.
    OwnerAliveDddZhilianAvailable,
    /// Owner just finished using or responding with a basic card, still has hand/water cards,
    /// and 浮世 has not fired this turn. ABI 630.
    OwnerAliveSourceBasicDddFushiAvailable,
    /// Owner is entering Draw/Discard and has an unused legal proactive ordinary trick. ABI 631.
    OwnerAliveDddFenjiAvailable,
    /// First observed phase of a round with at least one prior-round 丰政 user. ABI 632.
    OwnerAliveDddFengzhengRoundPending,
    /// A discard-entry batch contains a physical rank matching the owner's 玉律. ABI 632.
    OwnerAliveDddYulvDiscardMatches,
    /// First observed phase of each round for 管宁「游学」. ABI 634.
    OwnerAliveDddYouxueRoundPending,
    /// A marked player's turn ended with an eligible physical use still discarded. ABI 634.
    OwnerAliveDddChengjingAvailable,
    /// Another player's Play phase begins while 曹爽 can give a hand card. ABI 635.
    OwnerAliveDddZhuansheGift,
    /// Watched turn actor used a recorded logical card name. ABI 635.
    OwnerAliveDddZhuansheCardUsed,
    /// Watched actor's turn ends. ABI 635.
    OwnerAliveDddZhuanshePhaseEnd,
    /// 曹爽 has no hand before any recovery. ABI 635.
    OwnerAliveDddWeiqiu,
    /// Owner resolved a red card with recoverable physical materials. ABI 636.
    OwnerAliveDddLianerAvailable,
    /// A turn ended while 涟洏's rank ceiling is active. ABI 636.
    OwnerAliveDddLianerCeiling,
    /// Last globally used/responded card was black. ABI 636.
    OwnerAliveDddAnzhiAvailable,
    /// Record the latest global used/responded card color. ABI 636.
    OwnerAliveDddAnzhiTracking,
    /// Armed target's first black targeting this turn. ABI 636.
    OwnerAliveDddAnzhiTargetBlack,
    /// Armed 暗织 turn ended. ABI 636.
    OwnerAliveDddAnzhiTurnEnd,
    /// A new turn restores 失兴's “no trick used” branch and its derived skills changed. ABI 637.
    OwnerAliveDddShixingTurnRefresh,
    /// An HP transition changed whether an undamaged female exists. ABI 637.
    OwnerAliveDddShixingFemaleChanged,
    /// The first trick of the current turn removes 失兴's 酒诗 branch. ABI 637.
    OwnerAliveDddShixingFirstTrick,
    /// 刘宏 begins End with fewer linked players than living 群 characters. ABI 637.
    OwnerAliveDddDangguAvailable,
    /// A 群 character died while the living owner has active 覆宗. ABI 637.
    OwnerAliveDddFuzongDeath,
    /// The damaged skill owner is alive and currently face-down. ABI 637.
    OwnerIsTargetFaceDown,
    /// A turn ended while 夏侯玄 has a pending 玄论 settlement. ABI 638.
    OwnerAliveDddXuanlunPending,
    /// Another character is about to recover while unused 劫行 can replace it. ABI 639.
    OwnerAliveDddJiexingRecoverOncePerGame,
    /// Another character used equipment whose physical card can enter the owner's slot. ABI 639.
    OwnerAliveDddJiexingEquipOncePerGame,
    /// The owner recovered after 劫行 was consumed. ABI 639.
    OwnerAliveDddJiexingRecoveryReset,
    /// The owner finished using equipment after 劫行 was consumed. ABI 639.
    OwnerAliveDddJiexingEquipReset,
    /// The owner's Prepare phase has at least one legal 拜泪 target. ABI 639.
    OwnerAliveDddBaileiAvailable,
    /// The owner used 五谷丰登 through 仝御. ABI 640.
    OwnerSourceOwnViewAsDddTongyuUsed,
    /// One target was named by the owner's 仝御 五谷丰登. ABI 640.
    OwnerSourceOwnViewAsDddTongyuTargeted,
    /// 仝御 left revealed cards in discard and at least one rejecting target survives. ABI 640.
    OwnerAliveDddTongyuRemained,
    /// Card-name-scoped family (Phase 3): the owner used a card whose logical kind/suit matches
    /// the selector. Fires on `card_used` / `card_responded`. Unlocks card_name_scoped placeholders.
    OwnerSourceCardName(crate::query::CardSelector),
    /// Card-name-scoped family (Phase 3): the owner is the target of a card whose logical kind/suit
    /// matches the selector. Fires on `card_targeted`.
    OwnerIsTargetCardName(crate::query::CardSelector),
    /// Card-name-scoped family (Phase 3): the owner took damage whose originating card matches the
    /// selector. Fires on `damage`.
    OwnerIsTargetCardDamage(crate::query::CardSelector),
    /// Skill-provenance family (Phase 3): the event's card originated from a view-as skill with the
    /// given definition id (analogue of upstream `event.skill == "X"`). Parameter is the skill def id.
    EventViewAsSkill(u32),
    /// Skill-provenance name family (Phase 3): the event's card originated from a view-as skill whose
    /// content-metadata name matches (analogue of upstream `event.skill == "X_backup"` for dynamic
    /// helpers without a static def id). Parameter is the skill name.
    EventViewAsSkillName(String),
    /// The owner is the damage target and the damage is iron-chain-propagated
    /// (`damage_card_flag::CHAIN_PROPAGATED`). Implements 红棉百花袍 (hongmianbaihuapao_skill):
    /// `owner_is_target_chain_damage_linked`.
    OwnerIsTargetChainDamageLinked,
    /// The owner is the damage target, is wounded, and the damage comes from a Sha card.
    /// Implements 得势 (sp2/dcdeshi): `owner_is_target_wounded_sha`.
    OwnerIsTargetWoundedSha,
    /// The owner is at full health and the draw phase is not fixed (`!draw_phase_is_fixed()`).
    /// Implements 螭纹玉佩 (sp/chiwenyupei) draw branch: `owner_is_target_healthy_draw_not_fixed`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsTargetHealthyDrawNotFixed,
    /// The owner used a Sha or a black ordinary trick and has the configured stable mark.
    /// Implements 乱战·增 (sp/reluanzhan_add): `owner_source_sha_or_black_trick_mark_positive_<mark>`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceShaOrBlackTrickMarkPositive(u32),
    /// The living owner is targeted by another player's black card.
    /// Implements 协穆·指定 (sp/xiemu2): `owner_alive_targeted_by_other_black_card`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveTargetedByOtherBlackCard,
    /// The living owner observes another player being killed and is not the killer.
    /// Implements 起乱 draw-on-death (sp/qiluan): `owner_alive_other_source_death`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherSourceDeath,
    /// The owner is the event player during their own play phase and has no Sha in hand.
    /// Implements 摧怀/咄咄 (sp/cihuai, sp/oldcihuai): `owner_is_target_play_phase_hand_no_sha`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsTargetPlayPhaseHandNoSha,
    /// The living owner observes another player's end phase and that event player is not
    /// in the owner's attack range.
    /// Implements 礼下·OL (sp/ollixia): `owner_alive_other_phase_end_player_outside_owner_range`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherPhaseEndPlayerOutsideOwnerRange,
    /// The living owner used a Sha and the event player (a living target) has a
    /// discardable equipment card.
    /// Implements 风石 (sp/zfengshi): `owner_source_sha_target_alive_has_discardable_equip`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceShaTargetAliveHasDiscardableEquip,
    /// The living owner used a card and is among its targets.
    /// Implements 成都 (sp/pxchengdu): `owner_source_card_used_targets_self`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceCardUsedTargetsSelf,
    /// The living owner (current phase) targets a healthy other player with a card.
    /// Implements 廷机·OL (sp/oltingji): `owner_source_current_phase_card_targeted_other_healthy`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceCurrentPhaseCardTargetedOtherHealthy,
    /// The living owner just gained the skill with the given def id.
    /// Implements 州郡 skills (sp/px*): `owner_skill_added_self_<def>`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSkillAddedSelf(u32),
    /// The living owner used a card whose effective suit is the first of that suit this turn.
    /// Implements 文喜 (sp/pxwenxi): `owner_source_first_card_suit_this_turn`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceFirstCardSuitThisTurn,
    /// The living owner lost hand cards and their hand is now empty.
    /// Implements 平原 (sp/pxpingyuan): `owner_lost_hand_now_empty`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerLostHandNowEmpty,
    /// The living owner lost a physical spade 2-9 card from their hand.
    /// Implements 巨鹿 (sp/pxjulu): `owner_lost_spade_two_to_nine`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerLostSpadeTwoToNine,
    /// The living owner is either participant in a completed hand comparison.
    /// Implements 庐江 (sp/pxlujiang win/lose sub-skills):
    /// `owner_hand_compare_finished_participant`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerHandCompareFinishedParticipant,
    /// Owner is alive, the event player is alive, and the event player is either the owner
    /// or a member of `clan`. Implements 保族 (clan/clanbaozu):
    /// `owner_alive_event_player_self_or_clan_<CLAN>`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerSelfOrClan(String),
    /// The skill owner is the player whose card a `CardResponded` event answered
    /// (`ctx.respond_to == owner`). Implements 矢仄 (clan/clanshize):
    /// `owner_is_respond_to`. Note `ctx.source` for respond events is the RESPONDER.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsRespondTo,
    /// Owner took Sha-card damage (`owner_is_player && card_is_sha` on Damage).
    /// Mirror of `OwnerSourceSha` for the taken side. Implements 极寒冻动·贰
    /// (shahai card/swd); unlocks the「受击杀伤害」family
    /// (xingtigaizao_equip2 / gudonggeng / jinjilengdong_equip2).
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerTargetSha,
    /// The living owner is targeted by another player's red card. Mirror of
    /// `OwnerAliveTargetedByOtherBlackCard` (协穆·指定). Implements the
    /// per-color targeted family (郊夏 / 芳华, shahai):
    /// `owner_alive_targeted_by_other_red_card`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveTargetedByOtherRedCard,
    /// The owner is among the participants of a debate that just opened
    /// (DebateStarted window, ADR 0644 P3). Implements 凶暴 (jsrgxiongbao):
    /// `owner_debate_participant`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerDebateParticipant,
    /// Cards moved into the discard pile and at least one is a shadow card
    /// (jsrg funi③). `owner_cards_moved_contain_ying`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerCardsMovedContainYing,
    /// The skill owner has the Lord identity (identity mode).
    /// Implements 出师 (jsrgchushi) gate: `owner_is_lord`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsLord,
    /// A living owner observes a logical Sha targeting a player other than
    /// themselves (daimou 殆谋「他人」分支). `owner_alive_other_targeted_by_sha`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherTargetedBySha,
    /// A living owner is targeted by another player's Sha and holds at least
    /// one tagged xumou (scheme) card in their judge zone (daimou「我」分支).
    /// `owner_alive_targeted_by_other_sha_judge_xumou`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveTargetedByOtherShaJudgeXumou,
    /// The event player (turn owner at phase end) holds the Lord identity
    /// (jsrgzhenglve①「一号位的回合结束」identity-mode rendering).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerLord,
    /// Damage whose source is the living owner and whose target carries a
    /// positive count of the given mark, once per turn (jsrgzhenglve③).
    /// Filter string:
    /// `owner_source_damage_target_mark_positive_<def>_once`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceDamageTargetMarkPositiveOnce(u32),
    /// Own prepare phase while more than two living players carry the given
    /// mark (jsrghuilie awakening gate). Filter string:
    /// `owner_alive_marked_alive_above_two_<def>`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveMarkedAliveAboveTwo(u32),
    /// Any living player carries a positive count of the given mark
    /// (jsrgpingrong gate). Filter string:
    /// `owner_alive_someone_mark_positive_<def>`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveSomeoneMarkPositive(u32),
    /// A living owner holds a non-empty City pile for the given skill def and
    /// some other player lost cards this turn (jsrgshacheng②).
    /// Filter string: `owner_alive_sha_resolved_city_<def>`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveShaResolvedCity(u32),
    /// Phase end of the Lord, who dealt no damage this turn (jsrgzhenglve①
    /// 「至多两名」分支). `owner_alive_event_player_lord_no_damage`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerLordNoDamage,
    /// Phase end of the Lord, who dealt damage this turn (jsrgzhenglve① single
    /// grant branch). `owner_alive_event_player_lord_damaged`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerLordDamaged,
    /// Own turn ending with all four suits used this turn (refresh botu 博图).
    /// `owner_alive_turn_used_all_suits`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveTurnUsedAllSuits,
    /// Own turn ending while the turn player carries the given mark and dealt
    /// no damage this turn (xianding guili 归离绑定回收).
    /// `owner_alive_event_target_mark_positive_<mark>_no_damage`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventTargetMarkPositiveNoDamage(u32),
    /// A card-move of at least three cards into the discard pile
    /// (diy nsjihui 集计). `cards_moved_discard_at_least_three`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardsMovedDiscardAtLeastThree,
    /// Own turn ending while the turn player carries the given mark
    /// (tw twjieqiu end 子技判定). `owner_alive_event_target_mark_positive_<mark>`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventTargetMarkPositive(u32),
    /// A living owner observes a DamageBefore event whose victim holds a
    /// positive count of the given mark (kuanshi 宽释 protection arm, batch 244).
    /// Filter string: `owner_alive_damage_before_event_target_mark_positive_<mark>`.
    OwnerAliveDamageBeforeEventTargetMarkPositive(u32),
    /// A living owner observes a PhaseBegin event whose turn player holds a
    /// positive count of the given mark (kuanshi 宽释 expiry arm, batch 244).
    /// Filter string: `owner_alive_phase_begin_event_target_mark_positive_<mark>`.
    OwnerAlivePhaseBeginEventTargetMarkPositive(u32),
    /// A living owner observes a cards-moved-to-discard event where at least
    /// one discarded card matches the suit and rank of the first card in the
    /// owner's own skill pile for the given pile def (biaozhao2 表召·应,
    /// batch 259). Filter string:
    /// `owner_alive_discard_matches_own_pile_card_<pile_def>`.
    OwnerAliveDiscardMatchesOwnPileCard(u32),
    /// A cards-moved event moved cards INTO an expansion (skill-pile) zone
    /// (reliangyin 良姻① draw arm, batch 268). Filter string:
    /// `cards_moved_into_expansion`.
    CardsMovedIntoExpansion,
    /// A cards-moved event moved cards OUT OF an expansion (skill-pile) zone
    /// (reliangyin 良姻② discard arm, batch 268). Filter string:
    /// `cards_moved_out_of_expansion`.
    CardsMovedOutOfExpansion,
    /// Owner uses a card while holding no card with the given gaintag
    /// (natuxing 兴「手中无锁定兴」门). `owner_source_hand_gaintag_absent_<tag>`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceHandGaintagAbsent(u32),
    /// Own play phase ends with at least one card used this phase
    /// (offline zylianji 连计). `owner_alive_play_end_used_categories_positive`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAlivePlayEndUsedCategoriesPositive,
    /// The living owner gains cards during their own play phase while wounded
    /// (collab haoxue 好学回复臂). `owner_gained_cards_during_play_phase_wounded`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerGainedCardsDuringPlayPhaseWounded,
    /// The living owner discards hand/equip cards to the discard pile outside their own
    /// play phase (collab haoxue 好学流失臂). `owner_discarded_outside_play_phase`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerDiscardedOutsidePlayPhase,
    /// A multi-target (target count > 1) sha or ordinary trick is used with the living
    /// owner among its declared targets (sp danlao 啖酪).
    /// `owner_in_multitarget_sha_or_trick_targets`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerInMultitargetShaOrTrickTargets,
    /// The living owner becomes the target of a Nanman invasion (sp xinfu_zhennan 镇南).
    /// `owner_targeted_by_nanman`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerTargetedByNanman,
    /// Damage-before: the living owner is the damage target, the source is alive, and
    /// the source's HP and hand count both differ from the owner's (sp2 yinyi 隐翼;
    /// upstream linked-nature exclusion not modelled).
    /// `owner_targeted_source_hp_hand_differ_once_per_turn`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerTargetedSourceHpHandDifferOncePerTurn,
    /// Card-resolved: the living owner used an ordinary trick which has finished
    /// resolving during their own play phase, once per turn (sp2 xinfu_yisuan 翼算;
    /// gaining the discarded cards is the effect's business, not the filter's).
    /// `owner_source_trick_resolved_own_play_phase_once_per_turn`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceTrickResolvedOwnPlayPhaseOncePerTurn,
    /// A living owner observes another living player enter dying while the owner holds an
    /// equip-kind card in hand or equipment zone (onlyOL olrenxin 仁心; the upstream
    /// equip-card discard cost is approximated by an optional confirm).
    /// `owner_alive_other_dying_has_equip_card`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherDyingHasEquipCard,
    /// A living owner observes damage dealt by a Nanman invasion card, whoever the
    /// target (xianding mansi 蛮肆 main arm).
    /// `owner_alive_damage_card_nanman`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveDamageCardNanman,
    /// A black card enters or leaves the living owner's equipment or judgment zone
    /// (sp2 liangjue 狼灭). Both directions are scoped to the owner's own zones.
    /// `owner_black_card_moved_equip_judge`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerBlackCardMovedEquipJudge,
    /// A card of the declared suit moves into the discard pile from the living owner's
    /// zones while the owner lacks the declared falu mark (sp xinfu_falu 法箓 suit
    /// collection; suit encoded in the filter token, mark def carried by the skill's
    /// DSL add_mark).
    /// `owner_discard_suit_X_no_own_mark_<mark>`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerDiscardSuitNoOwnFaluMark(u8, u32),
    /// The owner (event player) just completed a turn-over in either direction
    /// (refresh shebian 涉编 upstream `player: "turnOverEnd"`).
    /// `owner_turned_over`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerTurnedOver,
    /// Another living wei-group player recovers HP during their own turn's phases
    /// (xianding zhushi 助势 主公技, upstream `global: "recoverEnd"` with
    /// `event.player.group == "wei" && event.player == currentPhase`).
    /// `owner_alive_other_wei_recovery_own_phase`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherWeiRecoveryOwnPhaseOncePerTurn,
    /// Any living player uses 【决斗】 or a red 【杀】 (offline ylyg_jiang 激:
    /// global useCardToPlayered with first target; owner involvement not required).
    /// `owner_alive_global_juedou_or_red_sha_used`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveGlobalJuedouOrRedShaUsed,
    /// Another living player (user != owner) uses 【决斗】 or a red 【杀】.
    /// `owner_alive_other_global_juedou_or_red_sha_used`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherGlobalJuedouOrRedShaUsed,
    /// The owner's named skill pile (武将牌上, keyed by a skill def literal —
    /// possibly a sibling skill) holds at least one card.
    /// `owner_skill_pile_nonempty_<def>` (expansion S1-B family gate).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSkillPileNonempty(u32),
}

/// Handlers: native primitives or Rule VM bytecode (Phase 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerHandler {
    /// Draw `count` cards for the skill owner.
    DrawSelf {
        /// Cards to draw.
        count: u32,
    },
    /// Emit a rule note (test / marker).
    EmitNote {
        /// Code.
        code: u32,
    },
    /// Recover HP for owner.
    RecoverSelf {
        /// Amount.
        amount: u32,
    },
    /// Run pre-verified bytecode module (same VM path for official & community).
    RunVm {
        /// Module name / key in the content registry.
        module: String,
        /// Entry PC.
        entry_pc: u32,
    },
}

/// One static trigger definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerDefinition {
    /// Definition id.
    pub id: TriggerDefId,
    /// Skill definition this trigger belongs to.
    pub skill_def: SkillDefId,
    /// Event that wakes this trigger.
    pub event: EventKind,
    /// Timing band.
    pub timing: TimingBand,
    /// Priority (lower first).
    pub priority: i32,
    /// Optional triggers may later open ChoiceRequest; mandatory triggers auto-run.
    pub optional: bool,
    /// Player who confirms an optional trigger.
    pub choice_actor: TriggerChoiceActor,
    /// Filter.
    pub filter: TriggerFilter,
    /// Max fires of this def+instance per resolution segment (anti-loop).
    pub max_per_segment: u32,
    /// Handler.
    pub handler: TriggerHandler,
}

/// Bootstrap skill def: 奸雄 — after taking damage, draw 1.
pub const BOOTSTRAP_JIANXIONG: u32 = 1;
/// Bootstrap skill def: 马术 — distance from owner -1 (modifier, no trigger).
pub const BOOTSTRAP_MASHU: u32 = 2;
/// Bootstrap skill def: 奇策 placeholder range +1 (modifier).
pub const BOOTSTRAP_QICAO: u32 = 3;

/// Bootstrap trigger table (sorted by id).
pub fn bootstrap_trigger_defs() -> Vec<TriggerDefinition> {
    vec![TriggerDefinition {
        id: TriggerDefId(1),
        skill_def: SkillDefId::new(BOOTSTRAP_JIANXIONG),
        event: EventKind::Damage,
        timing: TimingBand::Normal,
        priority: 0,
        optional: false,
        choice_actor: TriggerChoiceActor::Owner,
        filter: TriggerFilter::OwnerIsEventPlayer,
        max_per_segment: 8,
        handler: TriggerHandler::DrawSelf { count: 1 },
    }]
}

/// Lookup defs for a skill definition.
pub fn defs_for_skill(skill_def: SkillDefId) -> Vec<TriggerDefinition> {
    bootstrap_trigger_defs()
        .into_iter()
        .filter(|d| d.skill_def == skill_def)
        .collect()
}

/// All bootstrap defs for an event kind (content index preview).
#[allow(dead_code)]
pub fn defs_for_event(event: EventKind) -> Vec<TriggerDefinition> {
    bootstrap_trigger_defs()
        .into_iter()
        .filter(|d| d.event == event)
        .collect()
}
