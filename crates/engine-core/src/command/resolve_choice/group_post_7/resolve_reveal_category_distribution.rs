//! Command operations for `resolve_reveal_category_distribution` and related helpers.

use super::*;

pub(crate) fn resolve_reveal_category_distribution(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let (player, skill, cap, stage, remaining, mut counts, pending) =
        parse_category_distribution_payload(payload)?;
    if observer != player
        || cancelled
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "category distribution context changed".into(),
        ));
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("processing zone missing".into()))?;
    match stage {
        0 => {
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "category distribution requires one category".into(),
                ));
            }
            let category = selections[0];
            let mut matching = Vec::new();
            let mut rejected = Vec::new();
            for card in remaining {
                if tx.state().cards.get(card).map(|card| card.zone) != Some(processing) {
                    return Err(EffectError::Illegal(
                        "category distribution revealed card moved".into(),
                    ));
                }
                if u32::from(crate::effect::effective_card_category(
                    tx.state(),
                    player,
                    card,
                )) == category
                {
                    matching.push(card);
                } else {
                    rejected.push(card);
                }
            }
            if matching.is_empty() {
                return Err(EffectError::Illegal(
                    "selected category has no revealed cards".into(),
                ));
            }
            if !rejected.is_empty() {
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("discard pile missing".into()))?;
                run_effects(
                    tx,
                    vec![EffectRequest::MoveCards {
                        cards: rejected,
                        to: discard,
                        from: Some(processing),
                    }],
                )?;
            }
            open_category_distribution_card_choice(tx, player, skill, cap, &matching, &[])
        }
        1 => {
            let selected: Vec<CardId> = selections.iter().map(|id| CardId::new(*id)).collect();
            if selected.is_empty()
                || selected.len() > usize::from(cap)
                || selected.iter().any(|card| {
                    !remaining.contains(card)
                        || tx.state().cards.get(*card).map(|card| card.zone) != Some(processing)
                })
                || selected
                    .iter()
                    .enumerate()
                    .any(|(index, card)| selected[..index].contains(card))
            {
                return Err(EffectError::Illegal(
                    "invalid category distribution cards".into(),
                ));
            }
            let options: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    let used = counts
                        .iter()
                        .find(|(candidate, _)| candidate == target)
                        .map_or(0, |(_, count)| *count);
                    used + selected.len() as u32 <= u32::from(cap)
                })
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Err(EffectError::Illegal(
                    "no eligible category distribution recipient".into(),
                ));
            }
            let rest: Vec<CardId> = remaining
                .into_iter()
                .filter(|card| !selected.contains(card))
                .collect();
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                engine_api::ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                category_distribution_payload(player, skill, cap, 2, &rest, &counts, &selected),
            )
            .map(|_| ())
        }
        2 => {
            if selections.len() != 1 || pending.is_empty() {
                return Err(EffectError::Illegal(
                    "category distribution requires one recipient".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            let used = counts
                .iter()
                .find(|(candidate, _)| *candidate == target)
                .map_or(0, |(_, count)| *count);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
                || used + pending.len() as u32 > u32::from(cap)
                || pending.iter().any(|card| {
                    tx.state().cards.get(*card).map(|card| card.zone) != Some(processing)
                })
            {
                return Err(EffectError::Illegal(
                    "invalid category distribution recipient".into(),
                ));
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("recipient hand missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: pending.clone(),
                    to: hand,
                    from: Some(processing),
                }],
            )?;
            if let Some((_, count)) = counts
                .iter_mut()
                .find(|(candidate, _)| *candidate == target)
            {
                *count += pending.len() as u32;
            } else {
                counts.push((target, pending.len() as u32));
            }
            if !remaining.is_empty() {
                return open_category_distribution_card_choice(
                    tx, player, skill, cap, &remaining, &counts,
                );
            }
            let owner_count = counts
                .iter()
                .find(|(target, _)| *target == player)
                .map_or(0, |(_, count)| *count);
            let effects: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter_map(|target| {
                    counts
                        .iter()
                        .find(|(candidate, count)| *candidate == target && *count >= owner_count)
                        .map(|_| EffectRequest::DealDamage {
                            source: PlayerId::NONE,
                            target,
                            amount: 1,
                            is_lose_hp: true,
                            nature: rule_api::damage_nature::NORMAL,
                        })
                })
                .collect();
            run_effects(tx, effects).map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown category distribution stage".into(),
        )),
    }
}

