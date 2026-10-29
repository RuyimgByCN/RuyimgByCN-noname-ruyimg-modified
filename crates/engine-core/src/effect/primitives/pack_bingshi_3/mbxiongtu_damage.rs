//! Effect primitives for `resolve_mbrenxing` and related helpers.

use super::*;

pub(crate) fn mbxiongtu_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    set_mbxiongtu_flags(tx, skill, mbxiongtu_flags(tx, skill) | 0b100);
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveMbxiongtu {
            player,
            target,
            stage: 4,
            card: CardId::NONE,
            cards: Vec::new(),
            skill,
        });
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: player,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
}

pub(crate) fn resolve_mbxiongtu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    card: CardId,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let uses = tx.state().skills.get(skill).unwrap().play_phase_use_count;
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || player == target
                || uses == 0
                || uses > 2
                || (uses == 2 && mbxiongtu_flags(tx, skill) & 0b010 == 0)
            {
                return Err(EffectError::Illegal("凶图 is not available".into()));
            }
            let options =
                crate::command::play::target_zone_card_tokens(tx, target, true, false, false);
            if options.is_empty() {
                return Err(EffectError::Illegal("凶图 target has no hand card".into()));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    MBXIONGTU_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            if !tx
                .state()
                .cards
                .get(card)
                .is_some_and(|state| state.owner == target && Some(state.zone) == hand)
            {
                return Err(EffectError::Illegal("凶图 revealed card changed".into()));
            }
            let count = 4usize.saturating_sub(
                (tx.state().turn.flag(turn_flag::DISCARD_PILE_SUIT_MASK) as u32 & 0b1111)
                    .count_ones() as usize,
            );
            let mut events = vec![DomainEvent::CardsRevealed {
                cards: vec![card],
                to: Vec::new(),
            }];
            if count == 0 {
                events.extend(mbxiongtu_damage(tx, player, target, skill)?);
                return Ok(events);
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.len() < count {
                events.extend(resolve_mbxiongtu(
                    tx,
                    player,
                    target,
                    3,
                    card,
                    Vec::new(),
                    skill,
                )?);
                return Ok(events);
            }
            events.extend(open_choice(
                tx,
                player,
                count as u8,
                count as u8,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    MBXIONGTU_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )?);
            Ok(events)
        }
        2 => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if cards.is_empty()
                || cards.iter().any(|selected| !options.contains(selected))
                || cards.len()
                    != 4usize.saturating_sub(
                        (tx.state().turn.flag(turn_flag::DISCARD_PILE_SUIT_MASK) as u32 & 0b1111)
                            .count_ones() as usize,
                    )
            {
                return Err(EffectError::Illegal("invalid 凶图 discard".into()));
            }
            let mut events = run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }])?;
            events.extend(mbxiongtu_damage(tx, player, target, skill)?);
            Ok(events)
        }
        3 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            let valid = tx
                .state()
                .cards
                .get(card)
                .is_some_and(|state| state.owner == target && Some(state.zone) == hand);
            let mut events = Vec::new();
            if valid {
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .unwrap();
                events.extend(move_cards_with_reason(
                    tx,
                    vec![card],
                    discard,
                    hand,
                    CardMoveReason::Discard,
                )?);
            }
            set_mbxiongtu_flags(tx, skill, mbxiongtu_flags(tx, skill) | 0b001);
            Ok(events)
        }
        4 => {
            set_mbxiongtu_flags(tx, skill, (mbxiongtu_flags(tx, skill) & !0b100) | 0b001);
            Ok(Vec::new())
        }
        5 => {
            let flags = mbxiongtu_flags(tx, skill);
            if flags & 0b001 == 0 || flags & 0b100 != 0 {
                return Ok(Vec::new());
            }
            set_mbxiongtu_flags(tx, skill, (flags & !0b001) | 0b010);
            run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }])
        }
        6 => {
            set_mbxiongtu_flags(tx, skill, mbxiongtu_flags(tx, skill) & !0b010);
            Ok(Vec::new())
        }
        7 => {
            set_mbxiongtu_flags(tx, skill, 0);
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad 凶图 stage".into())),
    }
}

