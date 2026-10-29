//! Effect primitives for `first_shared_pile_card_by_kind` and related helpers.

use super::*;

pub(crate) fn choose_give_one_hand_equip_or_recover(
    tx: &mut Transaction,
    owner: PlayerId,
    giver: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let alive = |player| {
        tx.state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    };
    let target_dying =
        tx.state().players.get(target).is_some_and(|participant| {
            participant.alive && participant.dying && participant.hp <= 0
        });
    if !originating_skill_is_enabled(tx, owner, skill)
        || !alive(owner)
        || !alive(giver)
        || !target_dying
    {
        return Ok(Vec::new());
    }

    let hand = tx.state().zones.find(ZoneKind::Hand, giver);
    let equip = tx.state().zones.find(ZoneKind::Equip, giver);
    let mut options: Vec<_> = [hand, equip]
        .into_iter()
        .flatten()
        .flat_map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|state| state.cards.clone())
                .unwrap_or_default()
        })
        .collect();
    options.sort_by_key(|card| card.raw());
    options.dedup();
    if options.is_empty() {
        return recover_hp(tx, PlayerId::NONE, target, 1);
    }

    let mut payload = vec![
        owner.raw(),
        giver.raw(),
        target.raw(),
        skill.raw(),
        options.len() as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        giver,
        0,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseGiveOneHandEquipOrRecover,
        payload,
    )
}

pub(crate) fn choose_give_two_different_colors_or_recover(
    tx: &mut Transaction,
    owner: PlayerId,
    giver: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let alive = |player| {
        tx.state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    };
    let target_dying =
        tx.state().players.get(target).is_some_and(|participant| {
            participant.alive && participant.dying && participant.hp <= 0
        });
    if !originating_skill_is_enabled(tx, owner, skill)
        || !alive(owner)
        || !alive(giver)
        || !target_dying
    {
        return Ok(Vec::new());
    }

    let hand = tx.state().zones.find(ZoneKind::Hand, giver);
    let equip = tx.state().zones.find(ZoneKind::Equip, giver);
    let mut options: Vec<_> = [hand, equip]
        .into_iter()
        .flatten()
        .flat_map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|state| state.cards.clone())
                .unwrap_or_default()
        })
        .collect();
    options.sort_by_key(|card| card.raw());
    options.dedup();
    let has_valid_pair = options.iter().enumerate().any(|(index, first)| {
        let color = crate::query::effective_card_color_mask(tx.state(), giver, *first);
        options[index + 1..].iter().any(|second| {
            crate::query::effective_card_color_mask(tx.state(), giver, *second) != color
        })
    });
    if !has_valid_pair {
        return recover_hp(tx, PlayerId::NONE, target, 1);
    }

    let mut payload = vec![
        owner.raw(),
        giver.raw(),
        target.raw(),
        skill.raw(),
        options.len() as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        giver,
        2,
        2,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseGiveTwoDifferentColorsOrRecover,
        payload,
    )
}

pub(crate) fn resolve_mbzhuguo(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let alive = |id| {
        tx.state()
            .players
            .get(id)
            .is_some_and(|participant| participant.alive)
    };
    if !originating_skill_is_enabled(tx, player, skill) || !alive(player) || !alive(target) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if tx.state().turn.current != player || tx.state().turn.phase != Phase::Play {
                return Ok(Vec::new());
            }
            let limit = tx
                .state()
                .players
                .get(target)
                .map_or(0, |participant| participant.max_hp.min(5));
            let hand_count = tx.state().hand_count(target);
            if hand_count < limit {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveMbzhuguo {
                        player,
                        target,
                        stage: 1,
                        skill,
                    },
                );
                return draw_cards(tx, target, limit - hand_count);
            }
            if hand_count > limit {
                let hand = tx.state().zones.find(ZoneKind::Hand, target);
                let mut options: Vec<_> =
                    crate::query::discardable_hand_equip_cards(tx.state(), target)
                        .into_iter()
                        .filter(|card| {
                            tx.state()
                                .cards
                                .get(*card)
                                .is_some_and(|card| Some(card.zone) == hand)
                        })
                        .collect();
                options.sort_by_key(|card| card.raw());
                let need = usize::try_from(hand_count - limit).unwrap_or(usize::MAX);
                if options.len() >= need {
                    let mut payload = vec![
                        0,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                        need as u32,
                        options.len() as u32,
                    ];
                    payload.extend(options.iter().map(|card| card.raw()));
                    return open_choice(
                        tx,
                        target,
                        u8::try_from(need).unwrap_or(u8::MAX),
                        u8::try_from(need).unwrap_or(u8::MAX),
                        false,
                        options.into_iter().map(CardId::raw).collect(),
                        ChoiceKind::SelectCards,
                        ContinuationKind::ResolveMbzhuguo,
                        payload,
                    );
                }
            }
            recover_hp(tx, PlayerId::NONE, target, 1)
        }
        1 => {
            let target_hand = tx.state().hand_count(target);
            let maximum = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|id| tx.state().hand_count(id))
                .max()
                .unwrap_or(0);
            if target_hand < maximum {
                return Ok(Vec::new());
            }
            let candidates: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != player && *candidate != target)
                .collect();
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                1,
                player.raw(),
                target.raw(),
                skill.raw(),
                candidates.len() as u32,
            ];
            payload.extend(candidates.iter().map(|candidate| candidate.raw()));
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                candidates.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveMbzhuguo,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "invalid mbzhuguo effect stage".into(),
        )),
    }
}

