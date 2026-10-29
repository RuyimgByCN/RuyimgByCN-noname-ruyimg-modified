//! Effect primitives for `resolve_dc_sb_zhangxiu` and related helpers.

use super::*;

pub(crate) fn resolve_dc_sb_zhangxiu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    action: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != target
                || tx.state().turn.phase != Phase::Play
                || target == player
            {
                return Ok(Vec::new());
            }
            let max_hand = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|candidate| tx.state().hand_count(candidate))
                .max()
                .unwrap_or(0);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|target_state| target_state.alive)
                || tx.state().hand_count(target) != max_hand
            {
                return Ok(Vec::new());
            }
            let can_give = !dc_sb_zhangxiu_hand_equip_cards(tx, player).is_empty();
            let can_discard = !crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            )
            .is_empty();
            let can_sha = crate::command::play::source_free_sha_can_target_without_distance(
                tx.state(),
                player,
                target,
            );
            let mut options = Vec::new();
            if can_give {
                options.push(0);
            }
            if can_discard || can_sha {
                options.push(1);
            }
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
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DC_SB_ZHANGXIU_CHOICE,
                    0,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => try_use_virtual_sha_on_without_distance(tx, player, target, false, 0),
        10 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::End
            {
                return Ok(Vec::new());
            }
            let cards = dcsbhaoyi_cards(tx);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let events = gain_event_cards(tx, player, &cards)?;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDcSbZhangxiu {
                    player,
                    target: PlayerId::NONE,
                    action: 11,
                    cards,
                    skill,
                },
            );
            Ok(events)
        }
        11 => {
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("Haoyi owner hand missing".into()))?;
            let mut cards: Vec<_> = cards
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.owner == player && card.zone == hand)
                })
                .collect();
            cards.sort_by_key(|card| card.raw());
            cards.dedup();
            let has_other = tx
                .state()
                .players
                .iter_live()
                .any(|candidate| candidate.alive && candidate.id != player);
            if cards.is_empty() || !has_other {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_DC_SB_ZHANGXIU_CHOICE,
                3,
                player.raw(),
                skill.raw(),
                cards.len() as u32,
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                cards.len().min(u8::MAX as usize) as u8,
                true,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown DC SB Zhangxiu action".into(),
        )),
    }
}

