use super::*;

pub(super) fn check_modifier(m: &ModifierAst) -> Result<ModifierIr, CompileError> {
    let judgment_cost_mark = m
        .target
        .strip_prefix("judgment_rewrite_hand_equip_cost_mark_")
        .and_then(|mark| mark.parse::<u32>().ok())
        .is_some_and(|mark| mark > 0);
    let target_mark_exemption = (m
        .target
        .strip_prefix("card_distance_unlimited_target_mark_")
        .or_else(|| m.target.strip_prefix("sha_use_unlimited_target_mark_"))
        .or_else(|| {
            m.target
                .strip_prefix("card_distance_unlimited_skill_on_owner_")
        })
        .or_else(|| {
            m.target
                .strip_prefix("card_use_unrespondable_skill_on_owner_")
        }))
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
        if m.target == "card_target_cancel_if_other_user_card_use_count_lt_owner_mark"
            && (m.mode != "override" || m.value != 1)
        {
            return Err(CompileError::Type {
                pos: m.span.start as usize,
                message: "平定目标限制只接受 override 1".into(),
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
