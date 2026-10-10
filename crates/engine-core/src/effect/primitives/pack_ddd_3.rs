//! Effect primitive implementations — ddd 3 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

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

pub(crate) const RESOLVE_DDD_JIANSHUO_CHOICE: u32 = 0xDCDC_6330;
pub(crate) const DDD_SHICHAO_STATE_TAG: [u8; 4] = *b"SC33";

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

pub(crate) fn ddd_jianshuo_seat_order(state: &crate::state::GameState, start: PlayerId) -> Vec<PlayerId> {
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

pub(crate) fn resolve_ddd_jianshuo(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    action: u8,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if action <= 2 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        // 分野：发起者先选择拼点牌。
        0 => {
            if player == target
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || tx.state().hand_cards(player).is_empty()
                || tx.state().hand_cards(target).is_empty()
            {
                return Err(EffectError::Illegal("分野拼点目标无效".into()));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                tx.state()
                    .hand_cards(player)
                    .into_iter()
                    .map(CardId::raw)
                    .collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_JIANSHUO_CHOICE,
                    0,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        // 逝潮准备阶段：选择指定手牌名次角色。
        1 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob.clear();
            }
            let lord = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .find(|candidate| {
                    tx.state()
                        .players
                        .get(*candidate)
                        .is_some_and(|participant| participant.identity == Identity::Lord)
                });
            let options = ddd_shichao_candidates(tx.state(), skill);
            if lord.is_none() || options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_JIANSHUO_CHOICE, 5, player.raw(), skill.raw()],
            )
        }
        // 逝潮伤害前：由被记录的伤害来源决定是否防止并升级名次。
        2 => {
            let damage_id = tx.state().resolution.current_damage_id;
            let watched = tx
                .state()
                .skills
                .get(skill)
                .map(ddd_shichao_target)
                .unwrap_or(PlayerId::NONE);
            if damage_id == 0 || watched != target || target == PlayerId::NONE {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                0,
                1,
                true,
                vec![1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_JIANSHUO_CHOICE,
                    7,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    damage_id,
                ],
            )
        }
        // 分野全部选牌完成：按均值判定并统一移动实体牌。
        10 => {
            if cards.len() < 2 || cards.len() != targets.len() {
                return Err(EffectError::EngineFault("分野结算记录无效".into()));
            }
            let mut side_one = Vec::new();
            let mut side_two = Vec::new();
            let mut sum_one = 0u32;
            let mut sum_two = 0u32;
            for (card, side) in cards.iter().zip(&targets) {
                let value = tx
                    .state()
                    .cards
                    .get(*card)
                    .ok_or_else(|| EffectError::Illegal("分野拼点牌消失".into()))?;
                let owner = value.owner;
                let hand = tx.state().zones.find(ZoneKind::Hand, owner);
                if Some(value.zone) != hand || !matches!(*side, x if x == player || x == target) {
                    return Err(EffectError::Illegal("分野拼点牌或阵营失效".into()));
                }
                let number = u32::from(crate::query::effective_card_number(
                    tx.state(),
                    owner,
                    value,
                ));
                if *side == player {
                    side_one.push(owner);
                    sum_one = sum_one.saturating_add(number);
                } else {
                    side_two.push(owner);
                    sum_two = sum_two.saturating_add(number);
                }
            }
            let left = sum_one.saturating_mul(side_two.len() as u32);
            let right = sum_two.saturating_mul(side_one.len() as u32);
            let (winner_side, loser_side) = if left > right {
                (side_one, side_two)
            } else if right > left {
                (side_two, side_one)
            } else {
                (Vec::new(), side_one.into_iter().chain(side_two).collect())
            };
            let order = ddd_jianshuo_seat_order(tx.state(), player);
            let winners: Vec<_> = order
                .iter()
                .copied()
                .filter(|participant| winner_side.contains(participant))
                .collect();
            let losers: Vec<_> = order
                .into_iter()
                .filter(|participant| loser_side.contains(participant))
                .collect();
            let mut frozen = winners;
            frozen.push(PlayerId::NONE);
            frozen.extend(losers);
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDddJianshuo {
                    player,
                    target,
                    action: 11,
                    cards: Vec::new(),
                    targets: frozen,
                    skill,
                },
            );
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("弃牌堆不存在".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::RevealCards {
                        cards: cards.clone(),
                        to: Vec::new(),
                    },
                    EffectRequest::MoveCards {
                        cards,
                        to: discard,
                        from: None,
                    },
                ],
            )
        }
        // 分野胜方依座次逐人选择不同败方目标使用虚拟【杀】。
        11 => {
            let split = targets
                .iter()
                .position(|target| *target == PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("分野胜负分隔缺失".into()))?;
            ddd_jianshuo_open_sha(
                tx,
                targets[..split].to_vec(),
                targets[split + 1..].to_vec(),
                skill,
            )
        }
        _ => Err(EffectError::EngineFault("unknown 3D精选蹇硕 action".into())),
    }
}

