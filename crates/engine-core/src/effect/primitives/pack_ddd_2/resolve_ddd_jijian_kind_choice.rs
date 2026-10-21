//! Effect primitives for `resolve_ddd_jijian_kind_choice` and related helpers.

use super::*;

pub(crate) fn resolve_ddd_jijian_kind_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled || selections.len() != 1 || payload.len() < 10 {
        return Err(EffectError::Illegal("赍剑须选择一个可使用牌名".into()));
    }
    let player = PlayerId::new(payload[2]);
    let current = PlayerId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    let used_kind = u8::try_from(payload[5]).unwrap_or(0);
    let card_count = payload[6] as usize;
    let cards_end = 7 + card_count;
    if observer != current || payload.len() <= cards_end {
        return Err(EffectError::Illegal("赍剑用牌角色已变化".into()));
    }
    let cards: Vec<_> = payload[7..cards_end]
        .iter()
        .copied()
        .map(CardId::new)
        .collect();
    let target_count = payload[cards_end] as usize;
    let targets_end = cards_end + 1 + target_count;
    if payload.len() <= targets_end {
        return Err(EffectError::EngineFault("truncated 赍剑 user queue".into()));
    }
    let targets: Vec<_> = payload[cards_end + 1..targets_end]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    if targets.first() != Some(&current) {
        return Err(EffectError::Illegal("赍剑当前用牌角色已变化".into()));
    }
    let option_count = payload[targets_end] as usize;
    if payload.len() != targets_end + 1 + option_count
        || !payload[targets_end + 1..].contains(&selections[0])
        || !ddd_jijian_virtual_options(tx, current, &cards, used_kind)?.contains(&selections[0])
    {
        return Err(EffectError::Illegal("赍剑可用牌名已变化".into()));
    }
    let kind = u8::try_from(selections[0])
        .ok()
        .and_then(CardKind::from_tag)
        .ok_or_else(|| EffectError::Illegal("赍剑牌名无效".into()))?;
    let virtual_card = allocate_virtual_card_kind(tx, kind)?;
    let remaining: Vec<_> = targets.into_iter().skip(1).collect();
    match crate::command::play::skill_card_target_spec(tx, current, virtual_card)? {
        crate::command::play::SkillCardTargetSpec::Immediate(card_targets) => {
            ddd_jijian_play_virtual(
                tx,
                player,
                current,
                virtual_card,
                kind,
                &card_targets,
                cards,
                remaining,
                skill,
            )
        }
        crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
            let mut next = vec![
                RESOLVE_DDD_BAOSANNIANG_CHOICE,
                4,
                player.raw(),
                current.raw(),
                virtual_card.raw(),
                u32::from(kind.tag()),
                skill.raw(),
                cards.len() as u32,
            ];
            next.extend(cards.iter().map(|card| card.raw()));
            next.push(remaining.len() as u32);
            next.extend(remaining.iter().map(|target| target.raw()));
            next.push(u32::from(min));
            next.push(u32::from(max));
            next.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                current,
                min,
                max,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        crate::command::play::SkillCardTargetSpec::Pairs(pairs) => {
            let mut next = vec![
                RESOLVE_DDD_BAOSANNIANG_CHOICE,
                5,
                player.raw(),
                current.raw(),
                virtual_card.raw(),
                u32::from(kind.tag()),
                skill.raw(),
                cards.len() as u32,
            ];
            next.extend(cards.iter().map(|card| card.raw()));
            next.push(remaining.len() as u32);
            next.extend(remaining.iter().map(|target| target.raw()));
            next.push(pairs.len() as u32);
            for (first, second) in &pairs {
                next.push(first.raw());
                next.push(second.raw());
            }
            open_choice(
                tx,
                current,
                1,
                1,
                false,
                (0..pairs.len().min(u32::MAX as usize) as u32).collect(),
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
    }
}

pub(crate) fn resolve_ddd_jijian_target_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
    paired: bool,
) -> Result<(), EffectError> {
    if cancelled || payload.len() < 11 {
        return Err(EffectError::Illegal("赍剑虚拟牌目标选择无效".into()));
    }
    let player = PlayerId::new(payload[2]);
    let current = PlayerId::new(payload[3]);
    let card = CardId::new(payload[4]);
    let kind = u8::try_from(payload[5])
        .ok()
        .and_then(CardKind::from_tag)
        .ok_or_else(|| EffectError::EngineFault("bad 赍剑 virtual kind".into()))?;
    let skill = engine_api::SkillId::new(payload[6]);
    let card_count = payload[7] as usize;
    let cards_end = 8 + card_count;
    if observer != current || payload.len() <= cards_end {
        return Err(EffectError::Illegal("赍剑虚拟牌使用者已变化".into()));
    }
    let cards: Vec<_> = payload[8..cards_end]
        .iter()
        .copied()
        .map(CardId::new)
        .collect();
    let remaining_count = payload[cards_end] as usize;
    let remaining_end = cards_end + 1 + remaining_count;
    if payload.len() <= remaining_end {
        return Err(EffectError::EngineFault(
            "truncated 赍剑 remaining users".into(),
        ));
    }
    let remaining: Vec<_> = payload[cards_end + 1..remaining_end]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    if paired {
        let pair_count = payload[remaining_end] as usize;
        if selections.len() != 1
            || payload.len() != remaining_end + 1 + pair_count.saturating_mul(2)
            || selections[0] as usize >= pair_count
        {
            return Err(EffectError::Illegal("赍剑借刀目标组合无效".into()));
        }
        let index = remaining_end + 1 + selections[0] as usize * 2;
        let selected = (
            PlayerId::new(payload[index]),
            PlayerId::new(payload[index + 1]),
        );
        let crate::command::play::SkillCardTargetSpec::Pairs(current_pairs) =
            crate::command::play::skill_card_target_spec(tx, current, card)?
        else {
            return Err(EffectError::Illegal("赍剑借刀目标形态已变化".into()));
        };
        if !current_pairs.contains(&selected) {
            return Err(EffectError::Illegal("赍剑借刀目标已失效".into()));
        }
        ddd_jijian_play_virtual(
            tx,
            player,
            current,
            card,
            kind,
            &[selected.0, selected.1],
            cards,
            remaining,
            skill,
        )
    } else {
        if payload.len() < remaining_end + 3 {
            return Err(EffectError::EngineFault(
                "truncated 赍剑 target contract".into(),
            ));
        }
        let min = payload[remaining_end] as usize;
        let max = payload[remaining_end + 1] as usize;
        let offered: Vec<_> = payload[remaining_end + 2..]
            .iter()
            .copied()
            .map(PlayerId::new)
            .collect();
        let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
        let mut distinct = std::collections::BTreeSet::new();
        let crate::command::play::SkillCardTargetSpec::Select {
            min: current_min,
            max: current_max,
            options,
        } = crate::command::play::skill_card_target_spec(tx, current, card)?
        else {
            return Err(EffectError::Illegal("赍剑目标形态已变化".into()));
        };
        if selected.len() < min
            || selected.len() > max
            || selected.len() < usize::from(current_min)
            || selected.len() > usize::from(current_max)
            || selected.iter().any(|target| {
                !distinct.insert(*target) || !offered.contains(target) || !options.contains(target)
            })
        {
            return Err(EffectError::Illegal("赍剑虚拟牌目标已失效".into()));
        }
        ddd_jijian_play_virtual(
            tx, player, current, card, kind, &selected, cards, remaining, skill,
        )
    }
}

