//! NC10: multiple hidden traps remain private and physical in AI simulations.
use engine_native::ai::{redact_opponent_hands, BoundedPlanner};
use engine_native::content::extension::ExtensionPack;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};

fn pack() -> ExtensionPack {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json = std::fs::read_to_string(
        root.join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json"),
    )
    .unwrap();
    content_native_loader::bridge_pack(&json).unwrap().pack
}

fn fixture(seed: u64, pack: &ExtensionPack) -> Engine {
    let mut engine = Engine::new_with_extensions(seed, 4, &[pack]).unwrap();
    engine.debug_set_turn(0, Phase::Play, 1);
    for (owner, count) in [1, 2, 1, 0].into_iter().enumerate() {
        for _ in 0..count {
            let index = engine
                .state_core()
                .deck
                .iter()
                .position(|card| {
                    let suit = engine.catalog().cards[*card as usize].suit;
                    engine.state_core().players[owner]
                        .sxianjing_traps
                        .iter()
                        .all(|existing| engine.catalog().cards[*existing as usize].suit != suit)
                })
                .unwrap();
            let card = engine.state_core_mut().deck.remove(index);
            engine.state_core_mut().players[owner]
                .sxianjing_traps
                .push(card);
        }
        if count > 0 {
            engine.debug_grant(owner as u8, "sxianjing_gain");
        }
    }
    engine
}

fn alternate(mut engine: Engine) -> Engine {
    for (owner, trap) in [(1, 0), (1, 1), (2, 0)] {
        let id = engine.state_core().players[owner].sxianjing_traps[trap];
        let suit = engine.catalog().cards[id as usize].suit;
        let index = engine
            .state_core()
            .deck
            .iter()
            .position(|card| engine.catalog().cards[*card as usize].suit == suit)
            .unwrap();
        let core = engine.state_core_mut();
        std::mem::swap(
            &mut core.players[owner].sxianjing_traps[trap],
            &mut core.deck[index],
        );
    }
    engine
}

#[test]
fn multiple_traps_stay_physical_from_each_observer() {
    let pack = pack();
    let engine = fixture(3900, &pack);
    let original_hash = engine.state_hash();
    let mut errors = Vec::new();
    for observer in 0..4 {
        let mut sim = engine.clone();
        redact_opponent_hands(&mut sim, observer);
        assert_eq!(
            sim.state_core().player(observer).sxianjing_traps,
            engine.state_core().player(observer).sxianjing_traps,
            "observer must retain their own traps"
        );
        for owner in 0..4 {
            assert_eq!(
                sim.state_core().player(owner).sxianjing_traps.len(),
                engine.state_core().player(owner).sxianjing_traps.len()
            );
        }
        if let Err(error) = Engine::restore_with_extensions(&sim.save(), &[&pack]) {
            errors.push(format!("observer {observer}: {error:?}"));
        }
    }
    assert!(
        errors.is_empty(),
        "invalid simulation ownership: {errors:?}"
    );
    assert_eq!(engine.state_hash(), original_hash);
}

#[test]
fn multiple_hidden_traps_do_not_change_simulation_or_plans() {
    let pack = pack();
    for seed in 3900..3996 {
        let left = fixture(seed, &pack);
        let right = alternate(left.clone());
        assert_eq!(left.ai_session(0).view(), right.ai_session(0).view());
        for engine in [&left, &right] {
            assert!(Engine::restore_with_extensions(&engine.save(), &[&pack]).is_ok());
        }
        let mut a = left.clone();
        let mut b = right.clone();
        redact_opponent_hands(&mut a, 0);
        redact_opponent_hands(&mut b, 0);
        assert!(
            a.state_core() == b.state_core(),
            "seed {seed}: hidden traps leaked"
        );
        assert!(Engine::restore_with_extensions(&a.save(), &[&pack]).is_ok());
        assert!(Engine::restore_with_extensions(&b.save(), &[&pack]).is_ok());
        let traps = &a.state_core().player(1).sxianjing_traps;
        assert_ne!(
            a.catalog().cards[traps[0] as usize].suit,
            a.catalog().cards[traps[1] as usize].suit,
            "seed {seed}: a valid trap pile cannot repeat suits"
        );
        for budget in [0, 48] {
            let planner = BoundedPlanner::new(budget)
                .with_policy_variant(1)
                .with_lookahead(2);
            let plan = left.plan_action(0, &planner);
            assert_eq!(
                plan.action,
                right.plan_action(0, &planner).action,
                "seed {seed}, budget {budget}"
            );
            for source in [&left, &right] {
                let mut original = source.clone();
                let mut restored =
                    Engine::restore_with_extensions(&source.save(), &[&pack]).unwrap();
                assert!(!matches!(
                    original.apply_plan(&plan).unwrap(),
                    CommandOutcome::Rejected(_)
                ));
                assert!(!matches!(
                    restored.apply_plan(&plan).unwrap(),
                    CommandOutcome::Rejected(_)
                ));
                assert_eq!(original.state_hash(), restored.state_hash());
            }
        }
    }
}

