//! Command operations for `open_sha_response_step` and related helpers.

use super::*;

/// Open one step of 杀-闪 response (supports multi-shan for 无双 + 八卦).
/// payload: `[source, target, amount, need, have, nature, respondable, card]`
pub(crate) fn open_sha_response_step(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    need: u32,
    have: u32,
    nature: u8,
    card_flags: u8,
    respondable: bool,
    card: CardId,
) -> Result<(), EffectError> {
    let tokens = if respondable {
        let mut tokens = response_shan_options(tx.state(), target);
        if have == 0 {
            tokens.extend(response_youdishenru_tokens(tx.state(), target));
        }
        tokens
    } else {
        Vec::new()
    };
    if tokens.is_empty() {
        return deal_single_sha_damage(tx, source, target, amount, nature, card_flags, card);
    }
    crate::effect::open_choice(
        tx,
        target,
        0,
        1,
        true,
        tokens,
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::RespondToSha,
        vec![
            source.raw(),
            target.raw(),
            amount,
            need,
            have,
            pack_damage_card_context(nature, card_flags),
            u32::from(respondable),
            card.raw(),
        ],
    )?;
    Err(EffectError::ChoiceRequested)
}

pub(crate) fn deal_sha_damage(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    nature: u8,
    card_flags: u8,
    card: CardId,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    let amount =
        crate::query::sha_damage_outside_target_range_for(tx.state(), source, target, amount);
    let amount = crate::query::sha_damage_taken_for(tx.state(), target, amount);
    run_effects(
        tx,
        vec![EffectRequest::DealShaDamage {
            source,
            target,
            card,
            amount,
            nature,
            card_flags,
        }],
    )
}

/// 闪 response options: 八卦 token (if equipped) + hand 闪 / view_as 闪.
pub(crate) fn response_shan_options(state: &crate::state::GameState, player: PlayerId) -> Vec<u32> {
    if crate::query::card_response_cancelled_for(state, player) {
        return Vec::new();
    }
    let mut out = Vec::new();
    if crate::content::player_has_bagua(state, player) {
        out.push(crate::content::BAGUA_RESPONSE_TOKEN);
    }
    out.extend(response_shan_tokens(state, player));
    out
}

