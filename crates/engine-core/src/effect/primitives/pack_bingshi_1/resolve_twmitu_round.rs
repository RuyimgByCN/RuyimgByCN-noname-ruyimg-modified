//! Effect primitives for `resolve_twmitu_round` and related helpers.

use super::*;

/// 拼点轮各阶段确认（resolver 调用）。返回事件（虚拟杀等）。
pub(crate) fn resolve_twmitu_round(
    tx: &mut Transaction,
    observer: PlayerId,
    player: PlayerId,
    skill: engine_api::SkillId,
    payload: &[u32],
    selections: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    const HEADER: usize = 6;
    if payload.len() < HEADER || payload[0] != RESOLVE_TWMITU_MAGIC {
        return Err(EffectError::EngineFault(
            "bad resolve-twmitu round payload".into(),
        ));
    }
    let stage = payload[3];
    let k = payload[4] as usize;
    let done = payload[5] as usize;
    if k == 0 || done >= k {
        return Err(EffectError::EngineFault(
            "resolve-twmitu round index out of range".into(),
        ));
    }
    let targets: Vec<u32> = payload[HEADER..HEADER + k].to_vec();
    let shown: Vec<u32> = payload[HEADER + k..HEADER + 2 * k].to_vec();
    let consents_off = HEADER + 2 * k;
    let d_opponents = match stage {
        3 => done,
        4..=6 => done + 1,
        _ => return Err(EffectError::EngineFault("bad twmitu stage".into())),
    };
    let d_consents = match stage {
        3..=4 => done,
        5..=6 => done + 1,
        _ => return Err(EffectError::EngineFault("bad twmitu stage".into())),
    };
    let opponents_end = consents_off + d_opponents;
    let consents_end = opponents_end + d_consents;
    if payload.len() < consents_end {
        return Err(EffectError::EngineFault(
            "bad resolve-twmitu round payload layout".into(),
        ));
    }
    let opponents: Vec<u32> = payload[consents_off..opponents_end].to_vec();
    let consents: Vec<u32> = payload[opponents_end..consents_end].to_vec();
    let pend: Vec<u32> = payload[consents_end..].to_vec();
    let _ = &shown;
    let target_raw = targets[done];
    let target = PlayerId::new(target_raw);

    match stage {
        3 => {
            // owner 指定对手：须存活、有手牌、非该目标。
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-twmitu opponent pick requires owner".into(),
                ));
            }
            let opponent_raw = *selections
                .first()
                .ok_or_else(|| EffectError::Illegal("resolve-twmitu opponent required".into()))?;
            let opponent = PlayerId::new(opponent_raw);
            if opponent == target
                || !tx.state().players.get(opponent).is_some_and(|participant| {
                    participant.alive && !tx.state().hand_cards(opponent).is_empty()
                })
            {
                return Err(EffectError::Illegal(
                    "resolve-twmitu invalid opponent".into(),
                ));
            }
            let mut opponents2 = opponents;
            opponents2.push(opponent_raw);
            // stage 4：目标知情（1=同意 2=拒绝）。
            let payload = twmitu_round_payload(
                player,
                skill,
                4,
                &targets,
                &shown,
                done,
                &opponents2,
                &consents,
                &[],
            );
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwmituStage,
                payload,
            )?;
            Ok(Vec::new())
        }
        4 => {
            if observer != target {
                return Err(EffectError::Illegal(
                    "resolve-twmitu consent requires the target".into(),
                ));
            }
            let consent = *selections
                .first()
                .ok_or_else(|| EffectError::Illegal("resolve-twmitu consent required".into()))?;
            let mut consents2 = consents;
            consents2.push(consent);
            if consent != 1 {
                // 拒绝拼点：轮次完成。
                return twmitu_advance(
                    tx,
                    player,
                    skill,
                    &targets,
                    &shown,
                    done + 1,
                    &opponents,
                    &consents2,
                    &[],
                );
            }
            // stage 5：目标选拼点牌。
            let hand: Vec<u32> = tx
                .state()
                .hand_cards(target)
                .iter()
                .map(|card| card.raw())
                .collect();
            if hand.is_empty() {
                return Err(EffectError::Illegal(
                    "resolve-twmitu target has no hand card".into(),
                ));
            }
            let payload = twmitu_round_payload(
                player,
                skill,
                5,
                &targets,
                &shown,
                done,
                &opponents,
                &consents2,
                &[],
            );
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                hand,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwmituStage,
                payload,
            )?;
            Ok(Vec::new())
        }
        5 => {
            if observer != target {
                return Err(EffectError::Illegal(
                    "resolve-twmitu compare card requires the target".into(),
                ));
            }
            let card_raw = *selections.first().ok_or_else(|| {
                EffectError::Illegal("resolve-twmitu compare card required".into())
            })?;
            let target_hand = tx.state().hand_cards(target);
            if !target_hand.iter().any(|card| card.raw() == card_raw) {
                return Err(EffectError::Illegal(
                    "resolve-twmitu compare card outside target hand".into(),
                ));
            }
            let payload = twmitu_round_payload(
                player,
                skill,
                6,
                &targets,
                &shown,
                done,
                &opponents,
                &consents,
                &[card_raw],
            );
            let opponent = PlayerId::new(opponents[done]);
            let ohand: Vec<u32> = tx
                .state()
                .hand_cards(opponent)
                .iter()
                .map(|card| card.raw())
                .collect();
            open_choice(
                tx,
                opponent,
                1,
                1,
                false,
                ohand,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwmituStage,
                payload,
            )?;
            Ok(Vec::new())
        }
        6 => {
            if observer != PlayerId::new(opponents[done]) {
                return Err(EffectError::Illegal(
                    "resolve-twmitu compare card requires the opponent".into(),
                ));
            }
            if pend.len() != 1 {
                return Err(EffectError::EngineFault(
                    "resolve-twmitu missing target compare card".into(),
                ));
            }
            let t_card = CardId::new(pend[0]);
            let o_card = CardId::new(*selections.first().ok_or_else(|| {
                EffectError::Illegal("resolve-twmitu compare card required".into())
            })?);
            let opponent = PlayerId::new(opponents[done]);
            let opponent_hand = tx.state().hand_cards(opponent);
            if !opponent_hand.iter().any(|card| *card == o_card) {
                return Err(EffectError::Illegal(
                    "resolve-twmitu compare card outside opponent hand".into(),
                ));
            }
            let t_rank = tx.state().cards.get(t_card).map(|c| c.rank).unwrap_or(0);
            let o_rank = tx.state().cards.get(o_card).map(|c| c.rank).unwrap_or(0);
            let mut events: Vec<DomainEvent> = Vec::new();
            if t_rank != o_rank {
                let (winner, loser) = if t_rank > o_rank {
                    (target, opponent)
                } else {
                    (opponent, target)
                };
                // 赢家视为对输家使用虚拟【杀】：机内直调伤害结算（登记债：目标
                // 无应闪机会——可暂停的用牌管线无法在机内中段拼接）。
                events.extend(super::super::pack_mobile::deal_damage(
                    tx,
                    winner,
                    loser,
                    1,
                    false,
                    0,
                    false,
                    CardId::NONE,
                    0,
                    0,
                )?);
            }
            // slice 3 崩溃面：每个参与拼点的「迷途」目标，若其拼点用牌不是自己
            // 的展示牌（首张摸得牌），owner 减 1 点体力上限。
            let mut penalty = 0i32;
            for (participant_raw, used_card) in [(target_raw, t_card), (opponents[done], o_card)] {
                if let Some(idx) = targets.iter().position(|t| *t == participant_raw) {
                    let showed = shown.get(idx).copied().unwrap_or(0);
                    if showed != 0 && showed != used_card.raw() {
                        penalty += 1;
                    }
                }
            }
            if penalty > 0 {
                events.extend(super::super::change_max_hp(tx, player, -penalty)?);
            }
            twmitu_advance(
                tx,
                player,
                skill,
                &targets,
                &shown,
                done + 1,
                &opponents,
                &consents,
                &[],
            )?;
            Ok(events)
        }
        _ => Err(EffectError::EngineFault("bad twmitu stage".into())),
    }
}