pub(crate) fn resolve_mbganggeng(
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
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if player == target
                || cards.len() < 2
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || cards.iter().any(|card| {
                    !tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| state.owner == player && Some(state.zone) == hand)
                })
            {
                return Err(EffectError::Illegal("invalid 刚鲠 selection".into()));
            }
            let to = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
            let mut blob = b"GGG1".to_vec();
            blob.extend_from_slice(&target.raw().to_le_bytes());
            tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards,
                    to,
                    from: hand.unwrap(),
                }],
            )
        }
        1 => {
            let remembered = tx.state().skills.get(skill).and_then(|instance| {
                (instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"GGG1").then(
                    || {
                        PlayerId::new(u32::from_le_bytes(
                            instance.state_blob[4..8].try_into().unwrap(),
                        ))
                    },
                )
            });
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .state_blob
                .clear();
            let Some(target) = remembered else {
                return Ok(Vec::new());
            };
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let target_hand = tx.state().hand_count(target);
            let is_max = tx
                .state()
                .players
                .iter_live()
                .filter(|participant| participant.alive)
                .all(|participant| tx.state().hand_count(participant.id) <= target_hand);
            if is_max {
                run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }])
            } else {
                run_effects(
                    tx,
                    vec![EffectRequest::ChooseDiscardTargetCard {
                        player,
                        target,
                        scope: TargetCardScope::HandEquipJudge,
                    }],
                )
            }
        }
        _ => Err(EffectError::ContentFault("bad 刚鲠 stage".into())),
    }
}

pub(crate) fn mbsijian_both_count(tx: &Transaction, skill: engine_api::SkillId) -> u32 {
    tx.state()
        .skills
        .get(skill)
        .and_then(|instance| {
            (instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"SJN1")
                .then(|| u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap()))
        })
        .unwrap_or(0)
}

pub(crate) fn set_mbsijian_both_count(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    count: u32,
) {
    let mut blob = b"SJN1".to_vec();
    blob.extend_from_slice(&count.to_le_bytes());
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
}

pub(crate) fn resolve_mbsijian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    choice: u8,
    helper_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let current = tx.state().turn.current;
    let current_alive = tx
        .state()
        .players
        .get(current)
        .is_some_and(|participant| participant.alive);
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| *target != player)
        .collect();
    match stage {
        0 => {
            let mut options = Vec::new();
            if !others.is_empty() {
                options.push(1);
            }
            if current_alive {
                options.push(2);
            }
            if (!others.is_empty() || current_alive)
                && !tx
                    .state()
                    .players
                    .iter_live()
                    .any(|participant| participant.dying)
            {
                options.push(3);
            }
            if options.is_empty() {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.turn_fire_count = instance.turn_fire_count.saturating_sub(1);
                }
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    MBSIJIAN_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    helper_def.raw(),
                    skill.raw(),
                ],
            )
        }
        1 if matches!(choice, 1 | 3) && !others.is_empty() => open_choice(
            tx,
            player,
            1,
            1,
            false,
            others.into_iter().map(PlayerId::raw).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            vec![
                MBSIJIAN_CUSTOM_CHOICE,
                1,
                player.raw(),
                choice.into(),
                helper_def.raw(),
                skill.raw(),
            ],
        ),
        1 => resolve_mbsijian(tx, player, PlayerId::NONE, 2, choice, helper_def, skill),
        2 => {
            let mut effects = Vec::new();
            if choice != 2
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive && target != player)
            {
                effects.push(EffectRequest::AddSkill {
                    player: target,
                    skill_def: helper_def,
                });
            }
            if choice != 1 && current_alive {
                effects.push(EffectRequest::DrawCards {
                    player: current,
                    count: 2,
                });
            }
            if choice == 3 {
                let count = mbsijian_both_count(tx, skill);
                set_mbsijian_both_count(tx, skill, count.saturating_add(1));
                if count > 0 {
                    effects.push(EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: player,
                        amount: count,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    });
                }
            }
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("bad 死谏 stage".into())),
    }
}

pub(crate) fn mbjianji_choice_options(
    tx: &Transaction,
    participant: PlayerId,
    shared: CardId,
) -> Vec<u32> {
    let mut options: Vec<_> = tx
        .state()
        .hand_cards(participant)
        .into_iter()
        .map(CardId::raw)
        .collect();
    options.push(shared.raw());
    options.sort_unstable();
    options.dedup();
    options
}