pub(crate) fn resolve_dc_sb_zhangxiu_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DC_SB_ZHANGXIU_CHOICE) || payload.len() < 4 {
        return Err(EffectError::EngineFault(
            "truncated DC SB Zhangxiu choice".into(),
        ));
    }
    let mode = payload[1];
    let player = PlayerId::new(payload[2]);
    if observer != player {
        return Err(EffectError::Illegal("Zhangxiu chooser mismatch".into()));
    }
    match mode {
        0 => {
            if cancelled || selections.len() != 1 || payload.len() != 5 {
                return Err(EffectError::Illegal("invalid Fuxi branch".into()));
            }
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            match selections[0] {
                0 => {
                    let cards = dc_sb_zhangxiu_hand_equip_cards(tx, player);
                    if cards.is_empty() {
                        return Err(EffectError::Illegal("Fuxi give card is stale".into()));
                    }
                    let mut next = vec![
                        RESOLVE_DC_SB_ZHANGXIU_CHOICE,
                        1,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                        cards.len() as u32,
                    ];
                    next.extend(cards.iter().map(|card| card.raw()));
                    open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        cards.into_iter().map(CardId::raw).collect(),
                        ChoiceKind::SelectCards,
                        ContinuationKind::Custom,
                        next,
                    )
                    .map(|_| ())
                }
                1 => {
                    let cards = crate::query::discardable_target_cards(
                        tx.state(),
                        player,
                        target,
                        rule_api::TargetCardScope::HandEquip,
                    );
                    if cards.is_empty() {
                        return run_effects(
                            tx,
                            vec![EffectRequest::ResolveDcSbZhangxiu {
                                player,
                                target,
                                action: 1,
                                cards: Vec::new(),
                                skill,
                            }],
                        )
                        .map(|_| ());
                    }
                    let mut next = vec![
                        RESOLVE_DC_SB_ZHANGXIU_CHOICE,
                        2,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                        cards.len() as u32,
                    ];
                    next.extend(cards.iter().map(|card| card.raw()));
                    open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        cards.into_iter().map(CardId::raw).collect(),
                        ChoiceKind::SelectCards,
                        ContinuationKind::Custom,
                        next,
                    )
                    .map(|_| ())
                }
                _ => Err(EffectError::Illegal("unknown Fuxi branch".into())),
            }
        }
        1 | 2 => {
            if cancelled || selections.len() != 1 || payload.len() < 6 {
                return Err(EffectError::Illegal("invalid Fuxi card choice".into()));
            }
            let count = payload[5] as usize;
            if payload.len() != 6 + count || !payload[6..].contains(&selections[0]) {
                return Err(EffectError::Illegal("stale Fuxi card choice".into()));
            }
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let card = CardId::new(selections[0]);
            if mode == 1 {
                let from = tx
                    .state()
                    .cards
                    .get(card)
                    .filter(|card| card.owner == player)
                    .map(|card| card.zone)
                    .ok_or_else(|| EffectError::Illegal("Fuxi give card moved".into()))?;
                let to = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, target)
                    .ok_or_else(|| EffectError::Illegal("Fuxi target hand missing".into()))?;
                run_effects(
                    tx,
                    vec![
                        EffectRequest::GiveCards {
                            cards: vec![card],
                            to,
                            from,
                        },
                        EffectRequest::DrawCards { player, count: 2 },
                    ],
                )
                .map(|_| ())
            } else {
                if !crate::query::discardable_target_cards(
                    tx.state(),
                    player,
                    target,
                    rule_api::TargetCardScope::HandEquip,
                )
                .contains(&card)
                {
                    return Err(EffectError::Illegal("Fuxi discard card moved".into()));
                }
                run_effects(
                    tx,
                    vec![
                        EffectRequest::DiscardCards {
                            player: target,
                            cards: vec![card],
                        },
                        EffectRequest::ResolveDcSbZhangxiu {
                            player,
                            target,
                            action: 1,
                            cards: Vec::new(),
                            skill,
                        },
                    ],
                )
                .map(|_| ())
            }
        }
        3 => {
            if cancelled {
                return Ok(());
            }
            if selections.is_empty() || payload.len() < 5 {
                return Err(EffectError::Illegal("invalid Haoyi card choice".into()));
            }
            let count = payload[4] as usize;
            if payload.len() != 5 + count
                || selections.iter().any(|card| !payload[5..].contains(card))
            {
                return Err(EffectError::Illegal("stale Haoyi card choice".into()));
            }
            let mut selected = selections.to_vec();
            selected.sort_unstable();
            selected.dedup();
            if selected.len() != selections.len() {
                return Err(EffectError::Illegal("duplicate Haoyi card".into()));
            }
            let mut remaining: Vec<_> = payload[5..]
                .iter()
                .copied()
                .filter(|card| !selected.contains(card))
                .collect();
            remaining.sort_unstable();
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
                .collect();
            if targets.is_empty() {
                return Ok(());
            }
            let mut next = vec![
                RESOLVE_DC_SB_ZHANGXIU_CHOICE,
                4,
                player.raw(),
                payload[3],
                selected.len() as u32,
            ];
            next.extend(selected);
            next.push(remaining.len() as u32);
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
        4 => {
            if cancelled || selections.len() != 1 || payload.len() < 6 {
                return Err(EffectError::Illegal("invalid Haoyi target choice".into()));
            }
            let selected_count = payload[4] as usize;
            let remaining_index = 5 + selected_count;
            if payload.len() <= remaining_index {
                return Err(EffectError::EngineFault("truncated Haoyi plan".into()));
            }
            let remaining_count = payload[remaining_index] as usize;
            if payload.len() != remaining_index + 1 + remaining_count {
                return Err(EffectError::EngineFault("malformed Haoyi plan".into()));
            }
            let target = PlayerId::new(selections[0]);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|target| target.alive)
            {
                return Err(EffectError::Illegal("Haoyi target is stale".into()));
            }
            let cards: Vec<_> = payload[5..remaining_index]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let remaining: Vec<_> = payload[remaining_index + 1..]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("Haoyi owner hand missing".into()))?;
            if cards.iter().any(|card| {
                !tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.owner == player && card.zone == hand)
            }) {
                return Err(EffectError::Illegal("Haoyi cards moved".into()));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("Haoyi target hand missing".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::GiveCards {
                        cards,
                        to,
                        from: hand,
                    },
                    EffectRequest::ResolveDcSbZhangxiu {
                        player,
                        target: PlayerId::NONE,
                        action: 11,
                        cards: remaining,
                        skill: engine_api::SkillId::new(payload[3]),
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown DC SB Zhangxiu choice".into(),
        )),
    }
}

