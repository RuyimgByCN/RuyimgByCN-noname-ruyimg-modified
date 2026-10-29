//! Effect primitives for `choose_link_other` and related helpers.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn beige_stage_effect(
    policy: BeigeOutcomePolicy,
    player: PlayerId,
    target: PlayerId,
    source: PlayerId,
    stage: u8,
    result_card: CardId,
    suit: Suit,
    rank: u8,
    rewriters: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> EffectRequest {
    match policy {
        BeigeOutcomePolicy::Classic => EffectRequest::ResolveBeige {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            skill,
        },
        BeigeOutcomePolicy::Configured {
            heart_recover_amount,
            diamond_draw_count,
        } => EffectRequest::ResolveBeigeConfigured {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            heart_recover_amount,
            diamond_draw_count,
            skill,
        },
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_beige_with_policy(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    source: PlayerId,
    stage: u8,
    result_card: CardId,
    suit: Suit,
    rank: u8,
    rewriters: Vec<PlayerId>,
    skill: engine_api::SkillId,
    policy: BeigeOutcomePolicy,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.alive)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || source.is_none()
                || tx.state().players.get(source).is_none()
            {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let Some(only) = (options.len() == 1).then(|| options[0]) else {
                if options.is_empty() {
                    return Ok(Vec::new());
                }
                tx.state_mut()
                    .resolution
                    .deferred_tail_effects
                    .push(beige_stage_effect(
                        policy,
                        player,
                        target,
                        source,
                        1,
                        CardId::NONE,
                        Suit::None,
                        0,
                        Vec::new(),
                        skill,
                    ));
                let mut payload = vec![player.raw(), skill.raw(), 1, options.len() as u32];
                payload.extend(options.iter().map(|card| card.raw()));
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::ChooseDiscardHandEquipThenRemoveSelfSkill,
                    payload,
                );
            };
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(beige_stage_effect(
                    policy,
                    player,
                    target,
                    source,
                    1,
                    CardId::NONE,
                    Suit::None,
                    0,
                    Vec::new(),
                    skill,
                ));
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![only],
                }],
            )
        }
        1 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
            if tx
                .state()
                .zones
                .get(draw)
                .is_none_or(|zone| zone.cards.is_empty())
            {
                reshuffle_discard_into_draw(tx)?;
            }
            let result_card = tx
                .state()
                .zones
                .get(draw)
                .and_then(|zone| zone.cards.last())
                .copied()
                .ok_or_else(|| EffectError::Illegal("draw pile empty after reshuffle".into()))?;
            let card = tx
                .state()
                .cards
                .get(result_card)
                .ok_or_else(|| EffectError::Illegal("bad Beige judgment card".into()))?
                .clone();
            let suit = crate::query::effective_card_suit(tx.state(), target, &card);
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(beige_stage_effect(
                    policy,
                    player,
                    target,
                    source,
                    2,
                    result_card,
                    suit,
                    card.rank,
                    Vec::new(),
                    skill,
                ));
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![result_card],
                    to: discard,
                    from: Some(draw),
                }],
            )
        }
        2 => {
            let rewriters = crate::command::judge::judgment_rewriters(tx, target);
            continue_beige_judgment_rewriters_with_policy(
                tx,
                player,
                target,
                source,
                result_card,
                suit,
                rank,
                &rewriters,
                skill,
                policy,
            )
        }
        3 => match suit {
            Suit::Heart
                if tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive) =>
            {
                run_effects(
                    tx,
                    vec![EffectRequest::RecoverHp {
                        source: PlayerId::NONE,
                        player: target,
                        amount: match policy {
                            BeigeOutcomePolicy::Classic => 1,
                            BeigeOutcomePolicy::Configured {
                                heart_recover_amount,
                                ..
                            } => heart_recover_amount,
                        },
                    }],
                )
            }
            Suit::Diamond
                if tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive) =>
            {
                run_effects(
                    tx,
                    vec![EffectRequest::DrawCardsFromSkill {
                        player: target,
                        count: match policy {
                            BeigeOutcomePolicy::Classic => 2,
                            BeigeOutcomePolicy::Configured {
                                diamond_draw_count, ..
                            } => diamond_draw_count,
                        },
                        skill,
                    }],
                )
            }
            Suit::Club
                if tx
                    .state()
                    .players
                    .get(source)
                    .is_some_and(|participant| participant.alive) =>
            {
                run_effects(
                    tx,
                    vec![
                        EffectRequest::ChooseDiscardTargetCard {
                            player: source,
                            target: source,
                            scope: TargetCardScope::HandEquip,
                        },
                        EffectRequest::ChooseDiscardTargetCard {
                            player: source,
                            target: source,
                            scope: TargetCardScope::HandEquip,
                        },
                    ],
                )
            }
            Suit::Spade
                if tx
                    .state()
                    .players
                    .get(source)
                    .is_some_and(|participant| participant.alive) =>
            {
                run_effects(tx, vec![EffectRequest::TurnOver { player: source }])
            }
            _ => Ok(Vec::new()),
        },
        4 => continue_beige_judgment_rewriters_with_policy(
            tx,
            player,
            target,
            source,
            result_card,
            suit,
            rank,
            &rewriters,
            skill,
            policy,
        ),
        _ => Err(EffectError::ContentFault(
            "unknown Beige resolution stage".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn continue_beige_judgment_rewriters(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    source: PlayerId,
    result_card: CardId,
    suit: Suit,
    rank: u8,
    rewriters: &[PlayerId],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    continue_beige_judgment_rewriters_with_policy(
        tx,
        player,
        target,
        source,
        result_card,
        suit,
        rank,
        rewriters,
        skill,
        BeigeOutcomePolicy::Classic,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn continue_beige_judgment_rewriters_configured(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    source: PlayerId,
    result_card: CardId,
    suit: Suit,
    rank: u8,
    rewriters: &[PlayerId],
    heart_recover_amount: u32,
    diamond_draw_count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    continue_beige_judgment_rewriters_with_policy(
        tx,
        player,
        target,
        source,
        result_card,
        suit,
        rank,
        rewriters,
        skill,
        BeigeOutcomePolicy::Configured {
            heart_recover_amount,
            diamond_draw_count,
        },
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn continue_beige_judgment_rewriters_with_policy(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    source: PlayerId,
    result_card: CardId,
    suit: Suit,
    rank: u8,
    rewriters: &[PlayerId],
    skill: engine_api::SkillId,
    policy: BeigeOutcomePolicy,
) -> Result<Vec<DomainEvent>, EffectError> {
    for (index, rewriter) in rewriters.iter().copied().enumerate() {
        let options = crate::command::judge::judgment_rewrite_cards(tx, rewriter);
        if options.is_empty() {
            continue;
        }
        let remaining = &rewriters[index..];
        let mut payload = vec![
            player.raw(),
            target.raw(),
            source.raw(),
            result_card.raw(),
            crate::command::judge::suit_tag(suit),
            u32::from(rank),
            skill.raw(),
            remaining.len() as u32,
        ];
        payload.extend(remaining.iter().map(|player| player.raw()));
        if let BeigeOutcomePolicy::Configured {
            heart_recover_amount,
            diamond_draw_count,
        } = policy
        {
            payload.extend([1, heart_recover_amount, diamond_draw_count]);
        }
        return open_choice(
            tx,
            rewriter,
            0,
            1,
            true,
            options.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::RewriteBeigeJudgment,
            payload,
        );
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(beige_stage_effect(
            policy,
            player,
            target,
            source,
            3,
            result_card,
            suit,
            rank,
            Vec::new(),
            skill,
        ));
    run_effects(
        tx,
        vec![EffectRequest::EmitJudgmentResolved {
            player: target,
            result_card,
            suit,
            rank,
            cause_card: CardId::NONE,
        }],
    )
}

pub(crate) fn clear_phase_gift_target(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(instance) = tx.state_mut().skills.get_mut(skill) else {
        return Ok(Vec::new());
    };
    if instance.owner != player {
        return Err(EffectError::ContentFault(
            "phase gift state belongs to another player".into(),
        ));
    }
    instance.clear_phase_gift_target();
    Ok(Vec::new())
}

pub(crate) fn skip_current_phase(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().turn.current != player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "skip-current-phase owner is no longer current and alive".into(),
        ));
    }
    let phase = tx.state().turn.phase;
    if phase == Phase::Idle {
        return Err(EffectError::ContentFault(
            "skip_current_phase is not valid while idle".into(),
        ));
    }

    let mut events = vec![crate::command::turn::record_phase_skipped(
        tx, player, phase,
    )];
    if crate::query::restore_skipped_phase(tx.state(), player) {
        return Ok(events);
    }
    let transition = match phase {
        Phase::Draw => {
            if tx.state().turn.skip_play() {
                tx.state_mut().turn.set_flag(turn_flag::SKIP_PLAY, 0);
                events.push(crate::command::turn::record_phase_skipped(
                    tx,
                    player,
                    Phase::Play,
                ));
                set_turn(tx, player, Phase::Discard.to_u8())?
            } else {
                set_turn(tx, player, Phase::Play.to_u8())?
            }
        }
        Phase::Play if tx.state().turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE) != 0 => {
            advance_opening_extra_play(tx, player)?
        }
        Phase::Play => set_turn(tx, player, Phase::Discard.to_u8())?,
        Phase::Discard => set_turn(tx, player, Phase::End.to_u8())?,
        Phase::Prepare | Phase::Judge | Phase::End => {
            crate::command::turn::finish_current_phase_abruptly(tx)?;
            Vec::new()
        }
        Phase::Idle => unreachable!(),
    };
    events.extend(transition);
    Ok(events)
}

pub(crate) fn finish_current_phase_abort_resolution(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal(
            "phase-abort skill owner is missing".into(),
        ));
    }
    match stage {
        0 => {
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("processing zone is missing".into()))?;
            let cards = tx
                .state()
                .zones
                .get(processing)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("discard pile is missing".into()))?;
            move_cards_with_reason(
                tx,
                cards,
                discard,
                Some(processing),
                CardMoveReason::Discard,
            )
        }
        1 => {
            crate::command::turn::finish_current_phase_abruptly(tx)?;
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "invalid phase-abort stage".into(),
        )),
    }
}

