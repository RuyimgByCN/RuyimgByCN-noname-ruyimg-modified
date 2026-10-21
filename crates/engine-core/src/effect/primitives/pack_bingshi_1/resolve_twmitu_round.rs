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

/// 拼点轮各阶段确认（resolver 调用）。
pub(crate) fn resolve_yangming_round(
    tx: &mut Transaction,
    observer: PlayerId,
    player: PlayerId,
    skill: engine_api::SkillId,
    payload: &[u32],
    selections: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    const HEADER: usize = 6;
    if payload.len() < HEADER || payload[0] != RESOLVE_YANGMING_MAGIC {
        return Err(EffectError::EngineFault(
            "bad resolve-yangming continuation payload".into(),
        ));
    }
    let stage = payload[3];
    let target_raw = payload[4];
    let losses = payload[5];
    let pend: Vec<u32> = payload[HEADER..].to_vec();
    let target = PlayerId::new(target_raw);
    match stage {
        2 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-yangming compare card requires owner".into(),
                ));
            }
            let c_p = *selections.first().ok_or_else(|| {
                EffectError::Illegal("resolve-yangming compare card required".into())
            })?;
            let owner_hand: std::collections::BTreeSet<u32> = tx
                .state()
                .hand_cards(player)
                .iter()
                .map(|card| card.raw())
                .collect();
            if !owner_hand.contains(&c_p) {
                return Err(EffectError::Illegal(
                    "resolve-yangming compare card outside owner hand".into(),
                ));
            }
            let t_hand: Vec<u32> = tx
                .state()
                .hand_cards(target)
                .iter()
                .map(|card| card.raw())
                .collect();
            let payload = yangming_payload(player, skill, 3, target_raw, losses, &[c_p]);
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                t_hand,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveYangmingStage,
                payload,
            )?;
            Ok(Vec::new())
        }
        3 => {
            if observer != target {
                return Err(EffectError::Illegal(
                    "resolve-yangming compare card requires the target".into(),
                ));
            }
            if pend.len() != 1 {
                return Err(EffectError::EngineFault(
                    "resolve-yangming missing owner compare card".into(),
                ));
            }
            let c_p = pend[0];
            let c_t = *selections.first().ok_or_else(|| {
                EffectError::Illegal("resolve-yangming compare card required".into())
            })?;
            let target_hand: std::collections::BTreeSet<u32> = tx
                .state()
                .hand_cards(target)
                .iter()
                .map(|card| card.raw())
                .collect();
            if !target_hand.contains(&c_t) {
                return Err(EffectError::Illegal(
                    "resolve-yangming compare card outside target hand".into(),
                ));
            }
            // 拼点牌置入弃牌堆（chooseToCompare 规则）。
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            let mut events = Vec::new();
            for (holder, raw) in [(player, c_p), (target, c_t)] {
                let card = CardId::new(raw);
                if let Some(hand_zone) = tx.state().zones.find(ZoneKind::Hand, holder) {
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
            let p_rank = tx
                .state()
                .cards
                .get(CardId::new(c_p))
                .map(|c| c.rank)
                .unwrap_or(0);
            let t_rank = tx
                .state()
                .cards
                .get(CardId::new(c_t))
                .map(|c| c.rank)
                .unwrap_or(0);
            if t_rank > p_rank {
                // 目标赢：摸已输次数张 + owner 回复 1，结束。
                if losses > 0 {
                    events.extend(crate::effect::draw_cards(tx, target, losses)?);
                }
                events.extend(super::super::recover_hp(tx, player, player, 1)?);
                return Ok(events);
            }
            // owner 赢或平：记一次输。
            let losses2 = losses + 1;
            if !yangming_can_compare(tx, player, target) {
                return Ok(events);
            }
            // stage 4：owner 选择是否重复。
            let payload = yangming_payload(player, skill, 4, target_raw, losses2, &[]);
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::ResolveYangmingStage,
                payload,
            )?;
            Ok(events)
        }
        4 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-yangming repeat requires owner".into(),
                ));
            }
            let repeat = *selections.first().ok_or_else(|| {
                EffectError::Illegal("resolve-yangming repeat choice required".into())
            })?;
            if repeat == 1 && yangming_can_compare(tx, player, target) {
                resolve_yangming_stage(tx, player, skill, target_raw, losses)?;
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::EngineFault(
            "bad resolve-yangming stage".into(),
        )),
    }
}

