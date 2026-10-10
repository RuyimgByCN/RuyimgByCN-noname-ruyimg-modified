//! Effect primitive implementations — ddd 1 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

pub(crate) const RESOLVE_WDB_ZAOZHIRENJUN_CHOICE: u32 = 0xDCDC_6160;
pub(crate) const RESOLVE_DDD_XIANXI_CHOICE: u32 = 0xDCDC_6170;
pub(crate) const RESOLVE_DDD_XIAHENG_CHOICE: u32 = 0xDCDC_6180;
pub(crate) const RESOLVE_DDD_LIUCHONG_CHOICE: u32 = 0xDCDC_6190;
pub(crate) const RESOLVE_DDD_DUANBING_CHOICE: u32 = 0xDCDC_6210;
pub(crate) const RESOLVE_DDD_WUZHI_CHOICE: u32 = 0xDCDC_6220;
pub(crate) const DDD_XIANXI_STATE_MAGIC: u32 = 0xDDD0_6170;
pub(crate) const DDD_XIAHENG_STATE_MAGIC: u32 = 0xDDD0_6180;
pub(crate) const DDD_MOYAN_STATE_MAGIC: u32 = 0xDDD0_6190;

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

pub(crate) fn save_ddd_xiaheng_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    deleted: bool,
    targets: &[PlayerId],
) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob.clear();
        instance
            .state_blob
            .extend_from_slice(&DDD_XIAHENG_STATE_MAGIC.to_le_bytes());
        instance.state_blob.push(u8::from(deleted));
        instance
            .state_blob
            .extend_from_slice(&(targets.len().min(u32::MAX as usize) as u32).to_le_bytes());
        for target in targets {
            instance
                .state_blob
                .extend_from_slice(&target.raw().to_le_bytes());
        }
    }
}

pub(crate) fn resolve_ddd_xiaheng(
    tx: &mut Transaction,
    player: PlayerId,
    discard_target: PlayerId,
    damage_target: PlayerId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
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
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_XIAHENG_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
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
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_XIAHENG_CHOICE,
                    2,
                    player.raw(),
                    discard_target.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            let (deleted, mut targets) = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| ddd_xiaheng_state(&instance.state_blob))
                .unwrap_or_default();
            if !targets.contains(&damage_target) {
                targets.push(damage_target);
            }
            let remove = !deleted && discard_target == damage_target;
            save_ddd_xiaheng_state(tx, skill, deleted || targets.len() >= 3, &targets);
            if deleted {
                return Ok(Vec::new());
            }
            let mut effects = Vec::new();
            if discard_target != player && damage_target != player {
                effects.push(EffectRequest::ChangeMaxHp { player, delta: -1 });
            }
            if remove {
                effects.push(EffectRequest::RemoveSkill { skill });
            }
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault(
            "unknown 侠横 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_xiaheng_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DDD_XIAHENG_CHOICE) || cancelled {
        return Err(EffectError::Illegal("stale 侠横 choice".into()));
    }
    match payload.get(1).copied() {
        Some(0) => {
            if payload.len() != 4 {
                return Err(EffectError::Illegal(
                    "bad 侠横 discard target choice".into(),
                ));
            }
            let player = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            let target = PlayerId::new(selections[0]);
            if selections.len() != 1
                || observer != player
                || !originating_skill_is_enabled(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid 侠横 discard target".into()));
            }
            let cards = crate::query::discardable_hand_equip_cards(tx.state(), target);
            let count = cards.len().min(2);
            if count == 0 {
                return resolve_ddd_xiaheng(tx, player, target, PlayerId::NONE, 1, skill)
                    .map(|_| ());
            }
            open_choice(
                tx,
                target,
                count as u8,
                count as u8,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_XIAHENG_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    count as u32,
                ],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() != 6 {
                return Err(EffectError::Illegal("bad 侠横 discard choice".into()));
            }
            let player = PlayerId::new(payload[2]);
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let count = payload[5] as usize;
            let cards = crate::query::discardable_hand_equip_cards(tx.state(), target);
            let unique: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            if observer != target
                || !originating_skill_is_enabled(tx, player, skill)
                || selections.len() != count
                || count != cards.len().min(2)
                || unique.len() != selections.len()
                || selections
                    .iter()
                    .any(|raw| !cards.contains(&CardId::new(*raw)))
            {
                return Err(EffectError::Illegal("invalid 侠横 discard".into()));
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDddXiaheng {
                    player,
                    discard_target: target,
                    damage_target: PlayerId::NONE,
                    action: 1,
                    skill,
                },
            );
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                }],
            )
            .map(|_| ())
        }
        Some(2) => {
            if payload.len() != 5 {
                return Err(EffectError::Illegal("bad 侠横 damage target choice".into()));
            }
            let player = PlayerId::new(payload[2]);
            let discard_target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let target = PlayerId::new(selections[0]);
            if selections.len() != 1
                || observer != player
                || !originating_skill_is_enabled(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid 侠横 damage target".into()));
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDddXiaheng {
                    player,
                    discard_target,
                    damage_target: target,
                    action: 2,
                    skill,
                },
            );
            run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: 0,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal("unknown 侠横 choice stage".into())),
    }
}

pub(crate) fn ddd_moyan_state(blob: &[u8]) -> (u32, PlayerId, u32) {
    if blob.len() != 16
        || u32::from_le_bytes(blob[0..4].try_into().unwrap()) != DDD_MOYAN_STATE_MAGIC
    {
        return (u32::MAX, PlayerId::NONE, 0);
    }
    (
        u32::from_le_bytes(blob[4..8].try_into().unwrap()),
        PlayerId::new(u32::from_le_bytes(blob[8..12].try_into().unwrap())),
        u32::from_le_bytes(blob[12..16].try_into().unwrap()),
    )
}