pub(crate) fn choose_other_targets_by_skipped_phase_count_damage(
    tx: &mut Transaction,
    player: PlayerId,
    amount: u32,
    skill: engine_api::SkillId,
    draw_per_damage: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if amount == 0
        || !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
    {
        return Ok(Vec::new());
    }
    let skipped = tx.state().turn.skipped_phase_count();
    if skipped == 0 {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .collect();
    let max = skipped.min(options.len() as u32).min(u8::MAX as u32) as u8;
    if max == 0 {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        max,
        false,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOtherTargetsBySkippedPhaseCountDamage,
        {
            let mut payload = vec![player.raw(), amount, skill.raw()];
            if draw_per_damage {
                payload.push(1);
            }
            payload
        },
    )
}

pub(crate) fn choose_other_damage(
    tx: &mut Transaction,
    player: PlayerId,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if amount == 0
        || !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOtherDamage,
        vec![player.raw(), amount, skill.raw()],
    )
}

pub(crate) fn choose_bound_source_attack_range_damage(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_matches = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .and_then(|instance| instance.bound_target())
        == Some(source);
    if amount == 0
        || !skill_matches
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(source)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != source)
        .filter(|&target| crate::query::player_in_attack_range(tx.state(), source, target))
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseBoundSourceAttackRangeDamage,
        vec![player.raw(), source.raw(), amount, skill.raw()],
    )
}

pub(crate) fn choose_other_discard_target_card(
    tx: &mut Transaction,
    player: PlayerId,
    scope: rule_api::TargetCardScope,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .filter(|&target| {
            !crate::query::discardable_target_cards(tx.state(), player, target, scope).is_empty()
        })
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOtherDiscardTargetCard,
        vec![player.raw(), u32::from(scope.tag()), skill.raw()],
    )
}
