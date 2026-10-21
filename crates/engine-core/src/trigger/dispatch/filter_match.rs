//! Trigger operations for `filter_match` and related helpers.

use super::*;

pub(crate) fn filter_match(
    state: &crate::state::GameState,
    active: &ActiveTrigger,
    ctx: &TriggerContext,
    view_as_skill: u32,
    view_as_card: u32,
    history: Option<&TriggerHistorySnapshot>,
) -> bool {
    let Some(def) = state.content.find_def(active.trigger_def) else {
        return false;
    };
    match &def.filter {
        TriggerFilter::Always
        | TriggerFilter::OwnerIsEventPlayerOtherHandNonempty
        | TriggerFilter::OwnerIsEventPlayerPositiveDrawOtherHandNonempty
        | TriggerFilter::OwnerSourceFirstTargetBlackCardAnyTargetOtherDiscardableHandEquip
        | TriggerFilter::OwnerSourceDirectShaDamageTargetArmorOrMountCardGainable
        | TriggerFilter::OwnerAliveEventPlayerLostArmorOrMountHasOriginatingSkillPile
        | TriggerFilter::OwnerSourcePlayCardMatchesRememberedCategory
        | TriggerFilter::OwnerAliveEventPlayerHandNonemptyOncePerRound
        | TriggerFilter::OwnerSourceBoundCategoryFirstThreePlayPhase
        | TriggerFilter::OwnerPhaseGiftTargetIsEventSource
        | TriggerFilter::OwnerPhaseGiftTargetIsEventPlayer
        | TriggerFilter::OwnerSourceShaResolvedHasGainableCardsAndRecipient
        | TriggerFilter::OwnerAliveOtherSourceShaResolvedNotTargetGainableOncePerTurn
        | TriggerFilter::OwnerIsEventPlayerTiedMaxHand
        | TriggerFilter::OwnerIsEventPlayerTiedMaxHp
        | TriggerFilter::OwnerIsEventSourceOrPlayerXinyaomingAvailable
        | TriggerFilter::OwnerIsEventPlayerTiedMaxEquipNonempty
        | TriggerFilter::OwnerIsEventPlayerSkippedPhaseCountPositive
        | TriggerFilter::OwnerDiscardedHasHandOncePerPhase
        | TriggerFilter::OwnerPhaseEndLostCards
        | TriggerFilter::OwnerAliveSecondGlobalDying
        | TriggerFilter::OwnerIsTargetSourceHpGtOwnerOrSourceDamagedOwner
        | TriggerFilter::OwnerIsTargetOwnerHpGtSourceOrSourceDamagedOwner
        | TriggerFilter::OwnerIsTargetSanshouTypeTracked
        | TriggerFilter::OwnerSourceInTurnTargetUnharmedCardActionsGeTargetHp
        | TriggerFilter::OwnerUsedAndWasTargetedThisTurnInRange
        | TriggerFilter::OwnerAliveLostHandThisTurn
        | TriggerFilter::OwnerAlivePhaseEndLostCardsAtLeastTwo
        | TriggerFilter::OwnerSourceFirstShaThisPlayPhase
        | TriggerFilter::OwnerSourceDamageTargetDistanceAtMostOneEmptyEquipSlotUnused
        | TriggerFilter::OwnerSourceFirstShaTargetThisTurn
        | TriggerFilter::OwnerAliveOtherPhasePlayEndTwzhengjianCondition
        | TriggerFilter::OwnerSourceShaTargetTwzhenxiAvailableOncePerTurn
        | TriggerFilter::OwnerSourceFirstTargetDamageCardOncePerTurn
        | TriggerFilter::OwnerOutsideTurnDiscardedRedCardsInDiscard
        | TriggerFilter::OwnerAliveOtherTwkujianUsedOrResponded
        | TriggerFilter::OwnerAliveOtherTwkujianLost
        | TriggerFilter::OwnerAliveTwruilianRoundUnprompted
        | TriggerFilter::OwnerAliveTwruilianSelectedTargetDiscarded
        | TriggerFilter::OwnerAliveTwruilianSelectedTargetTurnEnded
        | TriggerFilter::OwnerAliveLostWeaponOutsideTurn
        | TriggerFilter::OwnerAliveOtherPhasePrepareCanCompareOwner
        | TriggerFilter::OwnerTargetPhysicalShaSourceHpGtOwner
        | TriggerFilter::OwnerSourceOtherTargetShaOrOrdinaryTrickTwguoyiCondition
        | TriggerFilter::OwnerAlivePhaseEndCausedTwoDistinctDying
        | TriggerFilter::OwnerGainedEventCardsFromOtherPersonalZone
        | TriggerFilter::OwnerAliveOtherSingleTargetUsedTrackedTwyizhu
        | TriggerFilter::OwnerTrackedTwyizhuEnteredDiscard
        | TriggerFilter::OwnerAliveOtherShaResolvedTrackedSkillCardDealtDamage
        | TriggerFilter::OwnerAliveOtherShaResolvedTrackedSkillCardNoDamage
        | TriggerFilter::OwnerTrackedSkillCardLeftHandNotUseOrRespond
        | TriggerFilter::OwnerAliveShaUsedBySelfOrInRangeDiscardableHandEquip
        | TriggerFilter::OwnerAliveTrackedCuijinShaResolvedNoDamage
        | TriggerFilter::OwnerAliveTrackedCuijinShaResolvedDealtDamage
        | TriggerFilter::OwnerAliveTrackedTwqingkouDuelResolved
        | TriggerFilter::OwnerAliveTwchenxunRoundUnprompted
        | TriggerFilter::OwnerAliveTrackedTwchenxunDuelTargetDamaged
        | TriggerFilter::OwnerAliveTrackedTwchenxunDuelTargetNotDamaged
        | TriggerFilter::OwnerAliveTrackedTwchenxunCardLeftProcessing
        | TriggerFilter::OwnerAliveOtherTwgonghuanDamageBeforeHpLteOwnerUnused
        | TriggerFilter::OwnerPhaseBeginPlayableSkillPileNonempty
        | TriggerFilter::OwnerUsedPlayableSkillPileCardUnused
        | TriggerFilter::OwnerTargetDamageBeforePlayableSkillPilesEmpty
        | TriggerFilter::OwnerAliveDamageBeforeTargetMinHpUnused
        | TriggerFilter::OwnerAliveGlobalPhaseEnd
        | TriggerFilter::OwnerAliveOtherPhaseEndMobiledanshou
        | TriggerFilter::OwnerAliveOtherPhaseEndNotDamagedOwnerThisTurn
        | TriggerFilter::OwnerAliveOtherPhaseEndUsedCardOnOwnerUndamagedThisTurn
        | TriggerFilter::OwnerAliveGlobalPhaseEndLostCardsTargetOtherAlive
        | TriggerFilter::OwnerAliveGlobalPhaseEndDiscardedCardsAvailable
        | TriggerFilter::OwnerSourceActiveDczhaowenTrackedRed
        | TriggerFilter::OwnerIsTargetEventCardJiu
        | TriggerFilter::OwnerIsTargetOtherSourceBlackDcjiudunAvailable
        | TriggerFilter::OwnerAlivePhaseEndUsedCardNoDamage
        | TriggerFilter::OwnerCardsEnteredDiscardNotUseUnused
        | TriggerFilter::OwnerIsEventPlayerDiscardedAtLeastTwoInDiscardPhase
        | TriggerFilter::OwnerAliveOtherDiscardedInDiscardPhase
        | TriggerFilter::OwnerAliveOtherPlayPhaseZhengsuMissingInRange
        | TriggerFilter::OwnerIsTargetMissingZhengsu
        | TriggerFilter::OwnerAliveOtherWuPlayPhaseCanCompareOwner
        | TriggerFilter::OwnerIsTargetMultiTargetTrickHasHandEquip
        | TriggerFilter::OwnerIsTargetFeijunNewTarget
        | TriggerFilter::OwnerSourceShaDamageTargetAliveBothHand
        | TriggerFilter::OwnerAliveOtherNanmanCardsDiscard
        | TriggerFilter::OwnerAliveSkillBoundTargetIsEventSource
        | TriggerFilter::OwnerLostCardsOutsideTurn
        | TriggerFilter::OwnerPhaseBeginSkillPileGteThree(..)
        | TriggerFilter::OwnerPhaseBeginHandEmpty
        | TriggerFilter::OwnerRespondedShan
        | TriggerFilter::OwnerAliveOtherQunPlayPhaseCanGiveOwner
        | TriggerFilter::OwnerAlivePlayPhaseActorHandNonemptyPotjiejieUnused
        | TriggerFilter::OwnerSourceShaDamageBeforePotyinzhanCondition
        | TriggerFilter::OwnerSourceDamageWithinOne
        | TriggerFilter::OwnerGainedCardsPotzhouyu
        | TriggerFilter::OwnerSourceFirstTargetHasHandPotyanhui
        | TriggerFilter::OwnerAliveFireDamagePotfentao
        | TriggerFilter::OwnerAlivePotfentaoPendingDamage => filters_always::filter_match(
            state,
            active,
            ctx,
            view_as_skill,
            view_as_card,
            history,
            &def,
        ),
        TriggerFilter::OwnerAliveTurnEndedRehengjiangPending
        | TriggerFilter::OwnerAlivePhasePlayEndZhengbiPending
        | TriggerFilter::OwnerIsTargetSourceHandNonemptyPerDamagePoint
        | TriggerFilter::OwnerSourceQiaojianExtraAvailable
        | TriggerFilter::OwnerXichaWatchedSourceCardUsed
        | TriggerFilter::OwnerQiaojianTrackedCardLeftHand
        | TriggerFilter::OwnerAliveDamageTargetSelfOrInRangeOtherSourceNumericCard
        | TriggerFilter::OwnerSourceWoundedExistsOncePerTurn
        | TriggerFilter::OwnerSourceFirstDamageCardTargetHandGteOwner
        | TriggerFilter::OwnerAliveNanmanResolvedTargetDeadOrDamagedGtThree
        | TriggerFilter::OwnerLostNonDamageCardPottuntianChargeNotFull
        | TriggerFilter::OwnerSourceShaResolvedTrackedPotceduanDealtDamage
        | TriggerFilter::OwnerAliveShaEnteredDiscardPotzhanlieAvailable
        | TriggerFilter::OwnerSourceShaResolvedTrackedPotzhanlieDraw
        | TriggerFilter::OwnerIsTargetDiscardPhaseOweLimitOtherQun
        | TriggerFilter::OwnerSourceSingleTargetShaOrBlackOrdinaryTrickCurrentPlayPhaseUnused
        | TriggerFilter::OwnerIsEventPlayerMaleSourceHasDiscardableEquipment
        | TriggerFilter::OwnerAliveOtherTurnEndedNoCardTargetedOther
        | TriggerFilter::OwnerAliveOtherTurnEndedNoDamageOther
        | TriggerFilter::OwnerAliveOtherTurnEndedCardTargetedOtherNoDamageOther
        | TriggerFilter::OwnerAlivePhaseTargetedHpChangedCanUseWanjian
        | TriggerFilter::OwnerAlivePhaseTargetedHpUnchangedCanUseJuedou
        | TriggerFilter::OwnerSourceShaHasWeapon
        | TriggerFilter::OwnerSourceShaDirectDamageBefore
        | TriggerFilter::OwnerSourceOtherDamageBefore
        | TriggerFilter::OwnerSourceOtherDamageBeforeOncePerTurn
        | TriggerFilter::OwnerSourceTargetEarlierSeatHandBelowTargetMaxHpOncePerTurn
        | TriggerFilter::OwnerSourceOtherDirectDamageBefore
        | TriggerFilter::OwnerIsTargetDiscardableHandEquip
        | TriggerFilter::OwnerSourceOrTargetHandCountDiffersOncePerTurn
        | TriggerFilter::OwnerIsEventPlayerAnyAliveHandNonempty
        | TriggerFilter::OwnerIsEventPlayerAnyAliveHandEmpty
        | TriggerFilter::OwnerIsEventPlayerOtherHandLteOwner
        | TriggerFilter::OwnerIsEventPlayerDrawReshuffleZeroWounded
        | TriggerFilter::OwnerIsEventPlayerDrawReshufflePositive
        | TriggerFilter::OwnerHandCompareParticipant
        | TriggerFilter::OwnerHandCompareFinishedParticipant
        | TriggerFilter::OwnerHandCompareParticipantHasSha
        | TriggerFilter::OwnerHandCompareParticipantResultCardInDiscard
        | TriggerFilter::OwnerDiscardedBlackBasicToDiscard
        | TriggerFilter::OwnerAliveDiscardedBlackBasicToDiscard
        | TriggerFilter::OwnerAliveOtherJudgePhaseInRangeHasJudgeCard
        | TriggerFilter::OwnerCardRespondedToByPlayerInRange
        | TriggerFilter::OwnerAliveFirstCardsMovedToHand
        | TriggerFilter::OwnerAliveNeighborGainedCardsOutsideOwnDrawOncePerTurn
        | TriggerFilter::OwnerDrawsDuringOwnPlayNotSelf
        | TriggerFilter::OwnerSourceNonEquipNoBasicHandHasTargets
        | TriggerFilter::OwnerEquipZoneChangedAtLeastTwoWounded
        | TriggerFilter::OwnerAliveEquipUsed
        | TriggerFilter::OwnerAliveEquipUsedSkillMarkBelowThree
        | TriggerFilter::OwnerAliveSourceAliveOncePerRound
        | TriggerFilter::OwnerAliveSourceAliveEventPlayerDyingOncePerRound
        | TriggerFilter::OwnerAliveOncePerRound
        | TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipGteTwoOncePerRound
        | TriggerFilter::OwnerIsEventPlayerOtherSourceAliveHandEquipNonemptyOldHuishengUnused
        | TriggerFilter::OwnerAliveOtherDyingOutsideTurnCanShaCurrent
        | TriggerFilter::OwnerSourceOldJiefanSha
        | TriggerFilter::OwnerSourceCurrentTurnFirstTargetNonEquip
        | TriggerFilter::OwnerSourceFaceDownViewAsChenweiAvailable
        | TriggerFilter::OwnerAlive
        | TriggerFilter::OwnerAliveKilledThisTurn
        | TriggerFilter::OwnerAliveHandBelowAliveCountCappedSeven
        | TriggerFilter::OwnerAliveHandBelowAliveCountCappedEight
        | TriggerFilter::OwnerAliveDelayedTrickTargetWithoutPendingJudgeSkip
        | TriggerFilter::OwnerSourceShaOrJuedouSingleTargetFirstSameNameThisTurn
        | TriggerFilter::OwnerSourceShaOrJuedouSingleTargetHandNonempty
        | TriggerFilter::OwnerSourceShaDamageBeforeEndpointDiscardableHandEquip
        | TriggerFilter::OwnerLongdanViewAsFirstCounterpartHandNonempty
        | TriggerFilter::OwnerAliveEventPlayerAlive
        | TriggerFilter::OwnerAliveEventPlayerEquipEmpty
        | TriggerFilter::OwnerAliveHandHasSameKindAsEventCard
        | TriggerFilter::OwnerAliveHasSkillTaggedCards
        | TriggerFilter::OwnerAliveTaggedCardLostThisTurn
        | TriggerFilter::OwnerAliveEventPlayerFromEquipEmpty
        | TriggerFilter::OwnerAliveShaTargetNoOtherRedCardThisTurn
        | TriggerFilter::OwnerAliveEventPlayerHandEmpty
        | TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmpty
        | TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmptyOncePerTurn
        | TriggerFilter::OwnerAliveEventPlayerDistanceLeOwnerHp
        | TriggerFilter::OwnerAliveOtherNormalDamageTargetUnlinked
        | TriggerFilter::OwnerAliveEquipSourceMaxEquip
        | TriggerFilter::OwnerAliveOtherCurrentSourceRepeatsCardName
        | TriggerFilter::OwnerAliveNotEventPlayerNeitherUniqueMaxHand
        | TriggerFilter::OwnerAliveSourceTargetSameSex
        | TriggerFilter::OwnerAliveEventPlayerSameGroup
        | TriggerFilter::OwnerAliveEventPlayerSameGroupHandNonempty
        | TriggerFilter::OwnerTrackedJuedouOriginalTargetDamageRecoverableInRange
        | TriggerFilter::OwnerIsTargetPhaseEndHandNonemptyOtherHandNonempty
        | TriggerFilter::OwnerIsTargetOrdinaryTrickCilvAvailable
        | TriggerFilter::CilvTrackedCardDamageBefore
        | TriggerFilter::CilvTrackedCardResolved
        | TriggerFilter::OwnerDyingZhupeilanOncePerGame
        | TriggerFilter::OwnerAliveOtherPlayPhaseTiedMaxHandDcsbfuxiAvailable
        | TriggerFilter::OwnerPhaseEndDcsbhaoyiAvailable
        | TriggerFilter::OwnerAliveGlobalFireDamageDcsbronghuo
        | TriggerFilter::OwnerSourceFireShaOrHuogongDcsbronghuo
        | TriggerFilter::OwnerSourceCardResolvedHasOtherTargetDcsbyingmouOncePerTurn
        | TriggerFilter::DcsbyingmouTrackedCardResolved
        | TriggerFilter::OwnerHandTransferOtherDcsbmengmouBranchAvailable
        | TriggerFilter::DcsbmengmouTrackedShaDamage
        | TriggerFilter::DcsbmengmouTrackedShaResolved
        | TriggerFilter::OwnerIsTargetCardDamageBefore
        | TriggerFilter::OwnerSourceSingleTargetShaOrTrickAllMaterialsFromHandCanSha => {
            filters_owner_alive_turn_ended_rehengjiang_pending::filter_match(
                state,
                active,
                ctx,
                view_as_skill,
                view_as_card,
                history,
                &def,
            )
        }
        TriggerFilter::DcyinjunTrackedShaResolved
        | TriggerFilter::OwnerSourceOwnViewAsCardTargetedTargetSkillPile(..)
        | TriggerFilter::OwnerAliveOtherRecoveredOutsideOwnerTurn
        | TriggerFilter::OwnerIsTargetDiscardedShaInDiscardPhase
        | TriggerFilter::OwnerSourceShaDamageBeforeTargetHealthy
        | TriggerFilter::OwnerPhaseEndOriginatingSkillPileEmpty
        | TriggerFilter::OwnerIsTargetShaSkillPileNonempty(..)
        | TriggerFilter::OwnerAliveOtherPlayDiscardSkillPileNonempty(..)
        | TriggerFilter::OwnerPrepareGroupOutsideAttackRangeExists
        | TriggerFilter::OwnerAliveEventCardsNonempty
        | TriggerFilter::OwnerAliveTargetedByOther
        | TriggerFilter::OwnerAliveTargetedByOtherBlackCard
        | TriggerFilter::OwnerAliveTargetedByOtherRedCard
        | TriggerFilter::OwnerAliveOtherSourceDeath
        | TriggerFilter::OwnerIsTargetPlayPhaseHandNoSha
        | TriggerFilter::OwnerAliveOtherPhaseEndPlayerOutsideOwnerRange
        | TriggerFilter::OwnerAliveEventPlayerGroup(..)
        | TriggerFilter::OwnerIsEventPlayerNoClanLostHandThisTurn(..)
        | TriggerFilter::OwnerIsRespondTo
        | TriggerFilter::OwnerAliveEventPlayerSelfOrClan(..)
        | TriggerFilter::OwnerAliveOtherDamageSourceGroup(..)
        | TriggerFilter::OwnerDyingOtherTaoResponderGroup(..)
        | TriggerFilter::OwnerAliveOtherJudgmentBlackEventPlayerGroup(..)
        | TriggerFilter::OwnerWoundedNotEventSource
        | TriggerFilter::OwnerIsEventPlayerWounded
        | TriggerFilter::OwnerIsEventPlayerLinked
        | TriggerFilter::OwnerIsTargetChainDamageLinked
        | TriggerFilter::OwnerIsTargetWoundedSha
        | TriggerFilter::OwnerIsTargetHealthyDrawNotFixed
        | TriggerFilter::OwnerIsEventPlayerAliveWoundedOrGuimingWuExists
        | TriggerFilter::OwnerWoundedEventPlayerGroup(..)
        | TriggerFilter::OwnerAliveNotEventPlayer
        | TriggerFilter::OwnerAliveNotEventPlayerHandEquipNonempty
        | TriggerFilter::OwnerSourceShaOrOrdinaryTrickTargetWoundedHasDiscardableHandEquip
        | TriggerFilter::OwnerSourceShaOrOrdinaryTrickHasDiscardableHandEquip
        | TriggerFilter::OwnerSourceShaOrOrdinaryTrick
        | TriggerFilter::OwnerSourceWuxieUsed
        | TriggerFilter::OwnerSourceShaOncePerTurn
        | TriggerFilter::OwnerEquipZoneChanged
        | TriggerFilter::OwnerSourceShaRepeatsPreviousShaTargets
        | TriggerFilter::OwnerSourcePlayCardRepeatsPreviousTargets
        | TriggerFilter::OwnerIsEventPlayerHandEquipJudgeAboveHp
        | TriggerFilter::OwnerSourceJiuBoostedShaDamageSkillEnabled(..)
        | TriggerFilter::OwnerAliveNotEventPlayerSourceSameGroup
        | TriggerFilter::OwnerAliveNotEventPlayerEqualHand
        | TriggerFilter::OwnerAliveNotEventPlayerHandBelowHp
        | TriggerFilter::OwnerAliveNotEventPlayerNonnegativeHpGtEventPlayer
        | TriggerFilter::OwnerAliveCurrentTurnOtherEventPlayer
        | TriggerFilter::OwnerAliveCurrentTurnOtherEventPlayerFromHandEmpty
        | TriggerFilter::OwnerGainsMultipleOutsideOwnDraw
        | TriggerFilter::OwnerGainedOrLostAtLeastTwo
        | TriggerFilter::OwnerGainsOutsideTurnMarkBelowAliveOncePerTurn(..)
        | TriggerFilter::OwnerCurrentTurnOtherEquipmentLost
        | TriggerFilter::OwnerMarkPositive(..)
        | TriggerFilter::OwnerAliveDamagedEventPlayer
        | TriggerFilter::OwnerAliveEventPlayerOwnerOrFemale
        | TriggerFilter::OwnerAliveNotEventPlayerOncePerTurn
        | TriggerFilter::OwnerAliveCurrentOrTurnFiresBelowHp
        | TriggerFilter::OwnerAliveCardIsSha
        | TriggerFilter::OwnerAliveOtherRespondedShaGainable
        | TriggerFilter::OwnerAliveCardTargetsGtOne
        | TriggerFilter::OwnerAliveNonMultitargetTrickTargetsGtOne
        | TriggerFilter::OwnerAliveWoundedCardTargetsGtOne
        | TriggerFilter::OwnerAliveTargetedByOtherSingleCardDiscardable
        | TriggerFilter::OwnerAliveTargetedByTrickSingleOrDiscardable
        | TriggerFilter::OwnerAliveTargetedByOtherTrick
        | TriggerFilter::OwnerAliveOtherDiscardedBlackToDiscard
        | TriggerFilter::OwnerAliveTargetedByOtherMatchingEquipSuit
        | TriggerFilter::OwnerMinHp
        | TriggerFilter::OwnerIsEventPlayerNotMinHp
        | TriggerFilter::OwnerIsEventPlayerMinHand
        | TriggerFilter::OwnerIsEventPlayerNotMinHand
        | TriggerFilter::OwnerShaUsedAtLeastTwo
        | TriggerFilter::OwnerAliveUsedShaThisTurn
        | TriggerFilter::OwnerMaxHpGtOneHandOrEquipEmpty
        | TriggerFilter::OwnerIsEventPlayer
        | TriggerFilter::OwnerAliveIsEventPlayerCurrentPlayPhase
        | TriggerFilter::OwnerIsEventPlayerOtherAlive
        | TriggerFilter::OwnerIsEventPlayerNotUniqueMaxCurrentHp
        | TriggerFilter::OwnerIsEventPlayerHandNonempty
        | TriggerFilter::OwnerIsEventPlayerHpBelow(..)
        | TriggerFilter::OwnerIsEventPlayerMarkPositive(..)
        | TriggerFilter::OwnerSourceTargetMarkPositive(..)
        | TriggerFilter::OwnerIsEventPlayerHandBelowMaxHp
        | TriggerFilter::OwnerIsEventPlayerFirstHpChangeTurnUniqueTrickMajority
        | TriggerFilter::OwnerIsEventPlayerHandAboveHp
        | TriggerFilter::OwnerIsEventPlayerHandAboveHpNoLiubeiLord
        | TriggerFilter::OwnerIsEventPlayerHandAtLeastHpOtherAlive
        | TriggerFilter::OwnerIsEventPlayerLostCardsAtMostHpGainable
        | TriggerFilter::OwnerIsEventPlayerSourceMovableFieldCard
        | TriggerFilter::OwnerIsEventPlayerNoTurnDamage
        | TriggerFilter::OwnerIsEventPlayerTurnDamage
        | TriggerFilter::OwnerIsEventPlayerTurnDamageAtLeastThree
        | TriggerFilter::OwnerIsEventPlayerCardUsedAtLeastHp
        | TriggerFilter::OwnerIsEventPlayerPlayPhaseCardCategoryCountPositive
        | TriggerFilter::OwnerAlivePlayPhaseCardCategoryCountPositive
        | TriggerFilter::OwnerAlivePlayPhaseShaUserHasDiscardableHandEquip
        | TriggerFilter::OwnerIsEventPlayerActiveUsedAtLeastThisPlayPhase(..)
        | TriggerFilter::OwnerIsEventPlayerUseResponseDiscardedAtLeastHp
        | TriggerFilter::OwnerIsEventPlayerDamagedOtherQun
        | TriggerFilter::OwnerIsEventPlayerSourceAlive
        | TriggerFilter::OwnerIsEventPlayerSourceAliveOncePerTurn
        | TriggerFilter::OwnerIsEventPlayerSourceAliveDisabledWeapon
        | TriggerFilter::OwnerIsEventPlayerSourceHasWeapon
        | TriggerFilter::OwnerIsEventPlayerOtherSourceAlive
        | TriggerFilter::OwnerIsEventPlayerSourceGroupQunDiscardableHandEquipGteTwo
        | TriggerFilter::OwnerIsEventPlayerOtherSourceAlivePerDamagePoint
        | TriggerFilter::OwnerIsEventPlayerPerDamagePoint
        | TriggerFilter::OwnerIsEventPlayerPerDamagePointOtherHandEquipJudgeNonempty
        | TriggerFilter::OwnerIsEventPlayerSourceAliveGainableCardPerDamagePoint
        | TriggerFilter::OwnerIsEventPlayerSourceAliveGainableCard
        | TriggerFilter::OwnerIsEventPlayerCardInProcessing
        | TriggerFilter::OwnerIsEventPlayerRedShaSourceAlive
        | TriggerFilter::OwnerIsEventPlayerNonRedSha
        | TriggerFilter::OwnerIsEventPlayerRedOrJiuSha
        | TriggerFilter::OwnerIsEventPlayerShaSourceHasDiscardableHandEquip
        | TriggerFilter::OwnerIsEventPlayerRedCardSourceAlive
        | TriggerFilter::OwnerIsEventPlayerNonRedCard
        | TriggerFilter::OwnerIsEventPlayerFirstDamagePerTurn
        | TriggerFilter::OwnerIsEventPlayerSecondDamagePerTurn
        | TriggerFilter::OwnerIsEventPlayerAfterFirstDamagePerTurn
        | TriggerFilter::OwnerIsEventSource
        | TriggerFilter::OwnerAliveSourceTargetWasLinked
        | TriggerFilter::OwnerAliveIsEventSource
        | TriggerFilter::OwnerSourceHandBelowAliveCountCappedSeven
        | TriggerFilter::OwnerIsEventSourceOrPlayer
        | TriggerFilter::OwnerSourceOrTargetOtherAlivePerDamagePoint
        | TriggerFilter::OwnerSourceHealthy
        | TriggerFilter::OwnerSourceAmountGtOne
        | TriggerFilter::OwnerSourceFire
        | TriggerFilter::OwnerTargetFire
        | TriggerFilter::OwnerTargetThunder
        | TriggerFilter::OwnerSourceThunder
        | TriggerFilter::OwnerSourceOtherTwicePerTurn
        | TriggerFilter::OwnerAliveOtherSourceShaTwicePerTurn
        | TriggerFilter::OwnerSourceWoundedDistanceLeOne
        | TriggerFilter::OwnerSourceDistanceLeOnePerDamagePoint
        | TriggerFilter::OwnerSourceTargetDistanceGtOne
        | TriggerFilter::OwnerSourceTargetNotGroup(..)
        | TriggerFilter::OwnerHandEmpty
        | TriggerFilter::OwnerTurnedOver
        | TriggerFilter::OwnerAliveOtherWeiRecoveryOwnPhaseOncePerTurn
        | TriggerFilter::OwnerFromHand
        | TriggerFilter::OwnerFromEquip
        | TriggerFilter::OwnerFromEquipPerCard
        | TriggerFilter::OwnerFromHandEmpty
        | TriggerFilter::OwnerFromHandEmptyOtherDiscardableHandEquip
        | TriggerFilter::OwnerFromFieldEmpty
        | TriggerFilter::OwnerTargetedByTrickEquipEmpty
        | TriggerFilter::OwnerTargetedByShaHandBelowHp
        | TriggerFilter::OwnerTargetedByShaOrJuedouDamagedHpLeSourceOnce
        | TriggerFilter::OwnerTargetedOutsideTurnByShaOrTrickHandEmpty
        | TriggerFilter::OwnerTargetedHandBelowMaxHp
        | TriggerFilter::OwnerTargetedByOtherSingleTargetTrick
        | TriggerFilter::OwnerSourceOrTargetedByJuedouOrRedSha
        | TriggerFilter::OwnerAliveGlobalJuedouOrRedShaUsed
        | TriggerFilter::OwnerAliveOtherGlobalJuedouOrRedShaUsed
        | TriggerFilter::OwnerSkillPileNonempty(..)
        | TriggerFilter::OwnerSourceOrTargetedByJuedou
        | TriggerFilter::OwnerSourceOrTargetedByShaEqualHp
        | TriggerFilter::OwnerSourceOrTargetedByOtherDiamond
        | TriggerFilter::OwnerSourceOrTargetedClubLinkedExistsOncePerTurn
        | TriggerFilter::OwnerSourceShaSingleTrickSourceCard
        | TriggerFilter::OwnerSourceShaSingleEquipSourceCard
        | TriggerFilter::OwnerSourceRed
        | TriggerFilter::OwnerLostOneHandEquipJudgeOtherCurrentPhaseOnce
        | TriggerFilter::OwnerIsTargetSingleTargetOtherSourceOncePerTurn => {
            filters_dcyinjun_tracked_sha_resolved::filter_match(
                state,
                active,
                ctx,
                view_as_skill,
                view_as_card,
                history,
                &def,
            )
        }
        TriggerFilter::OwnerAliveOtherMarkedXi
        | TriggerFilter::OwnerAliveOtherTrickFirstTargetSourceMaxHand
        | TriggerFilter::OwnerAliveOtherPlayPhaseDiscardableGteMark
        | TriggerFilter::OwnerSourceClub
        | TriggerFilter::OwnerSourceAlternatesColor
        | TriggerFilter::OwnerSourceRepeatsColorHasDiscardableHandEquip
        | TriggerFilter::OwnerAliveSkillBoundTargetIsEventPlayer
        | TriggerFilter::OwnerSourceMatchesPreviousSuitOrRank
        | TriggerFilter::OwnerSourcePlaySkillMarkMode(..)
        | TriggerFilter::OwnerSourceHandBasicEqualsNonBasic
        | TriggerFilter::OwnerIsEventPlayerEventCardSha
        | TriggerFilter::OwnerIsEventPlayerEventCardShaOrTrick
        | TriggerFilter::OwnerSourceOrWithinDistanceOneShaOrJuedou
        | TriggerFilter::OwnerSourceSha
        | TriggerFilter::OwnerTargetSha
        | TriggerFilter::OwnerSourceHeartSha
        | TriggerFilter::OwnerSourceShaFirstTarget
        | TriggerFilter::OwnerSourceShaOrJuedouSingleTarget
        | TriggerFilter::OwnerAliveGainedThisTurnCanCompareOther
        | TriggerFilter::OwnerAliveOtherSingleTargetJuedouOrRedSha
        | TriggerFilter::OwnerAliveCardTargetedMarkedPeer(..)
        | TriggerFilter::OwnerSourceNormalSha
        | TriggerFilter::OwnerSourceFireSha
        | TriggerFilter::OwnerIsEventPlayerHandShaSkillPileEmpty
        | TriggerFilter::OwnerAliveDyingHasSkillPileCards
        | TriggerFilter::OwnerSourceSingleTargetShaOrOrdinaryTrickCurrentTurn
        | TriggerFilter::OwnerSourceEventCardPolicyDraw
        | TriggerFilter::OwnerSourcePlayShaOtherAliveOncePerPlayPhase
        | TriggerFilter::OwnerSourcePlayCardCountEquals(..)
        | TriggerFilter::OwnerSourcePlayCardCountEqualsHp
        | TriggerFilter::OwnerIsEventPlayerSourceAliveMarkPositive(..)
        | TriggerFilter::OwnerIsEventPlayerAnyFieldCardNoSkillPile(..)
        | TriggerFilter::OwnerAliveSourceShaHandEquipSkillPile(..)
        | TriggerFilter::OwnerAliveDyingSkillPile(..)
        | TriggerFilter::OwnerAliveOtherSourceSingleShaTargetOther
        | TriggerFilter::OwnerAliveOtherSourceSingleShaTargetOtherOwnerInAttackRange
        | TriggerFilter::OwnerAliveOtherSourceShaResolvedTwxiaolianRedirectHasHandEquip
        | TriggerFilter::OwnerIsEventPlayerTwxiaolianRedirectedDamage
        | TriggerFilter::OwnerIsEventPlayerOtherHigherHandOncePerGame
        | TriggerFilter::OwnerSourceJiu
        | TriggerFilter::OwnerIsEventPlayerHpOdd
        | TriggerFilter::OwnerIsEventPlayerHpEvenWounded
        | TriggerFilter::OwnerSourceTrick
        | TriggerFilter::OwnerSourceShaOrTrickCurrentPlayPhase
        | TriggerFilter::OwnerSourceShaCurrentPlayPhase
        | TriggerFilter::OwnerSourceCardHasOtherTargetCurrentPlayPhaseFirstTwo
        | TriggerFilter::OwnerSourceOrdinaryTrick
        | TriggerFilter::OwnerSourceOrdinaryTrickMarkPositive(..)
        | TriggerFilter::OwnerSourceShaOrBlackTrickMarkPositive(..)
        | TriggerFilter::OwnerSourceBasic
        | TriggerFilter::OwnerSourceNonBasic
        | TriggerFilter::OwnerSourceWoundedNanmanWanjianOrNormalSpadeSha
        | TriggerFilter::OwnerSourceDamageCardOncePerTurn
        | TriggerFilter::OwnerAliveOtherPlayPhaseDuplicateCardCategoryHasHandEquip
        | TriggerFilter::OwnerAliveOtherPhaseEndDamageGtOneHasHandEquip
        | TriggerFilter::OwnerAliveOtherPhaseEndHasDiscardableBasicHand
        | TriggerFilter::OwnerAliveOtherPhaseEndHandNonempty
        | TriggerFilter::OwnerAlivePreparePlayerHpGtOwner
        | TriggerFilter::OwnerAlivePrepareLordHpGtOwner
        | TriggerFilter::OwnerAliveOtherShaTargetCanRedirectToOwner
        | TriggerFilter::OwnerShaTargetCanDiscardRedirectToOtherInAttackRange
        | TriggerFilter::OwnerShaTargetCanDiscardRedirectToZhengjianMarked
        | TriggerFilter::OwnerIsTargetShaSourceAliveHandEquipNonemptyOncePerTurn
        | TriggerFilter::OwnerIsTargetTrickTargetsGtSourceHpHandEquipNonempty
        | TriggerFilter::OwnerIsSourceShaUsed
        | TriggerFilter::OwnerIsResponderSha
        | TriggerFilter::OwnerIsSourceShaCardDamageBefore
        | TriggerFilter::OwnerIsTargetDamageAmountGtOne
        | TriggerFilter::OwnerIsTargetSourceHpGtOwnerPerRoleTurn
        | TriggerFilter::OwnerSourceBasicOrTrick
        | TriggerFilter::OwnerIsEventPlayerCanCompareOther
        | TriggerFilter::OwnerSourcePhysicalOrdinaryTrick
        | TriggerFilter::OwnerHandBelowLostHpAfterRelevantChange
        | TriggerFilter::OwnerAliveSourceFirstCardOrdinaryTrick
        | TriggerFilter::OwnerSourceMatchesLastUsedSuitOrRank
        | TriggerFilter::OncePerTurn
        | TriggerFilter::OwnerAliveEventPlayerPlayPhaseSameSuitAtLeastTwoOncePerTurn
        | TriggerFilter::OwnerIsEventPlayerDiscardPileFourSuitsBelowAliveCap
        | TriggerFilter::OwnerSourceOncePerTurn
        | TriggerFilter::OwnerIsEventPlayerOncePerTurn
        | TriggerFilter::OwnerSourceTargetAliveHasEquip
        | TriggerFilter::OwnerSourceBlackShaTargetAliveHasDiscardableEquip
        | TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableRegionCard
        | TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableHandEquip
        | TriggerFilter::OwnerSourceTiesuoTargetAliveUnlinkedHandEquipNonempty
        | TriggerFilter::OwnerSourceShaTargetAliveHandNonempty
        | TriggerFilter::OwnerSourceShaTargetAliveDistanceLeOne
        | TriggerFilter::OwnerAliveShaTargetDistanceLeOne
        | TriggerFilter::OwnerAliveShaTargetDistanceLeOneHasEquipCard
        | TriggerFilter::OwnerIsEventPlayerShaSourceAliveHandEquipNonempty
        | TriggerFilter::OwnerAliveWdbFuchouPendingLiving
        | TriggerFilter::OwnerAliveOtherGainedSingleEquipToHandOutsideTurnUsable
        | TriggerFilter::OwnerAliveWuguRemainedInDiscard
        | TriggerFilter::OwnerIsTargetNonMultitargetTrickMultipleTargets
        | TriggerFilter::OwnerSourcePlayPhaseShaHasAdditionalTargetIgnoringDistance
        | TriggerFilter::OwnerSourceCardResolvedTrackedDddXianxi
        | TriggerFilter::OwnerSourceCardResolvedDddXuyuPending
        | TriggerFilter::DddXuyuPending
        | TriggerFilter::OwnerAliveOtherSourceSecondPlayCardResolvedHandEquipNonempty
        | TriggerFilter::DddShijianWatchedSourceCardUsedTargeted
        | TriggerFilter::DddShijianPending
        | TriggerFilter::OwnerAliveHpChangedThisTurnEnteredDiscardAvailable
        | TriggerFilter::OwnerAliveOtherPhaseBeginBothHandNonempty
        | TriggerFilter::OwnerAliveDddDongchaPileNonempty
        | TriggerFilter::OwnerAliveOtherPhaseBeginUnshownHandGteHp
        | TriggerFilter::OwnerPlayDddZhengjunHpAvailable
        | TriggerFilter::OwnerPlayDddZhengjunCardsMovedAvailable
        | TriggerFilter::OwnerAliveOtherPhasePlayDddChashiAvailable
        | TriggerFilter::DddChashiMatchingCard
        | TriggerFilter::OwnerAliveDddQiceReady
        | TriggerFilter::OwnerAliveDddFenjiAvailable => {
            filters_owner_alive_other_marked_xi::filter_match(
                state,
                active,
                ctx,
                view_as_skill,
                view_as_card,
                history,
                &def,
            )
        }
        TriggerFilter::OwnerAliveDddFengzhengRoundPending
        | TriggerFilter::OwnerAliveDddYulvDiscardMatches
        | TriggerFilter::OwnerAliveDddYouxueRoundPending
        | TriggerFilter::OwnerAliveDddChengjingAvailable
        | TriggerFilter::DddChashiPendingPhasePlayEnd
        | TriggerFilter::OwnerAliveDddZhilianAvailable
        | TriggerFilter::OwnerAliveDddZhuansheGift
        | TriggerFilter::OwnerAliveDddZhuansheCardUsed
        | TriggerFilter::OwnerAliveDddZhuanshePhaseEnd
        | TriggerFilter::OwnerAliveDddWeiqiu
        | TriggerFilter::OwnerAliveDddLianerAvailable
        | TriggerFilter::OwnerAliveDddLianerCeiling
        | TriggerFilter::OwnerAliveDddAnzhiAvailable
        | TriggerFilter::OwnerAliveDddAnzhiTracking
        | TriggerFilter::OwnerAliveDddAnzhiTargetBlack
        | TriggerFilter::OwnerAliveDddAnzhiTurnEnd
        | TriggerFilter::OwnerAliveDddShixingTurnRefresh
        | TriggerFilter::OwnerAliveDddShixingFemaleChanged
        | TriggerFilter::OwnerAliveDddShixingFirstTrick
        | TriggerFilter::OwnerAliveDddDangguAvailable
        | TriggerFilter::OwnerAliveDddFuzongDeath
        | TriggerFilter::OwnerIsTargetFaceDown
        | TriggerFilter::OwnerAliveDddXuanlunPending
        | TriggerFilter::OwnerAliveDddJiexingRecoverOncePerGame
        | TriggerFilter::OwnerAliveDddJiexingEquipOncePerGame
        | TriggerFilter::OwnerAliveDddJiexingRecoveryReset
        | TriggerFilter::OwnerAliveDddJiexingEquipReset
        | TriggerFilter::OwnerAliveDddBaileiAvailable
        | TriggerFilter::OwnerSourceOwnViewAsDddTongyuUsed
        | TriggerFilter::OwnerSourceOwnViewAsDddTongyuTargeted
        | TriggerFilter::OwnerAliveDddTongyuRemained
        | TriggerFilter::OwnerAliveSourceBasicDddFushiAvailable
        | TriggerFilter::OwnerSourceShaTargetHandGteHpOrLteAttackRange
        | TriggerFilter::OwnerSourceShaTargetHandLteOwnerOrHpGteOwner
        | TriggerFilter::OwnerIsEventPlayerJudgeNonempty
        | TriggerFilter::OwnerIsEventPlayerDrawNotReplaced
        | TriggerFilter::OwnerIsEventPlayerDrawNotFixedHpOneOrHandEmpty
        | TriggerFilter::OwnerSourceShaDodgedTargetAliveHasDiscardableHandEquip
        | TriggerFilter::OwnerSourceShaDodgedTargetRecordedBySkillHasDiscardableHandEquip
        | TriggerFilter::OwnerSourceShaDodged
        | TriggerFilter::OwnerSourceShaDamageRecordedBySkill
        | TriggerFilter::OwnerAliveOtherDyingOncePerRound
        | TriggerFilter::OwnerIsTargetDiscardableHandEquipGteTwo
        | TriggerFilter::OwnerSourceTargetAliveHasHandOrEquip
        | TriggerFilter::OwnerSourceOtherTargetAliveHasDiscardableHandEquip
        | TriggerFilter::OwnerSourceShaTargetAliveOwnerOutsideTargetAttackRange
        | TriggerFilter::OwnerAliveFirstTargetVirtualOrConvertedCard
        | TriggerFilter::OwnerTargetedByOtherShaOncePerTurn
        | TriggerFilter::OwnerTargetedByOtherCardOncePerTurn
        | TriggerFilter::OwnerTwfeifuSingleShaPartyTargetHasHandEquip
        | TriggerFilter::OwnerTargetedByOtherSha
        | TriggerFilter::OwnerAliveEventPlayerTurnChangedOrBecameLinked
        | TriggerFilter::OwnerAliveEventPlayerBecameFaceUpOrLinked
        | TriggerFilter::OwnerCardActionCountEqualsAttackRange
        | TriggerFilter::OwnerLifetimeCardAction
        | TriggerFilter::OwnerAliveOtherShanOutsideTurnGroupOncePerTurn(..)
        | TriggerFilter::OwnerAliveOtherDiscardedBasicOutsideTurn
        | TriggerFilter::OwnerAliveOtherDiscardedBasicOutsideTurnOncePerTurn
        | TriggerFilter::OwnerFirstHandDiscardHasShaInDiscard
        | TriggerFilter::OwnerAliveDealtOrReceivedDamageThisTurn
        | TriggerFilter::OwnerReceivedGiftFromOther
        | TriggerFilter::OwnerInitiatedDuelTargetAliveLoserHasHandOrEquip
        | TriggerFilter::OwnerHasEnabledEquipSlot
        | TriggerFilter::OwnerIsEventPlayerNoEnabledEquipSlotOncePerGame
        | TriggerFilter::OwnerIsEventPlayerHasMovableEquipment
        | TriggerFilter::OwnerAliveOtherDyingHandNonemptyOncePerGame
        | TriggerFilter::OwnerDyingOtherHandEquipNonemptyOncePerGame
        | TriggerFilter::OwnerAliveOtherDyingFirstPerTarget
        | TriggerFilter::OwnerIsEventPlayerTrickUsedZero
        | TriggerFilter::OwnerIsEventPlayerHpLeOne
        | TriggerFilter::OwnerIsEventPlayerHpLeTwo
        | TriggerFilter::OwnerSourceOrTargetEventPlayerSecondDamageThisTurn
        | TriggerFilter::OwnerSourceOrTargetEventPlayerFirstDamageThisTurn
        | TriggerFilter::OwnerSourceTargetDistanceLeOneFirstDamageToTargetThisTurn
        | TriggerFilter::OwnerFirstTransformedShaUsedThisTurn
        | TriggerFilter::OwnerTargetHandEmptyElementalOrTrickDamageBefore
        | TriggerFilter::OwnerAliveDrawPlayerOwnerOrMinHpTwjiaohuaAvailable
        | TriggerFilter::OwnerAliveShaOrDuelDamageSourceTargetAliveHandEquipNonempty
        | TriggerFilter::OwnerAliveShaDamageSourceTargetAliveOwnerHandEquipNonempty
        | TriggerFilter::OwnerIsEventPlayerAnyAliveSelfDiscardableHandEquip
        | TriggerFilter::OwnerIsEventPlayerAnyAliveDiscardableByOwnerHandEquip
        | TriggerFilter::OwnerIsEventPlayerAnyAliveSelfDiscardableHand
        | TriggerFilter::OwnerIsEventPlayerAnyAliveWounded
        | TriggerFilter::OwnerIsEventPlayerHandEquipNonemptyOtherAlive
        | TriggerFilter::OwnerAliveGainedThreeThisTurnOtherAlive
        | TriggerFilter::OwnerAliveGainedTwoThisTurnCanCompareOther
        | TriggerFilter::OwnerAliveGaveCardsToOtherAlive
        | TriggerFilter::OwnerAliveTargetedByOtherHiddenSkillPile(..)
        | TriggerFilter::OwnerIsEventPlayerHandNonemptyHiddenSkillPile(..)
        | TriggerFilter::OwnerLostMatchingRevealedSkillPileSuit(..)
        | TriggerFilter::OwnerAliveGroupCountAtMostTwo
        | TriggerFilter::OwnerAliveCurrentTurn
        | TriggerFilter::OwnerIsEventPlayerCurrentTurn
        | TriggerFilter::DebateOwnerIsInitiator
        | TriggerFilter::DebateOwnerParticipated
        | TriggerFilter::OwnerDebateParticipant
        | TriggerFilter::OwnerIsLord
        | TriggerFilter::OwnerAliveEventPlayerLord
        | TriggerFilter::OwnerAliveEventPlayerLordNoDamage
        | TriggerFilter::OwnerAliveEventPlayerLordDamaged
        | TriggerFilter::OwnerAliveTurnUsedAllSuits
        | TriggerFilter::OwnerAliveEventTargetMarkPositiveNoDamage(..)
        | TriggerFilter::OwnerAliveEventTargetMarkPositive(..)
        | TriggerFilter::OwnerAliveDamageBeforeEventTargetMarkPositive(..)
        | TriggerFilter::OwnerAlivePhaseBeginEventTargetMarkPositive(..)
        | TriggerFilter::OwnerAlivePlayEndUsedCategoriesPositive
        | TriggerFilter::OwnerGainedCardsDuringPlayPhaseWounded
        | TriggerFilter::OwnerDiscardedOutsidePlayPhase
        | TriggerFilter::OwnerInMultitargetShaOrTrickTargets
        | TriggerFilter::OwnerTargetedByNanman => {
            filters_owner_alive_ddd_fengzheng_round_pending::filter_match(
                state,
                active,
                ctx,
                view_as_skill,
                view_as_card,
                history,
                &def,
            )
        }
        TriggerFilter::OwnerBlackCardMovedEquipJudge
        | TriggerFilter::OwnerAliveDamageCardNanman
        | TriggerFilter::OwnerDiscardSuitNoOwnFaluMark(..)
        | TriggerFilter::OwnerAliveOtherDyingHasEquipCard
        | TriggerFilter::OwnerSourceTrickResolvedOwnPlayPhaseOncePerTurn
        | TriggerFilter::OwnerTargetedSourceHpHandDifferOncePerTurn
        | TriggerFilter::OwnerSourceHandGaintagAbsent(..)
        | TriggerFilter::CardsMovedDiscardAtLeastThree
        | TriggerFilter::OwnerSourceDamageTargetMarkPositiveOnce(..)
        | TriggerFilter::OwnerAliveMarkedAliveAboveTwo(..)
        | TriggerFilter::CardsMovedIntoExpansion
        | TriggerFilter::CardsMovedOutOfExpansion
        | TriggerFilter::OwnerAliveDiscardMatchesOwnPileCard(..)
        | TriggerFilter::OwnerAliveSomeoneMarkPositive(..)
        | TriggerFilter::OwnerAliveShaResolvedCity(..)
        | TriggerFilter::OwnerAliveOtherTargetedBySha
        | TriggerFilter::OwnerAliveTargetedByOtherShaJudgeXumou
        | TriggerFilter::OwnerCardsMovedContainYing
        | TriggerFilter::OwnerAliveOtherSourceDamage
        | TriggerFilter::DebateOwnerInitiatorResultRed
        | TriggerFilter::DebateOwnerInitiatorResultBlack
        | TriggerFilter::DebateOwnerOpinionDiffersInitiator
        | TriggerFilter::OwnerLostPackageCardFromEquip
        | TriggerFilter::OwnerAliveSourceAllyShaDamageBefore
        | TriggerFilter::OwnerIsEventPlayerWoundedDeadAlly
        | TriggerFilter::OwnerIsEventPlayerPhaseEndHandEquipNonemptyOtherAlive
        | TriggerFilter::OwnerSourceShaDamageTargetAliveHasDiscardableEquip
        | TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableEquip
        | TriggerFilter::OwnerSourceCardUsedTargetsSelf
        | TriggerFilter::OwnerSourceCurrentPhaseCardTargetedOtherHealthy
        | TriggerFilter::OwnerSkillAddedSelf(..)
        | TriggerFilter::OwnerSourceFirstCardSuitThisTurn
        | TriggerFilter::OwnerLostHandNowEmpty
        | TriggerFilter::OwnerLostSpadeTwoToNine
        | TriggerFilter::OwnerSourceCardDamageTargetCanReceiveOwnerBingliang
        | TriggerFilter::OwnerSourceEquipUsedSkillMarkBelowSeven
        | TriggerFilter::OwnerSourceOutsideTurnCardUsedOrResponded
        | TriggerFilter::OwnerAliveShanUsedByOwnerOrDuringOwnerTurn
        | TriggerFilter::OwnerAliveOtherPhasePlaySkillPileNonempty
        | TriggerFilter::OwnerAliveOtherSourceDamageCardResolvedTargetedDamaged
        | TriggerFilter::OwnerAliveOtherSourceDamageCardResolvedTargetedNotDamaged
        | TriggerFilter::OwnerAliveOtherDyingMbchensheAvailable
        | TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipOncePerRound
        | TriggerFilter::OwnerAliveFirstNonDiscardPhaseDiscardMbrenxingAvailable
        | TriggerFilter::OwnerSourceFirstTargetHandDamageCardDiscardable
        | TriggerFilter::OwnerLostHandOwnerlessShaNotUseOncePerTurn
        | TriggerFilter::OwnerAlivePhaseAnyEndOriginatingSkillPileNonempty
        | TriggerFilter::OwnerSourceDamageTrackedMbduanyangTargetRecastable
        | TriggerFilter::OwnerMbrunweiLossThreshold
        | TriggerFilter::OwnerPhasePlayEndMbrunweiTrackedHand
        | TriggerFilter::OwnerAliveDamageBeforeTargetDistanceLeOneOncePerTurn
        | TriggerFilter::OwnerSourceDamageMbxiongtuArmed
        | TriggerFilter::OwnerPhasePlayEndMbxiongtuState
        | TriggerFilter::OwnerPhaseEndMbganggengTargetAlive
        | TriggerFilter::OwnerFromHandEmptyOrDyingMbsijianTwicePerTurn
        | TriggerFilter::OwnerIsTargetCanMoveFieldCardMbyuanmoTwicePerTurn
        | TriggerFilter::OwnerSourceFirstSingleTargetShaMbfeijingTwicePerTurn
        | TriggerFilter::OwnerSourceDamageBeforeMbfeijingExtra
        | TriggerFilter::OwnerSourceShaResolvedOriginalSingleMbxiaoge
        | TriggerFilter::OwnerAliveDamageTargetMinHandMbxuyeOncePerTurn
        | TriggerFilter::OwnerMbkuangxiangReceivedCardsExhausted
        | TriggerFilter::OwnerPhasePlayMbkuangxiangActive
        | TriggerFilter::OwnerGainedCardsPotduzuo
        | TriggerFilter::OwnerGainedCardsPotduzuoOncePerTurn
        | TriggerFilter::OwnerShaDamageBeforePotbihanAvailable
        | TriggerFilter::OwnerAliveCardTargetedHealthy
        | TriggerFilter::OwnerAliveCardResolvedSourceCurrentPotsifengPile
        | TriggerFilter::OwnerAlivePhaseEndPlayerPotsifengPile
        | TriggerFilter::OwnerAlivePotdingpinAvailableOncePerRound
        | TriggerFilter::OwnerSourceCardResolvedPhasePolicy
        | TriggerFilter::OwnerPhasePlayEndPotguanshaAvailableOncePerGame
        | TriggerFilter::OwnerDiscardedShaToDiscard
        | TriggerFilter::OwnerAliveOtherTurnDiscardedRecordedPotfuanSuit
        | TriggerFilter::OwnerSourceShaResolvedPotfuanDealtDamage
        | TriggerFilter::OwnerAliveOtherPlayPhasePothuilvAvailable
        | TriggerFilter::OwnerIsTargetCanUseWuzhong
        | TriggerFilter::OwnerAliveDiscardBatch
        | TriggerFilter::OwnerAliveOtherRespondedToOwner
        | TriggerFilter::OwnerSourceDamagePotjianzhuanAvailable
        | TriggerFilter::OwnerTargetDamagePotjianzhuanAvailable
        | TriggerFilter::OwnerAliveDyingPotjianzhuanAvailable
        | TriggerFilter::OwnerSourceDamagePotcaoshuangAssignedUntriggered
        | TriggerFilter::OwnerTargetDamagePotcaoshuangAssignedUntriggered
        | TriggerFilter::OwnerAliveDyingPotcaoshuangAssignedUntriggered
        | TriggerFilter::OwnerUsedPotnizunPileCard
        | TriggerFilter::OwnerTargetDamageBeforePotnizunUsed
        | TriggerFilter::OwnerAliveTurnEndedUsedCardTaggedDiscard
        | TriggerFilter::OwnerSourceCardName(..)
        | TriggerFilter::OwnerIsTargetCardName(..)
        | TriggerFilter::OwnerIsTargetCardDamage(..)
        | TriggerFilter::EventViewAsSkill(..)
        | TriggerFilter::EventViewAsSkillName(..) => {
            filters_owner_black_card_moved_equip_judge::filter_match(
                state,
                active,
                ctx,
                view_as_skill,
                view_as_card,
                history,
                &def,
            )
        }
    }
}