/// 装备区牌数。
pub(crate) fn canyun_equip_count(tx: &Transaction, player: PlayerId) -> usize {
    tx.state()
        .zones
        .find(ZoneKind::Equip, player)
        .map(|zone| tx.state().zones.get(zone).unwrap().cards.len())
        .unwrap_or(0)
}

/// 专属状态机 (批 255, ADR 0641 §8.44)：残殒——出牌阶段，可弃置一张牌并
/// 选择 1-2 名其他角色，比较装备数：owner 多则其回复 1 体力，相等则其摸
/// 1 张牌，其多则其失去 1 体力；目标体力为 1 时 owner 摸一张牌。
pub(crate) fn resolve_canyun(
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
        .filter(|&candidate| candidate != player)
        .map(PlayerId::raw)
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    // stage 0：弃置一张手牌（费用）。
    let hand: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .iter()
        .map(|card| card.raw())
        .collect();
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        hand,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveCanyunStage,
        vec![RESOLVE_CANYUN_MAGIC, player.raw(), skill.raw(), 2],
    )
}

/// stage 2：弃牌已定 → 选 1-2 名其他角色。
pub(crate) fn resolve_canyun_discarded(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    discard_card: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let targets: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != player)
        .map(PlayerId::raw)
        .collect();
    let mut payload = vec![
        RESOLVE_CANYUN_MAGIC,
        player.raw(),
        skill.raw(),
        3,
        discard_card,
    ];
    open_choice(
        tx,
        player,
        1,
        2,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveCanyunStage,
        payload,
    )
}

/// stage 3：目标已定 → 逐目标比较装备数结算；目标体力为 1 时 owner 摸 1。
pub(crate) fn resolve_canyun_resolve(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    discard_card: u32,
    targets: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let owner_equip = canyun_equip_count(tx, player) as i32;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let player_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no player hand".into()))?;
    let mut events = Vec::new();
    let mut owner_draws = 0i32;

    // 弃置费用牌。
    let cost = CardId::new(discard_card);
    tx.state_mut()
        .zones
        .get_mut(player_hand)
        .unwrap()
        .cards
        .retain(|c| *c != cost);
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(cost);
    if let Some(card_state) = tx.state_mut().cards.get_mut(cost) {
        card_state.zone = discard;
    }

    for raw in targets {
        let target = PlayerId::new(*raw);
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            continue;
        }
        let target_equip = canyun_equip_count(tx, target) as i32;
        if owner_equip > target_equip {
            events.extend(super::super::recover_hp(tx, player, target, 1)?);
        } else if owner_equip == target_equip {
            events.extend(crate::effect::draw_cards(tx, target, 1)?);
        } else {
            // 目标多：其失去 1 体力（体力流失通道）。
            events.extend(super::super::deal_damage(
                tx,
                PlayerId::NONE,
                target,
                1,
                true,
                0,
                false,
                CardId::NONE,
                0,
                0,
            )?);
        }
        if hp_of_player(tx, target) == 1 {
            owner_draws += 1;
        }
    }
    if owner_draws > 0 {
        events.extend(crate::effect::draw_cards(tx, player, owner_draws as u32)?);
    }
    Ok(events)
}

pub(crate) fn hp_of_player(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state()
        .players
        .get(player)
        .map(|participant| participant.hp)
        .unwrap_or(0)
}

/// 专属状态机 (批 256, ADR 0641 §8.45)：横骛——使用/打出牌后，若手牌无此
/// 花色且场上有同花色装备：摸「场上该花色装备数」张（frequent 锁定自动）。
pub(crate) fn resolve_hengwu(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    event_card: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    // 事件牌花色（无牌/无色 → 结束）。
    let Some(suit) = tx
        .state()
        .cards
        .get(CardId::new(event_card))
        .map(|card| card.suit)
        .filter(|suit| {
            matches!(
                suit,
                engine_api::Suit::Spade
                    | engine_api::Suit::Heart
                    | engine_api::Suit::Club
                    | engine_api::Suit::Diamond
            )
        })
    else {
        return Ok(Vec::new());
    };
    // 门控：手牌无此花色。
    if tx.state().hand_cards(player).iter().any(|card| {
        tx.state()
            .cards
            .get(*card)
            .is_some_and(|card_state| card_state.suit == suit)
    }) {
        return Ok(Vec::new());
    }
    // 场上同花色装备数。
    let mut count = 0usize;
    for zone in tx.state().zones.sorted_live() {
        if zone.kind != ZoneKind::Equip {
            continue;
        }
        for card in &zone.cards {
            if tx
                .state()
                .cards
                .get(*card)
                .is_some_and(|card_state| card_state.suit == suit)
            {
                count += 1;
            }
        }
    }
    if count == 0 {
        return Ok(Vec::new());
    }
    crate::effect::draw_cards(tx, player, count as u32)
}