pub(crate) fn resolve_ddd_jianshuo_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DDD_JIANSHUO_CHOICE) || payload.len() < 4 {
        return Err(EffectError::Illegal("蹇硕选择续体无效".into()));
    }
    match payload[1] {
        // 发起者拼点牌。
        0 => {
            if payload.len() != 5 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("分野发起者选牌无效".into()));
            }
            let player = PlayerId::new(payload[2]);
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            if observer != player {
                return Err(EffectError::Illegal("分野发起者变化".into()));
            }
            let card = CardId::new(selections[0]);
            if !tx.state().hand_cards(player).contains(&card) {
                return Err(EffectError::Illegal("分野发起者拼点牌失效".into()));
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                tx.state()
                    .hand_cards(target)
                    .into_iter()
                    .map(CardId::raw)
                    .collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_JIANSHUO_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    card.raw(),
                ],
            )?;
        }
        // 目标拼点牌，随后依座次询问其他角色。
        1 => {
            if payload.len() != 6 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("分野目标选牌无效".into()));
            }
            let player = PlayerId::new(payload[2]);
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let player_card = CardId::new(payload[5]);
            if observer != target
                || !tx
                    .state()
                    .hand_cards(target)
                    .contains(&CardId::new(selections[0]))
            {
                return Err(EffectError::Illegal("分野目标拼点牌失效".into()));
            }
            let others: Vec<_> = ddd_jianshuo_seat_order(tx.state(), player)
                .into_iter()
                .filter(|actor| *actor != player && *actor != target)
                .collect();
            ddd_jianshuo_advance_join(
                tx,
                player,
                target,
                skill,
                player_card,
                CardId::new(selections[0]),
                0,
                others,
                Vec::new(),
            )?;
        }
        // 其他角色选择加入哪一方，或取消。
        2 => {
            let (player, target, skill, player_card, target_card, index, others, joined, end) =
                ddd_jianshuo_decode_join(payload)?;
            if payload.len() != end
                || observer != others.get(index).copied().unwrap_or(PlayerId::NONE)
            {
                return Err(EffectError::Illegal("分野加入角色变化".into()));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal("取消加入分野时不得选择".into()));
                }
                ddd_jianshuo_advance_join(
                    tx,
                    player,
                    target,
                    skill,
                    player_card,
                    target_card,
                    index + 1,
                    others,
                    joined,
                )?;
            } else {
                if selections.len() != 1 || !matches!(selections[0], 1 | 2) {
                    return Err(EffectError::Illegal("分野加入阵营无效".into()));
                }
                let side = if selections[0] == 1 { player } else { target };
                let mut next = payload[..end].to_vec();
                next[1] = 3;
                next.push(side.raw());
                open_choice(
                    tx,
                    observer,
                    1,
                    1,
                    false,
                    tx.state()
                        .hand_cards(observer)
                        .into_iter()
                        .map(CardId::raw)
                        .collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    next,
                )?;
            }
        }
        // 加入者选择扣置的拼点牌。
        3 => {
            let (player, target, skill, player_card, target_card, index, others, mut joined, end) =
                ddd_jianshuo_decode_join(payload)?;
            if payload.len() != end + 1 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("分野加入选牌无效".into()));
            }
            let participant = others.get(index).copied().unwrap_or(PlayerId::NONE);
            let side = PlayerId::new(payload[end]);
            let card = CardId::new(selections[0]);
            if observer != participant
                || !matches!(side, x if x == player || x == target)
                || !tx.state().hand_cards(participant).contains(&card)
            {
                return Err(EffectError::Illegal("分野加入牌或阵营失效".into()));
            }
            joined.push((participant, side, card));
            ddd_jianshuo_advance_join(
                tx,
                player,
                target,
                skill,
                player_card,
                target_card,
                index + 1,
                others,
                joined,
            )?;
        }
        // 当前胜方选择不同败方目标使用【杀】。
        4 => {
            if payload.len() < 7 {
                return Err(EffectError::EngineFault("分野用杀续体截断".into()));
            }
            let winner = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            let card = CardId::new(payload[4]);
            let winner_count = payload[5] as usize;
            let winners_end = 6usize.saturating_add(winner_count);
            let loser_count = *payload
                .get(winners_end)
                .ok_or_else(|| EffectError::EngineFault("分野败方计数缺失".into()))?
                as usize;
            if payload.len() != winners_end + 1 + loser_count || observer != winner {
                return Err(EffectError::Illegal("分野用杀角色变化".into()));
            }
            let winners = payload[6..winners_end]
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            let mut losers: Vec<_> = payload[winners_end + 1..]
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal("取消分野用杀时不得选择".into()));
                }
                remove_unused_virtual_card(tx, card)?;
                ddd_jianshuo_open_sha(tx, winners, losers, skill)?;
            } else {
                if selections.len() != 1 {
                    return Err(EffectError::Illegal("分野用杀目标数量无效".into()));
                }
                let target = PlayerId::new(selections[0]);
                let Some(index) = losers.iter().position(|loser| *loser == target) else {
                    return Err(EffectError::Illegal("分野用杀目标失效".into()));
                };
                losers.remove(index);
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveDddJianshuo {
                        player: winner,
                        target,
                        action: 11,
                        cards: Vec::new(),
                        targets: winners
                            .into_iter()
                            .chain(std::iter::once(PlayerId::NONE))
                            .chain(losers)
                            .collect(),
                        skill,
                    },
                );
                crate::command::play::handle_virtual_sha_with_policies_and_bonus(
                    tx,
                    winner,
                    card,
                    &[target],
                    true,
                    true,
                    0,
                )?;
            }
        }
        // 逝潮选择手牌名次角色并调整手牌。
        5 => {
            if payload.len() != 4 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("逝潮目标选择无效".into()));
            }
            let player = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            let target = PlayerId::new(selections[0]);
            if observer != player || !ddd_shichao_candidates(tx.state(), skill).contains(&target) {
                return Err(EffectError::Illegal("逝潮手牌名次目标失效".into()));
            }
            let lord_max_hp = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .find_map(|candidate| {
                    tx.state().players.get(candidate).and_then(|participant| {
                        (participant.identity == Identity::Lord).then_some(participant.max_hp)
                    })
                })
                .unwrap_or(0);
            let desired = tx.state().hand_count(target).min(lord_max_hp.max(0) as u32);
            let current = tx.state().hand_count(player);
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                set_ddd_shichao_target(instance, target);
            }
            if current < desired {
                run_effects(
                    tx,
                    vec![EffectRequest::DrawCards {
                        player,
                        count: (desired - current) as u32,
                    }],
                )?;
            } else if current > desired {
                open_choice(
                    tx,
                    player,
                    (current - desired).min(u32::from(u8::MAX)) as u8,
                    (current - desired).min(u32::from(u8::MAX)) as u8,
                    false,
                    tx.state()
                        .hand_cards(player)
                        .into_iter()
                        .map(CardId::raw)
                        .collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    vec![
                        RESOLVE_DDD_JIANSHUO_CHOICE,
                        6,
                        player.raw(),
                        skill.raw(),
                        (current - desired) as u32,
                    ],
                )?;
            }
        }
        // 逝潮强制弃手牌。
        6 => {
            if payload.len() != 5 || cancelled || selections.len() != payload[4] as usize {
                return Err(EffectError::Illegal("逝潮弃牌数量无效".into()));
            }
            let player = PlayerId::new(payload[2]);
            if observer != player {
                return Err(EffectError::Illegal("逝潮弃牌角色变化".into()));
            }
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            let hand = tx.state().hand_cards(player);
            let mut distinct = std::collections::BTreeSet::new();
            if cards
                .iter()
                .any(|card| !distinct.insert(*card) || !hand.contains(card))
            {
                return Err(EffectError::Illegal("逝潮弃牌失效".into()));
            }
            run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }])?;
        }
        // 逝潮伤害来源选择防止伤害。
        7 => {
            if payload.len() != 6 {
                return Err(EffectError::EngineFault("逝潮防伤续体截断".into()));
            }
            let player = PlayerId::new(payload[2]);
            let source = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let damage_id = payload[5];
            if observer != source {
                return Err(EffectError::Illegal("逝潮防伤选择者变化".into()));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal("取消逝潮防伤时不得选择".into()));
                }
            } else {
                if selections != [1]
                    || tx.state().resolution.current_damage_id != damage_id
                    || tx.state().skills.get(skill).map(ddd_shichao_target) != Some(source)
                {
                    return Err(EffectError::Illegal("逝潮防伤状态失效".into()));
                }
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.mark = instance.mark.saturating_add(1);
                }
                run_effects(tx, vec![EffectRequest::PreventDamage { damage_id }])?;
            }
            let _ = player;
        }
        _ => return Err(EffectError::EngineFault("unknown 3D精选蹇硕 choice".into())),
    }
    Ok(())
}

