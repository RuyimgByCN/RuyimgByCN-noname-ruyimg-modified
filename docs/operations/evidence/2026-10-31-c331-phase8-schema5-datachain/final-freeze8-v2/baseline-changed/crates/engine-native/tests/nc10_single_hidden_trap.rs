//! NC10: an opponent's single hidden trap must not affect AI information or card ownership.
use engine_native::ai::{redact_opponent_hands, BoundedPlanner};
use engine_native::content::extension::ExtensionPack;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};

fn restore_archive(e: &Engine, pack: &ExtensionPack) -> Engine {
    let bytes = engine_native::archive::encode_archive(&e.save());
    let snapshot = engine_native::archive::decode_archive(&bytes).unwrap();
    Engine::restore_with_extensions(&snapshot, &[pack]).unwrap()
}

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
    let original_hash = engine.state_hash();
    let mut failures = Vec::new();
    for observer in [0, 1, 2, 3] {
        let mut sim = engine.clone();
        redact_opponent_hands(&mut sim, observer);
        assert_eq!(sim.state_core().player(1).sxianjing_traps.len(), 1);
        if observer == 1 {
            assert_eq!(
                sim.state_core().player(1).sxianjing_traps,
                engine.state_core().player(1).sxianjing_traps
            );
        }
        if let Err(error) = Engine::restore_with_extensions(&sim.save(), &[&pack]) {
            failures.push(format!("observer {observer}: {error:?}"));
        }
        assert_eq!(restore_archive(&sim, &pack).state_hash(), sim.state_hash());
    }
    assert!(failures.is_empty(), "redacted trap ownership: {failures:?}");
    assert_eq!(engine.state_hash(), original_hash);
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
    assert!(
        a.state_core() == b.state_core(),
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
            for source in [&left, &right] {
                let mut original = source.clone();
                let mut restored = restore_archive(source, &pack);
                let plan = source.plan_action(0, &planner);
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

#[test]
fn references_follow_their_synthetic_cards() {
    use engine_native::catalog::{CardKind, EquipSlot};
    let (left, pack) = fixture(3900);
    let mut pair = [left.clone(), alternate(left)];
    for engine in &mut pair {
        let core = engine.state_core_mut();
        let trap = core.players[1].sxianjing_traps[0];
        let hand = core.players[1].hand[0];
        let visible = core.players[0].hand[0];
        core.vm_aux.lost.insert(1, vec![trap, hand, visible]);
        core.turn.huanxia_track.push((hand, false));
        core.turn
            .kunlunjing_snapshot
            .push((1, vec![hand], vec![], vec![trap]));
        core.turn.luanjian_card = Some(hand);
        core.turn.qizuo_copy = Some(trap);
        core.players[1].ctianfu_card = Some(visible);
        core.players[1].jiguanyuan_cards.push(visible);
        core.players[1].yishan_lost.push(hand);
        core.players[1].card_mods.insert(hand, trap);
        core.declared_kinds.insert(trap, CardKind::Sha);
        core.declared_slots.insert(hand, EquipSlot::Weapon);
        core.card_natures.insert(trap, 1);
        assert!(Engine::restore_with_extensions(&engine.save(), &[&pack]).is_ok());
        redact_opponent_hands(engine, 0);
        let core = engine.state_core();
        let trap = core.players[1].sxianjing_traps[0];
        let hand = core.players[1].hand[0];
        assert_eq!(core.vm_aux.lost[&1], vec![trap, hand, visible]);
        assert_eq!(core.turn.huanxia_track, vec![(hand, false)]);
        assert_eq!(
            core.turn.kunlunjing_snapshot,
            vec![(1, vec![hand], vec![], vec![trap])]
        );
        assert_eq!(core.turn.luanjian_card, Some(hand));
        assert_eq!(core.turn.qizuo_copy, Some(trap));
        assert_eq!(core.players[1].ctianfu_card, Some(visible));
        assert_eq!(core.players[1].jiguanyuan_cards, vec![visible]);
        assert_eq!(core.players[1].yishan_lost, vec![hand]);
        assert_eq!(core.players[1].card_mods.get(&hand), Some(&trap));
        assert_eq!(core.declared_kinds.get(&trap), Some(&CardKind::Sha));
        assert_eq!(core.declared_slots.get(&hand), Some(&EquipSlot::Weapon));
        assert_eq!(core.card_natures.get(&trap), Some(&1));
        assert!(Engine::restore_with_extensions(&engine.save(), &[&pack]).is_ok());
        assert_eq!(
            restore_archive(engine, &pack).state_hash(),
            engine.state_hash()
        );
    }
    assert!(
        pair[0].state_core() == pair[1].state_core(),
        "equivalent hidden references diverged"
    );
}

#[test]
fn hidden_trap_sampling_is_invariant_across_seeds() {
    use engine_native::catalog::CardKind;
    let (_, pack) = fixture(3900);
    let mut checked = 0;
    for seed in 3900..4100 {
        let mut left = Engine::new_with_extensions(seed, 4, &[&pack]).unwrap();
        left.debug_set_turn(0, Phase::Play, 1);
        left.debug_grant(1, "sxianjing_gain");
        let Some(sha) = left
            .state_core()
            .player(0)
            .hand
            .iter()
            .copied()
            .find(|id| left.catalog().card_kind(*id) == CardKind::Sha)
        else {
            continue;
        };
        let suit = left.catalog().cards[sha as usize].suit;
        let Some(index) = left
            .state_core()
            .deck
            .iter()
            .position(|id| left.catalog().cards[*id as usize].suit == suit)
        else {
            continue;
        };
        let card = left.state_core_mut().deck.remove(index);
        left.state_core_mut().players[1].sxianjing_traps.push(card);
        let right = alternate(left.clone());
        assert_eq!(left.ai_session(0).view(), right.ai_session(0).view());
        let mut a = left.clone();
        let mut b = right.clone();
        redact_opponent_hands(&mut a, 0);
        redact_opponent_hands(&mut b, 0);
        assert!(
            a.state_core() == b.state_core(),
            "synthetic state depends on trap at seed {seed}"
        );
        assert!(Engine::restore_with_extensions(&a.save(), &[&pack]).is_ok());
        assert!(Engine::restore_with_extensions(&b.save(), &[&pack]).is_ok());
        for budget in [0, 48] {
            let planner = BoundedPlanner::new(budget)
                .with_policy_variant(1)
                .with_lookahead(2);
            let plan = left.plan_action(0, &planner);
            assert_eq!(
                plan.action,
                right.plan_action(0, &planner).action,
                "seed={seed}, budget={budget}"
            );
            if budget == 48 {
                for auxiliary in ["flag", "storage", "lost"] {
                    let mut extra = left.clone();
                    match auxiliary {
                        "flag" => {
                            extra
                                .state_core_mut()
                                .vm_aux
                                .flags
                                .insert((1, "nc10_inert".into()), 1);
                        }
                        "storage" => {
                            extra
                                .state_core_mut()
                                .vm_aux
                                .storage
                                .insert("nc10_inert".into(), 1);
                        }
                        "lost" => {
                            extra.state_core_mut().vm_aux.lost.insert(1, vec![card]);
                        }
                        _ => unreachable!(),
                    }
                    assert_eq!(left.ai_session(0).view(), extra.ai_session(0).view());
                    assert_eq!(
                        plan.action,
                        extra.plan_action(0, &planner).action,
                        "seed={seed}, auxiliary={auxiliary}"
                    );
                }
            }
            for source in [&left, &right] {
                let mut original = source.clone();
                let mut restored =
                    Engine::restore_with_extensions(&source.save(), &[&pack]).unwrap();
                assert!(
                    !matches!(
                        original.apply_plan(&plan).unwrap(),
                        CommandOutcome::Rejected(_)
                    ),
                    "seed={seed}, budget={budget}"
                );
                assert!(
                    !matches!(
                        restored.apply_plan(&plan).unwrap(),
                        CommandOutcome::Rejected(_)
                    ),
                    "seed={seed}, budget={budget}"
                );
                assert_eq!(original.state_hash(), restored.state_hash());
            }
        }
        checked += 1;
    }
    assert_eq!(checked, 96);
}

#[test]
fn vm_opaque_storage_keeps_its_trap_reference() {
    use engine_native::catalog::TriggerMeta;
    use engine_native::content::extension::{ExtensionDef, InitHook, VmSkillDecl};
    use engine_native::event::{EventKind, Timing};
    use engine_native::rvm_bridge::VmFilterPolicy;
    let (_, pack) = fixture(3900);
    let vm = ExtensionPack {
        def: ExtensionDef {
            name: "nc10-vm-probe",
            version: 1,
            description: "opaque storage reference",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        init_hook: InitHook::None,
        enabled: true,
        vm_skills: vec![VmSkillDecl {
            name: "nc10_vm_probe".into(),
            module: rule_vm::BytecodeModule {
                name: "nc10_vm_probe".into(),
                code: vec![rule_vm::bytecode::op::HALT],
                local_count: 0,
                max_stack: 1,
                entry_pc: 0,
                logic_hash: [5; 32],
                caps_mask: 0,
            },
            trigger: TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![],
            },
            filter: VmFilterPolicy::Always,
        }],
    };
    let mut e = Engine::new_with_extensions(3900, 4, &[&pack, &vm]).unwrap();
    e.debug_set_turn(0, Phase::Play, 1);
    let card = e.state_core_mut().deck.pop().unwrap();
    e.state_core_mut().players[1].sxianjing_traps.push(card);
    e.state_core_mut()
        .vm_aux
        .storage
        .insert("nc10_vm_probe#1".into(), card);
    assert!(Engine::restore_with_extensions(&e.save(), &[&pack, &vm]).is_ok());
    redact_opponent_hands(&mut e, 0);
    assert_eq!(
        e.state_core().vm_aux.storage["nc10_vm_probe#1"],
        e.state_core().players[1].sxianjing_traps[0]
    );
}
