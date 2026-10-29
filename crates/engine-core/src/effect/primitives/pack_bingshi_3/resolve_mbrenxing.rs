//! Effect primitives for `resolve_mbrenxing` and related helpers.

use super::*;

pub(crate) fn resolve_mbrenxing(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if stage == 1 {
        return run_effects(
            tx,
            vec![EffectRequest::ChooseDiscardTargetCard {
                player,
                target,
                scope: TargetCardScope::HandEquipJudge,
            }],
        );
    }
    let round = tx.state().turn.round;
    let count = tx
        .state()
        .skills
        .get(skill)
        .and_then(|instance| {
            (instance.state_blob.len() == 12 && &instance.state_blob[..4] == b"RNX1")
                .then(|| {
                    let stored = u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap());
                    (stored == round)
                        .then(|| u32::from_le_bytes(instance.state_blob[8..12].try_into().unwrap()))
                })
                .flatten()
        })
        .unwrap_or(0);
    if count >= 2 {
        return Ok(Vec::new());
    }
    let current = tx.state().turn.current;
    let draw_available = tx
        .state()
        .players
        .get(current)
        .is_some_and(|participant| participant.alive);
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            tx.state().turn.flag(turn_flag::sha_used_by_player(*target)) == 0
                && !crate::query::discardable_target_cards(
                    tx.state(),
                    player,
                    *target,
                    TargetCardScope::HandEquipJudge,
                )
                .is_empty()
        })
        .collect();
    let mut options = Vec::new();
    if draw_available {
        options.push(1);
    }
    if !targets.is_empty() {
        options.push(2);
    }
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut blob = b"RNX1".to_vec();
    blob.extend_from_slice(&round.to_le_bytes());
    blob.extend_from_slice(&count.saturating_add(1).to_le_bytes());
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
    let mut payload = vec![
        MBRENXING_CUSTOM_CHOICE,
        0,
        player.raw(),
        current.raw(),
        skill.raw(),
    ];
    payload.extend(targets.iter().map(|target| target.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options,
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn resolve_mbchizhang(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    stage: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if stage == 1 {
        let hand = tx.state().zones.find(ZoneKind::Hand, player);
        if cards.is_empty()
            || cards.iter().any(|selected| {
                !tx.state().cards.get(*selected).is_some_and(|state| {
                    state.owner == player
                        && Some(state.zone) == hand
                        && !crate::query::card_discard_protected_from(
                            tx.state(),
                            player,
                            player,
                            *selected,
                        )
                })
            })
        {
            return Err(EffectError::Illegal("invalid 鸱张 discard".into()));
        }
        let mask = cards.iter().fold(0, |mask, selected| {
            mask | crate::query::effective_card_color_mask(tx.state(), player, *selected)
        });
        tx.state_mut()
            .resolution
            .lock_card_response_colors(card, mask);
        return run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]);
    }
    let valid = tx
        .state()
        .resolution
        .pending_card_resolutions
        .iter()
        .rev()
        .find(|record| record.card == card)
        .is_some_and(|record| record.source == player && record.source_was_hand);
    let options =
        crate::query::discardable_target_cards(tx.state(), player, player, TargetCardScope::Hand);
    if !valid || options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        options.len().min(usize::from(u8::MAX)) as u8,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![
            MBCHIZHANG_CUSTOM_CHOICE,
            player.raw(),
            card.raw(),
            skill.raw(),
        ],
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_mbduanyang(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    mode: u8,
    stage: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let skill_def = tx.state().skills.get(skill).unwrap().def.raw();
    match (mode, stage) {
        (0, 0) => {
            let mut candidates: Vec<_> = cards
                .into_iter()
                .filter(|candidate| {
                    tx.state()
                        .cards
                        .get(*candidate)
                        .is_some_and(|state| state.owner.is_none() && state.kind.is_sha())
                })
                .collect();
            candidates.sort_by_key(|candidate| candidate.raw());
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            let mut rng = RuleRng::from_state(tx.state().rng);
            let selected = candidates[rng.gen_range_u32(candidates.len() as u32) as usize];
            tx.journal(JournalEntry::RngCheckpoint);
            tx.state_mut().rng = rng.into_state();
            let from = tx.state().cards.get(selected).unwrap().zone;
            let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
            if let Some(reference) = tx
                .state_mut()
                .skill_piles
                .iter_mut()
                .find(|reference| reference.owner == player && reference.skill_def == skill_def)
            {
                reference.usable_as_hand = true;
            }
            move_cards_with_reason(
                tx,
                vec![selected],
                pile,
                Some(from),
                CardMoveReason::Unspecified,
            )
        }
        (1, 0) => {
            let pile_cards = tx.state().skill_pile_cards(player, skill_def);
            let playable = pile_cards.into_iter().find_map(|candidate| {
                crate::command::play::skill_card_target_spec_with_options(
                    tx, player, candidate, true, false,
                )
                .ok()
                .map(|spec| (candidate, spec))
            });
            let Some((candidate, spec)) = playable else {
                let cards = tx.state().skill_pile_cards(player, skill_def);
                if cards.is_empty() {
                    return Ok(Vec::new());
                }
                let from = ensure_skill_pile_zone(tx, player, skill_def)?;
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("missing discard pile".into()))?;
                return move_cards_with_reason(
                    tx,
                    cards,
                    discard,
                    Some(from),
                    CardMoveReason::Discard,
                );
            };
            match spec {
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
                            MBDUANYANG_CUSTOM_CHOICE,
                            1,
                            player.raw(),
                            candidate.raw(),
                            skill.raw(),
                        ],
                    )
                }
                _ => Err(EffectError::ContentFault(
                    "断鞅 only stores physical Sha".into(),
                )),
            }
        }
        (1, 1) => {
            let mut blob = b"DYG1".to_vec();
            blob.extend_from_slice(&card.raw().to_le_bytes());
            tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveMbduanyang {
                    player,
                    target: PlayerId::NONE,
                    card: CardId::NONE,
                    mode: 1,
                    stage: 0,
                    cards: Vec::new(),
                    skill,
                },
            );
            crate::command::play::play_as_kind(
                tx,
                player,
                card,
                CardKind::Sha,
                &[target],
                true,
                false,
                false,
                false,
            )?;
            Ok(Vec::new())
        }
        (2, 0) => {
            let options =
                crate::command::play::target_zone_card_tokens(tx, target, true, true, true)
                    .into_iter()
                    .map(CardId::new)
                    .collect::<Vec<_>>();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                options.len().min(2) as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    MBDUANYANG_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        (2, 1) => run_effects(
            tx,
            vec![
                EffectRequest::RecastCards {
                    player: target,
                    cards,
                },
                EffectRequest::DrawCards { player, count: 4 },
            ],
        ),
        _ => Err(EffectError::ContentFault("bad 断鞅 stage".into())),
    }
}

