//! Effect primitives for `ddd_liuba_decode_distribution` and related helpers.

use super::*;

pub(crate) fn ddd_liuba_decode_distribution(
    payload: &[u32],
) -> Result<
    (
        PlayerId,
        engine_api::SkillId,
        Vec<CardId>,
        Vec<(CardId, PlayerId)>,
        usize,
    ),
    EffectError,
> {
    if payload.len() < 6 {
        return Err(EffectError::EngineFault("truncated 丰政分配续体".into()));
    }
    let player = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    let remaining_count = payload[4] as usize;
    let remaining_end = 5usize.saturating_add(remaining_count);
    let assigned_count = *payload
        .get(remaining_end)
        .ok_or_else(|| EffectError::EngineFault("truncated 丰政分配计数".into()))?
        as usize;
    let assigned_end = remaining_end
        .saturating_add(1)
        .saturating_add(assigned_count.saturating_mul(2));
    if payload.len() < assigned_end {
        return Err(EffectError::EngineFault("truncated 丰政分配记录".into()));
    }
    let remaining = payload[5..remaining_end]
        .iter()
        .copied()
        .map(CardId::new)
        .collect();
    let assigned = payload[remaining_end + 1..assigned_end]
        .chunks_exact(2)
        .map(|pair| (CardId::new(pair[0]), PlayerId::new(pair[1])))
        .collect();
    Ok((player, skill, remaining, assigned, assigned_end))
}

pub(crate) fn resolve_ddd_liuba_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DDD_LIUBA_CHOICE) || payload.len() < 4 {
        return Err(EffectError::Illegal("刘巴选择续体无效".into()));
    }
    match payload[1] {
        // 是否发动轮首丰政。
        0 => {
            let player = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            if observer != player {
                return Err(EffectError::Illegal("丰政发动者变化".into()));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal("取消丰政时不得选择".into()));
                }
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal("丰政发动选择无效".into()));
            }
            let count = payload.get(4).copied().unwrap_or(0) as usize;
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("丰政牌堆不存在".into()))?;
            if tx
                .state()
                .zones
                .get(draw)
                .map_or(0, |zone| zone.cards.len())
                < count
            {
                reshuffle_discard_into_draw(tx)?;
            }
            let zone = tx
                .state()
                .zones
                .get(draw)
                .ok_or_else(|| EffectError::EngineFault("丰政牌堆消失".into()))?;
            let take = count.min(zone.cards.len());
            if take == 0 {
                return Ok(());
            }
            let cards = zone.cards[zone.cards.len() - take..].to_vec();
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("丰政处理区不存在".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveDddLiuba {
                    player,
                    actor: player,
                    action: 10,
                    cards: cards.clone(),
                    targets: Vec::new(),
                    skill,
                });
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: cards.clone(),
                        to: processing,
                        from: Some(draw),
                    },
                    EffectRequest::RevealCards {
                        cards,
                        to: Vec::new(),
                    },
                ],
            )?;
        }
        // 选择本次分配的非空子集。
        1 => {
            let (player, _skill, remaining, _assigned, assigned_end) =
                ddd_liuba_decode_distribution(payload)?;
            if observer != player || cancelled || payload.len() != assigned_end {
                return Err(EffectError::Illegal("丰政子集选择无效".into()));
            }
            let selected: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            let mut distinct = std::collections::BTreeSet::new();
            if selected.is_empty()
                || selected
                    .iter()
                    .any(|card| !distinct.insert(*card) || !remaining.contains(card))
            {
                return Err(EffectError::Illegal("丰政子集必须来自待分配牌".into()));
            }
            let mut next = payload[..assigned_end].to_vec();
            next[1] = 2;
            next.push(selected.len() as u32);
            next.extend(selected.iter().map(|card| card.raw()));
            open_choice(
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
                next,
            )?;
        }
        // 为已选子集指定一名获得者。
        2 => {
            let (player, skill, remaining, mut assigned, assigned_end) =
                ddd_liuba_decode_distribution(payload)?;
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("丰政获得者选择无效".into()));
            }
            let selected_count = *payload
                .get(assigned_end)
                .ok_or_else(|| EffectError::EngineFault("丰政子集计数缺失".into()))?
                as usize;
            if payload.len() != assigned_end + 1 + selected_count {
                return Err(EffectError::EngineFault("丰政子集续体截断".into()));
            }
            let selected: Vec<_> = payload[assigned_end + 1..]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let target = PlayerId::new(selections[0]);
            let mut distinct = std::collections::BTreeSet::new();
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|target| target.alive)
                || selected.is_empty()
                || selected
                    .iter()
                    .any(|card| !distinct.insert(*card) || !remaining.contains(card))
            {
                return Err(EffectError::Illegal("丰政目标或子集已失效".into()));
            }
            assigned.extend(selected.iter().map(|card| (*card, target)));
            let remaining: Vec<_> = remaining
                .into_iter()
                .filter(|card| !selected.contains(card))
                .collect();
            ddd_liuba_open_distribution_subset(tx, player, skill, remaining, assigned)?;
        }
        // 玉律摸牌/弃牌二选一。
        3 => {
            if payload.len() != 6 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("玉律分支选择无效".into()));
            }
            let player = PlayerId::new(payload[2]);
            let actor = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let ordinal = payload[5];
            if observer != player || !matches!(selections[0], 0 | 1) {
                return Err(EffectError::Illegal("玉律分支选择者变化".into()));
            }
            if selections[0] == 0 {
                if ordinal == 2 {
                    tx.state_mut().resolution.deferred_tail_effects.push(
                        EffectRequest::ResolveDddLiuba {
                            player,
                            actor,
                            action: 12,
                            cards: Vec::new(),
                            targets: Vec::new(),
                            skill,
                        },
                    );
                }
                run_effects(
                    tx,
                    vec![EffectRequest::DrawCards {
                        player: actor,
                        count: 1,
                    }],
                )?;
            } else {
                let options = ddd_liuba_he_cards(tx.state(), actor);
                if options.is_empty() {
                    if ordinal == 2 {
                        run_effects(
                            tx,
                            vec![EffectRequest::ResolveDddLiuba {
                                player,
                                actor,
                                action: 12,
                                cards: Vec::new(),
                                targets: Vec::new(),
                                skill,
                            }],
                        )?;
                    }
                } else {
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
                            RESOLVE_DDD_LIUBA_CHOICE,
                            5,
                            player.raw(),
                            actor.raw(),
                            skill.raw(),
                            ordinal,
                        ],
                    )?;
                }
            }
        }
        // 起始玉律牌。
        4 => {
            let player = PlayerId::new(payload[2]);
            let _skill = engine_api::SkillId::new(payload[3]);
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("玉律起始置牌无效".into()));
            }
            let card = CardId::new(selections[0]);
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if !tx
                .state()
                .cards
                .get(card)
                .is_some_and(|card| Some(card.zone) == hand)
            {
                return Err(EffectError::Illegal("玉律起始牌已离开手牌".into()));
            }
            let pile = ensure_skill_pile_zone(tx, player, DDD_YULV_DEF)?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to: pile,
                    from: hand,
                }],
            )?;
        }
        // 当前回合角色强制弃一张牌。
        5 => {
            if payload.len() != 6 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("玉律弃牌选择无效".into()));
            }
            let player = PlayerId::new(payload[2]);
            let actor = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let ordinal = payload[5];
            if observer != actor {
                return Err(EffectError::Illegal("玉律弃牌角色变化".into()));
            }
            let card = CardId::new(selections[0]);
            if !ddd_liuba_he_cards(tx.state(), actor).contains(&card) {
                return Err(EffectError::Illegal("玉律弃牌已失效".into()));
            }
            if ordinal == 2 {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveDddLiuba {
                        player,
                        actor,
                        action: 12,
                        cards: Vec::new(),
                        targets: Vec::new(),
                        skill,
                    },
                );
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: actor,
                    cards: vec![card],
                }],
            )?;
        }
        // 用一张手牌交换公开的“玉律”。
        6 => {
            let player = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("玉律交换选择无效".into()));
            }
            let new_card = CardId::new(selections[0]);
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("玉律拥有者手牌区不存在".into()))?;
            if !tx
                .state()
                .cards
                .get(new_card)
                .is_some_and(|card| card.zone == hand)
            {
                return Err(EffectError::Illegal("玉律交换手牌已失效".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddLiuba {
                    player,
                    actor: player,
                    action: 15,
                    cards: vec![new_card],
                    targets: Vec::new(),
                    skill,
                }],
            )?;
        }
        _ => return Err(EffectError::EngineFault("unknown 3D精选刘巴 choice".into())),
    }
    Ok(())
}

