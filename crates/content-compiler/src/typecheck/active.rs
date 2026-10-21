use super::*;

pub(super) fn check_active(a: &ActiveAst) -> Result<ActiveIr, CompileError> {
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
