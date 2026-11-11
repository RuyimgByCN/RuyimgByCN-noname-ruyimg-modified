//! Tier 1 E2E Test Suite (Part 2: Match Initialization, Command Rejections & Determinism).
//!
//! Opaque-box integration tests over `OpaqueGameSession` verifying initial HP/hand setup,
//! command rejection guards, and deterministic replay invariants.

use crate::e2e_harness::OpaqueGameSession;
use engine_native::engine::{CommandOutcome, GameAction, GameCommand, RejectReason};
use engine_native::event::Phase;

// ============================================================================
// Match Initialization & Step Resolution
// ============================================================================

#[test]
fn test_t1_f9_initial_hp_distribution_1() {
    let session = OpaqueGameSession::new_identity_match(6001, 5).expect("match init");
    assert_eq!(session.players_count(), 5);
    assert_eq!(session.player_hp(0), Some(5), "Lord starts with 5 HP");
    for seat in 1..5 {
        assert_eq!(
            session.player_hp(seat),
            Some(4),
            "Non-lord starts with 4 HP"
        );
    }
}

#[test]
fn test_t1_f9_initial_hand_card_count_2() {
    let session = OpaqueGameSession::new_identity_match(6002, 5).expect("match init");
    for seat in 0..5 {
        assert_eq!(
            session.player_hand_count(seat),
            Some(4),
            "Each seat dealt 4 initial cards"
        );
    }
}

#[test]
fn test_t1_f9_auto_step_advances_state_3() {
    let mut session = OpaqueGameSession::new_identity_match(6003, 5).expect("match init");
    let rev_before = session.revision();
    let hash_before = session.state_hash();
    let outcome = session.auto_step().expect("step succeeds");
    assert!(
        matches!(
            outcome,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "auto_step produced valid outcome: {outcome:?}"
    );
    assert!(session.revision() > rev_before);
    assert_ne!(session.state_hash(), hash_before);
}

#[test]
fn test_t1_f9_active_player_has_legal_actions_4() {
    let session = OpaqueGameSession::new_identity_match(6004, 5).expect("match init");
    let actions = session.legal_actions(session.current_player());
    assert!(
        !actions.is_empty(),
        "Active player must have at least one legal action"
    );
}

#[test]
fn test_t1_f9_bounded_simulation_steps_5() {
    let mut session = OpaqueGameSession::new_identity_match(6005, 5).expect("match init");
    let steps = session.run_until_finished_or_limit(15);
    assert!(steps > 0, "Simulation stepped at least once");
}

// ============================================================================
// Command Rejection & Zero-Mutation Assertions
// ============================================================================

#[test]
fn test_t1_f10_unowned_card_play_rejected_1() {
    let mut session = OpaqueGameSession::new_identity_match(7001, 5).expect("match init");
    let cur = session.current_player();
    session.assert_action_rejected(
        cur,
        GameAction::PlayCard {
            card: 999999,
            targets: vec![],
        },
        None,
    );
}

#[test]
fn test_t1_f10_out_of_turn_card_play_rejected_2() {
    let mut session = OpaqueGameSession::new_identity_match(7002, 5).expect("match init");
    let inactive = (session.current_player() + 2) % 5;
    session.assert_action_rejected(
        inactive,
        GameAction::PlayCard {
            card: 1,
            targets: vec![0],
        },
        Some(RejectReason::NotYourTurn),
    );
}

#[test]
fn test_t1_f10_stale_revision_rejected_3() {
    let mut session = OpaqueGameSession::new_identity_match(7003, 5).expect("match init");
    let valid_rev = session.revision();
    let dedup = session.next_dedup_id();
    let stale_cmd = GameCommand {
        actor: session.current_player(),
        expected_revision: valid_rev + 1,
        dedup_id: dedup,
        action: GameAction::NextPhase,
    };
    session.assert_command_rejected(stale_cmd, Some(RejectReason::StaleRevision));
}

#[test]
fn test_t1_f10_duplicate_dedup_id_rejected_4() {
    let mut session = OpaqueGameSession::new_identity_match(7004, 5).expect("match init");
    let valid_rev = session.revision();
    let dup_cmd = GameCommand {
        actor: session.current_player(),
        expected_revision: valid_rev,
        dedup_id: 0,
        action: GameAction::NextPhase,
    };
    session.assert_command_rejected(dup_cmd, Some(RejectReason::DuplicateCommand));
}

#[test]
fn test_t1_f10_unknown_actor_rejected_5() {
    let mut session = OpaqueGameSession::new_identity_match(7005, 5).expect("match init");
    let valid_rev = session.revision();
    let dedup = session.next_dedup_id();
    let unknown_actor = GameCommand {
        actor: 250,
        expected_revision: valid_rev,
        dedup_id: dedup,
        action: GameAction::NextPhase,
    };
    session.assert_command_rejected(unknown_actor, Some(RejectReason::UnknownActor));
}

// ============================================================================
// Session Invariants & Deterministic Replay
// ============================================================================

#[test]
fn test_t1_session_initial_state_invariants() {
    let session = OpaqueGameSession::new_identity_match(8003, 5).expect("match init");
    assert_eq!(session.revision(), 0);
    assert_eq!(session.current_player(), 0);
    assert_eq!(session.players_count(), 5);
    assert_eq!(session.current_phase(), Phase::Judge);
}

#[test]
fn test_t1_repeated_invalid_skill_preserves_hash() {
    let mut session = OpaqueGameSession::new_identity_match(8004, 5).expect("match init");
    let hash_orig = session.state_hash();
    for _ in 0..5 {
        session.assert_action_rejected(
            session.current_player(),
            GameAction::ActivateSkill { skill: 9999 },
            None,
        );
    }
    assert_eq!(session.state_hash(), hash_orig);
}

#[test]
fn test_t1_monotonic_revision_increment() {
    let mut session = OpaqueGameSession::new_identity_match(9004, 5).expect("match init");
    let r0 = session.revision();
    let _ = session.auto_step();
    let r1 = session.revision();
    assert_eq!(
        r1,
        r0 + 1,
        "Single-writer engine guarantees monotonic revision increment"
    );
}

#[test]
fn test_t1_deterministic_multiple_snapshot_restores() {
    let session = OpaqueGameSession::new_identity_match(9005, 5).expect("match init");
    let hex = session.save_snapshot();
    let res1 = OpaqueGameSession::restore_snapshot(&hex).expect("restore 1");
    let res2 = OpaqueGameSession::restore_snapshot(&hex).expect("restore 2");
    assert_eq!(
        res1.state_hash(),
        res2.state_hash(),
        "Deterministic replay invariant must hold across multiple restores"
    );
}

#[test]
fn test_t1_snapshot_hex_encoding_format() {
    let session = OpaqueGameSession::new_identity_match(11004, 5).expect("match init");
    let hex = session.save_snapshot();
    assert!(!hex.is_empty());
    assert_eq!(hex.len() % 2, 0);
}

#[test]
fn test_t1_invalid_seat_count_rejected() {
    let bad_match = OpaqueGameSession::new_identity_match(11005, 2);
    assert!(
        bad_match.is_err(),
        "Starting match with invalid seat count (2) must return error cleanly"
    );
}

#[test]
fn test_t1_run_until_step_limit() {
    let mut session = OpaqueGameSession::new_identity_match(12004, 5).expect("match init");
    let steps = session.run_until_finished_or_limit(5);
    assert_eq!(steps, 5);
}
