//! Command operations for `complete_view_as_response` and related helpers.

use super::*;

pub(crate) fn complete_view_as_response(
    tx: &mut Transaction,
    payload: &[u32],
    physical_sources: &[CardId],
    multi_card_bonus: Option<&str>,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(1).unwrap_or(&0));
    let kind = payload
        .get(2)
        .and_then(|tag| CardKind::from_tag(*tag as u8))
        .ok_or_else(|| EffectError::EngineFault("bad source-free response kind".into()))?;
    let action = *payload.get(3).unwrap_or(&0);
    let original = payload.get(4..).unwrap_or_default();
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .unwrap_or(0);
    let response_card = physical_sources.first().copied().unwrap_or(CardId::NONE);
    let fusi_provider = physical_sources
        .first()
        .and_then(|card| crate::command::skill::dddfusi_approved_source(tx.state(), player, *card));
    let post_draw = crate::command::skill::source_free_post_draw(tx.state(), skill, kind);
    let view_events = |tx: &Transaction| -> Result<Vec<EffectRequest>, EffectError> {
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
        let mut effects = Vec::new();
        for source in physical_sources {
            effects.push(EffectRequest::MoveCardsWithReason {
                cards: vec![*source],
                to: discard,
                from: Some(response_card_zone(tx, player, *source)?),
                reason: CardMoveReason::Respond,
            });
        }
        append_pothaoshi_refills(tx.state(), player, physical_sources, &mut effects);
        effects.push(EffectRequest::EmitRuleEvent {
            code: 120,
            payload: vec![
                skill.raw(),
                skill_def,
                player.raw(),
                response_card.raw(),
                kind.tag() as u32,
            ],
        });
        if let Some(provider) = fusi_provider {
            let fusi_skill = tx
                .state()
                .players
                .get(provider)
                .and_then(|participant| {
                    participant.skills.iter().copied().find(|owned| {
                        tx.state().skills.get(*owned).is_some_and(|instance| {
                            instance.def.raw() == crate::command::skill::DDD_FUSI_DEF
                        })
                    })
                })
                .unwrap_or(engine_api::SkillId::NONE);
            effects.push(EffectRequest::ResolveDddXianglang {
                player: provider,
                actor: player,
                action: 2,
                skill: fusi_skill,
            });
        }
        Ok(effects)
    };
    let append_multi_card_bonus = |tx: &Transaction, effects: &mut Vec<EffectRequest>| {
        if multi_card_bonus != Some("discard_current_hand_equip") {
            return;
        }
        let current = tx.state().turn.current;
        if current != player
            && tx
                .state()
                .players
                .get(current)
                .is_some_and(|participant| participant.alive)
            && !crate::query::discardable_target_cards(
                tx.state(),
                player,
                current,
                rule_api::TargetCardScope::HandEquip,
            )
            .is_empty()
        {
            effects.push(EffectRequest::ChooseDiscardTargetCard {
                player,
                target: current,
                scope: rule_api::TargetCardScope::HandEquip,
            });
        }
    };
    let append_post_draw = |effects: &mut Vec<EffectRequest>| {
        if post_draw > 0 {
            effects.push(EffectRequest::DrawCards {
                player,
                count: u32::from(post_draw),
            });
        }
    };

    tx.state_mut().turn.set_flag(
        crate::state::turn_flag::VIEW_AS_CARD,
        response_card.raw() as i32,
    );
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
    tx.state_mut().turn.set_flag(
        crate::state::turn_flag::VIEW_AS_LOGICAL_KIND,
        i32::from(kind.tag()),
    );
    let result = (|| {
        match action {
            7 => {
                let owner = PlayerId::new(*original.first().unwrap_or(&0));
                let mengmou = engine_api::SkillId::new(*original.get(1).unwrap_or(&0));
                let mut effects = view_events(tx)?;
                effects.push(EffectRequest::CardResponded {
                    player,
                    respond_to: owner,
                    card: response_card,
                    response_kind: CardKind::Sha.tag(),
                });
                append_post_draw(&mut effects);
                append_multi_card_bonus(tx, &mut effects);
                effects.push(EffectRequest::ResolveDcSbLusu {
                    player: owner,
                    source: player,
                    card: response_card,
                    action: 22,
                    cards: Vec::new(),
                    amount: 0,
                    skill: mengmou,
                });
                run_response_effects(tx, effects, ContinuationKind::None, Vec::new())
            }
            1 => {
                // RespondToSha payload: [source, target, amount, need, have, context, respondable, card]
                let source = PlayerId::new(*original.first().unwrap_or(&0));
                let target = PlayerId::new(*original.get(1).unwrap_or(&0));
                let amount = *original.get(2).unwrap_or(&1);
                let need = *original.get(3).unwrap_or(&1);
                let have = original.get(4).copied().unwrap_or(0).saturating_add(1);
                let context = *original.get(5).unwrap_or(&0);
                let respondable = *original.get(6).unwrap_or(&1);
                let card = *original.get(7).unwrap_or(&0);
                let resume = if have < need {
                    vec![
                        source.raw(),
                        target.raw(),
                        amount,
                        need,
                        have,
                        context,
                        respondable,
                        card,
                    ]
                } else {
                    Vec::new()
                };
                let mut response_effects = view_events(tx)?;
                response_effects.extend(sha_response_effects(
                    target,
                    source,
                    response_card,
                    CardId::new(card),
                    have,
                    need,
                ));
                response_effects.push(EffectRequest::EmitRuleEvent {
                    code: 121,
                    payload: vec![source.raw(), target.raw(), response_card.raw(), have, need],
                });
                append_post_draw(&mut response_effects);
                append_multi_card_bonus(tx, &mut response_effects);
                if have >= need {
                    crate::command::play::save_played_sha_finish(
                        tx,
                        source,
                        CardId::new(card),
                        context,
                    )?;
                }
                run_response_effects(tx, response_effects, ContinuationKind::RespondToSha, resume)?;
                if have < need {
                    let (nature, card_flags) =
                        crate::command::play::unpack_damage_card_context(context);
                    return crate::command::play::open_sha_response_step(
                        tx,
                        source,
                        target,
                        amount,
                        need,
                        have,
                        nature,
                        card_flags,
                        respondable != 0,
                        CardId::new(card),
                    );
                }
                crate::command::play::finish_saved_played_sha(tx, source, CardId::new(card))
            }
            2 => {
                // RespondAoeQueue, including its kind=3 multi-target Sha form.
                let queue_kind = *original.first().unwrap_or(&0);
                let source = PlayerId::new(*original.get(1).unwrap_or(&0));
                let idx = *original.get(3).unwrap_or(&0) as usize;
                if queue_kind == 3 {
                    let have = original.get(7).copied().unwrap_or(0).saturating_add(1);
                    let need = *original.get(6).unwrap_or(&1);
                    let target = PlayerId::new(*original.get(8 + idx).unwrap_or(&0));
                    let len = *original.get(4).unwrap_or(&0) as usize;
                    let card = CardId::new(*original.get(8 + 2 * len).unwrap_or(&0));
                    let next = multi_sha_queue_next(original, idx, have);
                    let mut response_effects = view_events(tx)?;
                    response_effects.extend(sha_response_effects(
                        target,
                        source,
                        response_card,
                        card,
                        have,
                        need,
                    ));
                    append_post_draw(&mut response_effects);
                    append_multi_card_bonus(tx, &mut response_effects);
                    run_response_effects(
                        tx,
                        response_effects,
                        ContinuationKind::RespondAoeQueue,
                        next.clone(),
                    )?;
                    return continue_response_queue(tx, next);
                }
                let target = PlayerId::new(*original.get(5 + idx).unwrap_or(&0));
                let response_kind = if queue_kind == 1 {
                    CardKind::Sha
                } else {
                    CardKind::Shan
                };
                let mut next = original.to_vec();
                if next.len() > 3 {
                    next[3] = (idx + 1) as u32;
                }
                let mut response_effects = view_events(tx)?;
                response_effects.push(EffectRequest::CardResponded {
                    player: target,
                    respond_to: source,
                    card: response_card,
                    response_kind: response_kind.tag(),
                });
                response_effects.push(EffectRequest::EmitRuleEvent {
                    code: 123,
                    payload: vec![queue_kind, source.raw(), target.raw(), response_card.raw()],
                });
                append_post_draw(&mut response_effects);
                append_multi_card_bonus(tx, &mut response_effects);
                run_response_effects(
                    tx,
                    response_effects,
                    ContinuationKind::RespondAoeQueue,
                    next.clone(),
                )?;
                tx.state_mut().resolution.continuation_kind = ContinuationKind::RespondAoeQueue;
                tx.state_mut().resolution.continuation_payload = next;
                crate::command::play::advance_aoe_queue(tx)
            }
            3 => {
                // JuedouDuel payload:
                // [source, target, next_responder, exchange_count, need, have, tail...]
                let source = PlayerId::new(*original.first().unwrap_or(&0));
                let target = PlayerId::new(*original.get(1).unwrap_or(&0));
                let next = PlayerId::new(*original.get(2).unwrap_or(&0));
                let exchanges = *original.get(3).unwrap_or(&0);
                let need = *original.get(4).unwrap_or(&1);
                let have = *original.get(5).unwrap_or(&0);
                let tail = original.get(6..).unwrap_or_default();
                let resume = crate::command::play::juedou_payload_after_response(
                    source, target, next, exchanges, need, have, tail,
                );
                let mut response_effects = view_events(tx)?;
                response_effects.push(EffectRequest::CardResponded {
                    player: next,
                    respond_to: source,
                    card: response_card,
                    response_kind: CardKind::Sha.tag(),
                });
                response_effects.push(EffectRequest::EmitRuleEvent {
                    code: 124,
                    payload: vec![source.raw(), target.raw(), next.raw(), response_card.raw()],
                });
                append_post_draw(&mut response_effects);
                run_response_effects(
                    tx,
                    response_effects,
                    ContinuationKind::JuedouDuel,
                    resume.clone(),
                )?;
                crate::command::play::continue_juedou_payload(tx, &resume)
            }
            4 => {
                // WuxieWindow payload is preserved verbatim from the original ask.
                let ask_len = *original.get(2).unwrap_or(&0) as usize;
                let effect_off = 3 + ask_len;
                let nullified = original.first().copied().unwrap_or(0) ^ 1;
                let effect_tag = *original.get(effect_off).unwrap_or(&0);
                let actor = PlayerId::new(*original.get(effect_off + 1).unwrap_or(&0));
                let respond_to = PlayerId::new(*original.get(effect_off + 2).unwrap_or(&0));
                let effect_args = original.get(effect_off + 5..).unwrap_or_default().to_vec();
                let order = tx.state().players.alive_by_seat();
                let pos = order
                    .iter()
                    .position(|candidate| *candidate == player)
                    .unwrap_or(0);
                let start = if order.is_empty() {
                    player
                } else {
                    order[(pos + 1) % order.len()]
                };
                let response_suit = tx
                    .state()
                    .cards
                    .get(response_card)
                    .map(|card| crate::query::effective_card_suit(tx.state(), player, card))
                    .unwrap_or(engine_api::Suit::None);
                let mut resume = vec![
                    start.raw(),
                    nullified,
                    effect_tag,
                    actor.raw(),
                    player.raw(),
                    CardKind::TrickWuxie.tag() as u32,
                    crate::command::judge::suit_tag(response_suit),
                ];
                resume.extend_from_slice(&effect_args);
                let mut response_effects = view_events(tx)?;
                response_effects.push(EffectRequest::CardResponded {
                    player,
                    respond_to,
                    card: response_card,
                    response_kind: CardKind::TrickWuxie.tag(),
                });
                response_effects.push(EffectRequest::EmitRuleEvent {
                    code: 221,
                    payload: vec![player.raw(), response_card.raw()],
                });
                append_post_draw(&mut response_effects);
                append_multi_card_bonus(tx, &mut response_effects);
                run_response_effects(tx, response_effects, ContinuationKind::WuxieWindow, resume)?;
                crate::command::play::begin_wuxie_window_with_parity(
                    tx,
                    start,
                    nullified,
                    effect_tag,
                    actor,
                    player,
                    CardKind::TrickWuxie,
                    response_suit,
                    &effect_args,
                )
            }
            5 => {
                // DyingSave payload: [dying, killer, responder, rest_len, rest...].
                let dying = PlayerId::new(*original.first().unwrap_or(&0));
                let killer = PlayerId::new(*original.get(1).unwrap_or(&0));
                let responder = PlayerId::new(*original.get(2).unwrap_or(&0));
                let len = *original.get(3).unwrap_or(&u32::MAX) as usize;
                let remaining = original
                    .get(4..)
                    .filter(|rest| rest.len() == len)
                    .ok_or_else(|| {
                        EffectError::EngineFault("truncated source-free dying-save queue".into())
                    })?;
                if player != responder
                    || crate::query::card_save_cancelled_for_kind_to(
                        tx.state(),
                        player,
                        dying,
                        kind,
                    )
                    || (kind == CardKind::Jiu && player != dying)
                {
                    return Err(EffectError::Illegal("card save disabled".into()));
                }
                let mut effects = view_events(tx)?;
                effects.push(EffectRequest::CardResponded {
                    player,
                    respond_to: dying,
                    card: response_card,
                    response_kind: kind.tag(),
                });
                append_post_draw(&mut effects);
                if tx
                    .state()
                    .players
                    .get(dying)
                    .is_some_and(|state| state.hp <= 0)
                {
                    effects.push(EffectRequest::RecoverHp {
                        source: player,
                        player: dying,
                        amount: 1 + u32::from(multi_card_bonus == Some("recover_1")),
                    });
                }
                let mut responders = vec![responder];
                responders.extend(remaining.iter().copied().map(PlayerId::new));
                effects.push(EffectRequest::ContinueDyingSave {
                    player: dying,
                    source: killer,
                    responders,
                });
                run_effects(tx, effects)?;
                resume_interrupted_continuation(tx)
            }
            6 => {
                // Proactive delegated Sha payload: [target_count, targets...].
                if kind != CardKind::Sha {
                    return Err(EffectError::ContentFault(
                        "proactive delegated card must be Sha".into(),
                    ));
                }
                let target_count = *original.first().unwrap_or(&u32::MAX) as usize;
                let targets = original
                    .get(1..)
                    .filter(|targets| targets.len() == target_count)
                    .ok_or_else(|| {
                        EffectError::EngineFault("truncated proactive delegated Sha targets".into())
                    })?;
                let mut play = vec![
                    player.raw(),
                    skill.raw(),
                    kind.tag() as u32,
                    target_count as u32,
                ];
                play.extend_from_slice(targets);
                crate::command::skill::resume_source_free_play(tx, &play)
            }
            _ => Err(EffectError::EngineFault(
                "unknown source-free response continuation".into(),
            )),
        }
    })();
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_CARD, 0);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, 0);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    result
}

