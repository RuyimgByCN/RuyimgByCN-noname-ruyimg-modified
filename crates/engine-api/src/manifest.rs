//! MatchManifest — the immutable per-match contract (architecture plan §5).
//!
//! Every match pins its engine semantics, rule IR / VM ABI, ruleset + content
//! lock, RNG spec, budgets and initial players in a single hashable structure.
//! Replays, WAL, snapshots and reconnects all reference its canonical hash.
//!
//! This module deliberately separates three identity axes that the v2.0 review
//! flagged as commonly conflated:
//!
//! - [`EngineSemanticsId`] — rule semantics (Native & WASM may differ as
//!   binaries but share one semantics id once cross-target tests pass).
//! - [`ArtifactBuildId`] — a concrete binary build (deployment / rollback /
//!   exact reproduction).
//! - [`MatchManifestHash`] — hash of the normalized manifest itself.

use crate::ids::{PlayerId, RulesetId, SeatIndex};
use crate::version::VersionBundle;
use serde::{Deserialize, Serialize};

/// Semantic version tag for the rule IR (architecture §5 / §18.2).
///
/// Bumped whenever the Rule IR type or opcode set changes in a way that would
/// change replay outcomes or break bytecode verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(transparent)]
pub struct RuleIrVersion(pub u32);

impl RuleIrVersion {
    /// Current rule IR version baked into the workspace.
    pub const CURRENT: Self = Self(129);
    /// Raw value.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// VM ABI version — the bytecode calling convention + host API surface.
///
/// Distinct from [`RuleIrVersion`] because an IR revision may keep a stable VM
/// ABI, or vice-versa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(transparent)]
pub struct VmAbiVersion(pub u32);

impl VmAbiVersion {
    /// Current VM ABI version baked into the workspace.
    pub const CURRENT: Self = Self(3);
    /// Raw value.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Rule-relevant engine semantics identity.
///
/// Native and WASM artifacts can share one [`EngineSemanticsId`] as long as the
/// cross-target conformance suite passes for every official content release.
/// This is the value that enters the determinism contract, *not* the binary
/// build id.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EngineSemanticsId {
    /// Human-readable semantic label, e.g. `"noname-next/semantics/v1"`.
    pub label: String,
    /// Monotonic semantic revision.
    pub revision: u32,
}

impl EngineSemanticsId {
    /// Current workspace semantics identity.
    pub fn current() -> Self {
        Self {
            label: "noname-next/semantics/v1".into(),
            revision: 255,
        }
    }
}

/// A concrete binary artifact identity (Native / WASM / server / client).
///
/// Used for deployment, diagnostics, rollback and exact reproduction. Two
/// different builds (e.g. Native and WASM) normally carry different artifact
/// build ids but the same [`EngineSemanticsId`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactBuildId {
    /// Artifact label, e.g. `"native"` / `"wasm"` / `"server"`.
    pub target: String,
    /// Build identifier (commit + profile + toolchain, reproducible).
    pub build: String,
}

impl ArtifactBuildId {
    /// Current native build id (mirrors `ENGINE_BUILD_ID`).
    pub fn current_native() -> Self {
        Self {
            target: "native".into(),
            build: crate::ENGINE_BUILD_ID.into(),
        }
    }

    /// Current WASM build id.
    pub fn current_wasm() -> Self {
        Self {
            target: "wasm".into(),
            build: format!("{}-wasm", crate::ENGINE_BUILD_ID),
        }
    }
}

/// 32-byte content lock hash pinning the exact resolved ruleset + packages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(transparent)]
pub struct ContentLockHash(pub [u8; 32]);

impl ContentLockHash {
    /// Zero hash (no content packages loaded — bootstrap ruleset).
    pub const ZERO: Self = Self([0u8; 32]);

    /// Whether this is the all-zero (uninitialized / bootstrap) hash.
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }
}

/// Versioned rule RNG specification (algorithm + seed encoding).
///
/// Only the spec lives in the manifest; the live RNG *state* lives in
/// `GameState`. Replay reproducibility depends on both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleRngSpec {
    /// Algorithm tag.
    pub algorithm: RngAlgorithm,
    /// Major algorithm version (bumped on sampling-method changes).
    pub version: u32,
    /// Initial seed encoded as a 64-bit word (xorshift64* default).
    pub initial_seed: u64,
}

