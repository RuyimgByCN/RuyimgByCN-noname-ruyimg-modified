//! Ordered TriggerFilter variants, starting with `OwnerIsEventPlayer`.

macro_rules! variants {
    ([$($tail:ident),*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail),*] [
            $($variants)*
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
        ] }
    };
}
pub(super) use variants;