pub(crate) fn ddd_zhenji_pile_def(index: u8) -> Result<u32, EffectError> {
    match index {
        1 => Ok(DDD_MIAOXING_PILE_1),
        2 => Ok(DDD_MIAOXING_PILE_2),
        _ => Err(EffectError::Illegal("水相组号无效".into())),
    }
}

pub(crate) fn ddd_zhenji_move_to_pile(
    tx: &mut Transaction,
    cards: Vec<CardId>,
    pile: ZoneId,
    from: ZoneId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let moved = cards.clone();
    let events = move_cards(tx, cards, pile, Some(from))?;
    for card in moved {
        tx.state_mut()
            .cards
            .get_mut(card)
            .expect("moved water card remains live")
            .face_up = false;
    }
    Ok(events)
}

pub(crate) fn ddd_zhenji_exchange_all(
    tx: &mut Transaction,
    owner: PlayerId,
    target: PlayerId,
    pile_index: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    let pile_def = ddd_zhenji_pile_def(pile_index)?;
    let hand_cards = tx.state().hand_cards(target);
    let pile_cards = tx.state().skill_pile_cards(owner, pile_def);
    if hand_cards.is_empty() && pile_cards.is_empty() {
        return Err(EffectError::Illegal("浮世不能交换两个空区域".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("浮世目标没有手牌区".into()))?;
    let pile = ensure_skill_pile_zone(tx, owner, pile_def)?;
    let mut events = if hand_cards.is_empty() {
        Vec::new()
    } else {
        ddd_zhenji_move_to_pile(tx, hand_cards, pile, hand)?
    };
    if !pile_cards.is_empty() {
        events.extend(move_cards(tx, pile_cards, hand, Some(pile))?);
    }
    Ok(events)
}

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

pub(crate) fn resolve_ddd_zhenji_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload.first() != Some(&RESOLVE_DDD_ZHENJI_CHOICE) {
        return Err(EffectError::EngineFault(
            "truncated 3D精选甄宓 choice".into(),
        ));
    }
    let choice = payload[1];
    let player = PlayerId::new(payload[2]);
    if observer != player && choice != 5 {
        return Err(EffectError::Illegal("甄宓选择观察者不匹配".into()));
    }
    match choice {
        0 => {
            let skill = engine_api::SkillId::new(payload[3]);
            let count = payload.get(4).copied().unwrap_or(0) as usize;
            if cancelled || selections.len() != 1 || payload.len() != 5 + count {
                return Err(EffectError::Illegal("淼形必须保留一组起始手牌".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::NONE,
                    action: 0,
                    stage: 1,
                    piles: vec![selections[0] as u8],
                    cards: payload[5..].iter().copied().map(CardId::new).collect(),
                    skill,
                }],
            )?;
        }
        1 => {
            let skill = engine_api::SkillId::new(payload[3]);
            let current_hand = tx.state().hand_count(player) as usize;
            let legal: std::collections::BTreeSet<_> = [1u8, 2]
                .into_iter()
                .filter(|index| {
                    ddd_zhenji_pile_def(*index).is_ok_and(|pile| {
                        tx.state().skill_pile_cards(player, pile).len() != current_hand
                    })
                })
                .map(u32::from)
                .collect();
            let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            if cancelled {
                if !selections.is_empty() || legal.len() != 1 {
                    return Err(EffectError::Illegal("淼形不能取消强制平衡".into()));
                }
                return Ok(());
            }
            if selections.is_empty()
                || selections.len() > 2
                || selected.len() != selections.len()
                || !selected.is_subset(&legal)
            {
                return Err(EffectError::Illegal("淼形平衡组选择无效".into()));
            }
            let mut piles: Vec<_> = selections.iter().map(|value| *value as u8).collect();
            piles.sort_unstable();
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::NONE,
                    action: 1,
                    stage: 1,
                    piles,
                    cards: Vec::new(),
                    skill,
                }],
            )?;
        }
        2 => {
            let skill = engine_api::SkillId::new(payload[3]);
            let count = payload.get(4).copied().unwrap_or(0) as usize;
            if cancelled || count == 0 || payload.len() != 5 + count {
                return Err(EffectError::Illegal("淼形移除水相选择无效".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::NONE,
                    action: 1,
                    stage: 2,
                    piles: payload[5..].iter().map(|value| *value as u8).collect(),
                    cards: selections.iter().copied().map(CardId::new).collect(),
                    skill,
                }],
            )?;
        }
        3 => {
            let skill = engine_api::SkillId::new(payload[3]);
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("浮世必须选择一组水相".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::NONE,
                    action: 2,
                    stage: 1,
                    piles: vec![selections[0] as u8],
                    cards: Vec::new(),
                    skill,
                }],
            )?;
        }
        4 => {
            let skill = engine_api::SkillId::new(payload[3]);
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal("取消浮世目标时不得选人".into()));
                }
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("浮世必须选择一名男性角色".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::new(selections[0]),
                    action: 2,
                    stage: 3,
                    piles: Vec::new(),
                    cards: Vec::new(),
                    skill,
                }],
            )?;
        }
        5 => {
            if payload.len() != 5 {
                return Err(EffectError::EngineFault("truncated 浮世目标选择".into()));
            }
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            if observer != target {
                return Err(EffectError::Illegal("浮世男性角色观察者不匹配".into()));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal("取消浮世交换时不得选水相".into()));
                }
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("浮世男性角色必须选择一组水相".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target,
                    action: 2,
                    stage: 4,
                    piles: vec![selections[0] as u8],
                    cards: Vec::new(),
                    skill,
                }],
            )?;
        }
        _ => {
            return Err(EffectError::EngineFault("unknown 3D精选甄宓 choice".into()));
        }
    }
    Ok(())
}