pub(crate) const RESOLVE_DDD_GUANNING_CHOICE: u32 = 0xDCDC_6340;
pub(crate) const DDD_YOUXUE_MARK: u32 = 2_449_657_606;
pub(crate) const DDD_CHENGJING_DEF: u32 = 4_114_032_104;

pub(crate) fn ddd_guanning_mark_holder(state: &crate::state::GameState) -> Option<PlayerId> {
    state.players.alive_by_seat().into_iter().find(|player| {
        state.players.get(*player).is_some_and(|participant| {
            participant
                .marks
                .iter()
                .any(|(mark, count)| *mark == DDD_YOUXUE_MARK && *count > 0)
        })
    })
}

pub(crate) fn ddd_guanning_distance(state: &crate::state::GameState, from: PlayerId, to: PlayerId) -> u32 {
    let mut ctx = crate::query::QueryContext::new();
    match crate::query::query(state, &QueryRequest::Distance { from, to }, &mut ctx) {
        QueryResponse::Int(value) => value.max(0) as u32,
        _ => 0,
    }
}

pub(crate) fn ddd_guanning_target_spec(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    kind: CardKind,
    skill: engine_api::SkillId,
) -> Result<crate::command::play::SkillCardTargetSpec, EffectError> {
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_CARD, card.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, i32::from(kind.tag()));
    let result = crate::command::play::skill_card_target_spec(tx, player, card);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_CARD, 0);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_SKILL, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    result
}

pub(crate) fn ddd_guanning_play(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    kind: CardKind,
    targets: &[PlayerId],
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let source_was_hand = tx
        .state()
        .cards
        .get(card)
        .is_some_and(|source| Some(source.zone) == hand);
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveDddGuanning {
            player,
            actor: player,
            action: 12,
            card,
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
    let result = crate::command::play::play_as_kind(
        tx,
        player,
        card,
        kind,
        targets,
        false,
        false,
        false,
        source_was_hand,
    );
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_CARD, 0);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_SKILL, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    result
}

pub(crate) fn resolve_ddd_guanning(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    action: u8,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let round = tx.state().turn.round;
            if round == 0
                || !tx
                    .state_mut()
                    .skills
                    .get_mut(skill)
                    .is_some_and(|instance| {
                        if instance.round_fire_round == round {
                            false
                        } else {
                            instance.round_fire_round = round;
                            true
                        }
                    })
            {
                return Ok(Vec::new());
            }
            let source = if let Some(holder) = ddd_guanning_mark_holder(tx.state()) {
                holder
            } else {
                run_effects(
                    tx,
                    vec![EffectRequest::AddMark {
                        player,
                        mark: engine_api::MarkDefId::new(DDD_YOUXUE_MARK),
                        delta: 1,
                    }],
                )?;
                player
            };
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    tx.state().players.get(*target).is_some_and(|participant| {
                        !participant
                            .marks
                            .iter()
                            .any(|(mark, count)| *mark == DDD_YOUXUE_MARK && *count > 0)
                    })
                })
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
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_GUANNING_CHOICE,
                    0,
                    player.raw(),
                    source.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || actor != tx.state().turn.current
                || ddd_guanning_mark_holder(tx.state()) != Some(actor)
            {
                return Ok(Vec::new());
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
            let Some(candidate) = tx
                .state()
                .turn
                .ddd_chengjing_eligible_uses
                .iter()
                .rev()
                .copied()
                .find(|candidate| {
                    tx.state()
                        .cards
                        .get(*candidate)
                        .is_some_and(|card| card.zone == discard)
                })
            else {
                return Ok(Vec::new());
            };
            let pile = ensure_skill_pile_zone(tx, player, DDD_CHENGJING_DEF)?;
            let old = tx.state().skill_pile_cards(player, DDD_CHENGJING_DEF);
            let mut effects = Vec::new();
            if !old.is_empty() {
                effects.push(EffectRequest::MoveCardsWithReason {
                    cards: old,
                    to: discard,
                    from: Some(pile),
                    reason: CardMoveReason::Discard,
                });
            }
            effects.push(EffectRequest::MoveCardsWithReason {
                cards: vec![candidate],
                to: pile,
                from: Some(discard),
                reason: CardMoveReason::Unspecified,
            });
            run_effects(tx, effects)
        }
        2 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
            {
                return Ok(Vec::new());
            }
            let source = tx
                .state()
                .cards
                .get(card)
                .ok_or_else(|| EffectError::Illegal("承经素材牌失效".into()))?;
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let equip = tx.state().zones.find(ZoneKind::Equip, player);
            if source.owner != player || !matches!(Some(source.zone), x if x == hand || x == equip)
            {
                return Err(EffectError::Illegal("承经素材牌不在手牌或装备区".into()));
            }
            let jing = tx.state().skill_pile_cards(player, DDD_CHENGJING_DEF);
            let kind = jing
                .first()
                .and_then(|jing| tx.state().cards.get(*jing))
                .map(|jing| jing.kind)
                .ok_or_else(|| EffectError::Illegal("承经没有经".into()))?;
            match ddd_guanning_target_spec(tx, player, card, kind, skill)? {
                crate::command::play::SkillCardTargetSpec::Immediate(targets) => {
                    ddd_guanning_play(tx, player, card, kind, &targets, skill)?;
                    Ok(Vec::new())
                }
                crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
                    let mut payload = vec![
                        RESOLVE_DDD_GUANNING_CHOICE,
                        1,
                        player.raw(),
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
                        RESOLVE_DDD_GUANNING_CHOICE,
                        2,
                        player.raw(),
                        card.raw(),
                        u32::from(kind.tag()),
                        skill.raw(),
                        pairs.len() as u32,
                    ];
                    for (first, second) in &pairs {
                        payload.extend([first.raw(), second.raw()]);
                    }
                    open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        (0..pairs.len() as u32).collect(),
                        ChoiceKind::Custom,
                        ContinuationKind::Custom,
                        payload,
                    )
                }
            }
        }
        12 => {
            let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            if !tx
                .state()
                .cards
                .get(card)
                .is_some_and(|source| Some(source.zone) == discard)
            {
                return Ok(Vec::new());
            }
            let Some(holder) = ddd_guanning_mark_holder(tx.state()) else {
                return Ok(Vec::new());
            };
            let alive = tx.state().players.alive_by_seat();
            let Some(index) = alive.iter().position(|candidate| *candidate == holder) else {
                return Ok(Vec::new());
            };
            let target = alive[(index + 1) % alive.len()];
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                vec![1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_GUANNING_CHOICE,
                    3,
                    player.raw(),
                    card.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        _ => Err(EffectError::EngineFault("unknown 3D精选管宁 action".into())),
    }
}

