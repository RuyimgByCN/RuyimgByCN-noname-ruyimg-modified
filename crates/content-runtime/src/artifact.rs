//! Immutable artifact retention records (build identity, not rule semantics).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One retained build artifact record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRecord {
    /// Exact engine build id.
    pub build_id: String,
    /// Target triple or platform label.
    pub platform: String,
    /// Optional golden case hash (e.g. StartGame(4,42) state hash hex).
    pub golden_hash: Option<String>,
    /// Content lock hash this artifact was certified against.
    pub content_lock_hash: Option<String>,
    /// Retain flag (false → eligible for GC policy).
    pub retained: bool,
    /// Free-form notes (no secrets).
    pub notes: String,
}

/// Registry of retained artifacts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRegistry {
    /// Keyed by `build_id@platform`.
    pub artifacts: BTreeMap<String, ArtifactRecord>,
}

impl ArtifactRegistry {
    fn key(build_id: &str, platform: &str) -> String {
        format!("{build_id}@{platform}")
    }

    /// Insert or update; artifacts are append-friendly (immutable semantics by policy).
    pub fn retain(&mut self, record: ArtifactRecord) {
        let k = Self::key(&record.build_id, &record.platform);
        self.artifacts.insert(k, record);
    }

    /// Lookup.
    pub fn get(&self, build_id: &str, platform: &str) -> Option<&ArtifactRecord> {
        self.artifacts.get(&Self::key(build_id, platform))
    }

    /// Mark not retained (does not delete record — tombstone for audit).
    pub fn mark_unretained(&mut self, build_id: &str, platform: &str) -> bool {
        if let Some(a) = self.artifacts.get_mut(&Self::key(build_id, platform)) {
            a.retained = false;
            true
        } else {
            false
        }
    }
}