pub(crate) fn potcaoshuang_timing_ready(
    state: &crate::state::GameState,
    active: &ActiveTrigger,
    timing: u8,
) -> bool {
    state.skills.get(active.skill_id).is_some_and(|skill| {
        matches!(
            skill.def.raw(),
            crate::state::POTDIANYI_DEF | crate::state::POTSHEQUAN_DEF
        ) && {
            let (assigned, triggered, _) = skill.potcaoshuang_state(state.turn.round);
            assigned & timing != 0 && triggered & timing == 0
        }
    })
}

pub(crate) fn potjianzhuan_timing_available(
    state: &crate::state::GameState,
    active: &ActiveTrigger,
    timing: u8,
) -> bool {
    let Some(skill) = state.skills.get(active.skill_id) else {
        return false;
    };
    if skill.def.raw() != crate::state::POTJIANZHUAN_DEF || skill.lifetime_fire_count != 0 {
        return false;
    }
    let assigned = state
        .players
        .get(active.owner)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .filter_map(|id| state.skills.get(*id))
        .filter(|instance| {
            matches!(
                instance.def.raw(),
                crate::state::POTDIANYI_DEF | crate::state::POTSHEQUAN_DEF
            ) && state.skill_effectively_enabled(instance)
        })
        .fold(0, |mask, instance| {
            mask | instance.potcaoshuang_state(state.turn.round).0
        });
    assigned != 0b111 && assigned & timing == 0 || assigned == 0b111
}

