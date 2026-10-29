//! Trigger operations for `ddd_shixing_female_available` and related helpers.

use super::*;
mod event_contexts;
pub use event_contexts::*;

pub(crate) fn ddd_shixing_female_available(state: &crate::state::GameState) -> bool {
    state.players.alive_by_seat().into_iter().any(|player| {
        state.players.get(player).is_some_and(|participant| {
            participant.hp >= i32::try_from(participant.max_hp).unwrap_or(i32::MAX)
                && state
                    .content
                    .character(participant.character_def)
                    .is_some_and(|character| matches!(character.sex.as_str(), "female" | "double"))
        })
    })
}

pub(crate) fn ddd_player_linked(state: &crate::state::GameState, player: PlayerId) -> bool {
    state.players.get(player).is_some_and(|participant| {
        participant
            .marks
            .iter()
            .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
    })
}

pub(crate) fn ddd_zhangkai_unique_max_equip(state: &crate::state::GameState) -> Option<PlayerId> {
    let counts: Vec<_> = state
        .players
        .alive_by_seat()
        .into_iter()
        .map(|player| {
            let count = state
                .zones
                .find(ZoneKind::Equip, player)
                .and_then(|zone| state.zones.get(zone))
                .map_or(0, |zone| zone.cards.len());
            (player, count)
        })
        .collect();
    let max = counts.iter().map(|(_, count)| *count).max()?;
    let mut leaders = counts.into_iter().filter(|(_, count)| *count == max);
    let leader = leaders.next()?.0;
    leaders.next().is_none().then_some(leader)
}

pub(crate) fn ddd_zhangkai_can_take_equipment(
    state: &crate::state::GameState,
    player: PlayerId,
    actor: PlayerId,
    card: CardId,
) -> bool {
    let processing = state.zones.find(ZoneKind::Processing, PlayerId::NONE);
    let actor_hand = state.zones.find(ZoneKind::Hand, actor);
    state.cards.get(card).is_some_and(|card| {
        (Some(card.zone) == processing || Some(card.zone) == actor_hand)
            && card.kind.equip_slot().is_some_and(|slot| {
                state
                    .players
                    .get(player)
                    .is_some_and(|owner| owner.equip_slot_enabled(slot))
            })
    })
}

pub(crate) fn ddd_zhengjun_move_mask(state: &crate::state::GameState, ctx: &TriggerContext) -> u8 {
    let mut mask = match ctx.from_zone {
        Some(ZoneKind::Hand) => 1,
        Some(ZoneKind::Equip) => 4,
        _ => 0,
    };
    if let Some(kind) = ctx
        .to_zone
        .and_then(|zone| state.zones.get(zone))
        .map(|zone| zone.kind)
    {
        mask |= match kind {
            ZoneKind::Hand => 2,
            ZoneKind::Equip => 8,
            _ => 0,
        };
    }
    mask
}

pub(crate) fn no_player() -> PlayerId {
    PlayerId::NONE
}

pub(crate) fn target_contexts_from_event(
    state: &crate::state::GameState,
    ev: &DomainEvent,
) -> Vec<TriggerContext> {
    let (player, card, targets, added_target) = match ev {
        DomainEvent::CardUsed {
            player,
            card,
            targets,
        } => (*player, *card, targets, None),
        DomainEvent::CardTargetAdded {
            player,
            card,
            target,
            targets,
        } => (*player, *card, targets, Some(*target)),
        _ => return Vec::new(),
    };
    let card_kind = state
        .cards
        .get(card)
        .map(|value| crate::query::effective_card_kind(state, player, value).tag());
    let mut source_cards = state.resolution.card_source_cards(player, card);
    if source_cards.is_empty() {
        source_cards.push(card);
    }
    targets
        .iter()
        .filter(|target| added_target.is_none_or(|added| added == **target))
        .map(|target| TriggerContext {
            kind: EventKind::CardTargeted,
            player: *target,
            source: player,
            respond_to: PlayerId::NONE,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: Some(card),
            card_kind,
            card_flags: 0,
            target_count: targets.len() as u32,
            targets: targets.clone(),
            damaged_targets: Vec::new(),
            cards: source_cards.clone(),
            added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        })
        .collect()
}

/// Plan effect requests from triggers reacting to a batch of domain events.
pub fn dispatch_for_events(
    tx: &mut Transaction,
    events: &[DomainEvent],
    fires: &mut TriggerFireRecord,
) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events_with_damage_context(tx, events, fires, None)
}