pub(crate) fn resolve_mbanxian(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
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
    match stage {
        0 => {
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("shared discard pile missing".into()))?;
            let mut options: Vec<_> = cards
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card_state| {
                        card_state.zone == discard
                            && crate::query::effective_card_kind(tx.state(), player, card_state)
                                .is_sha()
                    })
                })
                .collect();
            options.sort_by_key(|card| card.raw());
            options.dedup();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![0, player.raw(), skill.raw(), options.len() as u32];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveMbanxian,
                payload,
            )
        }
        1 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if !tx.state().cards.get(card).is_some_and(|card_state| {
                Some(card_state.zone) == hand
                    && crate::query::effective_card_kind(tx.state(), player, card_state).is_sha()
            }) {
                return Ok(Vec::new());
            }
            let legal = crate::command::play::prompted_sha_targets_unlimited(tx, player, card);
            if legal.is_empty() {
                return Ok(Vec::new());
            }
            let (min_count, max_count) = if crate::query::sha_targets_all_for(tx.state(), player) {
                let count = u8::try_from(legal.len()).unwrap_or(u8::MAX);
                (count, count)
            } else {
                (
                    1,
                    u8::try_from(
                        legal
                            .len()
                            .min(crate::query::sha_target_limit_for(tx.state(), player) as usize),
                    )
                    .unwrap_or(u8::MAX),
                )
            };
            let mut payload = vec![1, player.raw(), card.raw(), skill.raw(), legal.len() as u32];
            payload.extend(legal.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                min_count,
                max_count,
                false,
                legal.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveMbanxian,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "invalid mbanxian effect stage".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_mbjinzu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cards: Vec<CardId>,
    card: CardId,
    helper_skill_def: engine_api::SkillDefId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 | 1 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.alive && !participant.dying)
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || helper_skill_def.is_none()
                || tx
                    .state()
                    .skills
                    .get(skill)
                    .is_some_and(|instance| instance.def == helper_skill_def)
            {
                return Err(EffectError::Illegal(
                    "invalid mbjinzu active context".into(),
                ));
            }
        }
        2 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || card.is_none()
                || !tx.state().cards.get(card).is_some_and(|card_state| {
                    crate::query::effective_card_kind(tx.state(), player, card_state).is_sha()
                })
            {
                return Err(EffectError::Illegal(
                    "invalid mbjinzu next-Sha context".into(),
                ));
            }
        }
        3 | 4 => {
            if !tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.owner == player)
            {
                return Ok(Vec::new());
            }
        }
        _ => {
            return Err(EffectError::ContentFault(
                "invalid mbjinzu effect stage".into(),
            ));
        }
    }

    match stage {
        0 => {
            let [owner_card] = cards.as_slice() else {
                return Err(EffectError::Illegal(
                    "mbjinzu requires one selected owner hand card".into(),
                ));
            };
            let owner_hand = tx.state().zones.find(ZoneKind::Hand, player);
            if !tx.state().cards.get(*owner_card).is_some_and(|card_state| {
                Some(card_state.zone) == owner_hand && card_state.owner == player
            }) {
                return Err(EffectError::Illegal("mbjinzu owner card left hand".into()));
            }
            let target_cards = tx.state().hand_cards(target);
            if target_cards.len() <= 2 {
                let mut revealed = vec![*owner_card];
                revealed.extend(target_cards);
                return resolve_mbjinzu(
                    tx,
                    player,
                    target,
                    revealed,
                    CardId::NONE,
                    helper_skill_def,
                    1,
                    skill,
                );
            }
            let mut payload = vec![
                0,
                player.raw(),
                target.raw(),
                owner_card.raw(),
                helper_skill_def.raw(),
                skill.raw(),
                target_cards.len() as u32,
            ];
            payload.extend(target_cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                2,
                2,
                false,
                target_cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveMbjinzu,
                payload,
            )
        }
        1 => {
            let Some(owner_card) = cards.first().copied() else {
                return Err(EffectError::Illegal(
                    "mbjinzu reveal snapshot is empty".into(),
                ));
            };
            let target_cards = &cards[1..];
            let owner_hand = tx.state().zones.find(ZoneKind::Hand, player);
            let target_hand = tx.state().zones.find(ZoneKind::Hand, target);
            if !tx.state().cards.get(owner_card).is_some_and(|card_state| {
                Some(card_state.zone) == owner_hand && card_state.owner == player
            }) || target_cards.len() > 2
                || (target_cards.is_empty() && tx.state().hand_count(target) != 0)
                || target_cards.iter().any(|card| {
                    !tx.state().cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == target_hand && card_state.owner == target
                    })
                })
            {
                return Err(EffectError::Illegal(
                    "mbjinzu reveal context changed".into(),
                ));
            }
            let owner_number = tx
                .state()
                .cards
                .get(owner_card)
                .map(|card| crate::query::effective_card_number(tx.state(), player, card))
                .ok_or_else(|| EffectError::Illegal("mbjinzu owner card missing".into()))?;
            let target_numbers: Vec<_> = target_cards
                .iter()
                .filter_map(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .map(|card| crate::query::effective_card_number(tx.state(), player, card))
                })
                .collect();
            let (extreme, grant_helper) = if target_numbers.is_empty() {
                (true, true)
            } else {
                let min = target_numbers
                    .iter()
                    .copied()
                    .chain(std::iter::once(owner_number))
                    .min()
                    .unwrap_or(owner_number);
                let max = target_numbers
                    .iter()
                    .copied()
                    .chain(std::iter::once(owner_number))
                    .max()
                    .unwrap_or(owner_number);
                (
                    owner_number == min || owner_number == max,
                    target_numbers.iter().any(|number| *number <= owner_number)
                        && target_numbers.iter().any(|number| *number >= owner_number),
                )
            };
            let mut events = vec![DomainEvent::CardsRevealed {
                cards: cards.clone(),
                to: Vec::new(),
            }];
            if !extreme {
                events.extend(add_mbjinzu_helper(tx, player, target, helper_skill_def)?);
                return Ok(events);
            }

            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveMbjinzu {
                    player,
                    target,
                    cards: Vec::new(),
                    card: CardId::NONE,
                    helper_skill_def,
                    stage: if grant_helper { 4 } else { 3 },
                    skill,
                });
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("shared discard pile missing".into()))?;
            events.extend(move_cards_with_reason(
                tx,
                vec![owner_card],
                discard,
                owner_hand,
                CardMoveReason::Discard,
            )?);
            if !target_cards.is_empty() {
                events.extend(move_cards_with_reason(
                    tx,
                    target_cards.to_vec(),
                    discard,
                    target_hand,
                    CardMoveReason::Discard,
                )?);
            }
            Ok(events)
        }
        2 => {
            let targets = crate::command::play::saved_sha_targets(tx, player, card)?;
            let bonuses = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .map(|instance| instance.take_mbjinzu_targets())
                .unwrap_or_default();
            let mut events = Vec::new();
            for (target, amount) in bonuses {
                if !targets.contains(&target) {
                    continue;
                }
                events.extend(make_current_sha_target_unrespondable(
                    tx, player, card, target,
                )?);
                events.extend(increase_current_sha_target_damage(
                    tx, player, card, target, amount,
                )?);
            }
            events.extend(remove_skill(tx, skill)?);
            Ok(events)
        }
        3 | 4 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.use_count = 0;
                instance.play_phase_use_count = 0;
            }
            if stage == 4 {
                add_mbjinzu_helper(tx, player, target, helper_skill_def)
            } else {
                Ok(Vec::new())
            }
        }
        _ => unreachable!(),
    }
}

