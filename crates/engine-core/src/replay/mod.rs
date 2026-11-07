//! Canonical resolution log + hash-chained records (architecture plan §22).
//!
//! The replay log is the **truth source** for match state. Two layers exist:
//!
//! - [`AcceptedCommand`] — the legacy compact entry (command + post-revision +
//!   post-hash). Kept for compatibility with existing golden vectors and the
//!   `Engine::replay_log()` API.
//! - [`ResolutionRecord`] — the v2.0 hash-chained canonical record (§22.1).
//!   Carries pre/post revision + state hash, RNG checkpoints, outcome, and a
//!   `previous_record_hash` link forming a tamper-evident chain (§22.2).
//!
//! Rejected commands **never** enter this log — they go to a separate audit
//! log (§22.1, §12).

use engine_api::{
    Command, MatchManifestHash, MatchOutcome, VersionBundle, ENGINE_BUILD_ID, REPLAY_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};

/// Replay file header (architecture §22.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayHeader {
    /// Schema version.
    pub schema_version: u32,
    /// Engine build id.
    pub engine_build_id: String,
    /// Version axes.
    pub versions: VersionBundle,
    /// Initial RNG seed recorded for convenience (also in first StartGame).
    pub initial_state_hash: [u8; 32],
    /// Budget configuration snapshot.
    pub effect_budget: u32,
    /// Trigger budget.
    pub trigger_budget: u32,
    /// Match manifest hash pinning the whole match (v2.0 §5.3).
    #[serde(default)]
    pub manifest_hash: MatchManifestHash,
}

impl ReplayHeader {
    /// Current header with empty-state hash.
    pub fn new(initial_state_hash: [u8; 32]) -> Self {
        Self {
            schema_version: REPLAY_SCHEMA_VERSION,
            engine_build_id: ENGINE_BUILD_ID.to_string(),
            versions: VersionBundle::current(),
            initial_state_hash,
            effect_budget: 10_000,
            trigger_budget: 1_000,
            manifest_hash: MatchManifestHash::ZERO,
        }
    }

    /// Header with an explicit manifest hash.
    pub fn with_manifest(initial_state_hash: [u8; 32], manifest_hash: MatchManifestHash) -> Self {
        let mut h = Self::new(initial_state_hash);
        h.manifest_hash = manifest_hash;
        h
    }
}

/// One accepted command with post-commit checkpoint (compact / legacy form).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptedCommand {
    /// Normalized command.
    pub command: Command,
    /// Revision after commit.
    pub revision_after: u64,
    /// State hash after commit.
    pub state_hash: [u8; 32],
}

/// Outcome of a single resolution segment (architecture §22.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ResolutionOutcome {
    /// Segment committed normally (no choice barrier).
    Committed = 1,
    /// Segment committed and opened a choice barrier.
    ChoiceBarrierCommitted = 2,
    /// Match completed with this segment.
    MatchCompleted = 3,
    /// Terminal content fault ended the match.
    TerminalContentFault = 4,
    /// Terminal engine fault ended the match.
    TerminalEngineFault = 5,
    /// Admin / external abort.
    AdminAborted = 6,
}

impl Default for ResolutionOutcome {
    fn default() -> Self {
        Self::Committed
    }
}

/// Checkpoint of rule RNG state (algorithm + state + call count).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RuleRngCheckpoint {
    /// Current xorshift64* state word.
    pub state: u64,
    /// Number of draws consumed so far.
    pub call_count: u64,
}

/// 32-byte record hash (BLAKE3 of the normalized record, chaining previous).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(transparent)]
pub struct RecordHash(pub [u8; 32]);

impl RecordHash {
    /// All-zero sentinel — the "previous" hash of the first record.
    pub const ZERO: RecordHash = RecordHash([0u8; 32]);
}

/// Canonical resolution record forming a hash chain (architecture §22.1).
///
/// Only successfully-committed resolution segments produce a record. Rejected
/// commands never appear here; they enter a separate audit log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionRecord {
    /// Monotonic record sequence (1-based).
    pub record_sequence: u64,
    /// The canonical command that drove this segment.
    pub command: Command,
    /// Revision before the segment committed.
    pub pre_revision: u64,
    /// Revision after commit.
    pub post_revision: u64,
    /// State hash before the segment.
    pub pre_state_hash: [u8; 32],
    /// State hash after commit.
    pub post_state_hash: [u8; 32],
    /// RNG state before the segment.
    pub rng_before: RuleRngCheckpoint,
    /// RNG state after the segment.
    pub rng_after: RuleRngCheckpoint,
    /// Segment outcome classification.
    pub outcome: ResolutionOutcome,
    /// Terminal match outcome (only meaningful when `outcome` is terminal).
    #[serde(default)]
    pub match_outcome: Option<MatchOutcome>,
    /// Hash of the previous record (chain link). Zero for the first record.
    pub previous_record_hash: RecordHash,
    /// This record's hash (chain link).
    pub record_hash: RecordHash,
}