pub(crate) fn mbfeijing_record_matches(
    state: &crate::state::GameState,
    owner: PlayerId,
    card: CardId,
    target: Option<PlayerId>,
) -> bool {
    state
        .players
        .get(owner)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .filter_map(|skill| state.skills.get(*skill))
        .any(|skill| {
            let blob = &skill.state_blob;
            blob.get(..4) == Some(b"FJG1")
                && blob.len() >= 12
                && (blob.len() - 12) % 8 == 0
                && u32::from_le_bytes(blob[4..8].try_into().unwrap()) == card.raw()
                && target.is_none_or(|target| {
                    blob[12..].chunks_exact(8).any(|pair| {
                        u32::from_le_bytes(pair[..4].try_into().unwrap()) == target.raw()
                    })
                })
        })
}

pub(crate) fn xicha_state_has_source(blob: &[u8], source: PlayerId) -> bool {
    if blob.get(..4) != Some(b"XCH1") {
        return false;
    }
    let mut cursor = 4;
    while cursor + 5 <= blob.len() {
        let saved = PlayerId::new(u32::from_le_bytes(
            blob[cursor..cursor + 4].try_into().unwrap(),
        ));
        let count = blob[cursor + 4] as usize;
        cursor += 5;
        let bytes = count.saturating_mul(4);
        if cursor + bytes > blob.len() {
            return false;
        }
        if saved == source {
            return true;
        }
        cursor += bytes;
    }
    false
}