/// 专属状态机 (批 252, ADR 0641 §8.42)：谦吞——出牌阶段限一次，令一名其他
/// 角色展示任意张手牌（其拼点只能用展示牌），与其拼点：你赢则获得其展示的
/// 手牌；没赢（含平点）则获得其其余手牌。
pub(crate) fn resolve_qiantun(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage != 0 {
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
    let targets: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != player)
        .filter(|&candidate| !tx.state().hand_cards(candidate).is_empty())
        .map(PlayerId::raw)
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
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveQiantunStage,
        vec![RESOLVE_QIANTUN_MAGIC, player.raw(), skill.raw(), 0],
    )
}

/// stage 2：目标展示任意张手牌（强制，≥1）。
pub(crate) fn resolve_qiantun_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target = PlayerId::new(target_raw);
    let hand: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .iter()
        .map(|card| card.raw())
        .collect();
    open_choice(
        tx,
        target,
        1,
        hand.len().min(u8::MAX as usize) as u8,
        false,
        hand,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveQiantunStage,
        vec![
            RESOLVE_QIANTUN_MAGIC,
            player.raw(),
            skill.raw(),
            2,
            target_raw,
        ],
    )
}

/// stage 3：展示已定 → owner 选自己的拼点牌（不受限）。
pub(crate) fn resolve_qiantun_shown(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
    shown: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let hand: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .iter()
        .map(|card| card.raw())
        .collect();
    let mut payload = vec![
        RESOLVE_QIANTUN_MAGIC,
        player.raw(),
        skill.raw(),
        3,
        target_raw,
        shown.len() as u32,
    ];
    payload.extend_from_slice(shown);
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        hand,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveQiantunStage,
        payload,
    )
}

