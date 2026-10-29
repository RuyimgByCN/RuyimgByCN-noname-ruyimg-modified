use super::*;
mod trigger;
use trigger::*;
mod view_as;
use view_as::*;

pub(super) fn check_skill(sk: &SkillAst) -> Result<SkillIr, CompileError> {
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
            SkillItemAst::Modifier(m) => {
                let modifier = check_modifier(m)?;
                if modifier.target
                    == "card_target_cancel_if_other_user_card_use_count_lt_owner_mark"
                    && (modifier.mode != "override" || modifier.value != 1)
                {
                    return Err(CompileError::Type {
                        pos: m.span.start as usize,
                        message: "平定目标限制只接受 override 1".into(),
                    });
                }
                modifiers.push(modifier);
            }
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

pub(super) fn valid_trigger_filter(filter: &str) -> bool {
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
pub(super) fn valid_event_view_as_skill_filter(filter: &str) -> bool {
    if let Some(def) = filter.strip_prefix("event_view_as_skill_name_") {
        return !def.is_empty()
            && def
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    }
    filter
        .strip_prefix("event_view_as_skill_")
        .is_some_and(|def| {
            !def.is_empty()
                && def.bytes().all(|b| b.is_ascii_digit())
                && def.parse::<u32>().is_ok_and(|v| v > 0)
        })
}

/// Card-name-scoped trigger filter family (Phase 3): accepts
/// `owner_source_card_<selector>`, `owner_is_target_card_<selector>`,
/// `owner_is_target_card_damage_<selector>` where `<selector>` is a known card-selector name
/// mirrored from the engine's `map_card_selector`.
pub(super) fn valid_card_name_scoped_filter(filter: &str) -> bool {
    const SELECTORS: &[&str] = &[
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
        "hand_rank_letter",
        "sole_hand_card",
        "sha_diamond",
        "damage_card",
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

pub(super) fn stmts_features(stmts: &[StmtAst]) -> u32 {
    stmts.iter().fold(0, |all, stmt| all | stmt_features(stmt))
}

pub(super) fn stmts_use_effect(stmts: &[StmtAst], effect: &str) -> bool {
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

pub(super) fn stmts_use_effect_ident_arg(
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

pub(super) fn stmt_features(stmt: &StmtAst) -> u32 {
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

pub(super) fn expr_features(expr: &ExprAst) -> u32 {
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