pub(crate) fn resolve_ddd_guanning_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 || payload.first() != Some(&RESOLVE_DDD_GUANNING_CHOICE) {
        return Err(EffectError::EngineFault("truncated 管宁 choice".into()));
    }
    let player = PlayerId::new(payload[2]);
    if observer != player {
        return Err(EffectError::Illegal("管宁选择者变化".into()));
    }
    match payload[1] {
        0 => {
            if payload.len() != 5 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("游学目标选择无效".into()));
            }
            let source = PlayerId::new(payload[3]);
            let target = PlayerId::new(selections[0]);
            if ddd_guanning_mark_holder(tx.state()) != Some(source)
                || source == target
                || !tx.state().players.get(target).is_some_and(|participant| {
                    participant.alive
                        && !participant
                            .marks
                            .iter()
                            .any(|(mark, count)| *mark == DDD_YOUXUE_MARK && *count > 0)
                })
            {
                return Err(EffectError::Illegal("游学标记或目标已变化".into()));
            }
            let draw = ddd_guanning_distance(tx.state(), source, target);
            let mut effects = vec![
                EffectRequest::RemoveMark {
                    player: source,
                    mark: engine_api::MarkDefId::new(DDD_YOUXUE_MARK),
                    amount: 0,
                },
                EffectRequest::AddMark {
                    player: target,
                    mark: engine_api::MarkDefId::new(DDD_YOUXUE_MARK),
                    delta: 1,
                },
            ];
            if draw > 0 {
                effects.push(EffectRequest::DrawCards {
                    player,
                    count: draw,
                });
            }
            run_effects(tx, effects)?;
        }
        1 => {
            if payload.len() < 8 || cancelled {
                return Err(EffectError::Illegal("承经目标选择无效".into()));
            }
            let card = CardId::new(payload[3]);
            let kind = CardKind::from_tag(payload[4] as u8)
                .ok_or_else(|| EffectError::Illegal("承经牌名无效".into()))?;
            let skill = engine_api::SkillId::new(payload[5]);
            let min = payload[6] as usize;
            let max = payload[7] as usize;
            let offered: Vec<_> = payload[8..].iter().copied().map(PlayerId::new).collect();
            let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            let mut distinct = std::collections::BTreeSet::new();
            let crate::command::play::SkillCardTargetSpec::Select {
                min: current_min,
                max: current_max,
                options,
            } = ddd_guanning_target_spec(tx, player, card, kind, skill)?
            else {
                return Err(EffectError::Illegal("承经目标形态已变化".into()));
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
                return Err(EffectError::Illegal("承经目标已失效".into()));
            }
            ddd_guanning_play(tx, player, card, kind, &selected, skill)?;
        }
        2 => {
            if payload.len() < 7 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("承经借刀选择无效".into()));
            }
            let card = CardId::new(payload[3]);
            let kind = CardKind::from_tag(payload[4] as u8)
                .ok_or_else(|| EffectError::Illegal("承经借刀牌名无效".into()))?;
            let skill = engine_api::SkillId::new(payload[5]);
            let pair_count = payload[6] as usize;
            if payload.len() != 7 + pair_count.saturating_mul(2)
                || selections[0] as usize >= pair_count
            {
                return Err(EffectError::Illegal("承经借刀目标组合无效".into()));
            }
            let index = 7 + selections[0] as usize * 2;
            let selected = (
                PlayerId::new(payload[index]),
                PlayerId::new(payload[index + 1]),
            );
            let crate::command::play::SkillCardTargetSpec::Pairs(current) =
                ddd_guanning_target_spec(tx, player, card, kind, skill)?
            else {
                return Err(EffectError::Illegal("承经借刀目标形态已变化".into()));
            };
            if !current.contains(&selected) {
                return Err(EffectError::Illegal("承经借刀目标已失效".into()));
            }
            ddd_guanning_play(tx, player, card, kind, &[selected.0, selected.1], skill)?;
        }
        3 => {
            if payload.len() != 6 {
                return Err(EffectError::EngineFault("truncated 承经赠牌 choice".into()));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal("取消承经赠牌时不得选择".into()));
                }
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal("承经赠牌选择无效".into()));
            }
            let card = CardId::new(payload[3]);
            let target = PlayerId::new(payload[4]);
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("承经受赠者没有手牌区".into()))?;
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
                || !tx
                    .state()
                    .cards
                    .get(card)
                    .is_some_and(|source| source.zone == discard)
            {
                return Err(EffectError::Illegal("承经赠牌目标或牌已失效".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::MoveCardsWithReason {
                    cards: vec![card],
                    to: hand,
                    from: Some(discard),
                    reason: CardMoveReason::Give,
                }],
            )?;
        }
        _ => return Err(EffectError::EngineFault("unknown 管宁 choice".into())),
    }
    Ok(())
}

pub(crate) const RESOLVE_DDD_CAOSHUANG_CHOICE: u32 = 0xDCDC_6350;

pub(crate) fn ddd_caoshuang_card_name(
    state: &crate::state::GameState,
    actor: PlayerId,
    card: CardId,
) -> Option<(u8, u32)> {
    let card_state = state.cards.get(card)?;
    let kind = crate::query::effective_card_kind(state, actor, card_state);
    Some((
        kind.tag(),
        (kind == CardKind::Other)
            .then(|| card_state.def.raw())
            .unwrap_or(0),
    ))
}

pub(crate) fn resolve_ddd_caoshuang(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            if actor == player
                || tx.state().turn.current != actor
                || tx.state().turn.phase != Phase::Play
            {
                return Ok(Vec::new());
            }
            let cards = tx.state().hand_cards(player);
            if cards.is_empty() {
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
                vec![
                    RESOLVE_DDD_CAOSHUANG_CHOICE,
                    0,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            let Some((kind, def)) = ddd_caoshuang_card_name(tx.state(), actor, card) else {
                return Ok(Vec::new());
            };
            let matched = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .is_some_and(|instance| instance.mark_ddd_zhuanshe_used(actor, kind, def));
            if !matched {
                return Ok(Vec::new());
            }
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    crate::command::play::saved_card_extra_target_is_legal_ignoring_distance(
                        tx, actor, card, *target,
                    )
                })
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_CAOSHUANG_CHOICE,
                    1,
                    player.raw(),
                    actor.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            let unused = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| {
                    instance
                        .ddd_zhuanshe_state()
                        .filter(|(stored_actor, _)| *stored_actor == actor)
                })
                .is_some_and(|(_, names)| names.iter().any(|(_, _, used)| !*used));
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.clear_ddd_zhuanshe_state();
            }
            if !unused {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                vec![1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_CAOSHUANG_CHOICE,
                    2,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        3 => {
            let recovery_id = tx.state().resolution.current_recovery_id;
            if recovery_id == 0 || !tx.state().hand_cards(player).is_empty() {
                return Ok(Vec::new());
            }
            let mut events = apply_effect(tx, EffectRequest::PreventRecovery { recovery_id })?;
            events.extend(apply_effect(
                tx,
                EffectRequest::DrawCards { player, count: 1 },
            )?);
            Ok(events)
        }
        _ => Err(EffectError::EngineFault("unknown 3D精选曹爽 action".into())),
    }
}