/// Shan options a different player may physically provide for a delegated response.
pub(crate) fn delegated_shan_options(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Vec<u32> {
    if crate::query::card_response_cancelled_for(state, player) {
        return Vec::new();
    }
    let mut out = Vec::new();
    if crate::content::player_has_bagua(state, player) {
        out.push(crate::content::BAGUA_RESPONSE_TOKEN);
    }
    if !crate::query::hand_card_use_response_cancelled_for(state, player) {
        out.extend(response_card_tokens(state, player, "shan", |kind| {
            kind == CardKind::Shan
        }));
    }
    out
}

/// Physical Sha options another player may provide without recursively delegating again.
pub(crate) fn delegated_sha_options(state: &crate::state::GameState, player: PlayerId) -> Vec<u32> {
    if crate::query::card_response_cancelled_for(state, player)
        || crate::query::sha_response_cancelled_for(state, player)
        || crate::query::hand_card_use_response_cancelled_for(state, player)
    {
        return Vec::new();
    }
    response_card_tokens(state, player, "sha", CardKind::is_sha)
}

/// Run 八卦判定：红心视为打出 闪。Returns `true` if successful.
pub(crate) fn try_bagua_as_shan(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<(bool, EffectRequest), EffectError> {
    let bagua = crate::content::player_bagua_source(tx.state(), player)
        .ok_or_else(|| EffectError::Illegal("no bagua".into()))?;
    let result =
        super::super::judge::flip_judgment(tx, player, super::super::judge::JudgmentFavor::Red)?;
    let suit = result.suit;
    let rank = result.rank;
    let ok = matches!(suit, engine_api::Suit::Heart | engine_api::Suit::Diamond);
    run_effects(
        tx,
        vec![EffectRequest::EmitRuleEvent {
            code: 231, // bagua judge
            payload: vec![
                player.raw(),
                super::super::judge::suit_tag(suit),
                rank as u32,
                u32::from(ok),
            ],
        }],
    )?;
    Ok((
        ok,
        EffectRequest::EmitJudgmentResolved {
            player,
            result_card: result.card,
            suit,
            rank,
            cause_card: bagua,
        },
    ))
}

/// Ordered response queue: kind 1=南蛮 need 杀, 2=万箭 need 闪, 3=multi-target 杀.
pub(crate) fn begin_aoe_queue(
    tx: &mut Transaction,
    kind: u32,
    source: PlayerId,
    amount: u32,
    targets: &[PlayerId],
    card: CardId,
) -> Result<(), EffectError> {
    if targets.is_empty() {
        return Ok(());
    }
    let mut payload = vec![
        kind,
        source.raw(),
        amount,
        0, // idx
        targets.len() as u32,
    ];
    for t in targets {
        payload.push(t.raw());
    }
    payload.push(card.raw());
    // Store payload then process from idx 0
    eprintln!(
        "DBG begin_aoe_queue kind={} targets={:?} card={}",
        kind,
        targets,
        card.raw()
    );
    {
        let r = &mut tx.state_mut().resolution;
        r.continuation_kind = crate::state::ContinuationKind::RespondAoeQueue;
        r.continuation_payload = payload;
    }
    advance_aoe_queue(tx)
}

/// Bump AOE queue index on resume payload and/or live continuation.
pub(crate) fn bump_aoe_resume_idx(tx: &mut Transaction, next_idx: usize) {
    let r = &mut tx.state_mut().resolution;
    if r.resume_kind == crate::state::ContinuationKind::RespondAoeQueue
        && r.resume_payload.len() > 3
    {
        r.resume_payload[3] = next_idx as u32;
        if r.resume_payload.first() == Some(&3) && r.resume_payload.len() > 7 {
            r.resume_payload[7] = 0;
        }
    }
    if r.continuation_kind == crate::state::ContinuationKind::RespondAoeQueue
        && r.continuation_payload.len() > 3
    {
        r.continuation_payload[3] = next_idx as u32;
        if r.continuation_payload.first() == Some(&3) && r.continuation_payload.len() > 7 {
            r.continuation_payload[7] = 0;
        }
    }
}

/// Process current AOE target or finish.
pub(crate) fn advance_aoe_queue(tx: &mut Transaction) -> Result<(), EffectError> {
    loop {
        let mut payload = tx.state().resolution.continuation_payload.clone();
        if payload.len() < 6 {
            tx.state_mut().resolution.clear_choice();
            return Ok(());
        }
        let kind = payload[0];
        let source = PlayerId::new(payload[1]);
        let amount = payload[2];
        let idx = payload[3] as usize;
        let len = payload[4] as usize;
        let target_offset = if kind == 3 { 8 } else { 5 };
        if idx >= len || target_offset + idx >= payload.len() {
            tx.state_mut().resolution.clear_choice();
            if kind == 3 {
                let card = CardId::new(*payload.get(8 + 2 * len).unwrap_or(&0));
                return finish_played_sha(tx, source, card, *payload.get(5).unwrap_or(&0));
            }
            return Ok(());
        }
        let target = PlayerId::new(payload[target_offset + idx]);
        let source_card = if kind == 3 {
            CardId::new(*payload.get(8 + 2 * len).unwrap_or(&0))
        } else {
            CardId::new(*payload.get(5 + len).unwrap_or(&0))
        };
        // Skip dead
        if tx
            .state()
            .players
            .get(target)
            .map(|p| !p.alive)
            .unwrap_or(true)
        {
            let queue = &mut tx.state_mut().resolution.continuation_payload;
            queue[3] = (idx + 1) as u32;
            if kind == 3 {
                queue[7] = 0;
            }
            continue;
        }
        if kind == 3 {
            payload[6] = crate::query::sha_response_count_for(tx.state(), source, target, 1);
        }
        // 批 155：消费暂存排除（啖酪族「令此牌对你无效」——触发效果执行时 AOE 队列
        // 尚未建立，排除意图经 deferred_card_exclusions 传递至此）。
        let deferred_hit =
            tx.state()
                .deferred_card_exclusions
                .iter()
                .any(|(card_raw, target_raw)| {
                    *card_raw == source_card.raw() && *target_raw == target.raw()
                });
        if deferred_hit && (kind == 1 || kind == 2) {
            tx.state_mut()
                .deferred_card_exclusions
                .retain(|(card_raw, target_raw)| {
                    !(*card_raw == source_card.raw() && *target_raw == target.raw())
                });
            tx.state_mut().resolution.continuation_payload[3] = (idx + 1) as u32;
            continue;
        }
        // 藤甲：南蛮入侵对该角色无效（无需出杀，不受伤）
        if kind == 1 && crate::content::player_has_tengjia(tx.state(), target) {
            let _ = run_effects(
                tx,
                vec![EffectRequest::EmitRuleEvent {
                    code: 230, // tengjia blocks nanman
                    payload: vec![target.raw()],
                }],
            );
            tx.state_mut().resolution.continuation_payload[3] = (idx + 1) as u32;
            continue;
        }
        let respondable = (kind != 3
            || sha_queue_target_respondable(&payload, target_offset, len, idx))
            && !crate::query::card_use_unrespondable_within_distance(tx.state(), source, target)
            && !crate::query::potshishi_blocks_response(tx.state(), source, source_card, target);
        let tokens = match kind {
            1 if respondable => response_sha_tokens(tx.state(), target),
            2 | 3 if respondable => {
                let mut options = response_shan_options(tx.state(), target);
                if kind == 3 && payload[7] == 0 {
                    options.extend(response_youdishenru_tokens(tx.state(), target));
                }
                options
            }
            _ => vec![],
        };
        if kind == 3 && payload[6] == 0 {
            let queue = &mut tx.state_mut().resolution.continuation_payload;
            queue[3] = (idx + 1) as u32;
            queue[7] = 0;
            continue;
        }
        eprintln!(
            "DBG aoe idx={} target={} tokens={} respondable={}",
            idx,
            target.raw(),
            tokens.len(),
            respondable
        );
        if tokens.is_empty() {
            eprintln!("DBG aoe dealing damage to {}", target.raw());
            // Fail → damage, next
            let dmg = if kind == 3 {
                let card = CardId::new(*payload.get(8 + 2 * len).unwrap_or(&0));
                let (nature, card_flags) = unpack_damage_card_context(payload[5]);
                let amount = amount.saturating_add(sha_queue_target_damage_bonus(
                    &payload,
                    target_offset,
                    len,
                    idx,
                ));
                deal_sha_damage(tx, source, target, amount, nature, card_flags, card)
            } else {
                eprintln!(
                    "DBG aoe DealDamage src={} tgt={} amount={}",
                    source.raw(),
                    target.raw(),
                    amount
                );
                let res = run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source,
                        target,
                        amount,
                        is_lose_hp: false,
                        nature: 0,
                    }],
                );
                eprintln!(
                    "DBG aoe after dmg hp={}",
                    tx.state().players.get(target).map(|p| p.hp).unwrap_or(-1)
                );
                res
            };
            match dmg {
                Ok(_) => {}
                Err(EffectError::ChoiceRequested) => {
                    // Damage applied; dying (or nested choice) interrupted.
                    // Always advance past this target so resume does not re-hit.
                    bump_aoe_resume_idx(tx, idx + 1);
                    return Err(EffectError::ChoiceRequested);
                }
                Err(e) => return Err(e),
            }
            if tx.state().has_pending_choice() {
                bump_aoe_resume_idx(tx, idx + 1);
                return Ok(());
            }
            // Still in AOE: advance idx on stored payload
            if tx.state().resolution.continuation_kind
                == crate::state::ContinuationKind::RespondAoeQueue
                && tx.state().resolution.continuation_payload.len() > 3
            {
                tx.state_mut().resolution.continuation_payload[3] = (idx + 1) as u32;
                if kind == 3 {
                    tx.state_mut().resolution.continuation_payload[7] = 0;
                }
            } else {
                // restore queue progress
                let mut p = payload;
                p[3] = (idx + 1) as u32;
                if kind == 3 {
                    p[7] = 0;
                }
                tx.state_mut().resolution.continuation_kind =
                    crate::state::ContinuationKind::RespondAoeQueue;
                tx.state_mut().resolution.continuation_payload = p;
            }
            continue;
        }
        // Open response for this target
        crate::effect::open_choice(
            tx,
            target,
            0,
            1,
            true,
            tokens,
            engine_api::ChoiceKind::SelectCards,
            crate::state::ContinuationKind::RespondAoeQueue,
            // keep full queue payload
            {
                let mut p = payload;
                p[3] = idx as u32; // current
                p
            },
        )?;
        return Err(EffectError::ChoiceRequested);
    }
}

