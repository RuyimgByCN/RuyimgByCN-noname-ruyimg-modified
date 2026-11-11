//! Tier 1 E2E Test Suite (Part 1: Session Phases, Choices, Skill Rejections & Snapshot Roundtrip).
//!
//! Opaque-box integration tests over `OpaqueGameSession` verifying phase progression,
//! action validation, choice projection, and snapshot save/restore roundtrips.

use crate::e2e_harness::OpaqueGameSession;
use engine_native::engine::{CommandOutcome, GameAction, GameCommand, RejectReason};
use engine_native::event::Phase;

// ============================================================================
// Phase Progression & Turn Order Invariants
// ============================================================================

#[test]
fn test_t1_f5_phase_progression_1() {
    let mut session = OpaqueGameSession::new_identity_match(2001, 5).expect("match init");
    let initial_phase = session.current_phase();
    assert_eq!(session.current_player(), 0);

    let outcome = session.next_phase(0);
    assert!(
        matches!(outcome, CommandOutcome::Accepted { .. }),
        "Advancing phase must succeed"
    );
    let next_phase = session.current_phase();
    assert_ne!(
        initial_phase, next_phase,
        "Phase must advance to discrete new phase"
    );
}

#[test]
fn test_t1_f5_phase_enum_distinct_2() {
    assert_ne!(Phase::Prepare, Phase::Judge);
    assert_ne!(Phase::Judge, Phase::Draw);
    assert_ne!(Phase::Draw, Phase::Play);
    assert_ne!(Phase::Play, Phase::Discard);
    assert_ne!(Phase::Discard, Phase::End);
}

#[test]
fn test_t1_f5_inactive_player_next_phase_rejected_3() {
    let mut session = OpaqueGameSession::new_identity_match(2003, 5).expect("match init");
    let active = session.current_player();
    let inactive = (active + 1) % 5;

    session.assert_action_rejected(
        inactive,
        GameAction::NextPhase,
        Some(RejectReason::NotYourTurn),
    );
}

#[test]
fn test_t1_f5_turn_rotation_to_next_seat_4() {
    let mut session = OpaqueGameSession::new_identity_match(2004, 5).expect("match init");
    let p0 = session.current_player();
    for _ in 0..40 {
        if session.current_player() != p0 {
            break;
        }
        let _ = session.auto_step();
    }
    assert_ne!(
        session.current_player(),
        p0,
        "After stepping through turn phases, turn advances to next player"
    );
}

#[test]
fn test_t1_f5_stale_revision_next_phase_rejected_5() {
    let mut session = OpaqueGameSession::new_identity_match(2005, 5).expect("match init");
    let rev = session.revision();
    let hash = session.state_hash();

    let stale_cmd = GameCommand {
        actor: session.current_player(),
        expected_revision: rev + 50,
        dedup_id: session.next_dedup_id(),
        action: GameAction::NextPhase,
    };
    session.assert_command_rejected(stale_cmd, Some(RejectReason::StaleRevision));
    assert_eq!(session.revision(), rev);
    assert_eq!(session.state_hash(), hash);
}

// ============================================================================
// Target Validation & Legal Action Projections
// ============================================================================

#[test]
fn test_t1_f6_invalid_card_play_rejected_1() {
    let mut session = OpaqueGameSession::new_identity_match(3001, 5).expect("match init");
    let current = session.current_player();
    session.assert_action_rejected(
        current,
        GameAction::PlayCard {
            card: 9999,
            targets: vec![],
        },
        None,
    );
}

#[test]
fn test_t1_f6_out_of_bounds_target_seat_rejected_2() {
    let mut session = OpaqueGameSession::new_identity_match(3002, 5).expect("match init");
    let current = session.current_player();
    session.assert_action_rejected(
        current,
        GameAction::PlayCard {
            card: 1,
            targets: vec![99],
        },
        None,
    );
}

#[test]
fn test_t1_f6_legal_actions_target_bounds_3() {
    let session = OpaqueGameSession::new_identity_match(3003, 5).expect("match init");
    let actions = session.legal_actions(session.current_player());
    for act in actions {
        if let GameAction::PlayCard { targets, .. } = act {
            for &t in &targets {
                assert!(
                    (t as usize) < session.players_count(),
                    "Target seat {t} must be within player count"
                );
            }
        }
    }
}

#[test]
fn test_t1_f6_open_choice_json_format_4() {
    let session = OpaqueGameSession::new_identity_match(3004, 5).expect("match init");
    let choice_json = session.open_choice_json(session.current_player());
    assert!(
        choice_json == "null" || choice_json.contains("id"),
        "Choice projection must be valid JSON null or object with id"
    );
}

#[test]
fn test_t1_f6_legal_actions_json_format_5() {
    let session = OpaqueGameSession::new_identity_match(3005, 5).expect("match init");
    let acts_json = session.legal_actions_json(session.current_player());
    let parsed: serde_json::Value =
        serde_json::from_str(&acts_json).expect("legal_actions_json must parse");
    assert_eq!(parsed["seat"], session.current_player());
    assert!(parsed["actions"].is_array());
}

// ============================================================================
// Skill Activation Validation & Zero-Mutation Invariants
// ============================================================================