pub(crate) fn gain_discard_card_by_kind(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    let kind = CardKind::from_tag(kind)
        .ok_or_else(|| EffectError::Illegal(format!("unknown card kind {kind}")))?;
    if tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal("bad player".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("shared discard pile missing".into()))?;
    let card = tx.state().zones.get(discard).and_then(|zone| {
        zone.cards.iter().rev().copied().find(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.kind == kind)
        })
    });
    card.map_or_else(
        || Ok(Vec::new()),
        |card| move_cards(tx, vec![card], hand, Some(discard)),
    )
}

pub(crate) fn choose_gain_discard_card_by_kind(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let kind = CardKind::from_tag(kind)
        .ok_or_else(|| EffectError::Illegal(format!("unknown card kind {kind}")))?;
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("shared discard pile missing".into()))?;
    let options: Vec<_> = tx
        .state()
        .zones
        .get(discard)
        .into_iter()
        .flat_map(|zone| zone.cards.iter().copied())
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.kind == kind)
        })
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseGainDiscardCardByKind,
        vec![player.raw(), u32::from(kind.tag()), skill.raw()],
    )
}

pub(crate) fn choose_gain_own_lost_discard_card_this_turn(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(owner) = tx.state().players.get(player) else {
        return Err(EffectError::Illegal("bad player".into()));
    };
    let loss_count = tx.state().turn.card_loss_count(player);
    if !originating_skill_is_enabled(tx, player, skill)
        || !owner.alive
        || tx.state().turn.current != player
        || loss_count == 0
        || loss_count > owner.hp.max(0) as u32
    {
        return Ok(Vec::new());
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("shared discard pile missing".into()))?;
    let options: Vec<_> = tx
        .state()
        .turn
        .lost_cards_for(player)
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.zone == discard)
        })
        .map(CardId::raw)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseGainOwnLostDiscardCardThisTurn,
        vec![player.raw(), skill.raw()],
    )
}
