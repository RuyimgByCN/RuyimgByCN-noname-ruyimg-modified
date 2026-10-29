//! Trigger predicates for `DcyinjunTrackedShaResolved` and related requests.

use super::*;

pub(super) fn filter_match(
    state: &crate::state::GameState,
    active: &ActiveTrigger,
    ctx: &TriggerContext,
    _view_as_skill: u32,
    _view_as_card: u32,
    _history: Option<&TriggerHistorySnapshot>,
    def: &crate::trigger::TriggerDefinition,
) -> bool {
    use crate::state::ZoneKind;
    let _owner_is_player = active.owner == ctx.player;
    let owner_is_source = active.owner == ctx.source;
    let owner_source_distance_le_one = owner_is_source
        && state
            .players
            .get(active.owner)
            .is_some_and(|player| player.alive)
        && matches!(
            query(
                state,
                &QueryRequest::Distance {
                    from: active.owner,
                    to: ctx.player,
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Int(distance) if distance <= 1
        );
    let _owner_source_target_distance_gt_one = owner_is_source
        && state
            .players
            .get(active.owner)
            .is_some_and(|player| player.alive)
        && matches!(
            query(
                state,
                &QueryRequest::Distance {
                    from: active.owner,
                    to: ctx.player,
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Int(distance) if distance > 1
        );
    let _owner_source_wounded_distance_le_one = owner_source_distance_le_one
        && state
            .players
            .get(active.owner)
            .is_some_and(|player| player.hp < player.max_hp as i32);
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
    let _event_player_alive = state
        .players
        .get(ctx.player)
        .map(|player| player.alive)
        .unwrap_or(false);
    let _equip_count = |player_id| {
        state
            .zones
            .find(ZoneKind::Equip, player_id)
            .and_then(|zone| state.zones.get(zone))
            .map_or(0, |zone| zone.cards.len())
    };
    let _owner_is_min_hp = state
        .players
        .get(active.owner)
        .filter(|player| player.alive)
        .map(|owner| {
            state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .all(|player| owner.hp <= player.hp)
        })
        .unwrap_or(false);
    let _owner_is_min_hand = state
        .players
        .get(active.owner)
        .filter(|player| player.alive)
        .map(|_| {
            let owner_hand = state.hand_count(active.owner);
            state
                .players
                .iter_live()
                .all(|player| owner_hand <= state.hand_count(player.id))
        })
        .unwrap_or(false);
    let _hand_empty = state.hand_count(active.owner) == 0;
    let _turn_fires = state
        .skills
        .get(active.skill_id)
        .map(|s| s.turn_fire_count)
        .unwrap_or(0);
    let card_kind = ctx.card_kind.and_then(CardKind::from_tag);
    let _card_is_trick = card_kind.is_some_and(CardKind::is_trick);
    let _card_is_ordinary_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    // The modelled ordinary tricks have no upstream `multitarget` property; selectTarget alone does not make them multitarget.
    let _card_is_non_multitarget_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    let _card_is_sha = matches!(ctx.card_kind, Some(1 | 5 | 6));
    let _single_sha_source_kind = ctx
        .card
        .and_then(|card| {
            state
                .resolution
                .sha_resolution_records
                .iter()
                .find(|record| record.source == ctx.source && record.card == card)
                .and_then(|record| match record.source_cards.as_slice() {
                    [] => Some(card),
                    [source] => Some(*source),
                    _ => None,
                })
        })
        .and_then(|card| state.cards.get(card))
        .map(|card| card.kind);
    let _card_is_juedou = ctx.card_kind == Some(CardKind::TrickJuedou.tag());
    let _owner_was_targeted = ctx.targets.contains(&active.owner);
    let _owner_equip_empty = state
        .zones
        .find(ZoneKind::Equip, active.owner)
        .and_then(|zone| state.zones.get(zone))
        .is_none_or(|zone| zone.cards.is_empty());
    let _owner_hand_below_hp = state
        .players
        .get(active.owner)
        .is_some_and(|player| state.hand_count(active.owner) < player.hp.max(0) as u32);
    let _owner_hand_above_hp = state
        .players
        .get(active.owner)
        .is_some_and(|player| state.hand_count(active.owner) > player.hp.max(0) as u32);
    let _owner_hand_below_max_hp = state
        .players
        .get(active.owner)
        .is_some_and(|player| state.hand_count(active.owner) < player.max_hp);
    let suit_owner = if ctx.source.is_some() {
        ctx.source
    } else {
        ctx.player
    };
    let card_suit = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .map(|card| crate::query::effective_card_suit(state, suit_owner, card));
    let _card_is_red = matches!(
        card_suit,
        Some(engine_api::Suit::Heart | engine_api::Suit::Diamond)
    );
    let _card_is_black = matches!(
        card_suit,
        Some(engine_api::Suit::Spade | engine_api::Suit::Club)
    );
    let _judgment_is_black = ctx.kind == EventKind::JudgmentResolved
        && ctx.cards.first().is_some_and(|result_card| {
            state.cards.get(*result_card).is_some_and(|card| {
                matches!(
                    crate::query::effective_card_suit(state, ctx.player, card),
                    engine_api::Suit::Spade | engine_api::Suit::Club
                )
            })
        });
    let _card_is_diamond = card_suit == Some(engine_api::Suit::Diamond);
    let _card_is_club = card_suit == Some(engine_api::Suit::Club);
    let _event_players_equal_hp = state
        .players
        .get(ctx.source)
        .zip(state.players.get(ctx.player))
        .is_some_and(|(source, target)| source.hp == target.hp);
    let _event_player_has_female_sex = state
        .players
        .get(ctx.player)
        .and_then(|player| state.content.character(player.character_def))
        .is_some_and(|character| matches!(character.sex.as_str(), "female" | "double"));
    let _owner_wounded = state
        .players
        .get(active.owner)
        .is_some_and(|owner| owner.alive && owner.hp < owner.max_hp as i32);
    let _event_player_group = state.player_group(ctx.player);
    let _event_source_group = state.player_group(ctx.source);
    let _owner_damaged_hp_le_source = state
        .players
        .get(active.owner)
        .zip(state.players.get(ctx.source))
        .is_some_and(|(owner, source)| owner.hp < owner.max_hp as i32 && owner.hp <= source.hp);
    match &def.filter {
        TriggerFilter::DcyinjunTrackedShaResolved => {
            ctx.kind == EventKind::CardResolved
                && ctx.card.is_some_and(|card| {
                    state.skills.get(active.skill_id).is_some_and(|instance| {
                        instance.state_blob.len() == 8
                            && instance.state_blob.get(..4) == Some(b"CYJ1")
                            && u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap())
                                == card.raw()
                    })
                })
        }

        _ => unreachable!("filter routed to a different predicate group"),
    }
}
