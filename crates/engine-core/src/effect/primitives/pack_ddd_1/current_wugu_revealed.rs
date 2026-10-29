//! Effect primitives for `current_wugu_revealed` and related helpers.

use super::*;

pub(crate) fn current_wugu_revealed(tx: &Transaction, cards: Vec<CardId>) -> Vec<CardId> {
    cards
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|state| {
                tx.state()
                    .zones
                    .get(state.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::Processing)
            })
        })
        .collect()
}

pub(crate) fn continue_wugu(
    tx: &mut Transaction,
    action: u8,
    actor: PlayerId,
    source_card: CardId,
    targets: Vec<PlayerId>,
    excluded: Vec<PlayerId>,
    revealed: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            let count = targets.len().saturating_add(
                tx.state()
                    .resolution
                    .ddd_tongyu_extra_reveal(actor, source_card),
            );
            if count == 0 {
                return Ok(Vec::new());
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
                < count
            {
                reshuffle_discard_into_draw(tx)?;
            }
            let cards: Vec<_> = tx
                .state()
                .zones
                .get(draw)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().rev())
                .take(count)
                .copied()
                .collect();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ContinueWugu {
                    action: 1,
                    actor,
                    source_card,
                    targets,
                    excluded,
                    revealed: cards.clone(),
                });
            let mut events = move_cards(tx, cards.clone(), processing, Some(draw))?;
            events.push(DomainEvent::CardsRevealed {
                cards,
                to: Vec::new(),
            });
            Ok(events)
        }
        1 => {
            let revealed = current_wugu_revealed(tx, revealed);
            let targets: Vec<_> = targets
                .into_iter()
                .filter(|target| !excluded.contains(target))
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|player| player.alive)
                })
                .collect();
            if targets.is_empty() || revealed.is_empty() {
                return continue_wugu(tx, 2, actor, source_card, Vec::new(), Vec::new(), revealed);
            }
            let source = tx
                .state()
                .cards
                .get(source_card)
                .ok_or_else(|| EffectError::EngineFault("五谷丰登 carrier missing".into()))?;
            let kind = crate::query::effective_card_kind(tx.state(), actor, source);
            let suit = crate::query::effective_card_suit(tx.state(), actor, source);
            let mut queue = vec![
                source_card.raw(),
                actor.raw(),
                kind.tag() as u32,
                crate::command::judge::suit_tag(suit),
                crate::command::play::trick_effect::WUGU,
                0,
                targets.len().min(u32::MAX as usize) as u32,
            ];
            queue.extend(targets.iter().map(|player| player.raw()));
            queue.push(crate::command::play::WUGU_POOL_MAGIC);
            queue.push(revealed.len().min(u32::MAX as usize) as u32);
            queue.extend(revealed.iter().map(|card| card.raw()));
            crate::command::play::advance_independent_trick_queue(tx, &queue)?;
            Ok(Vec::new())
        }
        2 => {
            let cards = current_wugu_revealed(tx, revealed);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            move_cards_with_reason(
                tx,
                cards,
                discard,
                Some(processing),
                CardMoveReason::WuguRemained,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown 五谷丰登 continuation action".into(),
        )),
    }
}