/// 专属状态机 (批 257, ADR 0641 §8.46)：表召·归——准备阶段若「表」堆有牌：
/// 弃置首张，令一名角色（可选自己，可取消）摸至全场最大手牌数（至多 5）并
/// 回复 1 点体力。全场最大手牌数机内计算。
pub(crate) fn resolve_biaozhao3(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            // 门控：「表」堆非空（forced 触发体内由 DSL 门控保证，机内再校验）。
            let pile = tx.state().skill_pile_cards(player, BIAOZHAO_PILE_DEF);
            if pile.is_empty() {
                return Ok(Vec::new());
            }
            // 弃置首张「表」。
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            let card = pile[0];
            if let Some(pile_zone) = tx
                .state()
                .skill_piles
                .iter()
                .find(|p| p.owner == player && p.skill_def == BIAOZHAO_PILE_DEF)
                .map(|p| p.zone)
            {
                tx.state_mut()
                    .zones
                    .get_mut(pile_zone)
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
            // 全场最大手牌数（机内计算）。
            let mut num = 0usize;
            for participant in tx.state().players.alive_by_seat() {
                num = num.max(tx.state().hand_cards(participant).len());
            }
            let targets: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveBiaozhao3Stage,
                vec![
                    RESOLVE_BIAOZHAO3_MAGIC,
                    player.raw(),
                    skill.raw(),
                    2,
                    num as u32,
                ],
            )
        }
        _ => Ok(Vec::new()),
    }
}

/// 续体：selections[0] = 受益角色。摸至 num（至多补 5 张）并回复 1 点体力。
pub(crate) fn resolve_biaozhao3_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    num: u32,
    selected: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    if selected.is_empty() {
        return Ok(Vec::new());
    }
    let target = PlayerId::new(selected[0]);
    let mut events = Vec::new();
    if num > tx.state().hand_cards(target).len() as u32 {
        let draw = (num - tx.state().hand_cards(target).len() as u32).min(5);
        if draw > 0 {
            events.extend(crate::effect::draw_cards(tx, target, draw)?);
        }
    }
    events.extend(super::super::recover_hp(tx, player, target, 1)?);
    Ok(events)
}

/// 专属状态机 (批 258, ADR 0641 §8.47)：穿云——「传术」衍生技。杀指定目标后
/// 若其装备区有牌：随机弃置其中一张。
pub(crate) fn resolve_chuanyun(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    event_player: PlayerId,
    event_card: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    // 事件牌须为【杀】（普/火/雷）。
    let is_sha = tx
        .state()
        .cards
        .get(CardId::new(event_card))
        .is_some_and(|card| {
            matches!(
                card.kind,
                crate::state::CardKind::Sha
                    | crate::state::CardKind::ShaFire
                    | crate::state::CardKind::ShaThunder
            )
        });
    if !is_sha || event_player == PlayerId::NONE || event_player == player {
        return Ok(Vec::new());
    }
    // 目标装备区有牌：RNG 随机弃置一张。
    let Some(equip_zone) = tx.state().zones.find(ZoneKind::Equip, event_player) else {
        return Ok(Vec::new());
    };
    let equips = tx.state().zones.get(equip_zone).unwrap().cards.clone();
    if equips.is_empty() {
        return Ok(Vec::new());
    }
    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
    let idx = rng.gen_range_u32(equips.len() as u32) as usize;
    tx.state_mut().rng = rng.into_state();
    let card = equips[idx];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    tx.state_mut()
        .zones
        .get_mut(equip_zone)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(card);
    if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
        card_state.zone = discard;
    }
    Ok(Vec::new())
}

