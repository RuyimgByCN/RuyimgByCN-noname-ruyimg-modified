//! State invariants used for engine faults and property tests.

use crate::state::{GameState, ZoneKind};
use engine_api::CardId;
use std::collections::HashSet;
use thiserror::Error;

/// Invariant violation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InvariantError {
    /// Card id does not exist.
    #[error("invalid card id {0}")]
    InvalidCardId(u32),
    /// Player id does not exist.
    #[error("invalid player id {0}")]
    InvalidPlayerId(u32),
    /// Zone id does not exist.
    #[error("invalid zone id {0}")]
    InvalidZoneId(u32),
    /// Same card appears in multiple zones or duplicate in one zone.
    #[error("card {0} location inconsistency")]
    CardLocation(u32),
    /// Zone list card missing from card store or wrong zone field.
    #[error("zone {zone} card {card} mismatch")]
    ZoneCardMismatch {
        /// Zone.
        zone: u32,
        /// Card.
        card: u32,
    },
    /// Card count / conservation failure.
    #[error("card conservation failed: store={store} zone_refs={zone_refs}")]
    Conservation {
        /// Live cards in store.
        store: usize,
        /// Total zone list entries.
        zone_refs: usize,
    },
    /// Skill id missing.
    #[error("invalid skill id {0}")]
    InvalidSkillId(u32),
    /// Player skill list references missing skill.
    #[error("player {player} skill {skill} missing")]
    PlayerSkillMissing {
        /// Player.
        player: u32,
        /// Skill.
        skill: u32,
    },
    /// Equipment remains in a player slot whose disable count is non-zero.
    #[error("player {player} disabled equipment slot {slot} still contains card {card}")]
    DisabledEquipSlotOccupied {
        /// Player.
        player: u32,
        /// Stable slot tag.
        slot: u8,
        /// Equipment card.
        card: u32,
    },
    /// Public skill-pile lookup does not point at its owner's expansion zone.
    #[error("player {owner} skill definition {skill_def} has invalid expansion zone {zone}")]
    InvalidSkillPile {
        /// Pile owner.
        owner: u32,
        /// Owning skill definition.
        skill_def: u32,
        /// Expansion zone.
        zone: u32,
    },
    /// Public skill-pile lookup is not uniquely sorted by owner and skill definition.
    #[error("skill-pile lookup is duplicated or out of canonical order")]
    SkillPileOrder,
}

/// Check all core invariants.
pub fn check_invariants(state: &GameState) -> Result<(), InvariantError> {
    // Players referenced by turn
    if state.turn.current.is_some() && state.players.get(state.turn.current).is_none() {
        return Err(InvariantError::InvalidPlayerId(state.turn.current.raw()));
    }

    // Skills on players
    for p in state.players.iter_live() {
        for sid in &p.skills {
            if state.skills.get(*sid).is_none() {
                return Err(InvariantError::PlayerSkillMissing {
                    player: p.id.raw(),
                    skill: sid.raw(),
                });
            }
        }
    }

    // Public skill-pile lookup is canonical and every entry owns one expansion zone.
    let mut previous_key = None;
    let mut pile_zones = HashSet::new();
    for pile in &state.skill_piles {
        let key = (pile.owner.raw(), pile.skill_def);
        if previous_key.is_some_and(|previous| previous >= key)
            || !pile_zones.insert(pile.zone.raw())
        {
            return Err(InvariantError::SkillPileOrder);
        }
        previous_key = Some(key);
        let Some(zone) = state.zones.get(pile.zone) else {
            return Err(InvariantError::InvalidSkillPile {
                owner: pile.owner.raw(),
                skill_def: pile.skill_def,
                zone: pile.zone.raw(),
            });
        };
        if state.players.get(pile.owner).is_none()
            || zone.kind != ZoneKind::Expansion
            || zone.owner != pile.owner
        {
            return Err(InvariantError::InvalidSkillPile {
                owner: pile.owner.raw(),
                skill_def: pile.skill_def,
                zone: pile.zone.raw(),
            });
        }
    }

    // Card ↔ zone consistency
    let mut seen: HashSet<u32> = HashSet::new();
    let mut zone_refs = 0usize;

    for z in state.zones.sorted_live() {
        let mut local: HashSet<u32> = HashSet::new();
        for cid in &z.cards {
            zone_refs += 1;
            if !local.insert(cid.raw()) {
                return Err(InvariantError::CardLocation(cid.raw()));
            }
            if !seen.insert(cid.raw()) {
                return Err(InvariantError::CardLocation(cid.raw()));
            }
            let Some(card) = state.cards.get(*cid) else {
                return Err(InvariantError::ZoneCardMismatch {
                    zone: z.id.raw(),
                    card: cid.raw(),
                });
            };
            if card.zone != z.id {
                return Err(InvariantError::ZoneCardMismatch {
                    zone: z.id.raw(),
                    card: cid.raw(),
                });
            }
            if z.kind == ZoneKind::Equip {
                if let Some(slot) = card.kind.equip_slot() {
                    if state
                        .players
                        .get(z.owner)
                        .is_some_and(|player| !player.equip_slot_enabled(slot))
                    {
                        return Err(InvariantError::DisabledEquipSlotOccupied {
                            player: z.owner.raw(),
                            slot: slot as u8,
                            card: cid.raw(),
                        });
                    }
                }
            }
        }
    }

    for c in state.cards.sorted_live() {
        if !seen.contains(&c.id.raw()) {
            // Card exists but not in any zone list
            return Err(InvariantError::CardLocation(c.id.raw()));
        }
        if state.zones.get(c.zone).is_none() {
            return Err(InvariantError::InvalidZoneId(c.zone.raw()));
        }
    }

    let store = state.cards.live_count();
    if store != zone_refs {
        return Err(InvariantError::Conservation { store, zone_refs });
    }

    Ok(())
}

/// Assert card is in exactly the given zone list.
pub fn card_in_zone(state: &GameState, card: CardId, zone_kind: ZoneKind) -> bool {
    let Some(c) = state.cards.get(card) else {
        return false;
    };
    let Some(z) = state.zones.get(c.zone) else {
        return false;
    };
    z.kind == zone_kind && z.cards.contains(&card)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_ok() {
        assert!(check_invariants(&GameState::empty()).is_ok());
    }
}
