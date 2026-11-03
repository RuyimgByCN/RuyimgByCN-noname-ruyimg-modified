//! Physical ownership shared by restore validation and authorized read-only views.
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalCardZone {
    Hand,
    Equip,
    Judge,
    DiscardPile,
    Processing,
    DrawPile,
    Removed,
    Expansion,
    /// R05：R 帧已抽未获牌（唯一 owner；对席位/观战隐藏牌面）。
    DrawHeld,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalCard {
    pub id: CardId,
    pub owner: Option<PlayerId>,
    pub zone: PhysicalCardZone,
}

/// R05：从帧栈收集所有 R 帧的 `held_cards`（抽牌所有权区）。
pub fn draw_held_cards(resolution: &crate::snap::ResolutionSnapshot) -> Vec<CardId> {
    let mut out = Vec::new();
    for frame in &resolution.stack {
        if let crate::event::EventData::Draw { held_cards, .. } = &frame.data {
            out.extend(held_cards.iter().copied());
        }
    }
    out
}

/// Enumerate actual card ownership, preserving each pile's order. `held` contains
/// only cards owned by parked machines, collected with `machine_held_cards`.
pub fn physical_cards(
    core: &GameStateCore,
    held: &[CardId],
    resolution: Option<&crate::snap::ResolutionSnapshot>,
) -> Vec<PhysicalCard> {
    use PhysicalCardZone::*;
    let mut cards = Vec::new();
    let mut pile = |ids: &[CardId], owner, zone| {
        cards.extend(ids.iter().map(|id| PhysicalCard {
            id: *id,
            owner,
            zone,
        }));
    };
    for (seat, player) in core.players.iter().enumerate() {
        pile(&player.hand, Some(seat as PlayerId), Hand);
    }
    for (seat, player) in core.players.iter().enumerate() {
        let owner = Some(seat as PlayerId);
        pile(&player.equips, owner, Equip);
        pile(&player.judge, owner, Judge);
    }
    pile(&core.discard, None, DiscardPile);
    pile(&core.table, None, Processing);
    pile(&core.deck, None, DrawPile);
    for (seat, player) in core.players.iter().enumerate() {
        let owner = Some(seat as PlayerId);
        pile(&player.off_court, owner, Removed);
        // yishan_lost, jiguanyuan_cards and similar history fields only reference cards.
        for ids in [
            &player.zhucheng,
            &player.lianyaohu_store,
            &player.sxianjing_traps,
            &player.hutian_hold,
        ] {
            pile(ids, owner, Expansion);
        }
    }
    for ((owner, _), ids) in &core.vm_aux.piles {
        pile(ids, Some(*owner), Expansion);
    }
    pile(held, None, Processing);
    if let Some(res) = resolution {
        for id in draw_held_cards(res) {
            cards.push(PhysicalCard { id, owner: None, zone: PhysicalCardZone::DrawHeld });
        }
    }
    cards
}
