//! Player store with stable IDs.

use engine_api::{PlayerId, SeatIndex, SkillId};
use serde::{Deserialize, Serialize};

/// Identity mode role (hidden on authority; projected per observer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Identity {
    /// Lord.
    Lord,
    /// Loyalist.
    Loyalist,
    /// Rebel.
    Rebel,
    /// Renegade.
    Renegade,
}

/// One player entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    /// Stable id.
    pub id: PlayerId,
    /// Seat around table.
    pub seat: SeatIndex,
    /// Current HP (may be 0 while dying).
    pub hp: i32,
    /// Max HP.
    pub max_hp: u32,
    /// Alive flag (false after death resolved).
    pub alive: bool,
    /// In dying state machine.
    pub dying: bool,
    /// Face-down character state. A face-down player skips their next whole turn,
    /// then returns face-up.
    #[serde(default)]
    pub turned_over: bool,
    /// Identity (full authority).
    pub identity: Identity,
    /// Character definition raw id (0 = unset).
    pub character_def: u32,
    /// Guozhan lieutenant (副将) definition raw id. 0 in identity mode (single general).
    /// See ADR 0598 §D1.
    #[serde(default)]
    pub character_def2: u32,
    /// Attached skill instance ids (stable order).
    pub skills: Vec<SkillId>,
    /// Mark stacks: (mark_def_raw, count), sorted by mark_def.
    pub marks: Vec<(u32, i32)>,
    /// Disable nesting count for equip1..equip5. A non-zero slot cannot hold equipment.
    #[serde(default)]
    pub disabled_equip_slots: [u8; 5],
    /// Guozhan: whether each general is face-down (暗将). `[main, lieutenant]`.
    /// Identity mode defaults to `[false, false]` (both revealed). A face-down general's
    /// skills do not take effect. See ADR 0598 §D1.
    #[serde(default = "default_unseen")]
    pub unseen: [bool; 2],
}

/// Default for `Player::unseen`: both generals revealed.
fn default_unseen() -> [bool; 2] {
    [false, false]
}

impl Player {
    /// Whether the specified equipment slot is currently enabled.
    pub fn equip_slot_enabled(&self, slot: super::EquipSlot) -> bool {
        self.disabled_equip_slots[slot.index()] == 0
    }
}

/// Dense player store; ids are stable and never reused.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PlayerStore {
    /// Next id to allocate (starts at 1; 0 is NONE).
    next_id: u32,
    /// Slot occupancy parallel to `players` (false = free hole).
    live: Vec<bool>,
    /// Player slots indexed by id-1 when live.
    players: Vec<Option<Player>>,
}

impl PlayerStore {
    /// Empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate a new player.
    pub fn alloc(&mut self, mut player: Player) -> PlayerId {
        self.next_id = self.next_id.max(1);
        let id = PlayerId::new(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        player.id = id;
        let idx = (id.raw() - 1) as usize;
        while self.players.len() <= idx {
            self.players.push(None);
            self.live.push(false);
        }
        self.players[idx] = Some(player);
        self.live[idx] = true;
        id
    }

    /// Get player by id.
    pub fn get(&self, id: PlayerId) -> Option<&Player> {
        if id.is_none() {
            return None;
        }
        let idx = (id.raw() - 1) as usize;
        self.players.get(idx).and_then(|p| p.as_ref())
    }

    /// Get mutable player.
    pub fn get_mut(&mut self, id: PlayerId) -> Option<&mut Player> {
        if id.is_none() {
            return None;
        }
        let idx = (id.raw() - 1) as usize;
        self.players.get_mut(idx).and_then(|p| p.as_mut())
    }

    /// Iterate live players in stable id order.
    pub fn iter_live(&self) -> impl Iterator<Item = &Player> {
        self.players.iter().filter_map(|p| p.as_ref())
    }

    /// Number of live players.
    pub fn live_count(&self) -> usize {
        self.live.iter().filter(|&&l| l).count()
    }

    /// Alive players in seat order (stable).
    pub fn alive_by_seat(&self) -> Vec<PlayerId> {
        let mut v: Vec<&Player> = self.iter_live().filter(|p| p.alive).collect();
        v.sort_by_key(|p| p.seat.raw());
        v.into_iter().map(|p| p.id).collect()
    }

    /// Next id (for hashing).
    pub fn next_id(&self) -> u32 {
        self.next_id
    }

    /// All live players sorted by id (for hashing).
    pub fn sorted_live(&self) -> Vec<&Player> {
        let mut v: Vec<&Player> = self.iter_live().collect();
        v.sort_by_key(|p| p.id.raw());
        v
    }
}
