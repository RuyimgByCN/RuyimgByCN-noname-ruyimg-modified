//! Command operations for `current_card_use_enhancement_extra_targets` and related helpers.

use super::*;

/// Revalidate the suspended one-target Sha/ordinary-trick context used by generic card-use
/// enhancement skills and return all legal additional targets in seat order.
pub(crate) fn current_card_use_enhancement_extra_targets(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
) -> Result<Option<Vec<PlayerId>>, EffectError> {
    if actor != tx.state().turn.current
        || !tx
            .state()
            .players
            .get(actor)
            .is_some_and(|player| player.alive)
    {
        return Ok(None);
    }
    let single_target = if matches!(
        tx.state().resolution.resume_kind,
        crate::state::ContinuationKind::ResumePlayedSha
            | crate::state::ContinuationKind::ResumePlayedShaQueue
    ) {
        saved_sha_targets(tx, actor, card).is_ok_and(|targets| targets.len() == 1)
    } else if tx.state().resolution.resume_kind == crate::state::ContinuationKind::WuxieWindow {
        if tx.state().resolution.resume_payload.get(2) == Some(&trick_effect::TIESUO) {
            saved_tiesuo_target_context(tx, actor, card).is_ok_and(|targets| targets.len() == 1)
        } else {
            saved_independent_trick_target_context(tx, actor, card).is_ok_and(|(kind, targets)| {
                kind.is_trick() && !kind.is_delayed_trick() && targets.len() == 1
            })
        }
    } else {
        false
    };
    if !single_target {
        return Ok(None);
    }

    for other in tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|other| *other != actor)
    {
        let distance = query(
            tx.state(),
            &QueryRequest::Distance {
                from: actor,
                to: other,
            },
            &mut QueryContext::new(),
        );
        if !matches!(distance, QueryResponse::Int(value) if value <= 1) {
            return Ok(None);
        }
    }

    Ok(Some(
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| saved_card_extra_target_is_legal(tx, actor, card, *target))
            .collect(),
    ))
}

/// Revalidate and append one target to the currently suspended one-target Sha or ordinary trick.
pub(crate) fn add_saved_card_target(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
) -> Result<Vec<PlayerId>, EffectError> {
    if !current_card_target_adjustments(tx, actor, card)?
        .0
        .contains(&target)
    {
        return Err(EffectError::Illegal(
            "not a legal bound additional card target".into(),
        ));
    }
    if matches!(
        tx.state().resolution.resume_kind,
        crate::state::ContinuationKind::ResumePlayedSha
            | crate::state::ContinuationKind::ResumePlayedShaQueue
    ) {
        return add_saved_sha_target(tx, actor, card, target);
    }
    let tiesuo = tx.state().resolution.resume_payload.get(2) == Some(&trick_effect::TIESUO);
    let (count_index, targets_start) = if tiesuo { (9, 10) } else { (14, 15) };
    let payload = &mut tx.state_mut().resolution.resume_payload;
    let len = payload[count_index] as usize;
    let mut targets: Vec<_> = payload[targets_start..targets_start + len]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    targets.push(target);
    payload[count_index] = targets.len() as u32;
    payload.insert(targets_start + len, target.raw());
    if actor == tx.state().turn.current && target != actor {
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::CARD_TARGETED_OTHER, 1);
    }
    Ok(targets)
}

pub(crate) fn add_saved_card_target_ignoring_distance(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
) -> Result<Vec<PlayerId>, EffectError> {
    if !saved_card_extra_target_is_legal_ignoring_distance(tx, actor, card, target) {
        return Err(EffectError::Illegal(
            "not a legal 专摄 additional target".into(),
        ));
    }
    if matches!(
        tx.state().resolution.resume_kind,
        crate::state::ContinuationKind::ResumePlayedSha
            | crate::state::ContinuationKind::ResumePlayedShaQueue
    ) {
        return add_saved_sha_target(tx, actor, card, target);
    }
    let tiesuo = tx.state().resolution.resume_payload.get(2) == Some(&trick_effect::TIESUO);
    let (count_index, targets_start) = if tiesuo { (9, 10) } else { (14, 15) };
    let payload = &mut tx.state_mut().resolution.resume_payload;
    let len = payload[count_index] as usize;
    payload.insert(targets_start + len, target.raw());
    payload[count_index] = payload[count_index].saturating_add(1);
    let targets: Vec<_> = payload[targets_start..targets_start + len + 1]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    if actor == tx.state().turn.current && target != actor {
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::CARD_TARGETED_OTHER, 1);
    }
    Ok(targets)
}

