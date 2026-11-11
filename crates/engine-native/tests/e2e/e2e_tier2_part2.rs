//! Tier 2 E2E Test Suite (Part 2: Seat Count, Seed, Deduplication & Revision Boundaries).
//!
//! Boundary value analysis and fault injection tests over `OpaqueGameSession`.

use crate::e2e_harness::OpaqueGameSession;
use engine_native::engine::{CommandOutcome, GameAction, GameCommand, RejectReason};

// ============================================================================
// Event & Match Initialization Boundaries
// ============================================================================

#[test]
fn test_t2_f9_real_event_boundary_1() {
    let mut session = OpaqueGameSession::new_identity_match(5001, 5).expect("match init");
    let hash_before = session.state_hash();
    let rev_before = session.revision();
    let cmd = GameCommand {
        actor: session.current_player(),
        expected_revision: rev_before + 100,
        dedup_id: session.next_dedup_id(),
        action: GameAction::NextPhase,
    };
    session.assert_command_rejected(cmd, Some(RejectReason::StaleRevision));
    assert_eq!(session.state_hash(), hash_before);
    assert_eq!(session.revision(), rev_before);
}

#[test]
fn test_t2_f9_real_event_boundary_2() {
    let mut session = OpaqueGameSession::new_identity_match(5002, 5).expect("match init");
    let non_current = (session.current_player() + 2) % 5;
    session.assert_action_rejected(
        non_current,
        GameAction::NextPhase,
        Some(RejectReason::NotYourTurn),
    );
}

#[test]
fn test_t2_f9_real_event_boundary_3() {
    let mut session = OpaqueGameSession::new_identity_match(5003, 5).expect("match init");
    let p0 = session.current_player();
    let phase_0 = session.current_phase();
    let _ = session.next_phase(p0);
    let phase_1 = session.current_phase();
    assert_ne!(phase_0, phase_1);
}

#[test]
fn test_t2_f9_real_event_boundary_4() {
    assert!(OpaqueGameSession::new_identity_match(5004, 4).is_ok());
    assert!(OpaqueGameSession::new_identity_match(5004, 5).is_ok());
    assert!(OpaqueGameSession::new_identity_match(5004, 8).is_ok());
    assert!(OpaqueGameSession::new_identity_match(5004, 2).is_err());
    assert!(OpaqueGameSession::new_identity_match(5004, 3).is_err());
    assert!(OpaqueGameSession::new_identity_match(5004, 9).is_err());
}

#[test]
fn test_t2_f9_real_event_boundary_5() {
    let s0 = OpaqueGameSession::new_identity_match(0, 5).expect("seed 0");
    let s_max = OpaqueGameSession::new_identity_match(u64::MAX, 5).expect("seed u64::MAX");
    assert_ne!(
        s0.state_hash(),
        s_max.state_hash(),
        "Seed 0 and seed u64::MAX must produce distinct state hashes"
    );
}

// ============================================================================
// Counterexample & Command Guard Boundaries
// ============================================================================

#[test]
fn test_t2_f10_counterexample_boundary_1() {
    let mut session = OpaqueGameSession::new_identity_match(6001, 5).expect("match init");
    let cur = session.current_player();
    session.assert_action_rejected(
        cur,
        GameAction::PlayCard {
            card: 88888,
            targets: vec![1],
        },
        None,
    );
}

#[test]
fn test_t2_f10_counterexample_boundary_2() {
    let mut session = OpaqueGameSession::new_identity_match(6002, 5).expect("match init");
    for _ in 0..10 {
        if let Some(c) = session.open_choice() {
            if c.spec.min > 0 {
                let obs = c.observer;
                let cid = c.id;
                session.assert_action_rejected(
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![],
                    },
                    None,
                );
                break;
            }
        }
        let _ = session.auto_step();
    }
}

#[test]
fn test_t2_f10_counterexample_boundary_3() {
    let session = OpaqueGameSession::new_identity_match(6003, 5).expect("match init");
    let hp = session.player_hp(0).unwrap();
    let max_hp = session.player_max_hp(0).unwrap() as i32;
    assert_eq!(hp, max_hp, "Initial HP starts capped at max HP");
}

#[test]
fn test_t2_f10_counterexample_boundary_4() {
    let mut session = OpaqueGameSession::new_identity_match(6004, 5).expect("match init");
    let cur = session.current_player();
    let rev = session.revision();
    let dedup = session.next_dedup_id();
    let cmd = GameCommand {
        actor: cur,
        expected_revision: rev,
        dedup_id: dedup,
        action: GameAction::NextPhase,
    };
    let outcome = session.submit(cmd);
    assert!(matches!(outcome, CommandOutcome::Accepted { .. }));

    let dup_cmd = GameCommand {
        actor: cur,
        expected_revision: session.revision(),
        dedup_id: dedup,
        action: GameAction::NextPhase,
    };
    session.assert_command_rejected(dup_cmd, Some(RejectReason::DuplicateCommand));
}

#[test]
fn test_t2_f10_counterexample_boundary_5() {
    let mut session = OpaqueGameSession::new_identity_match(6005, 5).expect("match init");
    let _ = session.auto_step();
    let cur = session.current_player();
    let rev = session.revision();
    let dedup = session.next_dedup_id();

    if rev > 0 {
        let stale_past = GameCommand {
            actor: cur,
            expected_revision: rev - 1,
            dedup_id: dedup,
            action: GameAction::NextPhase,
        };
        session.assert_command_rejected(stale_past, Some(RejectReason::StaleRevision));
    }

    let stale_future = GameCommand {
        actor: cur,
        expected_revision: rev + 5,
        dedup_id: dedup + 1,
        action: GameAction::NextPhase,
    };
    session.assert_command_rejected(stale_future, Some(RejectReason::StaleRevision));
}

// ============================================================================
// Seat Query & Snapshot Recovery Boundaries
// ============================================================================

#[test]
fn test_t2_out_of_bounds_seat_queries_return_none() {
    let session = OpaqueGameSession::new_identity_match(13002, 5).expect("match init");
    assert_eq!(session.player_hp(99), None);
    assert_eq!(session.player_max_hp(99), None);
    assert_eq!(session.player_hand_count(99), None);
    assert_eq!(session.player_hp(5), None);
}

#[test]
fn test_t2_rejected_command_state_immutability() {
    let mut session = OpaqueGameSession::new_identity_match(21005, 5).expect("match init");
    let hash_before = session.state_hash();
    let rev_before = session.revision();
    let bad_cmd = GameCommand {
        actor: session.current_player(),
        expected_revision: rev_before + 999,
        dedup_id: session.next_dedup_id(),
        action: GameAction::NextPhase,
    };
    session.assert_command_rejected(bad_cmd, Some(RejectReason::StaleRevision));
    assert_eq!(session.state_hash(), hash_before);
    assert_eq!(session.revision(), rev_before);
}

#[test]
fn test_t2_midgame_snapshot_recovery() {
    let mut session = OpaqueGameSession::new_identity_match(22004, 5).expect("match init");
    let _ = session.auto_step();
    let hex = session.save_snapshot();
    let restored = OpaqueGameSession::restore_snapshot(&hex).expect("restore snapshot");
    assert_eq!(session.revision(), restored.revision());
    assert_eq!(session.state_hash(), restored.state_hash());
}

#[test]
fn test_t2_invalid_seat_counts_rejected() {
    let invalid_seats = [0, 1, 2, 3, 9, 10, 100];
    for &seats in &invalid_seats {
        assert!(
            OpaqueGameSession::new_identity_match(22005, seats).is_err(),
            "Seat count {seats} must return Err"
        );
    }
}
