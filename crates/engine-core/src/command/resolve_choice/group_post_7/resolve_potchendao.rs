//! Command operations for `resolve_potweiyan_choice` and related helpers.

use super::*;

pub(crate) fn resolve_potchendao(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload[0] != crate::effect::POTCHENDAO_CUSTOM_CHOICE {
        return Err(EffectError::Illegal("invalid 势陈到 selection".into()));
    }
    match payload[1] {
        0 if payload.len() >= 6 && observer.raw() == payload[2] => {
            if cancelled {
                return Ok(());
            }
            if selections.len() != 1 || !payload[5..].contains(&selections[0]) {
                return Err(EffectError::Illegal("invalid 往烈 card".into()));
            }
            crate::effect::arm_potwanglie(
                tx,
                observer,
                CardId::new(selections[0]),
                SkillId::new(payload[4]),
            )
        }
        1 if payload.len() == 4 && observer.raw() == payload[2] && !cancelled => {
            if selections.len() != 1 || selections[0] > 1 {
                return Err(EffectError::Illegal("invalid 弘毅 branch".into()));
            }
            crate::effect::commit_pothongyi_prepare(
                tx,
                observer,
                SkillId::new(payload[3]),
                selections[0] as u8,
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal("invalid 势陈到 choice stage".into())),
    }
}

pub(crate) fn resolve_mbzhuji(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4
        || payload[0] != crate::effect::MBZHUJI_CUSTOM_CHOICE
        || observer.raw() != payload[2]
    {
        return Err(EffectError::Illegal("invalid 筑墼 selection".into()));
    }
    if cancelled && payload[1] == 0 {
        return Ok(());
    }
    if cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("invalid 筑墼 selection".into()));
    }
    let (stage, choice) = match (payload[1], selections[0]) {
        (0, choice @ 1..=4) => (1, choice as u8),
        (1, choice @ 1..=3) => (5, choice as u8),
        _ => return Err(EffectError::Illegal("invalid 筑墼 selection".into())),
    };
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbzhuji {
            player: observer,
            choice,
            stage,
            skill: SkillId::new(payload[3]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_potpoxi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 || payload[0] != crate::effect::POTPOXI_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("truncated 破袭 selection".into()));
    }
    let player = PlayerId::new(payload[1]);
    let card = CardId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    let remaining_count = payload[4] as usize;
    let legal_count_at = 5usize.saturating_add(remaining_count);
    let legal_count = *payload
        .get(legal_count_at)
        .ok_or_else(|| EffectError::EngineFault("truncated 破袭 target count".into()))?
        as usize;
    if payload.len() != legal_count_at.saturating_add(1).saturating_add(legal_count)
        || observer != player
        || cancelled
        || selections.is_empty()
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "破袭 selection context changed".into(),
        ));
    }
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    let legal_now = crate::command::play::prompted_sha_targets_unlimited(tx, player, card);
    if selected.len() != selections.len()
        || !selected
            .iter()
            .all(|target| payload[legal_count_at + 1..].contains(target))
        || !selections
            .iter()
            .all(|target| legal_now.contains(&PlayerId::new(*target)))
    {
        return Err(EffectError::Illegal(
            "破袭 targets are no longer legal".into(),
        ));
    }
    let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
    let owner_hand = tx.state().hand_count(player);
    let unrespondable: Vec<_> = targets
        .iter()
        .copied()
        .filter(|target| tx.state().hand_count(*target) <= owner_hand)
        .collect();
    let remaining: Vec<_> = payload[5..legal_count_at]
        .iter()
        .copied()
        .map(CardId::new)
        .collect();
    if !remaining.is_empty() {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ResolvePotpoxi {
                player,
                cards: remaining,
                skill,
            });
    }
    crate::command::play::play_discard_sha_unlimited(tx, player, card, &targets, &unrespondable)
}

pub(crate) fn resolve_potfuan(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5
        || payload[0] != crate::effect::POTFUAN_CUSTOM_CHOICE
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("invalid 伏暗 selection".into()));
    }
    match payload[1] {
        0 if payload.len() == 6 && observer.raw() == payload[2] => {
            crate::effect::commit_potfuan_suit(
                tx,
                observer,
                selections[0],
                SkillId::new(payload[5]),
            )
        }
        1 if payload.len() >= 9 && observer.raw() == payload[2] => {
            if !payload[8..].contains(&selections[0]) {
                return Err(EffectError::Illegal("invalid 伏暗 target".into()));
            }
            crate::effect::commit_potfuan_target(
                tx,
                observer,
                PlayerId::new(selections[0]),
                CardId::new(payload[3]),
                payload[4],
                SkillId::new(payload[7]),
            )
        }
        2 if payload.len() == 5 && observer.raw() == payload[2] => {
            crate::effect::commit_potfuan_branch(
                tx,
                observer,
                PlayerId::new(payload[3]),
                selections[0],
                SkillId::new(payload[4]),
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal("invalid 伏暗 choice stage".into())),
    }
}