#[test]
fn test_t1_f7_unknown_skill_activation_rejected_1() {
    let mut session = OpaqueGameSession::new_identity_match(4001, 5).expect("match init");
    session.assert_action_rejected(
        session.current_player(),
        GameAction::ActivateSkill { skill: 65535 },
        None,
    );
}

#[test]
fn test_t1_f7_inactive_seat_skill_activation_rejected_2() {
    let mut session = OpaqueGameSession::new_identity_match(4002, 5).expect("match init");
    let inactive = (session.current_player() + 1) % 5;
    session.assert_action_rejected(
        inactive,
        GameAction::ActivateSkill { skill: 1 },
        Some(RejectReason::NotYourTurn),
    );
}

#[test]
fn test_t1_f7_stale_revision_skill_activation_rejected_3() {
    let mut session = OpaqueGameSession::new_identity_match(4003, 5).expect("match init");
    let rev = session.revision();
    let dedup = session.next_dedup_id();
    let cmd = GameCommand {
        actor: session.current_player(),
        expected_revision: rev + 99,
        dedup_id: dedup,
        action: GameAction::ActivateSkill { skill: 1 },
    };
    session.assert_command_rejected(cmd, Some(RejectReason::StaleRevision));
}

#[test]
fn test_t1_f7_duplicate_dedup_skill_activation_rejected_4() {
    let mut session = OpaqueGameSession::new_identity_match(4004, 5).expect("match init");
    let rev = session.revision();
    let cmd = GameCommand {
        actor: session.current_player(),
        expected_revision: rev,
        dedup_id: 0,
        action: GameAction::ActivateSkill { skill: 1 },
    };
    session.assert_command_rejected(cmd, Some(RejectReason::DuplicateCommand));
}

#[test]
fn test_t1_f7_rejected_skill_activation_preserves_hash_5() {
    let mut session = OpaqueGameSession::new_identity_match(4005, 5).expect("match init");
    let hash = session.state_hash();
    let rev = session.revision();
    let _ = session.activate_skill(session.current_player(), 9999);
    assert_eq!(
        session.state_hash(),
        hash,
        "Failed skill activation must preserve state hash"
    );
    assert_eq!(
        session.revision(),
        rev,
        "Failed skill activation must preserve revision"
    );
}

// ============================================================================
// Open Choice Observation & Snapshot Roundtrips
// ============================================================================

#[test]
fn test_t1_f8_open_choice_projection_1() {
    let mut session = OpaqueGameSession::new_identity_match(5001, 5).expect("match init");
    assert!(
        session.open_choice().is_none(),
        "Initial standard match must start without open choice"
    );
    assert_eq!(
        session.open_choice_json(session.current_player()),
        "null",
        "open_choice_json must return 'null' when no choice is active"
    );

    let mut choice_detected = false;
    for _ in 0..15 {
        if let Some(choice) = session.open_choice() {
            choice_detected = true;
            assert!(choice.id > 0, "Open choice id must be positive");
            assert!(
                (choice.observer as usize) < session.players_count(),
                "Open choice observer must be a valid seat index"
            );
            let json_str = session.open_choice_json(choice.observer);
            assert!(
                json_str.contains(&format!("\"id\":{}", choice.id)),
                "Observer JSON view must contain open choice id"
            );
            break;
        }
        if session.is_finished() {
            break;
        }
        let _ = session.auto_step();
    }

    if !choice_detected {
        assert!(
            session.open_choice().is_none(),
            "Choice channel remains None when no interactive prompt is pending"
        );
    }
}

#[test]
fn test_t1_f8_snapshot_roundtrip_2() {
    let mut session = OpaqueGameSession::new_identity_match(5002, 5).expect("match init");
    for _ in 0..8 {
        let _ = session.auto_step();
    }
    session
        .verify_snapshot_roundtrip()
        .expect("Snapshot roundtrip must preserve exact session state");
}

#[test]
fn test_t1_f8_answer_without_open_choice_rejected_3() {
    let mut session = OpaqueGameSession::new_identity_match(5003, 5).expect("match init");
    if session.open_choice().is_none() {
        session.assert_action_rejected(
            session.current_player(),
            GameAction::Answer {
                choice_id: 9999,
                selection: vec![],
            },
            None,
        );
    }
}

#[test]
fn test_t1_f8_answer_by_inactive_player_rejected_4() {
    let mut session = OpaqueGameSession::new_identity_match(5004, 5).expect("match init");
    let inactive = (session.current_player() + 1) % 5;
    session.assert_action_rejected(
        inactive,
        GameAction::Answer {
            choice_id: 1,
            selection: vec![],
        },
        None,
    );
}

#[test]
fn test_t1_f8_initial_snapshot_roundtrip_5() {
    let session = OpaqueGameSession::new_identity_match(5005, 5).expect("match init");
    let hex = session.save_snapshot();
    let restored = OpaqueGameSession::restore_snapshot(&hex).expect("restore snapshot");
    assert_eq!(restored.revision(), session.revision());
    assert_eq!(restored.state_hash(), session.state_hash());
    assert_eq!(restored.state_hash_hex(), session.state_hash_hex());
}