pub(crate) fn ddd_shichao_target(instance: &crate::state::SkillInstance) -> PlayerId {
    if instance.state_blob.len() == 8 && instance.state_blob[..4] == DDD_SHICHAO_STATE_TAG {
        PlayerId::new(u32::from_le_bytes(
            instance.state_blob[4..8].try_into().unwrap(),
        ))
    } else {
        PlayerId::NONE
    }
}

pub(crate) fn set_ddd_shichao_target(instance: &mut crate::state::SkillInstance, target: PlayerId) {
    instance.state_blob = DDD_SHICHAO_STATE_TAG.to_vec();
    instance
        .state_blob
        .extend_from_slice(&target.raw().to_le_bytes());
}

pub(crate) fn ddd_jianshuo_seat_order(
    state: &crate::state::GameState,
    start: PlayerId,
) -> Vec<PlayerId> {
    let mut order = state.players.alive_by_seat();
    if let Some(index) = order.iter().position(|player| *player == start) {
        order.rotate_left(index);
    }
    order
}

pub(crate) fn ddd_shichao_candidates(
    state: &crate::state::GameState,
    skill: engine_api::SkillId,
) -> Vec<PlayerId> {
    let Some(instance) = state.skills.get(skill) else {
        return Vec::new();
    };
    let ordinal = instance.mark.saturating_add(1) as usize;
    let alive = state.players.alive_by_seat();
    if ordinal == 0 || ordinal > alive.len() {
        return Vec::new();
    }
    let mut counts: Vec<_> = alive
        .iter()
        .map(|player| state.hand_count(*player))
        .collect();
    counts.sort_unstable_by(|a, b| b.cmp(a));
    counts.dedup();
    let wanted = counts
        .get(ordinal - 1)
        .copied()
        .or_else(|| counts.last().copied());
    alive
        .into_iter()
        .filter(|player| Some(state.hand_count(*player)) == wanted)
        .collect()
}