pub(crate) fn sha_queue_target_respondable(
    payload: &[u32],
    target_offset: usize,
    len: usize,
    idx: usize,
) -> bool {
    let response_offset = target_offset + len;
    if payload.len() >= response_offset + len {
        payload.get(response_offset + idx).copied().unwrap_or(1) & SHA_RESUME_RESPONDABLE != 0
    } else {
        // Accept the previous single-flag in-flight queue shape.
        payload.get(response_offset).copied().unwrap_or(1) & SHA_RESUME_RESPONDABLE != 0
    }
}

pub(crate) fn sha_queue_target_damage_bonus(
    payload: &[u32],
    target_offset: usize,
    len: usize,
    idx: usize,
) -> u32 {
    let response_offset = target_offset.saturating_add(len);
    if payload.len() >= response_offset.saturating_add(len) {
        payload
            .get(response_offset + idx)
            .copied()
            .map(sha_target_damage_bonus)
            .unwrap_or(0)
    } else {
        0
    }
}

pub(crate) fn begin_juedou_with_tail(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    tail: &[u32],
) -> Result<(), EffectError> {
    open_juedou_step_n_with_tail(tx, source, target, target, 0, tail)
}

pub(crate) fn open_juedou_step_n_with_tail(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    next: PlayerId,
    exchanges: u32,
    tail: &[u32],
) -> Result<(), EffectError> {
    // Safety: prevent pathological view_as 杀 loops (max 32 exchanges).
    if exchanges >= 32 {
        return finish_juedou_with_tail(tx, source, target, next, 1, tail);
    }
    let opponent = if next == target { source } else { target };
    let need = crate::query::duel_response_count_for(tx.state(), opponent, 1);
    open_juedou_response_step(tx, source, target, next, exchanges, need, 0, tail)
}

