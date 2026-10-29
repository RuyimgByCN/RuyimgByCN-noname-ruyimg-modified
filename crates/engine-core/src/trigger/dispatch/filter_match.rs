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
        | TriggerFilter::OwnerAliveEventPlayerLostArmorOrMountHasOriginatingSkillPile
        | TriggerFilter::OwnerAliveEventPlayerHandNonemptyOncePerRound
        | TriggerFilter::OwnerPhaseGiftTargetIsEventSource
        | TriggerFilter::OwnerPhaseGiftTargetIsEventPlayer
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
        | TriggerFilter::OwnerUsedAndWasTargetedThisTurnInRange
        | TriggerFilter::OwnerAliveLostHandThisTurn
        | TriggerFilter::OwnerAlivePhaseEndLostCardsAtLeastTwo
        | TriggerFilter::OwnerOutsideTurnDiscardedRedCardsInDiscard
        | TriggerFilter::OwnerAliveTwruilianRoundUnprompted
        | TriggerFilter::OwnerAliveTwruilianSelectedTargetDiscarded
        | TriggerFilter::OwnerAliveTwruilianSelectedTargetTurnEnded
        | TriggerFilter::OwnerAliveLostWeaponOutsideTurn
        | TriggerFilter::OwnerTargetPhysicalShaSourceHpGtOwner
        | TriggerFilter::OwnerAlivePhaseEndCausedTwoDistinctDying
        | TriggerFilter::OwnerGainedEventCardsFromOtherPersonalZone
        | TriggerFilter::OwnerTrackedTwyizhuEnteredDiscard
        | TriggerFilter::OwnerTrackedSkillCardLeftHandNotUseOrRespond
        | TriggerFilter::OwnerAliveShaUsedBySelfOrInRangeDiscardableHandEquip
        | TriggerFilter::OwnerAliveTrackedCuijinShaResolvedNoDamage
        | TriggerFilter::OwnerAliveTrackedCuijinShaResolvedDealtDamage
        | TriggerFilter::OwnerAliveTrackedTwqingkouDuelResolved
        | TriggerFilter::OwnerAliveTwchenxunRoundUnprompted
        | TriggerFilter::OwnerAliveTrackedTwchenxunDuelTargetDamaged
        | TriggerFilter::OwnerAliveTrackedTwchenxunDuelTargetNotDamaged
        | TriggerFilter::OwnerAliveTrackedTwchenxunCardLeftProcessing
        | TriggerFilter::OwnerPhaseBeginPlayableSkillPileNonempty
        | TriggerFilter::OwnerUsedPlayableSkillPileCardUnused
        | TriggerFilter::OwnerTargetDamageBeforePlayableSkillPilesEmpty
        | TriggerFilter::OwnerAliveDamageBeforeTargetMinHpUnused
        | TriggerFilter::OwnerAliveGlobalPhaseEnd
        | TriggerFilter::OwnerAliveGlobalPhaseEndLostCardsTargetOtherAlive
        | TriggerFilter::OwnerAliveGlobalPhaseEndDiscardedCardsAvailable
        | TriggerFilter::OwnerIsTargetEventCardJiu
        | TriggerFilter::OwnerIsTargetOtherSourceBlackDcjiudunAvailable
        | TriggerFilter::OwnerAlivePhaseEndUsedCardNoDamage
        | TriggerFilter::OwnerCardsEnteredDiscardNotUseUnused
        | TriggerFilter::OwnerIsEventPlayerDiscardedAtLeastTwoInDiscardPhase
        | TriggerFilter::OwnerIsTargetMissingZhengsu
        | TriggerFilter::OwnerIsTargetMultiTargetTrickHasHandEquip
        | TriggerFilter::OwnerIsTargetFeijunNewTarget
        | TriggerFilter::OwnerAliveSkillBoundTargetIsEventSource
        | TriggerFilter::OwnerLostCardsOutsideTurn
        | TriggerFilter::OwnerPhaseBeginSkillPileGteThree(..)
        | TriggerFilter::OwnerPhaseBeginHandEmpty
        | TriggerFilter::OwnerRespondedShan
        | TriggerFilter::OwnerAlivePlayPhaseActorHandNonemptyPotjiejieUnused
        | TriggerFilter::OwnerGainedCardsPotzhouyu
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
        TriggerFilter::OwnerSourceFirstTargetBlackCardAnyTargetOtherDiscardableHandEquip
        | TriggerFilter::OwnerSourceDirectShaDamageTargetArmorOrMountCardGainable
        | TriggerFilter::OwnerSourcePlayCardMatchesRememberedCategory
        | TriggerFilter::OwnerSourceBoundCategoryFirstThreePlayPhase
        | TriggerFilter::OwnerSourceShaResolvedHasGainableCardsAndRecipient
        | TriggerFilter::OwnerAliveOtherSourceShaResolvedNotTargetGainableOncePerTurn
        | TriggerFilter::OwnerSourceInTurnTargetUnharmedCardActionsGeTargetHp
        | TriggerFilter::OwnerSourceFirstShaThisPlayPhase
        | TriggerFilter::OwnerSourceDamageTargetDistanceAtMostOneEmptyEquipSlotUnused
        | TriggerFilter::OwnerSourceFirstShaTargetThisTurn
        | TriggerFilter::OwnerAliveOtherPhasePlayEndTwzhengjianCondition
        | TriggerFilter::OwnerSourceShaTargetTwzhenxiAvailableOncePerTurn
        | TriggerFilter::OwnerSourceFirstTargetDamageCardOncePerTurn
        | TriggerFilter::OwnerAliveOtherTwkujianUsedOrResponded
        | TriggerFilter::OwnerAliveOtherTwkujianLost
        | TriggerFilter::OwnerAliveOtherPhasePrepareCanCompareOwner
        | TriggerFilter::OwnerSourceOtherTargetShaOrOrdinaryTrickTwguoyiCondition
        | TriggerFilter::OwnerAliveOtherSingleTargetUsedTrackedTwyizhu
        | TriggerFilter::OwnerAliveOtherShaResolvedTrackedSkillCardDealtDamage
        | TriggerFilter::OwnerAliveOtherShaResolvedTrackedSkillCardNoDamage
        | TriggerFilter::OwnerAliveOtherTwgonghuanDamageBeforeHpLteOwnerUnused
        | TriggerFilter::OwnerAliveOtherPhaseEndMobiledanshou
        | TriggerFilter::OwnerAliveOtherPhaseEndNotDamagedOwnerThisTurn
        | TriggerFilter::OwnerAliveOtherPhaseEndUsedCardOnOwnerUndamagedThisTurn
        | TriggerFilter::OwnerSourceActiveDczhaowenTrackedRed
        | TriggerFilter::OwnerAliveOtherDiscardedInDiscardPhase
        | TriggerFilter::OwnerAliveOtherPlayPhaseZhengsuMissingInRange
        | TriggerFilter::OwnerAliveOtherWuPlayPhaseCanCompareOwner
        | TriggerFilter::OwnerSourceShaDamageTargetAliveBothHand
        | TriggerFilter::OwnerAliveOtherNanmanCardsDiscard
        | TriggerFilter::OwnerAliveOtherQunPlayPhaseCanGiveOwner
        | TriggerFilter::OwnerSourceShaDamageBeforePotyinzhanCondition
        | TriggerFilter::OwnerSourceDamageWithinOne
        | TriggerFilter::OwnerSourceFirstTargetHasHandPotyanhui => filters_always_source_other::filter_match(
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
        | TriggerFilter::OwnerXichaWatchedSourceCardUsed
        | TriggerFilter::OwnerQiaojianTrackedCardLeftHand
        | TriggerFilter::OwnerAliveDamageTargetSelfOrInRangeOtherSourceNumericCard
        | TriggerFilter::OwnerAliveNanmanResolvedTargetDeadOrDamagedGtThree
        | TriggerFilter::OwnerLostNonDamageCardPottuntianChargeNotFull
        | TriggerFilter::OwnerAliveShaEnteredDiscardPotzhanlieAvailable
        | TriggerFilter::OwnerIsTargetDiscardPhaseOweLimitOtherQun
        | TriggerFilter::OwnerAlivePhaseTargetedHpChangedCanUseWanjian
        | TriggerFilter::OwnerAlivePhaseTargetedHpUnchangedCanUseJuedou
        | TriggerFilter::OwnerIsTargetDiscardableHandEquip
        | TriggerFilter::OwnerHandCompareParticipant
        | TriggerFilter::OwnerHandCompareFinishedParticipant
        | TriggerFilter::OwnerHandCompareParticipantHasSha
        | TriggerFilter::OwnerHandCompareParticipantResultCardInDiscard
        | TriggerFilter::OwnerDiscardedBlackBasicToDiscard
        | TriggerFilter::OwnerAliveDiscardedBlackBasicToDiscard
        | TriggerFilter::OwnerCardRespondedToByPlayerInRange
        | TriggerFilter::OwnerAliveFirstCardsMovedToHand
        | TriggerFilter::OwnerAliveNeighborGainedCardsOutsideOwnDrawOncePerTurn
        | TriggerFilter::OwnerDrawsDuringOwnPlayNotSelf
        | TriggerFilter::OwnerEquipZoneChangedAtLeastTwoWounded
        | TriggerFilter::OwnerAliveEquipUsed
        | TriggerFilter::OwnerAliveEquipUsedSkillMarkBelowThree
        | TriggerFilter::OwnerAliveOncePerRound
        | TriggerFilter::OwnerAlive
        | TriggerFilter::OwnerAliveKilledThisTurn
        | TriggerFilter::OwnerAliveHandBelowAliveCountCappedSeven
        | TriggerFilter::OwnerAliveHandBelowAliveCountCappedEight
        | TriggerFilter::OwnerAliveDelayedTrickTargetWithoutPendingJudgeSkip
        | TriggerFilter::OwnerLongdanViewAsFirstCounterpartHandNonempty
        | TriggerFilter::OwnerAliveHandHasSameKindAsEventCard
        | TriggerFilter::OwnerAliveHasSkillTaggedCards
        | TriggerFilter::OwnerAliveTaggedCardLostThisTurn
        | TriggerFilter::OwnerAliveShaTargetNoOtherRedCardThisTurn
        | TriggerFilter::OwnerAliveEquipSourceMaxEquip
        | TriggerFilter::OwnerAliveNotEventPlayerNeitherUniqueMaxHand
        | TriggerFilter::OwnerTrackedJuedouOriginalTargetDamageRecoverableInRange
        | TriggerFilter::OwnerIsTargetPhaseEndHandNonemptyOtherHandNonempty
        | TriggerFilter::OwnerIsTargetOrdinaryTrickCilvAvailable
        | TriggerFilter::OwnerDyingZhupeilanOncePerGame
        | TriggerFilter::OwnerPhaseEndDcsbhaoyiAvailable
        | TriggerFilter::OwnerAliveGlobalFireDamageDcsbronghuo
        | TriggerFilter::OwnerHandTransferOtherDcsbmengmouBranchAvailable
        | TriggerFilter::OwnerIsTargetCardDamageBefore => {
            filters_owner_alive_turn_ended_rehengjiang_pending_owner::filter_match(
                state,
                active,
                ctx,
                view_as_skill,
                view_as_card,
                history,
                &def,
            )
        }
        TriggerFilter::OwnerSourceQiaojianExtraAvailable
        | TriggerFilter::OwnerSourceWoundedExistsOncePerTurn
        | TriggerFilter::OwnerSourceFirstDamageCardTargetHandGteOwner
        | TriggerFilter::OwnerSourceShaResolvedTrackedPotceduanDealtDamage
        | TriggerFilter::OwnerSourceShaResolvedTrackedPotzhanlieDraw
        | TriggerFilter::OwnerSourceSingleTargetShaOrBlackOrdinaryTrickCurrentPlayPhaseUnused
        | TriggerFilter::OwnerIsEventPlayerMaleSourceHasDiscardableEquipment
        | TriggerFilter::OwnerAliveOtherTurnEndedNoCardTargetedOther
        | TriggerFilter::OwnerAliveOtherTurnEndedNoDamageOther
        | TriggerFilter::OwnerAliveOtherTurnEndedCardTargetedOtherNoDamageOther
        | TriggerFilter::OwnerSourceShaHasWeapon
        | TriggerFilter::OwnerSourceShaDirectDamageBefore
        | TriggerFilter::OwnerSourceOtherDamageBefore
        | TriggerFilter::OwnerSourceOtherDamageBeforeOncePerTurn
        | TriggerFilter::OwnerSourceTargetEarlierSeatHandBelowTargetMaxHpOncePerTurn
        | TriggerFilter::OwnerSourceOtherDirectDamageBefore
        | TriggerFilter::OwnerSourceOrTargetHandCountDiffersOncePerTurn
        | TriggerFilter::OwnerIsEventPlayerAnyAliveHandNonempty
        | TriggerFilter::OwnerIsEventPlayerAnyAliveHandEmpty
        | TriggerFilter::OwnerIsEventPlayerOtherHandLteOwner
        | TriggerFilter::OwnerIsEventPlayerDrawReshuffleZeroWounded
        | TriggerFilter::OwnerIsEventPlayerDrawReshufflePositive
        | TriggerFilter::OwnerAliveOtherJudgePhaseInRangeHasJudgeCard
        | TriggerFilter::OwnerSourceNonEquipNoBasicHandHasTargets
        | TriggerFilter::OwnerAliveSourceAliveOncePerRound
        | TriggerFilter::OwnerAliveSourceAliveEventPlayerDyingOncePerRound
        | TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipGteTwoOncePerRound
        | TriggerFilter::OwnerIsEventPlayerOtherSourceAliveHandEquipNonemptyOldHuishengUnused
        | TriggerFilter::OwnerAliveOtherDyingOutsideTurnCanShaCurrent
        | TriggerFilter::OwnerSourceOldJiefanSha
        | TriggerFilter::OwnerSourceCurrentTurnFirstTargetNonEquip
        | TriggerFilter::OwnerSourceFaceDownViewAsChenweiAvailable
        | TriggerFilter::OwnerSourceShaOrJuedouSingleTargetFirstSameNameThisTurn
        | TriggerFilter::OwnerSourceShaOrJuedouSingleTargetHandNonempty
        | TriggerFilter::OwnerSourceShaDamageBeforeEndpointDiscardableHandEquip
        | TriggerFilter::OwnerAliveEventPlayerAlive
        | TriggerFilter::OwnerAliveEventPlayerEquipEmpty
        | TriggerFilter::OwnerAliveEventPlayerFromEquipEmpty
        | TriggerFilter::OwnerAliveEventPlayerHandEmpty
        | TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmpty
        | TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmptyOncePerTurn
        | TriggerFilter::OwnerAliveEventPlayerDistanceLeOwnerHp
        | TriggerFilter::OwnerAliveOtherNormalDamageTargetUnlinked
        | TriggerFilter::OwnerAliveOtherCurrentSourceRepeatsCardName
        | TriggerFilter::OwnerAliveSourceTargetSameSex
        | TriggerFilter::OwnerAliveEventPlayerSameGroup
        | TriggerFilter::OwnerAliveEventPlayerSameGroupHandNonempty
        | TriggerFilter::OwnerAliveOtherPlayPhaseTiedMaxHandDcsbfuxiAvailable
        | TriggerFilter::OwnerSourceFireShaOrHuogongDcsbronghuo
        | TriggerFilter::OwnerSourceCardResolvedHasOtherTargetDcsbyingmouOncePerTurn
        | TriggerFilter::OwnerSourceSingleTargetShaOrTrickAllMaterialsFromHandCanSha => {
            filters_owner_alive_turn_ended_rehengjiang_pending_owner_source_other_event::filter_match(
                state,
                active,
                ctx,
                view_as_skill,
                view_as_card,
                history,
                &def,
            )
        }
        TriggerFilter::CilvTrackedCardDamageBefore
        | TriggerFilter::CilvTrackedCardResolved
        | TriggerFilter::DcsbyingmouTrackedCardResolved
        | TriggerFilter::DcsbmengmouTrackedShaDamage
        | TriggerFilter::DcsbmengmouTrackedShaResolved => {
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
        TriggerFilter::DcyinjunTrackedShaResolved => {
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
        TriggerFilter::OwnerSourceOwnViewAsCardTargetedTargetSkillPile(..)
        | TriggerFilter::OwnerSourceShaDamageBeforeTargetHealthy
        | TriggerFilter::OwnerAliveEventCardsNonempty
        | TriggerFilter::OwnerAliveEventPlayerGroup(..)
        | TriggerFilter::OwnerIsEventPlayerNoClanLostHandThisTurn(..)
        | TriggerFilter::OwnerAliveEventPlayerSelfOrClan(..)
        | TriggerFilter::OwnerIsEventPlayerWounded
        | TriggerFilter::OwnerIsEventPlayerLinked
        | TriggerFilter::OwnerIsEventPlayerAliveWoundedOrGuimingWuExists
        | TriggerFilter::OwnerSourceShaOrOrdinaryTrickTargetWoundedHasDiscardableHandEquip
        | TriggerFilter::OwnerSourceShaOrOrdinaryTrickHasDiscardableHandEquip
        | TriggerFilter::OwnerSourceShaOrOrdinaryTrick
        | TriggerFilter::OwnerSourceWuxieUsed
        | TriggerFilter::OwnerSourceShaOncePerTurn
        | TriggerFilter::OwnerSourceShaRepeatsPreviousShaTargets
        | TriggerFilter::OwnerSourcePlayCardRepeatsPreviousTargets
        | TriggerFilter::OwnerIsEventPlayerHandEquipJudgeAboveHp
        | TriggerFilter::OwnerSourceJiuBoostedShaDamageSkillEnabled(..)
        | TriggerFilter::OwnerAliveEventPlayerOwnerOrFemale
        | TriggerFilter::OwnerIsEventPlayerNotMinHp
        | TriggerFilter::OwnerIsEventPlayerMinHand
        | TriggerFilter::OwnerIsEventPlayerNotMinHand
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
        | TriggerFilter::OwnerSourceThunder
        | TriggerFilter::OwnerSourceOtherTwicePerTurn
        | TriggerFilter::OwnerSourceWoundedDistanceLeOne
        | TriggerFilter::OwnerSourceDistanceLeOnePerDamagePoint
        | TriggerFilter::OwnerSourceTargetDistanceGtOne
        | TriggerFilter::OwnerSourceTargetNotGroup(..)
        | TriggerFilter::OwnerSourceOrTargetedByJuedouOrRedSha
        | TriggerFilter::OwnerSourceOrTargetedByJuedou
        | TriggerFilter::OwnerSourceOrTargetedByShaEqualHp
        | TriggerFilter::OwnerSourceOrTargetedByOtherDiamond
        | TriggerFilter::OwnerSourceOrTargetedClubLinkedExistsOncePerTurn
        | TriggerFilter::OwnerSourceShaSingleTrickSourceCard
        | TriggerFilter::OwnerSourceShaSingleEquipSourceCard
        | TriggerFilter::OwnerSourceRed => {
            filters_dcyinjun_tracked_sha_resolved_owner_event_source::filter_match(
                state,
                active,
                ctx,
                view_as_skill,
                view_as_card,
                history,
                &def,
            )
        }
        TriggerFilter::OwnerAliveOtherRecoveredOutsideOwnerTurn
        | TriggerFilter::OwnerIsTargetDiscardedShaInDiscardPhase
        | TriggerFilter::OwnerPhaseEndOriginatingSkillPileEmpty
        | TriggerFilter::OwnerIsTargetShaSkillPileNonempty(..)
        | TriggerFilter::OwnerAliveOtherPlayDiscardSkillPileNonempty(..)
        | TriggerFilter::OwnerPrepareGroupOutsideAttackRangeExists
        | TriggerFilter::OwnerAliveTargetedByOther
        | TriggerFilter::OwnerAliveTargetedByOtherBlackCard
        | TriggerFilter::OwnerAliveTargetedByOtherRedCard
        | TriggerFilter::OwnerAliveOtherSourceDeath
        | TriggerFilter::OwnerIsTargetPlayPhaseHandNoSha
        | TriggerFilter::OwnerAliveOtherPhaseEndPlayerOutsideOwnerRange
        | TriggerFilter::OwnerIsRespondTo
        | TriggerFilter::OwnerAliveOtherDamageSourceGroup(..)
        | TriggerFilter::OwnerDyingOtherTaoResponderGroup(..)
        | TriggerFilter::OwnerAliveOtherJudgmentBlackEventPlayerGroup(..)
        | TriggerFilter::OwnerWoundedNotEventSource
        | TriggerFilter::OwnerIsTargetChainDamageLinked
        | TriggerFilter::OwnerIsTargetWoundedSha
        | TriggerFilter::OwnerIsTargetHealthyDrawNotFixed
        | TriggerFilter::OwnerWoundedEventPlayerGroup(..)
        | TriggerFilter::OwnerAliveNotEventPlayer
        | TriggerFilter::OwnerAliveNotEventPlayerHandEquipNonempty
        | TriggerFilter::OwnerEquipZoneChanged
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
        | TriggerFilter::OwnerShaUsedAtLeastTwo
        | TriggerFilter::OwnerAliveUsedShaThisTurn
        | TriggerFilter::OwnerMaxHpGtOneHandOrEquipEmpty
        | TriggerFilter::OwnerAlivePlayPhaseCardCategoryCountPositive
        | TriggerFilter::OwnerAlivePlayPhaseShaUserHasDiscardableHandEquip
        | TriggerFilter::OwnerTargetFire
        | TriggerFilter::OwnerTargetThunder
        | TriggerFilter::OwnerAliveOtherSourceShaTwicePerTurn
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
        | TriggerFilter::OwnerAliveGlobalJuedouOrRedShaUsed
        | TriggerFilter::OwnerAliveOtherGlobalJuedouOrRedShaUsed
        | TriggerFilter::OwnerSkillPileNonempty(..)
        | TriggerFilter::OwnerLostOneHandEquipJudgeOtherCurrentPhaseOnce
        | TriggerFilter::OwnerIsTargetSingleTargetOtherSourceOncePerTurn => {
            filters_dcyinjun_tracked_sha_resolved_owner::filter_match(
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
        | TriggerFilter::OwnerSourceMatchesPreviousSuitOrRank
        | TriggerFilter::OwnerSourcePlaySkillMarkMode(..)
        | TriggerFilter::OwnerSourceHandBasicEqualsNonBasic
        | TriggerFilter::OwnerSourceOrWithinDistanceOneShaOrJuedou
        | TriggerFilter::OwnerSourceSha
        | TriggerFilter::OwnerSourceHeartSha
        | TriggerFilter::OwnerSourceShaFirstTarget
        | TriggerFilter::OwnerSourceShaOrJuedouSingleTarget
        | TriggerFilter::OwnerAliveOtherSingleTargetJuedouOrRedSha
        | TriggerFilter::OwnerSourceNormalSha
        | TriggerFilter::OwnerSourceFireSha
        | TriggerFilter::OwnerSourceSingleTargetShaOrOrdinaryTrickCurrentTurn
        | TriggerFilter::OwnerSourceEventCardPolicyDraw
        | TriggerFilter::OwnerSourcePlayShaOtherAliveOncePerPlayPhase
        | TriggerFilter::OwnerSourcePlayCardCountEquals(..)
        | TriggerFilter::OwnerSourcePlayCardCountEqualsHp
        | TriggerFilter::OwnerAliveSourceShaHandEquipSkillPile(..)
        | TriggerFilter::OwnerAliveOtherSourceSingleShaTargetOther
        | TriggerFilter::OwnerAliveOtherSourceSingleShaTargetOtherOwnerInAttackRange
        | TriggerFilter::OwnerAliveOtherSourceShaResolvedTwxiaolianRedirectHasHandEquip
        | TriggerFilter::OwnerSourceJiu
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
        | TriggerFilter::OwnerAliveOtherShaTargetCanRedirectToOwner
        | TriggerFilter::OwnerIsSourceShaUsed
        | TriggerFilter::OwnerIsSourceShaCardDamageBefore
        | TriggerFilter::OwnerSourceBasicOrTrick
        | TriggerFilter::OwnerSourcePhysicalOrdinaryTrick
        | TriggerFilter::OwnerAliveSourceFirstCardOrdinaryTrick
        | TriggerFilter::OwnerSourceMatchesLastUsedSuitOrRank
        | TriggerFilter::OwnerSourceOncePerTurn
        | TriggerFilter::OwnerSourceTargetAliveHasEquip
        | TriggerFilter::OwnerSourceBlackShaTargetAliveHasDiscardableEquip
        | TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableRegionCard
        | TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableHandEquip
        | TriggerFilter::OwnerSourceTiesuoTargetAliveUnlinkedHandEquipNonempty
        | TriggerFilter::OwnerSourceShaTargetAliveHandNonempty
        | TriggerFilter::OwnerSourceShaTargetAliveDistanceLeOne
        | TriggerFilter::OwnerAliveOtherGainedSingleEquipToHandOutsideTurnUsable
        | TriggerFilter::OwnerSourcePlayPhaseShaHasAdditionalTargetIgnoringDistance
        | TriggerFilter::OwnerSourceCardResolvedTrackedDddXianxi
        | TriggerFilter::OwnerSourceCardResolvedDddXuyuPending
        | TriggerFilter::OwnerAliveOtherSourceSecondPlayCardResolvedHandEquipNonempty
        | TriggerFilter::OwnerAliveOtherPhaseBeginBothHandNonempty
        | TriggerFilter::OwnerAliveOtherPhaseBeginUnshownHandGteHp
        | TriggerFilter::OwnerAliveOtherPhasePlayDddChashiAvailable => {
            filters_owner_alive_other_marked_xi_owner_source_other::filter_match(
                state,
                active,
                ctx,
                view_as_skill,
                view_as_card,
                history,
                &def,
            )
        }
        TriggerFilter::OwnerAliveSkillBoundTargetIsEventPlayer
        | TriggerFilter::OwnerIsEventPlayerEventCardSha
        | TriggerFilter::OwnerIsEventPlayerEventCardShaOrTrick
        | TriggerFilter::OwnerTargetSha
        | TriggerFilter::OwnerAliveGainedThisTurnCanCompareOther
        | TriggerFilter::OwnerAliveCardTargetedMarkedPeer(..)
        | TriggerFilter::OwnerIsEventPlayerHandShaSkillPileEmpty
        | TriggerFilter::OwnerAliveDyingHasSkillPileCards
        | TriggerFilter::OwnerIsEventPlayerSourceAliveMarkPositive(..)
        | TriggerFilter::OwnerIsEventPlayerAnyFieldCardNoSkillPile(..)
        | TriggerFilter::OwnerAliveDyingSkillPile(..)
        | TriggerFilter::OwnerIsEventPlayerTwxiaolianRedirectedDamage
        | TriggerFilter::OwnerIsEventPlayerOtherHigherHandOncePerGame
        | TriggerFilter::OwnerIsEventPlayerHpOdd
        | TriggerFilter::OwnerIsEventPlayerHpEvenWounded
        | TriggerFilter::OwnerAlivePreparePlayerHpGtOwner
        | TriggerFilter::OwnerAlivePrepareLordHpGtOwner
        | TriggerFilter::OwnerShaTargetCanDiscardRedirectToOtherInAttackRange
        | TriggerFilter::OwnerShaTargetCanDiscardRedirectToZhengjianMarked
        | TriggerFilter::OwnerIsTargetShaSourceAliveHandEquipNonemptyOncePerTurn
        | TriggerFilter::OwnerIsTargetTrickTargetsGtSourceHpHandEquipNonempty
        | TriggerFilter::OwnerIsResponderSha
        | TriggerFilter::OwnerIsTargetDamageAmountGtOne
        | TriggerFilter::OwnerIsTargetSourceHpGtOwnerPerRoleTurn
        | TriggerFilter::OwnerIsEventPlayerCanCompareOther
        | TriggerFilter::OwnerHandBelowLostHpAfterRelevantChange
        | TriggerFilter::OwnerAliveEventPlayerPlayPhaseSameSuitAtLeastTwoOncePerTurn
        | TriggerFilter::OwnerIsEventPlayerDiscardPileFourSuitsBelowAliveCap
        | TriggerFilter::OwnerIsEventPlayerOncePerTurn
        | TriggerFilter::OwnerAliveShaTargetDistanceLeOne
        | TriggerFilter::OwnerAliveShaTargetDistanceLeOneHasEquipCard
        | TriggerFilter::OwnerIsEventPlayerShaSourceAliveHandEquipNonempty
        | TriggerFilter::OwnerAliveWdbFuchouPendingLiving
        | TriggerFilter::OwnerAliveWuguRemainedInDiscard
        | TriggerFilter::OwnerIsTargetNonMultitargetTrickMultipleTargets
        | TriggerFilter::OwnerAliveHpChangedThisTurnEnteredDiscardAvailable
        | TriggerFilter::OwnerAliveDddDongchaPileNonempty
        | TriggerFilter::OwnerPlayDddZhengjunHpAvailable
        | TriggerFilter::OwnerPlayDddZhengjunCardsMovedAvailable
        | TriggerFilter::OwnerAliveDddQiceReady
        | TriggerFilter::OwnerAliveDddFenjiAvailable => {
            filters_owner_alive_other_marked_xi_owner::filter_match(
                state,
                active,
                ctx,
                view_as_skill,
                view_as_card,
                history,
                &def,
            )
        }
        TriggerFilter::OncePerTurn
        | TriggerFilter::DddXuyuPending
        | TriggerFilter::DddShijianWatchedSourceCardUsedTargeted
        | TriggerFilter::DddShijianPending
        | TriggerFilter::DddChashiMatchingCard => {
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
        | TriggerFilter::OwnerAliveDddXuanlunPending
        | TriggerFilter::OwnerAliveDddJiexingRecoverOncePerGame
        | TriggerFilter::OwnerAliveDddJiexingEquipOncePerGame
        | TriggerFilter::OwnerAliveDddJiexingRecoveryReset
        | TriggerFilter::OwnerAliveDddJiexingEquipReset
        | TriggerFilter::OwnerAliveDddBaileiAvailable
        | TriggerFilter::OwnerAliveDddTongyuRemained
        | TriggerFilter::OwnerIsEventPlayerJudgeNonempty
        | TriggerFilter::OwnerIsEventPlayerDrawNotReplaced
        | TriggerFilter::OwnerIsEventPlayerDrawNotFixedHpOneOrHandEmpty
        | TriggerFilter::OwnerAliveEventPlayerTurnChangedOrBecameLinked
        | TriggerFilter::OwnerAliveEventPlayerBecameFaceUpOrLinked
        | TriggerFilter::OwnerIsEventPlayerNoEnabledEquipSlotOncePerGame
        | TriggerFilter::OwnerIsEventPlayerHasMovableEquipment
        | TriggerFilter::OwnerIsEventPlayerTrickUsedZero
        | TriggerFilter::OwnerIsEventPlayerHpLeOne
        | TriggerFilter::OwnerIsEventPlayerHpLeTwo
        | TriggerFilter::OwnerIsEventPlayerAnyAliveSelfDiscardableHandEquip
        | TriggerFilter::OwnerIsEventPlayerAnyAliveDiscardableByOwnerHandEquip
        | TriggerFilter::OwnerIsEventPlayerAnyAliveSelfDiscardableHand
        | TriggerFilter::OwnerIsEventPlayerAnyAliveWounded
        | TriggerFilter::OwnerIsEventPlayerHandEquipNonemptyOtherAlive
        | TriggerFilter::OwnerIsEventPlayerHandNonemptyHiddenSkillPile(..)
        | TriggerFilter::OwnerIsEventPlayerCurrentTurn
        | TriggerFilter::OwnerAliveEventPlayerLord
        | TriggerFilter::OwnerAliveEventPlayerLordNoDamage
        | TriggerFilter::OwnerAliveEventPlayerLordDamaged
        | TriggerFilter::OwnerAliveEventTargetMarkPositiveNoDamage(..)
        | TriggerFilter::OwnerAliveEventTargetMarkPositive(..) => {
            filters_owner_alive_ddd_fengzheng_round_pending_ddd_event::filter_match(
                state,
                active,
                ctx,
                view_as_skill,
                view_as_card,
                history,
                &def,
            )
        }
        TriggerFilter::OwnerIsTargetFaceDown
        | TriggerFilter::OwnerSourceOwnViewAsDddTongyuUsed
        | TriggerFilter::OwnerSourceOwnViewAsDddTongyuTargeted
        | TriggerFilter::OwnerAliveSourceBasicDddFushiAvailable
        | TriggerFilter::OwnerSourceShaTargetHandGteHpOrLteAttackRange
        | TriggerFilter::OwnerSourceShaTargetHandLteOwnerOrHpGteOwner
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
        | TriggerFilter::OwnerAliveOtherDyingHandNonemptyOncePerGame
        | TriggerFilter::OwnerDyingOtherHandEquipNonemptyOncePerGame
        | TriggerFilter::OwnerAliveOtherDyingFirstPerTarget
        | TriggerFilter::OwnerSourceOrTargetEventPlayerSecondDamageThisTurn
        | TriggerFilter::OwnerSourceOrTargetEventPlayerFirstDamageThisTurn
        | TriggerFilter::OwnerSourceTargetDistanceLeOneFirstDamageToTargetThisTurn
        | TriggerFilter::OwnerFirstTransformedShaUsedThisTurn
        | TriggerFilter::OwnerTargetHandEmptyElementalOrTrickDamageBefore
        | TriggerFilter::OwnerAliveDrawPlayerOwnerOrMinHpTwjiaohuaAvailable
        | TriggerFilter::OwnerAliveShaOrDuelDamageSourceTargetAliveHandEquipNonempty
        | TriggerFilter::OwnerAliveShaDamageSourceTargetAliveOwnerHandEquipNonempty
        | TriggerFilter::OwnerAliveGainedThreeThisTurnOtherAlive
        | TriggerFilter::OwnerAliveGainedTwoThisTurnCanCompareOther
        | TriggerFilter::OwnerAliveGaveCardsToOtherAlive
        | TriggerFilter::OwnerAliveTargetedByOtherHiddenSkillPile(..)
        | TriggerFilter::OwnerLostMatchingRevealedSkillPileSuit(..)
        | TriggerFilter::OwnerAliveGroupCountAtMostTwo
        | TriggerFilter::OwnerAliveCurrentTurn
        | TriggerFilter::DebateOwnerIsInitiator
        | TriggerFilter::DebateOwnerParticipated
        | TriggerFilter::OwnerDebateParticipant
        | TriggerFilter::OwnerIsLord
        | TriggerFilter::OwnerAliveTurnUsedAllSuits
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
        | TriggerFilter::OwnerTargetedSourceHpHandDifferOncePerTurn
        | TriggerFilter::CardsMovedDiscardAtLeastThree
        | TriggerFilter::OwnerAliveMarkedAliveAboveTwo(..)
        | TriggerFilter::CardsMovedIntoExpansion
        | TriggerFilter::CardsMovedOutOfExpansion
        | TriggerFilter::OwnerAliveDiscardMatchesOwnPileCard(..)
        | TriggerFilter::OwnerAliveSomeoneMarkPositive(..)
        | TriggerFilter::OwnerAliveShaResolvedCity(..)
        | TriggerFilter::OwnerAliveTargetedByOtherShaJudgeXumou
        | TriggerFilter::OwnerCardsMovedContainYing
        | TriggerFilter::DebateOwnerInitiatorResultRed
        | TriggerFilter::DebateOwnerInitiatorResultBlack
        | TriggerFilter::DebateOwnerOpinionDiffersInitiator
        | TriggerFilter::OwnerLostPackageCardFromEquip
        | TriggerFilter::OwnerIsEventPlayerWoundedDeadAlly
        | TriggerFilter::OwnerIsEventPlayerPhaseEndHandEquipNonemptyOtherAlive
        | TriggerFilter::OwnerSkillAddedSelf(..)
        | TriggerFilter::OwnerLostHandNowEmpty
        | TriggerFilter::OwnerLostSpadeTwoToNine
        | TriggerFilter::OwnerAliveShanUsedByOwnerOrDuringOwnerTurn
        | TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipOncePerRound
        | TriggerFilter::OwnerAliveFirstNonDiscardPhaseDiscardMbrenxingAvailable
        | TriggerFilter::OwnerLostHandOwnerlessShaNotUseOncePerTurn
        | TriggerFilter::OwnerAlivePhaseAnyEndOriginatingSkillPileNonempty
        | TriggerFilter::OwnerMbrunweiLossThreshold
        | TriggerFilter::OwnerPhasePlayEndMbrunweiTrackedHand
        | TriggerFilter::OwnerAliveDamageBeforeTargetDistanceLeOneOncePerTurn
        | TriggerFilter::OwnerPhasePlayEndMbxiongtuState
        | TriggerFilter::OwnerPhaseEndMbganggengTargetAlive
        | TriggerFilter::OwnerFromHandEmptyOrDyingMbsijianTwicePerTurn
        | TriggerFilter::OwnerIsTargetCanMoveFieldCardMbyuanmoTwicePerTurn
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
        | TriggerFilter::OwnerPhasePlayEndPotguanshaAvailableOncePerGame
        | TriggerFilter::OwnerDiscardedShaToDiscard
        | TriggerFilter::OwnerIsTargetCanUseWuzhong
        | TriggerFilter::OwnerAliveDiscardBatch
        | TriggerFilter::OwnerTargetDamagePotjianzhuanAvailable
        | TriggerFilter::OwnerAliveDyingPotjianzhuanAvailable
        | TriggerFilter::OwnerTargetDamagePotcaoshuangAssignedUntriggered
        | TriggerFilter::OwnerAliveDyingPotcaoshuangAssignedUntriggered
        | TriggerFilter::OwnerUsedPotnizunPileCard
        | TriggerFilter::OwnerTargetDamageBeforePotnizunUsed
        | TriggerFilter::OwnerAliveTurnEndedUsedCardTaggedDiscard
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
        TriggerFilter::OwnerAliveOtherDyingHasEquipCard
        | TriggerFilter::OwnerSourceTrickResolvedOwnPlayPhaseOncePerTurn
        | TriggerFilter::OwnerSourceHandGaintagAbsent(..)
        | TriggerFilter::OwnerSourceDamageTargetMarkPositiveOnce(..)
        | TriggerFilter::OwnerAliveOtherTargetedBySha
        | TriggerFilter::OwnerAliveOtherSourceDamage
        | TriggerFilter::OwnerAliveSourceAllyShaDamageBefore
        | TriggerFilter::OwnerSourceShaDamageTargetAliveHasDiscardableEquip
        | TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableEquip
        | TriggerFilter::OwnerSourceCardUsedTargetsSelf
        | TriggerFilter::OwnerSourceCurrentPhaseCardTargetedOtherHealthy
        | TriggerFilter::OwnerSourceFirstCardSuitThisTurn
        | TriggerFilter::OwnerSourceCardDamageTargetCanReceiveOwnerBingliang
        | TriggerFilter::OwnerSourceEquipUsedSkillMarkBelowSeven
        | TriggerFilter::OwnerSourceOutsideTurnCardUsedOrResponded
        | TriggerFilter::OwnerAliveOtherPhasePlaySkillPileNonempty
        | TriggerFilter::OwnerAliveOtherSourceDamageCardResolvedTargetedDamaged
        | TriggerFilter::OwnerAliveOtherSourceDamageCardResolvedTargetedNotDamaged
        | TriggerFilter::OwnerAliveOtherDyingMbchensheAvailable
        | TriggerFilter::OwnerSourceFirstTargetHandDamageCardDiscardable
        | TriggerFilter::OwnerSourceDamageTrackedMbduanyangTargetRecastable
        | TriggerFilter::OwnerSourceDamageMbxiongtuArmed
        | TriggerFilter::OwnerSourceFirstSingleTargetShaMbfeijingTwicePerTurn
        | TriggerFilter::OwnerSourceDamageBeforeMbfeijingExtra
        | TriggerFilter::OwnerSourceShaResolvedOriginalSingleMbxiaoge
        | TriggerFilter::OwnerSourceCardResolvedPhasePolicy
        | TriggerFilter::OwnerAliveOtherTurnDiscardedRecordedPotfuanSuit
        | TriggerFilter::OwnerSourceShaResolvedPotfuanDealtDamage
        | TriggerFilter::OwnerAliveOtherPlayPhasePothuilvAvailable
        | TriggerFilter::OwnerAliveOtherRespondedToOwner
        | TriggerFilter::OwnerSourceDamagePotjianzhuanAvailable
        | TriggerFilter::OwnerSourceDamagePotcaoshuangAssignedUntriggered
        | TriggerFilter::OwnerSourceCardName(..) => {
            filters_owner_black_card_moved_equip_judge_source_other::filter_match(
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
