//! Effect primitives for `choose_targets_damage` and related helpers.

use super::*;

pub(crate) fn replace_draw_gain_top_two_add_skills_if_different_colors(
    tx: &mut Transaction,
    player: PlayerId,
    first_skill_def: engine_api::SkillDefId,
    second_skill_def: engine_api::SkillDefId,
    stage: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill)
            || tx.state().turn.current != player
            || tx.state().turn.phase != Phase::Draw
            || tx.state().turn.draw_phase_is_fixed()
        {
            return Err(EffectError::Illegal(
                "invalid top-two draw replacement context".into(),
            ));
        }
        tx.state_mut().turn.set_draw_phase_fixed_count(0);
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
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
            .ok_or_else(|| EffectError::EngineFault("draw pile disappeared".into()))?;
        let take = 2.min(zone.cards.len());
        if take == 0 {
            return Ok(Vec::new());
        }
        let cards = zone.cards[zone.cards.len() - take..].to_vec();
        let processing = tx
            .state()
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
                player,
                first_skill_def,
                second_skill_def,
                stage: 1,
                cards: cards.clone(),
                skill,
            },
        );
        let mut events = move_cards(tx, cards.clone(), processing, Some(draw))?;
        events.push(DomainEvent::CardsRevealed {
            cards,
            to: Vec::new(),
        });
        return Ok(events);
    }
    if stage == 1 {
        let processing = tx
            .state()
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
        if cards.is_empty()
            || cards.iter().any(|card| {
                !tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.zone == processing)
            })
        {
            return Err(EffectError::Illegal(
                "top-two revealed cards left processing".into(),
            ));
        }
        if cards.len() == 2
            && crate::query::effective_card_color_mask(tx.state(), player, cards[0])
                != crate::query::effective_card_color_mask(tx.state(), player, cards[1])
        {
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
                    player,
                    first_skill_def,
                    second_skill_def,
                    stage: 2,
                    cards: cards.clone(),
                    skill,
                },
            );
        }
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::EngineFault("player hand missing".into()))?;
        return move_cards(tx, cards, hand, Some(processing));
    }
    if stage != 2 {
        return Err(EffectError::ContentFault(
            "bad top-two draw replacement stage".into(),
        ));
    }
    let mut events = add_skill_until_turn_end(tx, player, first_skill_def)?;
    events.extend(add_skill_until_turn_end(tx, player, second_skill_def)?);
    Ok(events)
}

