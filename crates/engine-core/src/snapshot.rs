//! Versioned binary snapshot (restore / seek acceleration only).
//!
//! Command log remains the source of truth. Snapshot must not be required
//! for cross-build replay. Phase 16 adds `manifest_hash` (architecture §23)
//! so a snapshot is self-describing: it records which match it belongs to.

use crate::hash::canonical_state_hash;
use crate::state::GameState;
use engine_api::{ChoiceRequest, MatchManifestHash, VersionBundle, SNAPSHOT_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Snapshot envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    /// Schema version.
    pub schema_version: u32,
    /// Version bundle at capture time.
    pub versions: VersionBundle,
    /// Revision captured.
    pub revision: u64,
    /// Canonical hash at capture (verified on restore).
    pub state_hash: [u8; 32],
    /// Match manifest hash pinned at StartGame (architecture §23).
    /// Zero for pre-start / pre-Phase-16 snapshots.
    #[serde(default)]
    pub manifest_hash: MatchManifestHash,
    /// Published choice contract when the snapshot was taken at an input barrier.
    #[serde(default)]
    pub pending_choice: Option<ChoiceRequest>,
    /// Full canonical state.
    pub state: GameState,
}

/// Snapshot errors.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    /// Schema mismatch.
    #[error("unsupported snapshot schema {found}, expected {expected}")]
    Schema {
        /// Found.
        found: u32,
        /// Expected.
        expected: u32,
    },
    /// Hash mismatch after restore decode.
    #[error("state hash mismatch on restore")]
    HashMismatch,
    /// Manifest hash mismatch: the snapshot belongs to a different match.
    #[error("manifest hash mismatch: snapshot {} != expected {}", .found_hex, .expected_hex)]
    ManifestMismatch {
        /// Found manifest hash hex.
        found_hex: String,
        /// Expected manifest hash hex.
        expected_hex: String,
    },
    /// Serialization failure.
    #[error("serialize error: {0}")]
    Serialize(String),
    /// Deserialization failure.
    #[error("deserialize error: {0}")]
    Deserialize(String),
    /// Choice metadata does not match the canonical continuation state.
    #[error("pending choice does not match snapshot resolution state")]
    ChoiceMismatch,
}

impl Snapshot {
    /// Capture a snapshot from state.
    pub fn capture(state: &GameState) -> Self {
        Self {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            versions: VersionBundle::current(),
            revision: state.revision,
            state_hash: canonical_state_hash(state),
            manifest_hash: MatchManifestHash::ZERO,
            pending_choice: None,
            state: state.clone(),
        }
    }

    /// Capture a snapshot from state, stamped with a manifest hash.
    ///
    /// Use this in live matches so the snapshot is self-describing (§23).
    pub fn capture_with_manifest(state: &GameState, manifest_hash: MatchManifestHash) -> Self {
        let mut snap = Self::capture(state);
        snap.manifest_hash = manifest_hash;
        snap
    }

    /// Restore state, verifying hash.
    pub fn restore(&self) -> Result<GameState, SnapshotError> {
        if self.schema_version != SNAPSHOT_SCHEMA_VERSION {
            return Err(SnapshotError::Schema {
                found: self.schema_version,
                expected: SNAPSHOT_SCHEMA_VERSION,
            });
        }
        let hash = canonical_state_hash(&self.state);
        if hash != self.state_hash {
            return Err(SnapshotError::HashMismatch);
        }
        if let Some(choice) = &self.pending_choice {
            let resolution = &self.state.resolution;
            if resolution.pending_choice_id != choice.choice_id.raw()
                || resolution.pending_observer != choice.observer.raw()
                || resolution.choice_revision != choice.revision
                || resolution.continuation_id != choice.continuation_id
            {
                return Err(SnapshotError::ChoiceMismatch);
            }
        }
        let mut state = self.state.clone();
        // Derived indexes must be rebuilt (architecture §5.2 / §9.2)
        state.rebuild_derived_indexes();
        Ok(state)
    }

    /// Restore state, verifying hash **and** that the snapshot's manifest hash
    /// matches the expected pinned hash (architecture §23).
    ///
    /// A zero snapshot manifest hash (pre-Phase-16 / pre-start) skips the
    /// manifest check for backward compatibility.
    pub fn restore_with_manifest(
        &self,
        expected: MatchManifestHash,
    ) -> Result<GameState, SnapshotError> {
        if !self.manifest_hash.is_zero() && self.manifest_hash != expected {
            return Err(SnapshotError::ManifestMismatch {
                found_hex: hex32(&self.manifest_hash.0),
                expected_hex: hex32(&expected.0),
            });
        }
        self.restore()
    }

