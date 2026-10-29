//! Trigger predicates for `OwnerAliveTurnEndedRehengjiangPending` and related requests.

use super::*;

pub(super) fn filter_match(
    state: &crate::state::GameState,
    active: &ActiveTrigger,
    ctx: &TriggerContext,
    _view_as_skill: u32,
    _view_as_card: u32,
    history: Option<&TriggerHistorySnapshot>,
    def: &crate::trigger::TriggerDefinition,
) -> bool {
    use crate::state::ZoneKind;
    let _owner_is_player = active.owner == ctx.player;
    let _owner_is_source = active.owner == ctx.source;
    let _source_alive = state
        .players
        .get(ctx.source)
        .map(|player| player.alive)
        .unwrap_or(false);
    let owner_alive = state
        .players
        .get(active.owner)
        .map(|player| player.alive)
        .unwrap_or(false);
    let _owner_hand_below_alive_count_capped_seven = owner_alive
        && state.hand_count(active.owner)
            < state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .count()
                .min(7) as u32;
    let _owner_hand_below_alive_count_capped_eight = owner_alive
        && state.hand_count(active.owner)
            < state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .count()
                .min(8) as u32;
    let event_player_alive = state
        .players
        .get(ctx.player)
        .map(|player| player.alive)
        .unwrap_or(false);
    let _event_player_distance_le_owner_hp = state
        .players
        .get(active.owner)
        .filter(|owner| owner.alive)
        .is_some_and(|owner| {
            event_player_alive
                && matches!(
                    query(
                        state,
                        &QueryRequest::Distance {
                            from: ctx.player,
                            to: active.owner,
                        },
                        &mut QueryContext::new(),
                    ),
                    QueryResponse::Int(distance) if distance <= owner.hp
                )
        });
    let _unique_max_hand = |player_id| {
        state
            .players
            .get(player_id)
            .filter(|player| player.alive)
            .is_some_and(|_| {
                let hand = state.hand_count(player_id);
                state
                    .players
                    .iter_live()
                    .filter(|other| other.alive && other.id != player_id)
                    .all(|other| state.hand_count(other.id) < hand)
            })
    };
    let equip_count = |player_id| {
        state
            .zones
            .find(ZoneKind::Equip, player_id)
            .and_then(|zone| state.zones.get(zone))
            .map_or(0, |zone| zone.cards.len())
    };
    let _source_is_max_equip = state
        .players
        .get(ctx.source)
        .filter(|source| source.alive)
        .is_some_and(|_| {
            let source_count = equip_count(ctx.source);
            state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .all(|player| equip_count(player.id) <= source_count)
        });
    let _turn_fires = state
        .skills
        .get(active.skill_id)
        .map(|s| s.turn_fire_count)
        .unwrap_or(0);
    let _play_phase_fires = state
        .skills
        .get(active.skill_id)
        .map(|s| s.play_phase_fire_count)
        .unwrap_or(0);
    let card_kind = ctx.card_kind.and_then(CardKind::from_tag);
    let _card_is_ordinary_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    let _card_is_sha = matches!(ctx.card_kind, Some(1 | 5 | 6));
    let owner_hand = state.hand_cards(active.owner);
    let _owner_hand_basic_count = owner_hand
        .iter()
        .filter(|card_id| {
            state.cards.get(**card_id).is_some_and(|card| {
                crate::query::effective_card_kind(state, active.owner, card).is_basic()
            })
        })
        .count();
    let suit_owner = if ctx.source.is_some() {
        ctx.source
    } else {
        ctx.player
    };
    let card_suit = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .map(|card| crate::query::effective_card_suit(state, suit_owner, card));
    let _card_is_black = matches!(
        card_suit,
        Some(engine_api::Suit::Spade | engine_api::Suit::Club)
    );
    let previous_card_kind = history
        .map(|snapshot| snapshot.previous_card_kind)
        .or_else(|| u8::try_from(state.turn.flag(turn_flag::LAST_CARD_KIND)).ok())
        .and_then(CardKind::from_tag);
    let previous_card_def = history
        .map(|snapshot| snapshot.previous_card_def)
        .unwrap_or_else(|| state.turn.flag(turn_flag::LAST_CARD_DEF) as u32);
    let current_card_def = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .map(|card| card.def.raw());
    let _repeats_previous_card_name = previous_card_kind
        .zip(card_kind)
        .zip(current_card_def)
        .is_some_and(|((previous_kind, current_kind), current_def)| {
            same_logical_card_name(previous_kind, previous_card_def, current_kind, current_def)
        });
    let _event_players_same_sex = state
        .players
        .get(ctx.source)
        .zip(state.players.get(ctx.player))
        .and_then(|(source, target)| {
            state
                .content
                .character(source.character_def)
                .zip(state.content.character(target.character_def))
        })
        .is_some_and(|(source, target)| same_sex(&source.sex, &target.sex));
    let _event_source_has_male_sex = state
        .players
        .get(ctx.source)
        .and_then(|player| state.content.character(player.character_def))
        .is_some_and(|character| matches!(character.sex.as_str(), "male" | "double"));
    let _owner_has_discardable_equipment =
        crate::query::discardable_hand_equip_cards(state, active.owner)
            .into_iter()
            .any(|card| {
                state.cards.get(card).is_some_and(|card_state| {
                    crate::query::effective_card_kind(state, active.owner, card_state).is_equip()
                })
            });
    let _owner_has_weapon = state
        .zones
        .find(ZoneKind::Equip, active.owner)
        .and_then(|zone| state.zones.get(zone))
        .is_some_and(|zone| {
            zone.cards.iter().any(|card| {
                state
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.kind == CardKind::Weapon)
            })
        });
    let _owner_event_player_same_group = state
        .player_group(active.owner)
        .is_some_and(|group| state.player_group(ctx.player) == Some(group));
    let _owner_wounded = state
        .players
        .get(active.owner)
        .is_some_and(|owner| owner.alive && owner.hp < owner.max_hp as i32);
    let _owner_history_bit = active
        .owner
        .raw()
        .checked_sub(1)
        .and_then(|shift| 1_u32.checked_shl(shift));
    match &def.filter {
        TriggerFilter::CilvTrackedCardDamageBefore => {
            ctx.kind == EventKind::DamageBefore
                && ctx.card.is_some_and(|card| {
                    state.skills.get(active.skill_id).is_some_and(|instance| {
                        crate::effect::cilv_tracks_card(instance, card, 0b010)
                    })
                })
        }

        TriggerFilter::CilvTrackedCardResolved => {
            ctx.kind == EventKind::CardResolved
                && ctx.card.is_some_and(|card| {
                    state.skills.get(active.skill_id).is_some_and(|instance| {
                        crate::effect::cilv_tracks_card(instance, card, 0b110)
                    })
                })
        }

        TriggerFilter::DcsbyingmouTrackedCardResolved => {
            ctx.kind == EventKind::CardResolved
                && ctx.card.is_some_and(|card| {
                    state.skills.get(active.skill_id).is_some_and(|instance| {
                        crate::effect::dcsbyingmou_tracks_card(instance, card)
                    })
                })
        }

        TriggerFilter::DcsbmengmouTrackedShaDamage => {
            ctx.kind == EventKind::Damage
                && ctx.card.is_some_and(|card| {
                    state.skills.get(active.skill_id).is_some_and(|instance| {
                        crate::effect::dcsbmengmou_tracks_sha(instance, ctx.source, card)
                    })
                })
        }

        TriggerFilter::DcsbmengmouTrackedShaResolved => {
            ctx.kind == EventKind::CardResolved
                && ctx.card.is_some_and(|card| {
                    state.skills.get(active.skill_id).is_some_and(|instance| {
                        crate::effect::dcsbmengmou_tracks_sha(instance, ctx.source, card)
                    })
                })
        }

        _ => unreachable!("filter routed to a different predicate group"),
    }
}
