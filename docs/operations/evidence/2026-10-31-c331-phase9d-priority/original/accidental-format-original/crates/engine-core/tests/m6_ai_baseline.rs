//! M6 gate 1: 标准包 AI 可复现对局统计基线入库.
//!
//! Runs the deterministic auto-play identity match over a fixed set of seeds and
//! pins: (a) per-seed reproducibility (same seed → same state hash, outcome,
//! steps), (b) the outcome distribution baseline, and (c) clean termination.
//! The pinned numbers are the "AI 可复现对局统计基线" recorded for M6.

use engine_core::{run_identity_match, MatchConfig};

const SEEDS: &[u64] = &[7, 42, 99, 1234, 20_260_101, 20_260_202];
const PLAYER_COUNT: u8 = 4;
const MAX_STEPS: u32 = 1500;

fn run(seed: u64) -> engine_core::MatchReport {
    run_identity_match(MatchConfig {
        player_count: PLAYER_COUNT,
        rng_seed: seed,
        max_steps: MAX_STEPS,
        ruleset: "identity-standard".into(),
    })
    .expect("AI match must run")
}

#[test]
fn ai_matches_are_reproducible_per_seed() {
    for seed in SEEDS {
        let a = run(*seed);
        let b = run(*seed);
        assert_eq!(a.state_hash, b.state_hash, "seed {seed} hash stable");
        assert_eq!(a.outcome, b.outcome, "seed {seed} outcome stable");
        assert_eq!(a.steps, b.steps, "seed {seed} steps stable");
    }
}

#[test]
fn ai_match_statistics_baseline() {
    // Statistics baseline: pin the outcome distribution for the fixed seed set.
    // Regenerate intentionally only when the engine rules change.
    let mut wins = std::collections::BTreeMap::new();
    for seed in SEEDS {
        let report = run(*seed);
        if report.capped {
            wins.entry("capped".to_string())
                .and_modify(|c| *c += 1)
                .or_insert(1);
        } else {
            assert!(report.outcome.is_finished(), "seed {seed} finished");
            wins.entry(format!("{:?}", report.outcome))
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }
    }
    // Deterministic baseline for the pinned seed set (4-player identity, standard content).
    // Current baseline (2026-08): the built-in AI wins the lord/loyalist side for every
    // pinned seed — regenerated intentionally only when engine rules or the AI change.
    let baseline: Vec<(String, usize)> = wins.into_iter().collect();
    println!("AI baseline distribution: {baseline:?}");
    assert_eq!(
        baseline,
        vec![("LoyalistWin".to_string(), SEEDS.len())],
        "pinned AI match statistics baseline"
    );
}

#[test]
fn ai_match_terminates_cleanly_for_all_seeds() {
    for seed in SEEDS {
        let report = run(*seed);
        assert!(report.steps >= 1);
        assert!(report.revisions >= 1);
        assert!(
            report.capped || report.outcome.is_finished(),
            "seed {seed} must terminate or cap cleanly"
        );
    }
}