pub(crate) fn resolve_ddd_caoshuang_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || payload.first() != Some(&RESOLVE_DDD_CAOSHUANG_CHOICE) {
        return Err(EffectError::EngineFault("truncated 曹爽 choice".into()));
    }
    let player = PlayerId::new(payload[2]);
    if observer != player {
        return Err(EffectError::Illegal("曹爽选择者变化".into()));
    }
    match payload[1] {
        0 => {
            if payload.len() != 5 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("专摄赠牌选择无效".into()));
            }
            let actor = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let card = CardId::new(selections[0]);
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if actor == player
                || !tx
                    .state()
                    .players
                    .get(actor)
                    .is_some_and(|target| target.alive)
                || !tx
                    .state()
                    .cards
                    .get(card)
                    .is_some_and(|state| Some(state.zone) == hand)
            {
                return Err(EffectError::Illegal("专摄赠牌上下文已变化".into()));
            }
            let card_state = tx.state().cards.get(card).unwrap();
            let kind = card_state.kind.tag();
            let def = (card_state.kind == CardKind::Other)
                .then(|| card_state.def.raw())
                .unwrap_or(0);
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, actor)
                .ok_or_else(|| EffectError::Illegal("专摄目标没有手牌区".into()))?;
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
                        from: hand.unwrap(),
                    },
                ],
            )?;
            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("专摄技能已失效".into()))?;
            instance.record_ddd_zhuanshe_name(actor, kind, def);
        }
        1 => {
            if payload.len() != 6 {
                return Err(EffectError::EngineFault(
                    "truncated 专摄加目标 choice".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("专摄须选择一个额外目标".into()));
            }
            crate::command::play::add_saved_card_target_ignoring_distance(
                tx,
                PlayerId::new(payload[3]),
                CardId::new(payload[4]),
                PlayerId::new(selections[0]),
            )?;
        }
        2 => {
            if payload.len() != 5 {
                return Err(EffectError::EngineFault("truncated 专摄伤害 choice".into()));
            }
            if !cancelled && selections == [1] {
                run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: player,
                        target: PlayerId::new(payload[3]),
                        amount: 1,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                )?;
            }
        }
        _ => return Err(EffectError::EngineFault("unknown 曹爽 choice stage".into())),
    }
    Ok(())
}

pub(crate) const RESOLVE_DDD_XUELINGYUN_CHOICE: u32 = 0xDCDC_6360;

pub(crate) fn resolve_ddd_xuelingyun(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    card: CardId,
    cards: Vec<CardId>,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("涟洏没有手牌区".into()))?;
            let gained: Vec<_> = cards
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|card| Some(card.zone) == discard)
                })
                .collect();
            let ceiling = (gained.len() == 1)
                .then(|| tx.state().cards.get(gained[0]).map(|card| card.rank))
                .flatten()
                .filter(|rank| *rank > 0);
            let events = if gained.is_empty() {
                Vec::new()
            } else {
                move_cards_with_reason(tx, gained, hand, discard, CardMoveReason::Unspecified)?
            };
            if let Some(rank) = ceiling {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.mark = u32::from(rank);
                }
            }
            Ok(events)
        }
        1 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.mark = 0;
            }
            Ok(Vec::new())
        }
        2 => {
            if let Some(card_state) = tx.state().cards.get(card) {
                let black = crate::query::card_color_mask_for_suit(
                    crate::query::effective_card_suit(tx.state(), actor, card_state),
                ) == 0b10;
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.record_ddd_anzhi_color(black);
                }
            }
            Ok(Vec::new())
        }
        3 => {
            let targets = tx.state().players.alive_by_seat();
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_XUELINGYUN_CHOICE,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        4 => {
            crate::command::play::exclude_saved_card_target(tx, card, actor)?;
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.clear_ddd_anzhi_target();
            }
            Ok(Vec::new())
        }
        5 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                let (_, stored_actor, _) = instance.ddd_anzhi_state();
                if stored_actor == actor {
                    instance.clear_ddd_anzhi_target();
                }
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::EngineFault(
            "unknown 3D精选薛灵芸 action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_xuelingyun_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4
        || payload.first() != Some(&RESOLVE_DDD_XUELINGYUN_CHOICE)
        || observer != PlayerId::new(payload[1])
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("暗织目标选择无效".into()));
    }
    let target = PlayerId::new(selections[0]);
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|target| target.alive)
    {
        return Err(EffectError::Illegal("暗织目标已失效".into()));
    }
    let skill = engine_api::SkillId::new(payload[3]);
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::Illegal("暗织技能已失效".into()))?;
    instance.arm_ddd_anzhi(PlayerId::new(payload[2]), target);
    Ok(())
}

pub(crate) const RESOLVE_DDD_LIUHONG_CHOICE: u32 = 0xDCDD_6370;

pub(crate) fn ddd_shixing_female_available(state: &crate::state::GameState) -> bool {
    state.players.alive_by_seat().into_iter().any(|player| {
        state.players.get(player).is_some_and(|participant| {
            participant.hp >= i32::try_from(participant.max_hp).unwrap_or(i32::MAX)
                && state
                    .content
                    .character(participant.character_def)
                    .is_some_and(|character| matches!(character.sex.as_str(), "female" | "double"))
        })
    })
}

pub(crate) fn refresh_ddd_shixing(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    desired: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    const XIANGLE: u32 = 2_880_679_348;
    const JIUSHI: u32 = 4_293_489_189;
    const JIUSHI1: u32 = 3_647_393_232;
    const JIUSHI3: u32 = 4_156_803_269;
    const REZHIHENG: u32 = 2_455_708_599;

    let old = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.ddd_shixing_grants())
        .unwrap_or_default();
    let mut events = Vec::new();
    for derived in old {
        if tx.state().skills.get(derived).is_some() {
            events.extend(remove_skill(tx, derived)?);
        }
    }
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.mark = desired;
        instance.set_ddd_shixing_grants(&[]);
    }

    let mut defs = Vec::new();
    if desired & 1 != 0 {
        defs.push(XIANGLE);
    }
    if desired & 2 != 0 {
        defs.extend([JIUSHI, JIUSHI1, JIUSHI3]);
    }
    if desired == 0 {
        defs.push(REZHIHENG);
    }
    let mut grants = Vec::new();
    for raw in defs {
        let def = engine_api::SkillDefId::new(raw);
        if tx.state().content.skill_metadata(def).is_none() {
            return Err(EffectError::ContentFault(format!(
                "失兴依赖技能 {raw} 未加载"
            )));
        }
        let already_owned = tx.state().players.get(player).is_some_and(|owner| {
            owner.skills.iter().any(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def == def)
            })
        });
        if !already_owned {
            let (derived, added) = grant_skill_instance(tx, player, def, Vec::new())?;
            grants.push(derived);
            events.extend(added);
        }
    }
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.set_ddd_shixing_grants(&grants);
    }
    Ok(events)
}