pub(crate) fn mark_sha_damage_in_payload(
    kind: crate::state::ContinuationKind,
    payload: &mut [u32],
    source: PlayerId,
    card: CardId,
) {
    let packed_index = match kind {
        crate::state::ContinuationKind::FinishPlayedSha
            if payload.first() == Some(&source.raw()) && payload.get(1) == Some(&card.raw()) =>
        {
            Some(2)
        }
        crate::state::ContinuationKind::RespondAoeQueue
            if payload.first() == Some(&3) && payload.get(1) == Some(&source.raw()) =>
        {
            let len = *payload.get(4).unwrap_or(&0) as usize;
            (payload.get(8 + 2 * len) == Some(&card.raw())).then_some(5)
        }
        _ => None,
    };
    let Some(packed_index) = packed_index else {
        return;
    };
    let Some(packed) = payload.get(packed_index).copied() else {
        return;
    };
    let (nature, flags) = unpack_damage_card_context(packed);
    payload[packed_index] = pack_damage_card_context(
        nature,
        flags | rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE,
    );
}

/// Record committed damage on whichever continuation owns the current post-use Sha rule.
pub(crate) fn mark_current_sha_dealt_damage(tx: &mut Transaction, source: PlayerId, card: CardId) {
    let resolution = &mut tx.state_mut().resolution;
    mark_sha_damage_in_payload(
        resolution.continuation_kind,
        &mut resolution.continuation_payload,
        source,
        card,
    );
    mark_sha_damage_in_payload(
        resolution.resume_kind,
        &mut resolution.resume_payload,
        source,
        card,
    );
}

/// Preserve the post-use boundary across damage, dying, and damage-trigger choices.
pub(crate) fn save_played_sha_finish(
    tx: &mut Transaction,
    source: PlayerId,
    card: CardId,
    packed_context: u32,
) -> Result<(), EffectError> {
    let resolution = &mut tx.state_mut().resolution;
    if resolution.resume_kind != crate::state::ContinuationKind::None {
        return Err(EffectError::EngineFault(
            "converted Sha post-use continuation would overwrite another resume".into(),
        ));
    }
    resolution.resume_kind = crate::state::ContinuationKind::FinishPlayedSha;
    resolution.resume_payload = vec![source.raw(), card.raw(), packed_context];
    Ok(())
}

/// Complete a converted Sha use after every target and nested response/dying window has ended.
pub(crate) fn finish_played_sha(
    tx: &mut Transaction,
    source: PlayerId,
    card: CardId,
    packed_context: u32,
) -> Result<(), EffectError> {
    let (_, flags) = unpack_damage_card_context(packed_context);
    let alive = tx
        .state()
        .players
        .get(source)
        .is_some_and(|player| player.alive);
    let no_damage_skill = tx.state_mut().resolution.take_sha_no_damage_draw(card);
    let resolution_record = tx.state_mut().resolution.take_sha_resolution(source, card);
    let dying_cost_skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == source && skill.twlihuo_dying_cost_card() == Some(card))
        .map(|skill| skill.id);
    let twlihuo_caused_dying = dying_cost_skill.and_then(|skill| {
        tx.state_mut()
            .skills
            .get_mut(skill)
            .unwrap()
            .take_twlihuo_dying_cost(card)
    });
    let mut effects = Vec::new();
    if alive
        && flags & rule_api::damage_card_flag::CONVERTED_FIRE_LOSE_HP_AFTER_DAMAGE != 0
        && twlihuo_caused_dying
            .unwrap_or(flags & rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE != 0)
    {
        effects.push(EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: source,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        });
    }
    if flags & rule_api::damage_card_flag::DRAW_IF_NO_DAMAGE != 0 {
        let skill = no_damage_skill.ok_or_else(|| {
            EffectError::EngineFault("Sha no-damage action lost its provenance".into())
        })?;
        if flags & rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE == 0 {
            if skill.is_none() {
                if resolution_record
                    .as_ref()
                    .is_some_and(|record| record.use_counted && !record.use_refunded)
                {
                    refund_sha_use_count(tx);
                }
            } else if alive {
                effects.push(EffectRequest::DrawCardsFromSkill {
                    player: source,
                    count: 1,
                    skill,
                });
            }
        }
    }
    if let Some(record) = resolution_record {
        let backwater_targets: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| {
                record
                    .backwater_targets
                    .iter()
                    .any(|(_, recorded)| recorded == target)
                    && !record.dying_targets.contains(target)
            })
            .collect();
        effects.push(EffectRequest::EmitShaResolved {
            source,
            card,
            targets: record.targets,
            response_cards: record.response_cards,
            card_flags: flags,
        });
        effects.extend(backwater_targets.into_iter().map(|target| {
            EffectRequest::ChooseDiscardTargetCard {
                player: target,
                target: source,
                scope: rule_api::TargetCardScope::HandEquip,
            }
        }));
    }
    if effects.is_empty() {
        crate::effect::finish_pending_card_replays_if_idle(tx)
    } else {
        run_effects(tx, effects)?;
        crate::effect::finish_pending_card_replays_if_idle(tx)
    }
}