pub(crate) fn save_ddd_moyan_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    round: u32,
    target: PlayerId,
    count: u32,
) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob.clear();
        instance
            .state_blob
            .extend_from_slice(&DDD_MOYAN_STATE_MAGIC.to_le_bytes());
        instance.state_blob.extend_from_slice(&round.to_le_bytes());
        instance
            .state_blob
            .extend_from_slice(&target.raw().to_le_bytes());
        instance.state_blob.extend_from_slice(&count.to_le_bytes());
    }
}

pub(crate) fn current_processing_cards(tx: &Transaction, cards: Vec<CardId>) -> Vec<CardId> {
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

pub(crate) fn discard_processing_cards(
    tx: &mut Transaction,
    cards: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let cards = current_processing_cards(tx, cards);
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
    run_effects(
        tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards,
            to: discard,
            from: Some(processing),
            reason: CardMoveReason::Discard,
        }],
    )
}

pub(crate) fn resolve_ddd_liuchong(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) || actor != tx.state().turn.current
            {
                return Ok(Vec::new());
            }
            let alive = tx.state().players.alive_by_seat();
            let first_ready = alive
                .iter()
                .copied()
                .find(|candidate| {
                    tx.state()
                        .players
                        .get(*candidate)
                        .is_some_and(|participant| !participant.turned_over)
                })
                .or_else(|| alive.first().copied());
            let round = tx.state().turn.round;
            let prompted = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| ddd_moyan_state(&instance.state_blob).0)
                .unwrap_or(u32::MAX);
            if first_ready != Some(actor) || prompted == round {
                return Ok(Vec::new());
            }
            save_ddd_moyan_state(tx, skill, round, PlayerId::NONE, 0);
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
            let cards: Vec<_> = tx
                .state()
                .zones
                .get(draw)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().rev())
                .take(3)
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
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDddLiuchong {
                    player,
                    actor,
                    card: CardId::NONE,
                    action: 3,
                    cards: cards.clone(),
                    skill,
                },
            );
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCardsWithReason {
                        cards: cards.clone(),
                        to: processing,
                        from: Some(draw),
                        reason: CardMoveReason::Unspecified,
                    },
                    EffectRequest::RevealCards {
                        cards,
                        to: Vec::new(),
                    },
                ],
            )
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let (round, target, count) = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| ddd_moyan_state(&instance.state_blob))
                .unwrap_or((u32::MAX, PlayerId::NONE, 0));
            if actor != player
                || round != tx.state().turn.round
                || count == 0
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || tx.state().hand_count(target) > count
            {
                return Ok(Vec::new());
            }
            run_effects(
                tx,
                vec![EffectRequest::ModifyCurrentDamage {
                    player,
                    delta: 1,
                    skill,
                }],
            )
        }
        2 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || card.is_none()
                || !tx
                    .state()
                    .cards
                    .get(card)
                    .is_some_and(|state| state.kind.equip_slot() == Some(EquipSlot::Weapon))
            {
                return Ok(Vec::new());
            }
            let range = current_attack_range(tx, player);
            if tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .any(|candidate| current_attack_range(tx, candidate) > range)
            {
                return Ok(Vec::new());
            }
            run_effects(
                tx,
                vec![EffectRequest::ChooseOtherDamage {
                    player,
                    amount: 1,
                    skill,
                }],
            )
        }
        3 => {
            let cards = current_processing_cards(tx, cards);
            if !originating_skill_is_enabled(tx, player, skill) {
                return discard_processing_cards(tx, cards);
            }
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let round = tx.state().turn.round;
            let mut payload = vec![
                RESOLVE_DDD_LIUCHONG_CHOICE,
                0,
                player.raw(),
                skill.raw(),
                round,
                cards.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                tx.state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .map(PlayerId::raw)
                    .collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown 刘宠 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_liuchong_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DDD_LIUCHONG_CHOICE) || payload.len() < 6 {
        return Err(EffectError::Illegal("stale 末焱 choice".into()));
    }
    match payload[1] {
        0 => {
            let player = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            let round = payload[4];
            let count = payload[5] as usize;
            if observer != player || payload.len() != 6 + count {
                return Err(EffectError::Illegal("invalid 末焱 target context".into()));
            }
            let cards: Vec<_> = payload[6..].iter().copied().map(CardId::new).collect();
            if cancelled || selections.is_empty() {
                return discard_processing_cards(tx, cards).map(|_| ());
            }
            if selections.len() != 1
                || !originating_skill_is_enabled(tx, player, skill)
                || round != tx.state().turn.round
            {
                return Err(EffectError::Illegal("invalid 末焱 target choice".into()));
            }
            let target = PlayerId::new(selections[0]);
            let cards = current_processing_cards(tx, cards);
            if cards.len() != count
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("illegal 末焱 recipient".into()));
            }
            let mut next = vec![
                RESOLVE_DDD_LIUCHONG_CHOICE,
                1,
                player.raw(),
                target.raw(),
                skill.raw(),
                round,
                count as u32,
            ];
            next.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                1,
                count.min(u8::MAX as usize) as u8,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() < 7 || cancelled || selections.is_empty() {
                return Err(EffectError::Illegal("invalid 末焱 card choice".into()));
            }
            let player = PlayerId::new(payload[2]);
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let round = payload[5];
            let count = payload[6] as usize;
            if observer != target
                || payload.len() != 7 + count
                || selections.len() > count
                || !originating_skill_is_enabled(tx, player, skill)
                || round != tx.state().turn.round
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid 末焱 card context".into()));
            }
            let cards: Vec<_> = payload[7..].iter().copied().map(CardId::new).collect();
            let current = current_processing_cards(tx, cards.clone());
            let unique: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            if current.len() != count
                || unique.len() != selections.len()
                || selections
                    .iter()
                    .any(|raw| !current.contains(&CardId::new(*raw)))
            {
                return Err(EffectError::Illegal("illegal 末焱 cards".into()));
            }
            let selected: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            let rest: Vec<_> = cards
                .into_iter()
                .filter(|card| !selected.contains(card))
                .collect();
            save_ddd_moyan_state(tx, skill, round, target, selected.len() as u32);
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("recipient hand missing".into()))?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let mut effects = vec![EffectRequest::MoveCardsWithReason {
                cards: selected,
                to: hand,
                from: Some(processing),
                reason: CardMoveReason::Unspecified,
            }];
            if !rest.is_empty() {
                effects.push(EffectRequest::MoveCardsWithReason {
                    cards: rest,
                    to: discard,
                    from: Some(processing),
                    reason: CardMoveReason::Discard,
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        _ => Err(EffectError::Illegal("unknown 末焱 choice stage".into())),
    }
}

pub(crate) fn resolve_ddd_duanbing(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            let cards = crate::query::ddd_duanbing_cost_cards(tx.state(), player);
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || cards.is_empty()
            {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_DUANBING_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            if card.is_none() || !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let cards = crate::query::ddd_duanbing_transfer_cards(tx.state(), player, target);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Judge, player)
                .ok_or_else(|| EffectError::EngineFault("短兵 owner judge zone missing".into()))?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Judge, target)
                .ok_or_else(|| EffectError::EngineFault("短兵 target judge zone missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards,
                    to,
                    from: Some(from),
                }],
            )
            .map(|_| Vec::new())
        }
        2 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDddDuanbing {
                    player,
                    target: PlayerId::NONE,
                    card: CardId::NONE,
                    action: 3,
                    skill,
                },
            );
            run_effects(tx, vec![EffectRequest::DrawCards { player, count: 2 }]).map(|_| Vec::new())
        }
        3 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            choose_use_virtual_sha(tx, player, true, false, false, skill, 0, 0)
        }
        _ => Err(EffectError::ContentFault(
            "unknown 3D精选短兵 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_duanbing_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [magic, stage, player, skill] = payload else {
        return Err(EffectError::Illegal("bad 3D精选短兵 choice".into()));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "3D精选短兵 requires one cost card".into(),
        ));
    };
    let selected = CardId::new(*selected);
    if *magic != RESOLVE_DDD_DUANBING_CHOICE
        || *stage != 0
        || cancelled
        || observer != player
        || !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !crate::query::ddd_duanbing_cost_cards(tx.state(), player).contains(&selected)
    {
        return Err(EffectError::Illegal("stale 3D精选短兵 cost".into()));
    }
    let from = tx
        .state()
        .cards
        .get(selected)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("3D精选短兵 cost disappeared".into()))?;
    let judge = tx
        .state()
        .zones
        .find(ZoneKind::Judge, player)
        .ok_or_else(|| EffectError::EngineFault("3D精选短兵 judge zone missing".into()))?;
    tx.state_mut()
        .cards
        .get_mut(selected)
        .expect("validated 3D精选短兵 cost")
        .kind = CardKind::TrickBingliang;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveDddDuanbing {
            player,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 2,
            skill,
        });
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![selected],
            to: judge,
            from: Some(from),
        }],
    )
    .map(|_| ())
}

