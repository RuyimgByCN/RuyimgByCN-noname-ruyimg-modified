//! 触发器（trigger）定义的类型检查与 IR 降级。

use super::*;

pub(super) fn check_trigger(t: &TriggerAst) -> Result<TriggerIr, CompileError> {
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
        && !matches!(
            t.event.as_str(),
            "card_used" | "card_targeted" | "cards_moved"
        )
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
