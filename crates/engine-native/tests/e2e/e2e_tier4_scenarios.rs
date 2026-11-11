//! Tier 4 E2E Test Suite: End-to-End Match Scenarios.
//!
//! Opaque-box integration tests verifying multi-turn identity and standard matches,
//! mid-game save/restore continuation, multi-room isolation, and archive fault injection.

use std::collections::HashSet;

use crate::e2e_harness::{hex_decode, hex_encode, OpaqueGameSession};
use engine_native::engine::CommandOutcome;
use engine_native::event::Phase;

/// Scenario 1: 5-Player Identity Match with Mid-Game Snapshot Save and Restore.
#[test]
fn test_t4_scenario_1_identity_5p_midgame_save_restore() {
    let mut session = OpaqueGameSession::new_identity_match(5001, 5).expect("init 5p match");
    assert_eq!(session.players_count(), 5);
    assert_eq!(session.player_hp(0), Some(5), "Lord starts with 5 HP");
    for seat in 1..5 {
        assert_eq!(session.player_hp(seat), Some(4), "Loyalists/Rebels start with 4 HP");
        assert_eq!(session.player_hand_count(seat), Some(4));
    }

    let steps_run = session.run_until_finished_or_limit(15);
    assert_eq!(steps_run, 15, "Executed 15 steps");
    assert!(session.revision() >= 15);

    let hex_snap = session.save_snapshot();
    assert!(hex_snap.len() > 100);

    let mut restored = OpaqueGameSession::restore_snapshot(&hex_snap)
        .expect("restore mid-game snapshot");

    assert_eq!(restored.revision(), session.revision());
    assert_eq!(restored.state_hash(), session.state_hash());
    assert_eq!(restored.current_player(), session.current_player());
    assert_eq!(restored.current_phase(), session.current_phase());
    for seat in 0..5 {
        assert_eq!(restored.player_hp(seat), session.player_hp(seat));
        assert_eq!(restored.player_hand_count(seat), session.player_hand_count(seat));
    }

    for step in 0..10 {
        let outcome_orig = session.auto_step().expect("step orig");
        let outcome_rest = restored.auto_step().expect("step restored");
        match (outcome_orig, outcome_rest) {
            (
                CommandOutcome::Accepted { state_hash: h1 },
                CommandOutcome::Accepted { state_hash: h2 },
            ) => {
                assert_eq!(h1, h2, "Step {step} hash mismatch between orig and restored");
            }
            (
                CommandOutcome::Pending { choice: c1 },
                CommandOutcome::Pending { choice: c2 },
            ) => {
                assert_eq!(c1.id, c2.id);
            }
            (o1, o2) => panic!("Divergence at step {step}: {o1:?} vs {o2:?}"),
        }
    }

    assert_eq!(restored.revision(), session.revision());
    assert_eq!(restored.state_hash(), session.state_hash());
}