pub(crate) fn resolve_ddd_liuhong(
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
            let desired = u32::from(ddd_shixing_female_available(tx.state())) | 2;
            refresh_ddd_shixing(tx, player, skill, desired)
        }
        1 => {
            let trick = tx
                .state()
                .skills
                .get(skill)
                .map_or(0, |instance| instance.mark & 2);
            let desired = trick | u32::from(ddd_shixing_female_available(tx.state()));
            refresh_ddd_shixing(tx, player, skill, desired)
        }
        2 => {
            let desired = tx
                .state()
                .skills
                .get(skill)
                .map_or(0, |instance| instance.mark & 1);
            refresh_ddd_shixing(tx, player, skill, desired)
        }
        3 => {
            let alive = tx.state().players.alive_by_seat();
            let linked = alive
                .iter()
                .filter(|target| {
                    tx.state().players.get(**target).is_some_and(|participant| {
                        participant
                            .marks
                            .iter()
                            .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                    })
                })
                .count();
            let qun = alive
                .iter()
                .filter(|target| tx.state().player_group(**target) == Some("qun"))
                .count();
            let minimum = qun.saturating_sub(linked);
            let options: Vec<_> = alive
                .into_iter()
                .filter(|target| {
                    tx.state().players.get(*target).is_some_and(|participant| {
                        !participant
                            .marks
                            .iter()
                            .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                    })
                })
                .map(PlayerId::raw)
                .collect();
            if minimum == 0 || options.len() < minimum {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                u8::try_from(minimum).unwrap_or(u8::MAX),
                u8::try_from(options.len()).unwrap_or(u8::MAX),
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_LIUHONG_CHOICE,
                    0,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        4 => {
            let count = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| tx.state().player_group(*target) == Some("qun"))
                .count()
                .min(u32::MAX as usize) as u32;
            run_effects(tx, vec![EffectRequest::DrawCards { player, count }])
        }
        5 => {
            let options = crate::query::discardable_target_cards(
                tx.state(),
                actor,
                player,
                rule_api::TargetCardScope::HandEquip,
            );
            if options.is_empty() {
                return Ok(Vec::new());
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
                    RESOLVE_DDD_LIUHONG_CHOICE,
                    1,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        _ => Err(EffectError::EngineFault("unknown 3D精选刘宏 action".into())),
    }
}

pub(crate) fn resolve_ddd_liuhong_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 || payload.first() != Some(&RESOLVE_DDD_LIUHONG_CHOICE) || cancelled {
        return Err(EffectError::Illegal("刘宏选择无效".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let actor = PlayerId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    if !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal("刘宏技能已失效".into()));
    }
    match stage {
        0 => {
            if observer != player {
                return Err(EffectError::Illegal("仅刘宏可选择党锢目标".into()));
            }
            let alive = tx.state().players.alive_by_seat();
            let linked = alive
                .iter()
                .filter(|target| {
                    tx.state().players.get(**target).is_some_and(|participant| {
                        participant
                            .marks
                            .iter()
                            .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                    })
                })
                .count();
            let minimum = alive
                .iter()
                .filter(|target| tx.state().player_group(**target) == Some("qun"))
                .count()
                .saturating_sub(linked);
            let mut distinct = std::collections::BTreeSet::new();
            let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            if targets.len() < minimum
                || targets.iter().any(|target| {
                    !distinct.insert(*target)
                        || !alive.contains(target)
                        || tx.state().players.get(*target).is_none_or(|participant| {
                            participant.marks.iter().any(|(mark, count)| {
                                *mark == crate::content::MARK_CHAIN && *count > 0
                            })
                        })
                })
            {
                return Err(EffectError::Illegal("党锢目标已变化".into()));
            }
            run_effects(
                tx,
                targets
                    .into_iter()
                    .map(|target| EffectRequest::AddMark {
                        player: target,
                        mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
                        delta: 1,
                    })
                    .collect(),
            )?;
            Ok(())
        }
        1 => {
            if observer != actor || selections.len() != 1 {
                return Err(EffectError::Illegal("仅死亡角色可选择覆宗弃牌".into()));
            }
            let card = CardId::new(selections[0]);
            if tx.state().player_group(actor) != Some("qun")
                || !crate::query::discardable_target_cards(
                    tx.state(),
                    actor,
                    player,
                    rule_api::TargetCardScope::HandEquip,
                )
                .contains(&card)
            {
                return Err(EffectError::Illegal("覆宗弃牌目标已变化".into()));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("覆宗牌已失效".into()))?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("弃牌堆不存在".into()))?;
            move_cards_with_reason(tx, vec![card], discard, Some(from), CardMoveReason::Discard)?;
            Ok(())
        }
        _ => Err(EffectError::Illegal("未知刘宏选择阶段".into())),
    }
}

pub(crate) const RESOLVE_DDD_XIAHOUXUAN_CHOICE: u32 = 0xDCDD_6380;
pub(crate) const DDD_LANGHUAI_DEF: u32 = 4_025_119_557;
pub(crate) const DDD_XUANLUN_DEF: u32 = 3_478_177_951;
pub(crate) const DDD_XUANLUN_PENDING: u32 = 1;
pub(crate) const DDD_XUANLUN_FORCE_LANGHUAI: u32 = 2;

pub(crate) fn ddd_xiahouxuan_hand_equip_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let mut cards = tx.state().hand_cards(player);
    if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, player) {
        cards.extend(
            tx.state()
                .zones
                .get(equip)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().copied()),
        );
    }
    cards
}

pub(crate) fn commit_ddd_langhuai(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx
            .state()
            .skills
            .get(skill)
            .is_none_or(|instance| instance.def.raw() != DDD_LANGHUAI_DEF)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Draw
        || tx.state().turn.draw_phase_fixed_count().is_some()
    {
        return Ok(Vec::new());
    }

    let hand = tx.state().hand_cards(player);
    let mut suits = 0u8;
    for card in &hand {
        let Some(card) = tx.state().cards.get(*card) else {
            continue;
        };
        suits |= match crate::query::effective_card_suit(tx.state(), player, card) {
            Suit::Spade => 1,
            Suit::Heart => 2,
            Suit::Club => 4,
            Suit::Diamond => 8,
            Suit::None => 0,
        };
    }
    let branch = tx
        .state()
        .skills
        .get(skill)
        .map_or(0, |instance| instance.mark & 1);
    let distinct = suits.count_ones();
    let count = if branch == 0 { distinct } else { 4 - distinct };
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.mark ^= 1;
    }
    if let Some(xuanlun) =
        skill_instance_by_def(tx, player, engine_api::SkillDefId::new(DDD_XUANLUN_DEF))
    {
        if let Some(instance) = tx.state_mut().skills.get_mut(xuanlun) {
            instance.mark &= !DDD_XUANLUN_FORCE_LANGHUAI;
        }
    }
    tx.state_mut().turn.set_draw_phase_fixed_count(0);

    let mut events = Vec::new();
    if !hand.is_empty() {
        events.push(DomainEvent::CardsRevealed {
            cards: hand,
            to: Vec::new(),
        });
    }
    events.extend(draw_cards(tx, player, count)?);
    Ok(events)
}

