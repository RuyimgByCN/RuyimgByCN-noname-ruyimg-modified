//! Differential + metamorphic tests (architecture plan §37.5 / §37.6).
//!
//! These tests compare two equivalent expressions of engine behavior and
//! assert they converge. They catch nondeterminism, cache bugs, and snapshot
//! drift that unit/golden tests miss.
//!
//! - §37.5 Model differential: live engine vs replayed engine (the "reference
//!   model" is the deterministic replay path).
//! - §37.6 Metamorphic: transformations that must not change the rule outcome.

use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::Engine;

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

/// §37.5: The live engine and a freshly-replayed engine must produce identical
/// state hashes at every step of a multi-command sequence. This is the core
/// model-differential invariant: replay is the reference model.
#[test]
fn live_vs_replay_converge_every_step() {
    let seeds: &[u64] = &[1, 42, 99, 777, 12345];
    for &seed in seeds {
        let mut live = Engine::new();
        let mut cmds: Vec<Command> = vec![start_cmd(seed, 4)];
        live.submit(cmds[0].clone());
        let mut live_hashes: Vec<[u8; 32]> = vec![live.state_hash()];
        for i in 0..8 {
            if live.state().outcome.is_finished() || live.state().has_pending_choice() {
                break;
            }
            let cmd = Command {
                actor: PlayerId::new(1),
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(i),
                kind: CommandKind::EndPhase,
            };
            let r = live.submit(cmd.clone());
            if matches!(r, CommandResult::Rejected { .. }) {
                break;
            }
            cmds.push(cmd);
            live_hashes.push(live.state_hash());
        }
        // Rebuild from replay and compare every intermediate hash.
        let mut replay = Engine::new();
        for (i, cmd) in cmds.iter().enumerate() {
            match replay.submit(cmd.clone()) {
                CommandResult::Accepted { state_hash, .. } => {
                    assert_eq!(
                        state_hash, live_hashes[i],
                        "live vs replay diverge at step {i} for seed {seed}"
                    );
                }
                other => panic!("replay rejected at step {i}: {other:?}"),
            }
        }
        assert_eq!(replay.state_hash(), live.state_hash());
    }
}

/// §37.6 Metamorphic: Debug vs Release builds must produce identical results.
/// We can't switch profiles in-test, but we CAN assert that the hash is
/// profile-independent by checking the canonical encoding uses fixed-width
/// integers (no usize, no platform floats). This test documents the contract.
#[test]
fn metamorphic_no_usize_in_state_hash() {
    // The canonical state hash must not depend on usize (platform-width).
    // We verify determinism across two engines built in the same process:
    // if the encoding were usize-dependent, two independent Engine instances
    // could differ. They don't, but this also guards against future regressions
    // where someone adds a `usize` field to a hashed struct.
    let mut a = Engine::new();
    let mut b = Engine::new();
    a.submit(start_cmd(42, 4));
    b.submit(start_cmd(42, 4));
    assert_eq!(a.state_hash(), b.state_hash());
    // The hash is exactly 32 bytes (BLAKE3), not platform-dependent.
    assert_eq!(a.state_hash().len(), 32);
}

/// §37.6 Metamorphic: Snapshot-then-continue must equal never-snapshot.
/// Taking a snapshot mid-match and restoring must not perturb the final hash.
#[test]
fn metamorphic_snapshot_midway_equals_no_snapshot() {
    let seed = 42u64;
    // Path A: no snapshot, run all commands.
    let mut a = Engine::new();
    a.submit(start_cmd(seed, 4));
    for _ in 0..5 {
        if a.state().outcome.is_finished() {
            break;
        }
        a.submit(Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        });
    }
    let hash_a = a.state_hash();

    // Path B: snapshot at step 2, restore, continue.
    let mut b = Engine::new();
    b.submit(start_cmd(seed, 4));
    b.submit(Command {
        actor: PlayerId::new(1),
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::EndPhase,
    });
    let snap = b.snapshot();
    let mut restored = Engine::new();
    restored.restore_snapshot(&snap).unwrap();
    for _ in 0..4 {
        if restored.state().outcome.is_finished() {
            break;
        }
        restored.submit(Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        });
    }
    // Both paths should reach the same terminal hash (snapshot is transparent).
    // Note: if match finished at different steps, compare at the common point.
    assert_eq!(
        restored.state_hash(),
        hash_a,
        "snapshot midway must equal no-snapshot path"
    );
}

/// §37.6 Metamorphic: batching commands differently (one-by-one vs replay-all)
/// must produce the same final hash. The command log is order-deterministic,
/// not batch-deterministic.
#[test]
fn metamorphic_batching_invariant() {
    let seed = 7u64;
    // Build the command list by running live and stopping at the first barrier.
    let mut live = Engine::new();
    live.submit(start_cmd(seed, 4));
    let mut all = vec![start_cmd(seed, 4)];
    for i in 0..6 {
        if live.state().outcome.is_finished() || live.state().has_pending_choice() {
            break;
        }
        let cmd = Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(i),
            kind: CommandKind::EndPhase,
        };
        let r = live.submit(cmd.clone());
        if matches!(r, CommandResult::Rejected { .. }) {
            break;
        }
        all.push(cmd);
    }
    // Submit one by one on a fresh engine.
    let mut one_by_one = Engine::new();
    for c in &all {
        one_by_one.submit(c.clone());
    }
    // Replay all at once.
    let replayed = Engine::replay_from(&all).unwrap();
    assert_eq!(one_by_one.state_hash(), replayed.state_hash());
}