pub(crate) fn ddd_wuzhi_can_receive_bingliang(tx: &Transaction, target: PlayerId) -> bool {
    tx.state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
        && !tx.state().disabled_judge_zones.contains(&target)
        && tx
            .state()
            .zones
            .find(ZoneKind::Judge, target)
            .and_then(|zone| tx.state().zones.get(zone))
            .is_some_and(|zone| {
                zone.cards.iter().all(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_none_or(|card| card.kind != CardKind::TrickBingliang)
                })
            })
}

pub(crate) fn ddd_wuzhi_xuxiao_card(tx: &Transaction, player: PlayerId, card: CardId) -> bool {
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    tx.state().cards.get(card).is_some_and(|state| {
        Some(state.zone) == discard
            && state.owner.is_none()
            && crate::query::effective_card_kind(tx.state(), player, state).is_basic()
            && crate::query::effective_card_color_mask(tx.state(), player, card) == 0b10
    }) && tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .any(|target| ddd_wuzhi_can_receive_bingliang(tx, target))
}

pub(crate) fn ddd_wuzhi_finish_lingyong_card(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("灵涌处理区不存在".into()))?;
    if tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state| state.zone == processing)
    {
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("灵涌弃牌堆不存在".into()))?;
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ResolveDddWuzhi {
                player,
                card: CardId::NONE,
                action: 5,
                cards: Vec::new(),
                skill,
            });
        return move_cards(tx, vec![card], discard, Some(processing));
    }
    resolve_ddd_wuzhi(tx, player, CardId::NONE, 5, Vec::new(), skill)
}

