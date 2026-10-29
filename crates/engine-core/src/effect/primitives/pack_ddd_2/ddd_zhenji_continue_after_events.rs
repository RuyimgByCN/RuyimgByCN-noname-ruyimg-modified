//! Effect primitives for `resolve_ddd_jijian_kind_choice` and related helpers.

use super::*;

pub(crate) fn ddd_zhenji_continue_after_events(
    tx: &mut Transaction,
    events: Vec<DomainEvent>,
    next: EffectRequest,
) -> Result<Vec<DomainEvent>, EffectError> {
    if events.is_empty() {
        run_effects(tx, vec![next])?;
    } else {
        tx.state_mut().resolution.deferred_tail_effects.push(next);
    }
    Ok(events)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_ddd_zhenji(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    action: u8,
    stage: u8,
    mut piles: Vec<u8>,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage == 0 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => {
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            let size = hand.len().div_ceil(3);
            let options: Vec<u32> = (0..3)
                .filter(|index| index * size < hand.len())
                .map(|index| index as u32)
                .collect();
            let mut payload = vec![
                RESOLVE_DDD_ZHENJI_CHOICE,
                0,
                player.raw(),
                skill.raw(),
                hand.len() as u32,
            ];
            payload.extend(hand.iter().map(|card| card.raw()));
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
        (0, 1) => {
            let selected = piles
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("淼形未选择起始手牌组".into()))?
                as usize;
            if tx.state().hand_cards(player) != cards || cards.is_empty() {
                return Err(EffectError::Illegal("淼形起始手牌已变化".into()));
            }
            let size = cards.len().div_ceil(3);
            let mut groups = Vec::new();
            for index in 0..3 {
                let start = index * size;
                if start >= cards.len() {
                    break;
                }
                groups.push(cards[start..(start + size).min(cards.len())].to_vec());
            }
            if selected >= groups.len() {
                return Err(EffectError::Illegal("淼形起始手牌组越界".into()));
            }
            groups.remove(selected);
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("淼形没有手牌区".into()))?;
            let mut events = Vec::new();
            for (index, group) in groups.into_iter().enumerate() {
                if group.is_empty() {
                    continue;
                }
                let pile =
                    ensure_skill_pile_zone(tx, player, ddd_zhenji_pile_def(index as u8 + 1)?)?;
                events.extend(ddd_zhenji_move_to_pile(tx, group, pile, hand)?);
            }
            Ok(events)
        }
        (1, 0) => {
            let hand_count = tx.state().hand_count(player) as usize;
            let options: Vec<u32> = [1u8, 2]
                .into_iter()
                .filter(|index| {
                    ddd_zhenji_pile_def(*index).is_ok_and(|pile| {
                        tx.state().skill_pile_cards(player, pile).len() != hand_count
                    })
                })
                .map(u32::from)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let forced = options.len() == 2;
            open_choice(
                tx,
                player,
                1,
                options.len() as u8,
                !forced,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_ZHENJI_CHOICE, 1, player.raw(), skill.raw()],
            )
        }
        (1, 1) => {
            while let Some(index) = piles.first().copied() {
                let pile_def = ddd_zhenji_pile_def(index)?;
                let pile_cards = tx.state().skill_pile_cards(player, pile_def);
                let hand_count = tx.state().hand_count(player) as usize;
                if pile_cards.len() == hand_count {
                    piles.remove(0);
                    continue;
                }
                if pile_cards.len() > hand_count {
                    let count = pile_cards.len() - hand_count;
                    let mut payload = vec![
                        RESOLVE_DDD_ZHENJI_CHOICE,
                        2,
                        player.raw(),
                        skill.raw(),
                        piles.len() as u32,
                    ];
                    payload.extend(piles.iter().map(|index| u32::from(*index)));
                    return open_choice(
                        tx,
                        player,
                        count as u8,
                        count as u8,
                        false,
                        pile_cards.into_iter().map(CardId::raw).collect(),
                        ChoiceKind::SelectCards,
                        ContinuationKind::Custom,
                        payload,
                    );
                }
                let count = hand_count - pile_cards.len();
                let draw = tx
                    .state()
                    .zones
                    .find(ZoneKind::DrawPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("淼形没有牌堆".into()))?;
                let pile = ensure_skill_pile_zone(tx, player, pile_def)?;
                let mut events = Vec::new();
                for _ in 0..count {
                    if tx
                        .state()
                        .zones
                        .get(draw)
                        .is_none_or(|zone| zone.cards.is_empty())
                    {
                        reshuffle_discard_into_draw(tx)?;
                    }
                    let Some(card) = tx
                        .state()
                        .zones
                        .get(draw)
                        .and_then(|zone| zone.cards.last())
                        .copied()
                    else {
                        break;
                    };
                    events.extend(ddd_zhenji_move_to_pile(tx, vec![card], pile, draw)?);
                }
                piles.remove(0);
                if piles.is_empty() {
                    return Ok(events);
                }
                return ddd_zhenji_continue_after_events(
                    tx,
                    events,
                    EffectRequest::ResolveDddZhenji {
                        player,
                        target: PlayerId::NONE,
                        action: 1,
                        stage: 1,
                        piles,
                        cards: Vec::new(),
                        skill,
                    },
                );
            }
            Ok(Vec::new())
        }
        (1, 2) => {
            let index = piles
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("淼形没有待缩减的水相".into()))?;
            let pile_def = ddd_zhenji_pile_def(index)?;
            let pile = ensure_skill_pile_zone(tx, player, pile_def)?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("淼形没有弃牌堆".into()))?;
            let current: std::collections::BTreeSet<_> = tx
                .state()
                .skill_pile_cards(player, pile_def)
                .into_iter()
                .collect();
            let selected: std::collections::BTreeSet<_> = cards.iter().copied().collect();
            let excess = current
                .len()
                .saturating_sub(tx.state().hand_count(player) as usize);
            if cards.is_empty()
                || cards.len() != excess
                || selected.len() != cards.len()
                || !selected.is_subset(&current)
            {
                return Err(EffectError::Illegal("淼形移除的水相已失效".into()));
            }
            let events = move_cards(tx, cards, discard, Some(pile))?;
            piles.remove(0);
            if piles.is_empty() {
                return Ok(events);
            }
            ddd_zhenji_continue_after_events(
                tx,
                events,
                EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::NONE,
                    action: 1,
                    stage: 1,
                    piles,
                    cards: Vec::new(),
                    skill,
                },
            )
        }
        (2, 0) => {
            let hand_nonempty = tx.state().hand_count(player) > 0;
            let options: Vec<u32> = [1u8, 2]
                .into_iter()
                .filter(|index| {
                    hand_nonempty
                        || ddd_zhenji_pile_def(*index)
                            .is_ok_and(|pile| !tx.state().skill_pile_cards(player, pile).is_empty())
                })
                .map(u32::from)
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
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_ZHENJI_CHOICE, 3, player.raw(), skill.raw()],
            )
        }
        (2, 1) => {
            let index = piles
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("浮世未选择水相".into()))?;
            let events = ddd_zhenji_exchange_all(tx, player, player, index)?;
            ddd_zhenji_continue_after_events(
                tx,
                events,
                EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::NONE,
                    action: 2,
                    stage: 2,
                    piles: Vec::new(),
                    cards: Vec::new(),
                    skill,
                },
            )
        }
        (2, 2) => {
            let has_water = !tx
                .state()
                .skill_pile_cards(player, DDD_MIAOXING_PILE_1)
                .is_empty()
                || !tx
                    .state()
                    .skill_pile_cards(player, DDD_MIAOXING_PILE_2)
                    .is_empty();
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|participant| {
                    ddd_baosanniang_is_male(tx.state(), *participant)
                        && (has_water || tx.state().hand_count(*participant) > 0)
                })
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_ZHENJI_CHOICE, 4, player.raw(), skill.raw()],
            )
        }
        (2, 3) => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
                || !ddd_baosanniang_is_male(tx.state(), target)
            {
                return Err(EffectError::Illegal("浮世男性目标已失效".into()));
            }
            let target_has_hand = tx.state().hand_count(target) > 0;
            let options: Vec<u32> = [1u8, 2]
                .into_iter()
                .filter(|index| {
                    target_has_hand
                        || ddd_zhenji_pile_def(*index)
                            .is_ok_and(|pile| !tx.state().skill_pile_cards(player, pile).is_empty())
                })
                .map(u32::from)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                true,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_ZHENJI_CHOICE,
                    5,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        (2, 4) => {
            let index = piles
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("浮世男性角色未选择水相".into()))?;
            ddd_zhenji_exchange_all(tx, player, target, index)
        }
        _ => Err(EffectError::EngineFault(
            "unknown 3D精选甄宓 continuation".into(),
        )),
    }
}
