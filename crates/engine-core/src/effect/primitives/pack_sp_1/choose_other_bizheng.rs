//! Effect primitives for `choose_other_bizheng` and related helpers.

use super::*;

pub(crate) fn choose_other_bizheng(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| target != player)
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ChooseOtherBizheng,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 => {
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ChooseOtherBizheng {
                    player,
                    target,
                    stage: 2,
                    skill,
                },
            );
            draw_cards(tx, target, 2)
        }
        2 | 3 => {
            let discarder = if stage == 2 { player } else { target };
            let needs_discard = tx
                .state()
                .players
                .get(discarder)
                .is_some_and(|participant| {
                    participant.alive && tx.state().hand_count(discarder) > participant.max_hp
                });
            if !needs_discard {
                return if stage == 2 {
                    choose_other_bizheng(tx, player, target, 3, skill)
                } else {
                    Ok(Vec::new())
                };
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), discarder);
            let required = options.len().min(2);
            if stage == 2 {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ChooseOtherBizheng {
                        player,
                        target,
                        stage: 3,
                        skill,
                    },
                );
            }
            if required == 0 {
                return Ok(Vec::new());
            }
            if required == options.len() {
                return discard_cards(tx, discarder, options);
            }
            let mut payload = vec![
                1,
                player.raw(),
                target.raw(),
                discarder.raw(),
                skill.raw(),
                options.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                discarder,
                required as u8,
                required as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ChooseOtherBizheng,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault("invalid Bizheng stage".into())),
    }
}

pub(crate) fn choose_other_xiashu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if tx.state().hand_count(player) == 0 {
                return Ok(Vec::new());
            }
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| target != player)
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ChooseOtherXiashu,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 => {
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let cards = tx.state().hand_cards(player);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("Xiashu owner hand missing".into()))?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("Xiashu target hand missing".into()))?;
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ChooseOtherXiashu {
                    player,
                    target,
                    stage: 2,
                    cards: Vec::new(),
                    skill,
                },
            );
            give_cards(tx, cards, to, from)
        }
        2 => {
            let options = tx.state().hand_cards(target);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            // ponytail: choice counts are u8; widen ChoiceRequest if real hands can exceed 255.
            let mut payload = vec![
                1,
                player.raw(),
                target.raw(),
                skill.raw(),
                options.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                1,
                options.len().min(u8::MAX as usize) as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ChooseOtherXiashu,
                payload,
            )
        }
        3 => {
            let mut payload = vec![
                2,
                player.raw(),
                target.raw(),
                skill.raw(),
                cards.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::ChooseOtherXiashu,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault("invalid Xiashu stage".into())),
    }
}

pub(crate) fn resolve_guanchao_rank(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Ok(Vec::new());
    }
    let mark = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::ContentFault("Guanchao skill is missing".into()))?;
    let mode = tx
        .state()
        .players
        .get(player)
        .and_then(|owner| owner.marks.iter().find(|(id, _)| *id == mark))
        .map(|(_, count)| *count)
        .unwrap_or(0);
    if !matches!(mode, 1 | 2) {
        return Ok(Vec::new());
    }
    let previous = (tx.state().turn.flag(turn_flag::LAST_PLAY_CARD_META) as u32 >> 8) & 0xff;
    if previous == 0 {
        return Ok(Vec::new());
    }
    let current = tx
        .state()
        .cards
        .get(card)
        .map(|card| u32::from(card.rank))
        .unwrap_or(0);
    let valid = if mode == 1 {
        current > previous
    } else {
        current < previous
    };
    if valid {
        draw_cards(tx, player, 1)
    } else {
        add_mark(tx, player, mark, 3 - mode)
    }
}

pub(crate) fn resolve_guanwei(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                player.raw(),
                target.raw(),
                skill.raw(),
                options.len().min(u32::MAX as usize) as u32,
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
                ContinuationKind::ResolveGuanwei,
                payload,
            )
        }
        1 => {
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveGuanwei {
                    player,
                    target,
                    stage: 2,
                    card: CardId::NONE,
                    skill,
                },
            );
            discard_cards(tx, player, vec![card])
        }
        2 => {
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveGuanwei {
                    player,
                    target,
                    stage: 3,
                    card: CardId::NONE,
                    skill,
                },
            );
            draw_cards(tx, target, 2)
        }
        3 => insert_extra_play_after_completed_play(tx, target),
        _ => Err(EffectError::ContentFault("invalid Guanwei stage".into())),
    }
}

