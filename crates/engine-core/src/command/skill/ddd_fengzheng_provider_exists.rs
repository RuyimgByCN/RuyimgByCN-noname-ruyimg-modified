//! Command operations for `ddd_fengzheng_provider_exists` and related helpers.

use super::*;

pub(crate) fn ddd_fengzheng_provider_exists(state: &crate::state::GameState) -> bool {
    state.players.iter_live().any(|player| {
        player.alive
            && player.skills.iter().any(|skill| {
                state.skills.get(*skill).is_some_and(|instance| {
                    instance.def.raw() == DDD_FENGZHENG_DEF
                        && state.skill_effectively_enabled(instance)
                })
            })
    })
}

pub(crate) fn dddfusi_kind_matches(actual: CardKind, declared: CardKind) -> bool {
    (actual.is_sha() && declared.is_sha()) || actual == declared
}

pub(crate) fn dddfusi_policy_entries(
    blob: &[u8],
    round: u32,
    current: PlayerId,
) -> Vec<(PlayerId, u8)> {
    if blob.len() < 20
        || &blob[..4] != b"FUS1"
        || u32::from_le_bytes(blob[4..8].try_into().unwrap()) != round
        || u32::from_le_bytes(blob[8..12].try_into().unwrap()) != current.raw()
        || (blob.len() - 20) % 5 != 0
    {
        return Vec::new();
    }
    blob[20..]
        .chunks_exact(5)
        .filter_map(|entry| {
            let provider = PlayerId::new(u32::from_le_bytes(entry[..4].try_into().ok()?));
            matches!(entry[4], 1 | 2).then_some((provider, entry[4]))
        })
        .collect()
}

pub(crate) fn dddfusi_policy(
    state: &crate::state::GameState,
    skill: SkillId,
    provider: PlayerId,
) -> u8 {
    state
        .skills
        .get(skill)
        .and_then(|instance| {
            dddfusi_policy_entries(&instance.state_blob, state.turn.round, state.turn.current)
                .into_iter()
                .find_map(|(owner, policy)| (owner == provider).then_some(policy))
        })
        .unwrap_or(0)
}

pub(crate) fn set_dddfusi_approval(
    state: &mut crate::state::GameState,
    skill: SkillId,
    card: CardId,
    provider: PlayerId,
    policy: Option<u8>,
) -> Result<(), EffectError> {
    let round = state.turn.round;
    let current = state.turn.current;
    let mut entries = state
        .skills
        .get(skill)
        .map(|instance| dddfusi_policy_entries(&instance.state_blob, round, current))
        .ok_or_else(|| EffectError::Illegal("腹笥借牌技能已失效".into()))?;
    if let Some(policy) = policy {
        entries.retain(|(owner, _)| *owner != provider);
        entries.push((provider, policy));
        entries.sort_by_key(|(owner, _)| owner.raw());
    }
    let mut blob = b"FUS1".to_vec();
    blob.extend_from_slice(&round.to_le_bytes());
    blob.extend_from_slice(&current.raw().to_le_bytes());
    blob.extend_from_slice(&card.raw().to_le_bytes());
    blob.extend_from_slice(&provider.raw().to_le_bytes());
    for (owner, policy) in entries {
        blob.extend_from_slice(&owner.raw().to_le_bytes());
        blob.push(policy);
    }
    state
        .skills
        .get_mut(skill)
        .expect("validated 腹笥借牌技能")
        .state_blob = blob;
    Ok(())
}

pub(crate) fn dddfusi_approved_source(
    state: &crate::state::GameState,
    borrower: PlayerId,
    card: CardId,
) -> Option<PlayerId> {
    state.skills.sorted_live().into_iter().find_map(|instance| {
        if instance.owner != borrower
            || instance.def.raw() != DDD_FUSI_GLOBAL_DEF
            || instance.state_blob.len() < 20
            || &instance.state_blob[..4] != b"FUS1"
            || u32::from_le_bytes(instance.state_blob[4..8].try_into().ok()?) != state.turn.round
            || u32::from_le_bytes(instance.state_blob[8..12].try_into().ok()?)
                != state.turn.current.raw()
            || u32::from_le_bytes(instance.state_blob[12..16].try_into().ok()?) != card.raw()
        {
            return None;
        }
        Some(PlayerId::new(u32::from_le_bytes(
            instance.state_blob[16..20].try_into().ok()?,
        )))
    })
}

