//! Effect primitives for `give_cards` and related helpers.

use super::*;

pub(crate) fn choose_use_virtual_card(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    choose_use_virtual_card_impl(tx, player, kind, skill, VirtualCardTracking::None, true)
}

pub(crate) fn choose_use_virtual_card_impl(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
    skill: engine_api::SkillId,
    tracking: VirtualCardTracking,
    suppress_phase_history: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let kind = CardKind::from_tag(kind)
        .filter(|kind| {
            matches!(
                kind,
                CardKind::TrickWanjian | CardKind::TrickJuedou | CardKind::TrickNanman
            )
        })
        .ok_or_else(|| {
            EffectError::ContentFault(
                "choose-use virtual card supports only Wanjian, Juedou or Nanman".into(),
            )
        })?;
    if !originating_skill_is_enabled(tx, player, skill)
        || !crate::command::play::source_free_card_has_legal_use(tx.state(), player, kind)
    {
        return Ok(Vec::new());
    }
    if suppress_phase_history {
        tx.state_mut()
            .turn
            .set_flag(turn_flag::PHASE_HISTORY_SUPPRESSED, 1);
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::FinishPhaseEndVirtualCard);
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind,
        suit: engine_api::Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .expect("validated processing zone")
        .cards
        .push(card);
    match tracking {
        VirtualCardTracking::None => {}
        VirtualCardTracking::SourceFree => tx
            .state_mut()
            .skills
            .get_mut(skill)
            .expect("validated originating skill")
            .set_source_free_physical_card(card),
        VirtualCardTracking::Dcctjiuxian => {
            if !tx
                .state_mut()
                .skills
                .get_mut(skill)
                .is_some_and(|instance| instance.add_tracked_cards(&[card]))
            {
                return Err(EffectError::ContentFault(
                    "Dcctjiuxian could not track its virtual Duel".into(),
                ));
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDcctjiuxian {
                    player,
                    event_player: PlayerId::NONE,
                    target: PlayerId::NONE,
                    card,
                    action: 6,
                    cards: Vec::new(),
                    skill,
                },
            );
        }
    }
    match crate::command::play::skill_card_target_spec(tx, player, card)? {
        crate::command::play::SkillCardTargetSpec::Immediate(targets) => {
            if tracking == VirtualCardTracking::Dcctjiuxian {
                let target = targets.first().copied().ok_or_else(|| {
                    EffectError::ContentFault("Dcctjiuxian Duel requires one target".into())
                })?;
                if !tx
                    .state_mut()
                    .skills
                    .get_mut(skill)
                    .is_some_and(|instance| instance.record_active_play_phase_target(target))
                {
                    return Err(EffectError::ContentFault(
                        "Dcctjiuxian could not record its Duel target".into(),
                    ));
                }
            }
            tx.state_mut()
                .turn
                .set_flag(turn_flag::VIEW_AS_CARD, card.raw() as i32);
            tx.state_mut()
                .turn
                .set_flag(turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
            tx.state_mut()
                .turn
                .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, i32::from(kind.tag()));
            let result = crate::command::play::handle_skill_card(tx, player, card, &targets);
            tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_CARD, 0);
            tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_SKILL, 0);
            tx.state_mut()
                .turn
                .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, 0);
            result?;
            Ok(Vec::new())
        }
        crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
            let mut payload = vec![
                player.raw(),
                card.raw(),
                kind.tag().into(),
                skill.raw(),
                options.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            if tracking == VirtualCardTracking::Dcctjiuxian {
                payload.push(1);
            }
            open_choice(
                tx,
                player,
                min,
                max,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ChooseUseVirtualCard,
                payload,
            )
        }
        crate::command::play::SkillCardTargetSpec::Pairs(_) => Err(EffectError::ContentFault(
            "choose-use virtual card does not support paired targets".into(),
        )),
    }
}

pub(crate) fn use_virtual_card_on(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let kind = CardKind::from_tag(kind)
        .ok_or_else(|| EffectError::ContentFault("unknown virtual card kind".into()))?;
    if !tx.state().players.get(player).is_some_and(|state| {
        state.alive && (!state.dying || (kind == CardKind::Jiu && player == target))
    }) || !tx
        .state()
        .players
        .get(target)
        .is_some_and(|state| state.alive)
    {
        return Ok(Vec::new());
    }
    let card = materialize_virtual_card(tx, player, kind)?;
    crate::command::play::play_as_kind(
        tx,
        player,
        card,
        kind,
        &[target],
        false,
        false,
        false,
        false,
    )?;
    Ok(Vec::new())
}