pub(crate) fn resolve_mbjianji(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 7
        || payload[0] != crate::effect::MBJIANJI_CUSTOM_CHOICE
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("invalid 间计 selection".into()));
    }
    let player = PlayerId::new(payload[2]);
    let shared = CardId::new(payload[3]);
    let first = PlayerId::new(payload[4]);
    let second = PlayerId::new(payload[5]);
    let skill = SkillId::new(payload[6]);
    let selected = CardId::new(selections[0]);
    let valid = |participant: PlayerId| {
        selected == shared
            || tx.state().cards.get(selected).is_some_and(|card| {
                card.owner == participant
                    && tx.state().zones.get(card.zone).is_some_and(|zone| {
                        zone.kind == ZoneKind::Hand && zone.owner == participant
                    })
            })
    };
    match payload[1] {
        1 if observer == first && valid(first) => {
            let mut options: Vec<_> = tx
                .state()
                .hand_cards(second)
                .into_iter()
                .map(CardId::raw)
                .collect();
            options.push(shared.raw());
            options.sort_unstable();
            options.dedup();
            crate::effect::open_choice(
                tx,
                second,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    crate::effect::MBJIANJI_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    shared.raw(),
                    first.raw(),
                    second.raw(),
                    skill.raw(),
                    selected.raw(),
                ],
            )
            .map(|_| ())
        }
        2 if payload.len() == 8 && observer == second && valid(second) => {
            crate::effect::commit_mbjianji_compare(
                tx,
                player,
                first,
                second,
                shared,
                CardId::new(payload[7]),
                selected,
                skill,
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "间计 comparison context changed".into(),
        )),
    }
}

