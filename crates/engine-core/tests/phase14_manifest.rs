//! Phase 14: MatchManifest freeze + hash-chained ResolutionRecord.
//!
//! Covers architecture plan §5 (MatchManifest), §22.1–22.2 (ResolutionRecord
//! + hash chain), §41 (live match version pinning).
//!
//! These tests assert the v2.0 review invariants that previously had no
//! representation in code:
//! - A match pins an immutable manifest at StartGame (frozen thereafter).
//! - The replay log header carries the manifest hash.
//! - Every accepted command emits a hash-chained ResolutionRecord whose links
//!   verify under `ReplayLog::verify_chain`.
//! - The manifest is independent of GameState, so state hashes (golden vectors)
//!   are unchanged.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, MatchManifestHash,
    PlayerId,
};
use engine_core::{Engine, ResolutionOutcome};

fn start_cmd(seed: u64, n: u8, ruleset: &str) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: n,
            rng_seed: seed,
            ruleset: ruleset.into(),
        },
    }
}

#[test]
fn manifest_is_pinned_at_startgame_and_frozen() {
    let mut eng = Engine::new();
    // Before start: no manifest.
    assert!(eng.manifest().is_none());
    assert_eq!(eng.manifest_hash(), MatchManifestHash::ZERO);
    assert!(eng.replay_log().manifest_hash().is_zero());

    eng.submit(start_cmd(42, 4, "identity-standard"));

    // After start: manifest pinned and non-zero.
    let m = eng.manifest().expect("manifest pinned at StartGame");
    let pinned_hash = eng.manifest_hash();
    assert!(!pinned_hash.is_zero(), "manifest hash must be non-zero");
    assert_eq!(
        eng.replay_log().manifest_hash(),
        pinned_hash,
        "replay header must carry the pinned manifest hash"
    );

    // Manifest content reflects the start command.
    assert_eq!(m.match_config.ruleset_name, "identity-standard");
    assert_eq!(m.match_config.player_count, 4);
    assert_eq!(m.rng_spec.initial_seed, 42);
    assert_eq!(m.initial_players.len(), 4);
    // Budgets are pinned from the resolved state.
    assert!(m.budget_profile.effect_budget > 0);
}

#[test]
fn same_start_command_pins_same_manifest_hash() {
    let mut a = Engine::new();
    let mut b = Engine::new();
    a.submit(start_cmd(7, 2, "identity-standard"));
    b.submit(start_cmd(7, 2, "identity-standard"));
    assert_eq!(
        a.manifest_hash(),
        b.manifest_hash(),
        "deterministic StartGame must pin identical manifest hash"
    );
}

#[test]
fn different_seed_pins_different_manifest_hash() {
    let mut a = Engine::new();
    let mut b = Engine::new();
    a.submit(start_cmd(1, 4, "identity-standard"));
    b.submit(start_cmd(2, 4, "identity-standard"));
    assert_ne!(
        a.manifest_hash(),
        b.manifest_hash(),
        "different RNG seed must change manifest hash"
    );
}

#[test]
fn every_accepted_command_produces_a_record() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(1, 2, "bootstrap"));
    eng.submit(Command {
        actor: PlayerId::new(1),
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::EndPhase,
    });
    eng.submit(Command {
        actor: PlayerId::new(1),
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::EndPhase,
    });

    let log = eng.replay_log();
    assert!(log.has_records(), "log must carry canonical records");
    assert_eq!(log.records.len(), 3);
    // Compact form stays in lockstep.
    assert_eq!(log.commands.len(), log.records.len());
    // Sequence is 1-based and monotonic.
    for (i, rec) in log.records.iter().enumerate() {
        assert_eq!(rec.record_sequence, (i + 1) as u64);
    }
}

