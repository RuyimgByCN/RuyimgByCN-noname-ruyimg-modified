//! 视为技（view-as）定义的类型检查与 IR 降级。

use super::*;

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