pub(crate) fn same_sex(left: &str, right: &str) -> bool {
    !matches!(left, "" | "unknown")
        && !matches!(right, "" | "unknown")
        && (left == "double" || right == "double" || left == right)
}

pub(crate) fn same_logical_card_name(
    previous_kind: CardKind,
    previous_def: u32,
    current_kind: CardKind,
    current_def: u32,
) -> bool {
    let canonical = |kind: CardKind| {
        if kind.is_sha() {
            CardKind::Sha
        } else {
            kind
        }
    };
    let previous_kind = canonical(previous_kind);
    let current_kind = canonical(current_kind);
    previous_kind == current_kind
        && (!previous_kind.is_equip() && previous_kind != CardKind::Other
            || previous_def == current_def)
}

pub(crate) fn turn_limit(filter: &TriggerFilter) -> Option<u32> {
    match filter {
        TriggerFilter::OncePerTurn
        | TriggerFilter::OwnerAliveEventPlayerPlayPhaseSameSuitAtLeastTwoOncePerTurn
        | TriggerFilter::OwnerSourceOncePerTurn
        | TriggerFilter::OwnerIsEventPlayerOncePerTurn
        | TriggerFilter::OwnerIsEventPlayerSourceAliveOncePerTurn
        | TriggerFilter::OwnerTargetedByOtherShaOncePerTurn
        | TriggerFilter::OwnerTargetedByOtherCardOncePerTurn
        | TriggerFilter::OwnerAliveNotEventPlayerOncePerTurn
        | TriggerFilter::OwnerTargetedByShaOrJuedouDamagedHpLeSourceOnce
        | TriggerFilter::OwnerAliveFirstCardsMovedToHand
        | TriggerFilter::OwnerAliveOtherDiscardedBasicOutsideTurnOncePerTurn
        | TriggerFilter::OwnerAliveOtherShanOutsideTurnGroupOncePerTurn(_)
        | TriggerFilter::OwnerAliveOtherSourceShaResolvedNotTargetGainableOncePerTurn
        | TriggerFilter::OwnerSourceDamageCardOncePerTurn
        | TriggerFilter::OwnerSourceOtherDamageBeforeOncePerTurn
        | TriggerFilter::OwnerSourceTargetEarlierSeatHandBelowTargetMaxHpOncePerTurn
        | TriggerFilter::OwnerSourceOrTargetedClubLinkedExistsOncePerTurn
        | TriggerFilter::OwnerGainsOutsideTurnMarkBelowAliveOncePerTurn(_)
        | TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmptyOncePerTurn
        | TriggerFilter::OwnerSourceDamageTargetDistanceAtMostOneEmptyEquipSlotUnused
        | TriggerFilter::OwnerFirstTransformedShaUsedThisTurn
        | TriggerFilter::OwnerSourceShaTargetTwzhenxiAvailableOncePerTurn
        | TriggerFilter::OwnerSourceFirstTargetDamageCardOncePerTurn
        | TriggerFilter::OwnerSourceDamageTargetMarkPositiveOnce(_) => Some(1),
        TriggerFilter::OwnerAliveSourceBasicDddFushiAvailable => Some(1),
        TriggerFilter::OwnerSourceCardResolvedHasOtherTargetDcsbyingmouOncePerTurn => Some(1),
        TriggerFilter::OwnerSourceWoundedExistsOncePerTurn => Some(1),
        TriggerFilter::OwnerLostHandOwnerlessShaNotUseOncePerTurn => Some(1),
        TriggerFilter::OwnerAliveDamageBeforeTargetDistanceLeOneOncePerTurn => Some(1),
        TriggerFilter::OwnerAliveNeighborGainedCardsOutsideOwnDrawOncePerTurn => Some(1),
        TriggerFilter::OwnerGainedCardsPotduzuoOncePerTurn => Some(1),
        TriggerFilter::OwnerSourceOrTargetHandCountDiffersOncePerTurn => Some(1),
        TriggerFilter::OwnerTargetedSourceHpHandDifferOncePerTurn => Some(1),
        TriggerFilter::OwnerSourceTrickResolvedOwnPlayPhaseOncePerTurn => Some(1),
        TriggerFilter::OwnerAliveCurrentOrTurnFiresBelowHp
        | TriggerFilter::OwnerIsEventPlayerFirstDamagePerTurn
        | TriggerFilter::OwnerIsEventPlayerSecondDamagePerTurn
        | TriggerFilter::OwnerIsEventPlayerAfterFirstDamagePerTurn => Some(u32::MAX),
        TriggerFilter::OwnerAliveOtherWeiRecoveryOwnPhaseOncePerTurn => Some(1),
        TriggerFilter::OwnerSourceOtherTwicePerTurn
        | TriggerFilter::OwnerAliveOtherSourceShaTwicePerTurn
        | TriggerFilter::OwnerFromHandEmptyOrDyingMbsijianTwicePerTurn
        | TriggerFilter::OwnerIsTargetCanMoveFieldCardMbyuanmoTwicePerTurn
        | TriggerFilter::OwnerSourceFirstSingleTargetShaMbfeijingTwicePerTurn => Some(2),
        _ => None,
    }
}