#[test]
fn record_outcome_classified_correctly() {
    let mut eng = Engine::new();
    let _ = eng.submit(start_cmd(5, 2, "bootstrap"));
    // StartGame alone commits a plain segment (no choice barrier).
    let r = &eng.replay_log().records[0];
    assert_eq!(r.outcome, ResolutionOutcome::Committed);
    assert!(r.match_outcome.is_none());
    // Pre/post revision captured.
    assert_eq!(r.pre_revision, 0);
    assert_eq!(r.post_revision, 1);
    // RNG checkpoints present.
    assert_eq!(r.rng_before.call_count, 0);
    // bootstrap_match draws many cards, so call count grew.
    assert!(r.rng_after.call_count > 0);
}

#[test]
fn hash_chain_verifies_after_play() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(9, 4, "bootstrap"));
    for _ in 0..5 {
        eng.submit(Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        });
    }
    let log = eng.replay_log();
    log.verify_chain()
        .expect("hash chain must verify end to end");
}

#[test]
fn record_hash_links_previous() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(3, 2, "bootstrap"));
    eng.submit(Command {
        actor: PlayerId::new(1),
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::EndPhase,
    });
    let log = eng.replay_log();
    assert_eq!(log.records.len(), 2);
    // First record's previous is zero sentinel.
    assert_eq!(
        log.records[0].previous_record_hash,
        engine_core::RecordHash::ZERO
    );
    // Second record's previous equals first record's hash.
    assert_eq!(
        log.records[1].previous_record_hash,
        log.records[0].record_hash
    );
    // A record's stored hash matches its recomputed hash.
    for rec in &log.records {
        assert_eq!(rec.record_hash, rec.compute_hash());
    }
}

#[test]
fn replay_from_rebuilds_identical_chain_and_manifest() {
    let mut live = Engine::new();
    let cmds = vec![
        start_cmd(11, 4, "bootstrap"),
        Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        },
        Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        },
    ];
    for c in &cmds {
        assert!(matches!(
            live.submit(c.clone()),
            CommandResult::Accepted { .. }
        ));
    }
    let live_hash = live.state_hash();
    let live_manifest = live.manifest_hash();

    let replayed = Engine::replay_from(&cmds).unwrap();
    assert_eq!(replayed.state_hash(), live_hash);
    assert_eq!(replayed.manifest_hash(), live_manifest);
    // Rebuilt chain verifies.
    replayed
        .replay_log()
        .verify_chain()
        .expect("replayed chain must verify");
    assert_eq!(
        replayed.replay_log().records.len(),
        live.replay_log().records.len()
    );
    // Record hashes are identical (deterministic).
    for (a, b) in live
        .replay_log()
        .records
        .iter()
        .zip(replayed.replay_log().records.iter())
    {
        assert_eq!(a.record_hash, b.record_hash);
    }
}

#[test]
fn snapshot_restore_with_manifest_preserves_pinned_hash() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(21, 3, "bootstrap"));
    let pinned = eng.manifest().unwrap().clone();
    let pinned_hash = eng.manifest_hash();
    let snap = eng.snapshot();

    let mut restored = Engine::new();
    restored
        .restore_snapshot_with_manifest(&snap, pinned.clone())
        .unwrap();
    assert_eq!(restored.manifest_hash(), pinned_hash);
    assert_eq!(restored.state_hash(), eng.state_hash());
}

#[test]
fn manifest_independent_of_state_hash_keeps_golden_vectors() {
    // The manifest is not part of GameState, so adding it must not perturb
    // the existing golden start vector (4,42).
    let mut eng = Engine::new();
    let r = eng.submit(start_cmd(42, 4, "bootstrap"));
    let h = match r {
        CommandResult::Accepted { state_hash, .. } => state_hash,
        _ => panic!("expected accept"),
    };
    let hex: String = h.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        hex, "86daeaffe7f71fb6facda6e6ebe358a87076f5d9e2e7be196deb111b698c31a7",
        "golden start (4,42) hash must be unchanged after manifest wiring"
    );
}
