//! 触发过滤器名映射链：来源（source_*）与目标（target_*）类过滤器。

use super::*;

pub(super) fn map_filter(s: &str) -> Option<TriggerFilter> {
    Some(match s {
        "owner_is_target" | "OwnerIsEventPlayer" | "owner_is_event_player" => {
            TriggerFilter::OwnerIsEventPlayer
        }
        "owner_source" => TriggerFilter::OwnerIsEventSource,
        "owner_is_target_not_unique_max_current_hp" => {
            TriggerFilter::OwnerIsEventPlayerNotUniqueMaxCurrentHp
        }
        "owner_is_target_wounded" => TriggerFilter::OwnerIsEventPlayerWounded,
        "owner_is_target_linked" => TriggerFilter::OwnerIsEventPlayerLinked,
        "owner_is_target_chain_damage_linked" => TriggerFilter::OwnerIsTargetChainDamageLinked,
        "owner_is_target_wounded_sha" => TriggerFilter::OwnerIsTargetWoundedSha,
        "owner_is_target_healthy_draw_not_fixed" => TriggerFilter::OwnerIsTargetHealthyDrawNotFixed,
        "owner_is_target_alive_wounded_or_guiming_wu_exists" => {
            TriggerFilter::OwnerIsEventPlayerAliveWoundedOrGuimingWuExists
        }
        "owner_is_target_hand_nonempty" => TriggerFilter::OwnerIsEventPlayerHandNonempty,
        "owner_is_target_hand_below_max_hp" => TriggerFilter::OwnerIsEventPlayerHandBelowMaxHp,
        "owner_is_target_first_hp_change_turn_unique_trick_majority" => {
            TriggerFilter::OwnerIsEventPlayerFirstHpChangeTurnUniqueTrickMajority
        }
        "owner_is_target_hand_above_hp" => TriggerFilter::OwnerIsEventPlayerHandAboveHp,
        "owner_is_target_hand_at_least_hp_other_alive" => {
            TriggerFilter::OwnerIsEventPlayerHandAtLeastHpOtherAlive
        }
        "owner_is_target_lost_cards_at_most_hp_gainable" => {
            TriggerFilter::OwnerIsEventPlayerLostCardsAtMostHpGainable
        }
        "owner_is_target_source_movable_field_card" => {
            TriggerFilter::OwnerIsEventPlayerSourceMovableFieldCard
        }
        "owner_is_target_no_turn_damage" => TriggerFilter::OwnerIsEventPlayerNoTurnDamage,
        "owner_is_target_turn_damage" => TriggerFilter::OwnerIsEventPlayerTurnDamage,
        "owner_is_target_turn_damage_at_least_three" => {
            TriggerFilter::OwnerIsEventPlayerTurnDamageAtLeastThree
        }
        "owner_source_target_distance_gt_one" => TriggerFilter::OwnerSourceTargetDistanceGtOne,
        "owner_is_target_card_used_at_least_hp" => {
            TriggerFilter::OwnerIsEventPlayerCardUsedAtLeastHp
        }
        "owner_is_target_play_phase_card_category_count_positive" => {
            TriggerFilter::OwnerIsEventPlayerPlayPhaseCardCategoryCountPositive
        }
        "owner_is_target_other_hand_nonempty" => TriggerFilter::OwnerIsEventPlayerOtherHandNonempty,
        "owner_is_target_positive_draw_other_hand_nonempty" => {
            TriggerFilter::OwnerIsEventPlayerPositiveDrawOtherHandNonempty
        }
        "owner_source_first_target_black_card_any_target_other_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceFirstTargetBlackCardAnyTargetOtherDiscardableHandEquip
        }
        "owner_source_direct_sha_damage_target_armor_or_mount_card_gainable" => {
            TriggerFilter::OwnerSourceDirectShaDamageTargetArmorOrMountCardGainable
        }
        "owner_is_source_or_target_xinyaoming_available" => {
            TriggerFilter::OwnerIsEventSourceOrPlayerXinyaomingAvailable
        }
        "owner_source_play_card_matches_remembered_category" => {
            TriggerFilter::OwnerSourcePlayCardMatchesRememberedCategory
        }
        "owner_source_damage_card_once_per_turn" => TriggerFilter::OwnerSourceDamageCardOncePerTurn,
        "owner_source_sha_damage_before_potyinzhan_condition" => {
            TriggerFilter::OwnerSourceShaDamageBeforePotyinzhanCondition
        }
        "owner_source_damage_within_one" => TriggerFilter::OwnerSourceDamageWithinOne,
        "owner_source_first_target_has_hand_potyanhui" => {
            TriggerFilter::OwnerSourceFirstTargetHasHandPotyanhui
        }
        "owner_is_target_source_hand_nonempty_per_damage_point" => {
            TriggerFilter::OwnerIsTargetSourceHandNonemptyPerDamagePoint
        }
        "owner_source_qiaojian_extra_available" => TriggerFilter::OwnerSourceQiaojianExtraAvailable,
        "owner_source_wounded_exists_once_per_turn" => {
            TriggerFilter::OwnerSourceWoundedExistsOncePerTurn
        }
        "owner_is_target_discardable_hand_equip_gte_two_once_per_round" => {
            TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipGteTwoOncePerRound
        }
        "owner_is_target_other_source_alive_hand_equip_nonempty_oldhuisheng_unused" => {
            TriggerFilter::OwnerIsEventPlayerOtherSourceAliveHandEquipNonemptyOldHuishengUnused
        }
        "owner_source_oldjiefan_sha" => TriggerFilter::OwnerSourceOldJiefanSha,
        "owner_source_current_turn_first_target_non_equip" => {
            TriggerFilter::OwnerSourceCurrentTurnFirstTargetNonEquip
        }
        "owner_source_face_down_view_as_chenwei_available" => {
            TriggerFilter::OwnerSourceFaceDownViewAsChenweiAvailable
        }
        "owner_source_bound_category_first_three_play_phase" => {
            TriggerFilter::OwnerSourceBoundCategoryFirstThreePlayPhase
        }
        "owner_source_sha_resolved_has_gainable_cards_and_recipient" => {
            TriggerFilter::OwnerSourceShaResolvedHasGainableCardsAndRecipient
        }
        "owner_is_target_tied_max_hand" => TriggerFilter::OwnerIsEventPlayerTiedMaxHand,
        "owner_is_target_tied_max_hp" => TriggerFilter::OwnerIsEventPlayerTiedMaxHp,
        "owner_is_target_tied_max_equip_nonempty" => {
            TriggerFilter::OwnerIsEventPlayerTiedMaxEquipNonempty
        }
        "owner_is_target_skipped_phase_count_positive" => {
            TriggerFilter::OwnerIsEventPlayerSkippedPhaseCountPositive
        }
        "owner_source_first_sha_this_play_phase" => TriggerFilter::OwnerSourceFirstShaThisPlayPhase,
        "owner_source_damage_target_distance_at_most_one_empty_equip_slot_unused" => {
            TriggerFilter::OwnerSourceDamageTargetDistanceAtMostOneEmptyEquipSlotUnused
        }
        "owner_source_first_sha_target_this_turn" => {
            TriggerFilter::OwnerSourceFirstShaTargetThisTurn
        }
        "owner_source_sha_target_twzhenxi_available_once_per_turn" => {
            TriggerFilter::OwnerSourceShaTargetTwzhenxiAvailableOncePerTurn
        }
        "owner_source_first_target_damage_card_once_per_turn" => {
            TriggerFilter::OwnerSourceFirstTargetDamageCardOncePerTurn
        }
        "owner_source_first_target_hand_damage_card_discardable" => {
            TriggerFilter::OwnerSourceFirstTargetHandDamageCardDiscardable
        }
        "owner_source_damage_tracked_mbduanyang_target_recastable" => {
            TriggerFilter::OwnerSourceDamageTrackedMbduanyangTargetRecastable
        }
        "owner_source_damage_mbxiongtu_armed" => TriggerFilter::OwnerSourceDamageMbxiongtuArmed,
        "owner_is_target_can_move_field_card_mbyuanmo_twice_per_turn" => {
            TriggerFilter::OwnerIsTargetCanMoveFieldCardMbyuanmoTwicePerTurn
        }
        "owner_source_first_single_target_sha_mbfeijing_twice_per_turn" => {
            TriggerFilter::OwnerSourceFirstSingleTargetShaMbfeijingTwicePerTurn
        }
        "owner_source_damage_before_mbfeijing_extra" => {
            TriggerFilter::OwnerSourceDamageBeforeMbfeijingExtra
        }
        "owner_source_sha_resolved_original_single_mbxiaoge" => {
            TriggerFilter::OwnerSourceShaResolvedOriginalSingleMbxiaoge
        }
        "owner_is_target_discarded_at_least_two_in_discard_phase" => {
            TriggerFilter::OwnerIsEventPlayerDiscardedAtLeastTwoInDiscardPhase
        }
        "owner_is_target_missing_zhengsu" => TriggerFilter::OwnerIsTargetMissingZhengsu,
        "owner_source_club" => TriggerFilter::OwnerSourceClub,
        "owner_is_target_discarded_sha_in_discard_phase" => {
            TriggerFilter::OwnerIsTargetDiscardedShaInDiscardPhase
        }
        "owner_source_sha_damage_before_target_healthy" => {
            TriggerFilter::OwnerSourceShaDamageBeforeTargetHealthy
        }
        "owner_is_target_single_target_other_source_once_per_turn" => {
            TriggerFilter::OwnerIsTargetSingleTargetOtherSourceOncePerTurn
        }
        "owner_is_target_multi_target_trick_has_hand_equip" => {
            TriggerFilter::OwnerIsTargetMultiTargetTrickHasHandEquip
        }
        "owner_is_target_feijun_new_target" => TriggerFilter::OwnerIsTargetFeijunNewTarget,
        "owner_is_target_discard_phase_owe_limit_other_qun" => {
            TriggerFilter::OwnerIsTargetDiscardPhaseOweLimitOtherQun
        }
        "owner_source_sha_damage_target_alive_both_hand" => {
            TriggerFilter::OwnerSourceShaDamageTargetAliveBothHand
        }
        "owner_source_trick_resolved_own_play_phase_once_per_turn" => {
            TriggerFilter::OwnerSourceTrickResolvedOwnPlayPhaseOncePerTurn
        }
        "owner_source_sha_resolved_tracked_potceduan_dealt_damage" => {
            TriggerFilter::OwnerSourceShaResolvedTrackedPotceduanDealtDamage
        }
        "owner_source_sha_resolved_tracked_potzhanlie_draw" => {
            TriggerFilter::OwnerSourceShaResolvedTrackedPotzhanlieDraw
        }
        "owner_source_first_damage_card_target_hand_gte_owner" => {
            TriggerFilter::OwnerSourceFirstDamageCardTargetHandGteOwner
        }
        "owner_source_single_target_sha_or_black_ordinary_trick_current_play_phase_unused" => {
            TriggerFilter::OwnerSourceSingleTargetShaOrBlackOrdinaryTrickCurrentPlayPhaseUnused
        }
        "owner_is_target_male_source_has_discardable_equipment" => {
            TriggerFilter::OwnerIsEventPlayerMaleSourceHasDiscardableEquipment
        }
        "owner_source_sha_has_weapon" => TriggerFilter::OwnerSourceShaHasWeapon,
        "owner_source_sha_direct_damage_before" => TriggerFilter::OwnerSourceShaDirectDamageBefore,
        "owner_source_other_damage_before" => TriggerFilter::OwnerSourceOtherDamageBefore,
        "owner_source_other_damage_before_once_per_turn" => {
            TriggerFilter::OwnerSourceOtherDamageBeforeOncePerTurn
        }
        "owner_source_other_direct_damage_before" => {
            TriggerFilter::OwnerSourceOtherDirectDamageBefore
        }
        "owner_is_target_discardable_hand_equip" => {
            TriggerFilter::OwnerIsTargetDiscardableHandEquip
        }
        "owner_is_target_discardable_hand_equip_gte_two" => {
            TriggerFilter::OwnerIsTargetDiscardableHandEquipGteTwo
        }
        "owner_source_or_target_hand_count_differs_once_per_turn" => {
            TriggerFilter::OwnerSourceOrTargetHandCountDiffersOncePerTurn
        }
        "owner_is_target_any_alive_hand_nonempty" => {
            TriggerFilter::OwnerIsEventPlayerAnyAliveHandNonempty
        }
        "owner_is_target_any_alive_hand_empty" => {
            TriggerFilter::OwnerIsEventPlayerAnyAliveHandEmpty
        }
        "owner_is_target_other_hand_lte_owner" => {
            TriggerFilter::OwnerIsEventPlayerOtherHandLteOwner
        }
        "owner_is_target_draw_reshuffle_zero_wounded" => {
            TriggerFilter::OwnerIsEventPlayerDrawReshuffleZeroWounded
        }
        "owner_is_target_draw_reshuffle_positive" => {
            TriggerFilter::OwnerIsEventPlayerDrawReshufflePositive
        }
        "owner_source_basic" => TriggerFilter::OwnerSourceBasic,
        "owner_source_non_basic" => TriggerFilter::OwnerSourceNonBasic,
        "owner_source_basic_or_trick" => TriggerFilter::OwnerSourceBasicOrTrick,
        "owner_source_wounded_nanman_wanjian_or_normal_spade_sha" => {
            TriggerFilter::OwnerSourceWoundedNanmanWanjianOrNormalSpadeSha
        }
        "owner_is_target_sha_source_alive_hand_equip_nonempty_once_per_turn" => {
            TriggerFilter::OwnerIsTargetShaSourceAliveHandEquipNonemptyOncePerTurn
        }
        "owner_is_target_trick_targets_gt_source_hp_hand_equip_nonempty" => {
            TriggerFilter::OwnerIsTargetTrickTargetsGtSourceHpHandEquipNonempty
        }
        "owner_is_source_sha_used" => TriggerFilter::OwnerIsSourceShaUsed,
        "owner_is_source_sha_card_damage_before" => TriggerFilter::OwnerIsSourceShaCardDamageBefore,
        "owner_is_target_source_hp_gt_owner_per_role_turn" => {
            TriggerFilter::OwnerIsTargetSourceHpGtOwnerPerRoleTurn
        }
        "owner_source_in_turn_target_unharmed_card_actions_ge_target_hp" => {
            TriggerFilter::OwnerSourceInTurnTargetUnharmedCardActionsGeTargetHp
        }
        "owner_is_target_source_hp_gt_owner_or_source_damaged_owner" => {
            TriggerFilter::OwnerIsTargetSourceHpGtOwnerOrSourceDamagedOwner
        }
        "owner_is_target_sanshou_type_tracked" => TriggerFilter::OwnerIsTargetSanshouTypeTracked,
        "owner_is_target_owner_hp_gt_source_or_source_damaged_owner" => {
            TriggerFilter::OwnerIsTargetOwnerHpGtSourceOrSourceDamagedOwner
        }
        "owner_is_target_damage_amount_gt_one" => TriggerFilter::OwnerIsTargetDamageAmountGtOne,
        "owner_is_target_can_compare_other" => TriggerFilter::OwnerIsEventPlayerCanCompareOther,
        "owner_is_target_use_response_discarded_at_least_hp" => {
            TriggerFilter::OwnerIsEventPlayerUseResponseDiscardedAtLeastHp
        }
        "owner_is_target_damaged_other_qun" => TriggerFilter::OwnerIsEventPlayerDamagedOtherQun,
        "owner_is_target_source_alive" => TriggerFilter::OwnerIsEventPlayerSourceAlive,
        "owner_is_target_twxiaolian_redirected_damage" => {
            TriggerFilter::OwnerIsEventPlayerTwxiaolianRedirectedDamage
        }
        "owner_is_target_other_higher_hand_once_per_game" => {
            TriggerFilter::OwnerIsEventPlayerOtherHigherHandOncePerGame
        }
        "owner_is_target_source_alive_once" => {
            TriggerFilter::OwnerIsEventPlayerSourceAliveOncePerTurn
        }
        "owner_is_target_source_has_weapon" => TriggerFilter::OwnerIsEventPlayerSourceHasWeapon,
        "owner_is_target_source_alive_disabled_weapon" => {
            TriggerFilter::OwnerIsEventPlayerSourceAliveDisabledWeapon
        }
        "owner_is_target_other_source_alive" => TriggerFilter::OwnerIsEventPlayerOtherSourceAlive,
        "owner_is_target_other_source_alive_per_damage_point" => {
            TriggerFilter::OwnerIsEventPlayerOtherSourceAlivePerDamagePoint
        }
        "owner_is_target_per_damage_point" => TriggerFilter::OwnerIsEventPlayerPerDamagePoint,
        "owner_is_target_per_damage_point_other_hand_equip_judge_nonempty" => {
            TriggerFilter::OwnerIsEventPlayerPerDamagePointOtherHandEquipJudgeNonempty
        }
        "owner_is_target_other_alive" => TriggerFilter::OwnerIsEventPlayerOtherAlive,
        "owner_is_target_source_alive_gainable_card_per_damage_point" => {
            TriggerFilter::OwnerIsEventPlayerSourceAliveGainableCardPerDamagePoint
        }
        "owner_is_target_source_alive_gainable_card" => {
            TriggerFilter::OwnerIsEventPlayerSourceAliveGainableCard
        }
        "owner_is_target_card_in_processing" => TriggerFilter::OwnerIsEventPlayerCardInProcessing,
        "owner_is_target_red_sha_source_alive" => {
            TriggerFilter::OwnerIsEventPlayerRedShaSourceAlive
        }
        "owner_is_target_nonred_sha" => TriggerFilter::OwnerIsEventPlayerNonRedSha,
        "owner_is_target_red_or_jiu_sha" => TriggerFilter::OwnerIsEventPlayerRedOrJiuSha,
        "owner_is_target_sha_source_has_discardable_hand_equip" => {
            TriggerFilter::OwnerIsEventPlayerShaSourceHasDiscardableHandEquip
        }
        "owner_is_target_red_card_source_alive" => {
            TriggerFilter::OwnerIsEventPlayerRedCardSourceAlive
        }
        "owner_is_target_nonred_card" => TriggerFilter::OwnerIsEventPlayerNonRedCard,
        "owner_is_target_first_damage" => TriggerFilter::OwnerIsEventPlayerFirstDamagePerTurn,
        "owner_is_target_second_damage" => TriggerFilter::OwnerIsEventPlayerSecondDamagePerTurn,
        "owner_is_target_after_first_damage" => {
            TriggerFilter::OwnerIsEventPlayerAfterFirstDamagePerTurn
        }
        "owner_source_distance_le_one_per_damage_point" => {
            TriggerFilter::OwnerSourceDistanceLeOnePerDamagePoint
        }
        "owner_is_source" | "OwnerIsEventSource" | "owner_is_event_source" => {
            TriggerFilter::OwnerIsEventSource
        }
        "owner_alive_is_source" => TriggerFilter::OwnerAliveIsEventSource,
        "owner_source_target_alive_has_hand_or_equip" => {
            TriggerFilter::OwnerSourceTargetAliveHasHandOrEquip
        }
        "owner_source_other_target_alive_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceOtherTargetAliveHasDiscardableHandEquip
        }
        "owner_source_sha_target_alive_owner_outside_target_attack_range" => {
            TriggerFilter::OwnerSourceShaTargetAliveOwnerOutsideTargetAttackRange
        }
        "owner_is_source_or_target" => TriggerFilter::OwnerIsEventSourceOrPlayer,
        "owner_is_source_or_target_other_alive_per_damage_point" => {
            TriggerFilter::OwnerSourceOrTargetOtherAlivePerDamagePoint
        }
        "owner_source_fire" => TriggerFilter::OwnerSourceFire,
        "owner_target_fire" => TriggerFilter::OwnerTargetFire,
        "owner_target_thunder" => TriggerFilter::OwnerTargetThunder,
        "owner_source_thunder" => TriggerFilter::OwnerSourceThunder,
        "owner_source_wounded_distance_le_one" => TriggerFilter::OwnerSourceWoundedDistanceLeOne,
        "owner_source_sha_or_juedou_single_target_first_same_name_this_turn" => {
            TriggerFilter::OwnerSourceShaOrJuedouSingleTargetFirstSameNameThisTurn
        }
        "owner_source_sha_or_juedou_single_target_hand_nonempty" => {
            TriggerFilter::OwnerSourceShaOrJuedouSingleTargetHandNonempty
        }
        "owner_source_sha_damage_before_endpoint_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaDamageBeforeEndpointDiscardableHandEquip
        }
        "owner_is_source_hand_below_alive_count_capped_seven" => {
            TriggerFilter::OwnerSourceHandBelowAliveCountCappedSeven
        }
        "owner_alive_source_target_same_sex" => TriggerFilter::OwnerAliveSourceTargetSameSex,
        "owner_is_target_phase_end_hand_nonempty_other_hand_nonempty" => {
            TriggerFilter::OwnerIsTargetPhaseEndHandNonemptyOtherHandNonempty
        }
        "owner_is_target_ordinary_trick_cilv_available" => {
            TriggerFilter::OwnerIsTargetOrdinaryTrickCilvAvailable
        }
        "owner_source_fire_sha_or_huogong_dcsbronghuo" => {
            TriggerFilter::OwnerSourceFireShaOrHuogongDcsbronghuo
        }
        "owner_source_card_resolved_has_other_target_dcsbyingmou_once_per_turn" => {
            TriggerFilter::OwnerSourceCardResolvedHasOtherTargetDcsbyingmouOncePerTurn
        }
        "owner_is_target_card_damage_before" => TriggerFilter::OwnerIsTargetCardDamageBefore,
        "owner_source_single_target_sha_or_trick_all_materials_from_hand_can_sha" => {
            TriggerFilter::OwnerSourceSingleTargetShaOrTrickAllMaterialsFromHandCanSha
        }
        "owner_source_sha_or_ordinary_trick_target_wounded_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaOrOrdinaryTrickTargetWoundedHasDiscardableHandEquip
        }
        "owner_source_sha_or_ordinary_trick_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaOrOrdinaryTrickHasDiscardableHandEquip
        }
        "owner_source_sha_or_ordinary_trick" => TriggerFilter::OwnerSourceShaOrOrdinaryTrick,
        "owner_source_wuxie_used" => TriggerFilter::OwnerSourceWuxieUsed,
        "owner_source_sha_once_per_turn" => TriggerFilter::OwnerSourceShaOncePerTurn,
        "owner_is_target_hej_above_hp" => TriggerFilter::OwnerIsEventPlayerHandEquipJudgeAboveHp,
        "owner_source_play_card_repeats_previous_targets" => {
            TriggerFilter::OwnerSourcePlayCardRepeatsPreviousTargets
        }
        "owner_source_sha_repeats_previous_sha_targets" => {
            TriggerFilter::OwnerSourceShaRepeatsPreviousShaTargets
        }
        "owner_is_target_current_turn" => TriggerFilter::OwnerIsEventPlayerCurrentTurn,
        "owner_source_non_equip_no_basic_hand_has_targets" => {
            TriggerFilter::OwnerSourceNonEquipNoBasicHandHasTargets
        }
        "owner_alive_source_alive_once_per_round" => {
            TriggerFilter::OwnerAliveSourceAliveOncePerRound
        }
        "owner_alive_source_alive_event_player_dying_once_per_round" => {
            TriggerFilter::OwnerAliveSourceAliveEventPlayerDyingOncePerRound
        }
        "owner_source_sha_or_trick_current_play_phase" => {
            TriggerFilter::OwnerSourceShaOrTrickCurrentPlayPhase
        }
        "owner_source_sha_current_play_phase" => TriggerFilter::OwnerSourceShaCurrentPlayPhase,
        "owner_source_card_has_other_target_current_play_phase_first_two" => {
            TriggerFilter::OwnerSourceCardHasOtherTargetCurrentPlayPhaseFirstTwo
        }
        "owner_is_target_play_phase_hand_no_sha" => TriggerFilter::OwnerIsTargetPlayPhaseHandNoSha,
        "owner_source_sha_target_alive_has_discardable_equip" => {
            TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableEquip
        }
        "owner_source_card_used_targets_self" => TriggerFilter::OwnerSourceCardUsedTargetsSelf,
        "owner_source_current_phase_card_targeted_other_healthy" => {
            TriggerFilter::OwnerSourceCurrentPhaseCardTargetedOtherHealthy
        }
        "owner_is_target_not_min_hp" => TriggerFilter::OwnerIsEventPlayerNotMinHp,
        "owner_target_min_hand" => TriggerFilter::OwnerIsEventPlayerMinHand,
        "owner_target_not_min_hand" => TriggerFilter::OwnerIsEventPlayerNotMinHand,
        "owner_target_physical_sha_source_hp_gt_owner" => {
            TriggerFilter::OwnerTargetPhysicalShaSourceHpGtOwner
        }
        "owner_source_other_target_sha_or_ordinary_trick_twguoyi_condition" => {
            TriggerFilter::OwnerSourceOtherTargetShaOrOrdinaryTrickTwguoyiCondition
        }
        "owner_target_damage_before_playable_skill_piles_empty" => {
            TriggerFilter::OwnerTargetDamageBeforePlayableSkillPilesEmpty
        }
        "owner_is_target_discardable_hand_equip_once_per_round" => {
            TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipOncePerRound
        }
        "owner_source_active_dczhaowen_tracked_red" => {
            TriggerFilter::OwnerSourceActiveDczhaowenTrackedRed
        }
        "owner_is_target_event_card_jiu" => TriggerFilter::OwnerIsTargetEventCardJiu,
        "owner_is_target_other_source_black_dcjiudun_available" => {
            TriggerFilter::OwnerIsTargetOtherSourceBlackDcjiudunAvailable
        }
        "owner_source_or_targeted_by_juedou_or_red_sha" => {
            TriggerFilter::OwnerSourceOrTargetedByJuedouOrRedSha
        }
        "owner_source_or_targeted_by_juedou" => TriggerFilter::OwnerSourceOrTargetedByJuedou,
        "owner_source_or_targeted_by_sha_equal_hp" => {
            TriggerFilter::OwnerSourceOrTargetedByShaEqualHp
        }
        "owner_source_or_targeted_by_other_diamond" => {
            TriggerFilter::OwnerSourceOrTargetedByOtherDiamond
        }
        "owner_source_red" => TriggerFilter::OwnerSourceRed,
        "owner_source_alternates_color" => TriggerFilter::OwnerSourceAlternatesColor,
        "owner_source_repeats_color_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceRepeatsColorHasDiscardableHandEquip
        }
        "owner_is_target_source_group_qun_discardable_hand_equip_gte_two" => {
            TriggerFilter::OwnerIsEventPlayerSourceGroupQunDiscardableHandEquipGteTwo
        }
        "owner_source_matches_previous_suit_or_rank" => {
            TriggerFilter::OwnerSourceMatchesPreviousSuitOrRank
        }
        "owner_source_ordinary_trick" => TriggerFilter::OwnerSourceOrdinaryTrick,
        "owner_source_physical_ordinary_trick" => TriggerFilter::OwnerSourcePhysicalOrdinaryTrick,
        "owner_alive_source_first_card_ordinary_trick" => {
            TriggerFilter::OwnerAliveSourceFirstCardOrdinaryTrick
        }
        "owner_source_matches_last_used_suit_or_rank" => {
            TriggerFilter::OwnerSourceMatchesLastUsedSuitOrRank
        }
        "owner_source_hand_basic_equals_non_basic" => {
            TriggerFilter::OwnerSourceHandBasicEqualsNonBasic
        }
        "owner_is_target_event_card_sha" => TriggerFilter::OwnerIsEventPlayerEventCardSha,
        "owner_is_target_sha_or_trick" => TriggerFilter::OwnerIsEventPlayerEventCardShaOrTrick,
        "owner_source_or_within_distance_one_sha_or_juedou" => {
            TriggerFilter::OwnerSourceOrWithinDistanceOneShaOrJuedou
        }
        "owner_source_target_earlier_seat_hand_below_target_max_hp_once_per_turn" => {
            TriggerFilter::OwnerSourceTargetEarlierSeatHandBelowTargetMaxHpOncePerTurn
        }
        "owner_source_or_targeted_club_linked_exists_once_per_turn" => {
            TriggerFilter::OwnerSourceOrTargetedClubLinkedExistsOncePerTurn
        }
        "owner_source_sha_single_trick_source_card" => {
            TriggerFilter::OwnerSourceShaSingleTrickSourceCard
        }
        "owner_source_sha_single_equip_source_card" => {
            TriggerFilter::OwnerSourceShaSingleEquipSourceCard
        }
        "owner_is_target_hand_above_hp_no_liubei_lord" => {
            TriggerFilter::OwnerIsEventPlayerHandAboveHpNoLiubeiLord
        }
        "owner_is_target_trick_used_zero" => TriggerFilter::OwnerIsEventPlayerTrickUsedZero,
        "owner_is_target_hp_le_one" => TriggerFilter::OwnerIsEventPlayerHpLeOne,
        "owner_is_target_hp_le_two" => TriggerFilter::OwnerIsEventPlayerHpLeTwo,
        "owner_source_or_target_second_damage_this_turn" => {
            TriggerFilter::OwnerSourceOrTargetEventPlayerSecondDamageThisTurn
        }
        "owner_source_or_target_first_damage_this_turn" => {
            TriggerFilter::OwnerSourceOrTargetEventPlayerFirstDamageThisTurn
        }
        "owner_source_target_distance_le_one_first_damage_to_target_this_turn" => {
            TriggerFilter::OwnerSourceTargetDistanceLeOneFirstDamageToTargetThisTurn
        }
        "owner_target_hand_empty_elemental_or_trick_damage_before" => {
            TriggerFilter::OwnerTargetHandEmptyElementalOrTrickDamageBefore
        }
        "owner_source_sha" => TriggerFilter::OwnerSourceSha,
        "owner_target_sha" => TriggerFilter::OwnerTargetSha,
        "owner_source_heart_sha" => TriggerFilter::OwnerSourceHeartSha,
        "owner_source_sha_first_target" => TriggerFilter::OwnerSourceShaFirstTarget,
        "owner_source_sha_or_juedou_single_target" => {
            TriggerFilter::OwnerSourceShaOrJuedouSingleTarget
        }
        "owner_source_sha_damage_recorded_by_skill" => {
            TriggerFilter::OwnerSourceShaDamageRecordedBySkill
        }
        "owner_source_normal_sha" => TriggerFilter::OwnerSourceNormalSha,
        "owner_source_fire_sha" => TriggerFilter::OwnerSourceFireSha,
        "owner_is_target_hand_sha_skill_pile_empty" => {
            TriggerFilter::OwnerIsEventPlayerHandShaSkillPileEmpty
        }
        "owner_source_single_target_sha_or_ordinary_trick_current_turn" => {
            TriggerFilter::OwnerSourceSingleTargetShaOrOrdinaryTrickCurrentTurn
        }
        "owner_source_event_card_policy_draw" => TriggerFilter::OwnerSourceEventCardPolicyDraw,
        "owner_source_card_resolved_phase_policy" => {
            TriggerFilter::OwnerSourceCardResolvedPhasePolicy
        }
        "owner_source_sha_resolved_potfuan_dealt_damage" => {
            TriggerFilter::OwnerSourceShaResolvedPotfuanDealtDamage
        }
        "owner_source_card_resolved_ddd_xuyu_pending" => {
            TriggerFilter::OwnerSourceCardResolvedDddXuyuPending
        }
        "owner_is_target_face_down" => TriggerFilter::OwnerIsTargetFaceDown,
        "owner_source_own_view_as_ddd_tongyu_used" => {
            TriggerFilter::OwnerSourceOwnViewAsDddTongyuUsed
        }
        "owner_source_own_view_as_ddd_tongyu_targeted" => {
            TriggerFilter::OwnerSourceOwnViewAsDddTongyuTargeted
        }
        "owner_alive_source_basic_dddfushi_available" => {
            TriggerFilter::OwnerAliveSourceBasicDddFushiAvailable
        }
        "owner_source_play_sha_other_alive_once_per_play_phase" => {
            TriggerFilter::OwnerSourcePlayShaOtherAliveOncePerPlayPhase
        }
        "owner_source_jiu" => TriggerFilter::OwnerSourceJiu,
        "owner_is_target_hp_odd" => TriggerFilter::OwnerIsEventPlayerHpOdd,
        "owner_is_target_hp_even_wounded" => TriggerFilter::OwnerIsEventPlayerHpEvenWounded,
        "owner_source_trick" | "card_is_trick" => TriggerFilter::OwnerSourceTrick,
        "owner_source_once" => TriggerFilter::OwnerSourceOncePerTurn,
        "owner_source_once_per_turn" => TriggerFilter::OwnerSourceOncePerTurn,
        "owner_source_target_alive_has_equip" => TriggerFilter::OwnerSourceTargetAliveHasEquip,
        "owner_source_black_sha_target_alive_has_discardable_equip" => {
            TriggerFilter::OwnerSourceBlackShaTargetAliveHasDiscardableEquip
        }
        "owner_source_sha_damage_target_alive_has_discardable_equip" => {
            TriggerFilter::OwnerSourceShaDamageTargetAliveHasDiscardableEquip
        }
        "owner_source_equip_used_skill_mark_below_seven" => {
            TriggerFilter::OwnerSourceEquipUsedSkillMarkBelowSeven
        }
        "owner_source_outside_turn_card_used_or_responded" => {
            TriggerFilter::OwnerSourceOutsideTurnCardUsedOrResponded
        }
        "owner_source_sha_target_alive_has_discardable_region_card" => {
            TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableRegionCard
        }
        "owner_source_sha_target_alive_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableHandEquip
        }
        "owner_source_tiesuo_target_alive_unlinked_hand_equip_nonempty" => {
            TriggerFilter::OwnerSourceTiesuoTargetAliveUnlinkedHandEquipNonempty
        }
        "owner_alive_source_target_was_linked" => TriggerFilter::OwnerAliveSourceTargetWasLinked,
        "owner_source_sha_target_alive_hand_nonempty" => {
            TriggerFilter::OwnerSourceShaTargetAliveHandNonempty
        }
        "owner_source_sha_target_hand_gte_hp_or_lte_attack_range" => {
            TriggerFilter::OwnerSourceShaTargetHandGteHpOrLteAttackRange
        }
        "owner_source_sha_target_hand_lte_owner_or_hp_gte_owner" => {
            TriggerFilter::OwnerSourceShaTargetHandLteOwnerOrHpGteOwner
        }
        "owner_is_target_judge_nonempty" => TriggerFilter::OwnerIsEventPlayerJudgeNonempty,
        "owner_is_target_draw_not_replaced" => TriggerFilter::OwnerIsEventPlayerDrawNotReplaced,
        "owner_is_target_draw_not_fixed_hp_one_or_hand_empty" => {
            TriggerFilter::OwnerIsEventPlayerDrawNotFixedHpOneOrHandEmpty
        }
        "owner_source_sha_dodged_target_alive_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaDodgedTargetAliveHasDiscardableHandEquip
        }
        "owner_source_sha_dodged_target_recorded_by_skill_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaDodgedTargetRecordedBySkillHasDiscardableHandEquip
        }
        "owner_source_sha_dodged" => TriggerFilter::OwnerSourceShaDodged,
        "owner_source_sha_target_alive_distance_le_one" => {
            TriggerFilter::OwnerSourceShaTargetAliveDistanceLeOne
        }
        "owner_is_target_sha_source_alive_hand_equip_nonempty" => {
            TriggerFilter::OwnerIsEventPlayerShaSourceAliveHandEquipNonempty
        }
        "owner_is_target_non_multitarget_trick_multiple_targets" => {
            TriggerFilter::OwnerIsTargetNonMultitargetTrickMultipleTargets
        }
        "owner_source_play_phase_sha_has_additional_target_ignoring_distance" => {
            TriggerFilter::OwnerSourcePlayPhaseShaHasAdditionalTargetIgnoringDistance
        }
        "owner_source_card_resolved_tracked_dddxianxi" => {
            TriggerFilter::OwnerSourceCardResolvedTrackedDddXianxi
        }
        "owner_source_card_damage_target_can_receive_owner_bingliang" => {
            TriggerFilter::OwnerSourceCardDamageTargetCanReceiveOwnerBingliang
        }
        "owner_alive_source_ally_sha_damage_before" => {
            TriggerFilter::OwnerAliveSourceAllyShaDamageBefore
        }
        "owner_is_target_wounded_dead_ally" => TriggerFilter::OwnerIsEventPlayerWoundedDeadAlly,
        "owner_is_target_any_alive_self_discardable_hand_equip" => {
            TriggerFilter::OwnerIsEventPlayerAnyAliveSelfDiscardableHandEquip
        }
        "owner_is_target_any_alive_discardable_by_owner_hand_equip" => {
            TriggerFilter::OwnerIsEventPlayerAnyAliveDiscardableByOwnerHandEquip
        }
        "owner_is_target_any_alive_self_discardable_hand" => {
            TriggerFilter::OwnerIsEventPlayerAnyAliveSelfDiscardableHand
        }
        "owner_is_target_any_alive_wounded" => TriggerFilter::OwnerIsEventPlayerAnyAliveWounded,
        "owner_is_target_hand_equip_nonempty_other_alive" => {
            TriggerFilter::OwnerIsEventPlayerHandEquipNonemptyOtherAlive
        }
        "owner_is_target_phase_end_hand_equip_nonempty_other_alive" => {
            TriggerFilter::OwnerIsEventPlayerPhaseEndHandEquipNonemptyOtherAlive
        }
        "owner_is_target_once" => TriggerFilter::OwnerIsEventPlayerOncePerTurn,
        "owner_source_healthy" => TriggerFilter::OwnerSourceHealthy,
        "owner_source_amount_gt_one" => TriggerFilter::OwnerSourceAmountGtOne,
        "owner_source_other_twice" => TriggerFilter::OwnerSourceOtherTwicePerTurn,
        "owner_is_target_has_movable_equipment" => {
            TriggerFilter::OwnerIsEventPlayerHasMovableEquipment
        }
        "owner_alive_is_target_current_play_phase" => {
            TriggerFilter::OwnerAliveIsEventPlayerCurrentPlayPhase
        }
        "owner_is_target_can_use_wuzhong" => TriggerFilter::OwnerIsTargetCanUseWuzhong,
        "owner_source_damage_potjianzhuan_available" => {
            TriggerFilter::OwnerSourceDamagePotjianzhuanAvailable
        }
        "owner_target_damage_potjianzhuan_available" => {
            TriggerFilter::OwnerTargetDamagePotjianzhuanAvailable
        }
        "owner_source_damage_potcaoshuang_assigned_untriggered" => {
            TriggerFilter::OwnerSourceDamagePotcaoshuangAssignedUntriggered
        }
        "owner_target_damage_potcaoshuang_assigned_untriggered" => {
            TriggerFilter::OwnerTargetDamagePotcaoshuangAssignedUntriggered
        }
        "owner_target_damage_before_potnizun_used" => {
            TriggerFilter::OwnerTargetDamageBeforePotnizunUsed
        }
        _ => return super::map_filter_other::map_filter(s),
    })
}
