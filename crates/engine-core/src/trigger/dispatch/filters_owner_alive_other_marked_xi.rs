//! Trigger predicates for `OwnerAliveOtherMarkedXi` and related requests.

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
    let owner_is_source = active.owner == ctx.source;
    let _owner_source_distance_le_one = owner_is_source
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
    let turn_fires = state
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
    let _card_is_trick = card_kind.is_some_and(CardKind::is_trick);
    let _card_is_ordinary_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    // The modelled ordinary tricks have no upstream `multitarget` property; selectTarget alone does not make them multitarget.
    let _card_is_non_multitarget_trick =
        card_kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick());
    let _card_is_sha = matches!(ctx.card_kind, Some(1 | 5 | 6));
    let _card_nature = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .zip(card_kind)
        .map(|(card, kind)| crate::query::effective_card_nature(state, ctx.source, card, kind));
    let _card_is_juedou = ctx.card_kind == Some(CardKind::TrickJuedou.tag());
    let owner_hand = state.hand_cards(active.owner);
    let owner_hand_basic_count = owner_hand
        .iter()
        .filter(|card_id| {
            state.cards.get(**card_id).is_some_and(|card| {
                crate::query::effective_card_kind(state, active.owner, card).is_basic()
            })
        })
        .count();
    let _owner_hand_basic_equals_non_basic = owner_hand_basic_count * 2 == owner_hand.len();
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
    let previous_card_meta = history
        .map(|snapshot| snapshot.previous_card_meta)
        .unwrap_or_else(|| state.turn.flag(turn_flag::LAST_CARD_META) as u32);
    let _previous_suit = previous_card_meta & 0xff;
    let previous_play_card_meta = history
        .map(|snapshot| snapshot.previous_play_card_meta)
        .unwrap_or_else(|| state.turn.flag(turn_flag::LAST_PLAY_CARD_META) as u32);
    let _previous_play_suit = previous_play_card_meta & 0xff;
    let _previous_play_rank = (previous_play_card_meta >> 8) & 0xff;
    let previous_game_card_meta = history
        .map(|snapshot| snapshot.previous_game_card_meta)
        .unwrap_or_else(|| {
            state
                .turn
                .flag(turn_flag::last_game_card_meta_by_player(active.owner)) as u32
        });
    let _previous_game_suit = previous_game_card_meta & 0xff;
    let _previous_game_rank = (previous_game_card_meta >> 8) & 0xff;
    let _current_suit = card_suit.map(crate::command::judge::suit_tag);
    let _current_rank = ctx
        .card
        .and_then(|card| state.cards.get(card))
        .map(|card| u32::from(card.rank));
    let _suit_is_red = |suit| matches!(suit, 2 | 4);
    let _owner_wounded = state
        .players
        .get(active.owner)
        .is_some_and(|owner| owner.alive && owner.hp < owner.max_hp as i32);
    match &def.filter {
        TriggerFilter::OncePerTurn => owner_alive && turn_fires == 0,

        TriggerFilter::DddXuyuPending => {
            ctx.kind == EventKind::TurnEnded
                && crate::effect::ddd_xuyu_pending(state, active.skill_id)
        }

        TriggerFilter::DddShijianWatchedSourceCardUsedTargeted => {
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && ctx.target_count > 0
                && crate::effect::ddd_shijian_watched(state, active.skill_id) == Some(ctx.source)
        }

        TriggerFilter::DddShijianPending => {
            ctx.kind == EventKind::TurnEnded
                && crate::effect::ddd_shijian_watched(state, active.skill_id).is_some()
        }

        TriggerFilter::DddChashiMatchingCard => {
            let pending = state
                .skills
                .get(active.skill_id)
                .and_then(|skill| skill.ddd_chashi_state());
            let suit = ctx
                .card
                .and_then(|card| state.cards.get(card))
                .map(|card| crate::query::effective_card_suit(state, ctx.source, card))
                .map(crate::command::judge::suit_tag)
                .and_then(|value| u8::try_from(value).ok())
                .unwrap_or(0);
            let category = card_kind.map_or(0, |kind| {
                if kind.is_basic() {
                    1
                } else if kind.is_trick() {
                    2
                } else if kind.is_equip() {
                    3
                } else {
                    0
                }
            });
            owner_alive
                && ctx.kind == EventKind::CardUsed
                && pending.is_some_and(|(target, expected_suit, expected_category)| {
                    target == ctx.source
                        && state.turn.current == target
                        && state.turn.phase == Phase::Play
                        && ((suit != 0 && suit == expected_suit)
                            || (category != 0 && category == expected_category))
                })
        }

        _ => unreachable!("filter routed to a different predicate group"),
    }
}