/// 整肃三技能中该角色未有效拥有的数量（ABI 432）。
pub(crate) fn zhengsu_missing_count(
    state: &crate::state::GameState,
    player: engine_api::PlayerId,
    defs: &[u32; 3],
) -> u32 {
    defs.iter()
        .filter(|def| {
            !state.skills.sorted_live().iter().any(|instance| {
                instance.owner == player
                    && instance.def.raw() == **def
                    && state.skill_effectively_enabled(instance)
            })
        })
        .count() as u32
}

pub(crate) fn play_phase_limit(filter: &TriggerFilter) -> Option<u32> {
    match filter {
        TriggerFilter::OwnerSourcePlayShaOtherAliveOncePerPlayPhase
        | TriggerFilter::OwnerDiscardedHasHandOncePerPhase
        | TriggerFilter::OwnerAliveLostHandThisTurn
        | TriggerFilter::OwnerSourceFirstShaThisPlayPhase => Some(1),
        TriggerFilter::OwnerSourceCardHasOtherTargetCurrentPlayPhaseFirstTwo => Some(2),
        TriggerFilter::OwnerSourceBoundCategoryFirstThreePlayPhase => Some(3),
        _ => None,
    }
}

pub(crate) fn once_per_round(filter: &TriggerFilter) -> bool {
    matches!(
        filter,
        TriggerFilter::OwnerAliveOtherDyingOncePerRound
            | TriggerFilter::OwnerAliveSourceAliveOncePerRound
            | TriggerFilter::OwnerAliveSourceAliveEventPlayerDyingOncePerRound
            | TriggerFilter::OwnerAliveOncePerRound
            | TriggerFilter::OwnerAliveEventPlayerHandNonemptyOncePerRound
            | TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipGteTwoOncePerRound
            | TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipOncePerRound
            | TriggerFilter::OwnerAlivePotdingpinAvailableOncePerRound
            | TriggerFilter::OwnerAliveOtherQunPlayPhaseCanGiveOwner
    )
}

