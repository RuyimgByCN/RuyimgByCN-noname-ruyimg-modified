//! Phase 16: self-describing snapshots + replay files carrying manifest hash.
//!
//! Covers architecture plan §23 (snapshot must carry MatchManifestHash) and
//! §22.3 (replay file must carry MatchManifest + version axes). Verifies:
//! - `Snapshot::capture_with_manifest` stamps the hash and `restore_with_manifest`
//!   rejects snapshots from a different match.
//! - `Engine::export_replay` / `import_replay` round-trip a replay file whose
//!   header carries the manifest hash; importing a replay with a mismatched
//!   manifest hash is rejected.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, MatchManifestHash,
    PlayerId,
};
use engine_core::{Engine, ReplayFileError, Snapshot, SnapshotError};

fn start_cmd(seed: u64, n: u8) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: n,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

#[test]
fn engine_snapshot_carries_manifest_hash() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(42, 4));
    let pinned = eng.manifest_hash();
    assert!(!pinned.is_zero());

    let snap = eng.snapshot();
    assert_eq!(
        snap.manifest_hash, pinned,
        "snapshot must carry the engine's pinned manifest hash"
    );

    // Bytes round-trip preserves the manifest hash.
    let bytes = snap.to_bytes().unwrap();
    let snap2 = Snapshot::from_bytes(&bytes).unwrap();
    assert_eq!(snap2.manifest_hash, pinned);
}

#[test]
fn snapshot_restore_with_correct_manifest_succeeds() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(7, 2));
    let pinned = eng.manifest_hash();
    let snap = eng.snapshot();

    let mut other = Engine::new();
    other
        .restore_snapshot_with_manifest(&snap, eng.manifest().unwrap().clone())
        .unwrap();
    assert_eq!(other.manifest_hash(), pinned);
    assert_eq!(other.state_hash(), eng.state_hash());
}

#[test]
fn snapshot_restore_with_wrong_manifest_rejected() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(7, 2));
    let snap = eng.snapshot();
    assert!(!snap.manifest_hash.is_zero());

    // Directly use restore_with_manifest with a wrong hash.
    let wrong = MatchManifestHash([0xff; 32]);
    let err = snap.restore_with_manifest(wrong).unwrap_err();
    assert!(
        matches!(err, SnapshotError::ManifestMismatch { .. }),
        "expected ManifestMismatch, got {err:?}"
    );
}

#[test]
fn replay_file_round_trip_preserves_manifest() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(11, 4));
    eng.submit(Command {
        actor: PlayerId::new(1),
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::EndPhase,
    });

    let pinned = eng.manifest_hash();
    let bytes = eng.export_replay().unwrap();

    // Import with the correct manifest expectation.
    let imported = Engine::import_replay(&bytes, Some(pinned)).unwrap();
    assert_eq!(imported.state_hash(), eng.state_hash());
    assert_eq!(imported.manifest_hash(), pinned);
    // Chain still verifies on the imported engine's log.
    imported
        .replay_log()
        .verify_chain()
        .expect("imported chain must verify");
}

#[test]
fn replay_file_manifest_mismatch_rejected() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(11, 4));
    let bytes = eng.export_replay().unwrap();

    // The replay carries a non-zero manifest hash; supplying a different one
    // must be rejected before replay even begins.
    let wrong = MatchManifestHash([0xaa; 32]);
    let err = Engine::import_replay(&bytes, Some(wrong)).unwrap_err();
    assert!(
        matches!(err, ReplayFileError::ManifestMismatch { .. }),
        "expected ManifestMismatch, got {err:?}"
    );
}

#[test]
fn replay_file_without_manifest_expectation_imports() {
    // Passing None for expected manifest skips the check (backward-compat /
    // diagnostic import path).
    let mut eng = Engine::new();
    eng.submit(start_cmd(3, 2));
    let bytes = eng.export_replay().unwrap();
    let imported = Engine::import_replay(&bytes, None).unwrap();
    assert_eq!(imported.state_hash(), eng.state_hash());
}

#[test]
fn golden_state_hash_unchanged_by_phase16() {
    // Snapshot now carries manifest_hash, but that field is outside GameState,
    // so the canonical state hash of the golden start vector is unaffected.
    // The golden vector uses the "bootstrap" ruleset (matches conformance suite).
    let mut eng = Engine::new();
    let r = eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 42,
            ruleset: "bootstrap".into(),
        },
    });
    let h = match r {
        CommandResult::Accepted { state_hash, .. } => state_hash,
        _ => panic!("expected accept"),
    };
    let hex: String = h.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        hex, "86daeaffe7f71fb6facda6e6ebe358a87076f5d9e2e7be196deb111b698c31a7",
        "golden start (4,42) hash must be unchanged after Phase 16"
    );
}