pub(crate) fn resolve_wdb_zaozhirenjun(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    cards: Vec<CardId>,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let cards: Vec<_> = cards
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| state.zone == discard)
                })
                .collect();
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
            if cards.is_empty() || options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_WDB_ZAOZHIRENJUN_CHOICE,
                0,
                player.raw(),
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
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill) || card.is_none() {
                return Ok(Vec::new());
            }
            let pending_wugu = tx.state().resolution.deferred_tail_effects.iter().find_map(
                |effect| match effect {
                    EffectRequest::ContinueWugu {
                        actor,
                        source_card,
                        targets,
                        ..
                    } if *actor == target && *source_card == card && targets.contains(&player) => {
                        Some(targets.clone())
                    }
                    _ => None,
                },
            );
            let targets = match pending_wugu {
                Some(targets) => targets,
                None => crate::command::play::saved_targeted_card_targets(tx, target, card)?,
            };
            let options: Vec<_> = targets
                .iter()
                .copied()
                .filter(|candidate| {
                    tx.state()
                        .players
                        .get(*candidate)
                        .is_some_and(|participant| participant.alive)
                })
                .map(PlayerId::raw)
                .collect();
            if !targets.contains(&player) || options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_WDB_ZAOZHIRENJUN_CHOICE,
                1,
                player.raw(),
                target.raw(),
                card.raw(),
                skill.raw(),
                targets.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        2 => {
            let Some(participant) = tx.state().players.get(target) else {
                return Ok(Vec::new());
            };
            if !participant.alive {
                return Ok(Vec::new());
            }
            let min_hp = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter_map(|candidate| tx.state().players.get(candidate).map(|p| p.hp))
                .min()
                .unwrap_or(participant.hp);
            if participant.hp == min_hp && participant.hp < participant.max_hp as i32 {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveWdbZaozhiRenjun {
                        player,
                        target,
                        card,
                        cards: Vec::new(),
                        action: 3,
                        skill,
                    },
                );
                run_effects(
                    tx,
                    vec![EffectRequest::RecoverHp {
                        source: player,
                        player: target,
                        amount: 1,
                    }],
                )
            } else {
                resolve_wdb_zaozhirenjun(tx, player, target, card, Vec::new(), 4, skill)
            }
        }
        3 | 4 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let hand = tx.state().hand_count(target);
            let min_hand = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|candidate| tx.state().hand_count(candidate))
                .min()
                .unwrap_or(hand);
            if hand != min_hand {
                return Ok(Vec::new());
            }
            run_effects(
                tx,
                vec![EffectRequest::DrawCards {
                    player: target,
                    count: if action == 3 { 1 } else { 2 },
                }],
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown 枣祗任峻 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_wdb_zaozhirenjun_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_WDB_ZAOZHIRENJUN_CHOICE)
        || payload.len() < 2
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal(
            "stale 枣祗任峻/五谷丰登 choice".into(),
        ));
    }
    match payload[1] {
        0 => {
            if payload.len() < 5 {
                return Err(EffectError::Illegal("bad 粮策 choice".into()));
            }
            let player = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            let count = payload[4] as usize;
            if observer != player
                || payload.len() != 5 + count
                || !originating_skill_is_enabled(tx, player, skill)
            {
                return Err(EffectError::Illegal("invalid 粮策 context".into()));
            }
            let target = PlayerId::new(selections[0]);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid 粮策 recipient".into()));
            }
            let cards = payload[5..].iter().copied().map(CardId::new).collect();
            run_effects(
                tx,
                vec![EffectRequest::GainEventCards {
                    player: target,
                    cards,
                }],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() < 7 {
                return Err(EffectError::Illegal("bad 坚壁 choice".into()));
            }
            let player = PlayerId::new(payload[2]);
            let source = PlayerId::new(payload[3]);
            let card = CardId::new(payload[4]);
            let skill = engine_api::SkillId::new(payload[5]);
            let count = payload[6] as usize;
            let chosen = PlayerId::new(selections[0]);
            if observer != player
                || payload.len() != 7 + count
                || !originating_skill_is_enabled(tx, player, skill)
                || !payload[7..].contains(&chosen.raw())
            {
                return Err(EffectError::Illegal("invalid 坚壁 context".into()));
            }
            if let Some(excluded) = tx
                .state_mut()
                .resolution
                .deferred_tail_effects
                .iter_mut()
                .find_map(|effect| match effect {
                    EffectRequest::ContinueWugu {
                        actor,
                        source_card,
                        targets,
                        excluded,
                        ..
                    } if *actor == source
                        && *source_card == card
                        && targets.contains(&player)
                        && targets.contains(&chosen) =>
                    {
                        Some(excluded)
                    }
                    _ => None,
                })
            {
                if !excluded.contains(&chosen) {
                    excluded.push(chosen);
                }
                Ok(())
            } else {
                crate::command::play::exclude_saved_card_target(tx, card, chosen)
            }
        }
        2 => {
            if payload.len() < 4 {
                return Err(EffectError::Illegal("bad 五谷丰登 pool choice".into()));
            }
            let queue_len = payload[2] as usize;
            if payload.len() != 3 + queue_len || queue_len < 9 {
                return Err(EffectError::Illegal("invalid 五谷丰登 queue".into()));
            }
            let queue = &payload[3..];
            let actor = PlayerId::new(queue[1]);
            let source_card = CardId::new(queue[0]);
            let index = queue[5] as usize;
            let count = queue[6] as usize;
            let targets_end = 7usize.saturating_add(count);
            if index >= count
                || queue.get(targets_end) != Some(&crate::command::play::WUGU_POOL_MAGIC)
            {
                return Err(EffectError::Illegal("invalid 五谷丰登 queue target".into()));
            }
            let revealed_count = *queue.get(targets_end + 1).unwrap_or(&u32::MAX) as usize;
            if queue.len() != targets_end.saturating_add(2).saturating_add(revealed_count) {
                return Err(EffectError::Illegal("invalid 五谷丰登 pool".into()));
            }
            let target = PlayerId::new(queue[7 + index]);
            if observer != target {
                return Err(EffectError::Illegal("五谷丰登 target changed".into()));
            }
            let selected = CardId::new(selections[0]);
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
            let mut revealed: Vec<_> = queue[targets_end + 2..]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            if !revealed.contains(&selected)
                || !tx
                    .state()
                    .cards
                    .get(selected)
                    .is_some_and(|card| card.zone == processing)
            {
                return Err(EffectError::Illegal(
                    "invalid 五谷丰登 revealed card".into(),
                ));
            }
            revealed.retain(|card| *card != selected);
            let remaining = queue[7 + index + 1..targets_end]
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("五谷丰登 target hand missing".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ContinueWugu {
                    action: 1,
                    actor,
                    source_card,
                    targets: remaining,
                    excluded: Vec::new(),
                    revealed,
                });
            run_effects(
                tx,
                vec![EffectRequest::MoveCardsWithReason {
                    cards: vec![selected],
                    to: hand,
                    from: Some(processing),
                    reason: CardMoveReason::Unspecified,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "unknown 枣祗任峻/五谷丰登 choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_xianxi(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    action: u8,
    event_targets: Vec<PlayerId>,
    damaged_targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || card.is_none() {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let options = crate::command::play::saved_sha_additional_targets_ignoring_distance(
                tx, player, card,
            )?;
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_DDD_XIANXI_CHOICE,
                0,
                player.raw(),
                card.raw(),
                skill.raw(),
                options.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                options.len().min(u8::MAX as usize) as u8,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        1 => {
            let tracked = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| instance.state_blob.clone())
                .unwrap_or_default();
            if tracked.len() < 12
                || u32::from_le_bytes(tracked[0..4].try_into().unwrap()) != DDD_XIANXI_STATE_MAGIC
                || u32::from_le_bytes(tracked[4..8].try_into().unwrap()) != card.raw()
                || tracked.len()
                    != 12 + 4 * u32::from_le_bytes(tracked[8..12].try_into().unwrap()) as usize
            {
                return Ok(Vec::new());
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob.clear();
            }
            let mut targets: Vec<_> = tracked[12..]
                .chunks_exact(4)
                .map(|raw| PlayerId::new(u32::from_le_bytes(raw.try_into().unwrap())))
                .filter(|target| {
                    event_targets.contains(target) && !damaged_targets.contains(target)
                })
                .collect();
            targets.sort_by_key(|target| {
                tx.state()
                    .players
                    .get(*target)
                    .map_or(u8::MAX, |participant| participant.seat.raw())
            });
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            let cards = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let count = targets.len().min(u8::MAX as usize) as u8;
            if cards.len() < targets.len() {
                let mut effects = vec![EffectRequest::DrawCards {
                    player,
                    count: targets.len().min(u32::MAX as usize) as u32,
                }];
                effects.push(EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: 0,
                });
                return run_effects(tx, effects);
            }
            let mut payload = vec![
                RESOLVE_DDD_XIANXI_CHOICE,
                1,
                player.raw(),
                skill.raw(),
                targets.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                count,
                count,
                true,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown 险袭 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_xianxi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DDD_XIANXI_CHOICE) || payload.len() < 5 {
        return Err(EffectError::Illegal("stale 险袭 choice".into()));
    }
    let player = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(if payload[1] == 0 {
        *payload.get(4).unwrap_or(&0)
    } else {
        *payload.get(3).unwrap_or(&0)
    });
    if observer != player || !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal("invalid 险袭 choice owner".into()));
    }
    match payload[1] {
        0 => {
            if cancelled {
                return Ok(());
            }
            let count = *payload.get(5).unwrap_or(&0) as usize;
            if payload.len() != 6 + count || selections.is_empty() || selections.len() > count {
                return Err(EffectError::Illegal("invalid 险袭 target choice".into()));
            }
            let card = CardId::new(payload[3]);
            let options = crate::command::play::saved_sha_additional_targets_ignoring_distance(
                tx, player, card,
            )?;
            let unique: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            if unique.len() != selections.len()
                || selections
                    .iter()
                    .any(|raw| !options.contains(&PlayerId::new(*raw)))
            {
                return Err(EffectError::Illegal("illegal 险袭 target".into()));
            }
            let mut targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            targets.sort_by_key(|target| {
                tx.state()
                    .players
                    .get(*target)
                    .map_or(u8::MAX, |participant| participant.seat.raw())
            });
            for target in &targets {
                crate::command::play::add_saved_sha_target_ignoring_distance(
                    tx, player, card, *target,
                )?;
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob.clear();
                instance
                    .state_blob
                    .extend_from_slice(&DDD_XIANXI_STATE_MAGIC.to_le_bytes());
                instance
                    .state_blob
                    .extend_from_slice(&card.raw().to_le_bytes());
                instance.state_blob.extend_from_slice(
                    &(targets.len().min(u32::MAX as usize) as u32).to_le_bytes(),
                );
                for target in &targets {
                    instance
                        .state_blob
                        .extend_from_slice(&target.raw().to_le_bytes());
                }
            }
            Ok(())
        }
        1 => {
            let count = payload[4] as usize;
            if payload.len() != 5 + count || count == 0 {
                return Err(EffectError::Illegal("invalid 险袭 settlement".into()));
            }
            if cancelled {
                return run_effects(
                    tx,
                    vec![
                        EffectRequest::DrawCards {
                            player,
                            count: count.min(u32::MAX as usize) as u32,
                        },
                        EffectRequest::DealDamage {
                            source: PlayerId::NONE,
                            target: player,
                            amount: 1,
                            is_lose_hp: true,
                            nature: 0,
                        },
                    ],
                )
                .map(|_| ());
            }
            if selections.len() != count {
                return Err(EffectError::Illegal(
                    "discard exactly X cards for 险袭".into(),
                ));
            }
            let cards = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let unique: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            if unique.len() != selections.len()
                || selections
                    .iter()
                    .any(|raw| !cards.contains(&CardId::new(*raw)))
            {
                return Err(EffectError::Illegal("illegal 险袭 discard".into()));
            }
            let mut effects = vec![EffectRequest::DiscardCards {
                player,
                cards: selections.iter().copied().map(CardId::new).collect(),
            }];
            effects.extend(
                payload[5..]
                    .iter()
                    .copied()
                    .map(|raw| EffectRequest::DealDamage {
                        source: player,
                        target: PlayerId::new(raw),
                        amount: 1,
                        is_lose_hp: false,
                        nature: 0,
                    }),
            );
            run_effects(tx, effects).map(|_| ())
        }
        _ => Err(EffectError::Illegal("unknown 险袭 choice stage".into())),
    }
}

pub(crate) fn ddd_xiaheng_state(blob: &[u8]) -> (bool, Vec<PlayerId>) {
    if blob.len() < 9
        || u32::from_le_bytes(blob[0..4].try_into().unwrap()) != DDD_XIAHENG_STATE_MAGIC
    {
        return (false, Vec::new());
    }
    let count = u32::from_le_bytes(blob[5..9].try_into().unwrap()) as usize;
    if blob[4] > 1 || blob.len() != 9usize.saturating_add(count.saturating_mul(4)) {
        return (false, Vec::new());
    }
    (
        blob[4] != 0,
        blob[9..]
            .chunks_exact(4)
            .map(|raw| PlayerId::new(u32::from_le_bytes(raw.try_into().unwrap())))
            .collect(),
    )
}