pub(crate) fn resolve_lanjiang(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    index: u32,
    _target: PlayerId,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let owner_hand = tx.state().hand_count(player);
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| tx.state().hand_count(target) >= owner_hand)
                .collect();
            resolve_lanjiang(tx, player, 1, 0, PlayerId::NONE, targets, skill)
        }
        1 => {
            let Some(target) = targets.get(index as usize).copied() else {
                return resolve_lanjiang(tx, player, 2, 0, PlayerId::NONE, targets, skill);
            };
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return resolve_lanjiang(
                    tx,
                    player,
                    1,
                    index.saturating_add(1),
                    PlayerId::NONE,
                    targets,
                    skill,
                );
            }
            let mut payload = vec![
                0,
                player.raw(),
                index,
                skill.raw(),
                targets.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Confirm,
                ContinuationKind::ResolveLanjiang,
                payload,
            )
        }
        2 => {
            let owner_hand = tx.state().hand_count(player);
            let options: Vec<_> = targets
                .iter()
                .copied()
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|participant| participant.alive)
                        && tx.state().hand_count(*target) == owner_hand
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                1,
                player.raw(),
                skill.raw(),
                targets.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveLanjiang,
                payload,
            )
        }
        3 => {
            let owner_hand = tx.state().hand_count(player);
            let options: Vec<_> = targets
                .iter()
                .copied()
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|participant| participant.alive)
                        && tx.state().hand_count(*target) < owner_hand
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![2, player.raw(), skill.raw()];
            payload.extend(targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveLanjiang,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault("invalid Lanjiang stage".into())),
    }
}

pub(crate) fn resolve_rebingyi(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let mut events = vec![DomainEvent::CardsRevealed {
        cards: cards.clone(),
        to: Vec::new(),
    }];
    let first_color = crate::query::effective_card_color_mask(tx.state(), player, cards[0]);
    let uniform_color = matches!(first_color, 0b01 | 0b10)
        && cards.iter().all(|card| {
            crate::query::effective_card_color_mask(tx.state(), player, *card) == first_color
        });
    if !uniform_color {
        return Ok(events);
    }
    let first_rank = tx.state().cards.get(cards[0]).map(|card| card.rank);
    let self_draw = first_rank.is_some()
        && cards
            .iter()
            .all(|card| tx.state().cards.get(*card).map(|card| card.rank) == first_rank);
    let options = tx.state().players.alive_by_seat();
    let max = cards.len().min(options.len()).min(usize::from(u8::MAX)) as u8;
    if max == 0 {
        if self_draw {
            events.extend(draw_cards(tx, player, 1)?);
        }
        return Ok(events);
    }
    events.extend(open_choice(
        tx,
        player,
        0,
        max,
        true,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveRebingyi,
        vec![
            player.raw(),
            u32::from(max),
            u32::from(self_draw),
            skill.raw(),
        ],
    )?);
    Ok(events)
}

pub(crate) fn resolve_rebotu(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.extra_turn_queue.len() >= 8
    {
        return Ok(Vec::new());
    }
    let current_round = tx.state().turn.round;
    let previous_count = tx
        .state()
        .skills
        .get(skill)
        .and_then(|skill| {
            (skill.state_blob.len() == 12 && &skill.state_blob[..4] == b"BTU1").then(|| {
                (
                    u32::from_le_bytes(skill.state_blob[4..8].try_into().unwrap()),
                    u32::from_le_bytes(skill.state_blob[8..12].try_into().unwrap()),
                )
            })
        })
        .filter(|(round, _)| *round == current_round)
        .map(|(_, count)| count)
        .unwrap_or(0);
    let cap = tx.state().players.alive_by_seat().len().min(3) as u32;
    if previous_count >= cap {
        return Ok(Vec::new());
    }
    let mut blob = b"BTU1".to_vec();
    blob.extend_from_slice(&current_round.to_le_bytes());
    blob.extend_from_slice(&previous_count.saturating_add(1).to_le_bytes());
    tx.state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::EngineFault("Rebotu skill disappeared".into()))?
        .state_blob = blob;
    tx.state_mut().turn.extra_turn_queue.push(player);
    Ok(Vec::new())
}