pub(crate) fn ddd_fenji_total_damage(state: &crate::state::GameState) -> u32 {
    state
        .damage_dealt_totals
        .values()
        .copied()
        .fold(0u32, u32::saturating_add)
}

pub(crate) fn ddd_fenji_phase_valid(tx: &Transaction, player: PlayerId, action: u8) -> bool {
    tx.state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        && tx.state().turn.current == player
        && match action {
            0 => {
                tx.state().turn.phase == Phase::Draw
                    && tx.state().turn.draw_phase_fixed_count().is_none()
            }
            1 => tx.state().turn.phase == Phase::Discard,
            _ => false,
        }
}

pub(crate) fn ddd_fenji_play(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    card: CardId,
    kind: CardKind,
    targets: &[PlayerId],
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    let count = if action == 0 {
        ddd_fenji_total_damage(tx.state())
    } else {
        targets.len().min(u32::MAX as usize) as u32
    };
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveDddZhaoang {
            player,
            action,
            stage: 2,
            card,
            kind: kind.tag(),
            count,
            skill,
        });
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_CARD, card.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, i32::from(kind.tag()));
    let result = crate::command::play::handle_skill_card(tx, player, card, targets);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_CARD, 0);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_SKILL, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    result
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_ddd_zhaoang(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    stage: u8,
    card: CardId,
    kind: u8,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !ddd_fenji_phase_valid(tx, player, action)
            {
                return Ok(Vec::new());
            }
            let used_mask = tx
                .state()
                .skills
                .get(skill)
                .map_or(0, |instance| instance.content_flags);
            let options = crate::command::play::source_free_ordinary_trick_kinds(
                tx.state(),
                player,
                used_mask,
            );
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options
                    .into_iter()
                    .map(|candidate| u32::from(candidate.tag()))
                    .collect(),
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_ZHAOANG_CHOICE,
                    0,
                    player.raw(),
                    u32::from(action),
                    skill.raw(),
                ],
            )
        }
        1 => {
            if !ddd_fenji_phase_valid(tx, player, action) {
                return Err(EffectError::Illegal("奋计阶段已经变化".into()));
            }
            let kind = CardKind::from_tag(kind)
                .ok_or_else(|| EffectError::Illegal("奋计锦囊牌名无效".into()))?;
            let used_mask = tx
                .state()
                .skills
                .get(skill)
                .map_or(0, |instance| instance.content_flags);
            if !crate::command::play::source_free_ordinary_trick_kinds(
                tx.state(),
                player,
                used_mask,
            )
            .contains(&kind)
            {
                return Err(EffectError::Illegal("奋计锦囊已使用或已无合法目标".into()));
            }
            if !tx
                .state_mut()
                .skills
                .get_mut(skill)
                .is_some_and(|instance| instance.set_content_flag(kind.tag(), true))
            {
                return Err(EffectError::ContentFault("奋计无法记录已用牌名".into()));
            }
            if action == 0 {
                tx.state_mut().turn.set_draw_phase_fixed_count(0);
            } else {
                tx.state_mut().turn.set_flag(turn_flag::SKIP_DISCARD, 1);
            }
            let card = allocate_virtual_card_kind(tx, kind)?;
            match crate::command::play::skill_card_target_spec(tx, player, card)? {
                crate::command::play::SkillCardTargetSpec::Immediate(targets) => {
                    ddd_fenji_play(tx, player, action, card, kind, &targets, skill)?;
                    Ok(Vec::new())
                }
                crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
                    let mut payload = vec![
                        RESOLVE_DDD_ZHAOANG_CHOICE,
                        1,
                        player.raw(),
                        u32::from(action),
                        card.raw(),
                        u32::from(kind.tag()),
                        skill.raw(),
                        u32::from(min),
                        u32::from(max),
                    ];
                    payload.extend(options.iter().map(|target| target.raw()));
                    open_choice(
                        tx,
                        player,
                        min,
                        max,
                        false,
                        options.into_iter().map(PlayerId::raw).collect(),
                        ChoiceKind::SelectTargets,
                        ContinuationKind::Custom,
                        payload,
                    )
                }
                crate::command::play::SkillCardTargetSpec::Pairs(pairs) => {
                    let mut payload = vec![
                        RESOLVE_DDD_ZHAOANG_CHOICE,
                        2,
                        player.raw(),
                        u32::from(action),
                        card.raw(),
                        u32::from(kind.tag()),
                        skill.raw(),
                        pairs.len().min(u32::MAX as usize) as u32,
                    ];
                    for (first, second) in &pairs {
                        payload.push(first.raw());
                        payload.push(second.raw());
                    }
                    open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        (0..pairs.len().min(u32::MAX as usize) as u32).collect(),
                        ChoiceKind::Custom,
                        ContinuationKind::Custom,
                        payload,
                    )
                }
            }
        }
        2 => {
            if action == 0 {
                let damage = ddd_fenji_total_damage(tx.state()).saturating_sub(count);
                if damage == 0 {
                    Ok(Vec::new())
                } else {
                    run_effects(
                        tx,
                        vec![EffectRequest::DrawCards {
                            player,
                            count: damage,
                        }],
                    )
                }
            } else if action == 1 {
                let actual = count.min(
                    crate::query::discardable_hand_equip_cards(tx.state(), player)
                        .len()
                        .min(u32::MAX as usize) as u32,
                );
                if actual == 0 {
                    Ok(Vec::new())
                } else {
                    run_effects(
                        tx,
                        vec![EffectRequest::DiscardHandEquipCount {
                            player,
                            count: actual,
                            skill,
                        }],
                    )
                }
            } else {
                Err(EffectError::EngineFault("unknown 奋计 phase action".into()))
            }
        }
        _ => Err(EffectError::EngineFault(
            "unknown 3D精选赵昂 continuation".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_zhaoang_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5
        || payload.first() != Some(&RESOLVE_DDD_ZHAOANG_CHOICE)
        || observer != PlayerId::new(payload[2])
        || cancelled
    {
        return Err(EffectError::Illegal("奋计选择无效".into()));
    }
    let player = PlayerId::new(payload[2]);
    let action = payload[3] as u8;
    match payload[1] {
        0 => {
            let [kind] = selections else {
                return Err(EffectError::Illegal("奋计必须选择一个锦囊牌名".into()));
            };
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhaoang {
                    player,
                    action,
                    stage: 1,
                    card: CardId::NONE,
                    kind: *kind as u8,
                    count: 0,
                    skill: engine_api::SkillId::new(payload[4]),
                }],
            )?;
        }
        1 => {
            if payload.len() < 9 {
                return Err(EffectError::EngineFault("truncated 奋计目标续体".into()));
            }
            let card = CardId::new(payload[4]);
            let kind = CardKind::from_tag(payload[5] as u8)
                .ok_or_else(|| EffectError::Illegal("奋计目标牌名无效".into()))?;
            let skill = engine_api::SkillId::new(payload[6]);
            let min = payload[7] as usize;
            let max = payload[8] as usize;
            let offered: Vec<_> = payload[9..].iter().copied().map(PlayerId::new).collect();
            let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            let mut distinct = std::collections::BTreeSet::new();
            let crate::command::play::SkillCardTargetSpec::Select {
                min: current_min,
                max: current_max,
                options,
            } = crate::command::play::skill_card_target_spec(tx, player, card)?
            else {
                return Err(EffectError::Illegal("奋计目标形态已变化".into()));
            };
            if selected.len() < min
                || selected.len() > max
                || selected.len() < usize::from(current_min)
                || selected.len() > usize::from(current_max)
                || selected.iter().any(|target| {
                    !distinct.insert(*target)
                        || !offered.contains(target)
                        || !options.contains(target)
                })
            {
                return Err(EffectError::Illegal("奋计目标已失效".into()));
            }
            ddd_fenji_play(tx, player, action, card, kind, &selected, skill)?;
        }
        2 => {
            if payload.len() < 8 || selections.len() != 1 {
                return Err(EffectError::EngineFault("truncated 奋计借刀续体".into()));
            }
            let card = CardId::new(payload[4]);
            let kind = CardKind::from_tag(payload[5] as u8)
                .ok_or_else(|| EffectError::Illegal("奋计借刀牌名无效".into()))?;
            let skill = engine_api::SkillId::new(payload[6]);
            let pair_count = payload[7] as usize;
            if payload.len() != 8 + pair_count.saturating_mul(2)
                || selections[0] as usize >= pair_count
            {
                return Err(EffectError::Illegal("奋计借刀目标组合无效".into()));
            }
            let index = 8 + selections[0] as usize * 2;
            let selected = (
                PlayerId::new(payload[index]),
                PlayerId::new(payload[index + 1]),
            );
            let crate::command::play::SkillCardTargetSpec::Pairs(current) =
                crate::command::play::skill_card_target_spec(tx, player, card)?
            else {
                return Err(EffectError::Illegal("奋计借刀目标形态已变化".into()));
            };
            if !current.contains(&selected) {
                return Err(EffectError::Illegal("奋计借刀目标已失效".into()));
            }
            ddd_fenji_play(
                tx,
                player,
                action,
                card,
                kind,
                &[selected.0, selected.1],
                skill,
            )?;
        }
        _ => return Err(EffectError::EngineFault("unknown 奋计 choice".into())),
    }
    Ok(())
}

