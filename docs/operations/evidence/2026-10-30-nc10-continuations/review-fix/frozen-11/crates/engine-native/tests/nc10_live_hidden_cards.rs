//! NC10: unknown physical cards in VM piles and live response windows.
use engine_native::ai::{redact_opponent_hands, BoundedPlanner};
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};

fn submit(e: &mut Engine, actor: u8, action: GameAction) {
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action,
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
}

fn assert_pair(left: Engine, right: Engine, observer: u8) {
    assert_pair_with_packs(left, right, observer, &[]);
}

fn assert_pair_with_packs(
    left: Engine,
    right: Engine,
    observer: u8,
    packs: &[&engine_native::content::extension::ExtensionPack],
) {
    assert_eq!(
        format!("{:?}", left.ai_session(observer).input()),
        format!("{:?}", right.ai_session(observer).input())
    );
    assert_eq!(
        left.ai_session(observer).events(),
        right.ai_session(observer).events()
    );
    let originals = [left.save(), right.save()];
    for e in [&left, &right] {
        let snapshot = e.save();
        let decoded = engine_native::archive::decode_archive(
            &engine_native::archive::encode_archive(&snapshot),
        )
        .expect("archive roundtrip");
        assert_eq!(decoded, snapshot);
        Engine::restore_with_extensions(&decoded, packs).expect("legal initial snapshot");
    }
    let mut a = left.clone();
    let mut b = right.clone();
    redact_opponent_hands(&mut a, observer);
    redact_opponent_hands(&mut b, observer);
    Engine::restore_with_extensions(&a.save(), packs).expect("physical redacted snapshot");
    Engine::restore_with_extensions(&b.save(), packs).expect("physical alternate snapshot");
    assert_eq!(a.state_core(), b.state_core(), "unknown ownership leaked");
    assert_eq!(
        a.save().resolution,
        b.save().resolution,
        "continuation leaked"
    );
    assert_eq!(a.legal_actions(observer), b.legal_actions(observer));
    for budget in [0, 48] {
        let planner = BoundedPlanner::new(budget)
            .with_policy_variant(1)
            .with_lookahead(2);
        let p = left.plan_action(observer, &planner);
        let q = right.plan_action(observer, &planner);
        assert_eq!(format!("{p:?}"), format!("{q:?}"));
        assert_eq!(p.sim_faults, 0);
        for original in [&left, &right] {
            let mut restored = Engine::restore_with_extensions(&original.save(), packs).unwrap();
            assert!(!matches!(
                restored.apply_plan(&p).unwrap(),
                CommandOutcome::Rejected(_)
            ));
        }
    }
    assert_eq!([left.save(), right.save()], originals);
}

#[test]
fn private_vm_piles_are_physical_and_counterfactually_equal() {
    for seed in 9700..9712 {
        let mut a = Engine::new(seed, 4);
        a.debug_set_turn(0, Phase::Play, 1);
        for owner in [0, 1, 2] {
            let card = a.state_core_mut().deck.pop().unwrap();
            a.state_core_mut()
                .vm_aux
                .piles
                .insert((owner, "private".into()), vec![card]);
        }
        let mut b = a.clone();
        let core = b.state_core_mut();
        for (i, owner) in [1, 2].into_iter().enumerate() {
            std::mem::swap(
                &mut core
                    .vm_aux
                    .piles
                    .get_mut(&(owner, "private".into()))
                    .unwrap()[0],
                &mut core.deck[i],
            );
        }
        assert_pair(a, b, 0);
    }
}

#[test]
fn live_sha_window_hides_opponent_hand_identity() {
    for seed in 9800..9812 {
        let mut a = Engine::new(seed, 4);
        a.debug_set_turn(0, Phase::Play, 1);
        a.debug_move_named_card_to_hand(1, "shan");
        let card = a.debug_move_named_card_to_hand(0, "sha");
        submit(
            &mut a,
            0,
            GameAction::PlayCard {
                card,
                targets: vec![1],
            },
        );
        let observer = a.open_choice().expect("real response window").observer;
        let mut b = a.clone();
        let core = b.state_core_mut();
        std::mem::swap(&mut core.players[2].hand[0], &mut core.deck[0]);
        assert_pair(a, b, observer);
    }
}