pub(crate) fn oldimeng_pair_legal(
    tx: &Transaction,
    player: PlayerId,
    first: PlayerId,
    second: PlayerId,
) -> bool {
    first != player
        && second != player
        && first != second
        && [first, second].into_iter().all(|target| {
            tx.state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        })
        && (tx.state().hand_count(first) > 0 || tx.state().hand_count(second) > 0)
        && tx
            .state()
            .hand_count(first)
            .abs_diff(tx.state().hand_count(second))
            <= crate::query::discardable_hand_equip_cards(tx.state(), player).len() as u32
}

pub(crate) fn resolve_oldimeng(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    first: PlayerId,
    second: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let alive = tx.state().players.alive_by_seat();
            let options: Vec<_> = alive
                .iter()
                .copied()
                .filter(|first| {
                    *first != player
                        && alive
                            .iter()
                            .copied()
                            .any(|second| oldimeng_pair_legal(tx, player, *first, second))
                })
                .map(PlayerId::raw)
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
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldimeng,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 if second.is_none() => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|second| oldimeng_pair_legal(tx, player, first, *second))
                .map(PlayerId::raw)
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
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldimeng,
                vec![1, player.raw(), first.raw(), skill.raw()],
            )
        }
        1 => {
            if !oldimeng_pair_legal(tx, player, first, second) {
                return Err(EffectError::Illegal(
                    "Oldimeng target pair became illegal".into(),
                ));
            }
            let first_cards = tx.state().hand_cards(first);
            let second_cards = tx.state().hand_cards(second);
            let first_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, first)
                .ok_or_else(|| EffectError::EngineFault("Oldimeng first hand missing".into()))?;
            let second_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, second)
                .ok_or_else(|| EffectError::EngineFault("Oldimeng second hand missing".into()))?;
            let mut events = move_cards(tx, first_cards, second_hand, Some(first_hand))?;
            events.extend(move_cards(tx, second_cards, first_hand, Some(second_hand))?);
            let mut blob = b"ODM1".to_vec();
            blob.extend_from_slice(&first.raw().to_le_bytes());
            blob.extend_from_slice(&second.raw().to_le_bytes());
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::EngineFault("Oldimeng skill disappeared".into()))?
                .state_blob = blob;
            Ok(events)
        }
        2 => {
            let pair = tx.state().skills.get(skill).and_then(|skill| {
                (skill.state_blob.len() == 12 && &skill.state_blob[..4] == b"ODM1").then(|| {
                    (
                        PlayerId::new(u32::from_le_bytes(
                            skill.state_blob[4..8].try_into().unwrap(),
                        )),
                        PlayerId::new(u32::from_le_bytes(
                            skill.state_blob[8..12].try_into().unwrap(),
                        )),
                    )
                })
            });
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated skill")
                .state_blob
                .clear();
            let Some((first, second)) = pair else {
                return Ok(Vec::new());
            };
            let required = tx
                .state()
                .hand_count(first)
                .abs_diff(tx.state().hand_count(second)) as usize;
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let required = required.min(options.len());
            if required == 0 {
                return Ok(Vec::new());
            }
            if required == options.len() {
                return discard_cards(tx, player, options);
            }
            let mut payload = vec![
                2,
                player.raw(),
                skill.raw(),
                options.len().min(u32::MAX as usize) as u32,
                required as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                required as u8,
                required as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveOldimeng,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault("invalid Oldimeng stage".into())),
    }
}

pub(crate) fn rejieyin_can_equip(tx: &Transaction, target: PlayerId, card: CardId) -> bool {
    tx.state()
        .cards
        .get(card)
        .and_then(|card| card.kind.equip_slot())
        .is_some_and(|slot| {
            tx.state()
                .players
                .get(target)
                .is_some_and(|target| target.alive && target.equip_slot_enabled(slot))
        })
}

