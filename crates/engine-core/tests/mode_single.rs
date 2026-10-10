//! Golden for M4 单挑 (single) mode: strict 1v1, the last living player wins.
//!
//! Verifies the ruleset validation (exactly 2 players), the neutral-identity
//! bootstrap (no lord bonus), and the kill-path victory dispatch producing
//! `MatchOutcome::SingleWin`.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, MatchOutcome,
    PlayerId,
};
use engine_core::effect::run_effects;
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

fn single_engine() -> Engine {
    let mut engine = Engine::new();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_261_001,
            ruleset: "single".into(),
        },
    });
    assert!(
        matches!(result, CommandResult::Accepted { .. }),
        "single ruleset start must be accepted: {result:?}"
    );
    engine
}

#[test]
fn single_ruleset_starts_two_neutral_players() {
    let engine = single_engine();
    let state = engine.state();
    assert_eq!(state.ruleset.name, "single");
    assert_eq!(state.players.live_count(), 2);
    // No lord bonus: both players use their character's base max HP (equal).
    let max_hp = state.players.iter_live().map(|player| player.max_hp).collect::<Vec<_>>();
    assert_eq!(max_hp[0], max_hp[1], "single mode grants no lord HP bonus");
    assert_eq!(max_hp[0], state.players.iter_live().next().unwrap().max_hp);
    // No 主公技: nobody carries the Lord identity.
    for player in state.players.iter_live() {
        assert_ne!(
            player.identity,
            engine_core::state::Identity::Lord,
            "单挑 has no lord"
        );
    }
}

#[test]
fn single_requires_exactly_two_players() {
    let mut engine = Engine::new();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(1),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 20_261_002,
            ruleset: "single".into(),
        },
    });
    assert!(
        matches!(result, CommandResult::Rejected { .. }),
        "single mode rejects non-2 player counts"
    );
}

#[test]
fn killing_the_opponent_ends_the_match_with_single_win() {
    let engine = single_engine();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());

    // Clear both hands so the kill discard is trivial, then kill the opponent.
    for player in [owner, other] {
        let hand = tx.state().zones.find(engine_core::state::ZoneKind::Hand, player).unwrap();
        let cards = tx.state().hand_cards(player);
        if !cards.is_empty() {
            let discard = tx
                .state()
                .zones
                .find(engine_core::state::ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap();
            run_effects(
                &mut tx,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(hand),
                }],
            )
            .unwrap();
        }
    }

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 99,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap_or_else(|e| panic!("kill damage error: {e:?}"));

    let finished = tx.state().outcome.clone();
    assert_eq!(
        finished,
        MatchOutcome::SingleWin { player: owner },
        "single mode ends with the survivor winning"
    );
    assert!(
        events.iter().any(|event| matches!(
            event,
            DomainEvent::MatchEnded { outcome } if *outcome == MatchOutcome::SingleWin { player: owner }
        )),
        "kill path emits the MatchEnded event"
    );
}

#[test]
fn match_stays_in_progress_while_both_alive() {
    let engine = single_engine();
    assert_eq!(engine.state().outcome, MatchOutcome::InProgress);
    let mut state: GameState = engine.state().clone();
    // Simulate an unrelated event (deal non-lethal damage); match continues.
    let mut tx = Transaction::begin(state);
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // Deal 1 damage (may open a passive choice window; ignore it) — match continues.
    let damage_result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    match damage_result {
        Ok(_) => {}
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("unexpected damage error: {e:?}"),
    }
    state = tx.commit().state;
    assert_eq!(state.outcome, MatchOutcome::InProgress);
}
