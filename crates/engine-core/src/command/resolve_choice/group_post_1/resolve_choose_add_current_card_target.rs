//! Command operations for `resolve_force_others_use_sha_on_nearest_or_lose_hp` and related helpers.

use super::*;

pub(crate) fn resolve_choose_add_current_card_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let card = CardId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    if observer != player || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one legal additional card target".into(),
        ));
    }
    let target = PlayerId::new(selections[0]);
    let Some(extra_targets) =
        crate::command::play::current_card_use_enhancement_extra_targets(tx, player, card)?
    else {
        return Err(EffectError::Illegal(
            "additional card target context changed".into(),
        ));
    };
    if !extra_targets.contains(&target) {
        return Err(EffectError::Illegal(
            "selected card target is no longer legal".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::CommitAddSkillBoundCardTarget {
            player,
            card,
            target,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_add_or_remove_current_card_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() == Some(&2) {
        return resolve_choose_current_trick_targets_to_nullify(
            tx, observer, payload, selections, cancelled,
        );
    }
    let stage = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let card = CardId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "current-card target adjustment context changed".into(),
        ));
    }
    if cancelled {
        return run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).map(|_| ());
    }
    let (add_targets, remove_targets) =
        crate::command::play::current_card_target_adjustments(tx, player, card)?;
    if stage == 0 {
        if selections.len() != 1 || !matches!(selections[0], 0 | 1) {
            return Err(EffectError::Illegal(
                "choose add-target or remove-target mode".into(),
            ));
        }
        let mode = selections[0];
        let targets = if mode == 0 {
            add_targets
        } else {
            remove_targets
        };
        if targets.is_empty() {
            return Err(EffectError::Illegal(
                "selected target-adjustment mode is no longer legal".into(),
            ));
        }
        return crate::effect::open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets.into_iter().map(PlayerId::raw).collect(),
            engine_api::ChoiceKind::SelectTargets,
            ContinuationKind::ChooseAddOrRemoveCurrentCardTarget,
            vec![1, player.raw(), card.raw(), skill.raw(), mode],
        )
        .map(|_| ());
    }
    if stage != 1 || payload.len() != 5 || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one current-card target".into(),
        ));
    }
    let mode = payload[4];
    let target = PlayerId::new(selections[0]);
    let legal = if mode == 0 {
        &add_targets
    } else if mode == 1 {
        &remove_targets
    } else {
        return Err(EffectError::Illegal("bad target-adjustment mode".into()));
    };
    if !legal.contains(&target) {
        return Err(EffectError::Illegal(
            "selected current-card target is no longer legal".into(),
        ));
    }
    if mode == 0 {
        return run_effects(
            tx,
            vec![
                EffectRequest::RemoveSkill { skill },
                EffectRequest::CommitAddSkillBoundCardTarget {
                    player,
                    card,
                    target,
                    skill,
                },
            ],
        )
        .map(|_| ());
    }
    crate::command::play::adjust_saved_card_target(tx, player, card, target, false)?;
    run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).map(|_| ())
}

pub(crate) fn resolve_choose_current_trick_targets_to_nullify(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [2, player, source, card, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad current-trick target nullification payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let source = PlayerId::new(*source);
    let card = CardId::new(*card);
    let skill = engine_api::SkillId::new(*skill);
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "current-trick target nullification context changed".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    let kind = tx
        .state()
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(tx.state(), source, card));
    let (_, legal) = crate::command::play::current_card_target_adjustments(tx, source, card)?;
    let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
    if !kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick())
        || selected.is_empty()
        || selected.len() > legal.len()
        || selected.iter().any(|target| !legal.contains(target))
        || (0..selected.len()).any(|index| selected[index + 1..].contains(&selected[index]))
    {
        return Err(EffectError::Illegal(
            "selected current-trick targets are no longer legal".into(),
        ));
    }
    crate::command::play::exclude_all_saved_card_targets(tx, source, card, &selected)?;
    run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).map(|_| ())
}