/// Finish and clear the single-target post-use resume when no nested choice interrupted it.
pub(crate) fn finish_saved_played_sha(
    tx: &mut Transaction,
    source: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    let resolution = &mut tx.state_mut().resolution;
    if resolution.resume_kind != crate::state::ContinuationKind::FinishPlayedSha {
        return Ok(());
    }
    let payload = std::mem::take(&mut resolution.resume_payload);
    resolution.resume_kind = crate::state::ContinuationKind::None;
    if payload.first() != Some(&source.raw()) || payload.get(1) != Some(&card.raw()) {
        return Err(EffectError::EngineFault(
            "converted Sha post-use continuation changed source/card".into(),
        ));
    }
    finish_played_sha(tx, source, card, *payload.get(2).unwrap_or(&0))
}

/// Deal one single-target Sha hit and complete its post-use boundary if uninterrupted.
pub(crate) fn deal_single_sha_damage(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    nature: u8,
    card_flags: u8,
    card: CardId,
) -> Result<(), EffectError> {
    save_played_sha_finish(
        tx,
        source,
        card,
        pack_damage_card_context(nature, card_flags),
    )?;
    deal_sha_damage(tx, source, target, amount, nature, card_flags, card)?;
    finish_saved_played_sha(tx, source, card)
}

pub(crate) fn remember_hand_card_suit_use(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    source_was_hand: bool,
) {
    if !source_was_hand
        || actor != tx.state().turn.current
        || !tx.state().content.tracks_turn_hand_card_suit_history()
    {
        return;
    }
    let suit = tx
        .state()
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_suit(tx.state(), actor, card));
    if let Some(suit) = suit {
        tx.state_mut()
            .turn
            .remember_hand_card_suit_used(actor, suit);
    }
}

pub(crate) fn handle_play_card(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    _card_targets: &[CardId],
) -> Result<(), EffectError> {
    if tx.state().revision == 0 {
        return Err(EffectError::Illegal("not started".into()));
    }
    if tx.state().has_pending_choice() {
        return Err(EffectError::Illegal("choice pending".into()));
    }
    if tx.state().turn.phase != Phase::Play {
        return Err(EffectError::Illegal("not play phase".into()));
    }
    if actor.is_none() || actor != tx.state().turn.current {
        return Err(EffectError::Illegal("not current player".into()));
    }
    handle_card_from_hand(tx, actor, card, targets)
}

pub(crate) fn handle_card_from_hand(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    handle_card_from_hand_with_options(tx, actor, card, targets, false, true)
}

pub(crate) fn use_equipment_from_hand(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    let kind = tx
        .state()
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(tx.state(), actor, card))
        .ok_or_else(|| EffectError::Illegal("bad equipment card".into()))?;
    if !kind.is_equip() {
        return Err(EffectError::Illegal("given card is not equipment".into()));
    }
    handle_card_from_hand(tx, actor, card, &[])
}

/// Use an equipment card directly from a shared source zone without fabricating a hand gain.
pub(crate) fn use_equipment_from_shared_zone(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad equipment card".into()))?;
    let kind = crate::query::effective_card_kind(tx.state(), actor, card_state);
    let shared = tx
        .state()
        .zones
        .get(card_state.zone)
        .is_some_and(|zone| zone.owner.is_none());
    if !kind.is_equip() || !shared {
        return Err(EffectError::Illegal(
            "equipment card is not in a shared zone".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(actor)
        .is_some_and(|player| player.alive && !player.dying)
        || !kind.equip_slot().is_some_and(|slot| {
            tx.state()
                .players
                .get(actor)
                .unwrap()
                .equip_slot_enabled(slot)
        })
        || crate::query::card_use_cancelled_for_card(tx.state(), actor, card)
    {
        return Err(EffectError::Illegal(
            "equipment card is not currently usable".into(),
        ));
    }
    tx.state_mut()
        .resolution
        .plan_card_source_cards(actor, card, vec![card], false);
    equip_card(tx, actor, card, kind)
}

pub(crate) fn handle_card_from_hand_with_options(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
    source_was_hand: bool,
) -> Result<(), EffectError> {
    handle_card_from_hand_with_policies(
        tx,
        actor,
        card,
        targets,
        ignore_distance,
        false,
        source_was_hand,
    )
}

pub(crate) fn handle_card_from_hand_with_policies(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
    ignore_use_count: bool,
    source_was_hand: bool,
) -> Result<(), EffectError> {
    handle_card_from_hand_with_policies_and_flags(
        tx,
        actor,
        card,
        targets,
        ignore_distance,
        ignore_use_count,
        source_was_hand,
        0,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_card_from_hand_with_policies_and_flags(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
    ignore_use_count: bool,
    source_was_hand: bool,
    extra_damage_flags: u8,
) -> Result<(), EffectError> {
    handle_card_with_policies_and_flags(
        tx,
        actor,
        card,
        targets,
        ignore_distance,
        ignore_use_count,
        source_was_hand,
        extra_damage_flags,
        false,
    )
}