pub(crate) fn begin_delegated_group_response(
    tx: &mut Transaction,
    owner: PlayerId,
    skill: engine_api::SkillId,
    kind: CardKind,
    action: u32,
    action_payload: &[u32],
    group: &str,
) -> Result<(), EffectError> {
    let providers =
        crate::command::skill::delegated_response_candidates(tx.state(), owner, group, kind);
    if providers.is_empty() {
        return Err(EffectError::Illegal(
            "no group member can provide the requested response".into(),
        ));
    }
    let mut payload = vec![
        owner.raw(),
        skill.raw(),
        kind.tag() as u32,
        action,
        0,
        providers.len() as u32,
    ];
    payload.extend(providers.iter().map(|player| player.raw()));
    payload.extend_from_slice(action_payload);
    open_delegated_group_response(tx, payload)
}

pub(crate) fn resolve_delegated_group_response(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let (_, _, kind, provider_index, providers, _) = delegated_response_payload(payload)?;
    let provider = *providers
        .get(provider_index)
        .ok_or_else(|| EffectError::Illegal("delegated response provider is missing".into()))?;
    if observer != provider {
        return Err(EffectError::Illegal(
            "only the current provider may resolve delegated response".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        let mut next = payload.to_vec();
        next[4] = (provider_index + 1) as u32;
        return open_delegated_group_response(tx, next);
    }
    if selections.len() != 1 || !matches!(kind, CardKind::Sha | CardKind::Shan) {
        return Err(EffectError::Illegal(
            "delegated response requires one card".into(),
        ));
    }
    let token = selections[0];
    let legal = match kind {
        CardKind::Sha => crate::command::play::delegated_sha_options(tx.state(), provider),
        CardKind::Shan => crate::command::play::delegated_shan_options(tx.state(), provider),
        _ => Vec::new(),
    };
    if !legal.contains(&token) {
        return Err(EffectError::Illegal(
            "not a legal delegated card response".into(),
        ));
    }
    if kind == CardKind::Shan && token == crate::content::BAGUA_RESPONSE_TOKEN {
        let (ok, judgment) = crate::command::play::try_bagua_as_shan(tx, provider)?;
        if ok {
            return complete_delegated_group_response(
                tx,
                payload,
                vec![
                    judgment,
                    EffectRequest::CardResponded {
                        player: provider,
                        respond_to: PlayerId::new(payload[0]),
                        card: CardId::NONE,
                        response_kind: CardKind::Shan.tag(),
                    },
                ],
            );
        }
        let mut next = payload.to_vec();
        next[4] = (provider_index + 1) as u32;
        run_response_effects(
            tx,
            vec![judgment],
            ContinuationKind::DelegatedGroupResponse,
            next.clone(),
        )?;
        return open_delegated_group_response(tx, next);
    }
    let card = CardId::new(token);
    let from = response_card_zone(tx, provider, card)?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
    let mut effects = vec![EffectRequest::MoveCardsWithReason {
        cards: vec![card],
        to: discard,
        from: Some(from),
        reason: CardMoveReason::Respond,
    }];
    append_pothaoshi_refills(tx.state(), provider, &[card], &mut effects);
    effects.push(EffectRequest::CardResponded {
        player: provider,
        respond_to: PlayerId::new(payload[0]),
        card,
        response_kind: kind.tag(),
    });
    effects.push(EffectRequest::EmitRuleEvent {
        code: 126,
        payload: vec![payload[0], provider.raw(), card.raw(), kind.tag() as u32],
    });
    complete_delegated_group_response(tx, payload, effects)
}

pub(crate) fn delegated_response_payload(
    payload: &[u32],
) -> Result<
    (
        PlayerId,
        engine_api::SkillId,
        CardKind,
        usize,
        Vec<PlayerId>,
        &[u32],
    ),
    EffectError,
> {
    let provider_count = *payload
        .get(5)
        .ok_or_else(|| EffectError::EngineFault("truncated delegated response".into()))?
        as usize;
    let original_offset = 6 + provider_count;
    if payload.len() < original_offset {
        return Err(EffectError::EngineFault(
            "truncated delegated response providers".into(),
        ));
    }
    let kind = payload
        .get(2)
        .and_then(|tag| CardKind::from_tag(*tag as u8))
        .ok_or_else(|| EffectError::EngineFault("bad delegated response kind".into()))?;
    Ok((
        PlayerId::new(payload[0]),
        engine_api::SkillId::new(payload[1]),
        kind,
        payload[4] as usize,
        payload[6..original_offset]
            .iter()
            .copied()
            .map(PlayerId::new)
            .collect(),
        &payload[original_offset..],
    ))
}

pub(crate) fn open_delegated_group_response(
    tx: &mut Transaction,
    mut payload: Vec<u32>,
) -> Result<(), EffectError> {
    loop {
        let (owner, _, kind, provider_index, providers, _) = delegated_response_payload(&payload)?;
        let Some(provider) = providers.get(provider_index).copied() else {
            return cancel_delegated_group_response(tx, &payload);
        };
        let options = match kind {
            CardKind::Sha => crate::command::play::delegated_sha_options(tx.state(), provider),
            CardKind::Shan => crate::command::play::delegated_shan_options(tx.state(), provider),
            _ => Vec::new(),
        };
        if options.is_empty()
            || provider == owner
            || !tx
                .state()
                .players
                .get(provider)
                .is_some_and(|player| player.alive)
        {
            payload[4] = (provider_index + 1) as u32;
            continue;
        }
        crate::effect::open_choice(
            tx,
            provider,
            0,
            1,
            true,
            options,
            engine_api::ChoiceKind::SelectCards,
            ContinuationKind::DelegatedGroupResponse,
            payload,
        )?;
        return Err(EffectError::ChoiceRequested);
    }
}

pub(crate) fn complete_delegated_group_response(
    tx: &mut Transaction,
    payload: &[u32],
    effects: Vec<EffectRequest>,
) -> Result<(), EffectError> {
    let (owner, skill, kind, _, _, original) = delegated_response_payload(payload)?;
    let action = payload[3];
    let mut resume = vec![owner.raw(), skill.raw(), kind.tag() as u32, action];
    resume.extend_from_slice(original);
    run_response_effects(
        tx,
        effects,
        ContinuationKind::SourceFreeViewAsResponse,
        resume.clone(),
    )?;
    resume_source_free_response(tx, &resume)
}

pub(crate) fn cancel_delegated_group_response(
    tx: &mut Transaction,
    payload: &[u32],
) -> Result<(), EffectError> {
    let (_, skill, _, _, _, original) = delegated_response_payload(payload)?;
    match payload[3] {
        1 => {
            let source = PlayerId::new(*original.first().unwrap_or(&0));
            let target = PlayerId::new(*original.get(1).unwrap_or(&0));
            let amount = *original.get(2).unwrap_or(&1);
            let (nature, card_flags) =
                crate::command::play::unpack_damage_card_context(*original.get(5).unwrap_or(&0));
            let card = CardId::new(*original.get(7).unwrap_or(&0));
            crate::command::play::deal_single_sha_damage(
                tx, source, target, amount, nature, card_flags, card,
            )
        }
        2 if original.first() == Some(&3) => {
            resolve_multi_sha_queue_choice(tx, original, &[], true)
        }
        2 => cancel_ordinary_aoe_response(tx, original),
        3 => {
            let source = PlayerId::new(*original.first().unwrap_or(&0));
            let target = PlayerId::new(*original.get(1).unwrap_or(&0));
            let next = PlayerId::new(*original.get(2).unwrap_or(&0));
            let tail = original.get(6..).unwrap_or_default();
            let dmg_source = if next == target { source } else { target };
            let amount = crate::query::sha_damage_for(tx.state(), dmg_source, 1);
            crate::command::play::finish_juedou_with_tail(tx, source, target, next, amount, tail)
        }
        6 => {
            let revision = tx.state().revision.saturating_add(1);
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.set_delegated_retry_lock_revision(revision);
            }
            Ok(())
        }
        7 => {
            let owner = PlayerId::new(*original.first().unwrap_or(&0));
            let mengmou = engine_api::SkillId::new(*original.get(1).unwrap_or(&0));
            run_effects(
                tx,
                vec![EffectRequest::ResolveDcSbLusu {
                    player: owner,
                    source: owner,
                    card: CardId::NONE,
                    action: 23,
                    cards: Vec::new(),
                    amount: 0,
                    skill: mengmou,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::ContentFault(
            "unsupported delegated response context".into(),
        )),
    }
}

pub(crate) fn cancel_ordinary_aoe_response(
    tx: &mut Transaction,
    payload: &[u32],
) -> Result<(), EffectError> {
    let source = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let amount = *payload.get(2).unwrap_or(&1);
    let idx = *payload.get(3).unwrap_or(&0) as usize;
    let target = PlayerId::new(*payload.get(5 + idx).unwrap_or(&0));
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: 0,
        }],
    )?;
    if tx.state().resolution.continuation_kind == ContinuationKind::DyingSave {
        if tx.state().resolution.resume_kind == ContinuationKind::RespondAoeQueue
            && tx.state().resolution.resume_payload.len() > 3
        {
            tx.state_mut().resolution.resume_payload[3] = (idx + 1) as u32;
        }
        return Ok(());
    }
    let mut next = payload.to_vec();
    next[3] = (idx + 1) as u32;
    tx.state_mut().resolution.continuation_kind = ContinuationKind::RespondAoeQueue;
    tx.state_mut().resolution.continuation_payload = next;
    crate::command::play::advance_aoe_queue(tx)
}

pub(crate) fn run_response_effects(
    tx: &mut Transaction,
    effects: Vec<EffectRequest>,
    resume_kind: ContinuationKind,
    resume_payload: Vec<u32>,
) -> Result<(), EffectError> {
    match run_effects(tx, effects) {
        Ok(_) => Ok(()),
        Err(EffectError::ChoiceRequested) => {
            if resume_kind != ContinuationKind::None && !resume_payload.is_empty() {
                let resolution = &mut tx.state_mut().resolution;
                resolution.resume_kind = resume_kind;
                resolution.resume_payload = resume_payload;
            }
            Err(EffectError::ChoiceRequested)
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn run_physical_response_effects(
    tx: &mut Transaction,
    mut effects: Vec<EffectRequest>,
    resume_kind: ContinuationKind,
    resume_payload: Vec<u32>,
    player: PlayerId,
    card: CardId,
    response_kind: CardKind,
) -> Result<(), EffectError> {
    for effect in &mut effects {
        if matches!(effect, EffectRequest::MoveCards { cards, .. } if cards.contains(&card)) {
            let EffectRequest::MoveCards { cards, to, from } = effect.clone() else {
                unreachable!();
            };
            *effect = EffectRequest::MoveCardsWithReason {
                cards,
                to,
                from,
                reason: CardMoveReason::Respond,
            };
        }
    }
    let refills = crate::effect::pothaoshi_refill_effects(tx.state(), player, &[card]);
    if !refills.is_empty() {
        let insert_at = effects
            .iter()
            .rposition(|effect| match effect {
                EffectRequest::MoveCards { cards, .. }
                | EffectRequest::MoveCardsWithReason { cards, .. } => cards.contains(&card),
                _ => false,
            })
            .map_or(0, |index| index + 1);
        effects.splice(insert_at..insert_at, refills);
    }
    let view_skill = tx.state().cards.get(card).and_then(|card_state| {
        let physical_kind = crate::query::effective_card_kind(tx.state(), player, card_state);
        let natural = if response_kind == CardKind::Shan {
            physical_kind == CardKind::Shan
        } else {
            response_kind.is_sha() && physical_kind.is_sha()
        };
        (!natural)
            .then(|| {
                crate::command::play::view_as_response_skill(
                    tx.state(),
                    player,
                    card_state,
                    if response_kind == CardKind::Shan {
                        "shan"
                    } else {
                        "sha"
                    },
                )
            })
            .flatten()
    });
    if let Some(skill) = view_skill {
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::VIEW_AS_CARD, card.raw() as i32);
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
        tx.state_mut().turn.set_flag(
            crate::state::turn_flag::VIEW_AS_LOGICAL_KIND,
            i32::from(response_kind.tag()),
        );
    }
    let result = run_response_effects(tx, effects, resume_kind, resume_payload);
    if view_skill.is_some() {
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::VIEW_AS_CARD, 0);
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, 0);
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    }
    result
}

pub(crate) fn sha_response_effects(
    target: PlayerId,
    source: PlayerId,
    response_card: CardId,
    sha_card: CardId,
    have: u32,
    need: u32,
) -> Vec<EffectRequest> {
    let mut effects = vec![EffectRequest::CardResponded {
        player: target,
        respond_to: source,
        card: response_card,
        response_kind: CardKind::Shan.tag(),
    }];
    if have >= need {
        effects.push(EffectRequest::ShaDodged {
            source,
            target,
            card: sha_card,
        });
    }
    effects
}

pub(crate) fn resolve_multi_sha_queue_choice(
    tx: &mut Transaction,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 9 {
        return Err(EffectError::EngineFault(
            "malformed multi-target sha queue".into(),
        ));
    }
    let source = PlayerId::new(payload[1]);
    let idx = payload[3] as usize;
    let len = payload[4] as usize;
    let amount = payload[2].saturating_add(crate::command::play::sha_queue_target_damage_bonus(
        payload, 8, len, idx,
    ));
    let (nature, card_flags) = crate::command::play::unpack_damage_card_context(payload[5]);
    let need = payload[6];
    let have = payload[7];
    let card = CardId::new(*payload.get(8 + 2 * len).unwrap_or(&0));
    if idx >= len || 8 + idx >= payload.len() {
        return Err(EffectError::EngineFault(
            "multi-target sha queue index out of bounds".into(),
        ));
    }
    let target = PlayerId::new(payload[8 + idx]);

    if cancelled || selections.is_empty() {
        let next = multi_sha_queue_next(payload, idx, need);
        {
            let resolution = &mut tx.state_mut().resolution;
            resolution.resume_kind = ContinuationKind::RespondAoeQueue;
            resolution.resume_payload = next;
        }
        return match crate::command::play::deal_sha_damage(
            tx, source, target, amount, nature, card_flags, card,
        ) {
            Ok(_) => {
                let resolution = &mut tx.state_mut().resolution;
                let next = std::mem::take(&mut resolution.resume_payload);
                resolution.resume_kind = ContinuationKind::None;
                continue_response_queue(tx, next)
            }
            Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
            Err(error) => Err(error),
        };
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal("play at most one 闪".into()));
    }

    let token = selections[0];
    if have == 0
        && crate::command::play::response_youdishenru_tokens(tx.state(), target).contains(&token)
    {
        return crate::command::play::use_youdishenru_for_saved_sha(
            tx,
            target,
            source,
            card,
            CardId::new(token),
            ContinuationKind::RespondAoeQueue,
            payload,
        );
    }
    if token == crate::content::BAGUA_RESPONSE_TOKEN {
        let (ok, judgment) = crate::command::play::try_bagua_as_shan(tx, target)?;
        if !ok {
            run_response_effects(
                tx,
                vec![judgment],
                ContinuationKind::RespondAoeQueue,
                payload.to_vec(),
            )?;
            return continue_response_queue(tx, payload.to_vec());
        }
        let next_have = have.saturating_add(1);
        let next = multi_sha_queue_next(payload, idx, next_have);
        let mut effects = vec![judgment];
        effects.extend(sha_response_effects(
            target,
            source,
            CardId::NONE,
            card,
            next_have,
            need,
        ));
        run_response_effects(tx, effects, ContinuationKind::RespondAoeQueue, next.clone())?;
        return continue_response_queue(tx, next);
    }

    if let Some(skill) =
        crate::command::skill::source_free_response_skill(tx.state(), target, CardKind::Shan, token)
    {
        return crate::command::skill::begin_source_free_response(
            tx,
            target,
            skill,
            CardKind::Shan,
            2,
            payload,
        );
    }
    if let Some(skill) =
        crate::command::skill::multi_card_response_skill(tx.state(), target, CardKind::Shan, token)
    {
        return crate::command::skill::begin_multi_card_response(
            tx,
            target,
            skill,
            CardKind::Shan,
            2,
            payload,
        );
    }

    let cid = CardId::new(token);
    if !crate::command::play::response_shan_tokens(tx.state(), target).contains(&cid.raw()) {
        return Err(EffectError::Illegal("not a legal 闪 response".into()));
    }
    let from = response_card_zone(tx, target, cid)?;
    tx.state_mut()
        .resolution
        .register_sha_response_card(source, card, cid);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
    let next_have = have.saturating_add(1);
    let next = multi_sha_queue_next(payload, idx, next_have);
    let mut response_effects = vec![EffectRequest::MoveCards {
        cards: vec![cid],
        to: discard,
        from: Some(from),
    }];
    response_effects.extend(sha_response_effects(
        target, source, cid, card, next_have, need,
    ));
    response_effects.push(EffectRequest::EmitRuleEvent {
        code: 121,
        payload: vec![source.raw(), target.raw(), cid.raw(), next_have, need],
    });
    run_physical_response_effects(
        tx,
        response_effects,
        ContinuationKind::RespondAoeQueue,
        next.clone(),
        target,
        cid,
        CardKind::Shan,
    )?;
    continue_response_queue(tx, next)
}