pub(crate) fn resolve_choose_discard_matching_equip_color_add_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill_def = engine_api::SkillDefId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "matching-color discard chooser mismatch".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "select exactly one matching-color non-basic discard cost".into(),
        ));
    }
    if player == target
        || tx.state().turn.current != target
        || tx.state().turn.phase != crate::state::Phase::Play
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "matching-color discard context changed".into(),
        ));
    }
    let cost = CardId::new(selections[0]);
    if !crate::query::discardable_non_basic_matching_equip_color_cards(tx.state(), player)
        .contains(&cost)
    {
        return Err(EffectError::Illegal(
            "matching-color discard cost is no longer legal".into(),
        ));
    }
    let color_mask = crate::query::effective_card_color_mask(tx.state(), player, cost);
    run_effects(
        tx,
        vec![
            EffectRequest::DiscardCards {
                player,
                cards: vec![cost],
            },
            EffectRequest::AddCardColorLockSkill {
                player: target,
                source: player,
                color_mask,
                skill_def,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_draw_or_discard_add_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let draw_skill_def = engine_api::SkillDefId::new(*payload.get(2).unwrap_or(&0));
    let discard_skill_def = engine_api::SkillDefId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    let three_way = payload.get(5).copied().unwrap_or(0) != 0;
    if observer != player {
        return Err(EffectError::Illegal(
            "draw-or-discard branch must be chosen by the skill owner".into(),
        ));
    }
    let state = tx.state();
    if state.turn.current != player
        || state.turn.phase != crate::state::Phase::Play
        || !state.skills.get(skill).is_some_and(|instance| {
            instance.owner == player && state.skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "draw-or-discard skill context changed".into(),
        ));
    }
    if cancelled {
        return if three_way && stage == 0 {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "this draw-or-discard choice cannot be cancelled".into(),
            ))
        };
    }

    match (stage, selections) {
        (0, [1]) if three_way => {
            run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }]).map(|_| ())
        }
        (0, [2]) if three_way => run_effects(
            tx,
            vec![
                EffectRequest::DrawCards { player, count: 2 },
                EffectRequest::AddSkill {
                    player,
                    skill_def: draw_skill_def,
                },
            ],
        )
        .map(|_| ()),
        (0, [1]) => run_effects(
            tx,
            vec![
                EffectRequest::AddSkill {
                    player,
                    skill_def: draw_skill_def,
                },
                EffectRequest::DrawCards { player, count: 1 },
            ],
        )
        .map(|_| ()),
        (0, [branch]) if *branch == if three_way { 3 } else { 2 } => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return Err(EffectError::Illegal(
                    "discard branch is no longer available".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ChooseDrawOrDiscardAddSkill,
                vec![
                    1,
                    player.raw(),
                    draw_skill_def.raw(),
                    discard_skill_def.raw(),
                    skill.raw(),
                    u32::from(three_way),
                ],
            )
            .map(|_| ())
        }
        (1, [card]) => {
            let card = CardId::new(*card);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card) {
                return Err(EffectError::Illegal(
                    "discard cost is no longer a legal hand or equipment card".into(),
                ));
            }
            let effects = if three_way {
                vec![
                    EffectRequest::DiscardCards {
                        player,
                        cards: vec![card],
                    },
                    EffectRequest::AddSkill {
                        player,
                        skill_def: discard_skill_def,
                    },
                ]
            } else {
                vec![
                    EffectRequest::AddSkill {
                        player,
                        skill_def: discard_skill_def,
                    },
                    EffectRequest::DiscardCards {
                        player,
                        cards: vec![card],
                    },
                ]
            };
            run_effects(tx, effects).map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "choose one valid draw-or-discard branch or discard cost".into(),
        )),
    }
}