/// stage 4：owner 拼点牌已定 → 目选拼点牌（手中有展示牌时只能从展示牌选）。
pub(crate) fn resolve_qiantun_player_card(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
    shown: &[u32],
    player_card: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target = PlayerId::new(target_raw);
    let target_hand: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .iter()
        .map(|card| card.raw())
        .collect();
    let shown_set: std::collections::BTreeSet<u32> = shown.iter().copied().collect();
    let has_shown_in_hand = target_hand.iter().any(|card| shown_set.contains(card));
    let options: Vec<u32> = if has_shown_in_hand {
        target_hand
            .iter()
            .copied()
            .filter(|card| shown_set.contains(card))
            .collect()
    } else {
        target_hand
    };
    let mut payload = vec![
        RESOLVE_QIANTUN_MAGIC,
        player.raw(),
        skill.raw(),
        4,
        target_raw,
        shown.len() as u32,
    ];
    payload.extend_from_slice(shown);
    payload.push(player_card);
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveQiantunStage,
        payload,
    )
}

pub(crate) fn resolve_qiantun_resolve(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
    shown: &[u32],
    player_card: u32,
    target_card: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let target = PlayerId::new(target_raw);
    let p_rank = tx
        .state()
        .cards
        .get(CardId::new(player_card))
        .map(|c| c.rank)
        .unwrap_or(0);
    let t_rank = tx
        .state()
        .cards
        .get(CardId::new(target_card))
        .map(|c| c.rank)
        .unwrap_or(0);
    let player_won = p_rank > t_rank;

    // 拼点牌置入弃牌堆（chooseToCompare 规则）。
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    for raw in [player_card, target_card] {
        let card = CardId::new(raw);
        if let Some(hand_zone) = {
            let holder = tx
                .state()
                .cards
                .get(card)
                .map(|card_state| card_state.owner)
                .unwrap_or(PlayerId::NONE);
            tx.state().zones.find(ZoneKind::Hand, holder)
        } {
            tx.state_mut()
                .zones
                .get_mut(hand_zone)
                .unwrap()
                .cards
                .retain(|c| *c != card);
        }
        tx.state_mut()
            .zones
            .get_mut(discard)
            .unwrap()
            .cards
            .push(card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = discard;
        }
    }

    let target_hand: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .iter()
        .map(|card| card.raw())
        .collect();
    let shown_set: std::collections::BTreeSet<u32> = shown.iter().copied().collect();
    let taken: Vec<u32> = if player_won {
        target_hand
            .iter()
            .copied()
            .filter(|card| shown_set.contains(card))
            .collect()
    } else {
        target_hand
            .iter()
            .copied()
            .filter(|card| !shown_set.contains(card))
            .collect()
    };
    let mut events = Vec::new();
    if taken.is_empty() {
        return Ok(events);
    }
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no target hand".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no player hand".into()))?;
    for raw in taken {
        let card = CardId::new(raw);
        tx.state_mut()
            .zones
            .get_mut(from)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        tx.state_mut().zones.get_mut(to).unwrap().cards.push(card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = to;
            card_state.owner = player;
        }
        events.push(engine_api::DomainEvent::CardsDrawn {
            player,
            cards: vec![card],
        });
    }
    Ok(events)
}

/// 专属状态机 (批 253, ADR 0641 §8.43)：养名——出牌阶段限一次，与一名角色
/// 拼点：目标赢 → 目标摸「已输次数」张 + owner 回复 1 点体力，结束；否则
/// 记一次输，owner 可选择重复此流程（目标无法再拼点时自动结束）。
pub(crate) fn resolve_yangming(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage != 0 {
        return Ok(Vec::new());
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().hand_cards(player).is_empty()
    {
        return Ok(Vec::new());
    }
    let targets: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != player && !tx.state().hand_cards(candidate).is_empty())
        .map(PlayerId::raw)
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
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveYangmingStage,
        vec![RESOLVE_YANGMING_MAGIC, player.raw(), skill.raw(), 0],
    )
}

/// 载荷：[magic, player, skill, stage, target, losses, pend...]
/// stage 2=owner 选拼点牌（pend 空）；stage 3=目选拼点牌（pend=[c_p]）；
/// stage 4=owner 知情（Custom 1=重复 2=结束）。
pub(crate) fn yangming_payload(
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    target_raw: u32,
    losses: u32,
    pend: &[u32],
) -> Vec<u32> {
    let mut payload = vec![
        RESOLVE_YANGMING_MAGIC,
        player.raw(),
        skill.raw(),
        stage,
        target_raw,
        losses,
    ];
    payload.extend_from_slice(pend);
    payload
}

/// 目标选择已确认（resolver 调用）：打开第一轮 owner 选拼点牌。
pub(crate) fn resolve_yangming_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
    losses: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let hand: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .iter()
        .map(|card| card.raw())
        .collect();
    let payload = yangming_payload(player, skill, 2, target_raw, losses, &[]);
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        hand,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveYangmingStage,
        payload,
    )?;
    Ok(Vec::new())
}

/// 两侧都能选拼点牌时才可继续。
pub(crate) fn yangming_can_compare(tx: &Transaction, player: PlayerId, target: PlayerId) -> bool {
    tx.state()
        .players
        .get(player)
        .is_some_and(|p| p.alive && !tx.state().hand_cards(player).is_empty())
        && tx
            .state()
            .players
            .get(target)
            .is_some_and(|p| p.alive && !tx.state().hand_cards(target).is_empty())
}
