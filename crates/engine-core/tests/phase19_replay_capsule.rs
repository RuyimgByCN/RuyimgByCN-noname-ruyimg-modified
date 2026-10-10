//! Replay Capsule integration tests (architecture plan §22.4).
//!
//! Verifies the full capsule lifecycle: build → serialize → deserialize → open →
//! replay. In Embedded mode, the capsule is self-contained and can be replayed
//! without any external artifacts.

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::{CapsuleMode, Engine};

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

/// §22.4: A capsule built from a live engine can be opened and replayed,
/// producing the same final state hash — no external artifacts needed.
#[test]
fn capsule_build_open_replay_round_trip() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(42, 4));
    eng.submit(Command {
        actor: PlayerId::new(1),
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::EndPhase,
    });
    let live_hash = eng.state_hash();
    let live_manifest = eng.manifest_hash();

    // Build an embedded capsule (self-contained, with snapshot).
    let capsule = eng
        .build_capsule(CapsuleMode::Embedded, true, &[])
        .expect("build capsule");

    // The capsule must be self-contained.
    assert!(
        capsule.is_self_contained(),
        "embedded capsule must be self-contained"
    );

    // Serialize → deserialize.
    let bytes = capsule.to_bytes().expect("serialize capsule");
    assert!(!bytes.is_empty());

    // Open the capsule and replay.
    let (replayed, restored_capsule) = Engine::open_capsule(&bytes).expect("open capsule");

    assert_eq!(
        replayed.state_hash(),
        live_hash,
        "capsule replay must match live state hash"
    );
    assert_eq!(
        replayed.manifest_hash(),
        live_manifest,
        "capsule manifest must match pinned manifest"
    );
    assert_eq!(restored_capsule.manifest_hash, live_manifest);
}

/// §22.4: A capsule without a snapshot still works (replay-only).
#[test]
fn capsule_without_snapshot() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(7, 2));
    let capsule = eng
        .build_capsule(CapsuleMode::Embedded, false, &[])
        .expect("build capsule");
    assert!(capsule.snapshot_bytes.is_none());
    let bytes = capsule.to_bytes().unwrap();
    let (replayed, _) = Engine::open_capsule(&bytes).expect("open capsule");
    assert_eq!(replayed.state_hash(), eng.state_hash());
}

/// §22.4: A capsule with embedded content packages includes their bytes.
#[test]
fn capsule_embeds_content_packages() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(3, 4));
    let fake_pkg_bytes = br#"{"package_id":"standard","version":"1.0.0"}"#.to_vec();
    let content_hash = [0xaa; 32];
    let capsule = eng
        .build_capsule(
            CapsuleMode::Embedded,
            false,
            &[(
                "standard".into(),
                "1.0.0".into(),
                content_hash,
                Some(fake_pkg_bytes.clone()),
            )],
        )
        .expect("build capsule");

    assert_eq!(capsule.content.len(), 1);
    assert_eq!(capsule.content[0].package_id, "standard");
    assert_eq!(
        capsule.content[0].package_bytes.as_ref().unwrap(),
        &fake_pkg_bytes
    );
    assert!(capsule.is_self_contained());
}

/// §22.4: Referenced mode does not embed package bytes (smaller capsule).
#[test]
fn capsule_referenced_mode_no_inline_bytes() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(9, 4));
    let capsule = eng
        .build_capsule(
            CapsuleMode::Referenced,
            false,
            &[(
                "standard".into(),
                "1.0.0".into(),
                [0xbb; 32],
                Some(vec![1, 2, 3]),
            )],
        )
        .expect("build capsule");

    assert_eq!(capsule.mode, CapsuleMode::Referenced);
    // In Referenced mode, package_bytes must be None even if provided.
    assert!(capsule.content[0].package_bytes.is_none());
    assert!(
        !capsule.is_self_contained(),
        "referenced capsule is not self-contained"
    );
}

/// §22.4: Capsule integrity verification catches a tampered manifest hash.
#[test]
fn capsule_tampered_manifest_detected() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(1, 2));
    let mut capsule = eng
        .build_capsule(CapsuleMode::Embedded, false, &[])
        .unwrap();
    // Tamper: flip a bit in the manifest hash.
    capsule.manifest_hash.0[0] ^= 0xff;
    let err = capsule.verify().unwrap_err();
    assert!(
        matches!(err, engine_core::CapsuleError::ManifestHashMismatch),
        "expected ManifestHashMismatch, got {err:?}"
    );
}

/// §22.4: Capsule carries compiler/VM compatibility info for long-term replay.
#[test]
fn capsule_carries_compat_info() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(5, 4));
    let capsule = eng
        .build_capsule(CapsuleMode::Embedded, false, &[])
        .unwrap();
    // Compat info must carry the current workspace versions.
    assert_eq!(
        capsule.compat.rule_ir_version,
        engine_api::RuleIrVersion::CURRENT
    );
    assert_eq!(
        capsule.compat.vm_abi_version,
        engine_api::VmAbiVersion::CURRENT
    );
    // Both native and wasm artifact build ids are recorded.
    assert!(!capsule.native_artifact.build.is_empty());
    assert!(!capsule.wasm_artifact.build.is_empty());
}

/// §22.4: Capsule from a match-in-progress has no outcome; completed match
/// carries outcome.
#[test]
fn capsule_outcome_only_when_finished() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(2, 2));
    // In-progress: no outcome.
    let capsule = eng
        .build_capsule(CapsuleMode::Embedded, false, &[])
        .unwrap();
    assert!(capsule.outcome.is_none());
}

/// §22.4: Building a capsule before StartGame fails (no manifest pinned).
#[test]
fn capsule_before_start_fails() {
    let eng = Engine::new();
    let result = eng.build_capsule(CapsuleMode::Embedded, false, &[]);
    assert!(result.is_err(), "capsule before StartGame must fail");
}

/// §22.4: A long match (many commands) produces a capsule that replays correctly.
#[test]
fn capsule_long_match_replays() {
    let mut eng = Engine::new();
    eng.submit(start_cmd(11, 4));
    for i in 0..10 {
        if eng.state().has_pending_choice() || eng.state().outcome.is_finished() {
            break;
        }
        eng.submit(Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(i),
            kind: CommandKind::EndPhase,
        });
    }
    let live_hash = eng.state_hash();
    let capsule = eng.build_capsule(CapsuleMode::Embedded, true, &[]).unwrap();
    let bytes = capsule.to_bytes().unwrap();
    let (replayed, _) = Engine::open_capsule(&bytes).expect("open capsule");
    assert_eq!(replayed.state_hash(), live_hash);
}
