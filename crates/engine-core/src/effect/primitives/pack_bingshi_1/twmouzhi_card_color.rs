//! Effect primitives for `resolve_dcbingji` and related helpers.

use super::*;

/// 事件牌颜色（上游 get.color 口径）：1=红（♥♦），2=黑（♠♣），None=无牌/无色。
pub(crate) fn twmouzhi_card_color(tx: &Transaction, raw: u32) -> Option<u32> {
    if raw == 0 {
        return None;
    }
    tx.state()
        .cards
        .get(CardId::new(raw))
        .and_then(|card| match card.suit {
            engine_api::Suit::Heart | engine_api::Suit::Diamond => Some(1),
            engine_api::Suit::Spade | engine_api::Suit::Club => Some(2),
            _ => None,
        })
}

/// 专属状态机 (批 245, ADR 0641 §8.39)：谋制——锁定技。mode 0（damage_before）：
/// 事件牌有颜色且与记录的上次受伤牌颜色相同时防止本次伤害（防止不更新记录）；
/// mode 1（damage）：记录事件牌颜色（无牌/无色清除记录）。
pub(crate) fn resolve_twmouzhi(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    mode: u32,
    event_card: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
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
            let Some(color) = twmouzhi_card_color(tx, event_card) else {
                return Ok(Vec::new());
            };
            let last = tx.state().skills.storage_get(skill, TWMOUZHI_COLOR_KEY);
            if last != 0 && last == color {
                let damage_id = tx.state().resolution.current_damage_id;
                if damage_id != 0
                    && !tx
                        .state()
                        .resolution
                        .prevented_damage_ids
                        .contains(&damage_id)
                {
                    tx.state_mut()
                        .resolution
                        .prevented_damage_ids
                        .push(damage_id);
                }
            }
        }
        1 => {
            let color = twmouzhi_card_color(tx, event_card).unwrap_or(0);
            if !tx
                .state_mut()
                .skills
                .storage_set(skill, TWMOUZHI_COLOR_KEY, color)
            {
                return Err(EffectError::Illegal("twmouzhi: bad skill".into()));
            }
        }
        _ => {}
    }
    Ok(Vec::new())
}

/// stage 3 轮次载荷：[magic, player, skill, 3, k, m, targets(k), revealed(m),
/// picks(2×r)]——picks 按 (chooser_raw, card_raw) 记录，card=0 表示该轮跳过
/// （目标死亡或余牌为空）。
pub(crate) fn twdaigui_round_payload(
    player: PlayerId,
    skill: engine_api::SkillId,
    targets: &[u32],
    revealed: &[u32],
    picks: &[(u32, u32)],
) -> Vec<u32> {
    let mut payload = vec![
        RESOLVE_TWDAIGUI_MAGIC,
        player.raw(),
        skill.raw(),
        3,
        targets.len() as u32,
        revealed.len() as u32,
    ];
    payload.extend_from_slice(targets);
    payload.extend_from_slice(revealed);
    for (chooser, card) in picks {
        payload.push(*chooser);
        payload.push(*card);
    }
    payload
}

/// 专属状态机 (批 247/248, ADR 0641 §8.40)：待归——出牌阶段结束时，若你的
/// 手牌颜色均相同，选择至多 X（手牌数）名角色，亮出牌堆底等量的牌，这些角色
/// 按座次依次选获其中一张（slice 2）。
pub(crate) fn resolve_twdaigui(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    count: u32,
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
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            // 手牌同色门控（机内权威校验，上游 filter：所有手牌 get.color 一致）。
            let mut red = 0usize;
            let mut black = 0usize;
            for card in &hand {
                if let Some(card_state) = tx.state().cards.get(*card) {
                    match card_state.suit {
                        engine_api::Suit::Heart | engine_api::Suit::Diamond => red += 1,
                        engine_api::Suit::Spade | engine_api::Suit::Club => black += 1,
                        _ => {}
                    }
                }
            }
            if red > 0 && black > 0 {
                return Ok(Vec::new());
            }
            if red == 0 && black == 0 {
                return Ok(Vec::new());
            }
            // 选 [1, 手牌数] 名任意存活角色（可含自己，可取消）。
            let targets: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            let max_n = hand.len().min(u8::MAX as usize) as u8;
            open_choice(
                tx,
                player,
                1,
                max_n,
                true,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwdaiguiStage,
                vec![RESOLVE_TWDAIGUI_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        _ => Ok(Vec::new()),
    }
}

/// 目标选择已确认（resolver 经由 crate::effect 重导出调用）：
/// selected = 「待归」目标（1..=手牌数）。亮出等量牌堆底牌（公开）并按座次
/// 打开第一轮选获。
pub(crate) fn resolve_twdaigui_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    selected: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    if selected.is_empty() {
        return Ok(Vec::new());
    }
    // 目标按座次排序（上游 sortBySeat）。
    let seat: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(PlayerId::raw)
        .collect();
    let mut targets: Vec<u32> = selected.to_vec();
    targets.sort_by_key(|raw| seat.iter().position(|s| s == raw).unwrap_or(usize::MAX));

    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let cards = tx.state().zones.get(draw).unwrap().cards.clone();
    let n = targets.len().min(cards.len());
    if n == 0 {
        return Ok(Vec::new());
    }
    let revealed: Vec<u32> = cards[..n].iter().map(|card| card.raw()).collect();
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::RevealCards {
            cards: revealed.iter().map(|raw| CardId::new(*raw)).collect(),
            to: Vec::new(),
        }],
    )?;
    open_twdaigui_round(tx, player, skill, targets, revealed, Vec::new())?;
    Ok(Vec::new())
}

