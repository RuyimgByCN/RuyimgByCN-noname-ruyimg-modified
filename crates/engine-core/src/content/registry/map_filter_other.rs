//! 触发过滤器名映射链：其他角色、事件、阶段、DDD 与被指定类过滤器（链尾）。

use super::*;

pub(super) fn map_filter(s: &str) -> Option<TriggerFilter> {
    Some(match s {
        "owner_alive_event_player_lost_armor_or_mount_has_originating_skill_pile" => {
            TriggerFilter::OwnerAliveEventPlayerLostArmorOrMountHasOriginatingSkillPile
        }
        "owner_alive_event_player_hand_nonempty_once_per_round" => {
            TriggerFilter::OwnerAliveEventPlayerHandNonemptyOncePerRound
        }
        "owner_alive_phase_play_end_zhengbi_pending" => {
            TriggerFilter::OwnerAlivePhasePlayEndZhengbiPending
        }
        "owner_alive_other_dying_outside_turn_can_sha_current" => {
            TriggerFilter::OwnerAliveOtherDyingOutsideTurnCanShaCurrent
        }
        "owner_phase_gift_target_is_source" => TriggerFilter::OwnerPhaseGiftTargetIsEventSource,
        "owner_phase_gift_target_is_target" => TriggerFilter::OwnerPhaseGiftTargetIsEventPlayer,
        "owner_alive_other_source_sha_resolved_not_target_gainable_once_per_turn" => {
            TriggerFilter::OwnerAliveOtherSourceShaResolvedNotTargetGainableOncePerTurn
        }
        "owner_alive_phase_end_lost_cards_at_least_two" => {
            TriggerFilter::OwnerAlivePhaseEndLostCardsAtLeastTwo
        }
        "owner_alive_other_phase_play_end_twzhengjian_condition" => {
            TriggerFilter::OwnerAliveOtherPhasePlayEndTwzhengjianCondition
        }
        "owner_alive_phase_any_end_originating_skill_pile_nonempty" => {
            TriggerFilter::OwnerAlivePhaseAnyEndOriginatingSkillPileNonempty
        }
        "owner_phase_play_end_mbrunwei_tracked_hand" => {
            TriggerFilter::OwnerPhasePlayEndMbrunweiTrackedHand
        }
        "owner_phase_play_end_mbxiongtu_state" => TriggerFilter::OwnerPhasePlayEndMbxiongtuState,
        "owner_phase_end_mbganggeng_target_alive" => {
            TriggerFilter::OwnerPhaseEndMbganggengTargetAlive
        }
        "owner_phase_play_mbkuangxiang_active" => TriggerFilter::OwnerPhasePlayMbkuangxiangActive,
        "owner_alive_phase_end_player_potsifeng_pile" => {
            TriggerFilter::OwnerAlivePhaseEndPlayerPotsifengPile
        }
        "owner_alive_other_twkujian_used_or_responded" => {
            TriggerFilter::OwnerAliveOtherTwkujianUsedOrResponded
        }
        "owner_alive_other_twkujian_lost" => TriggerFilter::OwnerAliveOtherTwkujianLost,
        "owner_alive_other_phase_prepare_can_compare_owner" => {
            TriggerFilter::OwnerAliveOtherPhasePrepareCanCompareOwner
        }
        "owner_alive_other_discarded_in_discard_phase" => {
            TriggerFilter::OwnerAliveOtherDiscardedInDiscardPhase
        }
        "owner_alive_other_wu_play_phase_can_compare_owner" => {
            TriggerFilter::OwnerAliveOtherWuPlayPhaseCanCompareOwner
        }
        "owner_alive_other_play_phase_zhengsu_missing_in_range" => {
            TriggerFilter::OwnerAliveOtherPlayPhaseZhengsuMissingInRange
        }
        "owner_alive_other_play_phase_discardable_gte_mark" => {
            TriggerFilter::OwnerAliveOtherPlayPhaseDiscardableGteMark
        }
        "owner_alive_other_trick_first_target_source_max_hand" => {
            TriggerFilter::OwnerAliveOtherTrickFirstTargetSourceMaxHand
        }
        "owner_alive_other_marked_xi" => TriggerFilter::OwnerAliveOtherMarkedXi,
        "owner_alive_other_recovered_outside_owner_turn" => {
            TriggerFilter::OwnerAliveOtherRecoveredOutsideOwnerTurn
        }
        "owner_phase_end_originating_skill_pile_empty" => {
            TriggerFilter::OwnerPhaseEndOriginatingSkillPileEmpty
        }
        "owner_alive_other_nanman_cards_discard" => {
            TriggerFilter::OwnerAliveOtherNanmanCardsDiscard
        }
        "owner_targeted_by_nanman" => TriggerFilter::OwnerTargetedByNanman,
        "owner_targeted_source_hp_hand_differ_once_per_turn" => {
            TriggerFilter::OwnerTargetedSourceHpHandDifferOncePerTurn
        }
        "owner_alive_other_dying_has_equip_card" => TriggerFilter::OwnerAliveOtherDyingHasEquipCard,
        "owner_phase_begin_skill_pile_gte_three" => {
            TriggerFilter::OwnerPhaseBeginSkillPileGteThree(0)
        }
        "owner_phase_begin_hand_empty" => TriggerFilter::OwnerPhaseBeginHandEmpty,
        "owner_alive_other_qun_play_phase_can_give_owner" => {
            TriggerFilter::OwnerAliveOtherQunPlayPhaseCanGiveOwner
        }
        "owner_alive_other_turn_ended_no_card_targeted_other" => {
            TriggerFilter::OwnerAliveOtherTurnEndedNoCardTargetedOther
        }
        "owner_alive_other_turn_ended_no_damage_other" => {
            TriggerFilter::OwnerAliveOtherTurnEndedNoDamageOther
        }
        "owner_alive_other_turn_ended_card_targeted_other_no_damage_other" => {
            TriggerFilter::OwnerAliveOtherTurnEndedCardTargetedOtherNoDamageOther
        }
        "owner_alive_phase_targeted_hp_changed_can_use_wanjian" => {
            TriggerFilter::OwnerAlivePhaseTargetedHpChangedCanUseWanjian
        }
        "owner_alive_phase_targeted_hp_unchanged_can_use_juedou" => {
            TriggerFilter::OwnerAlivePhaseTargetedHpUnchangedCanUseJuedou
        }
        "owner_alive_other_play_phase_duplicate_card_category_has_hand_equip" => {
            TriggerFilter::OwnerAliveOtherPlayPhaseDuplicateCardCategoryHasHandEquip
        }
        "owner_alive_other_phase_end_damage_gt_one_has_hand_equip" => {
            TriggerFilter::OwnerAliveOtherPhaseEndDamageGtOneHasHandEquip
        }
        "owner_alive_other_phase_end_has_discardable_basic_hand" => {
            TriggerFilter::OwnerAliveOtherPhaseEndHasDiscardableBasicHand
        }
        "owner_alive_other_phase_end_hand_nonempty" => {
            TriggerFilter::OwnerAliveOtherPhaseEndHandNonempty
        }
        "owner_alive_other_sha_target_can_redirect_to_owner" => {
            TriggerFilter::OwnerAliveOtherShaTargetCanRedirectToOwner
        }
        "owner_phase_end_lost_cards" => TriggerFilter::OwnerPhaseEndLostCards,
        "owner_alive_other_source_single_sha_target_other" => {
            TriggerFilter::OwnerAliveOtherSourceSingleShaTargetOther
        }
        "owner_alive_other_source_single_sha_target_other_owner_in_attack_range" => {
            TriggerFilter::OwnerAliveOtherSourceSingleShaTargetOtherOwnerInAttackRange
        }
        "owner_alive_other_source_sha_resolved_twxiaolian_redirect_has_hand_equip" => {
            TriggerFilter::OwnerAliveOtherSourceShaResolvedTwxiaolianRedirectHasHandEquip
        }
        "owner_alive_other_sha_resolved_tracked_skill_card_dealt_damage" => {
            TriggerFilter::OwnerAliveOtherShaResolvedTrackedSkillCardDealtDamage
        }
        "owner_alive_other_sha_resolved_tracked_skill_card_no_damage" => {
            TriggerFilter::OwnerAliveOtherShaResolvedTrackedSkillCardNoDamage
        }
        "owner_alive_event_player_alive" => TriggerFilter::OwnerAliveEventPlayerAlive,
        "owner_alive_event_player_equip_empty" => TriggerFilter::OwnerAliveEventPlayerEquipEmpty,
        "owner_alive_event_player_from_equip_empty" => {
            TriggerFilter::OwnerAliveEventPlayerFromEquipEmpty
        }
        "owner_alive_event_player_hand_empty" => TriggerFilter::OwnerAliveEventPlayerHandEmpty,
        "owner_alive_event_cards_nonempty" => TriggerFilter::OwnerAliveEventCardsNonempty,
        "owner_alive_event_player_outside_turn_from_hand_empty" => {
            TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmpty
        }
        "owner_alive_event_player_outside_turn_from_hand_empty_once" => {
            TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmptyOncePerTurn
        }
        "owner_alive_event_player_turn_changed_or_became_linked" => {
            TriggerFilter::OwnerAliveEventPlayerTurnChangedOrBecameLinked
        }
        "owner_alive_event_player_became_face_up_or_linked" => {
            TriggerFilter::OwnerAliveEventPlayerBecameFaceUpOrLinked
        }
        "owner_alive_event_player_distance_le_owner_hp" => {
            TriggerFilter::OwnerAliveEventPlayerDistanceLeOwnerHp
        }
        "owner_alive_other_normal_damage_target_unlinked" => {
            TriggerFilter::OwnerAliveOtherNormalDamageTargetUnlinked
        }
        "owner_alive_event_player_owner_or_female" => {
            TriggerFilter::OwnerAliveEventPlayerOwnerOrFemale
        }
        "owner_alive_event_player_same_group" => TriggerFilter::OwnerAliveEventPlayerSameGroup,
        "owner_alive_event_player_same_group_hand_nonempty" => {
            TriggerFilter::OwnerAliveEventPlayerSameGroupHandNonempty
        }
        "owner_alive_other_play_phase_tied_max_hand_dcsbfuxi_available" => {
            TriggerFilter::OwnerAliveOtherPlayPhaseTiedMaxHandDcsbfuxiAvailable
        }
        "owner_phase_end_dcsbhaoyi_available" => TriggerFilter::OwnerPhaseEndDcsbhaoyiAvailable,
        "owner_alive_other_targeted_by_sha" => TriggerFilter::OwnerAliveOtherTargetedBySha,
        "owner_alive_event_player_lord" => TriggerFilter::OwnerAliveEventPlayerLord,
        "owner_alive_event_player_lord_no_damage" => {
            TriggerFilter::OwnerAliveEventPlayerLordNoDamage
        }
        "owner_alive_event_player_lord_damaged" => TriggerFilter::OwnerAliveEventPlayerLordDamaged,
        "owner_alive_targeted_by_other_sha_judge_xumou" => {
            TriggerFilter::OwnerAliveTargetedByOtherShaJudgeXumou
        }
        "owner_alive_other_source_damage" => TriggerFilter::OwnerAliveOtherSourceDamage,
        "owner_alive_event_player_play_phase_same_suit_at_least_two_once_per_turn" => {
            TriggerFilter::OwnerAliveEventPlayerPlayPhaseSameSuitAtLeastTwoOncePerTurn
        }
        "owner_is_event_player_discard_pile_four_suits_below_alive_cap" => {
            TriggerFilter::OwnerIsEventPlayerDiscardPileFourSuitsBelowAliveCap
        }
        "owner_alive_other_discarded_basic_outside_turn" => {
            TriggerFilter::OwnerAliveOtherDiscardedBasicOutsideTurn
        }
        "owner_alive_other_discarded_basic_outside_turn_once" => {
            TriggerFilter::OwnerAliveOtherDiscardedBasicOutsideTurnOncePerTurn
        }
        "owner_alive_other_discarded_black_to_discard" => {
            TriggerFilter::OwnerAliveOtherDiscardedBlackToDiscard
        }
        "owner_alive_other_responded_sha_gainable" => {
            TriggerFilter::OwnerAliveOtherRespondedShaGainable
        }
        "owner_alive_targeted_by_other_single_card_discardable" => {
            TriggerFilter::OwnerAliveTargetedByOtherSingleCardDiscardable
        }
        "owner_alive_targeted_by_trick_single_or_discardable" => {
            TriggerFilter::OwnerAliveTargetedByTrickSingleOrDiscardable
        }
        "owner_alive_targeted_by_other_trick" => TriggerFilter::OwnerAliveTargetedByOtherTrick,
        "owner_alive_targeted_by_other_matching_equip_suit" => {
            TriggerFilter::OwnerAliveTargetedByOtherMatchingEquipSuit
        }
        "owner_alive_targeted_by_other" => TriggerFilter::OwnerAliveTargetedByOther,
        "owner_alive_targeted_by_other_black_card" => {
            TriggerFilter::OwnerAliveTargetedByOtherBlackCard
        }
        "owner_alive_targeted_by_other_red_card" => TriggerFilter::OwnerAliveTargetedByOtherRedCard,
        "owner_alive_other_source_death" => TriggerFilter::OwnerAliveOtherSourceDeath,
        "owner_alive_other_phase_end_player_outside_owner_range" => {
            TriggerFilter::OwnerAliveOtherPhaseEndPlayerOutsideOwnerRange
        }
        "owner_alive_other_wei_recovery_own_phase_once_per_turn" => {
            TriggerFilter::OwnerAliveOtherWeiRecoveryOwnPhaseOncePerTurn
        }
        "owner_alive_other_global_juedou_or_red_sha_used" => {
            TriggerFilter::OwnerAliveOtherGlobalJuedouOrRedShaUsed
        }
        "owner_targeted_by_trick_equip_empty" => TriggerFilter::OwnerTargetedByTrickEquipEmpty,
        "owner_targeted_by_sha_hand_below_hp" => TriggerFilter::OwnerTargetedByShaHandBelowHp,
        "owner_alive_phase_end_caused_two_distinct_dying" => {
            TriggerFilter::OwnerAlivePhaseEndCausedTwoDistinctDying
        }
        "owner_alive_other_single_target_used_tracked_twyizhu" => {
            TriggerFilter::OwnerAliveOtherSingleTargetUsedTrackedTwyizhu
        }
        "owner_alive_other_twgonghuan_damage_before_hp_lte_owner_unused" => {
            TriggerFilter::OwnerAliveOtherTwgonghuanDamageBeforeHpLteOwnerUnused
        }
        "owner_phase_begin_playable_skill_pile_nonempty" => {
            TriggerFilter::OwnerPhaseBeginPlayableSkillPileNonempty
        }
        "owner_alive_other_phase_end_mobiledanshou" => {
            TriggerFilter::OwnerAliveOtherPhaseEndMobiledanshou
        }
        "owner_alive_other_phase_end_not_damaged_owner_this_turn" => {
            TriggerFilter::OwnerAliveOtherPhaseEndNotDamagedOwnerThisTurn
        }
        "owner_alive_other_phase_end_used_card_on_owner_undamaged_this_turn" => {
            TriggerFilter::OwnerAliveOtherPhaseEndUsedCardOnOwnerUndamagedThisTurn
        }
        "owner_alive_other_source_damage_card_resolved_targeted_damaged" => {
            TriggerFilter::OwnerAliveOtherSourceDamageCardResolvedTargetedDamaged
        }
        "owner_alive_other_source_damage_card_resolved_targeted_not_damaged" => {
            TriggerFilter::OwnerAliveOtherSourceDamageCardResolvedTargetedNotDamaged
        }
        "owner_alive_other_dying_mbchenshe_available" => {
            TriggerFilter::OwnerAliveOtherDyingMbchensheAvailable
        }
        "owner_alive_phase_end_used_card_no_damage" => {
            TriggerFilter::OwnerAlivePhaseEndUsedCardNoDamage
        }
        "owner_targeted_by_sha_or_juedou_damaged_hp_le_source_once" => {
            TriggerFilter::OwnerTargetedByShaOrJuedouDamagedHpLeSourceOnce
        }
        "owner_targeted_outside_turn_by_sha_or_trick_hand_empty" => {
            TriggerFilter::OwnerTargetedOutsideTurnByShaOrTrickHandEmpty
        }
        "owner_targeted_hand_below_max_hp" => TriggerFilter::OwnerTargetedHandBelowMaxHp,
        "owner_targeted_by_other_single_target_trick" => {
            TriggerFilter::OwnerTargetedByOtherSingleTargetTrick
        }
        "owner_alive_other_current_source_repeats_card_name" => {
            TriggerFilter::OwnerAliveOtherCurrentSourceRepeatsCardName
        }
        "owner_alive_other_dying_first_per_target" => {
            TriggerFilter::OwnerAliveOtherDyingFirstPerTarget
        }
        "owner_alive_other_single_target_juedou_or_red_sha" => {
            TriggerFilter::OwnerAliveOtherSingleTargetJuedouOrRedSha
        }
        "owner_phase_play_end_potguansha_available_once_per_game" => {
            TriggerFilter::OwnerPhasePlayEndPotguanshaAvailableOncePerGame
        }
        "owner_alive_other_turn_discarded_recorded_potfuan_suit" => {
            TriggerFilter::OwnerAliveOtherTurnDiscardedRecordedPotfuanSuit
        }
        "ddd_xuyu_pending" => TriggerFilter::DddXuyuPending,
        "owner_alive_other_source_second_play_card_resolved_hand_equip_nonempty" => {
            TriggerFilter::OwnerAliveOtherSourceSecondPlayCardResolvedHandEquipNonempty
        }
        "ddd_shijian_watched_source_card_used_targeted" => {
            TriggerFilter::DddShijianWatchedSourceCardUsedTargeted
        }
        "ddd_shijian_pending" => TriggerFilter::DddShijianPending,
        "owner_alive_other_phase_begin_both_hand_nonempty" => {
            TriggerFilter::OwnerAliveOtherPhaseBeginBothHandNonempty
        }
        "owner_alive_ddd_dongcha_pile_nonempty" => TriggerFilter::OwnerAliveDddDongchaPileNonempty,
        "owner_alive_other_phase_begin_unshown_hand_gte_hp" => {
            TriggerFilter::OwnerAliveOtherPhaseBeginUnshownHandGteHp
        }
        "owner_alive_other_phase_play_dddchashi_available" => {
            TriggerFilter::OwnerAliveOtherPhasePlayDddChashiAvailable
        }
        "owner_alive_ddd_fenji_available" => TriggerFilter::OwnerAliveDddFenjiAvailable,
        "owner_alive_ddd_fengzheng_round_pending" => {
            TriggerFilter::OwnerAliveDddFengzhengRoundPending
        }
        "owner_alive_ddd_yulv_discard_matches" => TriggerFilter::OwnerAliveDddYulvDiscardMatches,
        "owner_alive_ddd_youxue_round_pending" => TriggerFilter::OwnerAliveDddYouxueRoundPending,
        "owner_alive_ddd_chengjing_available" => TriggerFilter::OwnerAliveDddChengjingAvailable,
        "owner_alive_ddd_zhuanshe_gift" => TriggerFilter::OwnerAliveDddZhuansheGift,
        "owner_alive_ddd_zhuanshe_card_used" => TriggerFilter::OwnerAliveDddZhuansheCardUsed,
        "owner_alive_ddd_zhuanshe_phase_end" => TriggerFilter::OwnerAliveDddZhuanshePhaseEnd,
        "owner_alive_ddd_weiqiu" => TriggerFilter::OwnerAliveDddWeiqiu,
        "owner_alive_ddd_lianer_available" => TriggerFilter::OwnerAliveDddLianerAvailable,
        "owner_alive_ddd_lianer_ceiling" => TriggerFilter::OwnerAliveDddLianerCeiling,
        "owner_alive_ddd_anzhi_available" => TriggerFilter::OwnerAliveDddAnzhiAvailable,
        "owner_alive_ddd_anzhi_tracking" => TriggerFilter::OwnerAliveDddAnzhiTracking,
        "owner_alive_ddd_anzhi_target_black" => TriggerFilter::OwnerAliveDddAnzhiTargetBlack,
        "owner_alive_ddd_anzhi_turn_end" => TriggerFilter::OwnerAliveDddAnzhiTurnEnd,
        "owner_alive_ddd_shixing_turn_refresh" => TriggerFilter::OwnerAliveDddShixingTurnRefresh,
        "owner_alive_ddd_shixing_female_changed" => {
            TriggerFilter::OwnerAliveDddShixingFemaleChanged
        }
        "owner_alive_ddd_shixing_first_trick" => TriggerFilter::OwnerAliveDddShixingFirstTrick,
        "owner_alive_ddd_danggu_available" => TriggerFilter::OwnerAliveDddDangguAvailable,
        "owner_alive_ddd_fuzong_death" => TriggerFilter::OwnerAliveDddFuzongDeath,
        "owner_alive_ddd_xuanlun_pending" => TriggerFilter::OwnerAliveDddXuanlunPending,
        "owner_alive_ddd_jiexing_recover_once_per_game" => {
            TriggerFilter::OwnerAliveDddJiexingRecoverOncePerGame
        }
        "owner_alive_ddd_jiexing_equip_once_per_game" => {
            TriggerFilter::OwnerAliveDddJiexingEquipOncePerGame
        }
        "owner_alive_ddd_jiexing_recovery_reset" => {
            TriggerFilter::OwnerAliveDddJiexingRecoveryReset
        }
        "owner_alive_ddd_jiexing_equip_reset" => TriggerFilter::OwnerAliveDddJiexingEquipReset,
        "owner_alive_ddd_bailei_available" => TriggerFilter::OwnerAliveDddBaileiAvailable,
        "owner_alive_ddd_tongyu_remained" => TriggerFilter::OwnerAliveDddTongyuRemained,
        "owner_alive_other_judge_phase_in_range_has_judge_card" => {
            TriggerFilter::OwnerAliveOtherJudgePhaseInRangeHasJudgeCard
        }
        "owner_alive_other_phase_play_skill_pile_nonempty" => {
            TriggerFilter::OwnerAliveOtherPhasePlaySkillPileNonempty
        }
        "owner_alive_other_gained_single_equip_to_hand_outside_turn_usable" => {
            TriggerFilter::OwnerAliveOtherGainedSingleEquipToHandOutsideTurnUsable
        }
        "owner_alive_other_dying_hand_nonempty_once_per_game" => {
            TriggerFilter::OwnerAliveOtherDyingHandNonemptyOncePerGame
        }
        "owner_alive_other_dying_once_per_round" => TriggerFilter::OwnerAliveOtherDyingOncePerRound,
        "owner_targeted_by_other_sha_once" => TriggerFilter::OwnerTargetedByOtherShaOncePerTurn,
        "owner_targeted_by_other_card_once_per_turn" => {
            TriggerFilter::OwnerTargetedByOtherCardOncePerTurn
        }
        "owner_targeted_by_other_sha" => TriggerFilter::OwnerTargetedByOtherSha,
        "owner_alive_other_source_sha_twice" => TriggerFilter::OwnerAliveOtherSourceShaTwicePerTurn,
        "owner_is_event_player_no_enabled_equip_slot_once_per_game" => {
            TriggerFilter::OwnerIsEventPlayerNoEnabledEquipSlotOncePerGame
        }
        "owner_alive_other_play_phase_pothuilv_available" => {
            TriggerFilter::OwnerAliveOtherPlayPhasePothuilvAvailable
        }
        "owner_alive_other_responded_to_owner" => TriggerFilter::OwnerAliveOtherRespondedToOwner,
        _ => return None,
    })
}