pub(crate) fn resolve_mbjianji(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    target: PlayerId,
    stage: u8,
    card: CardId,
    _cards: Vec<CardId>,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if source == target
                || !tx
                    .state()
                    .players
                    .get(source)
                    .is_some_and(|role| role.alive)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|role| role.alive)
                || !tx
                    .state()
                    .cards
                    .get(card)
                    .is_some_and(|state| state.owner == player && Some(state.zone) == hand)
            {
                return Err(EffectError::Illegal("invalid 间计 selection".into()));
            }
            open_choice(
                tx,
                source,
                1,
                1,
                false,
                mbjianji_choice_options(tx, source, card),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    MBJIANJI_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    card.raw(),
                    source.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        3 => {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveMbjianji {
                    player,
                    source: PlayerId::NONE,
                    target: PlayerId::NONE,
                    stage: 4,
                    card,
                    cards: Vec::new(),
                    targets,
                    skill,
                });
            if source == PlayerId::NONE || target == PlayerId::NONE {
                Ok(Vec::new())
            } else {
                try_use_virtual_sha_on_without_distance(tx, source, target, true, 0)
            }
        }
        4 => {
            let targets: Vec<_> = targets
                .into_iter()
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|role| role.alive)
                })
                .collect();
            if targets.is_empty() {
                Ok(Vec::new())
            } else {
                run_effects(
                    tx,
                    vec![EffectRequest::DealDamageEachInOrder {
                        source: player,
                        targets,
                        amount: 1,
                        skill,
                    }],
                )
            }
        }
        _ => Err(EffectError::ContentFault("bad 间计 stage".into())),
    }
}

pub(crate) fn commit_mbjianji_compare(
    tx: &mut Transaction,
    player: PlayerId,
    first: PlayerId,
    second: PlayerId,
    shared: CardId,
    first_card: CardId,
    second_card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || first == second {
        return Err(EffectError::Illegal("间计 context changed".into()));
    }
    let valid = |participant: PlayerId, selected: CardId| {
        selected == shared
            || tx.state().cards.get(selected).is_some_and(|card| {
                card.owner == participant
                    && tx.state().zones.get(card.zone).is_some_and(|zone| {
                        zone.kind == ZoneKind::Hand && zone.owner == participant
                    })
            })
    };
    let shared_state = tx.state().cards.get(shared).filter(|card| {
        card.owner == player
            && tx
                .state()
                .zones
                .get(card.zone)
                .is_some_and(|zone| zone.kind == ZoneKind::Hand && zone.owner == player)
    });
    if !valid(first, first_card) || !valid(second, second_card) || shared_state.is_none() {
        return Err(EffectError::Illegal("invalid 间计 comparison card".into()));
    }
    let shared_is_sha = shared_state.unwrap().kind == CardKind::Sha;
    let first_rank = tx.state().cards.get(first_card).unwrap().rank;
    let second_rank = tx.state().cards.get(second_card).unwrap().rank;
    let (winner, loser) = match first_rank.cmp(&second_rank) {
        std::cmp::Ordering::Greater => (first, second),
        std::cmp::Ordering::Less => (second, first),
        std::cmp::Ordering::Equal => (PlayerId::NONE, PlayerId::NONE),
    };
    let mut damage_targets = Vec::new();
    if shared_is_sha {
        if first_card == shared {
            damage_targets.push(first);
        }
        if second_card == shared {
            damage_targets.push(second);
        }
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("shared discard pile missing".into()))?;
    let mut effects = vec![EffectRequest::RevealCards {
        cards: if first_card == second_card {
            vec![first_card]
        } else {
            vec![first_card, second_card]
        },
        to: Vec::new(),
    }];
    let mut moved = Vec::new();
    for selected in [first_card, second_card] {
        if moved.contains(&selected) {
            continue;
        }
        moved.push(selected);
        let from = tx.state().cards.get(selected).unwrap().zone;
        effects.push(EffectRequest::MoveCards {
            cards: vec![selected],
            to: discard,
            from: Some(from),
        });
    }
    effects.extend([
        EffectRequest::EmitHandCompareFinished {
            player: first,
            target: second,
            player_card: first_card,
            target_card: second_card,
            winner,
        },
        EffectRequest::ResolveMbjianji {
            player,
            source: winner,
            target: loser,
            stage: 3,
            card: shared,
            cards: Vec::new(),
            targets: damage_targets,
            skill,
        },
    ]);
    run_effects(tx, effects)
}

pub(crate) fn mbfeijing_seat_paths(
    state: &crate::state::GameState,
    player: PlayerId,
    target: PlayerId,
) -> Option<(Vec<PlayerId>, Vec<PlayerId>)> {
    let order = state.players.alive_by_seat();
    let start = order.iter().position(|role| *role == player)?;
    let end = order.iter().position(|role| *role == target)?;
    if start == end || order.len() < 3 {
        return None;
    }
    let walk = |step: isize| {
        let mut path = Vec::new();
        let mut index = start;
        loop {
            index = (index as isize + step).rem_euclid(order.len() as isize) as usize;
            if index == end {
                break;
            }
            path.push(order[index]);
        }
        path
    };
    Some((walk(1), walk(-1)))
}
