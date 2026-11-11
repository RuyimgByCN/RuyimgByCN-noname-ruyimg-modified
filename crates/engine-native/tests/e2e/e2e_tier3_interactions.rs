//! Tier 3 E2E Test Suite: Cross-Feature Runtime Interactions.
//!
//! Opaque-box integration tests verifying interactions across phase progression,
//! command validation, choice projections, and NGSA archive roundtrips.

use crate::e2e_harness::{hex_decode, OpaqueGameSession};
use engine_native::engine::{CommandOutcome, GameAction, GameCommand, RejectReason};
use engine_native::event::Phase;

/// Verifies that phase transitions preserve state hash and trajectory continuation across save/restore.
#[test]
fn test_t3_interaction_3_phase_advance_save_restore_roundtrip() {
    let mut session = OpaqueGameSession::new_identity_match(3003, 5).expect("match init");
    let initial_phase = session.current_phase();

    for _ in 0..5 {
        let _ = session.auto_step();
    }
    assert_ne!(session.current_phase(), initial_phase);

    let hex = session.save_snapshot();
    let mut restored = OpaqueGameSession::restore_snapshot(&hex).expect("restore snapshot");

    assert_eq!(restored.revision(), session.revision());
    assert_eq!(restored.state_hash(), session.state_hash());
    assert_eq!(restored.current_player(), session.current_player());
    assert_eq!(restored.current_phase(), session.current_phase());

    let outcome1 = session.auto_step().expect("step orig");
    let outcome2 = restored.auto_step().expect("step restored");

    match (outcome1, outcome2) {
        (
            CommandOutcome::Accepted { state_hash: h1 },
            CommandOutcome::Accepted { state_hash: h2 },
        ) => {
            assert_eq!(h1, h2, "Trajectories must remain bitwise identical");
        }
        (
            CommandOutcome::Pending { choice: c1 },
            CommandOutcome::Pending { choice: c2 },
        ) => {
            assert_eq!(c1.id, c2.id);
            assert_eq!(c1.observer, c2.observer);
        }
        (other1, other2) => {
            panic!("Outcomes diverged after restore: {other1:?} vs {other2:?}");
        }
    }
}

/// Verifies that invalid target choices or out-of-turn actions trigger strict rejection
/// with zero mutation to revision, state hash, or open choice.
#[test]
fn test_t3_interaction_4_interactive_target_counterexample() {
    let mut session = OpaqueGameSession::new_identity_match(3004, 5).expect("match init");
    let cur = session.current_player();

    // 1. Invalid out-of-bounds target seat 8 in 5-player game
    session.assert_action_rejected(
        cur,
        GameAction::PlayCard {
            card: 1,
            targets: vec![8],
        },
        None,
    );

    // 2. Out-of-turn actor target play
    let inactive = (cur + 1) % 5;
    session.assert_action_rejected(
        inactive,
        GameAction::PlayCard {
            card: 1,
            targets: vec![cur],
        },
        Some(RejectReason::NotYourTurn),
    );

    // 3. Stale revision on target play
    let valid_rev = session.revision();
    let dedup = session.next_dedup_id();
    let stale_cmd = GameCommand {
        actor: cur,
        expected_revision: valid_rev + 10,
        dedup_id: dedup,
        action: GameAction::PlayCard {
            card: 1,
            targets: vec![(cur + 1) % 5],
        },
    };
    session.assert_command_rejected(stale_cmd, Some(RejectReason::StaleRevision));
}