pub(crate) fn use_virtual_card_on_selected_targets(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
    targets: Vec<PlayerId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let kind = CardKind::from_tag(kind)
        .ok_or_else(|| EffectError::ContentFault("unknown virtual card kind".into()))?;
    let card = materialize_virtual_card(tx, player, kind)?;
    match kind {
        CardKind::TrickJuedou => {
            crate::command::play::play_source_free_multi_target_juedou(tx, player, card, &targets)?;
        }
        CardKind::TrickShunshou => {
            crate::command::play::play_source_free_multi_target_shunshou(
                tx, player, card, &targets,
            )?;
        }
        CardKind::TrickTiesuo => {
            crate::command::play::play_as_kind(
                tx, player, card, kind, &targets, false, false, false, false,
            )?;
        }
        CardKind::Sha
        | CardKind::TrickGuohe
        | CardKind::TrickWugu
        | CardKind::TrickDiaobingqianjiang => {
            crate::command::play::play_as_kind(
                tx, player, card, kind, &targets, false, false, false, false,
            )?;
        }
        _ => {
            return Err(EffectError::ContentFault(
                "selected-target virtual card kind unsupported".into(),
            ));
        }
    }
    Ok(Vec::new())
}

pub(crate) fn materialize_virtual_card(
    tx: &mut Transaction,
    player: PlayerId,
    kind: CardKind,
) -> Result<CardId, EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no virtual card user hand".into()))?;
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind,
        suit: engine_api::Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .unwrap()
        .cards
        .push(card);
    move_one_card(tx, card, processing, hand, player, false)?;
    Ok(card)
}

pub(crate) fn try_use_virtual_sha_on_without_distance(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    ignore_use_count: bool,
    damage_bonus: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no removed zone".into()))?;
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind: CardKind::Sha,
        suit: engine_api::Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .unwrap()
        .cards
        .push(card);
    let use_count_allowed = ignore_use_count
        || crate::query::sha_play_phase_use_count(tx.state())
            < crate::query::sha_use_limit_for_card(tx.state(), source, card)
        || crate::query::sha_use_unlimited_for_target(tx.state(), source, target);
    if crate::query::card_use_cancelled_for_card(tx.state(), source, card)
        || !use_count_allowed
        || crate::command::play::validate_sha_target_with_options(tx, source, card, target, true)
            .is_err()
    {
        move_one_card(tx, card, processing, removed, PlayerId::NONE, true)?;
        return Ok(Vec::new());
    }
    crate::command::play::handle_virtual_sha_with_policies_and_bonus(
        tx,
        source,
        card,
        &[target],
        true,
        ignore_use_count,
        damage_bonus,
    )?;
    Ok(Vec::new())
}

/// 雷劫（releijie）：对目标判定——黑桃 2-9 则其摸两张牌，否则对其使用两张雷【杀】。
/// 上游 (sp/skill.js:19804)：target.judge(spade 2-9 → 有利)；判定成功 target.draw(2)，
/// 失败则 player 对 target 使用至多两张雷【杀】（每次可用才用）。
pub(crate) fn judge_spade_two_to_nine_draw_two_or_thunder_sha_twice(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("bad releijie judging player".into()));
    }
    let result = crate::command::judge::flip_judgment(
        tx,
        target,
        crate::command::judge::JudgmentFavor::Neutral,
    )?;
    let suit = result.suit;
    let rank = result.rank;
    tx.push_event(DomainEvent::RuleNote {
        code: 236, // standalone skill judgment (releijie)
        payload: vec![
            target.raw(),
            crate::command::judge::suit_tag(suit),
            u32::from(rank),
        ],
    });
    let emit = EffectRequest::EmitJudgmentResolved {
        player: target,
        result_card: result.card,
        suit,
        rank,
        cause_card: CardId::NONE,
    };
    crate::effect::run_effects(tx, vec![emit])?;
    let spade_two_to_nine = suit == engine_api::Suit::Spade && (2..=9).contains(&rank);
    if spade_two_to_nine {
        return crate::effect::run_effects(
            tx,
            vec![EffectRequest::DrawCards {
                player: target,
                count: 2,
            }],
        );
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
    for _ in 0..2 {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::NONE,
            kind: CardKind::ShaThunder,
            suit: engine_api::Suit::None,
            rank: 0,
            weapon_range: 0,
            zone: processing,
            owner: PlayerId::NONE,
            face_up: true,
        });
        tx.state_mut()
            .zones
            .get_mut(processing)
            .unwrap()
            .cards
            .push(card);
        if crate::command::play::validate_sha_target_with_options(tx, source, card, target, true)
            .is_err()
        {
            break;
        }
        crate::command::play::handle_virtual_sha_with_policies_and_bonus(
            tx,
            source,
            card,
            &[target],
            true,
            false,
            0,
        )?;
    }
    Ok(Vec::new())
}