#[test]
fn real_vm_commit_window_and_private_piles_are_counterfactually_equal() {
    use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
    let pkg = content_compiler::compile_source_named("nc10-commit", "skill nc10_commit(991041) { trigger card_targeted always { effect discard_hand_equip_count(event_player, 2); effect draw_cards(event_player, 1); } }").unwrap();
    let bridge = content_native_loader::bridge_vm_skill_decls(&pkg).unwrap();
    let mut pack = ExtensionPack {
        def: ExtensionDef {
            name: "nc10-commit",
            version: 1,
            description: "NC10 commit privacy",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: bridge.decls,
        init_hook: InitHook::None,
        enabled: true,
    };
    pack.vm_skills.truncate(1);
    for seed in 9900..9912 {
        let mut a = Engine::new_with_extensions(seed, 4, &[&pack]).unwrap();
        a.debug_set_turn(0, Phase::Play, 1);
        a.debug_grant(0, &pack.vm_skills[0].name);
        let card = a.debug_move_named_card_to_hand(0, "sha");
        submit(
            &mut a,
            0,
            GameAction::PlayCard {
                card,
                targets: vec![1],
            },
        );
        assert_eq!(a.open_choice().unwrap().spec.prompt, "rvm_discard_pick");
        assert!(a.save().resolution.parks.iter().any(|p| matches!(&p.machine, engine_native::MachineSnapshot::Rvm { paused: None, commit_pick: Some(_), commit_pending, .. } if !commit_pending.is_empty())));
        let card = a.state_core_mut().deck.pop().unwrap();
        a.state_core_mut()
            .vm_aux
            .piles
            .insert((2, "private".into()), vec![card]);
        let mut b = a.clone();
        let core = b.state_core_mut();
        std::mem::swap(&mut core.players[1].hand[0], &mut core.deck[0]);
        std::mem::swap(
            &mut core.vm_aux.piles.get_mut(&(2, "private".into())).unwrap()[0],
            &mut core.deck[1],
        );
        assert_pair_with_packs(a, b, 0, &[&pack]);
    }
}

#[test]
fn real_guanxing_machine_held_cards_are_not_duplicated() {
    for seed in 9950..9954 {
        let mut a = Engine::new(seed, 4);
        a.debug_grant(1, "观星");
        for _ in 0..100 {
            if a.open_choice()
                .is_some_and(|c| c.spec.prompt == "guanxing_arrange")
            {
                break;
            }
            if let Some(c) = a.open_choice().cloned() {
                let selection = if c
                    .spec
                    .options
                    .contains(&engine_native::ChoiceOption::Confirm)
                {
                    vec![engine_native::ChoiceOption::Confirm]
                } else {
                    c.spec.options.iter().take(c.spec.min).copied().collect()
                };
                submit(
                    &mut a,
                    c.observer,
                    GameAction::Answer {
                        choice_id: c.id,
                        selection,
                    },
                );
            } else {
                let actor = a.turn().current;
                submit(&mut a, actor, GameAction::NextPhase);
            }
        }
        assert_eq!(a.open_choice().unwrap().spec.prompt, "guanxing_arrange");
        let mut b = a.clone();
        let core = b.state_core_mut();
        std::mem::swap(&mut core.players[2].hand[0], &mut core.deck[0]);
        assert_pair(a, b, 1);
    }
}

#[test]
fn empty_zero_minimum_choice_has_a_legal_plan_even_without_cancellation() {
    let mut e = Engine::new(45000, 8);
    for seat in 0..8 {
        e.assign_general_named(seat, ["曹操", "刘备", "孙权", "赵云"][seat as usize % 4])
            .unwrap();
    }
    let planner = BoundedPlanner::new(8);
    for _ in 0..100 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.options.is_empty() && c.spec.min == 0 && !c.spec.allow_cancel {
                assert_eq!(c.spec.prompt, "hujia_respond");
                for budget in [0, 8] {
                    let p = e.plan_action(c.observer, &BoundedPlanner::new(budget));
                    assert!(
                        matches!(&p.action, GameAction::Answer { selection, .. } if selection.is_empty())
                    );
                    let mut restored = Engine::restore(&e.save()).unwrap();
                    assert!(!matches!(
                        restored.apply_plan(&p).unwrap(),
                        CommandOutcome::Rejected(_)
                    ));
                }
                return;
            }
        }
        let actor = e
            .open_choice()
            .map(|c| c.observer)
            .unwrap_or(e.turn().current);
        let p = e.plan_action(actor, &planner);
        assert!(!matches!(
            e.apply_plan(&p).unwrap(),
            CommandOutcome::Rejected(_)
        ));
    }
    panic!("expected the real zero-card hujia window");
}

#[test]
fn dead_current_actor_does_not_probe_attacks() {
    let mut e = Engine::new(45000, 4);
    e.debug_set_turn(0, Phase::Play, 1);
    e.debug_move_named_card_to_hand(0, "sha");
    e.state_core_mut().players[0].alive = false;
    e.state_core_mut().players[0].hp = 0;
    Engine::restore(&e.save()).expect("a dead current player can await phase advancement");
    let plan = e.plan_action(
        0,
        &BoundedPlanner::new(8)
            .with_policy_variant(1)
            .with_lookahead(2),
    );
    assert_eq!(plan.sim_faults, 0, "an observable dead actor cannot attack");
    assert_eq!(plan.action, GameAction::NextPhase);
    assert!(!matches!(
        e.apply_plan(&plan).unwrap(),
        CommandOutcome::Rejected(_)
    ));
}

