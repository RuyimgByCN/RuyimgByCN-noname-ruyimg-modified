//! Phase 8 acceptance: isolation, reproducibility, stale discard.

use engine_ai::{
    apply_if_fresh, build_information_set, legal_actions, AiRuntime, BeamSearch, HeuristicSearch,
    MctsSearch, SearchBudget, StateFork,
};
use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::Engine;

fn start(seed: u64) -> Engine {
    let mut eng = Engine::new();
    let r = eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(r, CommandResult::Accepted { .. }));
    eng
}

#[test]
fn fork_does_not_touch_authority_hash() {
    let eng = start(5);
    let h = eng.state_hash();
    let mut fork = StateFork::from_authority(&eng, 99);
    let actor = eng.state().turn.current;
    let acts = legal_actions(fork.engine(), actor);
    if let Some(a) = acts.first() {
        let cmd = a.to_command(actor, 0);
        let _ = fork.apply(cmd);
    }
    assert_eq!(eng.state_hash(), h);
}

#[test]
fn heuristic_reproducible_under_fixed_budget() {
    let eng = start(8);
    let actor = eng.state().turn.current;
    let budget = SearchBudget {
        max_nodes: 40,
        max_depth: 4,
    };
    let rt = AiRuntime::new(123);
    let mut p1 = HeuristicSearch;
    let mut p2 = HeuristicSearch;
    let d1 = rt.decide(&eng, actor, &mut p1, budget).unwrap();
    let d2 = rt.decide(&eng, actor, &mut p2, budget).unwrap();
    assert_eq!(d1.command.kind, d2.command.kind);
    assert_eq!(d1.base_revision, d2.base_revision);
}

#[test]
fn mcts_reproducible_under_fixed_nodes() {
    let eng = start(9);
    // advance to play for more actions
    let mut eng = eng;
    let actor = eng.state().turn.current;
    eng.submit(Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::EndPhase,
    });
    let actor = eng.state().turn.current;
    let budget = SearchBudget {
        max_nodes: 48,
        max_depth: 3,
    };
    let rt = AiRuntime::new(7);
    let mut a = MctsSearch::default();
    let mut b = MctsSearch::default();
    let d1 = rt.decide(&eng, actor, &mut a, budget).unwrap();
    let d2 = rt.decide(&eng, actor, &mut b, budget).unwrap();
    assert_eq!(d1.command, d2.command);
}

#[test]
fn beam_reproducible() {
    let eng = start(10);
    let actor = eng.state().turn.current;
    let budget = SearchBudget {
        max_nodes: 30,
        max_depth: 3,
    };
    let rt = AiRuntime::new(3);
    let mut a = BeamSearch { width: 3 };
    let mut b = BeamSearch { width: 3 };
    let d1 = rt.decide(&eng, actor, &mut a, budget).unwrap();
    let d2 = rt.decide(&eng, actor, &mut b, budget).unwrap();
    assert_eq!(d1.command, d2.command);
}

#[test]
fn info_set_no_foreign_hand() {
    let eng = start(12);
    let p1 = PlayerId::new(1);
    let p2 = PlayerId::new(2);
    let view = build_information_set(&eng, p1);
    for cid in eng.state().hand_cards(p2) {
        assert!(!view
            .observer
            .known_cards
            .iter()
            .any(|c| c.id == cid && c.def.is_some()));
    }
}

#[test]
fn apply_fresh_works_then_stale_fails() {
    let mut eng = start(15);
    let actor = eng.state().turn.current;
    let rt = AiRuntime::new(1);
    let mut policy = HeuristicSearch;
    let decision = rt
        .decide(&eng, actor, &mut policy, SearchBudget::default())
        .unwrap();
    // First apply succeeds
    apply_if_fresh(&mut eng, &decision).unwrap();
    // Same decision now stale
    let err = apply_if_fresh(&mut eng, &decision).unwrap_err();
    assert!(matches!(err, engine_ai::AiError::Stale { .. }));
}
