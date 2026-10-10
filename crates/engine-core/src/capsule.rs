//! Replay Capsule — self-contained long-term replay bundle (architecture §22.4).
//!
//! A high-value match's replay may need to be replayable years later, when the
//! exact engine binary, content packages, and compiler/VM versions may no longer
//! be the default. The Replay Capsule bundles everything needed to reproduce
//! the match into a single serializable artifact:
//!
//! - Capsule schema version
//! - Match manifest (engine semantics + ruleset + RNG + budgets)
//! - Artifact build ids (native + wasm)
//! - Compiler / VM ABI compatibility info
//! - Content packages (embedded or referenced)
//! - Replay log bytes
//! - Optional snapshot bytes
//!
//! Two modes:
//! - [`CapsuleMode::Embedded`] — content + replay bytes included inline. The
//!   capsule is self-contained; no external artifacts needed.
//! - [`CapsuleMode::Referenced`] — content referenced by hash; the capsule is
//!   smaller but depends on an external artifact store.

use crate::replay::ReplayLog;
use crate::snapshot::Snapshot;
use engine_api::{
    ArtifactBuildId, EngineSemanticsId, MatchManifest, MatchManifestHash, RuleIrVersion,
    VersionBundle, VmAbiVersion,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Replay Capsule schema version.
pub const CAPSULE_SCHEMA_VERSION: u32 = 1;

/// How content packages are carried in the capsule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CapsuleMode {
    /// Content packages embedded inline (self-contained capsule).
    Embedded = 1,
    /// Content packages referenced by hash (depends on external store).
    Referenced = 2,
}

/// One content package entry in the capsule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleContentEntry {
    /// Package logical id.
    pub package_id: String,
    /// Package version.
    pub version: String,
    /// Content lock hash (BLAKE3 of resolved ruleset + packages).
    pub content_hash: [u8; 32],
    /// Package JSON bytes (present only in Embedded mode).
    #[serde(default)]
    pub package_bytes: Option<Vec<u8>>,
}

/// Compiler / VM compatibility info recorded for long-term replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleCompat {
    /// Rule IR version.
    pub rule_ir_version: RuleIrVersion,
    /// VM ABI version.
    pub vm_abi_version: VmAbiVersion,
    /// Engine API version.
    pub engine_api_version: u32,
    /// Content schema version.
    pub content_schema_version: u32,
    /// Replay schema version.
    pub replay_schema_version: u32,
    /// Snapshot schema version.
    pub snapshot_schema_version: u32,
}

impl CapsuleCompat {
    /// Build compatibility info from the current workspace versions.
    pub fn current() -> Self {
        let v = VersionBundle::current();
        Self {
            rule_ir_version: RuleIrVersion::CURRENT,
            vm_abi_version: VmAbiVersion::CURRENT,
            engine_api_version: v.engine_api,
            content_schema_version: v.content_schema,
            replay_schema_version: v.replay_schema,
            snapshot_schema_version: v.snapshot_schema,
        }
    }
}

/// A self-contained (or referenced) replay bundle (architecture §22.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayCapsule {
    /// Capsule schema version.
    pub capsule_schema: u32,
    /// How content is carried.
    pub mode: CapsuleMode,
    /// Match manifest (pinned at StartGame).
    pub manifest: MatchManifest,
    /// Manifest hash (convenience; equals `manifest.manifest_hash()`).
    pub manifest_hash: MatchManifestHash,
    /// Engine semantics id.
    pub engine_semantics_id: EngineSemanticsId,
    /// Native artifact build id at capture time.
    pub native_artifact: ArtifactBuildId,
    /// WASM artifact build id at capture time.
    pub wasm_artifact: ArtifactBuildId,
    /// Compiler / VM compatibility info.
    pub compat: CapsuleCompat,
    /// Content packages (embedded bytes or hash references).
    pub content: Vec<CapsuleContentEntry>,
    /// Replay log JSON bytes.
    pub replay_bytes: Vec<u8>,
    /// Optional snapshot JSON bytes (seek acceleration).
    #[serde(default)]
    pub snapshot_bytes: Option<Vec<u8>>,
    /// Optional match outcome (if the match completed).
    #[serde(default)]
    pub outcome: Option<engine_api::MatchOutcome>,
}