pub(crate) fn resolve_new_reluoyi(
    tx: &mut Transaction,
    player: PlayerId,
    buff_skill_def: engine_api::SkillDefId,
    stage: u8,
    cards: Vec<CardId>,
    accepted: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if buff_skill_def.is_none() {
        return Err(EffectError::ContentFault(
            "New Reluoyi requires a damage helper skill".into(),
        ));
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill)
            || tx.state().turn.current != player
            || tx.state().turn.phase != Phase::Draw
            || tx.state().turn.draw_phase_is_fixed()
        {
            return Err(EffectError::Illegal(
                "invalid New Reluoyi draw replacement context".into(),
            ));
        }
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
        if tx
            .state()
            .zones
            .get(draw)
            .map_or(0, |zone| zone.cards.len())
            < 3
        {
            reshuffle_discard_into_draw(tx)?;
        }
        let zone = tx
            .state()
            .zones
            .get(draw)
            .ok_or_else(|| EffectError::EngineFault("draw pile disappeared".into()))?;
        let take = 3.min(zone.cards.len());
        if take == 0 {
            return Ok(Vec::new());
        }
        let cards = zone.cards[zone.cards.len() - take..].to_vec();
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ResolveNewReluoyi {
                player,
                buff_skill_def,
                stage: 1,
                cards: cards.clone(),
                accepted: false,
                skill,
            });
        let mut events = move_cards(tx, cards.clone(), processing, Some(draw))?;
        events.push(DomainEvent::CardsRevealed {
            cards,
            to: Vec::new(),
        });
        return Ok(events);
    }
    if stage == 3 {
        return add_source_turn_bound_skill(tx, player, buff_skill_def, player);
    }
    if cards.is_empty()
        || cards.iter().any(|card| {
            !tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.zone == processing)
        })
    {
        return Err(EffectError::Illegal(
            "New Reluoyi revealed cards left processing".into(),
        ));
    }
    if stage == 1 {
        let mut payload = vec![
            player.raw(),
            buff_skill_def.raw(),
            skill.raw(),
            cards.len() as u32,
        ];
        payload.extend(cards.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Confirm,
            ContinuationKind::ResolveNewReluoyi,
            payload,
        );
    }
    if stage != 2 {
        return Err(EffectError::ContentFault(
            "bad New Reluoyi resolution stage".into(),
        ));
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    if !accepted {
        return move_cards(tx, cards, discard, Some(processing));
    }

    tx.state_mut().turn.set_draw_phase_fixed_count(0);
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveNewReluoyi {
            player,
            buff_skill_def,
            stage: 3,
            cards: Vec::new(),
            accepted: true,
            skill,
        });
    let (gain, rest): (Vec<_>, Vec<_>) = cards.into_iter().partition(|card| {
        tx.state().cards.get(*card).is_some_and(|card| {
            card.kind.is_basic() || matches!(card.kind, CardKind::TrickJuedou | CardKind::Weapon)
        })
    });
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("player hand missing".into()))?;
    let mut events = Vec::new();
    if !gain.is_empty() {
        events.extend(move_cards(tx, gain, hand, Some(processing))?);
    }
    if !rest.is_empty() {
        events.extend(move_cards(tx, rest, discard, Some(processing))?);
    }
    Ok(events)
}

pub(crate) fn resolve_remoshi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    helper_skill_def: engine_api::SkillDefId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let pile_skill_def = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::Illegal("invalid Remoshi skill context".into()))?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            if helper_skill_def.is_none() {
                return Err(EffectError::ContentFault(
                    "Remoshi requires its presentation helper".into(),
                ));
            }
            let source_cards: Vec<_> = tx
                .state()
                .resolution
                .sha_source_cards(player, card)
                .into_iter()
                .filter_map(|card| {
                    tx.state()
                        .cards
                        .get(card)
                        .filter(|card| card.def != engine_api::CardDefId::NONE)
                        .and_then(|card| {
                            tx.state()
                                .zones
                                .get(card.zone)
                                .filter(|zone| {
                                    matches!(
                                        zone.kind,
                                        ZoneKind::Processing | ZoneKind::DiscardPile
                                    )
                                })
                                .map(|_| (card.id, card.zone))
                        })
                })
                .collect();
            if source_cards.is_empty() {
                return Err(EffectError::Illegal(
                    "Remoshi Sha sources are no longer gainable".into(),
                ));
            }
            let has_armor_or_mount = tx
                .state()
                .zones
                .find(ZoneKind::Equip, target)
                .and_then(|zone| tx.state().zones.get(zone))
                .is_some_and(|zone| {
                    zone.cards.iter().any(|card| {
                        tx.state().cards.get(*card).is_some_and(|card| {
                            matches!(
                                card.kind,
                                CardKind::Armor | CardKind::HorseOff | CardKind::HorseDef
                            )
                        })
                    })
                });
            if !has_armor_or_mount {
                return Ok(Vec::new());
            }
            let mut events = add_skill(tx, target, helper_skill_def)?;
            let pile = ensure_skill_pile_zone(tx, target, pile_skill_def)?;
            for (card, from) in source_cards {
                events.extend(move_cards(tx, vec![card], pile, Some(from))?);
            }
            Ok(events)
        }
        1 => {
            let cards = tx.state().skill_pile_cards(target, pile_skill_def);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let from = ensure_skill_pile_zone(tx, target, pile_skill_def)?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("Remoshi owner hand missing".into()))?;
            give_cards(tx, cards, to, from)
        }
        _ => Err(EffectError::ContentFault(
            "unknown Remoshi resolution action".into(),
        )),
    }
}