pub(crate) fn resolve_ddd_wuzhi(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let result = crate::command::judge::flip_judgment(
                tx,
                player,
                crate::command::judge::JudgmentFavor::Neutral,
            )?;
            let result_card = result.card;
            let is_sha = tx.state().cards.get(result_card).is_some_and(|card| {
                crate::query::effective_card_kind(tx.state(), player, card).is_sha()
            });
            if !is_sha {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveDddWuzhi {
                        player,
                        card: result_card,
                        action: 1,
                        cards: Vec::new(),
                        skill,
                    },
                );
            }
            Ok(vec![completed_judgment_event(player, result, CardId::NONE)])
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("灵涌弃牌堆不存在".into()))?;
            if !tx
                .state()
                .cards
                .get(card)
                .is_some_and(|state| state.zone == discard)
            {
                return resolve_ddd_wuzhi(tx, player, CardId::NONE, 5, Vec::new(), skill);
            }
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("灵涌处理区不存在".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveDddWuzhi {
                    player,
                    card,
                    action: 2,
                    cards: Vec::new(),
                    skill,
                });
            move_cards(tx, vec![card], processing, Some(discard))
        }
        2 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || crate::command::play::skill_card_target_spec(tx, player, card).is_err()
            {
                return ddd_wuzhi_finish_lingyong_card(tx, player, card, skill);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Confirm,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_WUZHI_CHOICE,
                    0,
                    player.raw(),
                    skill.raw(),
                    card.raw(),
                ],
            )
        }
        3 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return ddd_wuzhi_finish_lingyong_card(tx, player, card, skill);
            }
            match crate::command::play::skill_card_target_spec(tx, player, card)? {
                crate::command::play::SkillCardTargetSpec::Immediate(targets) => {
                    tx.state_mut().resolution.deferred_tail_effects.push(
                        EffectRequest::ResolveDddWuzhi {
                            player,
                            card,
                            action: 4,
                            cards: Vec::new(),
                            skill,
                        },
                    );
                    crate::command::play::handle_skill_card(tx, player, card, &targets)?;
                    Ok(Vec::new())
                }
                crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
                    open_choice(
                        tx,
                        player,
                        min,
                        max,
                        false,
                        options.into_iter().map(PlayerId::raw).collect(),
                        ChoiceKind::SelectTargets,
                        ContinuationKind::Custom,
                        vec![
                            RESOLVE_DDD_WUZHI_CHOICE,
                            1,
                            player.raw(),
                            skill.raw(),
                            card.raw(),
                        ],
                    )
                }
                crate::command::play::SkillCardTargetSpec::Pairs(pairs) => open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    (1..=pairs.len() as u32).collect(),
                    ChoiceKind::Custom,
                    ContinuationKind::Custom,
                    vec![
                        RESOLVE_DDD_WUZHI_CHOICE,
                        2,
                        player.raw(),
                        skill.raw(),
                        card.raw(),
                    ],
                ),
            }
        }
        4 => ddd_wuzhi_finish_lingyong_card(tx, player, card, skill),
        5 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                vec![0, 1],
                ChoiceKind::Confirm,
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_WUZHI_CHOICE, 3, player.raw(), skill.raw()],
            )
        }
        10 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let mut seen = std::collections::BTreeSet::new();
            let cards: Vec<_> = cards
                .into_iter()
                .filter(|card| seen.insert(card.raw()))
                .filter(|card| ddd_wuzhi_xuxiao_card(tx, player, *card))
                .collect();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_DDD_WUZHI_CHOICE,
                10,
                player.raw(),
                skill.raw(),
                cards.len() as u32,
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown 3D精选吴质 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_wuzhi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DDD_WUZHI_CHOICE) {
        return Err(EffectError::Illegal("bad 3D精选吴质 choice".into()));
    }
    let stage = *payload.get(1).unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal("吴质 choice observer changed".into()));
    }
    match stage {
        0 => {
            let card = CardId::new(*payload.get(4).unwrap_or(&0));
            if cancelled || selections == [0] {
                return run_effects(
                    tx,
                    vec![EffectRequest::ResolveDddWuzhi {
                        player,
                        card,
                        action: 4,
                        cards: Vec::new(),
                        skill,
                    }],
                )
                .map(|_| ());
            }
            if selections != [1] {
                return Err(EffectError::Illegal("灵涌使用选项非法".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddWuzhi {
                    player,
                    card,
                    action: 3,
                    cards: Vec::new(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            if cancelled {
                return Err(EffectError::Illegal("灵涌目标选择不可取消".into()));
            }
            let card = CardId::new(*payload.get(4).unwrap_or(&0));
            let crate::command::play::SkillCardTargetSpec::Select { min, max, options } =
                crate::command::play::skill_card_target_spec(tx, player, card)?
            else {
                return Err(EffectError::Illegal("灵涌牌目标形状已变化".into()));
            };
            let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            let mut seen = std::collections::BTreeSet::new();
            if targets.len() < usize::from(min)
                || targets.len() > usize::from(max)
                || targets
                    .iter()
                    .any(|target| !seen.insert(*target) || !options.contains(target))
            {
                return Err(EffectError::Illegal("灵涌牌目标非法或过期".into()));
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveDddWuzhi {
                    player,
                    card,
                    action: 4,
                    cards: Vec::new(),
                    skill,
                });
            crate::command::play::handle_skill_card(tx, player, card, &targets)
        }
        2 => {
            if cancelled {
                return Err(EffectError::Illegal("灵涌目标对选择不可取消".into()));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("灵涌须选择一个目标对".into()));
            };
            let card = CardId::new(*payload.get(4).unwrap_or(&0));
            let crate::command::play::SkillCardTargetSpec::Pairs(pairs) =
                crate::command::play::skill_card_target_spec(tx, player, card)?
            else {
                return Err(EffectError::Illegal("灵涌牌目标对形状已变化".into()));
            };
            let index = selected
                .checked_sub(1)
                .map(|value| value as usize)
                .ok_or_else(|| EffectError::Illegal("灵涌目标对索引非法".into()))?;
            let (left, right) = pairs
                .get(index)
                .copied()
                .ok_or_else(|| EffectError::Illegal("灵涌目标对已过期".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveDddWuzhi {
                    player,
                    card,
                    action: 4,
                    cards: Vec::new(),
                    skill,
                });
            crate::command::play::handle_skill_card(tx, player, card, &[left, right])
        }
        3 => {
            if cancelled || selections.is_empty() || selections == [0] {
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal("灵涌继续选项非法".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddWuzhi {
                    player,
                    card: CardId::NONE,
                    action: 0,
                    cards: Vec::new(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        10 => {
            let len = *payload.get(4).unwrap_or(&0) as usize;
            let offered = payload
                .get(5..5 + len)
                .ok_or_else(|| EffectError::EngineFault("虚孝候选牌被截断".into()))?;
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("虚孝每次只能选一张牌".into()));
            };
            let card = CardId::new(*selected);
            if !offered.contains(selected) || !ddd_wuzhi_xuxiao_card(tx, player, card) {
                return Err(EffectError::Illegal("虚孝候选牌非法或过期".into()));
            }
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| ddd_wuzhi_can_receive_bingliang(tx, *target))
                .collect();
            let remaining: Vec<_> = offered
                .iter()
                .copied()
                .filter(|candidate| *candidate != card.raw())
                .collect();
            let mut next = vec![
                RESOLVE_DDD_WUZHI_CHOICE,
                11,
                player.raw(),
                skill.raw(),
                card.raw(),
                remaining.len() as u32,
            ];
            next.extend(remaining);
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        11 => {
            if cancelled {
                return Err(EffectError::Illegal("虚孝目标选择不可取消".into()));
            }
            let [target] = selections else {
                return Err(EffectError::Illegal("虚孝须选择一名目标".into()));
            };
            let target = PlayerId::new(*target);
            let card = CardId::new(*payload.get(4).unwrap_or(&0));
            let len = *payload.get(5).unwrap_or(&0) as usize;
            let remaining = payload
                .get(6..6 + len)
                .ok_or_else(|| EffectError::EngineFault("虚孝剩余牌被截断".into()))?;
            if !originating_skill_is_enabled(tx, player, skill)
                || !ddd_wuzhi_xuxiao_card(tx, player, card)
                || !ddd_wuzhi_can_receive_bingliang(tx, target)
            {
                return Err(EffectError::Illegal("虚孝牌或目标已过期".into()));
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("虚孝弃牌堆不存在".into()))?;
            let judge = tx
                .state()
                .zones
                .find(ZoneKind::Judge, target)
                .ok_or_else(|| EffectError::EngineFault("虚孝判定区不存在".into()))?;
            tx.state_mut()
                .cards
                .get_mut(card)
                .expect("validated 虚孝 card")
                .kind = CardKind::TrickBingliang;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: vec![card],
                        to: judge,
                        from: Some(discard),
                    },
                    EffectRequest::DrawCards {
                        player: target,
                        count: 2,
                    },
                    EffectRequest::ResolveDddWuzhi {
                        player,
                        card: CardId::NONE,
                        action: 10,
                        cards: remaining.iter().copied().map(CardId::new).collect(),
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "unknown 3D精选吴质 choice stage".into(),
        )),
    }
}

pub(crate) const RESOLVE_DDD_XUJING_CHOICE: u32 = 0xDCDC_6230;
pub(crate) const DDD_XUYU_STATE: &[u8; 4] = b"DXU1";
pub(crate) const DDD_SHIJIAN_STATE: &[u8; 4] = b"DSJ1";

pub(crate) fn ddd_xuyu_pending(
    state: &crate::state::GameState,
    skill: engine_api::SkillId,
) -> bool {
    state
        .skills
        .get(skill)
        .is_some_and(|instance| instance.state_blob == [b'D', b'X', b'U', b'1', 1])
}

pub(crate) fn set_ddd_xuyu_pending(tx: &mut Transaction, skill: engine_api::SkillId, pending: bool) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = DDD_XUYU_STATE
            .iter()
            .copied()
            .chain(std::iter::once(u8::from(pending)))
            .collect();
    }
}

pub(crate) fn ddd_shijian_watched(
    state: &crate::state::GameState,
    skill: engine_api::SkillId,
) -> Option<PlayerId> {
    let blob = &state.skills.get(skill)?.state_blob;
    if blob.len() != 8 || &blob[..4] != DDD_SHIJIAN_STATE {
        return None;
    }
    let player = PlayerId::new(u32::from_le_bytes(blob[4..8].try_into().ok()?));
    player.is_some().then_some(player)
}

pub(crate) fn set_ddd_shijian_watched(tx: &mut Transaction, skill: engine_api::SkillId, player: PlayerId) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = DDD_SHIJIAN_STATE.to_vec();
        instance
            .state_blob
            .extend_from_slice(&player.raw().to_le_bytes());
    }
}

pub(crate) fn ddd_xujing_hand_equip_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, player))
        .filter_map(|zone| tx.state().zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .collect()
}

