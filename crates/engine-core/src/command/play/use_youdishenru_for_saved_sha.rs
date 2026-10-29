//! Command operations for `open_sha_response_step` and related helpers.

use super::*;

/// Use a physical or materialized 诱敌深入 against the exact suspended incoming Sha.
pub(crate) fn use_youdishenru_for_saved_sha(
    tx: &mut Transaction,
    protected: PlayerId,
    source: PlayerId,
    incoming: CardId,
    youdi_card: CardId,
    saved_kind: crate::state::ContinuationKind,
    saved_payload: &[u32],
) -> Result<(), EffectError> {
    let valid = tx.state().cards.get(youdi_card).is_some_and(|card| {
        card.kind == CardKind::TrickYoudishenru
            && tx
                .state()
                .playable_hand_cards(protected)
                .contains(&youdi_card)
    });
    if !valid || source == protected || incoming.is_none() {
        return Err(EffectError::Illegal(
            "invalid 诱敌深入 response context".into(),
        ));
    }
    let saved = encode_youdishenru_saved(protected, source, incoming, saved_kind, saved_payload);
    run_effects_then_wuxie(
        tx,
        vec![EffectRequest::UseCard {
            player: protected,
            card: youdi_card,
            targets: Vec::new(),
        }],
        protected,
        trick_effect::YOUDISHENRU,
        protected,
        &saved,
    )
}

pub(crate) fn resume_youdishenru_incoming(
    tx: &mut Transaction,
    saved: &[u32],
    cancel_target: bool,
) -> Result<(), EffectError> {
    let (protected, source, incoming, kind, payload) = parse_youdishenru_saved(saved)?;
    match kind {
        crate::state::ContinuationKind::ResumePlayedSha
        | crate::state::ContinuationKind::ResumePlayedShaQueue => {
            {
                let resolution = &mut tx.state_mut().resolution;
                resolution.resume_kind = kind;
                resolution.resume_payload = payload.to_vec();
            }
            if cancel_target
                && !tx
                    .state_mut()
                    .resolution
                    .exclude_saved_sha_target(incoming, protected)
            {
                return Err(EffectError::EngineFault(
                    "诱敌深入 lost its incoming Sha target".into(),
                ));
            }
            Ok(())
        }
        crate::state::ContinuationKind::RespondToSha => {
            if payload.len() < 8 {
                return Err(EffectError::EngineFault(
                    "bad saved single-target Sha response".into(),
                ));
            }
            if cancel_target {
                tx.state_mut().resolution.resume_kind =
                    crate::state::ContinuationKind::FinishPlayedSha;
                tx.state_mut().resolution.resume_payload =
                    vec![source.raw(), incoming.raw(), payload[5]];
            } else {
                tx.state_mut().resolution.resume_kind = kind;
                tx.state_mut().resolution.resume_payload = payload.to_vec();
            }
            Ok(())
        }
        crate::state::ContinuationKind::RespondAoeQueue => {
            let mut next = payload.to_vec();
            if next.first() != Some(&3) || next.len() < 8 {
                return Err(EffectError::EngineFault(
                    "bad saved multi-target Sha response".into(),
                ));
            }
            if cancel_target {
                next[3] = next[3].saturating_add(1);
                next[7] = 0;
            }
            tx.state_mut().resolution.resume_kind = crate::state::ContinuationKind::RespondAoeQueue;
            tx.state_mut().resolution.resume_payload = next;
            Ok(())
        }
        _ => Err(EffectError::EngineFault(
            "诱敌深入 cannot resume this continuation".into(),
        )),
    }
}

pub(crate) fn begin_youdishenru_force_loop(
    tx: &mut Transaction,
    protected: PlayerId,
    source: PlayerId,
    saved: &[u32],
) -> Result<(), EffectError> {
    let both_alive = tx
        .state()
        .players
        .get(protected)
        .is_some_and(|player| player.alive)
        && tx
            .state()
            .players
            .get(source)
            .is_some_and(|player| player.alive && !player.dying);
    if !both_alive {
        return resume_youdishenru_incoming(tx, saved, true);
    }
    let options: Vec<_> = prompted_sha_source_cards(tx.state(), source)
        .into_iter()
        .filter(|card| prompted_sha_targets_unlimited(tx, source, *card).contains(&protected))
        .map(CardId::raw)
        .collect();
    if options.is_empty() {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ContinueYoudishenru {
                action: 2,
                protected,
                source,
                forced_card: CardId::NONE,
                saved: saved.to_vec(),
            });
        return run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: protected,
                target: source,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .map(|_| ());
    }
    let mut payload = vec![
        YOUDISHENRU_FORCE_CHOICE,
        protected.raw(),
        source.raw(),
        saved.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend_from_slice(saved);
    crate::effect::open_choice(
        tx,
        source,
        0,
        1,
        true,
        options,
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::Custom,
        payload,
    )?;
    Err(EffectError::ChoiceRequested)
}