pub(crate) fn resolve_choose_draw_or_recast_add_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let draw_skill_def = engine_api::SkillDefId::new(*payload.get(2).unwrap_or(&0));
    let recast_skill_def = engine_api::SkillDefId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "draw-or-recast branch must be resolved by the skill owner".into(),
        ));
    }
    if cancelled {
        return if stage == 0 && selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "only the initial draw-or-recast choice can be cancelled".into(),
            ))
        };
    }
    let state = tx.state();
    if state.turn.current != player
        || state.turn.phase != crate::state::Phase::Play
        || !state.skills.get(skill).is_some_and(|instance| {
            instance.owner == player && state.skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "draw-or-recast skill context changed".into(),
        ));
    }

    match (stage, selections) {
        (0, [1]) => run_effects(
            tx,
            vec![
                EffectRequest::AddSkill {
                    player,
                    skill_def: draw_skill_def,
                },
                EffectRequest::DrawCards { player, count: 1 },
            ],
        )
        .map(|_| ()),
        (0, [2]) => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return Err(EffectError::Illegal(
                    "recast branch is no longer available".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ChooseDrawOrRecastAddSkill,
                vec![
                    1,
                    player.raw(),
                    draw_skill_def.raw(),
                    recast_skill_def.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        (1, [card]) => {
            let card = CardId::new(*card);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card) {
                return Err(EffectError::Illegal(
                    "recast card is no longer in hand or equipment".into(),
                ));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::AddSkill {
                        player,
                        skill_def: recast_skill_def,
                    },
                    EffectRequest::RecastCards {
                        player,
                        cards: vec![card],
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "choose one valid draw-or-recast branch or recast card".into(),
        )),
    }
}

pub(crate) fn resolve_choose_draw_or_discard_target_card_record_sha(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad draw-or-discard Sha continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let card = CardId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != player {
        return Err(EffectError::Illegal(
            "draw-or-discard Sha branch must be resolved by its user".into(),
        ));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled draw-or-discard Sha branch selected an option".into(),
            ))
        };
    }
    let state = tx.state();
    if ![player, target].into_iter().all(|participant| {
        state
            .players
            .get(participant)
            .is_some_and(|participant| participant.alive)
    }) || !state.skills.get(skill).is_some_and(|instance| {
        instance.owner == player && state.skill_effectively_enabled(instance)
    }) || !state.resolution.sha_target_resolving(player, card, target)
    {
        return Err(EffectError::Illegal(
            "draw-or-discard Sha context changed".into(),
        ));
    }

    let effect = match selections {
        [1] => EffectRequest::DrawCardsFromSkill {
            player,
            count: 1,
            skill,
        },
        [2] if !crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquip,
        )
        .is_empty() =>
        {
            EffectRequest::ChooseDiscardTargetCard {
                player,
                target,
                scope: rule_api::TargetCardScope::HandEquip,
            }
        }
        _ => {
            return Err(EffectError::Illegal(
                "choose one available draw-or-discard Sha branch".into(),
            ));
        }
    };
    if !tx
        .state_mut()
        .resolution
        .record_sha_skill_target(player, card, skill, target)
    {
        return Err(EffectError::Illegal(
            "resolving Sha target disappeared".into(),
        ));
    }
    run_effects(tx, vec![effect]).map(|_| ())
}

pub(crate) fn resolve_reveal_hand_choose_discard_color(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, offered_color_mask] = payload else {
        return Err(EffectError::EngineFault(
            "bad reveal-hand color continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let [color] = selections else {
        return Err(EffectError::Illegal(
            "choose exactly one revealed hand color".into(),
        ));
    };
    if observer != player || cancelled {
        return Err(EffectError::Illegal(
            "revealed hand color requires a forced choice by that player".into(),
        ));
    }
    if !matches!(*color, 0b01 | 0b10) || offered_color_mask & color == 0 {
        return Err(EffectError::Illegal(
            "selected hand color was not offered".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "revealed hand color player is no longer alive".into(),
        ));
    }
    let discardable: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .collect();
    let cards: Vec<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .filter(|card| {
            discardable.contains(card)
                && u32::from(crate::query::effective_card_color_mask(
                    tx.state(),
                    player,
                    *card,
                )) == *color
        })
        .collect();
    if cards.is_empty() {
        return Ok(());
    }
    run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).map(|_| ())
}

pub(crate) fn resolve_choose_discard_target_cards_recover_same_category(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled || payload.len() < 6 {
        return Err(EffectError::Illegal(
            "target discard-and-recover choice cannot be cancelled".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let count = payload[2];
    let amount = payload[3];
    let skill = engine_api::SkillId::new(payload[4]);
    let option_count = payload[5] as usize;
    let required = usize::try_from(count)
        .unwrap_or(usize::MAX)
        .min(option_count);
    if observer != player
        || count == 0
        || required == 0
        || payload.len() != 6 + option_count
        || selections.len() != required
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "invalid target discard-and-recover selection".into(),
        ));
    }
    let original: std::collections::BTreeSet<_> = payload[6..].iter().copied().collect();
    let current: std::collections::BTreeSet<_> = crate::query::discardable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquip,
    )
    .into_iter()
    .map(CardId::raw)
    .collect();
    let mut selected = std::collections::BTreeSet::new();
    if !selections
        .iter()
        .all(|card| selected.insert(*card) && original.contains(card) && current.contains(card))
    {
        return Err(EffectError::Illegal(
            "target discard card is duplicated or no longer legal".into(),
        ));
    }
    let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
    let same_category =
        crate::effect::target_discard_cards_share_category(tx.state(), target, &cards);
    if required == usize::try_from(count).unwrap_or(usize::MAX) && amount > 0 && same_category {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::RecoverHp {
                source: player,
                player,
                amount,
            });
    }
    run_effects(
        tx,
        vec![EffectRequest::DiscardTargetCards {
            actor: player,
            target,
            cards,
            scope: rule_api::TargetCardScope::HandEquip,
        }],
    )
    .map(|_| ())
}
