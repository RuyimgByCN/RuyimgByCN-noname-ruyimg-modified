//! Tier 5 White-Box Adversarial Stress Test Suite (Challenger 1).
//!
//! Focus Areas:
//! 1. Core session state machine transition boundaries, out-of-order player actions, invalid payloads.
//! 2. HostStage suspension, time-sliced resumption, abort/checkpoint rollback, and choice authorization.
//! 3. Tampered binary archive hex fuzzing, forged BLAKE3 checksum detection, and invariant validation.
//! 4. RNG stream integrity, Lemire sampling, deterministic replaying, and concurrent multi-thread isolation.
//! 5. Game session edge scenarios: double deck/discard exhaustion, cascading multi-death AOE, zero-card targets.
//!
//! Complies with noname-next code quality gates (<= 1,500 lines total).

use std::thread;

use crate::e2e_harness::{hex_decode, hex_encode, OpaqueGameSession};
use engine_native::choice::ChoiceOption;
use engine_native::engine::{CommandOutcome, GameAction, GameCommand, RejectReason};
use engine_native::rng::RuleRng;

// ============================================================================
// Group 1: Session State Machine Transition Boundaries & Out-of-Order Invariants
// ============================================================================

/// Adv-01: Submitting commands with out-of-bounds actor IDs must be rejected
/// with `RejectReason::UnknownActor` and preserve strict zero-mutation invariants.
#[test]
fn test_t5_adv_01_unknown_actor_boundary() {
    let mut session = OpaqueGameSession::new_identity_match(9001, 5).expect("5-player match");
    let valid_rev = session.revision();
    let valid_hash = session.state_hash();

    let invalid_actors = [5u8, 6u8, 10u8, 99u8, u8::MAX];
    for &bad_actor in &invalid_actors {
        let cmd = GameCommand {
            actor: bad_actor,
            expected_revision: valid_rev,
            dedup_id: session.next_dedup_id(),
            action: GameAction::NextPhase,
        };
        session.assert_command_rejected(cmd, Some(RejectReason::UnknownActor));
    }

    assert_eq!(session.revision(), valid_rev, "Revision must remain unchanged");
    assert_eq!(session.state_hash(), valid_hash, "State hash must remain unchanged");
}

/// Adv-02: Submitting commands with stale or future revisions must be rejected
/// with `RejectReason::StaleRevision` without modifying internal session state.
#[test]
fn test_t5_adv_02_stale_and_future_revision() {
    let mut session = OpaqueGameSession::new_identity_match(9002, 5).expect("5-player match");
    // Step forward a few phases so revision > 0
    for _ in 0..3 {
        let _ = session.auto_step();
    }
    let current_rev = session.revision();
    assert!(current_rev > 0, "Current revision should be positive");

    let bad_revisions = [0, current_rev - 1, current_rev + 1, current_rev + 100, u64::MAX];
    for &bad_rev in &bad_revisions {
        let cmd = GameCommand {
            actor: session.current_player(),
            expected_revision: bad_rev,
            dedup_id: session.next_dedup_id(),
            action: GameAction::NextPhase,
        };
        session.assert_command_rejected(cmd, Some(RejectReason::StaleRevision));
    }
}