pub(crate) fn resolve_youdishenru_force_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let protected = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let source = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let len = *payload.get(3).unwrap_or(&u32::MAX) as usize;
    let saved = payload
        .get(4..4usize.saturating_add(len))
        .ok_or_else(|| EffectError::Illegal("bad 诱敌深入 forced-Sha choice".into()))?;
    if payload.first() != Some(&YOUDISHENRU_FORCE_CHOICE)
        || payload.len() != 4 + len
        || observer != source
        || selections.len() > 1
        || (cancelled && !selections.is_empty())
    {
        return Err(EffectError::Illegal(
            "stale 诱敌深入 forced-Sha choice".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ContinueYoudishenru {
                action: 2,
                protected,
                source,
                forced_card: CardId::NONE,
                saved: saved.to_vec(),
            });
        return run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: protected,
                target: source,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .map(|_| ());
    }
    let forced_card = CardId::new(selections[0]);
    if !prompted_sha_source_cards(tx.state(), source).contains(&forced_card)
        || !prompted_sha_targets_unlimited(tx, source, forced_card).contains(&protected)
    {
        return Err(EffectError::Illegal("invalid 诱敌深入 forced Sha".into()));
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ContinueYoudishenru {
            action: 1,
            protected,
            source,
            forced_card,
            saved: saved.to_vec(),
        });
    play_prompted_sha_source_unlimited(tx, source, forced_card, &[protected])
}

pub(crate) fn continue_youdishenru(
    tx: &mut Transaction,
    action: u8,
    protected: PlayerId,
    source: PlayerId,
    forced_card: CardId,
    saved: &[u32],
) -> Result<(), EffectError> {
    match action {
        0 => {
            let (saved_protected, saved_source, incoming, saved_kind, saved_payload) =
                parse_youdishenru_saved(saved)?;
            if saved_protected != protected || saved_source != source {
                return Err(EffectError::EngineFault(
                    "诱敌深入 virtual response context changed".into(),
                ));
            }
            let card =
                crate::effect::materialize_virtual_card(tx, protected, CardKind::TrickYoudishenru)?;
            use_youdishenru_for_saved_sha(
                tx,
                protected,
                source,
                incoming,
                card,
                saved_kind,
                saved_payload,
            )
        }
        1 => {
            let damaged = tx
                .state()
                .resolution
                .pending_card_resolutions
                .iter()
                .rev()
                .find(|record| record.source == source && record.card == forced_card)
                .is_some_and(|record| record.damaged_targets.contains(&protected));
            if damaged {
                resume_youdishenru_incoming(tx, saved, true)
            } else {
                begin_youdishenru_force_loop(tx, protected, source, saved)
            }
        }
        2 => resume_youdishenru_incoming(tx, saved, true),
        _ => Err(EffectError::EngineFault(
            "unknown 诱敌深入 continuation action".into(),
        )),
    }
}

/// Cards that are or can be converted to 无懈可击.
pub(crate) fn wuxie_tokens(state: &crate::state::GameState, player: PlayerId) -> Vec<u32> {
    wuxie_tokens_for_card(state, player, CardId::NONE)
}

pub(crate) fn wuxie_tokens_for_card(
    state: &crate::state::GameState,
    player: PlayerId,
    root_card: CardId,
) -> Vec<u32> {
    let mut out = response_tokens(state, player, "wuxie", |kind| kind == CardKind::TrickWuxie);
    let colors = crate::query::potsongyan_response_colors(state, root_card, player);
    if colors == 0 {
        return out;
    }
    for zone_kind in [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Expansion] {
        let Some(zone) = state
            .zones
            .find(zone_kind, player)
            .and_then(|id| state.zones.get(id))
        else {
            continue;
        };
        for card in &zone.cards {
            if !out.contains(&card.raw())
                && !crate::query::card_color_locked_for_card(state, player, *card)
                && !crate::query::response_color_locked_for_current_card(state, player, *card)
                && crate::query::effective_card_color_mask(state, player, *card) & colors != 0
            {
                out.push(card.raw());
            }
        }
    }
    out
}

