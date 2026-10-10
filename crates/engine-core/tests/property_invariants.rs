//! Property-based tests for the deterministic core (architecture plan §37.4).
//!
//! These tests use `proptest` to exercise the engine over many random inputs
//! and assert the architecture's universal invariants hold on every case.
//! They complement the hand-written golden vectors (§37.1) and the
//! model/differential tests (§37.5/§37.6).

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, ObserverKind, PlayerId,
};
use engine_core::{project_observer, query, Engine, QueryContext};
use proptest::prelude::*;
use rule_api::QueryRequest;

/// Any u64 seed.
fn any_seed() -> impl Strategy<Value = u64> {
    any::<u64>()
}

/// Player counts 2..=8.
fn any_player_count() -> impl Strategy<Value = u8> {
    2u8..=8
}

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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// §37.4 #1/#2: After StartGame, every physical card is in exactly one zone
    /// and card conservation holds (store count == zone refs).
    #[test]
    fn card_conservation_after_start(seed in any_seed(), n in any_player_count()) {
        let mut eng = Engine::new();
        let r = eng.submit(start_cmd(seed, n));
        let _accepted = matches!(r, CommandResult::Accepted { .. }); prop_assert!(_accepted);
        engine_core::invariants::check_invariants(eng.state()).unwrap();
        let store = eng.state().cards.live_count();
        let zone_refs: usize = eng.state().zones.sorted_live()
            .iter()
            .map(|z| z.cards.len())
            .sum();
        prop_assert_eq!(store, zone_refs, "card conservation must hold");
    }

    /// §37.4 #3: An illegal command (stale revision) must not change the state hash.
    #[test]
    fn illegal_command_zero_state_change(seed in any_seed(), n in any_player_count()) {
        let mut eng = Engine::new();
        eng.submit(start_cmd(seed, n));
        let h_before = eng.state_hash();
        let r = eng.submit(Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(999_999),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        });
        let _rejected = matches!(r, CommandResult::Rejected { .. }); prop_assert!(_rejected);
        prop_assert_eq!(eng.state_hash(), h_before);
        prop_assert_eq!(eng.state().revision, 1);
    }

    /// §37.4 #4: A rejected command triggers internal rollback; the state hash
    /// must equal the pre-command hash.
    #[test]
    fn rollback_restores_hash(seed in any_seed()) {
        let mut eng2 = Engine::new();
        let h_empty = eng2.state_hash();
        let r = eng2.submit(Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        });
        let _rejected = matches!(r, CommandResult::Rejected { .. }); prop_assert!(_rejected);
        prop_assert_eq!(eng2.state_hash(), h_empty);
    }

    /// §37.4 #5: A dead player cannot resurrect by acting.
    #[test]
    fn dead_player_cannot_resurrect(seed in any_seed()) {
        let mut eng = Engine::new();
        eng.submit(start_cmd(seed, 4));
        let p2 = PlayerId::new(2);
        eng.submit(Command {
            actor: p2,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::Surrender,
        });
        let p2_alive = eng.state().players.get(p2).map(|p| p.alive).unwrap_or(false);
        if !p2_alive {
            eng.submit(Command {
                actor: p2,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0),
                kind: CommandKind::EndPhase,
            });
            let still_dead = !eng.state().players.get(p2).map(|p| p.alive).unwrap_or(true);
            prop_assert!(still_dead, "dead player must not resurrect");
        }
    }

    /// §37.4 #6: Disabling the query cache must not change query results.
    #[test]
    fn cache_disabled_query_unchanged(seed in any_seed(), n in any_player_count()) {
        let mut eng = Engine::new();
        eng.submit(start_cmd(seed, n));
        let p1 = PlayerId::new(1);
        let p2 = PlayerId::new(2);
        let mut ctx_cache = QueryContext::with_cache(true);
        let with_cache = query(
            eng.state(),
            &QueryRequest::Distance { from: p1, to: p2 },
            &mut ctx_cache,
        );
        let mut ctx_nocache = QueryContext::with_cache(false);
        let without_cache = query(
            eng.state(),
            &QueryRequest::Distance { from: p1, to: p2 },
            &mut ctx_nocache,
        );
        prop_assert_eq!(with_cache, without_cache);
    }

    /// §37.4 #7: Snapshot restore == full replay from the command log.
    #[test]
    fn snapshot_restore_equals_replay(seed in any_seed(), n in any_player_count()) {
        let mut eng = Engine::new();
        eng.submit(start_cmd(seed, n));
        for _ in 0..3 {
            eng.submit(Command {
                actor: PlayerId::new(1),
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0),
                kind: CommandKind::EndPhase,
            });
        }
        let live_hash = eng.state_hash();
        let snap = eng.snapshot();
        let mut restored = Engine::new();
        restored.restore_snapshot(&snap).unwrap();
        prop_assert_eq!(restored.state_hash(), live_hash);
        let commands: Vec<Command> = eng
            .replay_log()
            .commands
            .iter()
            .map(|c| c.command.clone())
            .collect();
        let replayed = Engine::replay_from(&commands).unwrap();
        prop_assert_eq!(replayed.state_hash(), live_hash);
    }

    /// §37.4 #8: Observer view for player A must not reveal card faces of
    /// player B's hand cards (architecture §21 anti-leak invariant).
    #[test]
    fn observer_view_hides_foreign_hands(seed in any_seed(), n in any_player_count()) {
        let mut eng = Engine::new();
        eng.submit(start_cmd(seed, n));
        let p1 = PlayerId::new(1);
        let view = project_observer(eng.state(), ObserverKind::Player(p1));
        for p in eng.state().players.sorted_live() {
            if p.id == p1 {
                continue;
            }
            let foreign_hand = eng.state().hand_cards(p.id);
            for cid in &foreign_hand {
                let leaked = view.known_cards.iter().any(|c| c.id == *cid && c.def.is_some());
                prop_assert!(!leaked, "foreign hand face leaked");
            }
        }
    }

    /// §37.4 #9: An expired/invalid Choice must not change state.
    #[test]
    fn invalid_choice_zero_state(seed in any_seed(), n in any_player_count()) {
        let mut eng = Engine::new();
        eng.submit(start_cmd(seed, n));
        let h_before = eng.state_hash();
        let r = eng.submit(Command {
            actor: PlayerId::new(1),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::ResolveChoice {
                choice_id: engine_api::ChoiceId::new(99999),
                selections: vec![],
                cancelled: false,
            },
        });
        let _rejected = matches!(r, CommandResult::Rejected { .. }); prop_assert!(_rejected);
        prop_assert_eq!(eng.state_hash(), h_before);
    }

    /// §37.4 #10: Determinism — same accepted command sequence → same final hash.
    #[test]
    fn dual_engine_determinism(seed in any_seed(), n in any_player_count()) {
        let cmds = vec![
            start_cmd(seed, n),
            Command {
                actor: PlayerId::new(1),
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0),
                kind: CommandKind::EndPhase,
            },
        ];
        let a = Engine::replay_from(&cmds).unwrap();
        let b = Engine::replay_from(&cmds).unwrap();
        prop_assert_eq!(a.state_hash(), b.state_hash());
    }
}

/// §37.6 Metamorphic: non-semantic fields (idempotency key) must not change
/// the canonical state hash.
#[test]
fn metamorphic_nonsemantic_field_invariance() {
    let mut a = Engine::new();
    let mut b = Engine::new();
    a.submit(start_cmd(42, 4));
    b.submit(start_cmd(42, 4));
    a.submit(Command {
        actor: PlayerId::new(1),
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(111),
        kind: CommandKind::EndPhase,
    });
    b.submit(Command {
        actor: PlayerId::new(1),
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(222),
        kind: CommandKind::EndPhase,
    });
    assert_eq!(a.state_hash(), b.state_hash());
}
