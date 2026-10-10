//! Ordered zones for cards.

use engine_api::{CardId, PlayerId, ZoneId};
use serde::{Deserialize, Serialize};

/// Kind of zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneKind {
    /// Shared draw pile.
    DrawPile,
    /// Shared discard pile.
    DiscardPile,
    /// Player hand.
    Hand,
    /// Player equip.
    Equip,
    /// Player judge area.
    Judge,
    /// Processing / virtual area.
    Processing,
    /// Removed from game.
    Removed,
    /// Public cards stored by one skill for a player (noname-style expansion pile).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    Expansion,
}

/// One zone with ordered card list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zone {
    /// Stable id.
    pub id: ZoneId,
    /// Kind.
    pub kind: ZoneKind,
    /// Owner if personal; NONE for shared.
    pub owner: PlayerId,
    /// Cards bottom→top or hand order (stable).
    pub cards: Vec<CardId>,
}

/// Zone store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ZoneStore {
    next_id: u32,
    zones: Vec<Option<Zone>>,
}

impl ZoneStore {
    /// Empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate zone.
    pub fn alloc(&mut self, mut zone: Zone) -> ZoneId {
        self.next_id = self.next_id.max(1);
        let id = ZoneId::new(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        zone.id = id;
        let idx = (id.raw() - 1) as usize;
        while self.zones.len() <= idx {
            self.zones.push(None);
        }
        self.zones[idx] = Some(zone);
        id
    }

    /// Get zone.
    pub fn get(&self, id: ZoneId) -> Option<&Zone> {
        if id.is_none() {
            return None;
        }
        let idx = (id.raw() - 1) as usize;
        self.zones.get(idx).and_then(|z| z.as_ref())
    }

    /// Get mutable zone.
    pub fn get_mut(&mut self, id: ZoneId) -> Option<&mut Zone> {
        if id.is_none() {
            return None;
        }
        let idx = (id.raw() - 1) as usize;
        self.zones.get_mut(idx).and_then(|z| z.as_mut())
    }

    /// Find first zone of kind with owner (owner NONE for shared).
    pub fn find(&self, kind: ZoneKind, owner: PlayerId) -> Option<ZoneId> {
        self.zones.iter().find_map(|z| {
            let z = z.as_ref()?;
            if z.kind == kind && z.owner == owner {
                Some(z.id)
            } else {
                None
            }
        })
    }

    /// Sorted live zones by id.
    pub fn sorted_live(&self) -> Vec<&Zone> {
        let mut v: Vec<&Zone> = self.zones.iter().filter_map(|z| z.as_ref()).collect();
        v.sort_by_key(|z| z.id.raw());
        v
    }

    /// Next id.
    pub fn next_id(&self) -> u32 {
        self.next_id
    }

    /// Remove a card id from a zone's order list.
    pub fn remove_card(&mut self, zone: ZoneId, card: CardId) -> bool {
        let Some(z) = self.get_mut(zone) else {
            return false;
        };
        if let Some(i) = z.cards.iter().position(|c| *c == card) {
            z.cards.remove(i);
            true
        } else {
            false
        }
    }

    /// Append card to zone order.
    pub fn push_card(&mut self, zone: ZoneId, card: CardId) -> bool {
        let Some(z) = self.get_mut(zone) else {
            return false;
        };
        z.cards.push(card);
        true
    }
}