pub(crate) fn open_ddd_xuanlun_card_choice(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = ddd_xiahouxuan_hand_equip_cards(tx, player);
    let count = options.len().min(4);
    if count == 0 {
        return Ok(Vec::new());
    }
    if options.len() <= 4 {
        let mut tokens: Vec<_> = options.iter().map(|card| card.raw()).collect();
        tokens.push(0);
        let mut payload = vec![
            RESOLVE_DDD_XIAHOUXUAN_CHOICE,
            3,
            player.raw(),
            skill.raw(),
            count as u32,
        ];
        payload.extend(options.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            (count + 1) as u8,
            (count + 1) as u8,
            false,
            tokens,
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            payload,
        );
    }
    let mut payload = vec![
        RESOLVE_DDD_XIAHOUXUAN_CHOICE,
        2,
        player.raw(),
        skill.raw(),
        count as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        count as u8,
        count as u8,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn commit_ddd_xuanlun_arrangement(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let encoded = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.state_blob.clone())
        .unwrap_or_default();
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob.clear();
    }
    if encoded.is_empty() || encoded.len() % 4 != 0 {
        return Err(EffectError::Illegal("玄论牌堆排列已失效".into()));
    }
    let order: Vec<u32> = encoded
        .chunks_exact(4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
        .collect();
    let Some(split) = order.iter().position(|token| *token == 0) else {
        return Err(EffectError::Illegal("玄论牌堆排列缺少顶底分界".into()));
    };
    if order.iter().filter(|token| **token == 0).count() != 1 {
        return Err(EffectError::Illegal("玄论牌堆排列分界无效".into()));
    }
    let cards: Vec<_> = order
        .iter()
        .copied()
        .filter(|token| *token != 0)
        .map(CardId::new)
        .collect();
    let current = ddd_xiahouxuan_hand_equip_cards(tx, player);
    let mut unique = std::collections::BTreeSet::new();
    if cards.is_empty()
        || cards
            .iter()
            .any(|card| !unique.insert(*card) || !current.contains(card))
    {
        return Err(EffectError::Illegal("玄论牌堆排列中的牌已失效".into()));
    }

    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("玄论找不到牌堆".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("玄论找不到手牌区".into()))?;
    let equip = tx.state().zones.find(ZoneKind::Equip, player);
    let hand_cards: Vec<_> = cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.zone == hand)
        })
        .collect();
    let equip_cards: Vec<_> = cards
        .iter()
        .copied()
        .filter(|card| {
            equip.is_some_and(|equip| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.zone == equip)
            })
        })
        .collect();
    if hand_cards.len() + equip_cards.len() != cards.len() {
        return Err(EffectError::Illegal(
            "玄论只能排列自己的手牌或装备牌".into(),
        ));
    }

    let top: Vec<_> = order[..split].iter().copied().map(CardId::new).collect();
    let bottom: Vec<_> = order[split + 1..]
        .iter()
        .copied()
        .map(CardId::new)
        .collect();
    let mut events = vec![DomainEvent::CardsRevealed {
        cards: cards.clone(),
        to: Vec::new(),
    }];
    if !hand_cards.is_empty() {
        events.extend(move_cards(tx, hand_cards, draw, Some(hand))?);
    }
    if !equip_cards.is_empty() {
        events.extend(move_cards(
            tx,
            equip_cards,
            draw,
            Some(equip.expect("non-empty equipment selection has an equipment zone")),
        )?);
    }
    let mut middle = tx
        .state()
        .zones
        .get(draw)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    middle.retain(|card| !unique.contains(card));
    let zone = tx
        .state_mut()
        .zones
        .get_mut(draw)
        .expect("validated draw pile remains live");
    zone.cards = bottom;
    zone.cards.extend(middle);
    zone.cards.extend(top.into_iter().rev());
    Ok(events)
}

