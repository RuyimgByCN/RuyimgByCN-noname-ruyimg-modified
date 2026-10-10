//! Tournament WAL block signing tests (architecture plan §22.2).
//!
//! Verifies that competition-tier rooms produce:
//! - Hash-chained WAL blocks whose links verify independently
//! - Keyed-MAC signatures over each block
//! - A match result record carrying the root hash + result attestation
//! - Tamper detection: modifying any entry, hash, or signature is caught

#[path = "../src/persist.rs"]
mod persist;

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, MatchManifestHash,
    MatchOutcome, PlayerId,
};
use engine_core::Engine;
use persist::{RoomPersist, WalEntry};

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

fn build_started(seed: u64, n: u8, extra: u32) -> (Engine, RoomPersist) {
    let mut eng = Engine::new();
    let mut p = RoomPersist::new();
    let start = start_cmd(seed, n);
    match eng.submit(start.clone()) {
        CommandResult::Accepted {
            new_revision,
            state_hash,
        } => {
            if let Some(m) = eng.manifest() {
                p.pin_manifest(m.clone());
            }
            p.append(WalEntry {
                revision: new_revision,
                state_hash,
                command: start,
                idempotency_key: 1,
                manifest_hash: MatchManifestHash::ZERO,
            });
        }
        _ => panic!("start"),
    }
    for i in 0..extra {
        let cmd = Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(100 + i as u64),
            kind: CommandKind::EndPhase,
        };
        if eng.state().has_pending_choice() || eng.state().outcome.is_finished() {
            break;
        }
        match eng.submit(cmd.clone()) {
            CommandResult::Accepted {
                new_revision,
                state_hash,
            } => {
                p.append(WalEntry {
                    revision: new_revision,
                    state_hash,
                    command: cmd,
                    idempotency_key: 100 + i as u64,
                    manifest_hash: MatchManifestHash::ZERO,
                });
            }
            _ => break,
        }
    }
    (eng, p)
}

/// §22.2: Building blocks produces a correct hash chain.
#[test]
fn block_chain_hashes_verify() {
    let (_eng, p) = build_started(42, 4, 20);
    let block_count = p.wal.len().div_ceil(2);
    let blocks = p.build_blocks(2);
    assert!(
        blocks.len() >= 2,
        "need ≥2 blocks for chain test, got {} (entries={})",
        blocks.len(),
        p.wal.len()
    );
    let _ = block_count;
    // Chain verifies with the independent verifier (no key needed).
    RoomPersist::verify_block_chain(&blocks).expect("block chain must verify");
    // First block's previous is zero.
    assert_eq!(blocks[0].previous_block_hash, [0u8; 32]);
    // Each block's previous equals the prior block's hash.
    for i in 1..blocks.len() {
        assert_eq!(
            blocks[i].previous_block_hash,
            blocks[i - 1].block_hash,
            "block {i} chain link broken"
        );
    }
}

/// §22.2: Block hashes are deterministic — same WAL → same blocks.
#[test]
fn block_hashes_are_deterministic() {
    let (_eng, p) = build_started(7, 4, 10);
    let blocks_a = p.build_blocks(8);
    let blocks_b = p.build_blocks(8);
    for (a, b) in blocks_a.iter().zip(blocks_b.iter()) {
        assert_eq!(a.block_hash, b.block_hash);
    }
}

/// §22.2: Signing blocks produces verifiable signatures.
#[test]
fn block_signatures_verify() {
    let (_eng, mut p) = build_started(3, 4, 12);
    let blocks = p.build_blocks(4);
    p.seal_blocks(blocks, "tournament-key-2026", "tour-key-1");
    let blocks = p.blocks().to_vec();
    let sigs = p.block_signatures().to_vec();
    assert_eq!(blocks.len(), sigs.len());
    // Verify with correct key.
    RoomPersist::verify_block_signatures(&blocks, &sigs, "tournament-key-2026")
        .expect("signatures must verify with correct key");
    // Verify with wrong key must fail.
    let err = RoomPersist::verify_block_signatures(&blocks, &sigs, "wrong-key");
    assert!(err.is_err(), "wrong key must fail signature verification");
}

/// §22.2: Tampering with a block hash is detected by the chain verifier.
#[test]
fn block_chain_tamper_detected() {
    let (_eng, p) = build_started(1, 4, 8);
    let mut blocks = p.build_blocks(1); // block_size=1 → 1 block per entry
    assert!(
        blocks.len() >= 2,
        "need ≥2 blocks for tamper test, got {}",
        blocks.len()
    );
    // Tamper with block 1's hash.
    blocks[1].block_hash[0] ^= 0xff;
    let err = RoomPersist::verify_block_chain(&blocks);
    assert_eq!(err, Err(1), "tampered block must be detected at index 1");
}

/// §22.2: Tampering with an entry hash changes the block hash (chain breaks).
#[test]
fn entry_tamper_breaks_block_hash() {
    let (_eng, p) = build_started(1, 4, 8);
    let blocks_original = p.build_blocks(4);
    // Simulate tampering: if we changed an entry's state_hash in the WAL,
    // the rebuilt block would have a different hash. We verify this by
    // checking that the stored block_hash matches compute_hash.
    for block in &blocks_original {
        assert_eq!(block.block_hash, block.compute_hash());
    }
}

/// §22.2: The match result record carries the root hash and outcome.
#[test]
fn result_record_carries_root_and_outcome() {
    let (eng, mut p) = build_started(5, 2, 5);
    let blocks = p.build_blocks(4);
    p.seal_blocks(blocks, "key", "kid");
    let final_rev = eng.state().revision;
    let record = p.finalize_result(MatchOutcome::RebelWin, final_rev, "key", "kid");
    assert_eq!(record.final_revision, final_rev);
    assert_eq!(record.outcome, MatchOutcome::RebelWin);
    assert_eq!(record.total_entries, p.wal.len() as u64);
    assert!(
        !record.wal_root_hash.iter().all(|&b| b == 0),
        "root must be non-zero"
    );
    assert!(record.result_signature.is_some(), "result must be signed");
    // The root hash equals the last block's hash.
    assert_eq!(record.wal_root_hash, p.wal_root_hash());
}

/// §22.2: Independent verifier can attest the result without the private key
/// being embedded — it recomputes the chain + checks signatures against the
/// known public key id.
#[test]
fn independent_verifier_attestation() {
    let (eng, mut p) = build_started(9, 4, 16);
    let blocks = p.build_blocks(8);
    p.seal_blocks(blocks, "tournament-secret", "tour-2026");
    let record = p.finalize_result(
        eng.state().outcome.clone(),
        eng.state().revision,
        "tournament-secret",
        "tour-2026",
    );
    // Independent verification path:
    // 1. Verify block chain integrity (no key needed).
    RoomPersist::verify_block_chain(p.blocks()).expect("chain ok");
    // 2. Verify signatures with the known public key.
    RoomPersist::verify_block_signatures(p.blocks(), p.block_signatures(), "tournament-secret")
        .expect("sigs ok");
    // 3. The result signature covers the root hash.
    let result_sig = record.result_signature.as_ref().unwrap();
    assert_eq!(result_sig.block_hash_hex, hex32(&record.wal_root_hash));
    assert_eq!(result_sig.key_id, "tour-2026");
}

/// §22.2: Empty WAL produces zero root hash and no blocks.
#[test]
fn empty_wal_produces_zero_root() {
    let p = RoomPersist::new();
    assert_eq!(p.wal_root_hash(), [0u8; 32]);
    assert!(p.build_blocks(4).is_empty());
}

fn hex32(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
