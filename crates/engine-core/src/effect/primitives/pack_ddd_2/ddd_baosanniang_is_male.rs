//! Effect primitives for `resolve_ddd_xianglang` and related helpers.

use super::*;

pub(crate) fn ddd_baosanniang_is_male(state: &crate::state::GameState, player: PlayerId) -> bool {
    state
        .players
        .get(player)
        .and_then(|participant| state.content.character(participant.character_def))
        .is_some_and(|character| matches!(character.sex.as_str(), "male" | "double"))
}

pub(crate) fn ddd_zhilian_participants(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Option<Vec<PlayerId>> {
    let mut participants: Vec<_> = state
        .players
        .iter_live()
        .filter(|actor| {
            state
                .turn
                .flag(turn_flag::card_action_count_by_player(actor.id))
                > 0
        })
        .map(|actor| (actor.seat.raw(), actor.id, actor.alive))
        .collect();
    let alive = state.players.alive_by_seat();
    participants.sort_by_key(|(seat, actor, alive_actor)| {
        if *alive_actor {
            u32::from(crate::trigger::seat_rel_to_current(
                &alive,
                state.turn.current,
                *actor,
            ))
        } else {
            u32::from(*seat)
        }
    });
    let all_alive = participants.iter().all(|(_, _, alive)| *alive);
    let participants: Vec<_> = participants
        .into_iter()
        .map(|(_, actor, _)| actor)
        .collect();
    (participants.len() == 2
        && all_alive
        && participants.contains(&player)
        && participants
            .iter()
            .any(|actor| *actor != player && ddd_baosanniang_is_male(state, *actor)))
    .then_some(participants)
}

pub(crate) fn ddd_zhilian_gain_sha(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let cards: Vec<_> = tx
        .state()
        .turn
        .entered_discard_cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|state| state.zone == discard && state.kind.is_sha())
        })
        .collect();
    if cards.is_empty() {
        Ok(Vec::new())
    } else {
        gain_event_cards(tx, player, &cards)
    }
}

pub(crate) fn ddd_jijian_name_kind(kind: CardKind) -> CardKind {
    if kind.is_sha() {
        CardKind::Sha
    } else {
        kind
    }
}

pub(crate) fn ddd_jijian_reveal_kind(
    state: &crate::state::GameState,
    holder: PlayerId,
    card: CardId,
) -> Option<CardKind> {
    let card = state.cards.get(card)?;
    let kind = crate::query::effective_card_kind(state, holder, card);
    (kind.is_sha() || (kind.is_trick() && !kind.is_delayed_trick())).then_some(kind)
}

pub(crate) fn ddd_jijian_virtual_options(
    tx: &mut Transaction,
    current: PlayerId,
    cards: &[CardId],
    used_kind: u8,
) -> Result<Vec<u32>, EffectError> {
    let holder = cards
        .iter()
        .find_map(|card| tx.state().cards.get(*card).map(|card| card.owner))
        .filter(|holder| holder.is_some())
        .unwrap_or(current);
    let used = CardKind::from_tag(used_kind).map(ddd_jijian_name_kind);
    let mut kinds = Vec::new();
    for card in cards {
        let Some(kind) = ddd_jijian_reveal_kind(tx.state(), holder, *card) else {
            continue;
        };
        if used == Some(ddd_jijian_name_kind(kind)) || kinds.contains(&kind) {
            continue;
        }
        let virtual_card = allocate_virtual_card_kind(tx, kind)?;
        let legal = crate::command::play::skill_card_target_spec(tx, current, virtual_card).is_ok();
        remove_unused_virtual_card(tx, virtual_card)?;
        if legal {
            kinds.push(kind);
        }
    }
    Ok(kinds
        .into_iter()
        .map(|kind| u32::from(kind.tag()))
        .collect())
}

pub(crate) fn ddd_jijian_schedule_next(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
    used_kind: CardKind,
    skill: engine_api::SkillId,
) {
    if !targets.is_empty() {
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::ResolveDddBaosanniang {
                player,
                target: targets[0],
                card: CardId::NONE,
                action: 12,
                cards,
                targets,
                used_kind: ddd_jijian_name_kind(used_kind).tag(),
                skill,
            },
        );
    }
}