pub(crate) fn mbrunwei_state(
    tx: &Transaction,
    skill: engine_api::SkillId,
) -> (u32, bool, Vec<CardId>) {
    let Some(blob) = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| &instance.state_blob)
    else {
        return (0, false, Vec::new());
    };
    if blob.len() < 16 || &blob[..4] != b"RNW1" {
        return (0, false, Vec::new());
    }
    let remaining = u32::from_le_bytes(blob[4..8].try_into().unwrap());
    let enabled = u32::from_le_bytes(blob[8..12].try_into().unwrap()) != 0;
    let count = u32::from_le_bytes(blob[12..16].try_into().unwrap()) as usize;
    if blob.len() != 16 + count * 4 {
        return (0, false, Vec::new());
    }
    let cards = blob[16..]
        .chunks_exact(4)
        .map(|raw| CardId::new(u32::from_le_bytes(raw.try_into().unwrap())))
        .collect();
    (remaining, enabled, cards)
}

pub(crate) fn set_mbrunwei_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    remaining: u32,
    enabled: bool,
    cards: &[CardId],
) {
    let mut blob = b"RNW1".to_vec();
    blob.extend_from_slice(&remaining.to_le_bytes());
    blob.extend_from_slice(&u32::from(enabled).to_le_bytes());
    blob.extend_from_slice(&(cards.len() as u32).to_le_bytes());
    for card in cards {
        blob.extend_from_slice(&card.raw().to_le_bytes());
    }
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
}