/// Capsule errors.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CapsuleError {
    /// Schema version mismatch.
    #[error("capsule schema mismatch: found {found}, expected {expected}")]
    Schema {
        /// Found version.
        found: u32,
        /// Expected version.
        expected: u32,
    },
    /// Referenced content package not available in this environment.
    #[error("referenced content package {0} not available")]
    ContentNotAvailable(String),
    /// Manifest hash mismatch between capsule manifest and recomputed hash.
    #[error("capsule manifest hash mismatch")]
    ManifestHashMismatch,
    /// Replay decode failure.
    #[error("replay decode: {0}")]
    ReplayDecode(String),
    /// Snapshot decode failure.
    #[error("snapshot decode: {0}")]
    SnapshotDecode(String),
    /// Serialization failure.
    #[error("serialize: {0}")]
    Serialize(String),
    /// Deserialization failure.
    #[error("deserialize: {0}")]
    Deserialize(String),
    /// Content package parse failure.
    #[error("content package parse: {0}")]
    ContentParse(String),
}

impl ReplayCapsule {
    /// Serialize the capsule as JSON bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, CapsuleError> {
        serde_json::to_vec(self).map_err(|e| CapsuleError::Serialize(e.to_string()))
    }

    /// Deserialize a capsule from JSON bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CapsuleError> {
        serde_json::from_slice(bytes).map_err(|e| CapsuleError::Deserialize(e.to_string()))
    }

    /// Verify the capsule's internal integrity:
    /// - Schema version matches
    /// - Manifest hash matches the manifest's recomputed hash
    /// - Replay bytes decode successfully
    /// - (If present) snapshot bytes decode successfully
    pub fn verify(&self) -> Result<(), CapsuleError> {
        if self.capsule_schema != CAPSULE_SCHEMA_VERSION {
            return Err(CapsuleError::Schema {
                found: self.capsule_schema,
                expected: CAPSULE_SCHEMA_VERSION,
            });
        }
        if self.manifest.manifest_hash() != self.manifest_hash {
            return Err(CapsuleError::ManifestHashMismatch);
        }
        // Replay must decode.
        let _ = ReplayLog::from_bytes(&self.replay_bytes)
            .map_err(|e| CapsuleError::ReplayDecode(e.to_string()))?;
        // Snapshot (optional) must decode if present.
        if let Some(ref snap_bytes) = self.snapshot_bytes {
            let _ = Snapshot::from_bytes(snap_bytes)
                .map_err(|e| CapsuleError::SnapshotDecode(e.to_string()))?;
        }
        Ok(())
    }

    /// Whether this capsule is self-contained (all content embedded).
    pub fn is_self_contained(&self) -> bool {
        self.mode == CapsuleMode::Embedded && self.content.iter().all(|c| c.package_bytes.is_some())
    }

    /// Decode the embedded replay log.
    pub fn replay(&self) -> Result<ReplayLog, CapsuleError> {
        ReplayLog::from_bytes(&self.replay_bytes)
            .map_err(|e| CapsuleError::ReplayDecode(e.to_string()))
    }

    /// Decode the embedded snapshot (if present).
    pub fn snapshot(&self) -> Result<Option<Snapshot>, CapsuleError> {
        match &self.snapshot_bytes {
            Some(b) => Snapshot::from_bytes(b)
                .map(Some)
                .map_err(|e| CapsuleError::SnapshotDecode(e.to_string())),
            None => Ok(None),
        }
    }

    /// Verify the capsule's manifest hash matches an expected pinned hash.
    pub fn verify_manifest(&self, expected: MatchManifestHash) -> Result<(), CapsuleError> {
        if self.manifest_hash != expected {
            return Err(CapsuleError::ManifestHashMismatch);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_api::{
        BudgetProfile, ContentLockHash, MatchConfig, ProtocolProfile, ResolvedRulesetRef,
        RuleRngSpec,
    };

    fn sample_manifest(seed: u64) -> MatchManifest {
        MatchManifest {
            manifest_version: engine_api::MANIFEST_SCHEMA_VERSION,
            engine_semantics_id: EngineSemanticsId::current(),
            rule_ir_version: RuleIrVersion::CURRENT,
            vm_abi_version: VmAbiVersion::CURRENT,
            ruleset: ResolvedRulesetRef::default(),
            content_lock_hash: ContentLockHash::ZERO,
            protocol_profile: ProtocolProfile::Headless,
            rng_spec: RuleRngSpec::new(seed),
            budget_profile: BudgetProfile::default(),
            match_config: MatchConfig {
                ruleset_name: "identity-standard".into(),
                player_count: 4,
                variant_tags: vec![],
            },
            initial_players: vec![],
            versions: VersionBundle::current(),
        }
    }

    #[test]
    fn capsule_schema_version_is_stable() {
        assert_eq!(CAPSULE_SCHEMA_VERSION, 1);
    }

    #[test]
    fn capsule_compat_current_matches_workspace() {
        let c = CapsuleCompat::current();
        assert_eq!(c.rule_ir_version, RuleIrVersion::CURRENT);
        assert_eq!(c.vm_abi_version, VmAbiVersion::CURRENT);
    }

    #[test]
    fn embedded_capsule_round_trips() {
        let manifest = sample_manifest(42);
        let replay = ReplayLog::new([0u8; 32]);
        let replay_bytes = replay.to_bytes().unwrap();
        let capsule = ReplayCapsule {
            capsule_schema: CAPSULE_SCHEMA_VERSION,
            mode: CapsuleMode::Embedded,
            manifest_hash: manifest.manifest_hash(),
            manifest,
            engine_semantics_id: EngineSemanticsId::current(),
            native_artifact: ArtifactBuildId::current_native(),
            wasm_artifact: ArtifactBuildId::current_wasm(),
            compat: CapsuleCompat::current(),
            content: vec![CapsuleContentEntry {
                package_id: "standard".into(),
                version: "1.0.0".into(),
                content_hash: [0u8; 32],
                package_bytes: Some(br#"{"package_id":"standard"}"#.to_vec()),
            }],
            replay_bytes,
            snapshot_bytes: None,
            outcome: None,
        };
        let bytes = capsule.to_bytes().unwrap();
        let restored = ReplayCapsule::from_bytes(&bytes).unwrap();
        assert_eq!(restored.capsule_schema, CAPSULE_SCHEMA_VERSION);
        assert!(restored.is_self_contained());
        assert!(restored.verify().is_ok());
        assert!(restored.replay().is_ok());
    }

    #[test]
    fn schema_mismatch_detected() {
        let manifest = sample_manifest(1);
        let capsule = ReplayCapsule {
            capsule_schema: 999,
            mode: CapsuleMode::Embedded,
            manifest_hash: manifest.manifest_hash(),
            manifest,
            engine_semantics_id: EngineSemanticsId::current(),
            native_artifact: ArtifactBuildId::current_native(),
            wasm_artifact: ArtifactBuildId::current_wasm(),
            compat: CapsuleCompat::current(),
            content: vec![],
            replay_bytes: ReplayLog::new([0u8; 32]).to_bytes().unwrap(),
            snapshot_bytes: None,
            outcome: None,
        };
        let err = capsule.verify().unwrap_err();
        assert!(matches!(err, CapsuleError::Schema { .. }));
    }

    #[test]
    fn manifest_hash_mismatch_detected() {
        let manifest = sample_manifest(1);
        let capsule = ReplayCapsule {
            capsule_schema: CAPSULE_SCHEMA_VERSION,
            mode: CapsuleMode::Embedded,
            manifest_hash: MatchManifestHash([0xff; 32]), // wrong
            manifest,
            engine_semantics_id: EngineSemanticsId::current(),
            native_artifact: ArtifactBuildId::current_native(),
            wasm_artifact: ArtifactBuildId::current_wasm(),
            compat: CapsuleCompat::current(),
            content: vec![],
            replay_bytes: ReplayLog::new([0u8; 32]).to_bytes().unwrap(),
            snapshot_bytes: None,
            outcome: None,
        };
        let err = capsule.verify().unwrap_err();
        assert!(matches!(err, CapsuleError::ManifestHashMismatch));
    }
}