impl ResolutionRecord {
    /// Compute the deterministic hash of a record *excluding* the
    /// `record_hash` field itself (which is derived from the rest).
    ///
    /// This is what links a record to its successor: the next record's
    /// `previous_record_hash` must equal this record's `record_hash`.
    pub fn compute_hash(&self) -> RecordHash {
        let mut h = blake3::Hasher::new();
        h.update(b"noname-next/resolution-record/v1");
        put_u64(&mut h, self.record_sequence);
        // command canonical bytes via serde_json (stable field order in our types)
        let cmd_bytes = serde_json::to_vec(&self.command).unwrap_or_default();
        put_u32(&mut h, cmd_bytes.len() as u32);
        h.update(&cmd_bytes);
        put_u64(&mut h, self.pre_revision);
        put_u64(&mut h, self.post_revision);
        h.update(&self.pre_state_hash);
        h.update(&self.post_state_hash);
        put_u64(&mut h, self.rng_before.state);
        put_u64(&mut h, self.rng_before.call_count);
        put_u64(&mut h, self.rng_after.state);
        put_u64(&mut h, self.rng_after.call_count);
        put_u8(&mut h, self.outcome as u8);
        // match outcome canonical tag
        if let Some(ref o) = self.match_outcome {
            put_u8(&mut h, outcome_tag(o));
        } else {
            put_u8(&mut h, 0);
        }
        h.update(&self.previous_record_hash.0);
        RecordHash(*h.finalize().as_bytes())
    }
}

fn outcome_tag(o: &MatchOutcome) -> u8 {
    match o {
        MatchOutcome::InProgress => 0,
        MatchOutcome::RebelWin => 1,
        MatchOutcome::LoyalistWin => 2,
        MatchOutcome::RenegadeWin { .. } => 3,
        MatchOutcome::Aborted { .. } => 4,
        MatchOutcome::Draw => 5,
        // Guozhan faction victory (ADR 0598 §D4). Identity matches never produce this,
        // so identity replay tags are unchanged.
        MatchOutcome::GroupWin { .. } => 6,
        // M4 单挑（single）: last living player wins.
        MatchOutcome::SingleWin { .. } => 7,
        MatchOutcome::TeamWin { .. } => 8,
    }
}

/// In-memory replay log carrying both compact entries and the hash chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayLog {
    /// Header.
    pub header: ReplayHeader,
    /// Accepted commands (compact form), in order.
    pub commands: Vec<AcceptedCommand>,
    /// Hash-chained canonical records (v2.0 §22.1). Parallel to `commands`.
    #[serde(default)]
    pub records: Vec<ResolutionRecord>,
}

impl ReplayLog {
    /// New log for a fresh engine.
    pub fn new(initial_state_hash: [u8; 32]) -> Self {
        Self {
            header: ReplayHeader::new(initial_state_hash),
            commands: Vec::new(),
            records: Vec::new(),
        }
    }

    /// New log with a pinned manifest hash.
    pub fn with_manifest(initial_state_hash: [u8; 32], manifest_hash: MatchManifestHash) -> Self {
        Self {
            header: ReplayHeader::with_manifest(initial_state_hash, manifest_hash),
            commands: Vec::new(),
            records: Vec::new(),
        }
    }

    /// Append accepted command + derived canonical record.
    ///
    /// `rng_before`/`rng_after` are the rule-RNG checkpoints bracketing the
    /// segment; `match_outcome` is `Some` only on terminal segments.
    pub fn push_record(
        &mut self,
        command: Command,
        post_revision: u64,
        pre_revision: u64,
        state_hash: [u8; 32],
        pre_state_hash: [u8; 32],
        rng_before: RuleRngCheckpoint,
        rng_after: RuleRngCheckpoint,
        outcome: ResolutionOutcome,
        match_outcome: Option<MatchOutcome>,
    ) {
        let seq = self.records.len() as u64 + 1;
        let previous = self
            .records
            .last()
            .map(|r| r.record_hash)
            .unwrap_or(RecordHash::ZERO);
        let mut rec = ResolutionRecord {
            record_sequence: seq,
            command: command.clone(),
            pre_revision,
            post_revision,
            pre_state_hash,
            post_state_hash: state_hash,
            rng_before,
            rng_after,
            outcome,
            match_outcome,
            previous_record_hash: previous,
            record_hash: RecordHash::ZERO,
        };
        rec.record_hash = rec.compute_hash();
        self.records.push(rec);
        // Compact form kept in lockstep.
        self.push(AcceptedCommand {
            command,
            revision_after: post_revision,
            state_hash,
        });
    }

    /// Append accepted command (compact form only; legacy path).
    ///
    /// Prefer [`ReplayLog::push_record`] for new code — it maintains the chain.
    pub fn push(&mut self, entry: AcceptedCommand) {
        self.commands.push(entry);
    }

    /// Locate first divergence between two logs (by revision hash).
    pub fn first_divergence(&self, other: &ReplayLog) -> Option<u64> {
        let n = self.commands.len().min(other.commands.len());
        for i in 0..n {
            if self.commands[i].state_hash != other.commands[i].state_hash {
                return Some(self.commands[i].revision_after);
            }
        }
        if self.commands.len() != other.commands.len() {
            Some(n as u64)
        } else {
            None
        }
    }