fn drive_to_play(engine: &mut Engine, seat: u8) {
    for _ in 0..128 {
        if engine.turn().current == seat
            && engine.turn().phase == Phase::Play
            && engine.open_choice().is_none()
        {
            return;
        }
        let (actor, action) = if let Some(choice) = engine.open_choice().cloned() {
            (
                choice.observer,
                GameAction::Answer {
                    choice_id: choice.id,
                    selection: choice
                        .spec
                        .options
                        .iter()
                        .take(choice.spec.min)
                        .copied()
                        .collect(),
                },
            )
        } else {
            (engine.turn().current, GameAction::NextPhase)
        };
        let outcome = engine.submit(GameCommand {
            actor,
            expected_revision: engine.revision(),
            dedup_id: engine.revision() + 1,
            action,
        });
        assert!(
            !matches!(outcome, CommandOutcome::Rejected(_)),
            "{outcome:?}"
        );
    }
    panic!("could not reach seat {seat} play phase");
}

#[test]
fn real_skill_places_two_traps_and_simulation_remains_restorable() {
    let pack = pack();
    let mut engine = (3900..4000)
        .map(|seed| Engine::new_with_extensions(seed, 4, &[&pack]).unwrap())
        .find(|engine| {
            let hand = &engine.state_core().player(1).hand;
            hand.iter().any(|a| {
                hand.iter().any(|b| {
                    engine.catalog().cards[*a as usize].suit
                        != engine.catalog().cards[*b as usize].suit
                })
            })
        })
        .expect("fixture must have two suits");
    engine.assign_general_named(1, "swd_shuwaner").unwrap();
    engine.debug_set_turn(1, Phase::Play, 1);
    let skill = engine.catalog().skill_by_name("sxianjing").unwrap();
    for index in 0..2 {
        if index == 1 {
            let result = engine.submit(GameCommand {
                actor: 1,
                expected_revision: engine.revision(),
                dedup_id: engine.revision() + 1,
                action: GameAction::NextPhase,
            });
            assert!(!matches!(result, CommandOutcome::Rejected(_)), "{result:?}");
            drive_to_play(&mut engine, 1);
        }
        let result = engine.submit(GameCommand {
            actor: 1,
            expected_revision: engine.revision(),
            dedup_id: engine.revision() + 1,
            action: GameAction::ActivateSkill { skill },
        });
        assert!(!matches!(result, CommandOutcome::Rejected(_)), "{result:?}");
        let choice = engine.open_choice().cloned().expect("trap choice");
        assert_eq!((choice.observer, choice.spec.prompt), (1, "sxianjing_pick"));
        let card = choice
            .spec
            .options
            .iter()
            .find_map(|option| match option {
                ChoiceOption::Card(card) => Some(*card),
                _ => None,
            })
            .unwrap();
        let result = engine.submit(GameCommand {
            actor: 1,
            expected_revision: engine.revision(),
            dedup_id: engine.revision() + 1,
            action: GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Card(card)],
            },
        });
        assert!(!matches!(result, CommandOutcome::Rejected(_)), "{result:?}");
    }
    assert_eq!(engine.state_core().player(1).sxianjing_traps.len(), 2);
    drive_to_play(&mut engine, 0);
    for observer in 0..4 {
        let mut sim = engine.clone();
        redact_opponent_hands(&mut sim, observer);
        assert!(
            Engine::restore_with_extensions(&sim.save(), &[&pack]).is_ok(),
            "observer {observer}"
        );
        let traps = &sim.state_core().player(1).sxianjing_traps;
        assert_ne!(
            sim.catalog().cards[traps[0] as usize].suit,
            sim.catalog().cards[traps[1] as usize].suit,
            "observer {observer}: a valid trap pile cannot repeat suits"
        );
        if observer == 1 {
            assert_eq!(
                sim.state_core().player(1).sxianjing_traps,
                engine.state_core().player(1).sxianjing_traps
            );
        }
    }
}

