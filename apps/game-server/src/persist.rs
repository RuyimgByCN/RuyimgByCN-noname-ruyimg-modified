//! Room persistence: WAL of accepted commands + periodic snapshots.
//!
//! Architecture §14.3 / §24 (simplified in-process for Phase 7). Phase 15 adds
//! MatchManifest pinning so recovered rooms verify the same frozen contract.
//! Phase 18 adds §22.2 tournament-tier WAL block signing: WAL entries are
//! grouped into hash-chained blocks, each block is signed with a keyed MAC,
//! and the final root hash + signature land in the match result record.

use engine_api::{Command, MatchManifest, MatchManifestHash, MatchOutcome};
use engine_core::Snapshot;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One WAL entry after successful commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    /// Revision after command.
    pub revision: u64,
    /// State hash after command.
    pub state_hash: [u8; 32],
    /// Accepted command.
    pub command: Command,
    /// Idempotency key.
    pub idempotency_key: u64,
    /// Match manifest hash pinned at StartGame (zero pre-start). Carried on
    /// every entry so the WAL is self-describing (architecture §5.3, §24).
    #[serde(default)]
    pub manifest_hash: MatchManifestHash,
}

/// Domain separator for WAL block hashing (architecture §22.2).
const WAL_BLOCK_DOMAIN: &[u8] = b"noname-next/wal-block/v1";

/// A hash-chained block of WAL entries (architecture §22.2).
///
/// In tournament / ranked rooms, WAL entries are grouped into blocks. Each
/// block's hash chains to the previous block, forming a tamper-evident root.
/// The block is then signed with a keyed MAC so an independent verifier can
/// prove integrity without replaying the full match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalBlock {
    /// Block sequence (1-based).
    pub block_sequence: u64,
    /// First WAL entry revision in this block (inclusive).
    pub from_revision: u64,
    /// Last WAL entry revision in this block (inclusive).
    pub to_revision: u64,
    /// Canonical hashes of each entry's post-state (in order).
    pub entry_hashes: Vec<[u8; 32]>,
    /// Hash of the previous block (zero for the first block).
    pub previous_block_hash: [u8; 32],
    /// This block's hash (BLAKE3 of all the above fields).
    pub block_hash: [u8; 32],
}

impl WalBlock {
    /// Compute the canonical hash of a block (excluding the `block_hash` field).
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(WAL_BLOCK_DOMAIN);
        h.update(&self.block_sequence.to_le_bytes());
        h.update(&self.from_revision.to_le_bytes());
        h.update(&self.to_revision.to_le_bytes());
        h.update(&(self.entry_hashes.len() as u32).to_le_bytes());
        for eh in &self.entry_hashes {
            h.update(eh);
        }
        h.update(&self.previous_block_hash);
        *h.finalize().as_bytes()
    }
}

/// A detached signature over a WAL block (architecture §22.2).
///
/// Uses keyed-BLAKE3 (same scheme as content-runtime package signing). The
/// signature proves the block was produced by a key holder, without revealing
/// the key. An independent verifier with the public key id can recompute.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalBlockSignature {
    /// Algorithm id.
    pub alg: String,
    /// Hex block_hash (content fingerprint).
    pub block_hash_hex: String,
    /// Hex keyed-BLAKE3 MAC over domain + block_hash.
    pub signature: String,
    /// Signing key id label.
    pub key_id: String,
}

/// Final match result record carrying the WAL root hash + outcome (§22.2).
///
/// Written at match completion. An independent verification service reads this
/// record + the signed blocks to attest the result without replaying state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResultRecord {
    /// Match manifest hash.
    pub manifest_hash: MatchManifestHash,
    /// Final outcome.
    pub outcome: MatchOutcome,
    /// Total entries in the WAL.
    pub total_entries: u64,
    /// Final revision.
    pub final_revision: u64,
    /// Root hash of the block chain (hash of last block, or zero if empty).
    pub wal_root_hash: [u8; 32],
    /// Signatures for each block (parallel to blocks).
    pub block_signatures: Vec<WalBlockSignature>,
    /// Final-result signature over the root hash (tournament attestation).
    pub result_signature: Option<WalBlockSignature>,
}

/// In-memory durable log for a room (can be flushed to disk later).
#[derive(Debug, Default, Clone)]
pub struct RoomPersist {
    /// Append-only WAL.
    pub wal: Vec<WalEntry>,
    /// Snapshots by revision.
    pub snapshots: BTreeMap<u64, Snapshot>,
    /// Latest durable revision.
    pub durable_revision: u64,
    /// Pinned match manifest (set at StartGame; verified on recovery).
    manifest: Option<MatchManifest>,
    /// Signed WAL blocks (tournament tier, architecture §22.2).
    blocks: Vec<WalBlock>,
    /// Signatures for each block (parallel to `blocks`).
    block_signatures: Vec<WalBlockSignature>,
    /// Final result record (set when match completes).
    result_record: Option<MatchResultRecord>,
}