    /// Encode as JSON bytes (Phase 1; binary format may replace later).
    pub fn to_bytes(&self) -> Result<Vec<u8>, SnapshotError> {
        serde_json::to_vec(self).map_err(|e| SnapshotError::Serialize(e.to_string()))
    }

    /// Decode from JSON bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SnapshotError> {
        serde_json::from_slice(bytes).map_err(|e| SnapshotError::Deserialize(e.to_string()))
    }
}

fn hex32(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_empty() {
        let s = GameState::empty();
        let snap = Snapshot::capture(&s);
        let restored = snap.restore().unwrap();
        assert_eq!(canonical_state_hash(&s), canonical_state_hash(&restored));
        let bytes = snap.to_bytes().unwrap();
        let snap2 = Snapshot::from_bytes(&bytes).unwrap();
        assert_eq!(snap, snap2);
    }

    #[test]
    fn manifest_hash_round_trips_and_is_verified() {
        let s = GameState::empty();
        let pinned = MatchManifestHash([3u8; 32]);
        let snap = Snapshot::capture_with_manifest(&s, pinned);
        assert_eq!(snap.manifest_hash, pinned);
        // Bytes round-trip preserves the manifest hash.
        let snap2 = Snapshot::from_bytes(&snap.to_bytes().unwrap()).unwrap();
        assert_eq!(snap2.manifest_hash, pinned);
        // Restore with the correct pinned hash succeeds.
        assert!(snap2.restore_with_manifest(pinned).is_ok());
        // Restore with a wrong pinned hash fails (§23).
        let wrong = MatchManifestHash([9u8; 32]);
        assert!(matches!(
            snap2.restore_with_manifest(wrong),
            Err(SnapshotError::ManifestMismatch { .. })
        ));
    }

    #[test]
    fn zero_manifest_hash_skips_check_for_backward_compat() {
        let s = GameState::empty();
        // Default capture produces zero manifest hash (pre-start / legacy).
        let snap = Snapshot::capture(&s);
        assert!(snap.manifest_hash.is_zero());
        // restore_with_manifest with any expected hash must NOT fail on the
        // manifest check (zero = unspecified).
        let any_hash = MatchManifestHash([7u8; 32]);
        assert!(snap.restore_with_manifest(any_hash).is_ok());
    }

    #[test]
    fn prior_schema_is_rejected_before_hash_restore() {
        let mut snapshot = Snapshot::capture(&GameState::empty());
        snapshot.schema_version = SNAPSHOT_SCHEMA_VERSION - 1;
        assert_eq!(
            snapshot.restore(),
            Err(SnapshotError::Schema {
                found: SNAPSHOT_SCHEMA_VERSION - 1,
                expected: SNAPSHOT_SCHEMA_VERSION,
            })
        );
    }

    #[test]
    fn pending_choice_round_trips_and_is_bound_to_resolution() {
        let mut state = GameState::empty();
        state.resolution.pending_choice_id = 7;
        state.resolution.pending_observer = 2;
        state.resolution.choice_revision = 11;
        state.resolution.continuation_id = 19;
        let choice = ChoiceRequest {
            choice_id: engine_api::ChoiceId::new(7),
            observer: engine_api::PlayerId::new(2),
            options: vec![engine_api::ChoiceOption {
                token: 42,
                label_key: String::new(),
            }],
            min_count: 1,
            max_count: 1,
            allow_cancel: false,
            timeout_policy: engine_api::TimeoutPolicy::None,
            revision: 11,
            continuation_id: 19,
            kind: engine_api::ChoiceKind::SelectCards,
        };
        let mut snapshot = Snapshot::capture(&state);
        snapshot.pending_choice = Some(choice.clone());
        let decoded = Snapshot::from_bytes(&snapshot.to_bytes().unwrap()).unwrap();
        assert_eq!(decoded.pending_choice, Some(choice));
        assert!(decoded.restore().is_ok());

        let mut tampered = decoded;
        tampered.pending_choice.as_mut().unwrap().continuation_id += 1;
        assert_eq!(tampered.restore(), Err(SnapshotError::ChoiceMismatch));
    }
}
