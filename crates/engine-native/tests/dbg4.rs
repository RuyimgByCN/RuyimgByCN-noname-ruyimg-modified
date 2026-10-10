use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn submit_cmd(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let expected_revision = e.revision();
    e.submit(GameCommand {
        actor,
        expected_revision,
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

#[test]
fn dbg_nullify() {
    let mut e = Engine::new(151, 4);
    for _ in 0..64 {
        let actor = e.turn().current;
        if actor == 0 && e.turn().phase == Phase::Play { break; }
        match submit_cmd(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                let sel: Vec<engine_native::ChoiceOption> = choice.spec.options.iter().filter(|o| matches!(o, engine_native::ChoiceOption::Card(_))).copied().take(choice.spec.min.max(1)).collect();
                let actor = choice.observer;
                let cid = choice.id;
                let _ = submit_cmd(&mut e, actor, GameAction::Answer { choice_id: cid, selection: sel });
            }
            o => panic!("{o:?}"),
        }
    }
    e.debug_clear_hand(0);
    for p in [1u8, 2, 3] { e.debug_clear_hand(p); }
    let savage = e.debug_place(0, "savage");
    let nul1 = e.debug_place(1, "nullify");
    println!("savage={savage} nul1={nul1}");
    let out = submit_cmd(&mut e, 0, GameAction::PlayCard { card: savage, targets: vec![] });
    println!("play -> {out:?}");
    println!("hps={:?} hand1={:?}", (0..4).map(|p| e.debug_hp(p)).collect::<Vec<_>>(), e.debug_hand(1));
    for l in e.log() {
        if let engine_native::LogEvent::DamageDealt { target, amount, .. } = l {
            println!("dmg target={target} amt={amount}");
        }
    }
}
