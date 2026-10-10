//! Optional query cache — never part of canonical state.
//!
//! Architecture §9.2: must be clearable; results with cache on/off identical.

use rule_api::{QueryRequest, QueryResponse};
use std::collections::BTreeMap;

/// Cache key fingerprint (deterministic encoding of request).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CacheKey {
    /// Encoded bytes.
    pub(crate) raw: Vec<u8>,
}

impl CacheKey {
    /// Build from request + dependency revision.
    pub fn from_request(req: &QueryRequest, dep_revision: u64) -> Self {
        let mut raw = Vec::new();
        raw.extend_from_slice(&dep_revision.to_le_bytes());
        match req {
            QueryRequest::Distance { from, to } => {
                raw.push(1);
                raw.extend_from_slice(&from.raw().to_le_bytes());
                raw.extend_from_slice(&to.raw().to_le_bytes());
            }
            QueryRequest::AttackRange { player } => {
                raw.push(2);
                raw.extend_from_slice(&player.raw().to_le_bytes());
            }
            QueryRequest::HandLimit { player } => {
                raw.push(3);
                raw.extend_from_slice(&player.raw().to_le_bytes());
            }
            QueryRequest::CardUsable { player, card } => {
                raw.push(4);
                raw.extend_from_slice(&player.raw().to_le_bytes());
                raw.extend_from_slice(&card.raw().to_le_bytes());
            }
            QueryRequest::TargetLegal { user, card, target } => {
                raw.push(5);
                raw.extend_from_slice(&user.raw().to_le_bytes());
                raw.extend_from_slice(&card.raw().to_le_bytes());
                raw.extend_from_slice(&target.raw().to_le_bytes());
            }
            QueryRequest::SkillUsesRemaining { skill } => {
                raw.push(6);
                raw.extend_from_slice(&skill.raw().to_le_bytes());
            }
            QueryRequest::CardAttribute { card, attr } => {
                raw.push(7);
                raw.extend_from_slice(&card.raw().to_le_bytes());
                raw.extend_from_slice(&attr.to_le_bytes());
            }
            QueryRequest::Relation { a, b } => {
                raw.push(8);
                raw.extend_from_slice(&a.raw().to_le_bytes());
                raw.extend_from_slice(&b.raw().to_le_bytes());
            }
            QueryRequest::Hp { player } => {
                raw.push(9);
                raw.extend_from_slice(&player.raw().to_le_bytes());
            }
            QueryRequest::HandCount { player } => {
                raw.push(10);
                raw.extend_from_slice(&player.raw().to_le_bytes());
            }
            QueryRequest::MaxHp { player } => {
                raw.push(11);
                raw.extend_from_slice(&player.raw().to_le_bytes());
            }
            QueryRequest::Zone { player, kind } => {
                raw.push(12);
                raw.extend_from_slice(&player.raw().to_le_bytes());
                raw.push(*kind);
            }
            QueryRequest::EquippedCount { player } => {
                raw.push(46);
                raw.extend_from_slice(&player.raw().to_le_bytes());
            }
            QueryRequest::CardZone { card } => {
                raw.push(13);
                raw.extend_from_slice(&card.raw().to_le_bytes());
            }
            QueryRequest::MarkCount { player, mark } => {
                raw.push(14);
                raw.extend_from_slice(&player.raw().to_le_bytes());
                raw.extend_from_slice(&mark.to_le_bytes());
            }
            QueryRequest::EmptyEquipSlots { player } => {
                raw.push(15);
                raw.extend_from_slice(&player.raw().to_le_bytes());
            }
            QueryRequest::AliveCount => raw.push(16),
            QueryRequest::CardActionCount { player } => {
                raw.push(17);
                raw.extend_from_slice(&player.raw().to_le_bytes());
            }
            QueryRequest::AliveGroupCount => raw.push(18),
            QueryRequest::UniqueMaxCurrentHp { player } => {
                raw.push(19);
                raw.extend_from_slice(&player.raw().to_le_bytes());
            }
            QueryRequest::UniqueMaxCurrentHpAt { player, value } => {
                raw.push(20);
                raw.extend_from_slice(&player.raw().to_le_bytes());
                raw.extend_from_slice(&value.to_le_bytes());
            }
            QueryRequest::LifetimeDamageDealt { player } => {
                raw.push(21);
                raw.extend_from_slice(&player.raw().to_le_bytes());
            }
            QueryRequest::PublicCardKindCount { kind } => {
                raw.push(22);
                raw.push(*kind);
            }
            QueryRequest::RoundNumberCapped { cap } => {
                raw.push(23);
                raw.extend_from_slice(&cap.to_le_bytes());
            }
            QueryRequest::FieldEquipmentKindCount { kind } => {
                raw.push(24);
                raw.push(*kind);
            }
            QueryRequest::AliveWoundedOrGuimingWuCount { player } => {
                raw.push(25);
                raw.extend_from_slice(&player.raw().to_le_bytes());
            }
            QueryRequest::HandEquipSuitCount { player, suit } => {
                raw.push(26);
                raw.extend_from_slice(&player.raw().to_le_bytes());
                raw.push(*suit);
            }
            QueryRequest::CurrentPhasePlayer => raw.push(27),
            QueryRequest::HandSuitCount { player, suit } => {
                raw.push(28);
                raw.extend_from_slice(&player.raw().to_le_bytes());
                raw.push(*suit);
            }
            QueryRequest::HandColorCount { player, color } => {
                raw.push(29);
                raw.extend_from_slice(&player.raw().to_le_bytes());
                raw.push(*color);
            }
            QueryRequest::SkillContentFlag {
                player,
                skill_def,
                bit,
            } => {
                raw.push(30);
                raw.extend_from_slice(&player.raw().to_le_bytes());
                raw.extend_from_slice(&skill_def.raw().to_le_bytes());
                raw.push(*bit);
            }
            QueryRequest::SkillPileCount { player, skill_def } => {
                raw.push(31);
                raw.extend_from_slice(&player.raw().to_le_bytes());
                raw.extend_from_slice(&skill_def.to_le_bytes());
            }
            QueryRequest::CardsDrawnThisTurn { player } => {
                raw.push(32);
                raw.extend_from_slice(&player.raw().to_le_bytes());
            }
            QueryRequest::DamageDealtThisTurn { player } => {
                raw.push(33);
                raw.extend_from_slice(&player.raw().to_le_bytes());
            }
            QueryRequest::HasSkill { player, skill_def } => {
                raw.push(34);
                raw.extend_from_slice(&player.raw().to_le_bytes());
                raw.extend_from_slice(&skill_def.raw().to_le_bytes());
            }
            QueryRequest::CardNameLength { player, card } => {
                raw.push(35);
                raw.extend_from_slice(&player.raw().to_le_bytes());
                raw.extend_from_slice(&card.raw().to_le_bytes());
            }
            QueryRequest::InPlayPhase => {
                raw.push(36);
            }
            QueryRequest::CardHasTag { card, tag } => {
                raw.push(37);
                raw.extend_from_slice(&card.raw().to_le_bytes());
                raw.extend_from_slice(&tag.to_le_bytes());
            }
            QueryRequest::StorageGet { skill, key } => {
                raw.push(38);
                raw.extend_from_slice(&skill.raw().to_le_bytes());
                raw.extend_from_slice(&key.to_le_bytes());
            }
            QueryRequest::StorageHas { skill, key } => {
                raw.push(39);
                raw.extend_from_slice(&skill.raw().to_le_bytes());
                raw.extend_from_slice(&key.to_le_bytes());
            }
            QueryRequest::CardsUsedThisRound { player } => {
                raw.push(40);
                raw.extend_from_slice(&player.raw().to_le_bytes());
            }
        }
        Self { raw }
    }
}

/// In-memory query cache (host-side; not serialized).
#[derive(Debug, Default)]
pub struct QueryCache {
    enabled: bool,
    map: BTreeMap<CacheKey, QueryResponse>,
    hits: u64,
    misses: u64,
}

impl QueryCache {
    /// Create disabled cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable.
    pub fn set_enabled(&mut self, on: bool) {
        self.enabled = on;
        if !on {
            self.clear();
        }
    }

    /// Is enabled?
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Lookup.
    pub fn get(&mut self, key: &CacheKey) -> Option<QueryResponse> {
        if !self.enabled {
            return None;
        }
        match self.map.get(key) {
            Some(v) => {
                self.hits += 1;
                Some(v.clone())
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }

    /// Insert.
    pub fn insert(&mut self, key: CacheKey, value: QueryResponse) {
        if self.enabled {
            self.map.insert(key, value);
        }
    }

    /// Stats.
    pub fn stats(&self) -> (u64, u64) {
        (self.hits, self.misses)
    }
}