pub(crate) fn multi_sha_queue_next(payload: &[u32], idx: usize, have: u32) -> Vec<u32> {
    let mut next = payload.to_vec();
    if have < payload[6] {
        next[7] = have;
    } else {
        next[3] = (idx + 1) as u32;
        next[7] = 0;
    }
    next
}

pub(crate) fn continue_response_queue(
    tx: &mut Transaction,
    payload: Vec<u32>,
) -> Result<(), EffectError> {
    if tx.state().resolution.continuation_kind == ContinuationKind::DyingSave {
        let resolution = &mut tx.state_mut().resolution;
        resolution.resume_kind = ContinuationKind::RespondAoeQueue;
        resolution.resume_payload = payload;
        return Ok(());
    }
    let resolution = &mut tx.state_mut().resolution;
    resolution.continuation_kind = ContinuationKind::RespondAoeQueue;
    resolution.continuation_payload = payload;
    crate::command::play::advance_aoe_queue(tx)
}

pub(crate) fn resume_interrupted_continuation(tx: &mut Transaction) -> Result<(), EffectError> {
    let kind = tx.state().resolution.resume_kind;
    let payload = tx.state().resolution.resume_payload.clone();
    if kind == ContinuationKind::None || payload.is_empty() {
        return Ok(());
    }
    tx.state_mut().resolution.resume_kind = ContinuationKind::None;
    tx.state_mut().resolution.resume_payload.clear();
    match kind {
        ContinuationKind::RespondAoeQueue => {
            tx.state_mut().resolution.continuation_kind = kind;
            tx.state_mut().resolution.continuation_payload = payload;
            crate::command::play::advance_aoe_queue(tx)
        }
        ContinuationKind::JuedouDuel => crate::command::play::continue_juedou_payload(tx, &payload),
        ContinuationKind::IndependentTrickQueue => {
            crate::command::play::advance_independent_trick_queue(tx, &payload)
        }
        ContinuationKind::RespondToSha => {
            let source = PlayerId::new(*payload.first().unwrap_or(&0));
            let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let amount = *payload.get(2).unwrap_or(&1);
            let need = *payload.get(3).unwrap_or(&1);
            let have = *payload.get(4).unwrap_or(&0);
            let (nature, card_flags) =
                crate::command::play::unpack_damage_card_context(*payload.get(5).unwrap_or(&0));
            let respondable = *payload.get(6).unwrap_or(&1) != 0;
            let card = CardId::new(*payload.get(7).unwrap_or(&0));
            crate::command::play::open_sha_response_step(
                tx,
                source,
                target,
                amount,
                need,
                have,
                nature,
                card_flags,
                respondable,
                card,
            )
        }
        ContinuationKind::ResumePlayedSha | ContinuationKind::ResumePlayedShaQueue => {
            crate::command::play::resume_played_sha_from_payload(tx, kind, &payload)
        }
        ContinuationKind::FinishPlayedSha => {
            let source = PlayerId::new(*payload.first().unwrap_or(&0));
            let card = CardId::new(*payload.get(1).unwrap_or(&0));
            crate::command::play::finish_played_sha(tx, source, card, *payload.get(2).unwrap_or(&0))
        }
        ContinuationKind::FinishDelayedJudgment => {
            crate::command::judge::finish_delayed_judgment(tx, &payload)
        }
        ContinuationKind::JudgeDamageOtherOnSuit => {
            let player = PlayerId::new(*payload.first().unwrap_or(&0));
            let amount = *payload.get(1).unwrap_or(&0);
            let nature = *payload.get(2).unwrap_or(&0) as u8;
            crate::effect::open_judgment_damage_other_choice(tx, player, amount, nature).map(|_| ())
        }
        ContinuationKind::GanglieSourceDiscardOrDamage => {
            let player = PlayerId::new(*payload.first().unwrap_or(&0));
            let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let hand_only = *payload.get(2).unwrap_or(&1) != 0;
            crate::effect::open_ganglie_target_discard_or_damage(tx, player, target, hand_only)
                .map(|_| ())
        }
        ContinuationKind::XiaoguoDiscardBasicThenEquipOrDamage => {
            let [stage, player, target, skill] = payload.as_slice() else {
                return Err(EffectError::EngineFault(
                    "bad saved Xiaoguo continuation".into(),
                ));
            };
            if *stage != 1 {
                return Err(EffectError::EngineFault(
                    "bad saved Xiaoguo continuation stage".into(),
                ));
            }
            crate::effect::open_xiaoguo_target_equipment_or_damage(
                tx,
                PlayerId::new(*player),
                PlayerId::new(*target),
                engine_api::SkillId::new(*skill),
            )
        }
        ContinuationKind::TargetDiscardThenDuelIfNonShaAndHpGteOwner => {
            let [player, target, _skill, discarded_was_sha] = payload.as_slice() else {
                return Err(EffectError::EngineFault(
                    "bad saved target-discard Duel continuation".into(),
                ));
            };
            crate::effect::resolve_target_discard_duel(
                tx,
                PlayerId::new(*player),
                PlayerId::new(*target),
                *discarded_was_sha != 0,
            )
            .map(|_| ())
        }
        ContinuationKind::WuxieWindow => {
            let start = PlayerId::new(*payload.first().unwrap_or(&0));
            let nullified = *payload.get(1).unwrap_or(&0);
            let effect_tag = *payload.get(2).unwrap_or(&0);
            let actor = PlayerId::new(*payload.get(3).unwrap_or(&0));
            let respond_to = PlayerId::new(*payload.get(4).unwrap_or(&0));
            let response_kind = payload
                .get(5)
                .and_then(|tag| CardKind::from_tag(*tag as u8))
                .unwrap_or(CardKind::Other);
            let response_suit = crate::command::judge::suit_from_tag(*payload.get(6).unwrap_or(&1));
            crate::command::play::resume_wuxie_after_card_used_triggers(
                tx,
                start,
                nullified,
                effect_tag,
                actor,
                respond_to,
                response_kind,
                response_suit,
                payload.get(7..).unwrap_or_default(),
            )
        }
        ContinuationKind::SourceFreeViewAsResponse => resume_source_free_response(tx, &payload),
        ContinuationKind::DelegatedGroupResponse => open_delegated_group_response(tx, payload),
        ContinuationKind::SourceFreeViewAsPlay => {
            crate::command::skill::resume_source_free_play(tx, &payload)
        }
        ContinuationKind::ContinueEnteredDrawPhase => {
            let player = PlayerId::new(*payload.first().unwrap_or(&0));
            crate::command::turn::continue_entered_draw_phase(tx, player)
        }
        _ => Err(EffectError::EngineFault(format!(
            "unsupported saved continuation {kind:?}"
        ))),
    }
}