pub(crate) fn resolve_ddd_xujing(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    action: u8,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveDddXujing {
                    player,
                    source,
                    card,
                    action: 3,
                    targets: Vec::new(),
                    skill,
                });
            draw_cards(tx, player, 1)
        }
        1 => {
            if !ddd_xuyu_pending(tx.state(), skill) {
                return Ok(Vec::new());
            }
            set_ddd_xuyu_pending(tx, skill, false);
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
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
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_XUJING_CHOICE, 1, player.raw(), skill.raw()],
            )
        }
        2 => {
            set_ddd_xuyu_pending(tx, skill, false);
            Ok(Vec::new())
        }
        3 => {
            if originating_skill_is_enabled(tx, player, skill) {
                set_ddd_xuyu_pending(tx, skill, true);
            }
            Ok(Vec::new())
        }
        10 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || source.is_none()
                || source == player
            {
                return Ok(Vec::new());
            }
            let options = ddd_xujing_hand_equip_cards(tx, player);
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
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_XUJING_CHOICE,
                    10,
                    player.raw(),
                    source.raw(),
                    skill.raw(),
                ],
            )
        }
        11 => {
            if ddd_shijian_watched(tx.state(), skill) != Some(source)
                || card.is_none()
                || targets.is_empty()
            {
                return Ok(Vec::new());
            }
            let kind = tx
                .state()
                .cards
                .get(card)
                .map(|card_state| crate::query::effective_card_kind(tx.state(), source, card_state))
                .ok_or_else(|| EffectError::Illegal("实荐所观察的牌已消失".into()))?;
            let mut payload = vec![
                RESOLVE_DDD_XUJING_CHOICE,
                11,
                player.raw(),
                source.raw(),
                card.raw(),
                kind.tag() as u32,
                skill.raw(),
                targets.len() as u32,
            ];
            payload.extend(targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                payload,
            )
        }
        12 => {
            set_ddd_shijian_watched(tx, skill, PlayerId::NONE);
            Ok(Vec::new())
        }
        13 => {
            if originating_skill_is_enabled(tx, player, skill)
                && tx
                    .state()
                    .players
                    .get(source)
                    .is_some_and(|participant| participant.alive)
            {
                set_ddd_shijian_watched(tx, skill, source);
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown 3D精选许靖 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_xujing_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DDD_XUJING_CHOICE) {
        return Err(EffectError::Illegal("bad 3D精选许靖 choice".into()));
    }
    let stage = *payload.get(1).unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    if observer != player || cancelled {
        return Err(EffectError::Illegal(
            "许靖 choice observer changed or cancelled".into(),
        ));
    }
    match stage {
        1 => {
            let [card] = selections else {
                return Err(EffectError::Illegal("虚誉须弃置一张牌".into()));
            };
            discard_cards(tx, player, vec![CardId::new(*card)]).map(|_| ())
        }
        10 => {
            let [selected] = selections else {
                return Err(EffectError::Illegal("实荐须交给一张牌".into()));
            };
            let source = PlayerId::new(*payload.get(3).unwrap_or(&0));
            let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
            let card = CardId::new(*selected);
            let card_state = tx
                .state()
                .cards
                .get(card)
                .ok_or_else(|| EffectError::Illegal("实荐所选牌已消失".into()))?;
            let from = card_state.zone;
            let owned = card_state.owner == player
                && tx.state().zones.get(from).is_some_and(|zone| {
                    zone.owner == player && matches!(zone.kind, ZoneKind::Hand | ZoneKind::Equip)
                });
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, source)
                .ok_or_else(|| EffectError::Illegal("实荐目标手牌区不存在".into()))?;
            if !owned
                || !originating_skill_is_enabled(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(source)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("实荐牌、技能或目标已过期".into()));
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveDddXujing {
                    player,
                    source,
                    card,
                    action: 13,
                    targets: Vec::new(),
                    skill,
                });
            give_cards(tx, vec![card], to, from).map(|_| ())
        }
        11 => {
            let [branch] = selections else {
                return Err(EffectError::Illegal("实荐须选择一项".into()));
            };
            if *branch > 1 {
                return Err(EffectError::Illegal("实荐选项非法".into()));
            }
            let source = PlayerId::new(*payload.get(3).unwrap_or(&0));
            let card = CardId::new(*payload.get(4).unwrap_or(&0));
            let kind = *payload.get(5).unwrap_or(&0) as u8;
            let skill = engine_api::SkillId::new(*payload.get(6).unwrap_or(&0));
            let len = *payload.get(7).unwrap_or(&0) as usize;
            let targets: Vec<_> = payload
                .get(8..8 + len)
                .ok_or_else(|| EffectError::EngineFault("实荐目标快照被截断".into()))?
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            if ddd_shijian_watched(tx.state(), skill) != Some(source) {
                return Err(EffectError::Illegal("实荐观察状态已过期".into()));
            }
            set_ddd_shijian_watched(tx, skill, PlayerId::NONE);
            if *branch == 0 {
                tx.state_mut()
                    .resolution
                    .register_card_replay(crate::state::PendingCardReplay {
                        source_card: card,
                        player: source,
                        kind,
                        targets,
                        skill,
                    });
                Ok(())
            } else {
                run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }]).map(|_| ())
            }
        }
        _ => Err(EffectError::Illegal(
            "unknown 3D精选许靖 choice stage".into(),
        )),
    }
}