pub(crate) fn resolve_xinyaoming(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "invalid Xinyaoming skill context".into(),
        ));
    }
    let used_mask = tx
        .state()
        .skills
        .get(skill)
        .map_or(u8::MAX, |instance| instance.xinyaoming_used_mask());
    if target.is_none() {
        if action != 0 {
            return Err(EffectError::ContentFault(
                "Xinyaoming target choice requires action zero".into(),
            ));
        }
        let options: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| {
                !crate::query::xinyaoming_available_actions(tx.state(), player, *target, used_mask)
                    .is_empty()
            })
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        let mut payload = vec![0, player.raw(), skill.raw(), options.len() as u32];
        payload.extend(options.iter().map(|target| target.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.into_iter().map(PlayerId::raw).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::ResolveXinyaoming,
            payload,
        );
    }
    let actions = crate::query::xinyaoming_available_actions(tx.state(), player, target, used_mask);
    if action == 0 {
        return match actions.as_slice() {
            [] => Err(EffectError::Illegal(
                "Xinyaoming target has no available action".into(),
            )),
            [only] => resolve_xinyaoming(tx, player, target, *only, skill),
            _ => {
                let mut payload = vec![
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    actions.len() as u32,
                ];
                payload.extend(actions.iter().map(|action| u32::from(*action)));
                open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    actions.into_iter().map(u32::from).collect(),
                    ChoiceKind::Custom,
                    ContinuationKind::ResolveXinyaoming,
                    payload,
                )
            }
        };
    }
    if !actions.contains(&action) {
        return Err(EffectError::Illegal(
            "Xinyaoming action is unavailable".into(),
        ));
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::Illegal("Xinyaoming skill disappeared".into()))?
        .mark_xinyaoming_action(action);
    match action {
        crate::query::XINYAOMING_DRAW => draw_cards(tx, target, 1),
        crate::query::XINYAOMING_DISCARD_HAND => {
            choose_discard_target_card(tx, player, target, rule_api::TargetCardScope::Hand)
        }
        crate::query::XINYAOMING_DISCARD_DRAW => {
            let options = crate::query::discardable_target_cards(
                tx.state(),
                target,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let max = options.len().min(2) as u8;
            let mut payload = vec![
                2,
                player.raw(),
                target.raw(),
                skill.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                1,
                max,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveXinyaoming,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown Xinyaoming action".into(),
        )),
    }
}

