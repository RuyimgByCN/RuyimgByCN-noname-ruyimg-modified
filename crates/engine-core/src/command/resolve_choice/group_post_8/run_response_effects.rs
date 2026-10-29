//! Command operations for `complete_view_as_response` and related helpers.

use super::*;

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