pub(crate) const RESOLVE_DDD_CAOMAO_CHOICE: u32 = 0xDCDC_6240;
pub(crate) const DDD_QUCHE_EFFECT_DEF: u32 = 3_815_717_800;

pub(crate) fn ddd_caomao_entered_discard_cards(tx: &Transaction) -> Vec<CardId> {
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    tx.state()
        .turn
        .entered_discard_cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| Some(card.zone) == discard)
        })
        .collect()
}

pub(crate) fn ddd_caomao_owned_color_cards(tx: &Transaction, player: PlayerId, color: u8) -> Vec<CardId> {
    [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, player))
        .filter_map(|zone| tx.state().zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .filter(|card| crate::query::effective_card_color_mask(tx.state(), player, *card) == color)
        .collect()
}

pub(crate) fn ddd_quche_sha_targets(tx: &Transaction, player: PlayerId, card: CardId) -> Vec<PlayerId> {
    if tx.state().cards.get(card).is_none_or(|state| {
        crate::query::card_use_cancelled_for_logical_kind(tx.state(), player, state, CardKind::Sha)
    }) {
        return Vec::new();
    }
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            crate::command::play::validate_sha_target_with_options(tx, player, card, *target, false)
                .is_ok()
        })
        .collect()
}

pub(crate) fn ddd_quche_mark_count(tx: &Transaction, player: PlayerId) -> u32 {
    tx.state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.marks.iter())
        .find(|(mark, _)| *mark == DDD_QUCHE_EFFECT_DEF)
        .map(|(_, count)| (*count).max(0) as u32)
        .unwrap_or(0)
}