/// Supported rule RNG algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum RngAlgorithm {
    /// xorshift64* — current workspace default.
    #[default]
    Xorshift64Star = 1,
}

impl RuleRngSpec {
    /// Build the default spec for a given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            algorithm: RngAlgorithm::Xorshift64Star,
            version: 1,
            initial_seed: seed,
        }
    }
}

/// Budget profile pinning effect / trigger / VM fuel caps for a match.
///
/// Budgets are part of the manifest (not runtime-configurable mid-match) so
/// that `BudgetExceeded` is deterministic and replayable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetProfile {
    /// Max effects per resolution segment.
    pub effect_budget: u32,
    /// Max trigger chain depth per segment.
    pub trigger_budget: u32,
    /// Default VM fuel per segment.
    pub vm_fuel: u64,
}

impl Default for BudgetProfile {
    fn default() -> Self {
        Self {
            effect_budget: 10_000,
            trigger_budget: 1_000,
            vm_fuel: 1_000_000,
        }
    }
}

/// Wire-level match configuration captured in the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchConfig {
    /// Mode / ruleset logical name (e.g. `"identity-standard"`).
    pub ruleset_name: String,
    /// Seat count.
    pub player_count: u8,
    /// Optional mode variant tags.
    pub variant_tags: Vec<String>,
}

/// One player's initial pinning (seat + config) at match start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitialPlayer {
    /// Player entity id assigned at bootstrap.
    pub player: PlayerId,
    /// Seat index.
    pub seat: SeatIndex,
}

/// Reference to a resolved ruleset (id + logical name).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedRulesetRef {
    /// Numeric ruleset id.
    pub id: RulesetId,
    /// Logical name.
    pub name: String,
}

impl Default for ResolvedRulesetRef {
    fn default() -> Self {
        Self {
            id: RulesetId::new(1),
            name: "bootstrap".into(),
        }
    }
}

/// Wire protocol profile tag (kept lightweight; full negotiation is server-side).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum ProtocolProfile {
    /// Headless / offline / single-process.
    #[default]
    Headless = 0,
    /// Standard multiplayer protobuf.
    MultiplayerStandard = 1,
}

/// The immutable per-match contract (architecture plan §5).
///
/// Constructed at match start, hashed, and then frozen for the lifetime of the
/// match. Replays, WAL entries, snapshots and reconnect handshakes all carry
/// the [`MatchManifest::manifest_hash`] so that divergence is detected early.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchManifest {
    /// Manifest schema version.
    pub manifest_version: u32,
    /// Engine semantics id (rule semantics, shared across targets).
    pub engine_semantics_id: EngineSemanticsId,
    /// Rule IR version.
    pub rule_ir_version: RuleIrVersion,
    /// VM ABI version.
    pub vm_abi_version: VmAbiVersion,
    /// Resolved ruleset reference.
    pub ruleset: ResolvedRulesetRef,
    /// Content dependency lock hash.
    pub content_lock_hash: ContentLockHash,
    /// Protocol profile.
    pub protocol_profile: ProtocolProfile,
    /// RNG specification.
    pub rng_spec: RuleRngSpec,
    /// Budget profile.
    pub budget_profile: BudgetProfile,
    /// Wire match configuration.
    pub match_config: MatchConfig,
    /// Initial player order.
    pub initial_players: Vec<InitialPlayer>,
    /// Version bundle snapshot at match start.
    pub versions: VersionBundle,
}

/// 32-byte hash of a normalized [`MatchManifest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(transparent)]
pub struct MatchManifestHash(pub [u8; 32]);

impl MatchManifestHash {
    /// All-zero sentinel (pre-start / no manifest pinned yet).
    pub const ZERO: Self = Self([0u8; 32]);

    /// Whether this is the all-zero sentinel.
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }
}

/// Current manifest schema version.
pub const MANIFEST_SCHEMA_VERSION: u32 = 1;

/// Why a manifest-related operation was rejected (stable error codes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManifestRejectionReason {
    /// Match already started — manifest is frozen (architecture §5, §41).
    AlreadyFrozen,
    /// Snapshot's manifest hash does not match the expected pinned hash.
    SnapshotInconsistent,
    /// Content lock hash mismatch on attempted package load mid-match.
    ContentLockMismatch,
    /// Engine semantics id mismatch on replay.
    SemanticsMismatch,
}

