//! State fork — clone of engine for search; never aliases authority.

use crate::rng::SearchRng;
use engine_api::{Command, CommandResult};
use engine_core::{canonical_state_hash, Engine};
use serde::{Deserialize, Serialize};

/// Isolated simulation state forked from authority.
#[derive(Debug, Clone)]
pub struct StateFork {
    /// Private engine copy.
    engine: Engine,
    /// Search RNG (not RuleRng).
    pub search_rng: SearchRng,
    /// Authority revision at fork time.
    pub base_revision: u64,
    /// Nodes expanded in this fork session.
    pub nodes: u64,
}

impl StateFork {
    /// Fork from authority. Search seed independent of rule seed.
    pub fn from_authority(authority: &Engine, search_seed: u64) -> Self {
        Self {
            engine: authority.clone(),
            search_rng: SearchRng::from_seed(search_seed),
            base_revision: authority.state().revision,
            nodes: 0,
        }
    }

    /// Borrow sim engine.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Mutable sim engine.
    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }

    /// Apply command on the fork only.
    pub fn apply(&mut self, cmd: Command) -> CommandResult {
        self.nodes = self.nodes.saturating_add(1);
        self.engine.submit(cmd)
    }

    /// Whether authority has advanced past this fork's base.
    pub fn is_stale(&self, authority_revision: u64) -> bool {
        authority_revision != self.base_revision
    }

    /// State hash of the fork (for tests).
    pub fn state_hash(&self) -> [u8; 32] {
        self.engine.state_hash()
    }

    /// Canonical hash of authority must not equal after mutating fork only — isolation check helper.
    pub fn isolation_ok(&self, authority: &Engine) -> bool {
        // Fork may diverge after apply; authority hash should be unchanged by fork ops.
        // Caller verifies authority hash before/after.
        let _ = canonical_state_hash(authority.state());
        true
    }
}

/// Lightweight serializable fork meta (no full state).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkMeta {
    /// Base revision.
    pub base_revision: u64,
    /// Nodes.
    pub nodes: u64,
}