pub(crate) fn resolve_resolve_sibian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 {
        return Err(EffectError::EngineFault("bad Sibian continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let count = payload[3] as usize;
    if payload.len() != 4 + count || observer != player {
        return Err(EffectError::Illegal(
            "Sibian continuation payload mismatch".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().turn.current != player
    {
        return Err(EffectError::Illegal("Sibian choice context changed".into()));
    }
    let middle: Vec<CardId> = payload[4..].iter().map(|id| CardId::new(*id)).collect();
    let target = if cancelled {
        // The ungiven middle cards go to the discard pile.
        None
    } else if selections.len() == 1 {
        let target = PlayerId::new(selections[0]);
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal("Sibian give target invalid".into()));
        }
        Some(target)
    } else {
        return Err(EffectError::Illegal("Sibian give selection invalid".into()));
    };
    let to = target.map_or_else(
        || {
            tx.state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))
        },
        |target| {
            tx.state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("target hand missing".into()))
        },
    )?;
    let mut moves = Vec::new();
    for card in middle {
        let from = tx
            .state()
            .cards
            .get(card)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("Sibian card missing".into()))?;
        moves.push(EffectRequest::MoveCards {
            cards: vec![card],
            to,
            from: Some(from),
        });
    }
    run_effects(tx, moves).map(|_| ())
}

pub(crate) fn resolve_resolve_spjiedao(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault("bad Spjiedao continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let stage = payload[3];
    if observer != player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Spjiedao choice context changed".into(),
        ));
    }
    match stage {
        0 => {
            if cancelled {
                return Ok(());
            }
            if selections.len() != 1
                || tx.state().resolution.current_damage_id == 0
                || tx.state().turn.player_dealt_damage(player)
            {
                return Err(EffectError::Illegal(
                    "Spjiedao boost context changed".into(),
                ));
            }
            let amount = *selections.first().unwrap();
            let Some(participant) = tx.state().players.get(player) else {
                return Err(EffectError::Illegal("Spjiedao player missing".into()));
            };
            let max = (participant.max_hp as i32 - participant.hp).max(1) as u32;
            if amount == 0 || amount > max {
                return Err(EffectError::Illegal(
                    "Spjiedao boost amount out of range".into(),
                ));
            }
            let damage_id = tx.state().resolution.current_damage_id;
            crate::effect::increase_bound_damage(tx, damage_id, amount)?;
            let options: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), player)
                .into_iter()
                .map(CardId::raw)
                .collect();
            if options.len() < amount as usize {
                // Cannot pay the full cost: legacy would fail at damage end; approximate by
                // skipping the discard.
                return Ok(());
            }
            crate::effect::open_choice(
                tx,
                player,
                amount.min(u8::MAX as u32) as u8,
                amount.min(u8::MAX as u32) as u8,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_SPJIEDAO_CUSTOM_CHOICE,
                    player.raw(),
                    engine_api::SkillId::new(payload[2]).raw(),
                    1,
                ],
            )
            .map(|_| ())
        }
        1 => {
            if selections.is_empty() {
                return Err(EffectError::Illegal(
                    "Spjiedao discard selection empty".into(),
                ));
            }
            let allowed = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let mut moves = Vec::new();
            for raw in selections {
                let card = CardId::new(*raw);
                if !allowed.contains(&card) {
                    return Err(EffectError::Illegal("Spjiedao discard card invalid".into()));
                }
                let from = tx
                    .state()
                    .cards
                    .get(card)
                    .map(|card| card.zone)
                    .ok_or_else(|| EffectError::Illegal("Spjiedao card missing".into()))?;
                moves.push(EffectRequest::MoveCards {
                    cards: vec![card],
                    to: tx
                        .state()
                        .zones
                        .find(ZoneKind::DiscardPile, PlayerId::NONE)
                        .unwrap(),
                    from: Some(from),
                });
            }
            run_effects(tx, moves).map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "Spjiedao continuation stage mismatch".into(),
        )),
    }
}