pub(crate) fn ddd_jianshuo_decode_join(
    payload: &[u32],
) -> Result<
    (
        PlayerId,
        PlayerId,
        engine_api::SkillId,
        CardId,
        CardId,
        usize,
        Vec<PlayerId>,
        Vec<(PlayerId, PlayerId, CardId)>,
        usize,
    ),
    EffectError,
> {
    if payload.len() < 10 {
        return Err(EffectError::EngineFault("分野加入续体截断".into()));
    }
    let player = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    let player_card = CardId::new(payload[5]);
    let target_card = CardId::new(payload[6]);
    let index = payload[7] as usize;
    let others_count = payload[8] as usize;
    let others_end = 9usize.saturating_add(others_count);
    let joined_count = *payload
        .get(others_end)
        .ok_or_else(|| EffectError::EngineFault("分野加入计数缺失".into()))?
        as usize;
    let joined_end = others_end
        .saturating_add(1)
        .saturating_add(joined_count.saturating_mul(3));
    if payload.len() < joined_end {
        return Err(EffectError::EngineFault("分野加入记录截断".into()));
    }
    let others = payload[9..others_end]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    let joined = payload[others_end + 1..joined_end]
        .chunks_exact(3)
        .map(|entry| {
            (
                PlayerId::new(entry[0]),
                PlayerId::new(entry[1]),
                CardId::new(entry[2]),
            )
        })
        .collect();
    Ok((
        player,
        target,
        skill,
        player_card,
        target_card,
        index,
        others,
        joined,
        joined_end,
    ))
}

pub(crate) fn ddd_jianshuo_advance_join(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
    player_card: CardId,
    target_card: CardId,
    index: usize,
    others: Vec<PlayerId>,
    joined: Vec<(PlayerId, PlayerId, CardId)>,
) -> Result<Vec<DomainEvent>, EffectError> {
    if index >= others.len() {
        let mut cards = vec![player_card, target_card];
        let mut sides = vec![player, target];
        for (_, side, card) in joined {
            cards.push(card);
            sides.push(side);
        }
        return run_effects(
            tx,
            vec![EffectRequest::ResolveDddJianshuo {
                player,
                target,
                action: 10,
                cards,
                targets: sides,
                skill,
            }],
        );
    }
    let actor = others[index];
    if !tx
        .state()
        .players
        .get(actor)
        .is_some_and(|participant| participant.alive)
        || tx.state().hand_cards(actor).is_empty()
    {
        return ddd_jianshuo_advance_join(
            tx,
            player,
            target,
            skill,
            player_card,
            target_card,
            index + 1,
            others,
            joined,
        );
    }
    let mut payload = vec![
        RESOLVE_DDD_JIANSHUO_CHOICE,
        2,
        player.raw(),
        target.raw(),
        skill.raw(),
        player_card.raw(),
        target_card.raw(),
        index as u32,
        others.len() as u32,
    ];
    payload.extend(others.iter().map(|player| player.raw()));
    payload.push(joined.len() as u32);
    for (participant, side, card) in joined {
        payload.extend([participant.raw(), side.raw(), card.raw()]);
    }
    open_choice(
        tx,
        actor,
        0,
        1,
        true,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn ddd_jianshuo_open_sha(
    tx: &mut Transaction,
    mut winners: Vec<PlayerId>,
    losers: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    while let Some(winner) = winners.first().copied() {
        winners.remove(0);
        if !tx
            .state()
            .players
            .get(winner)
            .is_some_and(|participant| participant.alive)
        {
            continue;
        }
        let card = allocate_virtual_sha(tx)?;
        if crate::query::card_use_cancelled_for_card(tx.state(), winner, card) {
            remove_unused_virtual_card(tx, card)?;
            continue;
        }
        let options: Vec<_> = losers
            .iter()
            .copied()
            .filter(|loser| {
                tx.state()
                    .players
                    .get(*loser)
                    .is_some_and(|participant| participant.alive)
                    && crate::command::play::validate_sha_target_with_options(
                        tx, winner, card, *loser, true,
                    )
                    .is_ok()
            })
            .collect();
        if options.is_empty() {
            remove_unused_virtual_card(tx, card)?;
            continue;
        }
        let mut payload = vec![
            RESOLVE_DDD_JIANSHUO_CHOICE,
            4,
            winner.raw(),
            skill.raw(),
            card.raw(),
            winners.len() as u32,
        ];
        payload.extend(winners.iter().map(|player| player.raw()));
        payload.push(losers.len() as u32);
        payload.extend(losers.iter().map(|player| player.raw()));
        return open_choice(
            tx,
            winner,
            0,
            1,
            true,
            options.into_iter().map(PlayerId::raw).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            payload,
        );
    }
    Ok(Vec::new())
}
