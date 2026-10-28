//! NC10: an opponent's single hidden trap must not affect AI information or card ownership.
use engine_native::ai::{redact_opponent_hands, BoundedPlanner};
use engine_native::content::extension::ExtensionPack;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};

fn submit(e: &mut Engine, actor: u8, action: GameAction) {
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: e.revision() + 1,
        action,
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
}

fn drive_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..96 {
        if e.turn().current == seat && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            return;
        }
        if let Some(c) = e.open_choice().cloned() {
            let selection = c.spec.options.iter().take(c.spec.min).copied().collect();
            submit(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection,
                },
            );
        } else {
            submit(e, e.turn().current, GameAction::NextPhase);
        }
    }
    panic!("could not reach seat {seat} play phase");
}

fn fixture(seed: u64) -> (Engine, ExtensionPack) {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json = std::fs::read_to_string(
        root.join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json"),
    )
    .unwrap();
    let pack = content_native_loader::bridge_pack(&json).unwrap().pack;
    let mut e = Engine::new_with_extensions(seed, 4, &[&pack]).unwrap();
    e.assign_general_named(1, "swd_shuwaner").unwrap();
    drive_to_play(&mut e, 1);
    let skill = e.catalog().skill_by_name("sxianjing").unwrap();
    submit(&mut e, 1, GameAction::ActivateSkill { skill });
    let choice = e.open_choice().cloned().expect("actual trap choice");
    assert_eq!((choice.observer, choice.spec.prompt), (1, "sxianjing_pick"));
    let card = choice
        .spec
        .options
        .iter()
        .find_map(|o| {
            if let ChoiceOption::Card(c) = o {
                Some(*c)
            } else {
                None
            }
        })
        .unwrap();
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: choice.id,
            selection: vec![ChoiceOption::Card(card)],
        },
    );
    assert_eq!(e.state_core().player(1).sxianjing_traps, vec![card]);
    drive_to_play(&mut e, 0);
    assert_eq!(e.state_core().player(1).sxianjing_traps, vec![card]);
    (e, pack)
}

fn alternate(mut e: Engine) -> Engine {
    let trap = e.state_core().player(1).sxianjing_traps[0];
    let suit = e.catalog().cards[trap as usize].suit;
    let index = e
        .state_core()
        .deck
        .iter()
        .position(|id| e.catalog().cards[*id as usize].suit != suit)
        .unwrap();
    let core = e.state_core_mut();
    std::mem::swap(
        &mut core.players[1].sxianjing_traps[0],
        &mut core.deck[index],
    );
    e
}

#[test]
fn real_single_opponent_trap_redacts_without_duplicate_cards() {
    let (engine, pack) = fixture(3900);
    let mut failures = Vec::new();
    for observer in [0, 2, 3] {
        let mut sim = engine.clone();
        redact_opponent_hands(&mut sim, observer);
        assert_eq!(sim.state_core().player(1).sxianjing_traps.len(), 1);
        if let Err(error) = Engine::restore_with_extensions(&sim.save(), &[&pack]) {
            failures.push(format!("observer {observer}: {error:?}"));
        }
    }
    assert!(failures.is_empty(), "redacted trap ownership: {failures:?}");
}

#[test]
fn single_opponent_trap_identity_cannot_change_planning() {
    let (left, pack) = fixture(3900);
    let right = alternate(left.clone());
    assert_eq!(left.ai_session(0).view(), right.ai_session(0).view());
    assert!(Engine::restore_with_extensions(&left.save(), &[&pack]).is_ok());
    assert!(Engine::restore_with_extensions(&right.save(), &[&pack]).is_ok());
    let mut a = left.clone();
    let mut b = right.clone();
    redact_opponent_hands(&mut a, 0);
    redact_opponent_hands(&mut b, 0);
    assert_eq!(
        a.state_core(),
        b.state_core(),
        "synthetic states depend on hidden trap"
    );
    for variant in 0..=2 {
        for budget in [0, 1, 48] {
            let planner = BoundedPlanner::new(budget)
                .with_policy_variant(variant)
                .with_lookahead(2);
            let x = left.plan_action(0, &planner);
            let y = right.plan_action(0, &planner);
            assert_eq!(x.action, y.action, "variant={variant}, budget={budget}");
        }
    }
}