#[test]
fn references_follow_multiple_synthetic_traps() {
    let pack = pack();
    let left = fixture(3900, &pack);
    let mut pair = [left.clone(), alternate(left)];
    for engine in &mut pair {
        let core = engine.state_core_mut();
        let first = core.players[1].sxianjing_traps[0];
        let second = core.players[1].sxianjing_traps[1];
        let third = core.players[2].sxianjing_traps[0];
        core.vm_aux.lost.insert(1, vec![first, second, third]);
        core.turn.huanxia_track.push((second, false));
        core.turn
            .kunlunjing_snapshot
            .push((1, vec![third], vec![], vec![first]));
        core.players[1].card_mods.insert(first, second);
        assert!(Engine::restore_with_extensions(&engine.save(), &[&pack]).is_ok());
        redact_opponent_hands(engine, 0);
        let core = engine.state_core();
        let first = core.players[1].sxianjing_traps[0];
        let second = core.players[1].sxianjing_traps[1];
        let third = core.players[2].sxianjing_traps[0];
        assert_eq!(core.vm_aux.lost[&1], vec![first, second, third]);
        assert_eq!(core.turn.huanxia_track, vec![(second, false)]);
        assert_eq!(
            core.turn.kunlunjing_snapshot,
            vec![(1, vec![third], vec![], vec![first])]
        );
        assert_eq!(core.players[1].card_mods.get(&first), Some(&second));
        assert!(Engine::restore_with_extensions(&engine.save(), &[&pack]).is_ok());
    }
    assert!(pair[0].state_core() == pair[1].state_core());
}

#[test]
fn scarce_suits_are_shared_across_trap_holders() {
    use engine_native::catalog::Suit;
    let pack = pack();
    let mut engine = Engine::new_with_extensions(3900, 4, &[&pack]).unwrap();
    engine.debug_set_turn(0, Phase::Play, 1);
    for (owner, suits) in [
        (1usize, vec![Suit::Spade, Suit::Heart, Suit::Club]),
        (2usize, vec![Suit::Spade, Suit::Heart]),
    ] {
        for suit in suits {
            let index = engine
                .state_core()
                .deck
                .iter()
                .position(|card| engine.catalog().cards[*card as usize].suit == suit)
                .unwrap();
            let card = engine.state_core_mut().deck.remove(index);
            engine.state_core_mut().players[owner]
                .sxianjing_traps
                .push(card);
        }
    }
    let core = engine.state_core_mut();
    core.discard.extend(std::mem::take(&mut core.deck));
    for owner in 1..4 {
        core.discard
            .extend(std::mem::take(&mut core.players[owner].hand));
    }
    assert!(Engine::restore_with_extensions(&engine.save(), &[&pack]).is_ok());
    redact_opponent_hands(&mut engine, 0);
    assert!(Engine::restore_with_extensions(&engine.save(), &[&pack]).is_ok());
    for owner in [1, 2] {
        let cards = &engine.state_core().players[owner].sxianjing_traps;
        for (index, card) in cards.iter().enumerate() {
            for other in &cards[..index] {
                assert_ne!(
                    engine.catalog().cards[*card as usize].suit,
                    engine.catalog().cards[*other as usize].suit
                );
            }
        }
    }
}
