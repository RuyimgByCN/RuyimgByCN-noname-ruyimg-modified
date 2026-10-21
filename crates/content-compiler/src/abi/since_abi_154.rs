use super::*;

pub(super) fn required_abi(ir: &RuleModuleIr, _module_required_abi: u32) -> u32 {
    if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_source_sha_target_alive_hand_nonempty")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_154
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter
                == "owner_source_sha_dodged_target_recorded_by_skill_has_discardable_hand_equip"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_152
    } else if ir.skills.iter().any(|skill| {
        skill.actives.iter().any(|active| {
            active.hand_nonempty
                || active.face_up
                || active.targets.as_ref().is_some_and(|targets| targets.male)
        }) || skill.triggers.iter().any(|trigger| {
            trigger.filter
                == "owner_alive_other_source_sha_resolved_not_target_gainable_once_per_turn"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_151
    } else if ir.skills.iter().any(|skill| {
        skill.actives.iter().any(|active| {
            active
                .targets
                .as_ref()
                .is_some_and(|targets| targets.hand_nonempty)
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_150
    } else if ir.skills.iter().any(|skill| {
        skill
            .modifiers
            .iter()
            .any(|modifier| modifier.target == "card_distance_unlimited_for_skill_card_policy")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_149
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_source_sha_direct_damage_before")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_147
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_alive_other_turn_ended_card_targeted_other_no_damage_other"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_146
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter
                == "owner_source_sha_or_ordinary_trick_target_wounded_has_discardable_hand_equip"
        }) || skill.modifiers.iter().any(|modifier| {
            modifier.value_source.as_deref() == Some("alive_wounded_or_guiming_wu_count_min_one")
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_145
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_alive_not_target_hand_equip_nonempty")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_144
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_source_damage_card_once_per_turn")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_141
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            matches!(
                trigger.filter.as_str(),
                "owner_alive_event_player_hand_nonempty_once_per_round"
                    | "owner_source_bound_category_first_three_play_phase"
            )
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_140
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_alive_gained_three_this_turn_other_alive")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_139
    } else if ir.skills.iter().any(|skill| {
        skill.actives.iter().any(|active| {
            active
                .targets
                .as_ref()
                .is_some_and(|targets| targets.has_discardable_equip)
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_136
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_is_target_source_group_qun_discardable_hand_equip_gte_two"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_134
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_alive_skill_bound_target_is_event_player")
            || skill.actives.iter().any(|active| {
                active
                    .targets
                    .as_ref()
                    .is_some_and(|targets| targets.hp_gt.is_some())
            })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_132
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_source_repeats_color_has_discardable_hand_equip"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_131
    } else if ir.skills.iter().any(|skill| {
        skill.actives.iter().any(|active| {
            active
                .selection
                .as_ref()
                .is_some_and(|selection| selection.filter.as_deref() == Some("card_armor"))
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_130
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_alive_source_alive_event_player_dying_once_per_round"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_129
    } else if ir.skills.iter().any(|skill| {
        skill.actives.iter().any(|active| {
            active
                .targets
                .as_ref()
                .is_some_and(|targets| targets.hand_lt.is_some())
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_128
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_is_target_any_alive_self_discardable_hand_equip"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_127
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter
                == "owner_alive_sha_or_duel_damage_source_target_alive_hand_equip_nonempty"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_126
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_alive_other_dying_hand_nonempty_once_per_game")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_125
    } else if ir.skills.iter().any(|skill| {
        skill.actives.iter().any(|active| {
            active
                .targets
                .as_ref()
                .is_some_and(|targets| targets.hp_gt_owner)
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_121
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_source_sha_target_alive_owner_outside_target_attack_range"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_119
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_source_other_target_alive_has_discardable_hand_equip"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_118
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_is_target_source_has_weapon")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_117
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_source_sha_or_trick_current_play_phase")
            || skill.modifiers.iter().any(|modifier| {
                modifier.value_source.as_deref() == Some("owner_play_phase_card_use_count_zero")
            })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_115
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_alive_wounded_card_targets_gt_one")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_114
    } else if ir.skills.iter().any(|skill| {
        skill.actives.iter().any(|active| {
            active
                .targets
                .as_ref()
                .is_some_and(|targets| targets.hand_gt_owner)
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_113
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_alive_other_discarded_black_to_discard")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_112
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_alive_targeted_by_other_trick")
            || skill.modifiers.iter().any(|modifier| {
                matches!(
                    modifier.target.as_str(),
                    "turn_face_down_cancel" | "restore_skipped_phase"
                )
            })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_111
    } else if ir.skills.iter().any(|skill| {
        skill.actives.iter().any(|active| {
            active
                .selection
                .as_ref()
                .is_some_and(|selection| selection.unused_category_by_skill_this_play_phase)
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_110
    } else if ir.skills.iter().any(|skill| {
        skill.actives.iter().any(|active| {
            active
                .targets
                .as_ref()
                .is_some_and(|targets| targets.wounded || targets.match_selected_card_count)
                || active
                    .selection
                    .as_ref()
                    .is_some_and(|selection| selection.distinct_categories)
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_109
    } else if ir.skills.iter().any(|skill| {
        skill
            .modifiers
            .iter()
            .any(|modifier| modifier.target == "card_target_cancel_if_other_user_hp_gte_target")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_107
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_alive_is_target_current_play_phase")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_106
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            matches!(
                trigger.filter.as_str(),
                "owner_alive_other_play_phase_duplicate_card_category_has_hand_equip"
                    | "owner_alive_other_phase_end_damage_gt_one_has_hand_equip"
            )
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_105
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_source_wounded_nanman_wanjian_or_normal_spade_sha"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_104
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            matches!(
                trigger.filter.as_str(),
                "owner_source_basic" | "owner_source_non_basic"
            )
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_99
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.event == "phase_any_end"
                || matches!(
                    trigger.filter.as_str(),
                    "owner_alive_phase_targeted_hp_changed_can_use_wanjian"
                        | "owner_alive_phase_targeted_hp_unchanged_can_use_juedou"
                )
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_98
    } else if ir.skills.iter().any(|skill| {
        skill.actives.iter().any(|active| {
            active
                .targets
                .as_ref()
                .is_some_and(|targets| targets.has_equip)
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_97
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_source_sha_target_alive_has_discardable_hand_equip"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_94
    } else if ir.skills.iter().any(|skill| {
        skill.modifiers.iter().any(|modifier| {
            modifier.target == "tao_use_save_cancel_for_non_dying_others_during_owner_turn"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_91
    } else if ir.skills.iter().any(|skill| {
        skill
            .modifiers
            .iter()
            .any(|modifier| modifier.target == "sha_response_count_if_other_female")
            || skill
                .triggers
                .iter()
                .any(|trigger| trigger.filter == "owner_is_target_not_min_hp")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_89
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger
                .filter
                .starts_with("owner_alive_other_damage_source_group_")
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_88
    } else if ir.skills.iter().any(|skill| {
        skill
            .modifiers
            .iter()
            .any(|modifier| modifier.target == "virtual_bagua_if_armor_slot_empty")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_87
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger
                .filter
                .starts_with("owner_alive_other_judgment_black_event_player_group_")
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_86
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_is_target_other_alive")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_85
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_is_target_per_damage_point")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_84
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_alive_event_cards_nonempty")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_82
    } else if ir.skills.iter().any(|skill| {
        skill
            .modifiers
            .iter()
            .any(|modifier| modifier.value_source.as_deref() == Some("owner_qun_other_shu"))
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_81
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_alive_other_normal_damage_target_unlinked")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_73
    } else if ir.skills.iter().any(|skill| {
        skill
            .modifiers
            .iter()
            .any(|modifier| modifier.target == "judgment_best_from_draw_pile")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_72
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger
                .filter
                .strip_prefix("owner_source_play_card_count_eq_")
                .is_some_and(|value| value.parse::<u32>().is_ok_and(|value| value > 0))
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_71
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_from_hand_empty_other_discardable_hand_equip")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_70
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_is_target_hand_at_least_hp_other_alive")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_69
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_is_target_can_compare_other")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_68
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_source_basic_or_trick")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_67
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_hand_compare_participant_result_card_in_discard"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_66
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_is_target_any_alive_hand_nonempty")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_62
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_source_or_target_hand_count_differs_once_per_turn"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_60
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_source_sha_has_weapon")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_59
    } else if ir.skills.iter().any(|skill| {
        skill
            .actives
            .iter()
            .any(|active| active.equip_nonempty || active.once_per_game)
            || skill.triggers.iter().any(|trigger| {
                trigger.filter == "owner_alive_other_turn_ended_no_card_targeted_other"
            })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_58
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            matches!(
                trigger.filter.as_str(),
                "owner_source_single_target_sha_or_black_ordinary_trick_current_play_phase_unused"
                    | "owner_is_target_male_source_has_discardable_equipment"
            )
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_57
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_discarded_has_hand_once_per_phase")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_56
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            matches!(
                trigger.filter.as_str(),
                "owner_is_target_tied_max_hand"
                    | "owner_is_target_tied_max_hp"
                    | "owner_is_target_tied_max_equip_nonempty"
                    | "owner_is_target_skipped_phase_count_positive"
            )
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_55
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            matches!(
                trigger.filter.as_str(),
                "owner_is_target_other_hand_nonempty"
                    | "owner_source_play_card_matches_remembered_category"
                    | "owner_phase_gift_target_is_source"
                    | "owner_phase_gift_target_is_target"
                    | "owner_source_sha_resolved_has_gainable_cards_and_recipient"
            )
        })
    }) {
        if ir.skills.iter().any(|skill| {
            skill.triggers.iter().any(|trigger| {
                trigger.filter == "owner_source_sha_resolved_has_gainable_cards_and_recipient"
            })
        }) {
            ADDITIVE_RULE_VM_ABI_VERSION_54
        } else {
            ADDITIVE_RULE_VM_ABI_VERSION_53
        }
    } else if ir.skills.iter().any(|skill| {
        skill.actives.iter().any(|active| {
            active
                .targets
                .as_ref()
                .is_some_and(|targets| targets.unused_by_skill_this_turn)
                || active
                    .selection
                    .as_ref()
                    .is_some_and(|selection| selection.unused_category_by_skill_this_turn)
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_52
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            matches!(
                trigger.filter.as_str(),
                "owner_discarded_black_basic_to_discard"
                    | "owner_alive_other_judge_phase_in_range_has_judge_card"
            )
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_51
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.event == "phase_draw_end")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_50
    } else if ir.skills.iter().any(|skill| {
        skill.modifiers.iter().any(|modifier| {
            matches!(
                modifier.target.as_str(),
                "ignore_armor_for_skill_card_policy"
                    | "card_use_unrespondable_for_skill_card_policy"
            ) || modifier.value_source.as_deref() == Some("owner_current_turn_card_use_count")
        }) || skill.triggers.iter().any(|trigger| {
            matches!(
                trigger.filter.as_str(),
                "owner_source_single_target_sha_or_ordinary_trick_current_turn"
                    | "owner_source_event_card_policy_draw"
            )
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_49
    } else if ir.skills.iter().any(|skill| {
        skill
            .modifiers
            .iter()
            .any(|modifier| modifier.target == "card_action_cancel_for_skill_color_mask")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_48
    } else if ir.skills.iter().any(|skill| {
        skill.actives.iter().any(|active| {
            active
                .targets
                .as_ref()
                .is_some_and(|targets| targets.can_compare)
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_47
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_alive_play_phase_sha_user_has_discardable_hand_equip"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_43
    } else if ir.skills.iter().any(|skill| {
        skill.modifiers.iter().any(|modifier| {
            modifier.value_source.as_deref()
                == Some("owner_distinct_hand_card_suits_used_this_turn")
        }) || skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_is_target_play_phase_card_category_count_positive"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_42
    } else if ir.skills.iter().any(|skill| {
        skill.actives.iter().any(|active| {
            active
                .selection
                .as_ref()
                .is_some_and(|selection| selection.filter.as_deref() == Some("card_sha"))
        }) || skill.triggers.iter().any(|trigger| {
            trigger
                .filter
                .strip_prefix("owner_is_target_active_used_at_least_")
                .and_then(|value| value.strip_suffix("_this_play_phase"))
                .is_some()
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_41
    } else if ir
        .skills
        .iter()
        .any(|skill| skill.actives.iter().any(|active| active.hand_gt_max_hp))
    {
        ADDITIVE_RULE_VM_ABI_VERSION_38
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_source_sha_target_alive_distance_le_one")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_37
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_source_sha_dodged_target_alive_has_discardable_hand_equip"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_36
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_source_sha_target_hand_gte_hp_or_lte_attack_range"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_35
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_source_sha_target_alive_has_discardable_region_card"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_34
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_source_black_sha_target_alive_has_discardable_equip"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_33
    } else if ir.skills.iter().any(|skill| {
        skill.actives.iter().any(|active| {
            active
                .targets
                .as_ref()
                .is_some_and(|targets| targets.equipment_pair_with_armor)
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_32
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_alive_used_sha_this_turn")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_31
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_is_target_has_movable_equipment")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_30
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_alive_sha_target_no_other_red_card_this_turn")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_29
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_alive_event_player_from_equip_empty")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_28
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            matches!(
                trigger.filter.as_str(),
                "owner_alive_event_player_turn_changed_or_became_linked"
                    | "owner_alive_event_player_became_face_up_or_linked"
            )
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_27
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_alive_event_player_outside_turn_from_hand_empty_once"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_26
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.event == "character_state_changed")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_25
    } else if ir.skills.iter().any(|skill| {
        skill
            .modifiers
            .iter()
            .any(|modifier| modifier.target == "judgment_rewrite_hand_equip")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_24
    } else if ir.skills.iter().any(|skill| {
        skill.actives.iter().any(|active| {
            active
                .targets
                .as_ref()
                .is_some_and(|targets| targets.can_use_on_owner.is_some())
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_23
    } else if ir.skills.iter().any(|skill| {
        skill.triggers.iter().any(|trigger| {
            trigger.filter == "owner_is_target_source_alive_gainable_card_per_damage_point"
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_22
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| trigger.filter == "owner_is_target_use_response_discarded_at_least_hp")
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_21
    } else if ir.skills.iter().any(|skill| {
        skill.modifiers.iter().any(|modifier| {
            matches!(
                modifier.target.as_str(),
                "observer_foreign_hand_faces_visible" | "observer_hidden_identities_visible"
            )
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_20
    } else if ir.skills.iter().any(|skill| {
        skill
            .modifiers
            .iter()
            .any(|modifier| modifier.target == "card_target_limit")
            || skill.triggers.iter().any(|trigger| {
                trigger.filter == "owner_is_event_player_no_enabled_equip_slot_once_per_game"
            })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_19
    } else if ir.skills.iter().any(|skill| {
        skill
            .view_as
            .iter()
            .any(|view| view.source_zone.as_deref() == Some("none"))
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_17
    } else if ir.skills.iter().any(|skill| {
        skill.actives.iter().any(|active| {
            active
                .selection
                .as_ref()
                .is_some_and(|selection| selection.filter.is_some())
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_16
    } else if ir.skills.iter().any(|skill| {
        skill.view_as.iter().any(|view| {
            view.filter == "non_basic" || view.red_unrespondable || view.black_no_use_count
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_14
    } else if ir.skills.iter().any(|skill| {
        skill.view_as.iter().any(|view| {
            matches!(
                view.filter.as_str(),
                "card_basic"
                    | "card_equip"
                    | "black_basic_equip"
                    | "owner_dying_club"
                    | "owner_single_red_hand"
                    | "owner_hand_gt_hp"
                    | "owner_hp_le_two_black_trick"
            )
        })
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_5
    } else if ir.skills.iter().any(|skill| {
        skill
            .triggers
            .iter()
            .any(|trigger| statements_require_abi4(&trigger.body))
            || skill
                .actives
                .iter()
                .any(|active| statements_require_abi4(&active.body))
            || skill.view_as.iter().any(|view| view.ignore_distance)
    }) {
        ADDITIVE_RULE_VM_ABI_VERSION_4
    } else {
        BASE_RULE_VM_ABI_VERSION
    }
}