impl MatchManifest {
    /// Compute the canonical hash of this manifest.
    ///
    /// The encoding is independent of `GameState` canonical encoding and of any
    /// Protobuf wire bytes. Field order is fixed and documented.
    pub fn manifest_hash(&self) -> MatchManifestHash {
        let mut h = blake3::Hasher::new();
        h.update(b"noname-next/match-manifest/v1");

        put_u32(&mut h, self.manifest_version);
        put_str(&mut h, &self.engine_semantics_id.label);
        put_u32(&mut h, self.engine_semantics_id.revision);
        put_u32(&mut h, self.rule_ir_version.raw());
        put_u32(&mut h, self.vm_abi_version.raw());
        put_u32(&mut h, self.ruleset.id.raw());
        put_str(&mut h, &self.ruleset.name);
        h.update(&self.content_lock_hash.0);
        put_u8(&mut h, self.protocol_profile as u8);
        put_u8(&mut h, self.rng_spec.algorithm as u8);
        put_u32(&mut h, self.rng_spec.version);
        put_u64(&mut h, self.rng_spec.initial_seed);
        put_u32(&mut h, self.budget_profile.effect_budget);
        put_u32(&mut h, self.budget_profile.trigger_budget);
        put_u64(&mut h, self.budget_profile.vm_fuel);
        put_str(&mut h, &self.match_config.ruleset_name);
        put_u8(&mut h, self.match_config.player_count);
        put_u32(&mut h, self.match_config.variant_tags.len() as u32);
        for t in &self.match_config.variant_tags {
            put_str(&mut h, t);
        }
        put_u32(&mut h, self.initial_players.len() as u32);
        for p in &self.initial_players {
            put_u32(&mut h, p.player.raw());
            put_u8(&mut h, p.seat.raw());
        }
        put_str(&mut h, &self.versions.engine_build_id);
        put_u32(&mut h, self.versions.engine_api);
        put_u32(&mut h, self.versions.rule_vm_abi);
        put_u32(&mut h, self.versions.content_schema);
        put_u32(&mut h, self.versions.network_protocol);
        put_u32(&mut h, self.versions.replay_schema);
        put_u32(&mut h, self.versions.snapshot_schema);
        put_u32(&mut h, self.versions.package_format);

        MatchManifestHash(*h.finalize().as_bytes())
    }
}

fn put_u8(h: &mut blake3::Hasher, v: u8) {
    h.update(&[v]);
}
fn put_u32(h: &mut blake3::Hasher, v: u32) {
    h.update(&v.to_le_bytes());
}
fn put_u64(h: &mut blake3::Hasher, v: u64) {
    h.update(&v.to_le_bytes());
}
fn put_str(h: &mut blake3::Hasher, s: &str) {
    put_u32(h, s.len() as u32);
    h.update(s.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_hash_is_stable_and_nonzero() {
        let m = MatchManifest {
            manifest_version: MANIFEST_SCHEMA_VERSION,
            engine_semantics_id: EngineSemanticsId::current(),
            rule_ir_version: RuleIrVersion::CURRENT,
            vm_abi_version: VmAbiVersion::CURRENT,
            ruleset: ResolvedRulesetRef::default(),
            content_lock_hash: ContentLockHash::ZERO,
            protocol_profile: ProtocolProfile::Headless,
            rng_spec: RuleRngSpec::new(42),
            budget_profile: BudgetProfile::default(),
            match_config: MatchConfig {
                ruleset_name: "identity-standard".into(),
                player_count: 4,
                variant_tags: vec![],
            },
            initial_players: vec![],
            versions: VersionBundle::current(),
        };
        let h1 = m.manifest_hash();
        let h2 = m.manifest_hash();
        assert_eq!(h1, h2);
        assert_ne!(h1, MatchManifestHash::ZERO);
    }

    #[test]
    fn different_seed_changes_hash() {
        let base = |seed: u64| MatchManifest {
            manifest_version: MANIFEST_SCHEMA_VERSION,
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
        };
        assert_ne!(base(1).manifest_hash(), base(2).manifest_hash());
    }
}
