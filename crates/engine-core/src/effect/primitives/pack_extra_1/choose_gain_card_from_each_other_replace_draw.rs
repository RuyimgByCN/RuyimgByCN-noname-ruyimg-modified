//! Effect primitives for `draw_then_optionally_distribute_hand_cards` and related helpers.

use super::*;

pub(crate) fn choose_gain_card_from_each_other_replace_draw(
    tx: &mut Transaction,
    player: PlayerId,
    initialized: bool,
    remaining_targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    choose_gain_card_from_each_other(tx, player, initialized, remaining_targets, skill, false)
}

pub(crate) fn choose_gain_card_from_each_other(
    tx: &mut Transaction,
    player: PlayerId,
    mut initialized: bool,
    mut remaining_targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
    turn_over_at_end: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if !initialized && !turn_over_at_end {
        if tx.state().turn.draw_phase_is_fixed() {
            return Ok(Vec::new());
        }
        tx.state_mut().turn.set_draw_phase_fixed_count(0);
    }
    if !initialized {
        let order = tx.state().players.alive_by_seat();
        remaining_targets = order
            .iter()
            .position(|&target| target == player)
            .map(|start| {
                (1..order.len())
                    .map(|offset| order[(start + offset) % order.len()])
                    .collect()
            })
            .unwrap_or_default();
        initialized = true;
    }
    let Some(target) = remaining_targets.first().copied() else {
        return if turn_over_at_end {
            turn_over(tx, player)
        } else {
            Ok(Vec::new())
        };
    };
    remaining_targets.remove(0);
    let next = if turn_over_at_end {
        EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
            player,
            initialized,
            remaining_targets,
            skill,
        }
    } else {
        EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
            player,
            initialized,
            remaining_targets,
            skill,
        }
    };
    tx.state_mut().resolution.deferred_tail_effects.push(next);
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options = crate::query::gainable_target_cards(
        tx.state(),
        player,
        target,
        TargetCardScope::HandEquipJudge,
    );
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        player.raw(),
        target.raw(),
        skill.raw(),
        options.len() as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseGainCardFromEachOtherThenTurnOver,
        payload,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_mass_damage_discard_turn_over(
    tx: &mut Transaction,
    player: PlayerId,
    mark: engine_api::MarkDefId,
    mark_count: u32,
    damage_amount: u32,
    hand_count: u32,
    mut stage: u8,
    mut index: u32,
    mut targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
    hand_equip: bool,
    turn_owner_over: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if mark.is_none() || mark_count == 0 || damage_amount == 0 || hand_count == 0 {
        return Err(EffectError::ContentFault(
            "mass damage/discard requires non-zero parameters".into(),
        ));
    }
    if stage == 0 {
        let state = tx.state();
        let instance = state
            .skills
            .get(skill)
            .ok_or_else(|| EffectError::ContentFault("mass damage active skill missing".into()))?;
        let contract = state
            .content
            .first_active_for_skill(instance.def)
            .ok_or_else(|| {
                EffectError::ContentFault("mass damage active contract missing".into())
            })?;
        let active_limit_matches = if hand_equip {
            contract.once_per_game
        } else {
            contract.uses_per_turn == Some(1)
        };
        if instance.owner != player
            || !state.skill_effectively_enabled(instance)
            || state.turn.current != player
            || state.turn.phase != Phase::Play
            || !active_limit_matches
            || contract.mark_at_least != Some((mark.raw(), mark_count))
        {
            return Err(EffectError::ContentFault(
                "mass damage/discard requires its declared active contract".into(),
            ));
        }
        let available = state
            .players
            .get(player)
            .and_then(|owner| {
                owner
                    .marks
                    .iter()
                    .find(|(definition, _)| *definition == mark.raw())
                    .map(|(_, count)| (*count).max(0) as u32)
            })
            .unwrap_or(0);
        if available < mark_count {
            return Err(EffectError::Illegal(
                "not enough marks for mass damage/discard".into(),
            ));
        }
        targets = state
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| *target != player)
            .collect();
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(if hand_equip {
                EffectRequest::ResolveMassDamageDiscardHandEquip {
                    player,
                    mark,
                    mark_count,
                    damage_amount,
                    discard_count: hand_count,
                    stage: 1,
                    index: 0,
                    targets,
                    skill,
                }
            } else {
                EffectRequest::ResolveMassDamageDiscardTurnOver {
                    player,
                    mark,
                    mark_count,
                    damage_amount,
                    hand_count,
                    stage: 1,
                    index: 0,
                    targets,
                    skill,
                }
            });
        return remove_mark(tx, player, mark.raw(), mark_count);
    }

    loop {
        if stage == if hand_equip { 3 } else { 4 } {
            return if turn_owner_over {
                turn_over(tx, player)
            } else {
                Ok(Vec::new())
            };
        }
        if stage > if hand_equip { 3 } else { 4 } {
            return Err(EffectError::ContentFault(
                "unknown mass damage/discard stage".into(),
            ));
        }
        let Some(target) = targets.get(index as usize).copied() else {
            stage += 1;
            index = 0;
            continue;
        };
        index = index.saturating_add(1);
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            continue;
        }
        let next = if hand_equip {
            EffectRequest::ResolveMassDamageDiscardHandEquip {
                player,
                mark,
                mark_count,
                damage_amount,
                discard_count: hand_count,
                stage,
                index,
                targets: targets.clone(),
                skill,
            }
        } else {
            EffectRequest::ResolveMassDamageDiscardTurnOver {
                player,
                mark,
                mark_count,
                damage_amount,
                hand_count,
                stage,
                index,
                targets: targets.clone(),
                skill,
            }
        };
        match stage {
            1 => {
                tx.state_mut().resolution.deferred_tail_effects.push(next);
                return deal_damage(
                    tx,
                    player,
                    target,
                    damage_amount,
                    false,
                    rule_api::damage_nature::NORMAL,
                    false,
                    CardId::NONE,
                    0,
                    0,
                );
            }
            2 if !hand_equip => {
                let Some(from) = tx.state().zones.find(ZoneKind::Equip, target) else {
                    continue;
                };
                let cards = tx
                    .state()
                    .zones
                    .get(from)
                    .map(|zone| zone.cards.clone())
                    .unwrap_or_default();
                if cards.is_empty() {
                    continue;
                }
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
                tx.state_mut().resolution.deferred_tail_effects.push(next);
                return move_cards_with_reason(
                    tx,
                    cards,
                    discard,
                    Some(from),
                    CardMoveReason::Discard,
                );
            }
            2 | 3 => {
                let mut options = crate::query::discardable_hand_equip_cards(tx.state(), target);
                if !hand_equip {
                    let hand_zone = tx.state().zones.find(ZoneKind::Hand, target);
                    options.retain(|card| {
                        hand_zone.is_some_and(|zone| {
                            tx.state()
                                .cards
                                .get(*card)
                                .is_some_and(|card| card.zone == zone)
                        })
                    });
                }
                let required = usize::try_from(hand_count)
                    .unwrap_or(usize::MAX)
                    .min(options.len());
                if required == 0 {
                    continue;
                }
                tx.state_mut().resolution.deferred_tail_effects.push(next);
                if required == options.len() {
                    return discard_cards(tx, target, options);
                }
                let required = u8::try_from(required).map_err(|_| {
                    EffectError::ContentFault("mass hand discard count exceeds 255".into())
                })?;
                let mut payload = vec![
                    target.raw(),
                    u32::from(required),
                    options.len().min(u32::MAX as usize) as u32,
                ];
                if hand_equip {
                    payload.push(1);
                }
                payload.extend(options.iter().map(|card| card.raw()));
                return open_choice(
                    tx,
                    target,
                    required,
                    required,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::DiscardHandForMassDamage,
                    payload,
                );
            }
            _ => unreachable!(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_mark_parity_control(
    tx: &mut Transaction,
    player: PlayerId,
    mark: engine_api::MarkDefId,
    mass_threshold: u32,
    damage_amount: u32,
    stage: u8,
    mut index: u32,
    target: PlayerId,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if mark.is_none() || mass_threshold == 0 || damage_amount == 0 {
        return Err(EffectError::ContentFault(
            "mark parity control requires non-zero parameters".into(),
        ));
    }
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill) {
            return Err(EffectError::ContentFault(
                "mark parity control requires its originating skill".into(),
            ));
        }
        let options = tx.state().players.alive_by_seat();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            true,
            options.iter().map(|target| target.raw()).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::ChooseMarkParityControlTarget,
            vec![
                player.raw(),
                mark.raw(),
                mass_threshold,
                damage_amount,
                skill.raw(),
            ],
        );
    }
    if stage == 1 {
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            return Ok(Vec::new());
        }
        let marks = tx
            .state()
            .players
            .get(player)
            .and_then(|owner| {
                owner
                    .marks
                    .iter()
                    .find(|(definition, _)| *definition == mark.raw())
                    .map(|(_, count)| (*count).max(0) as u32)
            })
            .unwrap_or(0);
        let next = EffectRequest::ResolveMarkParityControl {
            player,
            mark,
            mass_threshold,
            damage_amount,
            stage: if marks % 2 == 1 { 3 } else { 2 },
            index: 0,
            target,
            targets: Vec::new(),
            skill,
        };
        tx.state_mut().resolution.deferred_tail_effects.push(next);
        if marks % 2 == 1 {
            return deal_damage(
                tx,
                player,
                target,
                damage_amount,
                false,
                rule_api::damage_nature::NORMAL,
                false,
                CardId::NONE,
                0,
                0,
            );
        }
        let linked = tx
            .state()
            .players
            .get(target)
            .unwrap()
            .marks
            .iter()
            .any(|(definition, count)| *definition == crate::content::MARK_CHAIN && *count > 0);
        return if linked {
            Ok(Vec::new())
        } else {
            add_mark(tx, target, crate::content::MARK_CHAIN, 1)
        };
    }
    if stage == 2 {
        let options = crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            TargetCardScope::HandEquipJudge,
        );
        if options.is_empty() {
            return resolve_mark_parity_control(
                tx,
                player,
                mark,
                mass_threshold,
                damage_amount,
                3,
                0,
                target,
                Vec::new(),
                skill,
            );
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::DiscardMarkParityControlTargetCard,
            vec![
                player.raw(),
                target.raw(),
                mark.raw(),
                mass_threshold,
                damage_amount,
                skill.raw(),
            ],
        );
    }
    if stage == 3 {
        let marks = tx
            .state()
            .players
            .get(player)
            .and_then(|owner| {
                owner
                    .marks
                    .iter()
                    .find(|(definition, _)| *definition == mark.raw())
                    .map(|(_, count)| (*count).max(0) as u32)
            })
            .unwrap_or(0);
        if marks <= mass_threshold
            || !tx
                .state()
                .players
                .get(player)
                .is_some_and(|owner| owner.alive)
        {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::ConfirmMarkParityMassDamage,
            vec![
                player.raw(),
                mark.raw(),
                mass_threshold,
                damage_amount,
                skill.raw(),
            ],
        );
    }
    if stage != 4 {
        return Err(EffectError::ContentFault(
            "unknown mark parity control stage".into(),
        ));
    }
    let Some(target) = targets.get(index as usize).copied() else {
        return Ok(Vec::new());
    };
    index = index.saturating_add(1);
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return resolve_mark_parity_control(
            tx,
            player,
            mark,
            mass_threshold,
            damage_amount,
            4,
            index,
            PlayerId::NONE,
            targets,
            skill,
        );
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveMarkParityControl {
            player,
            mark,
            mass_threshold,
            damage_amount,
            stage: 4,
            index,
            target: PlayerId::NONE,
            targets,
            skill,
        });
    deal_damage(
        tx,
        player,
        target,
        damage_amount,
        false,
        rule_api::damage_nature::NORMAL,
        false,
        CardId::NONE,
        0,
        0,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_selected_equipment_discard_then_choose_damage(
    tx: &mut Transaction,
    player: PlayerId,
    damage_amount: u32,
    nature: u8,
    mut stage: u8,
    mut index: u32,
    mut targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if damage_amount == 0 || nature > rule_api::damage_nature::THUNDER {
        return Err(EffectError::ContentFault(
            "selected equipment discard/damage has invalid parameters".into(),
        ));
    }
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill) || targets.is_empty() {
            return Err(EffectError::ContentFault(
                "selected equipment discard/damage requires active targets".into(),
            ));
        }
        let order = tx.state().players.alive_by_seat();
        let mut seen = std::collections::BTreeSet::new();
        if targets.iter().any(|&target| {
            !order.contains(&target)
                || !seen.insert(target.raw())
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
        }) {
            return Err(EffectError::Illegal(
                "selected equipment discard/damage targets are invalid".into(),
            ));
        }
        targets.sort_by_key(|target| {
            order
                .iter()
                .position(|candidate| candidate == target)
                .expect("validated selected target")
        });
        stage = 1;
        index = 0;
    }
    if stage == 1 {
        while let Some(target) = targets.get(index as usize).copied() {
            index = index.saturating_add(1);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                continue;
            }
            let Some(from) = tx.state().zones.find(ZoneKind::Equip, target) else {
                continue;
            };
            let cards = tx
                .state()
                .zones
                .get(from)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default();
            if cards.is_empty() {
                continue;
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage {
                    player,
                    damage_amount,
                    nature,
                    stage: 1,
                    index,
                    targets,
                    skill,
                },
            );
            return move_cards_with_reason(tx, cards, discard, Some(from), CardMoveReason::Discard);
        }
        stage = 2;
    }
    if stage != 2 {
        return Err(EffectError::ContentFault(
            "unknown selected equipment discard/damage stage".into(),
        ));
    }
    let options: Vec<_> = targets
        .iter()
        .copied()
        .filter(|target| {
            tx.state()
                .players
                .get(*target)
                .is_some_and(|participant| participant.alive)
        })
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        player.raw(),
        damage_amount,
        u32::from(nature),
        skill.raw(),
        options.len() as u32,
    ];
    payload.extend(options.iter().map(|target| target.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseSelectedTargetDamage,
        payload,
    )
}
