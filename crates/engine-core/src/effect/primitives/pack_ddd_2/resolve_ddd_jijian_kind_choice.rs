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