pub(crate) fn open_juedou_response_step(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    next: PlayerId,
    exchanges: u32,
    need: u32,
    have: u32,
    tail: &[u32],
) -> Result<(), EffectError> {
    let source_card = queued_independent_trick(tail)
        .map(|(payload, _)| CardId::new(payload[0]))
        .unwrap_or_else(|| effect_args_source_card(tail));
    let tokens = if next == target
        && (crate::query::card_use_unrespondable_within_distance(tx.state(), source, target)
            || crate::query::potshishi_blocks_response(tx.state(), source, source_card, target))
    {
        Vec::new()
    } else {
        response_sha_tokens(tx.state(), next)
    };
    if tokens.is_empty() {
        return finish_juedou_with_tail(tx, source, target, next, 1, tail);
    }
    let mut payload = vec![
        source.raw(),
        target.raw(),
        next.raw(),
        exchanges,
        need,
        have,
    ];
    payload.extend_from_slice(tail);
    crate::effect::open_choice(
        tx,
        next,
        0,
        1,
        true,
        tokens,
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::JuedouDuel,
        payload,
    )?;
    Err(EffectError::ChoiceRequested)
}

pub(crate) fn juedou_payload_after_response(
    source: PlayerId,
    target: PlayerId,
    next: PlayerId,
    exchanges: u32,
    need: u32,
    have: u32,
    tail: &[u32],
) -> Vec<u32> {
    let have = have.saturating_add(1);
    let (next, exchanges, need, have) = if have < need {
        (next, exchanges, need, have)
    } else {
        (
            if next == target { source } else { target },
            exchanges.saturating_add(1),
            0,
            0,
        )
    };
    let mut payload = vec![
        source.raw(),
        target.raw(),
        next.raw(),
        exchanges,
        need,
        have,
    ];
    payload.extend_from_slice(tail);
    payload
}