pub(crate) fn ddd_quche_use_sha(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    if !ddd_caomao_owned_color_cards(tx, player, 0b10).contains(&card) || targets.is_empty() {
        return Err(EffectError::Illegal("驱车黑牌或目标已过期".into()));
    }
    let source_was_hand = tx.state().cards.get(card).is_some_and(|card| {
        tx.state()
            .zones
            .get(card.zone)
            .is_some_and(|zone| zone.kind == ZoneKind::Hand)
    });
    add_mark(tx, player, DDD_QUCHE_EFFECT_DEF, 1)?;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveDddCaomao {
            player,
            action: 12,
            cards: Vec::new(),
            targets: Vec::new(),
            skill,
        });
    crate::command::play::play_as_kind(
        tx,
        player,
        card,
        CardKind::Sha,
        &targets,
        false,
        false,
        true,
        source_was_hand,
    )
}

pub(crate) fn resolve_ddd_caomao(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.hp_change_count(player) == 0
                || ddd_caomao_entered_discard_cards(tx).is_empty()
            {
                return Ok(Vec::new());
            }
            let options = tx.state().players.alive_by_seat();
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_CAOMAO_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let target = targets.first().copied().unwrap_or(PlayerId::NONE);
            if tx.state().turn.damage_pair_count(target, player) > 0 {
                draw_cards(tx, player, 2)
            } else {
                Ok(Vec::new())
            }
        }
        10 => {
            let card = cards.first().copied().unwrap_or(CardId::NONE);
            let target = targets.first().copied().unwrap_or(PlayerId::NONE);
            if !originating_skill_is_enabled(tx, player, skill)
                || !ddd_caomao_owned_color_cards(tx, player, 0b01).contains(&card)
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|target| target.alive)
            {
                return Err(EffectError::Illegal("驱车初始牌、技能或目标已过期".into()));
            }
            let from = tx.state().cards.get(card).unwrap().zone;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("驱车目标手牌区不存在".into()))?;
            let mut events = add_mark(tx, player, DDD_QUCHE_EFFECT_DEF, 1)?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveDddCaomao {
                    player,
                    action: 11,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    skill,
                });
            events.extend(give_cards(tx, vec![card], to, from)?);
            Ok(events)
        }
        11 => {
            let options: Vec<_> = ddd_caomao_owned_color_cards(tx, player, 0b10)
                .into_iter()
                .filter(|card| !ddd_quche_sha_targets(tx, player, *card).is_empty())
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_CAOMAO_CHOICE, 20, player.raw(), skill.raw()],
            )
        }
        12 => {
            let max_hp = tx
                .state()
                .players
                .get(player)
                .map(|player| u32::from(player.max_hp))
                .unwrap_or(0);
            if ddd_quche_mark_count(tx, player) / 2 >= max_hp {
                return Ok(Vec::new());
            }
            let options = ddd_caomao_owned_color_cards(tx, player, 0b01);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_CAOMAO_CHOICE, 22, player.raw(), skill.raw()],
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown 3D精选曹髦 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_caomao_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DDD_CAOMAO_CHOICE) {
        return Err(EffectError::Illegal("bad 3D精选曹髦 choice".into()));
    }
    let stage = *payload.get(1).unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal("曹髦 choice observer changed".into()));
    }
    if cancelled {
        return if matches!(stage, 20 | 22) {
            Ok(())
        } else {
            Err(EffectError::Illegal("曹髦强制选择不可取消".into()))
        };
    }
    match stage {
        0 => {
            let [target] = selections else {
                return Err(EffectError::Illegal("太思须选择一名角色".into()));
            };
            let target = PlayerId::new(*target);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|target| target.alive)
            {
                return Err(EffectError::Illegal("太思目标已过期".into()));
            }
            let options = ddd_caomao_entered_discard_cards(tx);
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_CAOMAO_CHOICE,
                    1,
                    player.raw(),
                    skill.raw(),
                    target.raw(),
                ],
            )
            .map(|_| ())
        }
        1 => {
            let [card] = selections else {
                return Err(EffectError::Illegal("太思须选择一张牌".into()));
            };
            let card = CardId::new(*card);
            let target = PlayerId::new(*payload.get(4).unwrap_or(&0));
            if !ddd_caomao_entered_discard_cards(tx).contains(&card)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|target| target.alive)
            {
                return Err(EffectError::Illegal("太思牌或目标已过期".into()));
            }
            let from = tx.state().cards.get(card).unwrap().zone;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("太思目标手牌区不存在".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveDddCaomao {
                    player,
                    action: 1,
                    cards: Vec::new(),
                    targets: vec![target],
                    skill,
                });
            give_cards(tx, vec![card], to, from).map(|_| ())
        }
        20 => {
            let [card] = selections else {
                return Err(EffectError::Illegal("驱车须选择一张黑色牌".into()));
            };
            let card = CardId::new(*card);
            let options = ddd_quche_sha_targets(tx, player, card);
            if options.is_empty() {
                return Err(EffectError::Illegal("驱车黑牌已无合法目标".into()));
            }
            if crate::query::sha_targets_all_for(tx.state(), player) {
                return ddd_quche_use_sha(tx, player, card, options, skill);
            }
            let max = crate::query::sha_target_limit_for(tx.state(), player)
                .min(options.len() as u32)
                .min(u32::from(u8::MAX)) as u8;
            let mut next = vec![
                RESOLVE_DDD_CAOMAO_CHOICE,
                21,
                player.raw(),
                skill.raw(),
                card.raw(),
            ];
            next.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                max,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        21 => ddd_quche_use_sha(
            tx,
            player,
            CardId::new(*payload.get(4).unwrap_or(&0)),
            selections.iter().copied().map(PlayerId::new).collect(),
            skill,
        ),
        22 => {
            let [card] = selections else {
                return Err(EffectError::Illegal("驱车须选择一张红色牌".into()));
            };
            let card = CardId::new(*card);
            if !ddd_caomao_owned_color_cards(tx, player, 0b01).contains(&card) {
                return Err(EffectError::Illegal("驱车红牌已过期".into()));
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
                .collect();
            let mut next = vec![
                RESOLVE_DDD_CAOMAO_CHOICE,
                23,
                player.raw(),
                skill.raw(),
                card.raw(),
            ];
            next.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        23 => {
            let [target] = selections else {
                return Err(EffectError::Illegal("驱车须选择赠牌目标".into()));
            };
            let card = CardId::new(*payload.get(4).unwrap_or(&0));
            let target = PlayerId::new(*target);
            if !ddd_caomao_owned_color_cards(tx, player, 0b01).contains(&card)
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|target| target.alive)
            {
                return Err(EffectError::Illegal("驱车红牌或目标已过期".into()));
            }
            let from = tx.state().cards.get(card).unwrap().zone;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("驱车目标手牌区不存在".into()))?;
            add_mark(tx, player, DDD_QUCHE_EFFECT_DEF, 1)?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveDddCaomao {
                    player,
                    action: 11,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    skill,
                });
            give_cards(tx, vec![card], to, from).map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "unknown 3D精选曹髦 choice stage".into(),
        )),
    }
}