pub(crate) fn resolve_mbrunwei(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    value: u32,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let (_, enabled, _) = mbrunwei_state(tx, skill);
            let uses = tx.state().skills.get(skill).unwrap().play_phase_use_count;
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || uses == 0
                || uses > 2
                || (uses == 2 && !enabled)
            {
                return Err(EffectError::Illegal("润微 is not available".into()));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                vec![1, 2, 3, 4, 5],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![MBRUNWEI_CUSTOM_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let count = value.clamp(1, 5) as usize;
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("missing draw pile".into()))?;
            while tx
                .state()
                .zones
                .get(draw)
                .map_or(0, |zone| zone.cards.len())
                < count
            {
                let before = tx
                    .state()
                    .zones
                    .get(draw)
                    .map_or(0, |zone| zone.cards.len());
                reshuffle_discard_into_draw(tx)?;
                if tx
                    .state()
                    .zones
                    .get(draw)
                    .map_or(0, |zone| zone.cards.len())
                    == before
                {
                    break;
                }
            }
            let cards: Vec<_> = tx
                .state()
                .zones
                .get(draw)
                .map(|zone| zone.cards.iter().rev().take(count).copied().collect())
                .unwrap_or_default();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let mut colors: Vec<u32> = cards
                .iter()
                .map(|card| {
                    u32::from(crate::query::effective_card_color_mask(
                        tx.state(),
                        player,
                        *card,
                    ))
                })
                .filter(|color| *color != 0)
                .collect();
            colors.sort_unstable();
            colors.dedup();
            let mut events = vec![DomainEvent::CardsRevealed {
                cards: cards.clone(),
                to: Vec::new(),
            }];
            if colors.is_empty() {
                events.extend(discard_revealed_mbrunwei_cards(tx, &cards)?);
                return Ok(events);
            }
            let mut payload = vec![MBRUNWEI_CUSTOM_CHOICE, 1, player.raw(), skill.raw()];
            payload.extend(cards.iter().map(|card| card.raw()));
            events.extend(open_choice(
                tx,
                player,
                1,
                1,
                false,
                colors,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                payload,
            )?);
            Ok(events)
        }
        2 => {
            let (_, enabled, _) = mbrunwei_state(tx, skill);
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| !enabled || tx.state().turn.card_gain_count(*target) == 0)
                .collect();
            if targets.is_empty() {
                return discard_revealed_mbrunwei_cards(tx, &cards);
            }
            let mut payload = vec![MBRUNWEI_CUSTOM_CHOICE, 2, player.raw(), skill.raw(), value];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        3 => {
            let draw = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE);
            let valid: Vec<_> = cards
                .iter()
                .copied()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|state| {
                        Some(state.zone) == draw
                            && crate::query::effective_card_color_mask(tx.state(), player, *card)
                                == value as u8
                    })
                })
                .collect();
            if valid.is_empty()
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|target| target.alive)
            {
                return Err(EffectError::Illegal("润微 selection changed".into()));
            }
            let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap();
            let mut events =
                move_cards_with_reason(tx, valid.clone(), hand, draw, CardMoveReason::Unspecified)?;
            let rest: Vec<_> = cards
                .into_iter()
                .filter(|card| !valid.contains(card))
                .collect();
            if !rest.is_empty() {
                events.extend(move_cards_with_reason(
                    tx,
                    rest,
                    discard,
                    draw,
                    CardMoveReason::Unspecified,
                )?);
            }
            let (remaining, enabled, mut tracked) = mbrunwei_state(tx, skill);
            let remaining = if remaining == 0 && !enabled {
                valid.len() as u32
            } else {
                remaining
            };
            if target == player {
                tracked.extend(valid);
                tracked.sort_by_key(|card| card.raw());
                tracked.dedup();
            }
            set_mbrunwei_state(tx, skill, remaining, enabled, &tracked);
            Ok(events)
        }
        4 => {
            let (remaining, enabled, tracked) = mbrunwei_state(tx, skill);
            let remaining = remaining.saturating_sub(cards.len() as u32);
            set_mbrunwei_state(tx, skill, remaining, enabled || remaining == 0, &tracked);
            Ok(Vec::new())
        }
        5 => {
            let (_, _, tracked) = mbrunwei_state(tx, skill);
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let cards: Vec<_> = tracked
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|card| Some(card.zone) == hand)
                })
                .collect();
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .state_blob
                .clear();
            if cards.is_empty() {
                Ok(Vec::new())
            } else {
                run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }])
            }
        }
        _ => Err(EffectError::ContentFault("bad 润微 stage".into())),
    }
}

