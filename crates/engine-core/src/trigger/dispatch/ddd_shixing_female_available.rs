//! Trigger operations for `ddd_shixing_female_available` and related helpers.

use super::*;

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

/// Map domain events to trigger contexts (needs state for zone owner / kind).
pub fn contexts_from_event(
    state: &crate::state::GameState,
    ev: &DomainEvent,
) -> Option<TriggerContext> {
    match ev {
        DomainEvent::GameStarted { player_count } => Some(TriggerContext {
            kind: EventKind::GameStarted,
            player: PlayerId::NONE,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: u32::from(*player_count),
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
        }),
        DomainEvent::SkillAdded { player, skill } => Some(TriggerContext {
            kind: EventKind::SkillAdded,
            player: *player,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: 0,
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
            added_skill_def: state
                .skills
                .get(*skill)
                .map(|instance| instance.def.raw())
                .unwrap_or(0),
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::DamageDealt {
            source,
            target,
            amount,
            is_lose_hp,
            nature,
        } => Some(TriggerContext {
            kind: if *is_lose_hp {
                EventKind::HpLost
            } else {
                EventKind::Damage
            },
            player: *target,
            source: *source,
            respond_to: PlayerId::NONE,
            amount: *amount,
            nature: *nature,
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
        }),
        DomainEvent::DamagePrevented {
            source,
            target,
            current,
        } => Some(TriggerContext {
            kind: EventKind::DamagePrevented,
            player: *current,
            source: *source,
            respond_to: *target,
            amount: 0,
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
        }),
        DomainEvent::HpRecovered {
            source,
            player,
            amount,
        } => Some(TriggerContext {
            kind: EventKind::HpRecovered,
            player: *player,
            source: *source,
            respond_to: PlayerId::NONE,
            amount: *amount,
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
        }),
        DomainEvent::CardUsed {
            player,
            card,
            targets,
        } => Some(TriggerContext {
            kind: EventKind::CardUsed,
            player: *player,
            source: *player,
            respond_to: PlayerId::NONE,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: card.is_some().then_some(*card),
            card_kind: state
                .cards
                .get(*card)
                .map(|value| crate::query::effective_card_kind(state, *player, value).tag()),
            card_flags: 0,
            target_count: targets.len() as u32,
            targets: targets.clone(),
            damaged_targets: Vec::new(),
            cards: state.resolution.card_source_cards(*player, *card),
            added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::CardResolved {
            source,
            card,
            kind,
            targets,
            damaged_targets,
            source_cards,
            all_sources_were_hand,
        } => {
            let damaged_mask = targets
                .iter()
                .enumerate()
                .filter(|(_, target)| damaged_targets.contains(target))
                .fold(0u32, |mask, (index, _)| {
                    mask | 1u32.checked_shl(index as u32).unwrap_or(0)
                });
            Some(TriggerContext {
                kind: EventKind::CardResolved,
                player: *source,
                source: *source,
                respond_to: PlayerId::NONE,
                amount: damaged_mask,
                // CardResolved has no damage nature; carry its bounded (2–8 seats) damaged-target
                // mask separately from `amount`, whose legacy VM bridge also carries target count.
                nature: damaged_mask as u8,
                turn_damage_sources: 0,
                turn_damaged_players: 0,
                move_reason: CardMoveReason::Unspecified,
                moved_has_basic: false,
                from_zone: None,
                to_zone: None,
                card: Some(*card),
                card_kind: Some(*kind),
                // CardResolved uses bit 0 for the event-local all-materials-from-hand fact.
                card_flags: u8::from(*all_sources_were_hand),
                target_count: targets.len() as u32,
                targets: targets.clone(),
                damaged_targets: Vec::new(),
                cards: source_cards.clone(),
                added_skill_def: 0,
                debate_red: Vec::new(),
                debate_black: Vec::new(),
            })
        }
        DomainEvent::DuelResolved {
            card,
            card_flags,
            initiator,
            target,
            winner,
            loser,
        } => Some(TriggerContext {
            kind: EventKind::DuelResolved,
            player: *winner,
            source: *loser,
            respond_to: *initiator,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: Some(*card),
            card_kind: Some(CardKind::TrickJuedou.tag()),
            card_flags: *card_flags,
            target_count: 1,
            targets: vec![*target],
            damaged_targets: Vec::new(),
            cards: vec![],
            added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::EquipSlotDisabled { player, slot } => Some(TriggerContext {
            kind: EventKind::EquipSlotDisabled,
            player: *player,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: u32::from(*slot),
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
        }),
        DomainEvent::EquipSlotEnabled {
            source,
            player,
            slot,
        } => Some(TriggerContext {
            kind: EventKind::EquipSlotEnabled,
            player: *player,
            source: *source,
            respond_to: PlayerId::NONE,
            amount: u32::from(*slot),
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
        }),
        DomainEvent::PlayerTurnedOver {
            player,
            turned_over,
        } => Some(TriggerContext {
            kind: EventKind::CharacterStateChanged,
            player: *player,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: if *turned_over {
                CHARACTER_STATE_TURNED_FACE_DOWN
            } else {
                CHARACTER_STATE_TURNED_FACE_UP
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
        }),
        DomainEvent::PlayerLinkedChanged { player, linked } => Some(TriggerContext {
            kind: EventKind::CharacterStateChanged,
            player: *player,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: if *linked {
                CHARACTER_STATE_BECAME_LINKED
            } else {
                CHARACTER_STATE_BECAME_UNLINKED
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
        }),
        DomainEvent::CardResponded {
            player,
            respond_to,
            card,
            response_kind,
        } => Some(TriggerContext {
            kind: EventKind::CardResponded,
            player: *player,
            source: *player,
            respond_to: *respond_to,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: card.is_some().then_some(*card),
            card_kind: Some(*response_kind),
            card_flags: 0,
            target_count: 0,
            targets: vec![*respond_to],
            damaged_targets: Vec::new(),
            cards: vec![],
            added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::ShaDodged {
            source,
            target,
            card,
        } => Some(TriggerContext {
            kind: EventKind::ShaDodged,
            player: *target,
            source: *source,
            respond_to: PlayerId::NONE,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: Some(*card),
            card_kind: Some(CardKind::Sha.tag()),
            card_flags: 0,
            target_count: 1,
            targets: vec![*target],
            damaged_targets: Vec::new(),
            cards: vec![],
            added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::ShaResolved {
            source,
            card,
            targets,
            response_cards,
            card_flags,
        } => Some(TriggerContext {
            kind: EventKind::ShaResolved,
            player: *source,
            source: *source,
            respond_to: PlayerId::NONE,
            amount: response_cards.len() as u32,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: Some(*card),
            card_kind: Some(CardKind::Sha.tag()),
            card_flags: *card_flags,
            target_count: targets.len() as u32,
            targets: targets.clone(),
            damaged_targets: Vec::new(),
            cards: response_cards.clone(),
            added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::HandCompareStarted {
            player,
            target,
            player_card,
        } => Some(TriggerContext {
            kind: EventKind::HandCompareStarted,
            player: *player,
            source: *target,
            respond_to: PlayerId::NONE,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: Some(*player_card),
            card_kind: state.cards.get(*player_card).map(|card| card.kind.tag()),
            card_flags: 0,
            target_count: 1,
            targets: vec![*target],
            damaged_targets: Vec::new(),
            cards: vec![*player_card],
            added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::DebateResolved {
            initiator,
            red,
            black,
            result,
        } => Some(TriggerContext {
            kind: EventKind::DebateResolved,
            player: *initiator,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: *result as u32,
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
            target_count: (red.len() + black.len()) as u32,
            targets: red.iter().chain(black.iter()).copied().collect(),
            damaged_targets: Vec::new(),
            cards: Vec::new(),
            added_skill_def: 0,
            debate_red: red.clone(),
            debate_black: black.clone(),
        }),
        DomainEvent::DebateStarted {
            initiator,
            participants,
        } => Some(TriggerContext {
            kind: EventKind::DebateStarted,
            player: *initiator,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: participants.len() as u32,
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
            target_count: participants.len() as u32,
            targets: participants.clone(),
            damaged_targets: Vec::new(),
            cards: Vec::new(),
            added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::HandCompareFinished {
            player,
            target,
            player_card,
            target_card,
            winner,
        } => Some(TriggerContext {
            kind: EventKind::HandCompareFinished,
            player: *player,
            source: *target,
            respond_to: *winner,
            // `run_pending_triggers` shares this host scalar with target count. Offset by one so
            // a tie (`NONE == 0`) survives the `max(target_count)` compatibility path.
            amount: winner.raw().saturating_add(1),
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: Some(*player_card),
            card_kind: state.cards.get(*player_card).map(|card| card.kind.tag()),
            card_flags: 0,
            target_count: 1,
            targets: vec![*target],
            damaged_targets: Vec::new(),
            cards: vec![*player_card, *target_card],
            added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::JudgmentResolved {
            player,
            result_card,
            cause_card,
            ..
        } => Some(TriggerContext {
            kind: EventKind::JudgmentResolved,
            player: *player,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: 0,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Unspecified,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            // Legacy `judgeEnd.card` is the delayed trick/equipment cause, while
            // `judgeEnd.result.card` is carried in `cards` for result-card effects.
            card: cause_card.is_some().then_some(*cause_card),
            card_kind: state.cards.get(*cause_card).map(|card| card.kind.tag()),
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: vec![*result_card],
            added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::EnteredDying { player, source } => Some(TriggerContext {
            kind: EventKind::Dying,
            player: *player,
            source: *source,
            respond_to: PlayerId::NONE,
            amount: 0,
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
        }),
        DomainEvent::LeftDying { player } => Some(TriggerContext {
            kind: EventKind::DyingAfter,
            player: *player,
            source: PlayerId::NONE,
            respond_to: PlayerId::NONE,
            amount: 0,
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
        }),
        DomainEvent::MaxHpChanged { player, .. } => Some(TriggerContext {
            kind: EventKind::MaxHpChanged,
            player: *player,
            source: *player,
            respond_to: PlayerId::NONE,
            amount: 0,
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
        }),
        DomainEvent::PlayerKilled {
            player,
            killer,
            hand_equip_cards,
        } => Some(TriggerContext {
            kind: EventKind::Death,
            player: *player,
            source: *killer,
            respond_to: PlayerId::NONE,
            amount: 0,
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
            cards: hand_equip_cards.clone(),
            added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::CardsDrawn { player, cards } => Some(TriggerContext {
            kind: EventKind::CardsDrawn,
            player: *player,
            source: *player,
            respond_to: PlayerId::NONE,
            amount: cards.len() as u32,
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
            cards: cards.clone(),
            added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::CardsGiven {
            giver,
            recipient,
            cards,
        } => Some(TriggerContext {
            kind: EventKind::CardsGiven,
            player: *recipient,
            source: *giver,
            respond_to: PlayerId::NONE,
            amount: cards.len() as u32,
            nature: 0,
            turn_damage_sources: 0,
            turn_damaged_players: 0,
            move_reason: CardMoveReason::Give,
            moved_has_basic: false,
            from_zone: None,
            to_zone: None,
            card: None,
            card_kind: None,
            card_flags: 0,
            target_count: 0,
            targets: vec![],
            damaged_targets: Vec::new(),
            cards: cards.clone(),
            added_skill_def: 0,
            debate_red: Vec::new(),
            debate_black: Vec::new(),
        }),
        DomainEvent::CardsMoved {
            cards,
            from,
            to,
            reason,
        } => {
            let z = state.zones.get(*from);
            Some(TriggerContext {
                kind: EventKind::CardsMoved,
                player: z.map(|z| z.owner).unwrap_or(PlayerId::NONE),
                source: z.map(|z| z.owner).unwrap_or(PlayerId::NONE),
                respond_to: PlayerId::NONE,
                amount: cards.len() as u32,
                nature: 0,
                turn_damage_sources: 0,
                turn_damaged_players: 0,
                move_reason: *reason,
                moved_has_basic: cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.kind.is_basic())
                }),
                from_zone: z.map(|z| z.kind),
                to_zone: Some(*to),
                card: None,
                card_kind: None,
                card_flags: 0,
                target_count: 0,
                targets: vec![],
                damaged_targets: Vec::new(),
                cards: cards.clone(),
                added_skill_def: 0,
                debate_red: Vec::new(),
                debate_black: Vec::new(),
            })
        }
        DomainEvent::RoundEnded { round, current } => Some(TriggerContext {
            kind: EventKind::RoundEnd,
            player: *current,
            source: *current,
            respond_to: PlayerId::NONE,
            amount: *round,
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
        }),
        DomainEvent::RoundStarted { round, current } => Some(TriggerContext {
            kind: EventKind::RoundStart,
            player: *current,
            source: *current,
            respond_to: PlayerId::NONE,
            amount: *round,
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
        }),
        DomainEvent::TurnBegan { player } => Some(TriggerContext {
            kind: EventKind::PhaseBegin,
            player: *player,
            source: *player,
            respond_to: PlayerId::NONE,
            amount: 0,
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
        }),
        DomainEvent::TurnStateChanged { current, phase } => {
            let kind = match *phase {
                engine_api::TurnPhaseTag::Prepare => EventKind::PhasePrepare,
                engine_api::TurnPhaseTag::Judge => EventKind::PhaseJudge,
                engine_api::TurnPhaseTag::Draw => EventKind::PhaseDraw,
                engine_api::TurnPhaseTag::Play => EventKind::PhasePlay,
                engine_api::TurnPhaseTag::Discard => EventKind::PhaseDiscard,
                engine_api::TurnPhaseTag::End => EventKind::PhaseEnd,
                engine_api::TurnPhaseTag::Idle => return None,
            };
            Some(TriggerContext {
                kind,
                player: *current,
                source: *current,
                respond_to: PlayerId::NONE,
                amount: if *phase == engine_api::TurnPhaseTag::End {
                    state.turn.damaged_player_count()
                } else {
                    0
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
            })
        }
        DomainEvent::PhaseEnded {
            current,
            phase,
            damage_sources,
            damaged_players,
            card_targeted_other,
            current_dealt_damage_other,
            ..
        } => {
            let kind = match *phase {
                engine_api::TurnPhaseTag::Draw => EventKind::PhaseDrawEnd,
                engine_api::TurnPhaseTag::Play => EventKind::PhasePlayEnd,
                engine_api::TurnPhaseTag::Discard => EventKind::PhaseDiscardEnd,
                engine_api::TurnPhaseTag::End => EventKind::TurnEnded,
                _ => return None,
            };
            Some(TriggerContext {
                kind,
                player: *current,
                source: *current,
                respond_to: PlayerId::NONE,
                amount: 0,
                nature: 0,
                turn_damage_sources: *damage_sources,
                turn_damaged_players: *damaged_players,
                move_reason: CardMoveReason::Unspecified,
                moved_has_basic: false,
                from_zone: None,
                to_zone: None,
                card: None,
                card_kind: None,
                card_flags: u8::from(*card_targeted_other)
                    | (u8::from(*current_dealt_damage_other) << 1),
                target_count: 0,
                targets: vec![],
                damaged_targets: Vec::new(),
                cards: vec![],
                added_skill_def: 0,
                debate_red: Vec::new(),
                debate_black: Vec::new(),
            })
        }
        DomainEvent::PhaseSkipped { current, phase } => {
            if *phase != engine_api::TurnPhaseTag::Draw {
                return None;
            }
            Some(TriggerContext {
                kind: EventKind::PhaseDrawSkipped,
                player: *current,
                source: *current,
                respond_to: PlayerId::NONE,
                amount: 0,
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
            })
        }
        _ => None,
    }
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