pub(crate) fn resolve_rejieyin(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    card: CardId,
    equip: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card) {
                return Err(EffectError::Illegal(
                    "Rejieyin selected card became illegal".into(),
                ));
            }
            let source_is_equip = tx.state().cards.get(card).is_some_and(|card| {
                tx.state()
                    .zones
                    .get(card.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::Equip)
            });
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| {
                    let male = tx
                        .state()
                        .players
                        .get(target)
                        .and_then(|target| tx.state().content.character(target.character_def))
                        .is_some_and(|character| {
                            matches!(character.sex.as_str(), "male" | "double")
                        });
                    male && (!source_is_equip || rejieyin_can_equip(tx, target, card))
                })
                .map(PlayerId::raw)
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
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveRejieyin,
                vec![0, player.raw(), card.raw(), skill.raw()],
            )
        }
        1 => {
            let source_is_equip = tx.state().cards.get(card).is_some_and(|card| {
                tx.state()
                    .zones
                    .get(card.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::Equip)
            });
            if source_is_equip {
                return resolve_rejieyin(tx, player, target, 2, card, true, skill);
            }
            if rejieyin_can_equip(tx, target, card) {
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    vec![0, 1],
                    ChoiceKind::Custom,
                    ContinuationKind::ResolveRejieyin,
                    vec![1, player.raw(), target.raw(), card.raw(), skill.raw()],
                );
            }
            resolve_rejieyin(tx, player, target, 2, card, false, skill)
        }
        2 => {
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveRejieyin {
                    player,
                    target,
                    stage: 3,
                    card: CardId::NONE,
                    equip: false,
                    skill,
                },
            );
            if !equip {
                return discard_cards(tx, player, vec![card]);
            }
            if !rejieyin_can_equip(tx, target, card) {
                return Err(EffectError::Illegal(
                    "Rejieyin equipment target became illegal".into(),
                ));
            }
            let card_state = tx
                .state()
                .cards
                .get(card)
                .cloned()
                .ok_or_else(|| EffectError::Illegal("missing Rejieyin card".into()))?;
            let slot = card_state.kind.equip_slot().unwrap();
            let target_equip = tx
                .state()
                .zones
                .find(ZoneKind::Equip, target)
                .ok_or_else(|| EffectError::EngineFault("target equip zone missing".into()))?;
            let existing: Vec<_> = tx
                .state()
                .zones
                .get(target_equip)
                .into_iter()
                .flat_map(|zone| zone.cards.iter())
                .copied()
                .filter(|existing| {
                    tx.state()
                        .cards
                        .get(*existing)
                        .and_then(|card| card.kind.equip_slot())
                        == Some(slot)
                })
                .collect();
            let mut events = Vec::new();
            if !existing.is_empty() {
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
                events.extend(move_cards(tx, existing, discard, Some(target_equip))?);
            }
            events.extend(move_cards(
                tx,
                vec![card],
                target_equip,
                Some(card_state.zone),
            )?);
            Ok(events)
        }
        3 => {
            let (drawer, recovering) = match (
                tx.state().players.get(player).map(|player| player.hp),
                tx.state().players.get(target).map(|target| target.hp),
            ) {
                (Some(left), Some(right)) if left > right => (player, target),
                (Some(left), Some(right)) if left < right => (target, player),
                _ => return Ok(Vec::new()),
            };
            if tx
                .state()
                .players
                .get(recovering)
                .is_some_and(|participant| participant.hp < participant.max_hp as i32)
            {
                tx.state_mut().resolution.deferred_effects.insert(
                    0,
                    EffectRequest::RecoverHp {
                        source: drawer,
                        player: recovering,
                        amount: 1,
                    },
                );
            }
            draw_cards(tx, drawer, 1)
        }
        _ => Err(EffectError::ContentFault("invalid Rejieyin stage".into())),
    }
}

