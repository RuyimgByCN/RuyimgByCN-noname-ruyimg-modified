//! Command operations for `draw_give_or_discard_hand_cards` and related helpers.

use super::*;

pub(crate) fn draw_give_or_discard_hand_cards(
    tx: &Transaction,
    player: PlayerId,
    discardable_only: bool,
) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let mut cards = if discardable_only {
        crate::query::discardable_hand_equip_cards(tx.state(), player)
    } else {
        tx.state().hand_cards(player)
    };
    cards.retain(|card| {
        tx.state()
            .cards
            .get(*card)
            .is_some_and(|state| state.owner == player && Some(state.zone) == hand)
    });
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}
