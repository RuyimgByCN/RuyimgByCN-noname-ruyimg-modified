use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};

static DEDUP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
fn fresh() -> u64 {
    DEDUP.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn main() {
    let mut e = Engine::new(467, 4);
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
    e.debug_grant(1, "遗计");
    let out = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    println!("damage -> {out:?}");
    // gate
    let c = e.open_choice().unwrap();
    println!("gate: {} obs={}", c.spec.prompt, c.observer);
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    println!("confirm -> {out:?}");
    let c = e.open_choice().unwrap();
    println!(
        "ask1: {} obs={} hand1={:?}",
        c.spec.prompt,
        c.observer,
        e.debug_hand(1)
    );
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Player(0)],
        },
    });
    println!("target -> {out:?}");
    let c = e.open_choice().unwrap();
    println!("ask2: {}", c.spec.prompt);
    let card = match c.spec.options.first() {
        Some(ChoiceOption::Card(x)) => *x,
        _ => 0,
    };
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Card(card)],
        },
    });
    println!("card -> {out:?} hand1={:?}", e.debug_hand(1));
    match e.open_choice() {
        Some(c) => println!("ask3: {} obs={}", c.spec.prompt, c.observer),
        None => println!("ask3: NONE"),
    }
    for l in e.log().iter().rev().take(6) {
        println!("log: {l:?}");
    }
}
