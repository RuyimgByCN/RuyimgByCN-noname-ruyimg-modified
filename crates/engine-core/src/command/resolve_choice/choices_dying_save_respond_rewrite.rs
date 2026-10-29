//! 选择续行（resolve_choice）：respond、rewrite 族效果。

use super::*;

pub(super) fn resolve_choice(
    tx: &mut Transaction,
    _actor: PlayerId,
    observer: PlayerId,
    kind: ContinuationKind,
    payload: Vec<u32>,
    selections: &[u32],
    cancelled: bool,
    _vm_cont: (Vec<u8>, String, u32, u32, [u32; 4], u32, [u32; 3]),
    _optional_trigger: Option<crate::trigger::PendingTrigger>,
) -> Result<(), EffectError> {
    match kind {
        ContinuationKind::RespondToSha => {
            // payload: [source, target, amount, need, have, nature, respondable, card]
            let source = PlayerId::new(*payload.first().unwrap_or(&0));
            let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let amount = *payload.get(2).unwrap_or(&1);
            let need = *payload.get(3).unwrap_or(&1);
            let have = *payload.get(4).unwrap_or(&0);
            let damage_card_context = *payload.get(5).unwrap_or(&0);
            let (nature, card_flags) =
                super::super::play::unpack_damage_card_context(damage_card_context);
            let respondable = *payload.get(6).unwrap_or(&1) != 0;
            let card = CardId::new(*payload.get(7).unwrap_or(&0));
            if cancelled || selections.is_empty() {
                return super::super::play::deal_single_sha_damage(
                    tx, source, target, amount, nature, card_flags, card,
                );
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("play at most one 闪".into()));
            }
            let token = selections[0];
            if have == 0
                && super::super::play::response_youdishenru_tokens(tx.state(), target)
                    .contains(&token)
            {
                return super::super::play::use_youdishenru_for_saved_sha(
                    tx,
                    target,
                    source,
                    card,
                    CardId::new(token),
                    ContinuationKind::RespondToSha,
                    &payload,
                );
            }
            // 八卦：判定红心当闪
            if token == crate::content::BAGUA_RESPONSE_TOKEN {
                let (ok, judgment) = super::super::play::try_bagua_as_shan(tx, target)?;
                if ok {
                    let next_have = have + 1;
                    let resume_payload = if next_have < need {
                        vec![
                            source.raw(),
                            target.raw(),
                            amount,
                            need,
                            next_have,
                            damage_card_context,
                            u32::from(respondable),
                            card.raw(),
                        ]
                    } else {
                        Vec::new()
                    };
                    if next_have >= need {
                        super::super::play::save_played_sha_finish(
                            tx,
                            source,
                            card,
                            damage_card_context,
                        )?;
                    }
                    let mut effects = vec![judgment.clone()];
                    effects.extend(sha_response_effects(
                        target,
                        source,
                        CardId::NONE,
                        card,
                        next_have,
                        need,
                    ));
                    run_response_effects(
                        tx,
                        effects,
                        ContinuationKind::RespondToSha,
                        resume_payload,
                    )?;
                    if next_have < need {
                        return super::super::play::open_sha_response_step(
                            tx,
                            source,
                            target,
                            amount,
                            need,
                            next_have,
                            nature,
                            card_flags,
                            respondable,
                            card,
                        );
                    }
                    return super::super::play::finish_saved_played_sha(tx, source, card);
                }
                run_response_effects(
                    tx,
                    vec![judgment],
                    ContinuationKind::RespondToSha,
                    payload.clone(),
                )?;
                // fail → ask again (may play real 闪 or retry 八卦)
                return super::super::play::open_sha_response_step(
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
                );
            }
            if let Some(skill) = super::super::skill::source_free_response_skill(
                tx.state(),
                target,
                CardKind::Shan,
                token,
            ) {
                return super::super::skill::begin_source_free_response(
                    tx,
                    target,
                    skill,
                    CardKind::Shan,
                    1,
                    &payload,
                );
            }
            if let Some(skill) = super::super::skill::multi_card_response_skill(
                tx.state(),
                target,
                CardKind::Shan,
                token,
            ) {
                return super::super::skill::begin_multi_card_response(
                    tx,
                    target,
                    skill,
                    CardKind::Shan,
                    1,
                    &payload,
                );
            }
            let cid = CardId::new(token);
            let legal = super::super::play::response_shan_tokens(tx.state(), target);
            if !legal.contains(&cid.raw()) {
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
            let next_have = have + 1;
            let resume_payload = if next_have < need {
                vec![
                    source.raw(),
                    target.raw(),
                    amount,
                    need,
                    next_have,
                    damage_card_context,
                    u32::from(respondable),
                    card.raw(),
                ]
            } else {
                Vec::new()
            };
            if next_have >= need {
                super::super::play::save_played_sha_finish(tx, source, card, damage_card_context)?;
            }
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
                ContinuationKind::RespondToSha,
                resume_payload,
                target,
                cid,
                CardKind::Shan,
            )?;
            if next_have < need {
                return super::super::play::open_sha_response_step(
                    tx,
                    source,
                    target,
                    amount,
                    need,
                    next_have,
                    nature,
                    card_flags,
                    respondable,
                    card,
                );
            }
            super::super::play::finish_saved_played_sha(tx, source, card)
        }

        ContinuationKind::RespondAoeQueue => {
            let kind = *payload.first().unwrap_or(&0);
            if kind == 3 {
                return resolve_multi_sha_queue_choice(tx, &payload, selections, cancelled);
            }
            let source = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let amount = *payload.get(2).unwrap_or(&1);
            let idx = *payload.get(3).unwrap_or(&0) as usize;
            let target = PlayerId::new(*payload.get(5 + idx).unwrap_or(&0));
            if !cancelled && !selections.is_empty() {
                if selections.len() != 1 {
                    return Err(EffectError::Illegal("play one response card".into()));
                }
                let token = selections[0];
                // 万箭：八卦当闪
                if kind == 2 && token == crate::content::BAGUA_RESPONSE_TOKEN {
                    let (ok, judgment) = super::super::play::try_bagua_as_shan(tx, target)?;
                    if ok {
                        let mut next = payload.clone();
                        next[3] = (idx + 1) as u32;
                        run_response_effects(
                            tx,
                            vec![
                                judgment.clone(),
                                EffectRequest::CardResponded {
                                    player: target,
                                    respond_to: source,
                                    card: CardId::NONE,
                                    response_kind: CardKind::Shan.tag(),
                                },
                            ],
                            ContinuationKind::RespondAoeQueue,
                            next,
                        )?;
                    }
                    if !ok {
                        run_response_effects(
                            tx,
                            vec![judgment],
                            ContinuationKind::RespondAoeQueue,
                            payload.clone(),
                        )?;
                        // fail: re-open same target
                        let tokens = super::super::play::response_shan_options(tx.state(), target);
                        if tokens.is_empty() {
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
                        } else {
                            crate::effect::open_choice(
                                tx,
                                target,
                                0,
                                1,
                                true,
                                tokens,
                                engine_api::ChoiceKind::SelectCards,
                                ContinuationKind::RespondAoeQueue,
                                payload.clone(),
                            )?;
                            return Err(EffectError::ChoiceRequested);
                        }
                    }
                    // success or damage applied after fail-empty → advance
                } else {
                    let response_kind = if kind == 1 {
                        CardKind::Sha
                    } else {
                        CardKind::Shan
                    };
                    if let Some(skill) = super::super::skill::source_free_response_skill(
                        tx.state(),
                        target,
                        response_kind,
                        token,
                    ) {
                        return super::super::skill::begin_source_free_response(
                            tx,
                            target,
                            skill,
                            response_kind,
                            2,
                            &payload,
                        );
                    }
                    if let Some(skill) = super::super::skill::multi_card_response_skill(
                        tx.state(),
                        target,
                        response_kind,
                        token,
                    ) {
                        return super::super::skill::begin_multi_card_response(
                            tx,
                            target,
                            skill,
                            response_kind,
                            2,
                            &payload,
                        );
                    }
                    let cid = CardId::new(token);
                    let legal = match kind {
                        1 => super::super::play::response_sha_tokens(tx.state(), target),
                        2 => super::super::play::response_shan_tokens(tx.state(), target),
                        _ => vec![],
                    };
                    if !legal.contains(&cid.raw()) {
                        return Err(EffectError::Illegal("illegal response card".into()));
                    }
                    let from = response_card_zone(tx, target, cid)?;
                    let discard = tx
                        .state()
                        .zones
                        .find(ZoneKind::DiscardPile, PlayerId::NONE)
                        .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
                    let response_kind = if kind == 1 {
                        CardKind::Sha
                    } else {
                        CardKind::Shan
                    };
                    let mut next = payload.clone();
                    next[3] = (idx + 1) as u32;
                    run_physical_response_effects(
                        tx,
                        vec![
                            EffectRequest::MoveCards {
                                cards: vec![cid],
                                to: discard,
                                from: Some(from),
                            },
                            EffectRequest::CardResponded {
                                player: target,
                                respond_to: source,
                                card: cid,
                                response_kind: response_kind.tag(),
                            },
                            EffectRequest::EmitRuleEvent {
                                code: 123,
                                payload: vec![kind, source.raw(), target.raw(), cid.raw()],
                            },
                        ],
                        ContinuationKind::RespondAoeQueue,
                        next,
                        target,
                        cid,
                        response_kind,
                    )?;
                }
            } else {
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
            }
            if tx.state().resolution.continuation_kind == ContinuationKind::DyingSave {
                if tx.state().resolution.resume_kind == ContinuationKind::RespondAoeQueue
                    && tx.state().resolution.resume_payload.len() > 3
                {
                    tx.state_mut().resolution.resume_payload[3] = (idx + 1) as u32;
                }
                return Ok(());
            }
            let mut next = payload.clone();
            next[3] = (idx + 1) as u32;
            tx.state_mut().resolution.continuation_kind = ContinuationKind::RespondAoeQueue;
            tx.state_mut().resolution.continuation_payload = next;
            super::super::play::advance_aoe_queue(tx)
        }

        ContinuationKind::RewriteJudgment => {
            // payload: [owner, delayed, result_card, suit, rank, kind_tag, rewriter_len,
            //           rewriters..., rest_len, delayed_rest...]
            let owner = PlayerId::new(*payload.first().unwrap_or(&0));
            let delayed = CardId::new(*payload.get(1).unwrap_or(&0));
            let mut result_card = CardId::new(*payload.get(2).unwrap_or(&0));
            let mut suit = super::super::judge::suit_from_tag(*payload.get(3).unwrap_or(&1));
            let mut rank = *payload.get(4).unwrap_or(&1) as u8;
            let kind = super::super::judge::kind_from_tag(*payload.get(5).unwrap_or(&0));
            let rewriter_len = *payload.get(6).unwrap_or(&0) as usize;
            let mut rewriters = Vec::new();
            for index in 0..rewriter_len {
                if let Some(player) = payload.get(7 + index) {
                    rewriters.push(PlayerId::new(*player));
                }
            }
            let Some(rewriter) = rewriters.first().copied() else {
                return Err(EffectError::Illegal("missing judgment rewriter".into()));
            };
            let rest_len_index = 7 + rewriter_len;
            let rest_len = *payload.get(rest_len_index).unwrap_or(&0) as usize;
            let mut rest = Vec::new();
            for index in 0..rest_len {
                if let Some(id) = payload.get(rest_len_index + 1 + index) {
                    rest.push(CardId::new(*id));
                }
            }
            if !cancelled && !selections.is_empty() {
                if selections.len() != 1 {
                    return Err(EffectError::Illegal("pick one card to rewrite".into()));
                }
                let cid = CardId::new(selections[0]);
                if !super::super::judge::judgment_rewrite_cards(tx, rewriter).contains(&cid) {
                    return Err(EffectError::Illegal(
                        "card is not legal for judgment rewrite".into(),
                    ));
                }
                let c = tx
                    .state()
                    .cards
                    .get(cid)
                    .ok_or_else(|| EffectError::Illegal("bad card".into()))?
                    .clone();
                let from = response_card_zone(tx, rewriter, cid)?;
                result_card = cid;
                suit = crate::query::effective_card_suit(tx.state(), owner, &c);
                rank = c.rank;
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
                let mut effects = Vec::new();
                if let Some(mark) =
                    crate::query::judgment_rewrite_mark_cost_for_card(tx.state(), rewriter, &c)
                {
                    effects.push(EffectRequest::RemoveMark {
                        player: rewriter,
                        mark: engine_api::MarkDefId::new(mark),
                        amount: 1,
                    });
                }
                effects.extend([
                    EffectRequest::MoveCards {
                        cards: vec![cid],
                        to: discard,
                        from: Some(from),
                    },
                    EffectRequest::EmitRuleEvent {
                        code: 201, // guicai rewrite
                        payload: vec![
                            rewriter.raw(),
                            owner.raw(),
                            delayed.raw(),
                            super::super::judge::suit_tag(suit),
                            rank as u32,
                        ],
                    },
                ]);
                run_effects(tx, effects)?;
            }
            super::super::judge::continue_judgment_rewriters(
                tx,
                owner,
                delayed,
                kind,
                result_card,
                suit,
                rank,
                &rewriters[1..],
                &rest,
            )
        }

        ContinuationKind::RewriteBeigeJudgment => {
            if payload.len() < 8 {
                return Err(EffectError::EngineFault(
                    "truncated Beige judgment rewrite".into(),
                ));
            }
            let player = PlayerId::new(payload[0]);
            let target = PlayerId::new(payload[1]);
            let source = PlayerId::new(payload[2]);
            let mut result_card = CardId::new(payload[3]);
            let mut suit = super::super::judge::suit_from_tag(payload[4]);
            let mut rank = payload[5] as u8;
            let skill = engine_api::SkillId::new(payload[6]);
            let rewriter_len = payload[7] as usize;
            let rewriter_end = 8usize.saturating_add(rewriter_len);
            if rewriter_len == 0 || payload.len() < rewriter_end {
                return Err(EffectError::EngineFault(
                    "truncated Beige judgment rewriter queue".into(),
                ));
            }
            let configured_counts = match payload.len().saturating_sub(rewriter_end) {
                0 => None,
                3 if payload[rewriter_end] == 1 => {
                    Some((payload[rewriter_end + 1], payload[rewriter_end + 2]))
                }
                _ => {
                    return Err(EffectError::EngineFault(
                        "invalid Beige judgment policy suffix".into(),
                    ));
                }
            };
            let rewriters: Vec<_> = payload[8..rewriter_end]
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            let rewriter = rewriters[0];
            if observer != rewriter {
                return Err(EffectError::Illegal(
                    "only the current player may rewrite Beige judgment".into(),
                ));
            }
            if cancelled || selections.is_empty() {
                return match configured_counts {
                    Some((heart_recover_amount, diamond_draw_count)) => {
                        crate::effect::continue_beige_judgment_rewriters_configured(
                            tx,
                            player,
                            target,
                            source,
                            result_card,
                            suit,
                            rank,
                            &rewriters[1..],
                            heart_recover_amount,
                            diamond_draw_count,
                            skill,
                        )
                    }
                    None => crate::effect::continue_beige_judgment_rewriters(
                        tx,
                        player,
                        target,
                        source,
                        result_card,
                        suit,
                        rank,
                        &rewriters[1..],
                        skill,
                    ),
                }
                .map(|_| ());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "pick one card to rewrite Beige judgment".into(),
                ));
            }
            let cid = CardId::new(selections[0]);
            if !super::super::judge::judgment_rewrite_cards(tx, rewriter).contains(&cid) {
                return Err(EffectError::Illegal(
                    "card is not legal for Beige judgment rewrite".into(),
                ));
            }
            let card = tx
                .state()
                .cards
                .get(cid)
                .ok_or_else(|| EffectError::Illegal("bad rewrite card".into()))?
                .clone();
            let from = response_card_zone(tx, rewriter, cid)?;
            result_card = cid;
            suit = crate::query::effective_card_suit(tx.state(), target, &card);
            rank = card.rank;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(match configured_counts {
                    Some((heart_recover_amount, diamond_draw_count)) => {
                        EffectRequest::ResolveBeigeConfigured {
                            player,
                            target,
                            source,
                            stage: 4,
                            result_card,
                            suit,
                            rank,
                            rewriters: rewriters[1..].to_vec(),
                            heart_recover_amount,
                            diamond_draw_count,
                            skill,
                        }
                    }
                    None => EffectRequest::ResolveBeige {
                        player,
                        target,
                        source,
                        stage: 4,
                        result_card,
                        suit,
                        rank,
                        rewriters: rewriters[1..].to_vec(),
                        skill,
                    },
                });
            let mut effects = Vec::new();
            if let Some(mark) =
                crate::query::judgment_rewrite_mark_cost_for_card(tx.state(), rewriter, &card)
            {
                effects.push(EffectRequest::RemoveMark {
                    player: rewriter,
                    mark: engine_api::MarkDefId::new(mark),
                    amount: 1,
                });
            }
            effects.extend([
                EffectRequest::MoveCards {
                    cards: vec![cid],
                    to: discard,
                    from: Some(from),
                },
                EffectRequest::EmitRuleEvent {
                    code: 201,
                    payload: vec![
                        rewriter.raw(),
                        target.raw(),
                        CardId::NONE.raw(),
                        super::super::judge::suit_tag(suit),
                        u32::from(rank),
                    ],
                },
            ]);
            run_effects(tx, effects).map(|_| ())
        }

        _ => unreachable!("choice routed to a different continuation group"),
    }
}
