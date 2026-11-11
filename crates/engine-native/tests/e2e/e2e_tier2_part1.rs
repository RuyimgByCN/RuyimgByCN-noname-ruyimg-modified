//! Tier 2 E2E Test Suite (Part 1: Timing, Choice, Skill & Archive Restore Boundaries).
//!
//! Boundary value analysis and fault injection tests over `OpaqueGameSession`.

use crate::e2e_harness::{hex_decode, hex_encode, OpaqueGameSession};
use engine_native::engine::{CommandOutcome, GameAction, GameCommand, RejectReason};

// ============================================================================
// Turn & Phase Timing Boundaries
// ============================================================================

#[test]
fn test_t2_f5_timing_boundary_1() {
    let session = OpaqueGameSession::new_identity_match(1001, 5).expect("match init");
    assert_eq!(session.player_hp(0), Some(5));
    assert_eq!(session.player_hp(1), Some(4));
    assert!(session.player_hp(0).unwrap() > 0);
}

#[test]
fn test_t2_f5_timing_boundary_2() {
    let mut session = OpaqueGameSession::new_identity_match(1002, 5).expect("match init");
    let current = session.current_player();
    let other = (current + 1) % 5;
    session.assert_action_rejected(
        other,
        GameAction::NextPhase,
        Some(RejectReason::NotYourTurn),
    );
}

#[test]
fn test_t2_f5_timing_boundary_3() {
    let mut session = OpaqueGameSession::new_identity_match(1003, 5).expect("match init");
    let initial_phase = session.current_phase();
    let initial_actor = session.current_player();
    let outcome = session.next_phase(initial_actor);
    assert!(
        matches!(outcome, CommandOutcome::Accepted { .. }),
        "NextPhase by active player must succeed"
    );
    assert_ne!(session.current_phase(), initial_phase);
}

#[test]
fn test_t2_f5_timing_boundary_4() {
    let mut session = OpaqueGameSession::new_identity_match(1004, 5).expect("match init");
    let mut prev_rev = session.revision();
    let mut prev_hash = session.state_hash();
    for _ in 0..6 {
        let _ = session.auto_step();
        assert!(
            session.revision() > prev_rev,
            "Revision must monotonically increase"
        );
        assert_ne!(
            session.state_hash(),
            prev_hash,
            "State hash must evolve on phase advance"
        );
        prev_rev = session.revision();
        prev_hash = session.state_hash();
    }
}

#[test]
fn test_t2_f5_timing_boundary_5() {
    let session = OpaqueGameSession::new_identity_match(1005, 5).expect("match init");
    assert_eq!(session.current_player(), 0, "Lord (seat 0) takes first turn");
    assert_eq!(session.revision(), 0);
}

// ============================================================================
// Target & Choice Validation Boundaries
// ============================================================================

#[test]
fn test_t2_f6_choice_boundary_1() {
    let mut session = OpaqueGameSession::new_identity_match(2001, 5).expect("match init");
    let cur = session.current_player();
    let stale_choice_cmd = GameCommand {
        actor: cur,
        expected_revision: session.revision(),
        dedup_id: session.next_dedup_id(),
        action: GameAction::Answer {
            choice_id: 999999,
            selection: vec![],
        },
    };
    session.assert_command_rejected(stale_choice_cmd, None);
}

#[test]
fn test_t2_f6_choice_boundary_2() {
    let mut session = OpaqueGameSession::new_identity_match(2002, 5).expect("match init");
    let cur = session.current_player();
    session.assert_action_rejected(
        cur,
        GameAction::PlayCard {
            card: 1,
            targets: vec![5],
        },
        None,
    );
    session.assert_action_rejected(
        cur,
        GameAction::PlayCard {
            card: 1,
            targets: vec![255],
        },
        None,
    );
}

#[test]
fn test_t2_f6_choice_boundary_3() {
    let mut session = OpaqueGameSession::new_identity_match(2003, 5).expect("match init");
    let cur = session.current_player();
    session.assert_action_rejected(
        cur,
        GameAction::PlayCard {
            card: 1,
            targets: vec![],
        },
        None,
    );
}

#[test]
fn test_t2_f6_choice_boundary_4() {
    let mut session = OpaqueGameSession::new_identity_match(2004, 5).expect("match init");
    let unknown_actor_cmd = GameCommand {
        actor: 99,
        expected_revision: session.revision(),
        dedup_id: session.next_dedup_id(),
        action: GameAction::NextPhase,
    };
    session.assert_command_rejected(unknown_actor_cmd, Some(RejectReason::UnknownActor));
}

#[test]
fn test_t2_f6_choice_boundary_5() {
    let mut session = OpaqueGameSession::new_identity_match(2005, 5).expect("match init");
    for _ in 0..10 {
        if let Some(c) = session.open_choice() {
            let cid = c.id;
            let bad_answer = session.answer_choice_indices(cid, &[9999]);
            assert!(bad_answer.is_err(), "Out of bounds option index must return Err");
            break;
        }
        let _ = session.auto_step();
    }
}

