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
