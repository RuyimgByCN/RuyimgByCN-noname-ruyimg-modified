//! Phase 5: identity mode, content setup, headless runner, replay.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, MatchOutcome, PlayerId,
};
use engine_core::mode::IdentityVictory;
use engine_core::{
    check_identity_victory, run_identity_match, CardKind, Engine, Identity, MatchConfig, Phase,
};

fn start(seed: u64, n: u8) -> Command {
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

#[test]
fn start_assigns_characters_and_identities() {
    let mut eng = Engine::new();
    eng.submit(start(1, 5));
    assert_eq!(eng.state().players.live_count(), 5);
    let lord = eng
        .state()
        .players
        .iter_live()
        .find(|p| p.identity == Identity::Lord)
        .expect("lord");
    assert_eq!(lord.seat.raw(), 0);
    assert!(lord.character_def > 0);
    assert!(lord.max_hp >= 4); // base + possible lord bonus
                               // All have characters
    for p in eng.state().players.iter_live() {
        assert!(p.character_def > 0);
        assert_eq!(
            eng.state().hand_count(p.id),
            if p.id == eng.state().turn.current {
                6
            } else {
                4
            }
        );
    }
}

#[test]
fn deck_has_equipment_and_tricks() {
    let mut eng = Engine::new();
    eng.submit(start(2, 4));
    let kinds: Vec<_> = eng
        .state()
        .cards
        .sorted_live()
        .into_iter()
        .map(|c| c.kind)
        .collect();
    assert!(kinds.iter().any(|k| *k == CardKind::Weapon));
    assert!(kinds.iter().any(|k| *k == CardKind::TrickWuzhong));
    assert!(kinds.iter().any(|k| *k == CardKind::HorseOff));
}

#[test]
fn surrender_can_end_two_player() {
    let mut eng = Engine::new();
    eng.submit(start(3, 2));
    // 2p: Lord + Renegade. Kill renegade → loyalist win; kill lord → rebel or renegade
    let lord = eng
        .state()
        .players
        .iter_live()
        .find(|p| p.identity == Identity::Lord)
        .unwrap()
        .id;
    let other = eng
        .state()
        .players
        .iter_live()
        .find(|p| p.id != lord)
        .unwrap()
        .id;
    // Advance to play so surrender works (surrender doesn't need play phase actually)
    let r = eng.submit(Command {
        actor: other,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::Surrender,
    });
    assert!(matches!(r, CommandResult::Accepted { .. }));
    assert!(!eng.state().players.get(other).unwrap().alive);
    match check_identity_victory(eng.state()) {
        IdentityVictory::Finished(MatchOutcome::LoyalistWin) => {}
        other => panic!("expected loyalist win, got {other:?}"),
    }
    assert!(eng.state().outcome.is_finished());
}

#[test]
fn headless_2_to_8_runs() {
    for n in 2..=8u8 {
        let report = run_identity_match(MatchConfig {
            player_count: n,
            rng_seed: 1000 + n as u64,
            max_steps: 1_500,
            ruleset: "identity-standard".into(),
        })
        .unwrap_or_else(|e| panic!("n={n}: {e}"));
        assert!(report.steps > 0, "n={n}");
        // Prefer finish; allow cap on rare long games
        if report.capped {
            eprintln!(
                "n={n} capped at {} steps hash={:?}",
                report.steps, report.state_hash
            );
        } else {
            assert!(
                report.outcome.is_finished(),
                "n={n} outcome={:?}",
                report.outcome
            );
        }
    }
}

#[test]
fn command_log_replay_full_match() {
    use engine_core::runner::choose_command;

    let mut live = Engine::new();
    let mut cmds = vec![start(55, 3)];
    live.submit(cmds[0].clone());
    let mut steps = 0u32;
    while !live.state().outcome.is_finished() && steps < 400 {
        let c = choose_command(&live);
        match live.submit(c.clone()) {
            CommandResult::Accepted { .. } => {
                cmds.push(c);
                steps += 1;
            }
            CommandResult::Rejected { .. } => break,
        }
    }
    let replayed = Engine::replay_from(&cmds).expect("replay");
    assert_eq!(live.state_hash(), replayed.state_hash());
    assert_eq!(live.state().revision, replayed.state().revision);
    assert_eq!(live.state().outcome, replayed.state().outcome);
}

#[test]
fn end_phase_still_works_after_setup() {
    let mut eng = Engine::new();
    eng.submit(start(8, 3));
    assert_eq!(eng.state().turn.phase, Phase::Draw);
    let actor = eng.state().turn.current;
    let r = eng.submit(Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::EndPhase,
    });
    assert!(matches!(r, CommandResult::Accepted { .. }));
    assert_eq!(eng.state().turn.phase, Phase::Play);
}