pub(crate) fn lifetime_limit(filter: &TriggerFilter) -> Option<u32> {
    matches!(
        filter,
        TriggerFilter::OwnerIsEventPlayerNoEnabledEquipSlotOncePerGame
            | TriggerFilter::OwnerAliveOtherDyingHandNonemptyOncePerGame
            | TriggerFilter::OwnerDyingOtherHandEquipNonemptyOncePerGame
            | TriggerFilter::OwnerIsEventPlayerOtherHigherHandOncePerGame
            | TriggerFilter::OwnerPhasePlayEndPotguanshaAvailableOncePerGame
            | TriggerFilter::OwnerDyingZhupeilanOncePerGame
            | TriggerFilter::OwnerAliveDddJiexingRecoverOncePerGame
            | TriggerFilter::OwnerAliveDddJiexingEquipOncePerGame
    )
    .then_some(1)
}

pub(crate) fn native_handler_effects(
    active: &ActiveTrigger,
    handler: &TriggerHandler,
) -> Vec<EffectRequest> {
    match handler {
        TriggerHandler::DrawSelf { count } => vec![EffectRequest::DrawCardsFromSkill {
            player: active.owner,
            count: *count,
            skill: active.skill_id,
        }],
        TriggerHandler::EmitNote { code } => vec![EffectRequest::EmitRuleEvent {
            code: *code,
            payload: vec![active.skill_id.raw()],
        }],
        TriggerHandler::RecoverSelf { amount } => vec![EffectRequest::RecoverHp {
            source: active.owner,
            player: active.owner,
            amount: *amount,
        }],
        TriggerHandler::RunVm { .. } => Vec::new(),
    }
}

pub(crate) fn timing_from_u16(v: u16) -> TimingBand {
    match v {
        0 => TimingBand::FirstDo,
        200 => TimingBand::LastDo,
        _ => TimingBand::Normal,
    }
}

/// Decrement trigger budget once per planned trigger batch item that is a "fire".
pub fn spend_trigger_budget(tx: &mut Transaction, n: u32) -> Result<(), EffectError> {
    let b = tx.state().resolution.trigger_budget;
    if b < n {
        return Err(EffectError::BudgetExceeded);
    }
    tx.state_mut().resolution.trigger_budget = b - n;
    Ok(())
}
