//! Name resolution, type check, capability / termination checks → Rule IR.

use crate::ast::*;
use crate::CompileError;
use content_schema::{
    ActiveIr, ActiveSelectionIr, ActiveTargetsIr, DrawTargetScopeIr, ExprIr, ModifierIr,
    RuleModuleIr, SkillIr, Span, StmtIr, TargetCardScopeIr, TriggerIr, ViewAsIr,
};

/// Allowed events.
const EVENTS: &[&str] = &[
    "game_started",
    "damage_before",
    "recover_before",
    "damage_prevented",
    "damage",
    "hp_lost",
    "card_used",
    "card_resolved",
    "card_targeted",
    "card_responded",
    "dying",
    "dying_after",
    "max_hp_changed",
    "death",
    "phase_prepare",
    "phase_begin",
    "phase_judge",
    "phase_draw",
    "phase_play",
    "phase_discard",
    "phase_end",
    "phase_draw_end",
    "debate_resolved",
    "debate_started",
    "phase_play_end",
    "phase_discard_end",
    "phase_any_end",
    "phase_draw_skipped",
    "phase_any_skipped",
    "cards_drawn",
    "cards_moved",
    "cards_given",
    "skill_added",
    "turn_ended",
    "round_started",
    "round_ended",
    "hp_recovered",
    "duel_resolved",
    "equip_slot_disabled",
    "equip_slot_enabled",
    "character_state_changed",
    "sha_dodged",
    "sha_resolved",
    "hand_compare_started",
    "hand_compare_finished",
    "judgment_resolved",
];
/// Allowed filters.
const FILTERS: &[&str] = &[
    "owner_is_target",
    "owner_source",
    "owner_is_target_not_unique_max_current_hp",
    "owner_is_target_once",
    "owner_targeted_by_other_sha_once",
    "owner_targeted_by_other_card_once_per_turn",
    "owner_twfeifu_single_sha_party_target_has_hand_equip",
    "owner_targeted_by_other_sha",
    "owner_is_target_wounded",
    "owner_is_target_linked",
    "owner_is_target_chain_damage_linked",
    "owner_is_target_wounded_sha",
    "owner_is_target_healthy_draw_not_fixed",
    "owner_is_target_alive_wounded_or_guiming_wu_exists",
    "owner_is_target_hand_nonempty",
    "owner_is_target_hand_below_max_hp",
    "owner_is_target_first_hp_change_turn_unique_trick_majority",
    "owner_is_target_hand_above_hp",
    "owner_is_target_hand_above_hp_no_liubei_lord",
    "owner_first_transformed_sha_used_this_turn",
    "owner_target_hand_empty_elemental_or_trick_damage_before",
    "owner_is_target_hand_at_least_hp_other_alive",
    "owner_is_target_lost_cards_at_most_hp_gainable",
    "owner_is_target_source_movable_field_card",
    "owner_from_hand_empty_other_discardable_hand_equip",
    "owner_is_target_no_turn_damage",
    "owner_is_target_turn_damage",
    "owner_is_target_turn_damage_at_least_three",
    "owner_is_target_card_used_at_least_hp",
    "owner_is_target_play_phase_card_category_count_positive",
    "owner_alive_play_phase_card_category_count_positive",
    "owner_is_target_other_hand_nonempty",
    "owner_is_target_positive_draw_other_hand_nonempty",
    "owner_source_first_target_black_card_any_target_other_discardable_hand_equip",
    "owner_source_play_card_matches_remembered_category",
    "owner_source_damage_card_once_per_turn",
    "owner_alive_event_player_hand_nonempty_once_per_round",
    "owner_alive_play_phase_actor_hand_nonempty_potjiejie_unused",
    "owner_source_sha_damage_before_potyinzhan_condition",
    "owner_source_damage_within_one",
    "owner_gained_cards_potzhouyu",
    "owner_source_first_target_has_hand_potyanhui",
    "owner_alive_fire_damage_potfentao",
    "owner_alive_potfentao_pending_damage",
    "owner_alive_other_play_phase_pothuilv_available",
    "owner_source_damage_potjianzhuan_available",
    "owner_target_damage_potjianzhuan_available",
    "owner_alive_dying_potjianzhuan_available",
    "owner_source_damage_potcaoshuang_assigned_untriggered",
    "owner_target_damage_potcaoshuang_assigned_untriggered",
    "owner_alive_dying_potcaoshuang_assigned_untriggered",
    "owner_used_potnizun_pile_card",
    "owner_target_damage_before_potnizun_used",
    "owner_alive_turn_ended_used_card_tagged_discard",
    "owner_is_target_can_use_wuzhong",
    "owner_alive_discard_batch",
    "owner_alive_other_responded_to_owner",
    "owner_is_target_discardable_hand_equip_gte_two_once_per_round",
    "owner_is_target_other_source_alive_hand_equip_nonempty_oldhuisheng_unused",
    "owner_alive_other_dying_outside_turn_can_sha_current",
    "owner_source_oldjiefan_sha",
    "owner_source_current_turn_first_target_non_equip",
    "owner_source_face_down_view_as_chenwei_available",
    "owner_source_bound_category_first_three_play_phase",
    "owner_phase_gift_target_is_source",
    "owner_phase_gift_target_is_target",
    "owner_source_sha_resolved_has_gainable_cards_and_recipient",
    "owner_is_target_tied_max_hand",
    "owner_is_target_tied_max_hp",
    "owner_is_target_tied_max_equip_nonempty",
    "owner_is_target_skipped_phase_count_positive",
    "owner_discarded_has_hand_once_per_phase",
    "owner_alive_lost_hand_this_turn",
    "owner_alive_hand_same_kind_event_card",
    "owner_alive_has_skill_tagged_cards",
    "owner_alive_tagged_card_lost_this_turn",
    "owner_alive_phase_end_lost_cards_at_least_two",
    "owner_source_first_sha_this_play_phase",
    "owner_source_damage_target_distance_at_most_one_empty_equip_slot_unused",
    "owner_source_first_sha_target_this_turn",
    "owner_alive_other_phase_play_end_twzhengjian_condition",
    "owner_source_sha_target_twzhenxi_available_once_per_turn",
    "owner_source_first_target_damage_card_once_per_turn",
    "owner_source_first_target_hand_damage_card_discardable",
    "owner_lost_hand_ownerless_sha_not_use_once_per_turn",
    "owner_alive_phase_any_end_originating_skill_pile_nonempty",
    "owner_phase_end_originating_skill_pile_empty",
    "owner_prepare_group_outside_attack_range_exists",
    "owner_source_damage_tracked_mbduanyang_target_recastable",
    "owner_mbrunwei_loss_threshold",
    "owner_phase_play_end_mbrunwei_tracked_hand",
    "owner_source_damage_mbxiongtu_armed",
    "owner_phase_play_end_mbxiongtu_state",
    "owner_phase_end_mbganggeng_target_alive",
    "owner_from_hand_empty_or_dying_mbsijian_twice_per_turn",
    "owner_is_target_can_move_field_card_mbyuanmo_twice_per_turn",
    "owner_source_first_single_target_sha_mbfeijing_twice_per_turn",
    "owner_source_damage_before_mbfeijing_extra",
    "owner_source_sha_resolved_original_single_mbxiaoge",
    "owner_alive_damage_target_min_hand_mbxuye_once_per_turn",
    "owner_mbkuangxiang_received_cards_exhausted",
    "owner_phase_play_mbkuangxiang_active",
    "owner_gained_cards_potduzuo",
    "owner_gained_cards_potduzuo_once_per_turn",
    "owner_sha_damage_before_potbihan_available",
    "owner_alive_card_targeted_healthy",
    "owner_alive_card_resolved_source_current_potsifeng_pile",
    "owner_alive_phase_end_player_potsifeng_pile",
    "owner_alive_damage_before_target_distance_le_one_once_per_turn",
    "owner_outside_turn_discarded_red_cards_in_discard",
    "owner_alive_other_twkujian_used_or_responded",
    "owner_alive_other_twkujian_lost",
    "owner_alive_twruilian_round_unprompted",
    "owner_alive_twruilian_selected_target_discarded",
    "owner_alive_twruilian_selected_target_turn_ended",
    "owner_alive_lost_weapon_outside_turn",
    "owner_alive_other_phase_prepare_can_compare_owner",
    "owner_target_physical_sha_source_hp_gt_owner",
    "owner_source_other_target_sha_or_ordinary_trick_twguoyi_condition",
    "owner_alive_phase_end_caused_two_distinct_dying",
    "owner_gained_event_cards_from_other_personal_zone",
    "owner_alive_other_single_target_used_tracked_twyizhu",
    "owner_tracked_twyizhu_entered_discard",
    "owner_alive_other_twgonghuan_damage_before_hp_lte_owner_unused",
    "owner_phase_begin_playable_skill_pile_nonempty",
    "owner_used_playable_skill_pile_card_unused",
    "owner_target_damage_before_playable_skill_piles_empty",
    "owner_alive_damage_before_target_min_hp_unused",
    "owner_alive_global_phase_end",
    "owner_alive_other_phase_end_mobiledanshou",
    "owner_alive_other_source_damage_card_resolved_targeted_damaged",
    "owner_alive_other_source_damage_card_resolved_targeted_not_damaged",
    "owner_alive_other_dying_mbchenshe_available",
    "owner_is_target_discardable_hand_equip_once_per_round",
    "owner_alive_potdingpin_available_once_per_round",
    "owner_alive_first_non_discard_phase_discard_mbrenxing_available",
    "owner_alive_global_phase_end_lost_cards_target_other_alive",
    "owner_alive_global_phase_end_discarded_cards_available",
    "owner_alive_phase_end_used_card_no_damage",
    "owner_cards_entered_discard_not_use_unused",
    "owner_is_target_discarded_at_least_two_in_discard_phase",
    "owner_alive_other_discarded_in_discard_phase",
    "owner_alive_other_wu_play_phase_can_compare_owner",
    "owner_is_target_multi_target_trick_has_hand_equip",
    "owner_is_target_feijun_new_target",
    "owner_is_target_discard_phase_owe_limit_other_qun",
    "owner_source_sha_damage_target_alive_both_hand",
    "owner_alive_other_nanman_cards_discard",
    "owner_alive_skill_bound_target_is_event_source",
    "owner_lost_cards_outside_turn",
    "owner_gained_cards_during_play_phase_wounded",
    "owner_discarded_outside_play_phase",
    "owner_in_multitarget_sha_or_trick_targets",
    "owner_targeted_by_nanman",
    "owner_targeted_source_hp_hand_differ_once_per_turn",
    "owner_source_trick_resolved_own_play_phase_once_per_turn",
    "owner_alive_other_dying_has_equip_card",
    "owner_alive_damage_card_nanman",
    "owner_black_card_moved_equip_judge",
    "owner_discard_suit_no_own_mark_spade_falu",
    "owner_discard_suit_no_own_mark_heart_falu",
    "owner_discard_suit_no_own_mark_club_falu",
    "owner_discard_suit_no_own_mark_diamond_falu",
    "owner_lost_non_damage_card_pottuntian_charge_not_full",
    "owner_source_sha_resolved_tracked_potceduan_dealt_damage",
    "owner_alive_sha_entered_discard_potzhanlie_available",
    "owner_source_sha_resolved_tracked_potzhanlie_draw",
    "owner_phase_begin_hand_empty",
    "owner_alive_nanman_resolved_target_dead_or_damaged_gt_three",
    "owner_source_first_damage_card_target_hand_gte_owner",
    "owner_responded_shan",
    "owner_alive_other_qun_play_phase_can_give_owner",
    "owner_is_target_trick_used_zero",
    "owner_is_target_hp_le_one",
    "owner_is_target_hp_le_two",
    "owner_source_or_target_second_damage_this_turn",
    "owner_source_or_target_first_damage_this_turn",
    "owner_source_target_distance_le_one_first_damage_to_target_this_turn",
    "owner_is_target_current_turn",
    "owner_source_single_target_sha_or_black_ordinary_trick_current_play_phase_unused",
    "owner_is_target_male_source_has_discardable_equipment",
    "owner_alive_other_turn_ended_no_card_targeted_other",
    "owner_alive_other_turn_ended_no_damage_other",
    "owner_alive_gained_two_this_turn_can_compare_other",
    "owner_alive_other_turn_ended_card_targeted_other_no_damage_other",
    "owner_alive_phase_targeted_hp_changed_can_use_wanjian",
    "owner_alive_phase_targeted_hp_unchanged_can_use_juedou",
    "owner_source_sha_has_weapon",
    "owner_source_sha_single_trick_source_card",
    "owner_source_sha_single_equip_source_card",
    "owner_alive_other_dying_first_per_target",
    "owner_source_sha_direct_damage_before",
    "owner_source_other_damage_before",
    "owner_source_other_damage_before_once_per_turn",
    "owner_source_other_direct_damage_before",
    "owner_is_target_discardable_hand_equip",
    "owner_is_target_discardable_hand_equip_gte_two",
    "owner_alive_other_dying_once_per_round",
    "owner_source_or_target_hand_count_differs_once_per_turn",
    "owner_is_target_any_alive_hand_nonempty",
    "owner_is_target_any_alive_hand_empty",
    "owner_is_target_other_hand_lte_owner",
    "owner_is_target_draw_reshuffle_zero_wounded",
    "owner_is_target_draw_reshuffle_positive",
    "owner_hand_compare_participant",
    "owner_hand_compare_finished_participant",
    "owner_hand_compare_participant_has_sha",
    "owner_hand_compare_participant_result_card_in_discard",
    "owner_is_target_use_response_discarded_at_least_hp",
    "owner_is_target_damaged_other_qun",
    "owner_is_target_source_alive",
    "owner_is_target_source_alive_once",
    "owner_is_target_source_has_weapon",
    "owner_is_target_source_alive_disabled_weapon",
    "owner_is_target_other_source_alive",
    "owner_is_target_other_source_alive_per_damage_point",
    "owner_is_target_per_damage_point",
    "owner_alive_turn_ended_rehengjiang_pending",
    "owner_alive_phase_play_end_zhengbi_pending",
    "owner_is_target_source_hand_nonempty_per_damage_point",
    "owner_source_qiaojian_extra_available",
    "owner_xicha_watched_source_card_used",
    "owner_qiaojian_tracked_card_left_hand",
    "owner_alive_damage_target_self_or_in_range_other_source_numeric_card",
    "owner_source_wounded_exists_once_per_turn",
    "owner_is_target_per_damage_point_other_hand_equip_judge_nonempty",
    "owner_is_target_judge_nonempty",
    "owner_is_target_draw_not_replaced",
    "owner_is_target_draw_not_fixed_hp_one_or_hand_empty",
    "owner_is_target_other_alive",
    "owner_is_target_hand_equip_nonempty_other_alive",
    "owner_is_target_phase_end_hand_equip_nonempty_other_alive",
    "owner_alive_gained_three_this_turn_other_alive",
    "owner_is_target_source_alive_gainable_card_per_damage_point",
    "owner_is_target_source_alive_gainable_card",
    "owner_is_target_card_in_processing",
    "owner_is_target_red_sha_source_alive",
    "owner_is_target_nonred_sha",
    "owner_is_target_red_or_jiu_sha",
    "owner_is_target_sha_source_has_discardable_hand_equip",
    "owner_is_target_red_card_source_alive",
    "owner_is_target_nonred_card",
    "owner_is_target_first_damage",
    "owner_is_target_second_damage",
    "owner_is_target_after_first_damage",
    "owner_source_distance_le_one_per_damage_point",
    "owner_source_target_distance_gt_one",
    "owner_is_source",
    "debate_owner_is_initiator",
    "debate_owner_participated",
    "owner_debate_participant",
    "owner_cards_moved_contain_ying",
    "owner_is_lord",
"debate_owner_opinion_differs_initiator",
"debate_owner_is_initiator_result_red",
    "owner_alive_other_source_damage",
    "debate_owner_is_initiator_result_black",
    "owner_is_respond_to",
    "owner_alive_source_target_was_linked",
    "owner_alive_is_source",
    "owner_source_target_alive_has_equip",
    "owner_source_black_sha_target_alive_has_discardable_equip",
    "owner_source_sha_damage_target_alive_has_discardable_equip",
    "owner_source_card_damage_target_can_receive_owner_bingliang",
    "owner_source_equip_used_skill_mark_below_seven",
    "owner_source_outside_turn_card_used_or_responded",
    "owner_source_sha_target_alive_has_discardable_region_card",
    "owner_source_sha_target_alive_has_discardable_hand_equip",
    "owner_source_tiesuo_target_alive_unlinked_hand_equip_nonempty",
    "owner_source_sha_target_alive_hand_nonempty",
    "owner_source_sha_target_hand_gte_hp_or_lte_attack_range",
    "owner_source_sha_target_hand_lte_owner_or_hp_gte_owner",
    "owner_source_direct_sha_damage_target_armor_or_mount_card_gainable",
    "owner_is_source_or_target_xinyaoming_available",
    "owner_source_sha_target_alive_owner_outside_target_attack_range",
    "owner_source_sha_dodged_target_alive_has_discardable_hand_equip",
    "owner_source_sha_dodged_target_recorded_by_skill_has_discardable_hand_equip",
    "owner_source_sha_dodged",
    "cards_moved_into_expansion",
    "cards_moved_out_of_expansion",
    "owner_source_sha_damage_recorded_by_skill",
    "owner_source_sha_target_alive_distance_le_one",
    "owner_alive_sha_target_distance_le_one",
    "owner_alive_sha_target_distance_le_one_has_equip_card",
    "owner_is_target_sha_source_alive_hand_equip_nonempty",
    "owner_alive_wdb_fuchou_pending_living",
    "owner_source_target_alive_has_hand_or_equip",
    "owner_source_other_target_alive_has_discardable_hand_equip",
    "owner_is_source_or_target",
    "owner_is_source_or_target_other_alive_per_damage_point",
    "owner_source_fire",
    "owner_target_fire",
    "owner_target_thunder",
    "owner_source_thunder",
    "owner_source_wounded_distance_le_one",
    "owner_alive",
    "owner_alive_event_player_lost_armor_or_mount_has_originating_skill_pile",
    "owner_alive_hand_below_alive_count_capped_seven",
    "owner_alive_hand_below_alive_count_capped_eight",
    "owner_alive_delayed_trick_target_without_pending_judge_skip",
    "owner_source_sha_or_juedou_single_target_first_same_name_this_turn",
    "owner_source_sha_or_juedou_single_target_hand_nonempty",
    "owner_source_sha_damage_before_endpoint_discardable_hand_equip",
    "owner_longdan_view_as_first_counterpart_hand_nonempty",
    "owner_is_source_hand_below_alive_count_capped_seven",
    "owner_alive_event_player_alive",
    "owner_alive_event_player_equip_empty",
    "owner_alive_event_player_from_equip_empty",
    "owner_alive_sha_target_no_other_red_card_this_turn",
    "owner_alive_event_player_hand_empty",
    "owner_alive_event_cards_nonempty",
    "owner_alive_neighbor_gained_cards_outside_own_draw_once_per_turn",
    "owner_alive_draw_player_owner_or_min_hp_twjiaohua_available",
    "owner_first_hand_discard_has_sha_in_discard",
    "owner_alive_event_player_outside_turn_from_hand_empty",
    "owner_alive_event_player_outside_turn_from_hand_empty_once",
    "owner_alive_other_gained_single_equip_to_hand_outside_turn_usable",
    "owner_alive_wugu_remained_in_discard",
    "owner_is_target_non_multitarget_trick_multiple_targets",
    "owner_source_play_phase_sha_has_additional_target_ignoring_distance",
    "owner_source_card_resolved_tracked_dddxianxi",
    "owner_alive_event_player_turn_changed_or_became_linked",
    "owner_alive_event_player_became_face_up_or_linked",
    "owner_turned_over",
    "owner_alive_other_wei_recovery_own_phase_once_per_turn",
    "owner_alive_global_juedou_or_red_sha_used",
    "owner_alive_other_global_juedou_or_red_sha_used",
    "owner_alive_event_player_distance_le_owner_hp",
    "owner_alive_other_normal_damage_target_unlinked",
    "owner_alive_damaged_event_player",
    "owner_alive_event_player_owner_or_female",
    "owner_alive_source_target_same_sex",
    "owner_alive_event_player_same_group",
    "owner_alive_event_player_same_group_hand_nonempty",
    "owner_tracked_juedou_original_target_damage_recoverable_in_range",
    "owner_is_target_phase_end_hand_nonempty_other_hand_nonempty",
    "owner_is_target_ordinary_trick_cilv_available",
    "cilv_tracked_card_damage_before",
    "cilv_tracked_card_resolved",
    "owner_dying_zhupeilan_once_per_game",
    "owner_alive_other_play_phase_tied_max_hand_dcsbfuxi_available",
    "owner_phase_end_dcsbhaoyi_available",
    "owner_alive_global_fire_damage_dcsbronghuo",
    "owner_source_fire_sha_or_huogong_dcsbronghuo",
    "owner_source_card_resolved_has_other_target_dcsbyingmou_once_per_turn",
    "dcsbyingmou_tracked_card_resolved",
    "owner_hand_transfer_other_dcsbmengmou_branch_available",
    "dcsbmengmou_tracked_sha_damage",
    "dcsbmengmou_tracked_sha_resolved",
    "owner_is_target_card_damage_before",
    "owner_source_single_target_sha_or_trick_all_materials_from_hand_can_sha",
    "dcyinjun_tracked_sha_resolved",
    "owner_wounded_not_event_source",
    "owner_wounded_event_player_qun",
    "owner_alive_not_target",
    "owner_alive_shan_used_by_owner_or_during_owner_turn",
    "owner_alive_other_phase_play_skill_pile_nonempty",
    "owner_alive_not_target_hand_equip_nonempty",
    "owner_source_sha_or_ordinary_trick_target_wounded_has_discardable_hand_equip",
    "owner_source_sha_or_ordinary_trick_has_discardable_hand_equip",
    "owner_source_sha_or_ordinary_trick",
    "owner_source_wuxie_used",
    "owner_source_sha_once_per_turn",
    "owner_is_target_hej_above_hp",
    "owner_source_play_card_repeats_previous_targets",
    "owner_source_sha_repeats_previous_sha_targets",
    "owner_equip_zone_changed",
    "owner_alive_not_target_neither_unique_max_hand",
    "owner_alive_not_target_source_same_group",
    "owner_alive_not_target_equal_hand",
    "owner_alive_not_target_hand_below_hp",
    "owner_alive_not_target_nonnegative_hp_gt_target",
    "owner_alive_current_turn_other_event_player",
    "owner_alive_current_turn_other_event_player_from_hand_empty",
    "owner_alive_current_turn",
    "owner_alive_group_count_at_most_2",
    "owner_lost_package_card_from_equip",
    "owner_gains_multiple_outside_own_draw",
    "owner_gained_or_lost_at_least_two",
    "owner_alive_event_player_play_phase_same_suit_at_least_two_once_per_turn",
    "owner_is_event_player_discard_pile_four_suits_below_alive_cap",
    "owner_card_responded_to_by_player_in_range",
    "owner_alive_first_cards_moved_to_hand",
    "owner_draws_during_own_play_not_self",
    "owner_source_non_equip_no_basic_hand_has_targets",
    "owner_equip_zone_changed_alive_wounded_at_least_two",
    "owner_current_turn_other_equipment_lost",
    "owner_alive_other_discarded_basic_outside_turn",
    "owner_alive_other_discarded_basic_outside_turn_once",
    "owner_alive_other_discarded_black_to_discard",
    "owner_alive_dealt_or_received_damage_this_turn",
    "owner_received_gift_from_other",
    "owner_initiated_duel_target_alive_loser_has_hand_or_equip",
    "owner_alive_equip_used",
    "owner_alive_equip_used_skill_mark_below_three",
    "owner_alive_source_alive_once_per_round",
    "owner_alive_source_alive_event_player_dying_once_per_round",
    "owner_alive_once_per_round",
    "owner_alive_not_target_once",
    "owner_alive_current_or_turn_fires_below_hp",
    "owner_alive_card_is_sha",
    "owner_alive_other_responded_sha_gainable",
    "owner_alive_card_targets_gt_one",
    "owner_alive_non_multitarget_trick_targets_gt_one",
    "owner_alive_wounded_card_targets_gt_one",
    "owner_source_sha_or_trick_current_play_phase",
    "owner_source_card_has_other_target_current_play_phase_first_two",
    "owner_alive_play_phase_sha_user_has_discardable_hand_equip",
    "owner_alive_first_target_virtual_or_converted_card",
    "owner_alive_targeted_by_other_single_card_discardable",
    "owner_alive_targeted_by_trick_single_or_discardable",
    "owner_alive_targeted_by_other_trick",
    "owner_alive_targeted_by_other_matching_equip_suit",
    "owner_alive_targeted_by_other",
    "owner_alive_targeted_by_other_black_card",
    "owner_alive_targeted_by_other_red_card",
    "owner_alive_other_source_death",
    "owner_is_target_play_phase_hand_no_sha",
    "owner_alive_other_phase_end_player_outside_owner_range",
    "owner_source_sha_target_alive_has_discardable_equip",
    "owner_source_card_used_targets_self",
    "owner_source_current_phase_card_targeted_other_healthy",
    "owner_source_first_card_suit_this_turn",
    "owner_lost_hand_now_empty",
    "owner_lost_spade_two_to_nine",
    "owner_min_hp",
    "owner_is_target_not_min_hp",
    "owner_target_min_hand",
    "owner_target_not_min_hand",
    "owner_sha_used_at_least_two",
    "owner_max_hp_gt_one_hand_or_equip_empty",
    "owner_has_enabled_equip_slot",
    "owner_is_event_player_no_enabled_equip_slot_once_per_game",
    "owner_is_target_has_movable_equipment",
    "owner_alive_used_sha_this_turn",
    "owner_alive_killed_this_turn",
    "always",
    // State / zone filters (CardsMoved etc.)
    "owner_hand_empty",
    "hand_empty",
    "owner_from_hand",
    "from_hand",
    "owner_from_equip",
    "from_equip",
    "owner_from_equip_per_card",
    "owner_from_hand_empty",
    "from_hand_empty",
    "owner_from_field_empty",
    "from_field_empty",
    "owner_targeted_by_trick_equip_empty",
    "owner_targeted_by_sha_hand_below_hp",
    "owner_targeted_by_sha_or_juedou_damaged_hp_le_source_once",
    "owner_targeted_outside_turn_by_sha_or_trick_hand_empty",
    "owner_targeted_hand_below_max_hp",
    "owner_targeted_by_other_single_target_trick",
    "owner_source_or_targeted_by_juedou",
    "owner_source_or_targeted_by_juedou_or_red_sha",
    "owner_source_or_targeted_by_sha_equal_hp",
    "owner_source_or_targeted_by_other_diamond",
    "owner_alive_is_target_current_play_phase",
    "owner_source_red",
    "owner_source_club",
    "owner_alive_other_play_phase_discardable_gte_mark",
    "owner_alive_other_trick_first_target_source_max_hand",
    "owner_alive_other_marked_xi",
    "owner_alive_other_recovered_outside_owner_turn",
    "owner_is_target_discarded_sha_in_discard_phase",
    "owner_source_sha_damage_before_target_healthy",
    "owner_is_target_single_target_other_source_once_per_turn",
    "owner_lost_one_hand_equip_judge_other_current_phase_once",
    "owner_source_alternates_color",
    "owner_source_repeats_color_has_discardable_hand_equip",
    "owner_alive_skill_bound_target_is_event_player",
    "owner_is_target_source_group_qun_discardable_hand_equip_gte_two",
    "owner_source_matches_previous_suit_or_rank",
    "owner_source_ordinary_trick",
    "owner_source_physical_ordinary_trick",
    "owner_hand_below_lost_hp_after_relevant_change",
    "owner_alive_source_first_card_ordinary_trick",
    "owner_card_action_count_eq_attack_range",
    "owner_lifetime_card_action",
    "owner_source_matches_last_used_suit_or_rank",
    "owner_alive_equip_source_max_equip",
    "owner_alive_other_current_source_repeats_card_name",
    "owner_source_hand_basic_equals_non_basic",
    "owner_is_target_event_card_sha",
    "owner_is_target_sha_or_trick",
    "owner_source_or_within_distance_one_sha_or_juedou",
    "owner_source_target_earlier_seat_hand_below_target_max_hp_once_per_turn",
    "owner_source_or_targeted_club_linked_exists_once_per_turn",
    "owner_source_sha",
    "owner_target_sha",
    "owner_source_heart_sha",
    "owner_source_active_dczhaowen_tracked_red",
    "owner_is_target_event_card_jiu",
    "owner_is_target_other_source_black_dcjiudun_available",
    "owner_source_sha_first_target",
    "owner_alive_other_single_target_juedou_or_red_sha",
    "owner_alive_hand_gained_this_turn_can_compare_other",
    "owner_alive_other_play_phase_zhengsu_missing_in_range",
    "owner_is_target_missing_zhengsu",
    "owner_source_sha_or_juedou_single_target",
    "owner_source_sha_current_play_phase",
    "owner_source_normal_sha",
    "owner_source_fire_sha",
    "owner_alive_other_source_single_sha_target_other",
    "owner_alive_other_source_single_sha_target_other_owner_in_attack_range",
    "owner_alive_other_source_sha_resolved_twxiaolian_redirect_has_hand_equip",
    "owner_alive_other_sha_resolved_tracked_skill_card_dealt_damage",
    "owner_alive_other_sha_resolved_tracked_skill_card_no_damage",
    "owner_tracked_skill_card_left_hand_not_use_or_respond",
    "owner_alive_sha_used_by_self_or_in_range_discardable_hand_equip",
    "owner_alive_tracked_cuijin_sha_resolved_no_damage",
    "owner_alive_tracked_cuijin_sha_resolved_dealt_damage",
    "owner_alive_tracked_twqingkou_duel_resolved",
    "owner_alive_twchenxun_round_unprompted",
    "owner_alive_tracked_twchenxun_duel_target_damaged",
    "owner_alive_tracked_twchenxun_duel_target_not_damaged",
    "owner_alive_tracked_twchenxun_card_left_processing",
    "owner_is_target_twxiaolian_redirected_damage",
    "owner_is_target_other_higher_hand_once_per_game",
    "owner_is_target_hand_sha_skill_pile_empty",
    "owner_alive_dying_has_skill_pile_cards",
    "owner_source_play_sha_other_alive_once_per_play_phase",
    "owner_source_jiu",
    "owner_is_target_hp_odd",
    "owner_is_target_hp_even_wounded",
    // Card / rate limits
    "owner_source_trick",
    "card_is_trick",
    "once_per_turn",
    "owner_source_once_per_turn",
    "owner_source_once",
    "owner_source_healthy",
    "owner_source_amount_gt_one",
    "owner_source_other_twice",
    "owner_alive_other_source_sha_twice",
    "owner_alive_other_source_sha_resolved_not_target_gainable_once_per_turn",
    "owner_source_single_target_sha_or_ordinary_trick_current_turn",
    "owner_source_event_card_policy_draw",
    "owner_source_card_resolved_phase_policy",
    "owner_phase_play_end_potguansha_available_once_per_game",
    "owner_discarded_sha_to_discard",
    "owner_alive_other_turn_discarded_recorded_potfuan_suit",
    "owner_source_sha_resolved_potfuan_dealt_damage",
    "owner_discarded_black_basic_to_discard",
    "owner_alive_discarded_black_basic_to_discard",
    "owner_alive_other_judge_phase_in_range_has_judge_card",
    "owner_source_basic",
    "owner_source_non_basic",
    "owner_source_basic_or_trick",
    "owner_source_wounded_nanman_wanjian_or_normal_spade_sha",
    "owner_alive_other_play_phase_duplicate_card_category_has_hand_equip",
    "owner_alive_other_phase_end_damage_gt_one_has_hand_equip",
    "owner_alive_other_phase_end_not_damaged_owner_this_turn",
    "owner_alive_other_phase_end_used_card_on_owner_undamaged_this_turn",
    "owner_alive_other_phase_end_has_discardable_basic_hand",
    "owner_alive_other_phase_end_hand_nonempty",
    "owner_alive_other_phase_begin_both_hand_nonempty",
    "owner_alive_other_phase_begin_unshown_hand_gte_hp",
    "owner_play_dddzhengjun_hp_available",
    "owner_play_dddzhengjun_cards_moved_available",
    "owner_alive_other_phase_play_dddchashi_available",
    "dddchashi_matching_card",
    "owner_alive_dddqice_ready",
    "owner_alive_ddd_fenji_available",
    "owner_alive_ddd_fengzheng_round_pending",
    "owner_alive_ddd_yulv_discard_matches",
    "owner_alive_ddd_youxue_round_pending",
    "owner_alive_ddd_chengjing_available",
    "owner_alive_ddd_zhuanshe_gift",
    "owner_alive_ddd_zhuanshe_card_used",
    "owner_alive_ddd_zhuanshe_phase_end",
    "owner_alive_ddd_weiqiu",
    "owner_alive_ddd_lianer_available",
    "owner_alive_ddd_lianer_ceiling",
    "owner_alive_ddd_anzhi_available",
    "owner_alive_ddd_anzhi_tracking",
    "owner_alive_ddd_anzhi_target_black",
    "owner_alive_ddd_anzhi_turn_end",
    "owner_alive_ddd_shixing_turn_refresh",
    "owner_alive_ddd_shixing_female_changed",
    "owner_alive_ddd_shixing_first_trick",
    "owner_alive_ddd_danggu_available",
    "owner_alive_ddd_fuzong_death",
    "owner_is_target_face_down",
    "owner_alive_ddd_xuanlun_pending",
    "owner_alive_ddd_jiexing_recover_once_per_game",
    "owner_alive_ddd_jiexing_equip_once_per_game",
    "owner_alive_ddd_jiexing_recovery_reset",
    "owner_alive_ddd_jiexing_equip_reset",
    "owner_alive_ddd_bailei_available",
    "owner_source_own_view_as_ddd_tongyu_used",
    "owner_source_own_view_as_ddd_tongyu_targeted",
    "owner_alive_ddd_tongyu_remained",
    "dddchashi_pending_phase_play_end",
    "owner_alive_dddzhilian_available",
    "owner_alive_source_basic_dddfushi_available",
    "owner_alive_ddd_dongcha_pile_nonempty",
    "owner_alive_prepare_player_hp_gt_owner",
    "owner_alive_prepare_lord_hp_gt_owner",
    "owner_alive_other_sha_target_can_redirect_to_owner",
    "owner_sha_target_can_discard_redirect_to_other_in_attack_range",
    "owner_sha_target_can_discard_redirect_to_zhengjian_marked",
    "owner_is_target_sha_source_alive_hand_equip_nonempty_once_per_turn",
    "owner_is_target_trick_targets_gt_source_hp_hand_equip_nonempty",
    "owner_is_source_sha_used",
    "owner_is_responder_sha",
    "owner_is_source_sha_card_damage_before",
    "owner_phase_end_lost_cards",
    "owner_alive_second_global_dying",
    "owner_used_and_was_targeted_this_turn_in_range",
    "owner_source_in_turn_target_unharmed_card_actions_ge_target_hp",
    "owner_is_target_sanshou_type_tracked",
    "owner_is_target_source_hp_gt_owner_or_source_damaged_owner",
    "owner_is_target_owner_hp_gt_source_or_source_damaged_owner",
    "owner_is_target_damage_amount_gt_one",
    "owner_is_target_source_hp_gt_owner_per_role_turn",
    "owner_alive_gave_cards_to_other_alive",
    "owner_alive_other_dying_hand_nonempty_once_per_game",
    "owner_dying_other_hand_equip_nonempty_once_per_game",
    "owner_alive_sha_or_duel_damage_source_target_alive_hand_equip_nonempty",
    "owner_is_target_any_alive_self_discardable_hand_equip",
    "owner_is_target_any_alive_discardable_by_owner_hand_equip",
    "owner_is_target_any_alive_self_discardable_hand",
    "owner_is_target_any_alive_wounded",
    "owner_is_target_can_compare_other",
    "owner_alive_sha_damage_source_target_alive_owner_hand_equip_nonempty",
    "owner_alive_source_ally_sha_damage_before",
    "owner_is_target_wounded_dead_ally",
    "owner_source_card_resolved_ddd_xuyu_pending",
    "ddd_xuyu_pending",
    "owner_alive_other_source_second_play_card_resolved_hand_equip_nonempty",
    "ddd_shijian_watched_source_card_used_targeted",
    "ddd_shijian_pending",
    "owner_alive_hp_changed_this_turn_entered_discard_available",
    "owner_alive_other_targeted_by_sha",
    "owner_alive_targeted_by_other_sha_judge_xumou",
    "owner_alive_event_player_lord",
    "owner_alive_event_player_lord_no_damage",
    "owner_alive_event_player_lord_damaged",
    "owner_alive_turn_used_all_suits",
    "cards_moved_discard_at_least_three",
    "owner_alive_play_end_used_categories_positive",
];
/// Allowed modifier targets.
const MOD_TARGETS: &[&str] = &[
    "distance_from",
    "distance_from_if_hp_gte_target",
    "distance_from_skill_bound_target",
    "distance_to",
    "global_distance_to",
    "nanman_damage_source_as_owner",
    "attack_range",
    "global_attack_range",
    "global_damage_taken_amount",
    "global_damage_taken_final_amount",
    "global_draw_count",
    "global_hand_limit",
    "global_sha_use_limit",
    "hand_limit",
    "hand_limit_base",
    "hand_limit_base_max_hp",
    "card_suit",
    "card_number",
    "card_name",
    "card_nature",
    "sha_unrespondable",
    "card_use_unrespondable",
    "sha_unrespondable_within_distance",
    "sha_use_unrespondable_within_distance",
    "card_use_unrespondable_within_responder_to_owner_distance",
    "card_use_unrespondable_within_owner_to_responder_distance",
    "sha_target_unrespondable",
    "damage_taken_source_range_threshold",
    "sha_damage_source_range_threshold",
    "card_distance_unlimited",
    "card_discard_protected",
    "card_gain_protected",
    "card_distance_unlimited_if_target_hand_gte_owner",
    "card_distance_unlimited_for_skill_bound_target",
    "card_action_cancel_for_skill_color_mask",
    "card_distance_limit",
    "sha_response_cancel",
    "sha_ignored_hand_limit",
    "card_ignored_hand_limit",
    "card_use_unlimited",
    "first_hand_card_suit_use_unlimited",
    "card_target_limit",
    "observer_foreign_hand_faces_visible",
    "observer_hidden_identities_visible",
    "draw_count",
    "optional_draw_count",
    "draw_phase",
    "sha_damage",
    "sha_damage_taken",
    "sha_damage_if_outside_target_range",
    "sha_use_limit",
    "sha_use_unlimited_for_skill_bound_target",
    "sha_distance_unlimited_if_rank_lt_skill_mark",
    "sha_distance_unlimited_if_target_distance_lte_rank",
    "sha_no_use_count_if_rank_gt_skill_mark",
    "sha_target_limit",
    "card_targets_all",
    "card_target_cancel",
    "armor_card_target_cancel",
    "armor_link_gain_cancel",
    "link_loss_cancel",
    "ignore_armor_for_skill_bound_target",
    "ignore_armor_for_skill_card_policy",
    "card_use_unrespondable_for_skill_card_policy",
    "card_distance_unlimited_for_skill_card_policy",
    "card_target_cancel_from_current",
    "card_target_cancel_owner_turn_banned",
    "card_target_cancel_if_target_lacks_matching_suit",
    "card_target_cancel_if_other_user_hand_gte_target_and_armor_empty",
    "card_target_cancel_if_other_user_hp_gte_target",
    "card_target_cancel_if_other_user_distance_gt_one",
    "card_targets_self_only",
    "response_count",
    "duel_opponent_response_count",
    "sha_response_count_if_other_female",
    "sha_response_count_if_source_owner_target_distance_le_one",
    "tao_use_save_cancel_for_non_dying_others_during_owner_turn",
    "tao_use_save_cancel_for_dying_beyond_distance",
    "skip_discard_if_no_sha",
    "skip_discard_phase",
    "restore_skipped_phase",
    "turn_face_down_cancel",
    "turn_over_cancel",
    "discard_phase_includes_equip",
    "skip_judge_phase",
    "initial_hand_count",
    "draw_replacement_damage",
    "prepare_scry_count",
    "draw_phase_scry_four_split_two",
    "judgment_rewrite",
    "judgment_rewrite_hand_equip",
    "judgment_rewrite_red_hand_equip_expansion",
    "judgment_rewrite_black_hand_equip",
    "skills_disabled_when_hp_one",
    "judgment_best_from_draw_pile",
    "virtual_bagua_if_armor_slot_empty",
    "damage_amount",
    "damage_as_hp_loss",
    "damage_between_owner_and_other_as_hp_loss",
    "damage_nature",
    "damage_cancel",
    "damage_taken_cancel",
    "damage_taken_amount",
    "armor_damage_taken_amount",
    "damage_taken_final_amount",
    "damage_taken_nature",
    "card_use_cancel",
    "card_response_cancel",
    "card_save_cancel",
    "hand_card_use_response_cancel",
    "hand_card_gained_this_turn_only",
    "sha_use_cancel",
    "immune_shunshou",
    "immune_le",
    "ignore_armor",
    "armor_effect_disabled",
];
/// Allowed modifier modes.
const MOD_MODES: &[&str] = &["add", "override", "multiply", "min", "max"];
/// Active windows.
const ACTIVE_WINDOWS: &[&str] = &["play_phase", "any", "response"];
const VIEW_AS_KINDS: &[&str] = &[
    "basic",
    "trick",
    "ordinary_trick",
    "sha",
    "shan",
    "tao",
    "jiu",
    // Trick view-as (奇袭 / 国色 …): map to existing play_as_kind trick handlers.
    "guohe",
    "shunshou",
    "juedou",
    "wuzhong",
    "le",
    "bingliang",
    "huogong",
    "jiedao",
    "tiesuo",
    "wuxie",
    "wanjian",
    "wugu",
    "nanman",
    "chuqibuyi",
];
const VIEW_AS_FILTERS: &[&str] = &[
    "red",
    "black",
    "any",
    "card_sha",
    "card_shan",
    "card_tao",
    "card_jiu",
    "card_ying",
    "card_basic",
    "card_equip",
    "card_trick",
    "damage_trick",
    "non_basic",
    "black_basic_equip",
    "heart",
    "spade",
    "club",
    "diamond",
    "owner_dying_club",
    "owner_outside_turn",
    "owner_outside_turn_red",
    "owner_single_hand",
    "owner_single_red_hand",
    "owner_hand_gt_hp",
    "owner_hp_le_two_black_trick",
    "color_ne_shuangxiong",
];
const VIEW_AS_SOURCE_ZONES: &[&str] = &[
    "hand",
    "equip",
    "hand_equip",
    "hand_expansion",
    "hand_equip_expansion",
    "none",
];
const VIEW_AS_COSTS: &[&str] = &[
    "gain_max_hp_1",
    "disable_equip_slot",
    "guhuo",
    "face_down_hand",
    "swap_lower_hand",
    "draw_one_outside_turn_empty_hand",
    "hand_equip_mark",
    "lose_max_hp_1",
    "all_hand",
    "potdaozhuan",
    "dczhaowen",
    "dddshilie",
    "dddzhijie",
    "dddfusi",
    "dddtongyu",
    "turn_over",
];
const USES_EVENT_AMOUNT: u32 = 1;
const USES_CHOOSE: u32 = 2;
const USES_EVENT_TARGET_COUNT: u32 = 4;
const USES_TURN_DAMAGED_PLAYER_COUNT: u32 = 8;
const USES_PREVENT_CURRENT_DAMAGE: u32 = 16;
const USES_JUDGE_CANCEL_CURRENT_SHA_TARGET: u32 = 32;
const USES_EVENT_CARD_COUNT: u32 = 65536;
const USES_INCREASE_CURRENT_DRAW_COUNT: u32 = 64;
const USES_PREPARE_TARGET_DISCARD_BRANCH: u32 = 128;
const USES_DAMAGE_TARGET_NEIGHBOR_GAIN: u32 = 256;
const USES_BOUND_DAMAGE_BRANCH: u32 = 512;
const USES_HAND_COMPARE_RANDOMIZE: u32 = 1024;
const USES_HAND_COMPARE_GAIN: u32 = 2048;
const USES_CARD_TARGET_ADJUST: u32 = 4096;
const USES_SELECTED_TARGETS: u32 = 8192;
const USES_ACTIVE_DAMAGE_BAN: u32 = 16384;
const USES_ACTIVE_TARGET_RECLAIM: u32 = 32768;

/// Typecheck AST → IR.
pub fn typecheck(file: &SourceFile) -> Result<RuleModuleIr, CompileError> {
    // Phase 3 compile-time cross-skill name resolution: build name→def map from all skill
    // declarations in this compilation unit, then rewrite `add_skill_by_name(player, NAME)` to
    // `add_skill(player, DEF)` at the AST level. This solves chained-helper resolution
    // (addTempSkill("X_effect")) without VM string-transport (ADR 0640 limit).
    let mut name_to_def: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    for sk in &file.skills {
        name_to_def.entry(sk.name.clone()).or_insert(sk.def_id);
    }
    let mut file = file.clone();
    if !name_to_def.is_empty() {
        for sk in &mut file.skills {
            for item in &mut sk.items {
                rewrite_add_skill_by_name(item, &name_to_def);
            }
        }
    }
    let mut skills = Vec::new();
    let mut seen_defs = std::collections::BTreeSet::new();
    for sk in &file.skills {
        if !seen_defs.insert(sk.def_id) {
            return Err(CompileError::Type {
                pos: sk.span.start as usize,
                message: format!("duplicate skill def id {}", sk.def_id),
            });
        }
        skills.push(check_skill(sk)?);
    }
    Ok(RuleModuleIr {
        name: "main".into(),
        skills,
    })
}

/// Recursively rewrite `add_skill_by_name(player, NAME)` → `add_skill(player, DEF)` in a skill item.
fn rewrite_add_skill_by_name(item: &mut SkillItemAst, name_to_def: &std::collections::BTreeMap<String, u32>) {
    match item {
        SkillItemAst::Trigger(t) => rewrite_stmts(&mut t.body, name_to_def),
        SkillItemAst::Active(a) => rewrite_stmts(&mut a.body, name_to_def),
        SkillItemAst::ViewAs(_) => {} // view_as uses declarative cost, no stmt body
        SkillItemAst::Modifier(_) => {}
    }
}

fn rewrite_stmts(stmts: &mut Vec<StmtAst>, name_to_def: &std::collections::BTreeMap<String, u32>) {
    for stmt in stmts.iter_mut() {
        rewrite_stmt(stmt, name_to_def);
    }
}

fn rewrite_stmt(stmt: &mut StmtAst, name_to_def: &std::collections::BTreeMap<String, u32>) {
    match stmt {
        StmtAst::Effect { name, args, .. } if name == "add_skill_by_name" => {
            if args.len() == 2 {
                if let ExprAst::Ident(skill_name) = &args[1] {
                    if let Some(def) = name_to_def.get(skill_name) {
                        *name = "add_skill".into();
                        args[1] = ExprAst::U32(*def);
                    }
                }
            }
        }
        StmtAst::If { then_body, else_body, .. } => {
            rewrite_stmts(then_body, name_to_def);
            rewrite_stmts(else_body, name_to_def);
        }
        StmtAst::Loop { body, .. } => {
            rewrite_stmts(body, name_to_def);
        }
        StmtAst::Let { value: _, .. } => {
            // no nested stmts in let value
        }
        _ => {}
    }
}

fn check_skill(sk: &SkillAst) -> Result<SkillIr, CompileError> {
    let mut triggers = Vec::new();
    let mut actives = Vec::new();
    let mut view_as = Vec::new();
    let mut modifiers = Vec::new();
    for item in &sk.items {
        match item {
            SkillItemAst::Trigger(t) => {
                if sk.locked && t.optional {
                    return Err(CompileError::Type {
                        pos: t.span.start as usize,
                        message: "locked skill trigger cannot be optional".into(),
                    });
                }
                triggers.push(check_trigger(t)?)
            }
            SkillItemAst::Active(a) => actives.push(check_active(a)?),
            SkillItemAst::ViewAs(v) => view_as.push(check_view_as(v)?),
            SkillItemAst::Modifier(m) => modifiers.push(check_modifier(m)?),
        }
    }
    Ok(SkillIr {
        name: sk.name.clone(),
        def_id: sk.def_id,
        locked: sk.locked,
        lord_only: sk.lord_only,
        non_gainable: sk.non_gainable,
        conversion: sk.conversion,
        triggers,
        actives,
        view_as,
        modifiers,
        span: sk.span,
    })
}

fn check_view_as(v: &ViewAsAst) -> Result<ViewAsIr, CompileError> {
    if !VIEW_AS_KINDS.contains(&v.as_kind.as_str()) {
        return Err(CompileError::Type {
            pos: v.span.start as usize,
            message: format!("unknown view_as kind {}", v.as_kind),
        });
    }
    if !ACTIVE_WINDOWS.contains(&v.window.as_str()) {
        return Err(CompileError::Type {
            pos: v.span.start as usize,
            message: format!("unknown view_as window {}", v.window),
        });
    }
    if !VIEW_AS_FILTERS.contains(&v.filter.as_str()) {
        return Err(CompileError::Type {
            pos: v.span.start as usize,
            message: format!("unknown view_as filter {}", v.filter),
        });
    }
    if let Some(zone) = &v.source_zone {
        let delegated_group = zone.strip_prefix("group_").is_some_and(|group| {
            !group.is_empty()
                && group
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        });
        if !VIEW_AS_SOURCE_ZONES.contains(&zone.as_str()) && !delegated_group {
            return Err(CompileError::Type {
                pos: v.span.start as usize,
                message: format!("unknown view_as source zone {zone}"),
            });
        }
        let valid_delegated_card = (v.as_kind == "shan" && v.window == "response")
            || (v.as_kind == "sha" && v.window == "any");
        if delegated_group
            && (!valid_delegated_card
                || v.filter != "any"
                || v.uses_per_turn.is_some()
                || v.ignore_distance
                || v.red_unrespondable
                || v.black_no_use_count
                || v.no_use_count
                || v.target_lacks_source_suit_unrespondable
                || v.refund_use_count_if_no_damage
                || v.min_cards != 1
                || v.max_cards != 1
                || v.owner_hp_min_one_cards || v.owner_basic_uses_plus_one_cards
                || v.same_suit
                || v.nature.is_some()
                || v.multi_card_bonus.is_some()
                || v.branch.is_some()
                || v.once_per_round
                || v.next_branch.is_some()
                || v.cost.is_some())
        {
            return Err(CompileError::Type {
                pos: v.span.start as usize,
                message: "group-provided view_as requires `view_as shan response any from group_<tag>` or `view_as sha any any from group_<tag>`".into(),
            });
        }
    }
    if v.uses_per_turn == Some(0) {
        return Err(CompileError::Type {
            pos: v.span.start as usize,
            message: "view_as uses_per_turn requires a positive count".into(),
        });
    }
    if (v.red_unrespondable
        || v.black_no_use_count
        || v.no_use_count
        || v.target_lacks_source_suit_unrespondable
        || v.refund_use_count_if_no_damage)
        && v.as_kind != "sha"
    {
        return Err(CompileError::Type {
            pos: v.span.start as usize,
            message: "view_as source-color outcome clauses require sha".into(),
        });
    }
    let potqingyan = v.cost.as_deref() == Some("potqingyan");
    let dddtongyu = v.cost.as_deref() == Some("dddtongyu");
    if v.min_cards == 0
        || v.min_cards > v.max_cards
        || if potqingyan {
            v.max_cards != 5
        } else if dddtongyu {
            v.max_cards != 4
        } else {
            v.max_cards > 2 && v.max_cards != u8::MAX
        }
    {
        return Err(CompileError::Type {
            pos: v.span.start as usize,
            message: "view_as source-card count is invalid for this conversion".into(),
        });
    }
    if v.same_suit
            && v.max_cards == 1
            && !v.owner_hp_min_one_cards
            && !v.owner_basic_uses_plus_one_cards
        {
        return Err(CompileError::Type {
            pos: v.span.start as usize,
            message: "view_as same_suit requires up_to 2".into(),
        });
    }
    if v.nature
        .as_deref()
        .is_some_and(|nature| !matches!(nature, "fire" | "thunder"))
        || (v.nature.is_some() && v.as_kind != "sha")
    {
        return Err(CompileError::Type {
            pos: v.span.start as usize,
            message: "view_as nature supports only `sha ... nature fire|thunder`".into(),
        });
    }
    if let Some(bonus) = v.multi_card_bonus.as_deref() {
        let valid = v.max_cards > 1
            && matches!(
                (v.as_kind.as_str(), bonus),
                ("sha", "damage_1")
                    | ("tao", "recover_1")
                    | ("shan" | "wuxie", "discard_current_hand_equip")
            );
        if !valid {
            return Err(CompileError::Type {
                pos: v.span.start as usize,
                message: "view_as multi_card_bonus requires up_to 2 and a matching damage_1, recover_1, or discard_current_hand_equip conversion".into(),
            });
        }
    }
    let source_free = v.source_zone.as_deref() == Some("none");
    let dynamic_family = matches!(v.as_kind.as_str(), "basic" | "trick" | "ordinary_trick")
        || source_free && matches!(v.as_kind.as_str(), "tao" | "jiu");
    let has_branch_contract = v.branch.is_some()
        || v.once_per_round
        || v.next_branch.is_some()
        || v.cost.is_some()
        || v.mark_cost.is_some()
        || v.post_draw != 0;
    if source_free != dynamic_family {
        return Err(CompileError::Type {
            pos: v.span.start as usize,
            message:
                "source-free view_as families require `basic|trick|ordinary_trick ... from none`"
                    .into(),
        });
    }
    if source_free {
        let unrestricted_physical = matches!(
            v.cost.as_deref(),
            Some(
                "guhuo"
                    | "face_down_hand"
                    | "dczhaowen"
                    | "draw_one_outside_turn_empty_hand"
                    | "hand_equip_mark"
                    | "dddfusi"
                    | "turn_over"
            )
        );
        let turn_limited = matches!(
            v.cost.as_deref(),
            Some(
                "swap_lower_hand" | "hand_equip_mark" | "lose_max_hp_1" | "all_hand" | "dddshilie"
            )
        );
        let potdaozhuan = v.cost.as_deref() == Some("potdaozhuan");
        let concrete_liezhi = matches!(v.as_kind.as_str(), "tao" | "jiu")
            && v.cost.as_deref() == Some("lose_max_hp_1")
            && v.uses_per_turn == Some(1)
            && v.window == "any";
        if v.filter != "any"
            || !v
                .cost
                .as_deref()
                .is_some_and(|cost| VIEW_AS_COSTS.contains(&cost))
            || if concrete_liezhi {
                v.branch.is_some() || v.once_per_round || v.next_branch.is_some()
            } else if potdaozhuan {
                v.branch.is_some()
                    || v.once_per_round
                    || v.next_branch.is_some()
                    || v.uses_per_turn.is_some()
                    || v.window != "any"
            } else if turn_limited {
                v.branch.is_some()
                    || v.once_per_round
                    || v.next_branch.is_some()
                    || v.uses_per_turn != Some(1)
            } else if unrestricted_physical {
                v.branch.is_some()
                    || v.once_per_round
                    || v.next_branch.is_some()
                    || v.window != "any"
            } else {
                v.branch.is_none() || !v.once_per_round || v.next_branch.is_none()
            }
        {
            return Err(CompileError::Type {
                pos: v.span.start as usize,
                message: "source-free view_as requires a branch cost contract, or an unrestricted physical-card cost".into(),
            });
        }
        if v.branch.is_some_and(|branch| branch >= 32)
            || v.next_branch.is_some_and(|branch| branch >= 32)
        {
            return Err(CompileError::Type {
                pos: v.span.start as usize,
                message: "source-free view_as branch must be below 32".into(),
            });
        }
        if (v.uses_per_turn.is_some() && !turn_limited)
            || v.ignore_distance
            || v.red_unrespondable
            || v.black_no_use_count
            || v.no_use_count
            || v.target_lacks_source_suit_unrespondable
            || v.refund_use_count_if_no_damage
            || v.min_cards != 1
            || v.max_cards != 1
            || v.owner_hp_min_one_cards || v.owner_basic_uses_plus_one_cards
            || v.same_suit
            || v.nature.is_some()
            || v.multi_card_bonus.is_some()
        {
            return Err(CompileError::Type {
                pos: v.span.start as usize,
                message: "source-free view_as cannot use source-card clauses".into(),
            });
        }
        if v.cost.as_deref() == Some("turn_over") && (v.as_kind != "jiu" || v.window != "any") {
            return Err(CompileError::Type {
                pos: v.span.start as usize,
                message: "turn_over source-free cost requires `view_as jiu any any from none`"
                    .into(),
            });
        }
        if v.cost.as_deref() == Some("dddshilie")
            && (v.as_kind != "basic" || v.window != "any" || v.uses_per_turn != Some(1))
        {
            return Err(CompileError::Type {
                pos: v.span.start as usize,
                message: "dddshilie requires `view_as basic any any from none cost dddshilie uses_per_turn 1`".into(),
            });
        }
        if v.cost.as_deref() == Some("dddfusi")
            && (!matches!(v.as_kind.as_str(), "basic" | "trick") || v.window != "any")
        {
            return Err(CompileError::Type {
                pos: v.span.start as usize,
                message: "dddfusi requires `view_as basic|trick any any from none cost dddfusi`"
                    .into(),
            });
        }
        let mark_contract = v.cost.as_deref() == Some("hand_equip_mark");
        if mark_contract != v.mark_cost.is_some() || (v.post_draw != 0 && !mark_contract) {
            return Err(CompileError::Type {
                pos: v.span.start as usize,
                message: "hand_equip_mark requires mark_cost and exclusively supports post_draw"
                    .into(),
            });
        }
    } else if dddtongyu {
        if v.as_kind != "wugu"
            || v.window != "play_phase"
            || v.filter != "any"
            || v.source_zone.as_deref().is_some_and(|zone| zone != "hand")
            || v.min_cards != 1
            || v.max_cards != 4
            || v.uses_per_turn != Some(1)
            || v.ignore_distance
            || v.red_unrespondable
            || v.black_no_use_count
            || v.no_use_count
            || v.target_lacks_source_suit_unrespondable
            || v.refund_use_count_if_no_damage
            || v.owner_hp_min_one_cards || v.owner_basic_uses_plus_one_cards
            || v.same_suit
            || v.nature.is_some()
            || v.multi_card_bonus.is_some()
            || v.branch.is_some()
            || v.once_per_round
            || v.next_branch.is_some()
            || v.mark_cost.is_some()
            || v.post_draw != 0
        {
            return Err(CompileError::Type {
                pos: v.span.start as usize,
                message: "dddtongyu requires `view_as wugu play_phase any from hand up_to 4 uses_per_turn 1 cost dddtongyu`".into(),
            });
        }
    } else if v.cost.as_deref() == Some("dddzhijie") {
        if !matches!(v.as_kind.as_str(), "shan" | "wuxie")
            || v.window != "response"
            || v.filter != "any"
            || v.source_zone.as_deref() != Some("hand_expansion")
            || v.min_cards != 2
            || v.max_cards != 2
            || v.uses_per_turn.is_some()
            || v.ignore_distance
            || v.red_unrespondable
            || v.black_no_use_count
            || v.no_use_count
            || v.target_lacks_source_suit_unrespondable
            || v.refund_use_count_if_no_damage
            || v.owner_hp_min_one_cards || v.owner_basic_uses_plus_one_cards
            || v.same_suit
            || v.nature.is_some()
            || v.multi_card_bonus.is_some()
            || v.branch.is_some()
            || v.once_per_round
            || v.next_branch.is_some()
            || v.mark_cost.is_some()
            || v.post_draw != 2
        {
            return Err(CompileError::Type {
                pos: v.span.start as usize,
                message: "dddzhijie requires `shan|wuxie response any from hand_expansion exactly 2 cost dddzhijie post_draw 2`".into(),
            });
        }
    } else if potqingyan {
        if !matches!(v.as_kind.as_str(), "shan" | "wuxie")
            || v.window != "response"
            || v.filter != "any"
            || v.source_zone.as_deref().is_some_and(|zone| zone != "hand")
            || v.min_cards != 1
            || v.uses_per_turn.is_some()
            || v.ignore_distance
            || v.red_unrespondable
            || v.black_no_use_count
            || v.no_use_count
            || v.target_lacks_source_suit_unrespondable
            || v.refund_use_count_if_no_damage
            || v.owner_hp_min_one_cards || v.owner_basic_uses_plus_one_cards
            || v.same_suit
            || v.nature.is_some()
            || v.multi_card_bonus.is_some()
            || v.branch.is_some()
            || v.once_per_round
            || v.next_branch.is_some()
            || v.mark_cost.is_some()
            || v.post_draw != 0
        {
            return Err(CompileError::Type {
                pos: v.span.start as usize,
                message: "potqingyan requires `shan|wuxie response any from hand up_to 5 cost potqingyan`".into(),
            });
        }
    } else if has_branch_contract {
        return Err(CompileError::Type {
            pos: v.span.start as usize,
            message: "view_as branch clauses require `from none`".into(),
        });
    }
    Ok(ViewAsIr {
        as_kind: v.as_kind.clone(),
        window: v.window.clone(),
        filter: v.filter.clone(),
        source_zone: v
            .source_zone
            .as_ref()
            .filter(|zone| zone.as_str() != "hand")
            .cloned(),
        uses_per_turn: v.uses_per_turn,
        ignore_distance: v.ignore_distance,
        red_unrespondable: v.red_unrespondable,
        black_no_use_count: v.black_no_use_count,
        no_use_count: v.no_use_count,
        target_lacks_source_suit_unrespondable: v.target_lacks_source_suit_unrespondable,
        refund_use_count_if_no_damage: v.refund_use_count_if_no_damage,
        min_cards: v.min_cards,
        max_cards: v.max_cards,
        owner_hp_min_one_cards: v.owner_hp_min_one_cards,
        owner_basic_uses_plus_one_cards: v.owner_basic_uses_plus_one_cards,
        once_per_turn_per_branch: v.once_per_turn_per_branch,
        same_suit: v.same_suit,
        nature: v.nature.clone(),
        multi_card_bonus: v.multi_card_bonus.clone(),
        branch: v.branch,
        once_per_round: v.once_per_round,
        next_branch: v.next_branch,
        cost: v.cost.clone(),
        mark_cost: v.mark_cost,
        post_draw: v.post_draw,
        span: v.span,
    })
}

fn check_trigger(t: &TriggerAst) -> Result<TriggerIr, CompileError> {
    if stmts_use_effect(&t.body, "resolve_dingpan") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_dingpan is only valid in an active skill".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_tanlu")
        && !(t.event == "phase_begin"
            && t.filter == "owner_alive_not_target"
            && t.optional
            && t.choice_actor.is_owner())
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_tanlu requires `trigger phase_begin owner_alive_not_target optional`"
                .into(),
        });
    }
    if stmts_use_effect(&t.body, "choose_current_trick_targets_to_nullify")
        && !(t.event == "card_used"
            && t.filter == "owner_alive_non_multitarget_trick_targets_gt_one"
            && !t.optional)
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "choose_current_trick_targets_to_nullify requires `trigger card_used owner_alive_non_multitarget_trick_targets_gt_one`".into(),
        });
    }
    if !EVENTS.contains(&t.event.as_str()) {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: format!("unknown event {}", t.event),
        });
    }
    if !valid_trigger_filter(&t.filter) {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: format!("unknown filter {}", t.filter),
        });
    }
    let features = stmts_features(&t.body);
    if stmts_use_effect(&t.body, "discard_selected_draw_all_hand_bonus") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "discard-selected draw bonus is only available in active skills".into(),
        });
    }
    if stmts_use_effect(&t.body, "draw_then_choose_give_or_discard_hand") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "draw_then_choose_give_or_discard_hand is only available in active skills"
                .into(),
        });
    }
    if stmts_use_effect(&t.body, "inspect_target_hand_choose_heart_discard_or_top") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "target-hand heart inspection is only available in active skills".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_yeyan") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_yeyan is only available in active skills".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_rehuaiyi") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_rehuaiyi is only available in active skills".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_reanguo") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_reanguo is only available in active skills".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_beizhu") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_beizhu is only available in active skills".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_mbshanjia") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_mbshanjia is only available in active skills".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_daoji") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_daoji is only available in active skills".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_tw_daoji") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_tw_daoji is only available in active skills".into(),
        });
    }
    if stmts_use_effect(&t.body, "give_all_hand_then_choose_use_one_or_damage") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message:
                "give_all_hand_then_choose_use_one_or_damage is only available in active skills"
                    .into(),
        });
    }
    if stmts_use_effect(&t.body, "choose_lose_hp_use_virtual_sha_by_basic_variants")
        && (t.event != "phase_end" || t.filter != "owner_is_target")
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "choose_lose_hp_use_virtual_sha_by_basic_variants requires `trigger phase_end owner_is_target`".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_jiaohua") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_jiaohua is only available in active skills".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_luanqun") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_luanqun is only available in active skills".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_mbzhuguo") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_mbzhuguo is only available in active skills".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_mbanxian")
        && (t.event != "cards_moved" || t.filter != "owner_first_hand_discard_has_sha_in_discard")
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_mbanxian requires `trigger cards_moved owner_first_hand_discard_has_sha_in_discard`".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_mbjinzu") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_mbjinzu is only available in active skills".into(),
        });
    }
    if stmts_use_effect(&t.body, "apply_mbjinzu_next_sha")
        && (t.event != "card_used" || t.filter != "owner_source_sha")
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "apply_mbjinzu_next_sha requires `trigger card_used owner_source_sha`".into(),
        });
    }
    if stmts_use_effect(&t.body, "choose_discard_weapon_or_lose_hp") {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "discard-weapon-or-lose-HP cost is only available in active skills".into(),
        });
    }
    if stmts_use_effect(
        &t.body,
        "choose_gain_target_card_alternate_zone_then_target_draw",
    ) {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "alternate-zone target gain is only available in active skills".into(),
        });
    }
    if stmts_use_effect(&t.body, "choose_all_lose_hp_or_recover")
        && (t.event != "phase_discard_end"
            || t.filter != "owner_is_target_discarded_at_least_two_in_discard_phase")
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "choose_all_lose_hp_or_recover requires phase_discard_end with owner_is_target_discarded_at_least_two_in_discard_phase".into(),
        });
    }
    if stmts_use_effect(&t.body, "replace_draw_with_distinct_suit_selection")
        && t.event != "phase_draw"
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "distinct-suit draw replacement is only available in phase_draw triggers"
                .into(),
        });
    }
    if stmts_use_effect(
        &t.body,
        "replace_draw_gain_top_two_add_skills_if_different_colors",
    ) && t.event != "phase_draw"
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "top-two draw replacement is only available in phase_draw triggers".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_new_reluoyi") && t.event != "phase_draw" {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_new_reluoyi is only available in phase_draw triggers".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_remoshi")
        && !matches!(t.event.as_str(), "damage" | "cards_moved")
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_remoshi is only available in damage or cards_moved triggers".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_xinyaoming") && t.event != "damage" {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_xinyaoming is only available in damage triggers".into(),
        });
    }
    if stmts_use_effect(&t.body, "choose_use_virtual_card") && t.event != "phase_any_end" {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "choose_use_virtual_card is only available in phase_any_end triggers".into(),
        });
    }
    if stmts_use_effect(
        &t.body,
        "choose_draw_or_increase_current_card_damage_or_both_mark",
    ) && t.event != "card_used"
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message:
                "choose_draw_or_increase_current_card_damage_or_both_mark is only available in card_used triggers"
                    .into(),
        });
    }
    if features & USES_EVENT_AMOUNT != 0
        && !matches!(
            t.event.as_str(),
            "damage_before" | "damage" | "hp_lost" | "hp_recovered" | "hand_compare_finished"
        )
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "event_amount is only available in damage_before, damage, hp_lost, hp_recovered, or hand_compare_finished triggers"
                .into(),
        });
    }
    if stmts_use_effect(&t.body, "lose_hp_double_current_damage_then_replace")
        && t.event != "damage_before"
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message:
                "lose_hp_double_current_damage_then_replace is only available in damage_before triggers"
                    .into(),
        });
    }
    if stmts_use_effect(&t.body, "modify_current_damage") && t.event != "damage_before" {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "modify_current_damage is only available in damage_before triggers".into(),
        });
    }
    if features & USES_PREVENT_CURRENT_DAMAGE != 0 && t.event != "damage_before" {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "prevent_current_damage is only available in damage_before triggers".into(),
        });
    }
    if stmts_use_effect(&t.body, "choose_discard_heart_redirect_current_damage")
        && t.event != "damage_before"
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message:
                "choose_discard_heart_redirect_current_damage is only available in damage_before triggers"
                    .into(),
        });
    }
    if stmts_use_effect(&t.body, "choose_discard_heart_cancel_damage_branch")
        && t.event != "damage_before"
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message:
                "choose_discard_heart_cancel_damage_branch is only available in damage_before triggers"
                    .into(),
        });
    }
    if stmts_use_effect(&t.body, "choose_link_other_same_hp") && t.event != "phase_prepare" {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "choose_link_other_same_hp is only available in phase_prepare triggers".into(),
        });
    }
    if features & USES_INCREASE_CURRENT_DRAW_COUNT != 0 && t.event != "phase_draw" {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "increase_current_draw_count is only available in phase_draw triggers".into(),
        });
    }
    if features & USES_PREPARE_TARGET_DISCARD_BRANCH != 0 && t.event != "phase_prepare" {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "choose_target_discard_excess_then_self_discard_or_target_draw is only available in phase_prepare triggers".into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_decadezhenjun")
        && !matches!(t.event.as_str(), "phase_prepare" | "phase_end")
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message:
                "resolve_decadezhenjun is only available in phase_prepare or phase_end triggers"
                    .into(),
        });
    }
    if stmts_use_effect(&t.body, "resolve_sp_xiaoguo") && t.event != "phase_end" {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "resolve_sp_xiaoguo is only available in phase_end triggers".into(),
        });
    }
    if features & USES_DAMAGE_TARGET_NEIGHBOR_GAIN != 0 && t.event != "damage" {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "draw_then_choose_gain_one_from_damage_target_and_neighbors is only available in damage triggers".into(),
        });
    }
    if features & USES_BOUND_DAMAGE_BRANCH != 0 && t.event != "damage_before" {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message:
                "choose_prevent_damage_or_lose_max_hp_increase_damage is only available in damage_before triggers"
                    .into(),
        });
    }
    if features & USES_HAND_COMPARE_RANDOMIZE != 0 && t.event != "hand_compare_started" {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "randomize_current_hand_compare_opponent is only available in hand_compare_started triggers".into(),
        });
    }
    if features & USES_HAND_COMPARE_GAIN != 0 && t.event != "hand_compare_finished" {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "comparison result-card gain effects are only available in hand_compare_finished triggers".into(),
        });
    }
    if features & USES_CARD_TARGET_ADJUST != 0 && t.event != "card_used" {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message:
                "choose_add_or_remove_current_card_target is only available in card_used triggers"
                    .into(),
        });
    }
    if features & USES_SELECTED_TARGETS != 0 {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "selected-target effects are only available in active skills".into(),
        });
    }
    if features & USES_ACTIVE_DAMAGE_BAN != 0 {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "damage-linked active effects are only available in active skills".into(),
        });
    }
    if features & USES_ACTIVE_TARGET_RECLAIM != 0 {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "target-equipment reclamation is only available in active skills".into(),
        });
    }
    if features & USES_JUDGE_CANCEL_CURRENT_SHA_TARGET != 0 && t.event != "card_targeted" {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "current Sha target effects are only available in card_targeted triggers"
                .into(),
        });
    }
    if features & USES_EVENT_TARGET_COUNT != 0
        && !matches!(t.event.as_str(), "card_used" | "card_targeted")
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "event_target_count is only available in card triggers".into(),
        });
    }
    if features & USES_EVENT_CARD_COUNT != 0
        && !matches!(t.event.as_str(), "card_used" | "card_targeted" | "cards_moved")
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "event_card_count is only available in card/move triggers".into(),
        });
    }
    if features & USES_TURN_DAMAGED_PLAYER_COUNT != 0 && t.event != "phase_end" {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "turn_damaged_player_count is only available in phase_end triggers".into(),
        });
    }
    // ponytail: VM continuations do not persist event scalar values yet; lift
    // this restriction when a real skill needs one together with choose().
    if features
        & (USES_EVENT_AMOUNT
            | USES_EVENT_TARGET_COUNT
            | USES_EVENT_CARD_COUNT
            | USES_TURN_DAMAGED_PLAYER_COUNT)
        != 0
        && features & USES_CHOOSE != 0
    {
        return Err(CompileError::Type {
            pos: t.span.start as usize,
            message: "event values cannot be combined with choose()".into(),
        });
    }
    let mut locals = Vec::new();
    let body = check_stmts(&t.body, &mut locals)?;
    Ok(TriggerIr {
        event: t.event.clone(),
        filter: t.filter.clone(),
        optional: t.optional,
        choice_actor: t.choice_actor,
        body,
        span: t.span,
    })
}

fn valid_trigger_filter(filter: &str) -> bool {
    FILTERS.contains(&filter)
        || filter
            .strip_prefix("owner_is_target_sha_skill_pile_nonempty_")
            .is_some_and(|value| value.parse::<u32>().is_ok_and(|value| value > 0))
        || filter
            .strip_prefix("owner_alive_other_play_discard_skill_pile_nonempty_")
            .is_some_and(|value| value.parse::<u32>().is_ok_and(|value| value > 0))
        || filter
            .strip_prefix("owner_source_own_view_as_card_targeted_target_skill_pile_")
            .is_some_and(|value| value.parse::<u32>().is_ok_and(|value| value > 0))
        || filter == "owner_source_play_card_count_eq_hp"
        || filter
            .strip_prefix("owner_source_play_skill_mark_mode_")
            .is_some_and(|mark| mark.parse::<u32>().is_ok_and(|value| value > 0))
        || filter
            .strip_prefix("owner_source_play_card_count_eq_")
            .is_some_and(|count| count.parse::<u32>().is_ok_and(|value| value > 0))
        || filter
            .strip_prefix("owner_is_target_active_used_at_least_")
            .and_then(|value| value.strip_suffix("_this_play_phase"))
            .is_some_and(|threshold| threshold.parse::<u32>().is_ok_and(|value| value > 0))
        || filter
            .strip_prefix("owner_is_target_hp_below_")
            .is_some_and(|threshold| threshold.parse::<u32>().is_ok_and(|value| value > 0))
        || filter
            .strip_prefix("owner_is_target_source_alive_mark_positive_")
            .is_some_and(|mark| mark.parse::<u32>().is_ok_and(|value| value > 0))
        || filter
            .strip_prefix("owner_is_target_any_field_card_no_skill_pile_")
            .is_some_and(|value| value.parse::<u32>().is_ok_and(|value| value > 0))
        || filter
            .strip_prefix("owner_alive_source_sha_hand_equip_skill_pile_")
            .is_some_and(|value| value.parse::<u32>().is_ok_and(|value| value > 0))
        || filter
            .strip_prefix("owner_alive_dying_skill_pile_")
            .is_some_and(|value| value.parse::<u32>().is_ok_and(|value| value > 0))
        || filter
            .strip_prefix("owner_is_target_mark_positive_")
            .is_some_and(|mark| mark.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_source_target_mark_positive_")
            .is_some_and(|mark| mark.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_alive_card_targeted_marked_peer_")
            .is_some_and(|mark| mark.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_mark_positive_")
            .is_some_and(|mark| mark.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_skill_added_self_")
            .is_some_and(|skill_def| skill_def.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_skill_pile_nonempty_")
            .is_some_and(|pile_def| pile_def.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_source_ordinary_trick_mark_positive_")
            .is_some_and(|mark| mark.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_source_sha_or_black_trick_mark_positive_")
            .is_some_and(|mark| mark.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_gains_outside_turn_mark_below_alive_once_")
            .is_some_and(|mark| mark.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_source_jiu_boosted_sha_damage_skill_")
            .and_then(|value| value.strip_suffix("_enabled"))
            .is_some_and(|skill_def| skill_def.parse::<u32>().is_ok_and(|value| value > 0))
        || filter
            .strip_prefix("owner_source_damage_target_mark_positive_")
            .and_then(|value| value.strip_suffix("_once"))
            .is_some_and(|mark| mark.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_alive_marked_alive_above_two_")
            .is_some_and(|mark| mark.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_alive_someone_mark_positive_")
            .is_some_and(|mark| mark.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_alive_discard_matches_own_pile_card_")
            .is_some_and(|pile_def| pile_def.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_alive_damage_before_event_target_mark_positive_")
            .is_some_and(|mark| mark.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_alive_phase_begin_event_target_mark_positive_")
            .is_some_and(|mark| mark.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_alive_sha_resolved_city_")
            .is_some_and(|skill_def| skill_def.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_alive_event_target_mark_positive_")
            .and_then(|value| value.strip_suffix("_no_damage"))
            .is_some_and(|mark| mark.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_alive_event_target_mark_positive_")
            .is_some_and(|mark| mark.parse::<u32>().is_ok())
        || filter
            .strip_prefix("owner_source_hand_gaintag_absent_")
            .is_some_and(|tag| tag.parse::<u32>().is_ok())
        || [
            "owner_alive_targeted_by_other_hidden_skill_pile_",
            "owner_is_target_hand_nonempty_hidden_skill_pile_",
            "owner_lost_matching_revealed_skill_pile_suit_",
            "owner_phase_begin_skill_pile_gte_three_",
        ]
        .into_iter()
        .any(|prefix| {
            filter
                .strip_prefix(prefix)
                .is_some_and(|skill_def| skill_def.parse::<u32>().is_ok_and(|value| value > 0))
        })
        || filter
            .strip_prefix("owner_alive_other_shan_outside_turn_group_")
            .and_then(|value| value.strip_suffix("_once_per_turn"))
            .is_some_and(|group| {
                !group.is_empty()
                    && group.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                    })
            })
        // Clan 中流 family: clan name is an arbitrary (CJK) string between a fixed prefix
        // and suffix, so only non-emptiness is required (no ascii-group validation).
        || filter
            .strip_prefix("owner_is_event_player_no_")
            .and_then(|rest| rest.strip_suffix("_lost_hand_this_turn"))
            .is_some_and(|clan| !clan.is_empty())
        // 保族 (clanbaozu) family: event player is owner or member of <clan> (CJK).
        || filter
            .strip_prefix("owner_alive_event_player_self_or_clan_")
            .is_some_and(|clan| !clan.is_empty())
        || [
            "owner_alive_event_player_group_",
            "owner_source_target_not_group_",
            "owner_wounded_event_player_group_",
            "owner_alive_other_judgment_black_event_player_group_",
            "owner_alive_other_damage_source_group_",
            "owner_dying_other_tao_responder_group_",
        ]
        .into_iter()
        .any(|prefix| {
            filter.strip_prefix(prefix).is_some_and(|group| {
                !group.is_empty()
                    && group.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                    })
            })
        })
        || valid_card_name_scoped_filter(filter)
        || valid_event_view_as_skill_filter(filter)
}

/// Skill-provenance trigger filter family (Phase 3): accepts `event_view_as_skill_<DEF>` where
/// `<DEF>` is a positive u32 skill definition id (analogue of upstream `event.skill == "X"`),
/// and `event_view_as_skill_name_<NAME>` where `<NAME>` is a lowercase skill name (covers
/// dynamic _backup helpers without a static def id).
fn valid_event_view_as_skill_filter(filter: &str) -> bool {
    if let Some(def) = filter.strip_prefix("event_view_as_skill_name_") {
        return !def.is_empty()
            && def.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    }
    filter
        .strip_prefix("event_view_as_skill_")
        .is_some_and(|def| !def.is_empty() && def.bytes().all(|b| b.is_ascii_digit()) && def.parse::<u32>().is_ok_and(|v| v > 0))
}

/// Card-name-scoped trigger filter family (Phase 3): accepts
/// `owner_source_card_<selector>`, `owner_is_target_card_<selector>`,
/// `owner_is_target_card_damage_<selector>` where `<selector>` is a known card-selector name
/// mirrored from the engine's `map_card_selector`.
fn valid_card_name_scoped_filter(filter: &str) -> bool {
    const SELECTORS: &[&str] = &[
        "any", "sha", "tao", "bingliang", "lebu", "basic", "non_basic", "trick", "equip",
        "spade", "heart", "club", "diamond", "red", "black", "juedou", "black_trick",
        "damage_trick", "sha_fire", "huogong", "jiu", "physical_jiu", "sha_red", "nanman",
        "ordinary_trick", "sha_black", "sha_elemental", "equip_non_mount", "armor_or_treasure",
        "wanjian", "sha_normal", "sha_heart", "delayed_trick", "wuxie", "guohe", "shunshou",
        "sha_thunder", "hand_rank_letter", "sole_hand_card", "sha_diamond", "damage_card",
    ];
    for prefix in [
        "owner_source_card_",
        "owner_is_target_card_damage_",
        "owner_is_target_card_",
    ] {
        if let Some(selector) = filter.strip_prefix(prefix) {
            return SELECTORS.contains(&selector);
        }
    }
    false
}

fn stmts_features(stmts: &[StmtAst]) -> u32 {
    stmts.iter().fold(0, |all, stmt| all | stmt_features(stmt))
}

fn stmts_use_effect(stmts: &[StmtAst], effect: &str) -> bool {
    stmts.iter().any(|stmt| match stmt {
        StmtAst::Effect { name, .. } => name == effect,
        StmtAst::If {
            then_body,
            else_body,
            ..
        } => stmts_use_effect(then_body, effect) || stmts_use_effect(else_body, effect),
        StmtAst::Loop { body, .. } => stmts_use_effect(body, effect),
        StmtAst::Let { .. } => false,
    })
}

fn stmts_use_effect_ident_arg(
    stmts: &[StmtAst],
    effect: &str,
    arg_index: usize,
    ident: &str,
) -> bool {
    stmts.iter().any(|stmt| match stmt {
        StmtAst::Effect { name, args, .. } => {
            name == effect
                && matches!(args.get(arg_index), Some(ExprAst::Ident(value)) if value == ident)
        }
        StmtAst::If {
            then_body,
            else_body,
            ..
        } => {
            stmts_use_effect_ident_arg(then_body, effect, arg_index, ident)
                || stmts_use_effect_ident_arg(else_body, effect, arg_index, ident)
        }
        StmtAst::Loop { body, .. } => stmts_use_effect_ident_arg(body, effect, arg_index, ident),
        StmtAst::Let { .. } => false,
    })
}

fn stmt_features(stmt: &StmtAst) -> u32 {
    match stmt {
        StmtAst::Effect { name, args, .. } => {
            args.iter().fold(0, |all, expr| all | expr_features(expr))
                | u32::from(
                    name == "prevent_current_damage"
                        || name == "judge_prevent_damage_lose_max_hp_on_nonheart"
                        || name == "choose_discard_equipment_reduce_current_damage"
                        || name == "choose_discard_equipment_prevent_current_damage",
                ) * USES_PREVENT_CURRENT_DAMAGE
                | u32::from(name == "judge_cancel_current_sha_target_on_red")
                    * USES_JUDGE_CANCEL_CURRENT_SHA_TARGET
                | u32::from(name == "make_current_sha_target_unrespondable")
                    * USES_JUDGE_CANCEL_CURRENT_SHA_TARGET
                | u32::from(name == "judge_make_current_sha_target_unrespondable_on_red")
                    * USES_JUDGE_CANCEL_CURRENT_SHA_TARGET
                | u32::from(
                    name == "resolve_retieji"
                        || name == "choose_category_then_target_give_matching_or_sha_unrespondable",
                ) * USES_JUDGE_CANCEL_CURRENT_SHA_TARGET
                | u32::from(name == "choose_current_sha_target_discard_redirect_to_owner")
                    * USES_JUDGE_CANCEL_CURRENT_SHA_TARGET
                | u32::from(name == "choose_discard_target_card_resolve_current_sha")
                    * USES_JUDGE_CANCEL_CURRENT_SHA_TARGET
                | u32::from(name == "increase_current_draw_count")
                    * USES_INCREASE_CURRENT_DRAW_COUNT
                | u32::from(name == "choose_target_discard_excess_then_self_discard_or_target_draw")
                    * USES_PREPARE_TARGET_DISCARD_BRANCH
                | u32::from(name == "draw_then_choose_gain_one_from_damage_target_and_neighbors")
                    * USES_DAMAGE_TARGET_NEIGHBOR_GAIN
                | u32::from(name == "choose_prevent_damage_or_lose_max_hp_increase_damage")
                    * USES_BOUND_DAMAGE_BRANCH
                | u32::from(name == "choose_category_then_target_give_matching_or_damage_plus_one")
                    * USES_BOUND_DAMAGE_BRANCH
                | u32::from(name == "randomize_current_hand_compare_opponent")
                    * USES_HAND_COMPARE_RANDOMIZE
                | u32::from(name == "gain_highest_rank_sha_from_event_cards")
                    * USES_HAND_COMPARE_GAIN
                | u32::from(name == "gain_hand_compare_result_card") * USES_HAND_COMPARE_GAIN
                | u32::from(name == "choose_add_or_remove_current_card_target")
                    * USES_CARD_TARGET_ADJUST
                | u32::from(
                    name == "use_virtual_card_on_selected_targets"
                        || name == "draw_selected_targets"
                        || name == "gain_one_hand_from_selected_targets"
                        || name == "recover_selected_targets"
                        || name == "take_target_hand_then_repeat_until_max"
                        || name == "resolve_selected_equipment_discard_then_choose_damage",
                ) * USES_SELECTED_TARGETS
                | u32::from(
                    name == "deal_damage_ban_active_if_dying"
                        || name == "lose_hp_then_damage_other_lose_max_hp_if_dying",
                ) * USES_ACTIVE_DAMAGE_BAN
                | u32::from(name == "choose_target_reclaim_equipment_draw_if_range_reduced")
                    * USES_ACTIVE_TARGET_RECLAIM
        }
        StmtAst::If {
            cond,
            then_body,
            else_body,
            ..
        } => expr_features(cond) | stmts_features(then_body) | stmts_features(else_body),
        StmtAst::Loop { body, .. } => stmts_features(body),
        StmtAst::Let { value, .. } => expr_features(value),
    }
}

fn expr_features(expr: &ExprAst) -> u32 {
    match expr {
        ExprAst::Ident(name) if name == "event_amount" => USES_EVENT_AMOUNT,
        ExprAst::Ident(name) if name == "event_target_count" => USES_EVENT_TARGET_COUNT,
        ExprAst::Ident(name) if name == "event_card_count" => USES_EVENT_CARD_COUNT,
        ExprAst::Ident(name) if name == "turn_damaged_player_count" => {
            USES_TURN_DAMAGED_PLAYER_COUNT
        }
        ExprAst::Call { name, args } => {
            u32::from(name == "choose") * USES_CHOOSE
                | args.iter().fold(0, |all, expr| all | expr_features(expr))
        }
        ExprAst::BinOp { left, right, .. } => expr_features(left) | expr_features(right),
        _ => 0,
    }
}

fn check_active(a: &ActiveAst) -> Result<ActiveIr, CompileError> {
    if stmts_use_effect(&a.body, "resolve_tanlu") {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "resolve_tanlu is only valid in a phase_begin trigger".into(),
        });
    }
    if stmts_use_effect(&a.body, "choose_use_virtual_card") {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "choose_use_virtual_card is only available in phase_any_end triggers".into(),
        });
    }
    if !ACTIVE_WINDOWS.contains(&a.window.as_str()) {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: format!("unknown active window {}", a.window),
        });
    }
    if stmts_use_effect(&a.body, "resolve_rehuaiyi")
        && (a.window != "play_phase"
            || a.uses_per_play_phase != Some(2)
            || !a.hand_nonempty
            || a.targets.is_some()
            || a.selection.is_some())
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "resolve_rehuaiyi requires `active play_phase uses_per_play_phase 2 hand_nonempty` without declarative targets or card selection".into(),
        });
    }
    if stmts_use_effect(&a.body, "resolve_reanguo")
        && (a.window != "play_phase"
            || a.uses_per_turn != Some(1)
            || a.targets
                .as_ref()
                .is_none_or(|targets| targets.min != 1 || targets.max != 1 || targets.allow_self)
            || a.selection.is_some())
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "resolve_reanguo requires `active play_phase uses_per_turn 1 targets 1 1 other` without card selection".into(),
        });
    }
    if stmts_use_effect(&a.body, "resolve_beizhu")
        && (a.window != "play_phase"
            || a.uses_per_turn != Some(1)
            || a.targets.as_ref().is_none_or(|targets| {
                targets.min != 1 || targets.max != 1 || targets.allow_self || !targets.hand_nonempty
            })
            || a.selection.is_some())
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "resolve_beizhu requires `active play_phase uses_per_turn 1 targets 1 1 other hand_nonempty` without card selection".into(),
        });
    }
    if stmts_use_effect(&a.body, "resolve_mbshanjia")
        && (a.window != "play_phase"
            || a.uses_per_turn != Some(1)
            || a.targets.is_some()
            || a.selection.is_some())
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "resolve_mbshanjia requires `active play_phase uses_per_turn 1` without declarative targets or card selection".into(),
        });
    }
    if stmts_use_effect(&a.body, "resolve_daoji")
        && (a.window != "play_phase"
            || a.uses_per_turn != Some(1)
            || a.targets.as_ref().is_none_or(|targets| {
                targets.min != 1
                    || targets.max != 1
                    || targets.allow_self
                    || !targets.has_gainable_equip
            })
            || a.selection.as_ref().is_none_or(|selection| {
                selection.zone != "hand_equip"
                    || selection.min != 1
                    || selection.max != 1
                    || selection.filter.as_deref() != Some("card_nonbasic")
            }))
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "resolve_daoji requires `active play_phase uses_per_turn 1 targets 1 1 other has_gainable_equip select_he 1 1 card_nonbasic`".into(),
        });
    }
    if stmts_use_effect(&a.body, "resolve_tw_daoji")
        && (a.window != "play_phase"
            || a.uses_per_turn != Some(1)
            || a.targets.as_ref().is_none_or(|targets| {
                targets.min != 1
                    || targets.max != 1
                    || targets.allow_self
                    || !targets.has_gainable_hand_equip
                    || !targets.in_attack_range
            })
            || a.selection.as_ref().is_none_or(|selection| {
                selection.zone != "hand_equip"
                    || selection.min != 1
                    || selection.max != 1
                    || selection.filter.as_deref() != Some("card_nonbasic")
            }))
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "resolve_tw_daoji requires `active play_phase uses_per_turn 1 targets 1 1 other has_gainable_hand_equip in_attack_range select_he 1 1 card_nonbasic`".into(),
        });
    }
    if stmts_use_effect(&a.body, "give_all_hand_then_choose_use_one_or_damage")
        && (a.window != "play_phase"
            || a.uses_per_turn != Some(1)
            || !a.hand_nonempty
            || a.targets
                .as_ref()
                .is_none_or(|targets| targets.min != 1 || targets.max != 1 || targets.allow_self)
            || a.selection.is_some())
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "give_all_hand_then_choose_use_one_or_damage requires `active play_phase uses_per_turn 1 hand_nonempty targets 1 1 other` without card selection".into(),
        });
    }
    if stmts_use_effect(&a.body, "resolve_jiaohua")
        && (a.window != "play_phase"
            || a.uses_per_play_phase != Some(2)
            || a.uses_per_turn.is_some()
            || a.targets.is_some()
            || a.selection.is_some())
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "resolve_jiaohua requires `active play_phase uses_per_play_phase 2` without declarative targets or card selection".into(),
        });
    }
    if stmts_use_effect(&a.body, "resolve_luanqun")
        && (a.window != "play_phase"
            || a.uses_per_turn != Some(1)
            || a.uses_per_play_phase.is_some()
            || !a.hand_nonempty
            || a.targets.is_some()
            || a.selection.is_some())
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "resolve_luanqun requires `active play_phase uses_per_turn 1 hand_nonempty` without declarative targets or card selection".into(),
        });
    }
    if stmts_use_effect(&a.body, "resolve_refuhai")
        && (a.window != "play_phase"
            || a.uses_per_turn != Some(1)
            || a.uses_per_play_phase.is_some()
            || a.targets.is_some()
            || a.selection.is_some())
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "resolve_refuhai requires `active play_phase uses_per_turn 1` without declarative targets or card selection".into(),
        });
    }
    if stmts_use_effect(&a.body, "choose_gain_target_hand_ceil_half_difference")
        && (a.window != "play_phase"
            || a.uses_per_turn != Some(1)
            || !a.targets.as_ref().is_some_and(|targets| {
                targets.min == 1 && targets.max == 1 && !targets.allow_self && targets.hand_gt_owner
            })
            || !a.selection.as_ref().is_some_and(|selection| {
                selection.zone == "hand"
                    && selection.min == 1
                    && selection.max == 1
                    && selection.filter.as_deref() == Some("card_heart")
            }))
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "choose_gain_target_hand_ceil_half_difference requires `active play_phase uses_per_turn 1 targets 1 1 other hand_gt_owner select_hand 1 1 card_heart`".into(),
        });
    }
    if stmts_use_effect(&a.body, "resolve_mbzhuguo")
        && (a.window != "play_phase"
            || a.uses_per_turn != Some(1)
            || a.uses_per_play_phase.is_some()
            || !matches!(
                a.targets.as_ref(),
                Some(targets)
                    if targets.min == 1
                        && targets.max == 1
                        && targets.allow_self
            )
            || a.selection.is_some())
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "resolve_mbzhuguo requires `active play_phase uses_per_turn 1 targets 1 1` without card selection".into(),
        });
    }
    if stmts_use_effect(&a.body, "resolve_mbanxian") {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "resolve_mbanxian is only available in trigger skills".into(),
        });
    }
    if stmts_use_effect(&a.body, "resolve_mbjinzu")
        && (a.window != "play_phase"
            || a.uses_per_turn != Some(1)
            || a.uses_per_play_phase.is_some()
            || a.targets
                .as_ref()
                .is_none_or(|targets| targets.min != 1 || targets.max != 1 || targets.allow_self)
            || a.selection.as_ref().is_none_or(|selection| {
                selection.zone != "hand"
                    || selection.min != 1
                    || selection.max != 1
                    || selection.allow_cancel
                    || selection.filter.is_some()
            }))
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "resolve_mbjinzu requires `active play_phase uses_per_turn 1 targets 1 1 other select_hand 1 1`".into(),
        });
    }
    if stmts_use_effect(&a.body, "apply_mbjinzu_next_sha") {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "apply_mbjinzu_next_sha is only available in trigger skills".into(),
        });
    }
    if stmts_use_effect(
        &a.body,
        "choose_category_then_target_give_matching_or_sha_unrespondable",
    ) {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "choose_category_then_target_give_matching_or_sha_unrespondable is only available in trigger skills".into(),
        });
    }
    if stmts_use_effect(
        &a.body,
        "choose_category_then_target_give_matching_or_damage_plus_one",
    ) {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "choose_category_then_target_give_matching_or_damage_plus_one is only available in trigger skills".into(),
        });
    }
    if stmts_use_effect(&a.body, "choose_lose_hp_use_virtual_sha_by_basic_variants") {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "choose_lose_hp_use_virtual_sha_by_basic_variants is only available in trigger skills".into(),
        });
    }
    if a.targets
        .as_ref()
        .is_some_and(|targets| targets.linked && targets.unlinked)
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "active targets cannot require both linked and unlinked".into(),
        });
    }
    if a.owner_missing_skill == Some(0) {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "owner_missing_skill requires a non-zero skill definition".into(),
        });
    }
    if a.targets
        .as_ref()
        .and_then(|targets| targets.max_by_owner_mark)
        == Some(0)
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "max_by_owner_mark requires a non-zero mark id".into(),
        });
    }
    if stmts_use_effect(
        &a.body,
        "resolve_selected_equipment_discard_then_choose_damage",
    ) && a.targets.is_none()
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "selected equipment discard/damage requires active targets".into(),
        });
    }
    if stmts_use_effect(&a.body, "replace_draw_with_distinct_suit_selection") {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "distinct-suit draw replacement is only available in triggers".into(),
        });
    }
    if stmts_use_effect(&a.body, "inspect_target_hand_choose_heart_discard_or_top")
        && !a.targets.as_ref().is_some_and(|targets| {
            targets.min == 1 && targets.max == 1 && !targets.allow_self && targets.hand_nonempty
        })
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "target-hand heart inspection requires `targets 1 1 other hand_nonempty`"
                .into(),
        });
    }
    if stmts_use_effect(&a.body, "choose_all_lose_hp_or_recover") {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "choose_all_lose_hp_or_recover is only available in triggers".into(),
        });
    }
    if stmts_use_effect(&a.body, "resolve_yeyan")
        && (!a.once_per_game || a.targets.is_some() || a.selection.is_some())
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "resolve_yeyan requires a targetless, cardless `once_per_game` active skill"
                .into(),
        });
    }
    if stmts_use_effect(&a.body, "discard_selected_draw_all_hand_bonus")
        && !a
            .selection
            .as_ref()
            .is_some_and(|selection| selection.zone == "hand_equip" && selection.min > 0)
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "discard-selected draw bonus requires `select_he`".into(),
        });
    }
    if let Some(selection) = &a.selection {
        if selection.min == 0 || selection.max < selection.min {
            return Err(CompileError::Type {
                pos: a.span.start as usize,
                message: "active selection requires 1 <= min <= max".into(),
            });
        }
        if selection.filter.as_deref().is_some_and(|filter| {
            !matches!(
                filter,
                "card_equip"
                    | "card_armor"
                    | "card_sha"
                    | "card_basic"
                    | "card_nonbasic"
                    | "card_non_equip"
                    | "card_red"
                    | "card_black"
                    | "card_heart"
            )
        }) {
            return Err(CompileError::Type {
                pos: a.span.start as usize,
                message:
                    "active selection filter must be card_equip, card_armor, card_sha, card_basic, card_nonbasic, card_non_equip, card_red, card_black, or card_heart"
                        .into(),
            });
        }
    }
    if let Some(targets) = &a.targets {
        if targets.max < targets.min {
            return Err(CompileError::Type {
                pos: a.span.start as usize,
                message: "active targets require min <= max".into(),
            });
        }
        if targets.lowest_hp && targets.highest_hp {
            return Err(CompileError::Type {
                pos: a.span.start as usize,
                message: "active targets cannot require both lowest_hp and highest_hp".into(),
            });
        }
        if targets
            .can_use_on_owner
            .as_deref()
            .is_some_and(|kind| kind != "juedou")
        {
            return Err(CompileError::Type {
                pos: a.span.start as usize,
                message: "active can_use_on_owner currently supports juedou".into(),
            });
        }
        if targets.second_can_use_juedou_on_first
            && (targets.min != 2 || targets.max != 2 || targets.allow_self)
        {
            return Err(CompileError::Type {
                pos: a.span.start as usize,
                message: "second_can_use_juedou_on_first requires `targets 2 2 other`".into(),
            });
        }
        if targets.equipment_pair_with_armor && (targets.min != 2 || targets.max != 2) {
            return Err(CompileError::Type {
                pos: a.span.start as usize,
                message: "equipment_pair_with_armor requires targets 2 2".into(),
            });
        }
        if targets.match_selected_card_count
            && !a.selection.as_ref().is_some_and(|selection| {
                selection.min == targets.min && selection.max == targets.max
            })
        {
            return Err(CompileError::Type {
                pos: a.span.start as usize,
                message:
                    "match_selected_card_count requires a card selection with the same min/max"
                        .into(),
            });
        }
        if targets.selected_cards_match_target_equip_count
            && !(targets.min == 1 && targets.max == 1 && targets.has_equip && a.selection.is_some())
        {
            return Err(CompileError::Type {
                pos: a.span.start as usize,
                message: "selected_cards_match_target_equip_count requires `targets 1 1 has_equip` and a card selection".into(),
            });
        }
    }
    if a.uses_per_turn == Some(0) {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "active uses_per_turn requires a positive count".into(),
        });
    }
    if a.uses_per_play_phase == Some(0) {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "active uses_per_play_phase requires a positive count".into(),
        });
    }
    if a.uses_per_play_phase.is_some() && a.window != "play_phase" {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "active uses_per_play_phase requires the play_phase window".into(),
        });
    }
    if a.uses_per_play_phase.is_some()
        && (a.uses_per_turn.is_some() || a.uses_per_turn_source.is_some())
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "active use limits cannot combine turn and Play-phase scopes".into(),
        });
    }
    if a.uses_per_turn_source.as_deref().is_some_and(|source| {
        !matches!(
            source,
            "owner_hp" | "owner_lost_hp_min_one" | "living_rebel_count"
        )
    }) {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message:
                "active uses_per_turn source must be owner_hp, owner_lost_hp_min_one, or living_rebel_count"
                    .into(),
        });
    }
    if a.mark_at_least
        .is_some_and(|(mark, count)| mark == 0 || count == 0)
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "active mark_at_least requires non-zero mark and count".into(),
        });
    }
    if a.selection.as_ref().is_some_and(|selection| {
        [
            selection.unused_category_by_skill_this_turn,
            selection.unused_category_by_skill_this_play_phase,
            selection.unused_category_by_owner_history_this_turn,
        ]
        .into_iter()
        .filter(|enabled| *enabled)
        .count()
            > 1
    }) {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "active selection cannot combine category-history constraints".into(),
        });
    }
    if a.selection.as_ref().is_some_and(|selection| {
        selection.sum_effective_number_exact.is_some() && selection.zone != "hand"
    }) {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "sum_effective_number_exact requires select_hand".into(),
        });
    }
    if a.targets.as_ref().is_some_and(|targets| {
        targets.unused_by_skill_this_turn && targets.unused_by_skill_this_play_phase
    }) {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "active targets cannot combine turn and play-phase uniqueness constraints"
                .into(),
        });
    }
    let features = stmts_features(&a.body);
    let turn_unique_category_target = a.targets.as_ref().is_some_and(|targets| {
        targets.min == 1
            && targets.max == 1
            && !targets.allow_self
            && targets.unused_by_skill_this_turn
    }) && a.selection.as_ref().is_some_and(|selection| {
        selection.zone == "hand_equip"
            && selection.min == 1
            && selection.max == 1
            && selection.unused_category_by_skill_this_turn
    });
    let play_phase_unique_category_target = a.targets.as_ref().is_some_and(|targets| {
        targets.min == 1
            && targets.max == 1
            && targets.allow_self
            && targets.unused_by_skill_this_play_phase
    }) && a.selection.as_ref().is_some_and(|selection| {
        selection.zone == "hand_equip"
            && selection.min == 1
            && selection.max == 1
            && selection.unused_category_by_skill_this_play_phase
    });
    if stmts_use_effect(&a.body, "discard_selected_record_unique_category_target")
        && !(turn_unique_category_target || play_phase_unique_category_target)
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "unique-category target discard requires the exact turn-scoped `other` contract or play-phase-scoped self-allowed contract".into(),
        });
    }
    if stmts_use_effect(
        &a.body,
        "choose_target_draw_or_discard_by_skill_use_count_link_if_wounded",
    ) && !turn_unique_category_target
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "link-if-wounded count branch requires the exact turn-scoped unique-category target contract".into(),
        });
    }
    if stmts_use_effect(
        &a.body,
        "choose_target_draw_or_discard_by_skill_use_count_toggle_link_if_wounded",
    ) && !play_phase_unique_category_target
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "toggle-link-if-wounded count branch requires the exact play-phase-scoped unique-category target contract".into(),
        });
    }
    let selected_virtual_duel =
        stmts_use_effect_ident_arg(&a.body, "use_virtual_card_on_selected_targets", 1, "juedou");
    let selected_virtual_tiesuo =
        stmts_use_effect_ident_arg(&a.body, "use_virtual_card_on_selected_targets", 1, "tiesuo");
    if (selected_virtual_duel || selected_virtual_tiesuo)
        && !a.targets.as_ref().is_some_and(|targets| {
            targets.min > 0
                && (!selected_virtual_duel || !targets.allow_self)
                && (!selected_virtual_tiesuo || targets.max <= 2)
        })
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "use_virtual_card_on_selected_targets requires legal active targets (`juedou`: other; `tiesuo`: 1..2)".into(),
        });
    }
    if stmts_use_effect(
        &a.body,
        "choose_gain_target_card_alternate_zone_then_target_draw",
    ) && !a.targets.as_ref().is_some_and(|targets| {
        targets.min == 1 && targets.max == 1 && !targets.allow_self && targets.hand_equip_nonempty
    }) {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "alternate-zone target gain requires `targets 1 1 other hand_equip_nonempty`"
                .into(),
        });
    }
    if stmts_use_effect(&a.body, "use_virtual_juedou_second_on_first")
        && !a.targets.as_ref().is_some_and(|targets| {
            targets.min == 2
                && targets.max == 2
                && !targets.allow_self
                && targets.male
                && targets.second_can_use_juedou_on_first
        })
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "use_virtual_juedou_second_on_first requires `targets 2 2 other male second_can_use_juedou_on_first`".into(),
        });
    }
    if stmts_use_effect(
        &a.body,
        "target_discard_then_duel_if_non_sha_and_hp_gte_owner",
    ) && !a.targets.as_ref().is_some_and(|targets| {
        targets.min == 1
            && targets.max == 1
            && !targets.allow_self
            && targets.in_attack_range
            && targets.hand_equip_nonempty
    }) {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "conditional target-discard Duel requires `targets 1 1 other in_attack_range hand_equip_nonempty`".into(),
        });
    }
    if stmts_use_effect(&a.body, "draw_selected_targets")
        && !a.targets.as_ref().is_some_and(|targets| targets.min > 0)
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "draw_selected_targets requires at least one active target".into(),
        });
    }
    if stmts_use_effect(&a.body, "gain_one_hand_from_selected_targets")
        && !a.targets.as_ref().is_some_and(|targets| {
            targets.min > 0
                && !targets.allow_self
                && targets.max_by_owner_hp
                && targets.hand_nonempty
        })
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "gain_one_hand_from_selected_targets requires `targets ... other max_by_owner_hp hand_nonempty`".into(),
        });
    }
    if stmts_use_effect(&a.body, "recover_selected_targets")
        && !a.targets.as_ref().is_some_and(|targets| targets.min > 0)
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "recover_selected_targets requires at least one active target".into(),
        });
    }
    if stmts_use_effect(&a.body, "damage_then_draw_selected_targets")
        && !a.targets.as_ref().is_some_and(|targets| targets.min > 0)
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "damage_then_draw_selected_targets requires active targets".into(),
        });
    }
    if stmts_use_effect(&a.body, "record_selected_play_phase_category")
        && !a.selection.as_ref().is_some_and(|selection| {
            selection.min == 1
                && selection.max == 1
                && selection.unused_category_by_skill_this_play_phase
        })
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "record_selected_play_phase_category requires exact one-card selection with unused_category_by_skill_this_play_phase"
                .into(),
        });
    }
    if stmts_use_effect(&a.body, "lose_hp_then_damage_other_lose_max_hp_if_dying")
        && !a
            .targets
            .as_ref()
            .is_some_and(|targets| targets.min == 1 && targets.max == 1 && !targets.allow_self)
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "HP-loss-linked damage requires `targets 1 1 other`".into(),
        });
    }
    if stmts_use_effect(&a.body, "take_target_hand_then_repeat_until_max")
        && !a.targets.as_ref().is_some_and(|targets| {
            targets.min == 1 && targets.max == 1 && !targets.allow_self && targets.hand_gt_owner
        })
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "repeated larger-hand take requires `targets 1 1 other hand_gt_owner`".into(),
        });
    }
    if features & USES_ACTIVE_TARGET_RECLAIM != 0
        && !a.targets.as_ref().is_some_and(|targets| {
            targets.min == 1 && targets.max == 1 && !targets.allow_self && targets.has_equip
        })
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "target-equipment reclamation requires `targets 1 1 other has_equip`".into(),
        });
    }
    if stmts_use_effect(&a.body, "discard_target_equipment_draw_weapon_range")
        && !a.targets.as_ref().is_some_and(|targets| {
            targets.min == 1 && targets.max == 1 && targets.has_discardable_equip
        })
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "weapon-range equipment discard requires `targets 1 1 has_discardable_equip`"
                .into(),
        });
    }
    if stmts_use_effect(&a.body, "reveal_target_hand_gain_shan_armor_or_draw_two")
        && !a.targets.as_ref().is_some_and(|targets| {
            targets.min == 1 && targets.max == 1 && !targets.allow_self && targets.hand_nonempty
        })
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "hand reveal gain requires `targets 1 1 other hand_nonempty`".into(),
        });
    }
    if stmts_use_effect(&a.body, "choose_discard_target_cards_recover_same_category")
        && !a.targets.as_ref().is_some_and(|targets| {
            targets.min == 1
                && targets.max == 1
                && !targets.allow_self
                && targets.in_attack_range
                && targets.has_discardable_hand_equip
        })
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "target discard-and-recover requires `targets 1 1 other in_attack_range has_discardable_hand_equip`".into(),
        });
    }
    if stmts_use_effect(
        &a.body,
        "choose_compare_selected_hand_card_second_failure_seat_damage",
    ) && !(a.targets.as_ref().is_some_and(|targets| {
        targets.min == 1
            && targets.max == 1
            && !targets.allow_self
            && targets.can_compare
            && targets.unused_by_skill_this_turn
    }) && a.selection.as_ref().is_some_and(|selection| {
        selection.zone == "hand" && selection.min == 1 && selection.max == 1
    })) {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "comparison failure path requires `targets 1 1 other can_compare unused_by_skill_this_turn select_hand 1 1`".into(),
        });
    }
    if stmts_use_effect(&a.body, "give_all_hand")
        && !(a.hand_nonempty
            && a.face_up
            && a.targets.as_ref().is_some_and(|targets| {
                targets.min == 1 && targets.max == 1 && !targets.allow_self && targets.male
            }))
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "give_all_hand requires `hand_nonempty face_up targets 1 1 other male`".into(),
        });
    }
    if stmts_use_effect(
        &a.body,
        "target_choose_suit_gain_owner_hand_damage_if_mismatch",
    ) && !(a.hand_nonempty
        && a.targets
            .as_ref()
            .is_some_and(|targets| targets.min == 1 && targets.max == 1 && !targets.allow_self))
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "suit-guess hand gain requires `hand_nonempty targets 1 1 other`".into(),
        });
    }
    if stmts_use_effect(&a.body, "give_selected_recover_after_phase_total_two")
        && !(a
            .targets
            .as_ref()
            .is_some_and(|targets| targets.min == 1 && targets.max == 1 && !targets.allow_self)
            && a.selection.as_ref().is_some_and(|selection| {
                selection.zone == "hand" && selection.min == 1 && selection.max == u8::MAX
            }))
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "phase gift requires `targets 1 1 other select_hand 1 255`".into(),
        });
    }
    if stmts_use_effect(
        &a.body,
        "give_selected_then_continue_distinct_recover_at_two",
    ) && !(a.uses_per_turn == Some(1)
        && a.targets
            .as_ref()
            .is_some_and(|targets| targets.min == 1 && targets.max == 1 && !targets.allow_self)
        && a.selection.as_ref().is_some_and(|selection| {
            selection.zone == "hand" && selection.min == 1 && selection.max == u8::MAX
        }))
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message:
                "distinct phase gift requires `uses_per_turn 1 targets 1 1 other select_hand 1 255`"
                    .into(),
        });
    }
    if stmts_use_effect(&a.body, "give_selected_then_recover_target")
        && !(a
            .targets
            .as_ref()
            .is_some_and(|targets| targets.min == 1 && targets.max == 1 && !targets.allow_self)
            && a.selection
                .as_ref()
                .is_some_and(|selection| selection.zone == "hand" && selection.min > 0))
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "selected gift and recovery requires `targets 1 1 other select_hand 1 ...`"
                .into(),
        });
    }
    if stmts_use_effect(&a.body, "give_selected_then_target_may_use_draw_owner")
        && !(a.window == "play_phase"
            && a.uses_per_turn == Some(1)
            && a.targets
                .as_ref()
                .is_some_and(|targets| targets.min == 1 && targets.max == 1 && !targets.allow_self)
            && a.selection.as_ref().is_some_and(|selection| {
                selection.zone == "hand"
                    && selection.min == 1
                    && selection.max == 1
                    && selection.filter.as_deref() == Some("card_non_equip")
            }))
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "yuanlve requires `active play_phase uses_per_turn 1 targets 1 1 other select_hand 1 1 card_non_equip`".into(),
        });
    }
    if stmts_use_effect(&a.body, "resolve_dingpan")
        && !(a.window == "play_phase"
            && a.uses_per_turn_source.as_deref() == Some("living_rebel_count")
            && a.targets.as_ref().is_some_and(|targets| {
                targets.min == 1 && targets.max == 1 && targets.allow_self && targets.has_equip
            }))
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "Dingpan requires `active play_phase uses_per_turn living_rebel_count targets 1 1 has_equip`".into(),
        });
    }
    if a.targets.as_ref().is_some_and(|targets| {
        targets.pair_hand_difference_lte_three && (targets.min != 2 || targets.max != 2)
    }) {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "pair_hand_difference_lte_three requires `targets 2 2`".into(),
        });
    }
    if stmts_use_effect(&a.body, "resolve_potdimeng")
        && !(a.window == "play_phase"
            && a.uses_per_turn == Some(1)
            && a.targets.as_ref().is_some_and(|targets| {
                targets.min == 2 && targets.max == 2 && targets.pair_hand_difference_lte_three
            }))
    {
        return Err(CompileError::Type {
            pos: a.span.start as usize,
            message: "potdimeng requires `active play_phase uses_per_turn 1 targets 2 2 pair_hand_difference_lte_three`".into(),
        });
    }
    let mut locals = Vec::new();
    let body = check_stmts(&a.body, &mut locals)?;
    Ok(ActiveIr {
        window: a.window.clone(),
        targets: a.targets.as_ref().map(|targets| ActiveTargetsIr {
            min: targets.min,
            max: targets.max,
            allow_self: targets.allow_self,
            lowest_hp: targets.lowest_hp,
            highest_hp: targets.highest_hp,
            unlinked: targets.unlinked,
            linked: targets.linked,
            max_by_owner_mark: targets.max_by_owner_mark,
            max_by_owner_lost_hp: targets.max_by_owner_lost_hp,
            max_by_owner_hp: targets.max_by_owner_hp,
            wounded: targets.wounded,
            male: targets.male,
            hp_at_least_owner: targets.hp_at_least_owner,
            hp_gt_owner: targets.hp_gt_owner,
            hp_lte_owner: targets.hp_lte_owner,
            hp_gt: targets.hp_gt,
            hand_gt_owner: targets.hand_gt_owner,
            hand_lte_owner: targets.hand_lte_owner,
            pair_hand_difference_lte_three: targets.pair_hand_difference_lte_three,
            hand_lt: targets.hand_lt,
            hand_nonempty: targets.hand_nonempty,
            can_compare: targets.can_compare,
            can_use_on_owner: targets.can_use_on_owner.clone(),
            second_can_use_juedou_on_first: targets.second_can_use_juedou_on_first,
            equipment_pair_with_armor: targets.equipment_pair_with_armor,
            has_equip: targets.has_equip,
            has_discardable_equip: targets.has_discardable_equip,
            has_gainable_equip: targets.has_gainable_equip,
            has_gainable_hand_equip: targets.has_gainable_hand_equip,
            in_attack_range: targets.in_attack_range,
            owner_in_target_attack_range: targets.owner_in_target_attack_range,
            hand_equip_nonempty: targets.hand_equip_nonempty,
            has_discardable_hand_equip: targets.has_discardable_hand_equip,
            unused_by_skill_this_turn: targets.unused_by_skill_this_turn,
            unused_by_skill_this_play_phase: targets.unused_by_skill_this_play_phase,
            match_selected_card_count: targets.match_selected_card_count,
            selected_cards_match_target_equip_count: targets
                .selected_cards_match_target_equip_count,
        }),
        uses_per_turn: a.uses_per_turn,
        uses_per_turn_source: a.uses_per_turn_source.clone(),
        uses_per_play_phase: a.uses_per_play_phase,
        mark_at_least: a.mark_at_least,
        owner_missing_skill: a.owner_missing_skill,
        hand_gt_hp: a.hand_gt_hp,
        hand_gt_max_hp: a.hand_gt_max_hp,
        hand_limit_positive: a.hand_limit_positive,
        equip_nonempty: a.equip_nonempty,
        hand_nonempty: a.hand_nonempty,
        hand_missing_gaintag: a.hand_missing_gaintag,
        face_up: a.face_up,
        owner_unlinked: a.owner_unlinked,
        once_per_game: a.once_per_game,
        selection: a.selection.as_ref().map(|selection| ActiveSelectionIr {
            zone: selection.zone.clone(),
            min: selection.min,
            max: selection.max,
            allow_cancel: selection.allow_cancel,
            same_suit: selection.same_suit,
            same_color: selection.same_color,
            distinct_categories: selection.distinct_categories,
            distinct_suits: selection.distinct_suits,
            filter: selection.filter.clone(),
            unused_category_by_skill_this_turn: selection.unused_category_by_skill_this_turn,
            unused_category_by_skill_this_play_phase: selection
                .unused_category_by_skill_this_play_phase,
            unused_category_by_owner_history_this_turn: selection
                .unused_category_by_owner_history_this_turn,
            sum_effective_number_exact: selection.sum_effective_number_exact,
        }),
        body,
        span: a.span,
    })
}

fn check_modifier(m: &ModifierAst) -> Result<ModifierIr, CompileError> {
    let judgment_cost_mark = m
        .target
        .strip_prefix("judgment_rewrite_hand_equip_cost_mark_")
        .and_then(|mark| mark.parse::<u32>().ok())
        .is_some_and(|mark| mark > 0);
    let target_mark_exemption = (m
        .target
        .strip_prefix("card_distance_unlimited_target_mark_")
        .or_else(|| m.target.strip_prefix("sha_use_unlimited_target_mark_"))
        .or_else(|| m.target.strip_prefix("card_distance_unlimited_skill_on_owner_"))
        .or_else(|| m.target.strip_prefix("card_use_unrespondable_skill_on_owner_")))
    .and_then(|value| value.parse::<u32>().ok())
    .is_some_and(|value| value > 0);
    if !MOD_TARGETS.contains(&m.target.as_str()) && !judgment_cost_mark && !target_mark_exemption {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: format!("unknown modifier target {}", m.target),
        });
    }
    if target_mark_exemption && (m.mode != "override" || m.value_source.is_some() || m.value != 1) {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: "target-mark exemptions require `override 1`".into(),
        });
    }
    if judgment_cost_mark && (m.mode != "override" || m.value_source.is_some() || m.value != 1) {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: "judgment rewrite mark cost requires `override 1`".into(),
        });
    }
    if !MOD_MODES.contains(&m.mode.as_str()) {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: format!("unknown modifier mode {}", m.mode),
        });
    }
    if matches!(
        m.target.as_str(),
        "distance_from_if_hp_gte_target" | "distance_from_skill_bound_target"
    ) && (m.mode != "override" || m.value_source.is_some() || m.value != 1)
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: format!("{} requires `override 1`", m.target),
        });
    }
    if matches!(
        m.target.as_str(),
        "ignore_armor_for_skill_card_policy"
            | "card_use_unrespondable_for_skill_card_policy"
            | "card_distance_unlimited_for_skill_card_policy"
            | "sha_distance_unlimited_if_rank_lt_skill_mark"
            | "sha_no_use_count_if_rank_gt_skill_mark"
    ) && (m.mode != "override" || m.value_source.is_some() || m.value != 1)
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: format!("{} requires `override 1`", m.target),
        });
    }
    if matches!(
        m.target.as_str(),
        "damage_as_hp_loss" | "damage_between_owner_and_other_as_hp_loss"
    ) && (m.mode != "override" || m.value_source.is_some() || m.value != 1)
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: format!("{} requires `override 1`", m.target),
        });
    }
    if m.target == "damage_as_hp_loss"
        && m.selector
            .as_deref()
            .is_some_and(|selector| selector != "sha")
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: "damage_as_hp_loss accepts only the sha selector".into(),
        });
    }
    if m.target == "card_discard_protected"
        && (m.mode != "override"
            || !((m.value_source.is_none() && m.value == 1)
                || m.value_source.as_deref() == Some("owner_equip_count_exactly_one")))
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: "card_discard_protected requires `override 1` or `override owner_equip_count_exactly_one`".into(),
        });
    }
    if m.target == "card_gain_protected"
        && (m.mode != "override" || m.value_source.is_some() || m.value != 1)
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: "card_gain_protected requires `override 1`".into(),
        });
    }
    if matches!(
        m.target.as_str(),
        "armor_link_gain_cancel" | "link_loss_cancel"
    ) && (m.mode != "override" || m.value_source.is_some() || m.value != 1)
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: format!("{} requires `override 1`", m.target),
        });
    }
    if matches!(
        m.target.as_str(),
        "restore_skipped_phase" | "turn_face_down_cancel" | "turn_over_cancel"
    ) && (m.mode != "override" || m.value_source.is_some() || m.value != 1)
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: format!("{} requires `override 1`", m.target),
        });
    }
    if m.target == "discard_phase_includes_equip"
        && (m.mode != "override"
            || !((m.value_source.is_none() && m.value == 1)
                || m.value_source.as_deref() == Some("queried_player_linked_other_than_owner")))
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: "discard_phase_includes_equip requires `override 1` or `override queried_player_linked_other_than_owner`".into(),
        });
    }
    if m.target == "optional_draw_count" && m.mode != "add" {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: "optional_draw_count requires `add`".into(),
        });
    }
    if m.target == "card_target_limit" && m.selector.is_none() {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: "card_target_limit requires a card selector".into(),
        });
    }
    if m.target == "card_target_limit" && (m.mode != "add" || m.value_source.is_some()) {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: "card_target_limit(selector) requires a constant `add` modifier".into(),
        });
    }
    if matches!(
        m.target.as_str(),
        "observer_foreign_hand_faces_visible" | "observer_hidden_identities_visible"
    ) && (m.mode != "override" || m.value_source.is_some() || m.value != 1)
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: format!("{} requires `override 1`", m.target),
        });
    }
    const CARD_SELECTORS: &[&str] = &[
        "any",
        "sha",
        "tao",
        "bingliang",
        "lebu",
        "basic",
        "non_basic",
        "trick",
        "equip",
        "spade",
        "heart",
        "club",
        "diamond",
        "red",
        "black",
        "juedou",
        "black_trick",
        "damage_trick",
        "sha_fire",
        "huogong",
        "jiu",
        "physical_jiu",
        "sha_red",
        "nanman",
        "ordinary_trick",
        "sha_black",
        "sha_elemental",
        "equip_non_mount",
        "armor_or_treasure",
        "wanjian",
        "sha_normal",
        "sha_heart",
        "delayed_trick",
        "wuxie",
        "guohe",
        "shunshou",
        "sha_thunder",
        "sha_diamond",
        "hand_rank_letter",
        "sole_hand_card",
        "skill_tracked",
        "damage_card",
        "face_up",
    ];
    if let Some(selector) = m.selector.as_deref() {
        if matches!(
            m.target.as_str(),
            "damage_amount"
                | "damage_taken_cancel"
                | "damage_taken_amount"
                | "global_damage_taken_amount"
                | "armor_damage_taken_amount"
                | "damage_taken_final_amount"
                | "global_damage_taken_final_amount"
        ) {
            let allowed = if m.target == "armor_damage_taken_amount" {
                matches!(selector, "normal" | "fire" | "thunder")
            } else if m.target == "damage_amount" {
                matches!(selector, "any" | "normal" | "fire" | "thunder" | "sha")
            } else {
                matches!(selector, "any" | "normal" | "fire" | "thunder")
            };
            if !allowed {
                return Err(CompileError::Type {
                    pos: m.span.start as usize,
                    message: format!("unknown damage nature selector {selector}"),
                });
            }
        } else if !matches!(
            m.target.as_str(),
            "damage_as_hp_loss"
                | "card_ignored_hand_limit"
                | "card_distance_unlimited"
                | "card_discard_protected"
                | "card_gain_protected"
                | "card_distance_unlimited_if_target_hand_gte_owner"
                | "card_use_unlimited"
                | "card_target_limit"
                | "card_distance_limit"
                | "card_targets_all"
                | "card_target_cancel"
                | "armor_card_target_cancel"
                | "card_target_cancel_from_current"
                | "card_target_cancel_owner_turn_banned"
                | "card_target_cancel_if_other_user_hand_gte_target_and_armor_empty"
                | "card_target_cancel_if_other_user_distance_gt_one"
                | "card_use_cancel"
                | "card_response_cancel"
                | "card_save_cancel"
                | "card_suit"
                | "card_name"
                | "card_nature"
                | "sha_unrespondable"
                | "card_use_unrespondable"
                | "sha_damage"
        ) {
            return Err(CompileError::Type {
                pos: m.span.start as usize,
                message: format!(
                    "modifier target {} does not accept a card selector",
                    m.target
                ),
            });
        }
        if !matches!(
            m.target.as_str(),
            "damage_amount"
                | "damage_taken_cancel"
                | "damage_taken_amount"
                | "global_damage_taken_amount"
                | "armor_damage_taken_amount"
                | "damage_taken_final_amount"
                | "global_damage_taken_final_amount"
        ) && !CARD_SELECTORS.contains(&selector)
        {
            return Err(CompileError::Type {
                pos: m.span.start as usize,
                message: format!("unknown card selector {selector}"),
            });
        }
        if m.target == "card_save_cancel" && !matches!(selector, "tao" | "jiu") {
            return Err(CompileError::Type {
                pos: m.span.start as usize,
                message: format!("{} currently accepts only tao or jiu selectors", m.target),
            });
        }
        if m.target == "card_suit" && !matches!(selector, "spade" | "heart" | "club" | "diamond") {
            return Err(CompileError::Type {
                pos: m.span.start as usize,
                message: "card_suit requires a suit selector".into(),
            });
        }
    } else if matches!(
        m.target.as_str(),
        "card_ignored_hand_limit"
            | "card_distance_unlimited_if_target_hand_gte_owner"
            | "card_discard_protected"
            | "card_gain_protected"
            | "card_distance_limit"
            | "card_targets_all"
            | "card_target_cancel"
            | "armor_card_target_cancel"
            | "armor_damage_taken_amount"
            | "card_target_cancel_from_current"
            | "card_target_cancel_if_other_user_hand_gte_target_and_armor_empty"
            | "card_suit"
            | "card_name"
            | "card_nature"
            | "sha_unrespondable"
            | "card_use_unrespondable"
    ) {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: format!("{} requires a card selector", m.target),
        });
    }
    if m.target == "card_distance_limit"
        && (m.mode != "override" || m.value_source.is_some() || m.value < 0)
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: "card_distance_limit requires `override` with a non-negative integer".into(),
        });
    }
    if matches!(
        m.target.as_str(),
        "sha_unrespondable_within_distance" | "sha_use_unrespondable_within_distance"
    ) && (m.mode != "override" || m.value_source.is_some() || m.value <= 0)
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: format!("{} requires `override` with a positive distance", m.target),
        });
    }
    if m.target == "damage_taken_source_range_threshold"
        && (m.mode != "override" || m.value_source.is_some() || m.value <= 0)
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: "damage_taken_source_range_threshold requires `override` with a positive attack range"
                .into(),
        });
    }
    if m.target == "sha_damage_source_range_threshold"
        && (m.mode != "override" || m.value_source.is_some() || m.value <= 0)
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message:
                "sha_damage_source_range_threshold requires `override` with a positive attack range"
                    .into(),
        });
    }
    if m.target == "damage_taken_nature"
        && (m.mode != "override" || m.value_source.is_some() || !(0..=2).contains(&m.value))
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message:
                "damage_taken_nature requires `override` with 0 (normal), 1 (fire), or 2 (thunder)"
                    .into(),
        });
    }
    if m.target == "card_suit"
        && (m.mode != "override"
            || !matches!(
                m.value_source.as_deref(),
                Some("spade" | "heart" | "club" | "diamond")
            ))
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: "card_suit requires `override` with a suit value".into(),
        });
    }
    if m.target == "card_number"
        && (m.mode != "override"
            || m.value_source.is_some()
            || !(1..=13).contains(&m.value)
            || m.selector.is_some())
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: "card_number requires `override` with an integer from 1 to 13".into(),
        });
    }
    if m.target == "card_name"
        && (m.mode != "override"
            || m.value_source
                .as_deref()
                .and_then(card_kind_value)
                .is_none())
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: "card_name requires `override` with a supported card name".into(),
        });
    }
    if m.target == "card_nature"
        && (m.mode != "override"
            || !matches!(
                m.value_source.as_deref(),
                Some("normal" | "fire" | "thunder" | "kami")
            ))
    {
        return Err(CompileError::Type {
            pos: m.span.start as usize,
            message: "card_nature requires `override` with normal, fire, thunder, or kami".into(),
        });
    }
    if let Some(source) = m
        .value_source
        .as_deref()
        .filter(|_| !matches!(m.target.as_str(), "card_suit" | "card_name" | "card_nature"))
    {
        let alive_wounded_threshold = source
            .strip_prefix("alive_wounded_count_at_least_")
            .and_then(|threshold| threshold.parse::<u32>().ok())
            .is_some_and(|threshold| threshold > 0);
        let owner_equip_threshold = source
            .strip_prefix("owner_equip_count_at_least_")
            .and_then(|threshold| threshold.parse::<u32>().ok())
            .is_some_and(|threshold| threshold > 0)
            || source
                .strip_prefix("owner_equip_count_floor_")
                .and_then(|threshold| threshold.parse::<u32>().ok())
                .is_some_and(|threshold| threshold > 0);
        let queried_player_mark = source
            .strip_prefix("queried_player_mark_positive_")
            .and_then(|mark| mark.parse::<u32>().ok())
            .is_some();
        let owner_skill_play_phase_fire_limit = source
            .strip_prefix("owner_skill_play_phase_fire_count_below_")
            .and_then(|limit| limit.parse::<u32>().ok())
            .is_some_and(|limit| limit > 0);
        let owner_skill_pile_negative = source
            .strip_prefix("owner_skill_pile_count_negative_")
            .and_then(|skill_def| skill_def.parse::<u32>().ok())
            .is_some_and(|skill_def| skill_def > 0);
        let owner_skill_pile = owner_skill_pile_negative
            || source
                .strip_prefix("owner_skill_pile_count_")
                .and_then(|skill_def| skill_def.parse::<u32>().ok())
                .is_some_and(|skill_def| skill_def > 0);
        if !source
            .strip_prefix("owner_mark_")
            .is_some_and(|mark| mark.parse::<u32>().is_ok())
            && !alive_wounded_threshold
            && !owner_equip_threshold
            && !queried_player_mark
            && !owner_skill_play_phase_fire_limit
            && !owner_skill_pile
            && !matches!(
                source,
                "owner_hp"
                    | "owner_max_hp"
                    | "owner_hp_minus_one"
                    | "owner_lost_hp"
                    | "owner_lost_hp_minus_one"
                    | "owner_lost_hp_min_one"
                    | "owner_skill_mark"
                    | "owner_skill_pile_count"
                    | "owner_skill_state_u32"
                    | "owner_skill_state_u32_negative"
                    | "queried_player_qun_owner_lord_alive"
                    | "owner_linked"
                    | "owner_skill_use_count"
                    | "alive_wounded_or_guiming_wu_count_min_one"
                    | "alive_group_count"
                    | "alive_female_count"
                    | "alive_other_count_capped_four"
                    | "owner_no_mount"
                    | "owner_hand_empty"
                    | "owner_has_treasure"
                    | "alive_mounted_player_count_min_one"
                    | "owner_equip_count_odd"
                    | "owner_equip_count_even"
                    | "owner_equip_count_half_floor"
                    | "owner_equip_count_exactly_one"
                    | "owner_equip_count"
                    | "owner_hp_gt_two"
                    | "owner_hp_le_two"
                    | "owner_lost_hp_at_least_two"
                    | "owner_all_others_in_attack_range"
                    | "owner_current_turn_all_others_distance_at_most_one"
                    | "owner_sha_used_zero"
                    | "owner_sha_used_nonzero"
                    | "owner_sha_used_exactly_one"
                    | "owner_sha_used_exactly_two"
                    | "owner_max_hand_before_skill_fires_this_turn"
                    | "owner_non_damage_hand_type_count"
                    | "owner_sha_hand_count"
                    | "owner_turn_damage_dealt"
                    | "owner_distinct_hand_card_suits_used_this_turn"
                    | "owner_current_turn_card_use_count"
                    | "owner_play_phase_card_use_count_zero"
                    | "owner_has_heart_equip"
                    | "alive_other_qun_count"
                    | "owner_has_weapon"
                    | "owner_no_weapon"
                    | "owner_has_armor_equip"
                    | "round_number_capped_five"
                    | "round_at_least_alive_qun_count"
                    | "round_below_alive_qun_count"
                    | "alive_field_rank_thirteen_player_count"
                    | "field_mount_card_count_ceil_half"
                    | "owner_unique_max_hand"
                    | "owner_hand_gt_other"
                    | "owner_hand_lt_other"
                    | "owner_qun_other_shu"
                    | "owner_turned_over_diff_other"
                    | "owner_linked_diff_other"
                    | "owner_hand_equip_gte_other"
                    | "owner_sha_damage_dealt_zero_this_turn"
                    | "queried_player_hand_gt_hp_signed"
                    | "queried_player_judge_nonempty_other_than_owner"
                    | "queried_player_linked_other_than_owner"
                    | "queried_player_linked_first_living_skill_owner"
                    | "queried_player_other_than_owner"
                    | "queried_player_wounded_in_owner_attack_range_other"
                    | "queried_source_enemy_owner_ally_target_other_than_owner"
                    | "owner_armor_empty"
                    | "owner_healthy"
                    | "alive_wounded_count_zero_or_at_least_four"
            )
        {
            return Err(CompileError::Type {
                pos: m.span.start as usize,
                message: format!("unknown modifier value source {source}"),
            });
        }
        let valid_use = if owner_skill_play_phase_fire_limit {
            m.mode == "override" && m.target == "card_distance_unlimited"
        } else if queried_player_mark {
            m.mode == "add"
                && matches!(
                    m.target.as_str(),
                    "global_draw_count" | "global_hand_limit" | "global_sha_use_limit"
                )
        } else if alive_wounded_threshold {
            m.mode == "max"
                && matches!(
                    m.target.as_str(),
                    "draw_phase_scry_four_split_two" | "skip_judge_phase" | "skip_discard_phase"
                )
        } else if source == "alive_wounded_count_zero_or_at_least_four" {
            m.mode == "add" && m.target == "damage_amount"
        } else if matches!(
            source,
            "owner_hand_gt_other"
                | "owner_hand_lt_other"
                | "owner_qun_other_shu"
                | "owner_turned_over_diff_other"
                | "owner_linked_diff_other"
                | "owner_hand_equip_gte_other"
        ) {
            m.mode == "add" && matches!(m.target.as_str(), "damage_amount" | "damage_taken_amount")
        } else if source == "owner_sha_damage_dealt_zero_this_turn" {
            m.mode == "add" && m.target == "damage_amount"
        } else if source == "queried_player_hand_gt_hp_signed" {
            m.mode == "add" && m.target == "global_attack_range"
        } else if source == "queried_player_judge_nonempty_other_than_owner" {
            m.mode == "add" && m.target == "global_damage_taken_amount"
        } else if source == "queried_player_linked_other_than_owner" {
            (m.mode == "add" && m.target == "global_damage_taken_final_amount")
                || (m.mode == "override" && m.target == "discard_phase_includes_equip")
        } else if source == "queried_player_linked_first_living_skill_owner" {
            m.mode == "add" && m.target == "global_hand_limit"
        } else if source == "queried_player_other_than_owner" {
            m.mode == "add" && m.target == "global_hand_limit"
        } else if source == "queried_player_wounded_in_owner_attack_range_other" {
            m.mode == "add" && m.target == "global_hand_limit"
        } else if owner_skill_pile_negative {
            m.mode == "add" && m.target == "distance_from"
        } else if owner_skill_pile {
            m.mode == "add" && m.target == "global_distance_to"
        } else if source == "queried_source_enemy_owner_ally_target_other_than_owner" {
            m.mode == "add" && m.target == "global_distance_to"
        } else if source == "owner_unique_max_hand" {
            m.mode == "add" && matches!(m.target.as_str(), "damage_amount" | "damage_taken_amount")
        } else if source == "queried_player_qun_owner_lord_alive" {
            m.mode == "override" && m.target == "card_distance_unlimited"
        } else if source == "owner_skill_state_u32" {
            (m.mode == "add"
                && matches!(
                    m.target.as_str(),
                    "distance_from"
                        | "attack_range"
                        | "hand_limit"
                        | "sha_use_limit"
                        | "sha_target_limit"
                ))
                || (m.mode == "override" && m.target == "card_use_cancel")
        } else if source == "owner_skill_state_u32_negative" {
            m.mode == "add" && m.target == "distance_from"
        } else if source == "owner_skill_mark" {
            (m.mode == "add"
                && matches!(
                    m.target.as_str(),
                    "distance_from"
                        | "distance_to"
                        | "attack_range"
                        | "hand_limit"
                        | "draw_count"
                        | "optional_draw_count"
                        | "initial_hand_count"
                        | "sha_use_limit"
                        | "damage_amount"
                ))
                || (m.mode == "override"
                    && m.target == "tao_use_save_cancel_for_non_dying_others_during_owner_turn")
        } else if source == "owner_skill_pile_count" {
            m.mode == "add" && matches!(m.target.as_str(), "hand_limit" | "attack_range")
        } else if source == "owner_skill_use_count" {
            m.mode == "add" && m.target == "hand_limit"
        } else if source == "alive_wounded_or_guiming_wu_count_min_one" {
            m.mode == "override" && m.target == "draw_count"
        } else if source == "alive_group_count" {
            (m.mode == "add"
                && matches!(
                    m.target.as_str(),
                    "distance_from"
                        | "distance_to"
                        | "attack_range"
                        | "hand_limit"
                        | "draw_count"
                        | "optional_draw_count"
                        | "initial_hand_count"
                        | "sha_use_limit"
                ))
                || (m.mode == "override" && m.target == "draw_count")
        } else if source == "owner_hand_empty" {
            (m.mode == "override" && m.target == "card_target_cancel")
                || (m.mode == "add"
                    && matches!(
                        m.target.as_str(),
                        "damage_taken_amount" | "sha_damage_taken"
                    ))
        } else if source == "owner_hp_le_two" {
            (m.mode == "override"
                && matches!(
                    m.target.as_str(),
                    "card_use_cancel" | "card_response_cancel"
                ))
                || (m.mode == "add" && m.target == "distance_to")
        } else if source == "owner_armor_empty" {
            (m.mode == "override" && m.target == "card_target_cancel")
                || (m.mode == "add" && m.target == "distance_to")
        } else if matches!(source, "owner_sha_used_zero" | "owner_sha_used_nonzero") {
            m.mode == "override" && m.target == "card_distance_unlimited"
        } else if matches!(
            source,
            "owner_sha_used_exactly_one" | "owner_sha_used_exactly_two"
        ) {
            (m.mode == "add" && m.target == "sha_damage")
                || (m.mode == "override" && m.target == "sha_unrespondable")
        } else if source == "owner_max_hand_before_skill_fires_this_turn" {
            m.mode == "add" && m.target == "damage_amount"
        } else if source == "owner_all_others_in_attack_range" {
            m.mode == "add" && m.target == "sha_use_limit"
        } else if source == "owner_current_turn_all_others_distance_at_most_one" {
            (m.mode == "add" && m.target == "sha_target_limit")
                || (m.mode == "override" && m.target == "ignore_armor")
        } else if matches!(source, "owner_equip_count_odd" | "owner_equip_count_even") {
            (m.mode == "override"
                && matches!(
                    m.target.as_str(),
                    "card_use_unlimited" | "card_distance_unlimited"
                ))
                || (source == "owner_equip_count_odd"
                    && m.mode == "add"
                    && m.target == "optional_draw_count")
        } else if source == "owner_equip_count_exactly_one" {
            (m.mode == "override" && m.target == "card_discard_protected")
                || (m.mode == "add" && matches!(m.target.as_str(), "distance_to" | "distance_from"))
        } else if source == "owner_non_damage_hand_type_count" {
            m.mode == "add" && m.target == "hand_limit"
        } else if source == "owner_sha_hand_count" {
            m.mode == "add" && m.target == "hand_limit"
        } else if source == "owner_turn_damage_dealt" {
            m.mode == "override" && m.target == "hand_limit_base"
        } else if source == "owner_distinct_hand_card_suits_used_this_turn" {
            m.mode == "add" && m.target == "hand_limit"
        } else if source == "owner_current_turn_card_use_count" {
            m.mode == "add" && m.target == "distance_from"
        } else if source == "owner_play_phase_card_use_count_zero" {
            m.mode == "override" && m.target == "card_distance_unlimited"
        } else if source == "owner_has_heart_equip" {
            m.mode == "override" && m.target == "hand_limit_base_max_hp"
        } else if source == "owner_has_weapon" {
            (m.mode == "add" && m.target == "response_count")
                || (m.mode == "add" && m.target == "sha_damage")
        } else {
            m.mode == "add"
                && matches!(
                    m.target.as_str(),
                    "distance_from"
                        | "distance_to"
                        | "attack_range"
                        | "hand_limit"
                        | "draw_count"
                        | "optional_draw_count"
                        | "initial_hand_count"
                        | "sha_use_limit"
                )
        };
        if !valid_use {
            return Err(CompileError::Type {
                pos: m.span.start as usize,
                message: format!(
                    "modifier value source {source} is not valid for {} {}",
                    m.target, m.mode
                ),
            });
        }
    }
    let (value_source, value) = if m.target == "card_suit" {
        let value = match m.value_source.as_deref() {
            Some("spade") => 1,
            Some("heart") => 2,
            Some("club") => 3,
            Some("diamond") => 4,
            _ => unreachable!("card_suit checked above"),
        };
        (None, value)
    } else if m.target == "card_name" {
        (
            None,
            card_kind_value(m.value_source.as_deref().unwrap()).expect("card_name checked above"),
        )
    } else if m.target == "card_nature" {
        let value = match m.value_source.as_deref() {
            Some("normal") => 0,
            Some("fire") => 1,
            Some("thunder") => 2,
            Some("kami") => 3,
            _ => unreachable!("card_nature checked above"),
        };
        (None, value)
    } else {
        (m.value_source.clone(), m.value)
    };
    Ok(ModifierIr {
        target: m.target.clone(),
        selector: m.selector.clone(),
        mode: m.mode.clone(),
        value_source,
        value,
        span: m.span,
    })
}

fn card_kind_value(name: &str) -> Option<i32> {
    Some(match name {
        "sha" => 1,
        "tao" => 2,
        "shan" => 3,
        "jiu" => 4,
        "sha_fire" => 5,
        "sha_thunder" => 6,
        "weapon" => 10,
        "armor" => 11,
        "horse_off" => 12,
        "horse_def" => 13,
        "treasure" => 14,
        "wuzhong" => 20,
        "guohe" => 21,
        "shunshou" => 22,
        "juedou" => 23,
        "nanman" => 24,
        "wanjian" => 25,
        "taoyuan" => 26,
        "wugu" => 27,
        "le" => 30,
        "bingliang" => 31,
        "shandian" => 32,
        "jiedao" => 33,
        "tiesuo" => 34,
        "huogong" => 35,
        "wuxie" => 36,
        "yuanjiao" => 37,
        "tiaojiyanmei" => 38,
        "mantianguohai" => 39,
        "youdishenru" => 40,
        "diaobingqianjiang" => 41,
        "wangmeizhike" => 42,
        _ => return None,
    })
}

fn lifetime_card_reward_group_value(name: &str) -> Option<u8> {
    Some(match name {
        "sha_or_shan" => rule_api::lifetime_card_reward_group::SHA_OR_SHAN,
        "tao_or_jiu" => rule_api::lifetime_card_reward_group::TAO_OR_JIU,
        "juedou_or_wuzhong" => rule_api::lifetime_card_reward_group::JUE_DOU_OR_WU_ZHONG,
        _ => return None,
    })
}

fn check_stmts(stmts: &[StmtAst], locals: &mut Vec<String>) -> Result<Vec<StmtIr>, CompileError> {
    let mut out = Vec::new();
    for s in stmts {
        out.push(check_stmt(s, locals)?);
    }
    Ok(out)
}

fn check_reveal_discard_suit_gain_or_use(
    args: &[ExprAst],
    span: content_schema::Span,
    locals: &[String],
) -> Result<StmtIr, CompileError> {
    if args.len() != 4 {
        return Err(CompileError::Type {
            pos: span.start as usize,
            message: "reveal_discard_suit_gain_or_use(player, mark_def, base_count, max_mark)"
                .into(),
        });
    }
    let bounded_u8 = |arg: &ExprAst, name: &str| {
        u8::try_from(expect_int(arg, span.start as usize)?).map_err(|_| CompileError::Type {
            pos: span.start as usize,
            message: format!("{name} must be 0..255"),
        })
    };
    Ok(StmtIr::EffectRevealDiscardSuitGainOrUse {
        player: check_expr(&args[0], locals)?,
        mark: expect_u32(&args[1], span.start as usize)?,
        base_count: bounded_u8(&args[2], "base_count")?,
        max_mark: bounded_u8(&args[3], "max_mark")?,
        span,
    })
}

fn check_choose_discard_or_fire_damage_skip_phases(
    args: &[ExprAst],
    span: content_schema::Span,
    locals: &[String],
) -> Result<StmtIr, CompileError> {
    if args.len() != 3 {
        return Err(CompileError::Type {
            pos: span.start as usize,
            message: "choose_discard_or_fire_damage_skip_phases(player, max_discard, damage)"
                .into(),
        });
    }
    let positive_u8 = |arg: &ExprAst, name: &str| {
        let value = expect_int(arg, span.start as usize)?;
        u8::try_from(value)
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| CompileError::Type {
                pos: span.start as usize,
                message: format!("{name} must be 1..255"),
            })
    };
    Ok(StmtIr::EffectChooseDiscardOrFireDamageSkipPhases {
        player: check_expr(&args[0], locals)?,
        max_discard: positive_u8(&args[1], "max_discard")?,
        damage: positive_u8(&args[2], "damage")?,
        span,
    })
}

fn check_stmt(s: &StmtAst, locals: &mut Vec<String>) -> Result<StmtIr, CompileError> {
    match s {
        StmtAst::Effect { name, args, span } => check_effect_stmt(name.as_str(), args, span, locals),

        StmtAst::If {
            cond,
            then_body,
            else_body,
            span,
        } => {
            let cond = check_expr(cond, locals)?;
            let then_body = check_stmts(then_body, locals)?;
            let else_body = check_stmts(else_body, locals)?;
            Ok(StmtIr::If {
                cond,
                then_body,
                else_body,
                span: *span,
            })
        }
        StmtAst::Loop { times, body, span } => {
            if *times == 0 {
                return Err(CompileError::Type {
                    pos: span.start as usize,
                    message: "loop count must be > 0".into(),
                });
            }
            Ok(StmtIr::Loop {
                times: *times,
                body: check_stmts(body, locals)?,
                span: *span,
            })
        }
        StmtAst::Let { name, value, span } => {
            let value = check_expr(value, locals)?;
            if !locals.iter().any(|l| l == name) {
                locals.push(name.clone());
            }
            Ok(StmtIr::Let {
                name: name.clone(),
                value,
                span: *span,
            })
        }
    }
}

fn check_effect_stmt(
    name: &str,
    args: &[ExprAst],
    span: &Span,
    locals: &mut Vec<String>,
) -> Result<StmtIr, CompileError> {
    match name {
            "set_draw_phase_bottom" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "set_draw_phase_bottom(player)".into(),
                    });
                }
                Ok(StmtIr::EffectSetDrawPhaseFromBottom {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "draw_cards_from_bottom" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_cards_from_bottom(player, count)".into(),
                    });
                }
                Ok(StmtIr::EffectDrawFromBottom {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_chenglve" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_chenglve(player, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveChenglve {
                    player: check_expr(&args[0], locals)?,
                    stage: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_shicai" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_shicai(player, card, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveShicai {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    stage: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_sbbiyue" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_sbbiyue(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSbbiyue {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_sxrmqiewang" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_sxrmqiewang(player, event_player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSxrmqiewang {
                    player: check_expr(&args[0], locals)?,
                    event_player: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_sxrmqianliu" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_sxrmqianliu(player, source, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSxrmqianliu {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    stage: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_gzbuqu" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_gzbuqu(player, delta, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveGzbuqu {
                    player: check_expr(&args[0], locals)?,
                    delta: check_expr(&args[1], locals)?,
                    stage: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_gzbuqu_recover" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_gzbuqu_recover(player, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveGzbuquRecover {
                    player: check_expr(&args[0], locals)?,
                    stage: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_xiongluan" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xiongluan(player, target, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXiongluan {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    stage: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_twjuxiang" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twjuxiang(player, event_player, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwjuxiang {
                    player: check_expr(&args[0], locals)?,
                    event_player: check_expr(&args[1], locals)?,
                    stage: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "draw_cards" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_cards(player, count)".into(),
                    });
                }
                Ok(StmtIr::EffectDraw {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "recover_hp" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "recover_hp(player, amount)".into(),
                    });
                }
                Ok(StmtIr::EffectRecover {
                    player: check_expr(&args[0], locals)?,
                    amount: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "recover_hp_as" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "recover_hp_as(source, player, amount)".into(),
                    });
                }
                Ok(StmtIr::EffectRecoverAs {
                    source: check_expr(&args[0], locals)?,
                    player: check_expr(&args[1], locals)?,
                    amount: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_duoduan" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_duoduan(player, source, card)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDuoduan {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_qiangyong" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qiangyong(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveQiangyong {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_twxiongjun" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twxiongjun(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwxiongjun {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_dcqiaolve" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcqiaolve(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcqiaolve {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_yixiang" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_yixiang(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveYixiang {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_xinfu_qiai" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xinfu_qiai(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXinfuQiai {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_olwenren" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_olwenren(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveOlwenren {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_spolaige" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spolaige(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpolaige {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_olzhijie" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_olzhijie(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveOlzhijie {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_olsuyi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_olsuyi(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveOlsuyi {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_olqiaozhi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_olqiaozhi(player, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveOlqiaozhi {
                    player: check_expr(&args[0], locals)?,
                    stage: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_dczhee" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dczhee(player, mode)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDczhee {
                    player: check_expr(&args[0], locals)?,
                    mode: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_dchaoxian" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dchaoxian(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDchaoxian {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_zhongjie" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhongjie(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhongjie {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_starcuxia_weiming" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_starcuxia_weiming(player, source, variant)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveStarcuxiaWeiming {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    variant: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_sanshou" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_sanshou(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSanshou {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_nhlundao" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_nhlundao(player, source)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveNhlundao {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_xiangshu" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xiangshu(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXiangshu {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_dingce" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dingce(player, source)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDingce {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_meiyong" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_meiyong(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMeiyong {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_mbzhenann" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbzhenann(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbzhenann {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_friendbiwei" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_friendbiwei(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveFriendbiwei {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_chulao" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_chulao(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveChulao {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_dcanxu" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcanxu(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcanxu {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_gaoyuan" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_gaoyuan(player, source, card)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveGaoyuan {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "choose_discard_redirect_current_sha" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_redirect_current_sha(player, source, card)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardRedirectCurrentSha {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "judge_set_skill_mark_to_rank" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judge_set_skill_mark_to_rank(player, skill)".into(),
                    });
                }
                Ok(StmtIr::EffectJudgeSetSkillMarkToRank {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_discard_one_from_each_linked" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_one_from_each_linked(player, skill)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardOneFromEachLinked {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "disable_skill_by_def_until_turn_end" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "disable_skill_by_def_until_turn_end(player, skill_def)".into(),
                    });
                }
                let skill_def = match &args[1] {
                    ExprAst::Int(value) if *value >= 0 => *value as u32,
                    ExprAst::U32(value) => *value,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "skill_def must be a nonnegative stable id".into(),
                        });
                    }
                };
                Ok(StmtIr::EffectDisableSkillByDefUntilTurnEnd {
                    player: check_expr(&args[0], locals)?,
                    skill_def,
                    span: *span,
                })
            }
            "target_discard_then_duel_if_non_sha_and_hp_gte_owner" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "target_discard_then_duel_if_non_sha_and_hp_gte_owner(player, target)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectTargetDiscardThenDuelIfNonShaAndHpGteOwner {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_target_lose_hp_discard_hand_sha_if_basic" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_target_lose_hp_discard_hand_sha_if_basic(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseTargetLoseHpDiscardHandShaIfBasic {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "reset_linked_selected_targets" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reset_linked_selected_targets(source)".into(),
                    });
                }
                Ok(StmtIr::EffectResetLinkedSelectedTargets {
                    source: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "deal_damage" => {
                if !(3..=4).contains(&args.len()) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "deal_damage(source, target, amount[, normal|fire|thunder])"
                            .into(),
                    });
                }
                let nature = match args.get(3) {
                    None => 0,
                    Some(ExprAst::Ident(name)) if name == "normal" => 0,
                    Some(ExprAst::Ident(name)) if name == "fire" => 1,
                    Some(ExprAst::Ident(name)) if name == "thunder" => 2,
                    Some(_) => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "damage nature must be normal, fire, or thunder".into(),
                        });
                    }
                };
                Ok(StmtIr::EffectDamage {
                    source: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    amount: check_expr(&args[2], locals)?,
                    nature,
                    span: *span,
                })
            }
            "deal_damage_ban_active_if_dying" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "deal_damage_ban_active_if_dying(source, target, amount, self_skill)"
                                .into(),
                    });
                }
                if !matches!(&args[3], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "damage-linked active ban must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectDealDamageBanActiveIfDying {
                    source: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    amount: check_expr(&args[2], locals)?,
                    skill: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "spjungong_discard_or_lose_then_sha" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "spjungong_discard_or_lose_then_sha(player, target, self_skill)"
                            .into(),
                    });
                }
                if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "峻攻 must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectSpjungongDiscardOrLoseThenSha {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "ban_active_until_turn_end" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "ban_active_until_turn_end(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "turn-end active ban must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectBanActiveUntilTurnEnd {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "spwanwei_active" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "spwanwei_active(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectSpwanweiActive {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "spwanwei_dying" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "spwanwei_dying(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectSpwanweiDying {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_spxizhan" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spxizhan(player, phase_player, fangzong_skill_def)"
                            .into(),
                    });
                }
                let fangzong = expect_u32(&args[2], span.start as usize)?;
                if fangzong == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spxizhan fangzong_skill_def must be nonzero".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpxizhan {
                    player: check_expr(&args[0], locals)?,
                    phase_player: check_expr(&args[1], locals)?,
                    fangzong_skill_def: fangzong,
                    span: *span,
                })
            }
            "resolve_spsongshu" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spsongshu(player, phase_player, renku_skill_def, block_skill_def)"
                            .into(),
                    });
                }
                let renku = expect_u32(&args[2], span.start as usize)?;
                let block = expect_u32(&args[3], span.start as usize)?;
                if renku == 0 || block == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spsongshu skill defs must be nonzero".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpsongshu {
                    player: check_expr(&args[0], locals)?,
                    phase_player: check_expr(&args[1], locals)?,
                    renku_skill_def: renku,
                    block_skill_def: block,
                    span: *span,
                })
            }
            "resolve_spcunsi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spcunsi(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpcunsi {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_yuli" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_yuli(player, mode)".into(),
                    });
                }
                let mode = expect_u32(&args[1], span.start as usize)?;
                if mode > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_yuli mode must be 0 (source) or 1 (target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveYuli {
                    player: check_expr(&args[0], locals)?,
                    mode: mode as u8,
                    span: *span,
                })
            }
            "resolve_tingwei" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_tingwei(player, mode)".into(),
                    });
                }
                let mode = expect_u32(&args[1], span.start as usize)?;
                if mode > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_tingwei mode must be 0 (select) or 1 (resolve)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTingwei {
                    player: check_expr(&args[0], locals)?,
                    mode: mode as u8,
                    span: *span,
                })
            }
            "resolve_xinmingshi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xinmingshi(player, source)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXinmingshi {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_xunyi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xunyi(player, mode)".into(),
                    });
                }
                let mode = expect_u32(&args[1], span.start as usize)?;
                if mode > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xunyi mode must be 0 (select) or 1 (damage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXunyi {
                    player: check_expr(&args[0], locals)?,
                    mode: mode as u8,
                    span: *span,
                })
            }
            "resolve_boming" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_boming(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveBoming {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_ejian" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ejian(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveEjian {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_yiyong" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_yiyong(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveYiyong {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_shanxie" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_shanxie(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveShanxie {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_sheyi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_sheyi(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSheyi {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_tianyin" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_tianyin(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTianyin {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_dbzhuifeng" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dbzhuifeng(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDbzhuifeng {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_dbquedi" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dbquedi(player, target, card)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDbquedi {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_spyinju" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spyinju(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpyinju {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_spchijie" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spchijie(player, source, card)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpchijie {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_spshangyi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spshangyi(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpshangyi {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_spyajun" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spyajun(player, less_def, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpyajun {
                    player: check_expr(&args[0], locals)?,
                    less_def: expect_u32(&args[1], span.start as usize)?,
                    stage: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_spzundi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spzundi(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpzundi {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_xinyangjie" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xinyangjie(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXinyangjie {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_qingyu" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qingyu(player, action, xuancun_def, mbyongjue_def)"
                            .into(),
                    });
                }
                let action = expect_u32(&args[1], span.start as usize)?;
                if action > 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qingyu action must be 0 through 3".into(),
                    });
                }
                Ok(StmtIr::EffectResolveQingyu {
                    player: check_expr(&args[0], locals)?,
                    action: action as u8,
                    xuancun_def: expect_u32(&args[2], span.start as usize)?,
                    mbyongjue_def: expect_u32(&args[3], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_zhengsu" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhengsu(player, mode, stage, card)".into(),
                    });
                }
                let mode = expect_u32(&args[1], span.start as usize)?;
                if mode > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhengsu mode must be 0 through 2".into(),
                    });
                }
                let stage = expect_u32(&args[2], span.start as usize)?;
                if stage > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhengsu stage must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhengsu {
                    player: check_expr(&args[0], locals)?,
                    mode: mode as u8,
                    stage: stage as u8,
                    card: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "resolve_mbdiancai" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbdiancai(player, actor)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbdiancai {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_xiongshu" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xiongshu(player, target, card, action, stage)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xiongshu action must be 0 or 1".into(),
                    });
                }
                let stage = expect_u32(&args[4], span.start as usize)?;
                if stage > 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xiongshu stage must be 0 through 3".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXiongshu {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: action as u8,
                    stage: stage as u8,
                    span: *span,
                })
            }
            "resolve_jianhui" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jianhui(player, source, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveJianhui {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    target: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_spqiai" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spqiai(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpqiai {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_spshanxi" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spshanxi(player, target, stage)".into(),
                    });
                }
                let stage = expect_u32(&args[2], span.start as usize)?;
                if stage > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spshanxi stage must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpshanxi {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    stage: stage as u8,
                    span: *span,
                })
            }
            "resolve_tongduo" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_tongduo(player, target, card, stage)".into(),
                    });
                }
                let stage = expect_u32(&args[3], span.start as usize)?;
                if stage > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_tongduo stage must be 0 through 2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTongduo {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    stage: stage as u8,
                    span: *span,
                })
            }
            "resolve_yuanqing" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_yuanqing(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveYuanqing {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_shuchen" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_shuchen(player, dying, renku_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveShuchen {
                    player: check_expr(&args[0], locals)?,
                    dying: check_expr(&args[1], locals)?,
                    renku_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_duanbi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_duanbi(player, stage)".into(),
                    });
                }
                let stage = expect_u32(&args[1], span.start as usize)?;
                if stage > 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_duanbi stage must be 0 through 3".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDuanbi {
                    player: check_expr(&args[0], locals)?,
                    stage: stage as u8,
                    span: *span,
                })
            }
            "resolve_zaoli" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zaoli(player, mark)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZaoli {
                    player: check_expr(&args[0], locals)?,
                    mark: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_guying" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_guying(player, actor, card, stage)".into(),
                    });
                }
                let stage = expect_u32(&args[3], span.start as usize)?;
                if stage > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_guying stage must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveGuying {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    stage: stage as u8,
                    span: *span,
                })
            }
            "resolve_spshanxi_suoming" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spshanxi_suoming(player, actor, stage)".into(),
                    });
                }
                let stage = expect_u32(&args[2], span.start as usize)?;
                if stage > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spshanxi_suoming stage must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpshanxiSuoming {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    stage: stage as u8,
                    span: *span,
                })
            }
            "resolve_rechuhai" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_rechuhai(player, target, card, target_card, stage)"
                            .into(),
                    });
                }
                let stage = expect_u32(&args[4], span.start as usize)?;
                if stage > 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_rechuhai stage must be 0 through 3".into(),
                    });
                }
                Ok(StmtIr::EffectResolveRechuhai {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    target_card: check_expr(&args[3], locals)?,
                    stage: stage as u8,
                    span: *span,
                })
            }
            "resolve_houfeng" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_houfeng(player, target, mode, stage)".into(),
                    });
                }
                let mode = expect_u32(&args[2], span.start as usize)?;
                if mode > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_houfeng mode must be 0 or 1".into(),
                    });
                }
                let stage = expect_u32(&args[3], span.start as usize)?;
                if stage > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_houfeng stage must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveHoufeng {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    mode: mode as u8,
                    stage: stage as u8,
                    span: *span,
                })
            }
            "resolve_spjianyi" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spjianyi(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpjianyi {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_fyjianyu" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_fyjianyu(player, a, b)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveFyjianyu {
                    player: check_expr(&args[0], locals)?,
                    a: check_expr(&args[1], locals)?,
                    b: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_liubing" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_liubing(player, card)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveLiubing {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_heji" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_heji(player, target, card)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveHeji {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_twjiaohua" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twjiaohua(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwjiaohua {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_twkujian" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twkujian(player, target, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twkujian action must be 0, 1, or 2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwkujian {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twruilian" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twruilian(player, target, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twruilian action must be 0, 1, or 2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwruilian {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "equip_weapon_from_draw_pile" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "equip_weapon_from_draw_pile(player)".into(),
                    });
                }
                Ok(StmtIr::EffectEquipWeaponFromDrawPile {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_twjianwei" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twjianwei(player, actor, stage)".into(),
                    });
                }
                let stage = expect_u32(&args[2], span.start as usize)?;
                if stage > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twjianwei stage must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwjianwei {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    stage: stage as u8,
                    span: *span,
                })
            }
            "resolve_twtanfeng" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twtanfeng(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwtanfeng {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_twzhibian" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twzhibian(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwzhibian {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_twyuyan" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twyuyan(player, source, card)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwyuyan {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_twguoyi" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twguoyi(player, target, card, hand_lock_skill_def)"
                            .into(),
                    });
                }
                let hand_lock_skill_def = expect_u32(&args[3], span.start as usize)?;
                Ok(StmtIr::EffectResolveTwguoyi {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    hand_lock_skill_def,
                    span: *span,
                })
            }
            "resolve_twchuhai" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twchuhai(player, action)".into(),
                    });
                }
                let action = expect_u32(&args[1], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twchuhai action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwchuhai {
                    player: check_expr(&args[0], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twyizhu" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twyizhu(player, source, card, action)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twyizhu action must be 0, 1, or 2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwyizhu {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twluanchou" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twluanchou(player, helper_skill_def, action)".into(),
                    });
                }
                let helper_skill_def = expect_u32(&args[1], span.start as usize)?;
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twluanchou action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwluanchou {
                    player: check_expr(&args[0], locals)?,
                    helper_skill_def,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twxiawei" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twxiawei(player, wangxing_skill_def, action)".into(),
                    });
                }
                let wangxing_skill_def = expect_u32(&args[1], span.start as usize)?;
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twxiawei action must be 0 through 4".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwxiawei {
                    player: check_expr(&args[0], locals)?,
                    wangxing_skill_def,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twbianfuren" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twbianfuren(player, target, helper_skill_def, action)"
                            .into(),
                    });
                }
                let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twbianfuren action must be 0 through 2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwbianfuren {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    helper_skill_def,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twfeiyi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twfeiyi(player, action)".into(),
                    });
                }
                let action = expect_u32(&args[1], span.start as usize)?;
                if action > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twfeiyi action must be 0 through 2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwfeiyi {
                    player: check_expr(&args[0], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twchenzhen" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twchenzhen(player, target, helper_skill_def, record_skill_def, action)".into(),
                    });
                }
                let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
                let record_skill_def = expect_u32(&args[3], span.start as usize)?;
                let action = expect_u32(&args[4], span.start as usize)?;
                if action > 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twchenzhen action must be 0 through 3".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwchenzhen {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    helper_skill_def,
                    record_skill_def,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twqianju" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twqianju(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwqianju {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_twqingxi" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twqingxi(player, target, card)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwqingxi {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_twzaoli" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twzaoli(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwzaoli {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_twzhengjian" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twzhengjian(player, target, helper_skill_def, action)"
                            .into(),
                    });
                }
                let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twzhengjian action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwzhengjian {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    helper_skill_def,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twtianyu" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twtianyu(player, target, helper_skill_def, action)"
                            .into(),
                    });
                }
                let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twtianyu action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwtianyu {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    helper_skill_def,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twwujing" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twwujing(player, target_count, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twwujing action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwwujing {
                    player: check_expr(&args[0], locals)?,
                    target_count: check_expr(&args[1], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twshanghe" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twshanghe(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwshanghe {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_twwangchang" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twwangchang(player, target, card, action)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twwangchang action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwwangchang {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twcaozhao" => {
                if args.len() != 7 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twcaozhao(player, target, source, card, twfuzuan_def, twfeifu_def, action)".into(),
                    });
                }
                let action = expect_u32(&args[6], span.start as usize)?;
                if action > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twcaozhao action must be 0, 1, or 2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwcaozhao {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    source: check_expr(&args[2], locals)?,
                    card: check_expr(&args[3], locals)?,
                    twfuzuan_skill_def: expect_u32(&args[4], span.start as usize)?,
                    twfeifu_skill_def: expect_u32(&args[5], span.start as usize)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twguohuai" => {
                if args.len() != 7 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twguohuai(player, target, source, card, mark_def, helper_def, action)".into(),
                    });
                }
                let action = expect_u32(&args[6], span.start as usize)?;
                if action > 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twguohuai action must be 0..=4".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwguohuai {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    source: check_expr(&args[2], locals)?,
                    card: check_expr(&args[3], locals)?,
                    mark_def: expect_u32(&args[4], span.start as usize)?,
                    helper_skill_def: expect_u32(&args[5], span.start as usize)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twchengpu" => {
                if args.len() != 6 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_twchengpu(player, target, source, card, chunlao_def, action)"
                                .into(),
                    });
                }
                let action = expect_u32(&args[5], span.start as usize)?;
                if action > 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twchengpu action must be 0..=3".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwchengpu {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    source: check_expr(&args[2], locals)?,
                    card: check_expr(&args[3], locals)?,
                    chunlao_skill_def: expect_u32(&args[4], span.start as usize)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twcaoang" => {
                if args.len() != 6 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_twcaoang(player, target, source, card, helper_def, action)"
                                .into(),
                    });
                }
                let action = expect_u32(&args[5], span.start as usize)?;
                if action > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twcaoang action must be 0, 1, or 2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwcaoang {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    source: check_expr(&args[2], locals)?,
                    card: check_expr(&args[3], locals)?,
                    helper_skill_def: expect_u32(&args[4], span.start as usize)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twniujin" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twniujin(player, limited_skill_def, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twniujin action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwniujin {
                    player: check_expr(&args[0], locals)?,
                    limited_skill_def: expect_u32(&args[1], span.start as usize)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twmoukui" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twmoukui(player, target, card)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwmoukui {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_twfuman" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twfuman(player, target, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twfuman action must be 0, 1, or 2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwfuman {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_cuijin" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_cuijin(player, source, card, action)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_cuijin action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveCuijin {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twqingkou" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twqingkou(player, winner, card, action)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twqingkou action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwqingkou {
                    player: check_expr(&args[0], locals)?,
                    winner: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twchenxun" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twchenxun(player, target, card, action)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twchenxun action must be 0 through 4".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwchenxun {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_twfenxian" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twfenxian(player, target, card, action)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twfenxian action must be 0 through 4".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwfenxian {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "set_skill_content_flag" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "set_skill_content_flag(player, bit, 0|1)".into(),
                    });
                }
                let bit = literal_i32(&args[1]).ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "skill content flag bit must be a literal".into(),
                })?;
                let value = literal_i32(&args[2]).ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: "skill content flag value must be 0 or 1".into(),
                })?;
                if !(0..64).contains(&bit) || !matches!(value, 0 | 1) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "set_skill_content_flag requires bit 0..63 and value 0|1".into(),
                    });
                }
                Ok(StmtIr::EffectSetSkillContentFlag {
                    player: check_expr(&args[0], locals)?,
                    bit: bit as u8,
                    value: value != 0,
                    span: *span,
                })
            }
            "choose_target_reclaim_equipment_draw_if_range_reduced" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_target_reclaim_equipment_draw_if_range_reduced(player, target, self_skill)"
                                .into(),
                    });
                }
                if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "target-equipment reclamation must reference self_skill".into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseTargetReclaimEquipmentDrawIfRangeReduced {
                        player: check_expr(&args[0], locals)?,
                        target: check_expr(&args[1], locals)?,
                        skill: check_expr(&args[2], locals)?,
                        span: *span,
                    },
                )
            }
            "choose_use_virtual_card" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_use_virtual_card(player, wanjian|juedou, self_skill)"
                            .into(),
                    });
                }
                let ExprAst::Ident(kind_name) = &args[1] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_use_virtual_card second argument must be a card kind"
                            .into(),
                    });
                };
                if !matches!(kind_name.as_str(), "wanjian" | "juedou") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_use_virtual_card supports wanjian or juedou".into(),
                    });
                }
                if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "virtual-card use must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectChooseUseVirtualCard {
                    player: check_expr(&args[0], locals)?,
                    kind: card_kind_value(kind_name).expect("validated card kind"),
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "choose_use_virtual_nanman" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_use_virtual_nanman(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "virtual-card use must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectChooseUseVirtualNanman {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_qiluan" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qiluan(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qiluan must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveQiluan {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "discard_all_own_judge" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_all_own_judge(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_all_own_judge must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectDiscardAllOwnJudge {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "store_event_card_in_skill_pile" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "store_event_card_in_skill_pile(player, card, self_skill)"
                            .into(),
                    });
                }
                if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "store_event_card_in_skill_pile must reference self_skill"
                            .into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "store-event-card card must be a card operand".into(),
                    });
                }
                Ok(StmtIr::EffectStoreEventCardInSkillPile {
                    player: check_expr(&args[0], locals)?,
                    card,
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_ollinjie" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ollinjie(player, source)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveOllinjie {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_target_gain_max_hp" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_target_gain_max_hp(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_target_gain_max_hp must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectChooseTargetGainMaxHp {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "discard_basic_draw_equal" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_basic_draw_equal(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_basic_draw_equal must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectDiscardBasicDrawEqual {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "start_debate_all_others" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "start_debate_all_others(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "start_debate_all_others must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectStartDebateAllOthers {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "start_debate_targets" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "start_debate_targets(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "start_debate_targets must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectStartDebateTargets {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "pin_debate_opinions" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "pin_debate_opinions(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "pin_debate_opinions must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectPinDebateOpinions {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "align_debate_opinions" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "align_debate_opinions(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "align_debate_opinions must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectAlignDebateOpinions {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "gain_shadow_cards" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_shadow_cards(player, count, self_skill)".into(),
                    });
                }
                if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_shadow_cards must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectGainShadowCards {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "change_group" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "change_group(player, \"wei|shu|wu|qun|jin\", self_skill)".into(),
                    });
                }
                let ExprAst::Ident(group) = &args[1] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "change_group group must be a faction ident (wei/shu/wu/qun/jin)".into(),
                    });
                };
                if !matches!(group.as_str(), "wei" | "shu" | "wu" | "qun" | "jin") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "change_group unknown faction".into(),
                    });
                }
                if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "change_group must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectChangeGroup {
                    player: check_expr(&args[0], locals)?,
                    group: group.clone(),
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "distribute_shadow_cards" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "distribute_shadow_cards(player, count, self_skill)".into(),
                    });
                }
                if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "distribute_shadow_cards must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectDistributeShadowCards {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_turn_end_debates" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_turn_end_debates(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(n) if n == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_turn_end_debates must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTurnEndDebates {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_group" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_group(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_group must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectChooseGroup {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_gain_other_skill" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_gain_other_skill(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_gain_other_skill must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectChooseGainOtherSkill {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "lose_gained_skill" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "lose_gained_skill(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "lose_gained_skill must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectLoseGainedSkill {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_potheyun" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potheyun(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potheyun must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotheyun {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "place_xumou" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "place_xumou(player, card, self_skill)".into(),
                    });
                }
                if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "place_xumou must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectPlaceXumou {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "place_deck_top_xumou" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "place_deck_top_xumou(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(n) if n == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "place_deck_top_xumou must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectPlaceDeckTopXumou {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_place_hand_xumou" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_place_hand_xumou(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(n) if n == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_place_hand_xumou must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectChoosePlaceHandXumou {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "discard_one_own_xumou" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_one_own_xumou(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(n) if n == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_one_own_xumou must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectDiscardOneOwnXumou {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "place_deck_top_expansion" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "place_deck_top_expansion(player, count, self_skill)".into(),
                    });
                }
                let count = expect_u32(&args[1], span.start as usize)?;
                if !matches!(&args[2], ExprAst::Ident(n) if n == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "place_deck_top_expansion must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectPlaceDeckTopExpansion {
                    player: check_expr(&args[0], locals)?,
                    count,
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }

            "city_consume" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "city_consume(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(n) if n == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "city_consume must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectCityConsume {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "queue_extra_turn" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "queue_extra_turn(player, check, self_skill)".into(),
                    });
                }
                let check = expect_u32(&args[1], span.start as usize)?;
                if check > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "queue_extra_turn check must be 0 or 1".into(),
                    });
                }
                if !matches!(&args[2], ExprAst::Ident(n) if n == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "queue_extra_turn must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectQueueExtraTurn {
                    player: check_expr(&args[0], locals)?,
                    check,
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }            "choose_others_insert_extra_turn" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_others_insert_extra_turn(player, check, self_skill)".into(),
                    });
                }
                let check = expect_u32(&args[1], span.start as usize)?;
                if check > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_others_insert_extra_turn check must be 0 or 1".into(),
                    });
                }
                if !matches!(&args[2], ExprAst::Ident(n) if n == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_others_insert_extra_turn must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectChooseOthersInsertExtraTurn {
                    player: check_expr(&args[0], locals)?,
                    check,
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "queue_extra_phases" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "queue_extra_phases(player, phases_mask, self_skill)".into(),
                    });
                }
                let phases_mask = expect_u32(&args[1], span.start as usize)?;
                if phases_mask == 0 || phases_mask > 0b0111_110 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "queue_extra_phases mask out of range".into(),
                    });
                }
                if !matches!(&args[2], ExprAst::Ident(n) if n == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "queue_extra_phases must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectQueueExtraPhases {
                    player: check_expr(&args[0], locals)?,
                    phases_mask,
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_twjieqiu" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twjieqiu(player, target, subaction, self_skill)".into(),
                    });
                }
                let subaction = expect_u32(&args[2], span.start as usize)?;
                if subaction != 0 && subaction != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twjieqiu subaction must be 0 or 2".into(),
                    });
                }
                if !matches!(&args[3], ExprAst::Ident(n) if n == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twjieqiu must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwjieqiu {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    subaction,
                    skill: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "skip_remaining_phases" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "skip_remaining_phases(player, phases_mask, self_skill)".into(),
                    });
                }
                let phases_mask = expect_u32(&args[1], span.start as usize)?;
                if phases_mask == 0 || phases_mask > 0b0001_1100 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "skip_remaining_phases mask limited to Draw/Play/Discard".into(),
                    });
                }
                if !matches!(&args[2], ExprAst::Ident(n) if n == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "skip_remaining_phases must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectSkipRemainingPhases {
                    player: check_expr(&args[0], locals)?,
                    phases_mask,
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "choose_others_add_mark" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_others_add_mark(player, mark_def, delta, max_count, self_skill)".into(),
                    });
                }
                let mark = expect_u32(&args[1], span.start as usize)?;
                let delta = expect_int(&args[2], span.start as usize)?;
                let max_count = expect_u32(&args[3], span.start as usize)?;
                if !matches!(&args[4], ExprAst::Ident(n) if n == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_others_add_mark must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectChooseOthersAddMark {
                    player: check_expr(&args[0], locals)?,
                    mark,
                    delta,
                    max_count,
                    skill: check_expr(&args[4], locals)?,
                    span: *span,
                })
            }
            "register_turn_end_debate" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "register_turn_end_debate(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(n) if n == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "register_turn_end_debate must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectRegisterTurnEndDebate {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_jsrgchaozheng" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jsrgchaozheng(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jsrgchaozheng must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveJsrgchaozheng {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_jsrgsheju" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jsrgsheju(player, counterpart, self_skill)".into(),
                    });
                }
                if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jsrgsheju must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveJsrgsheju {
                    player: check_expr(&args[0], locals)?,
                    counterpart: check_expr(&args[1], locals)?,
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "choose_other_add_mark" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_add_mark(player, mark_def, delta)".into(),
                    });
                }
                let player = check_expr(&args[0], locals)?;
                let mark = expect_u32(&args[1], span.start as usize)?;
                let delta = expect_int(&args[2], span.start as usize)
                    .map_err(|e| CompileError::Type {
                        pos: span.start as usize,
                        message: format!("choose_other_add_mark delta must be an integer literal: {e}"),
                    })?;
                Ok(StmtIr::EffectChooseOtherAddMark {
                    player,
                    mark,
                    delta,
                    span: *span,
                })
            }
            "gain_whole_skill_pile" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_whole_skill_pile(player, self_skill, pile_def)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_whole_skill_pile must reference self_skill".into(),
                    });
                }
                let pile_def = expect_u32(&args[2], 0)?;
                Ok(StmtIr::EffectGainWholeSkillPile {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    pile_def,
                    span: *span,
                })
            }
            "place_hand_cards_to_deck_top" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "place_hand_cards_to_deck_top(player, self_skill, count)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "place_hand_cards_to_deck_top must reference self_skill".into(),
                    });
                }
                let count = check_expr(&args[2], locals)?;
                Ok(StmtIr::EffectPlaceHandCardsToDeckTop {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    count,
                    span: *span,
                })
            }
            "rearrange_pile_to_deck_top" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "rearrange_pile_to_deck_top(player, self_skill, pile_def, count)"
                            .into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "rearrange_pile_to_deck_top must reference self_skill".into(),
                    });
                }
                let pile_def = expect_u32(&args[2], 0)?;
                let count = check_expr(&args[3], locals)?;
                Ok(StmtIr::EffectRearrangePileToDeckTop {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    pile_def,
                    count,
                    span: *span,
                })
            }
            "resolve_qianlong" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qianlong(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qianlong must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveQianlong {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_jsrgzuozhan" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jsrgzuozhan(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jsrgzuozhan must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveJsrgzuozhan {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_twmouzhi" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twmouzhi(player, self_skill, mode, event_card)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twmouzhi must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwmouzhi {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    mode: check_expr(&args[2], locals)?,
                    event_card: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "resolve_twdaigui" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twdaigui(player, self_skill, stage, count)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twdaigui must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwdaigui {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    stage: check_expr(&args[2], locals)?,
                    count: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "resolve_twmitu" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twmitu(player, self_skill, stage, count)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twmitu must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwmitu {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    stage: check_expr(&args[2], locals)?,
                    count: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "resolve_qiantun" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qiantun(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qiantun must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveQiantun {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_yangming" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_yangming(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_yangming must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveYangming {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_canyun" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_canyun(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_canyun must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveCanyun {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_hengwu" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_hengwu(player, self_skill, event_card)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_hengwu must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveHengwu {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    event_card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_biaozhao3" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_biaozhao3(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_biaozhao3 must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveBiaozhao3 {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_biaozhao2" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_biaozhao2(player, self_skill, event_player)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_biaozhao2 must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveBiaozhao2 {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    event_player: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_dcniji" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcniji(player, self_skill, mode, event_card)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcniji must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcniji {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    mode: check_expr(&args[2], locals)?,
                    event_card: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "resolve_dcmoukui" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcmoukui(player, self_skill, stage, target)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcmoukui must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcmoukui {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    stage: check_expr(&args[2], locals)?,
                    target: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "resolve_zhongyan" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhongyan(player, self_skill, stage)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhongyan must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhongyan {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    stage: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_chumei" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_chumei(player, self_skill, stage)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_chumei must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveChumei {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    stage: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_shuaiyan" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_shuaiyan(player, self_skill, stage)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_shuaiyan must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveShuaiyan {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    stage: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_mbbeiming" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbbeiming(player, self_skill, stage)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbbeiming must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbbeiming {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    stage: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_zhengjian" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhengjian(player, self_skill, stage, event_player)"
                            .into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhengjian must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhengjian {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    stage: check_expr(&args[2], locals)?,
                    event_player: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "resolve_qinmian" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qinmian(player, self_skill, mode, amount)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qinmian must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveQinmian {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    mode: check_expr(&args[2], locals)?,
                    amount: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "resolve_chuanyun" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_chuanyun(player, self_skill, event_player, event_card)"
                            .into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_chuanyun must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveChuanyun {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    event_player: check_expr(&args[2], locals)?,
                    event_card: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "resolve_jsrgzuozhan_gain" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jsrgzuozhan_gain(player, self_skill, event_player)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jsrgzuozhan_gain must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveJsrgzuozhanGain {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    event_player: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_jsrgdaoren" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jsrgdaoren(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jsrgdaoren must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveJsrgdaoren {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_duwang" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_duwang(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_duwang must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDuwang {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_quanjia" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_quanjia(player, self_skill, source, target)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_quanjia must reference self_skill".into(),
                    });
                }
                let source = check_expr(&args[2], locals)?;
                let target = check_expr(&args[3], locals)?;
                Ok(StmtIr::EffectResolveQuanjia {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    source,
                    target,
                    span: *span,
                })
            }
            "resolve_pxqixian" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_pxqixian(player, self_skill, stage, kind)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_pxqixian must reference self_skill".into(),
                    });
                }
                let stage = check_expr(&args[2], locals)?;
                let kind = check_expr(&args[3], locals)?;
                Ok(StmtIr::EffectResolvePxqixian {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    stage,
                    kind,
                    span: *span,
                })
            }
            "resolve_dcbingji" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcbingji(player, self_skill, stage, kind)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcbingji must reference self_skill".into(),
                    });
                }
                let stage = check_expr(&args[2], locals)?;
                let kind = check_expr(&args[3], locals)?;
                Ok(StmtIr::EffectResolveDcbingji {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    stage,
                    kind,
                    span: *span,
                })
            }
            "choose_hand_cards_to_skill_pile" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_hand_cards_to_skill_pile(player, self_skill, min_count, max_count)"
                            .into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_hand_cards_to_skill_pile must reference self_skill".into(),
                    });
                }
                let min_count = check_expr(&args[2], locals)?;
                let max_count = check_expr(&args[3], locals)?;
                Ok(StmtIr::EffectChooseHandCardsToSkillPile {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    min_count,
                    max_count,
                    span: *span,
                })
            }
            "choose_target_hand_card_to_pile" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_target_hand_card_to_pile(player, self_skill, pile_def)"
                            .into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_target_hand_card_to_pile must reference self_skill".into(),
                    });
                }
                let pile_def = expect_u32(&args[2], 0)?;
                Ok(StmtIr::EffectChooseTargetHandCardToPile {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    pile_def,
                    span: *span,
                })
            }
            "choose_one_from_skill_pile_discard" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_one_from_skill_pile_discard(player, self_skill, pile_def)"
                            .into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_one_from_skill_pile_discard must reference self_skill"
                            .into(),
                    });
                }
                let pile_def = expect_u32(&args[2], 0)?;
                Ok(StmtIr::EffectChooseOneFromSkillPileDiscard {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    pile_def,
                    span: *span,
                })
            }
            "place_top_deck_to_expansion" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "place_top_deck_to_expansion(player, self_skill, pile_def, count)"
                            .into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "place_top_deck_to_expansion must reference self_skill".into(),
                    });
                }
                let pile_def = expect_u32(&args[2], 0)?;
                let count = check_expr(&args[3], locals)?;
                Ok(StmtIr::EffectPlaceTopDeckToExpansion {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    pile_def,
                    count,
                    span: *span,
                })
            }
            "discard_skill_pile" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_skill_pile(player, self_skill, pile_def)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_skill_pile must reference self_skill".into(),
                    });
                }
                let pile_def = expect_u32(&args[2], 0)?;
                Ok(StmtIr::EffectDiscardSkillPile {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    pile_def,
                    span: *span,
                })
            }
            "gain_from_skill_pile" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_from_skill_pile(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_from_skill_pile must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectGainFromSkillPile {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "place_hand_card_to_expansion" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "place_hand_card_to_expansion(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "place_hand_card_to_expansion must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectPlaceHandCardToExpansion {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "extra_resolve_current_trick" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "extra_resolve_current_trick(player, card, self_skill)".into(),
                    });
                }
                if !matches!(&args[2], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "extra_resolve_current_trick must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectExtraResolveCurrentTrick {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "choose_clan_not_max_hand_limit_gain" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_clan_not_max_hand_limit_gain(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_clan_not_max_hand_limit_gain must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectChooseClanNotMaxHandLimitGain {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "gain_target_hand_card" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_target_hand_card(player, self_skill)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_target_hand_card must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectGainTargetHandCard {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "increase_current_sha_base_damage" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "increase_current_sha_base_damage(player, card, amount)".into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "increase-current-Sha card must be a card operand".into(),
                    });
                }
                Ok(StmtIr::EffectIncreaseCurrentShaBaseDamage {
                    player: check_expr(&args[0], locals)?,
                    card,
                    amount: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "skip_next_play_phase" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "skip_next_play_phase(player)".into(),
                    });
                }
                Ok(StmtIr::EffectSkipNextPlayPhase {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "skip_next_judge_phase" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "skip_next_judge_phase(player)".into(),
                    });
                }
                Ok(StmtIr::EffectSkipNextJudgePhase {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "skip_judge_and_draw_phases" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "skip_judge_and_draw_phases(player)".into(),
                    });
                }
                Ok(StmtIr::EffectSkipJudgeAndDrawPhases {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_discard_damage_endpoint_card_increase_if_suit" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_damage_endpoint_card_increase_if_suit(player, target, spade|heart|club|diamond, amount)".into(),
                    });
                }
                let suit = match &args[2] {
                    ExprAst::Ident(name) if name == "spade" => 1,
                    ExprAst::Ident(name) if name == "heart" => 2,
                    ExprAst::Ident(name) if name == "club" => 3,
                    ExprAst::Ident(name) if name == "diamond" => 4,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "damage-endpoint discard suit must be spade, heart, club, or diamond".into(),
                        });
                    }
                };
                Ok(
                    StmtIr::EffectChooseDiscardDamageEndpointCardIncreaseIfSuit {
                        player: check_expr(&args[0], locals)?,
                        target: check_expr(&args[1], locals)?,
                        suit,
                        amount: check_expr(&args[3], locals)?,
                        span: *span,
                    },
                )
            }
            "choose_other_draw_or_recover" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_other_draw_or_recover(player, draw_count, recover_count, times)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectChooseOtherDrawOrRecover {
                    player: check_expr(&args[0], locals)?,
                    draw_count: check_expr(&args[1], locals)?,
                    recover_count: check_expr(&args[2], locals)?,
                    times: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "choose_other_recover" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_recover(player, amount)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseOtherRecover {
                    player: check_expr(&args[0], locals)?,
                    amount: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "enable_equip_slot" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "enable_equip_slot(player, slot, source)".into(),
                    });
                }
                Ok(StmtIr::EffectEnableEquipSlot {
                    player: check_expr(&args[0], locals)?,
                    slot: check_expr(&args[1], locals)?,
                    source: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "choose_discard_hand_equip_then_remove_self_skill" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_hand_equip_then_remove_self_skill(player, count)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardHandEquipThenRemoveSelfSkill {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "draw_selected_targets" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_selected_targets(count)".into(),
                    });
                }
                Ok(StmtIr::EffectDrawSelectedTargets {
                    count: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "recover_selected_targets" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "recover_selected_targets(amount)".into(),
                    });
                }
                Ok(StmtIr::EffectRecoverSelectedTargets {
                    amount: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "record_selected_play_phase_category" => {
                if args.len() != 1
                    || !matches!(&args[0], ExprAst::Ident(name) if name == "self_skill")
                {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "record_selected_play_phase_category(self_skill)".into(),
                    });
                }
                Ok(StmtIr::EffectRecordSelectedPlayPhaseCategory {
                    skill: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "recover_if_min_current_hp" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "recover_if_min_current_hp(player, amount)".into(),
                    });
                }
                Ok(StmtIr::EffectRecoverIfMinCurrentHp {
                    player: check_expr(&args[0], locals)?,
                    amount: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "increase_current_card_base_damage" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "increase_current_card_base_damage(player, card, amount)".into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "increase-current-card card must be a card operand".into(),
                    });
                }
                Ok(StmtIr::EffectIncreaseCurrentCardBaseDamage {
                    player: check_expr(&args[0], locals)?,
                    card,
                    amount: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "modify_current_damage" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "modify_current_damage(player, delta)".into(),
                    });
                }
                Ok(StmtIr::EffectModifyCurrentDamage {
                    player: check_expr(&args[0], locals)?,
                    delta: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_draw_or_increase_current_card_damage_or_both_mark" => {
                if args.len() != 6 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_draw_or_increase_current_card_damage_or_both_mark(player, card, draw_count, damage_amount, mark_def, mark_count)"
                                .into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "current-card branch card must be a card operand".into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
                        player: check_expr(&args[0], locals)?,
                        card,
                        draw_count: check_expr(&args[2], locals)?,
                        damage_amount: check_expr(&args[3], locals)?,
                        mark_def: expect_u32(&args[4], span.start as usize)?,
                        mark_count: check_expr(&args[5], locals)?,
                        span: *span,
                    },
                )
            }
            "draw_then_choose_give_or_discard_hand" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "draw_then_choose_give_or_discard_hand(player, draw_count, payment_count)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectDrawThenChooseGiveOrDiscardHand {
                    player: check_expr(&args[0], locals)?,
                    draw_count: check_expr(&args[1], locals)?,
                    payment_count: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "choose_give_or_discard_hand" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_give_or_discard_hand(player, payment_count, hand_empty)"
                            .into(),
                    });
                }
                let scope = expect_draw_target_scope(&args[2], span.start as usize)?;
                if scope != DrawTargetScopeIr::HandEmpty {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_give_or_discard_hand currently requires hand_empty scope"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectChooseGiveOrDiscardHand {
                    player: check_expr(&args[0], locals)?,
                    payment_count: check_expr(&args[1], locals)?,
                    scope,
                    span: *span,
                })
            }
            "choose_discard_hand_equip_then_recover_target" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_hand_equip_then_recover_target(chooser, target, count, amount)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardHandEquipThenRecoverTarget {
                    chooser: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    count: check_expr(&args[2], locals)?,
                    amount: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "gain_target_equipment" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "gain_target_equipment(player, target, weapon|armor|horse_def|horse_off|treasure)"
                                .into(),
                    });
                }
                let ExprAst::Ident(slot) = &args[2] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "equipment slot must be weapon, armor, horse_def, horse_off, or treasure"
                                .into(),
                    });
                };
                let slot = match slot.as_str() {
                    "weapon" => 1,
                    "armor" => 2,
                    "horse_def" => 3,
                    "horse_off" => 4,
                    "treasure" => 5,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message:
                                "equipment slot must be weapon, armor, horse_def, horse_off, or treasure"
                                    .into(),
                        })
                    }
                };
                Ok(StmtIr::EffectGainTargetEquipment {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    slot,
                    span: *span,
                })
            }
            "discard_all_target_equipment" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_all_target_equipment(actor, target)".into(),
                    });
                }
                Ok(StmtIr::EffectDiscardAllTargetEquipment {
                    actor: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "increase_current_sha_target_damage" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "increase_current_sha_target_damage(source, card, target, amount)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectIncreaseCurrentShaTargetDamage {
                    source: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    target: check_expr(&args[2], locals)?,
                    amount: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "choose_target_draw_then_owner_if_below_max_hp" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_target_draw_then_owner_if_below_max_hp(player, count)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectChooseTargetDrawThenOwnerIfBelowMaxHp {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_bound_source_attack_range_damage" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_bound_source_attack_range_damage(player, source, amount)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectChooseBoundSourceAttackRangeDamage {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    amount: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "choose_target_draw_max_hp_cap_then_discard_excess" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_target_draw_max_hp_cap_then_discard_excess(player, cap)"
                            .into(),
                    });
                }
                let cap = u32::try_from(expect_int(&args[1], span.start as usize)?)
                    .ok()
                    .filter(|cap| *cap > 0)
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "maximum-HP draw/discard cap must be a positive integer".into(),
                    })?;
                Ok(StmtIr::EffectChooseTargetDrawMaxHpCapThenDiscardExcess {
                    player: check_expr(&args[0], locals)?,
                    cap,
                    span: *span,
                })
            }
            "choose_target_draw_to_max_hp_cap_then_add_skill" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_target_draw_to_max_hp_cap_then_add_skill(player, cap, skill_def)"
                                .into(),
                    });
                }
                let cap = u32::try_from(expect_int(&args[1], span.start as usize)?)
                    .ok()
                    .filter(|cap| *cap > 0)
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "maximum-HP draw cap must be a positive integer".into(),
                    })?;
                let skill_def = expect_u32(&args[2], span.start as usize)?;
                if skill_def == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "granted skill def must be non-zero".into(),
                    });
                }
                Ok(StmtIr::EffectChooseTargetDrawMaxHpCapThenAddSkill {
                    player: check_expr(&args[0], locals)?,
                    cap,
                    skill_def,
                    span: *span,
                })
            }
            "choose_discard_heart_redirect_current_damage" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_discard_heart_redirect_current_damage(player, draw_skill_def)"
                                .into(),
                    });
                }
                let draw_skill_def = expect_u32(&args[1], span.start as usize)?;
                if draw_skill_def == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw skill def must be non-zero".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTianxiang {
                    player: check_expr(&args[0], locals)?,
                    draw_skill_def,
                    span: *span,
                })
            }
            "choose_discard_heart_cancel_damage_branch" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_heart_cancel_damage_branch(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveRetianxiang {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_link_other_same_hp" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_link_other_same_hp(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveLianzhou {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_give_hand_equip_to_event_player" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_give_hand_equip_to_event_player(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveStddechong {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_jieliang_gain" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jieliang_gain(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveJieliangGain {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_other_gain_event_cards" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_gain_event_cards(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveStdjujian {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_lulve" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_lulve(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveLulve {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_dctingxian" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dctingxian(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDctingxian {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_other_add_skill" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_add_skill(player, skill_def)".into(),
                    });
                }
                let grant_def = expect_u32(&args[1], span.start as usize)?;
                if grant_def == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "granted skill def must be non-zero".into(),
                    });
                }
                Ok(StmtIr::EffectResolveHfjieying {
                    player: check_expr(&args[0], locals)?,
                    grant_def,
                    span: *span,
                })
            }
            "resolve_dcnuchen" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcnuchen(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcnuchen {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_dcdanji" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcdanji(player, skill)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcdanji {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_dczhiheng" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dczhiheng(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDczhiheng {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_stdgaobian" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_stdgaobian(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveStdgaobian {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_spjiedao" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spjiedao(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpjiedao {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_sibian" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_sibian(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSibian {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "reveal_choose_category_distribute_self_threshold_hp_loss" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_choose_category_distribute_self_threshold_hp_loss(player, reveal_count, recipient_cap)".into(),
                    });
                }
                let reveal_count = bounded_u8(&args[1], "reveal_count")?;
                let recipient_cap = bounded_u8(&args[2], "recipient_cap")?;
                if reveal_count == 0 || recipient_cap == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_count and recipient_cap must be positive".into(),
                    });
                }
                Ok(
                    StmtIr::EffectRevealChooseCategoryDistributeSelfThresholdHpLoss {
                        player: check_expr(&args[0], locals)?,
                        reveal_count,
                        recipient_cap,
                        span: *span,
                    },
                )
            }
            "choose_other_random_gain_region_then_draw_compare_color_lose_hp" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_random_gain_region_then_draw_compare_color_lose_hp(player)".into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
                        player: check_expr(&args[0], locals)?,
                        span: *span,
                    },
                )
            }
            "resolve_refuhai" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_refuhai(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveRefuhai {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_gain_target_hand_ceil_half_difference" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_gain_target_hand_ceil_half_difference(player, target)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectChooseGainTargetHandCeilHalfDifference {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_gzjinfa" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_gzjinfa(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveGzjinfa {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_wn_qifeng" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wn_qifeng(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveWnQifeng {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_dcjue" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcjue(player, event_player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcjue {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_ruanji" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ruanji(player, source, card, action)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveRuanji {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "resolve_zhengbi" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhengbi(player, target, action)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhengbi {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    action: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_fengying" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_fengying(player, helper_def, action)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveFengying {
                    player: check_expr(&args[0], locals)?,
                    helper_def: check_expr(&args[1], locals)?,
                    action: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_qiaojian" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qiaojian(player, target, helper_def, action)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveQiaojian {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    helper_def: check_expr(&args[2], locals)?,
                    action: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "resolve_xicha" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xicha(player, source, card, action)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXicha {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "resolve_huzun" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_huzun(player, source, card, action)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveHuzun {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "resolve_zhugemengxue" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhugemengxue(player, action)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhugemengxue {
                    player: check_expr(&args[0], locals)?,
                    action: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_bailingyun" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_bailingyun(player, action)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveBailingyun {
                    player: check_expr(&args[0], locals)?,
                    action: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_dcshaowei" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcshaowei(player, action)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcshaowei {
                    player: check_expr(&args[0], locals)?,
                    action: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_dcdichou" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcdichou(player, action, helper_skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcdichou {
                    player: check_expr(&args[0], locals)?,
                    action: check_expr(&args[1], locals)?,
                    helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_dcdufeng" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcdufeng(player, action, helper_skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcdufeng {
                    player: check_expr(&args[0], locals)?,
                    action: check_expr(&args[1], locals)?,
                    helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "choose_discard_hand_range" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_hand_range(player, min, max)".into(),
                    });
                }
                let min = expect_u32(&args[1], span.start as usize)?;
                let max = expect_u32(&args[2], span.start as usize)?;
                if min == 0 || max < min {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard range requires 0 < min <= max".into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardHandRange {
                    player: check_expr(&args[0], locals)?,
                    min,
                    max,
                    span: *span,
                })
            }
            "resolve_dcctjiuxian" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcctjiuxian(player, event_player, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if !matches!(action, 0 | 2) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcctjiuxian action must be 0 or 2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcctjiuxian {
                    player: check_expr(&args[0], locals)?,
                    event_player: check_expr(&args[1], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_bianyue" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_bianyue(player, target, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if !matches!(action, 0 | 10) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_bianyue action must be 0 or 10".into(),
                    });
                }
                Ok(StmtIr::EffectResolveBianyue {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_zhupeilan" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhupeilan(player, target, card, helper_def, action)"
                            .into(),
                    });
                }
                let helper_skill_def = expect_u32(&args[3], span.start as usize)?;
                let action = expect_u32(&args[4], span.start as usize)?;
                if !matches!(action, 0 | 1 | 2 | 10) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhupeilan action must be 0, 1, 2, or 10".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhupeilan {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    helper_skill_def,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_dc_sb_zhangxiu" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dc_sb_zhangxiu(player, target, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if !matches!(action, 0 | 10) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dc_sb_zhangxiu action must be 0 or 10".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcSbZhangxiu {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_dc_sb_zhouyu" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dc_sb_zhouyu(player, source, card, action)".into(),
                    });
                }
                let card = check_expr(&args[2], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dc_sb_zhouyu card must be a card".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if !matches!(action, 0 | 1 | 10 | 11 | 20) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dc_sb_zhouyu action must be 0, 1, 10, 11, or 20".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcSbZhouyu {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_dc_sb_lusu" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dc_sb_lusu(player, source, card, action)".into(),
                    });
                }
                let card = check_expr(&args[2], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dc_sb_lusu card must be a card".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if !matches!(action, 0 | 10 | 11 | 20 | 21) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dc_sb_lusu action must be 0, 10, 11, 20, or 21".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcSbLusu {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_zhangjian" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhangjian(player, card, grant_skill_def, action)".into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhangjian card must be a card".into(),
                    });
                }
                let grant_skill_def = expect_u32(&args[2], span.start as usize)?;
                let action = expect_u32(&args[3], span.start as usize)?;
                if !matches!(action, 0 | 1) || (action == 1 && grant_skill_def == 0) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhangjian action must be 0, or 1 with a non-zero granted skill definition".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhangjian {
                    player: check_expr(&args[0], locals)?,
                    card,
                    grant_skill_def,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_zhugeruoxue" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhugeruoxue(player, target, card, action)".into(),
                    });
                }
                let card = check_expr(&args[2], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhugeruoxue card must be a card".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if !matches!(action, 0 | 10) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhugeruoxue action must be 0 or 10".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhugeruoxue {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_caoyi" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_caoyi(player, card, helper_skill_def, action)".into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_caoyi card must be a card".into(),
                    });
                }
                let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
                let action = expect_u32(&args[3], span.start as usize)?;
                if !matches!(action, 0 | 10 | 11 | 20 | 21) || helper_skill_def == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_caoyi requires a non-zero helper and action 0, 10, 11, 20, or 21".into(),
                    });
                }
                Ok(StmtIr::EffectResolveCaoyi {
                    player: check_expr(&args[0], locals)?,
                    card,
                    helper_skill_def,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_malingli" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_malingli(player, card, helper_skill_def, action)".into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_malingli card must be a card".into(),
                    });
                }
                let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
                let action = expect_u32(&args[3], span.start as usize)?;
                if !matches!(action, 0 | 10 | 20) || helper_skill_def == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_malingli requires a non-zero helper and action 0, 10, or 20"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectResolveMalingli {
                    player: check_expr(&args[0], locals)?,
                    card,
                    helper_skill_def,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_nsshuaiyan" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_nsshuaiyan(player, recovered_player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveNsshuaiyan {
                    player: check_expr(&args[0], locals)?,
                    recovered_player: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_wdb_hanlong" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_wdb_hanlong(player, target, card, helper_skill_def, action)"
                                .into(),
                    });
                }
                let target = check_expr(&args[1], locals)?;
                let card = check_expr(&args[2], locals)?;
                if !matches!(
                    target,
                    ExprIr::EventPlayer | ExprIr::U32(_) | ExprIr::Int(_)
                ) || !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_))
                {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_hanlong target/card operands have invalid types"
                            .into(),
                    });
                }
                let helper_skill_def = expect_u32(&args[3], span.start as usize)?;
                let action = expect_u32(&args[4], span.start as usize)?;
                if helper_skill_def == 0 || action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_hanlong requires a non-zero helper and action 0 or 1"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectResolveWdbHanlong {
                    player: check_expr(&args[0], locals)?,
                    target,
                    card,
                    helper_skill_def,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_wdb_liufu" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_wdb_liufu(player, target, card, zhucheng_skill_def, action)"
                                .into(),
                    });
                }
                let target = check_expr(&args[1], locals)?;
                let card = check_expr(&args[2], locals)?;
                if !matches!(
                    target,
                    ExprIr::EventPlayer | ExprIr::EventSource | ExprIr::U32(_) | ExprIr::Int(_)
                ) || !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_))
                {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_liufu target/card operands have invalid types".into(),
                    });
                }
                let zhucheng_skill_def = expect_u32(&args[3], span.start as usize)?;
                let action = expect_u32(&args[4], span.start as usize)?;
                if zhucheng_skill_def == 0 || action > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_wdb_liufu requires a non-zero 筑城 definition and action 0, 1, or 2"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectResolveWdbLiufu {
                    player: check_expr(&args[0], locals)?,
                    target,
                    card,
                    zhucheng_skill_def,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_wdb_liuyan" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_liuyan(player, card, action)".into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_liuyan card operand has invalid type".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_liuyan action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveWdbLiuyan {
                    player: check_expr(&args[0], locals)?,
                    card,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_wdb_liuzan" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_liuzan(player, target, helper_skill_def, action)"
                            .into(),
                    });
                }
                let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
                if helper_skill_def == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_liuzan requires a helper skill definition".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action != 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_liuzan public action must be 0".into(),
                    });
                }
                Ok(StmtIr::EffectResolveWdbLiuzan {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    helper_skill_def,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_wdb_tianyu" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_tianyu(player, target, card, action)".into(),
                    });
                }
                let card = check_expr(&args[2], locals)?;
                if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_tianyu card operand has invalid type".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action != 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_tianyu public action must be 0".into(),
                    });
                }
                Ok(StmtIr::EffectResolveWdbTianyu {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_wdb_xizhenxihong" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_xizhenxihong(player, target, card, action)".into(),
                    });
                }
                let card = check_expr(&args[2], locals)?;
                if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_xizhenxihong card operand has invalid type".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_xizhenxihong public action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveWdbXizhenxihong {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_wdb_yangyi" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_yangyi(player, target, card, action)".into(),
                    });
                }
                let card = check_expr(&args[2], locals)?;
                if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_yangyi card operand has invalid type".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_yangyi public action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveWdbYangyi {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_wdb_zaozhirenjun" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_zaozhirenjun(player, target, card, action)".into(),
                    });
                }
                let card = check_expr(&args[2], locals)?;
                if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_zaozhirenjun card operand has invalid type".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wdb_zaozhirenjun public action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveWdbZaozhiRenjun {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_dddxianxi" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dddxianxi(player, card, action)".into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dddxianxi card operand has invalid type".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dddxianxi action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddXianxi {
                    player: check_expr(&args[0], locals)?,
                    card,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_dddxiaheng" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dddxiaheng(player, action)".into(),
                    });
                }
                let action = expect_u32(&args[1], span.start as usize)?;
                if action != 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dddxiaheng public action must be 0".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddXiaheng {
                    player: check_expr(&args[0], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_liuchong" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_liuchong(player, actor, card, action)".into(),
                    });
                }
                let card = check_expr(&args[2], locals)?;
                if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_liuchong card operand has invalid type".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_liuchong public action must be 0, 1, or 2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddLiuchong {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    card,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_duanbing" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_duanbing(player, target, card, action)".into(),
                    });
                }
                let card = check_expr(&args[2], locals)?;
                if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_duanbing card operand has invalid type".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_duanbing public action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddDuanbing {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_wuzhi" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_wuzhi(player, card, action)".into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_wuzhi card operand has invalid type".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if !matches!(action, 0 | 10) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_wuzhi public action must be 0 or 10".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddWuzhi {
                    player: check_expr(&args[0], locals)?,
                    card,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_xujing" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_xujing(player, source, card, action)".into(),
                    });
                }
                let card = check_expr(&args[2], locals)?;
                if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_xujing card operand has invalid type".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if !matches!(action, 0 | 1 | 2 | 10 | 11 | 12) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_xujing public action must be 0, 1, 2, 10, 11, or 12"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddXujing {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_caomao" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_caomao(player, action)".into(),
                    });
                }
                let action = expect_u32(&args[1], span.start as usize)?;
                if !matches!(action, 0 | 10) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_caomao public action must be 0 or 10".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddCaomao {
                    player: check_expr(&args[0], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_xinxianying" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_xinxianying(player, actor, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if !matches!(action, 0 | 1) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_xinxianying public action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddXinxianying {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_xianglang" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_xianglang(player, actor, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if !matches!(action, 0 | 1 | 2) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_xianglang public action must be 0, 1, or 2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddXianglang {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_yujin" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_yujin(player, actor, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if !matches!(action, 0 | 1) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_yujin public action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddYujin {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_liuye" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_liuye(player, actor, card, action)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_liuye public action must be 0..=4".into(),
                    });
                }
                let card = check_expr(&args[2], locals)?;
                if !matches!(card, ExprIr::EventCard | ExprIr::U32(_) | ExprIr::Int(_)) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_liuye card operand has invalid type".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddLiuye {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    card,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_baosanniang" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_baosanniang(player, action)".into(),
                    });
                }
                let action = expect_u32(&args[1], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_baosanniang public action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddBaosanniang {
                    player: check_expr(&args[0], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_zhenji" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_zhenji(player, action)".into(),
                    });
                }
                let action = expect_u32(&args[1], span.start as usize)?;
                if action > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_zhenji public action must be 0..=2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddZhenji {
                    player: check_expr(&args[0], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_zhaoang" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_zhaoang(player, action)".into(),
                    });
                }
                let action = expect_u32(&args[1], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_zhaoang public action must be 0..=1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddZhaoang {
                    player: check_expr(&args[0], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_liuba" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_liuba(player, actor, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_liuba public action must be 0..=4".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddLiuba {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_jianshuo" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_jianshuo(player, target, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_jianshuo public action must be 0..=2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddJianshuo {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_guanning" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_guanning(player, actor, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_guanning public action must be 0..=2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddGuanning {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_caoshuang" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_caoshuang(player, actor, card, action)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_caoshuang public action must be 0..=3".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddCaoshuang {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_xuelingyun" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_xuelingyun(player, actor, card, action)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_xuelingyun public action must be 0..=5".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddXuelingyun {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_liuhong" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_liuhong(player, actor, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_liuhong public action must be 0..=5".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddLiuhong {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_xiahouxuan" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_xiahouxuan(player, action)".into(),
                    });
                }
                let action = expect_u32(&args[1], span.start as usize)?;
                if action > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_xiahouxuan public action must be 0..=2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddXiahouxuan {
                    player: check_expr(&args[0], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_zhangkai" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_zhangkai(player, actor, card, action)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_zhangkai public action must be 0..=4".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddZhangkai {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_ddd_liangxi" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_liangxi(player, actor, card, action)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ddd_liangxi public action must be 0..=2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDddLiangxi {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "guozhan_use_zhulian_bihe" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "guozhan_use_zhulian_bihe(player)".into(),
                    });
                }
                Ok(StmtIr::EffectGuozhanUseZhulianBihe {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_shoulijian" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_shoulijian(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveShoulijian {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "cancel_incoming_card_use" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "cancel_incoming_card_use(player, source, card)".into(),
                    });
                }
                Ok(StmtIr::EffectCancelIncomingCardUse {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "choose_other_gain_one_of_two_skills" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_gain_one_of_two_skills(player, target, first_def, second_def)"
                            .into(),
                    });
                }
                let first_def = expect_u32(&args[2], span.start as usize)?;
                let second_def = expect_u32(&args[3], span.start as usize)?;
                if first_def == 0 || second_def == 0 || first_def == second_def {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_gain_one_of_two_skills expects two distinct non-zero skill definitions".into(),
                    });
                }
                Ok(StmtIr::EffectChooseOtherGainOneOfTwoSkills {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    first_def,
                    second_def,
                    span: *span,
                })
            }
            "gwyewu_random_discard_chain" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gwyewu_random_discard_chain(player, wuyao_def, lang_def)".into(),
                    });
                }
                let wuyao_def = expect_u32(&args[1], span.start as usize)?;
                let lang_def = expect_u32(&args[2], span.start as usize)?;
                if wuyao_def == 0 || lang_def == 0 || wuyao_def == lang_def {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gwyewu_random_discard_chain expects two distinct non-zero skill definitions".into(),
                    });
                }
                Ok(StmtIr::EffectGwyewuRandomDiscardChain {
                    player: check_expr(&args[0], locals)?,
                    wuyao_def,
                    lang_def,
                    span: *span,
                })
            }
            "give_selected_then_target_may_use_draw_owner" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "give_selected_then_target_may_use_draw_owner(player, target)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectGiveSelectedThenTargetMayUseDrawOwner {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "skip_next_discard_phase" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "skip_next_discard_phase(player)".into(),
                    });
                }
                Ok(StmtIr::EffectSkipNextDiscardPhase {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_spfengyin" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spfengyin(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpfengyin {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_xinfu_lianpian" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xinfu_lianpian(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXinfuLianpian {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_gongjian" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_gongjian(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveGongjian {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_xinfu_qinguo" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xinfu_qinguo(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXinfuQinguo {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_dcmanzhi" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcmanzhi(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcmanzhi {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_xinxingluan" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xinxingluan(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXinxingluan {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_dckuizhen" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dckuizhen(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDckuizhen {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_dckuizhen_resolve" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dckuizhen_resolve(player, winner, loser)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDckuizhenResolve {
                    player: check_expr(&args[0], locals)?,
                    winner: check_expr(&args[1], locals)?,
                    loser: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_nsdingzhou" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_nsdingzhou(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveNsdingzhou {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_sxrmlanjiao" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_sxrmlanjiao(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSxrmlanjiao {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_rekuanshi" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_rekuanshi(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveRekuanshi {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_rekuanshi_resolve" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_rekuanshi_resolve(player, damaged)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveRekuanshiResolve {
                    player: check_expr(&args[0], locals)?,
                    damaged: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_dcmoyu" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcmoyu(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcmoyu {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_xizhen" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xizhen(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXizhen {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_xuxie" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xuxie(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXuxie {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_xuxie_add" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xuxie_add(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXuxieAdd {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_dcchiying" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcchiying(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcchiying {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_xinkuangfu" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xinkuangfu(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXinkuangfu {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_dcxiangshu" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcxiangshu(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcxiangshu {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_shuangxiong" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_shuangxiong(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveShuangxiong {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_zhijian" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhijian(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhijian {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_guzheng" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_guzheng(player, event_player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveGuzheng {
                    player: check_expr(&args[0], locals)?,
                    event_player: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_zhiba" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhiba(player, actor)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhiba {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_drlt_congjian" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_drlt_congjian(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDrltCongjian {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_nzry_feijun" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_nzry_feijun(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveNzryFeijun {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_drlt_yongsi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_drlt_yongsi(player, stage)".into(),
                    });
                }
                let stage = expect_u32(&args[1], span.start as usize)?;
                if stage > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_drlt_yongsi stage must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDrltYongsi {
                    player: check_expr(&args[0], locals)?,
                    stage: stage as u8,
                    span: *span,
                })
            }
            "resolve_drlt_weidi" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_drlt_weidi(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDrltWeidi {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_dcxiangshu_lottery" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcxiangshu_lottery(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcxiangshuLottery {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_dcyingtu" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcyingtu(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcyingtu {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "discard_hand_equip_count" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_hand_equip_count(player, count)".into(),
                    });
                }
                let count = u32::try_from(expect_int(&args[1], span.start as usize)?)
                    .ok()
                    .filter(|count| *count > 0)
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "discard count must be a positive integer".into(),
                    })?;
                Ok(StmtIr::EffectDiscardHandEquipCount {
                    player: check_expr(&args[0], locals)?,
                    count,
                    span: *span,
                })
            }
            "give_selected_then_target_choose_discard_same_suit_or_lose_hp" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "give_selected_then_target_choose_discard_same_suit_or_lose_hp(player, target)"
                                .into(),
                    });
                }
                Ok(
                    StmtIr::EffectGiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
                        player: check_expr(&args[0], locals)?,
                        target: check_expr(&args[1], locals)?,
                        span: *span,
                    },
                )
            }
            "gain_equipment_from_draw_pile" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_equipment_from_draw_pile(player)".into(),
                    });
                }
                Ok(StmtIr::EffectGainEquipmentFromDrawPile {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "target_choose_suit_gain_owner_hand_damage_if_mismatch" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "target_choose_suit_gain_owner_hand_damage_if_mismatch(player, target)"
                                .into(),
                    });
                }
                Ok(
                    StmtIr::EffectTargetChooseSuitGainOwnerHandDamageIfMismatch {
                        player: check_expr(&args[0], locals)?,
                        target: check_expr(&args[1], locals)?,
                        span: *span,
                    },
                )
            }
            "judge_source_discard_two_hand_or_damage" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judge_source_discard_two_hand_or_damage(player, source)".into(),
                    });
                }
                Ok(StmtIr::EffectJudgeSourceDiscardTwoHandOrDamage {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "give_selected_recover_after_phase_total_two" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "give_selected_recover_after_phase_total_two(player, target)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectGiveSelectedRecoverAfterPhaseTotalTwo {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_other_hand_targets_gain_one_replace_draw" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_hand_targets_gain_one_replace_draw(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseOtherHandTargetsGainOneReplaceDraw {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_other_hand_targets_gain_one_reduce_draw" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_hand_targets_gain_one_reduce_draw(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseOtherHandTargetsGainOneReduceDraw {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "distribute_top_two_cards" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "distribute_top_two_cards(player)".into(),
                    });
                }
                Ok(StmtIr::EffectDistributeTopTwoCards {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "give_all_hand_recover_to_one" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "give_all_hand_recover_to_one(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectGiveAllHandRecoverToOne {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(player)"
                                .into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
                        player: check_expr(&args[0], locals)?,
                        span: *span,
                    },
                )
            }
            "choose_give_one_hand_equip_or_recover" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_give_one_hand_equip_or_recover(owner, giver, target)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectChooseGiveOneHandEquipOrRecover {
                    owner: check_expr(&args[0], locals)?,
                    giver: check_expr(&args[1], locals)?,
                    target: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "choose_give_two_different_colors_or_recover" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_give_two_different_colors_or_recover(owner, giver, target)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectChooseGiveTwoDifferentColorsOrRecover {
                    owner: check_expr(&args[0], locals)?,
                    giver: check_expr(&args[1], locals)?,
                    target: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_mbzhuguo" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbzhuguo(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbzhuguo {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_mbanxian" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbanxian(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbanxian {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_mbjinzu" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbjinzu(player, target, helper_skill_def)".into(),
                    });
                }
                let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
                if helper_skill_def == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "mbjinzu helper_skill_def must be positive".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbjinzu {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    helper_skill_def,
                    span: *span,
                })
            }
            "apply_mbjinzu_next_sha" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "apply_mbjinzu_next_sha(player, card)".into(),
                    });
                }
                Ok(StmtIr::EffectApplyMbjinzuNextSha {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "lose_hp_then_damage_other_lose_max_hp_if_dying" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "lose_hp_then_damage_other_lose_max_hp_if_dying(player, target, amount, self_skill)"
                                .into(),
                    });
                }
                if !matches!(&args[3], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "HP-loss-linked damage must reference self_skill".into(),
                    });
                }
                Ok(StmtIr::EffectLoseHpThenDamageOtherLoseMaxHpIfDying {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    amount: check_expr(&args[2], locals)?,
                    skill: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "emit_note" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "emit_note(code)".into(),
                    });
                }
                Ok(StmtIr::EffectNote {
                    code: expect_int(&args[0], span.start as usize)?,
                    span: *span,
                })
            }
            "change_max_hp" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "change_max_hp(player, delta)".into(),
                    });
                }
                Ok(StmtIr::EffectChangeMaxHp {
                    player: check_expr(&args[0], locals)?,
                    delta: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "add_mark" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "add_mark(player, mark_def, delta)".into(),
                    });
                }
                let player = check_expr(&args[0], locals)?;
                let mark = expect_u32(&args[1], span.start as usize)?;
                match expect_int(&args[2], span.start as usize) {
                    Ok(delta) => Ok(StmtIr::EffectAddMark {
                        player,
                        mark,
                        delta,
                        span: *span,
                    }),
                    Err(_) => Ok(StmtIr::EffectAddMarkExpr {
                        player,
                        mark,
                        delta: check_expr(&args[2], locals)?,
                        span: *span,
                    }),
                }
            }
            "remove_mark" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "remove_mark(player, mark_def, amount)".into(),
                    });
                }
                let player = check_expr(&args[0], locals)?;
                let mark = expect_u32(&args[1], span.start as usize)?;
                match expect_int(&args[2], span.start as usize) {
                    Ok(amount) => Ok(StmtIr::EffectRemoveMark {
                        player,
                        mark,
                        amount,
                        span: *span,
                    }),
                    Err(_) => Ok(StmtIr::EffectRemoveMarkExpr {
                        player,
                        mark,
                        amount: check_expr(&args[2], locals)?,
                        span: *span,
                    }),
                }
            }
            "discard_hand" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_hand(player, count)".into(),
                    });
                }
                Ok(StmtIr::EffectDiscardHand {
                    player: check_expr(&args[0], locals)?,
                    count: expect_int(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "discard_hand_expr" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_hand_expr(player, count)".into(),
                    });
                }
                Ok(StmtIr::EffectDiscardHandExpr {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "discard_hand_equip_expr" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_hand_equip_expr(player, count)".into(),
                    });
                }
                Ok(StmtIr::EffectDiscardHandEquipExpr {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "discard_hand_equip" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_hand_equip(player, count)".into(),
                    });
                }
                Ok(StmtIr::EffectDiscardHandEquip {
                    player: check_expr(&args[0], locals)?,
                    count: expect_int(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "adjust_hand_to" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "adjust_hand_to(player, count)".into(),
                    });
                }
                Ok(StmtIr::EffectAdjustHandTo {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "discard_selected" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_selected(player)".into(),
                    });
                }
                Ok(StmtIr::EffectDiscardSelected {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "recast_selected" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "recast_selected(player)".into(),
                    });
                }
                Ok(StmtIr::EffectRecastSelected {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "disable_all_equip_slots" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "disable_all_equip_slots(player)".into(),
                    });
                }
                Ok(StmtIr::EffectDisableAllEquipSlots {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "move_equipment_then_draw" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "move_equipment_then_draw(player)".into(),
                    });
                }
                Ok(StmtIr::EffectMoveEquipmentThenDraw {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_use_virtual_sha_or_draw" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_use_virtual_sha_or_draw(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseUseVirtualShaOrDraw {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "discard_equipment_pair_with_armor" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_equipment_pair_with_armor(player)".into(),
                    });
                }
                Ok(StmtIr::EffectDiscardEquipmentPairWithArmor {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "discard_target_equipment_gain_mount" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_target_equipment_gain_mount(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectDiscardTargetEquipmentGainMount {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "discard_target_equipment_draw_weapon_range" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_target_equipment_draw_weapon_range(player, target)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectDiscardTargetEquipmentDrawWeaponRange {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_target_reveal_hand_damage_if_sha" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_target_reveal_hand_damage_if_sha(player, amount)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseTargetRevealHandDamageIfSha {
                    player: check_expr(&args[0], locals)?,
                    amount: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_other_give_one_hand_equip" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_give_one_hand_equip(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseOtherGiveOneHandEquip {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_reveal_hand_card_bind_category_target" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_reveal_hand_card_bind_category_target(player, target)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectChooseRevealHandCardBindCategoryTarget {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "recover_to_max_then_optional_exchange_skill" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "recover_to_max_then_optional_exchange_skill(player, removable_skill_def)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectRecoverToMaxThenOptionalExchangeSkill {
                    player: check_expr(&args[0], locals)?,
                    removable_skill_def: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "reveal_hand_discard_kind" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_hand_discard_kind(player, card_kind)".into(),
                    });
                }
                let ExprAst::Ident(kind) = &args[1] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_hand_discard_kind expects a card-kind name".into(),
                    });
                };
                let kind = card_kind_value(kind).ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: format!("unknown card kind {kind}"),
                })?;
                Ok(StmtIr::EffectRevealHandDiscardKind {
                    player: check_expr(&args[0], locals)?,
                    kind,
                    span: *span,
                })
            }
            "discard_target_card_gain_mount" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_target_card_gain_mount(player, target, equip|hand_equip|hand_equip_judge|equip_judge)".into(),
                    });
                }
                let ExprAst::Ident(scope) = &args[2] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_target_card_gain_mount scope must be equip, hand_equip, hand_equip_judge, or equip_judge".into(),
                    });
                };
                let scope = match scope.as_str() {
                    "equip" => TargetCardScopeIr::Equip,
                    "hand_equip" => TargetCardScopeIr::HandEquip,
                    "hand_equip_judge" => TargetCardScopeIr::HandEquipJudge,
                    "equip_judge" => TargetCardScopeIr::EquipJudge,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "discard_target_card_gain_mount scope must be equip, hand_equip, hand_equip_judge, or equip_judge".into(),
                        })
                    }
                };
                Ok(StmtIr::EffectDiscardTargetCardGainMount {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    scope,
                    span: *span,
                })
            }
            "make_current_sha_target_unrespondable" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "make_current_sha_target_unrespondable(source, card, target)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectMakeCurrentShaTargetUnrespondable {
                    source: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    target: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "judge_make_current_sha_target_unrespondable_on_red" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "judge_make_current_sha_target_unrespondable_on_red(player, card, target)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectJudgeMakeCurrentShaTargetUnrespondableOnRed {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    target: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_retieji" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_retieji(player, card, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveRetieji {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    target: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_new_reluoyi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_new_reluoyi(player, buff_skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveNewReluoyi {
                    player: check_expr(&args[0], locals)?,
                    buff_skill_def: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_remoshi" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_remoshi(player, target, card, helper_skill_def, action)"
                            .into(),
                    });
                }
                let action = expect_u32(&args[4], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_remoshi action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveRemoshi {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    helper_skill_def: expect_u32(&args[3], span.start as usize)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_xinyaoming" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xinyaoming(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXinyaoming {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_rehuaiyi" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_rehuaiyi(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveRehuaiyi {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_reanguo" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_reanguo(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveReanguo {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_target_discard_distance_hand_or_increase_damage" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_target_discard_distance_hand_or_increase_damage(player, target)"
                                .into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseTargetDiscardDistanceHandOrIncreaseDamage {
                        player: check_expr(&args[0], locals)?,
                        target: check_expr(&args[1], locals)?,
                        span: *span,
                    },
                )
            }
            "choose_draw_view_two_gain_or_move_equipment" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_draw_view_two_gain_or_move_equipment(player, target)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectChooseDrawViewTwoGainOrMoveEquipment {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_beizhu" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_beizhu(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveBeizhu {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_mbshanjia" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_mbshanjia(player, lost_equipment_count, distance_skill_def)"
                                .into(),
                    });
                }
                let distance_skill_def = expect_u32(&args[2], span.start as usize)?;
                Ok(StmtIr::EffectResolveMbshanjia {
                    player: check_expr(&args[0], locals)?,
                    lost_equipment_count: check_expr(&args[1], locals)?,
                    distance_skill_def,
                    span: *span,
                })
            }
            "resolve_daoji" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_daoji(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDaoji {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_tw_daoji" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_tw_daoji(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwDaoji {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "give_all_hand_then_choose_use_one_or_damage" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "give_all_hand_then_choose_use_one_or_damage(player, target)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectGiveAllHandThenChooseUseOneOrDamage {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_category_then_target_give_matching_or_sha_unrespondable" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_category_then_target_give_matching_or_sha_unrespondable(player, card, target)".into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
                        player: check_expr(&args[0], locals)?,
                        card: check_expr(&args[1], locals)?,
                        target: check_expr(&args[2], locals)?,
                        span: *span,
                    },
                )
            }
            "choose_category_then_target_give_matching_or_damage_plus_one" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_category_then_target_give_matching_or_damage_plus_one(player, target)".into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
                        player: check_expr(&args[0], locals)?,
                        target: check_expr(&args[1], locals)?,
                        span: *span,
                    },
                )
            }
            "choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip(player, source)".into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
                        player: check_expr(&args[0], locals)?,
                        source: check_expr(&args[1], locals)?,
                        span: *span,
                    },
                )
            }
            "resolve_mobiledanshou" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mobiledanshou(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMobiledanshou {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_jsrgchiying" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jsrgchiying(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveJsrgchiying {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_mbqingdao" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbqingdao(player, source, card, damaged)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbqingdao {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    damaged: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "resolve_mbxiugeng" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbxiugeng(player, helper_def, hand_limit_def, stage)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbxiugeng {
                    player: check_expr(&args[0], locals)?,
                    helper_def: expect_u32(&args[1], span.start as usize)?,
                    hand_limit_def: expect_u32(&args[2], span.start as usize)?,
                    stage: u8::try_from(expect_u32(&args[3], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "修耕 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_mbchenshe" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbchenshe(player, dying, source)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbchenshe {
                    player: check_expr(&args[0], locals)?,
                    dying: check_expr(&args[1], locals)?,
                    source: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_mbquanchong" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbquanchong(player, helper_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbquanchong {
                    player: check_expr(&args[0], locals)?,
                    helper_def: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_mbrenxing" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbrenxing(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbrenxing {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_mbchizhang" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbchizhang(player, card)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbchizhang {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_mbduanyang" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbduanyang(player, target, card, mode)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbduanyang {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    mode: u8::try_from(expect_u32(&args[3], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "断鞅 mode must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_mbrunwei" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbrunwei(player, target, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbrunwei {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "润微 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_mbshuanghuai" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbshuanghuai(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbshuanghuai {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_mbxiongtu" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbxiongtu(player, target, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbxiongtu {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "凶图 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_mbganggeng" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbganggeng(player, target, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbganggeng {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "刚鲠 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_mbsijian" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbsijian(player, helper_skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbsijian {
                    player: check_expr(&args[0], locals)?,
                    helper_def: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_mbjianji" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbjianji(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbjianji {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_mbyuanmo" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbyuanmo(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbyuanmo {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_mbfeijing" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbfeijing(player, target, card)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbfeijing {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_mbxiaoge" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbxiaoge(player, target, card, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbxiaoge {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    stage: u8::try_from(expect_u32(&args[3], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "骁戈 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_mbxuye" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbxuye(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbxuye {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_mbkuangxiang" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbkuangxiang(player, target, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbkuangxiang {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "狂想 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_potchengpu" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potchengpu(player, target, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotchengpu {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "势程普 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_potjugu" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potjugu(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotjugu {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_potsifeng" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potsifeng(player, target, card, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotsifeng {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    stage: u8::try_from(expect_u32(&args[3], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "伺锋 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_potchenqun" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potchenqun(player, target, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotchenqun {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "势陈群 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_potwanglie" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potwanglie(player, card, helper_skill_def, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotwanglie {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                    stage: u8::try_from(expect_u32(&args[3], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "势陈到往烈 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_pothongyi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_pothongyi(player, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePothongyi {
                    player: check_expr(&args[0], locals)?,
                    stage: u8::try_from(expect_u32(&args[1], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "势陈到弘毅 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_potguansha" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potguansha(player, helper_skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotguansha {
                    player: check_expr(&args[0], locals)?,
                    helper_skill_def: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_potjiyu" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potjiyu(player, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotjiyu {
                    player: check_expr(&args[0], locals)?,
                    stage: u8::try_from(expect_u32(&args[1], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "势娄圭急御 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_mbzhuji" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbzhuji(player, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbzhuji {
                    player: check_expr(&args[0], locals)?,
                    stage: u8::try_from(expect_u32(&args[1], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "兵势孙韶筑墼 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_potlibing" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potlibing(player, card, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotlibing {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "势夏侯霸砺兵 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_potpoxi" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potpoxi(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotpoxi {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_potfuan" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_potfuan(player, card, stage, range_helper_def, yinxian_def)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotfuan {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "势张任伏暗 stage must fit u8".into(),
                        },
                    )?,
                    range_helper_def: expect_u32(&args[3], span.start as usize)?,
                    yinxian_def: expect_u32(&args[4], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_potyinxian" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potyinxian(player, card, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotyinxian {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "势张任引弦 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_pothuanjie" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_pothuanjie(player, target, stage, qice_def, kanpo_def)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectResolvePothuanjie {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "势桓阶 resolver stage must fit u8".into(),
                        },
                    )?,
                    qice_def: expect_u32(&args[3], span.start as usize)?,
                    kanpo_def: expect_u32(&args[4], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_potnigu" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potnigu(player, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotnigu {
                    player: check_expr(&args[0], locals)?,
                    stage: u8::try_from(expect_u32(&args[1], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "势孙綝逆固 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_potlulian" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potlulian(player, card, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotlulian {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    stage: u8::try_from(expect_u32(&args[2], span.start as usize)?).map_err(
                        |_| CompileError::Type {
                            pos: span.start as usize,
                            message: "势孙綝戮连 stage must fit u8".into(),
                        },
                    )?,
                    span: *span,
                })
            }
            "resolve_potdimeng" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potdimeng(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotdimeng {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_pothaoshi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_pothaoshi(player, action)".into(),
                    });
                }
                let action =
                    u8::try_from(expect_u32(&args[1], span.start as usize)?).map_err(|_| {
                        CompileError::Type {
                            pos: span.start as usize,
                            message: "pothaoshi action must fit u8".into(),
                        }
                    })?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "pothaoshi content action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePothaoshi {
                    player: check_expr(&args[0], locals)?,
                    action,
                    span: *span,
                })
            }
            "resolve_spmiaolve" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spmiaolve(player, action)".into(),
                    });
                }
                let action =
                    u8::try_from(expect_u32(&args[1], span.start as usize)?).map_err(|_| {
                        CompileError::Type {
                            pos: span.start as usize,
                            message: "spmiaolve action must fit u8".into(),
                        }
                    })?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "spmiaolve action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpmiaolve {
                    player: check_expr(&args[0], locals)?,
                    action,
                    span: *span,
                })
            }
            "resolve_spyingjia" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_spyingjia(player, action, helper_def)".into(),
                    });
                }
                let action =
                    u8::try_from(expect_u32(&args[1], span.start as usize)?).map_err(|_| {
                        CompileError::Type {
                            pos: span.start as usize,
                            message: "spyingjia action must fit u8".into(),
                        }
                    })?;
                if action > 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "spyingjia action must be 0, 1, 2, or 3".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpyingjia {
                    player: check_expr(&args[0], locals)?,
                    action,
                    helper_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "choose_lose_hp_use_virtual_sha_by_basic_variants" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_lose_hp_use_virtual_sha_by_basic_variants(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseLoseHpUseVirtualShaByBasicVariants {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_twfupan" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twfupan(player, damage_amount)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwfupan {
                    player: check_expr(&args[0], locals)?,
                    damage_amount: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "suppress_non_locked_skills_until_turn_end" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "suppress_non_locked_skills_until_turn_end(player)".into(),
                    });
                }
                Ok(StmtIr::EffectSuppressNonLockedSkillsUntilTurnEnd {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_recover_target" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_recover_target(chooser, target, amount)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseRecoverTarget {
                    chooser: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    amount: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_jiaohua" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jiaohua(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveJiaohua {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_luanqun" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_luanqun(player, effect_skill_def, direct_hit_skill_def)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectResolveLuanqun {
                    player: check_expr(&args[0], locals)?,
                    effect_skill_def: expect_u32(&args[1], span.start as usize)?,
                    direct_hit_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "choose_discard_basic_then_target_discard_equip_or_damage" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_discard_basic_then_target_discard_equip_or_damage(player, target)"
                                .into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseDiscardBasicThenTargetDiscardEquipOrDamage {
                        player: check_expr(&args[0], locals)?,
                        target: check_expr(&args[1], locals)?,
                        span: *span,
                    },
                )
            }
            "choose_current_sha_target_discard_redirect_to_owner" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_current_sha_target_discard_redirect_to_owner(owner, source, card, target)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectChooseCurrentShaTargetDiscardRedirectToOwner {
                    owner: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    target: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "choose_target_gain_bottom_card" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_target_gain_bottom_card(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseTargetGainBottomCard {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "judge_black_gain_repeat" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judge_black_gain_repeat(player)".into(),
                    });
                }
                Ok(StmtIr::EffectJudgeBlackGainRepeat {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "judge_black_gain_repeat_protect_hand_limit" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judge_black_gain_repeat_protect_hand_limit(player)".into(),
                    });
                }
                Ok(StmtIr::EffectJudgeBlackGainRepeatProtectHandLimit {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "judge_same_color_gain_repeat" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judge_same_color_gain_repeat(player)".into(),
                    });
                }
                Ok(StmtIr::EffectJudgeSameColorGainRepeat {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_current_trick_targets_to_nullify" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_current_trick_targets_to_nullify(player, source, card)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectChooseCurrentTrickTargetsToNullify {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "use_virtual_juedou_second_on_first" => {
                if !args.is_empty() {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "use_virtual_juedou_second_on_first()".into(),
                    });
                }
                Ok(StmtIr::EffectUseVirtualJuedouSecondOnFirst { span: *span })
            }
            effect @ ("choose_discard_target_card" | "choose_optional_discard_target_card") => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: format!(
                            "{effect}(player, target, hand|equip|hand_equip|hand_equip_judge)"
                        ),
                    });
                }
                let ExprAst::Ident(scope) = &args[2] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: format!(
                            "{effect} scope must be hand, equip, hand_equip, or hand_equip_judge"
                        ),
                    });
                };
                let scope = match scope.as_str() {
                    "hand" => TargetCardScopeIr::Hand,
                    "equip" => TargetCardScopeIr::Equip,
                    "hand_equip" => TargetCardScopeIr::HandEquip,
                    "hand_equip_judge" => TargetCardScopeIr::HandEquipJudge,
                    "equip_judge" => TargetCardScopeIr::EquipJudge,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: format!(
                                "{effect} scope must be hand, equip, hand_equip, or hand_equip_judge"
                            ),
                        });
                    }
                };
                let player = check_expr(&args[0], locals)?;
                let target = check_expr(&args[1], locals)?;
                Ok(if effect == "choose_discard_target_card" {
                    StmtIr::EffectChooseDiscardTargetCard {
                        player,
                        target,
                        scope,
                        span: *span,
                    }
                } else {
                    StmtIr::EffectChooseOptionalDiscardTargetCard {
                        player,
                        target,
                        scope,
                        span: *span,
                    }
                })
            }
            "reveal_hand_choose_discard_color" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_hand_choose_discard_color(player)".into(),
                    });
                }
                Ok(StmtIr::EffectRevealHandChooseDiscardColor {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "reveal_hand" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_hand(player)".into(),
                    });
                }
                Ok(StmtIr::EffectRevealHand {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "draw_linked_players" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_linked_players(count)".into(),
                    });
                }
                Ok(StmtIr::EffectDrawLinkedPlayers {
                    count: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_discard_target_cards_recover_same_category" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_target_cards_recover_same_category(player, target, count, amount)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardTargetCardsRecoverSameCategory {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    count: check_expr(&args[2], locals)?,
                    amount: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "choose_scskuiji_discard" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_scskuiji_discard(player, target, count)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseScskuijiDiscard {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    count: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "choose_pskuiji_discard" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_pskuiji_discard(player, target, count)".into(),
                    });
                }
                Ok(StmtIr::EffectChoosePskuijiDiscard {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    count: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "choose_compare_selected_hand_card_second_failure_seat_damage" => {
                if args.len() != 6 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_compare_selected_hand_card_second_failure_seat_damage(player, target, draw_count, failure_count, lose_hp, damage)".into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseCompareSelectedHandCardSecondFailureSeatDamage {
                        player: check_expr(&args[0], locals)?,
                        target: check_expr(&args[1], locals)?,
                        draw_count: check_expr(&args[2], locals)?,
                        failure_count: check_expr(&args[3], locals)?,
                        lose_hp: check_expr(&args[4], locals)?,
                        damage: check_expr(&args[5], locals)?,
                        span: *span,
                    },
                )
            }
            "judge_prevent_damage_lose_max_hp_on_nonheart" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judge_prevent_damage_lose_max_hp_on_nonheart(player, target)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectJudgePreventDamageLoseMaxHpOnNonHeart {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_gain_top_cards_by_suit" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_gain_top_cards_by_suit(player, count, spade|heart|club|diamond)"
                                .into(),
                    });
                }
                let count = u8::try_from(expect_int(&args[1], span.start as usize)?)
                    .ok()
                    .filter(|count| *count > 0)
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "top-card view count must be 1..255".into(),
                    })?;
                let suit = match &args[2] {
                    ExprAst::Ident(name) if name == "spade" => 1,
                    ExprAst::Ident(name) if name == "heart" => 2,
                    ExprAst::Ident(name) if name == "club" => 3,
                    ExprAst::Ident(name) if name == "diamond" => 4,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message:
                                "top-card selection suit must be spade, heart, club, or diamond"
                                    .into(),
                        });
                    }
                };
                Ok(StmtIr::EffectChooseGainTopCardsBySuit {
                    player: check_expr(&args[0], locals)?,
                    count,
                    suit,
                    span: *span,
                })
            }
            "choose_other_draw_by_hand_count" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_draw_by_hand_count(player, normal_count, empty_count, times)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectChooseOtherDrawByHandCount {
                    player: check_expr(&args[0], locals)?,
                    normal_count: check_expr(&args[1], locals)?,
                    empty_count: check_expr(&args[2], locals)?,
                    times: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "discard_one_hand_or_equip" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_one_hand_or_equip(player)".into(),
                    });
                }
                Ok(StmtIr::EffectDiscardOneHandOrEquip {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_discard_one_then_gain_target_equipment" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_one_then_gain_target_equipment(player, target, weapon|armor|horse_def|horse_off|treasure)".into(),
                    });
                }
                let ExprAst::Ident(slot) = &args[2] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "equipment slot must be weapon, armor, horse_def, horse_off, or treasure".into(),
                    });
                };
                let slot = match slot.as_str() {
                    "weapon" => 1,
                    "armor" => 2,
                    "horse_def" => 3,
                    "horse_off" => 4,
                    "treasure" => 5,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "equipment slot must be weapon, armor, horse_def, horse_off, or treasure".into(),
                        })
                    }
                };
                Ok(StmtIr::EffectChooseDiscardOneThenGainTargetEquipment {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    slot,
                    span: *span,
                })
            }
            "choose_target_draw" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_target_draw(player, count, other|male_other|female_other|any|min_hand|same_hp)"
                                .into(),
                    });
                }
                let scope = expect_draw_target_scope(&args[2], span.start as usize)?;
                Ok(StmtIr::EffectChooseTargetDraw {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    scope,
                    span: *span,
                })
            }
            "choose_targets_draw" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_targets_draw(player, count, target_count, other|male_other|female_other|any|min_hand|same_hp)".into(),
                    });
                }
                let target_count = u8::try_from(expect_int(&args[2], span.start as usize)?)
                    .ok()
                    .filter(|count| *count > 0)
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "draw target count must be 1..255".into(),
                    })?;
                let scope = expect_draw_target_scope(&args[3], span.start as usize)?;
                Ok(StmtIr::EffectChooseTargetsDraw {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    target_count,
                    scope,
                    span: *span,
                })
            }
            "choose_optional_targets_draw" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_optional_targets_draw(player, count, min_count, max_count, scope)"
                                .into(),
                    });
                }
                let min_count = u8::try_from(expect_int(&args[2], span.start as usize)?)
                    .ok()
                    .filter(|count| *count > 0)
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "draw target minimum must be 1..255".into(),
                    })?;
                let max_count = u8::try_from(expect_int(&args[3], span.start as usize)?)
                    .ok()
                    .filter(|count| *count >= min_count)
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "draw target maximum must be min_count..255".into(),
                    })?;
                Ok(StmtIr::EffectChooseOptionalTargetsDraw {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    min_count,
                    max_count,
                    scope: expect_draw_target_scope(&args[4], span.start as usize)?,
                    span: *span,
                })
            }
            "reveal_target_hand_sha_or_discard_bind" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "reveal_target_hand_sha_or_discard_bind(player, target, helper_skill_def)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectRevealTargetHandShaOrDiscardBind {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "draw_remembered_hand_cards" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_remembered_hand_cards(player, count)".into(),
                    });
                }
                Ok(StmtIr::EffectDrawRememberedHandCards {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "discard_remembered_hand_cards" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_remembered_hand_cards(player)".into(),
                    });
                }
                Ok(StmtIr::EffectDiscardRememberedHandCards {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "gain_draw_cards_for_lifetime_card_action_divisors" => {
                if args.len() != 7 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_draw_cards_for_lifetime_card_action_divisors(player, divisor, group, divisor, group, divisor, group)".into(),
                    });
                }
                let mut divisors = [0_u32; 3];
                let mut groups = [0_u8; 3];
                for index in 0..3 {
                    let ExprAst::Int(divisor) = &args[index * 2 + 1] else {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "gain_draw_cards_for_lifetime_card_action_divisors expects positive integer divisors".into(),
                        });
                    };
                    if *divisor <= 0 {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "gain_draw_cards_for_lifetime_card_action_divisors divisors must be positive".into(),
                        });
                    }
                    let ExprAst::Ident(group) = &args[index * 2 + 2] else {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "gain_draw_cards_for_lifetime_card_action_divisors expects reward group identifiers".into(),
                        });
                    };
                    divisors[index] = *divisor as u32;
                    groups[index] = lifetime_card_reward_group_value(group).ok_or_else(|| {
                        CompileError::Type {
                            pos: span.start as usize,
                            message: format!("unknown lifetime card reward group `{group}`"),
                        }
                    })?;
                }
                Ok(StmtIr::EffectGainDrawCardsForLifetimeCardActionDivisors {
                    player: check_expr(&args[0], locals)?,
                    divisors,
                    groups,
                    span: *span,
                })
            }
            "lose_hp_gain_discard_card_for_opening_extra_play" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "lose_hp_gain_discard_card_for_opening_extra_play(player, card_kind, mandatory_skill_def)".into(),
                    });
                }
                let ExprAst::Ident(kind) = &args[1] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "lose_hp_gain_discard_card_for_opening_extra_play expects a card-kind name".into(),
                    });
                };
                let kind = card_kind_value(kind).ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: format!("unknown card kind {kind}"),
                })?;
                let mandatory_skill_def = expect_u32(&args[2], span.start as usize)?;
                if mandatory_skill_def == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "mandatory_skill_def must be a positive stable id".into(),
                    });
                }
                Ok(StmtIr::EffectLoseHpGainDiscardCardForOpeningExtraPlay {
                    player: check_expr(&args[0], locals)?,
                    kind,
                    mandatory_skill_def,
                    span: *span,
                })
            }
            "choose_gain_own_lost_discard_card_this_turn" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_gain_own_lost_discard_card_this_turn(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseGainOwnLostDiscardCardThisTurn {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_move_field_card_to" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_move_field_card_to(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseMoveFieldCardTo {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "draw_then_optionally_distribute_hand_cards" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "draw_then_optionally_distribute_hand_cards(player, draw_count, give_cap)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectDrawThenOptionallyDistributeHandCards {
                    player: check_expr(&args[0], locals)?,
                    draw_count: check_expr(&args[1], locals)?,
                    give_cap: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "give_selected_then_continue_distinct_recover_at_two" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "give_selected_then_continue_distinct_recover_at_two(player, target)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectGiveSelectedThenContinueDistinctRecoverAtTwo {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "give_selected_then_recover_target" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "give_selected_then_recover_target(player, target, amount)".into(),
                    });
                }
                Ok(StmtIr::EffectGiveSelectedThenRecoverTarget {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    amount: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "choose_other_give_then_recover" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_give_then_recover(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseOtherGiveThenRecover {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "guanxing_top_by_falu_marks" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "guanxing_top_by_falu_marks(player, spade, heart, club, diamond)"
                            .into(),
                    });
                }
                let mut parsed = [0u32; 4];
                for (slot, arg) in args[1..].iter().enumerate() {
                    parsed[slot] = expect_u32(arg, span.start as usize)?;
                }
                Ok(StmtIr::EffectGuanxingTopByFaluMarks {
                    player: check_expr(&args[0], locals)?,
                    marks: parsed,
                    span: *span,
                })
            }
            "guanxing_top" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "guanxing_top(player, count)".into(),
                    });
                }
                Ok(StmtIr::EffectGuanxingTop {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_hand_compare_between" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_hand_compare_between(player, first)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseHandCompareBetween {
                    player: check_expr(&args[0], locals)?,
                    first: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "give_selected_then_recover_self" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "give_selected_then_recover_self(player, target, amount)".into(),
                    });
                }
                Ok(StmtIr::EffectGiveSelectedThenRecoverSelf {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    amount: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_dingpan" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dingpan(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDingpan {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_tanlu" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_tanlu(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTanlu {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_max_mark_other_judge_kill_unless_tao_or_taoyuan" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(player, mark_def)"
                                .into(),
                    });
                }
                let mark_def = expect_u32(&args[1], span.start as usize)?;
                if mark_def == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "mark definition must be non-zero".into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
                        player: check_expr(&args[0], locals)?,
                        mark_def,
                        span: *span,
                    },
                )
            }
            "choose_zone_priority_gain_random_card_from_each_other_then_turn_over" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_zone_priority_gain_random_card_from_each_other_then_turn_over(player)"
                                .into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
                        player: check_expr(&args[0], locals)?,
                        span: *span,
                    },
                )
            }
            "choose_gain_card_from_each_other_then_turn_over" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_gain_card_from_each_other_then_turn_over(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseGainCardFromEachOtherThenTurnOver {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_hidden_skill_pile" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_hidden_skill_pile(player, pile_skill_def, action, card)"
                            .into(),
                    });
                }
                let pile_skill_def = expect_u32(&args[1], span.start as usize)?;
                let action = u8::try_from(expect_u32(&args[2], span.start as usize)?)
                    .ok()
                    .filter(|action| *action <= 4)
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "hidden skill-pile action must be 0..4".into(),
                    })?;
                if pile_skill_def == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "hidden skill-pile definition must be non-zero".into(),
                    });
                }
                Ok(StmtIr::EffectResolveHiddenSkillPile {
                    player: check_expr(&args[0], locals)?,
                    pile_skill_def,
                    action,
                    card: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "set_non_locked_skill_suppression" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "set_non_locked_skill_suppression(player, enabled)".into(),
                    });
                }
                let enabled = match &args[1] {
                    ExprAst::Int(0) => false,
                    ExprAst::Int(1) => true,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "non-locked skill suppression enabled must be 0 or 1".into(),
                        });
                    }
                };
                Ok(StmtIr::EffectSetNonLockedSkillSuppression {
                    player: check_expr(&args[0], locals)?,
                    enabled,
                    span: *span,
                })
            }
            "resolve_dili_yuheng" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dili_yuheng(player, action, card)".into(),
                    });
                }
                let action = match &args[1] {
                    ExprAst::Int(value) if (0..=u8::MAX as i32).contains(value) => *value as u8,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "Dili/Yuheng action must be a u8 literal".into(),
                        });
                    }
                };
                let card = check_expr(&args[2], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "Dili/Yuheng card must be a card operand".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDiliYuheng {
                    player: check_expr(&args[0], locals)?,
                    action,
                    card,
                    span: *span,
                })
            }
            "discard_selected_draw_all_hand_bonus" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_selected_draw_all_hand_bonus(player)".into(),
                    });
                }
                Ok(StmtIr::EffectDiscardSelectedDrawAllHandBonus {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_other_yinghun" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_yinghun(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseOtherYinghun {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_other_bizheng" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_bizheng(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseOtherBizheng {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_other_xiashu" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_xiashu(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseOtherXiashu {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_guanchao_rank" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_guanchao_rank(player, card)".into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "Guanchao card must be a card operand".into(),
                    });
                }
                Ok(StmtIr::EffectResolveGuanchaoRank {
                    player: check_expr(&args[0], locals)?,
                    card,
                    span: *span,
                })
            }
            "resolve_guanwei" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_guanwei(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveGuanwei {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_lanjiang" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_lanjiang(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveLanjiang {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_rebingyi" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_rebingyi(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveRebingyi {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_rebotu" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_rebotu(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveRebotu {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_oldimeng" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_oldimeng(player, 0|1)".into(),
                    });
                }
                let action = expect_u32(&args[1], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "oldimeng action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveOldimeng {
                    player: check_expr(&args[0], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_rejieyin" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_rejieyin(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveRejieyin {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_reanxu" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_reanxu(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveReanxu {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_drlt_jueyan" => {
                if args.len() != 6 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_drlt_jueyan(player, sha_skill_def, weapon_skill_def, mount_skill_def, armor_skill_def, treasure_skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDrltJueyan {
                    player: check_expr(&args[0], locals)?,
                    sha_skill_def: expect_u32(&args[1], span.start as usize)?,
                    weapon_skill_def: expect_u32(&args[2], span.start as usize)?,
                    mount_skill_def: expect_u32(&args[3], span.start as usize)?,
                    armor_skill_def: expect_u32(&args[4], span.start as usize)?,
                    treasure_skill_def: expect_u32(&args[5], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_rejizhi" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_rejizhi(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveRejizhi {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_jiexun" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jiexun(player, funan_helper_skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveJiexun {
                    player: check_expr(&args[0], locals)?,
                    funan_helper_skill_def: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_wengua_setup" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wengua_setup(player, helper_skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveWenguaSetup {
                    player: check_expr(&args[0], locals)?,
                    helper_skill_def: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_wengua" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_wengua(player, main_skill_def, marker_skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveWengua {
                    player: check_expr(&args[0], locals)?,
                    main_skill_def: expect_u32(&args[1], span.start as usize)?,
                    marker_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_xinanguo" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xinanguo(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXinanguo {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_ganglie_three" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ganglie_three(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveGanglieThree {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_zhongyi" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhongyi(player, action, helper_skill_def)".into(),
                    });
                }
                let action = expect_u32(&args[1], span.start as usize)?;
                if action > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhongyi action must be 0, 1, or 2".into(),
                    });
                }
                let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
                if helper_skill_def == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhongyi helper skill must be non-zero".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhongyi {
                    player: check_expr(&args[0], locals)?,
                    action: action as u8,
                    helper_skill_def,
                    span: *span,
                })
            }
            "discard_equipment_slot" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "discard_equipment_slot(player, weapon|armor|horse_def|horse_off|treasure)"
                                .into(),
                    });
                }
                let ExprAst::Ident(slot) = &args[1] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_equipment_slot expects an equipment-slot name".into(),
                    });
                };
                let slot = match slot.as_str() {
                    "weapon" => 1,
                    "armor" => 2,
                    "horse_def" => 3,
                    "horse_off" => 4,
                    "treasure" => 5,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: format!("unknown equipment slot {slot}"),
                        });
                    }
                };
                Ok(StmtIr::EffectDiscardEquipmentSlot {
                    player: check_expr(&args[0], locals)?,
                    slot,
                    span: *span,
                })
            }
            "resolve_oldanxu" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_oldanxu(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveOldAnxu {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "store_selected_in_skill_pile" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "store_selected_in_skill_pile(player)".into(),
                    });
                }
                Ok(StmtIr::EffectStoreSelectedInSkillPile {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "end_match_for_player_camp_if_skill_pile_at_least" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "end_match_for_player_camp_if_skill_pile_at_least(player, pile_skill_def, count)"
                                .into(),
                    });
                }
                let pile_skill_def = expect_u32(&args[1], span.start as usize)?;
                let count = expect_u32(&args[2], span.start as usize)?;
                if pile_skill_def == 0 || count == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "pile skill definition and count must be non-zero".into(),
                    });
                }
                Ok(StmtIr::EffectEndMatchForCampIfSkillPileAtLeast {
                    player: check_expr(&args[0], locals)?,
                    pile_skill_def,
                    count,
                    span: *span,
                })
            }
            "resolve_oldqinqing" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_oldqinqing(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveOldQinqing {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_oldmingjian" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_oldmingjian(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveOldMingjian {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_oldxuanfeng" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_oldxuanfeng(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveOldXuanfeng {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_jianrou" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jianrou(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveJianrou {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_oldhuisheng" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_oldhuisheng(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveOldHuisheng {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_oldjiefan" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_oldjiefan(player, stage)".into(),
                    });
                }
                let stage = expect_u32(&args[1], span.start as usize)?;
                if stage > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_oldjiefan stage must be 0..=2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveOldJiefan {
                    player: check_expr(&args[0], locals)?,
                    stage: stage as u8,
                    span: *span,
                })
            }
            "resolve_chegu" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_chegu(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveChegu {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_junkguixin" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_junkguixin(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveJunkguixin {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_yao_chenwei" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_yao_chenwei(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveYaoChenwei {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "add_skill_until_turn_end" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "add_skill_until_turn_end(player, skill_def)".into(),
                    });
                }
                let skill_def = expect_u32(&args[1], span.start as usize)?;
                if skill_def == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "skill definition must be non-zero".into(),
                    });
                }
                Ok(StmtIr::EffectAddSkillUntilTurnEnd {
                    player: check_expr(&args[0], locals)?,
                    skill_def,
                    span: *span,
                })
            }
            "resolve_mass_damage_discard_turn_over" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mass_damage_discard_turn_over(player, mark_def, mark_count, damage_amount, hand_count)".into(),
                    });
                }
                let mark_def = expect_u32(&args[1], span.start as usize)?;
                let mark_count = expect_u32(&args[2], span.start as usize)?;
                let damage_amount = expect_u32(&args[3], span.start as usize)?;
                let hand_count = expect_u32(&args[4], span.start as usize)?;
                if mark_def == 0 || mark_count == 0 || damage_amount == 0 || hand_count == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "mass damage/discard arguments must be non-zero".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMassDamageDiscardTurnOver {
                    player: check_expr(&args[0], locals)?,
                    mark_def,
                    mark_count,
                    damage_amount,
                    hand_count,
                    span: *span,
                })
            }
            "resolve_mass_damage_discard_hand_equip" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mass_damage_discard_hand_equip(player, mark_def, mark_count, damage_amount, discard_count)".into(),
                    });
                }
                let mark_def = expect_u32(&args[1], span.start as usize)?;
                let mark_count = expect_u32(&args[2], span.start as usize)?;
                let damage_amount = expect_u32(&args[3], span.start as usize)?;
                let discard_count = expect_u32(&args[4], span.start as usize)?;
                if mark_def == 0 || mark_count == 0 || damage_amount == 0 || discard_count == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "mass damage/discard arguments must be non-zero".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMassDamageDiscardHandEquip {
                    player: check_expr(&args[0], locals)?,
                    mark_def,
                    mark_count,
                    damage_amount,
                    discard_count,
                    span: *span,
                })
            }
            "record_active_target" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "record_active_target(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectRecordActiveTarget {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_discard_weapon_or_lose_hp" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_weapon_or_lose_hp(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardWeaponOrLoseHp {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_mark_parity_control" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_mark_parity_control(player, mark_def, mass_threshold, damage_amount)"
                                .into(),
                    });
                }
                let mark_def = expect_u32(&args[1], span.start as usize)?;
                let mass_threshold = expect_u32(&args[2], span.start as usize)?;
                let damage_amount = expect_u32(&args[3], span.start as usize)?;
                if mark_def == 0 || mass_threshold == 0 || damage_amount == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "mark parity control arguments must be non-zero".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMarkParityControl {
                    player: check_expr(&args[0], locals)?,
                    mark_def,
                    mass_threshold,
                    damage_amount,
                    span: *span,
                })
            }
            "resolve_selected_equipment_discard_then_choose_damage" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_selected_equipment_discard_then_choose_damage(player, damage_amount, normal|fire|thunder)"
                                .into(),
                    });
                }
                let damage_amount = expect_u32(&args[1], span.start as usize)?;
                if damage_amount == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "selected-target damage amount must be non-zero".into(),
                    });
                }
                let nature = match &args[2] {
                    ExprAst::Ident(name) if name == "normal" => 0,
                    ExprAst::Ident(name) if name == "fire" => 1,
                    ExprAst::Ident(name) if name == "thunder" => 2,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "damage nature must be normal, fire, or thunder".into(),
                        });
                    }
                };
                Ok(
                    StmtIr::EffectResolveSelectedEquipmentDiscardThenChooseDamage {
                        player: check_expr(&args[0], locals)?,
                        damage_amount,
                        nature,
                        span: *span,
                    },
                )
            }
            "replace_draw_with_distinct_suit_selection" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "replace_draw_with_distinct_suit_selection(player, count)".into(),
                    });
                }
                let count = u8::try_from(expect_u32(&args[1], span.start as usize)?)
                    .ok()
                    .filter(|count| *count > 0)
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "distinct-suit reveal count must be 1..255".into(),
                    })?;
                Ok(StmtIr::EffectReplaceDrawWithDistinctSuitSelection {
                    player: check_expr(&args[0], locals)?,
                    count,
                    span: *span,
                })
            }
            "inspect_target_hand_choose_heart_discard_or_top" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "inspect_target_hand_choose_heart_discard_or_top(player, target)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectInspectTargetHandChooseHeartDiscardOrTop {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_yeyan" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_yeyan(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveYeyan {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_all_lose_hp_or_recover" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_all_lose_hp_or_recover(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseAllLoseHpOrRecover {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_qixing" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qixing(player, initialize)".into(),
                    });
                }
                let initialize = match &args[1] {
                    ExprAst::Int(0) => false,
                    ExprAst::Int(1) => true,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "resolve_qixing initialize must be 0 or 1".into(),
                        });
                    }
                };
                Ok(StmtIr::EffectResolveQixing {
                    player: check_expr(&args[0], locals)?,
                    initialize,
                    span: *span,
                })
            }
            "resolve_qixing_weather" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qixing_weather(player, qixing_skill_def, helper_skill_def, max_targets)".into(),
                    });
                }
                let qixing_skill_def = expect_u32(&args[1], span.start as usize)?;
                let helper_skill_def = expect_u32(&args[2], span.start as usize)?;
                let max_targets = u8::try_from(expect_u32(&args[3], span.start as usize)?)
                    .ok()
                    .filter(|count| *count > 0)
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "Qixing weather max_targets must be 1..255".into(),
                    })?;
                Ok(StmtIr::EffectResolveQixingWeather {
                    player: check_expr(&args[0], locals)?,
                    qixing_skill_def,
                    helper_skill_def,
                    max_targets,
                    span: *span,
                })
            }
            "resolve_nzry_longnu" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_nzry_longnu(player, fire_skill_def, thunder_skill_def)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectResolveNzryLongnu {
                    player: check_expr(&args[0], locals)?,
                    fire_skill_def: expect_u32(&args[1], span.start as usize)?,
                    thunder_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "choose_link_other" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_link_other(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseLinkOther {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_drlt_poxi" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_drlt_poxi(player, target, hand_limit_skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDrltPoxi {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    hand_limit_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_drlt_jieying" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_drlt_jieying(player, event_player, mark_def, mode)"
                            .into(),
                    });
                }
                let mode = u8::try_from(expect_u32(&args[3], span.start as usize)?)
                    .ok()
                    .filter(|mode| *mode <= 2)
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "Jieying mode must be 0, 1, or 2".into(),
                    })?;
                Ok(StmtIr::EffectResolveDrltJieying {
                    player: check_expr(&args[0], locals)?,
                    event_player: check_expr(&args[1], locals)?,
                    mark: expect_u32(&args[2], span.start as usize)?,
                    mode,
                    span: *span,
                })
            }
            "resolve_drlt_duorui" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_drlt_duorui(player, target, helper_skill_def, stage)"
                            .into(),
                    });
                }
                let stage = u8::try_from(expect_u32(&args[3], span.start as usize)?)
                    .ok()
                    .filter(|stage| matches!(stage, 0 | 3))
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "Duorui package stage must be 0 or 3".into(),
                    })?;
                Ok(StmtIr::EffectResolveDrltDuorui {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                    stage,
                    span: *span,
                })
            }
            "resolve_drlt_zhiti" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_drlt_zhiti(player, event_player, event_source, event_amount, mode)"
                                .into(),
                    });
                }
                let mode = u8::try_from(expect_u32(&args[4], span.start as usize)?)
                    .ok()
                    .filter(|mode| *mode <= 2)
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "Zhiti mode must be 0, 1, or 2".into(),
                    })?;
                Ok(StmtIr::EffectResolveDrltZhiti {
                    player: check_expr(&args[0], locals)?,
                    event_player: check_expr(&args[1], locals)?,
                    event_source: check_expr(&args[2], locals)?,
                    event_amount: check_expr(&args[3], locals)?,
                    mode,
                    span: *span,
                })
            }
            "resolve_shen_simayi" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_shen_simayi(player, event_player, event_amount, mark, mode)"
                                .into(),
                    });
                }
                let mode = u8::try_from(expect_u32(&args[4], span.start as usize)?)
                    .ok()
                    .filter(|mode| *mode <= 4)
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "Shen Simayi mode must be 0..4".into(),
                    })?;
                Ok(StmtIr::EffectResolveShenSimayi {
                    player: check_expr(&args[0], locals)?,
                    event_player: check_expr(&args[1], locals)?,
                    event_amount: check_expr(&args[2], locals)?,
                    mark: expect_u32(&args[3], span.start as usize)?,
                    mode,
                    span: *span,
                })
            }
            "resolve_beige" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_beige(player, target, source)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveBeige {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    source: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_beige_configured" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_beige_configured(player, target, source, heart_recover_amount, diamond_draw_count)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveBeigeConfigured {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    source: check_expr(&args[2], locals)?,
                    heart_recover_amount: check_expr(&args[3], locals)?,
                    diamond_draw_count: check_expr(&args[4], locals)?,
                    span: *span,
                })
            }
            "resolve_zhuiyi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhuiyi(player, killer)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhuiyi {
                    player: check_expr(&args[0], locals)?,
                    killer: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_decadezhenjun" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_decadezhenjun(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDecadezhenjun {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_sp_xiaoguo" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_sp_xiaoguo(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveSpXiaoguo {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_gain_card_from_each_other_replace_draw" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_gain_card_from_each_other_replace_draw(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseGainCardFromEachOtherReplaceDraw {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_targets_damage" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_targets_damage(player, min_count, max_count, other|male_other|female_other|any|min_hand|same_hp|hand_empty|wounded|hand_lte_chooser, amount)"
                                .into(),
                    });
                }
                let min_count = u8::try_from(expect_int(&args[1], span.start as usize)?)
                    .ok()
                    .filter(|count| *count > 0)
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "damage target minimum must be 1..255".into(),
                    })?;
                let max_count = u8::try_from(expect_int(&args[2], span.start as usize)?)
                    .ok()
                    .filter(|count| *count >= min_count)
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "damage target maximum must be within min_count..255".into(),
                    })?;
                Ok(StmtIr::EffectChooseTargetsDamage {
                    player: check_expr(&args[0], locals)?,
                    min_count,
                    max_count,
                    scope: expect_draw_target_scope(&args[3], span.start as usize)?,
                    amount: check_expr(&args[4], locals)?,
                    span: *span,
                })
            }
            "draw_by_play_phase_card_category_count" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_by_play_phase_card_category_count(player)".into(),
                    });
                }
                Ok(StmtIr::EffectDrawByPlayPhaseCardCategoryCount {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_discard_one_for_sha_refund_and_draw_if_red" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_discard_one_for_sha_refund_and_draw_if_red(player, sha_user, card)"
                                .into(),
                    });
                }
                let card = check_expr(&args[2], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_discard_one_for_sha_refund_and_draw_if_red card must be a card operand"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardOneForShaRefundAndDrawIfRed {
                    player: check_expr(&args[0], locals)?,
                    sha_user: check_expr(&args[1], locals)?,
                    card,
                    span: *span,
                })
            }
            "choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank(player, sha_user, card, restore_skill_def)".into(),
                    });
                }
                let card = check_expr(&args[2], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank card must be a card operand".into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
                        player: check_expr(&args[0], locals)?,
                        sha_user: check_expr(&args[1], locals)?,
                        card,
                        restore_skill_def: expect_u32(&args[3], span.start as usize)?,
                        span: *span,
                    },
                )
            }
            "queue_opening_extra_play_phase" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "queue_opening_extra_play_phase(player)".into(),
                    });
                }
                Ok(StmtIr::EffectQueueOpeningExtraPlayPhase {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "convert_current_sha_to_fire_lose_hp_if_damage" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "convert_current_sha_to_fire_lose_hp_if_damage(player, card)"
                            .into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "convert-current-Sha card must be a card operand".into(),
                    });
                }
                Ok(StmtIr::EffectConvertCurrentShaToFireLoseHpIfDamage {
                    player: check_expr(&args[0], locals)?,
                    card,
                    span: *span,
                })
            }
            "choose_add_current_sha_target" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_add_current_sha_target(player, card)".into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose-add-current-Sha-target card must be a card operand".into(),
                    });
                }
                Ok(StmtIr::EffectChooseAddCurrentShaTarget {
                    player: check_expr(&args[0], locals)?,
                    card,
                    span: *span,
                })
            }
            "choose_add_current_sha_target_within_distance" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_add_current_sha_target_within_distance(player, card, max_distance)".into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "distance-capped additional-Sha-target card must be a card operand"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectChooseAddCurrentShaTargetWithinDistance {
                    player: check_expr(&args[0], locals)?,
                    card,
                    max_distance: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "refund_current_sha_use" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "refund_current_sha_use(player, card)".into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "refund-current-Sha-use card must be a card operand".into(),
                    });
                }
                Ok(StmtIr::EffectRefundCurrentShaUse {
                    player: check_expr(&args[0], locals)?,
                    card,
                    span: *span,
                })
            }
            "damage_then_draw_selected_targets" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "damage_then_draw_selected_targets(source, damage, draw)".into(),
                    });
                }
                Ok(StmtIr::EffectDamageThenDrawSelectedTargets {
                    source: check_expr(&args[0], locals)?,
                    damage: check_expr(&args[1], locals)?,
                    draw: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_youdi" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_youdi(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveYoudi {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_kuangfu" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_kuangfu(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveKuangfu {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_shanjia" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_shanjia(player, count)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveShanjia {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_new_yajiao" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_new_yajiao(player, card)".into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_new_yajiao second argument must be a card operand".into(),
                    });
                }
                Ok(StmtIr::EffectResolveNewYajiao {
                    player: check_expr(&args[0], locals)?,
                    card,
                    span: *span,
                })
            }
            "resolve_dingpin" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dingpin(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDingpin {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_repolu_targets_draw" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_repolu_targets_draw(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseRepoluTargetsDraw {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_xinfu_guolun" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xinfu_guolun(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXinfuGuolun {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_store_hand_sha_in_skill_pile" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_store_hand_sha_in_skill_pile(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseStoreHandShaInSkillPile {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_spend_skill_pile_for_dying_jiu" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_spend_skill_pile_for_dying_jiu(player, dying)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseSpendSkillPileForDyingJiu {
                    player: check_expr(&args[0], locals)?,
                    dying: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_compare_selected_hand_card" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_compare_selected_hand_card(player, target, win_skill_def, lose_skill_def)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectChooseCompareSelectedHandCard {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    win_skill_def: expect_u32(&args[2], span.start as usize)?,
                    lose_skill_def: expect_u32(&args[3], span.start as usize)?,
                    span: *span,
                })
            }
            "choose_add_skill_bound_card_target" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_add_skill_bound_card_target(player, card)".into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose-add-skill-bound-card-target card must be a card operand"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectChooseAddSkillBoundCardTarget {
                    player: check_expr(&args[0], locals)?,
                    card,
                    span: *span,
                })
            }
            "choose_discard_matching_equip_color_add_skill" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_matching_equip_color_add_skill(player, target, skill_def)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardMatchingEquipColorAddSkill {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "use_virtual_sha_from_card_color_lock_source_if_no_sha" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "use_virtual_sha_from_card_color_lock_source_if_no_sha(player)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectUseVirtualShaFromCardColorLockSourceIfNoSha {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_current_card_use_enhancements" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_current_card_use_enhancements(player, card, helper_skill_def)"
                                .into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "card-use-enhancement card must be a card operand".into(),
                    });
                }
                Ok(StmtIr::EffectChooseCurrentCardUseEnhancements {
                    player: check_expr(&args[0], locals)?,
                    card,
                    helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "make_current_card_unrespondable" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "make_current_card_unrespondable(player, card, helper_skill_def)"
                            .into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "current-card unrespondable card must be a card operand".into(),
                    });
                }
                Ok(StmtIr::EffectMakeCurrentCardUnrespondable {
                    player: check_expr(&args[0], locals)?,
                    card,
                    helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "choose_draw_or_discard_add_skill" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_draw_or_discard_add_skill(player, draw_skill_def, discard_skill_def)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectChooseDrawOrDiscardAddSkill {
                    player: check_expr(&args[0], locals)?,
                    draw_skill_def: expect_u32(&args[1], span.start as usize)?,
                    discard_skill_def: expect_u32(&args[2], span.start as usize)?,
                    three_way: false,
                    span: *span,
                })
            }
            "choose_three_way_draw_or_discard_add_skill" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_three_way_draw_or_discard_add_skill(player, draw_skill_def, discard_skill_def)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectChooseDrawOrDiscardAddSkill {
                    player: check_expr(&args[0], locals)?,
                    draw_skill_def: expect_u32(&args[1], span.start as usize)?,
                    discard_skill_def: expect_u32(&args[2], span.start as usize)?,
                    three_way: true,
                    span: *span,
                })
            }
            "choose_discarded_black_basic_as_bingliang" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discarded_black_basic_as_bingliang(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardedBlackBasicAsBingliang {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_discard_judge_use_virtual_sha_draw_if_no_damage" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_discard_judge_use_virtual_sha_draw_if_no_damage(player, target)"
                                .into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
                        player: check_expr(&args[0], locals)?,
                        target: check_expr(&args[1], locals)?,
                        span: *span,
                    },
                )
            }
            "discard_selected_record_unique_category_target" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_selected_record_unique_category_target(player, target)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectDiscardSelectedRecordUniqueCategoryTarget {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            name @ ("choose_target_draw_or_discard_by_skill_use_count_link_if_wounded"
            | "choose_target_draw_or_discard_by_skill_use_count_toggle_link_if_wounded") => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: format!("{name}(player, target)"),
                    });
                }
                Ok(
                    StmtIr::EffectChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
                        player: check_expr(&args[0], locals)?,
                        target: check_expr(&args[1], locals)?,
                        toggle_link: name.contains("_toggle_link_"),
                        span: *span,
                    },
                )
            }
            "choose_reveal_other_hand_card_remember_category" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_reveal_other_hand_card_remember_category(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseRevealOtherHandCardRememberCategory {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "draw_then_give_exact_record_phase_target" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_then_give_exact_record_phase_target(player, target, count)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectDrawThenGiveExactRecordPhaseTarget {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    count: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "clear_phase_gift_target" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "clear_phase_gift_target(player)".into(),
                    });
                }
                Ok(StmtIr::EffectClearPhaseGiftTarget {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_gain_resolved_sha_or_shan_then_optional_use_sha" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_gain_resolved_sha_or_shan_then_optional_use_sha(player, sha_card)"
                                .into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseGainResolvedShaOrShanThenOptionalUseSha {
                        player: check_expr(&args[0], locals)?,
                        sha_card: check_expr(&args[1], locals)?,
                        span: *span,
                    },
                )
            }
            "choose_gain_all_resolved_sha_shan_then_color_rewards" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_gain_all_resolved_sha_shan_then_color_rewards(player, sha_card)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectChooseGainAllResolvedShaShanThenColorRewards {
                    player: check_expr(&args[0], locals)?,
                    sha_card: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "skip_current_phase" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "skip_current_phase(player)".into(),
                    });
                }
                Ok(StmtIr::EffectSkipCurrentPhase {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "finish_current_phase_abort_resolution" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "finish_current_phase_abort_resolution(player)".into(),
                    });
                }
                Ok(StmtIr::EffectFinishCurrentPhaseAbortResolution {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "gain_top_card_to_skill_pile" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_top_card_to_skill_pile(player)".into(),
                    });
                }
                Ok(StmtIr::EffectGainTopCardToSkillPile {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_discard_skill_pile_card_add_marked_skill" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_discard_skill_pile_card_add_marked_skill(player, target, helper_skill_def)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardSkillPileCardAddMarkedSkill {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "gain_one_hand_from_selected_targets" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_one_hand_from_selected_targets(player)".into(),
                    });
                }
                Ok(StmtIr::EffectGainOneHandFromSelectedTargets {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_other_targets_by_skipped_phase_count_damage" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_other_targets_by_skipped_phase_count_damage(player, amount)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectChooseOtherTargetsBySkippedPhaseCountDamage {
                    player: check_expr(&args[0], locals)?,
                    amount: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "reveal_uniform_hand_draw_self_choose_others" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_uniform_hand_draw_self_choose_others(player)".into(),
                    });
                }
                Ok(StmtIr::EffectRevealUniformHandDrawSelfChooseOthers {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_card_user_or_add_target" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_card_user_or_add_target(player, card)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseCardUserOrAddTarget {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_discard_equipment_reduce_current_damage" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_equipment_reduce_current_damage(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardEquipmentReduceCurrentDamage {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_discard_equipment_prevent_current_damage" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_equipment_prevent_current_damage(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardEquipmentPreventCurrentDamage {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_move_target_equipment_or_draw" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_move_target_equipment_or_draw(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseMoveTargetEquipmentOrDraw {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_move_target_equipment_and_or_draw" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_move_target_equipment_and_or_draw(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseMoveTargetEquipmentAndOrDraw {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(player, target)".into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
                        player: check_expr(&args[0], locals)?,
                        target: check_expr(&args[1], locals)?,
                        span: *span,
                    },
                )
            }
            "choose_draw_or_recast_add_skill" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_draw_or_recast_add_skill(player, draw_skill_def, recast_skill_def)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectChooseDrawOrRecastAddSkill {
                    player: check_expr(&args[0], locals)?,
                    draw_skill_def: expect_u32(&args[1], span.start as usize)?,
                    recast_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "choose_gain_sha_for_opening_extra_play" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_gain_sha_for_opening_extra_play(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseGainShaForOpeningExtraPlay {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "finish_opening_extra_play_sha_gain" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "finish_opening_extra_play_sha_gain(player)".into(),
                    });
                }
                Ok(StmtIr::EffectFinishOpeningExtraPlayShaGain {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "reveal_target_hand_gain_shan_armor_or_draw_two" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_target_hand_gain_shan_armor_or_draw_two(player, target)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectRevealTargetHandGainShanArmorOrDrawTwo {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "give_all_equipment_choose_recover_or_damage" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "give_all_equipment_choose_recover_or_damage(player, target)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectGiveAllEquipmentChooseRecoverOrDamage {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_target_discard_weapon_range_hand_or_increase_damage" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_target_discard_weapon_range_hand_or_increase_damage(player, target)"
                                .into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
                        player: check_expr(&args[0], locals)?,
                        target: check_expr(&args[1], locals)?,
                        span: *span,
                    },
                )
            }
            "choose_hand_count_different_target_draw_or_random_discard" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_hand_count_different_target_draw_or_random_discard(player)"
                                .into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseHandCountDifferentTargetDrawOrRandomDiscard {
                        player: check_expr(&args[0], locals)?,
                        span: *span,
                    },
                )
            }
            "increase_current_draw_count" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "increase_current_draw_count(player, count)".into(),
                    });
                }
                Ok(StmtIr::EffectIncreaseCurrentDrawCount {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_target_discard_excess_then_self_discard_or_target_draw" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_target_discard_excess_then_self_discard_or_target_draw(player)"
                                .into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
                        player: check_expr(&args[0], locals)?,
                        span: *span,
                    },
                )
            }
            "draw_then_choose_gain_one_from_damage_target_and_neighbors" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "draw_then_choose_gain_one_from_damage_target_and_neighbors(player, target)"
                                .into(),
                    });
                }
                Ok(
                    StmtIr::EffectDrawThenChooseGainOneFromDamageTargetAndNeighbors {
                        player: check_expr(&args[0], locals)?,
                        target: check_expr(&args[1], locals)?,
                        span: *span,
                    },
                )
            }
            "choose_prevent_damage_or_lose_max_hp_increase_damage" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_prevent_damage_or_lose_max_hp_increase_damage(player, amount)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectChoosePreventDamageOrLoseMaxHpIncreaseDamage {
                    player: check_expr(&args[0], locals)?,
                    amount: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "randomize_current_hand_compare_opponent" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "randomize_current_hand_compare_opponent(player)".into(),
                    });
                }
                Ok(StmtIr::EffectRandomizeCurrentHandCompareOpponent {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "gain_highest_rank_sha_from_event_cards" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_highest_rank_sha_from_event_cards(player)".into(),
                    });
                }
                Ok(StmtIr::EffectGainHighestRankShaFromEventCards {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "gain_hand_compare_result_card" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_hand_compare_result_card(player)".into(),
                    });
                }
                Ok(StmtIr::EffectGainHandCompareResultCard {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_add_or_remove_current_card_target" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_add_or_remove_current_card_target(player, card)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseAddOrRemoveCurrentCardTarget {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_hand_compare" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_hand_compare(player, win_skill_def, lose_skill_def)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectChooseHandCompare {
                    player: check_expr(&args[0], locals)?,
                    win_skill_def: expect_u32(&args[1], span.start as usize)?,
                    lose_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "choose_hand_compare_grant_target_on_win" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_hand_compare_grant_target_on_win(player, win_grant_def, lose_skill_def)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectChooseHandCompareGrantTargetOnWin {
                    player: check_expr(&args[0], locals)?,
                    win_grant_def: expect_u32(&args[1], span.start as usize)?,
                    lose_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_lieren" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_lieren(player, target, win_skill_def, lose_skill_def)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectResolveLieren {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    win_skill_def: expect_u32(&args[2], span.start as usize)?,
                    lose_skill_def: expect_u32(&args[3], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_tuntian" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_tuntian(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTuntian {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_zaoxian" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zaoxian(player, jixi_skill_def, tuntian_skill_def)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectResolveZaoxian {
                    player: check_expr(&args[0], locals)?,
                    jixi_def: expect_u32(&args[1], span.start as usize)?,
                    tuntian_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_tiaoxin" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_tiaoxin(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTiaoxin {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_zhiji" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhiji(player, guanxing_skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhiji {
                    player: check_expr(&args[0], locals)?,
                    guanxing_def: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_zaiqi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zaiqi(player, extra)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZaiqi {
                    player: check_expr(&args[0], locals)?,
                    extra: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_twqiushou" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_twqiushou(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveTwqiushou {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_zhengrong" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhengrong(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhengrong {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_hongju" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_hongju(player, qingjian_skill_def, rong_skill_def)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectResolveHongju {
                    player: check_expr(&args[0], locals)?,
                    qingjian_def: expect_u32(&args[1], span.start as usize)?,
                    rong_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_qingjian" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qingjian(player, rong_skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveQingjian {
                    player: check_expr(&args[0], locals)?,
                    rong_def: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_releiji" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_releiji(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveReleiji {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_huangtian" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_huangtian(player, actor)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveHuangtian {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_haoshi" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_haoshi(player, stage)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveHaoshi {
                    player: check_expr(&args[0], locals)?,
                    stage: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_dimeng" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dimeng(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDimeng {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_qiaobian" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qiaobian(player, phase)".into(),
                    });
                }
                let phase = expect_u32(&args[1], span.start as usize)?;
                if phase > 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qiaobian phase must be 0..=3".into(),
                    });
                }
                Ok(StmtIr::EffectResolveQiaobian {
                    player: check_expr(&args[0], locals)?,
                    phase: phase as u8,
                    span: *span,
                })
            }
            "exclude_current_sha_target" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "exclude_current_sha_target(card, target)".into(),
                    });
                }
                Ok(StmtIr::EffectExcludeSavedShaTarget {
                    card: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_xiangle" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_xiangle(player, source, card)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveXiangle {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_fangquan_skip" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_fangquan_skip(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveFangquanSkip {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_fangquan_grant" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_fangquan_grant(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveFangquanGrant {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_ruoyu" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_ruoyu(player, jijiang_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveRuoyu {
                    player: check_expr(&args[0], locals)?,
                    jijiang_def: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_zuilun" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zuilun(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZuilun {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_fuyin" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_fuyin(player, source, card)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveFuyin {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "ban_use_against" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "ban_use_against(user, target)".into(),
                    });
                }
                Ok(StmtIr::EffectBanUseAgainst {
                    user: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "resolve_juzhan" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_juzhan(player, source, card)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveJuzhan {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_qizhi" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_qizhi(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveQizhi {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_jinqu" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jinqu(player, qizhi_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveJinqu {
                    player: check_expr(&args[0], locals)?,
                    qizhi_def: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_jianxiang" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_jianxiang(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveJianxiang {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_shenshi" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_shenshi(player, source, stage)".into(),
                    });
                }
                let stage = expect_u32(&args[2], span.start as usize)?;
                if !matches!(stage, 0 | 1 | 4) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_shenshi stage must be 0, 1, or 4".into(),
                    });
                }
                Ok(StmtIr::EffectResolveShenshi {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    stage: stage as u8,
                    span: *span,
                })
            }
            "resolve_mingren" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mingren(player, stage)".into(),
                    });
                }
                let stage = expect_u32(&args[1], span.start as usize)?;
                if !matches!(stage, 0 | 2) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mingren stage must be 0 or 2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMingren {
                    player: check_expr(&args[0], locals)?,
                    stage: stage as u8,
                    span: *span,
                })
            }
            "resolve_zhenliang" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhenliang(player, source, card, mingren_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhenliang {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    mingren_def: expect_u32(&args[3], span.start as usize)?,
                    stage: 0,
                    span: *span,
                })
            }
            "resolve_kuizhu" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_kuizhu(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveKuizhu {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_zhizheng" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_zhizheng(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveZhizheng {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_lijun" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_lijun(player, source, card)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveLijun {
                    player: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "resolve_liangyin" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_liangyin(player, stage, kongsheng_def)".into(),
                    });
                }
                let stage = expect_u32(&args[1], span.start as usize)?;
                if !matches!(stage, 0 | 3) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_liangyin stage must be 0 or 3".into(),
                    });
                }
                Ok(StmtIr::EffectResolveLiangyin {
                    player: check_expr(&args[0], locals)?,
                    kongsheng_def: expect_u32(&args[2], span.start as usize)?,
                    stage: stage as u8,
                    span: *span,
                })
            }
            "resolve_reliangyin" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_reliangyin(player, self_skill, mode)".into(),
                    });
                }
                if !matches!(&args[1], ExprAst::Ident(name) if name == "self_skill") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_reliangyin must reference self_skill".into(),
                    });
                }
                let mode = expect_u32(&args[2], span.start as usize)?;
                Ok(StmtIr::EffectResolveReliangyin {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    mode,
                    span: *span,
                })
            }
            "resolve_kongsheng" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_kongsheng(player, stage)".into(),
                    });
                }
                let stage = expect_u32(&args[1], span.start as usize)?;
                if !matches!(stage, 0 | 2) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_kongsheng stage must be 0 or 2".into(),
                    });
                }
                Ok(StmtIr::EffectResolveKongsheng {
                    player: check_expr(&args[0], locals)?,
                    stage: stage as u8,
                    span: *span,
                })
            }
            "resolve_pottuntian" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_pottuntian(player, action, zaoxian_def)".into(),
                    });
                }
                let action = expect_u32(&args[1], span.start as usize)?;
                if action > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_pottuntian action must be 0, 1, or 2".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePottuntian {
                    player: check_expr(&args[0], locals)?,
                    action: action as u8,
                    zaoxian_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_potjixi" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potjixi(player, target, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potjixi action must be 0, 1, or 2".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotjixi {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_potceduan" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potceduan(player, target, action)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potceduan action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotceduan {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_pottaishici" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_pottaishici(player, target, action, hanzhan_def, zhanlie_def)"
                                .into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_pottaishici action must be 0 through 5".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePottaishici {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    action: action as u8,
                    hanzhan_def: expect_u32(&args[3], span.start as usize)?,
                    zhanlie_def: expect_u32(&args[4], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_mbzhonghui" => {
                if args.len() != 6 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbzhonghui(player, target, action, amount, mbsizi_def, mbkechang_def)".into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 7 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_mbzhonghui action must be 0 through 7".into(),
                    });
                }
                Ok(StmtIr::EffectResolveMbzhonghui {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    action: action as u8,
                    amount: check_expr(&args[3], locals)?,
                    mbsizi_def: expect_u32(&args[4], span.start as usize)?,
                    mbkechang_def: expect_u32(&args[5], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_potjiejie" => {
                if args.len() != 8 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potjiejie(player, actor, used_def, spade_def, heart_def, club_def, diamond_def, qingshi_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotjiejie {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    used_def: expect_u32(&args[2], span.start as usize)?,
                    suit_defs: [
                        expect_u32(&args[3], span.start as usize)?,
                        expect_u32(&args[4], span.start as usize)?,
                        expect_u32(&args[5], span.start as usize)?,
                        expect_u32(&args[6], span.start as usize)?,
                    ],
                    qingshi_def: expect_u32(&args[7], span.start as usize)?,
                    span: *span,
                })
            }
            "resolve_potqingshi" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potqingshi(player)".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotqingshi {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "resolve_potweiyan" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potweiyan(player, target, card, action)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 10 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potweiyan action must be 0 through 10".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotweiyan {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_potzhouyu" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potzhouyu(player, target, card, action)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 7 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potzhouyu action must be 0 through 7".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotzhouyu {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_potlvyi" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potlvyi(player, target, card, action)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 9 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potlvyi action must be 0 through 9".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotlvyi {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_potcaoshuang" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potcaoshuang(player, target, card, action)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 11 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potcaoshuang action must be 0 through 11".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotcaoshuang {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_potyuji" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potyuji(player, target, card, action)".into(),
                    });
                }
                let action = expect_u32(&args[3], span.start as usize)?;
                if action > 8 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_potyuji action must be 0 through 8".into(),
                    });
                }
                Ok(StmtIr::EffectResolvePotyuji {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    card: check_expr(&args[2], locals)?,
                    action: action as u8,
                    span: *span,
                })
            }
            "resolve_rehengjiang" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "resolve_rehengjiang(player, actor, action, hand_limit_def, draw_def)"
                                .into(),
                    });
                }
                let action = expect_u32(&args[2], span.start as usize)?;
                if action > 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_rehengjiang action must be 0 or 1".into(),
                    });
                }
                Ok(StmtIr::EffectResolveRehengjiang {
                    player: check_expr(&args[0], locals)?,
                    actor: check_expr(&args[1], locals)?,
                    action: action as u8,
                    hand_limit_def: expect_u32(&args[3], span.start as usize)?,
                    draw_def: expect_u32(&args[4], span.start as usize)?,
                    span: *span,
                })
            }
            "choose_other_damage" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_damage(player, amount)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseOtherDamage {
                    player: check_expr(&args[0], locals)?,
                    amount: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_other_discard_target_card" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_discard_target_card(player, equip|hand_equip|hand_equip_judge|equip_judge)".into(),
                    });
                }
                let ExprAst::Ident(scope) = &args[1] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_discard_target_card scope must be equip, hand_equip, hand_equip_judge, or equip_judge".into(),
                    });
                };
                let scope = match scope.as_str() {
                    "equip" => TargetCardScopeIr::Equip,
                    "hand_equip" => TargetCardScopeIr::HandEquip,
                    "hand_equip_judge" => TargetCardScopeIr::HandEquipJudge,
                    "equip_judge" => TargetCardScopeIr::EquipJudge,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "choose_other_discard_target_card scope must be equip, hand_equip, hand_equip_judge, or equip_judge".into(),
                        });
                    }
                };
                Ok(StmtIr::EffectChooseOtherDiscardTargetCard {
                    player: check_expr(&args[0], locals)?,
                    scope,
                    span: *span,
                })
            }
            "discard_all_hand_and_equip" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_all_hand_and_equip(player)".into(),
                    });
                }
                Ok(StmtIr::EffectDiscardAllHandAndEquip {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "remove_all_skills" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "remove_all_skills(player)".into(),
                    });
                }
                Ok(StmtIr::EffectRemoveAllSkills {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "judge_gain_equipment_by_color" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judge_gain_equipment_by_color(player)".into(),
                    });
                }
                Ok(StmtIr::EffectJudgeGainEquipmentByColor {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "judge_recover_on_suit" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judge_recover_on_suit(player, spade|heart|club|diamond, amount)"
                            .into(),
                    });
                }
                let suit = match &args[1] {
                    ExprAst::Ident(name) if name == "spade" => 1,
                    ExprAst::Ident(name) if name == "heart" => 2,
                    ExprAst::Ident(name) if name == "club" => 3,
                    ExprAst::Ident(name) if name == "diamond" => 4,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "judgment suit must be spade, heart, club, or diamond".into(),
                        });
                    }
                };
                Ok(StmtIr::EffectJudgeRecoverOnSuit {
                    player: check_expr(&args[0], locals)?,
                    suit,
                    amount: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "judge_recover_other_on_suit" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judge_recover_other_on_suit(judge_player, recover_player, spade|heart|club|diamond, amount)"
                            .into(),
                    });
                }
                let suit = match &args[2] {
                    ExprAst::Ident(name) if name == "spade" => 1,
                    ExprAst::Ident(name) if name == "heart" => 2,
                    ExprAst::Ident(name) if name == "club" => 3,
                    ExprAst::Ident(name) if name == "diamond" => 4,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "judgment suit must be spade, heart, club, or diamond".into(),
                        });
                    }
                };
                Ok(StmtIr::EffectJudgeRecoverOtherOnSuit {
                    judge_player: check_expr(&args[0], locals)?,
                    recover_player: check_expr(&args[1], locals)?,
                    suit,
                    amount: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "force_others_use_sha_on_nearest_or_lose_hp" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "force_others_use_sha_on_nearest_or_lose_hp(player)".into(),
                    });
                }
                Ok(StmtIr::EffectForceOthersUseShaOnNearestOrLoseHp {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "judge_damage_other_on_suit" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judge_damage_other_on_suit(player, spade|heart|club|diamond, amount, normal|fire|thunder)"
                            .into(),
                    });
                }
                let suit = match &args[1] {
                    ExprAst::Ident(name) if name == "spade" => 1,
                    ExprAst::Ident(name) if name == "heart" => 2,
                    ExprAst::Ident(name) if name == "club" => 3,
                    ExprAst::Ident(name) if name == "diamond" => 4,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "judgment suit must be spade, heart, club, or diamond".into(),
                        });
                    }
                };
                let nature = match &args[3] {
                    ExprAst::Ident(name) if name == "normal" => 0,
                    ExprAst::Ident(name) if name == "fire" => 1,
                    ExprAst::Ident(name) if name == "thunder" => 2,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "damage nature must be normal, fire, or thunder".into(),
                        });
                    }
                };
                Ok(StmtIr::EffectJudgeDamageOtherOnSuit {
                    player: check_expr(&args[0], locals)?,
                    suit,
                    amount: check_expr(&args[2], locals)?,
                    nature,
                    span: *span,
                })
            }
            "reveal_discard_suit_gain_or_use" => {
                check_reveal_discard_suit_gain_or_use(args, *span, locals)
            }
            "choose_discard_or_fire_damage_skip_phases" => {
                check_choose_discard_or_fire_damage_skip_phases(args, *span, locals)
            }
            "draw_selected" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_selected(player)".into(),
                    });
                }
                Ok(StmtIr::EffectDrawSelected {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "draw_each_by_seat" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_each_by_seat(first, second, count)".into(),
                    });
                }
                Ok(StmtIr::EffectDrawEachBySeat {
                    first: check_expr(&args[0], locals)?,
                    second: check_expr(&args[1], locals)?,
                    count: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "draw_then_give_one" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_then_give_one(player, target, count, any_hand_equip|drawn_only|any_hand_equip_use_equipment)".into(),
                    });
                }
                let ExprAst::Ident(scope) = &args[3] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_then_give_one scope must be any_hand_equip, drawn_only, or any_hand_equip_use_equipment".into(),
                    });
                };
                let (drawn_only, use_given_equipment) = match scope.as_str() {
                    "any_hand_equip" => (false, false),
                    "drawn_only" => (true, false),
                    "any_hand_equip_use_equipment" => (false, true),
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "draw_then_give_one scope must be any_hand_equip, drawn_only, or any_hand_equip_use_equipment".into(),
                        })
                    }
                };
                Ok(StmtIr::EffectDrawThenGiveOne {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    count: check_expr(&args[2], locals)?,
                    drawn_only,
                    use_given_equipment,
                    span: *span,
                })
            }
            "draw_then_take_one" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_then_take_one(player, taker, count)".into(),
                    });
                }
                Ok(StmtIr::EffectDrawThenTakeOne {
                    player: check_expr(&args[0], locals)?,
                    taker: check_expr(&args[1], locals)?,
                    count: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "take_one_card" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "take_one_card(from, to, hand_equip|equip|hand_equip_judge)"
                            .into(),
                    });
                }
                let ExprAst::Ident(scope) = &args[2] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "take_one_card scope must be hand_equip, equip, or hand_equip_judge"
                                .into(),
                    });
                };
                if scope == "hand_equip_judge" {
                    return Ok(StmtIr::EffectTakeOneScopedCard {
                        from: check_expr(&args[0], locals)?,
                        to: check_expr(&args[1], locals)?,
                        scope: TargetCardScopeIr::HandEquipJudge,
                        span: *span,
                    });
                }
                let equip_only =
                    match scope.as_str() {
                        "hand_equip" => false,
                        "equip" => true,
                        _ => return Err(CompileError::Type {
                            pos: span.start as usize,
                            message:
                                "take_one_card scope must be hand_equip, equip, or hand_equip_judge"
                                    .into(),
                        }),
                    };
                Ok(StmtIr::EffectTakeOneCard {
                    from: check_expr(&args[0], locals)?,
                    to: check_expr(&args[1], locals)?,
                    equip_only,
                    span: *span,
                })
            }
            "take_one_event_target_hand" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "take_one_event_target_hand(player)".into(),
                    });
                }
                Ok(StmtIr::EffectTakeOneHandCard {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_other_hand_store_skill_pile" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_hand_store_skill_pile(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseOtherHandStoreSkillPile {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "gain_all_skill_pile_cards" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_all_skill_pile_cards(player)".into(),
                    });
                }
                Ok(StmtIr::EffectGainAllSkillPileCards {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "gain_other_skill_pile_cards" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_other_skill_pile_cards(player, target, skill_def)".into(),
                    });
                }
                let skill_def = expect_u32(&args[2], span.start as usize)?;
                if skill_def == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_other_skill_pile_cards skill_def must be non-zero".into(),
                    });
                }
                Ok(StmtIr::EffectGainOtherSkillPileCards {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    skill_def,
                    span: *span,
                })
            }
            "choose_discard_one_he_or_lose_hp" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_one_he_or_lose_hp(player, lose_amount)".into(),
                    });
                }
                let lose_amount = expect_u32(&args[1], span.start as usize)?;
                if lose_amount == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_one_he_or_lose_hp lose_amount must be non-zero".into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardOneHeOrLoseHp {
                    player: check_expr(&args[0], locals)?,
                    lose_amount,
                    span: *span,
                })
            }
            "judge_spade_two_to_nine_draw_two_or_thunder_sha_twice" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "judge_spade_two_to_nine_draw_two_or_thunder_sha_twice(source, target)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectJudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
                    source: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_source_confirm_owner_lose_hp" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_source_confirm_owner_lose_hp(owner, source, amount)".into(),
                    });
                }
                let amount = expect_u32(&args[2], span.start as usize)?;
                if amount == 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_source_confirm_owner_lose_hp amount must be non-zero".into(),
                    });
                }
                Ok(StmtIr::EffectChooseSourceConfirmOwnerLoseHp {
                    owner: check_expr(&args[0], locals)?,
                    source: check_expr(&args[1], locals)?,
                    amount,
                    span: *span,
                })
            }
            "choose_end_extra_draw_or_play_by_used_suits" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_end_extra_draw_or_play_by_used_suits(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseEndExtraDrawOrPlayByUsedSuits {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "lose_hp_double_current_damage_then_replace" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "lose_hp_double_current_damage_then_replace(player, replacement_skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectLoseHpDoubleCurrentDamageThenReplace {
                    player: check_expr(&args[0], locals)?,
                    replacement_skill_def: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "take_target_hand_then_repeat_until_max" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "take_target_hand_then_repeat_until_max(player, target, skill)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectTakeTargetHandThenRepeatUntilMax {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "gain_event_cards" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_event_cards(player)".into(),
                    });
                }
                Ok(StmtIr::EffectGainEventCards {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "choose_target_draw_to_max_hp_cap" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_target_draw_to_max_hp_cap(player, cap)".into(),
                    });
                }
                let cap = u32::try_from(expect_int(&args[1], span.start as usize)?)
                    .ok()
                    .filter(|cap| *cap > 0)
                    .ok_or_else(|| CompileError::Type {
                        pos: span.start as usize,
                        message: "maximum-HP draw cap must be a positive integer".into(),
                    })?;
                Ok(StmtIr::EffectChooseTargetDrawToMaxHpCap {
                    player: check_expr(&args[0], locals)?,
                    cap,
                    span: *span,
                })
            }
            "choose_other_draw_owner_lost_hp_turn_over" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_other_draw_owner_lost_hp_turn_over(player)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseOtherDrawOwnerLostHpTurnOver {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "turn_over" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "turn_over(player)".into(),
                    });
                }
                Ok(StmtIr::EffectTurnOver {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "restore_character_state" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "restore_character_state(player)".into(),
                    });
                }
                Ok(StmtIr::EffectRestoreCharacterState {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "discard_all_hand_equip_judge" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "discard_all_hand_equip_judge(player)".into(),
                    });
                }
                Ok(StmtIr::EffectDiscardAllHandEquipJudge {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "damage_then_discard_all_judge" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "damage_then_discard_all_judge(player)".into(),
                    });
                }
                Ok(StmtIr::EffectDamageThenDiscardAllJudge {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "replace_draw_gain_top_two_add_skills_if_different_colors" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "replace_draw_gain_top_two_add_skills_if_different_colors(player, first_skill_def, second_skill_def)"
                                .into(),
                    });
                }
                Ok(
                    StmtIr::EffectReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
                        player: check_expr(&args[0], locals)?,
                        first_skill_def: expect_u32(&args[1], span.start as usize)?,
                        second_skill_def: expect_u32(&args[2], span.start as usize)?,
                        span: *span,
                    },
                )
            }
            "choose_gain_target_card_alternate_zone_then_target_draw" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_gain_target_card_alternate_zone_then_target_draw(player, target)"
                                .into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseGainTargetCardAlternateZoneThenTargetDraw {
                        player: check_expr(&args[0], locals)?,
                        target: check_expr(&args[1], locals)?,
                        span: *span,
                    },
                )
            }
            "choose_discard_target_card_resolve_current_sha" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_discard_target_card_resolve_current_sha(source, card, target)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardTargetCardResolveCurrentSha {
                    source: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    target: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "choose_discard_target_card_resolve_current_sha_nonbasic_bonus" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_discard_target_card_resolve_current_sha_nonbasic_bonus(source, card, target, bonus_skill_def)"
                                .into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
                        source: check_expr(&args[0], locals)?,
                        card: check_expr(&args[1], locals)?,
                        target: check_expr(&args[2], locals)?,
                        bonus_skill_def: expect_u32(&args[3], span.start as usize)?,
                        span: *span,
                    },
                )
            }
            "resolve_dcqiaomeng" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dcqiaomeng(player, card, helper_skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDcqiaomeng {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    helper_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "remove_skill_by_def" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "remove_skill_by_def(player, skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectRemoveSkillByDef {
                    player: check_expr(&args[0], locals)?,
                    skill_def: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "choose_discard_hand_add_skill_by_color" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_hand_add_skill_by_color(player, red_skill_def, black_skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardHandAddSkillByColor {
                    player: check_expr(&args[0], locals)?,
                    red_skill_def: expect_u32(&args[1], span.start as usize)?,
                    black_skill_def: expect_u32(&args[2], span.start as usize)?,
                    span: *span,
                })
            }
            "reveal_top_gain_non_basic_or_swap_basic" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_top_gain_non_basic_or_swap_basic(player)".into(),
                    });
                }
                Ok(StmtIr::EffectRevealTopGainNonBasicOrSwapBasic {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "reveal_top_gain_if_color" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_top_gain_if_color(player, red|black)".into(),
                    });
                }
                let color_mask = match &args[1] {
                    ExprAst::Ident(color) if color == "red" => 1,
                    ExprAst::Ident(color) if color == "black" => 2,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "reveal_top_gain_if_color color must be red or black".into(),
                        });
                    }
                };
                Ok(StmtIr::EffectRevealTopGainIfColor {
                    player: check_expr(&args[0], locals)?,
                    color_mask,
                    span: *span,
                })
            }
            "prevent_current_damage" => {
                if !args.is_empty() {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "prevent_current_damage()".into(),
                    });
                }
                Ok(StmtIr::EffectPreventCurrentDamage { span: *span })
            }
            "judge_cancel_current_sha_target_on_red" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "judge_cancel_current_sha_target_on_red(player, card, target)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectJudgeCancelCurrentShaTargetOnRed {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    target: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "choose_draw_or_recover" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_draw_or_recover(player, draw_count, recover_count, allow_cancel)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectChooseDrawOrRecover {
                    player: check_expr(&args[0], locals)?,
                    draw_count: check_expr(&args[1], locals)?,
                    recover_count: check_expr(&args[2], locals)?,
                    allow_cancel: check_expr(&args[3], locals)?,
                    span: *span,
                })
            }
            "take_hand" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "take_hand(from, to)".into(),
                    });
                }
                Ok(StmtIr::EffectTakeHand {
                    from: check_expr(&args[0], locals)?,
                    to: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "give_all_hand" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "give_all_hand(player, target)".into(),
                    });
                }
                Ok(StmtIr::EffectGiveAllHand {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "gain_card_if_discard" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_card_if_discard(player, card)".into(),
                    });
                }
                Ok(StmtIr::EffectGainCardIfDiscard {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_draw_or_discard_target_card_record_sha" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_draw_or_discard_target_card_record_sha(player, card, target)"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectChooseDrawOrDiscardTargetCardRecordSha {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    target: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "reveal_hand_to_player" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_hand_to_player(player, observer)".into(),
                    });
                }
                Ok(StmtIr::EffectRevealHandToPlayer {
                    player: check_expr(&args[0], locals)?,
                    observer: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "lose_hp" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "lose_hp(player, amount)".into(),
                    });
                }
                Ok(StmtIr::EffectLoseHp {
                    player: check_expr(&args[0], locals)?,
                    amount: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "add_skill" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "add_skill(player, skill_def)".into(),
                    });
                }
                Ok(StmtIr::EffectAddSkill {
                    player: check_expr(&args[0], locals)?,
                    skill_def: expect_u32(&args[1], span.start as usize)?,
                    span: *span,
                })
            }
            "add_bound_skill" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "add_bound_skill(player, skill_def, target)".into(),
                    });
                }
                Ok(StmtIr::EffectAddBoundSkill {
                    player: check_expr(&args[0], locals)?,
                    skill_def: expect_u32(&args[1], span.start as usize)?,
                    target: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "replace_skill_with_bound_after_resolution_if_target_alive" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "replace_skill_with_bound_after_resolution_if_target_alive(player, skill, skill_def, target)".into(),
                    });
                }
                Ok(
                    StmtIr::EffectReplaceSkillWithBoundAfterResolutionIfTargetAlive {
                        player: check_expr(&args[0], locals)?,
                        skill: check_expr(&args[1], locals)?,
                        skill_def: expect_u32(&args[2], span.start as usize)?,
                        target: check_expr(&args[3], locals)?,
                        span: *span,
                    },
                )
            }
            "choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(player)"
                                .into(),
                    });
                }
                Ok(
                    StmtIr::EffectChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
                        player: check_expr(&args[0], locals)?,
                        span: *span,
                    },
                )
            }
            "choose_discard_hand_equip_then_recover" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_discard_hand_equip_then_recover(player, count, amount)"
                            .into(),
                    });
                }
                Ok(StmtIr::EffectChooseDiscardHandEquipThenRecover {
                    player: check_expr(&args[0], locals)?,
                    count: check_expr(&args[1], locals)?,
                    amount: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "remove_skill" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "remove_skill(skill_instance)".into(),
                    });
                }
                Ok(StmtIr::EffectRemoveSkill {
                    skill: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "refresh_skill" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "refresh_skill(skill_instance)".into(),
                    });
                }
                Ok(StmtIr::EffectRefreshSkill {
                    skill: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "tag_event_cards" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "tag_event_cards(skill_instance)".into(),
                    });
                }
                Ok(StmtIr::EffectTagEventCards {
                    skill: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "untag_event_cards" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "untag_event_cards(skill_instance)".into(),
                    });
                }
                Ok(StmtIr::EffectUntagEventCards {
                    skill: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "storage_set" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "storage_set(skill_instance, key, value)".into(),
                    });
                }
                Ok(StmtIr::EffectStorageSet {
                    skill: check_expr(&args[0], locals)?,
                    key: check_expr(&args[1], locals)?,
                    value: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "draw_all_event_targets" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "draw_all_event_targets(count)".into(),
                    });
                }
                Ok(StmtIr::EffectDrawAllEventTargets {
                    count: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "clanshenjun_reveal" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "clanshenjun_reveal(self, event_card, self_skill)".into(),
                    });
                }
                Ok(StmtIr::EffectClanshenjunReveal {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "clanshenjun_use" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "clanshenjun_use(self, self_skill)".into(),
                    });
                }
                Ok(StmtIr::EffectClanshenjunUse {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "clanyuzhi_reveal" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "clanyuzhi_reveal(self, self_skill)".into(),
                    });
                }
                Ok(StmtIr::EffectClanyuzhiReveal {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "clanyuzhi_round_end" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "clanyuzhi_round_end(self, self_skill)".into(),
                    });
                }
                Ok(StmtIr::EffectClanyuzhiRoundEnd {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "qieyi_phase_start" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "qieyi_phase_start(self, self_skill)".into(),
                    });
                }
                Ok(StmtIr::EffectQieyiPhaseStart {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "qieyi_use_after" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "qieyi_use_after(self, event_card, self_skill)".into(),
                    });
                }
                Ok(StmtIr::EffectQieyiUseAfter {
                    player: check_expr(&args[0], locals)?,
                    card: check_expr(&args[1], locals)?,
                    skill: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "jianzhi_phase_end" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "jianzhi_phase_end(self, self_skill)".into(),
                    });
                }
                Ok(StmtIr::EffectJianzhiPhaseEnd {
                    player: check_expr(&args[0], locals)?,
                    skill: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "move_card" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "move_card(card, to_zone, from_zone_or_zero)".into(),
                    });
                }
                let card = check_expr(&args[0], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "move_card first argument must be a card operand".into(),
                    });
                }
                Ok(StmtIr::EffectMoveCard {
                    card,
                    to: check_expr(&args[1], locals)?,
                    from: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "set_turn_state" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "set_turn_state(current, phase_tag)".into(),
                    });
                }
                let phase = expect_int(&args[1], span.start as usize)?;
                if !(0..=6).contains(&phase) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "phase_tag must be in 0..=6".into(),
                    });
                }
                Ok(StmtIr::EffectSetTurnState {
                    current: check_expr(&args[0], locals)?,
                    phase,
                    span: *span,
                })
            }
            "use_card_on" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "use_card_on(player, card, target)".into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "use_card_on second argument must be a card operand".into(),
                    });
                }
                Ok(StmtIr::EffectUseCardOn {
                    player: check_expr(&args[0], locals)?,
                    card,
                    target: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "use_virtual_card_on" => {
                if args.len() != 3 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "use_virtual_card_on(player, card_kind, target)".into(),
                    });
                }
                let ExprAst::Ident(kind_name) = &args[1] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "use_virtual_card_on second argument must be a card kind".into(),
                    });
                };
                let kind = card_kind_value(kind_name).ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: format!("unknown virtual card kind {kind_name}"),
                })?;
                Ok(StmtIr::EffectUseVirtualCardOn {
                    player: check_expr(&args[0], locals)?,
                    kind,
                    target: check_expr(&args[2], locals)?,
                    span: *span,
                })
            }
            "use_virtual_card_on_selected_targets" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "use_virtual_card_on_selected_targets(player, card_kind)".into(),
                    });
                }
                let ExprAst::Ident(kind_name) = &args[1] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "use_virtual_card_on_selected_targets second argument must be a card kind"
                                .into(),
                    });
                };
                if !matches!(kind_name.as_str(), "juedou" | "tiesuo") {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "use_virtual_card_on_selected_targets supports juedou or tiesuo"
                            .into(),
                    });
                }
                let kind = card_kind_value(kind_name).ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: format!("unknown virtual card kind {kind_name}"),
                })?;
                Ok(StmtIr::EffectUseVirtualCardOnSelectedTargets {
                    player: check_expr(&args[0], locals)?,
                    kind,
                    span: *span,
                })
            }
            "enter_dying" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "enter_dying(player)".into(),
                    });
                }
                Ok(StmtIr::EffectEnterDying {
                    player: check_expr(&args[0], locals)?,
                    span: *span,
                })
            }
            "kill_player" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "kill_player(player, killer)".into(),
                    });
                }
                Ok(StmtIr::EffectKillPlayer {
                    player: check_expr(&args[0], locals)?,
                    killer: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "reveal_card" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_card(card)".into(),
                    });
                }
                let card = check_expr(&args[0], locals)?;
                // Card operand must be a card-typed expression, not a player/int.
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_card expects a card operand (e.g. event_card)".into(),
                    });
                }
                Ok(StmtIr::EffectRevealCard { card, span: *span })
            }
            "use_card" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "use_card(player, card)".into(),
                    });
                }
                let player = check_expr(&args[0], locals)?;
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "use_card second argument must be a card operand".into(),
                    });
                }
                Ok(StmtIr::EffectUseCard {
                    player,
                    card,
                    span: *span,
                })
            }
            "choose_use_virtual_sha" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_use_virtual_sha(player, normal_distance|ignore_distance)"
                            .into(),
                    });
                }
                let ExprAst::Ident(policy) = &args[1] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "virtual Sha distance policy must be normal_distance or ignore_distance"
                                .into(),
                    });
                };
                let ignore_distance = match policy.as_str() {
                    "normal_distance" => false,
                    "ignore_distance" => true,
                    _ => {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "virtual Sha distance policy must be normal_distance or ignore_distance"
                                .into(),
                        });
                    }
                };
                Ok(StmtIr::EffectChooseUseVirtualSha {
                    player: check_expr(&args[0], locals)?,
                    ignore_distance,
                    span: *span,
                })
            }
            "remove_cards_by_kind" => {
                if args.len() != 1 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "remove_cards_by_kind(card_kind)".into(),
                    });
                }
                let ExprAst::Ident(kind) = &args[0] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "remove_cards_by_kind expects a card-kind name".into(),
                    });
                };
                let kind = card_kind_value(kind).ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: format!("unknown card kind {kind}"),
                })?;
                Ok(StmtIr::EffectRemoveCardsByKind { kind, span: *span })
            }
            "gain_card_by_kind" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_card_by_kind(player, card_kind)".into(),
                    });
                }
                let ExprAst::Ident(kind) = &args[1] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_card_by_kind expects a card-kind name".into(),
                    });
                };
                let kind = card_kind_value(kind).ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: format!("unknown card kind {kind}"),
                })?;
                Ok(StmtIr::EffectGainCardByKind {
                    player: check_expr(&args[0], locals)?,
                    kind,
                    span: *span,
                })
            }
            "gain_discard_card_by_kind" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_discard_card_by_kind(player, card_kind)".into(),
                    });
                }
                let ExprAst::Ident(kind) = &args[1] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "gain_discard_card_by_kind expects a card-kind name".into(),
                    });
                };
                let kind = card_kind_value(kind).ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: format!("unknown card kind {kind}"),
                })?;
                Ok(StmtIr::EffectGainDiscardCardByKind {
                    player: check_expr(&args[0], locals)?,
                    kind,
                    span: *span,
                })
            }
            "reveal_uniform_hand_choose_damage" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "reveal_uniform_hand_choose_damage(player, amount)".into(),
                    });
                }
                Ok(StmtIr::EffectRevealUniformHandChooseDamage {
                    player: check_expr(&args[0], locals)?,
                    amount: check_expr(&args[1], locals)?,
                    span: *span,
                })
            }
            "choose_gain_discard_card_by_kind" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_gain_discard_card_by_kind(player, card_kind)".into(),
                    });
                }
                let ExprAst::Ident(kind) = &args[1] else {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_gain_discard_card_by_kind expects a card-kind name".into(),
                    });
                };
                let kind = card_kind_value(kind).ok_or_else(|| CompileError::Type {
                    pos: span.start as usize,
                    message: format!("unknown card kind {kind}"),
                })?;
                Ok(StmtIr::EffectChooseGainDiscardCardByKind {
                    player: check_expr(&args[0], locals)?,
                    kind,
                    span: *span,
                })
            }
            "draw_for_lifetime_card_action_divisors" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "draw_for_lifetime_card_action_divisors(player, first, second, third)"
                                .into(),
                    });
                }
                let mut divisors = [0_u32; 3];
                for (index, arg) in args[1..].iter().enumerate() {
                    let ExprAst::Int(value) = arg else {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message: "draw_for_lifetime_card_action_divisors expects three positive integer literals".into(),
                        });
                    };
                    if *value <= 0 {
                        return Err(CompileError::Type {
                            pos: span.start as usize,
                            message:
                                "draw_for_lifetime_card_action_divisors divisors must be positive"
                                    .into(),
                        });
                    }
                    divisors[index] = *value as u32;
                }
                Ok(StmtIr::EffectDrawForLifetimeCardActionDivisors {
                    player: check_expr(&args[0], locals)?,
                    divisors,
                    span: *span,
                })
            }
            "choose_add_missing_skill" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_add_missing_skill(player, first, second, third)".into(),
                    });
                }
                let skill_defs = [
                    expect_u32(&args[1], span.start as usize)?,
                    expect_u32(&args[2], span.start as usize)?,
                    expect_u32(&args[3], span.start as usize)?,
                ];
                if skill_defs.contains(&0)
                    || skill_defs[0] == skill_defs[1]
                    || skill_defs[0] == skill_defs[2]
                    || skill_defs[1] == skill_defs[2]
                {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "choose_add_missing_skill expects three distinct non-zero skill definitions".into(),
                    });
                }
                Ok(StmtIr::EffectChooseAddMissingSkill {
                    player: check_expr(&args[0], locals)?,
                    skill_defs,
                    span: *span,
                })
            }
            "add_first_missing_skill" => {
                if args.len() != 4 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "add_first_missing_skill(player, first, second, third)".into(),
                    });
                }
                let skill_defs = [
                    expect_u32(&args[1], span.start as usize)?,
                    expect_u32(&args[2], span.start as usize)?,
                    expect_u32(&args[3], span.start as usize)?,
                ];
                if skill_defs.contains(&0)
                    || skill_defs[0] == skill_defs[1]
                    || skill_defs[0] == skill_defs[2]
                    || skill_defs[1] == skill_defs[2]
                {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "add_first_missing_skill expects three distinct non-zero skill definitions".into(),
                    });
                }
                Ok(StmtIr::EffectAddFirstMissingSkill {
                    player: check_expr(&args[0], locals)?,
                    skill_defs,
                    span: *span,
                })
            }
            "resolve_dccunsi" => {
                if args.len() != 5 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dccunsi(player, target, guixiu_def, yongjue_def, stage)"
                            .into(),
                    });
                }
                let guixiu_def = expect_u32(&args[2], span.start as usize)?;
                let yongjue_def = expect_u32(&args[3], span.start as usize)?;
                let stage = expect_u32(&args[4], span.start as usize)?;
                if guixiu_def == 0 || yongjue_def == 0 || guixiu_def == yongjue_def {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dccunsi expects two distinct non-zero skill definitions"
                            .into(),
                    });
                }
                if stage != 0 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "resolve_dccunsi public stage must be 0".into(),
                    });
                }
                Ok(StmtIr::EffectResolveDccunsi {
                    player: check_expr(&args[0], locals)?,
                    target: check_expr(&args[1], locals)?,
                    guixiu_def,
                    yongjue_def,
                    stage: stage as u8,
                    span: *span,
                })
            }
            "record_current_sha_all_targets" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message: "record_current_sha_all_targets(player, card)".into(),
                    });
                }
                let card = check_expr(&args[1], locals)?;
                if !is_card_operand(&card) {
                    return Err(CompileError::Type {
                        pos: span.start as usize,
                        message:
                            "record_current_sha_all_targets second argument must be a card operand"
                                .into(),
                    });
                }
                Ok(StmtIr::EffectRecordCurrentShaAllTargets {
                    player: check_expr(&args[0], locals)?,
                    card,
                    span: *span,
                })
            }
            other => Err(CompileError::Type {
                pos: span.start as usize,
                message: format!("unknown effect {other}"),
            }),    }
}


fn check_expr(e: &ExprAst, locals: &[String]) -> Result<ExprIr, CompileError> {
    match e {
        ExprAst::Int(n) => Ok(ExprIr::Int(*n)),
        ExprAst::U32(_) => Err(CompileError::Type {
            pos: 0,
            message: "u32 literal is only valid for an identifier argument".into(),
        }),
        ExprAst::Ident(name) => match name.as_str() {
            "self" => Ok(ExprIr::SelfPlayer),
            "self_skill" => Ok(ExprIr::SelfSkill),
            "event_player" => Ok(ExprIr::EventPlayer),
            "event_source" => Ok(ExprIr::EventSource),
            "event_card" => Ok(ExprIr::EventCard),
            "event_amount" => Ok(ExprIr::EventAmount),
            "event_target_count" => Ok(ExprIr::EventTargetCount),
            "event_card_count" => Ok(ExprIr::EventCardCount),
            "turn_damaged_player_count" => Ok(ExprIr::TurnDamagedPlayerCount),
            other => {
                if locals.iter().any(|l| l == other) {
                    Ok(ExprIr::Local(other.into()))
                } else {
                    Err(CompileError::Type {
                        pos: 0,
                        message: format!("unknown name {other}"),
                    })
                }
            }
        },
        ExprAst::Call { name, args } => match name.as_str() {
            "distance"
            | "attack_range"
            | "hand_limit"
            | "hp"
            | "hand_count"
            | "empty_equip_slots"
            | "max_hp"
            | "relation"
            | "card_usable"
            | "target_legal"
            | "card_attribute"
            | "skill_uses_remaining"
            | "zone"
            | "card_zone"
            | "equipped_count"
            | "mark_count"
            | "skill_pile_count"
            | "alive_count"
            | "alive_group_count"
            | "alive_wounded_or_guiming_wu_count"
            | "unique_max_current_hp"
            | "unique_max_current_hp_at"
            | "lifetime_damage_dealt"
            | "public_card_kind_count"
            | "field_equipment_kind_count"
            | "hand_equip_suit_count"
            | "hand_suit_count"
            | "hand_color_count"
            | "skill_content_flag"
            | "round_number_capped"
            | "card_action_count"
            | "cards_drawn_this_turn"
            | "damage_dealt_this_turn"
            | "has_skill"
            | "card_name_length"
            | "in_play_phase"
            | "card_has_tag"
            | "storage_get"
            | "storage_has"
            | "cards_used_this_round"
            | "current_phase_player" => {
                // Per-query arity + which arg positions are card-typed (0-based).
                let (arity, card_positions): (usize, &[usize]) = match name.as_str() {
                    "alive_count" | "alive_group_count" | "current_phase_player" | "in_play_phase" => (0, &[]),
                    "equipped_count" => (1, &[]),
                    "unique_max_current_hp_at" => (2, &[]),
                    "hand_equip_suit_count" | "hand_suit_count" | "hand_color_count" => (2, &[]),
                    "skill_content_flag" => (3, &[]),
                    "distance" | "relation" | "mark_count" | "skill_pile_count" => (2, &[]),
                    "has_skill" => (2, &[]),
                    "card_name_length" => (2, &[1]),
                    "card_has_tag" => (2, &[0]),   // (card, skill_instance)
                    "storage_get" | "storage_has" => (2, &[]), // (skill_instance, key)
                    "cards_used_this_round" => (1, &[]),       // (player)
                    "card_usable" => (2, &[1]),    // (player, card)
                    "target_legal" => (3, &[1]),   // (user, card, target)
                    "card_attribute" => (2, &[0]), // (card, attr)
                    "skill_uses_remaining" => (1, &[]),
                    "zone" => (2, &[]),
                    "card_zone" => (1, &[0]),
                    "round_number_capped" => (1, &[]),
                    _ => (1, &[]),
                };
                if args.len() != arity {
                    return Err(CompileError::Type {
                        pos: 0,
                        message: format!("{name} expects {arity} argument(s)"),
                    });
                }
                let mut a = Vec::new();
                for (i, x) in args.iter().enumerate() {
                    let ir = if matches!(name.as_str(), "hand_equip_suit_count" | "hand_suit_count")
                        && i == 1
                    {
                        let ExprAst::Ident(suit) = x else {
                            return Err(CompileError::Type {
                                pos: 0,
                                message: format!("{name} expects a suit name"),
                            });
                        };
                        ExprIr::Int(match suit.as_str() {
                            "spade" => 1,
                            "heart" => 2,
                            "club" => 3,
                            "diamond" => 4,
                            _ => {
                                return Err(CompileError::Type {
                                    pos: 0,
                                    message: format!("unknown suit {suit}"),
                                });
                            }
                        })
                    } else if name == "hand_color_count" && i == 1 {
                        let ExprAst::Ident(color) = x else {
                            return Err(CompileError::Type {
                                pos: 0,
                                message: "hand_color_count expects a color name".into(),
                            });
                        };
                        ExprIr::Int(match color.as_str() {
                            "red" => 1,
                            "black" => 2,
                            _ => {
                                return Err(CompileError::Type {
                                    pos: 0,
                                    message: format!("unknown color {color}"),
                                });
                            }
                        })
                    } else if matches!(
                        name.as_str(),
                        "public_card_kind_count" | "field_equipment_kind_count"
                    ) {
                        let ExprAst::Ident(kind) = x else {
                            return Err(CompileError::Type {
                                pos: 0,
                                message: format!("{name} expects a card-kind name"),
                            });
                        };
                        ExprIr::Int(card_kind_value(kind).ok_or_else(|| CompileError::Type {
                            pos: 0,
                            message: format!("unknown card kind {kind}"),
                        })?)
                    } else if name == "mark_count" && i == 1 {
                        let mark = expect_u32(x, 0)?;
                        i32::try_from(mark).map_or(ExprIr::U32(mark), ExprIr::Int)
                    } else if name == "skill_pile_count" && i == 1 {
                        let skill_def = expect_u32(x, 0)?;
                        i32::try_from(skill_def).map_or(ExprIr::U32(skill_def), ExprIr::Int)
                    } else if name == "skill_content_flag" && i == 1 {
                        let skill_def = expect_u32(x, 0)?;
                        i32::try_from(skill_def).map_or(ExprIr::U32(skill_def), ExprIr::Int)
                    } else if name == "skill_content_flag" && i == 2 {
                        let bit = literal_i32(x).ok_or_else(|| CompileError::Type {
                            pos: 0,
                            message: "skill_content_flag bit must be a literal in 0..63".into(),
                        })?;
                        if !(0..64).contains(&bit) {
                            return Err(CompileError::Type {
                                pos: 0,
                                message: "skill_content_flag bit must be in 0..63".into(),
                            });
                        }
                        ExprIr::Int(bit)
                    } else {
                        check_expr(x, locals)?
                    };
                    if card_positions.contains(&i) && !is_card_operand(&ir) {
                        return Err(CompileError::Type {
                            pos: 0,
                            message: format!(
                                "{name} argument {i} must be a card operand (e.g. event_card)"
                            ),
                        });
                    }
                    a.push(ir);
                }
                Ok(ExprIr::Query {
                    name: name.clone(),
                    args: a,
                })
            }
            "choose" => {
                if args.len() < 2 {
                    return Err(CompileError::Type {
                        pos: 0,
                        message: "choose expects at least 2 option tokens".into(),
                    });
                }
                if args.len() > 255 {
                    return Err(CompileError::Type {
                        pos: 0,
                        message: "choose supports at most 255 options".into(),
                    });
                }
                let mut opts = Vec::new();
                for x in args {
                    opts.push(check_expr(x, locals)?);
                }
                Ok(ExprIr::Choose { options: opts })
            }
            "choose_range" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: 0,
                        message: "choose_range expects inclusive min and max bounds".into(),
                    });
                }
                Ok(ExprIr::ChooseRange {
                    min: Box::new(check_expr(&args[0], locals)?),
                    max: Box::new(check_expr(&args[1], locals)?),
                })
            }
            "min" => {
                if args.len() != 2 {
                    return Err(CompileError::Type {
                        pos: 0,
                        message: "min expects exactly two arguments".into(),
                    });
                }
                Ok(ExprIr::Min {
                    left: Box::new(check_expr(&args[0], locals)?),
                    right: Box::new(check_expr(&args[1], locals)?),
                })
            }
            other => Err(CompileError::Type {
                pos: 0,
                message: format!("unknown function {other}"),
            }),
        },
        ExprAst::BinOp { op, left, right } => Ok(ExprIr::BinOp {
            op: op.clone(),
            left: Box::new(check_expr(left, locals)?),
            right: Box::new(check_expr(right, locals)?),
        }),
    }
}

/// Whether an expression is a card-typed operand (currently `event_card` or a
/// local, since locals are untyped for now). Rejects players / integers.
fn is_card_operand(e: &ExprIr) -> bool {
    matches!(e, ExprIr::EventCard | ExprIr::Local(_))
}

fn expect_int(e: &ExprAst, pos: usize) -> Result<i32, CompileError> {
    match e {
        ExprAst::Int(n) => Ok(*n),
        _ => Err(CompileError::Type {
            pos,
            message: "expected integer literal".into(),
        }),
    }
}

fn bounded_u8(arg: &ExprAst, name: &str) -> Result<u8, CompileError> {
    u8::try_from(expect_int(arg, 0).map_err(|_| CompileError::Type {
        pos: 0,
        message: format!("{name} must be an integer literal"),
    })?)
    .map_err(|_| CompileError::Type {
        pos: 0,
        message: format!("{name} must be 0..255"),
    })
}

fn literal_i32(e: &ExprAst) -> Option<i32> {
    match e {
        ExprAst::Int(n) => Some(*n),
        _ => None,
    }
}

fn expect_draw_target_scope(
    expression: &ExprAst,
    pos: usize,
) -> Result<DrawTargetScopeIr, CompileError> {
    match expression {
        ExprAst::Ident(scope) if scope == "other" => Ok(DrawTargetScopeIr::Other),
        ExprAst::Ident(scope) if scope == "male_other" => Ok(DrawTargetScopeIr::MaleOther),
        ExprAst::Ident(scope) if scope == "female_other" => Ok(DrawTargetScopeIr::FemaleOther),
        ExprAst::Ident(scope) if scope == "any" => Ok(DrawTargetScopeIr::Any),
        ExprAst::Ident(scope) if scope == "min_hand" => Ok(DrawTargetScopeIr::MinHand),
        ExprAst::Ident(scope) if scope == "same_hp" => Ok(DrawTargetScopeIr::SameHp),
        ExprAst::Ident(scope) if scope == "hand_empty" => Ok(DrawTargetScopeIr::HandEmpty),
        ExprAst::Ident(scope) if scope == "wounded" => Ok(DrawTargetScopeIr::Wounded),
        ExprAst::Ident(scope) if scope == "hand_lte_chooser" => {
            Ok(DrawTargetScopeIr::HandLteChooser)
        }
        _ => Err(CompileError::Type {
            pos,
            message:
                "target scope must be other, male_other, female_other, any, min_hand, same_hp, hand_empty, wounded, or hand_lte_chooser"
                    .into(),
        }),
    }
}

fn expect_u32(e: &ExprAst, pos: usize) -> Result<u32, CompileError> {
    match e {
        ExprAst::Int(n) if *n >= 0 => Ok(*n as u32),
        ExprAst::U32(n) => Ok(*n),
        _ => Err(CompileError::Type {
            pos,
            message: "expected u32 literal".into(),
        }),
    }
}
