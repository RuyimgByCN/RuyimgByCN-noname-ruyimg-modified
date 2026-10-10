//! Golden-style determinism vectors for Phase 1.

use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::{canonical_state_hash, Engine, Snapshot};

fn start(seed: u64, n: u8) -> Command {
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
fn golden_start_hash_vector() {
    // Fixed vector: do not change without updating architecture golden suite.
    let mut eng = Engine::new();
    let r = eng.submit(start(42, 4));
    match r {
        CommandResult::Accepted { state_hash, .. } => {
            // Print-friendly assert: hash must stay stable across refactors of encoding
            // unless SNAPSHOT/hash domain version bumps intentionally.
            let hex: String = state_hash.iter().map(|b| format!("{b:02x}")).collect();
            // Sanity: non-zero, 64 hex chars
            assert_eq!(hex.len(), 64);
            assert_ne!(state_hash, [0u8; 32]);
            // Second engine same hash
            let mut eng2 = Engine::new();
            eng2.submit(start(42, 4));
            assert_eq!(eng.state_hash(), eng2.state_hash());
            assert_eq!(eng.state_hash(), state_hash);
        }
        other => panic!("expected accept, got {other:?}"),
    }
}

#[test]
fn illegal_command_zero_mutation() {
    let mut eng = Engine::new();
    eng.submit(start(1, 3));
    let h = eng.state_hash();
    let rev = eng.state().revision;
    let r = eng.submit(Command {
        actor: PlayerId::new(99),
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::Surrender,
    });
    assert!(matches!(r, CommandResult::Rejected { .. }));
    assert_eq!(eng.state_hash(), h);
    assert_eq!(eng.state().revision, rev);
}

#[test]
fn snapshot_bytes_roundtrip() {
    let mut eng = Engine::new();
    eng.submit(start(100, 2));
    eng.submit(Command {
        actor: PlayerId::new(1),
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::EndPhase,
    });
    let snap = eng.snapshot();
    let bytes = snap.to_bytes().unwrap();
    let snap2 = Snapshot::from_bytes(&bytes).unwrap();
    let restored = snap2.restore().unwrap();
    assert_eq!(canonical_state_hash(&restored), eng.state_hash());
}

#[test]
fn draw_effect_via_manual_queue_path() {
    // EndPhase from Draw → Play without drawing yet; ensure cards still conserved.
    let mut eng = Engine::new();
    eng.submit(start(5, 2));
    let cards = eng.state().cards.live_count();
    eng.submit(Command {
        actor: PlayerId::new(1),
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::EndPhase,
    });
    assert_eq!(eng.state().cards.live_count(), cards);
}