/// Default block size (entries per block) for tournament tier.
pub const DEFAULT_BLOCK_SIZE: usize = 16;

impl RoomPersist {
    /// Empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pin the match manifest at StartGame (architecture §5). Idempotent.
    pub fn pin_manifest(&mut self, manifest: MatchManifest) {
        self.manifest = Some(manifest);
    }

    /// Pinned manifest hash (zero if no match started yet).
    pub fn manifest_hash(&self) -> MatchManifestHash {
        self.manifest
            .as_ref()
            .map(|m| m.manifest_hash())
            .unwrap_or_default()
    }

    /// Pinned manifest, if the match has started.
    pub fn manifest(&self) -> Option<&MatchManifest> {
        self.manifest.as_ref()
    }

    /// Append accepted command (must be called before publishing to clients).
    ///
    /// The entry's manifest hash is stamped from the pinned manifest so every
    /// WAL entry is self-describing.
    pub fn append(&mut self, mut entry: WalEntry) {
        // Stamp the pinned manifest hash onto the entry if not already set.
        if entry.manifest_hash.is_zero() {
            entry.manifest_hash = self.manifest_hash();
        }
        self.durable_revision = entry.revision;
        self.wal.push(entry);
    }

    /// Store snapshot.
    pub fn put_snapshot(&mut self, snap: Snapshot) {
        self.snapshots.insert(snap.revision, snap);
    }

    /// Latest snapshot at or before revision.
    #[allow(dead_code)]
    pub fn latest_snapshot_at(&self, rev: u64) -> Option<&Snapshot> {
        self.snapshots.range(..=rev).next_back().map(|(_, s)| s)
    }

    /// WAL entries after a revision (exclusive).
    pub fn wal_after(&self, rev: u64) -> impl Iterator<Item = &WalEntry> {
        self.wal.iter().filter(move |e| e.revision > rev)
    }

    /// Number of WAL entries (tests / export).
    #[allow(dead_code)]
    pub fn wal_len(&self) -> usize {
        self.wal.len()
    }

    // ---- §22.2 Tournament WAL block signing ----

    /// Build hash-chained blocks from the WAL, grouping `block_size` entries per block.
    ///
    /// Returns the blocks (unsigned). Use [`seal_blocks`] to sign them.
    pub fn build_blocks(&self, block_size: usize) -> Vec<WalBlock> {
        let block_size = block_size.max(1);
        let mut blocks = Vec::new();
        let mut prev_hash = [0u8; 32];
        for (seq, chunk) in self.wal.chunks(block_size).enumerate() {
            let entry_hashes: Vec<[u8; 32]> = chunk.iter().map(|e| e.state_hash).collect();
            let from_rev = chunk.first().map(|e| e.revision).unwrap_or(0);
            let to_rev = chunk.last().map(|e| e.revision).unwrap_or(0);
            let mut block = WalBlock {
                block_sequence: (seq + 1) as u64,
                from_revision: from_rev,
                to_revision: to_rev,
                entry_hashes,
                previous_block_hash: prev_hash,
                block_hash: [0u8; 32],
            };
            block.block_hash = block.compute_hash();
            prev_hash = block.block_hash;
            blocks.push(block);
        }
        blocks
    }

    /// Seal (sign) a set of blocks with a keyed MAC (architecture §22.2).
    ///
    /// Stores the blocks + signatures internally for later attestation.
    pub fn seal_blocks(&mut self, blocks: Vec<WalBlock>, key: &str, key_id: &str) {
        let key_arr = key_bytes(key);
        let mut sigs = Vec::with_capacity(blocks.len());
        for block in &blocks {
            let sig = sign_block_hash(&block.block_hash, &key_arr, key_id);
            sigs.push(sig);
        }
        self.blocks = blocks;
        self.block_signatures = sigs;
    }

    /// Compute the WAL root hash (hash of the last block, or zero if no blocks).
    pub fn wal_root_hash(&self) -> [u8; 32] {
        self.blocks
            .last()
            .map(|b| b.block_hash)
            .unwrap_or([0u8; 32])
    }

    /// Signed blocks (tournament tier).
    pub fn blocks(&self) -> &[WalBlock] {
        &self.blocks
    }

    /// Block signatures (parallel to `blocks`).
    pub fn block_signatures(&self) -> &[WalBlockSignature] {
        &self.block_signatures
    }