pub(crate) fn resolve_ddd_xiahouxuan(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let forced =
                skill_instance_by_def(tx, player, engine_api::SkillDefId::new(DDD_XUANLUN_DEF))
                    .and_then(|xuanlun| tx.state().skills.get(xuanlun))
                    .is_some_and(|instance| instance.mark & DDD_XUANLUN_FORCE_LANGHUAI != 0);
            if forced {
                commit_ddd_langhuai(tx, player, skill)
            } else {
                open_choice(
                    tx,
                    player,
                    0,
                    1,
                    true,
                    vec![1],
                    ChoiceKind::Confirm,
                    ContinuationKind::Custom,
                    vec![RESOLVE_DDD_XIAHOUXUAN_CHOICE, 0, player.raw(), skill.raw()],
                )
            }
        }
        1 => {
            if tx
                .state()
                .skills
                .get(skill)
                .is_none_or(|instance| instance.def.raw() != DDD_XUANLUN_DEF)
            {
                return Err(EffectError::Illegal("玄论技能实例无效".into()));
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.mark |= DDD_XUANLUN_PENDING;
            }
            draw_cards(tx, player, 4)
        }
        2 => {
            let mark = tx
                .state()
                .skills
                .get(skill)
                .filter(|instance| instance.def.raw() == DDD_XUANLUN_DEF)
                .map_or(0, |instance| instance.mark);
            if mark & DDD_XUANLUN_PENDING == 0 {
                return Ok(Vec::new());
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.mark &= !DDD_XUANLUN_PENDING;
            }
            let forced = mark & DDD_XUANLUN_FORCE_LANGHUAI != 0;
            if forced && tx.state().hand_cards(player).is_empty() {
                return Ok(Vec::new());
            }
            let card_count = ddd_xiahouxuan_hand_equip_cards(tx, player).len();
            let can_arrange = forced || card_count >= 4;
            if forced {
                return open_ddd_xuanlun_card_choice(tx, player, skill);
            }
            if !can_arrange {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.mark |= DDD_XUANLUN_FORCE_LANGHUAI;
                }
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_XIAHOUXUAN_CHOICE, 1, player.raw(), skill.raw()],
            )
        }
        3 => commit_ddd_xuanlun_arrangement(tx, player, skill),
        4 => commit_ddd_langhuai(tx, player, skill),
        _ => Err(EffectError::EngineFault(
            "unknown 3D精选夏侯玄 action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_xiahouxuan_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4
        || payload.first() != Some(&RESOLVE_DDD_XIAHOUXUAN_CHOICE)
        || observer != PlayerId::new(payload[2])
    {
        return Err(EffectError::Illegal("夏侯玄选择无效".into()));
    }
    let player = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal("夏侯玄技能已失效".into()));
    }
    match payload[1] {
        0 => {
            if payload.len() != 4 {
                return Err(EffectError::Illegal("朗怀确认数据无效".into()));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal("朗怀确认选择无效".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddXiahouxuan {
                    player,
                    action: 4,
                    skill,
                }],
            )?;
            Ok(())
        }
        1 => {
            if payload.len() != 4 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("玄论分支选择无效".into()));
            }
            match selections[0] {
                1 => {
                    open_ddd_xuanlun_card_choice(tx, player, skill)?;
                }
                2 => {
                    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                        instance.mark |= DDD_XUANLUN_FORCE_LANGHUAI;
                    }
                }
                _ => return Err(EffectError::Illegal("玄论分支不存在".into())),
            }
            Ok(())
        }
        2 => {
            if payload.len() < 5 {
                return Err(EffectError::Illegal("玄论选牌数据无效".into()));
            }
            let count = usize::try_from(payload[4])
                .map_err(|_| EffectError::Illegal("玄论选牌数量无效".into()))?;
            if cancelled
                || selections.len() != count
                || payload.len() != 5 + ddd_xiahouxuan_hand_equip_cards(tx, player).len()
            {
                return Err(EffectError::Illegal("玄论选牌无效".into()));
            }
            let offered = &payload[5..];
            let current = ddd_xiahouxuan_hand_equip_cards(tx, player);
            let mut unique = std::collections::BTreeSet::new();
            if selections.iter().any(|card| {
                !unique.insert(*card)
                    || !offered.contains(card)
                    || !current.contains(&CardId::new(*card))
            }) {
                return Err(EffectError::Illegal("玄论所选牌已失效".into()));
            }
            let mut options = selections.to_vec();
            options.push(0);
            let mut next = vec![
                RESOLVE_DDD_XIAHOUXUAN_CHOICE,
                3,
                player.raw(),
                skill.raw(),
                count as u32,
            ];
            next.extend(selections);
            open_choice(
                tx,
                player,
                (count + 1) as u8,
                (count + 1) as u8,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                next,
            )?;
            Ok(())
        }
        3 => {
            if payload.len() < 5 {
                return Err(EffectError::Illegal("玄论排列数据无效".into()));
            }
            let count = usize::try_from(payload[4])
                .map_err(|_| EffectError::Illegal("玄论排列数量无效".into()))?;
            if payload.len() != 5 + count
                || cancelled
                || selections.len() != count + 1
                || selections.iter().filter(|token| **token == 0).count() != 1
            {
                return Err(EffectError::Illegal("玄论牌堆排列无效".into()));
            }
            let expected: std::collections::BTreeSet<_> = payload[5..].iter().copied().collect();
            let actual: std::collections::BTreeSet<_> = selections
                .iter()
                .copied()
                .filter(|token| *token != 0)
                .collect();
            if expected.len() != count || actual != expected {
                return Err(EffectError::Illegal("玄论牌堆排列内容无效".into()));
            }
            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("玄论技能已失效".into()))?;
            instance.state_blob.clear();
            for token in selections {
                instance.state_blob.extend_from_slice(&token.to_le_bytes());
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddXiahouxuan {
                    player,
                    action: 3,
                    skill,
                }],
            )?;
            Ok(())
        }
        _ => Err(EffectError::Illegal("未知夏侯玄选择阶段".into())),
    }
}

pub(crate) const RESOLVE_DDD_ZHANGKAI_CHOICE: u32 = 0xDCDD_6390;
pub(crate) const DDD_JIEXING_DEF: u32 = 3_377_566_778;
pub(crate) const DDD_BAILEI_DEF: u32 = 2_689_073_031;
pub(crate) const DDD_JIEXING_IGNORE_STOLEN_RECOVERY: u64 = 1;

pub(crate) fn ddd_zhangkai_unique_max_equip(tx: &Transaction) -> Option<PlayerId> {
    let counts: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|player| {
            let count = tx
                .state()
                .zones
                .find(ZoneKind::Equip, player)
                .and_then(|zone| tx.state().zones.get(zone))
                .map_or(0, |zone| zone.cards.len());
            (player, count)
        })
        .collect();
    let max = counts.iter().map(|(_, count)| *count).max()?;
    let mut leaders = counts.into_iter().filter(|(_, count)| *count == max);
    let leader = leaders.next()?.0;
    leaders.next().is_none().then_some(leader)
}

pub(crate) fn ddd_zhangkai_gainable(tx: &Transaction, player: PlayerId, target: PlayerId) -> Vec<CardId> {
    crate::query::gainable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquip,
    )
}

pub(crate) fn ddd_zhangkai_equip_materials(
    tx: &Transaction,
    player: PlayerId,
    actor: PlayerId,
    cards: &[CardId],
) -> Vec<CardId> {
    let processing = tx.state().zones.find(ZoneKind::Processing, PlayerId::NONE);
    let actor_hand = tx.state().zones.find(ZoneKind::Hand, actor);
    cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card| {
                (Some(card.zone) == processing || Some(card.zone) == actor_hand)
                    && card.kind.equip_slot().is_some_and(|slot| {
                        tx.state()
                            .players
                            .get(player)
                            .is_some_and(|owner| owner.equip_slot_enabled(slot))
                    })
            })
        })
        .collect()
}

pub(crate) fn ddd_zhangkai_equip(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, player)
        .ok_or_else(|| EffectError::EngineFault("劫行找不到装备区".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("劫行找不到弃牌堆".into()))?;
    let mut events = Vec::new();
    for card in cards {
        let Some(card_state) = tx.state().cards.get(card).cloned() else {
            continue;
        };
        let Some(slot) = card_state.kind.equip_slot() else {
            continue;
        };
        if !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.equip_slot_enabled(slot))
        {
            continue;
        }
        let replaced: Vec<_> = tx
            .state()
            .zones
            .get(equip)
            .into_iter()
            .flat_map(|zone| zone.cards.iter().copied())
            .filter(|existing| {
                tx.state()
                    .cards
                    .get(*existing)
                    .and_then(|card| card.kind.equip_slot())
                    == Some(slot)
            })
            .collect();
        if !replaced.is_empty() {
            events.extend(move_cards_with_reason(
                tx,
                replaced,
                discard,
                Some(equip),
                CardMoveReason::Discard,
            )?);
        }
        events.extend(move_cards_with_reason(
            tx,
            vec![card],
            equip,
            Some(card_state.zone),
            CardMoveReason::Use,
        )?);
        events.push(DomainEvent::RuleNote {
            code: 50,
            payload: vec![player.raw(), card.raw()],
        });
    }
    Ok(events)
}
