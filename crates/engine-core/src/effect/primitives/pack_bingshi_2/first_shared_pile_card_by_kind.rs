//! Effect primitives for `first_shared_pile_card_by_kind` and related helpers.

use super::*;

pub(crate) fn first_shared_pile_card_by_kind(
    tx: &Transaction,
    kind: CardKind,
) -> Result<Option<(ZoneId, CardId)>, EffectError> {
    for zone_kind in [ZoneKind::DrawPile, ZoneKind::DiscardPile] {
        let from = tx
            .state()
            .zones
            .find(zone_kind, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("shared card zone missing".into()))?;
        if let Some(card) = tx.state().zones.get(from).and_then(|zone| {
            zone.cards.iter().rev().copied().find(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.kind == kind)
            })
        }) {
            return Ok(Some((from, card)));
        }
    }
    Ok(None)
}

pub(crate) fn gain_equipment_from_draw_pile(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal("bad player".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let draw_pile = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
    let equipment = tx.state().zones.get(draw_pile).and_then(|zone| {
        zone.cards.iter().rev().copied().find(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.kind.is_equip())
        })
    });
    let Some(equipment) = equipment else {
        return Ok(Vec::new());
    };
    move_cards(tx, vec![equipment], hand, Some(draw_pile))
}

pub(crate) fn give_all_hand_recover_to_one(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    ready_to_recover: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let owner_alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive);
    let target_alive = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive);
    if player == target || !owner_alive || !target_alive {
        return Ok(Vec::new());
    }
    if ready_to_recover {
        let amount = tx
            .state()
            .players
            .get(target)
            .map(|participant| (1_i32 - participant.hp).max(0) as u32)
            .unwrap_or(0);
        return if amount == 0 {
            Ok(Vec::new())
        } else {
            recover_hp(tx, player, target, amount)
        };
    }
    let events = give_all_hand(tx, player, target)?;
    if events.is_empty() {
        return Ok(events);
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::GiveAllHandRecoverToOne {
            player,
            target,
            ready_to_recover: true,
        });
    Ok(events)
}

pub(crate) fn give_selected_recover_after_phase_total_two(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
    ready_to_recover: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ready_to_recover {
        return if tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        {
            recover_hp(tx, player, player, 1)
        } else {
            Ok(Vec::new())
        };
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
        || cards.is_empty()
    {
        return Err(EffectError::Illegal("invalid Rende gift context".into()));
    }
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Rende owner has no hand".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("Rende target has no hand".into()))?;
    let mut seen = std::collections::BTreeSet::new();
    if cards.iter().any(|card| {
        !seen.insert(card.raw())
            || !tx
                .state()
                .cards
                .get(*card)
                .is_some_and(|state| state.owner == player && state.zone == from)
    }) {
        return Err(EffectError::Illegal(
            "Rende cards must be distinct current hand cards".into(),
        ));
    }
    let previous = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.play_phase_fire_count)
        .ok_or_else(|| EffectError::Illegal("Rende skill is missing".into()))?;
    let current = previous.saturating_add(cards.len() as u32);
    tx.state_mut()
        .skills
        .get_mut(skill)
        .expect("validated Rende skill")
        .play_phase_fire_count = current;
    if previous < 2 && current >= 2 {
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo {
                player,
                target,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
                ready_to_recover: true,
            },
        );
    }
    give_cards(tx, cards, to, from)
}