// ============================================================================
// Skill Activation Boundaries
// ============================================================================

#[test]
fn test_t2_f7_skill_boundary_1() {
    let mut session = OpaqueGameSession::new_identity_match(3001, 5).expect("match init");
    let cur = session.current_player();
    session.assert_action_rejected(
        cur,
        GameAction::ActivateSkill { skill: 9999 },
        None,
    );
}

#[test]
fn test_t2_f7_skill_boundary_2() {
    let mut session = OpaqueGameSession::new_identity_match(3002, 5).expect("match init");
    let cur = session.current_player();
    session.assert_action_rejected(
        cur,
        GameAction::ActivateSkill { skill: 0 },
        None,
    );
}

#[test]
fn test_t2_f7_skill_boundary_3() {
    let mut session = OpaqueGameSession::new_identity_match(3003, 5).expect("match init");
    let cur = session.current_player();
    let inactive = (cur + 1) % 5;
    session.assert_action_rejected(
        inactive,
        GameAction::ActivateSkill { skill: 1 },
        Some(RejectReason::NotYourTurn),
    );
}

#[test]
fn test_t2_f7_skill_boundary_4() {
    let mut session = OpaqueGameSession::new_identity_match(3004, 5).expect("match init");
    let cur = session.current_player();
    session.assert_action_rejected(
        cur,
        GameAction::ActivateSkill { skill: u16::MAX },
        None,
    );
}

#[test]
fn test_t2_f7_skill_boundary_5() {
    let mut session = OpaqueGameSession::new_identity_match(3005, 5).expect("match init");
    let rev_before = session.revision();
    let hash_before = session.state_hash();
    let _ = session.activate_skill(session.current_player(), 54321);
    assert_eq!(session.revision(), rev_before);
    assert_eq!(session.state_hash(), hash_before);
}

// ============================================================================
// Archive Decode & Snapshot Restore Boundaries
// ============================================================================

#[test]
fn test_t2_f8_archive_restore_boundary_1() {
    let session = OpaqueGameSession::new_identity_match(4001, 5).expect("match init");
    let hex = session.save_snapshot();
    let truncated = &hex[..hex.len() / 2];
    let res = OpaqueGameSession::restore_snapshot(truncated);
    assert!(res.is_err(), "Truncated hex archive must fail to restore");
}

#[test]
fn test_t2_f8_archive_restore_boundary_2() {
    assert!(hex_decode("abc").is_err(), "Odd-length hex must fail");
    assert!(hex_decode("zzzz").is_err(), "Non-hex string must fail");
    assert!(hex_decode("123g").is_err(), "Non-hex characters must fail");
    let res = OpaqueGameSession::restore_snapshot("deadbeefcafebabe");
    assert!(res.is_err(), "Garbage snapshot data must fail to restore");
}

#[test]
fn test_t2_f8_archive_restore_boundary_3() {
    let session = OpaqueGameSession::new_identity_match(4003, 5).expect("match init");
    let hex = session.save_snapshot();
    let mut raw = hex_decode(&hex).expect("decode valid hex");
    if raw.len() >= 4 {
        raw[0] ^= 0xFF;
        raw[1] ^= 0xFF;
    }
    let corrupted_hex = hex_encode(&raw);
    let res = OpaqueGameSession::restore_snapshot(&corrupted_hex);
    assert!(res.is_err(), "Corrupted magic header must fail to restore");
}

#[test]
fn test_t2_f8_archive_restore_boundary_4() {
    let mut session = OpaqueGameSession::new_identity_match(4004, 5).expect("match init");
    for _ in 0..12 {
        if session.open_choice().is_some() {
            let hex = session.save_snapshot();
            let restored = OpaqueGameSession::restore_snapshot(&hex).expect("restore choice");
            assert_eq!(
                session.open_choice().map(|c| c.id),
                restored.open_choice().map(|c| c.id)
            );
            assert_eq!(
                session.open_choice().map(|c| c.observer),
                restored.open_choice().map(|c| c.observer)
            );
            break;
        }
        let _ = session.auto_step();
    }
}

#[test]
fn test_t2_f8_archive_restore_boundary_5() {
    let mut session = OpaqueGameSession::new_identity_match(4005, 5).expect("match init");
    for _ in 0..5 {
        let _ = session.auto_step();
    }
    let hex1 = session.save_snapshot();
    let restored1 = OpaqueGameSession::restore_snapshot(&hex1).expect("restore 1");
    assert_eq!(restored1.revision(), session.revision());
    assert_eq!(restored1.state_hash(), session.state_hash());
    assert_eq!(restored1.state_hash_hex(), session.state_hash_hex());

    let hex2 = restored1.save_snapshot();
    let restored2 = OpaqueGameSession::restore_snapshot(&hex2).expect("restore 2");
    assert_eq!(restored2.revision(), session.revision());
    assert_eq!(restored2.state_hash(), session.state_hash());
    assert_eq!(restored2.state_hash_hex(), session.state_hash_hex());
}
