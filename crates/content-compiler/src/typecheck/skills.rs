use super::*;

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

pub(super) fn check_view_as(v: &ViewAsAst) -> Result<ViewAsIr, CompileError> {
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
                || v.owner_hp_min_one_cards
                || v.owner_basic_uses_plus_one_cards
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
            || v.owner_hp_min_one_cards
            || v.owner_basic_uses_plus_one_cards
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
            || v.owner_hp_min_one_cards
            || v.owner_basic_uses_plus_one_cards
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
            || v.owner_hp_min_one_cards
            || v.owner_basic_uses_plus_one_cards
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
            || v.owner_hp_min_one_cards
            || v.owner_basic_uses_plus_one_cards
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