/// Dispatch events while attaching scheduler-owned card provenance to damage.
pub(crate) fn dispatch_for_events_with_damage_context(
    tx: &mut Transaction,
    events: &[DomainEvent],
    fires: &mut TriggerFireRecord,
    damage_context: Option<(engine_api::CardId, u8, u8)>,
) -> Result<Vec<EffectRequest>, EffectError> {
    let mut pending = Vec::new();
    for ev in events {
        let planned = (|| -> Result<(), EffectError> {
            if let Some(mut ctx) = contexts_from_event(tx.state(), ev) {
                if ctx.kind == EventKind::Damage {
                    if let Some((card, kind, flags)) = damage_context {
                        if card.is_some() {
                            ctx.card = Some(card);
                        }
                        if kind != 0 {
                            ctx.card_kind = Some(kind);
                        }
                        ctx.card_flags = flags;
                        if card.is_some() {
                            ctx.cards = tx.state().resolution.card_source_cards(ctx.source, card);
                        }
                    }
                }
                if matches!(
                    ev,
                    DomainEvent::TurnStateChanged {
                        phase: engine_api::TurnPhaseTag::Prepare,
                        ..
                    }
                ) && tx.state().turn.flag(turn_flag::EXPLICIT_TURN_BEGIN_EVENT) == 0
                {
                    let mut phase_begin = ctx.clone();
                    phase_begin.kind = EventKind::PhaseBegin;
                    pending.extend(plan_triggers(tx, phase_begin, fires)?);
                }
                pending.extend(plan_triggers(tx, ctx, fires)?);
            }
            if let DomainEvent::PhaseSkipped { current, phase } = ev {
                let phase_any_skipped = TriggerContext {
                    kind: EventKind::PhaseAnySkipped,
                    player: *current,
                    source: *current,
                    respond_to: PlayerId::NONE,
                    amount: match phase {
                        engine_api::TurnPhaseTag::Prepare => 1,
                        engine_api::TurnPhaseTag::Judge => 2,
                        engine_api::TurnPhaseTag::Draw => 3,
                        engine_api::TurnPhaseTag::Play => 4,
                        engine_api::TurnPhaseTag::Discard => 5,
                        engine_api::TurnPhaseTag::End => 6,
                        engine_api::TurnPhaseTag::Idle => 0,
                    },
                    nature: 0,
                    turn_damage_sources: 0,
                    turn_damaged_players: 0,
                    move_reason: CardMoveReason::Unspecified,
                    moved_has_basic: false,
                    from_zone: None,
                    to_zone: None,
                    card: None,
                    card_kind: None,
                    card_flags: 0,
                    target_count: 0,
                    targets: vec![],
                    damaged_targets: Vec::new(),
                    cards: vec![],
                    added_skill_def: 0,
                    debate_red: Vec::new(),
                    debate_black: Vec::new(),
                };
                pending.extend(plan_triggers(tx, phase_any_skipped, fires)?);
            }
            if let DomainEvent::PhaseEnded {
                current,
                phase_targeted_players,
                phase_hp_changed_players,
                ..
            } = ev
            {
                let phase_any_end = TriggerContext {
                    kind: EventKind::PhaseAnyEnd,
                    player: *current,
                    source: *current,
                    respond_to: PlayerId::NONE,
                    amount: 0,
                    nature: 0,
                    turn_damage_sources: *phase_targeted_players,
                    turn_damaged_players: *phase_hp_changed_players,
                    move_reason: CardMoveReason::Unspecified,
                    moved_has_basic: false,
                    from_zone: None,
                    to_zone: None,
                    card: None,
                    card_kind: None,
                    card_flags: 0,
                    target_count: 0,
                    targets: vec![],
                    damaged_targets: Vec::new(),
                    cards: vec![],
                    added_skill_def: 0,
                    debate_red: Vec::new(),
                    debate_black: Vec::new(),
                };
                pending.extend(plan_triggers(tx, phase_any_end, fires)?);
            }
            for ctx in target_contexts_from_event(tx.state(), ev) {
                if ctx.kind == EventKind::CardTargeted
                    && tx.state().content.tracks_card_target_history()
                {
                    tx.state_mut().turn.record_card_targeted(ctx.player);
                }
                pending.extend(plan_triggers(tx, ctx, fires)?);
            }
            Ok(())
        })();
        if let DomainEvent::CardUsed { card, .. } = ev {
            if tx.state().content.tracks_basic_use_history()
                && tx
                    .state()
                    .cards
                    .get(*card)
                    .is_some_and(|card_state| card_state.kind.is_basic())
            {
                tx.state_mut().lifetime_basic_card_uses += 1;
            }
        }
        if let DomainEvent::CardUsed { card, .. } = ev {
            if tx.state().content.tracks_used_type_groups() {
                if let Some(card_state) = tx.state().cards.get(*card) {
                    let bit = 1u8 << card_state.kind.card_type_group().min(2);
                    tx.state_mut().lifetime_used_type_groups |= bit;
                }
            }
        }
        remember_card_action(tx, ev);
        planned?;
    }
    run_pending_triggers(tx, pending)
}