pub(crate) fn wuxie_tokens_for_payload(
    state: &crate::state::GameState,
    player: PlayerId,
    payload: &[u32],
) -> Vec<u32> {
    let ask_len = payload.get(2).copied().unwrap_or(0) as usize;
    let effect_off = 3usize.saturating_add(ask_len);
    wuxie_tokens_for_card(
        state,
        player,
        effect_args_source_card(payload.get(effect_off + 5..).unwrap_or_default()),
    )
}

/// Seat-ordered alive players starting from `start` (inclusive).
pub(crate) fn alive_from_seat(tx: &Transaction, start: PlayerId) -> Vec<PlayerId> {
    let order = tx.state().players.alive_by_seat();
    if order.is_empty() {
        return order;
    }
    let pos = order.iter().position(|p| *p == start).unwrap_or(0);
    let mut out = order[pos..].to_vec();
    out.extend_from_slice(&order[..pos]);
    out
}

/// Open 无懈 window before deferred trick effect resolves.
///
/// payload: `[nullified, ask_idx, ask_len, p0.., effect_tag, actor, respond_to, response_kind,
/// response_suit, ...args]`
pub(crate) fn begin_wuxie_window(
    tx: &mut Transaction,
    ask_start: PlayerId,
    effect_tag: u32,
    actor: PlayerId,
    response_kind: CardKind,
    response_suit: engine_api::Suit,
    effect_args: &[u32],
) -> Result<(), EffectError> {
    begin_wuxie_window_with_parity(
        tx,
        ask_start,
        0,
        effect_tag,
        actor,
        actor,
        response_kind,
        response_suit,
        effect_args,
    )
}

/// Like [`begin_wuxie_window`] but carries current nullify parity (after a 无懈).
pub(crate) fn begin_wuxie_window_with_parity(
    tx: &mut Transaction,
    ask_start: PlayerId,
    nullified: u32,
    effect_tag: u32,
    actor: PlayerId,
    respond_to: PlayerId,
    response_kind: CardKind,
    response_suit: engine_api::Suit,
    effect_args: &[u32],
) -> Result<(), EffectError> {
    let askers = alive_from_seat(tx, ask_start);
    let mut payload = vec![nullified & 1, 0, askers.len() as u32];
    payload.extend(askers.iter().map(|p| p.raw()));
    payload.push(effect_tag);
    payload.push(actor.raw());
    payload.push(respond_to.raw());
    payload.push(response_kind.tag() as u32);
    payload.push(super::super::judge::suit_tag(response_suit));
    payload.extend_from_slice(effect_args);
    tx.state_mut().resolution.continuation_kind = crate::state::ContinuationKind::WuxieWindow;
    tx.state_mut().resolution.continuation_payload = payload;
    advance_wuxie_window(tx)
}

