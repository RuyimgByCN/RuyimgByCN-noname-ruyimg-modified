use engine_native::{
    CardKind, ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};

static DEDUP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
fn fresh() -> u64 {
    DEDUP.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn main() {
    for seed in 9u64..=60 {
        let mut e = match Engine::try_new_identity(seed * 1000, 5) {
            Ok(e) => e,
            Err(_) => continue,
        };
        let mut cmds = 0;
        let mut converged = true;
        while e.outcome().is_none() {
            cmds += 1;
            if cmds > 6000 {
                converged = false;
                break;
            }
            let out = if let Some(c) = e.open_choice() {
                let sel: Vec<ChoiceOption> = c
                    .spec
                    .options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .take(c.spec.min.max(1))
                    .collect();
                e.submit(GameCommand {
                    actor: c.observer,
                    expected_revision: e.revision(),
                    dedup_id: fresh(),
                    action: GameAction::Answer {
                        choice_id: c.id,
                        selection: sel,
                    },
                })
            } else if e.turn().phase == Phase::Play {
                let actor = e.turn().current;
                match e
                    .debug_hand(actor)
                    .iter()
                    .copied()
                    .find(|c| e.catalog().card_kind(*c) == CardKind::Sha)
                {
                    Some(sha) => {
                        let target = e.state_core().next_alive(actor);
                        e.submit(GameCommand {
                            actor,
                            expected_revision: e.revision(),
                            dedup_id: fresh(),
                            action: GameAction::PlayCard {
                                card: sha,
                                targets: vec![target],
                            },
                        })
                    }
                    None => e.submit(GameCommand {
                        actor,
                        expected_revision: e.revision(),
                        dedup_id: fresh(),
                        action: GameAction::NextPhase,
                    }),
                }
            } else {
                let actor = e.turn().current;
                e.submit(GameCommand {
                    actor,
                    expected_revision: e.revision(),
                    dedup_id: fresh(),
                    action: GameAction::NextPhase,
                })
            };
            match out {
                CommandOutcome::Rejected(_) => {
                    let a = e.turn().current;
                    let _ = e.submit(GameCommand {
                        actor: a,
                        expected_revision: e.revision(),
                        dedup_id: fresh(),
                        action: GameAction::NextPhase,
                    });
                }
                _ => {}
            }
        }
        if converged {
            if let Some(o) = e.outcome() {
                println!("seed {} -> {:?}", seed, o.winner);
            }
        }
    }
}
