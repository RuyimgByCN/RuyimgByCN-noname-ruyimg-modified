use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};

static DEDUP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
fn fresh() -> u64 {
    DEDUP.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn main() {
    let mut e = Engine::new(241, 4);
    // advance_to_play(0)
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
    let _sd = e.debug_place_judge(1, "shandian");
    let t: Vec<u32> = (0..3).map(|_| e.debug_place(1, "tao")).collect();
    e.debug_set_deck_top_by(|d| d.suit.is_spade() && (2..=9).contains(&d.rank));
    loop {
        if e.open_choice().is_some() {
            break;
        }
        let a = e.turn().current;
        let out = e.submit(GameCommand {
            actor: a,
            expected_revision: e.revision(),
            dedup_id: fresh(),
            action: GameAction::NextPhase,
        });
        match out {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    println!("hp after hit: {}", e.debug_hp(1));
    for i in 0..3 {
        let Some(c) = e.open_choice() else {
            println!("ask {i}: NO CHOICE, hp={}", e.debug_hp(1));
            break;
        };
        println!("ask {i}: observer={} prompt={}", c.observer, c.spec.prompt);
        let Some(ChoiceOption::Card(tao)) = c
            .spec
            .options
            .iter()
            .copied()
            .find(|o| matches!(o, ChoiceOption::Card(id) if t.contains(&id)))
        else {
            println!("no tao option: {:?}", c.spec.options);
            break;
        };
        let out = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: fresh(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Card(tao)],
            },
        });
        println!("  answer -> {out:?}, hp now {}", e.debug_hp(1));
    }
    for l in e.log().iter().rev().take(8) {
        println!("log: {l:?}");
    }
}