pub(crate) fn resolve_resolve_stdgaobian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 6 {
        return Err(EffectError::EngineFault(
            "bad Stdgaobian continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let chooser = PlayerId::new(payload[3]);
    let stage = payload[4];
    if cancelled
        || selections.len() != 1
        || observer != chooser
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(chooser)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Stdgaobian choice context changed".into(),
        ));
    }
    match stage {
        0 => {
            let branch = *selections.first().unwrap();
            if branch == 0 {
                // The damaged player uses a virtual Sha against a picked target.
                run_effects(
                    tx,
                    vec![EffectRequest::ResolveStdgaobian {
                        player,
                        chooser,
                        skill,
                    }],
                )
                .map(|_| ())
            } else if branch == 1 {
                // Reset the owner's Anran counter mark.
                let count = tx
                    .state()
                    .players
                    .get(player)
                    .map(|p| {
                        p.marks
                            .iter()
                            .find(|(mark, _)| *mark == crate::effect::STDANRAN_COUNTER_MARK)
                            .map(|(_, count)| *count)
                            .unwrap_or(0)
                    })
                    .unwrap_or(0);
                if count > 0 {
                    run_effects(
                        tx,
                        vec![EffectRequest::RemoveMark {
                            player,
                            mark: engine_api::MarkDefId::new(crate::effect::STDANRAN_COUNTER_MARK),
                            amount: count.max(0) as u32,
                        }],
                    )
                    .map(|_| ())
                } else {
                    Ok(())
                }
            } else {
                Err(EffectError::Illegal(
                    "Stdgaobian branch out of range".into(),
                ))
            }
        }
        1 => {
            let target = PlayerId::new(*selections.first().unwrap());
            if target == chooser
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("Stdgaobian Sha target invalid".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::UseVirtualCardOn {
                    player: chooser,
                    kind: crate::state::CardKind::Sha.tag(),
                    target,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "Stdgaobian continuation stage mismatch".into(),
        )),
    }
}