pub(crate) fn resolve_twfupan(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [stage, player, damage_amount, card, target, skill] = payload else {
        return Err(EffectError::EngineFault("bad twfupan payload".into()));
    };
    let player = PlayerId::new(*player);
    let card = CardId::new(*card);
    let target = PlayerId::new(*target);
    let skill = SkillId::new(*skill);
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "twfupan owner or skill changed".into(),
        ));
    }
    let card_zone = |tx: &Transaction, card: CardId| {
        tx.state().cards.get(card).and_then(|state| {
            (state.owner == player
                && tx
                    .state()
                    .zones
                    .get(state.zone)
                    .is_some_and(|zone| matches!(zone.kind, ZoneKind::Hand | ZoneKind::Equip)))
            .then_some(state.zone)
        })
    };
    let target_eligible = |tx: &Transaction, target: PlayerId| {
        target != player
            && tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            && tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.twfupan_target_state(target) < 2)
    };
    match *stage {
        1 => {
            if cancelled {
                return Err(EffectError::Illegal(
                    "twfupan card choice cannot cancel".into(),
                ));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("twfupan requires one card".into()));
            };
            let selected = CardId::new(*selected);
            if card_zone(tx, selected).is_none() {
                return Err(EffectError::Illegal("twfupan selected card moved".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwfupan {
                    player,
                    damage_amount: *damage_amount,
                    card: selected,
                    target: PlayerId::NONE,
                    stage: 2,
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            if cancelled {
                return Err(EffectError::Illegal(
                    "twfupan target choice cannot cancel".into(),
                ));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("twfupan requires one target".into()));
            };
            let selected = PlayerId::new(*selected);
            let from = card_zone(tx, card)
                .ok_or_else(|| EffectError::Illegal("twfupan selected card moved".into()))?;
            if !target_eligible(tx, selected) {
                return Err(EffectError::Illegal(
                    "twfupan target became ineligible".into(),
                ));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, selected)
                .ok_or_else(|| EffectError::EngineFault("twfupan target hand missing".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::GiveCards {
                        cards: vec![card],
                        to,
                        from,
                    },
                    EffectRequest::ResolveTwfupan {
                        player,
                        damage_amount: *damage_amount,
                        card: CardId::NONE,
                        target: selected,
                        stage: 3,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        3 => {
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            if selections != [1] || !target_eligible(tx, target) {
                return Err(EffectError::Illegal("invalid twfupan damage choice".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwfupan {
                    player,
                    damage_amount: *damage_amount,
                    card: CardId::NONE,
                    target,
                    stage: 4,
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown twfupan choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_choose_spjungong_discard_or_lose(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad 峻攻 continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let skill = engine_api::SkillId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let n = payload[3];
    if observer != player
        || cancelled
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "峻攻 discard-or-lose context changed".into(),
        ));
    }
    let selected: Vec<CardId> = selections.iter().map(|c| CardId::new(*c)).collect();
    let mut queue: Vec<EffectRequest> = Vec::new();
    if selected.is_empty() {
        // 选 0 张 → 失去等量体力
        if tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.hp >= 1)
        {
            queue.push(EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: player,
                amount: n,
                is_lose_hp: true,
                nature: rule_api::damage_nature::NORMAL,
            });
        }
    } else {
        // 重验所选为可弃手牌/装备
        let discardable = crate::query::discardable_target_cards(
            tx.state(),
            player,
            player,
            rule_api::TargetCardScope::HandEquip,
        );
        if !selected.iter().all(|c| discardable.contains(c)) {
            return Err(EffectError::Illegal("峻攻 discard changed".into()));
        }
        queue.push(EffectRequest::DiscardCards {
            player,
            cards: selected,
        });
    }
    run_effects(tx, queue)?;
    // 使用虚拟【杀】：先 materialize 卡，预注册 Sha 结算记录与技能绑定，
    // 再以普通结算队列使用命中目标时由 `owner_source_sha_damage_recorded_by_skill`
    // 触发禁用本主动技至回合结束。旧 JS 的 `tempBanSkill` 只在该杀造成伤害时触发。
    let card = crate::effect::materialize_virtual_card(tx, player, CardKind::Sha)?;
    tx.state_mut().resolution.register_sha_resolution(
        player,
        card,
        std::slice::from_ref(&target),
        false,
    );
    tx.state_mut()
        .resolution
        .record_sha_skill_target(player, card, skill, target)
        .then_some(())
        .ok_or_else(|| EffectError::Illegal("峻攻 Sha target disappeared".into()))?;
    crate::command::play::play_as_kind(
        tx,
        player,
        card,
        CardKind::Sha,
        std::slice::from_ref(&target),
        false,
        false,
        false,
        false,
    )?;
    Ok(())
}