/// Scenario 2: 8-Player Identity Match Multi-Turn Progression.
#[test]
fn test_t4_scenario_2_identity_8p_multi_turn_progression() {
    let mut session = OpaqueGameSession::new_identity_match(8002, 8).expect("init 8p match");
    assert_eq!(session.players_count(), 8);
    assert_eq!(session.player_hp(0), Some(5), "Lord has 5 HP in 8-player mode");
    for seat in 1..8 {
        assert_eq!(session.player_hp(seat), Some(4));
        assert_eq!(session.player_hand_count(seat), Some(4));
    }

    let mut phases_observed: Vec<Phase> = Vec::new();
    let initial_hash = session.state_hash();

    for step in 0..45 {
        let cur_phase = session.current_phase();
        if !phases_observed.contains(&cur_phase) {
            phases_observed.push(cur_phase);
        }
        let outcome = session.auto_step().expect("auto_step execution");
        assert!(
            matches!(
                outcome,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "Step {step} encountered invalid outcome: {outcome:?}"
        );

        for seat in 0..8 {
            if let Some(hp) = session.player_hp(seat) {
                let max_hp = session.player_max_hp(seat).unwrap_or(4) as i32;
                assert!(hp <= max_hp, "Seat {seat} HP {hp} exceeds max_hp {max_hp}");
            }
        }

        if session.is_finished() {
            break;
        }
    }

    assert_ne!(session.state_hash(), initial_hash);
    assert!(phases_observed.contains(&Phase::Draw) || phases_observed.contains(&Phase::Play));

    session.verify_snapshot_roundtrip().expect("roundtrip at 45 steps");
}

/// Scenario 3: Client Disconnect, Reconnect & Session Recovery Parity.
#[test]
fn test_t4_scenario_3_client_disconnect_reconnect_recovery() {
    let seed = 3003;
    let mut session_interrupted = OpaqueGameSession::new_identity_match(seed, 5).expect("init match");

    for _ in 0..12 {
        let _ = session_interrupted.auto_step();
    }

    let disconnect_hex = session_interrupted.save_snapshot();
    drop(session_interrupted);

    let mut session_resumed = OpaqueGameSession::restore_snapshot(&disconnect_hex)
        .expect("reconstruct session on reconnect");

    for _ in 0..12 {
        let _ = session_resumed.auto_step();
    }

    let mut session_control = OpaqueGameSession::new_identity_match(seed, 5).expect("init control");
    for _ in 0..24 {
        let _ = session_control.auto_step();
    }

    assert_eq!(
        session_resumed.revision(),
        session_control.revision(),
        "Revision mismatch between resumed and control sessions"
    );
    assert_eq!(
        session_resumed.state_hash(),
        session_control.state_hash(),
        "State hash mismatch between resumed and control sessions"
    );
    for seat in 0..5 {
        assert_eq!(
            session_resumed.player_hp(seat),
            session_control.player_hp(seat),
            "HP mismatch at seat {seat}"
        );
        assert_eq!(
            session_resumed.player_hand_count(seat),
            session_control.player_hand_count(seat),
            "Hand count mismatch at seat {seat}"
        );
    }
}

/// Scenario 4: Multi-Room Interleaved Execution Isolation.
#[test]
fn test_t4_scenario_4_high_concurrency_multi_room_stress() {
    let configs = [
        (4, 0x5347_0001),
        (5, 0x5347_0002),
        (6, 0x5347_0003),
        (7, 0x5347_0004),
        (8, 0x5347_0005),
        (4, 0x5347_0006),
        (5, 0x5347_0007),
        (6, 0x5347_0008),
    ];

    let mut rooms: Vec<OpaqueGameSession> = configs
        .iter()
        .map(|&(seats, seed)| OpaqueGameSession::new_identity_match(seed, seats).expect("init room"))
        .collect();

    assert_eq!(rooms.len(), 8);

    for _round in 0..20 {
        for room in &mut rooms {
            let _ = room.auto_step();
        }
    }

    let mut unique_hashes = HashSet::new();
    for (idx, room) in rooms.iter().enumerate() {
        assert!(room.revision() >= 20, "Room {idx} revision should be >= 20");
        let hash = room.state_hash();
        assert!(
            unique_hashes.insert(hash),
            "Room {idx} produced duplicate state hash (crosstalk detected)"
        );
        room.verify_snapshot_roundtrip()
            .unwrap_or_else(|e| panic!("Room {idx} roundtrip failed: {e}"));
    }
    assert_eq!(unique_hashes.len(), 8);
}

/// Scenario 5: Malicious Save File Fault Injection & Tamper Detection.
#[test]
fn test_t4_scenario_5_malicious_save_fault_injection_zero_leak() {
    let session = OpaqueGameSession::new_identity_match(5005, 5).expect("init match");
    let valid_hex = session.save_snapshot();
    let valid_bytes = hex_decode(&valid_hex).expect("decode valid hex");

    // Vector 1: Truncated hex string (< 16 chars)
    let corrupt_trunc = &valid_hex[..12];
    assert!(OpaqueGameSession::restore_snapshot(corrupt_trunc).is_err());

    // Vector 2: Odd-length hex string
    let corrupt_odd = format!("{valid_hex}a");
    assert!(OpaqueGameSession::restore_snapshot(&corrupt_odd).is_err());

    // Vector 3: Non-hex characters
    let corrupt_chars = format!("{}ZZZZ", &valid_hex[..valid_hex.len() - 4]);
    assert!(OpaqueGameSession::restore_snapshot(&corrupt_chars).is_err());

    // Vector 4: Corrupted magic bytes ("NGSA" -> "BAD!")
    let mut bad_magic_bytes = valid_bytes.clone();
    bad_magic_bytes[0..4].copy_from_slice(b"BAD!");
    let bad_magic_hex = hex_encode(&bad_magic_bytes);
    assert!(OpaqueGameSession::restore_snapshot(&bad_magic_hex).is_err());

    // Vector 5: Bit-flipped checksum (flip byte in trailing 32-byte digest)
    let mut bad_checksum_bytes = valid_bytes.clone();
    let len = bad_checksum_bytes.len();
    bad_checksum_bytes[len - 1] ^= 0xFF;
    let bad_checksum_hex = hex_encode(&bad_checksum_bytes);
    assert!(OpaqueGameSession::restore_snapshot(&bad_checksum_hex).is_err());

    // Vector 6: Incomplete payload bytes (truncated binary buffer)
    let truncated_bytes = &valid_bytes[..valid_bytes.len() / 2];
    let truncated_hex = hex_encode(truncated_bytes);
    assert!(OpaqueGameSession::restore_snapshot(&truncated_hex).is_err());

    // Vector 7: Empty string
    assert!(OpaqueGameSession::restore_snapshot("").is_err());

    // Vector 8: Extra trailing garbage bytes
    let mut trailing_bytes = valid_bytes;
    trailing_bytes.extend_from_slice(b"TRAILING_CORRUPTION_DATA");
    let trailing_hex = hex_encode(&trailing_bytes);
    assert!(OpaqueGameSession::restore_snapshot(&trailing_hex).is_err());

    assert_eq!(session.players_count(), 5);
    assert_eq!(session.revision(), 0);
}

/// Scenario 6: Standard 4-Player Match Progression and Snapshot Roundtrip.
#[test]
fn test_t4_scenario_6_standard_4p_match_progression_and_roundtrip() {
    let mut session = OpaqueGameSession::new_standard_match(6006, 4);
    assert_eq!(session.players_count(), 4);
    assert_eq!(session.revision(), 0);

    for _ in 0..10 {
        let _ = session.auto_step();
    }
    assert_eq!(session.revision(), 10);

    session.verify_snapshot_roundtrip().expect("roundtrip on standard match");
}

/// Scenario 9: Bounded Step Progression Across 4p, 5p, and 8p Identity Matches.
#[test]
fn test_t4_scenario_9_bounded_ai_planner_tournament() {
    let tournaments = [(4, 9001), (5, 9002), (8, 9003)];

    for &(seats, seed) in &tournaments {
        let mut session = OpaqueGameSession::new_identity_match(seed, seats)
            .unwrap_or_else(|e| panic!("Failed to init {seats}p tournament: {e}"));

        let mut last_rev = session.revision();
        let mut step_count = 0;
        let max_quota = 60;

        while !session.is_finished() && step_count < max_quota {
            let outcome = session.auto_step().expect("tournament step");
            assert!(
                matches!(
                    outcome,
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                ),
                "Invalid outcome in {seats}p tournament: {outcome:?}"
            );
            let cur_rev = session.revision();
            assert!(
                cur_rev > last_rev,
                "Deadlock detected in {seats}p tournament at step {step_count}"
            );
            last_rev = cur_rev;
            step_count += 1;
        }

        assert!(step_count > 0);
        session.verify_snapshot_roundtrip()
            .unwrap_or_else(|e| panic!("Tournament {seats}p snapshot failed: {e}"));
    }
}

/// Scenario 10: Deterministic Trajectory Parity and Seed Divergence.
#[test]
fn test_t4_scenario_10_deterministic_trajectory_parity_and_seed_divergence() {
    let seed_a = 0x5452_414A;
    let mut session1 = OpaqueGameSession::new_identity_match(seed_a, 5).expect("init session 1");
    let mut session2 = OpaqueGameSession::new_identity_match(seed_a, 5).expect("init session 2");

    for step in 0..25 {
        assert_eq!(session1.revision(), session2.revision(), "Revision diverged at step {step}");
        assert_eq!(session1.state_hash(), session2.state_hash(), "State hash diverged at step {step}");
        assert_eq!(session1.current_player(), session2.current_player());
        assert_eq!(session1.current_phase(), session2.current_phase());

        for seat in 0..5 {
            assert_eq!(session1.player_hp(seat), session2.player_hp(seat));
            assert_eq!(session1.player_hand_count(seat), session2.player_hand_count(seat));
        }

        let _ = session1.auto_step();
        let _ = session2.auto_step();
    }

    let seed_b = 0x4449_5647;
    let mut session_diff = OpaqueGameSession::new_identity_match(seed_b, 5).expect("init session diff");
    for _ in 0..25 {
        let _ = session_diff.auto_step();
    }
    assert_ne!(
        session1.state_hash(),
        session_diff.state_hash(),
        "Differing seeds must produce distinct state hashes"
    );
}