pub(crate) fn deal_damage(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    is_lose_hp: bool,
    nature: u8,
    logical_sha: bool,
    card: CardId,
    card_kind: u8,
    card_flags: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(mut prepared) = prepare_damage(
        tx.state(),
        source,
        target,
        amount,
        is_lose_hp,
        nature,
        logical_sha,
        card,
    )?
    else {
        return Ok(Vec::new());
    };
    let EffectRequest::CommitDamage {
        card: prepared_card,
        card_kind: prepared_kind,
        card_flags: prepared_flags,
        ..
    } = &mut prepared
    else {
        unreachable!();
    };
    *prepared_card = card;
    *prepared_kind = card_kind;
    *prepared_flags = card_flags;
    let EffectRequest::CommitDamage {
        damage_id,
        source,
        target,
        amount,
        base_amount,
        is_lose_hp,
        nature,
        card,
        card_kind,
        card_flags,
    } = prepared
    else {
        unreachable!();
    };
    let (mut events, followups) = commit_damage(
        tx,
        damage_id,
        source,
        target,
        amount,
        base_amount,
        is_lose_hp,
        nature,
        card,
        card_kind,
        card_flags,
    )?;
    for followup in followups {
        events.extend(apply_effect(tx, followup)?);
    }
    Ok(events)
}

/// Resolve static damage modifiers without mutating state.
/// 主动原语 (批 270, ADR 0641 §8.58)：断索——先重置所选全部连环角色
/// （移除 MARK_CHAIN），再对其各造成 1 点火焰伤害。两遍循环与上游 content
/// 一致：重置完成后伤害不再经铁索传导。
pub(crate) fn reset_linked_selected_targets(
    tx: &mut Transaction,
    source: PlayerId,
    targets: &[PlayerId],
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut events = Vec::new();
    for target in targets {
        if tx
            .state()
            .players
            .get(*target)
            .is_some_and(|participant| participant.alive)
        {
            events.extend(super::super::remove_mark(
                tx,
                *target,
                crate::content::MARK_CHAIN,
                0,
            )?);
        }
    }
    for target in targets {
        if tx
            .state()
            .players
            .get(*target)
            .is_some_and(|participant| participant.alive)
        {
            events.extend(deal_damage(
                tx,
                source,
                *target,
                1,
                false,
                1,
                false,
                CardId::NONE,
                0,
                0,
            )?);
        }
    }
    Ok(events)
}

pub(crate) fn prepare_damage(
    state: &crate::state::GameState,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    is_lose_hp: bool,
    nature: u8,
    logical_sha: bool,
    card: CardId,
) -> Result<Option<EffectRequest>, EffectError> {
    let p = state
        .players
        .get(target)
        .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
    if amount == 0 || !p.alive {
        return Ok(None);
    }
    let source = crate::query::nanman_damage_source_for(state, source, card).unwrap_or(source);
    let amount = if logical_sha {
        crate::query::sha_damage_source_range_bonus_for(state, source, amount)
    } else {
        amount
    };
    let intent = crate::intent::resolve_damage_intent_with_kind(
        state,
        crate::intent::DamageIntent {
            source,
            target,
            amount,
            is_lose_hp,
            nature,
            cancelled: false,
        },
        logical_sha,
    );
    if intent.cancelled || intent.amount == 0 {
        return Ok((!intent.is_lose_hp).then_some(EffectRequest::CommitDamage {
            damage_id: 0,
            source,
            target,
            amount: 0,
            base_amount: 0,
            is_lose_hp: false,
            nature: intent.nature,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }));
    }
    let amount = intent.amount;
    let is_lose_hp = intent.is_lose_hp;
    let nature = intent.nature;
    // 藤甲：火焰伤害 +1（失去体力通道不触发）；传导时各目标独立结算。
    let base_amount = amount;
    let mut amount = if is_lose_hp {
        amount
    } else {
        crate::query::damage_taken_amount_for(state, source, target, nature, amount)
    };
    if amount == 0 {
        return Ok(Some(EffectRequest::CommitDamage {
            damage_id: 0,
            source,
            target,
            amount: 0,
            base_amount,
            is_lose_hp: false,
            nature,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }));
    }
    if !is_lose_hp
        && nature == rule_api::damage_nature::FIRE
        && crate::content::player_has_tengjia(state, target)
        && !crate::query::armor_effect_ignored_for_card(state, source, target, card)
    {
        amount = amount.saturating_add(1);
    }
    if !is_lose_hp {
        amount = crate::query::damage_taken_final_amount_for(state, target, nature, amount);
        if logical_sha {
            amount = crate::query::sha_damage_source_range_cap_for(state, source, amount);
        }
        if amount == 0 {
            return Ok(Some(EffectRequest::CommitDamage {
                damage_id: 0,
                source,
                target,
                amount: 0,
                base_amount,
                is_lose_hp: false,
                nature,
                card: CardId::NONE,
                card_kind: 0,
                card_flags: 0,
            }));
        }
    }
    Ok(Some(EffectRequest::CommitDamage {
        damage_id: 0,
        source,
        target,
        amount,
        base_amount,
        is_lose_hp,
        nature,
        card: CardId::NONE,
        card_kind: 0,
        card_flags: 0,
    }))
}