pub(crate) fn resolve_mbyuanmo(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4
        || payload[0] != crate::effect::MBYUANMO_CUSTOM_CHOICE
        || observer != PlayerId::new(payload[2])
    {
        return Err(EffectError::EngineFault("bad 远谟 continuation".into()));
    }
    let player = PlayerId::new(payload[2]);
    match payload[1] {
        0 if !cancelled && selections.len() == 1 => {
            let card = CardId::new(selections[0]);
            let card_state = tx
                .state()
                .cards
                .get(card)
                .ok_or_else(|| EffectError::Illegal("远谟 field card missing".into()))?;
            let source = tx
                .state()
                .zones
                .get(card_state.zone)
                .filter(|zone| matches!(zone.kind, ZoneKind::Equip | ZoneKind::Judge))
                .map(|zone| zone.owner)
                .ok_or_else(|| EffectError::Illegal("远谟 card left the field".into()))?;
            let destinations = crate::query::field_card_move_destinations(tx.state(), card);
            if destinations.is_empty() {
                return Err(EffectError::Illegal(
                    "远谟 card is no longer movable".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                destinations.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    crate::effect::MBYUANMO_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    payload[3],
                    card.raw(),
                    source.raw(),
                    crate::effect::attack_range_population(tx, source),
                ],
            )
            .map(|_| ())
        }
        1 if payload.len() == 7 && !cancelled && selections.len() == 1 => {
            let skill = SkillId::new(payload[3]);
            let card = CardId::new(payload[4]);
            let source = PlayerId::new(payload[5]);
            let destination = PlayerId::new(selections[0]);
            if !crate::query::field_card_move_destinations(tx.state(), card).contains(&destination)
            {
                return Err(EffectError::Illegal("远谟 destination changed".into()));
            }
            let from = tx.state().cards.get(card).unwrap().zone;
            let kind = tx.state().zones.get(from).unwrap().kind;
            let to = tx
                .state()
                .zones
                .find(kind, destination)
                .ok_or_else(|| EffectError::Illegal("远谟 destination zone missing".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: vec![card],
                        to,
                        from: Some(from),
                    },
                    EffectRequest::ResolveMbyuanmo {
                        player,
                        target: source,
                        stage: 1,
                        card,
                        value: payload[6],
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        2 if cancelled && selections.is_empty() => Ok(()),
        2 if payload.len() == 6 && !cancelled && selections == [1] => run_effects(
            tx,
            vec![EffectRequest::ResolveMbyuanmo {
                player,
                target: PlayerId::new(payload[3]),
                stage: 2,
                card: CardId::NONE,
                value: payload[4],
                skill: SkillId::new(payload[5]),
            }],
        )
        .map(|_| ()),
        _ => Err(EffectError::Illegal("invalid 远谟 selection".into())),
    }
}

pub(crate) fn resolve_refuhai(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::EngineFault("truncated Refuhai payload".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let index = payload[3] as usize;
    let count = payload[4] as usize;
    if payload[0] != crate::effect::REFUHAI_CUSTOM_CHOICE
        || count == 0
        || index >= count
        || payload.len() != 5 + count + index
        || cancelled
        || selections.len() != 1
        || !matches!(selections[0], 1 | 2)
    {
        return Err(EffectError::Illegal("invalid Refuhai choice".into()));
    }
    let participants: Vec<_> = payload[5..5 + count]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    if observer != participants[index]
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal("Refuhai context changed".into()));
    }
    let mut choices = payload[5 + count..].to_vec();
    choices.push(selections[0]);
    if index + 1 < count {
        let mut next_payload = vec![
            crate::effect::REFUHAI_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            (index + 1) as u32,
            count as u32,
        ];
        next_payload.extend(participants.iter().map(|target| target.raw()));
        next_payload.extend(choices);
        return crate::effect::open_choice(
            tx,
            participants[index + 1],
            1,
            1,
            false,
            vec![1, 2],
            engine_api::ChoiceKind::Custom,
            ContinuationKind::Custom,
            next_payload,
        )
        .map(|_| ());
    }
    let mut reveal = Vec::with_capacity(count * 2);
    for (participant, choice) in participants.iter().zip(&choices) {
        reveal.extend([participant.raw(), *choice]);
    }
    tx.push_event(engine_api::DomainEvent::RuleNote {
        code: 0x4930_0001,
        payload: reveal,
    });
    let run = choices
        .iter()
        .take_while(|choice| **choice == choices[0])
        .count() as u32;
    if run > 1 {
        run_effects(
            tx,
            vec![EffectRequest::DrawCardsFromSkill {
                player,
                count: run,
                skill,
            }],
        )?;
    }
    Ok(())
}

pub(crate) fn resolve_yangwu_gain_hand(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [marker, player_raw, target_raw, skill_raw, count] = payload else {
        return Err(EffectError::EngineFault("bad Yangwu continuation".into()));
    };
    let player = PlayerId::new(*player_raw);
    let target = PlayerId::new(*target_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    if *marker != crate::effect::YANGWU_GAIN_HAND_CUSTOM_CHOICE
        || observer != player
        || cancelled
        || selections.len() != *count as usize
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal("invalid Yangwu hand choice".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("Yangwu target hand missing".into()))?;
    let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
    let mut seen = std::collections::BTreeSet::new();
    if cards.iter().any(|card| {
        !seen.insert(card.raw())
            || !tx
                .state()
                .cards
                .get(*card)
                .is_some_and(|state| state.owner == target && state.zone == hand)
    }) {
        return Err(EffectError::Illegal(
            "Yangwu requires distinct current target hand cards".into(),
        ));
    }
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Yangwu owner hand missing".into()))?;
    run_effects(
        tx,
        vec![
            EffectRequest::RevealCards {
                cards: cards.clone(),
                to: Vec::new(),
            },
            EffectRequest::GiveCards {
                cards,
                to,
                from: hand,
            },
        ],
    )?;
    Ok(())
}

pub(crate) fn resolve_gzjinfa(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 || payload[0] != crate::effect::GZJINFA_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("bad Jinfa continuation".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    let option_count = payload[5] as usize;
    if payload.len() != 6 + option_count
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
    {
        return Err(EffectError::Illegal("invalid Jinfa choice context".into()));
    }
    match stage {
        1 => {
            if observer != target || selections.len() > 1 || (cancelled && !selections.is_empty()) {
                return Err(EffectError::Illegal(
                    "invalid Jinfa equipment choice".into(),
                ));
            }
            if cancelled || selections.is_empty() {
                let options = crate::query::gainable_target_cards(
                    tx.state(),
                    player,
                    target,
                    rule_api::TargetCardScope::HandEquip,
                );
                if options.is_empty() {
                    return Ok(());
                }
                let mut next = vec![
                    crate::effect::GZJINFA_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    options.len() as u32,
                ];
                next.extend(options.iter().map(|card| card.raw()));
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    next,
                )
                .map(|_| ());
            }
            let card = CardId::new(selections[0]);
            let from = response_card_zone(tx, target, card)?;
            if !payload[6..].contains(&card.raw())
                || !tx
                    .state()
                    .cards
                    .get(card)
                    .is_some_and(|state| state.owner == target && state.kind.is_equip())
            {
                return Err(EffectError::Illegal("invalid Jinfa equipment gift".into()));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("Jinfa owner hand missing".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::RevealCards {
                        cards: vec![card],
                        to: Vec::new(),
                    },
                    EffectRequest::GiveCards {
                        cards: vec![card],
                        to,
                        from,
                    },
                    EffectRequest::ResolveGzjinfa {
                        player,
                        target,
                        cost_card: CardId::NONE,
                        given_card: card,
                        stage: 2,
                        skill,
                    },
                ],
            )?;
            Ok(())
        }
        2 => {
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("invalid Jinfa gain choice".into()));
            }
            let card = CardId::new(selections[0]);
            let allowed = crate::query::gainable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            if !payload[6..].contains(&card.raw()) || !allowed.contains(&card) {
                return Err(EffectError::Illegal("invalid Jinfa gained card".into()));
            }
            let from = response_card_zone(tx, target, card)?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("Jinfa owner hand missing".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::RevealCards {
                        cards: vec![card],
                        to: Vec::new(),
                    },
                    EffectRequest::GiveCards {
                        cards: vec![card],
                        to,
                        from,
                    },
                ],
            )?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault("bad Jinfa choice stage".into())),
    }
}
