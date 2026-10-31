use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};

static DEDUP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
fn fresh() -> u64 {
    DEDUP.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn main() {
    let mut e = Engine::new(311, 4);
    for _ in 0..64 {
        let t = e.turn();
        if t.current == 0 && t.phase == engine_native::Phase::Play {
            break;
        }
        let actor = t.current;
        let out = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: fresh(),
            action: GameAction::NextPhase,
        });
        if let CommandOutcome::Pending { choice } = &out {
            let sel: Vec<ChoiceOption> = choice
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(choice.spec.min.max(1))
                .collect();
            let _ = e.submit(GameCommand {
                actor: choice.observer,
                expected_revision: e.revision(),
                dedup_id: fresh(),
                action: GameAction::Answer {
                    choice_id: choice.id,
                    selection: sel,
                },
            });
        }
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 1);
    let sd1 = e.debug_place_judge(1, "shandian");
    let sd2 = e.debug_place_judge(1, "shandian");
    e.debug_set_deck_top_by(|d| d.suit.is_spade() && (2..=9).contains(&d.rank));
    loop {
        if e.log()
            .iter()
            .any(|l| matches!(l, engine_native::LogEvent::PlayerDied { player: 1, .. }))
        {
            break;
        }
        if e.open_choice().is_some() {
            break;
        }
        let a = e.turn().current;
        let _ = e.submit(GameCommand {
            actor: a,
            expected_revision: e.revision(),
            dedup_id: fresh(),
            action: GameAction::NextPhase,
        });
    }
    for _ in 0..24 {
        if e.outcome().is_some() {
            break;
        }
        let out = e.submit(GameCommand {
            actor: e.turn().current,
            expected_revision: e.revision(),
            dedup_id: fresh(),
            action: GameAction::NextPhase,
        });
        match out {
            CommandOutcome::Pending { choice } => {
                let sel: Vec<ChoiceOption> = choice
                    .spec
                    .options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .take(choice.spec.min.max(1))
                    .collect();
                let _ = e.submit(GameCommand {
                    actor: choice.observer,
                    expected_revision: e.revision(),
                    dedup_id: fresh(),
                    action: GameAction::Answer {
                        choice_id: choice.id,
                        selection: sel,
                    },
                });
            }
            CommandOutcome::Rejected(_) => break,
            _ => {}
        }
    }
    println!("alive1={} judge1={:?}", e.debug_alive(1), e.debug_judge(1));
    println!(
        "discard sd1={} sd2={}",
        e.state_core().discard.iter().filter(|c| **c == sd1).count(),
        e.state_core().discard.iter().filter(|c| **c == sd2).count()
    );
    let core = e.state_core();
    println!(
        "sd2 in deck={} table={}",
        core.deck.contains(&sd2),
        core.table.contains(&sd2)
    );
    for (i, p) in core.players.iter().enumerate() {
        if p.judge.contains(&sd2) {
            println!("sd2 in p{i} judge");
        }
    }
    for park in e.save().resolution.parks {
        println!("park: {:?}", park.machine);
    }
}