#[test]
fn real_moyan_public_machine_held_cards_survive_both_windows() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json = std::fs::read_to_string(
        root.join("../../packages/extensions/d3-upstream-noname/assets/character.json"),
    )
    .unwrap();
    let pack = content_native_loader::bridge_pack(&json).unwrap().pack;
    for gain_window in [false, true] {
        let mut a = Engine::new_with_extensions(9970, 4, &[&pack]).unwrap();
        a.debug_grant(0, "dddmoyan");
        a.debug_set_turn(3, Phase::End, 1);
        submit(&mut a, 3, GameAction::NextPhase);
        assert_eq!(a.open_choice().unwrap().spec.prompt, "d3_moyan_target");
        if gain_window {
            let id = a.open_choice().unwrap().id;
            submit(
                &mut a,
                0,
                GameAction::Answer {
                    choice_id: id,
                    selection: vec![engine_native::ChoiceOption::Player(1)],
                },
            );
            assert_eq!(a.open_choice().unwrap().spec.prompt, "d3_moyan_gain");
        }
        let observer = a.open_choice().unwrap().observer;
        let shown = match &a.save().resolution.parks[0].machine {
            engine_native::MachineSnapshot::Host {
                behavior, cards, ..
            } if *behavior == engine_native::content::host::HostBehavior::D3Moyan.tag() => {
                cards.clone()
            }
            other => panic!("real public held cards: {other:?}"),
        };
        assert_eq!(shown.len(), 3);
        let mut b = a.clone();
        let core = b.state_core_mut();
        std::mem::swap(&mut core.players[2].hand[0], &mut core.deck[0]);
        assert_eq!(
            a.ai_session(observer).events(),
            b.ai_session(observer).events()
        );
        assert_pair_with_packs(a, b, observer, &[&pack]);
    }
}

#[test]
fn real_liaoyuan_suit_scalar_is_preserved_in_conservative_host_path() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json = std::fs::read_to_string(
        root.join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json"),
    )
    .unwrap();
    let pack = content_native_loader::bridge_pack(&json).unwrap().pack;
    let mut e = Engine::new_with_extensions(122001, 4, &[&pack]).unwrap();
    e.debug_grant(0, "liaoyuan");
    e.debug_set_turn(0, Phase::Play, 1);
    let sha = e.debug_move_named_card_to_hand(0, "sha");
    let suit = e.catalog().cards[sha as usize].suit;
    for _ in 0..3 {
        let index = e
            .state_core()
            .deck
            .iter()
            .position(|c| e.catalog().cards[*c as usize].suit == suit)
            .unwrap();
        let card = e.state_core_mut().deck.remove(index);
        e.state_core_mut().players[0].hand.push(card);
    }
    eprintln!(
        "LIAOYUAN_COMMAND actor=0 revision={} dedup={} PlayCard card={sha} targets=[1]",
        e.revision(),
        e.save().last_dedup + 1
    );
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    for _ in 0..16 {
        let c = e.open_choice().cloned().unwrap();
        if c.spec.prompt == "liaoyuan_discard" {
            break;
        }
        let selection = if c
            .spec
            .options
            .contains(&engine_native::ChoiceOption::Confirm)
        {
            vec![engine_native::ChoiceOption::Confirm]
        } else {
            c.spec.options.iter().take(c.spec.min).copied().collect()
        };
        eprintln!("LIAOYUAN_COMMAND actor={} revision={} dedup={} Answer choice={} selection={selection:?}", c.observer, e.revision(), e.save().last_dedup + 1, c.id);
        submit(
            &mut e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection,
            },
        );
    }
    assert_eq!(e.open_choice().unwrap().spec.prompt, "liaoyuan_discard");
    let before = e.save();
    let expected: Vec<_> = before
        .resolution
        .parks
        .iter()
        .filter_map(|p| engine_native::snap::active_ask(&p.machine, &before.core, e.catalog()))
        .collect();
    eprintln!("LIAOYUAN_EXPECTED {expected:#?}");
    eprintln!("LIAOYUAN_ACTUAL {:#?}", before.resolution.open_choice);
    eprintln!("LIAOYUAN_SNAPSHOT {before:#?}");
    eprintln!(
        "LIAOYUAN_ARCHIVE {:?}",
        engine_native::archive::encode_archive(&before)
    );
    Engine::restore_with_extensions(&before, &[&pack]).unwrap();
    let scalar = before
        .resolution
        .parks
        .iter()
        .find_map(|park| match &park.machine {
            engine_native::MachineSnapshot::Host {
                behavior,
                stage,
                card,
                ..
            } if *behavior == engine_native::content::host::HostBehavior::Liaoyuan.tag() => {
                assert_eq!(
                    *stage,
                    engine_native::content::host::HostStage::SelectCard.tag()
                );
                *card
            }
            _ => None,
        })
        .expect("real suit scalar");
    assert_eq!(scalar, suit as u32);
    redact_opponent_hands(&mut e, 0);
    assert_eq!(e.save().resolution, before.resolution);
    Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
}