pub(crate) fn discard_revealed_mbrunwei_cards(
    tx: &mut Transaction,
    cards: &[CardId],
) -> Result<Vec<DomainEvent>, EffectError> {
    let draw = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE);
    let cards: Vec<_> = cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| Some(card.zone) == draw)
        })
        .collect();
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    move_cards_with_reason(tx, cards, discard, draw, CardMoveReason::Unspecified)
}

pub(crate) fn resolve_mbshuanghuai(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    choice: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if stage == 0 {
        let mut options = vec![1];
        let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
        if discard
            .and_then(|zone| tx.state().zones.get(zone))
            .is_some_and(|zone| {
                zone.cards.iter().any(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.kind == CardKind::Tao)
                })
            })
        {
            options.push(2);
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options,
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![
                MBSHUANGHUAI_CUSTOM_CHOICE,
                player.raw(),
                target.raw(),
                skill.raw(),
            ],
        );
    }
    if !matches!(choice, 1 | 2) {
        return Err(EffectError::Illegal("bad 霜怀 choice".into()));
    }
    let previous = tx.state().skills.get(skill).and_then(|instance| {
        (instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"SHH1").then(|| {
            PlayerId::new(u32::from_le_bytes(
                instance.state_blob[4..8].try_into().unwrap(),
            ))
        })
    });
    let mut blob = b"SHH1".to_vec();
    blob.extend_from_slice(&target.raw().to_le_bytes());
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
    let mut effects = if choice == 1 {
        vec![EffectRequest::PreventDamage {
            damage_id: tx.state().resolution.current_damage_id,
        }]
    } else {
        vec![EffectRequest::GainCardByKind {
            player: target,
            kind: CardKind::Tao.tag(),
        }]
    };
    if previous == Some(target) {
        effects.extend([
            EffectRequest::DrawCards { player, count: 1 },
            EffectRequest::DrawCards {
                player: target,
                count: 1,
            },
        ]);
    } else if previous.is_some() {
        effects.push(EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: player,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        });
    }
    run_effects(tx, effects)
}

pub(crate) fn mbxiongtu_flags(tx: &Transaction, skill: engine_api::SkillId) -> u32 {
    tx.state()
        .skills
        .get(skill)
        .and_then(|instance| {
            (instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"XTG1")
                .then(|| u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap()))
        })
        .unwrap_or(0)
}

pub(crate) fn set_mbxiongtu_flags(tx: &mut Transaction, skill: engine_api::SkillId, flags: u32) {
    let instance = tx.state_mut().skills.get_mut(skill).unwrap();
    if flags == 0 {
        instance.state_blob.clear();
    } else {
        instance.state_blob = b"XTG1".to_vec();
        instance.state_blob.extend_from_slice(&flags.to_le_bytes());
    }
}