/// 打开第 r 轮选获（r = picks.len()）。已死亡目标与余牌为空的轮次按上游
/// `continue` 语义跳过；全部轮次完成后余牌归弃牌堆。
pub(crate) fn open_twdaigui_round(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    targets: Vec<u32>,
    revealed: Vec<u32>,
    mut picks: Vec<(u32, u32)>,
) -> Result<(), EffectError> {
    let k = targets.len();
    loop {
        let r = picks.len();
        if r >= k {
            return twdaigui_finish(tx, revealed, &picks);
        }
        let chooser_raw = targets[r];
        let chooser = PlayerId::new(chooser_raw);
        if !tx
            .state()
            .players
            .get(chooser)
            .is_some_and(|participant| participant.alive)
        {
            picks.push((chooser_raw, 0));
            continue;
        }
        let picked: std::collections::BTreeSet<u32> = picks.iter().map(|(_, card)| *card).collect();
        let remaining: Vec<u32> = revealed
            .iter()
            .copied()
            .filter(|card| !picked.contains(card))
            .collect();
        if remaining.is_empty() {
            picks.push((chooser_raw, 0));
            continue;
        }
        let payload = twdaigui_round_payload(player, skill, &targets, &revealed, &picks);
        open_choice(
            tx,
            chooser,
            1,
            1,
            false,
            remaining,
            ChoiceKind::SelectCards,
            ContinuationKind::ResolveTwdaiguiStage,
            payload,
        )?;
        return Ok(());
    }
}

/// 轮次选获已确认（resolver 调用）：observer = 当前选获目标，selections[0] =
/// 选获牌。入手后开下一轮（或终局）。
pub(crate) fn resolve_twdaigui_round(
    tx: &mut Transaction,
    observer: PlayerId,
    player: PlayerId,
    skill: engine_api::SkillId,
    payload: &[u32],
    selections: &[u32],
) -> Result<(), EffectError> {
    if payload.len() < 7 || payload[0] != RESOLVE_TWDAIGUI_MAGIC || payload[3] != 3 {
        return Err(EffectError::EngineFault(
            "bad resolve-twdaigui round payload".into(),
        ));
    }
    let k = payload[4] as usize;
    let m = payload[5] as usize;
    if payload.len() < 6 + k + m || (payload.len() - 6 - k - m) % 2 != 0 {
        return Err(EffectError::EngineFault(
            "bad resolve-twdaigui round payload layout".into(),
        ));
    }
    let targets: Vec<u32> = payload[6..6 + k].to_vec();
    let revealed: Vec<u32> = payload[6 + k..6 + k + m].to_vec();
    let picks_rest = &payload[6 + k + m..];
    let mut picks: Vec<(u32, u32)> = picks_rest
        .chunks_exact(2)
        .map(|pair| (pair[0], pair[1]))
        .collect();
    let r = picks.len();
    let chooser_raw = targets.get(r).copied().ok_or_else(|| {
        EffectError::EngineFault("resolve-twdaigui round index out of range".into())
    })?;
    let chooser = PlayerId::new(chooser_raw);
    if observer != chooser {
        return Err(EffectError::Illegal(
            "resolve-twdaigui round chooser mismatch".into(),
        ));
    }
    let picked: std::collections::BTreeSet<u32> = picks.iter().map(|(_, card)| *card).collect();
    let card_raw = *selections
        .first()
        .ok_or_else(|| EffectError::Illegal("resolve-twdaigui pick required".into()))?;
    let revealed_set: std::collections::BTreeSet<u32> = revealed.iter().copied().collect();
    if !revealed_set.contains(&card_raw)
        || picked.contains(&card_raw)
        || !tx.state().players.get(chooser).is_some_and(|p| p.alive)
    {
        return Err(EffectError::Illegal(
            "resolve-twdaigui pick outside revealed pool".into(),
        ));
    }
    // 选获入手：牌堆 → 选获者手牌。
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, chooser)
        .ok_or_else(|| EffectError::Illegal("no chooser hand".into()))?;
    let card = CardId::new(card_raw);
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
        card_state.zone = hand;
        card_state.owner = chooser;
    }
    picks.push((chooser_raw, card_raw));
    open_twdaigui_round(tx, player, skill, targets, revealed, picks)?;
    Ok(())
}