pub(crate) fn choose_other_hand_targets_gain_one_replace_draw(
    tx: &mut Transaction,
    player: PlayerId,
    targets: Vec<PlayerId>,
    mut index: u32,
    selected: CardId,
    skill: engine_api::SkillId,
    reduce_from_draw_count: Option<u32>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let fixed_draw_count = reduce_from_draw_count
        .map(|count| count.saturating_sub(targets.len() as u32))
        .unwrap_or(0);
    let continuing =
        !targets.is_empty() && tx.state().turn.draw_phase_fixed_count() == Some(fixed_draw_count);
    if (!continuing && !originating_skill_is_enabled(tx, player, skill))
        || (!continuing && tx.state().turn.draw_phase_is_fixed())
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Draw
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("invalid Tuxi draw context".into()));
    }

    if targets.is_empty() {
        let draw_count = reduce_from_draw_count.map_or(0, |captured| {
            if captured == 0 {
                crate::query::draw_count_for(tx.state(), player).saturating_add(
                    tx.state()
                        .turn
                        .flag(turn_flag::CURRENT_DRAW_COUNT_BONUS)
                        .max(0) as u32,
                )
            } else {
                captured
            }
        });
        if reduce_from_draw_count.is_some() && draw_count == 0 {
            return Ok(Vec::new());
        }
        let options: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&target| target != player && tx.state().hand_count(target) > 0)
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        let stage = if reduce_from_draw_count.is_some() {
            2
        } else {
            0
        };
        let mut payload = vec![stage, player.raw(), skill.raw()];
        if reduce_from_draw_count.is_some() {
            payload.push(draw_count);
        }
        payload.push(options.len() as u32);
        payload.extend(options.iter().map(|target| target.raw()));
        return open_choice(
            tx,
            player,
            1,
            options
                .len()
                .min(if reduce_from_draw_count.is_some() {
                    draw_count as usize
                } else {
                    2
                })
                .min(u8::MAX as usize) as u8,
            false,
            options.iter().map(|target| target.raw()).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::TuxiGainOtherHandsReplaceDraw,
            payload,
        );
    }

    tx.state_mut()
        .turn
        .set_draw_phase_fixed_count(fixed_draw_count);

    if selected.is_some() {
        let target = targets
            .get(index as usize)
            .copied()
            .ok_or_else(|| EffectError::Illegal("Tuxi target index changed".into()))?;
        let from = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::Illegal("Tuxi target hand is missing".into()))?;
        if !tx
            .state()
            .cards
            .get(selected)
            .is_some_and(|card| card.owner == target && card.zone == from)
        {
            return Err(EffectError::Illegal(
                "Tuxi selection is not a current target hand card".into(),
            ));
        }
        let to = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("Tuxi owner hand is missing".into()))?;
        tx.state_mut().resolution.deferred_tail_effects.push(
            if let Some(draw_count) = reduce_from_draw_count {
                EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
                    player,
                    targets,
                    index: index.saturating_add(1),
                    selected: CardId::NONE,
                    draw_count,
                    skill,
                }
            } else {
                EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
                    player,
                    targets,
                    index: index.saturating_add(1),
                    selected: CardId::NONE,
                    skill,
                }
            },
        );
        return move_cards(tx, vec![selected], to, Some(from));
    }

    while let Some(target) = targets.get(index as usize).copied() {
        let mut options = tx.state().hand_cards(target);
        options.sort_by_key(|card| card.raw());
        if options.is_empty() {
            index = index.saturating_add(1);
            continue;
        }
        let stage = if reduce_from_draw_count.is_some() {
            3
        } else {
            1
        };
        let mut payload = vec![stage, player.raw(), skill.raw()];
        if let Some(draw_count) = reduce_from_draw_count {
            payload.push(draw_count);
        }
        payload.extend([index, targets.len() as u32]);
        payload.extend(targets.iter().map(|target| target.raw()));
        payload.push(options.len() as u32);
        payload.extend(options.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.iter().map(|card| card.raw()).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::TuxiGainOtherHandsReplaceDraw,
            payload,
        );
    }
    Ok(Vec::new())
}

pub(crate) fn gain_one_hand_from_selected_targets(
    tx: &mut Transaction,
    player: PlayerId,
    targets: Vec<PlayerId>,
    mut index: u32,
    selected: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("invalid Cuorui play context".into()));
    }

    if index == 0 && selected.is_none() {
        let hp = tx
            .state()
            .players
            .get(player)
            .map(|participant| participant.hp.max(0) as usize)
            .unwrap_or(0);
        let mut seen = std::collections::BTreeSet::new();
        if targets.is_empty()
            || targets.len() > hp
            || targets.iter().any(|&target| {
                target == player
                    || !seen.insert(target.raw())
                    || !tx
                        .state()
                        .players
                        .get(target)
                        .is_some_and(|participant| participant.alive)
                    || tx.state().hand_count(target) == 0
            })
        {
            return Err(EffectError::Illegal("invalid Cuorui targets".into()));
        }
    }

    if selected.is_some() {
        let target = targets
            .get(index as usize)
            .copied()
            .ok_or_else(|| EffectError::Illegal("Cuorui target index changed".into()))?;
        let from = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::Illegal("Cuorui target hand is missing".into()))?;
        if !tx
            .state()
            .cards
            .get(selected)
            .is_some_and(|card| card.owner == target && card.zone == from)
        {
            return Err(EffectError::Illegal(
                "Cuorui selection is not a current target hand card".into(),
            ));
        }
        let to = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("Cuorui owner hand is missing".into()))?;
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::GainOneHandFromSelectedTargets {
                player,
                targets,
                index: index.saturating_add(1),
                selected: CardId::NONE,
                skill,
            },
        );
        return move_cards(tx, vec![selected], to, Some(from));
    }

    while let Some(target) = targets.get(index as usize).copied() {
        let mut options = tx.state().hand_cards(target);
        options.sort_by_key(|card| card.raw());
        if options.is_empty() {
            index = index.saturating_add(1);
            continue;
        }
        let mut payload = vec![player.raw(), skill.raw(), index, targets.len() as u32];
        payload.extend(targets.iter().map(|target| target.raw()));
        payload.push(options.len() as u32);
        payload.extend(options.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.iter().map(|card| card.raw()).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::GainOneHandFromSelectedTargets,
            payload,
        );
    }
    Ok(Vec::new())
}