pub(crate) fn ddd_liuba_he_cards(state: &crate::state::GameState, player: PlayerId) -> Vec<CardId> {
    let mut cards = state.hand_cards(player);
    if let Some(zone) = state.zones.find(ZoneKind::Equip, player) {
        cards.extend(
            state
                .zones
                .get(zone)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().copied()),
        );
    }
    cards.sort_by_key(|card| card.raw());
    cards
}

pub(crate) fn ddd_liuba_open_distribution_subset(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    remaining: Vec<CardId>,
    assigned: Vec<(CardId, PlayerId)>,
) -> Result<Vec<DomainEvent>, EffectError> {
    if remaining.is_empty() {
        let (cards, targets): (Vec<_>, Vec<_>) = assigned.into_iter().unzip();
        return run_effects(
            tx,
            vec![EffectRequest::ResolveDddLiuba {
                player,
                actor: player,
                action: 14,
                cards,
                targets,
                skill,
            }],
        );
    }
    if remaining.iter().any(|card| {
        !tx.state().cards.get(*card).is_some_and(|card| {
            tx.state()
                .zones
                .get(card.zone)
                .is_some_and(|zone| zone.kind == ZoneKind::Processing)
        })
    }) {
        return Err(EffectError::Illegal("丰政待分配牌已离开处理区".into()));
    }
    let mut payload = vec![
        RESOLVE_DDD_LIUBA_CHOICE,
        1,
        player.raw(),
        skill.raw(),
        remaining.len() as u32,
    ];
    payload.extend(remaining.iter().map(|card| card.raw()));
    payload.push(assigned.len() as u32);
    for (card, target) in assigned {
        payload.extend([card.raw(), target.raw()]);
    }
    if remaining.len() == 1 {
        payload[1] = 2;
        payload.push(1);
        payload.push(remaining[0].raw());
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            tx.state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            payload,
        );
    }
    open_choice(
        tx,
        player,
        1,
        remaining.len().min(u8::MAX as usize) as u8,
        false,
        remaining.iter().map(|card| card.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn ddd_liuba_finish_distribution(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    assigned: Vec<(CardId, PlayerId)>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("丰政处理区不存在".into()))?;
    let mut groups = std::collections::BTreeMap::<PlayerId, Vec<CardId>>::new();
    for (card, target) in assigned {
        if !tx
            .state()
            .cards
            .get(card)
            .is_some_and(|card| card.zone == processing)
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|target| target.alive)
        {
            return Err(EffectError::Illegal("丰政最终分配状态失效".into()));
        }
        groups.entry(target).or_default().push(card);
    }
    let lose = groups.values().any(|cards| cards.len() > 2);
    let mut events = Vec::new();
    for (target, cards) in groups {
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::EngineFault("丰政目标手牌区不存在".into()))?;
        events.extend(move_cards(tx, cards, hand, Some(processing))?);
    }
    if lose {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::RemoveSkill { skill });
    }
    let _ = player;
    Ok(events)
}

pub(crate) fn ddd_liuba_open_yulv_exchange(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let hand = tx.state().hand_cards(player);
    if hand.is_empty() {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ResolveDddLiuba {
                player,
                actor: player,
                action: 13,
                cards: Vec::new(),
                targets: Vec::new(),
                skill,
            });
        return run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }]);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        hand.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![RESOLVE_DDD_LIUBA_CHOICE, 6, player.raw(), skill.raw()],
    )
}