/// 终局：余牌（亮出未选获）归弃牌堆。
pub(crate) fn twdaigui_finish(
    tx: &mut Transaction,
    revealed: Vec<u32>,
    picks: &[(u32, u32)],
) -> Result<(), EffectError> {
    let picked: std::collections::BTreeSet<u32> = picks.iter().map(|(_, card)| *card).collect();
    let leftover: Vec<CardId> = revealed
        .iter()
        .copied()
        .filter(|card| !picked.contains(card))
        .map(CardId::new)
        .collect();
    if leftover.is_empty() {
        return Ok(());
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    crate::effect::move_cards(tx, leftover, discard, None)?;
    Ok(())
}

/// 专属状态机 (批 249, ADR 0641 §8.41 slice 1)：迷途——准备阶段，你选择至多
/// 3 名角色，这些角色各摸 2 张牌、摸得牌打上「迷途」标记并公开亮出
/// （slice 2 将接续拼点轮与崩溃面）。
pub(crate) fn resolve_twmitu(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            let _ = count;
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
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                3,
                true,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwmituStage,
                vec![RESOLVE_TWMITU_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        _ => Ok(Vec::new()),
    }
}

/// 目标选择已确认（resolver 调用）：各目标按座次摸 2 张、打「迷途」标记并
/// 公开亮出，然后进入拼点轮推进（slice 2）。
pub(crate) fn resolve_twmitu_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    selected: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    if selected.is_empty() {
        return Ok(Vec::new());
    }
    let seat: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(PlayerId::raw)
        .collect();
    let mut targets: Vec<u32> = selected.to_vec();
    targets.sort_by_key(|raw| seat.iter().position(|s| s == raw).unwrap_or(usize::MAX));

    let mut events = Vec::new();
    let mut shown: Vec<CardId> = Vec::new();
    let mut shown_by_target: Vec<u32> = Vec::new();
    for raw in targets.clone() {
        let target = PlayerId::new(raw);
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            shown_by_target.push(0);
            continue;
        }
        let drawn = crate::effect::draw_cards(tx, target, 2)?;
        let mut first = 0u32;
        for event in &drawn {
            if let engine_api::DomainEvent::CardsDrawn { cards, .. } = event {
                for card in cards {
                    tx.state_mut().add_content_card_tag(*card, TWMITU_TAG);
                    shown.push(*card);
                    if first == 0 {
                        first = card.raw();
                    }
                }
            }
        }
        shown_by_target.push(first);
        events.extend(drawn);
    }
    if !shown.is_empty() {
        crate::effect::run_effects(
            tx,
            vec![EffectRequest::RevealCards {
                cards: shown,
                to: Vec::new(),
            }],
        )?;
    }
    // slice 2：拼点轮推进（第 0 轮；upstream compare 轮按座次逐目标）。
    twmitu_advance(
        tx,
        player,
        skill,
        &targets,
        &shown_by_target,
        0,
        &[],
        &[],
        &[],
    )?;
    Ok(events)
}

/// 拼点轮载荷：[magic, player, skill, stage, k, done, targets(k), opponents(d),
/// consents(d), pend]——stage 3=owner 选对手；4=目标知情（Custom 1/2）；
/// 5=目标选拼点牌；6=对手选拼点牌。pend 为 5→6 之间暂存的目标牌。
pub(crate) fn twmitu_round_payload(
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    targets: &[u32],
    shown: &[u32],
    done: usize,
    opponents: &[u32],
    consents: &[u32],
    pend: &[u32],
) -> Vec<u32> {
    let mut payload = vec![
        RESOLVE_TWMITU_MAGIC,
        player.raw(),
        skill.raw(),
        stage,
        targets.len() as u32,
        done as u32,
    ];
    payload.extend_from_slice(targets);
    payload.extend_from_slice(shown);
    payload.extend_from_slice(opponents);
    payload.extend_from_slice(consents);
    payload.extend_from_slice(pend);
    payload
}

/// 轮次推进：跳过死亡/无手牌/无合法对手的轮次；done == k 时收口（slice 3
/// 崩溃面将在此接续）。
pub(crate) fn twmitu_advance(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    targets: &[u32],
    shown: &[u32],
    done: usize,
    opponents: &[u32],
    consents: &[u32],
    pend: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let k = targets.len();
    if done >= k {
        return Ok(Vec::new());
    }
    let target_raw = targets[done];
    let target = PlayerId::new(target_raw);
    // 上游 !isIn / canCompare 不可满足 → 跳过该轮。
    let target_alive = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive);
    let candidates: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != target)
        .filter(|&candidate| !tx.state().hand_cards(candidate).is_empty())
        .map(PlayerId::raw)
        .collect();
    if !target_alive || tx.state().hand_cards(target).is_empty() || candidates.is_empty() {
        return twmitu_advance(
            tx,
            player,
            skill,
            targets,
            shown,
            done + 1,
            opponents,
            consents,
            pend,
        );
    }
    // stage 3：owner 为该目标指定拼点对手（上游 forced chooseTarget）。
    let payload = twmitu_round_payload(
        player, skill, 3, targets, shown, done, opponents, consents, pend,
    );
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        candidates,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveTwmituStage,
        payload,
    )?;
    Ok(Vec::new())
}