pub(crate) fn distribute_top_two_cards(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    selected: Vec<CardId>,
    target: PlayerId,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !ready_to_choose {
        if !originating_skill_is_enabled(tx, player, skill) {
            return Err(EffectError::Illegal("invalid Yiji skill context".into()));
        }
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("Yiji draw pile is missing".into()))?;
        if tx
            .state()
            .zones
            .get(draw)
            .map_or(0, |zone| zone.cards.len())
            < 2
        {
            reshuffle_discard_into_draw(tx)?;
        }
        let zone = tx
            .state()
            .zones
            .get(draw)
            .ok_or_else(|| EffectError::EngineFault("Yiji draw pile disappeared".into()))?;
        let take = zone.cards.len().min(2);
        if take == 0 {
            return Ok(Vec::new());
        }
        let cards = zone.cards[zone.cards.len() - take..].to_vec();
        let processing = tx
            .state()
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("Yiji processing zone is missing".into()))?;
        tx.state_mut().resolution.deferred_effects.insert(
            0,
            EffectRequest::DistributeTopTwoCards {
                player,
                cards: cards.clone(),
                selected: Vec::new(),
                target: PlayerId::NONE,
                skill,
                ready_to_choose: true,
            },
        );
        let mut events = move_cards(tx, cards.clone(), processing, Some(draw))?;
        events.push(DomainEvent::CardsRevealed {
            cards,
            to: Vec::new(),
        });
        return Ok(events);
    }

    if cards.is_empty() {
        return Ok(Vec::new());
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("Yiji processing zone is missing".into()))?;
    if cards.iter().any(|card| {
        !tx.state()
            .cards
            .get(*card)
            .is_some_and(|state| state.zone == processing)
    }) {
        return Err(EffectError::Illegal(
            "Yiji remaining cards left processing".into(),
        ));
    }

    if selected.is_empty() {
        if cards.len() == 1 {
            return distribute_top_two_cards(
                tx,
                player,
                cards.clone(),
                cards,
                PlayerId::NONE,
                skill,
                true,
            );
        }
        let mut payload = vec![0, player.raw(), skill.raw(), cards.len() as u32];
        payload.extend(cards.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            cards.len() as u8,
            false,
            cards.iter().map(|card| card.raw()).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::YijiDistributeTopCards,
            payload,
        );
    }

    let mut seen = std::collections::BTreeSet::new();
    if selected.is_empty()
        || selected
            .iter()
            .any(|card| !seen.insert(card.raw()) || !cards.contains(card))
    {
        return Err(EffectError::Illegal(
            "Yiji subset must contain distinct remaining cards".into(),
        ));
    }
    if target.is_none() {
        let options = tx.state().players.alive_by_seat();
        let mut payload = vec![1, player.raw(), skill.raw(), cards.len() as u32];
        payload.extend(cards.iter().map(|card| card.raw()));
        payload.push(selected.len() as u32);
        payload.extend(selected.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.iter().map(|target| target.raw()).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::YijiDistributeTopCards,
            payload,
        );
    }
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Yiji recipient is no longer alive".into(),
        ));
    }
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("Yiji recipient hand is missing".into()))?;
    let remaining: Vec<_> = cards
        .iter()
        .copied()
        .filter(|card| !selected.contains(card))
        .collect();
    tx.state_mut().resolution.deferred_effects.insert(
        0,
        EffectRequest::DistributeTopTwoCards {
            player,
            cards: remaining,
            selected: Vec::new(),
            target: PlayerId::NONE,
            skill,
            ready_to_choose: true,
        },
    );
    move_cards(tx, selected, to, Some(processing))
}

pub(crate) fn give_all_hand(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let owner_alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive);
    let target_alive = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive);
    if player == target || !owner_alive || !target_alive {
        return Ok(Vec::new());
    }
    let Some(from) = tx.state().zones.find(ZoneKind::Hand, player) else {
        return Ok(Vec::new());
    };
    let cards = tx
        .state()
        .zones
        .get(from)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let Some(to) = tx.state().zones.find(ZoneKind::Hand, target) else {
        return Ok(Vec::new());
    };
    give_cards(tx, cards, to, from)
}

pub(crate) fn choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
    ready_to_check: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let owner_alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive);
    if !owner_alive {
        return Ok(Vec::new());
    }
    if ready_to_check {
        let Some(target_state) = tx.state().players.get(target) else {
            return Ok(Vec::new());
        };
        let hand_count = i32::try_from(tx.state().hand_count(target)).unwrap_or(i32::MAX);
        return if hand_count != target_state.hp {
            super::super::run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: 0,
                }],
            )
            .map(|_| Vec::new())
        } else {
            Ok(Vec::new())
        };
    }
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if target.is_none() {
        let targets: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&target| {
                !crate::query::discardable_target_cards(
                    tx.state(),
                    target,
                    target,
                    rule_api::TargetCardScope::HandEquip,
                )
                .is_empty()
            })
            .map(PlayerId::raw)
            .collect();
        if targets.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets,
            ChoiceKind::SelectTargets,
            ContinuationKind::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp,
            vec![0, player.raw(), skill.raw()],
        );
    }
    let options = crate::query::discardable_target_cards(
        tx.state(),
        target,
        target,
        rule_api::TargetCardScope::HandEquip,
    );
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
        || options.is_empty()
    {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp,
        vec![1, player.raw(), target.raw(), skill.raw()],
    )
}