pub(crate) fn dddfusi_sources(
    state: &crate::state::GameState,
    borrower: PlayerId,
    skill: SkillId,
    kind: CardKind,
) -> Vec<CardId> {
    state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|provider| *provider != borrower && dddfusi_policy(state, skill, *provider) != 2)
        .filter(|provider| {
            state.players.get(*provider).is_some_and(|participant| {
                participant.skills.iter().any(|owned| {
                    state.skills.get(*owned).is_some_and(|instance| {
                        instance.def.raw() == DDD_FUSI_DEF
                            && state.skill_effectively_enabled(instance)
                    })
                })
            })
        })
        .flat_map(|provider| state.hand_cards(provider))
        .filter(|card| {
            state
                .cards
                .get(*card)
                .is_some_and(|card| card.face_up && dddfusi_kind_matches(card.kind, kind))
        })
        .collect()
}

pub(crate) fn handle_activate_skill(
    tx: &mut Transaction,
    actor: PlayerId,
    skill: SkillId,
    targets: &[PlayerId],
    card_targets: &[CardId],
) -> Result<(), EffectError> {
    if tx.state().revision == 0 {
        return Err(EffectError::Illegal("not started".into()));
    }
    if tx.state().has_pending_choice() {
        return Err(EffectError::Illegal("choice pending".into()));
    }
    if actor.is_none() {
        return Err(EffectError::Illegal("bad actor".into()));
    }

    let inst = tx
        .state()
        .skills
        .get(skill)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("unknown skill".into()))?;
    if inst.owner != actor {
        return Err(EffectError::Illegal("not skill owner".into()));
    }
    if !tx.state().skill_effectively_enabled(&inst) {
        return Err(EffectError::Illegal("skill disabled".into()));
    }
    if inst.def.raw() == DDD_FENGZHENG_GLOBAL_DEF && !ddd_fengzheng_provider_exists(tx.state()) {
        return Err(EffectError::Illegal(
            "丰政 global provider unavailable".into(),
        ));
    }
    if inst.active_turn_banned {
        return Err(EffectError::Illegal(
            "active skill banned for current turn".into(),
        ));
    }

    // Prefer view-as when the skill has view_as rules and a source card is provided.
    let views = tx.state().content.view_as_defs_for_skill(inst.def);
    if !views.is_empty() {
        if card_targets.is_empty() {
            return Err(EffectError::Illegal(
                "view_as requires at least one card_target".into(),
            ));
        }
        return handle_view_as(tx, actor, skill, &inst, &views, targets, card_targets);
    }

    if tx
        .state()
        .players
        .get(actor)
        .is_none_or(|player| !player.alive || player.dying)
    {
        return Err(EffectError::Illegal(
            "active skill owner must be alive and not dying".into(),
        ));
    }

    let active = tx
        .state()
        .content
        .first_active_for_skill(inst.def)
        .ok_or_else(|| EffectError::Illegal("skill not activatable".into()))?;
    let (used_category_mask, used_targets) = inst.active_unique_state().unwrap_or_default();
    let used_play_phase_targets = inst.active_play_phase_targets().unwrap_or_default();
    let play_phase_category_mask = inst.play_phase_category_mask;
    let owner_history_category_mask = tx.state().turn.owner_card_category_mask(actor);

    validate_active_window(tx, actor, &active.window)?;
    let dynamic_use_limit = match active.uses_per_turn_source.as_deref() {
        Some("owner_hp") => Some(
            tx.state()
                .players
                .get(actor)
                .map(|player| player.hp.max(0) as u32)
                .unwrap_or(0),
        ),
        Some("owner_lost_hp_min_one") => Some(
            tx.state()
                .players
                .get(actor)
                .map(|player| (i64::from(player.max_hp) - i64::from(player.hp)).max(1) as u32)
                .unwrap_or(1),
        ),
        Some("living_rebel_count") => Some(
            tx.state()
                .players
                .iter_live()
                .filter(|player| player.alive && player.identity == crate::state::Identity::Rebel)
                .count()
                .min(u32::MAX as usize) as u32,
        ),
        Some(_) => {
            return Err(EffectError::ContentFault(
                "bad active use limit source".into(),
            ))
        }
        None => None,
    };
    if active
        .uses_per_turn
        .map(u32::from)
        .or(dynamic_use_limit)
        .is_some_and(|limit| inst.use_count >= limit)
    {
        return Err(EffectError::Illegal(
            "active skill use limit reached".into(),
        ));
    }
    if active
        .uses_per_play_phase
        .is_some_and(|limit| inst.play_phase_use_count >= u32::from(limit))
    {
        return Err(EffectError::Illegal(
            "active skill Play-phase use limit reached".into(),
        ));
    }
    if let Some((mark, count)) = active.mark_at_least {
        let available = tx
            .state()
            .players
            .get(actor)
            .and_then(|player| {
                player
                    .marks
                    .iter()
                    .find(|(definition, _)| *definition == mark)
                    .map(|(_, count)| (*count).max(0) as u32)
            })
            .unwrap_or(0);
        if available < count {
            return Err(EffectError::Illegal(
                "active skill mark requirement is not met".into(),
            ));
        }
    }
    if let Some(missing_skill) = active.owner_missing_skill {
        let has_skill = tx.state().players.get(actor).is_some_and(|player| {
            player.skills.iter().any(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def.raw() == missing_skill)
            })
        });
        if has_skill {
            return Err(EffectError::Illegal(
                "active skill requires its owner not to have the configured skill".into(),
            ));
        }
    }
    if active.hand_gt_hp {
        let hp = tx
            .state()
            .players
            .get(actor)
            .ok_or_else(|| EffectError::Illegal("unknown actor".into()))?
            .hp;
        if i64::from(tx.state().hand_count(actor)) <= i64::from(hp) {
            return Err(EffectError::Illegal(
                "active skill requires hand count above hp".into(),
            ));
        }
    }
    if active.hand_gt_max_hp {
        let max_hp = tx
            .state()
            .players
            .get(actor)
            .ok_or_else(|| EffectError::Illegal("unknown actor".into()))?
            .max_hp;
        if tx.state().hand_count(actor) <= u32::from(max_hp) {
            return Err(EffectError::Illegal(
                "active skill requires hand count above max hp".into(),
            ));
        }
    }
    if active.hand_limit_positive {
        let mut ctx = QueryContext::default();
        if !matches!(
            query(
                tx.state(),
                &QueryRequest::HandLimit { player: actor },
                &mut ctx
            ),
            QueryResponse::Int(limit) if limit > 0
        ) {
            return Err(EffectError::Illegal(
                "active skill requires a positive hand limit".into(),
            ));
        }
    }
    if active.equip_nonempty {
        let has_equipment = tx
            .state()
            .zones
            .find(ZoneKind::Equip, actor)
            .and_then(|zone| tx.state().zones.get(zone))
            .is_some_and(|zone| !zone.cards.is_empty());
        if !has_equipment {
            return Err(EffectError::Illegal(
                "active skill requires a nonempty equipment zone".into(),
            ));
        }
    }
    if active.hand_nonempty && tx.state().hand_count(actor) == 0 {
        return Err(EffectError::Illegal(
            "active skill requires a nonempty hand".into(),
        ));
    }
    if let Some(tag) = active.hand_missing_gaintag {
        let owns_tagged = tx.state().hand_cards(actor).iter().any(|card| {
            tx.state()
                .content_card_tags
                .get(card)
                .is_some_and(|tags| tags.contains(&tag))
        });
        if owns_tagged {
            return Err(EffectError::Illegal(
                "active skill gated by gaintag in hand".into(),
            ));
        }
    }
    if active.face_up
        && tx
            .state()
            .players
            .get(actor)
            .is_none_or(|player| player.turned_over)
    {
        return Err(EffectError::Illegal(
            "active skill requires its owner to be face up".into(),
        ));
    }
    if active.owner_unlinked
        && tx.state().players.get(actor).is_some_and(|player| {
            player
                .marks
                .iter()
                .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
        })
    {
        return Err(EffectError::Illegal(
            "active skill requires its owner to be unlinked".into(),
        ));
    }
    if active.once_per_game && inst.lifetime_fire_count > 0 {
        return Err(EffectError::Illegal(
            "active skill once-per-game limit reached".into(),
        ));
    }
    if let Some(targets_spec) = &active.targets {
        let mut target_max = usize::from(targets_spec.max);
        if let Some(mark) = targets_spec.max_by_owner_mark {
            target_max = target_max.min(
                tx.state()
                    .players
                    .get(actor)
                    .and_then(|player| {
                        player
                            .marks
                            .iter()
                            .find(|(definition, _)| *definition == mark)
                            .map(|(_, count)| (*count).max(0) as usize)
                    })
                    .unwrap_or(0),
            );
        }
        if targets_spec.max_by_owner_lost_hp {
            target_max = target_max.min(
                tx.state()
                    .players
                    .get(actor)
                    .map(|player| (player.max_hp as i32 - player.hp).max(0) as usize)
                    .unwrap_or(0),
            );
        }
        if targets_spec.max_by_owner_hp {
            target_max = target_max.min(
                tx.state()
                    .players
                    .get(actor)
                    .map(|player| player.hp.max(0) as usize)
                    .unwrap_or(0),
            );
        }
        let target_max = target_max as u8;
        if targets_spec.selected_cards_match_target_equip_count
            && (targets_spec.min != 1
                || targets_spec.max != 1
                || !targets_spec.has_equip
                || active.selection.is_none())
        {
            return Err(EffectError::ContentFault(
                "equipment-count card selection requires one equipped target and a card selection"
                    .into(),
            ));
        }
        validate_active_targets(
            tx,
            actor,
            targets,
            targets_spec.min,
            target_max,
            targets_spec.allow_self,
            targets_spec.lowest_hp,
            targets_spec.highest_hp,
            targets_spec.unlinked,
            targets_spec.linked,
            targets_spec.wounded,
            targets_spec.male,
            targets_spec.hp_at_least_owner,
            targets_spec.hp_gt_owner,
            targets_spec.hp_lte_owner,
            targets_spec.hp_gt,
            targets_spec.max_hp_gt,
            targets_spec.hand_gt_owner,
            targets_spec.hand_lte_owner,
            targets_spec.pair_hand_difference_lte_three,
            targets_spec.hand_lt,
            targets_spec.hand_nonempty,
            targets_spec.can_compare,
            targets_spec.can_use_on_owner.as_deref(),
            targets_spec.second_can_use_juedou_on_first,
            targets_spec.has_equip,
            targets_spec.has_discardable_equip,
            targets_spec.has_gainable_equip,
            targets_spec.has_gainable_hand_equip,
            targets_spec.in_attack_range,
            targets_spec.owner_in_target_attack_range,
            targets_spec.hand_equip_nonempty,
            targets_spec.has_discardable_hand_equip,
            targets_spec.equipment_pair_with_armor,
            if targets_spec.unused_by_skill_this_turn {
                &used_targets
            } else if targets_spec.unused_by_skill_this_play_phase {
                &used_play_phase_targets
            } else {
                &[]
            },
        )?;
    }

    if let Some(selection) = &active.selection {
        let excluded_category_mask = if selection.unused_category_by_skill_this_turn {
            used_category_mask
        } else if selection.unused_category_by_skill_this_play_phase {
            play_phase_category_mask
        } else if selection.unused_category_by_owner_history_this_turn {
            owner_history_category_mask
        } else {
            0
        };
        let choices = active_selection_cards(
            tx,
            actor,
            &selection.zone,
            selection.filter.as_deref(),
            excluded_category_mask,
        )?;
        let max = usize::from(selection.max).min(choices.len());
        let match_selected_card_count = active
            .targets
            .as_ref()
            .is_some_and(|targets| targets.match_selected_card_count);
        let target_equip_count = active
            .targets
            .as_ref()
            .filter(|targets| targets.selected_cards_match_target_equip_count)
            .and_then(|_| targets.first())
            .and_then(|target| tx.state().zones.find(ZoneKind::Equip, *target))
            .and_then(|zone| tx.state().zones.get(zone))
            .map(|zone| zone.cards.len());
        let required_count = if let Some(count) = target_equip_count {
            count
        } else if match_selected_card_count {
            targets.len()
        } else {
            usize::from(selection.min)
        };
        if required_count < usize::from(selection.min) || max < required_count {
            return Err(EffectError::Illegal(
                "not enough cards for active selection".into(),
            ));
        }
        if selection.same_color {
            let red = choices
                .iter()
                .filter(|card| active_card_is_red(tx, actor, **card))
                .count();
            let black = choices.len().saturating_sub(red);
            if red.max(black) < required_count {
                return Err(EffectError::Illegal(
                    "not enough same-color cards for active selection".into(),
                ));
            }
        }
        if selection.same_suit
            && !crate::query::effective_suit_card_groups(tx.state(), actor, &choices)
                .iter()
                .any(|group| group.len() >= required_count)
        {
            return Err(EffectError::Illegal(
                "not enough same-suit cards for active selection".into(),
            ));
        }
        if selection.distinct_categories {
            let categories = choices.iter().fold(0u8, |mask, card| {
                mask | active_card_category_bit(tx, actor, *card)
            });
            if categories.count_ones() < required_count as u32 {
                return Err(EffectError::Illegal(
                    "not enough distinct card categories for active selection".into(),
                ));
            }
        }
        if selection.distinct_suits {
            let suits = choices.iter().fold(0u8, |mask, card| {
                let suit = tx
                    .state()
                    .cards
                    .get(*card)
                    .map(|card| crate::query::effective_card_suit(tx.state(), actor, card));
                mask | match suit {
                    Some(engine_api::Suit::Spade) => 1,
                    Some(engine_api::Suit::Heart) => 2,
                    Some(engine_api::Suit::Club) => 4,
                    Some(engine_api::Suit::Diamond) => 8,
                    _ => 0,
                }
            });
            if suits.count_ones() < required_count as u32 {
                return Err(EffectError::Illegal(
                    "not enough distinct-suit cards for active selection".into(),
                ));
            }
        }
        if let Some(sum) = selection.sum_effective_number_exact {
            let exact_dynamic_count = match_selected_card_count || target_equip_count.is_some();
            let sum_min = if exact_dynamic_count {
                required_count
            } else {
                usize::from(selection.min)
            };
            let sum_max = if exact_dynamic_count {
                required_count
            } else {
                max
            };
            let has_valid_sum = if selection.same_suit {
                crate::query::effective_suit_card_groups(tx.state(), actor, &choices)
                    .iter()
                    .any(|group| {
                        !crate::query::effective_number_sum_subsets(
                            tx.state(),
                            actor,
                            group,
                            sum_min,
                            sum_max,
                            sum,
                            1,
                        )
                        .is_empty()
                    })
            } else {
                !crate::query::effective_number_sum_subsets(
                    tx.state(),
                    actor,
                    &choices,
                    sum_min,
                    sum_max,
                    sum,
                    1,
                )
                .is_empty()
            };
            if !has_valid_sum {
                return Err(EffectError::Illegal(
                    "no active card selection satisfies the required effective-number sum".into(),
                ));
            }
        }
        if card_targets.is_empty() {
            let mut payload = vec![skill.raw(), targets.len() as u32];
            payload.extend(targets.iter().map(|target| target.raw()));
            let exact_dynamic_count = match_selected_card_count || target_equip_count.is_some();
            crate::effect::open_choice(
                tx,
                actor,
                if exact_dynamic_count {
                    required_count as u8
                } else {
                    selection.min
                },
                if exact_dynamic_count {
                    required_count as u8
                } else {
                    max as u8
                },
                selection.allow_cancel,
                choices.iter().map(|card| card.raw()).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::ActiveSkillSelection,
                payload,
            )?;
            return Err(EffectError::ChoiceRequested);
        }
        validate_selected_cards(tx, actor, card_targets, selection, excluded_category_mask)?;
        if match_selected_card_count && card_targets.len() != targets.len() {
            return Err(EffectError::Illegal(
                "active target count must match selected card count".into(),
            ));
        }
        if target_equip_count.is_some_and(|count| card_targets.len() != count) {
            return Err(EffectError::Illegal(
                "selected card count must match target equipment count".into(),
            ));
        }
    } else if !card_targets.is_empty() {
        return Err(EffectError::Illegal(
            "active skill does not accept card inputs".into(),
        ));
    }

    let module = tx
        .state()
        .content
        .module(&active.module)
        .cloned()
        .ok_or_else(|| EffectError::ContentFault(format!("missing module {}", active.module)))?;

    let target0 = targets.first().copied().unwrap_or(PlayerId::NONE);
    let fuel = tx.state().resolution.effect_budget.min(10_000);
    let ctx = [actor.raw(), actor.raw(), target0.raw(), 0];
    let selected: Vec<u32> = card_targets.iter().map(|card| card.raw()).collect();
    let selected_targets: Vec<u32> = targets.iter().map(|target| target.raw()).collect();
    let outcome = crate::vm_host::run_module_with_selected_inputs(
        tx.state(),
        &module,
        skill.raw(),
        ctx,
        0,
        &selected,
        &selected_targets,
        fuel,
    )
    .map_err(|e| EffectError::ContentFault(format!("vm: {e}")))?;

    if let Some(s) = tx.state_mut().skills.get_mut(skill) {
        s.use_count = s.use_count.saturating_add(1);
        s.play_phase_use_count = s.play_phase_use_count.saturating_add(1);
        if active.once_per_game {
            s.lifetime_fire_count = s.lifetime_fire_count.saturating_add(1);
        }
    }

    let mut all = outcome.effects;
    all.push(EffectRequest::EmitRuleEvent {
        code: 110,
        payload: vec![skill.raw(), inst.def.raw(), actor.raw()],
    });
    run_effects(tx, all)?;

    // If the skill body paused on choose{}, open the choice and suspend.
    if let Some(pause) = outcome.paused {
        let module_name = active.module.clone();
        super::super::resolve_choice::open_vm_choice(
            tx,
            actor,
            &module_name,
            skill.raw(),
            ctx,
            [0; 3],
            pause,
        )?;
        return Err(EffectError::ChoiceRequested);
    }
    Ok(())
}