/// 专属状态机 (批 259, ADR 0641 §8.48)：表召·应——任意角色弃置与「表」
/// 同花色同点数的牌时（cards_moved 过滤器已判定），将「表」交给弃牌者，
/// owner 失去 1 点体力。无弃牌者（来源不可考）时「表」入弃牌堆。
pub(crate) fn resolve_biaozhao2(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    event_player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let pile = tx.state().skill_pile_cards(player, BIAOZHAO_PILE_DEF);
    if pile.is_empty() {
        return Ok(Vec::new());
    }
    let card = pile[0];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    // 从「表」堆移除。
    if let Some(pile_zone) = tx
        .state()
        .skill_piles
        .iter()
        .find(|p| p.owner == player && p.skill_def == BIAOZHAO_PILE_DEF)
        .map(|p| p.zone)
    {
        tx.state_mut()
            .zones
            .get_mut(pile_zone)
            .unwrap()
            .cards
            .retain(|c| *c != card);
    }
    // 有存活弃牌者 → 「表」交给弃牌者；否则「表」入弃牌堆。
    let giver_target = if event_player != PlayerId::NONE
        && tx
            .state()
            .players
            .get(event_player)
            .is_some_and(|participant| participant.alive)
    {
        event_player
    } else {
        player
    };
    if giver_target == player {
        // 无可交付对象 → 「表」入弃牌堆。
        tx.state_mut()
            .zones
            .get_mut(discard)
            .unwrap()
            .cards
            .push(card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = discard;
        }
    } else {
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, giver_target)
            .ok_or_else(|| EffectError::Illegal("no discarder hand".into()))?;
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = hand;
            card_state.owner = giver_target;
        }
    }
    // owner 失去 1 点体力。
    let lose = super::super::deal_damage(
        tx,
        PlayerId::NONE,
        player,
        1,
        true,
        0,
        false,
        CardId::NONE,
        0,
        0,
    )?;
    Ok(lose)
}

/// 专属状态机 (批 260, ADR 0641 §8.49)：逆击——成为非装备牌目标后摸 1 张牌
/// 并打上「逆击」标记；结束阶段弃置手牌中全部「逆击」牌。
pub(crate) fn resolve_dcniji(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    mode: u32,
    event_card: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match mode {
        0 => {
            // 非装备牌目标：事件牌须存在且非装备。
            let Some(targeting) = tx.state().cards.get(CardId::new(event_card)) else {
                return Ok(Vec::new());
            };
            if targeting.kind.equip_slot().is_some() {
                return Ok(Vec::new());
            }
            let mut events = crate::effect::draw_cards(tx, player, 1)?;
            if let Some(engine_api::DomainEvent::CardsDrawn { cards, .. }) = events.last() {
                for card in cards {
                    tx.state_mut().add_content_card_tag(*card, DCNIJI_TAG);
                }
            }
            Ok(events)
        }
        1 => {
            // 结束阶段：弃置手牌中全部「逆击」牌。
            let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            let tagged: Vec<CardId> = tx
                .state()
                .zones
                .get(hand)
                .unwrap()
                .cards
                .iter()
                .copied()
                .filter(|card| tx.state().card_has_content_tag(*card, DCNIJI_TAG))
                .collect();
            for card in tagged {
                tx.state_mut()
                    .zones
                    .get_mut(hand)
                    .unwrap()
                    .cards
                    .retain(|c| *c != card);
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
            Ok(Vec::new())
        }
        _ => Ok(Vec::new()),
    }
}

/// 专属状态机 (批 261, ADR 0641 §8.50)：谋溃——杀指定目标后，chooseButton
/// [1,2]：摸 1 张牌／弃置目标一张手牌（owner 从目标手牌选），可同选两项。
/// 未命中追罚面留 slice 2。
pub(crate) fn resolve_dcmoukui(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            // 新一次杀指定：清除上一杀可能残留的追罚标记。
            tx.state_mut()
                .skills
                .storage_set(skill, DCMOUKUI_FLAG_KEY, 0);
            // Custom 双选：1=摸一张牌 2=弃目标一张牌；可选 1-2 项，不可取消。
            open_choice(
                tx,
                player,
                1,
                2,
                false,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::ResolveDcmoukuiStage,
                vec![
                    RESOLVE_DCMOUKUI_MAGIC,
                    player.raw(),
                    skill.raw(),
                    2,
                    target.raw(),
                ],
            )
        }
        5 => {
            // 追罚入口：杀未命中且双选标记在 → 闪避者选 owner 一张手牌弃置。
            resolve_dcmoukui_conseq(tx, player, skill, target)
        }
        _ => Ok(Vec::new()),
    }
}
