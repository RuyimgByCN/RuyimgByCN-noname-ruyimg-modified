//! Effect primitives for `ddd_liuba_decode_distribution` and related helpers.

use super::*;

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

pub(crate) fn ddd_guanning_distance(
    state: &crate::state::GameState,
    from: PlayerId,
    to: PlayerId,
) -> u32 {
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