/// Advance / open next 无懈 ask, or finish into deferred effect.
pub(crate) fn advance_wuxie_window(tx: &mut Transaction) -> Result<(), EffectError> {
    loop {
        let payload = tx.state().resolution.continuation_payload.clone();
        if payload.len() < 5 {
            tx.state_mut().resolution.clear_choice();
            return Ok(());
        }
        let ask_idx = payload[1] as usize;
        let ask_len = payload[2] as usize;
        if ask_idx >= ask_len || 3 + ask_len > payload.len() {
            return finish_wuxie_window(tx, &payload);
        }
        let player = PlayerId::new(payload[3 + ask_idx]);
        let alive = tx
            .state()
            .players
            .get(player)
            .map(|p| p.alive && !p.dying)
            .unwrap_or(false);
        if !alive {
            tx.state_mut().resolution.continuation_payload[1] = (ask_idx + 1) as u32;
            continue;
        }
        let effect_off = 3 + ask_len;
        let respond_to = PlayerId::new(*payload.get(effect_off + 2).unwrap_or(&0));
        let response_kind = payload
            .get(effect_off + 3)
            .and_then(|tag| CardKind::from_tag(*tag as u8))
            .unwrap_or(CardKind::Other);
        let response_suit =
            super::super::judge::suit_from_tag(*payload.get(effect_off + 4).unwrap_or(&1));
        let source_card =
            effect_args_source_card(payload.get(effect_off + 5..).unwrap_or_default());
        // tanhu3（批 170）：独立锦囊队列的 effect_args 布局统一
        // `[MAGIC, card, actor, kind, suit, effect_tag, 0, len, targets...]`——
        // 目标集合含「拥有被声明技能且挂有对应声明」的角色时，无懈不可响应。
        let wuxie_immune_targets = (|| {
            let args = payload.get(effect_off + 5..).unwrap_or_default();
            if args.first().copied() != Some(crate::command::play::INDEPENDENT_TRICK_QUEUE_MAGIC) {
                return None;
            }
            let len = *args.get(7).unwrap_or(&0) as usize;
            let targets: Vec<_> = args
                .get(8..8 + len)?
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            Some(targets)
        })();
        let tokens = if crate::query::card_use_unrespondable_for_kind(
            tx.state(),
            respond_to,
            response_kind,
            response_suit,
        ) || crate::query::card_use_unrespondable_within_distance(
            tx.state(),
            respond_to,
            player,
        ) || crate::query::potshishi_blocks_response(
            tx.state(),
            respond_to,
            source_card,
            player,
        ) {
            Vec::new()
        } else if let Some(targets) = wuxie_immune_targets {
            if crate::query::wuxie_blocked_by_target_skill_declarations(tx.state(), &targets) {
                Vec::new()
            } else {
                wuxie_tokens_for_card(tx.state(), player, source_card)
            }
        } else {
            wuxie_tokens_for_card(tx.state(), player, source_card)
        };
        if tokens.is_empty() {
            tx.state_mut().resolution.continuation_payload[1] = (ask_idx + 1) as u32;
            continue;
        }
        let mut p = payload;
        p[1] = ask_idx as u32;
        crate::effect::open_choice(
            tx,
            player,
            0,
            1,
            true,
            tokens,
            engine_api::ChoiceKind::SelectCards,
            crate::state::ContinuationKind::WuxieWindow,
            p,
        )?;
        return Err(EffectError::ChoiceRequested);
    }
}

pub(crate) fn finish_wuxie_window(
    tx: &mut Transaction,
    payload: &[u32],
) -> Result<(), EffectError> {
    let nullified = *payload.first().unwrap_or(&0);
    let ask_len = *payload.get(2).unwrap_or(&0) as usize;
    let effect_off = 3 + ask_len;
    let effect_tag = *payload.get(effect_off).unwrap_or(&0);
    let actor = PlayerId::new(*payload.get(effect_off + 1).unwrap_or(&0));
    let args = if payload.len() > effect_off + 5 {
        &payload[effect_off + 5..]
    } else {
        &[]
    };
    let marker = args
        .iter()
        .rposition(|value| *value == SAVED_WUXIE_SOURCE_CARD_MAGIC);
    let (effect_args, source_card) = marker.map_or((args, CardId::NONE), |index| {
        (
            &args[..index],
            CardId::new(*args.get(index + 1).unwrap_or(&0)),
        )
    });

    tx.state_mut().resolution.continuation_kind = crate::state::ContinuationKind::None;
    tx.state_mut().resolution.continuation_payload.clear();
    tx.state_mut().resolution.clear_choice();

    if nullified % 2 == 1 {
        run_effects(
            tx,
            vec![EffectRequest::EmitRuleEvent {
                code: 220, // trick nullified by 无懈
                payload: vec![effect_tag, actor.raw()],
            }],
        )?;
        if effect_tag == trick_effect::YOUDISHENRU {
            return resume_youdishenru_incoming(tx, effect_args, false);
        }
        finish_independent_trick_target(tx, effect_args)?;
        return crate::effect::finish_pending_card_replays_if_idle(tx);
    }
    apply_deferred_trick_with_extras(tx, effect_tag, actor, effect_args, source_card)?;
    crate::effect::finish_pending_card_replays_if_idle(tx)
}

/// Apply trick effect after 无懈 window clears without odd nullify, honoring
/// queued 额外结算 (extra_effect_count) passes (clanfenshi family, ABI 649).
pub(crate) fn apply_deferred_trick_with_extras(
    tx: &mut Transaction,
    effect_tag: u32,
    actor: PlayerId,
    args: &[u32],
    source_card: CardId,
) -> Result<(), EffectError> {
    apply_deferred_trick(tx, effect_tag, actor, args, source_card)?;
    let extras = tx
        .state()
        .extra_effect_count
        .get(&source_card.raw())
        .copied()
        .unwrap_or(0);
    for _ in 0..extras.min(2) {
        apply_deferred_trick(tx, effect_tag, actor, args, source_card)?;
    }
    if source_card.is_some() {
        tx.state_mut().extra_effect_count.remove(&source_card.raw());
    }
    Ok(())
}