pub(crate) fn dcsbyingmou_sequence(
    instance: &crate::state::SkillInstance,
) -> Option<DcsbyingmouSequence> {
    (instance.state_blob.len() == 17
        && instance.state_blob.get(..4) == Some(DCSBYINGMOU_STATE_TAG.as_slice()))
    .then(|| {
        let flags = instance.state_blob[16];
        DcsbyingmouSequence {
            target: PlayerId::new(u32::from_le_bytes(
                instance.state_blob[4..8].try_into().unwrap(),
            )),
            source: PlayerId::new(u32::from_le_bytes(
                instance.state_blob[8..12].try_into().unwrap(),
            )),
            card: CardId::new(u32::from_le_bytes(
                instance.state_blob[12..16].try_into().unwrap(),
            )),
            any_damage: flags & 1 != 0,
            had_damage_cards: flags & 2 != 0,
            draw_branch: flags & 4 != 0,
        }
    })
}

pub(crate) fn set_dcsbyingmou_sequence(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    sequence: DcsbyingmouSequence,
) -> Result<(), EffectError> {
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::Illegal("Yingmou skill disappeared".into()))?;
    instance.state_blob = DCSBYINGMOU_STATE_TAG.to_vec();
    instance
        .state_blob
        .extend_from_slice(&sequence.target.raw().to_le_bytes());
    instance
        .state_blob
        .extend_from_slice(&sequence.source.raw().to_le_bytes());
    instance
        .state_blob
        .extend_from_slice(&sequence.card.raw().to_le_bytes());
    instance.state_blob.push(
        u8::from(sequence.any_damage)
            | u8::from(sequence.had_damage_cards) << 1
            | u8::from(sequence.draw_branch) << 2,
    );
    Ok(())
}

pub(crate) fn dcsbyingmou_tracks_card(
    instance: &crate::state::SkillInstance,
    card: CardId,
) -> bool {
    card.is_some() && dcsbyingmou_sequence(instance).is_some_and(|state| state.card == card)
}

pub(crate) fn dcsbyingmou_damage_cards(tx: &Transaction, source: PlayerId) -> Vec<CardId> {
    let mut cards: Vec<_> = tx
        .state()
        .hand_cards(source)
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card_state| {
                let kind = crate::query::effective_card_kind(tx.state(), source, card_state);
                kind.is_sha() || kind.is_damage_trick()
            })
        })
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards
}

pub(crate) fn finish_dcsbyingmou(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(sequence) = tx.state().skills.get(skill).and_then(dcsbyingmou_sequence) else {
        return Ok(Vec::new());
    };
    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .state_blob
        .clear();
    if sequence.any_damage
        || !tx
            .state()
            .players
            .get(sequence.target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: player,
            target: sequence.target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
        }],
    )
}