pub(crate) fn ddd_jijian_play_virtual(
    tx: &mut Transaction,
    player: PlayerId,
    current: PlayerId,
    card: CardId,
    kind: CardKind,
    targets_for_card: &[PlayerId],
    cards: Vec<CardId>,
    remaining: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    ddd_jijian_schedule_next(tx, player, cards, remaining, kind, skill);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_CARD, card.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, i32::from(kind.tag()));
    let result = crate::command::play::handle_skill_card(tx, current, card, targets_for_card);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_CARD, 0);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_SKILL, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    result
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_ddd_baosanniang(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    mut targets: Vec<PlayerId>,
    used_kind: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if matches!(action, 0 | 1) && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let Some(participants) = ddd_zhilian_participants(tx.state(), player) else {
                return Ok(Vec::new());
            };
            let can_discard = participants.iter().any(|participant| {
                !crate::query::discardable_hand_equip_cards(tx.state(), *participant).is_empty()
            });
            let mut options = vec![0];
            if can_discard {
                options.push(1);
            }
            let mut payload = vec![RESOLVE_DDD_BAOSANNIANG_CHOICE, 0, player.raw(), skill.raw()];
            payload.extend(participants.iter().map(|participant| participant.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                payload,
            )
        }
        1 => {
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || !ddd_baosanniang_is_male(tx.state(), target)
            {
                return Ok(Vec::new());
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("赍剑发动者手牌区不存在".into()))?;
            let selected = cards.first().copied().unwrap_or(card);
            if !tx.state().cards.get(selected).is_some_and(|state| {
                state.owner == player
                    && state.zone == hand
                    && crate::query::effective_card_kind(tx.state(), player, state).is_sha()
            }) {
                return Err(EffectError::Illegal("赍剑须交出一张当前手牌【杀】".into()));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("赍剑目标手牌区不存在".into()))?;
            let mut events = vec![DomainEvent::CardsRevealed {
                cards: vec![selected],
                to: Vec::new(),
            }];
            events.extend(give_cards_as(tx, player, vec![selected], to, hand)?);
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDddBaosanniang {
                    player,
                    target,
                    card: CardId::NONE,
                    action: 10,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    used_kind: 0,
                    skill,
                },
            );
            Ok(events)
        }
        2 => ddd_zhilian_gain_sha(tx, player),
        3 => {
            while let Some(current) = targets.first().copied() {
                let options = crate::query::discardable_hand_equip_cards(tx.state(), current);
                if options.is_empty() {
                    targets.remove(0);
                    continue;
                }
                let mut payload = vec![
                    RESOLVE_DDD_BAOSANNIANG_CHOICE,
                    1,
                    player.raw(),
                    current.raw(),
                    skill.raw(),
                    targets.len() as u32,
                ];
                payload.extend(targets.iter().map(|target| target.raw()));
                payload.push(options.len() as u32);
                payload.extend(options.iter().map(|card| card.raw()));
                return open_choice(
                    tx,
                    current,
                    1,
                    1,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    payload,
                );
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddBaosanniang {
                    player,
                    target: PlayerId::NONE,
                    card: CardId::NONE,
                    action: 2,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    used_kind: 0,
                    skill,
                }],
            )
        }
        10 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("赍剑目标手牌区不存在".into()))?;
            let options: Vec<_> = tx
                .state()
                .hand_cards(target)
                .into_iter()
                .filter(|card| ddd_jijian_reveal_kind(tx.state(), target, *card).is_some())
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_DDD_BAOSANNIANG_CHOICE,
                2,
                player.raw(),
                target.raw(),
                skill.raw(),
                hand.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                1,
                options.len().min(2) as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        11 => {
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let alive = tx.state().players.alive_by_seat();
            let anchor = if tx.state().turn.current.is_some() {
                tx.state().turn.current
            } else {
                player
            };
            targets = vec![player, target];
            targets.sort_by_key(|participant| {
                crate::trigger::seat_rel_to_current(&alive, anchor, *participant)
            });
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddBaosanniang {
                    player,
                    target: targets[0],
                    card: CardId::NONE,
                    action: 12,
                    cards,
                    targets,
                    used_kind: 0,
                    skill,
                }],
            )
        }
        12 => {
            while let Some(current) = targets.first().copied() {
                if tx
                    .state()
                    .players
                    .get(current)
                    .is_some_and(|participant| participant.alive)
                {
                    let options = ddd_jijian_virtual_options(tx, current, &cards, used_kind)?;
                    if options.is_empty() {
                        return Ok(Vec::new());
                    }
                    let mut payload = vec![
                        RESOLVE_DDD_BAOSANNIANG_CHOICE,
                        3,
                        player.raw(),
                        current.raw(),
                        skill.raw(),
                        used_kind.into(),
                        cards.len() as u32,
                    ];
                    payload.extend(cards.iter().map(|card| card.raw()));
                    payload.push(targets.len() as u32);
                    payload.extend(targets.iter().map(|target| target.raw()));
                    payload.push(options.len() as u32);
                    payload.extend(options.iter().copied());
                    return open_choice(
                        tx,
                        current,
                        1,
                        1,
                        false,
                        options,
                        ChoiceKind::Custom,
                        ContinuationKind::Custom,
                        payload,
                    );
                }
                targets.remove(0);
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown 3D精选鲍三娘 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_baosanniang_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload.first() != Some(&RESOLVE_DDD_BAOSANNIANG_CHOICE) {
        return Err(EffectError::EngineFault(
            "truncated 3D精选鲍三娘 choice".into(),
        ));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    match stage {
        0 => {
            if observer != player {
                return Err(EffectError::Illegal("枝连选择者已变化".into()));
            }
            if cancelled && selections.is_empty() {
                return Ok(());
            }
            if cancelled || selections.len() != 1 || payload.len() != 6 {
                return Err(EffectError::Illegal("枝连须选择摸牌、弃牌或取消".into()));
            }
            let skill = engine_api::SkillId::new(payload[3]);
            let participants: Vec<_> = payload[4..6].iter().copied().map(PlayerId::new).collect();
            if ddd_zhilian_participants(tx.state(), player).as_deref()
                != Some(participants.as_slice())
            {
                return Err(EffectError::Illegal("枝连参与角色已变化".into()));
            }
            match selections[0] {
                0 => {
                    let mut effects: Vec<_> = participants
                        .iter()
                        .copied()
                        .map(|participant| EffectRequest::DrawCards {
                            player: participant,
                            count: 1,
                        })
                        .collect();
                    effects.push(EffectRequest::ResolveDddBaosanniang {
                        player,
                        target: PlayerId::NONE,
                        card: CardId::NONE,
                        action: 2,
                        cards: Vec::new(),
                        targets: Vec::new(),
                        used_kind: 0,
                        skill,
                    });
                    run_effects(tx, effects).map(|_| ())
                }
                1 if participants.iter().any(|participant| {
                    !crate::query::discardable_hand_equip_cards(tx.state(), *participant).is_empty()
                }) =>
                {
                    run_effects(
                        tx,
                        vec![EffectRequest::ResolveDddBaosanniang {
                            player,
                            target: participants[0],
                            card: CardId::NONE,
                            action: 3,
                            cards: Vec::new(),
                            targets: participants,
                            used_kind: 0,
                            skill,
                        }],
                    )
                    .map(|_| ())
                }
                _ => Err(EffectError::Illegal("枝连分支已失效".into())),
            }
        }
        1 => {
            if cancelled || selections.len() != 1 || payload.len() < 8 {
                return Err(EffectError::Illegal("枝连弃牌选择无效".into()));
            }
            let current = PlayerId::new(payload[3]);
            if observer != current {
                return Err(EffectError::Illegal("枝连弃牌角色已变化".into()));
            }
            let skill = engine_api::SkillId::new(payload[4]);
            let target_count = payload[5] as usize;
            let options_index = 6 + target_count;
            if payload.len() <= options_index {
                return Err(EffectError::EngineFault(
                    "truncated 枝连 discard queue".into(),
                ));
            }
            let option_count = payload[options_index] as usize;
            if payload.len() != options_index + 1 + option_count
                || !payload[options_index + 1..].contains(&selections[0])
            {
                return Err(EffectError::Illegal("枝连弃牌候选已变化".into()));
            }
            let card = CardId::new(selections[0]);
            if !crate::query::discardable_hand_equip_cards(tx.state(), current).contains(&card) {
                return Err(EffectError::Illegal("枝连所弃牌已失效".into()));
            }
            let remaining: Vec<_> = payload[6..options_index]
                .iter()
                .copied()
                .map(PlayerId::new)
                .skip(1)
                .collect();
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player: current,
                        cards: vec![card],
                    },
                    EffectRequest::ResolveDddBaosanniang {
                        player,
                        target: remaining.first().copied().unwrap_or(PlayerId::NONE),
                        card: CardId::NONE,
                        action: 3,
                        cards: Vec::new(),
                        targets: remaining,
                        used_kind: 0,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        2 => {
            if cancelled || selections.is_empty() || selections.len() > 2 || payload.len() < 8 {
                return Err(EffectError::Illegal("赍剑须展示一至两张牌".into()));
            }
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let hand = ZoneId::new(payload[5]);
            let option_count = payload[6] as usize;
            if observer != target
                || payload.len() != 7 + option_count
                || selections.iter().any(|card| !payload[7..].contains(card))
            {
                return Err(EffectError::Illegal("赍剑展示候选已变化".into()));
            }
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            let mut distinct = std::collections::BTreeSet::new();
            if cards.iter().any(|card| {
                !distinct.insert(*card)
                    || !tx.state().cards.get(*card).is_some_and(|state| {
                        state.owner == target
                            && state.zone == hand
                            && ddd_jijian_reveal_kind(tx.state(), target, *card).is_some()
                    })
            }) {
                return Err(EffectError::Illegal("赍剑展示牌已离开目标手牌".into()));
            }
            let colors: std::collections::BTreeSet<_> = cards
                .iter()
                .map(|card| crate::query::effective_card_color_mask(tx.state(), target, *card))
                .collect();
            if colors.len() != 1 || colors.contains(&0) {
                return Err(EffectError::Illegal("赍剑展示牌必须颜色相同".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::RevealCards {
                        cards: cards.clone(),
                        to: Vec::new(),
                    },
                    EffectRequest::ResolveDddBaosanniang {
                        player,
                        target,
                        card: CardId::NONE,
                        action: 11,
                        cards,
                        targets: Vec::new(),
                        used_kind: 0,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        3 => resolve_ddd_jijian_kind_choice(tx, observer, payload, selections, cancelled),
        4 => resolve_ddd_jijian_target_choice(tx, observer, payload, selections, cancelled, false),
        5 => resolve_ddd_jijian_target_choice(tx, observer, payload, selections, cancelled, true),
        _ => Err(EffectError::EngineFault(
            "unknown 3D精选鲍三娘 choice stage".into(),
        )),
    }
}