pub(crate) fn resolve_resolve_dczhiheng(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad Dczhiheng continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player || cancelled || selections.is_empty() {
        return Err(EffectError::Illegal(
            "Dczhiheng discard context changed".into(),
        ));
    }
    if tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Dczhiheng player context changed".into(),
        ));
    }
    let hand_before = tx.state().hand_cards(player);
    let allowed: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let mut seen = std::collections::BTreeSet::new();
    let mut moves = Vec::new();
    for raw in selections {
        let card = CardId::new(*raw);
        if !seen.insert(card.raw()) || !allowed.contains(&card) {
            return Err(EffectError::Illegal(
                "Dczhiheng discard card invalid or duplicate".into(),
            ));
        }
        let from = tx
            .state()
            .cards
            .get(card)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("Dczhiheng discard card missing".into()))?;
        moves.push(EffectRequest::MoveCards {
            cards: vec![card],
            to: tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap(),
            from: Some(from),
        });
    }
    let count = selections.len() as u32;
    let all_hand_discarded =
        !hand_before.is_empty() && hand_before.iter().all(|card| seen.contains(&card.raw()));
    let mut effects = moves;
    effects.push(EffectRequest::DrawCardsFromSkill {
        player,
        count: count + u32::from(all_hand_discarded),
        skill: engine_api::SkillId::new(payload[2]),
    });
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_resolve_dcnuchen(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 6 {
        return Err(EffectError::EngineFault("bad Dcnuchen continuation".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    let target = PlayerId::new(payload[4]);
    // NB: the skill instance id is SkillId::NONE on the VM-first path (host effects do
    // not capture it); guard on turn/phase/ownership instead.
    if observer != player
        || (!skill.is_none()
            && !tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player && tx.state().skill_effectively_enabled(instance)
            }))
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Dcnuchen choice context changed".into(),
        ));
    }
    match (stage, cancelled, selections) {
        (0, false, [target_raw]) => {
            let target = PlayerId::new(*target_raw);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || tx.state().hand_count(target) == 0
            {
                return Err(EffectError::Illegal(
                    "Dcnuchen target invalid or has no hand".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDcnuchen {
                    player,
                    target,
                    skill,
                }],
            )
            .map(|_| ())
        }
        (1, false, [card_raw]) => {
            let card = CardId::new(*card_raw);
            if !tx.state().hand_cards(target).contains(&card) {
                return Err(EffectError::Illegal(
                    "revealed card is no longer in the target hand".into(),
                ));
            }
            let suit = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.suit as u32)
                .unwrap_or(u32::MAX);
            let he: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), player)
                .into_iter()
                .filter(|candidate| {
                    tx.state()
                        .cards
                        .get(*candidate)
                        .is_some_and(|card| card.suit as u32 == suit)
                })
                .map(CardId::raw)
                .collect();
            let mut effects = vec![EffectRequest::RevealCards {
                cards: vec![card],
                to: Vec::new(),
            }];
            if !he.is_empty() {
                crate::effect::open_choice(
                    tx,
                    player,
                    1,
                    he.len().min(u8::MAX as usize) as u8,
                    true,
                    he,
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    vec![
                        crate::effect::RESOLVE_DCNUCHEN_CUSTOM_CHOICE,
                        2,
                        player.raw(),
                        skill.raw(),
                        target.raw(),
                        suit,
                    ],
                )?;
                return crate::effect::run_effects(tx, effects).map(|_| ());
            }
            // No same-suit discardable cards: the cancel (gain) path directly.
            effects.push(EffectRequest::ResolveDcnuchen {
                player,
                target,
                skill,
            });
            run_effects(tx, effects).map(|_| ())
        }
        (2, cancelled, _) => {
            let suit = payload[5];
            if cancelled {
                let gain: Vec<_> = tx
                    .state()
                    .hand_cards(target)
                    .into_iter()
                    .filter(|card| {
                        tx.state()
                            .cards
                            .get(*card)
                            .is_some_and(|card| card.suit as u32 == suit)
                    })
                    .collect();
                if gain.is_empty() {
                    return Ok(());
                }
                let zones: Vec<_> = gain
                    .iter()
                    .filter_map(|card| tx.state().cards.get(*card))
                    .map(|card| card.zone)
                    .collect();
                run_effects(
                    tx,
                    zones
                        .into_iter()
                        .zip(gain)
                        .map(|(from, card)| EffectRequest::MoveCards {
                            cards: vec![card],
                            to: tx.state().zones.find(ZoneKind::Hand, player).unwrap(),
                            from: Some(from),
                        })
                        .collect(),
                )
                .map(|_| ())
            } else if selections.is_empty() {
                Err(EffectError::Illegal(
                    "Dcnuchen discard branch needs at least one card".into(),
                ))
            } else {
                let cards: Vec<_> = selections.iter().map(|id| CardId::new(*id)).collect();
                for card in &cards {
                    let card_state =
                        tx.state().cards.get(*card).ok_or_else(|| {
                            EffectError::Illegal("bad Dcnuchen discard card".into())
                        })?;
                    if card_state.owner != player
                        || card_state.suit as u32 != suit
                        || !crate::query::discardable_hand_equip_cards(tx.state(), player)
                            .contains(card)
                    {
                        return Err(EffectError::Illegal("Dcnuchen discard card invalid".into()));
                    }
                }
                let mut effects = Vec::new();
                for card in &cards {
                    let from = tx.state().cards.get(*card).map(|card| card.zone);
                    effects.push(EffectRequest::MoveCards {
                        cards: vec![*card],
                        to: tx
                            .state()
                            .zones
                            .find(ZoneKind::DiscardPile, PlayerId::NONE)
                            .unwrap(),
                        from,
                    });
                }
                effects.push(EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: cards.len() as u32,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                });
                run_effects(tx, effects).map(|_| ())
            }
        }
        _ => Err(EffectError::Illegal(
            "Dcnuchen continuation stage mismatch".into(),
        )),
    }
}