pub(crate) const RESOLVE_DDD_XINXIANYING_CHOICE: u32 = 0xDCDC_6250;
pub(crate) const DDD_DONGCHA_DEF: u32 = 3_036_718_178;

pub(crate) fn ddd_dongcha_move_to_pile(
    tx: &mut Transaction,
    owner: PlayerId,
    from_player: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    if !tx.state().hand_cards(from_player).contains(&card) {
        return Err(EffectError::Illegal("洞察手牌已过期".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, from_player)
        .ok_or_else(|| EffectError::EngineFault("洞察手牌区不存在".into()))?;
    let pile = ensure_skill_pile_zone(tx, owner, DDD_DONGCHA_DEF)?;
    move_cards(tx, vec![card], pile, Some(hand)).map(|_| ())
}

pub(crate) fn ddd_dongcha_gain(
    tx: &mut Transaction,
    owner: PlayerId,
    recipient: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    if !tx
        .state()
        .skill_pile_cards(owner, DDD_DONGCHA_DEF)
        .contains(&card)
    {
        return Err(EffectError::Illegal("洞察‘鉴’已过期".into()));
    }
    let pile = ensure_skill_pile_zone(tx, owner, DDD_DONGCHA_DEF)?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, recipient)
        .ok_or_else(|| EffectError::EngineFault("洞察目标手牌区不存在".into()))?;
    move_cards(tx, vec![card], hand, Some(pile)).map(|_| ())
}

pub(crate) fn resolve_ddd_xinxianying(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            if player == actor
                || ![player, actor].into_iter().all(|participant| {
                    tx.state()
                        .players
                        .get(participant)
                        .is_some_and(|state| state.alive)
                        && !tx.state().hand_cards(participant).is_empty()
                })
            {
                return Ok(Vec::new());
            }
            let options = tx.state().hand_cards(player);
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_XINXIANYING_CHOICE,
                    0,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            let pile = tx.state().skill_pile_cards(player, DDD_DONGCHA_DEF);
            if pile.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                pile.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_XINXIANYING_CHOICE,
                    10,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown 3D精选辛宪英 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_xinxianying_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DDD_XINXIANYING_CHOICE) || cancelled {
        return Err(EffectError::Illegal("bad 3D精选辛宪英 choice".into()));
    }
    let stage = *payload.get(1).unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let actor = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    let [raw] = selections else {
        return Err(EffectError::Illegal("洞察须选择一张牌".into()));
    };
    let card = CardId::new(*raw);
    match stage {
        0 if observer == player => {
            ddd_dongcha_move_to_pile(tx, player, player, card)?;
            let options = tx.state().hand_cards(actor);
            if options.is_empty() {
                return Ok(());
            }
            open_choice(
                tx,
                actor,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_XINXIANYING_CHOICE,
                    1,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        1 if observer == actor => ddd_dongcha_move_to_pile(tx, player, actor, card),
        10 if observer == player => {
            ddd_dongcha_gain(tx, player, player, card)?;
            let remaining = tx.state().skill_pile_cards(player, DDD_DONGCHA_DEF);
            if remaining.is_empty()
                || !tx
                    .state()
                    .players
                    .get(actor)
                    .is_some_and(|state| state.alive)
            {
                return Ok(());
            }
            open_choice(
                tx,
                actor,
                1,
                1,
                false,
                remaining.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_XINXIANYING_CHOICE,
                    11,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        11 if observer == actor => ddd_dongcha_gain(tx, player, actor, card),
        _ => Err(EffectError::Illegal(
            "辛宪英 choice observer or stage changed".into(),
        )),
    }
}