pub(crate) fn reanxu_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let mut cards = tx.state().hand_cards(player);
    if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, player) {
        cards.extend(
            tx.state()
                .zones
                .get(equip)
                .into_iter()
                .flat_map(|zone| zone.cards.iter())
                .copied(),
        );
    }
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_reanxu(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    first: PlayerId,
    second: PlayerId,
    card: CardId,
    from_hand: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let alive = tx.state().players.alive_by_seat();
            if alive.len() <= 2 {
                return Ok(Vec::new());
            }
            let options: Vec<_> = alive
                .into_iter()
                .filter(|&target| target != player && !reanxu_cards(tx, target).is_empty())
                .map(PlayerId::raw)
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
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveReanxu,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 if second.is_none() => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| target != player && target != first)
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() || reanxu_cards(tx, first).is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveReanxu,
                vec![1, player.raw(), first.raw(), skill.raw()],
            )
        }
        1 => {
            if player == first
                || player == second
                || first == second
                || ![first, second].into_iter().all(|target| {
                    tx.state()
                        .players
                        .get(target)
                        .is_some_and(|participant| participant.alive)
                })
            {
                return Err(EffectError::Illegal(
                    "Reanxu target pair became illegal".into(),
                ));
            }
            let options = reanxu_cards(tx, first);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                second,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveReanxu,
                vec![2, player.raw(), first.raw(), second.raw(), skill.raw()],
            )
        }
        2 => {
            let card_state = tx
                .state()
                .cards
                .get(card)
                .cloned()
                .ok_or_else(|| EffectError::Illegal("missing Reanxu card".into()))?;
            if card_state.owner != first || !reanxu_cards(tx, first).contains(&card) {
                return Err(EffectError::Illegal(
                    "Reanxu selected card became illegal".into(),
                ));
            }
            let from_hand = tx
                .state()
                .zones
                .get(card_state.zone)
                .is_some_and(|zone| zone.kind == ZoneKind::Hand);
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, second)
                .ok_or_else(|| EffectError::EngineFault("Reanxu target hand missing".into()))?;
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveReanxu {
                    player,
                    stage: 3,
                    first,
                    second,
                    card: CardId::NONE,
                    from_hand,
                    skill,
                },
            );
            move_cards(tx, vec![card], hand, Some(card_state.zone))
        }
        3 => {
            if from_hand {
                tx.state_mut().resolution.deferred_effects.insert(
                    0,
                    EffectRequest::ResolveReanxu {
                        player,
                        stage: 4,
                        first,
                        second,
                        card: CardId::NONE,
                        from_hand: false,
                        skill,
                    },
                );
                return draw_cards(tx, player, 1);
            }
            resolve_reanxu(tx, player, 4, first, second, CardId::NONE, false, skill)
        }
        4 => {
            if ![first, second].into_iter().all(|target| {
                tx.state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            }) {
                return Ok(Vec::new());
            }
            let first_hand = tx.state().hand_count(first);
            let second_hand = tx.state().hand_count(second);
            if first_hand == second_hand {
                return Ok(Vec::new());
            }
            let lower = if first_hand < second_hand {
                first
            } else {
                second
            };
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Confirm,
                ContinuationKind::ResolveReanxu,
                vec![3, player.raw(), lower.raw(), skill.raw()],
            )
        }
        _ => Err(EffectError::ContentFault("invalid Reanxu stage".into())),
    }
}

pub(crate) fn resolve_oldanxu(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    giver: PlayerId,
    receiver: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let alive = tx.state().players.alive_by_seat();
            let options = alive
                .iter()
                .copied()
                .filter(|giver| {
                    *giver != player
                        && alive.iter().copied().any(|receiver| {
                            receiver != player
                                && receiver != *giver
                                && tx.state().hand_count(receiver) < tx.state().hand_count(*giver)
                        })
                })
                .map(PlayerId::raw)
                .collect::<Vec<_>>();
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
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldAnxu,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let options: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| {
                    target != player
                        && target != giver
                        && tx.state().hand_count(target) < tx.state().hand_count(giver)
                })
                .map(PlayerId::raw)
                .collect::<Vec<_>>();
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
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldAnxu,
                vec![1, player.raw(), giver.raw(), skill.raw()],
            )
        }
        2 => {
            if player == giver
                || player == receiver
                || giver == receiver
                || tx.state().hand_count(giver) <= tx.state().hand_count(receiver)
            {
                return Err(EffectError::Illegal(
                    "old Anxu target pair became illegal".into(),
                ));
            }
            let options = tx.state().hand_cards(giver);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                receiver,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveOldAnxu,
                vec![2, player.raw(), giver.raw(), receiver.raw(), skill.raw()],
            )
        }
        3 => {
            let card_state = tx
                .state()
                .cards
                .get(card)
                .cloned()
                .ok_or_else(|| EffectError::Illegal("missing old Anxu card".into()))?;
            let giver_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, giver)
                .ok_or_else(|| EffectError::EngineFault("old Anxu giver hand missing".into()))?;
            if card_state.owner != giver
                || card_state.zone != giver_hand
                || tx.state().hand_count(giver) <= tx.state().hand_count(receiver)
            {
                return Err(EffectError::Illegal(
                    "old Anxu selected card became illegal".into(),
                ));
            }
            let receiver_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, receiver)
                .ok_or_else(|| EffectError::EngineFault("old Anxu receiver hand missing".into()))?;
            if card_state.suit != Suit::Spade {
                tx.state_mut()
                    .resolution
                    .deferred_effects
                    .insert(0, EffectRequest::DrawCards { player, count: 1 });
            }
            move_cards(tx, vec![card], receiver_hand, Some(giver_hand))
        }
        _ => Err(EffectError::ContentFault("invalid old Anxu stage".into())),
    }
}