pub(crate) fn continue_juedou_payload(
    tx: &mut Transaction,
    payload: &[u32],
) -> Result<(), EffectError> {
    let source = PlayerId::new(*payload.first().unwrap_or(&0));
    let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let next = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let exchanges = *payload.get(3).unwrap_or(&0);
    let need = *payload.get(4).unwrap_or(&0);
    let have = *payload.get(5).unwrap_or(&0);
    let tail = payload.get(6..).unwrap_or_default();
    if need == 0 {
        open_juedou_step_n_with_tail(tx, source, target, next, exchanges, tail)
    } else {
        open_juedou_response_step(tx, source, target, next, exchanges, need, have, tail)
    }
}

pub(crate) fn finish_juedou_with_tail(
    tx: &mut Transaction,
    initiator: PlayerId,
    target: PlayerId,
    loser: PlayerId,
    amount: u32,
    tail: &[u32],
) -> Result<(), EffectError> {
    let winner = if loser == target { initiator } else { target };
    let amount = amount.saturating_add(saved_independent_trick_damage_bonus(tail));
    let queued = queued_independent_trick(tail);
    let card = queued
        .map(|(payload, _)| CardId::new(payload[0]))
        .unwrap_or(CardId::NONE);
    let damage = if let Some((payload, _)) = queued {
        EffectRequest::DealCardDamage {
            source: winner,
            target: loser,
            amount,
            is_lose_hp: false,
            nature: 0,
            card: CardId::new(payload[0]),
            card_kind: CardKind::TrickJuedou.tag(),
            card_flags: 0,
        }
    } else {
        EffectRequest::DealDamage {
            source: winner,
            target: loser,
            amount,
            is_lose_hp: false,
            nature: 0,
        }
    };
    run_effects_then_finish_independent_trick_target(
        tx,
        vec![
            damage,
            EffectRequest::EmitDuelResolved {
                card,
                initiator,
                target,
                winner,
                loser,
            },
        ],
        tail,
    )
}

pub(crate) fn player_has_mark(tx: &Transaction, player: PlayerId, mark: u32) -> bool {
    tx.state()
        .players
        .get(player)
        .map(|p| p.marks.iter().any(|(m, c)| *m == mark && *c > 0))
        .unwrap_or(false)
}

pub(crate) fn encode_youdishenru_saved(
    protected: PlayerId,
    source: PlayerId,
    incoming: CardId,
    kind: crate::state::ContinuationKind,
    payload: &[u32],
) -> Vec<u32> {
    let mut saved = vec![
        YOUDISHENRU_SAVED_MAGIC,
        protected.raw(),
        source.raw(),
        incoming.raw(),
        u32::from(kind.tag()),
        payload.len().min(u32::MAX as usize) as u32,
    ];
    saved.extend_from_slice(payload);
    saved
}

pub(crate) fn parse_youdishenru_saved(
    saved: &[u32],
) -> Result<
    (
        PlayerId,
        PlayerId,
        CardId,
        crate::state::ContinuationKind,
        &[u32],
    ),
    EffectError,
> {
    if saved.first() != Some(&YOUDISHENRU_SAVED_MAGIC) {
        return Err(EffectError::EngineFault(
            "bad 诱敌深入 continuation marker".into(),
        ));
    }
    let len = *saved.get(5).unwrap_or(&u32::MAX) as usize;
    if saved.len() != 6usize.saturating_add(len) {
        return Err(EffectError::EngineFault(
            "truncated 诱敌深入 continuation".into(),
        ));
    }
    Ok((
        PlayerId::new(*saved.get(1).unwrap_or(&0)),
        PlayerId::new(*saved.get(2).unwrap_or(&0)),
        CardId::new(*saved.get(3).unwrap_or(&0)),
        crate::state::ContinuationKind::from_tag(*saved.get(4).unwrap_or(&0) as u16),
        &saved[6..],
    ))
}