    /// Verify the hash chain integrity of the canonical records.
    ///
    /// Returns the index of the first broken link, or `Ok(())` if the whole
    /// chain is consistent.
    pub fn verify_chain(&self) -> Result<(), usize> {
        let mut prev = RecordHash::ZERO;
        for (i, rec) in self.records.iter().enumerate() {
            if rec.previous_record_hash != prev {
                return Err(i);
            }
            if rec.record_hash != rec.compute_hash() {
                return Err(i);
            }
            prev = rec.record_hash;
        }
        Ok(())
    }

    /// Whether the log has canonical records (v2.0 form).
    pub fn has_records(&self) -> bool {
        !self.records.is_empty()
    }

    /// Manifest hash pinned in the header (zero if unset).
    pub fn manifest_hash(&self) -> MatchManifestHash {
        self.header.manifest_hash
    }

    /// Serialize the replay log as JSON bytes (architecture §22.3 replay file).
    pub fn to_bytes(&self) -> Result<Vec<u8>, ReplayFileError> {
        serde_json::to_vec(self).map_err(|e| ReplayFileError::Serialize(e.to_string()))
    }

    /// Deserialize a replay log from JSON bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ReplayFileError> {
        serde_json::from_slice(bytes).map_err(|e| ReplayFileError::Deserialize(e.to_string()))
    }

    /// Verify this replay file is compatible with the given manifest hash
    /// (architecture §22.3). A zero header manifest hash skips the check for
    /// backward compatibility with pre-Phase-14 replays.
    pub fn verify_manifest(&self, expected: MatchManifestHash) -> Result<(), ReplayFileError> {
        if !self.header.manifest_hash.is_zero() && self.header.manifest_hash != expected {
            return Err(ReplayFileError::ManifestMismatch {
                found: self.header.manifest_hash,
                expected,
            });
        }
        Ok(())
    }
}

/// Replay file serialization errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ReplayFileError {
    /// Serialization failure.
    #[error("serialize error: {0}")]
    Serialize(String),
    /// Deserialization failure.
    #[error("deserialize error: {0}")]
    Deserialize(String),
    /// Replay manifest hash does not match the running engine's pinned hash.
    #[error("replay manifest mismatch: found {} expected {}", .found.0.iter().map(|b| format!("{b:02x}")).collect::<String>(), .expected.0.iter().map(|b| format!("{b:02x}")).collect::<String>())]
    ManifestMismatch {
        /// Found manifest hash.
        found: MatchManifestHash,
        /// Expected manifest hash.
        expected: MatchManifestHash,
    },
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

#[cfg(test)]
mod tests {
    use super::*;
    use engine_api::{CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};

    fn sample_cmd(seed: u64, n: u8) -> Command {
        Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: n,
                rng_seed: seed,
                ruleset: "bootstrap".into(),
            },
        }
    }

    #[test]
    fn hash_chain_is_self_consistent() {
        let mut log = ReplayLog::new([0u8; 32]);
        for i in 0..5u64 {
            log.push_record(
                sample_cmd(i + 1, 4),
                i + 1,
                i,
                [i as u8; 32],
                [(i as u8).wrapping_sub(1); 32],
                RuleRngCheckpoint::default(),
                RuleRngCheckpoint {
                    state: i,
                    call_count: i,
                },
                ResolutionOutcome::Committed,
                None,
            );
        }
        assert_eq!(log.records.len(), 5);
        log.verify_chain().expect("chain must verify");
        // Tamper with record 2 — must be detected.
        let saved = log.records[2].record_hash;
        log.records[2].record_hash = RecordHash([9u8; 32]);
        assert_eq!(log.verify_chain(), Err(2));
        log.records[2].record_hash = saved;
        // Tamper with a field — compute_hash changes.
        log.records[2].post_revision = 999;
        assert_eq!(log.verify_chain(), Err(2));
    }

    #[test]
    fn record_hash_depends_on_command() {
        let mut a = ReplayLog::new([0u8; 32]);
        let mut b = ReplayLog::new([0u8; 32]);
        a.push_record(
            sample_cmd(1, 4),
            1,
            0,
            [1u8; 32],
            [0u8; 32],
            RuleRngCheckpoint::default(),
            RuleRngCheckpoint::default(),
            ResolutionOutcome::Committed,
            None,
        );
        b.push_record(
            sample_cmd(2, 4),
            1,
            0,
            [1u8; 32],
            [0u8; 32],
            RuleRngCheckpoint::default(),
            RuleRngCheckpoint::default(),
            ResolutionOutcome::Committed,
            None,
        );
        assert_ne!(a.records[0].record_hash, b.records[0].record_hash);
    }

    #[test]
    fn manifest_hash_round_trips_through_header() {
        let h = ReplayHeader::with_manifest([0u8; 32], MatchManifestHash([7u8; 32]));
        assert_eq!(h.manifest_hash, MatchManifestHash([7u8; 32]));
    }
}