pub(crate) fn resolve_oldqinqing(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    lord: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let Some(lord) = tx
                .state()
                .players
                .iter_live()
                .find(|participant| participant.alive && participant.identity == Identity::Lord)
                .map(|participant| participant.id)
            else {
                return Ok(Vec::new());
            };
            let alive = tx.state().players.alive_by_seat();
            if !alive.iter().copied().any(|candidate| {
                candidate != lord
                    && candidate != player
                    && crate::query::player_in_attack_range(tx.state(), candidate, lord)
            }) {
                return Ok(Vec::new());
            }
            let options = alive
                .into_iter()
                .filter(|candidate| {
                    *candidate != player
                        && crate::query::player_in_attack_range(tx.state(), *candidate, lord)
                })
                .map(PlayerId::raw)
                .collect::<Vec<_>>();
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldQinqing,
                vec![player.raw(), lord.raw(), skill.raw()],
            )
        }
        1 => {
            if ![player, target, lord].into_iter().all(|participant| {
                tx.state()
                    .players
                    .get(participant)
                    .is_some_and(|participant| participant.alive)
            }) || target == player
                || !crate::query::player_in_attack_range(tx.state(), target, lord)
            {
                return Err(EffectError::Illegal(
                    "old Qinqing target became illegal".into(),
                ));
            }
            let mut effects = Vec::new();
            if !crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                TargetCardScope::HandEquip,
            )
            .is_empty()
            {
                effects.push(EffectRequest::ChooseDiscardTargetCard {
                    player,
                    target,
                    scope: TargetCardScope::HandEquip,
                });
            }
            effects.push(EffectRequest::DrawCards {
                player: target,
                count: 1,
            });
            effects.push(EffectRequest::ResolveOldQinqing {
                player,
                target,
                lord,
                stage: 2,
                skill,
            });
            run_effects(tx, effects)
        }
        2 => {
            if tx
                .state()
                .players
                .get(lord)
                .is_some_and(|participant| participant.alive)
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                && tx.state().hand_count(target) > tx.state().hand_count(lord)
            {
                draw_cards(tx, player, 1)
            } else {
                Ok(Vec::new())
            }
        }
        _ => Err(EffectError::ContentFault(
            "invalid old Qinqing stage".into(),
        )),
    }
}

pub(crate) fn resolve_oldmingjian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || tx.state().hand_count(player) == 0
            {
                return Ok(Vec::new());
            }
            let options: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| target != player)
                .map(PlayerId::raw)
                .collect::<Vec<_>>();
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
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldMingjian,
                vec![player.raw(), skill.raw()],
            )
        }
        1 => {
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || tx.state().hand_count(player) == 0
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "old Mingjian target became illegal".into(),
                ));
            }
            if tx.state().turn.extra_play_phase_queue.len() >= 8 {
                return Err(EffectError::ContentFault(
                    "extra Play phase queue exceeds the eight-segment bound".into(),
                ));
            }
            tx.state_mut().turn.extra_play_phase_queue.push(target);
            run_effects(
                tx,
                vec![
                    EffectRequest::GiveAllHand { player, target },
                    EffectRequest::AdvanceTurn,
                ],
            )
        }
        _ => Err(EffectError::ContentFault(
            "invalid old Mingjian stage".into(),
        )),
    }
}