    /// Finalize the match result record (architecture §22.2).
    ///
    /// Called at match completion. Computes the root hash, signs it, and stores
    /// the result record for independent verification.
    pub fn finalize_result(
        &mut self,
        outcome: MatchOutcome,
        final_revision: u64,
        key: &str,
        key_id: &str,
    ) -> MatchResultRecord {
        let root = self.wal_root_hash();
        let result_sig = if key.is_empty() {
            None
        } else {
            Some(sign_block_hash(&root, &key_bytes(key), key_id))
        };
        let record = MatchResultRecord {
            manifest_hash: self.manifest_hash(),
            outcome,
            total_entries: self.wal.len() as u64,
            final_revision,
            wal_root_hash: root,
            block_signatures: self.block_signatures.clone(),
            result_signature: result_sig,
        };
        self.result_record = Some(record.clone());
        record
    }

    /// Final result record, if the match has been finalized.
    pub fn result_record(&self) -> Option<&MatchResultRecord> {
        self.result_record.as_ref()
    }

    /// Verify the block chain integrity (hash links + recomputed hashes).
    ///
    /// Returns `Ok(())` if the chain is consistent, or `Err(block_index)` at
    /// the first broken link. This is the independent-verifier path (§22.2):
    /// it only needs the blocks, not the private key.
    pub fn verify_block_chain(blocks: &[WalBlock]) -> Result<(), usize> {
        let mut prev = [0u8; 32];
        for (i, block) in blocks.iter().enumerate() {
            if block.previous_block_hash != prev {
                return Err(i);
            }
            if block.block_hash != block.compute_hash() {
                return Err(i);
            }
            prev = block.block_hash;
        }
        Ok(())
    }

    /// Verify all block signatures against a key (independent verifier path).
    pub fn verify_block_signatures(
        blocks: &[WalBlock],
        sigs: &[WalBlockSignature],
        key: &str,
    ) -> Result<(), usize> {
        if blocks.len() != sigs.len() {
            return Err(0);
        }
        let key_arr = key_bytes(key);
        for (i, (block, sig)) in blocks.iter().zip(sigs.iter()).enumerate() {
            if sig.block_hash_hex != hex32(&block.block_hash) {
                return Err(i);
            }
            let expected = sign_block_hash(&block.block_hash, &key_arr, &sig.key_id);
            if &expected != sig {
                return Err(i);
            }
        }
        Ok(())
    }
}

/// Sign a block hash with keyed-BLAKE3.
fn sign_block_hash(block_hash: &[u8; 32], key_arr: &[u8; 32], key_id: &str) -> WalBlockSignature {
    let mut hasher = blake3::Hasher::new_keyed(key_arr);
    hasher.update(WAL_BLOCK_DOMAIN);
    hasher.update(b"block-sig");
    hasher.update(block_hash);
    WalBlockSignature {
        alg: "blake3-keyed-v1".into(),
        block_hash_hex: hex32(block_hash),
        signature: hex32(hasher.finalize().as_bytes()),
        key_id: key_id.into(),
    }
}

/// Derive a 32-byte key from a string passphrase.
fn key_bytes(key: &str) -> [u8; 32] {
    *blake3::hash(key.as_bytes()).as_bytes()
}