/// Verifies that multi-step game progression monotonically increments revision and evolves state hash.
#[test]
fn test_t3_interaction_5_multi_step_event_progression() {
    let mut session = OpaqueGameSession::new_identity_match(3005, 5).expect("match init");
    let start_rev = session.revision();
    let start_hash = session.state_hash();

    let mut last_rev = start_rev;
    for step_idx in 0..12 {
        let outcome = session.auto_step().expect("auto_step execution");
        assert!(
            matches!(
                outcome,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "Step {step_idx} failed: {outcome:?}"
        );
        let cur_rev = session.revision();
        assert!(
            cur_rev > last_rev,
            "Revision must monotonically increase on real event step"
        );
        last_rev = cur_rev;
    }

    assert_ne!(
        session.state_hash(),
        start_hash,
        "State hash must evolve under real event dispatch"
    );
    assert_eq!(session.players_count(), 5);
}

/// Verifies open choice JSON projection per seat observer.
#[test]
fn test_t3_interaction_6_open_choice_json_seat_projection() {
    let session = OpaqueGameSession::new_identity_match(3006, 5).expect("match init");

    let json_p0 = session.open_choice_json(0);
    let json_p1 = session.open_choice_json(1);

    assert!(json_p0 == "null" || json_p0.contains("id"));
    assert!(json_p1 == "null" || json_p1.contains("id"));
}

/// Verifies that across 10 deterministic seeds, counterexample rejection
/// maintains the zero-mutation invariant with 100% consistency.
#[test]
fn test_t3_interaction_8_counterexample_prng_sampling() {
    let base_seed: u64 = 0x5347_5300;
    for sample_id in 1..=10 {
        let seed = base_seed + sample_id;
        let mut session = OpaqueGameSession::new_identity_match(seed, 5).expect("match init");
        let initial_rev = session.revision();
        let initial_hash = session.state_hash();

        let dedup = session.next_dedup_id();
        let stale_cmd = GameCommand {
            actor: session.current_player(),
            expected_revision: initial_rev + 99,
            dedup_id: dedup,
            action: GameAction::NextPhase,
        };
        session.assert_command_rejected(stale_cmd, Some(RejectReason::StaleRevision));

        assert_eq!(
            session.revision(),
            initial_rev,
            "Revision mutated on seed {seed}"
        );
        assert_eq!(
            session.state_hash(),
            initial_hash,
            "State hash mutated on seed {seed}"
        );
    }
}

/// Verifies that in Play phase, playing an unowned card with empty targets is rejected cleanly.
#[test]
fn test_t3_interaction_15_play_phase_invalid_card_rejected() {
    let mut session = OpaqueGameSession::new_identity_match(3015, 5).expect("match init");
    let cur = session.current_player();

    while session.current_phase() != Phase::Play && !session.is_finished() {
        let _ = session.auto_step();
    }

    session.assert_action_rejected(
        cur,
        GameAction::PlayCard {
            card: 9999,
            targets: vec![],
        },
        None,
    );
}

/// Verifies that mid-game match states save and restore with identical continuation.
#[test]
fn test_t3_interaction_16_midgame_save_restore_continuation() {
    let mut session1 = OpaqueGameSession::new_identity_match(3016, 5).expect("match init");

    for _ in 0..8 {
        let _ = session1.auto_step();
    }

    let snap_hex = session1.save_snapshot();
    let mut session2 = OpaqueGameSession::restore_snapshot(&snap_hex).expect("restore");

    for _ in 0..5 {
        let _ = session1.auto_step();
        let _ = session2.auto_step();
    }

    assert_eq!(session1.revision(), session2.revision());
    assert_eq!(session1.state_hash(), session2.state_hash());
    assert_eq!(session1.current_player(), session2.current_player());
}

/// Verifies that binary archives begin with the NGSA magic header and decode cleanly.
#[test]
fn test_t3_interaction_17_ngsa_archive_magic_and_restore() {
    let session = OpaqueGameSession::new_identity_match(3017, 5).expect("match init");
    let hex = session.save_snapshot();

    let bytes = hex_decode(&hex).expect("decode hex snapshot");
    assert!(bytes.len() >= 4, "Archive buffer must be at least 4 bytes");
    assert_eq!(
        &bytes[0..4],
        b"NGSA",
        "Archive must start with NGSA magic bytes"
    );

    let restored = OpaqueGameSession::restore_snapshot(&hex).expect("restore from handover");
    assert_eq!(restored.revision(), session.revision());
    assert_eq!(restored.state_hash(), session.state_hash());
}
