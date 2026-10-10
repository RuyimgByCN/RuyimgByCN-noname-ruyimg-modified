//! Golden vector regression (architecture plan §37.1).
//!
//! These tests pin the exact state hashes for representative match scenarios.
//! Any change to rule semantics, hashing, or RNG must intentionally update
//! these vectors and record an ADR note (§41).
//!
//! The vectors mirror `crates/conformance` but live here as a second
//! independent check so a refactor in one place can't silently desync them.

use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::{run_identity_match, Engine, MatchConfig};

fn hex32(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

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

/// §37.1: StartGame golden vectors (bootstrap ruleset, cross-checked vs conformance crate).
#[test]
fn golden_start_vectors() {
    let cases: &[(u8, u64, &str)] = &[
        (
            4,
            42,
            "86daeaffe7f71fb6facda6e6ebe358a87076f5d9e2e7be196deb111b698c31a7",
        ),
        (
            2,
            1,
            "5834b20ea9fbb7d2d3664af54bc49edad763e3089641d88fc181eb9ca19292ee",
        ),
        (
            8,
            99,
            "8835f8955fcc4af79589c2c30718e3054ca20b78c31dcddeddbe6726685caa7e",
        ),
    ];
    for &(players, seed, expected) in cases {
        let mut eng = Engine::new();
        let r = eng.submit(start_cmd(seed, players, "bootstrap"));
        match r {
            CommandResult::Accepted { state_hash, .. } => {
                let got = hex32(&state_hash);
                assert_eq!(got, expected, "start ({players},{seed}) hash drift");
            }
            other => panic!("start ({players},{seed}) rejected: {other:?}"),
        }
    }
}

/// §37.1: Identity match golden vectors (full match to terminal state).
#[test]
fn golden_identity_match_vectors() {
    let cases: &[(u8, u64, u32, &str)] = &[
        (
            4,
            42,
            5_000,
            "b351f4876316e27bb5e5aea05dbe3fc29c127e910eacd7b28b4f24a07681abd1",
        ),
        (
            2,
            7,
            5_000,
            "8273d6cbf62d9045576f92a8cea42a4dc9431e5fee83a3461b18a584635ca217",
        ),
        (
            8,
            99,
            5_000,
            "9ba17d5af5ffef0f8dd46314acd35b41af6c24ee9f80aa8c4b50319b13b5d406",
        ),
    ];
    for &(players, seed, max_steps, expected) in cases {
        let report = run_identity_match(MatchConfig {
            player_count: players,
            rng_seed: seed,
            max_steps,
            ruleset: "identity-standard".into(),
        })
        .unwrap_or_else(|e| panic!("match ({players},{seed}): {e}"));
        assert!(
            !report.capped,
            "match ({players},{seed}) capped at {}",
            report.steps
        );
        let got = hex32(&report.state_hash);
        assert_eq!(
            got, expected,
            "identity match ({players},{seed}) hash drift"
        );
    }
}