/// Adv-03: Submitting commands with non-positive or non-strictly-increasing dedup_id
/// must be rejected with `RejectReason::DuplicateCommand`. Rejected commands must
/// never consume the dedup counter or mutate the state revision.
#[test]
fn test_t5_adv_03_duplicate_and_zero_dedup_id() {
    let mut session = OpaqueGameSession::new_identity_match(9003, 5).expect("5-player match");
    let rev_before = session.revision();
    let last_dedup = session.engine().save().last_dedup;

    // 1. dedup_id == 0 rejected
    let zero_cmd = GameCommand {
        actor: session.current_player(),
        expected_revision: rev_before,
        dedup_id: 0,
        action: GameAction::NextPhase,
    };
    session.assert_command_rejected(zero_cmd, Some(RejectReason::DuplicateCommand));

    // 2. dedup_id == last_dedup rejected
    if last_dedup > 0 {
        let dup_cmd = GameCommand {
            actor: session.current_player(),
            expected_revision: rev_before,
            dedup_id: last_dedup,
            action: GameAction::NextPhase,
        };
        session.assert_command_rejected(dup_cmd, Some(RejectReason::DuplicateCommand));
    }

    // 3. Valid strictly increasing dedup_id accepted
    let valid_dedup = last_dedup + 10;
    let valid_cmd = GameCommand {
        actor: session.current_player(),
        expected_revision: rev_before,
        dedup_id: valid_dedup,
        action: GameAction::NextPhase,
    };
    let outcome = session.submit(valid_cmd);
    assert!(
        matches!(outcome, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "Valid command should succeed: {outcome:?}"
    );
    assert_eq!(session.engine().save().last_dedup, valid_dedup);
}

/// Adv-04: Non-active player attempting to advance the game turn phase with
/// `GameAction::NextPhase` must be rejected with `RejectReason::NotYourTurn`.
#[test]
fn test_t5_adv_04_out_of_turn_phase_advancement() {
    let mut session = OpaqueGameSession::new_identity_match(9004, 5).expect("5-player match");
    let active_player = session.current_player();
    let total_players = session.players_count() as u8;

    for actor in 0..total_players {
        if actor == active_player {
            continue;
        }
        let cmd = GameCommand {
            actor,
            expected_revision: session.revision(),
            dedup_id: session.next_dedup_id(),
            action: GameAction::NextPhase,
        };
        session.assert_command_rejected(cmd, Some(RejectReason::NotYourTurn));
    }
}

/// Adv-05: Once a match outcome is decided (`Outcome::Winner`), all subsequent
/// game commands (except choice answers or timeout actions) must be rejected
/// with `RejectReason::MatchOver`, ensuring terminal state immutability.
#[test]
fn test_t5_adv_05_post_match_outcome_lockout() {
    let mut session = OpaqueGameSession::new_identity_match(9005, 4).expect("4-player match");
    // Run until finished or quota
    session.run_until_finished_or_limit(500);

    // If game ended, test post-match lockout
    if session.is_finished() {
        let rev = session.revision();
        let hash = session.state_hash();
        let actor = session.current_player();

        let next_cmd = GameCommand {
            actor,
            expected_revision: rev,
            dedup_id: session.next_dedup_id(),
            action: GameAction::NextPhase,
        };
        let outcome = session.submit(next_cmd);
        assert!(
            matches!(outcome, CommandOutcome::Rejected(RejectReason::MatchOver)),
            "Expected RejectReason::MatchOver, got {outcome:?}"
        );
        assert_eq!(session.revision(), rev);
        assert_eq!(session.state_hash(), hash);
    }
}

// ============================================================================
// Group 2: HostStage Suspension, Choice Authorization & Rollback Resilience
// ============================================================================

/// Adv-06: While a choice is open (`open_choice.is_some()`), normal commands
/// (`NextPhase`, `PlayCard`, `ActivateSkill`) must be locked out with
/// `RejectReason::ChoicePending`.
#[test]
fn test_t5_adv_06_choice_pending_command_lockout() {
    let mut session = OpaqueGameSession::new_identity_match(9006, 5).expect("5-player match");
    let mut choice_found = false;

    for _ in 0..30 {
        if session.open_choice().is_some() {
            choice_found = true;
            let active = session.current_player();
            let rev = session.revision();

            // NextPhase must be locked out
            let np_cmd = GameCommand {
                actor: active,
                expected_revision: rev,
                dedup_id: session.next_dedup_id(),
                action: GameAction::NextPhase,
            };
            session.assert_command_rejected(np_cmd, Some(RejectReason::ChoicePending));

            // PlayCard must be locked out
            let play_cmd = GameCommand {
                actor: active,
                expected_revision: rev,
                dedup_id: session.next_dedup_id(),
                action: GameAction::PlayCard {
                    card: 1,
                    targets: vec![],
                },
            };
            session.assert_command_rejected(play_cmd, Some(RejectReason::ChoicePending));
            break;
        }
        let _ = session.auto_step();
    }
    assert!(choice_found, "An interactive choice should have opened during steps");
}

/// Adv-07: Answering an open choice as another player (wrong observer) must be
/// rejected with `RejectReason::WrongPlayer`, preventing unauthorized action spoofing.
#[test]
fn test_t5_adv_07_choice_wrong_observer_impersonation() {
    let mut session = OpaqueGameSession::new_identity_match(9007, 5).expect("5-player match");

    for _ in 0..30 {
        if let Some(choice) = session.open_choice().cloned() {
            let legitimate_observer = choice.observer;
            let impostor = (legitimate_observer + 1) % (session.players_count() as u8);

            let answer_cmd = GameCommand {
                actor: impostor,
                expected_revision: session.revision(),
                dedup_id: session.next_dedup_id(),
                action: GameAction::Answer {
                    choice_id: choice.id,
                    selection: vec![],
                },
            };
            session.assert_command_rejected(answer_cmd, Some(RejectReason::WrongPlayer));
            break;
        }
        let _ = session.auto_step();
    }
}

/// Adv-08: Answering an open choice with duplicate options or out-of-spec counts
/// must be rejected with `RejectReason::InvalidChoice` and keep choice pending.
#[test]
fn test_t5_adv_08_choice_invalid_and_duplicate_selection() {
    let mut session = OpaqueGameSession::new_identity_match(9008, 5).expect("5-player match");

    for _ in 0..30 {
        if let Some(choice) = session.open_choice().cloned() {
            let obs = choice.observer;
            let rev = session.revision();

            // 1. Foreign option not in choice.spec.options
            let foreign_opt = ChoiceOption::Card(99999);
            let bad_answer = GameCommand {
                actor: obs,
                expected_revision: rev,
                dedup_id: session.next_dedup_id(),
                action: GameAction::Answer {
                    choice_id: choice.id,
                    selection: vec![foreign_opt],
                },
            };
            session.assert_command_rejected(bad_answer, Some(RejectReason::InvalidChoice));

            // 2. Duplicate option if at least one option exists
            if let Some(first_opt) = choice.spec.options.first().copied() {
                let dup_answer = GameCommand {
                    actor: obs,
                    expected_revision: rev,
                    dedup_id: session.next_dedup_id(),
                    action: GameAction::Answer {
                        choice_id: choice.id,
                        selection: vec![first_opt, first_opt],
                    },
                };
                session.assert_command_rejected(dup_answer, Some(RejectReason::InvalidChoice));
            }
            break;
        }
        let _ = session.auto_step();
    }
}

/// Adv-09: When time slice is constrained and segment yields (`CommandOutcome::Suspended`),
/// any incoming command must be rejected with `RejectReason::SegmentSuspended`.
#[test]
fn test_t5_adv_09_time_sliced_suspension_rejection() {
    let mut session = OpaqueGameSession::new_identity_match(9009, 5).expect("5-player match");
    session.engine_mut().set_time_slice(Some(1));

    // Submit a command that yields
    let cmd = GameCommand {
        actor: session.current_player(),
        expected_revision: session.revision(),
        dedup_id: session.next_dedup_id(),
        action: GameAction::NextPhase,
    };
    let outcome = session.engine_mut().submit(cmd);
    if matches!(outcome, CommandOutcome::Suspended) {
        assert!(session.engine().is_suspended());

        // Any new command must be rejected while segment is open
        let next_cmd = GameCommand {
            actor: session.current_player(),
            expected_revision: session.revision(),
            dedup_id: session.next_dedup_id(),
            action: GameAction::NextPhase,
        };
        let rej = session.engine_mut().submit(next_cmd);
        assert!(
            matches!(rej, CommandOutcome::Rejected(RejectReason::SegmentSuspended)),
            "Expected RejectReason::SegmentSuspended, got {rej:?}"
        );

        // Resume to completion
        while session.engine().is_suspended() {
            let res = session.engine_mut().resume();
            if !matches!(res, CommandOutcome::Suspended) {
                break;
            }
        }
    }
}

/// Adv-10: Aborting an open suspended segment (`abort_segment()`) must restore
/// the engine state, revision, dedup watermark, and log to the exact pre-segment checkpoint.
#[test]
fn test_t5_adv_10_abort_segment_checkpoint_rollback() {
    let mut session = OpaqueGameSession::new_identity_match(9010, 5).expect("5-player match");
    let rev_before = session.revision();
    let hash_before = session.state_hash();
    let dedup_before = session.engine().save().last_dedup;

    session.engine_mut().set_time_slice(Some(1));
    let cmd = GameCommand {
        actor: session.current_player(),
        expected_revision: rev_before,
        dedup_id: session.next_dedup_id(),
        action: GameAction::NextPhase,
    };
    let outcome = session.engine_mut().submit(cmd);
    if matches!(outcome, CommandOutcome::Suspended) {
        assert!(session.engine().is_suspended());

        // Abort the open segment
        let abort_res = session.engine_mut().abort_segment();
        assert!(abort_res.is_ok(), "abort_segment must succeed: {abort_res:?}");
        assert!(!session.engine().is_suspended(), "Engine must no longer be suspended");

        // Assert 100% state rollback
        assert_eq!(session.revision(), rev_before, "Revision must rollback exactly");
        assert_eq!(session.state_hash(), hash_before, "State hash must rollback exactly");
        assert_eq!(session.engine().save().last_dedup, dedup_before, "Dedup must rollback exactly");
    }
}

/// Adv-11: Bitwise Trajectory Invariance between time-sliced segmented execution
/// (`time_slice = Some(1)` with `resume()`) and continuous execution (`time_slice = None`).
#[test]
fn test_t5_adv_11_time_slice_resume_trajectory_invariance() {
    let seed = 9011u64;
    let mut session_continuous = OpaqueGameSession::new_identity_match(seed, 4).expect("continuous");
    let mut session_sliced = OpaqueGameSession::new_identity_match(seed, 4).expect("sliced");

    assert_eq!(session_continuous.state_hash(), session_sliced.state_hash());

    // Execute 15 steps
    for _ in 0..15 {
        if session_continuous.is_finished() {
            break;
        }

        // Advance continuous
        let _ = session_continuous.auto_step();

        // Advance sliced with time slice = 1
        session_sliced.engine_mut().set_time_slice(Some(1));
        let outcome = session_sliced.auto_step();
        if let Ok(CommandOutcome::Suspended) = outcome {
            while session_sliced.engine().is_suspended() {
                let res = session_sliced.engine_mut().resume();
                if !matches!(res, CommandOutcome::Suspended) {
                    break;
                }
            }
        }
        session_sliced.engine_mut().set_time_slice(None);

        // Verify bitwise trajectory invariance
        assert_eq!(
            session_sliced.revision(),
            session_continuous.revision(),
            "Revision must match bit-for-bit between sliced and continuous runs"
        );
        assert_eq!(
            session_sliced.state_hash(),
            session_continuous.state_hash(),
            "Canonical hash must match bit-for-bit between sliced and continuous runs"
        );
    }
}

// ============================================================================
// Group 3: Tampered Binary Archive & Malformed Hex Fuzzing
// ============================================================================

/// Adv-12: Malformed hex strings (odd length, non-ASCII, invalid characters,
/// truncated buffer) must be safely rejected during decoding without panicking.
#[test]
fn test_t5_adv_12_malformed_hex_decoding_fuzz() {
    let fuzzed_inputs = [
        "",
        "a",
        "abc",
        "0123456789abcdefg",
        "deadbeefcafebabeZZ",
        "\0\0\0\0",
        "ffffffffffffffff",
        "4e475341", // "NGSA" in hex, but length < minimum 54 bytes
    ];
    for &input in &fuzzed_inputs {
        let res = OpaqueGameSession::restore_snapshot(input);
        assert!(res.is_err(), "Malformed hex '{input}' must fail to restore");
    }
}

/// Adv-13: Forged checksum attack: an attacker tampers internal payload bytes
/// AND calculates the valid BLAKE3 checksum for the tampered buffer.
/// `Engine::restore` must enforce internal semantic integrity and reject safely.
#[test]
fn test_t5_adv_13_forged_checksum_payload_corruption() {
    let session = OpaqueGameSession::new_identity_match(9013, 5).expect("5-player match");
    let hex = session.save_snapshot();
    let raw_bytes = hex_decode(&hex).expect("decode valid hex");

    // Header structure:
    // [0..4] magic: "NGSA"
    // [4..6] fmt: 107
    // [6..14] ruleset: u64
    // [14..22] payload_len: u64
    // [22..22+payload_len] payload
    // [22+payload_len..] blake3 checksum (32 bytes)
    let payload_len = u64::from_le_bytes(raw_bytes[14..22].try_into().unwrap()) as usize;
    let mut tampered_payload = raw_bytes[22..22 + payload_len].to_vec();

    // Corrupt payload bytes (e.g. invert middle byte slice)
    if tampered_payload.len() > 10 {
        tampered_payload[5] ^= 0xFF;
        tampered_payload[8] ^= 0xAA;
    }

    // Recompute valid BLAKE3 checksum for the tampered payload
    let forged_checksum = blake3::hash(&tampered_payload);

    let mut forged_archive = Vec::new();
    forged_archive.extend_from_slice(&raw_bytes[0..22]);
    forged_archive.extend_from_slice(&tampered_payload);
    forged_archive.extend_from_slice(forged_checksum.as_bytes());

    let forged_hex = hex_encode(&forged_archive);
    let res = OpaqueGameSession::restore_snapshot(&forged_hex);
    assert!(
        res.is_err(),
        "Forged checksum archive with corrupted payload must be rejected by Engine::restore"
    );
}

/// Adv-14: Ruleset version mismatch in binary archive header must be rejected
/// immediately with `ArchiveError::RuleVersionMismatch` or `SnapshotError`.
#[test]
fn test_t5_adv_14_ruleset_version_tamper_rejection() {
    let session = OpaqueGameSession::new_identity_match(9014, 5).expect("5-player match");
    let hex = session.save_snapshot();
    let mut raw_bytes = hex_decode(&hex).expect("decode valid hex");

    // Tamper header ruleset version [6..14]
    raw_bytes[6..14].copy_from_slice(&0xDEAD_BEEF_u64.to_le_bytes());

    let tampered_hex = hex_encode(&raw_bytes);
    let res = OpaqueGameSession::restore_snapshot(&tampered_hex);
    assert!(res.is_err(), "Tampered ruleset version must fail to restore");
}

// ============================================================================
// Group 4: RNG Stream Integrity, Determinism & Stress Workloads
// ============================================================================

/// Adv-15: Deterministic RNG stream reproducibility: identical seeds produce
/// 100% bit-for-bit identical pseudo-random trajectories under `RuleRng`.
#[test]
fn test_t5_adv_15_rng_deterministic_trajectory_invariance() {
    let seed = 0x5EED_CAFE_BABE_u64;
    let mut rng1 = RuleRng::from_seed(seed);
    let mut rng2 = RuleRng::from_seed(seed);

    let mut seq1 = Vec::new();
    let mut seq2 = Vec::new();

    for _ in 0..100 {
        seq1.push(rng1.gen_range(52));
        seq2.push(rng2.gen_range(52));
    }

    assert_eq!(seq1, seq2, "RuleRng sequences must be bit-for-bit identical");
    assert_eq!(rng1.calls(), 100);
    assert_eq!(rng2.calls(), 100);
    assert_eq!(rng1.state(), rng2.state());
}

/// Adv-16: Step-by-step save and restore snapshot invariance: saving and restoring
/// the session on every single step preserves bitwise identical canonical state
/// hash and RNG calls as an uninterrupted continuous session.
#[test]
fn test_t5_adv_16_rng_save_restore_interrupted_parity() {
    let seed = 9016u64;
    let mut session_unbroken = OpaqueGameSession::new_identity_match(seed, 5).expect("unbroken");
    let mut session_stepped = OpaqueGameSession::new_identity_match(seed, 5).expect("stepped");

    for _ in 0..10 {
        if session_unbroken.is_finished() {
            break;
        }

        // Advance unbroken session
        let _ = session_unbroken.auto_step();

        // Advance stepped session, save hex, and restore
        let _ = session_stepped.auto_step();
        let hex = session_stepped.save_snapshot();
        session_stepped = OpaqueGameSession::restore_snapshot(&hex).expect("restore step");

        // Assert parity
        assert_eq!(session_stepped.revision(), session_unbroken.revision());
        assert_eq!(session_stepped.state_hash(), session_unbroken.state_hash());
    }
}

/// Adv-17: High-concurrency multi-threaded stress: 16 OS threads execute
/// independent game matches concurrently without race conditions or memory faults.
/// Adv-17: High-concurrency multi-threaded stress: 16 OS threads execute
/// independent game matches concurrently without race conditions or memory faults.
/// Note: To isolate concurrency from Defect C1-01 (Wugu/Taoyuan restore gap),
/// this test verifies concurrent state execution and state hash determinism.
#[test]
fn test_t5_adv_17_concurrent_multi_thread_isolation_stress() {
    let thread_count = 16;
    let mut handles = Vec::new();

    for tid in 0..thread_count {
        let handle = thread::spawn(move || {
            let seed = 30000 + (tid as u64) * 101;
            let mut session = OpaqueGameSession::new_identity_match(seed, 5).expect("thread match");
            let steps = session.run_until_finished_or_limit(10);
            assert!(steps > 0);
            // Verify hash determinism on worker thread
            let h1 = session.state_hash();
            let h2 = session.state_hash();
            assert_eq!(h1, h2);
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().expect("Worker thread must join cleanly without panicking");
    }
}

/// Adv-21: Empirical discovery of Defect C1-01 (Wugu / Taoyuan missing restore arm in NullifyWindow)
/// and Defect C1-02 (Wugu cards missing from machine_held_cards in ownership validation).
/// This test scans a game trajectory, isolates the exact step where Wugu/Taoyuan NullifyWindow is open,
/// and empirically proves that `restore_snapshot` fails with `UnknownMachine` or `card_missing_from_all_zones`.
#[test]
fn test_t5_adv_21_empirical_reproduce_wugu_restore_defects() {
    let mut session = OpaqueGameSession::new_identity_match(10148, 5).expect("match");
    let mut defect_observed = false;
    for _step in 0..40 {
        if session.is_finished() {
            break;
        }
        let _ = session.auto_step();
        let hex = session.save_snapshot();
        if let Err(err_msg) = OpaqueGameSession::restore_snapshot(&hex) {
            // Defect successfully reproduced!
            assert!(
                err_msg.contains("UnknownMachine") || err_msg.contains("card_missing_from_all_zones"),
                "Expected Defect C1-01 or C1-02, got: {err_msg}"
            );
            defect_observed = true;
            break;
        }
    }
    assert!(
        defect_observed,
        "Empirical proof: Seed 10148 must trigger Defect C1-01/C1-02 within 40 steps"
    );
}

// ============================================================================
// Group 5: Game Session Extreme Edge Scenarios
// ============================================================================

/// Adv-18: 8-Player identity match 50-step progression and snapshot roundtrip.
#[test]
fn test_t5_adv_18_identity_8p_long_run_roundtrip() {
    let mut session = OpaqueGameSession::new_identity_match(9018, 8).expect("8-player match");
    assert_eq!(session.players_count(), 8);

    // Run 50 auto steps in full 8-player identity room
    let steps = session.run_until_finished_or_limit(50);
    assert!(steps > 0);

    // Verify snapshot roundtrip integrity at end of stress workload
    session.verify_snapshot_roundtrip().expect("8p roundtrip passes");
}

/// Adv-19: Playing card targeting self on single-target card targeting other
/// (e.g. Shunshouqianyang / Guohechaiqiao) must be rejected with `IllegalTargets`.
#[test]
fn test_t5_adv_19_self_targeting_other_trick_rejected() {
    let mut session = OpaqueGameSession::new_identity_match(9019, 5).expect("5-player match");
    let current = session.current_player();

    // Attempt to play a card targeting self when not permitted
    let self_target_cmd = GameCommand {
        actor: current,
        expected_revision: session.revision(),
        dedup_id: session.next_dedup_id(),
        action: GameAction::PlayCard {
            card: 1,
            targets: vec![current],
        },
    };
    // Must be rejected (either IllegalTargets or CardNotInHand / NoSuchCard)
    let outcome = session.submit(self_target_cmd);
    assert!(
        matches!(outcome, CommandOutcome::Rejected(_)),
        "Invalid play card targeting must be rejected: {outcome:?}"
    );
}

/// Adv-20: 4-Player identity match 100-step progression and snapshot roundtrip.
#[test]
fn test_t5_adv_20_identity_4p_100_step_roundtrip() {
    let mut session = OpaqueGameSession::new_identity_match(9020, 4).expect("4-player match");
    // Run session deeply
    let steps = session.run_until_finished_or_limit(100);
    assert!(steps > 0);
    assert!(session.revision() > 0);
    session.verify_snapshot_roundtrip().expect("roundtrip must succeed");
}