fn hex32(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Rebuild engine from snapshot + WAL tail, preserving the pinned manifest.
///
/// Recovery contract (architecture §24, §31.5):
/// 1. If a manifest is pinned and a snapshot exists, restore *with* the
///    manifest so the recovered engine keeps the same `manifest_hash`.
/// 2. Replay the WAL tail, asserting each entry's manifest hash matches the
///    pinned one — a mismatch means the WAL belongs to a different match and
///    must be rejected.
/// 3. The recovered engine's `manifest_hash()` must equal the pinned one.
pub fn recover_engine(persist: &RoomPersist) -> Result<engine_core::Engine, String> {
    recover_engine_from(persist, engine_core::Engine::new())
}

/// Recover using a preloaded content-locked engine when no snapshot exists yet.
pub fn recover_engine_from(
    persist: &RoomPersist,
    initial: engine_core::Engine,
) -> Result<engine_core::Engine, String> {
    use engine_api::CommandResult;
    use engine_core::Engine;

    let pinned_hash = persist.manifest_hash();
    let pinned_manifest = persist.manifest().cloned();

    let mut eng = if let Some(snap) = persist.snapshots.values().next_back() {
        if let Some(ref manifest) = pinned_manifest {
            // Restore with manifest pinned (reconnect path — architecture §5).
            let mut e = Engine::new();
            e.restore_snapshot_with_manifest(snap, manifest.clone())
                .map_err(|e| format!("snapshot+manifest restore: {e:?}"))?;
            e
        } else {
            let mut e = Engine::new();
            e.restore_snapshot(snap).map_err(|e| e.to_string())?;
            e
        }
    } else {
        initial
    };

    let base_rev = eng.state().revision;
    for entry in persist.wal_after(base_rev) {
        // Verify the WAL entry belongs to the same match (architecture §5.3).
        if !pinned_hash.is_zero() && entry.manifest_hash != pinned_hash {
            return Err(format!(
                "wal manifest mismatch at rev {}: entry={} pinned={}",
                entry.revision,
                hex32(&entry.manifest_hash.0),
                hex32(&pinned_hash.0),
            ));
        }
        match eng.submit(entry.command.clone()) {
            CommandResult::Accepted {
                new_revision,
                state_hash,
            } => {
                if new_revision != entry.revision || state_hash != entry.state_hash {
                    return Err(format!(
                        "wal mismatch at rev {}: expected {}/{:x?}, got {}/{:x?}",
                        entry.revision,
                        entry.revision,
                        &entry.state_hash[..4],
                        new_revision,
                        &state_hash[..4]
                    ));
                }
            }
            CommandResult::Rejected { reason } => {
                return Err(format!(
                    "wal replay rejected at rev {}: {reason:?}",
                    entry.revision
                ));
            }
        }
    }

    // Final invariant: recovered manifest hash matches pinned hash (§5.3).
    if !pinned_hash.is_zero() && eng.manifest_hash() != pinned_hash {
        return Err(format!(
            "recovered manifest hash {} != pinned {}",
            hex32(&eng.manifest_hash().0),
            hex32(&pinned_hash.0),
        ));
    }

    Ok(eng)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_api::{
        Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    };
    use engine_core::Engine;

    fn start_cmd(seed: u64, n: u8) -> Command {
        Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::StartGame {
                player_count: n,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }
    }

    #[test]
    fn wal_and_snapshot_recover_preserves_manifest() {
        let mut eng = Engine::new();
        let mut persist = RoomPersist::new();
        let start = start_cmd(9, 2);
        match eng.submit(start.clone()) {
            CommandResult::Accepted {
                new_revision,
                state_hash,
            } => {
                // Pin the manifest from the live engine.
                persist.pin_manifest(eng.manifest().unwrap().clone());
                persist.append(WalEntry {
                    revision: new_revision,
                    state_hash,
                    command: start,
                    idempotency_key: 1,
                    manifest_hash: MatchManifestHash::ZERO,
                });
                persist.put_snapshot(eng.snapshot());
            }
            _ => panic!("start"),
        }
        let end = Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::EndPhase,
        };
        match eng.submit(end.clone()) {
            CommandResult::Accepted {
                new_revision,
                state_hash,
            } => {
                persist.append(WalEntry {
                    revision: new_revision,
                    state_hash,
                    command: end,
                    idempotency_key: 2,
                    manifest_hash: MatchManifestHash::ZERO,
                });
            }
            other => panic!("{other:?}"),
        }
        let live_hash = eng.state_hash();
        let live_manifest = eng.manifest_hash();
        let recovered = recover_engine(&persist).unwrap();
        assert_eq!(recovered.state_hash(), live_hash);
        assert_eq!(recovered.state().revision, eng.state().revision);
        assert_eq!(
            recovered.manifest_hash(),
            live_manifest,
            "recovered engine must keep the pinned manifest hash"
        );
        // Every WAL entry carries the manifest hash.
        for e in &persist.wal {
            assert_eq!(e.manifest_hash, live_manifest);
        }
    }

    #[test]
    fn wal_manifest_mismatch_is_rejected() {
        let mut eng = Engine::new();
        let mut persist = RoomPersist::new();
        let start = start_cmd(9, 2);
        match eng.submit(start.clone()) {
            CommandResult::Accepted {
                new_revision,
                state_hash,
            } => {
                persist.pin_manifest(eng.manifest().unwrap().clone());
                persist.append(WalEntry {
                    revision: new_revision,
                    state_hash,
                    command: start,
                    idempotency_key: 1,
                    manifest_hash: MatchManifestHash::ZERO,
                });
            }
            _ => panic!("start"),
        }
        // Tamper: inject a WAL entry claiming a different manifest hash.
        persist.wal.push(WalEntry {
            revision: 999,
            state_hash: [0u8; 32],
            command: Command {
                actor: PlayerId::new(1),
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(2),
                kind: CommandKind::EndPhase,
            },
            idempotency_key: 2,
            manifest_hash: MatchManifestHash([0xff; 32]),
        });
        let err = recover_engine(&persist).unwrap_err();
        assert!(
            err.contains("manifest mismatch"),
            "expected manifest mismatch, got: {err}"
        );
    }
}