pub(crate) fn rehuaiyi_eligible_colors(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Vec<u8> {
    let hand = state.hand_cards(player);
    let discardable: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(state, player)
            .into_iter()
            .collect();
    [0b01, 0b10]
        .into_iter()
        .filter(|color| {
            let matching: Vec<_> = hand
                .iter()
                .copied()
                .filter(|card| {
                    crate::query::effective_card_color_mask(state, player, *card) == *color
                })
                .collect();
            !matching.is_empty() && matching.iter().all(|card| discardable.contains(card))
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_rehuaiyi(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    color: u8,
    discarded_count: u32,
    targets: Vec<PlayerId>,
    mut index: u32,
    gained_count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_valid = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if !skill_valid
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive && !participant.dying)
    {
        return Err(EffectError::Illegal(
            "invalid Rehuaiyi skill context".into(),
        ));
    }

    match stage {
        0 if color == 0 => {
            if discarded_count != 0 || !targets.is_empty() || index != 0 || gained_count != 0 {
                return Err(EffectError::ContentFault(
                    "bad initial Rehuaiyi progress".into(),
                ));
            }
            let cards = tx.state().hand_cards(player);
            if cards.is_empty() {
                return Err(EffectError::Illegal(
                    "Rehuaiyi requires a nonempty hand".into(),
                ));
            }
            let color_mask = cards.iter().fold(0, |mask, card| {
                mask | crate::query::effective_card_color_mask(tx.state(), player, *card)
            }) & 0b11;
            let mut events = vec![DomainEvent::CardsRevealed {
                cards,
                to: Vec::new(),
            }];
            if matches!(color_mask, 0b01 | 0b10) {
                events.extend(draw_cards(tx, player, 1)?);
                return Ok(events);
            }
            if color_mask != 0b11 {
                return Ok(events);
            }

            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("Rehuaiyi skill disappeared".into()))?;
            instance.play_phase_use_count = instance.play_phase_use_count.max(2);

            let colors = rehuaiyi_eligible_colors(tx.state(), player);
            match colors.as_slice() {
                [] => Ok(events),
                [only] => {
                    events.extend(resolve_rehuaiyi(
                        tx,
                        player,
                        0,
                        *only,
                        0,
                        Vec::new(),
                        0,
                        0,
                        skill,
                    )?);
                    Ok(events)
                }
                _ => {
                    let mut payload = vec![0, player.raw(), skill.raw(), colors.len() as u32];
                    payload.extend(colors.iter().map(|color| u32::from(*color)));
                    events.extend(open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        colors.into_iter().map(u32::from).collect(),
                        ChoiceKind::Custom,
                        ContinuationKind::ResolveRehuaiyi,
                        payload,
                    )?);
                    Ok(events)
                }
            }
        }
        0 if matches!(color, 0b01 | 0b10) => {
            let eligible = rehuaiyi_eligible_colors(tx.state(), player);
            if !eligible.contains(&color) {
                return Err(EffectError::Illegal(
                    "selected Rehuaiyi color is no longer discardable".into(),
                ));
            }
            let cards: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .filter(|card| {
                    crate::query::effective_card_color_mask(tx.state(), player, *card) == color
                })
                .collect();
            if cards.is_empty() {
                return Err(EffectError::Illegal(
                    "selected Rehuaiyi color has no hand cards".into(),
                ));
            }
            let count = cards.len().min(u32::MAX as usize) as u32;
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards { player, cards },
                    EffectRequest::ResolveRehuaiyi {
                        player,
                        stage: 1,
                        color: 0,
                        discarded_count: count,
                        targets: Vec::new(),
                        index: 0,
                        gained_count: 0,
                        skill,
                    },
                ],
            )
        }
        1 => {
            if color != 0
                || discarded_count == 0
                || !targets.is_empty()
                || index != 0
                || gained_count != 0
            {
                return Err(EffectError::ContentFault(
                    "bad Rehuaiyi target-choice progress".into(),
                ));
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| {
                    target != player
                        && !crate::query::gainable_target_cards(
                            tx.state(),
                            player,
                            target,
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .is_empty()
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let max = options
                .len()
                .min(discarded_count as usize)
                .min(u8::MAX as usize) as u8;
            let mut payload = vec![
                1,
                player.raw(),
                skill.raw(),
                discarded_count,
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                max,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveRehuaiyi,
                payload,
            )
        }
        2 => {
            if color != 0 || discarded_count != 0 || targets.is_empty() {
                return Err(EffectError::ContentFault(
                    "bad Rehuaiyi gain progress".into(),
                ));
            }
            while let Some(target) = targets.get(index as usize).copied() {
                let options = crate::query::gainable_target_cards(
                    tx.state(),
                    player,
                    target,
                    rule_api::TargetCardScope::HandEquip,
                );
                if options.is_empty() {
                    index = index.saturating_add(1);
                    continue;
                }
                let mut payload = vec![
                    2,
                    player.raw(),
                    skill.raw(),
                    index,
                    gained_count,
                    targets.len() as u32,
                ];
                payload.extend(targets.iter().map(|target| target.raw()));
                payload.push(options.len() as u32);
                payload.extend(options.iter().map(|card| card.raw()));
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::ResolveRehuaiyi,
                    payload,
                );
            }
            if gained_count >= 2 {
                return run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: player,
                        amount: 1,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                );
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown Rehuaiyi resolution stage".into(),
        )),
    }
}
